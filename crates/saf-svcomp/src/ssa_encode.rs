//! Shared SSA→bitvector encoding primitives for the Z3-backed `unreach-call`
//! engines (`bmc` fixed-k path BMC and `se_interp` KLEE-style forward symbolic
//! execution).
//!
//! Both engines translate AIR SSA definitions into fixed-width Z3 bitvectors so a
//! branch guard becomes a constraint over the actual `__VERIFIER_nondet_*` inputs.
//! The pure, per-operation pieces of that translation live here so the two engines
//! share ONE arithmetic semantics (any divergence would mean one engine proposes a
//! model the other could not — and both are replay-gated, so a bug here can only
//! ever cost recall, never soundness).

use saf_core::air::BinaryOp;
use z3::ast::{BV, Bool};

use crate::property_kind::DataModel;

/// Uniform bitvector width for the SSA encoding. Machine `int`/`long` arithmetic
/// that stays in range is faithful at 64 bits; wider/narrower widths differ only
/// on overflow, which the native replay filters out (a wrong model → `unknown`).
pub(crate) const BV_WIDTH: u32 = 64;

/// A 64-bit zero bitvector.
#[must_use]
pub(crate) fn zero() -> BV {
    BV::from_i64(0, BV_WIDTH)
}

/// A 64-bit one bitvector.
#[must_use]
pub(crate) fn one() -> BV {
    BV::from_i64(1, BV_WIDTH)
}

/// Structural equality of two bitvectors, isolating the single `_eq` deprecation
/// allow used across the encoders.
#[must_use]
pub(crate) fn eq(a: &BV, b: &BV) -> Bool {
    #[allow(deprecated)]
    a._eq(b)
}

/// Materialize an i1 boolean as a 0/1 bitvector (the AIR representation of an
/// `icmp` result feeding further arithmetic / selects).
#[must_use]
pub(crate) fn from_bool(cond: &Bool) -> BV {
    cond.ite(&one(), &zero())
}

/// The Z3 boolean `v != 0` — the AIR "truthiness" of a value operand.
#[must_use]
pub(crate) fn truthy(v: &BV) -> Bool {
    eq(v, &zero()).not()
}

/// Encode a binary operation over already-encoded operand bitvectors.
///
/// Returns `None` for operations not modelled in the integer bitvector theory
/// (floating point) — the caller substitutes a fresh havoc symbol. For the
/// division / remainder family, a `divisor != 0` side condition is pushed into
/// `guards`; the caller adds each guard to its solver (BMC) or path condition
/// (forward SE) so a divide-by-zero path is pruned rather than mis-modelled.
#[must_use]
pub(crate) fn encode_binop(kind: BinaryOp, a: &BV, b: &BV, guards: &mut Vec<Bool>) -> Option<BV> {
    let bv = match kind {
        BinaryOp::Add => a.bvadd(b),
        BinaryOp::Sub => a.bvsub(b),
        BinaryOp::Mul => a.bvmul(b),
        BinaryOp::SDiv => {
            guards.push(truthy(b));
            a.bvsdiv(b)
        }
        BinaryOp::UDiv => {
            guards.push(truthy(b));
            a.bvudiv(b)
        }
        BinaryOp::SRem => {
            guards.push(truthy(b));
            a.bvsrem(b)
        }
        BinaryOp::URem => {
            guards.push(truthy(b));
            a.bvurem(b)
        }
        BinaryOp::And => a.bvand(b),
        BinaryOp::Or => a.bvor(b),
        BinaryOp::Xor => a.bvxor(b),
        BinaryOp::Shl => a.bvshl(b),
        BinaryOp::LShr => a.bvlshr(b),
        BinaryOp::AShr => a.bvashr(b),
        BinaryOp::ICmpEq => from_bool(&eq(a, b)),
        BinaryOp::ICmpNe => from_bool(&eq(a, b).not()),
        BinaryOp::ICmpSlt => from_bool(&a.bvslt(b)),
        BinaryOp::ICmpSle => from_bool(&a.bvsle(b)),
        BinaryOp::ICmpSgt => from_bool(&a.bvsgt(b)),
        BinaryOp::ICmpSge => from_bool(&a.bvsge(b)),
        BinaryOp::ICmpUlt => from_bool(&a.bvult(b)),
        BinaryOp::ICmpUle => from_bool(&a.bvule(b)),
        BinaryOp::ICmpUgt => from_bool(&a.bvugt(b)),
        BinaryOp::ICmpUge => from_bool(&a.bvuge(b)),
        // Float arithmetic / comparisons are not modelled in the BV theory.
        _ => return None,
    };
    Some(bv)
}

/// In-range `[min, max]` for a scalar-int `__VERIFIER_nondet_*` result, honouring
/// the data model for width-dependent types (`long`/`ulong`/`size_t`). Values are
/// kept representable as `i64` (64-bit unsigned is clamped to the non-negative
/// `i64` half — still in range, R5-compliant, just not exhaustive).
#[must_use]
pub(crate) fn nondet_range(name: &str, dm: DataModel) -> Option<(i64, i64)> {
    let long_bits = match dm {
        DataModel::ILP32 => 32u32,
        DataModel::LP64 => 64,
    };
    let signed = |bits: u32| -> (i64, i64) {
        if bits >= 64 {
            (i64::MIN, i64::MAX)
        } else {
            let hi = (1i64 << (bits - 1)) - 1;
            (-(1i64 << (bits - 1)), hi)
        }
    };
    let unsigned = |bits: u32| -> (i64, i64) {
        if bits >= 64 {
            (0, i64::MAX)
        } else {
            (0, (1i64 << bits) - 1)
        }
    };
    Some(match name {
        "__VERIFIER_nondet_int" => signed(32),
        "__VERIFIER_nondet_uint" => unsigned(32),
        "__VERIFIER_nondet_short" => signed(16),
        "__VERIFIER_nondet_ushort" => unsigned(16),
        "__VERIFIER_nondet_char" => signed(8),
        "__VERIFIER_nondet_uchar" => unsigned(8),
        "__VERIFIER_nondet_bool" => (0, 1),
        "__VERIFIER_nondet_long" => signed(long_bits),
        "__VERIFIER_nondet_ulong" | "__VERIFIER_nondet_size_t" => unsigned(long_bits),
        "__VERIFIER_nondet_longlong" => signed(64),
        "__VERIFIER_nondet_ulonglong" => unsigned(64),
        _ => return None,
    })
}
