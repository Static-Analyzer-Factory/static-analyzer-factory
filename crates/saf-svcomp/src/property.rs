//! SV-COMP property analyzers.
//!
//! Maps each SV-COMP property type to the optimal combination of SAF analyses.
//! Each analyzer returns a `PropertyResult` indicating whether the property holds,
//! is violated, or cannot be determined.
//!
//! # Supported Properties
//!
//! - `unreach-call`: Uses Z3 path reachability to prove `reach_error()` unreachable
//! - `valid-memsafety`: Uses SVFG checkers for null-deref, UAF, double-free
//! - `no-overflow`: Uses interprocedural abstract interpretation for signed overflow
//! - `no-data-race`: Uses MTA MHP analysis for concurrent access detection

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use saf_analysis::absint::{
    AbstractInterpConfig, NumericCheckResult, NumericSeverity, check_integer_overflow_with_specs,
};
use saf_analysis::callgraph::CallGraph;
use saf_analysis::cfg::Cfg;
use saf_analysis::checkers::finding::NullSourceKind;
use saf_analysis::checkers::{
    GuardContext, GuardedSolverConfig, PathSensitiveConfig, PathSensitiveResult, ResourceTable,
    SolverConfig, run_checkers_path_sensitive,
};
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::mta::{AccessKind, MtaAnalysis, MtaConfig, MtaResult, ThreadId};
use saf_analysis::svfg::{Svfg, SvfgBuilder, SvfgNodeId};
use saf_analysis::z3_utils::reachability::{PathReachability, check_path_reachable};
use saf_analysis::{AliasResult, PtaConfig, PtaContext, PtaResult};
use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use saf_core::spec::{AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec};
use serde::{Deserialize, Serialize};

use super::fast_paths::{
    build_value_function_map, find_allocation_sites, find_deallocation_sites,
    find_nonterminating_loops, function_is_loop_free, has_heap_allocations,
    has_threading_primitives, program_is_loop_free, reachable_functions,
    reachable_has_heap_allocations, reachable_is_loop_free,
};
use super::summaries::{ErrorSummary, compute_error_summaries};
use crate::property_kind::Property;

/// Result of analyzing a single property.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result")]
pub enum PropertyResult {
    /// Property holds (TRUE verdict).
    True,

    /// Property is violated (FALSE verdict).
    False {
        /// Optional witness trace (block path or source locations).
        witness: Option<Vec<String>>,
    },

    /// Property cannot be determined (UNKNOWN verdict).
    Unknown {
        /// Reason for unknown result.
        reason: String,
    },
}

impl PropertyResult {
    /// Returns true if this is a TRUE verdict.
    pub fn is_true(&self) -> bool {
        matches!(self, Self::True)
    }

    /// Returns true if this is a FALSE verdict.
    pub fn is_false(&self) -> bool {
        matches!(self, Self::False { .. })
    }

    /// Returns true if this is UNKNOWN.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown { .. })
    }
}

/// Configuration for property analysis.
#[derive(Debug, Clone)]
pub struct PropertyAnalysisConfig {
    /// Z3 timeout in milliseconds per query.
    pub z3_timeout_ms: u64,

    /// Maximum number of path guards to track.
    pub max_guards: usize,

    /// Maximum number of paths to enumerate.
    pub max_paths: usize,

    /// Enable context-sensitive PTA.
    pub context_sensitive: bool,

    /// k-CFA depth for context sensitivity.
    pub k_cfa_depth: usize,

    /// Conservative mode (default: true).
    ///
    /// When true, only return TRUE/FALSE when we have high confidence.
    /// When false (aggressive mode), return verdicts with lower confidence
    /// for higher coverage but small risk of incorrect results.
    pub conservative: bool,
}

impl Default for PropertyAnalysisConfig {
    fn default() -> Self {
        Self {
            z3_timeout_ms: 5000,
            max_guards: 50,
            max_paths: 1000,
            context_sensitive: true,
            k_cfa_depth: 2,
            conservative: true,
        }
    }
}

// ---------------------------------------------------------------------------
// AnalysisContext: Lazy caching for expensive analysis results
// ---------------------------------------------------------------------------

/// Lazily builds and caches analysis results as needed by property analyzers.
///
/// This avoids redundant computation when multiple property checks need the
/// same underlying analysis (e.g., both `valid-memsafety` and `no-data-race`
/// need PTA results).
pub struct AnalysisContext<'a> {
    module: &'a AirModule,
    config: &'a PropertyAnalysisConfig,
    callgraph: OnceCell<CallGraph>,
    icfg: OnceCell<Icfg>,
    defuse: OnceCell<DefUseGraph>,
    cfgs: OnceCell<BTreeMap<FunctionId, Cfg>>,
    pta_result: OnceCell<PtaResult>,
    svfg: OnceCell<Svfg>,
    absint_result: OnceCell<NumericCheckResult>,
    mta_result: OnceCell<MtaResult>,
    reachable: OnceCell<BTreeSet<FunctionId>>,
    value_func_map: OnceCell<BTreeMap<ValueId, FunctionId>>,
}

impl<'a> AnalysisContext<'a> {
    /// Create a new analysis context for the given module.
    pub fn new(module: &'a AirModule, config: &'a PropertyAnalysisConfig) -> Self {
        Self {
            module,
            config,
            callgraph: OnceCell::new(),
            icfg: OnceCell::new(),
            defuse: OnceCell::new(),
            cfgs: OnceCell::new(),
            pta_result: OnceCell::new(),
            svfg: OnceCell::new(),
            absint_result: OnceCell::new(),
            mta_result: OnceCell::new(),
            reachable: OnceCell::new(),
            value_func_map: OnceCell::new(),
        }
    }

    /// Get the call graph (builds if not cached).
    pub fn callgraph(&self) -> &CallGraph {
        self.callgraph.get_or_init(|| CallGraph::build(self.module))
    }

    /// Get the ICFG (builds if not cached).
    pub fn icfg(&self) -> &Icfg {
        self.icfg
            .get_or_init(|| Icfg::build(self.module, self.callgraph()))
    }

    /// Get the def-use graph (builds if not cached).
    pub fn defuse(&self) -> &DefUseGraph {
        self.defuse.get_or_init(|| DefUseGraph::build(self.module))
    }

    /// Get CFGs for all functions (builds if not cached).
    pub fn cfgs(&self) -> &BTreeMap<FunctionId, Cfg> {
        self.cfgs.get_or_init(|| {
            self.module
                .functions
                .iter()
                .filter(|f| !f.is_declaration)
                .map(|f| (f.id, Cfg::build(f)))
                .collect()
        })
    }

    /// Get PTA result (runs if not cached).
    pub fn pta(&self) -> &PtaResult {
        self.pta_result.get_or_init(|| {
            let pta_config = PtaConfig::default();
            let mut ctx = PtaContext::new(pta_config);
            let raw = ctx.analyze(self.module);
            PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics)
        })
    }

    /// Get SVFG and program point map (builds if not cached). Requires MSSA which is built internally.
    ///
    /// Note: SVFG construction requires mutable MSSA, so we build it fresh here.
    /// This is acceptable since SVFG is typically only needed once per analysis.
    ///
    /// Returns both the SVFG and the `ProgramPointMap` which tracks where each
    /// value is defined. The program point map is used for temporal ordering
    /// queries in UAF filtering.
    pub fn build_svfg_with_program_points(&self) -> (Svfg, saf_analysis::svfg::ProgramPointMap) {
        // Build MSSA (requires its own PTA)
        let pta_config = PtaConfig::default();
        let mut ctx = PtaContext::new(pta_config);
        let raw = ctx.analyze(self.module);
        let mssa_pta = PtaResult::new(raw.pts, Arc::new(raw.factory), raw.diagnostics);
        let mut mssa = MemorySsa::build(
            self.module,
            self.cfgs(),
            Arc::new(mssa_pta),
            self.callgraph(),
        );

        // Build SVFG with program points
        SvfgBuilder::new(
            self.module,
            self.defuse(),
            self.callgraph(),
            self.pta(),
            &mut mssa,
        )
        .build()
    }

    /// Get SVFG only (builds if not cached).
    ///
    /// This is a convenience wrapper around `build_svfg_with_program_points()`
    /// that discards the program point map.
    pub fn build_svfg(&self) -> Svfg {
        let (svfg, _program_points) = self.build_svfg_with_program_points();
        svfg
    }

    /// Get cached SVFG (builds if not cached).
    pub fn svfg(&self) -> &Svfg {
        self.svfg.get_or_init(|| self.build_svfg())
    }

    /// Get abstract interpretation overflow results (runs if not cached).
    ///
    /// Uses spec-aware abstract interpretation so that external function calls
    /// (e.g., `atoi`, `rand`) return bounded intervals instead of TOP, enabling
    /// the integer overflow checker to detect overflows involving external inputs.
    pub fn absint(&self) -> &NumericCheckResult {
        self.absint_result.get_or_init(|| {
            let ai_config = AbstractInterpConfig::default();
            let analyzed_specs = build_analyzed_specs();
            check_integer_overflow_with_specs(self.module, &ai_config, Some(&analyzed_specs))
        })
    }

    /// Get MTA results (runs if not cached).
    pub fn mta(&self) -> &MtaResult {
        self.mta_result.get_or_init(|| {
            let mta_config = MtaConfig::default();
            let mta = MtaAnalysis::new(self.module, self.callgraph(), self.icfg(), mta_config);
            mta.analyze()
        })
    }

    /// Get the set of functions reachable from main (computes if not cached).
    pub fn reachable(&self) -> &BTreeSet<FunctionId> {
        self.reachable
            .get_or_init(|| reachable_functions(self.callgraph(), self.module))
    }

    /// Get the value-to-function map (computes if not cached).
    pub fn value_function_map(&self) -> &BTreeMap<ValueId, FunctionId> {
        self.value_func_map
            .get_or_init(|| build_value_function_map(self.module))
    }
}

