//! Z3-based path feasibility checking.
//!
//! This module re-exports types from `z3_utils::solver` for backward
//! compatibility with the E18 checker framework. New code should import
//! directly from `z3_utils`.

pub use crate::z3_utils::solver::{FeasibilityResult, PathFeasibilityChecker, Z3FilterDiagnostics};
