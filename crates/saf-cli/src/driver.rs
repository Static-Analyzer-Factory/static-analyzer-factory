//! Analysis driver — orchestrates frontend ingestion, pipeline execution,
//! and result formatting for the `saf run` command.

// Fields for future analysis modes (CSPTA, FSPTA, DDA, Z3, IFDS, etc.)
// are defined now and wired in Phase 3.
#![allow(dead_code)]

use std::fmt::Write as _;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use saf_analysis::absint::{AbstractInterpConfig, NumericFinding};
use saf_analysis::cfg::Cfg;
use saf_analysis::combined::{CombinedAnalysisConfig, CombinedAnalysisResult, analyze_combined};
use saf_analysis::cspta::{
    CsPtaConfig, solve_context_sensitive, solve_context_sensitive_with_resolved,
};
use saf_analysis::database::ProgramDatabase;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::fspta::{FsPtaConfig, FsSvfgBuilder, solve_flow_sensitive};
use saf_analysis::ifds::config::IfdsConfig;
use saf_analysis::ifds::solver::solve_ifds;
use saf_analysis::ifds::taint::{TaintFact, TaintIfdsProblem};
use saf_analysis::ifds::typestate::{TypestateIdeProblem, TypestateSpec, builtin_typestate_spec};
use saf_analysis::mssa::MemorySsa;
use saf_analysis::pipeline::{PipelineConfig, PipelineStats};
use saf_analysis::selector::Selector;
use saf_analysis::svfg::SvfgBuilder;
use saf_analysis::{AliasResult, FieldSensitivity, PtaConfig, PtsConfig, PtsRepresentation};
use saf_analysis::{
    PathSensitiveAliasChecker, PathSensitiveAliasConfig, PathSensitiveConfig,
    PathSensitiveDiagnostics, build_param_indices,
    path_sensitive_alias_interprocedural_with_resolved,
};
use saf_core::air::{AirBundle, AirModule, Operation};
use saf_core::config::{AnalysisMode, Config};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use saf_core::spec::SpecRegistry;

use crate::commands::{CliFrontend, RunArgs};

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// PTA algorithm variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtaVariant {
    /// Andersen's inclusion-based analysis.
    Andersen,
    /// Context-sensitive PTA (k-CFA).
    CsPta,
    /// Flow-sensitive PTA.
    FsPta,
    /// Demand-driven alias analysis.
    Dda,
}

/// PTA solver backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtaSolverKind {
    /// Worklist-based imperative solver.
    Worklist,
    /// Datalog fixpoint solver (Ascent).
    Datalog,
}

/// Which checkers to run.
#[derive(Debug, Clone)]
pub enum CheckerSelection {
    /// Run all registered checkers.
    All,
    /// Run no checkers.
    None,
    /// Run only the named checkers.
    Specific(Vec<String>),
}

/// Z3 solver configuration.
#[derive(Debug, Clone)]
pub struct Z3Config {
    /// Enable assertion proving via Z3.
    pub prove: bool,
    /// Refine alias results via Z3.
    pub refine_alias: bool,
    /// Check path reachability via Z3.
    pub check_reachability: bool,
    /// Z3 solver timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Output format for analysis results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text output.
    Human,
    /// JSON output.
    Json,
    /// SARIF 2.1.0 output.
    Sarif,
}

/// Output configuration.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Output format.
    pub format: OutputFormat,
    /// Output file path (stdout if `None`).
    pub path: Option<PathBuf>,
    /// Show verbose timing and stats.
    pub verbose: bool,
    /// Include checker diagnostics.
    pub diagnostics: bool,
}

/// Points-to set representation selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtsRepr {
    /// Auto-select based on program size.
    Auto,
    /// `BTreeSet` baseline.
    BTreeSet,
    /// `FxHashSet` for fast operations.
    FxHash,
    /// Roaring bitmap.
    Roaring,
    /// Binary Decision Diagram.
    Bdd,
}

/// Full driver configuration derived from CLI arguments.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct DriverConfig {
    /// Analysis mode (fast/precise).
    pub mode: AnalysisMode,
    /// PTA algorithm variant.
    pub pta_variant: PtaVariant,
    /// PTA solver backend.
    pub pta_solver: PtaSolverKind,
    /// k-CFA depth (for `CsPta` only).
    pub pta_k: u32,
    /// Field sensitivity level.
    pub field_sensitivity: CliFieldSensitivityKind,
    /// Maximum PTA solver iterations.
    pub max_pta_iterations: Option<usize>,
    /// Points-to set representation.
    pub pts_repr: PtsRepr,
    /// Which checkers to run.
    pub checkers: CheckerSelection,
    /// Z3 configuration.
    pub z3: Z3Config,
    /// Run combined PTA + abstract interpretation.
    pub combined: bool,
    /// IFDS taint analysis config file.
    pub ifds_taint_config: Option<PathBuf>,
    /// Built-in typestate spec name.
    pub typestate_spec: Option<String>,
    /// Custom typestate spec YAML path.
    pub typestate_custom: Option<PathBuf>,
    /// Enable path-sensitive checker filtering.
    pub path_sensitive: bool,
    /// Spec files/directories path.
    pub specs_path: Option<PathBuf>,
    /// Bench-config JSON path (benchmark mode).
    pub bench_config: Option<PathBuf>,
    /// Output configuration.
    pub output: OutputConfig,

    // ── Bench-mode PTA overrides ──────────────────────────────────────
    // These fields carry `BenchPtaConfig` values that can't be expressed
    // through CLI enum types. They are applied in `build()` after base
    // config setup, overriding the pipeline defaults for benchmark runs.
    /// Override field sensitivity depth (bench mode).
    pub bench_field_depth: Option<u32>,
    /// Enable constant index sensitivity (bench mode).
    pub bench_constant_indices: bool,
    /// Enable Z3 index refinement (bench mode).
    pub bench_z3_index: bool,
    /// Override CG refinement max iterations (bench mode).
    pub bench_refinement_iters: Option<usize>,
}

/// Field sensitivity level for CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliFieldSensitivityKind {
    /// Track struct fields (depth 2).
    StructFields,
    /// Track array indices.
    ArrayIndex,
    /// No field sensitivity.
    Flat,
}

// ---------------------------------------------------------------------------
// DriverConfig construction
// ---------------------------------------------------------------------------