/// Build a [`SpecRegistry`] with return interval specs for common C library functions.
///
/// These specs give external functions non-TOP return intervals, enabling the
/// integer overflow checker to reason about expressions involving external inputs.
/// Without these specs, calls like `atoi(buf)` return TOP and the checker skips
/// any arithmetic involving them.
///
/// Note: `[INT_MIN, INT_MAX]` is different from TOP — it tells the checker the
/// value is bounded (even if the full range), so `val + 1` can be detected as
/// potentially overflowing.
fn build_c_library_specs() -> saf_core::spec::SpecRegistry {
    use saf_core::spec::{FunctionSpec, ReturnSpec, SpecRegistry};

    let mut registry = SpecRegistry::new();

    // Helper to add a spec with a return interval
    let mut add_interval = |name: &str, lo: i64, hi: i64| {
        let mut spec = FunctionSpec::new(name);
        spec.returns = Some(ReturnSpec {
            interval: Some((lo, hi)),
            ..ReturnSpec::default()
        });
        // Ignore duplicate errors (shouldn't happen with unique names)
        let _ = registry.add(spec);
    };

    // Integer parsing functions — return full i32 range
    let i32_min: i64 = -2_147_483_648;
    let i32_max: i64 = 2_147_483_647;

    add_interval("atoi", i32_min, i32_max);
    add_interval("strtol", i32_min, i32_max);
    add_interval("strtoul", 0, i32_max); // returns unsigned but treated as i32 in LLVM IR

    // Random / nondeterministic — POSIX rand() returns [0, RAND_MAX]
    add_interval("rand", 0, i32_max);

    // Absolute value — always non-negative (ignoring INT_MIN edge case for abs)
    add_interval("abs", 0, i32_max);
    add_interval("labs", 0, i32_max);

    // Character input — returns [-1, 255] (EOF or unsigned char)
    add_interval("getchar", -1, 255);
    add_interval("fgetc", -1, 255);
    add_interval("getc", -1, 255);

    // SV-COMP / Juliet nondeterministic input functions — full i32 range
    add_interval("__VERIFIER_nondet_int", i32_min, i32_max);
    add_interval("__VERIFIER_nondet_uint", 0, i32_max);
    add_interval("__VERIFIER_nondet_short", -32768, 32767);
    add_interval("__VERIFIER_nondet_char", -128, 127);

    // Network functions that return status codes
    add_interval("recv", -1, i32_max);
    add_interval("connect", -1, 0);
    add_interval("listen", -1, 0);
    add_interval("accept", -1, i32_max);
    add_interval("socket", -1, i32_max);

    // I/O functions
    add_interval("fscanf", -1, i32_max); // returns number of matched items or EOF
    add_interval("scanf", -1, i32_max);

    registry
}

/// Build an `AnalyzedSpecRegistry` with YAML specs and computed return bounds.
///
/// Wraps `build_c_library_specs()` and adds derived specs for functions
/// whose return value is bounded by an argument's allocation size.
fn build_analyzed_specs() -> AnalyzedSpecRegistry {
    let yaml_specs = build_c_library_specs();
    let mut analyzed = AnalyzedSpecRegistry::new(yaml_specs);

    // Register computed return bounds for strlen-like functions.
    // Return ∈ [0, alloc_size(arg0) - 1].
    //
    // NOTE: wcslen returns wide-character count, not byte count.
    // AllocSizeMinusOne gives byte-level bounds which over-approximates
    // (e.g., [0, 43] for wchar_t buf[11] instead of [0, 10]).
    // Deferred: add AllocElementCountMinusOne mode that divides by sizeof(wchar_t).
    for name in ["strlen", "strnlen", "wcslen", "ldv_strlen"] {
        analyzed.add_derived(
            name,
            DerivedSpec {
                computed_return_bound: Some(ComputedBound {
                    param_index: 0,
                    mode: BoundMode::AllocSizeMinusOne,
                }),
                ..DerivedSpec::empty()
            },
        );
    }

    analyzed
}

/// Analyze a property for a given module.
pub fn analyze_property(
    property: &Property,
    module: &AirModule,
    config: &PropertyAnalysisConfig,
) -> PropertyResult {
    let ctx = AnalysisContext::new(module, config);
    analyze_property_with_context(property, &ctx)
}

/// Analyze a property using a shared analysis context.
///
/// This allows multiple properties to share cached analysis results.
pub fn analyze_property_with_context(
    property: &Property,
    ctx: &AnalysisContext<'_>,
) -> PropertyResult {
    match property {
        Property::UnreachCall => analyze_unreachability(ctx),
        Property::ValidMemsafety => analyze_memsafety(ctx),
        Property::ValidMemcleanup => analyze_memcleanup(ctx),
        Property::NoOverflow => analyze_no_overflow(ctx),
        Property::NoDataRace => analyze_no_data_race(ctx),
        Property::Termination => analyze_termination(ctx),
        Property::Coverage | Property::Unknown => PropertyResult::Unknown {
            reason: format!("Property {} not supported", property.name()),
        },
    }
}

/// Find calls to a specific function by name.
fn find_calls_to(module: &AirModule, func_name: &str) -> Vec<(FunctionId, BlockId, InstId)> {
    let mut results = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if let Some(target) = module.function(*callee) {
                        if target.name == func_name {
                            results.push((func.id, block.id, inst.id));
                        }
                    }
                }
            }
        }
    }

    results
}

/// Get the entry block of a function.
fn get_entry_block(module: &AirModule, func_id: FunctionId) -> Option<BlockId> {
    module
        .function(func_id)
        .and_then(|f| f.entry_block.or_else(|| f.blocks.first().map(|b| b.id)))
}

// ---------------------------------------------------------------------------
// Property Analyzers
// ---------------------------------------------------------------------------

/// Analyze `unreach-call` property (`reach_error()` or `__VERIFIER_error()`).
///
/// Strategy:
/// 1. Fast-path: if no error calls exist → TRUE
/// 2. Check call graph reachability from main
/// 3. For error sites in main: use Z3 path feasibility
/// 4. P2: For error sites in callees: use function summaries
/// 5. If all paths UNSAT → TRUE, else FALSE with witness
// NOTE: This function implements the full unreachability analysis pipeline
// as a cohesive unit. Splitting would fragment the analysis logic.
#[allow(clippy::too_many_lines)]
fn analyze_unreachability(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let module = ctx.module;
    let config = ctx.config;

    // SV-COMP uses either reach_error() or __VERIFIER_error()
    let error_names = ["reach_error", "__VERIFIER_error"];

    let mut error_calls = Vec::new();
    for name in &error_names {
        error_calls.extend(find_calls_to(module, name));
    }

    if error_calls.is_empty() {
        // No error calls in the program — property trivially holds
        return PropertyResult::True;
    }

    // Find main function
    let main_id = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .map(|f| f.id);

    let Some(main_id) = main_id else {
        return PropertyResult::Unknown {
            reason: "No main function found".into(),
        };
    };

    let Some(main_entry) = get_entry_block(module, main_id) else {
        return PropertyResult::Unknown {
            reason: "Main function has no entry block".into(),
        };
    };

    let callgraph = ctx.callgraph();

    // Track which error sites we need to check with Z3
    let mut main_error_sites: Vec<(FunctionId, BlockId, InstId)> = Vec::new();
    let mut callee_error_funcs: Vec<FunctionId> = Vec::new();

    // First pass: filter by call graph reachability and categorize
    for (func_id, block_id, inst_id) in &error_calls {
        if *func_id == main_id {
            main_error_sites.push((*func_id, *block_id, *inst_id));
        } else if is_reachable_in_callgraph(callgraph, main_id, *func_id) {
            // Track unique callee functions with errors
            if !callee_error_funcs.contains(func_id) {
                callee_error_funcs.push(*func_id);
            }
        }
    }

    if main_error_sites.is_empty() && callee_error_funcs.is_empty() {
        // All error calls are in functions unreachable from main
        return PropertyResult::True;
    }

    let mut any_reachable = false;
    let mut witness_path: Option<Vec<String>> = None;

    // Second pass: check error sites in main directly with Z3
    for (_func_id, block_id, _inst_id) in &main_error_sites {
        let result = check_path_reachable(
            main_entry,
            *block_id,
            main_id,
            module,
            config.z3_timeout_ms,
            config.max_guards,
            config.max_paths,
        );

        match result.result {
            PathReachability::Reachable(path) => {
                any_reachable = true;
                witness_path = Some(path.iter().map(|b| format!("block_{}", b.raw())).collect());
                break;
            }
            PathReachability::Unreachable => {
                // This path is infeasible, continue checking others
            }
            PathReachability::Unknown => {
                // Couldn't determine, be conservative
                return PropertyResult::Unknown {
                    reason: format!(
                        "Z3 could not determine path feasibility (checked {} paths)",
                        result.paths_checked
                    ),
                };
            }
        }
    }

    // Return early if error in main is reachable
    if any_reachable {
        return PropertyResult::False {
            witness: witness_path,
        };
    }

    // Third pass: P2 - Use function summaries for callee errors
    if !callee_error_funcs.is_empty() {
        // Compute error summaries for all functions
        let summaries = compute_error_summaries(module, callgraph, config);

        // Check each callee that contains error calls
        for callee_id in &callee_error_funcs {
            let callee_summary =
                summaries
                    .get(callee_id)
                    .cloned()
                    .unwrap_or_else(|| ErrorSummary::Unknown {
                        reason: "No summary computed".into(),
                    });

            match &callee_summary {
                ErrorSummary::NeverErrors => {
                    // Error in this callee is intraprocedurally unreachable —
                    // fall through to the next callee.
                }
                ErrorSummary::Unknown { reason } => {
                    // Can't determine callee behavior, be conservative
                    return PropertyResult::Unknown {
                        reason: format!(
                            "Error in callee function, summary computation failed: {reason}"
                        ),
                    };
                }
                ErrorSummary::AlwaysErrors | ErrorSummary::MayError { .. } => {
                    // Callee may reach error - check if any call site to this callee is reachable
                    let call_sites = find_call_sites_to(module, *callee_id);

                    for (caller_id, call_block, _inst_id) in &call_sites {
                        // Only care about call sites in main (for now - single-level interprocedural)
                        if *caller_id != main_id {
                            // Skip non-main callers for now (would need recursive summary application)
                            continue;
                        }

                        // Check if call site is reachable from main entry
                        let result = check_path_reachable(
                            main_entry,
                            *call_block,
                            main_id,
                            module,
                            config.z3_timeout_ms,
                            config.max_guards,
                            config.max_paths,
                        );

                        match result.result {
                            PathReachability::Reachable(path) => {
                                // In aggressive mode, report FALSE for reachable call to may-error callee
                                if !config.conservative {
                                    return PropertyResult::False {
                                        witness: Some(
                                            path.iter()
                                                .map(|b| format!("block_{}", b.raw()))
                                                .collect(),
                                        ),
                                    };
                                }
                                // In conservative mode, return UNKNOWN (callee MAY error)
                                return PropertyResult::Unknown {
                                    reason: format!(
                                        "Call to may-error function is reachable (checked {} paths)",
                                        result.paths_checked
                                    ),
                                };
                            }
                            PathReachability::Unreachable => {
                                // Call site is unreachable, this callee's error is safe
                            }
                            PathReachability::Unknown => {
                                return PropertyResult::Unknown {
                                    reason: format!(
                                        "Z3 could not determine call site reachability (checked {} paths)",
                                        result.paths_checked
                                    ),
                                };
                            }
                        }
                    }

                    // Check for non-main callers - we can only handle 1-level deep for now
                    let non_main_callers: Vec<_> = call_sites
                        .iter()
                        .filter(|(caller_id, _, _)| *caller_id != main_id)
                        .collect();

                    if !non_main_callers.is_empty() {
                        // There are calls from non-main functions - need deeper interprocedural analysis
                        // For now, return UNKNOWN in conservative mode, or continue in aggressive mode
                        if config.conservative {
                            return PropertyResult::Unknown {
                                reason: format!(
                                    "Error in callee called from {} non-main function(s); \
                                     deeper interprocedural analysis needed",
                                    non_main_callers.len()
                                ),
                            };
                        }
                        // In aggressive mode, we checked main call sites; assume non-main paths are safe
                    }
                }
            }
        }
    }

    // All checked paths are infeasible
    PropertyResult::True
}

