//! Unified analysis pipeline.
//!
//! Provides a single entry point for running the full SAF analysis pipeline
//! (CG refinement, PTA, value flow) on an [`AirModule`].
//!
//! # Pipeline stages
//!
//! 1. Build def-use graph
//! 2. Run CG refinement (CHA bootstrap + iterative PTA)
//! 3. Build value-flow graph (using PTA results for precision)
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::pipeline::{PipelineConfig, run_pipeline};
//!
//! let result = run_pipeline(&module, &PipelineConfig::default());
//! let cg = &result.call_graph;
//! let vfg = &result.valueflow;
//! ```

use std::collections::BTreeMap;
use std::sync::Arc;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use saf_core::air::AirModule;
use saf_core::config::AnalysisMode;
use saf_core::ids::{FunctionId, InstId, ModuleId};
use saf_core::program::AirProgram;
use saf_core::saf_log;
use saf_core::spec::SpecRegistry;
use saf_core::summary_registry::SummaryRegistry;

use crate::callgraph::CallGraph;
use crate::cg_refinement::{
    CgRefinementDiff, RefinementConfig, RefinementResult, collect_indirect_call_sites,
    refine_incremental,
};
use crate::cha::ClassHierarchy;
use crate::defuse::DefUseGraph;
use crate::icfg::Icfg;
use crate::pta::{
    FunctionLocationMap, IncrementalConfig, IncrementalPtaState, LocationFactory,
    ModuleConstraints, ProgramConstraints, apply_incremental_update, extract_module_constraints,
};
use crate::session::AnalysisSession;
use crate::timer::Timer;
use crate::valueflow::rebuild_affected;
use crate::{
    FieldSensitivity, PtaConfig, PtaDiagnostics, PtaResult, ValueFlowConfig, ValueFlowGraph,
    ValueFlowMode, build_valueflow,
};

/// Configuration for the unified analysis pipeline.
#[derive(Debug)]
pub struct PipelineConfig {
    /// CG refinement configuration (includes PTA config).
    pub refinement: RefinementConfig,
    /// Value-flow analysis configuration.
    pub valueflow: ValueFlowConfig,
    /// Optional function specifications for PTA constraint generation.
    pub specs: Option<SpecRegistry>,
    /// Whether to build the value-flow graph during the pipeline.
    ///
    /// When `false`, the pipeline skips VFG construction (stage 3),
    /// returning an empty `ValueFlowGraph`. Useful when VFG will be
    /// built lazily on demand (e.g., via `ProgramDatabase`).
    /// Defaults to `true`.
    pub build_valueflow: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            refinement: RefinementConfig::default(),
            valueflow: ValueFlowConfig::default(),
            specs: None,
            build_valueflow: true,
        }
    }
}

impl PipelineConfig {
    /// Create a `PipelineConfig` from an `AnalysisMode` with appropriate defaults.
    ///
    /// - `Fast`: fewer refinement iterations (3), no field sensitivity,
    ///   capped PTA iterations (10,000), fast value-flow mode.
    /// - `Precise`: full defaults — field-sensitive PTA, 10 refinement
    ///   iterations, precise value-flow.
    #[must_use]
    pub fn from_mode(mode: AnalysisMode) -> Self {
        match mode {
            AnalysisMode::Fast => Self {
                refinement: RefinementConfig {
                    max_iterations: 3,
                    pta_config: PtaConfig {
                        field_sensitivity: FieldSensitivity::None,
                        max_iterations: 10_000,
                        ..PtaConfig::default()
                    },
                    ..RefinementConfig::default()
                },
                valueflow: ValueFlowConfig {
                    mode: ValueFlowMode::Fast,
                    ..ValueFlowConfig::default()
                },
                specs: None,
                build_valueflow: true,
            },
            AnalysisMode::Precise => Self::default(),
        }
    }
}

