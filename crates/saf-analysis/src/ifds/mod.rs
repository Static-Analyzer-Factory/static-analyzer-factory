//! IFDS/IDE (Interprocedural Finite Distributive Subset / Interprocedural
//! Distributive Environment) framework.
//!
//! Provides generic solvers for interprocedural data-flow problems:
//! - **IFDS** (Reps, Horwitz, Sagiv, POPL'95): tracks which facts hold.
//! - **IDE** (Sagiv, Reps, Horwitz, TCS'96): extends IFDS with a value lattice,
//!   tracking what value each fact has via edge functions.
//!
//! # Usage
//!
//! 1. Implement [`IfdsProblem`] for IFDS, or [`IdeProblem`] for IDE.
//! 2. Call [`solve_ifds`] or [`solve_ide`] with the problem, ICFG, call graph, and config.
//! 3. Query the [`IfdsResult`] or [`IdeResult`] for facts/values at program points.
//!
//! # Built-in Clients
//!
//! - [`TaintIfdsProblem`] — taint analysis via IFDS (source/sink/sanitizer).
//! - [`TypestateIdeProblem`] — typestate analysis via IDE (file I/O, mutex, memory).

pub mod config;
pub mod edge_fn;
pub mod export;
pub mod icfg_index;
pub mod ide_problem;
pub mod ide_result;
pub mod ide_solver;
pub mod lattice;
pub mod problem;
pub mod result;
pub mod solver;
pub mod taint;
#[cfg(feature = "z3-solver")]
pub mod taint_z3;
pub mod typestate;
#[cfg(feature = "z3-solver")]
pub mod typestate_z3;

pub use config::IfdsConfig;
pub use edge_fn::BuiltinEdgeFn;
pub use export::IfdsExport;
pub use icfg_index::IcfgIndex;
pub use ide_problem::IdeProblem;
pub use ide_result::{IdeDiagnostics, IdeResult};
pub use ide_solver::solve_ide;
pub use lattice::Lattice;
pub use problem::IfdsProblem;
pub use result::{IfdsDiagnostics, IfdsResult};
pub use solver::solve_ifds;
pub use taint::{TaintFact, TaintIfdsProblem};
#[cfg(feature = "z3-solver")]
pub use taint_z3::{
    TaintWitnessPath, TaintZ3Result, filter_taint_paths_z3, reconstruct_taint_paths,
};
pub use typestate::{
    TypestateFact, TypestateFinding, TypestateFindingKind, TypestateIdeProblem, TypestateLattice,
    TypestateSpec, TypestateTransition, builtin_typestate_spec,
};

/// Simple glob-style name matching (supports `*` prefix/suffix).
///
/// Used by taint analysis and typestate analysis to match function names
/// against selector patterns and transition triggers.
///
/// # Patterns
///
/// - `"*"` matches any name.
/// - `"prefix*"` matches names starting with `prefix`.
/// - `"*suffix"` matches names ending with `suffix`.
/// - `"exact"` matches only the exact name.
pub(crate) fn matches_name(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return name.ends_with(suffix);
    }
    name == pattern
}