/// Find all call sites to a specific function.
fn find_call_sites_to(
    module: &AirModule,
    target_id: FunctionId,
) -> Vec<(FunctionId, BlockId, InstId)> {
    let mut results = Vec::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Operation::CallDirect { callee, .. } = &inst.op {
                    if *callee == target_id {
                        results.push((func.id, block.id, inst.id));
                    }
                }
            }
        }
    }

    results
}

/// Analyze `valid-memsafety` property (null-deref, UAF, double-free, buffer overflow).
///
/// Strategy:
/// 1. P4 Fast-path: stack-only programs → only check null-deref (no UAF/double-free possible)
/// 2. Build SVFG with PTA-guided memory SSA and program point tracking
/// 3. Run path-sensitive SVFG checkers for null-deref, UAF, double-free
/// 4. P5 Flow-sensitive UAF filtering: filter out UAF where use happens before free
/// 5. Run abstract interpretation buffer overflow checker (array bounds)
/// 6. If any feasible findings → FALSE, else TRUE (if no potential issues)
// NOTE: This function implements the full memory safety analysis pipeline
// as a cohesive unit. Splitting would fragment the analysis logic.
#[allow(clippy::too_many_lines)]
fn analyze_memsafety(ctx: &AnalysisContext<'_>) -> PropertyResult {
    use saf_analysis::absint::{
        AbstractInterpConfig, NumericSeverity, check_buffer_overflow_with_specs,
        check_memcpy_overflow_with_result,
    };
    use saf_analysis::checkers::filter_temporal_infeasible;
    use saf_analysis::checkers::spec::{double_free, null_deref, use_after_free};
    use saf_analysis::checkers::summary::compute_parameter_effect_summaries;

    let module = ctx.module;
    let config = ctx.config;
    let reachable = ctx.reachable();

    // Build analyzed specs with computed bounds (used by both temporal filter and buffer overflow)
    let mut analyzed_specs = build_analyzed_specs();

    // P4: Fast-path for stack-only programs (scoped to reachable functions)
    // Without heap allocations, UAF and double-free are impossible
    let is_stack_only = !reachable_has_heap_allocations(module, reachable);

    // Build SVFG with program points for temporal ordering (P5)
    let (mut svfg, program_points) = ctx.build_svfg_with_program_points();
    tracing::debug!(
        svfg_nodes = svfg.node_count(),
        program_points = program_points.len(),
        "Built SVFG with program points"
    );

    // Run SCCP pre-pass for dead block identification
    let sccp_result = saf_analysis::absint::sccp::run_sccp_module(module);
    // Prune SVFG edges from dead PHI incoming blocks to reduce false positives
    let pruned =
        saf_analysis::svfg::prune_dead_phi_edges(&mut svfg, module, &sccp_result.dead_blocks);
    if pruned > 0 {
        tracing::debug!(
            pruned_edges = pruned,
            "Pruned dead PHI incoming edges from SVFG"
        );
    }

    // Build block_of map: maps each defined `ValueId` to its containing `BlockId`
    let mut block_of_map = std::collections::BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    block_of_map.insert(dst, block.id);
                }
            }
        }
    }

    // Build guard context for path-sensitive checker
    let guard_ctx = GuardContext {
        dead_blocks: sccp_result.dead_blocks,
        block_of: block_of_map,
        config: GuardedSolverConfig {
            base: SolverConfig::default(),
            max_disjuncts: 20,
        },
    };

    // Set up path-sensitive checker configuration
    let ps_config = PathSensitiveConfig {
        z3_timeout_ms: config.z3_timeout_ms,
        max_guards_per_trace: config.max_guards,
        guard_context: Some(guard_ctx),
        ..PathSensitiveConfig::default()
    };

    // Build resource table for checker classification
    let table = ResourceTable::new();

    // Define the memory safety checkers
    // P4: For stack-only programs, only check null-deref
    let specs = if is_stack_only {
        vec![null_deref()]
    } else {
        vec![null_deref(), use_after_free(), double_free()]
    };

    // Run path-sensitive analysis for null-deref, UAF, double-free
    let result: PathSensitiveResult =
        run_checkers_path_sensitive(&specs, module, &svfg, &table, &ps_config);

    // P5: Apply flow-sensitive temporal filtering for UAF
    // This filters out UAF findings where the "use" happens BEFORE the "free"
    // in program order (false positives due to SVFG not capturing temporal ordering)
    let cfgs = ctx.cfgs();
    let pre_filter_feasible = result.feasible.len();
    let pre_filter_uaf = result
        .feasible
        .iter()
        .filter(|f| f.checker_name == "use-after-free")
        .count();
    let summaries = compute_parameter_effect_summaries(module, &table);
    // Populate analyzed registry with summary-derived specs
    for (func_id, summary) in &summaries {
        if let Some(func) = module.functions.iter().find(|f| f.id == *func_id) {
            let existing = analyzed_specs
                .lookup_derived(&func.name)
                .cloned()
                .unwrap_or_else(DerivedSpec::empty);
            let merged = DerivedSpec {
                computed_return_bound: existing.computed_return_bound,
                param_freed: summary.param_freed.clone(),
                param_dereferenced: summary.param_dereferenced.clone(),
                return_is_allocated: summary.return_is_allocated,
            };
            analyzed_specs.add_derived(&func.name, merged);
        }
    }
    let result = filter_temporal_infeasible(
        result,
        &program_points,
        cfgs,
        Some(&analyzed_specs),
        module,
        &table,
    );
    let post_filter_feasible = result.feasible.len();
    let post_filter_uaf = result
        .feasible
        .iter()
        .filter(|f| f.checker_name == "use-after-free")
        .count();

    // Log filtering statistics when temporal filter has an effect
    if pre_filter_uaf != post_filter_uaf {
        tracing::debug!(
            pre_filter_feasible,
            post_filter_feasible,
            pre_filter_uaf,
            post_filter_uaf,
            "Temporal filter removed UAF false positives"
        );
    }

    // Filter SVFG findings to reachable functions only
    let value_func_map = ctx.value_function_map();
    let is_node_in_reachable = |node: &SvfgNodeId| -> bool {
        match node {
            SvfgNodeId::Value(vid) => value_func_map
                .get(vid)
                .is_none_or(|fid| reachable.contains(fid)),
            SvfgNodeId::MemPhi(_) => true,
        }
    };
    let pre_reachable_feasible = result.feasible.len();
    let result = PathSensitiveResult {
        feasible: result
            .feasible
            .into_iter()
            .filter(|f| is_node_in_reachable(&f.source_node) || is_node_in_reachable(&f.sink_node))
            .collect(),
        infeasible: result
            .infeasible
            .into_iter()
            .filter(|f| is_node_in_reachable(&f.source_node) || is_node_in_reachable(&f.sink_node))
            .collect(),
        unknown: result
            .unknown
            .into_iter()
            .filter(|f| is_node_in_reachable(&f.source_node) || is_node_in_reachable(&f.sink_node))
            .collect(),
        diagnostics: result.diagnostics,
    };
    if result.feasible.len() < pre_reachable_feasible {
        tracing::debug!(
            before = pre_reachable_feasible,
            after = result.feasible.len(),
            "Reachability filter removed unreachable SVFG findings"
        );
    }

    // Phase 5: Deduplicate SVFG findings by root cause (checker_name, source_node).
    // Multiple findings from the same allocation/free site should count as one
    // to prevent inflated finding counts from flipping verdicts.
    let pre_dedup_feasible = result.feasible.len();
    let mut seen_roots: BTreeSet<(String, SvfgNodeId)> = BTreeSet::new();
    let result = PathSensitiveResult {
        feasible: result
            .feasible
            .into_iter()
            .filter(|f| seen_roots.insert((f.checker_name.clone(), f.source_node)))
            .collect(),
        infeasible: result.infeasible,
        unknown: result.unknown,
        diagnostics: result.diagnostics,
    };
    if result.feasible.len() < pre_dedup_feasible {
        tracing::debug!(
            before = pre_dedup_feasible,
            after = result.feasible.len(),
            "Root-cause deduplication removed redundant SVFG findings"
        );
    }

    // Separate null-deref findings from UAF/double-free findings
    // UAF and double-free are data flow issues - if feasible, they're confirmed violations
    // Null-deref from malloc without null check is NOT confirmed because:
    //   1. malloc CAN return NULL, but usually doesn't in practice
    //   2. SV-COMP assumes malloc succeeds (no out-of-memory failures)
    //   3. We can't prove malloc actually returns NULL without explicit evidence
    let non_null_deref_feasible: Vec<_> = result
        .feasible
        .iter()
        .filter(|f| f.checker_name != "null-deref")
        .collect();

    // If there are non-null-deref feasible findings (UAF, double-free), report FALSE.
    // These findings are already Z3-confirmed (path-feasible), so high confidence.
    if !non_null_deref_feasible.is_empty() {
        let descriptions: Vec<String> = non_null_deref_feasible
            .iter()
            .take(3)
            .map(|f| {
                format!(
                    "{}: {} (CWE-{})",
                    f.checker_name,
                    f.message,
                    f.cwe.unwrap_or(0)
                )
            })
            .collect();

        return PropertyResult::False {
            witness: Some(descriptions),
        };
    }

    // Separate null-deref findings by source kind
    let explicit_null_deref: Vec<_> = result
        .feasible
        .iter()
        .filter(|f| f.checker_name == "null-deref" && f.source_kind == NullSourceKind::ExplicitNull)
        .collect();

    // Explicit NULL dereferences are definite bugs — report FALSE
    if !explicit_null_deref.is_empty() {
        let descriptions: Vec<String> = explicit_null_deref
            .iter()
            .take(3)
            .map(|f| {
                format!(
                    "{}: {} (CWE-{})",
                    f.checker_name,
                    f.message,
                    f.cwe.unwrap_or(0)
                )
            })
            .collect();

        return PropertyResult::False {
            witness: Some(descriptions),
        };
    }

    // Function-may-return-NULL dereferences remain UNKNOWN (can't prove malloc returns NULL)
    let maybe_null_deref_count = result
        .feasible
        .iter()
        .filter(|f| f.checker_name == "null-deref" && f.source_kind != NullSourceKind::ExplicitNull)
        .count();

    if maybe_null_deref_count > 0 {
        return PropertyResult::Unknown {
            reason: format!(
                "{maybe_null_deref_count} potential null-deref(s) from heap allocation without null check; \
                 cannot confirm malloc returns NULL"
            ),
        };
    }

    // Run buffer overflow checker using spec-aware abstract interpretation
    // This catches out-of-bounds array accesses (valid-deref violations)
    // Using specs so external calls (atoi, fgets, etc.) return bounded intervals instead of TOP
    let ai_config = AbstractInterpConfig::default();
    let bo_result = check_buffer_overflow_with_specs(module, &ai_config, Some(&analyzed_specs));

    // Reuse the abstract interpretation result for memcpy overflow checking
    // to avoid re-running the expensive fixpoint solver
    let mut all_overflow_findings: Vec<_> = bo_result.findings;
    if let Some(ref absint_result) = bo_result.absint_result {
        let memcpy_findings =
            check_memcpy_overflow_with_result(module, absint_result, Some(analyzed_specs.yaml()));
        let pre_memcpy_count = memcpy_findings.len();
        if pre_memcpy_count > 0 {
            tracing::debug!(
                count = pre_memcpy_count,
                "Memcpy overflow checker found findings"
            );
        }
        all_overflow_findings.extend(memcpy_findings);
    }

    // Filter all overflow findings (GEP + memcpy) to reachable functions only
    let pre_bo_count = all_overflow_findings.len();
    let bo_findings: Vec<_> = all_overflow_findings
        .into_iter()
        .filter(|f| reachable.contains(&f.location.0))
        .collect();
    if bo_findings.len() < pre_bo_count {
        tracing::debug!(
            before = pre_bo_count,
            after = bo_findings.len(),
            "Reachability filter removed unreachable overflow findings"
        );
    }

    // Phase 5: Deduplicate overflow findings by root cause (function, inst_id).
    // Multiple findings from the same instruction should count as one.
    let pre_dedup_bo = bo_findings.len();
    let mut seen_bo: BTreeSet<(FunctionId, String)> = BTreeSet::new();
    let bo_findings: Vec<_> = bo_findings
        .into_iter()
        .filter(|f| seen_bo.insert((f.location.0, f.inst_id.clone())))
        .collect();
    if bo_findings.len() < pre_dedup_bo {
        tracing::debug!(
            before = pre_dedup_bo,
            after = bo_findings.len(),
            "Root-cause deduplication removed redundant overflow findings"
        );
    }

    // Check for definite buffer overflows (Error severity)
    let definite_overflows: Vec<_> = bo_findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Error)
        .collect();

    if !definite_overflows.is_empty() {
        let descriptions: Vec<String> = definite_overflows
            .iter()
            .take(3)
            .map(|f| format!("buffer-overflow: {} (CWE-{})", f.description, f.cwe))
            .collect();

        return PropertyResult::False {
            witness: Some(descriptions),
        };
    }

    // Warning-severity buffer overflows indicate potential (not confirmed) issues.
    // Unlike Error findings which are definite, Warnings require Z3 confirmation
    // to avoid false positives from imprecise interval analysis.
    let potential_overflows: Vec<_> = bo_findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Warning)
        .cloned()
        .collect();

    if !potential_overflows.is_empty() && !config.conservative {
        use saf_analysis::absint::numeric_z3::filter_numeric_z3;

        let z3_result = filter_numeric_z3(
            &potential_overflows,
            module,
            config.z3_timeout_ms,
            config.max_guards,
        );

        tracing::debug!(
            total = potential_overflows.len(),
            confirmed = z3_result.confirmed.len(),
            refuted = z3_result.refuted.len(),
            uncertain = z3_result.uncertain.len(),
            "Z3 buffer overflow refinement"
        );

        // Z3-confirmed findings are high-confidence real bugs
        if !z3_result.confirmed.is_empty() {
            let descriptions: Vec<String> = z3_result
                .confirmed
                .iter()
                .take(3)
                .map(|f| format!("buffer-overflow: {} (CWE-{})", f.description, f.cwe))
                .collect();

            return PropertyResult::False {
                witness: Some(descriptions),
            };
        }

        // If all findings were refuted by Z3, the warnings were false positives
        if z3_result.uncertain.is_empty() {
            // All buffer overflow warnings were on infeasible paths
            // Continue to final verdict (no violations found)
        } else {
            // Still have unconfirmed findings after Z3 refinement
            return PropertyResult::Unknown {
                reason: format!(
                    "{} potential buffer overflow(s) after Z3 refinement ({} uncertain)",
                    z3_result.uncertain.len(),
                    z3_result.uncertain.len()
                ),
            };
        }
    } else if !potential_overflows.is_empty() {
        return PropertyResult::Unknown {
            reason: format!(
                "{} potential buffer overflow(s) detected; more precise analysis needed",
                potential_overflows.len()
            ),
        };
    }

    // If all potential findings were proven infeasible by Z3, the program is
    // verified safe for the checked memory safety properties.
    if !result.infeasible.is_empty() && result.unknown.is_empty() {
        return PropertyResult::True;
    }

    // If there are unknown findings (Z3 timeout), we cannot claim safety
    if !result.unknown.is_empty() {
        return PropertyResult::Unknown {
            reason: format!(
                "{} findings could not be verified by Z3 (timeout/unknown)",
                result.unknown.len()
            ),
        };
    }

    // No feasible findings, no unknown findings, and no infeasible findings
    // means our checkers found no issues at all - claim safety
    // This is a TRUE verdict: checkers ran successfully with no findings
    PropertyResult::True
}