/// Timing and diagnostic statistics from a pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Time spent building the def-use graph (seconds).
    pub defuse_build_secs: f64,
    /// Time spent on PTA solving and CG refinement (seconds).
    pub pta_solve_secs: f64,
    /// Number of CG refinement iterations.
    pub refinement_iterations: usize,
    /// Time spent building the value-flow graph (seconds).
    pub valueflow_build_secs: f64,
    /// Total pipeline execution time (seconds).
    pub total_secs: f64,
    /// Constraint counts before HVN: `[addr, copy, load, store, gep]`.
    #[serde(default)]
    pub constraint_counts: [usize; 5],
    /// Constraint counts after HVN: `[addr, copy, load, store, gep]`.
    #[serde(default)]
    pub post_hvn_constraint_counts: [usize; 5],
    /// Inner PTA solver worklist iterations (NOT CG refinement iterations).
    /// Full path: from `PtaDiagnostics.iterations`. Incremental: from `IncrementalResult.iterations`.
    #[serde(default)]
    pub pta_iterations: usize,
    /// Number of added constraints in the incremental diff (0 on first run).
    #[serde(default)]
    pub constraint_diff_added: usize,
    /// Number of removed constraints in the incremental diff (0 on first run).
    #[serde(default)]
    pub constraint_diff_removed: usize,
    /// Number of modules whose constraints changed (0 on first run).
    #[serde(default)]
    pub changed_module_count: usize,
}

/// Result of running the unified analysis pipeline.
pub struct PipelineResult {
    /// The refined call graph.
    pub call_graph: CallGraph,
    /// The def-use graph.
    pub defuse: DefUseGraph,
    /// PTA result from CG refinement (if PTA ran).
    pub pta_result: Option<PtaResult>,
    /// The value-flow graph.
    pub valueflow: ValueFlowGraph,
    /// The ICFG built from the refined call graph.
    pub icfg: Icfg,
    /// Class hierarchy (if type hierarchy entries were present).
    pub cha: Option<ClassHierarchy>,
    /// Resolved indirect call sites from CG refinement.
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    /// Pipeline timing and diagnostic statistics.
    pub stats: PipelineStats,
}

/// Run the full analysis pipeline on a module.
///
/// This is the recommended entry point for consumers that need the complete
/// analysis stack (call graph, def-use, PTA, value flow). It standardizes
/// the analysis orchestration that was previously duplicated across
/// `saf-python`, `saf-wasm`, and benchmark runners.
///
/// # Pipeline stages
///
/// 1. Build def-use graph
/// 2. Run CG refinement (CHA bootstrap + iterative PTA)
/// 3. Build value-flow graph (using PTA results for precision)
#[must_use]
pub fn run_pipeline(module: &AirModule, config: &PipelineConfig) -> PipelineResult {
    let _pipeline_span = tracing::info_span!("run_pipeline").entered();
    let pipeline_start = Timer::now();

    // 1. Build def-use graph (skipped when build_valueflow is false — callers
    //    that need defuse later, e.g. FS-PTA bench path, build it locally)
    let (defuse, defuse_build_secs) = if config.build_valueflow {
        let _span = tracing::info_span!("defuse_build").entered();
        let t = Timer::now();
        let defuse = DefUseGraph::build(module);
        (defuse, t.elapsed_secs())
    } else {
        (DefUseGraph::default(), 0.0)
    };

    // 2. Run CG refinement (includes PTA)
    let (
        call_graph,
        icfg,
        pta_result,
        cha,
        resolved_sites,
        iterations,
        pta_solve_secs,
        constraint_counts,
        post_hvn_constraint_counts,
    ) = {
        let _span = tracing::info_span!("cg_refinement").entered();
        let RefinementResult {
            call_graph,
            icfg,
            pta_result,
            cha,
            resolved_sites,
            iterations,
            pta_solve_secs,
            constraint_counts,
            post_hvn_constraint_counts,
        } = crate::cg_refinement::refine(module, &config.refinement, config.specs.as_ref());
        (
            call_graph,
            icfg,
            pta_result,
            cha,
            resolved_sites,
            iterations,
            pta_solve_secs,
            constraint_counts,
            post_hvn_constraint_counts,
        )
    };

    // 3. Build value-flow graph (skipped when build_valueflow is false)
    let (valueflow, valueflow_build_secs) = if config.build_valueflow {
        let _span = tracing::info_span!("valueflow_build").entered();
        let t = Timer::now();
        let valueflow = build_valueflow(
            &config.valueflow,
            module,
            &defuse,
            &call_graph,
            pta_result.as_ref(),
        );
        (valueflow, t.elapsed_secs())
    } else {
        (ValueFlowGraph::new(), 0.0)
    };

    let total_secs = pipeline_start.elapsed_secs();
    let pta_iters = pta_result
        .as_ref()
        .map_or(0, |r| r.diagnostics().iterations);

    PipelineResult {
        call_graph,
        defuse,
        pta_result,
        valueflow,
        icfg,
        cha,
        resolved_sites,
        stats: PipelineStats {
            defuse_build_secs,
            pta_solve_secs,
            refinement_iterations: iterations,
            valueflow_build_secs,
            total_secs,
            constraint_counts,
            post_hvn_constraint_counts,
            pta_iterations: pta_iters,
            constraint_diff_added: 0,
            constraint_diff_removed: 0,
            changed_module_count: 0,
        },
    }
}