impl DriverConfig {
    /// Build a `DriverConfig` from parsed CLI `RunArgs`.
    #[must_use]
    pub fn from_run_args(args: &RunArgs) -> Self {
        let checkers = match args.checkers.as_str() {
            "all" => CheckerSelection::All,
            "none" => CheckerSelection::None,
            s => CheckerSelection::Specific(s.split(',').map(|c| c.trim().to_string()).collect()),
        };

        Self {
            mode: args.mode.into(),
            pta_variant: args.pta.into(),
            pta_solver: args.solver.into(),
            pta_k: args.pta_k,
            field_sensitivity: args.field_sensitivity.into(),
            max_pta_iterations: args.max_pta_iterations,
            pts_repr: args.pts_repr.into(),
            checkers,
            z3: Z3Config {
                prove: args.z3_prove,
                refine_alias: args.z3_refine_alias,
                check_reachability: args.z3_check_reachability,
                timeout_ms: args.z3_timeout,
            },
            combined: args.combined,
            ifds_taint_config: args.ifds_taint.clone(),
            typestate_spec: args.typestate.clone(),
            typestate_custom: args.typestate_custom.clone(),
            path_sensitive: args.path_sensitive,
            specs_path: args.specs.clone(),
            bench_config: args.bench_config.clone(),
            output: OutputConfig {
                format: args.format.into(),
                path: args.output.clone(),
                verbose: args.verbose,
                diagnostics: args.diagnostics,
            },
            bench_field_depth: None,
            bench_constant_indices: false,
            bench_z3_index: false,
            bench_refinement_iters: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Analysis output
// ---------------------------------------------------------------------------

/// IFDS taint analysis results summary.
pub struct TaintResult {
    /// Number of tainted values at program points.
    pub tainted_count: usize,
    /// Solver diagnostics: iterations performed.
    pub iterations: usize,
    /// Solver diagnostics: path edges explored.
    pub path_edges: usize,
}

/// Typestate analysis results summary.
pub struct TypestateResult {
    /// Spec name.
    pub spec_name: String,
    /// Number of tracked resources.
    pub tracked_count: usize,
    /// Solver diagnostics: iterations performed.
    pub iterations: usize,
}

/// Output from a full analysis run.
pub struct AnalysisOutput {
    /// Pipeline timing statistics.
    pub stats: PipelineStats,
    /// Protocol-level findings (JSON-serializable).
    pub findings: Vec<saf_analysis::database::protocol::Finding>,
    /// Numeric checker findings from abstract interpretation.
    pub numeric_findings: Vec<NumericFinding>,
    /// Context-sensitive PTA results (if CSPTA was run).
    pub cspta_export: Option<serde_json::Value>,
    /// IFDS taint analysis results (if run).
    pub taint_result: Option<TaintResult>,
    /// Typestate analysis results (if run).
    pub typestate_result: Option<TypestateResult>,
    /// Combined analysis results (if run).
    pub combined_result: Option<CombinedAnalysisResult>,
}

// ---------------------------------------------------------------------------
// AnalysisDriver
// ---------------------------------------------------------------------------

/// The main analysis driver. Owns a `ProgramDatabase`.
pub struct AnalysisDriver {
    /// The program database with all precomputed graphs.
    pub db: ProgramDatabase,
}

impl AnalysisDriver {
    /// Ingest input files via the selected frontend, producing an `AirBundle`.
    pub fn ingest(inputs: &[PathBuf], frontend: CliFrontend) -> anyhow::Result<AirBundle> {
        let config = Config::default();
        let paths: Vec<&std::path::Path> = inputs.iter().map(PathBuf::as_path).collect();
        match frontend {
            CliFrontend::Llvm => {
                use saf_frontends::api::Frontend;
                use saf_frontends::llvm::LlvmFrontend;
                let fe = LlvmFrontend::default();
                Ok(fe.ingest(&paths, &config)?)
            }
            CliFrontend::AirJson => {
                use saf_frontends::air_json::AirJsonFrontend;
                use saf_frontends::api::Frontend;
                let fe = AirJsonFrontend;
                Ok(fe.ingest(&paths, &config)?)
            }
        }
    }

    /// Build an `AnalysisDriver` from input files and configuration.
    pub fn build(
        inputs: &[PathBuf],
        config: &DriverConfig,
        frontend: CliFrontend,
    ) -> anyhow::Result<Self> {
        let bundle = Self::ingest(inputs, frontend)?;
        let module = bundle.module;

        let mut pipeline_config = PipelineConfig::from_mode(config.mode);

        // Configure field sensitivity
        pipeline_config.refinement.pta_config.field_sensitivity = match config.field_sensitivity {
            CliFieldSensitivityKind::StructFields => {
                FieldSensitivity::StructFields { max_depth: 2 }
            }
            CliFieldSensitivityKind::ArrayIndex => FieldSensitivity::StructFields { max_depth: 1 },
            CliFieldSensitivityKind::Flat => FieldSensitivity::None,
        };

        // Configure max PTA iterations
        if let Some(max) = config.max_pta_iterations {
            pipeline_config.refinement.pta_config.max_iterations = max;
        }

        // Apply bench-mode PTA overrides (field depth, index sensitivity, Z3)
        if let Some(depth) = config.bench_field_depth {
            pipeline_config.refinement.pta_config.field_sensitivity =
                FieldSensitivity::StructFields { max_depth: depth };
            pipeline_config.refinement.field_sensitivity =
                FieldSensitivity::StructFields { max_depth: depth };
        }
        if config.bench_constant_indices {
            pipeline_config.refinement.pta_config.index_sensitivity =
                saf_analysis::IndexSensitivity::ConstantOnly;
        }
        if config.bench_z3_index {
            pipeline_config.refinement.pta_config.z3_index_enabled = true;
        }
        if let Some(iters) = config.bench_refinement_iters {
            pipeline_config.refinement.max_iterations = iters;
        }

        // Configure PTS representation
        pipeline_config.refinement.pta_config.pts_config = match config.pts_repr {
            PtsRepr::Auto => PtsConfig::default(),
            PtsRepr::BTreeSet => PtsConfig {
                representation: PtsRepresentation::BTreeSet,
                ..PtsConfig::default()
            },
            PtsRepr::FxHash => PtsConfig {
                representation: PtsRepresentation::FxHash,
                ..PtsConfig::default()
            },
            PtsRepr::Roaring => PtsConfig {
                representation: PtsRepresentation::Roaring,
                ..PtsConfig::default()
            },
            PtsRepr::Bdd => PtsConfig {
                representation: PtsRepresentation::Bdd,
                ..PtsConfig::default()
            },
        };

        // Load specs
        let specs = if let Some(ref path) = config.specs_path {
            SpecRegistry::load_from(&[path.clone()])?
        } else {
            SpecRegistry::load()?
        };
        pipeline_config.specs = Some(specs);
        // VFG is built lazily by ProgramDatabase on first access,
        // so skip eager construction in the pipeline to save memory.
        pipeline_config.build_valueflow = false;

        let db = ProgramDatabase::build(module, &pipeline_config);

        Ok(Self { db })
    }

    /// Register `PTABen` oracle wrapper functions in the resource table.
    ///
    /// `PTABen` tests use wrapper functions like `SAFEMALLOC`/`SAFEFREE` around
    /// `malloc`/`free`. Without registering these, the SVFG checker's
    /// `filter_wrapper_internal_sources` can't filter out internal `malloc()`
    /// calls, causing spurious findings and broken alloc-site matching.
    pub fn register_ptaben_wrappers(&mut self) {
        use saf_analysis::checkers::{ResourceRole, ResourceTable};

        let mut table = ResourceTable::new();

        // Memory allocation oracle wrappers (all wrap malloc)
        let alloc_oracles = [
            "SAFEMALLOC",
            "DOUBLEFREEMALLOC",
            "DOUBLEFREEMALLOCFN",
            "SAFEMALLOCFP",
            "PLKMALLOC",
            "NFRMALLOC",
            "CLKMALLOC",
            "NFRLEAKFP",
            "PLKLEAKFP",
            "LEAKFN",
        ];
        for name in alloc_oracles {
            table.add(name, ResourceRole::Allocator);
            table.add(name, ResourceRole::NullSource);
        }

        // Deallocation oracle wrappers (wrap free)
        for name in ["SAFEFREE", "DOUBLEFREE"] {
            table.add(name, ResourceRole::Deallocator);
        }

        self.db.set_resource_table(table);
    }

    /// Run the configured analysis and return findings.
    ///
    /// Executes SVFG checkers, numeric checkers (abstract interpretation),
    /// extended PTA variants (CSPTA, FSPTA, DDA), IFDS taint analysis,
    /// typestate analysis, and combined PTA + abstract interpretation as
    /// configured by the `DriverConfig`.
    // NOTE: This function orchestrates multiple independent analysis passes.
    // Splitting would fragment the unified output assembly.
    #[allow(clippy::too_many_lines)]
    pub fn analyze(&self, config: &DriverConfig) -> anyhow::Result<AnalysisOutput> {
        let stats = self.db.stats().clone();
        let mut all_findings = Vec::new();
        let mut numeric_findings = Vec::new();
        let mut cspta_export = None;
        let mut taint_result = None;
        let mut typestate_result = None;
        let mut combined_result = None;

        // ── SVFG checkers ───────────────────────────────────────────────
        let checker_names = match &config.checkers {
            CheckerSelection::All => Some(vec![]),
            CheckerSelection::Specific(names) => Some(names.clone()),
            CheckerSelection::None => None,
        };
        // Identify which names are SVFG checkers vs numeric checkers.
        // Numeric checker names: "numeric", "buffer-overflow", "integer-overflow",
        // "division-by-zero", "shift-count".
        let numeric_checker_names: &[&str] = &[
            "numeric",
            "buffer-overflow",
            "integer-overflow",
            "division-by-zero",
            "shift-count",
        ];

        let (run_svfg, run_numeric_all, specific_numeric) = match &checker_names {
            None => (false, false, vec![]),
            Some(names) if names.is_empty() => {
                // "all" — run both SVFG and all numeric checkers
                (true, true, vec![])
            }
            Some(names) => {
                let svfg_names: Vec<String> = names
                    .iter()
                    .filter(|n| !numeric_checker_names.contains(&n.as_str()))
                    .cloned()
                    .collect();
                let num_names: Vec<String> = names
                    .iter()
                    .filter(|n| numeric_checker_names.contains(&n.as_str()))
                    .cloned()
                    .collect();
                let has_svfg = !svfg_names.is_empty();
                let has_numeric_all = num_names.iter().any(|n| n == "numeric");
                (has_svfg, has_numeric_all, num_names)
            }
        };

        if run_svfg {
            match &config.checkers {
                CheckerSelection::All => {
                    let resp_json = self
                        .db
                        .handle_request(r#"{"action": "check_all"}"#)
                        .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))?;
                    let resp: saf_analysis::database::protocol::Response =
                        serde_json::from_str(&resp_json)?;
                    if let Some(findings) = resp.findings {
                        all_findings.extend(findings);
                    }
                }
                CheckerSelection::Specific(names) => {
                    for name in names {
                        if numeric_checker_names.contains(&name.as_str()) {
                            continue; // handled below
                        }
                        let req = serde_json::json!({"action": "check", "name": name});
                        let resp_json = self
                            .db
                            .handle_request(&req.to_string())
                            .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))?;
                        let resp: saf_analysis::database::protocol::Response =
                            serde_json::from_str(&resp_json)?;
                        if let Some(findings) = resp.findings {
                            all_findings.extend(findings);
                        }
                    }
                }
                CheckerSelection::None => {}
            }
        }

        // ── Numeric checkers (abstract interpretation) ──────────────────
        if run_numeric_all || !specific_numeric.is_empty() {
            let absint_config = AbstractInterpConfig::default();
            if run_numeric_all || specific_numeric.iter().any(|n| n == "numeric") {
                let result =
                    saf_analysis::absint::check_all_numeric(self.db.module(), &absint_config);
                numeric_findings.extend(result.findings);
            } else {
                // Run specific numeric checkers
                for name in &specific_numeric {
                    let result = match name.as_str() {
                        "buffer-overflow" => saf_analysis::absint::check_buffer_overflow(
                            self.db.module(),
                            &absint_config,
                        ),
                        "shift-count" => saf_analysis::absint::check_shift_count(
                            self.db.module(),
                            &absint_config,
                        ),
                        // integer-overflow and division-by-zero are checked by
                        // `check_all_numeric`; fall back to the full check.
                        _ => saf_analysis::absint::check_all_numeric(
                            self.db.module(),
                            &absint_config,
                        ),
                    };
                    numeric_findings.extend(result.findings);
                }
            }
        }

        // ── Extended PTA variants (CSPTA, FSPTA, DDA) ───────────────────
        match config.pta_variant {
            PtaVariant::Andersen => {} // already run by ProgramDatabase::build()
            PtaVariant::CsPta => {
                let cspta_config = CsPtaConfig {
                    k: config.pta_k,
                    field_sensitivity: match config.field_sensitivity {
                        CliFieldSensitivityKind::StructFields => {
                            FieldSensitivity::StructFields { max_depth: 2 }
                        }
                        CliFieldSensitivityKind::ArrayIndex => {
                            FieldSensitivity::StructFields { max_depth: 1 }
                        }
                        CliFieldSensitivityKind::Flat => FieldSensitivity::None,
                    },
                    max_iterations: config.max_pta_iterations.unwrap_or(100),
                    max_objects: 100_000,
                    pts_config: PtsConfig::default(),
                };
                eprintln!("Running context-sensitive PTA (k={})...", cspta_config.k);
                let result =
                    solve_context_sensitive(self.db.module(), self.db.call_graph(), &cspta_config);
                let export = result.export(&cspta_config);
                cspta_export = Some(serde_json::to_value(&export)?);
                eprintln!(
                    "  CSPTA complete: {} iterations, {} contexts",
                    result.diagnostics().iterations,
                    result.diagnostics().context_count,
                );
            }
            PtaVariant::FsPta => {
                // FSPTA requires the flow-sensitive SVFG (FsSvfg) which is built
                // from MemorySsa. These intermediate structures are not publicly
                // accessible from ProgramDatabase.
                anyhow::bail!(
                    "Flow-sensitive PTA (--pta fspta) requires intermediate SVFG structures \
                     that are not exposed via the CLI driver.\n\
                     Use `saf run --serve` and the JSON protocol to access FSPTA, \
                     or use the Python SDK: `saf.analysis.solve_flow_sensitive()`."
                );
            }
            PtaVariant::Dda => {
                // DDA requires Svfg, MemorySsa, and ModuleIndex which are not
                // publicly accessible from ProgramDatabase.
                anyhow::bail!(
                    "Demand-driven alias analysis (--pta dda) requires SVFG and MemorySsa \
                     structures that are not exposed via the CLI driver.\n\
                     Use `saf run --serve` and the JSON protocol to access DDA, \
                     or use the Python SDK: `saf.analysis.DdaPta()`."
                );
            }
        }

        // ── Z3 path-sensitive features ──────────────────────────────────
        if config.path_sensitive
            || config.z3.prove
            || config.z3.refine_alias
            || config.z3.check_reachability
        {
            anyhow::bail!(
                "Z3-based features (--path-sensitive, --z3-prove, --z3-refine-alias, \
                 --z3-check-reachability) require the `z3-solver` feature flag.\n\
                 Rebuild with: cargo build --features z3-solver"
            );
        }

        // ── IFDS taint analysis ─────────────────────────────────────────
        if let Some(ref taint_path) = config.ifds_taint_config {
            let taint_yaml = std::fs::read_to_string(taint_path).map_err(|e| {
                anyhow::anyhow!("Failed to read taint config {}: {e}", taint_path.display())
            })?;
            let taint_cfg = TaintConfigFile::from_yaml(&taint_yaml)?;

            let sources: Vec<Selector> = taint_cfg
                .sources
                .iter()
                .map(|s| Selector::call_to(s.as_str()))
                .collect();
            let sanitizers: Vec<Selector> = taint_cfg
                .sanitizers
                .iter()
                .map(|s| Selector::call_to(s.as_str()))
                .collect();

            eprintln!(
                "Running IFDS taint analysis ({} sources, {} sanitizers)...",
                sources.len(),
                sanitizers.len()
            );

            let problem = TaintIfdsProblem::new(self.db.module(), &sources, &sanitizers);
            let ifds_config = IfdsConfig::default();
            let result = solve_ifds(&problem, self.db.icfg(), self.db.call_graph(), &ifds_config);

            // Count tainted values across all program points
            let tainted_count = result
                .facts
                .values()
                .flat_map(|facts| facts.iter())
                .filter(|f| matches!(f, TaintFact::Tainted(_)))
                .count();

            eprintln!(
                "  Taint analysis complete: {} tainted values, {} iterations",
                tainted_count, result.diagnostics.iterations
            );

            taint_result = Some(TaintResult {
                tainted_count,
                iterations: result.diagnostics.iterations,
                path_edges: result.diagnostics.path_edges_explored,
            });
        }

        // ── Typestate analysis ──────────────────────────────────────────
        let typestate_spec = if let Some(ref name) = config.typestate_spec {
            Some(builtin_typestate_spec(name).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown built-in typestate spec: '{name}'. \
                     Available: file_io, mutex_lock, memory_alloc"
                )
            })?)
        } else if let Some(ref path) = config.typestate_custom {
            let yaml = std::fs::read_to_string(path).map_err(|e| {
                anyhow::anyhow!("Failed to read typestate spec {}: {e}", path.display())
            })?;
            let spec = parse_typestate_yaml(&yaml)
                .map_err(|e| anyhow::anyhow!("Failed to parse typestate spec YAML: {e}"))?;
            Some(spec)
        } else {
            None
        };

        if let Some(spec) = typestate_spec {
            let spec_name = spec.name.clone();
            eprintln!("Running typestate analysis (spec: {spec_name})...");

            let problem = TypestateIdeProblem::new(self.db.module(), spec);
            let ifds_config = IfdsConfig::default();
            let result = saf_analysis::ifds::ide_solver::solve_ide(
                &problem,
                self.db.icfg(),
                self.db.call_graph(),
                &ifds_config,
            );

            let tracked_count = result
                .ifds_result
                .facts
                .values()
                .flat_map(|facts| facts.iter())
                .filter(|f| matches!(f, saf_analysis::ifds::typestate::TypestateFact::Tracked(_)))
                .count();

            eprintln!(
                "  Typestate complete: {} tracked resources, {} iterations",
                tracked_count, result.ifds_result.diagnostics.iterations
            );

            typestate_result = Some(TypestateResult {
                spec_name,
                tracked_count,
                iterations: result.ifds_result.diagnostics.iterations,
            });
        }

        // ── Combined PTA + abstract interpretation ──────────────────────
        if config.combined {
            eprintln!("Running combined PTA + abstract interpretation...");
            let combined_config = CombinedAnalysisConfig {
                pta: PtaConfig::default(),
                absint: AbstractInterpConfig::default(),
                enable_refinement: true,
                max_refinement_iterations: 3,
                context_sensitive_indirect: false,
            };
            let result = analyze_combined(self.db.module(), &combined_config);
            eprintln!(
                "  Combined analysis complete: {} refinement iterations, {} summaries",
                result.refinement_iterations,
                result.summaries.len()
            );
            combined_result = Some(result);
        }

        Ok(AnalysisOutput {
            stats,
            findings: all_findings,
            numeric_findings,
            cspta_export,
            taint_result,
            typestate_result,
            combined_result,
        })
    }

    /// Format and write analysis output.
    ///
    /// The `db` parameter provides access to the `DisplayResolver` for
    /// enriching human-readable and SARIF output with resolved names and
    /// source locations.
    pub fn format_output(
        output: &AnalysisOutput,
        config: &OutputConfig,
        db: &ProgramDatabase,
    ) -> anyhow::Result<()> {
        let text = match config.format {
            OutputFormat::Human => format_human(output, config.verbose, db),
            OutputFormat::Json => {
                let mut json = serde_json::json!({
                    "stats": output.stats,
                    "findings": output.findings,
                });
                if !output.numeric_findings.is_empty() {
                    json["numeric_findings"] = serde_json::to_value(&output.numeric_findings)?;
                }
                if let Some(ref cspta) = output.cspta_export {
                    json["cspta"] = cspta.clone();
                }
                if let Some(ref taint) = output.taint_result {
                    json["taint"] = serde_json::json!({
                        "tainted_count": taint.tainted_count,
                        "iterations": taint.iterations,
                        "path_edges": taint.path_edges,
                    });
                }
                if let Some(ref ts) = output.typestate_result {
                    json["typestate"] = serde_json::json!({
                        "spec": ts.spec_name,
                        "tracked_count": ts.tracked_count,
                        "iterations": ts.iterations,
                    });
                }
                if let Some(ref combined) = output.combined_result {
                    json["combined"] = serde_json::json!({
                        "refinement_iterations": combined.refinement_iterations,
                        "summaries": combined.summaries.len(),
                    });
                }
                serde_json::to_string_pretty(&json)?
            }
            OutputFormat::Sarif => format_sarif(output)?,
        };

        if let Some(ref path) = config.path {
            std::fs::write(path, &text)?;
            eprintln!("Output written to {}", path.display());
        } else {
            print!("{text}");
        }
        Ok(())
    }

    /// Format analysis output as a SARIF 2.1.0 JSON string.
    ///
    /// This is the public entry point for SARIF formatting, used by both
    /// `saf run --format sarif` and `saf export findings --format sarif`.
    pub fn format_sarif_string(output: &AnalysisOutput) -> anyhow::Result<String> {
        format_sarif(output)
    }

    /// Run the JSON protocol server on stdin/stdout.
    pub fn serve(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        eprintln!("SAF JSON protocol server ready. Send JSON requests, one per line.");

        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let response = self.db.handle_request(trimmed).unwrap_or_else(|e| {
                format!(r#"{{"status":"error","error":{{"code":"INTERNAL","message":"{e}"}}}}"#)
            });
            writeln!(stdout, "{response}")?;
            stdout.flush()?;
        }

        Ok(())
    }

    /// Run in benchmark mode: execute analyses specified by `BenchConfig`
    /// and return structured `BenchResult`.
    // NOTE: This function orchestrates multiple independent analysis passes
    // driven by the bench config. Splitting would fragment the unified result assembly.
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::similar_names
    )]
    pub fn run_bench_mode(
        &mut self,
        bench_config: &saf_cli::bench_types::BenchConfig,
    ) -> anyhow::Result<saf_cli::bench_types::BenchResult> {
        use saf_cli::bench_types::{
            AliasResultEntry, AssertionResultEntry, BenchAnalysisStats, BenchBufferFinding,
            BenchCheckerFinding, BenchInterleavingEntry, BenchIrStats, BenchMtaResults,
            BenchResult, BenchStats, BenchTctEntry, BenchThreadContextEntry, IntervalResultEntry,
            NullnessResultEntry,
        };

        let stats = self.db.stats();
        let mut result = BenchResult {
            success: true,
            error: None,
            stats: BenchStats {
                total_secs: stats.total_secs,
                pta_solve_secs: stats.pta_solve_secs,
                refinement_iterations: stats.refinement_iterations as u32,
                defuse_build_secs: stats.defuse_build_secs,
                valueflow_build_secs: stats.valueflow_build_secs,
                cspta_secs: None,
                mssa_svfg_secs: None,
                fspta_secs: None,
                frontend_secs: 0.0,
                cfg_build_secs: None,
                pta_clone_secs: None,
                defuse_local_secs: None,
            },
            ..Default::default()
        };

        // ── Populate IR stats ───────────────────────────────────────────
        {
            let module = self.db.module();
            let functions = module
                .functions
                .iter()
                .filter(|f| !f.is_declaration)
                .count();
            let instructions: usize = module
                .functions
                .iter()
                .filter(|f| !f.is_declaration)
                .map(|f| f.blocks.iter().map(|b| b.instructions.len()).sum::<usize>())
                .sum();
            let globals = module.globals.len();
            result.ir_stats = Some(BenchIrStats {
                functions,
                instructions,
                globals,
            });
        }

        // ── Populate analysis stats ─────────────────────────────────────
        {
            let cc = stats.constraint_counts;
            let phc = stats.post_hvn_constraint_counts;

            let mut bench_stats = BenchAnalysisStats {
                addr_constraints: cc[0],
                copy_constraints: cc[1],
                load_constraints: cc[2],
                store_constraints: cc[3],
                gep_constraints: cc[4],
                post_hvn_total_constraints: phc.iter().sum(),
                ..Default::default()
            };

            // PTA metrics
            if let Some(pta) = self.db.pta_result() {
                bench_stats.pta_pointers = pta.value_count();
                bench_stats.obj_count = pta.obj_count();
                bench_stats.field_location_count = pta.location_count();
                bench_stats.solve_iterations = pta.diagnostics().iterations;
                bench_stats.total_pointer_values = self.db.module().pointer_value_count();

                // PTS size statistics
                let pts_map = pta.points_to_map();
                if !pts_map.is_empty() {
                    let mut total_pts: usize = 0;
                    let mut max_pts: usize = 0;
                    for pts_set in pts_map.values() {
                        let sz = pts_set.len();
                        total_pts += sz;
                        if sz > max_pts {
                            max_pts = sz;
                        }
                    }
                    bench_stats.max_pts_size = max_pts;
                    #[allow(clippy::cast_precision_loss)]
                    {
                        bench_stats.avg_pts_size = total_pts as f64 / pts_map.len() as f64;
                        if bench_stats.total_pointer_values > 0 {
                            bench_stats.avg_pts_size_svf =
                                total_pts as f64 / bench_stats.total_pointer_values as f64;
                        }
                    }
                }
            }

            // Call graph metrics
            {
                let cg = self.db.call_graph();
                bench_stats.cg_nodes = cg.nodes.iter().filter(|n| !n.is_indirect()).count();
                // Unique (caller_fn, callee_fn) pairs
                let mut unique_pairs = std::collections::BTreeSet::new();
                for (caller, callees) in &cg.edges {
                    if let Some(caller_fid) = caller.function_id() {
                        for callee in callees {
                            if let Some(callee_fid) = callee.function_id() {
                                unique_pairs.insert((caller_fid, callee_fid));
                            }
                        }
                    }
                }
                bench_stats.cg_edges = unique_pairs.len();
                bench_stats.cg_callsite_edges = cg.call_sites.len();
                bench_stats.indirect_calls_resolved = self.db.resolved_sites().len();
                bench_stats.ind_call_sites = self
                    .db
                    .module()
                    .functions
                    .iter()
                    .filter(|f| !f.is_declaration)
                    .flat_map(|f| f.blocks.iter().flat_map(|bb| &bb.instructions))
                    .filter(|inst| matches!(inst.op, Operation::CallIndirect { .. }))
                    .count();
            }

            result.analysis_stats = Some(bench_stats);
        }

        // ── 0. Optional higher-precision PTA solves ─────────────────────
        // Run CS-PTA / FS-PTA when requested, independently of alias queries.
        // Results are reused by alias queries below when present.
        let cspta_result = if bench_config.analyses.cspta {
            let t0 = std::time::Instant::now();
            let cspta_config = CsPtaConfig {
                k: 2,
                field_sensitivity: FieldSensitivity::StructFields {
                    max_depth: bench_config.pta_config.field_depth,
                },
                max_iterations: bench_config.pta_config.max_iterations,
                max_objects: 200_000,
                pts_config: PtsConfig::default(),
            };
            let r = solve_context_sensitive_with_resolved(
                self.db.module(),
                self.db.call_graph(),
                &cspta_config,
                self.db.resolved_sites(),
            );
            result.stats.cspta_secs = Some(t0.elapsed().as_secs_f64());
            Some(r)
        } else {
            None
        };

        let (fspta_result, fs_load_pts) = if bench_config.analyses.fspta {
            if let Some(pta) = self.db.pta_result() {
                let t0 = std::time::Instant::now();
                let tcfg = std::time::Instant::now();
                let cfgs: std::collections::BTreeMap<saf_core::ids::FunctionId, Cfg> = self
                    .db
                    .module()
                    .functions
                    .iter()
                    .filter(|f| !f.is_declaration)
                    .map(|f| (f.id, Cfg::build(f)))
                    .collect();
                result.stats.cfg_build_secs = Some(tcfg.elapsed().as_secs_f64());

                let tclone = std::time::Instant::now();
                let mssa_pta = pta.clone();
                result.stats.pta_clone_secs = Some(tclone.elapsed().as_secs_f64());

                let mut mssa =
                    MemorySsa::build(self.db.module(), &cfgs, mssa_pta, self.db.call_graph());
                // CFGs no longer needed after MSSA build
                drop(cfgs);

                // Build defuse locally so it can be dropped after SVFG
                // construction — avoids retaining it in memory during the
                // FS-PTA solve (saves hundreds of MB on large programs).
                let tdefuse = std::time::Instant::now();
                let defuse = DefUseGraph::build(self.db.module());
                result.stats.defuse_local_secs = Some(tdefuse.elapsed().as_secs_f64());

                let (svfg, program_points) = SvfgBuilder::new(
                    self.db.module(),
                    &defuse,
                    self.db.call_graph(),
                    pta,
                    &mut mssa,
                )
                .build();
                // Defuse and program points not needed after SVFG construction
                drop(defuse);
                drop(program_points);

                let fs_svfg = FsSvfgBuilder::new(
                    self.db.module(),
                    &svfg,
                    pta,
                    &mut mssa,
                    self.db.call_graph(),
                )
                .build();
                let mssa_svfg_secs = t0.elapsed().as_secs_f64();
                result.stats.mssa_svfg_secs = Some(mssa_svfg_secs);
                // Donate SVFG to ProgramDatabase so checkers can reuse it
                // instead of rebuilding MSSA+SVFG from scratch.
                self.db.set_svfg(svfg);
                drop(mssa);

                let t1 = std::time::Instant::now();
                let fs_config = FsPtaConfig {
                    skip_df_materialization: bench_config.analyses.fspta_skip_df,
                    ..FsPtaConfig::default()
                };
                let fs_result = solve_flow_sensitive(
                    self.db.module(),
                    &fs_svfg,
                    pta,
                    self.db.call_graph(),
                    &fs_config,
                );
                let load_pts = fs_result.compute_load_sensitive_pts(&fs_svfg);
                result.stats.fspta_secs = Some(t1.elapsed().as_secs_f64());
                // FS-SVFG no longer needed after solve
                drop(fs_svfg);
                (Some(fs_result), load_pts)
            } else {
                (None, std::collections::BTreeMap::new())
            }
        } else {
            (None, std::collections::BTreeMap::new())
        };

        // ── 1. Alias queries ──────────────────────────────────────────────
        if !bench_config.alias_queries.is_empty() {
            for query in &bench_config.alias_queries {
                let ptr_a = bench_parse_value_id(&query.ptr_a)?;
                let ptr_b = bench_parse_value_id(&query.ptr_b)?;

                // CI-PTA alias query
                let ci = self.db.may_alias(ptr_a, ptr_b);
                let ci_str = format_alias_result(ci);

                // Debug: capture CI-PTA points-to set sizes and uniqueness
                let (ci_pts_a_size, ci_pts_b_size, ci_unique) = {
                    if let Some(pta) = self.db.pta_result() {
                        let a_pts = pta.points_to(ptr_a);
                        let b_pts = pta.points_to(ptr_b);
                        let a_sz = a_pts.len();
                        let b_sz = b_pts.len();
                        let unique = if a_sz == 1 && b_sz == 1 {
                            let a_loc = *a_pts.first().unwrap();
                            Some(pta.is_unique(a_loc))
                        } else {
                            None
                        };
                        (Some(a_sz), Some(b_sz), unique)
                    } else {
                        (None, None, None)
                    }
                };

                // CS-PTA alias query (if run)
                let cs_str = cspta_result
                    .as_ref()
                    .map(|cs| format_alias_result(cs.may_alias_any(ptr_a, ptr_b)));

                // FS-PTA alias query (if run) — prefers load-sensitive PTS
                let (fs_str, fs_pts_a_size, fs_pts_b_size) = if ptr_a == ptr_b {
                    // Self-alias: same SSA value always aliases with itself
                    (Some("MustAlias".to_string()), None, None)
                } else if let Some(fs) = fspta_result.as_ref() {
                    // Prefer load-sensitive PTS when available, fall back to global
                    let p_pts: Option<std::collections::BTreeSet<saf_core::ids::LocId>> =
                        fs_load_pts.get(&ptr_a).cloned().or_else(|| {
                            let pts = fs.points_to(ptr_a);
                            if pts.is_empty() {
                                None
                            } else {
                                Some(pts.iter().copied().collect())
                            }
                        });
                    let q_pts: Option<std::collections::BTreeSet<saf_core::ids::LocId>> =
                        fs_load_pts.get(&ptr_b).cloned().or_else(|| {
                            let pts = fs.points_to(ptr_b);
                            if pts.is_empty() {
                                None
                            } else {
                                Some(pts.iter().copied().collect())
                            }
                        });
                    let a_sz = p_pts.as_ref().map(std::collections::BTreeSet::len);
                    let b_sz = q_pts.as_ref().map(std::collections::BTreeSet::len);
                    let result = match (p_pts, q_pts) {
                        (Some(p), Some(q)) => {
                            if p.is_disjoint(&q) {
                                "NoAlias".to_string()
                            } else if p.len() == 1 && p == q {
                                "MustAlias".to_string()
                            } else if p == q {
                                "MayAlias".to_string()
                            } else if p.is_subset(&q) || q.is_subset(&p) {
                                "PartialAlias".to_string()
                            } else {
                                "MayAlias".to_string()
                            }
                        }
                        _ => "Unknown".to_string(),
                    };
                    (Some(result), a_sz, b_sz)
                } else {
                    (None, None, None)
                };

                // ── Path-sensitive refinement ──────────────────────────
                let ps_results = try_path_sensitive_refinement(
                    self.db.module(),
                    self.db.call_graph(),
                    &self.db,
                    self.db.resolved_sites(),
                    ptr_a,
                    ptr_b,
                    query.oracle_function.as_deref(),
                    query.oracle_block.as_deref(),
                );

                // Pick best (most precise non-Unknown) result
                let best = pick_best_alias(
                    &ci_str,
                    cs_str.as_deref(),
                    fs_str.as_deref(),
                    ps_results.combined.as_deref(),
                );

                result.alias_results.push(AliasResultEntry {
                    ptr_a: query.ptr_a.clone(),
                    ptr_b: query.ptr_b.clone(),
                    ci: ci_str,
                    cs: cs_str,
                    fs: fs_str,
                    ps: ps_results.combined,
                    ps_perpath: ps_results.perpath,
                    ps_callsite: ps_results.callsite,
                    ps_guard: ps_results.guard,
                    ps_dead_code: ps_results.dead_code,
                    best,
                    ci_pts_a_size,
                    ci_pts_b_size,
                    ci_unique,
                    fs_pts_a_size,
                    fs_pts_b_size,
                });
            }
        }

        // Compute reachable functions from entry points for filtering
        // findings from dead code (e.g., Juliet harness utility functions).
        // This is sound as long as the call graph is complete — unreachable
        // functions cannot execute, so their findings are guaranteed false positives.
        let reachable_funcs = {
            let module = self.db.module();
            let entry_fids: Vec<FunctionId> = module
                .functions
                .iter()
                .filter(|f| f.name == "main" || f.name == "_main")
                .map(|f| f.id)
                .collect();
            if entry_fids.is_empty() {
                None // No main → skip filtering (analyze everything)
            } else {
                Some(self.db.cg_reachable_from(&entry_fids))
            }
        };

        // Build `ValueId` → `FunctionId` map for reachability check.
        let value_to_func: std::collections::BTreeMap<ValueId, FunctionId> =
            if reachable_funcs.is_some() {
                let module = self.db.module();
                let mut map = std::collections::BTreeMap::new();
                for func in &module.functions {
                    if func.is_declaration {
                        continue;
                    }
                    for param in &func.params {
                        map.insert(param.id, func.id);
                    }
                    for block in &func.blocks {
                        for inst in &block.instructions {
                            if let Some(dst) = inst.dst {
                                map.insert(dst, func.id);
                            }
                            for &operand in &inst.operands {
                                map.entry(operand).or_insert(func.id);
                            }
                        }
                    }
                }
                map
            } else {
                std::collections::BTreeMap::new()
            };

        // ── 2. Checker findings ───────────────────────────────────────────
        if bench_config.analyses.checkers {
            {
                let resp_json = self
                    .db
                    .handle_request(r#"{"action":"check_all"}"#)
                    .map_err(|e| anyhow::anyhow!("JSON serialization error: {e}"))?;
                let resp: saf_analysis::database::protocol::Response =
                    serde_json::from_str(&resp_json)?;
                if let Some(findings) = resp.findings {
                    for f in &findings {
                        let alloc_site = f.object.clone().unwrap_or_default();

                        // Skip findings from functions not reachable from main().
                        if let Some(ref reachable) = reachable_funcs {
                            let in_reachable = parse_id_from_hex::<ValueId>(&alloc_site)
                                .is_some_and(|vid| {
                                    value_to_func
                                        .get(&vid)
                                        .is_some_and(|fid| reachable.contains(fid))
                                });
                            if !in_reachable {
                                continue;
                            }
                        }

                        result.checker_findings.push(BenchCheckerFinding {
                            check: f.check.clone(),
                            alloc_site,
                            severity: f.severity.clone(),
                            call_sites: f.path.iter().map(|e| e.location.clone()).collect(),
                        });
                    }
                }
            }
        }

        // ── 3. Assertion proving (Z3/absint) ──────────────────────────────
        if bench_config.analyses.z3_prove {
            if let Some(pta) = self.db.pta_result() {
                let absint_config = saf_analysis::absint::AbstractInterpConfig {
                    narrowing_iterations: 5,
                    ..saf_analysis::absint::AbstractInterpConfig::default()
                };
                let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();
                let interp_result = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
                    self.db.module(),
                    &absint_config,
                    pta,
                    Some(&specs),
                );

                let cond_result = saf_analysis::z3_utils::prove_conditions_interprocedural(
                    self.db.module(),
                    &interp_result,
                    "svf_assert",
                );

                for finding in &cond_result.proven {
                    result.assertion_results.push(AssertionResultEntry {
                        call_site: finding.inst.to_hex(),
                        kind: "svf_assert".to_string(),
                        proved: true,
                        status: "proven".to_string(),
                    });
                }
                for finding in &cond_result.may_fail {
                    result.assertion_results.push(AssertionResultEntry {
                        call_site: finding.inst.to_hex(),
                        kind: "svf_assert".to_string(),
                        proved: false,
                        status: "may_fail".to_string(),
                    });
                }
                for finding in &cond_result.unknown {
                    result.assertion_results.push(AssertionResultEntry {
                        call_site: finding.inst.to_hex(),
                        kind: "svf_assert".to_string(),
                        proved: false,
                        status: "unknown".to_string(),
                    });
                }
            }
        }

        // ── 4. Interval queries (assert_eq) ──────────────────────────────
        if !bench_config.interval_queries.is_empty() {
            if let Some(pta) = self.db.pta_result() {
                use saf_analysis::absint::AbstractDomain as _;

                let absint_config = saf_analysis::absint::AbstractInterpConfig {
                    narrowing_iterations: 5,
                    ..saf_analysis::absint::AbstractInterpConfig::default()
                };
                let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();
                let interp_result = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
                    self.db.module(),
                    &absint_config,
                    pta,
                    Some(&specs),
                );

                for iq in &bench_config.interval_queries {
                    let call_site = bench_parse_inst_id(&iq.call_site)?;
                    let left_val = bench_parse_value_id(&iq.left_value)?;
                    let right_val = bench_parse_value_id(&iq.right_value)?;

                    let left_iv = interp_result.interval_at_inst(call_site, left_val, 32);
                    let right_iv = interp_result.interval_at_inst(call_site, right_val, 32);

                    let overlap = !left_iv.is_bottom()
                        && !right_iv.is_bottom()
                        && left_iv.lo() <= right_iv.hi()
                        && right_iv.lo() <= left_iv.hi();

                    result.interval_results.push(IntervalResultEntry {
                        call_site: iq.call_site.clone(),
                        left: iq.left_value.clone(),
                        right: iq.right_value.clone(),
                        left_interval: format!("[{}, {}]", left_iv.lo(), left_iv.hi()),
                        right_interval: format!("[{}, {}]", right_iv.lo(), right_iv.hi()),
                        overlap,
                    });
                }
            }
        }

        // ── 5. Nullness analysis ──────────────────────────────────────────
        if bench_config.analyses.nullness && !bench_config.nullness_queries.is_empty() {
            if let Some(pta) = self.db.pta_result() {
                let pta_integration = saf_analysis::absint::PtaIntegration::new(pta);
                let absint_config = saf_analysis::absint::AbstractInterpConfig {
                    narrowing_iterations: 5,
                    ..saf_analysis::absint::AbstractInterpConfig::default()
                };
                let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();
                let interp_result = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
                    self.db.module(),
                    &absint_config,
                    pta,
                    Some(&specs),
                );
                let nullness_config = saf_analysis::absint::nullness::NullnessConfig::default();
                let nullness_result =
                    saf_analysis::absint::analyze_nullness_with_pta_specs_and_summaries(
                        self.db.module(),
                        &nullness_config,
                        &pta_integration,
                        &specs,
                        interp_result.summaries(),
                    );

                for query in &bench_config.nullness_queries {
                    let ptr = bench_parse_value_id(&query.ptr)?;
                    let call_site = bench_parse_inst_id(&query.call_site)?;
                    let nullness = nullness_result.nullness_at(call_site, ptr);
                    let may_null = matches!(
                        nullness,
                        saf_analysis::absint::nullness::Nullness::Null
                            | saf_analysis::absint::nullness::Nullness::MaybeNull
                    );
                    result.nullness_results.push(NullnessResultEntry {
                        ptr: query.ptr.clone(),
                        call_site: query.call_site.clone(),
                        may_null,
                    });
                }
            }
        }

        // ── 6. Buffer overflow analysis ───────────────────────────────────
        if bench_config.analyses.buffer_overflow {
            // Compute reachable function names for buffer finding filtering.
            let reachable_func_names: Option<std::collections::BTreeSet<String>> =
                reachable_funcs.as_ref().map(|reachable| {
                    let module = self.db.module();
                    module
                        .functions
                        .iter()
                        .filter(|f| reachable.contains(&f.id))
                        .map(|f| f.name.clone())
                        .collect()
                });

            if let Some(pta) = self.db.pta_result() {
                let pta_integration = saf_analysis::absint::PtaIntegration::new(pta);
                let absint_config = saf_analysis::absint::AbstractInterpConfig {
                    narrowing_iterations: 5,
                    ..saf_analysis::absint::AbstractInterpConfig::default()
                };
                let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();

                // Helper: check if a buffer finding is in a reachable function.
                let is_reachable_buf = |f: &saf_analysis::absint::NumericFinding| -> bool {
                    reachable_func_names
                        .as_ref()
                        .is_none_or(|names| names.contains(&f.function))
                };

                // Buffer overflow check
                let buf_result = saf_analysis::absint::check_buffer_overflow_with_pta(
                    self.db.module(),
                    &absint_config,
                    &pta_integration,
                );
                for f in &buf_result.findings {
                    if !is_reachable_buf(f) {
                        continue;
                    }
                    result.buffer_findings.push(BenchBufferFinding {
                        ptr: f
                            .affected_ptr
                            .map_or_else(String::new, saf_core::ids::ValueId::to_hex),
                        function: f.function.clone(),
                        kind: format!("{:?}", f.checker),
                        description: f.description.clone(),
                    });
                }

                // Memcpy overflow check
                let memcpy_result = saf_analysis::absint::check_memcpy_overflow_with_pta_and_specs(
                    self.db.module(),
                    &absint_config,
                    &pta_integration,
                    &specs,
                );
                for f in &memcpy_result.findings {
                    if !is_reachable_buf(f) {
                        continue;
                    }
                    result.buffer_findings.push(BenchBufferFinding {
                        ptr: f
                            .affected_ptr
                            .map_or_else(String::new, saf_core::ids::ValueId::to_hex),
                        function: f.function.clone(),
                        kind: format!("{:?}", f.checker),
                        description: f.description.clone(),
                    });
                }

                // Non-PTA memcpy overflow check (catches cases where PTA has no info)
                let memcpy_specs_result = saf_analysis::absint::check_memcpy_overflow_with_specs(
                    self.db.module(),
                    &absint_config,
                    &specs,
                );
                for f in &memcpy_specs_result.findings {
                    if !is_reachable_buf(f) {
                        continue;
                    }
                    // Deduplicate: skip if we already have a finding for same function+description
                    let already_found = result.buffer_findings.iter().any(|existing| {
                        existing.function == f.function && existing.description == f.description
                    });
                    if !already_found {
                        result.buffer_findings.push(BenchBufferFinding {
                            ptr: f
                                .affected_ptr
                                .map_or_else(String::new, saf_core::ids::ValueId::to_hex),
                            function: f.function.clone(),
                            kind: format!("{:?}", f.checker),
                            description: f.description.clone(),
                        });
                    }
                }
            }
        }

        // ── 7. MTA (multi-thread analysis) ────────────────────────────────
        if bench_config.analyses.mta {
            if let Some(pta) = self.db.pta_result() {
                use saf_analysis::mta::ThreadId;

                let mta_config = saf_analysis::mta::MtaConfig::default();
                let mta = saf_analysis::mta::MtaAnalysis::with_pta(
                    self.db.module(),
                    self.db.call_graph(),
                    self.db.icfg(),
                    mta_config,
                    pta.points_to_map(),
                    pta.location_factory(),
                );
                let mta_result = mta.analyze();

                // Thread context existence
                let mut thread_contexts = Vec::new();
                for tid in mta_result.threads().keys() {
                    thread_contexts.push(BenchThreadContextEntry {
                        thread_id: u64::from(tid.0),
                        exists: true,
                    });
                }

                // Interleaving queries: check concurrent threads at each call site
                let mut interleaving = Vec::new();
                for query in &bench_config.interleaving_queries {
                    let tid = ThreadId::new(
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            query.thread_id as u32
                        },
                    );
                    if mta_result.threads().get(&tid).is_none() {
                        continue;
                    }
                    let Ok(call_site) = bench_parse_inst_id(&query.call_site) else {
                        continue;
                    };
                    let concurrent = mta_result.concurrent_threads_at(tid, call_site);
                    interleaving.push(BenchInterleavingEntry {
                        thread_id: query.thread_id,
                        call_site: query.call_site.clone(),
                        interleaved: concurrent.contains(&tid),
                    });
                }

                // TCT access queries: check concurrent_with for thread graph
                let mut tct_access = Vec::new();
                for query in &bench_config.tct_queries {
                    let tid = ThreadId::new(
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            query.thread_id as u32
                        },
                    );
                    if mta_result.threads().get(&tid).is_none() {
                        continue;
                    }
                    let concurrent = mta_result.thread_graph.concurrent_with(tid);
                    tct_access.push(BenchTctEntry {
                        thread_id: query.thread_id,
                        call_site: query.call_site.clone(),
                        accessible: !concurrent.is_empty() || tid.0 == 0,
                    });
                }

                result.mta_results = Some(BenchMtaResults {
                    interleaving,
                    thread_contexts,
                    tct_access,
                });
            }
        }

        // Capture peak RSS of this analysis process (not the wrapper).
        result.peak_rss_mb = bench_peak_rss_mb();

        Ok(result)
    }
}