/// Analyze `no-overflow` property (signed integer overflow).
///
/// Strategy:
/// 1. Run interprocedural abstract interpretation
/// 2. Check Add/Sub/Mul operations against signed bounds
/// 3. P3: For loop-free programs, intervals are exact → trust results
/// 4. If definite overflow (Error severity) → FALSE (aggressive) or UNKNOWN (conservative)
fn analyze_no_overflow(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let result = ctx.absint();
    let cfgs = ctx.cfgs();
    let reachable = ctx.reachable();

    // Filter absint findings to reachable functions only
    let reachable_findings: Vec<_> = result
        .findings
        .iter()
        .filter(|f| reachable.contains(&f.location.0))
        .collect();
    if reachable_findings.len() < result.findings.len() {
        tracing::debug!(
            before = result.findings.len(),
            after = reachable_findings.len(),
            "Reachability filter removed unreachable overflow findings"
        );
    }

    // Check for definite overflows (Error severity)
    let definite_overflows: Vec<_> = reachable_findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Error)
        .collect();

    if !definite_overflows.is_empty() {
        let descriptions: Vec<String> = definite_overflows
            .iter()
            .take(3)
            .map(|f| format!("{} in {} ({})", f.description, f.function, f.interval))
            .collect();

        // Phase 3: Per-finding loop-free check (L5 refinement)
        // Instead of requiring ALL reachable functions to be loop-free,
        // check if the function containing each finding is loop-free.
        // In a loop-free function, intervals are exact (no widening involved),
        // so Error-severity findings are definite.
        if !ctx.config.conservative {
            let any_in_loop_free_fn = definite_overflows
                .iter()
                .any(|f| function_is_loop_free(cfgs, f.location.0));

            if any_in_loop_free_fn {
                return PropertyResult::False {
                    witness: Some(descriptions),
                };
            }
        }

        return PropertyResult::Unknown {
            reason: format!(
                "{} potential overflow(s) detected: {}",
                definite_overflows.len(),
                descriptions.join("; ")
            ),
        };
    }

    // Check for potential overflows (Warning severity)
    let potential_overflows: Vec<_> = reachable_findings
        .iter()
        .filter(|f| f.severity == NumericSeverity::Warning)
        .collect();

    if !potential_overflows.is_empty() {
        // Warnings indicate possible overflow - be conservative
        return PropertyResult::Unknown {
            reason: format!(
                "{} potential overflow(s) detected; more precise analysis needed",
                potential_overflows.len()
            ),
        };
    }

    // P3: For loop-free programs in aggressive mode, no findings = likely safe
    // Without loops, there's no widening, so intervals are exact
    let is_loop_free = reachable_is_loop_free(cfgs, reachable);
    if is_loop_free && !ctx.config.conservative {
        return PropertyResult::True;
    }

    PropertyResult::Unknown {
        reason: if is_loop_free {
            "Loop-free program with no overflow findings, but analysis coverage uncertain".into()
        } else {
            "No overflows detected, but interval analysis has limited precision".into()
        },
    }
}

