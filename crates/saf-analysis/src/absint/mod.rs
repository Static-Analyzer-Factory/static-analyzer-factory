//! Abstract interpretation framework for numeric value analysis.
//!
//! Provides an [`AbstractDomain`] trait, concrete domain implementations, a forward
//! fixpoint iterator over CFG, and built-in checkers for common vulnerability classes
//! (CWE-120 buffer overflow, CWE-190 integer overflow, division by zero).
//!
//! This is **Pillar 3** of SAF's analysis methodology — entirely independent from
//! pointer/value-flow analysis (Pillar 2) and IFDS (Pillar 1).
//!
//! # Architecture
//!
//! ```text
//! Layer 5: Interprocedural analysis (function summaries, SCC-aware iteration)
//! Layer 4: Numeric Checkers (buffer overflow, integer overflow, div-by-zero)
//! Layer 3: Query Interface (invariant_at, check_condition)
//! Layer 2: Fixpoint Iterator (worklist + widening/narrowing)
//! Layer 1: Abstract Domain (trait + domain implementations)
//! ```
//!
//! # Abstract Domains
//!
//! - **Interval** ([`Interval`]): Tracks `[lo, hi]` ranges per variable. Efficient
//!   (O(1) per operation) and sufficient for most numeric checks. Supports configurable
//!   bit widths and threshold-based widening.
//! - **Octagon** ([`octagon::OctagonDomain`]): Tracks relational constraints of the
//!   form `+-x +- y <= c` via a Difference-Bound Matrix (DBM). More precise than
//!   intervals for loop bound relationships but O(n^3) closure cost.
//! - **Nullness** ([`nullness::Nullness`]): Four-element lattice (Bottom/Null/NotNull/
//!   MaybeNull) for tracking pointer nullability. Used by PTABen oracle checks.
//! - **Escape** ([`escape::EscapeState`]): Tracks whether pointers escape their
//!   creating scope (NoEscape/ReturnEscape/GlobalEscape).
//!
//! # PTA Integration
//!
//! The abstract interpreter can optionally receive a [`PtaIntegration`] context that
//! provides alias information from the pointer analysis. This enables alias-aware
//! memory tracking: stores through aliased pointers correctly invalidate loaded values,
//! and function call side-effects are modeled via Mod/Ref summaries.
//!
//! # Interprocedural Analysis
//!
//! The [`interprocedural`](solve_interprocedural) layer extends intraprocedural
//! analysis with function summaries (return value intervals), SCC-aware bottom-up
//! traversal, and spec-based summaries for external/library functions. Callees are
//! analyzed before callers; recursive SCCs use widening across iterations.

mod checker;
mod config;
mod domain;
pub mod escape;
mod export;
mod fixpoint;
pub mod function_properties;
mod generic_fixpoint;
mod interprocedural;
mod interval;
pub mod nullness;
#[cfg(feature = "z3-solver")]
pub mod numeric_z3;
pub mod octagon;
pub mod partition;
mod pta_integration;
mod result;
pub mod sccp;
mod state;
mod threshold;
mod transfer;
mod transfer_fn;

pub use checker::{
    NumericCheckResult, NumericCheckerKind, NumericFinding, NumericSeverity, check_all_numeric,
    check_buffer_overflow, check_buffer_overflow_with_pta, check_buffer_overflow_with_specs,
    check_division_by_zero, check_integer_overflow, check_integer_overflow_with_specs,
    check_memcpy_overflow, check_memcpy_overflow_with_pta_and_specs,
    check_memcpy_overflow_with_result, check_memcpy_overflow_with_specs, check_shift_count,
};
pub use config::AbstractInterpConfig;
pub use domain::AbstractDomain;
pub use export::AbstractInterpExport;
pub use fixpoint::{
    FixpointContext, detect_loop_headers, solve_abstract_interp,
    solve_abstract_interp_with_context, solve_abstract_interp_with_pta,
    solve_abstract_interp_with_pta_and_summaries, solve_abstract_interp_with_specs,
};
// block_ends_noreturn and build_func_names are pub(crate) in fixpoint.rs,
// used directly via crate::absint::fixpoint:: paths by sibling modules.
pub use generic_fixpoint::{
    GenericAbstractInterpResult, GenericFixpointConfig, GenericFixpointDiagnostics, solve_generic,
};
pub use interprocedural::{
    FunctionSummary, InterproceduralContext, InterproceduralResult, solve_interprocedural,
    solve_interprocedural_with_context, solve_interprocedural_with_pta,
    solve_interprocedural_with_pta_and_specs, solve_interprocedural_with_specs,
    summary_from_spec as interprocedural_summary_from_spec,
};
pub use interval::{Interval, signed_max, signed_min};
pub use nullness::analyze_nullness_with_pta_specs_and_summaries;
pub use partition::{PartitionConfig, PartitionedState};
pub use pta_integration::PtaIntegration;
pub use result::AbstractInterpResult;
pub use sccp::SccpResult;
pub use state::AbstractState;
pub use threshold::extract_thresholds;
pub use transfer::{TransferContext, transfer_instruction_with_context};
pub use transfer_fn::{GenericState, IntervalTransfer, TransferFn};