/// Read peak RSS (`VmHWM`) from `/proc/self/status` (Linux only).
/// Returns megabytes, or 0 on non-Linux.
fn bench_peak_rss_mb() -> usize {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("VmHWM:") {
                    let kb: usize = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb / 1024;
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

// ---------------------------------------------------------------------------
// Bench-mode helpers
// ---------------------------------------------------------------------------

/// Parse a hex string (with or without `0x` prefix) into a `ValueId`.
fn bench_parse_value_id(s: &str) -> anyhow::Result<saf_core::ids::ValueId> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    let raw = u128::from_str_radix(hex_str, 16)?;
    Ok(saf_core::ids::ValueId::new(raw))
}

/// Parse a hex string (with or without `0x` prefix) into an `InstId`.
fn bench_parse_inst_id(s: &str) -> anyhow::Result<saf_core::ids::InstId> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    let raw = u128::from_str_radix(hex_str, 16)?;
    Ok(saf_core::ids::InstId::new(raw))
}

/// Format an `AliasResult` as a human-readable string for bench output.
fn format_alias_result(r: saf_analysis::AliasResult) -> String {
    match r {
        saf_analysis::AliasResult::Must => "MustAlias".to_string(),
        saf_analysis::AliasResult::Partial => "PartialAlias".to_string(),
        saf_analysis::AliasResult::May => "MayAlias".to_string(),
        saf_analysis::AliasResult::No => "NoAlias".to_string(),
        saf_analysis::AliasResult::Unknown => "Unknown".to_string(),
    }
}