/// Check if target is reachable from source in the call graph.
fn is_reachable_in_callgraph(
    callgraph: &CallGraph,
    source: FunctionId,
    target: FunctionId,
) -> bool {
    use saf_analysis::callgraph::CallGraphNode;
    use std::collections::BTreeSet;

    if source == target {
        return true;
    }

    let source_node = CallGraphNode::Function(source);
    let target_node = CallGraphNode::Function(target);

    // BFS traversal
    let mut visited = BTreeSet::new();
    let mut queue = vec![source_node.clone()];
    visited.insert(source_node);

    while let Some(current) = queue.pop() {
        if let Some(callees) = callgraph.edges.get(&current) {
            for callee in callees {
                if *callee == target_node {
                    return true;
                }
                if visited.insert(callee.clone()) {
                    queue.push(callee.clone());
                }
            }
        }
    }

    false
}

/// Analyze `no-data-race` property.
///
/// Strategy:
/// 1. P0 Fast-path: no threading primitives → TRUE (no threads = no races)
/// 2. Run MTA analysis to discover threads and MHP relationships
/// 3. Fast-path: single thread → TRUE
/// 4. Collect Load/Store accesses per thread
/// 5. For concurrent threads, check if they access same memory (via PTA aliasing)
/// 6. If racing pair found → UNKNOWN (conservative), else UNKNOWN
fn analyze_no_data_race(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let module = ctx.module;

    // P0: Fast-path for sequential programs
    // If no threading primitives exist, there can be no data races
    if !has_threading_primitives(module) {
        return PropertyResult::True;
    }

    let mta_result = ctx.mta();

    // If no threads or only main thread, property trivially holds
    if mta_result.thread_graph.threads.len() <= 1 {
        return PropertyResult::True;
    }

    // Collect memory accesses per thread
    let mut thread_accesses: BTreeMap<u32, Vec<MemoryAccessInfo>> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                let access_kind = match &inst.op {
                    Operation::Load => Some(AccessKind::Read),
                    Operation::Store => Some(AccessKind::Write),
                    _ => None,
                };

                if let Some(kind) = access_kind {
                    // Get the pointer being accessed
                    let ptr = match &inst.op {
                        Operation::Load => inst.operands.first().copied(),
                        Operation::Store => inst.operands.get(1).copied(), // operands: [value, ptr]
                        _ => None,
                    };

                    if let Some(ptr_val) = ptr {
                        // For now, assume main thread for all accesses
                        // A more sophisticated approach would track which function
                        // belongs to which thread
                        let access = MemoryAccessInfo { ptr: ptr_val, kind };

                        // Check which threads might execute this function
                        for (thread_id, thread_ctx) in mta_result.threads() {
                            if thread_ctx.entry_function == func.id {
                                thread_accesses
                                    .entry(thread_id.0)
                                    .or_default()
                                    .push(access.clone());
                            }
                        }

                        // Also add to main thread if in main
                        if func.name == "main" {
                            thread_accesses.entry(0).or_default().push(access);
                        }
                    }
                }
            }
        }
    }

    // Check for races between concurrent threads
    let pta = ctx.pta();
    let threads: Vec<_> = thread_accesses.keys().copied().collect();

    for i in 0..threads.len() {
        for j in (i + 1)..threads.len() {
            let t1 = threads[i];
            let t2 = threads[j];

            // Check if these threads may run concurrently
            if !mta_result.may_run_concurrently(ThreadId(t1), ThreadId(t2)) {
                continue;
            }

            let accesses_t1 = thread_accesses.get(&t1).map_or(&[][..], |v| v.as_slice());
            let accesses_t2 = thread_accesses.get(&t2).map_or(&[][..], |v| v.as_slice());

            // Check for conflicting accesses (at least one write, same location)
            for a1 in accesses_t1 {
                for a2 in accesses_t2 {
                    // Race requires at least one write
                    if matches!(a1.kind, AccessKind::Read) && matches!(a2.kind, AccessKind::Read) {
                        continue;
                    }

                    // Check if pointers may alias
                    let alias = pta.may_alias(a1.ptr, a2.ptr);
                    if matches!(
                        alias,
                        AliasResult::Must | AliasResult::May | AliasResult::Partial
                    ) {
                        // Found a potential race - but may be false positive
                        // due to imprecise alias/MHP analysis
                        return PropertyResult::Unknown {
                            reason: format!(
                                "Potential race between thread {} ({:?}) and thread {} ({:?}), but may be false positive",
                                t1, a1.kind, t2, a2.kind
                            ),
                        };
                    }
                }
            }
        }
    }

    // No races found - but our analysis might have missed some
    // We can only prove absence of races for simple cases
    PropertyResult::Unknown {
        reason: "No data races detected, but MHP and alias analysis have limited precision".into(),
    }
}

/// Information about a memory access for race detection.
#[derive(Clone)]
struct MemoryAccessInfo {
    ptr: ValueId,
    kind: AccessKind,
}

// ---------------------------------------------------------------------------
// Termination Analysis
// ---------------------------------------------------------------------------

/// Analyze the termination property.
///
/// A program satisfies the termination property if it always terminates
/// (no infinite loops). This is undecidable in general, but we can detect
/// specific patterns.
///
/// For TRUE verdicts: We only need to prove termination, not check other properties.
/// For FALSE verdicts: We need to prove non-termination (very hard without ranking functions).
fn analyze_termination(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let module = ctx.module;
    let config = ctx.config;

    // Build CFGs for all functions
    let mut cfgs = std::collections::BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        cfgs.insert(func.id, saf_analysis::cfg::Cfg::build(func));
    }

    // If program is loop-free, it definitely terminates
    if program_is_loop_free(&cfgs) {
        return PropertyResult::True;
    }

    // Check for loops that depend on nondeterministic input
    let nondet_loops = find_nonterminating_loops(module, &cfgs);

    if nondet_loops.is_empty() {
        // Program has loops but none depend on nondeterministic input
        // In aggressive mode, assume bounded loops terminate
        // This is a heuristic: most loops in real programs have finite bounds
        if !config.conservative {
            return PropertyResult::True;
        }

        return PropertyResult::Unknown {
            reason: "Program has loops; termination analysis requires ranking functions".into(),
        };
    }

    // Has nondeterministic loops - these may or may not terminate
    // Cannot safely return FALSE (loops may terminate via break/return/exit)
    // Cannot safely return TRUE (loops may run forever)
    PropertyResult::Unknown {
        reason: format!(
            "{} loop(s) with nondeterministic condition; termination undecidable",
            nondet_loops.len()
        ),
    }
}

// ---------------------------------------------------------------------------
// Memory Cleanup (Leak Detection) Analysis
// ---------------------------------------------------------------------------