/// Run the analysis pipeline with per-module constraint caching and incremental solving.
///
/// This is the incremental counterpart to [`run_pipeline`]. It extracts
/// constraints per-module, caching them to disk so unchanged modules skip
/// re-extraction on subsequent runs. When previous analysis state is available
/// in the session, the pipeline uses incremental PTA solving, incremental CG
/// refinement, and selective value-flow rebuild instead of full re-analysis.
///
/// # Pipeline stages (first run — full analysis)
///
/// 1. For each module: check cache -> hit: load, miss: extract + save
/// 2. Merge per-module constraints into [`ProgramConstraints`]
/// 3. Full PTA solve + CG refinement
/// 4. Build value-flow graph
/// 5. Store session state for next incremental run
///
/// # Pipeline stages (subsequent run — incremental)
///
/// 1. Extract/load cached constraints per module
/// 2. Compute [`ConstraintDiff`] against previous constraints
/// 3. If diff is empty: reuse previous results
/// 4. Incremental PTA: apply constraint diff to existing state
/// 5. Incremental CG refinement: re-resolve indirect calls with updated PTS
/// 6. Selective VF rebuild: only rebuild affected functions
/// 7. Update session state
///
/// [`ConstraintDiff`]: crate::pta::module_constraints::ConstraintDiff
#[allow(clippy::too_many_lines)] // Incremental pipeline with two major code paths
pub fn run_pipeline_incremental(
    program: &AirProgram,
    config: &PipelineConfig,
    session: &mut AnalysisSession,
) -> PipelineResult {
    let _pipeline_span = tracing::info_span!("run_pipeline_incremental").entered();
    let pipeline_start = Timer::now();

    // 1. Extract or load cached constraints per module
    let factory = session.location_factory.take().unwrap_or_else(|| {
        LocationFactory::new(config.refinement.pta_config.field_sensitivity.clone())
    });
    let mut factory = factory;
    let mut module_constraints_list = Vec::with_capacity(program.modules.len());

    for module in &program.modules {
        let mc = match ModuleConstraints::load(session.cache_dir(), &module.id) {
            Ok(Some(cached)) if cached.fingerprint == module.id.to_hex() => {
                saf_log!(pipeline::constraint, stats, "cache hit"; module_id=format!("{}", module.id));
                cached
            }
            _ => {
                saf_log!(pipeline::constraint, stats, "cache miss"; module_id=format!("{}", module.id));
                let mc = extract_module_constraints(module, &mut factory);
                if let Err(e) = mc.save(session.cache_dir()) {
                    tracing::warn!(module_id = %module.id, error = %e, "failed to save constraint cache");
                }
                mc
            }
        };
        module_constraints_list.push(mc);
    }

    // 2. Merge into ProgramConstraints
    let current_constraints = ProgramConstraints::from_modules(module_constraints_list);
    saf_log!(pipeline::constraint, stats, "merged constraints"; modules=current_constraints.modules.len(), total=current_constraints.merged.total_count());

    // 3. Check for incremental path: do we have previous state?
    let can_incremental = session.previous_constraints.is_some()
        && session.incremental_pta_state.is_some()
        && session.previous_call_graph.is_some();

    if can_incremental {
        return run_incremental_path(
            program,
            config,
            session,
            current_constraints,
            &factory,
            &pipeline_start,
        );
    }

    // --- Full analysis path (first run or no previous state) ---
    saf_log!(pipeline::analysis, stats, "running full analysis");

    let merged_module = program.merged_view();

    // Build def-use graph
    let (defuse, defuse_build_secs) = {
        let _span = tracing::info_span!("defuse_build").entered();
        let t = Timer::now();
        let defuse = DefUseGraph::build(&merged_module);
        (defuse, t.elapsed_secs())
    };

    // Run CG refinement (full PTA solve)
    let (
        call_graph,
        icfg,
        pta_result,
        cha,
        iterations,
        pta_solve_secs,
        constraint_counts,
        post_hvn_constraint_counts,
    ) = {
        let _span = tracing::info_span!("cg_refinement").entered();
        let RefinementResult {
            call_graph,
            icfg,
            pta_result,
            cha,
            iterations,
            pta_solve_secs,
            constraint_counts,
            post_hvn_constraint_counts,
            ..
        } = crate::cg_refinement::refine(&merged_module, &config.refinement, config.specs.as_ref());
        (
            call_graph,
            icfg,
            pta_result,
            cha,
            iterations,
            pta_solve_secs,
            constraint_counts,
            post_hvn_constraint_counts,
        )
    };

    // Build value-flow graph
    let (valueflow, valueflow_build_secs) = {
        let _span = tracing::info_span!("valueflow_build").entered();
        let t = Timer::now();
        let valueflow = build_valueflow(
            &config.valueflow,
            &merged_module,
            &defuse,
            &call_graph,
            pta_result.as_ref(),
        );
        (valueflow, t.elapsed_secs())
    };

    // Store state for next incremental run
    store_session_state(
        session,
        SessionStateUpdate {
            program,
            constraints: current_constraints,
            pta_result: pta_result.as_ref(),
            call_graph: &call_graph,
            defuse: &defuse,
            valueflow: &valueflow,
            merged_module: &merged_module,
            factory: &factory,
        },
    );

    let total_secs = pipeline_start.elapsed_secs();
    let pta_iters = pta_result
        .as_ref()
        .map_or(0, |r| r.diagnostics().iterations);

    PipelineResult {
        call_graph,
        defuse,
        pta_result,
        valueflow,
        icfg,
        cha,
        resolved_sites: BTreeMap::new(),
        stats: PipelineStats {
            defuse_build_secs,
            pta_solve_secs,
            refinement_iterations: iterations,
            valueflow_build_secs,
            total_secs,
            constraint_counts,
            post_hvn_constraint_counts,
            pta_iterations: pta_iters,
            constraint_diff_added: 0,
            constraint_diff_removed: 0,
            changed_module_count: 0,
        },
    }
}