/// Results from all path-sensitive refinement strategies.
struct PsRefinementResults {
    /// Combined best result (for backwards compatibility).
    combined: Option<String>,
    /// Strategy 1: Per-path interprocedural refinement.
    perpath: Option<String>,
    /// Strategy 2: Callsite argument mapping.
    callsite: Option<String>,
    /// Strategy 3: Guard-based path-sensitive.
    guard: Option<String>,
    /// True if the oracle function has no direct callers (only CHA-resolved
    /// indirect callers). The oracle is vacuously correct (dead code).
    dead_code: bool,
}

/// Run all path-sensitive refinement strategies for an alias query.
///
/// Tries three strategies and returns ALL results independently:
/// 1. Per-path interprocedural refinement (enumerates branch combinations)
/// 2. Callsite refinement (maps params to caller args and re-queries CI-PTA)
/// 3. Guard-based refinement (Z3 path conditions, if available)
///
/// Unlike the old short-circuiting approach, all strategies are tried so the
/// validator can pick the best one with knowledge of the expected result.
// NOTE: This function implements the path-sensitive refinement pipeline as a
// single unit matching the old in-process PTABen code.
// NOTE: This function runs all three path-sensitive strategies independently,
// returning separate results for each. The validator uses expected-kind filtering
// to pick the first non-unsound result. Splitting would lose the strategy context.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn try_path_sensitive_refinement(
    module: &AirModule,
    _call_graph: &saf_analysis::callgraph::CallGraph,
    db: &ProgramDatabase,
    resolved_sites: &std::collections::BTreeMap<InstId, Vec<FunctionId>>,
    ptr_a: ValueId,
    ptr_b: ValueId,
    oracle_function_hex: Option<&str>,
    oracle_block_hex: Option<&str>,
) -> PsRefinementResults {
    let empty = PsRefinementResults {
        combined: None,
        perpath: None,
        callsite: None,
        guard: None,
        dead_code: false,
    };

    let Some(func_id) = oracle_function_hex.and_then(parse_id_from_hex::<FunctionId>) else {
        return empty;
    };

    let Some(func) = module.functions.iter().find(|f| f.id == func_id) else {
        return empty;
    };

    let mut perpath_result: Option<String> = None;
    let mut callsite_result: Option<String> = None;
    let mut guard_result: Option<String> = None;

    // Dead code detection (unconditional, before any PS strategy).
    // If the oracle function has no direct (CallDirect) callers — only
    // CHA-resolved CallIndirect — the function is effectively dead code.
    // CHA over-resolves virtual calls, so the function may appear in the
    // call graph but never actually be dispatched to at runtime.  The
    // oracle is vacuously correct.  This check MUST run before Strategy 1
    // because `path_sensitive_alias_interprocedural_with_resolved` may
    // return a spurious result when resolved_sites contain CHA targets
    // that don't actually call this function.
    //
    // Exception: `main` (and `_start`, `__libc_start_main`) are entry
    // points called by the runtime — they have no in-module callers but
    // are NOT dead code.
    let is_entry_point = matches!(func.name.as_str(), "main" | "_start" | "__libc_start_main");
    let has_direct_caller = is_entry_point
        || module.functions.iter().any(|caller| {
            !caller.is_declaration
                && caller.blocks.iter().any(|block| {
                    block.instructions.iter().any(|inst| {
                        matches!(
                            &inst.op,
                            Operation::CallDirect { callee }
                                if *callee == func_id
                        )
                    })
                })
        });
    let dead_code = !has_direct_caller;

    // Strategy 1: Per-path interprocedural refinement (skip for dead code)
    if !dead_code {
        let ps_config = PathSensitiveAliasConfig::default();
        let ps_result = path_sensitive_alias_interprocedural_with_resolved(
            ptr_a,
            ptr_b,
            func,
            module,
            &ps_config,
            resolved_sites,
        );
        if ps_result.alias != AliasResult::Unknown {
            perpath_result = Some(format_alias_result(ps_result.alias));
        }
    }

    // Strategy 2: Callsite refinement — map params to caller args, re-query CI-PTA
    let param_indices = build_param_indices(func);
    let idx_a = param_indices.get(&ptr_a);
    let idx_b = param_indices.get(&ptr_b);
    if let (Some(&ia), Some(&ib)) = (idx_a, idx_b) {
        // Iterate all call sites targeting this function.
        // Use first-non-Unknown semantics (matches old code behavior):
        // return immediately when ANY call site produces a result.
        for caller_func in &module.functions {
            if callsite_result.is_some() {
                break;
            }
            for block in &caller_func.blocks {
                if callsite_result.is_some() {
                    break;
                }
                for inst in &block.instructions {
                    let targets_func = match &inst.op {
                        Operation::CallDirect { callee } => *callee == func_id,
                        Operation::CallIndirect { .. } => resolved_sites
                            .get(&inst.id)
                            .is_some_and(|targets| targets.contains(&func_id)),
                        _ => false,
                    };
                    if targets_func {
                        // SAF AIR uses callee-LAST convention for CallIndirect:
                        // operands = [arg0, arg1, ..., argN, callee_ptr]
                        // So arguments are at operands[idx] directly, but we must
                        // cap the index to exclude the trailing callee pointer.
                        let arg_count = if matches!(inst.op, Operation::CallIndirect { .. }) {
                            inst.operands.len().saturating_sub(1)
                        } else {
                            inst.operands.len()
                        };
                        let mapped_a = if ia < arg_count {
                            Some(inst.operands[ia])
                        } else {
                            None
                        };
                        let mapped_b = if ib < arg_count {
                            Some(inst.operands[ib])
                        } else {
                            None
                        };
                        if let (Some(ma), Some(mb)) = (mapped_a, mapped_b) {
                            let caller_alias = db.may_alias(ma, mb);
                            if caller_alias != AliasResult::Unknown {
                                callsite_result = Some(format_alias_result(caller_alias));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Strategy 3: Guard-based refinement (requires PTA result)
    if let Some(block_hex) = oracle_block_hex {
        if let (Some(block_id), Some(pta)) =
            (parse_id_from_hex::<BlockId>(block_hex), db.pta_result())
        {
            let mut diag = PathSensitiveDiagnostics::default();
            let guard_config = PathSensitiveConfig::default();
            let mut checker = PathSensitiveAliasChecker::new(
                pta.points_to_map(),
                pta.locations(),
                module,
                guard_config,
            );
            let guard_alias = checker.may_alias_at(ptr_a, ptr_b, block_id, func_id, &mut diag);
            if guard_alias != AliasResult::Unknown {
                guard_result = Some(format_alias_result(guard_alias));
            }
        }
    }

    // Combined: pick the first non-None result in strategy order (for backwards compat)
    let combined = perpath_result
        .clone()
        .or_else(|| callsite_result.clone())
        .or_else(|| guard_result.clone());

    PsRefinementResults {
        combined,
        perpath: perpath_result,
        callsite: callsite_result,
        guard: guard_result,
        dead_code,
    }
}

/// Numeric precision of an alias result for comparison.
fn alias_precision(r: AliasResult) -> u8 {
    match r {
        AliasResult::No => 5,
        AliasResult::Must => 4,
        AliasResult::Partial => 3,
        AliasResult::May => 2,
        AliasResult::Unknown => 0,
    }
}

/// Parse a hex ID string into a specific ID type.
fn parse_id_from_hex<T: From<u128>>(hex: &str) -> Option<T> {
    let hex_str = hex.strip_prefix("0x").unwrap_or(hex);
    let raw = u128::from_str_radix(hex_str, 16).ok()?;
    Some(T::from(raw))
}

/// Pick the best (most precise non-Unknown) alias result across analysis levels.
fn pick_best_alias(ci: &str, cs: Option<&str>, fs: Option<&str>, ps: Option<&str>) -> String {
    // Precision order: NoAlias > MustAlias > PartialAlias > MayAlias > Unknown
    fn precision(s: &str) -> u8 {
        match s {
            "NoAlias" => 5,
            "MustAlias" => 4,
            "PartialAlias" => 3,
            "MayAlias" => 2,
            "Unknown" => 0,
            _ => 1,
        }
    }

    let mut best = ci.to_string();
    let mut best_prec = precision(ci);

    for candidate in [cs, fs, ps].into_iter().flatten() {
        let p = precision(candidate);
        if p > best_prec {
            best = candidate.to_string();
            best_prec = p;
        }
    }

    best
}

// ---------------------------------------------------------------------------
// Taint config file schema
// ---------------------------------------------------------------------------

/// Parsed IFDS taint analysis configuration from YAML.
///
/// Expected YAML format:
/// ```yaml
/// sources:
///   - getenv
///   - read
/// sanitizers:
///   - sanitize_input
/// sinks:
///   - system
///   - execve
/// ```
struct TaintConfigFile {
    /// Function names whose return values are taint sources.
    sources: Vec<String>,
    /// Function names whose calls kill taint (sanitizers).
    sanitizers: Vec<String>,
    /// Function names whose arguments are taint sinks (for reporting).
    sinks: Vec<String>,
}

impl TaintConfigFile {
    /// Parse a `TaintConfigFile` from a YAML string.
    fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        let value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
        let map = value
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("Taint config must be a YAML mapping"))?;

        let extract_string_list = |key: &str| -> Vec<String> {
            map.get(serde_yaml::Value::String(key.to_string()))
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(Self {
            sources: extract_string_list("sources"),
            sanitizers: extract_string_list("sanitizers"),
            sinks: extract_string_list("sinks"),
        })
    }
}

/// Parse a `TypestateSpec` from a YAML string.
///
/// Expected YAML format:
/// ```yaml
/// name: file_io
/// states: [uninit, opened, closed, error]
/// initial_state: opened
/// error_states: [error]
/// accepting_states: [closed, uninit]
/// constructors: [fopen]
/// transitions:
///   - { from: opened, call: fclose, to: closed }
///   - { from: opened, call: fread, to: opened }
///   - { from: closed, call: fclose, to: error }
/// ```
fn parse_typestate_yaml(yaml: &str) -> anyhow::Result<TypestateSpec> {
    use saf_analysis::ifds::typestate::TypestateTransition;

    let value: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    let map = value
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("Typestate spec must be a YAML mapping"))?;

    let get_string = |key: &str| -> anyhow::Result<String> {
        map.get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("Missing required field: {key}"))
    };

    let get_string_list = |key: &str| -> Vec<String> {
        map.get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    };

    let transitions_raw = map
        .get(serde_yaml::Value::String("transitions".to_string()))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| {
                    let m = v.as_mapping()?;
                    let from = m
                        .get(serde_yaml::Value::String("from".to_string()))?
                        .as_str()?
                        .to_string();
                    let call = m
                        .get(serde_yaml::Value::String("call".to_string()))?
                        .as_str()?
                        .to_string();
                    let to = m
                        .get(serde_yaml::Value::String("to".to_string()))?
                        .as_str()?
                        .to_string();
                    Some(TypestateTransition { from, call, to })
                })
                .collect()
        })
        .unwrap_or_default();

    let spec = TypestateSpec {
        name: get_string("name")?,
        states: get_string_list("states"),
        initial_state: get_string("initial_state")?,
        error_states: get_string_list("error_states"),
        accepting_states: get_string_list("accepting_states"),
        transitions: transitions_raw,
        constructors: get_string_list("constructors"),
    };

    // Validate the spec
    if let Err(e) = spec.validate() {
        anyhow::bail!("Invalid typestate spec: {e}");
    }

    Ok(spec)
}

// ---------------------------------------------------------------------------
// Human-readable formatting
// ---------------------------------------------------------------------------

/// Format analysis output as human-readable text.
/// Format analysis output as human-readable text with resolved names.
///
/// Uses the `DisplayResolver` from the `ProgramDatabase` to enrich findings
/// with human-readable names and source locations alongside hex IDs.
// NOTE: This function formats multiple finding types (SVFG, numeric, CSPTA,
// taint, typestate, combined) as a single cohesive text report.
#[allow(clippy::too_many_lines)]
fn format_human(output: &AnalysisOutput, verbose: bool, _db: &ProgramDatabase) -> String {
    let mut buf = String::new();

    if verbose {
        buf.push_str("=== Pipeline Statistics ===\n");

        writeln!(
            buf,
            "  Def-use build:       {:.3}s",
            output.stats.defuse_build_secs
        )
        .unwrap();

        writeln!(
            buf,
            "  PTA solve:           {:.3}s",
            output.stats.pta_solve_secs
        )
        .unwrap();

        writeln!(
            buf,
            "  CG refinement iters: {}",
            output.stats.refinement_iterations
        )
        .unwrap();

        writeln!(
            buf,
            "  Value-flow build:    {:.3}s",
            output.stats.valueflow_build_secs
        )
        .unwrap();

        writeln!(
            buf,
            "  Total:               {:.3}s",
            output.stats.total_secs
        )
        .unwrap();
        buf.push('\n');
    }

    // SVFG checker findings
    if output.findings.is_empty() {
        buf.push_str("No SVFG checker findings.\n");
    } else {
        writeln!(
            buf,
            "=== {} SVFG Checker Finding(s) ===",
            output.findings.len()
        )
        .unwrap();
        for finding in &output.findings {
            // Header: [SEVERITY] check-name (CWE-NNN)

            writeln!(
                buf,
                "\n[{}] {} ({})",
                finding.severity.to_uppercase(),
                finding.check,
                finding
                    .cwe
                    .map_or_else(|| "no CWE".to_string(), |c| format!("CWE-{c}"))
            )
            .unwrap();

            // Enriched message: include display_name if available
            if let Some(ref name) = finding.display_name {
                writeln!(buf, "  {}: {}", finding.check, name).unwrap();
            }
            writeln!(buf, "  {}", finding.message).unwrap();
            if let Some(ref obj) = finding.object {
                writeln!(buf, "  Object: {obj}").unwrap();
            }

            // Path events with enriched names and source locations
            for event in &finding.path {
                let name_suffix = event
                    .display_name
                    .as_deref()
                    .map_or(String::new(), |n| format!(" '{n}'"));
                let loc_suffix = event.source_loc.as_ref().map_or(String::new(), |loc| {
                    format!(" at {}:{}:{}", loc.file, loc.line, loc.col)
                });

                writeln!(
                    buf,
                    "  -> {}{}{} ({})",
                    event.location, name_suffix, loc_suffix, event.event
                )
                .unwrap();
            }
        }
    }

    // Numeric checker findings
    if !output.numeric_findings.is_empty() {
        writeln!(
            buf,
            "\n=== {} Numeric Finding(s) ===",
            output.numeric_findings.len()
        )
        .unwrap();
        for finding in &output.numeric_findings {
            writeln!(
                buf,
                "\n[{:?}] {:?} (CWE-{})",
                finding.severity, finding.checker, finding.cwe
            )
            .unwrap();
            writeln!(buf, "  {}", finding.description).unwrap();

            writeln!(
                buf,
                "  Function: {}, Interval: {}",
                finding.function, finding.interval
            )
            .unwrap();
        }
    }

    // CSPTA results
    if output.cspta_export.is_some() {
        buf.push_str("\n=== Context-Sensitive PTA ===\n");
        buf.push_str("  CSPTA results included (see JSON output for details).\n");
    }

    // IFDS taint results
    if let Some(ref taint) = output.taint_result {
        buf.push_str("\n=== IFDS Taint Analysis ===\n");
        writeln!(buf, "  Tainted values:  {}", taint.tainted_count).unwrap();
        writeln!(buf, "  Iterations:      {}", taint.iterations).unwrap();
        writeln!(buf, "  Path edges:      {}", taint.path_edges).unwrap();
    }

    // Typestate results
    if let Some(ref ts) = output.typestate_result {
        buf.push_str("\n=== Typestate Analysis ===\n");
        writeln!(buf, "  Spec:            {}", ts.spec_name).unwrap();
        writeln!(buf, "  Tracked:         {}", ts.tracked_count).unwrap();
        writeln!(buf, "  Iterations:      {}", ts.iterations).unwrap();
    }

    // Combined analysis results
    if let Some(ref combined) = output.combined_result {
        buf.push_str("\n=== Combined PTA + Abstract Interpretation ===\n");

        writeln!(
            buf,
            "  Refinement iterations: {}",
            combined.refinement_iterations
        )
        .unwrap();

        writeln!(buf, "  Function summaries:    {}", combined.summaries.len()).unwrap();
    }

    buf
}

/// Format analysis output as SARIF 2.1.0 JSON.
///
/// Populates `physicalLocation` with source file, line, and column from
/// the enriched `PathEvent.source_loc` fields, and uses resolved display
/// names in finding messages.
// NOTE: SARIF construction involves iterating findings, path events, and
// building nested JSON structures. Splitting would fragment the format logic.
#[allow(clippy::too_many_lines)]
fn format_sarif(output: &AnalysisOutput) -> anyhow::Result<String> {
    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut rules: Vec<serde_json::Value> = Vec::new();
    let mut seen_rules = std::collections::BTreeSet::new();

    // Convert protocol findings to SARIF results
    for finding in &output.findings {
        // Register the rule if not already seen
        if seen_rules.insert(finding.check.clone()) {
            let mut rule = serde_json::json!({
                "id": finding.check,
                "shortDescription": { "text": finding.check },
                "fullDescription": { "text": finding.message },
            });
            if let Some(cwe) = finding.cwe {
                rule["properties"] = serde_json::json!({
                    "cwe": [format!("CWE-{cwe}")]
                });
            }
            rules.push(rule);
        }

        // Build SARIF locations from path events
        let mut locations: Vec<serde_json::Value> = Vec::new();
        let mut code_flows: Vec<serde_json::Value> = Vec::new();
        let mut thread_flow_locs: Vec<serde_json::Value> = Vec::new();

        for event in &finding.path {
            let mut loc_json = serde_json::json!({});

            if let Some(ref src) = event.source_loc {
                let mut physical = serde_json::json!({
                    "artifactLocation": { "uri": src.file },
                    "region": {
                        "startLine": src.line,
                        "startColumn": src.col,
                    }
                });
                if let Some(el) = src.end_line {
                    physical["region"]["endLine"] = serde_json::json!(el);
                }
                if let Some(ec) = src.end_col {
                    physical["region"]["endColumn"] = serde_json::json!(ec);
                }
                loc_json["physicalLocation"] = physical;
            }

            // Build the message from display_name and event description
            let msg = if let Some(ref name) = event.display_name {
                format!("{}: {}", event.event, name)
            } else {
                format!("{}: {}", event.event, event.location)
            };
            loc_json["message"] = serde_json::json!({ "text": msg });

            // First event location becomes the primary location
            if locations.is_empty() {
                locations.push(loc_json.clone());
            }

            // All events contribute to the code flow
            thread_flow_locs.push(serde_json::json!({
                "location": loc_json,
            }));
        }

        if !thread_flow_locs.is_empty() {
            code_flows.push(serde_json::json!({
                "threadFlows": [{
                    "locations": thread_flow_locs,
                }]
            }));
        }

        // Build the enriched message
        let message_text = if let Some(ref name) = finding.display_name {
            format!("{}: {}", finding.check, name)
        } else {
            finding.message.clone()
        };

        let level = match finding.severity.as_str() {
            "error" | "critical" => "error",
            "info" => "note",
            // "warning" and any unknown severity map to SARIF "warning"
            _ => "warning",
        };

        let mut result_json = serde_json::json!({
            "ruleId": finding.check,
            "level": level,
            "message": { "text": message_text },
        });

        if !locations.is_empty() {
            result_json["locations"] = serde_json::json!(locations);
        }
        if !code_flows.is_empty() {
            result_json["codeFlows"] = serde_json::json!(code_flows);
        }
        if let Some(cwe) = finding.cwe {
            result_json["properties"] = serde_json::json!({
                "cwe": [format!("CWE-{cwe}")]
            });
        }

        results.push(result_json);
    }

    // Also include numeric findings in SARIF
    for finding in &output.numeric_findings {
        let rule_id = format!("{:?}", finding.checker).to_lowercase();
        if seen_rules.insert(rule_id.clone()) {
            rules.push(serde_json::json!({
                "id": rule_id,
                "shortDescription": { "text": format!("{:?}", finding.checker) },
                "fullDescription": { "text": finding.description },
                "properties": { "cwe": [format!("CWE-{}", finding.cwe)] },
            }));
        }

        let level = finding.severity.name();
        let sarif_level = match level {
            "error" => "error",
            "warning" => "warning",
            _ => "note",
        };

        results.push(serde_json::json!({
            "ruleId": rule_id,
            "level": sarif_level,
            "message": { "text": finding.description },
        }));
    }

    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "SAF",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/Static-Analyzer-Factory/static-analyzer-factory",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    });

    Ok(serde_json::to_string_pretty(&sarif)?)
}