/// Analyze the valid-memcleanup property.
///
/// A program satisfies valid-memcleanup if all dynamically allocated memory
/// is eventually freed before program termination.
///
/// For TRUE verdicts: We only need to prove no memory leaks exist.
/// For FALSE verdicts: We need to prove a leak exists on some path.
fn analyze_memcleanup(ctx: &AnalysisContext<'_>) -> PropertyResult {
    let module = ctx.module;
    let config = ctx.config;

    // Fast-path: no heap allocations means no memory leaks possible
    if !has_heap_allocations(module) {
        return PropertyResult::True;
    }

    // Find all allocation and deallocation sites
    let alloc_sites = find_allocation_sites(module);
    let dealloc_sites = find_deallocation_sites(module);

    if alloc_sites.is_empty() {
        // No allocations found in code (only in declarations)
        return PropertyResult::True;
    }

    // If there are allocations but no deallocations, there's definitely a leak
    if dealloc_sites.is_empty() {
        if !config.conservative {
            return PropertyResult::False {
                witness: Some(vec![format!(
                    "{} allocation(s) found but no deallocation calls",
                    alloc_sites.len()
                )]),
            };
        }
        return PropertyResult::Unknown {
            reason: format!(
                "{} allocation(s) found but no deallocation calls detected",
                alloc_sites.len()
            ),
        };
    }

    // Check for _Exit calls which skip atexit handlers entirely
    let has_exit_bypass = module
        .functions
        .iter()
        .any(|f| f.name == "_Exit" || f.name == "_exit" || f.name == "quick_exit");

    // Check if program uses atexit() for cleanup
    // If yes, we need to be very careful because:
    // 1. exit() runs atexit handlers in reverse registration order
    // 2. If exit() is called before some atexit() registrations, those handlers never run
    // 3. Handler interaction can cause leaks (e.g., one handler sets ptr to NULL before another checks it)
    let has_atexit = module.functions.iter().any(|f| f.name == "atexit");
    let has_exit = module.functions.iter().any(|f| f.name == "exit");

    // Programs using both atexit() and exit() have complex cleanup semantics
    // We cannot safely return TRUE without path-sensitive analysis of handler registration order
    let has_complex_cleanup = has_atexit && has_exit;

    // Use SVFG to track allocation to deallocation
    // Build PTA and SVFG for precise tracking
    let pta = ctx.pta();
    let (svfg, _program_points) = ctx.build_svfg_with_program_points();

    // For each allocation, check if it flows to a deallocation
    let mut potentially_leaked = Vec::new();
    let mut all_tracked = true;

    for (func_id, block_id, _inst_id, alloc_value) in &alloc_sites {
        // Check if this allocation value reaches any free call
        let mut reaches_free = false;

        // Check if the value is tracked in SVFG
        let value_in_svfg = svfg.value_nodes().any(|v| v == *alloc_value);
        if value_in_svfg {
            reaches_free = true;
        }

        // Also check via points-to analysis
        if !reaches_free {
            let pt_set = pta.points_to(*alloc_value);
            if !pt_set.is_empty() {
                reaches_free = true;
            }
        }

        if !reaches_free {
            potentially_leaked.push((*func_id, *block_id, *alloc_value));
            all_tracked = false;
        }
    }

    // If all allocations are tracked and no _Exit bypass exists,
    // we can be more confident about returning TRUE
    if potentially_leaked.is_empty() && all_tracked {
        // In aggressive mode, trust the SVFG tracking
        // But be careful about:
        // 1. _Exit bypassing atexit handlers entirely
        // 2. atexit + exit combinations where handler registration order matters
        if !config.conservative && !has_exit_bypass && !has_complex_cleanup {
            // All allocations flow to deallocations, no _Exit bypass,
            // and no complex atexit+exit interactions
            return PropertyResult::True;
        }

        // Conservative: can't prove all paths free memory
        return PropertyResult::Unknown {
            reason: if has_exit_bypass {
                "Program may use _Exit which bypasses atexit cleanup handlers".into()
            } else if has_complex_cleanup {
                "Program uses atexit() with exit(); cleanup depends on registration order".into()
            } else {
                "Memory allocations detected but couldn't prove all paths free memory".into()
            },
        };
    }

    // Some allocations might leak
    if !config.conservative {
        let descriptions: Vec<String> = potentially_leaked
            .iter()
            .take(3)
            .filter_map(|(func_id, _block_id, _value_id)| {
                module.function(*func_id).map(|f| {
                    format!(
                        "Potential memory leak: allocation in {} may not be freed",
                        f.name
                    )
                })
            })
            .collect();

        return PropertyResult::False {
            witness: Some(descriptions),
        };
    }

    PropertyResult::Unknown {
        reason: format!(
            "{} allocation(s) may not be freed on all paths",
            potentially_leaked.len()
        ),
    }
}

// ---------------------------------------------------------------------------
// Sound must-reach analysis for unreach-call (plan 192 §1.6, revised after the
// slice-1 blind eval).
//
// The Z3 path-feasibility engine *over*-approximates reachability (it models a
// guard's operands as unconstrained fresh variables, ignoring the arithmetic /
// casts / pointer-identity that define them, and does not compose caller↔callee
// argument constraints). "SAT" therefore means "could not prove infeasible", not
// "provably feasible" — sound for dropping false positives in a bug *reporter*,
// but UNSOUND as grounds for emitting a FALSE verdict (a wrong `false` scores
// −16 in SV-COMP). This function is the dual, *under*-approximate check: it
// reports a violation only when `reach_error` is guaranteed to execute.
// ---------------------------------------------------------------------------

/// Result of the unconditional (must-execute) reachability walk of a function.
#[derive(Debug, Clone)]
enum MustResult {
    /// `reach_error` is unconditionally reached; carries the block path witness.
    ReachesError(Vec<BlockId>),
    /// The function unconditionally runs to a normal return.
    Returns,
    /// Neither can be established (a branch, loop, recursion, or opaque call).
    Indeterminate,
}

/// Error-call target names for the unreach-call property.
const REACH_ERROR_NAMES: &[&str] = &["reach_error", "__VERIFIER_error"];

/// Assumption intrinsics. An assumption conditions all downstream execution
/// (`assume(false)` blocks the path entirely), so nothing after one is
/// *unconditionally* reached — the must-reach walk stops at an assume.
const ASSUME_FUNCTIONS: &[&str] = &["__VERIFIER_assume", "__CPROVER_assume"];

/// Functions that never return — execution stops before anything after the call.
const NORETURN_FUNCTIONS: &[&str] = &[
    "exit",
    "_exit",
    "_Exit",
    "abort",
    "__assert_fail",
    "__assert_rtn",
    "__builtin_unreachable",
    "__builtin_trap",
    "longjmp",
    "siglongjmp",
];

/// Sound *must-reach* check for the unreach-call property.
///
/// Returns the unconditionally-executed block path to a `reach_error` /
/// `__VERIFIER_error` call if — and only if — that call is guaranteed to execute
/// on every run from `main`: it follows forced single-successor CFG edges and
/// unconditionally-executed calls, and stops (yielding "no proof") at any branch,
/// loop back-edge, recursion cycle, or no-return call. A `Some` result is a
/// genuine, guard-free violation, so emitting `false(unreach-call)` on it cannot
/// be a false alarm. Returns `None` otherwise (the caller then emits `unknown`).
#[must_use]
pub fn must_reach_error(module: &AirModule) -> Option<Vec<BlockId>> {
    let main = module.function_by_name("main")?;
    let mut visiting = BTreeSet::new();
    match must_reach_fn(main.id, module, &mut visiting) {
        MustResult::ReachesError(path) if !path.is_empty() => Some(path),
        _ => None,
    }
}

/// Recursion-guarded wrapper around [`must_reach_body`].
fn must_reach_fn(
    func_id: FunctionId,
    module: &AirModule,
    visiting: &mut BTreeSet<FunctionId>,
) -> MustResult {
    if !visiting.insert(func_id) {
        // Recursion cycle: cannot establish that this call must return, so we
        // cannot claim anything after it is must-reached.
        return MustResult::Indeterminate;
    }
    let result = must_reach_body(func_id, module, visiting);
    visiting.remove(&func_id);
    result
}

/// Walk the unconditional (forced single-successor) block chain of one function,
/// scanning unconditionally-executed instructions in order.
fn must_reach_body(
    func_id: FunctionId,
    module: &AirModule,
    visiting: &mut BTreeSet<FunctionId>,
) -> MustResult {
    let Some(func) = module.function(func_id) else {
        return MustResult::Indeterminate;
    };
    if func.is_declaration {
        // Opaque external: assume it returns normally. (Residual: a diverging
        // external not listed in NORETURN_FUNCTIONS is modeled optimistically.)
        return MustResult::Returns;
    }
    // The LLVM frontend leaves `entry_block` unset and uses blocks[0] as the
    // entry (mirroring `Cfg::build`'s own fallback), so fall back to it here.
    let Some(entry) = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))
    else {
        return MustResult::Indeterminate;
    };

    let cfg = Cfg::build(func);
    let mut path = Vec::new();
    let mut visited_blocks = BTreeSet::new();
    let mut block_id = entry;

    loop {
        if !visited_blocks.insert(block_id) {
            // Loop back-edge: may not terminate, so nothing past it is must-reached.
            return MustResult::Indeterminate;
        }
        path.push(block_id);
        let Some(block) = func.blocks.iter().find(|b| b.id == block_id) else {
            return MustResult::Indeterminate;
        };

        for inst in &block.instructions {
            let Operation::CallDirect { callee } = &inst.op else {
                continue;
            };
            let name = module.function(*callee).map_or("", |f| f.name.as_str());
            if REACH_ERROR_NAMES.contains(&name) {
                return MustResult::ReachesError(path);
            }
            if NORETURN_FUNCTIONS.contains(&name) {
                // Execution stops here; any later error is not reached.
                return MustResult::Indeterminate;
            }
            if ASSUME_FUNCTIONS.contains(&name) {
                // An assumption conditions the rest of the path (it may block it),
                // so nothing after it is *unconditionally* reached.
                return MustResult::Indeterminate;
            }
            match must_reach_fn(*callee, module, visiting) {
                MustResult::ReachesError(mut sub) => {
                    path.append(&mut sub);
                    return MustResult::ReachesError(path);
                }
                // Callee returns normally: continue scanning after the call.
                MustResult::Returns => {}
                // Callee may not return: cannot claim anything after it.
                MustResult::Indeterminate => return MustResult::Indeterminate,
            }
        }

        let Some(succs) = cfg.successors.get(&block_id) else {
            // No successors recorded (a `ret`/`unreachable` terminator): returns.
            return MustResult::Returns;
        };
        match succs.len() {
            // A `ret`/`unreachable` terminator: the function returns.
            0 => return MustResult::Returns,
            // Exactly one successor: a forced edge — follow it.
            1 => block_id = *succs.iter().next().expect("len == 1 has an element"),
            // A branch (CondBr/Switch): the chain is no longer unconditional.
            _ => return MustResult::Indeterminate,
        }
    }
}

