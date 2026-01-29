//! SABER-style checker framework using SVFG graph reachability.
//!
//! Provides a declarative, AI-agent-friendly checker framework where checkers
//! are defined as data (`CheckerSpec`), not code. Ships with 9 built-in
//! checkers for memory safety, resource safety, and common bug patterns.
//!
//! # Architecture
//!
//! ```text
//! Layer 4: Path-Sensitive Filter (Z3-based guard feasibility)
//! Layer 3: Built-in Checkers (9 specs)
//! Layer 2: Reachability Solver (may_reach / must_not_reach)
//! Layer 1: Resource Specification (ResourceTable + site classifier)
//! Foundation: Svfg (E12) + PtaResult (E4/E13)
//! ```
//!
//! # Usage (Rust)
//!
//! ```ignore
//! use saf_analysis::checkers::{run_checker, run_checkers, spec, ResourceTable, SolverConfig};
//!
//! // Run a single checker (path-insensitive)
//! let result = run_checker(&spec::memory_leak(), &module, &svfg, &table, &config);
//!
//! // Run all built-in checkers (path-insensitive)
//! let result = run_checkers(&spec::builtin_checkers(), &module, &svfg, &table, &config);
//!
//! // Run with path-sensitive filtering (Z3-based)
//! use saf_analysis::checkers::{run_checkers_path_sensitive, PathSensitiveConfig};
//! let ps_config = PathSensitiveConfig::default();
//! let ps_result = run_checkers_path_sensitive(
//!     &spec::builtin_checkers(), &module, &svfg, &table, &ps_config,
//! );
//! // ps_result.feasible — confirmed findings
//! // ps_result.infeasible — false positives filtered out
//! ```

pub mod finding;
#[cfg(feature = "z3-solver")]
pub mod pathsens;
#[cfg(feature = "z3-solver")]
pub mod pathsens_runner;
pub mod resource_table;
pub mod runner;
pub mod site_classifier;
pub mod solver;
pub mod spec;
pub mod summary;
#[cfg(feature = "z3-solver")]
pub mod z3solver;

// Re-export key types for convenience
pub use finding::{
    CheckerFinding, FindingExport, NullSourceKind, SinkTraceExport, export_findings_json,
    export_findings_sarif,
};
pub use resource_table::{ResourceRole, ResourceTable};
pub use runner::{
    CheckerDiagnostics, CheckerResult, GuardContext, run_checker, run_checkers,
    run_checkers_guarded,
};
pub use site_classifier::{ClassifiedSite, ClassifiedSites, classify};
pub use solver::{GuardedSolverConfig, SolverConfig};
pub use spec::{
    CheckerSpec, ReachabilityMode, Severity, SitePattern, builtin_checker, builtin_checker_names,
    builtin_checkers,
};
pub use summary::{ParameterEffectSummary, compute_parameter_effect_summaries};

// Path-sensitive re-exports (Z3-dependent)
#[cfg(feature = "z3-solver")]
pub use pathsens::{Guard, PathCondition, ValueLocationIndex, extract_guards};
#[cfg(feature = "z3-solver")]
pub use pathsens_runner::{
    PathSensitiveConfig, PathSensitiveDiagnostics, PathSensitiveResult, filter_infeasible,
    filter_multi_reach_infeasible, filter_temporal_infeasible, run_checkers_path_sensitive,
};
#[cfg(feature = "z3-solver")]
pub use z3solver::{FeasibilityResult, PathFeasibilityChecker};