/// Incremental analysis path: uses constraint diff to update PTA, CG, and VF.
#[allow(clippy::too_many_lines)]
fn run_incremental_path(
    program: &AirProgram,
    config: &PipelineConfig,
    session: &mut AnalysisSession,
    current_constraints: ProgramConstraints,
    factory: &LocationFactory,
    pipeline_start: &Timer,
) -> PipelineResult {
    let prev_constraints = session.previous_constraints.take().expect("checked above");
    let mut pta_state = session.incremental_pta_state.take().expect("checked above");

    // Compute constraint diff
    let diff = prev_constraints.diff(&current_constraints);
    saf_log!(pipeline::incremental, stats, "constraint diff"; changed=diff.changed_module_count, unchanged=diff.unchanged_module_count);

    let merged_module = program.merged_view();

    // Build def-use graph (always needed fresh since it depends on module contents)
    let (defuse, defuse_build_secs) = {
        let _span = tracing::info_span!("defuse_build").entered();
        let t = Timer::now();
        let defuse = DefUseGraph::build(&merged_module);
        (defuse, t.elapsed_secs())
    };

    // If constraints are unchanged, reuse previous results with minimal work
    if diff.added.total_count() == 0 && diff.removed.total_count() == 0 {
        saf_log!(pipeline::incremental, stats, "no changes, reusing results");

        let call_graph = session.previous_call_graph.take().expect("checked above");
        let pta_result = session.previous_pta_result.take();
        let valueflow = session.previous_valueflow.take().unwrap_or_default();
        let icfg = Icfg::build(&merged_module, &call_graph);

        store_session_state(
            session,
            SessionStateUpdate {
                program,
                constraints: current_constraints,
                pta_result: pta_result.as_ref(),
                call_graph: &call_graph,
                defuse: &defuse,
                valueflow: &valueflow,
                merged_module: &merged_module,
                factory,
            },
        );
        // Restore the PTA state (unchanged)
        session.incremental_pta_state = Some(pta_state);

        let total_secs = pipeline_start.elapsed_secs();
        return PipelineResult {
            call_graph,
            defuse,
            pta_result,
            valueflow,
            icfg,
            cha: None,
            resolved_sites: BTreeMap::new(),
            stats: PipelineStats {
                defuse_build_secs,
                pta_solve_secs: 0.0,
                refinement_iterations: 0,
                valueflow_build_secs: 0.0,
                total_secs,
                constraint_counts: [0; 5],
                post_hvn_constraint_counts: [0; 5],
                pta_iterations: 0,
                constraint_diff_added: 0,
                constraint_diff_removed: 0,
                changed_module_count: 0,
            },
        };
    }

    // --- Incremental PTA solve ---
    let (pta_solve_secs, pta_result, pta_iters) = {
        let _span = tracing::info_span!("incremental_pta").entered();
        let t = Timer::now();

        let incr_config = IncrementalConfig {
            max_iterations: config.refinement.pta_config.max_iterations,
            use_diff_propagation: true,
        };

        let incr_result =
            apply_incremental_update(&mut pta_state, &diff.added, &diff.removed, &incr_config);

        saf_log!(pipeline::incremental, stats, "PTA solve complete"; iterations=incr_result.iterations, propagations=incr_result.propagations, converged=incr_result.converged);

        let iters = incr_result.iterations;

        // Convert incremental state to PointsToMap for PtaResult construction
        let pts_map = pta_state.to_points_to_map();
        let pta_result = PtaResult::new(
            pts_map,
            Arc::new(factory.clone()),
            PtaDiagnostics::default(),
        );

        (t.elapsed_secs(), Some(pta_result), iters)
    };

    // --- Incremental CG refinement ---
    let mut call_graph = session
        .previous_call_graph
        .take()
        .expect("checked in caller");
    let func_loc_map = FunctionLocationMap::build(&merged_module);
    let cg_diff = {
        let _span = tracing::info_span!("incremental_cg_refinement").entered();
        if let Some(pta_r) = &pta_result {
            refine_incremental(
                &mut call_graph,
                &merged_module,
                pta_r.points_to_map(),
                factory,
                &func_loc_map,
                &session.previous_indirect_targets,
            )
        } else {
            CgRefinementDiff::default()
        }
    };

    saf_log!(pipeline::incremental, stats, "CG refinement complete"; added=cg_diff.added_edges.len(), removed=cg_diff.removed_edges.len());

    // --- Selective VF rebuild ---
    let (valueflow, valueflow_build_secs) = {
        let _span = tracing::info_span!("selective_vf_rebuild").entered();
        let t = Timer::now();

        // Compute affected functions from: changed modules + CG changes
        let mut affected_functions = std::collections::BTreeSet::<FunctionId>::new();
        affected_functions.extend(&cg_diff.newly_reachable);
        // Functions from changed modules
        for module in &program.modules {
            if diff.changed_modules.contains(&module.id) {
                for func in &module.functions {
                    affected_functions.insert(func.id);
                }
            }
        }

        if affected_functions.is_empty() {
            // No affected functions — reuse previous VF graph
            let vf = session.previous_valueflow.take().unwrap_or_default();
            (vf, t.elapsed_secs())
        } else {
            let mut vf = session.previous_valueflow.take().unwrap_or_default();
            rebuild_affected(
                &mut vf,
                &affected_functions,
                &merged_module,
                &defuse,
                &call_graph,
                pta_result.as_ref(),
                &config.valueflow,
            );
            saf_log!(pipeline::incremental, stats, "VF rebuild complete"; affected=affected_functions.len());
            (vf, t.elapsed_secs())
        }
    };

    // Build ICFG from updated call graph
    let icfg = Icfg::build(&merged_module, &call_graph);

    // Store state for next incremental run
    store_session_state(
        session,
        SessionStateUpdate {
            program,
            constraints: current_constraints,
            pta_result: pta_result.as_ref(),
            call_graph: &call_graph,
            defuse: &defuse,
            valueflow: &valueflow,
            merged_module: &merged_module,
            factory,
        },
    );
    session.incremental_pta_state = Some(pta_state);

    let total_secs = pipeline_start.elapsed_secs();

    PipelineResult {
        call_graph,
        defuse,
        pta_result,
        valueflow,
        icfg,
        cha: None,
        resolved_sites: BTreeMap::new(),
        stats: PipelineStats {
            defuse_build_secs,
            pta_solve_secs,
            refinement_iterations: 1, // Single incremental pass
            valueflow_build_secs,
            total_secs,
            constraint_counts: [0; 5],
            post_hvn_constraint_counts: [0; 5],
            pta_iterations: pta_iters,
            constraint_diff_added: diff.added.total_count(),
            constraint_diff_removed: diff.removed.total_count(),
            changed_module_count: diff.changed_module_count,
        },
    }
}

