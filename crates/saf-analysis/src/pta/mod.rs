//! Points-to analysis (PTA) subsystem — context-insensitive algorithms.
//!
//! This module provides flow-insensitive, context-insensitive (CI) pointer
//! analysis algorithms with configurable field sensitivity. It is the
//! foundation on which the context-sensitive PTA ([`crate::cspta`]) and the
//! value-flow graph ([`crate::valueflow`]) are built.
//!
//! # Algorithms
//!
//! - **Andersen** (inclusion-based, [`solve`]/[`solve_with_config`]): The primary
//!   algorithm. Models pointer constraints as subset (inclusion) relations and
//!   iterates a worklist solver to a fixed point. O(n^3) worst case, but the
//!   priority-worklist and indexed constraints keep it practical for programs
//!   with hundreds of thousands of allocation sites.
//! - **Steensgaard** (unification-based, `experimental` feature): Models pointer
//!   constraints as equality (unification) via union-find. O(n*alpha(n)) nearly
//!   linear, but lower precision due to aggressive merging. Best for very large
//!   programs where speed trumps precision.
//! - **Incremental**: SILVA-inspired difference
//!   propagation — only re-propagates changed points-to entries after code edits.
//!   Useful for IDE-style incremental re-analysis.
//!
//! # Constraint Pipeline
//!
//! ```text
//! AIR Module
//!   → extract_constraints() / extract_constraints_reachable()
//!     → ConstraintSet { addr, copy, load, store, gep }
//!       → precompute_indexed_locations()  (field-sensitive locations)
//!         → solve() / solve_with_config()
//!           → PointsToMap (ValueId → BTreeSet<LocId>)
//!             → PtaResult (alias queries, diagnostics)
//! ```
//!
//! Constraint extraction ([`extract`]) walks AIR instructions and generates five
//! constraint kinds: `Addr` (address-of), `Copy` (pointer copy/cast/phi),
//! `Load`, `Store`, and `Gep` (field access). The solver iterates these to a
//! fixed point using a priority worklist ordered by copy-graph topological rank.
//!
//! # Points-to Set Representations
//!
//! The solver is generic over the [`ptsset::PtsSet`] trait, allowing runtime
//! selection of the backing data structure:
//!
//! - [`ptsset::BTreePtsSet`]: `BTreeSet<LocId>` — simple baseline, good for small programs
//! - [`ptsset::RoaringPtsSet`]: Roaring bitmap with shared indexer — fast set operations for
//!   medium programs (10K-100K allocation sites)
//! - [`ptsset::BddPtsSet`]: BDD-backed — compact representation for large programs
//!
//! Selection is automatic via [`ptsset::PtsConfig`] or explicit via
//! [`PtaConfig`].
//!
//! # Callgraph Refinement
//!
//! PTA results feed back into callgraph construction: indirect call targets
//! resolved by PTA are added to the callgraph, which may make new functions
//! reachable. The [`extract_constraints_reachable`] variant filters extraction
//! to only reachable functions, enabling iterative CG+PTA refinement.
//!
//! # Supporting Analyses
//!
//! - **Mod/Ref** ([`mod_ref`]): Computes per-function memory side-effect summaries
//!   (which locations a function may modify or reference).
//! - **Path-sensitive alias** ([`value_origin`]): Refines alias queries using value
//!   origin tracking and branch conditions.
//! - **Object clustering** ([`clustering`]): Groups co-occurring objects for denser
//!   bitmaps in `RoaringPtsSet`.
//!
//! See FR-PTA-001 through FR-PTA-004 for requirements.

mod clustering;
mod config;
mod constraint;
pub mod constraint_index;
mod context;
mod export;
mod extract;
mod func_location;
pub mod hvn;
// TODO(incremental): Remove allow once Task 3.5 wires incremental solver into pipeline.
#[allow(dead_code)]
mod incremental;
mod location;
pub mod mod_ref;
pub mod module_constraints;
pub(crate) mod multiplicity;
// TODO(incremental): Remove allow once summary instantiation is wired into pipeline.
#[cfg(feature = "z3-solver")]
mod path_sensitive;
pub mod ptsset;
mod result;
mod solver;
pub(crate) mod solver_stats;
#[allow(dead_code)]
pub mod summary_constraints;
#[allow(unused_imports)] // Public API for profiling stats
pub use solver_stats::SolverStats;
mod spec_constraints;
#[cfg(feature = "experimental")]
mod steensgaard;
mod value_origin;
#[cfg(feature = "z3-solver")]
mod z3_index;

