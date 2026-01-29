//! Shared Z3 SMT solver utilities for path-sensitive analysis.
//!
//! This module extracts reusable Z3 infrastructure from E18's checker
//! framework for use across all analysis pillars (IFDS, ValueFlow,
//! typestate, numeric, assertions, alias, path-reachability).
//!
//! # Architecture
//!
//! ```text
//! z3_utils/
//!   solver.rs     — Z3Solver, FeasibilityResult, PathFeasibilityChecker
//!   guard.rs      — Guard, PathCondition, OperandInfo, ConditionInfo,
//!                   ValueLocationIndex, extract_guards()
//!   dominator.rs  — compute_dominators(), extract_dominating_guards()
//! ```

pub mod alias;
pub mod assertions;
pub mod condition_prover;
pub mod dominator;
pub mod interprocedural;
pub mod reachability;
pub mod solver;

// Re-export guard types from the top-level guard module (Z3-independent)
pub use crate::guard;

// Re-export key types for convenience
pub use crate::guard::{
    ConditionInfo, Guard, OperandInfo, PathCondition, TerminatorInfo, ValueLocationIndex,
    extract_guards, extract_guards_from_blocks, is_icmp, resolve_operand,
};
pub use alias::{AliasRefinement, AliasRefinementResult, refine_alias};
pub use assertions::{
    AssertionDiagnostics, AssertionFinding, AssertionResult, AssertionStatus,
    DEFAULT_ASSERT_FUNCTIONS, prove_assertions,
};
pub use condition_prover::{
    ConditionDiagnostics, ConditionFinding, ConditionResult, ConditionStatus, IntervalQuery,
    prove_conditions, prove_conditions_interprocedural,
};
pub use dominator::{compute_dominators, extract_dominating_guards};
pub use interprocedural::{CallerGuardContext, augment_with_caller_guards};
pub use reachability::{PathReachability, PathReachabilityResult, check_path_reachable};
pub use solver::{FeasibilityResult, PathFeasibilityChecker, Z3FilterDiagnostics};