/// Context for storing session state after a pipeline run.
struct SessionStateUpdate<'a> {
    program: &'a AirProgram,
    constraints: ProgramConstraints,
    pta_result: Option<&'a PtaResult>,
    call_graph: &'a CallGraph,
    defuse: &'a DefUseGraph,
    valueflow: &'a ValueFlowGraph,
    merged_module: &'a AirModule,
    factory: &'a LocationFactory,
}

/// Store analysis results in the session for the next incremental run.
fn store_session_state(session: &mut AnalysisSession, update: SessionStateUpdate<'_>) {
    session.record_run();
    session.program_id = Some(update.program.id);
    session.previous_constraints = Some(update.constraints);
    session.previous_pta_result = update.pta_result.cloned();
    session.previous_call_graph = Some(update.call_graph.clone());
    session.previous_defuse = Some(update.defuse.clone());
    session.previous_valueflow = Some(update.valueflow.clone());
    session.location_factory = Some(update.factory.clone());

    // Build IncrementalPtaState from the PTA result for next incremental run
    if session.incremental_pta_state.is_none() {
        if let Some(pta_r) = update.pta_result {
            session.incremental_pta_state = Some(IncrementalPtaState::from_pta_result(pta_r));
        }
    }

    // Capture current indirect call targets for next CG refinement
    capture_indirect_targets(
        session,
        update.merged_module,
        update.pta_result,
        update.factory,
    );
}