#[cfg(test)]
mod solver_repr_tests;

#[allow(unused_imports)] // Public API for external use
pub use clustering::{
    ClusteringConfig, ClusteringResult, CooccurrenceMatrix, CoreBitVector,
    approximate_cooccurrence, cluster_objects,
};
pub use config::{FieldSensitivity, IndexSensitivity, PtaConfig};
#[allow(unused_imports)] // Public API when pta module is exposed directly
pub use constraint::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};
#[allow(unused_imports)] // Used internally by solver; public for external use
pub use constraint_index::{ConstraintIndex, IndexedConstraints};
#[allow(unused_imports)] // Public API when pta module is exposed directly
pub use context::{PtaAnalysisResult, PtaContext, PtaDiagnostics, precompute_indexed_locations};
#[allow(unused_imports)] // Public API when pta module is exposed directly
pub use export::{PtaExport, export_constraints};
#[allow(unused_imports)] // Public API when pta module is exposed directly
pub use extract::{
    extract_constraints, extract_constraints_reachable, extract_global_initializers,
    extract_intraprocedural_constraints, extract_module_constraints,
    extract_resolved_indirect_constraints,
};
pub use func_location::FunctionLocationMap;
#[allow(unused_imports)] // Public API for external use
pub use hvn::{HvnResult, hvn_preprocess};
#[allow(unused_imports)] // Public API for external use
pub use incremental::{
    DiffPointsToSet, IncrementalConfig, IncrementalPtaState, IncrementalResult,
    apply_incremental_update, solve_incremental,
};
#[allow(unused_imports)] // Used by cspta::solver and other crate-internal consumers
pub use location::{
    AllocationMultiplicity, CollapseWarning, ConstantsTable, FieldPath, IndexExpr, Location,
    LocationFactory, MemoryRegion, PathStep, merge_gep_with_base_path, resolve_gep_path,
};
#[allow(unused_imports)] // Public API for external use
pub use mod_ref::{
    ModRefSummary, compute_all_mod_ref, compute_all_mod_ref_with_specs, summary_from_spec,
};
#[allow(unused_imports)] // Public API for incremental analysis
pub use module_constraints::{ConstraintDiff, ModuleConstraints, ProgramConstraints};
#[allow(unused_imports)] // Used by pta::context
pub use multiplicity::classify_multiplicity;
#[cfg(feature = "z3-solver")]
pub use path_sensitive::{
    PathSensitiveAliasChecker, PathSensitiveConfig, PathSensitiveDiagnostics,
};
#[allow(unused_imports)] // Public API for external use
pub use ptsset::RoaringPtsSet;
pub use result::{AliasResult, PtaResult};
pub(crate) use solver::GenericSolver;
#[allow(unused_imports)] // Public API for future CG refinement callers
pub(crate) use solver::create_template;
#[allow(unused_imports)] // Public API for external use
pub use solver::solve_with_config;
#[allow(unused_imports)] // Public API for external use
pub use solver::{GenericPointsToMap, normalize_and_expand_hvn, solve_bitvec};
pub use solver::{PointsToMap, solve};
#[allow(unused_imports)] // Public API for external use
pub use spec_constraints::extract_spec_constraints;
#[cfg(feature = "experimental")]
#[allow(unused_imports)] // Public API for external use
pub use steensgaard::{
    NodeId, SteensgaardConfig, SteensgaardDiagnostics, SteensgaardResult, UnionFind,
    solve_steensgaard,
};
#[cfg(feature = "z3-solver")]
#[allow(unused_imports)]
pub use value_origin::filter_pts_for_path;
#[allow(unused_imports)] // Public API for external use
pub use value_origin::{
    PathSensitiveAliasConfig, PathSensitiveAliasResult, ValueOriginMap, build_param_indices,
    path_sensitive_alias, path_sensitive_alias_interprocedural,
    path_sensitive_alias_interprocedural_with_resolved,
};
#[cfg(feature = "z3-solver")]
pub use z3_index::{IndexComparisonResult, Z3IndexChecker, Z3IndexDiagnostics, indices_may_equal};
