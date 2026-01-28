//! Pattern-based LLVM intrinsic handling.
//!
//! Classifies intrinsics by their relevance to pointer/value-flow analysis:
//! - Memory operations (memcpy, memset) → mapped to AIR operations
//! - No-op for analysis (lifetime, debug) → skipped
//! - Pass-through (expect) → identity copy
//! - Unknown → treated as external call

use saf_core::air::Operation;

/// Classification of LLVM intrinsics for AIR mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrinsicMapping {
    /// Map to a specific AIR operation.
    MapTo(IntrinsicOp),
    /// Skip — no AIR instruction emitted (no pointer/value-flow effect).
    Skip,
    /// Treat as external function call.
    External,
    /// Pass-through — value identity (emit Copy operation).
    PassThrough,
}

/// AIR operations that intrinsics can map to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicOp {
    /// Memory copy (llvm.memcpy, llvm.memmove).
    Memcpy,
    /// Memory set (llvm.memset).
    Memset,
}

impl IntrinsicOp {
    /// Convert to the corresponding AIR operation.
    #[must_use]
    pub fn to_operation(self) -> Operation {
        match self {
            Self::Memcpy => Operation::Memcpy,
            Self::Memset => Operation::Memset,
        }
    }
}

/// Classify an LLVM intrinsic by name.
///
/// Uses pattern-based matching on intrinsic name prefixes.
/// See `plans/FUTURE.md` for discussion of full intrinsic table approach.
// NOTE: This function is intentionally long because it serves as the canonical
// intrinsic classification table, mapping all LLVM intrinsic families to their
// analysis-relevant categories. Splitting by category would scatter this reference.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn classify_intrinsic(name: &str) -> IntrinsicMapping {
    // Memory operations — important for pointer analysis
    if name.starts_with("llvm.memcpy.") {
        return IntrinsicMapping::MapTo(IntrinsicOp::Memcpy);
    }
    if name.starts_with("llvm.memmove.") {
        return IntrinsicMapping::MapTo(IntrinsicOp::Memcpy);
    }
    if name.starts_with("llvm.memset.") {
        return IntrinsicMapping::MapTo(IntrinsicOp::Memset);
    }

    // No-op for analysis — skip entirely
    if name.starts_with("llvm.lifetime.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.dbg.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.assume") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.annotation.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.invariant.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.experimental.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.var.annotation") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.ptr.annotation") {
        // `llvm.ptr.annotation` returns its first argument (a pointer) with an
        // attached annotation. Treat as pass-through so the result `ValueId`
        // propagates the pointer, matching `llvm.launder.invariant.group` semantics.
        return IntrinsicMapping::PassThrough;
    }
    if name.starts_with("llvm.codeview.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.instrprof.") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.is.constant") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.sideeffect") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.pseudoprobe") {
        return IntrinsicMapping::Skip;
    }
    // Stack save/restore — no pointer analysis effect
    if name.starts_with("llvm.stacksave") {
        return IntrinsicMapping::Skip;
    }
    if name.starts_with("llvm.stackrestore") {
        return IntrinsicMapping::Skip;
    }
    // Vararg intrinsics — no pointer effect
    if name == "llvm.va_start" || name == "llvm.va_end" || name == "llvm.va_copy" {
        return IntrinsicMapping::Skip;
    }

    // Pass-through — value identity
    if name.starts_with("llvm.expect.") {
        return IntrinsicMapping::PassThrough;
    }
    if name.starts_with("llvm.launder.invariant.group") {
        return IntrinsicMapping::PassThrough;
    }
    if name.starts_with("llvm.strip.invariant.group") {
        return IntrinsicMapping::PassThrough;
    }
    if name.starts_with("llvm.ptrmask") {
        return IntrinsicMapping::PassThrough;
    }

    // Arithmetic intrinsics — treat as external (result doesn't affect pointers)
    if name.starts_with("llvm.abs.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.smax.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.smin.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.umax.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.umin.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.sadd.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.uadd.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.ssub.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.usub.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.smul.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.umul.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.ctlz.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.cttz.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.ctpop.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.bswap.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.bitreverse.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.fshl.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.fshr.") {
        return IntrinsicMapping::External;
    }

    // Floating point intrinsics
    if name.starts_with("llvm.sqrt.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.sin.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.cos.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.pow.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.exp.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.log.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.fma.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.fabs.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.floor.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.ceil.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.trunc.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.round.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.rint.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.nearbyint.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.copysign.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.minnum.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.maxnum.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.minimum.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.maximum.") {
        return IntrinsicMapping::External;
    }

    // Trap and unreachable
    if name == "llvm.trap" || name == "llvm.debugtrap" {
        return IntrinsicMapping::External;
    }
    if name == "llvm.ubsantrap" {
        return IntrinsicMapping::External;
    }

    // Stack/frame address intrinsics
    if name.starts_with("llvm.frameaddress") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.returnaddress") {
        return IntrinsicMapping::External;
    }

    // Address space intrinsics
    if name.starts_with("llvm.addressofreturnaddress") {
        return IntrinsicMapping::External;
    }

    // Atomic intrinsics — treat as external
    if name.starts_with("llvm.atomic.") {
        return IntrinsicMapping::External;
    }

    // Vector intrinsics
    if name.starts_with("llvm.vector.") {
        return IntrinsicMapping::External;
    }
    if name.starts_with("llvm.masked.") {
        return IntrinsicMapping::External;
    }

    // Object size intrinsics
    if name.starts_with("llvm.objectsize.") {
        return IntrinsicMapping::External;
    }

    // Freeze
    if name == "llvm.freeze" {
        return IntrinsicMapping::PassThrough;
    }

    // Default: treat as external call
    IntrinsicMapping::External
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memcpy_intrinsic() {
        assert_eq!(
            classify_intrinsic("llvm.memcpy.p0i8.p0i8.i64"),
            IntrinsicMapping::MapTo(IntrinsicOp::Memcpy)
        );
        assert_eq!(
            classify_intrinsic("llvm.memcpy.p0.p0.i64"),
            IntrinsicMapping::MapTo(IntrinsicOp::Memcpy)
        );
    }

    #[test]
    fn memmove_intrinsic() {
        assert_eq!(
            classify_intrinsic("llvm.memmove.p0i8.p0i8.i64"),
            IntrinsicMapping::MapTo(IntrinsicOp::Memcpy)
        );
    }

    #[test]
    fn memset_intrinsic() {
        assert_eq!(
            classify_intrinsic("llvm.memset.p0i8.i64"),
            IntrinsicMapping::MapTo(IntrinsicOp::Memset)
        );
    }

    #[test]
    fn skip_intrinsics() {
        assert_eq!(
            classify_intrinsic("llvm.lifetime.start.p0i8"),
            IntrinsicMapping::Skip
        );
        assert_eq!(
            classify_intrinsic("llvm.lifetime.end.p0i8"),
            IntrinsicMapping::Skip
        );
        assert_eq!(
            classify_intrinsic("llvm.dbg.declare"),
            IntrinsicMapping::Skip
        );
        assert_eq!(classify_intrinsic("llvm.dbg.value"), IntrinsicMapping::Skip);
        assert_eq!(classify_intrinsic("llvm.assume"), IntrinsicMapping::Skip);
    }

    #[test]
    fn passthrough_intrinsics() {
        assert_eq!(
            classify_intrinsic("llvm.expect.i64"),
            IntrinsicMapping::PassThrough
        );
        assert_eq!(
            classify_intrinsic("llvm.expect.i1"),
            IntrinsicMapping::PassThrough
        );
    }

    #[test]
    fn unknown_intrinsic_is_external() {
        assert_eq!(
            classify_intrinsic("llvm.some.unknown.intrinsic"),
            IntrinsicMapping::External
        );
    }

    #[test]
    fn arithmetic_intrinsics_are_external() {
        assert_eq!(
            classify_intrinsic("llvm.abs.i32"),
            IntrinsicMapping::External
        );
        assert_eq!(
            classify_intrinsic("llvm.ctlz.i32"),
            IntrinsicMapping::External
        );
    }

    #[test]
    fn va_intrinsics_are_skip() {
        assert_eq!(classify_intrinsic("llvm.va_start"), IntrinsicMapping::Skip);
        assert_eq!(classify_intrinsic("llvm.va_end"), IntrinsicMapping::Skip);
        assert_eq!(classify_intrinsic("llvm.va_copy"), IntrinsicMapping::Skip);
    }

    #[test]
    fn stack_intrinsics_are_skip() {
        assert_eq!(classify_intrinsic("llvm.stacksave"), IntrinsicMapping::Skip);
        assert_eq!(
            classify_intrinsic("llvm.stacksave.p0"),
            IntrinsicMapping::Skip
        );
        assert_eq!(
            classify_intrinsic("llvm.stackrestore"),
            IntrinsicMapping::Skip
        );
        assert_eq!(
            classify_intrinsic("llvm.stackrestore.p0"),
            IntrinsicMapping::Skip
        );
    }
}
