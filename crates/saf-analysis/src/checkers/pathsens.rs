//! Path-sensitive checker reachability via guard extraction and Z3 feasibility checking.
//!
//! This module re-exports types and functions from `z3_utils` for backward
//! compatibility with the E18 checker framework. New code should import
//! directly from `z3_utils`.

// Re-export all guard types and functions from the guard module
pub use crate::guard::{
    ConditionInfo, Guard, OperandInfo, PathCondition, TerminatorInfo, ValueLocationIndex,
    extract_guards, extract_guards_from_blocks, is_icmp, resolve_operand,
};

// Re-export interprocedural guard propagation types
pub use crate::z3_utils::interprocedural::{CallerGuardContext, augment_with_caller_guards};