// ---------------------------------------------------------------------------
// Concrete-replay candidates for unreach-call (slice 1c).
//
// `must_reach_error` (above) yields sound FALSE only for *unconditional*
// reaches. Guarded / nondet-driven violations need the over-approximate Z3
// reachability engine to *propose* them, then a concrete native execution (in
// saf-cli) to *confirm* them. `enumerate_false_candidates` produces those
// proposals plus the Z3 model that seeds the replay; it is UNSOUND on its own
// and must never feed a verdict without execution.
// ---------------------------------------------------------------------------

/// One `__VERIFIER_nondet_*` call resolved along a candidate's path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NondetCall {
    /// The nondet function name, e.g. `__VERIFIER_nondet_int`.
    pub func_name: String,
    /// The value the replay harness returns for this call: the Z3 model value,
    /// or `0` when the model does not constrain it.
    pub value: i64,
}

/// An over-approximate FALSE *candidate* for the unreach-call property: a
/// `reach_error` site the Z3 path engine could not prove unreachable, plus the
/// satisfying model that seeds a concrete-replay attempt.
///
/// A candidate is **not** a verdict. Because Z3 path feasibility
/// over-approximates (it models guard operands as unconstrained fresh
/// variables, ignoring the arithmetic/casts/pointer-identity that define them),
/// a candidate may not correspond to any real execution. Slice 1c confirms each
/// candidate by pinning the nondet inputs to `nondet_sequence` and running the
/// program natively — only a run that actually reaches `reach_error` yields
/// `false(unreach-call)`; everything else stays `unknown`.
#[derive(Debug, Clone)]
pub struct FalseCandidate {
    /// The `reach_error` / `__VERIFIER_error` call instruction.
    pub reach_error_inst: InstId,
    /// The block path (within the error site's function) Z3 reported feasible.
    pub block_path: Vec<BlockId>,
    /// Raw Z3 model: each symbolic operand `ValueId` → concrete value.
    pub assignments: BTreeMap<ValueId, i64>,
    /// The scalar-integer `__VERIFIER_nondet_*` calls executed along
    /// `block_path`, in program order, each paired with the value to return
    /// (model value, or `0` when unconstrained). The harness groups these per
    /// function name into the pinned input arrays.
    pub nondet_sequence: Vec<NondetCall>,
}

/// Scalar-*integer* `__VERIFIER_nondet_*` names whose result the replay harness
/// can pin from an integer model. Pointer / float / double nondet are excluded:
/// a program using them still runs (the driver returns `0`/`NULL`), but their
/// values are not model-pinned, so an error gated on them simply does not
/// reproduce → `unknown` (sound).
fn is_scalar_integer_nondet(name: &str) -> bool {
    matches!(
        name,
        "__VERIFIER_nondet_int"
            | "__VERIFIER_nondet_uint"
            | "__VERIFIER_nondet_long"
            | "__VERIFIER_nondet_ulong"
            | "__VERIFIER_nondet_longlong"
            | "__VERIFIER_nondet_ulonglong"
            | "__VERIFIER_nondet_short"
            | "__VERIFIER_nondet_ushort"
            | "__VERIFIER_nondet_char"
            | "__VERIFIER_nondet_uchar"
            | "__VERIFIER_nondet_bool"
            | "__VERIFIER_nondet_size_t"
    )
}

/// Enumerate over-approximate FALSE *candidates* for the unreach-call property.
///
/// For each `reach_error` / `__VERIFIER_error` call site, asks the Z3 path
/// engine whether the call is reachable *within its own function*; on a
/// feasible path it emits a [`FalseCandidate`] carrying the block path, the Z3
/// model, and the scalar nondet-input sequence along that path. These
/// candidates are the input to slice-1c concrete replay — see [`FalseCandidate`]
/// for the soundness contract (they must be confirmed by native execution
/// before any verdict).
#[must_use]
pub fn enumerate_false_candidates(
    module: &AirModule,
    config: &PropertyAnalysisConfig,
) -> Vec<FalseCandidate> {
    let mut error_calls = Vec::new();
    for name in REACH_ERROR_NAMES {
        error_calls.extend(find_calls_to(module, name));
    }

    let mut candidates = Vec::new();
    for (func_id, block_id, inst_id) in error_calls {
        let Some(entry) = get_entry_block(module, func_id) else {
            continue;
        };
        let result = check_path_reachable(
            entry,
            block_id,
            func_id,
            module,
            config.z3_timeout_ms,
            config.max_guards,
            config.max_paths,
        );
        if let PathReachability::Reachable(path) = result.result {
            let nondet_sequence = resolve_nondet_sequence(module, func_id, &path, &result.model);
            candidates.push(FalseCandidate {
                reach_error_inst: inst_id,
                block_path: path,
                assignments: result.model,
                nondet_sequence,
            });
        }
    }

    candidates
}

/// Collect the scalar-integer nondet calls executed along `block_path` (in
/// program order) within `func_id`, pairing each with its model value or `0`
/// when unconstrained. Keeping a slot for every nondet call — even unmodeled
/// ones — is what keeps the harness's per-function counter aligned with the
/// program's actual call sequence.
fn resolve_nondet_sequence(
    module: &AirModule,
    func_id: FunctionId,
    block_path: &[BlockId],
    assignments: &BTreeMap<ValueId, i64>,
) -> Vec<NondetCall> {
    let Some(func) = module.function(func_id) else {
        return Vec::new();
    };
    let mut seq = Vec::new();
    for block_id in block_path {
        let Some(block) = func.blocks.iter().find(|b| b.id == *block_id) else {
            continue;
        };
        for inst in &block.instructions {
            let Operation::CallDirect { callee } = &inst.op else {
                continue;
            };
            let Some(name) = module.function(*callee).map(|f| f.name.as_str()) else {
                continue;
            };
            if !is_scalar_integer_nondet(name) {
                continue;
            }
            let value = inst
                .dst
                .and_then(|dst| assignments.get(&dst).copied())
                .unwrap_or(0);
            seq.push(NondetCall {
                func_name: name.to_string(),
                value,
            });
        }
    }
    seq
}