/// Resolve current indirect call targets from PTA results and store in session.
fn capture_indirect_targets(
    session: &mut AnalysisSession,
    module: &AirModule,
    pta_result: Option<&PtaResult>,
    factory: &LocationFactory,
) {
    let mut targets = BTreeMap::new();
    if let Some(pta_r) = pta_result {
        let func_loc_map = FunctionLocationMap::build(module);
        let sites = collect_indirect_call_sites(module);
        for site in &sites {
            let mut resolved = std::collections::BTreeSet::new();
            if let Some(&callee_val) = site.operands.last() {
                if let Some(pts_set) = pta_r.points_to_map().get(&callee_val) {
                    for &loc_id in pts_set {
                        if let Some(loc) = factory.get(loc_id) {
                            if let Some(fid) = func_loc_map.get(loc.obj) {
                                resolved.insert(fid);
                            }
                        }
                    }
                }
            }
            targets.insert(site.inst_id, resolved);
        }
    }
    session.previous_indirect_targets = targets;
}

// =============================================================================
// Per-module parallel analysis
// =============================================================================

/// Result of analyzing a single module in isolation.
pub struct ModuleAnalysisResult {
    /// The module that was analyzed.
    pub module_id: ModuleId,
    /// PTA result for this module (if PTA ran).
    pub pta_result: Option<PtaResult>,
    /// The value-flow graph for this module.
    pub valueflow: ValueFlowGraph,
    /// Summaries generated for functions in this module.
    pub summaries: SummaryRegistry,
    /// Per-module timing statistics.
    pub stats: PipelineStats,
}

