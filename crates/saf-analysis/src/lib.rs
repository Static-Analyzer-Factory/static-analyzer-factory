// Documentation lints — kept at crate level for now, tracked as future work in FUTURE.md.
// These require extensive changes across all doc comments and would be disruptive.
#![allow(clippy::doc_markdown)] // Requires adding backticks to type names in doc comments
#![allow(clippy::missing_errors_doc)] // Requires adding # Errors sections to all pub fns
#![allow(clippy::missing_panics_doc)] // Requires adding # Panics sections to all pub fns

pub mod callgraph;
pub mod cfg;
pub mod cg_refinement;
pub mod cha;
pub mod defuse;

// Human-readable display resolution for AIR entity IDs.
pub mod display;
pub use display::{DisplayResolver, EntityKind, HumanLabel, SourceLoc, short_hex};

pub mod error;
pub mod export;
pub mod graph_algo;
pub mod icfg;
pub mod module_index;
pub use module_index::ModuleIndex;
pub mod points_to_query;
pub use points_to_query::PointsToQuery;

// PTA module — internal types (constraint.rs, clustering.rs) are not
// exposed. Only user-facing types are re-exported at the crate root.
mod pta;
pub use pta::{
    AddrConstraint, AliasResult, ConstraintSet, CopyConstraint, FieldPath, FieldSensitivity,
    GepConstraint, HvnResult, IndexExpr, IndexSensitivity, LoadConstraint, Location,
    LocationFactory, PathSensitiveAliasConfig, PathSensitiveAliasResult, PathStep, PointsToMap,
    PtaAnalysisResult, PtaConfig, PtaContext, PtaDiagnostics, PtaResult, StoreConstraint,
    ValueOriginMap, build_param_indices, classify_multiplicity, export_constraints,
    extract_constraints, extract_intraprocedural_constraints,
    extract_resolved_indirect_constraints, extract_spec_constraints, hvn_preprocess,
    merge_gep_with_base_path, path_sensitive_alias, path_sensitive_alias_interprocedural,
    path_sensitive_alias_interprocedural_with_resolved, precompute_indexed_locations,
    ptsset::{PtsConfig, PtsRepresentation},
    resolve_gep_path, solve as pta_solve,
};
#[cfg(feature = "z3-solver")]
pub use pta::{
    IndexComparisonResult, PathSensitiveAliasChecker, PathSensitiveConfig,
    PathSensitiveDiagnostics, Z3IndexChecker, Z3IndexDiagnostics, indices_may_equal,
};

// ValueFlow module - explicit re-exports for API stability (avoid `pub use *`)
mod valueflow;
#[cfg(feature = "z3-solver")]
pub use valueflow::taint_z3;
pub use valueflow::{
    EXPORT_SCHEMA_VERSION, EdgeKind, EnrichedStep, EnrichedTrace, ExportedConfig, ExportedFinding,
    Finding, FindingId, Flow, IncludeLocations, NodeId, NodeInfo, OpKind, QueryLimits, SarifExport,
    SpanInfo, Trace, TraceStep, TransformPropagation, ValueFlowBuilder, ValueFlowConfig,
    ValueFlowDiagnostics, ValueFlowExport, ValueFlowGraph, ValueFlowMode, build_valueflow,
    to_property_graph,
};
#[cfg(feature = "z3-solver")]
pub use valueflow::{TaintFlowZ3Result, filter_taint_flows_z3};

// Selector module for taint source/sink/sanitizer specification
pub mod selector;

// IFDS framework module
pub mod ifds;

// Memory SSA module
pub mod mssa;

// Sparse Value-Flow Graph module
pub mod svfg;

// Flow-sensitive PTA module
pub mod fspta;

// Checker framework module
pub mod checkers;

// Abstract interpretation framework
pub mod absint;

// Context-sensitive pointer analysis (k-CFA)
pub mod cspta;

// Guard extraction from branch conditions (Z3-independent)
pub mod guard;

// Shared Z3 SMT solver utilities
#[cfg(feature = "z3-solver")]
pub mod z3_utils;

// Demand-driven pointer analysis (DDA)
pub mod dda;

// Multi-Threaded Analysis (MTA) - concurrency analysis
#[cfg(feature = "analysis-mta")]
pub mod mta;

// Counter-Example Guided Abstraction Refinement (CEGAR)
#[cfg(feature = "analysis-cegar")]
pub mod cegar;

// Loop invariant synthesis
#[cfg(feature = "analysis-invariants")]
pub mod invariants;

// Combined PTA + Abstract Interpretation analysis
pub mod combined;

// Incremental analysis session state
pub mod session;

// Cascading invalidation controller for incremental analysis
pub mod invalidation;

// Composable analysis pass framework
pub mod pass;
pub use pass::{AnalysisContext, AnalysisPass, PassId, PassManager};

// Concrete pass implementations wrapping pipeline stages
pub mod passes;

// Bottom-up function summary generation
pub mod summary_gen;

// Unified analysis pipeline
pub mod pipeline;

// WASM-compatible timer utilities
pub(crate) mod timer;

// Program database — graph-first analysis framework
pub mod database;

#[cfg(test)]
mod proptest_arb;

#[cfg(test)]
mod proptest_tests;