#[cfg(test)]
mod must_reach_tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, BinaryOp, Constant, Instruction};
    use saf_core::ids::ModuleId;

    fn decl(id: FunctionId, name: &str) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn func(id: FunctionId, name: &str, blocks: Vec<AirBlock>, entry: BlockId) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn blk(id: BlockId, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions,
        }
    }

    fn call(iid: u128, callee: FunctionId) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::CallDirect { callee })
    }

    fn ret(iid: u128) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::Ret)
    }

    /// `main() { reach_error(); }` — unconditional → must-reach.
    #[test]
    fn unconditional_direct_is_must_reach() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let entry = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(entry, vec![call(1, err_id), ret(2)])],
            entry,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_some());
    }

    /// `main() { if (x != 0) reach_error(); }` — guarded → NOT must-reach.
    #[test]
    fn guarded_is_not_must_reach() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let (entry, err_blk, exit) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let (x, zero, cond) = (ValueId::new(100), ValueId::new(101), ValueId::new(102));
        let e = blk(
            entry,
            vec![
                Instruction::new(
                    InstId::new(1),
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpNe,
                    },
                )
                .with_operands(vec![x, zero])
                .with_dst(cond),
                Instruction::new(
                    InstId::new(2),
                    Operation::CondBr {
                        then_target: err_blk,
                        else_target: exit,
                    },
                )
                .with_operands(vec![cond]),
            ],
        );
        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(zero, Constant::Int { value: 0, bits: 32 });
        m.functions.push(func(
            main_id,
            "main",
            vec![
                e,
                blk(err_blk, vec![call(3, err_id), ret(4)]),
                blk(exit, vec![ret(5)]),
            ],
            entry,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_none());
    }

    /// `main() { bug(); }  bug() { reach_error(); }` — interprocedural must-reach.
    #[test]
    fn unconditional_interproc_is_must_reach() {
        let (main_id, bug_id, err_id) =
            (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (m_entry, b_entry) = (BlockId::new(10), BlockId::new(20));
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(m_entry, vec![call(1, bug_id), ret(2)])],
            m_entry,
        ));
        m.functions.push(func(
            bug_id,
            "bug",
            vec![blk(b_entry, vec![call(3, err_id), ret(4)])],
            b_entry,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_some());
    }

    /// `f(n){ if(n<3) return; else ERROR: reach_error(); }  main(){ f(2); }` —
    /// the error is behind a guard in the callee → NOT must-reach (afterrec-2 shape).
    #[test]
    fn guarded_callee_is_not_must_reach() {
        let (main_id, f_id, err_id) = (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (m_entry, f_entry, f_ret, f_err) = (
            BlockId::new(10),
            BlockId::new(20),
            BlockId::new(21),
            BlockId::new(22),
        );
        let (n, three, cond) = (ValueId::new(100), ValueId::new(101), ValueId::new(102));
        let fe = blk(
            f_entry,
            vec![
                Instruction::new(
                    InstId::new(3),
                    Operation::BinaryOp {
                        kind: BinaryOp::ICmpSlt,
                    },
                )
                .with_operands(vec![n, three])
                .with_dst(cond),
                Instruction::new(
                    InstId::new(4),
                    Operation::CondBr {
                        then_target: f_ret,
                        else_target: f_err,
                    },
                )
                .with_operands(vec![cond]),
            ],
        );
        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(three, Constant::Int { value: 3, bits: 32 });
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(m_entry, vec![call(1, f_id), ret(2)])],
            m_entry,
        ));
        m.functions.push(func(
            f_id,
            "f",
            vec![
                fe,
                blk(f_ret, vec![ret(5)]),
                blk(f_err, vec![call(6, err_id), ret(7)]),
            ],
            f_entry,
        ));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_none());
    }

    /// The LLVM frontend leaves `entry_block` unset (blocks[0] is the entry), so
    /// must_reach must fall back to the first block. Regression for the slice-1
    /// blind-eval fix (the real IR gave `entry_block = None`).
    #[test]
    fn entry_block_none_uses_first_block() {
        let (main_id, err_id) = (FunctionId::new(1), FunctionId::new(2));
        let entry = BlockId::new(10);
        let mut main = func(
            main_id,
            "main",
            vec![blk(entry, vec![call(1, err_id), ret(2)])],
            entry,
        );
        main.entry_block = None; // mimic the real frontend
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(main);
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_some());
    }

    /// `main() { __VERIFIER_assume(c); reach_error(); }` — the assumption
    /// conditions the path (assume(false) would block it), so the error is NOT
    /// unconditionally reached.
    #[test]
    fn assume_before_error_is_not_must_reach() {
        let (main_id, assume_id, err_id) =
            (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let entry = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(
                entry,
                vec![call(1, assume_id), call(2, err_id), ret(3)],
            )],
            entry,
        ));
        m.functions.push(decl(assume_id, "__VERIFIER_assume"));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_none());
    }

    /// `main() { abort(); reach_error(); }` — a no-return call precedes the error
    /// → NOT must-reach (execution stops at `abort`).
    #[test]
    fn noreturn_before_error_is_not_must_reach() {
        let (main_id, abort_id, err_id) =
            (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let entry = BlockId::new(10);
        let mut m = AirModule::new(ModuleId::new(1));
        m.functions.push(func(
            main_id,
            "main",
            vec![blk(entry, vec![call(1, abort_id), call(2, err_id), ret(3)])],
            entry,
        ));
        m.functions.push(decl(abort_id, "abort"));
        m.functions.push(decl(err_id, "reach_error"));
        assert!(must_reach_error(&m).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_result_accessors() {
        assert!(PropertyResult::True.is_true());
        assert!(!PropertyResult::True.is_false());
        assert!(!PropertyResult::True.is_unknown());

        assert!(PropertyResult::False { witness: None }.is_false());
        assert!(
            PropertyResult::Unknown {
                reason: "test".into()
            }
            .is_unknown()
        );
    }

    #[test]
    fn test_default_config() {
        let config = PropertyAnalysisConfig::default();
        assert_eq!(config.z3_timeout_ms, 5000);
        assert_eq!(config.max_paths, 1000);
        // Conservative mode is the safe default
        assert!(config.conservative);
    }

    #[test]
    fn test_aggressive_config() {
        let config = PropertyAnalysisConfig {
            conservative: false,
            ..Default::default()
        };
        assert!(!config.conservative);
    }
}

#[cfg(test)]
mod enumerate_tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, BinaryOp, Constant, Instruction};
    use saf_core::ids::ModuleId;

    fn decl(id: FunctionId, name: &str) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn func(id: FunctionId, name: &str, blocks: Vec<AirBlock>, entry: BlockId) -> AirFunction {
        AirFunction {
            id,
            name: name.into(),
            params: vec![],
            blocks,
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn blk(id: BlockId, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id,
            label: None,
            instructions,
        }
    }

    fn nondet_call(iid: u128, callee: FunctionId, dst: ValueId) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::CallDirect { callee }).with_dst(dst)
    }

    fn icmp_sgt(iid: u128, lhs: ValueId, rhs: ValueId, dst: ValueId) -> Instruction {
        Instruction::new(
            InstId::new(iid),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpSgt,
            },
        )
        .with_operands(vec![lhs, rhs])
        .with_dst(dst)
    }

    fn condbr(iid: u128, cond: ValueId, then_target: BlockId, else_target: BlockId) -> Instruction {
        Instruction::new(
            InstId::new(iid),
            Operation::CondBr {
                then_target,
                else_target,
            },
        )
        .with_operands(vec![cond])
    }

    fn call(iid: u128, callee: FunctionId) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::CallDirect { callee })
    }

    fn ret(iid: u128) -> Instruction {
        Instruction::new(InstId::new(iid), Operation::Ret)
    }

    /// `int x = nondet_int(); if (x > 5) reach_error();` — the guarded error is
    /// Z3-reachable, so `enumerate_false_candidates` must yield one candidate
    /// carrying the reach_error `InstId`, the block path, a model constraining
    /// `x` to satisfy `x > 5`, and a one-element nondet sequence pinning
    /// `__VERIFIER_nondet_int` to that value.
    #[test]
    fn enumerate_finds_guarded_nondet_candidate() {
        let (main_id, nondet_id, err_id) =
            (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (entry, err_blk, exit) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let (x, five, cond) = (ValueId::new(100), ValueId::new(101), ValueId::new(102));

        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(five, Constant::Int { value: 5, bits: 32 });
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(
                    entry,
                    vec![
                        nondet_call(1, nondet_id, x),
                        icmp_sgt(2, x, five, cond),
                        condbr(3, cond, err_blk, exit),
                    ],
                ),
                blk(err_blk, vec![call(4, err_id), ret(5)]),
                blk(exit, vec![ret(6)]),
            ],
            entry,
        ));
        m.functions.push(decl(nondet_id, "__VERIFIER_nondet_int"));
        m.functions.push(decl(err_id, "reach_error"));

        let candidates = enumerate_false_candidates(&m, &PropertyAnalysisConfig::default());

        assert_eq!(
            candidates.len(),
            1,
            "one guarded error site → one candidate"
        );
        let c = &candidates[0];
        assert_eq!(c.reach_error_inst, InstId::new(4));
        assert_eq!(c.block_path, vec![entry, err_blk]);

        let v = c
            .assignments
            .get(&x)
            .copied()
            .expect("model must constrain the nondet operand x");
        assert!(v > 5, "model value {v} must satisfy the guard x > 5");

        assert_eq!(c.nondet_sequence.len(), 1);
        assert_eq!(c.nondet_sequence[0].func_name, "__VERIFIER_nondet_int");
        assert_eq!(
            c.nondet_sequence[0].value, v,
            "the pinned value must match the model"
        );
    }

    /// Two nondet reads where only the second is constrained by the guard. The
    /// sequence must keep BOTH slots in call order (the first defaults to `0`)
    /// so the harness's per-function counter stays aligned with the program's
    /// actual call sequence.
    #[test]
    fn nondet_sequence_zero_fills_unconstrained_in_call_order() {
        let (main_id, nondet_id, err_id) =
            (FunctionId::new(1), FunctionId::new(2), FunctionId::new(3));
        let (entry, err_blk, exit) = (BlockId::new(10), BlockId::new(11), BlockId::new(12));
        let (a, b, five, cond) = (
            ValueId::new(100),
            ValueId::new(101),
            ValueId::new(102),
            ValueId::new(103),
        );

        let mut m = AirModule::new(ModuleId::new(1));
        m.constants
            .insert(five, Constant::Int { value: 5, bits: 32 });
        m.functions.push(func(
            main_id,
            "main",
            vec![
                blk(
                    entry,
                    vec![
                        nondet_call(1, nondet_id, a), // unconstrained read
                        nondet_call(2, nondet_id, b), // guarded read
                        icmp_sgt(3, b, five, cond),
                        condbr(4, cond, err_blk, exit),
                    ],
                ),
                blk(err_blk, vec![call(5, err_id), ret(6)]),
                blk(exit, vec![ret(7)]),
            ],
            entry,
        ));
        m.functions.push(decl(nondet_id, "__VERIFIER_nondet_int"));
        m.functions.push(decl(err_id, "reach_error"));

        let candidates = enumerate_false_candidates(&m, &PropertyAnalysisConfig::default());
        assert_eq!(candidates.len(), 1);
        let seq = &candidates[0].nondet_sequence;
        assert_eq!(seq.len(), 2, "both nondet reads keep a slot");
        assert_eq!(seq[0].value, 0, "unconstrained first read defaults to 0");
        assert!(seq[1].value > 5, "second read pinned to satisfy x > 5");
    }
}