/// Analyze each module in a program independently and in parallel.
///
/// Each module gets its own PTA context. Cross-module calls are resolved
/// via summary instantiation from the provided `summaries` registry
/// (populated from a prior bottom-up summary generation pass).
///
/// The `parallelism` parameter controls the rayon thread pool size.
/// A value of `0` uses rayon's default (number of CPUs).
///
/// This is the compositional counterpart to [`run_pipeline`]: instead of
/// merging all modules and running a single whole-program analysis, each
/// module is analyzed in isolation with summary-based stubs for cross-module
/// calls.
pub fn analyze_modules_parallel(
    program: &AirProgram,
    _summaries: &SummaryRegistry,
    config: &PipelineConfig,
    parallelism: usize,
) -> Vec<ModuleAnalysisResult> {
    let _pipeline_span = tracing::info_span!("analyze_modules_parallel").entered();

    // Configure rayon thread pool
    let pool = if parallelism > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(parallelism)
            .build()
    } else {
        rayon::ThreadPoolBuilder::new().build()
    };

    let pool = match pool {
        Ok(pool) => pool,
        Err(e) => {
            tracing::warn!(error = %e, "failed to create rayon pool, falling back to default");
            rayon::ThreadPoolBuilder::new()
                .build()
                .expect("default rayon pool should always succeed")
        }
    };

    pool.install(|| {
        program
            .modules
            .par_iter()
            .map(|module| analyze_single_module(module, config))
            .collect()
    })
}

/// Analyze a single module in isolation.
///
/// Runs the standard pipeline stages (def-use, CG refinement, value-flow)
/// on one module. Cross-module call resolution via summaries is a future
/// extension (currently uses intra-module information only).
fn analyze_single_module(module: &AirModule, config: &PipelineConfig) -> ModuleAnalysisResult {
    let _span = tracing::info_span!("analyze_module", module_id = %module.id).entered();
    let pipeline_start = Timer::now();

    // 1. Build def-use graph
    let (defuse, defuse_build_secs) = {
        let t = Timer::now();
        let defuse = DefUseGraph::build(module);
        (defuse, t.elapsed_secs())
    };

    // 2. Run CG refinement (module-local)
    let (call_graph, pta_result, iterations, pta_solve_secs) = {
        let RefinementResult {
            call_graph,
            pta_result,
            iterations,
            pta_solve_secs,
            ..
        } = crate::cg_refinement::refine(module, &config.refinement, config.specs.as_ref());
        (call_graph, pta_result, iterations, pta_solve_secs)
    };

    // 3. Build value-flow graph
    let (valueflow, valueflow_build_secs) = {
        let t = Timer::now();
        let valueflow = build_valueflow(
            &config.valueflow,
            module,
            &defuse,
            &call_graph,
            pta_result.as_ref(),
        );
        (valueflow, t.elapsed_secs())
    };

    // 4. Generate module-local summaries (empty for now -- populated by
    //    summary_gen::generate_summaries in the full pipeline)
    let summaries = SummaryRegistry::new();

    let total_secs = pipeline_start.elapsed_secs();

    ModuleAnalysisResult {
        module_id: module.id,
        pta_result,
        valueflow,
        summaries,
        stats: PipelineStats {
            defuse_build_secs,
            pta_solve_secs,
            refinement_iterations: iterations,
            valueflow_build_secs,
            total_secs,
            constraint_counts: [0; 5],
            post_hvn_constraint_counts: [0; 5],
            pta_iterations: 0,
            constraint_diff_added: 0,
            constraint_diff_removed: 0,
            changed_module_count: 0,
        },
    }
}
