//! Interval abstract domain for numeric value analysis.
//!
//! Implements closed intervals `[lo, hi]` over fixed-width integers with
//! wrapped semantics for soundness with LLVM IR's modular arithmetic.
//! Supports widening with thresholds (Astrée/IKOS technique).

use std::cmp;
use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::domain::AbstractDomain;

/// Minimum value for a signed integer of the given bit-width.
#[must_use]
pub fn signed_min(bits: u8) -> i128 {
    if bits == 0 {
        return 0;
    }
    if bits >= 128 {
        return i128::MIN; // `-(1 << 127)` overflows
    }
    -(1i128 << (bits - 1))
}

/// Maximum value for a signed integer of the given bit-width.
#[must_use]
pub fn signed_max(bits: u8) -> i128 {
    if bits == 0 {
        return 0;
    }
    if bits >= 128 {
        return i128::MAX;
    }
    (1i128 << (bits - 1)) - 1
}

/// A closed interval `[lo, hi]` over fixed-width integers.
///
/// Represents the set of integers `{ x | lo <= x <= hi }`.
/// Bottom (empty set) is represented by setting the `bottom` flag.
/// Top (all values for the bit-width) is `[signed_min, signed_max]`.
///
/// The interval respects bit-width for wraparound semantics:
/// arithmetic results are clamped to the bit-width range via widening.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    /// Lower bound.
    lo: i128,
    /// Upper bound.
    hi: i128,
    /// Bit-width (8, 16, 32, 64).
    bits: u8,
    /// Whether this is the bottom (empty) interval.
    bottom: bool,
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bottom {
            write!(f, "⊥(i{})", self.bits)
        } else if self.is_top() {
            write!(f, "⊤(i{})", self.bits)
        } else if self.lo == self.hi {
            write!(f, "[{}](i{})", self.lo, self.bits)
        } else {
            write!(f, "[{}, {}](i{})", self.lo, self.hi, self.bits)
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bottom {
            write!(f, "⊥")
        } else if self.is_top() {
            write!(f, "⊤")
        } else if self.lo == self.hi {
            write!(f, "[{}]", self.lo)
        } else {
            write!(f, "[{}, {}]", self.lo, self.hi)
        }
    }
}

impl Interval {
    /// Create a new interval `[lo, hi]` with the given bit-width.
    ///
    /// Returns bottom if `lo > hi`.
    #[must_use]
    pub fn new(lo: i128, hi: i128, bits: u8) -> Self {
        if lo > hi {
            return Self::make_bottom(bits);
        }
        Self {
            lo,
            hi,
            bits,
            bottom: false,
        }
    }

    /// Create a singleton interval `[v, v]`.
    #[must_use]
    pub fn singleton(value: i128, bits: u8) -> Self {
        Self {
            lo: value,
            hi: value,
            bits,
            bottom: false,
        }
    }

    /// Create the bottom (empty) interval for a given bit-width.
    #[must_use]
    pub fn make_bottom(bits: u8) -> Self {
        Self {
            lo: 0,
            hi: 0,
            bits,
            bottom: true,
        }
    }

    /// Create the top (full range) interval for a given bit-width.
    #[must_use]
    pub fn make_top(bits: u8) -> Self {
        Self {
            lo: signed_min(bits),
            hi: signed_max(bits),
            bits,
            bottom: false,
        }
    }

    /// Get the lower bound.
    #[must_use]
    pub fn lo(&self) -> i128 {
        self.lo
    }

    /// Get the upper bound.
    #[must_use]
    pub fn hi(&self) -> i128 {
        self.hi
    }

    /// Get the bit-width.
    #[must_use]
    pub fn bits(&self) -> u8 {
        self.bits
    }

    /// Check whether this is a singleton interval.
    #[must_use]
    pub fn is_singleton(&self) -> bool {
        !self.bottom && self.lo == self.hi
    }

    /// Get the singleton value if this interval is a singleton.
    ///
    /// Returns `Some(value)` if this is a singleton interval `[v, v]`,
    /// or `None` if the interval is bottom, top, or has width > 0.
    #[must_use]
    pub fn as_singleton(&self) -> Option<i64> {
        if self.is_singleton() {
            // Convert to i64, saturating if out of range
            #[allow(clippy::cast_possible_truncation)]
            Some(self.lo as i64)
        } else {
            None
        }
    }

    /// Check whether this interval contains a specific value.
    #[must_use]
    pub fn contains(&self, value: i128) -> bool {
        !self.bottom && value >= self.lo && value <= self.hi
    }

    /// Check whether this interval is fully contained within `[lo, hi)`.
    #[must_use]
    pub fn within_range(&self, lo: i128, hi_exclusive: i128) -> bool {
        !self.bottom && self.lo >= lo && self.hi < hi_exclusive
    }

    /// Check whether this interval may overlap with `[lo, hi)`.
    #[must_use]
    pub fn may_overlap(&self, lo: i128, hi_exclusive: i128) -> bool {
        !self.bottom && self.lo < hi_exclusive && self.hi >= lo
    }

    /// Check whether this interval contains zero.
    #[must_use]
    pub fn contains_zero(&self) -> bool {
        self.contains(0)
    }

    /// Check whether this interval is the singleton `[0, 0]`.
    #[must_use]
    pub fn is_singleton_zero(&self) -> bool {
        !self.bottom && self.lo == 0 && self.hi == 0
    }

    // =========================================================================
    // Encoding normalisation
    //
    // `Interval` carries no signedness tag. The domain's canonical encoding of a
    // `w`-bit value (w >= 2) is its SIGNED value in `[signed_min(w),
    // signed_max(w)]`; `i1` is canonically `{0, 1}` (what `icmp` produces). An
    // interval may still arrive in another encoding — an unsigned bound such as
    // `0xFFFFFFFF`, or a width label that is not the value's type width — and the
    // signed reasoning (`join`'s range clamp, the signed refiners, the
    // no-overflow sentinel) is unsound on those. `wrap_to_width` restores the
    // canonical encoding EXACTLY (modular arithmetic), never by clamping.
    // =========================================================================

    /// Truth-value view of an `i1` interval: `(may_be_false, may_be_true)`.
    ///
    /// `i1` values reach the domain as `{0, 1}` (comparison results) and as `-1`
    /// (sign-extended IR constants), so a boolean is read by zero / non-zero,
    /// never by its raw bounds.
    fn truth(&self) -> (bool, bool) {
        if self.bottom {
            return (false, false);
        }
        (self.contains(0), self.lo != 0 || self.hi != 0)
    }

    /// Canonical `i1`-encoded interval (`{0, 1}` subset) with the given truth values,
    /// labelled `bits` (callers pass 1, or a target width for `zext`).
    fn from_truth(may_false: bool, may_true: bool, bits: u8) -> Self {
        match (may_false, may_true) {
            (false, false) => Self::make_bottom(bits),
            (true, false) => Self::singleton(0, bits),
            (false, true) => Self::singleton(1, bits),
            (true, true) => Self::new(0, 1, bits),
        }
    }

    /// Re-encode this interval as `bits`-bit values in the canonical encoding.
    ///
    /// Every bound `v` stands for the bit pattern `v mod 2^bits`; the result is
    /// the tightest interval over those patterns' SIGNED values (`{0, 1}` truth
    /// encoding for `bits == 1`). Exact when the patterns form one contiguous
    /// signed run, otherwise `⊤` — never smaller than the set it represents, so
    /// it is always sound to apply. Widths outside `1..127` are relabelled only.
    #[must_use]
    pub fn wrap_to_width(&self, bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(bits);
        }
        if bits == 1 {
            let (may_false, may_true) = self.truth();
            return Self::from_truth(may_false, may_true, 1);
        }
        // `1 << 127` does not fit `i128`; such widths are never wrapped.
        if bits == 0 || bits >= 127 {
            return Self::new(self.lo, self.hi, bits);
        }
        let min = signed_min(bits);
        let max = signed_max(bits);
        if self.lo >= min && self.hi <= max {
            return Self::new(self.lo, self.hi, bits);
        }
        let modulus = 1i128 << bits;
        // `span + 1` values: covering a whole period means every pattern.
        let covers_all = self
            .hi
            .checked_sub(self.lo)
            .is_none_or(|span| span >= modulus - 1);
        if covers_all {
            return Self::make_top(bits);
        }
        let wrap = |v: i128| {
            let r = v.rem_euclid(modulus);
            if r > max { r - modulus } else { r }
        };
        let (lo, hi) = (wrap(self.lo), wrap(self.hi));
        // Fewer than `2^bits` consecutive values map to one contiguous run iff
        // the run does not cross the `max -> min` seam, i.e. iff `lo <= hi`.
        if lo <= hi {
            Self::new(lo, hi, bits)
        } else {
            Self::make_top(bits)
        }
    }

    /// The values read as UNSIGNED `self.bits`-bit integers, as `(lo, hi)` within
    /// `[0, 2^bits - 1]`. A run that holds both negative and non-negative signed
    /// values covers both ends of the unsigned range, so it reads as the full
    /// range. `None` for bottom or a width the `i128` encoding cannot hold.
    fn unsigned_bounds(&self) -> Option<(i128, i128)> {
        if self.bottom || self.bits == 0 || self.bits >= 127 {
            return None;
        }
        let w = self.wrap_to_width(self.bits);
        if w.bottom {
            return None;
        }
        let modulus = 1i128 << self.bits;
        Some(if w.lo >= 0 {
            (w.lo, w.hi)
        } else if w.hi < 0 {
            (w.lo + modulus, w.hi + modulus)
        } else {
            (0, modulus - 1)
        })
    }

    /// Canonical interval for the unsigned `bits`-bit range `[lo, hi]`.
    fn from_unsigned_bounds(lo: i128, hi: i128, bits: u8) -> Self {
        Self::new(lo, hi, bits).wrap_to_width(bits)
    }

    /// Widening with threshold set.
    ///
    /// Instead of jumping to ±∞ when bounds grow, uses the nearest
    /// threshold constant from `thresholds`. This dramatically improves
    /// precision at loops (Astrée/IKOS technique).
    #[must_use]
    pub fn widen_with_thresholds(&self, other: &Self, thresholds: &BTreeSet<i128>) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }
        // `i1` has three raw values; joining terminates without widening, and
        // the signed bound `max = 0` would drop `true` (= 1).
        if self.bits == 1 {
            return self.join(other);
        }
        // Range bounds below are only meaningful on the canonical encoding.
        let this = self.wrap_to_width(self.bits);
        let other = other.wrap_to_width(self.bits);
        let (this, other) = (&this, &other);

        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        // Lower bound: if other.lo < this.lo, find largest threshold <= other.lo
        let new_lo = if other.lo < this.lo {
            thresholds
                .range(..=other.lo)
                .next_back()
                .copied()
                .unwrap_or(min_bound)
                .max(min_bound)
        } else {
            this.lo
        };

        // Upper bound: if other.hi > this.hi, find smallest threshold >= other.hi
        let new_hi = if other.hi > this.hi {
            thresholds
                .range(other.hi..)
                .next()
                .copied()
                .unwrap_or(max_bound)
                .min(max_bound)
        } else {
            this.hi
        };

        Self::new(new_lo, new_hi, self.bits)
    }

    // =========================================================================
    // Arithmetic operations
    // =========================================================================

    /// Addition: `[a, b] + [c, d] = [a+c, b+d]`.
    ///
    /// Returns top if overflow occurs beyond the bit-width range.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let lo = self.lo.saturating_add(other.lo);
        let hi = self.hi.saturating_add(other.hi);
        self.clamp_or_top(lo, hi)
    }

    /// "Unwrapped" addition: computes mathematical result without clamping.
    ///
    /// Returns `(lo, hi)` as the exact mathematical bounds. Used by the
    /// integer overflow checker to detect when the mathematical result
    /// exceeds the target bit-width range.
    #[must_use]
    pub fn add_unwrapped(&self, other: &Self) -> (i128, i128) {
        if self.bottom || other.bottom {
            return (0, 0);
        }
        (
            self.lo.saturating_add(other.lo),
            self.hi.saturating_add(other.hi),
        )
    }

    /// Subtraction: `[a, b] - [c, d] = [a-d, b-c]`.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let lo = self.lo.saturating_sub(other.hi);
        let hi = self.hi.saturating_sub(other.lo);
        self.clamp_or_top(lo, hi)
    }

    /// Unwrapped subtraction.
    #[must_use]
    pub fn sub_unwrapped(&self, other: &Self) -> (i128, i128) {
        if self.bottom || other.bottom {
            return (0, 0);
        }
        (
            self.lo.saturating_sub(other.hi),
            self.hi.saturating_sub(other.lo),
        )
    }

    /// Multiplication: `[a, b] * [c, d]`.
    ///
    /// Takes min/max of all four products to handle negative factors.
    #[must_use]
    pub fn mul(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let products = [
            self.lo.saturating_mul(other.lo),
            self.lo.saturating_mul(other.hi),
            self.hi.saturating_mul(other.lo),
            self.hi.saturating_mul(other.hi),
        ];
        let lo = products.iter().copied().min().unwrap_or(0);
        let hi = products.iter().copied().max().unwrap_or(0);
        self.clamp_or_top(lo, hi)
    }

    /// Unwrapped multiplication.
    #[must_use]
    pub fn mul_unwrapped(&self, other: &Self) -> (i128, i128) {
        if self.bottom || other.bottom {
            return (0, 0);
        }
        let products = [
            self.lo.saturating_mul(other.lo),
            self.lo.saturating_mul(other.hi),
            self.hi.saturating_mul(other.lo),
            self.hi.saturating_mul(other.hi),
        ];
        let lo = products.iter().copied().min().unwrap_or(0);
        let hi = products.iter().copied().max().unwrap_or(0);
        (lo, hi)
    }

    /// Signed division: `[a, b] / [c, d]`.
    ///
    /// Returns top if the divisor range includes zero.
    #[must_use]
    pub fn sdiv(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        // Division by zero → top
        if other.lo <= 0 && other.hi >= 0 {
            return Self::make_top(self.bits);
        }
        let quotients = [
            self.lo.checked_div(other.lo).unwrap_or(0),
            self.lo.checked_div(other.hi).unwrap_or(0),
            self.hi.checked_div(other.lo).unwrap_or(0),
            self.hi.checked_div(other.hi).unwrap_or(0),
        ];
        let lo = quotients.iter().copied().min().unwrap_or(0);
        let hi = quotients.iter().copied().max().unwrap_or(0);
        self.clamp_or_top(lo, hi)
    }

    /// Unsigned division at width `self.bits` (both operands read as UNSIGNED).
    ///
    /// A negative bound is a `bits`-bit pattern above the signed maximum
    /// (`-1` is `0xFFFFFFFF` at 32 bits), so the operands are read through
    /// [`Self::unsigned_bounds`] — reading `hi.max(0)` instead once bounded
    /// `0xFFFFFFFF / 2` by `0`. The caller must label `self` with the
    /// operation's type width.
    #[must_use]
    pub fn udiv(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let (Some((a_lo, a_hi)), Some((d_lo, d_hi))) = (
            self.unsigned_bounds(),
            other.wrap_to_width(self.bits).unsigned_bounds(),
        ) else {
            return Self::make_top(self.bits);
        };
        // Division by zero → top
        if d_lo == 0 {
            return Self::make_top(self.bits);
        }
        Self::from_unsigned_bounds(a_lo / d_hi, a_hi / d_lo, self.bits)
    }

    /// Signed remainder.
    #[must_use]
    pub fn srem(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if other.lo <= 0 && other.hi >= 0 {
            return Self::make_top(self.bits);
        }

        // Singleton case: compute exact result
        if self.is_singleton() && other.is_singleton() {
            let result = self.lo.checked_rem(other.lo).unwrap_or(0);
            return Self::singleton(result, self.bits);
        }

        // Conservative: result magnitude < |divisor|
        // If the dividend is non-negative, the result of signed remainder
        // is also non-negative (C/LLVM semantics: sign of result = sign of dividend).
        let max_abs = cmp::max(other.lo.abs(), other.hi.abs()) - 1;
        let lo = if self.lo >= 0 { 0 } else { -max_abs };
        let hi = if self.hi > 0 { max_abs } else { 0 };
        Self::new(lo, hi, self.bits)
    }

    /// Unsigned remainder at width `self.bits` (both operands read as UNSIGNED).
    ///
    /// The divisor bound is its unsigned value, not `|divisor|`: `x %u -2` at 32
    /// bits divides by `0xFFFFFFFE`, so the remainder reaches `0xFFFFFFFD`.
    #[must_use]
    pub fn urem(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let (Some((a_lo, a_hi)), Some((d_lo, d_hi))) = (
            self.unsigned_bounds(),
            other.wrap_to_width(self.bits).unsigned_bounds(),
        ) else {
            return Self::make_top(self.bits);
        };
        if d_lo == 0 {
            return Self::make_top(self.bits);
        }
        if a_hi < d_lo {
            // Every dividend is below every divisor: the remainder is the dividend.
            return Self::from_unsigned_bounds(a_lo, a_hi, self.bits);
        }
        Self::from_unsigned_bounds(0, a_hi.min(d_hi - 1), self.bits)
    }

    /// Left shift: `[a, b] << [c, d]` at width `self.bits`.
    ///
    /// The result is re-encoded with [`Self::wrap_to_width`]: bits shifted past
    /// the width are discarded, so `[0, 1] << 31` at 32 bits is `{0, INT_MIN}`,
    /// not `[0, 2^31]`. (An earlier version kept non-negative results up to
    /// `unsigned_max(bits)`; that unsigned encoding made `x < 0` look infeasible
    /// for a value that IS negative, and `join` then clamped the large values
    /// away.)
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // shift validated to [0, bits)
    pub fn shl(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        // Corner products below need `|v| << s` to fit in `i128`.
        if self.bits == 0 || self.bits > 64 {
            return Self::make_top(self.bits);
        }
        let this = self.wrap_to_width(self.bits);
        let shift = shift.wrap_to_width(self.bits);
        let bits = i128::from(self.bits);
        // Shift amount must be in [0, bits); `i1` shift amounts are `{0, 1}`.
        if shift.lo < 0 || shift.hi >= bits {
            return Self::make_top(self.bits);
        }
        // INVARIANT: 0 <= shift < bits <= 64, |value| <= 2^63, so no product
        // exceeds 2^126.
        let lo_shift = shift.lo as u32;
        let hi_shift = shift.hi as u32;
        let products = [
            this.lo << lo_shift,
            this.lo << hi_shift,
            this.hi << lo_shift,
            this.hi << hi_shift,
        ];
        let lo = products.iter().copied().min().unwrap_or(0);
        let hi = products.iter().copied().max().unwrap_or(0);
        Self::new(lo, hi, self.bits).wrap_to_width(self.bits)
    }

    /// Logical shift right at width `self.bits` (value read as UNSIGNED).
    ///
    /// A negative bound is a pattern with the top bit set, i.e. an unsigned value
    /// above the signed maximum: `0xFFFFFFFF >> 1` is `INT_MAX`. Reading it as
    /// `max(hi, 0)` once bounded that by `2^30 - 1` and let `x + x` be proven
    /// overflow-free. The caller must label `self` with the operation's width.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // shift validated to [0, bits)
    pub fn lshr(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        let (Some((a_lo, a_hi)), Some((s_lo, s_hi))) = (
            self.unsigned_bounds(),
            shift.wrap_to_width(self.bits).unsigned_bounds(),
        ) else {
            return Self::make_top(self.bits);
        };
        // A shift amount >= the width is poison.
        if s_hi >= i128::from(self.bits) {
            return Self::make_top(self.bits);
        }
        // INVARIANT: 0 <= s_lo <= s_hi < bits < 127.
        let lo = a_lo >> (s_hi as u32);
        let hi = a_hi >> (s_lo as u32);
        Self::from_unsigned_bounds(lo, hi, self.bits)
    }

    /// Arithmetic shift right (signed) at width `self.bits`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // shift validated to [0, bits)
    pub fn ashr(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        let this = self.wrap_to_width(self.bits);
        let shift = shift.wrap_to_width(self.bits);
        let bits = i128::from(self.bits);
        if this.bottom || shift.lo < 0 || shift.hi >= bits || bits >= 128 {
            return Self::make_top(self.bits);
        }
        let lo_shift = shift.lo as u32;
        let hi_shift = shift.hi as u32;
        let results = [
            this.lo >> lo_shift,
            this.lo >> hi_shift,
            this.hi >> lo_shift,
            this.hi >> hi_shift,
        ];
        let lo = results.iter().copied().min().unwrap_or(0);
        let hi = results.iter().copied().max().unwrap_or(0);
        Self::new(lo, hi, self.bits)
    }

    /// Bitwise AND (conservative) at width `self.bits`.
    #[must_use]
    pub fn bitand(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if self.bits == 1 {
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return Self::from_truth(fa || fb, ta && tb, 1);
        }
        let a = self.wrap_to_width(self.bits);
        let b = other.wrap_to_width(self.bits);
        // A non-negative operand has its sign bit clear, and so does the result,
        // which is also at most that operand.
        match (a.lo >= 0, b.lo >= 0) {
            (true, true) => Self::new(0, cmp::min(a.hi, b.hi), self.bits),
            (true, false) => Self::new(0, a.hi, self.bits),
            (false, true) => Self::new(0, b.hi, self.bits),
            (false, false) => Self::make_top(self.bits),
        }
    }

    /// Bitwise OR (conservative) at width `self.bits`.
    #[must_use]
    pub fn bitor(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if self.bits == 1 {
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return Self::from_truth(fa && fb, ta || tb, 1);
        }
        let a = self.wrap_to_width(self.bits);
        let b = other.wrap_to_width(self.bits);
        // Both non-negative: the sign bit stays clear and `a | b <= a + b`, so
        // the result is within the SIGNED range (never `unsigned_max`, which
        // would put a negative pattern in the unsigned encoding).
        if a.lo >= 0 && b.lo >= 0 {
            let hi = signed_max(self.bits).min(a.hi.saturating_add(b.hi));
            return Self::new(cmp::max(a.lo, b.lo), hi, self.bits);
        }
        Self::make_top(self.bits)
    }

    /// Bitwise XOR (conservative) at width `self.bits`.
    #[must_use]
    pub fn bitxor(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if self.bits == 1 {
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return Self::from_truth((fa && fb) || (ta && tb), (ta && fb) || (fa && tb), 1);
        }
        let a = self.wrap_to_width(self.bits);
        let b = other.wrap_to_width(self.bits);
        // Both non-negative: sign bit stays clear and `a ^ b <= a + b`.
        if a.lo >= 0 && b.lo >= 0 {
            let hi = signed_max(self.bits).min(a.hi.saturating_add(b.hi));
            return Self::new(0, hi, self.bits);
        }
        Self::make_top(self.bits)
    }

    // =========================================================================
    // Cast operations
    // =========================================================================

    /// Zero-extend to a larger bit-width.
    ///
    /// `Operation::Cast` carries only the TARGET width, so the source width is
    /// `self.bits` — trusted only when it is below the target (a `zext` always
    /// widens; a label `>= target` is a placeholder such as `⊤(i64)` for a
    /// loaded `i32`) and below 64. With an untrusted label, a non-negative value keeps its
    /// value and anything else is bounded only by `[0, signed_max(target)]`
    /// (the source is narrower than the target, so its unsigned range is).
    #[must_use]
    pub fn zext(&self, target_bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(target_bits);
        }
        if self.bits == 1 {
            let (may_false, may_true) = self.truth();
            return Self::from_truth(may_false, may_true, target_bits);
        }
        let cap = signed_max(target_bits);
        // 64 is also the placeholder label of an untyped value (`⊤(i64)` for a
        // parameter, or an `i32` refined from it), so it is never trusted as
        // the source width: reading `-1` as `2^64 - 1` where the source is `i32`
        // would miss the real `2^32 - 1`. Labels below 64 are exact.
        if self.bits < target_bits && self.bits < 64 {
            if let Some((lo, hi)) = self.unsigned_bounds() {
                return Self::new(lo, hi, target_bits).wrap_to_width(target_bits);
            }
        }
        if self.lo >= 0 && self.lo <= cap {
            return Self::new(self.lo, self.hi.min(cap), target_bits);
        }
        Self::new(0, cap, target_bits)
    }

    /// Sign-extend to a larger bit-width.
    ///
    /// A value in the canonical (signed) encoding keeps its value. An `i1`
    /// extends `true` to `-1`; a source label below the target is first wrapped
    /// to it so an unsigned-encoded bound becomes the negative value it is.
    #[must_use]
    pub fn sext(&self, target_bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(target_bits);
        }
        if self.bits == 1 {
            let (may_false, may_true) = self.truth();
            return match (may_false, may_true) {
                (false, false) => Self::make_bottom(target_bits),
                (true, false) => Self::singleton(0, target_bits),
                (false, true) => Self::singleton(-1, target_bits),
                (true, true) => Self::new(-1, 0, target_bits),
            };
        }
        let src = if self.bits < target_bits {
            self.wrap_to_width(self.bits)
        } else {
            self.clone()
        };
        Self::new(src.lo, src.hi, target_bits).wrap_to_width(target_bits)
    }

    /// Truncate to a smaller bit-width: the low `target_bits` bits, read as a
    /// signed value of that width (exact via [`Self::wrap_to_width`]).
    #[must_use]
    pub fn trunc(&self, target_bits: u8) -> Self {
        // To `i1` the result is the LOW BIT, not the zero/non-zero reading that
        // `wrap_to_width(1)` gives an already-boolean value: `trunc 2 to i1` is
        // 0. Exact for a singleton or a `{0, 1}` subset, else both truth values.
        if target_bits == 1 && self.bits != 1 && !self.bottom {
            if self.lo == self.hi {
                return Self::singleton(self.lo.rem_euclid(2), 1);
            }
            if self.lo >= 0 && self.hi <= 1 {
                return Self::new(self.lo, self.hi, 1);
            }
            return Self::new(0, 1, 1);
        }
        self.wrap_to_width(target_bits)
    }

    // =========================================================================
    // Comparison operations (produce i1 = [0, 1] results)
    // =========================================================================

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_slt(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.hi < other.lo {
            Self::singleton(1, 1) // always true
        } else if self.lo >= other.hi {
            Self::singleton(0, 1) // always false
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_sle(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.hi <= other.lo {
            Self::singleton(1, 1) // always true
        } else if self.lo > other.hi {
            Self::singleton(0, 1) // always false
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_sgt(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo > other.hi {
            Self::singleton(1, 1) // always true
        } else if self.hi <= other.lo {
            Self::singleton(0, 1) // always false
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_sge(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo >= other.hi {
            Self::singleton(1, 1) // always true
        } else if self.hi < other.lo {
            Self::singleton(0, 1) // always false
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_eq(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo == self.hi && other.lo == other.hi && self.lo == other.lo {
            Self::singleton(1, 1) // always true: both singletons with same value
        } else if self.hi < other.lo || other.hi < self.lo {
            Self::singleton(0, 1) // always false: disjoint ranges
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    #[must_use]
    pub fn icmp_ne(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.hi < other.lo || other.hi < self.lo {
            Self::singleton(1, 1) // always true: disjoint ranges
        } else if self.lo == self.hi && other.lo == other.hi && self.lo == other.lo {
            Self::singleton(0, 1) // always false: both singletons with same value
        } else {
            Self::new(0, 1, 1) // may be either
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    ///
    /// Unsigned less-than. Delegates to `icmp_slt` when both intervals are non-negative;
    /// otherwise conservatively returns `[0, 1]`.
    #[must_use]
    pub fn icmp_ult(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo >= 0 && other.lo >= 0 {
            self.icmp_slt(other)
        } else {
            Self::new(0, 1, 1)
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    ///
    /// Unsigned less-or-equal. Delegates to `icmp_sle` when both intervals are non-negative;
    /// otherwise conservatively returns `[0, 1]`.
    #[must_use]
    pub fn icmp_ule(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo >= 0 && other.lo >= 0 {
            self.icmp_sle(other)
        } else {
            Self::new(0, 1, 1)
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    ///
    /// Unsigned greater-than. Delegates to `icmp_sgt` when both intervals are non-negative;
    /// otherwise conservatively returns `[0, 1]`.
    #[must_use]
    pub fn icmp_ugt(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo >= 0 && other.lo >= 0 {
            self.icmp_sgt(other)
        } else {
            Self::new(0, 1, 1)
        }
    }

    /// Integer comparison producing a boolean interval `[0, 0]`, `[1, 1]`, or `[0, 1]`.
    ///
    /// Unsigned greater-or-equal. Delegates to `icmp_sge` when both intervals are non-negative;
    /// otherwise conservatively returns `[0, 1]`.
    #[must_use]
    pub fn icmp_uge(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(1);
        }
        if self.lo >= 0 && other.lo >= 0 {
            self.icmp_sge(other)
        } else {
            Self::new(0, 1, 1)
        }
    }

    /// Refine this interval given that `self < bound` is true.
    #[must_use]
    pub fn refine_slt_true(&self, bound: &Self) -> Self {
        if self.bottom || bound.bottom {
            return self.clone();
        }
        // self < bound → self.hi < bound.hi, so clamp hi
        let new_hi = cmp::min(self.hi, bound.hi - 1);
        if new_hi < self.lo {
            Self::make_bottom(self.bits)
        } else {
            Self::new(self.lo, new_hi, self.bits)
        }
    }

    /// Refine this interval given that `self < bound` is false (i.e. `self >= bound`).
    #[must_use]
    pub fn refine_slt_false(&self, bound: &Self) -> Self {
        if self.bottom || bound.bottom {
            return self.clone();
        }
        // !(self < bound) → self >= bound.lo
        let new_lo = cmp::max(self.lo, bound.lo);
        if new_lo > self.hi {
            Self::make_bottom(self.bits)
        } else {
            Self::new(new_lo, self.hi, self.bits)
        }
    }

    /// Refine this interval given that `self <= bound` is true.
    #[must_use]
    pub fn refine_sle_true(&self, bound: &Self) -> Self {
        if self.bottom || bound.bottom {
            return self.clone();
        }
        let new_hi = cmp::min(self.hi, bound.hi);
        if new_hi < self.lo {
            Self::make_bottom(self.bits)
        } else {
            Self::new(self.lo, new_hi, self.bits)
        }
    }

    /// Refine this interval given that `self <= bound` is false (i.e. `self > bound`).
    #[must_use]
    pub fn refine_sle_false(&self, bound: &Self) -> Self {
        if self.bottom || bound.bottom {
            return self.clone();
        }
        let new_lo = cmp::max(self.lo, bound.lo + 1);
        if new_lo > self.hi {
            Self::make_bottom(self.bits)
        } else {
            Self::new(new_lo, self.hi, self.bits)
        }
    }

    /// Refine this interval given that `self == other` is true.
    #[must_use]
    pub fn refine_eq_true(&self, other: &Self) -> Self {
        // Intersection with the other interval
        self.meet(other)
    }

    /// Refine this interval given that `self == other` is false.
    #[must_use]
    pub fn refine_eq_false(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return self.clone();
        }
        // If other is a singleton and equals one of our bounds, we can refine
        if other.is_singleton() {
            // `self == other` is asserted FALSE. When `self` is the SAME
            // singleton, no concrete value satisfies the edge at all, so the
            // edge is infeasible and the sound answer is bottom. Both arms
            // below are guarded by `self.lo < self.hi`, which a singleton never
            // satisfies, so without this case an equal singleton falls through
            // to `self.clone()` and a provably dead branch stays alive. That is
            // the `error-reachable` wall: `(ICmpEq, false)` is exactly what
            // clang emits for `if (!(cond)) { reach_error(); }` (plans/213 §2a).
            //
            // `bits` must match. `Interval` carries no signedness tag, so bounds
            // from different widths are not comparable; requiring equal widths
            // can only make this MORE conservative, never less.
            if self.bits == other.bits && self.lo == other.lo && self.hi == other.hi {
                return Self::make_bottom(self.bits);
            }
            if self.lo == other.lo && self.lo < self.hi {
                return Self::new(self.lo + 1, self.hi, self.bits);
            }
            if self.hi == other.hi && self.lo < self.hi {
                return Self::new(self.lo, self.hi - 1, self.bits);
            }
        }
        // Otherwise, no refinement possible with intervals
        self.clone()
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Clamp the result to the bit-width range, or return top if it overflows.
    fn clamp_or_top(&self, lo: i128, hi: i128) -> Self {
        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        if lo < min_bound || hi > max_bound {
            // Overflow: widen to top
            Self::make_top(self.bits)
        } else {
            Self::new(lo, hi, self.bits)
        }
    }
}

impl AbstractDomain for Interval {
    fn bottom() -> Self {
        // Default to 32-bit; callers should use make_bottom(bits) for specific widths
        Self::make_bottom(32)
    }

    fn top() -> Self {
        // Default to 32-bit; callers should use make_top(bits) for specific widths
        Self::make_top(32)
    }

    fn is_bottom(&self) -> bool {
        self.bottom
    }

    fn is_top(&self) -> bool {
        !self.bottom && self.lo == signed_min(self.bits) && self.hi == signed_max(self.bits)
    }

    fn leq(&self, other: &Self) -> bool {
        if self.bottom {
            return true; // ⊥ ⊑ anything
        }
        if other.bottom {
            return false; // non-⊥ ⋢ ⊥
        }
        if self.bits == 1 && other.bits == 1 {
            // Booleans compare by truth value (`true` may be `1` or `-1`).
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return (!fa || fb) && (!ta || tb);
        }
        self.lo >= other.lo && self.hi <= other.hi
    }

    fn join(&self, other: &Self) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }
        if self.bits == 1 {
            // Truth-value join. The signed `i1` range is `[-1, 0]`, so the old
            // range clamp turned `[0,0] ⊔ [1,1]` into `[0,0]` and dropped `true`.
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return Self::from_truth(fa || fb, ta || tb, 1);
        }
        // Re-encode both sides at `self.bits` first (e.g. `[0,0](i32)` with
        // `⊤(i64)` from the `resolve_operand` fallback becomes `⊤(i32)`). This
        // used to CLAMP out-of-range bounds instead, which is unsound: an
        // unsigned-encoded `0xFFFFFFFF` clamped to `INT_MAX` loses the `-1` the
        // program holds. Wrapping is exact.
        let a = self.wrap_to_width(self.bits);
        let b = other.wrap_to_width(self.bits);
        let lo = cmp::min(a.lo, b.lo);
        let hi = cmp::max(a.hi, b.hi);
        Self::new(lo, hi, self.bits)
    }

    fn meet(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if self.bits == 1 {
            let ((fa, ta), (fb, tb)) = (self.truth(), other.truth());
            return Self::from_truth(fa && fb, ta && tb, 1);
        }
        let a = self.wrap_to_width(self.bits);
        let b = other.wrap_to_width(self.bits);
        let lo = cmp::max(a.lo, b.lo);
        let hi = cmp::min(a.hi, b.hi);
        if lo > hi {
            Self::make_bottom(self.bits)
        } else {
            Self::new(lo, hi, self.bits)
        }
    }

    fn widen(&self, other: &Self) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }
        // `i1` is finite: join terminates, and the signed bound `max = 0` would
        // drop `true` (= 1).
        if self.bits == 1 {
            return self.join(other);
        }
        let this = self.wrap_to_width(self.bits);
        let other = other.wrap_to_width(self.bits);

        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        // Standard widening: jump to ±∞ when bounds grow
        let lo = if other.lo < this.lo {
            min_bound
        } else {
            this.lo
        };
        let hi = if other.hi > this.hi {
            max_bound
        } else {
            this.hi
        };

        Self::new(lo, hi, self.bits)
    }

    fn narrow(&self, other: &Self) -> Self {
        if self.bottom {
            return Self::make_bottom(self.bits);
        }
        if other.bottom {
            return Self::make_bottom(self.bits);
        }
        if self.bits == 1 {
            return self.meet(other);
        }
        let this = self.wrap_to_width(self.bits);
        let other = other.wrap_to_width(self.bits);
        let (this, other) = (&this, &other);

        // Singleton specialization: if `other` is a singleton contained
        // within `self`, narrow directly to the singleton. This helps
        // post-loop assertions like `assert(i == 5)` where the loop
        // exit produces the exact bound value.
        if other.is_singleton() && other.lo >= this.lo && other.hi <= this.hi {
            return other.clone();
        }

        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        // Generalized narrowing: refine any bound where `other` is strictly
        // tighter. The classic formulation only refines at ±infinity, but
        // threshold widening can leave bounds at intermediate values (e.g.,
        // [0, 100] instead of [0, MAX]), preventing further tightening. This
        // generalized version is still monotonically decreasing (we only
        // shrink the interval) so the narrowing sequence converges.
        let lo = if this.lo == min_bound || other.lo > this.lo {
            other.lo
        } else {
            this.lo
        };
        let hi = if this.hi == max_bound || other.hi < this.hi {
            other.hi
        } else {
            this.hi
        };

        if lo > hi {
            Self::make_bottom(self.bits)
        } else {
            Self::new(lo, hi, self.bits)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction tests
    // =========================================================================

    #[test]
    fn singleton_interval() {
        let i = Interval::singleton(42, 32);
        assert!(!i.is_bottom());
        assert!(!i.is_top());
        assert!(i.is_singleton());
        assert_eq!(i.lo(), 42);
        assert_eq!(i.hi(), 42);
        assert!(i.contains(42));
        assert!(!i.contains(41));
    }

    #[test]
    fn bottom_interval() {
        let b = Interval::make_bottom(32);
        assert!(b.is_bottom());
        assert!(!b.is_top());
        assert!(!b.contains(0));
    }

    #[test]
    fn top_interval() {
        let t = Interval::make_top(32);
        assert!(!t.is_bottom());
        assert!(t.is_top());
        assert!(t.contains(0));
        assert!(t.contains(i128::from(i32::MAX)));
        assert!(t.contains(i128::from(i32::MIN)));
    }

    #[test]
    fn inverted_bounds_is_bottom() {
        let i = Interval::new(10, 5, 32);
        assert!(i.is_bottom());
    }

    // =========================================================================
    // Lattice law tests
    // =========================================================================

    #[test]
    fn join_commutativity() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(5, 15, 32);
        assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn join_idempotency() {
        let a = Interval::new(0, 10, 32);
        assert_eq!(a.join(&a), a);
    }

    #[test]
    fn join_with_bottom() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::make_bottom(32);
        assert_eq!(a.join(&b), a);
        assert_eq!(b.join(&a), a);
    }

    #[test]
    fn meet_commutativity() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(5, 15, 32);
        assert_eq!(a.meet(&b), b.meet(&a));
    }

    #[test]
    fn meet_idempotency() {
        let a = Interval::new(0, 10, 32);
        assert_eq!(a.meet(&a), a);
    }

    #[test]
    fn meet_disjoint_is_bottom() {
        let a = Interval::new(0, 5, 32);
        let b = Interval::new(10, 15, 32);
        assert!(a.meet(&b).is_bottom());
    }

    #[test]
    fn meet_with_top() {
        let a = Interval::new(0, 10, 32);
        let t = Interval::make_top(32);
        assert_eq!(a.meet(&t), a);
    }

    #[test]
    fn leq_reflexivity() {
        let a = Interval::new(0, 10, 32);
        assert!(a.leq(&a));
    }

    #[test]
    fn leq_bottom_leq_anything() {
        let b = Interval::make_bottom(32);
        let a = Interval::new(0, 10, 32);
        assert!(b.leq(&a));
        assert!(b.leq(&Interval::make_top(32)));
    }

    #[test]
    fn leq_anything_leq_top() {
        let a = Interval::new(0, 10, 32);
        let t = Interval::make_top(32);
        assert!(a.leq(&t));
    }

    #[test]
    fn leq_subset() {
        let a = Interval::new(3, 7, 32);
        let b = Interval::new(0, 10, 32);
        assert!(a.leq(&b));
        assert!(!b.leq(&a));
    }

    #[test]
    fn join_is_upper_bound() {
        let a = Interval::new(0, 5, 32);
        let b = Interval::new(3, 10, 32);
        let j = a.join(&b);
        assert!(a.leq(&j));
        assert!(b.leq(&j));
    }

    #[test]
    fn meet_is_lower_bound() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(3, 7, 32);
        let m = a.meet(&b);
        assert!(m.leq(&a));
        assert!(m.leq(&b));
    }

    // =========================================================================
    // Widening/narrowing tests
    // =========================================================================

    #[test]
    fn widen_soundness() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(0, 15, 32);
        let w = a.widen(&b);
        assert!(a.leq(&w), "a ⊑ widen(a, b)");
        assert!(b.leq(&w), "b ⊑ widen(a, b)");
    }

    #[test]
    fn widen_with_bottom() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::make_bottom(32);
        assert_eq!(a.widen(&b), a);
        assert_eq!(b.widen(&a), a);
    }

    #[test]
    fn widen_upper_bound_grows() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(0, 11, 32);
        let w = a.widen(&b);
        // Standard widening: hi grows → jump to max
        assert_eq!(w.hi(), signed_max(32));
        assert_eq!(w.lo(), 0);
    }

    #[test]
    fn widen_lower_bound_shrinks() {
        let a = Interval::new(5, 10, 32);
        let b = Interval::new(4, 10, 32);
        let w = a.widen(&b);
        // Standard widening: lo shrinks → jump to min
        assert_eq!(w.lo(), signed_min(32));
        assert_eq!(w.hi(), 10);
    }

    #[test]
    fn widen_stable_no_change() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(0, 10, 32);
        let w = a.widen(&b);
        assert_eq!(w, a);
    }

    #[test]
    fn narrow_refinement() {
        let a = Interval::make_top(32);
        let b = Interval::new(0, 100, 32);
        let n = a.narrow(&b);
        assert!(n.leq(&a), "narrow(a, b) ⊑ a");
        assert_eq!(n, b); // top narrows to b
    }

    #[test]
    fn narrow_partial() {
        // Generalized narrowing: refine any bound where other is tighter
        let a = Interval::new(signed_min(32), 100, 32);
        let b = Interval::new(5, 50, 32);
        let n = a.narrow(&b);
        assert_eq!(n.lo(), 5); // was -infinity, narrowed to 5
        assert_eq!(n.hi(), 50); // other.hi < self.hi, narrowed to 50
    }

    #[test]
    fn widen_with_thresholds() {
        let thresholds: BTreeSet<i128> = [0, 10, 100, 1000].iter().copied().collect();
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(0, 15, 32);
        let w = a.widen_with_thresholds(&b, &thresholds);
        // Upper bound 15 > 10, so find smallest threshold >= 15 → 100
        assert_eq!(w.lo(), 0);
        assert_eq!(w.hi(), 100);
    }

    #[test]
    fn widen_with_thresholds_lower() {
        let thresholds: BTreeSet<i128> = [-100, -10, 0, 10].iter().copied().collect();
        let a = Interval::new(0, 10, 32);
        let b = Interval::new(-5, 10, 32);
        let w = a.widen_with_thresholds(&b, &thresholds);
        // Lower bound -5 < 0, so find largest threshold <= -5 → -10
        assert_eq!(w.lo(), -10);
        assert_eq!(w.hi(), 10);
    }

    // =========================================================================
    // Arithmetic tests
    // =========================================================================

    #[test]
    fn add_intervals() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(10, 20, 32);
        let c = a.add(&b);
        assert_eq!(c.lo(), 11);
        assert_eq!(c.hi(), 25);
    }

    #[test]
    fn add_with_bottom() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::make_bottom(32);
        assert!(a.add(&b).is_bottom());
    }

    #[test]
    fn sub_intervals() {
        let a = Interval::new(10, 20, 32);
        let b = Interval::new(1, 5, 32);
        let c = a.sub(&b);
        assert_eq!(c.lo(), 5);
        assert_eq!(c.hi(), 19);
    }

    #[test]
    fn mul_intervals_positive() {
        let a = Interval::new(2, 4, 32);
        let b = Interval::new(3, 5, 32);
        let c = a.mul(&b);
        assert_eq!(c.lo(), 6);
        assert_eq!(c.hi(), 20);
    }

    #[test]
    fn mul_intervals_mixed_sign() {
        let a = Interval::new(-3, 2, 32);
        let b = Interval::new(1, 4, 32);
        let c = a.mul(&b);
        assert_eq!(c.lo(), -12);
        assert_eq!(c.hi(), 8);
    }

    #[test]
    fn sdiv_intervals() {
        let a = Interval::new(10, 20, 32);
        let b = Interval::new(2, 5, 32);
        let c = a.sdiv(&b);
        assert_eq!(c.lo(), 2); // 10/5
        assert_eq!(c.hi(), 10); // 20/2
    }

    #[test]
    fn sdiv_by_zero_is_top() {
        let a = Interval::new(10, 20, 32);
        let b = Interval::new(-1, 1, 32);
        assert!(a.sdiv(&b).is_top());
    }

    #[test]
    fn shl_intervals() {
        let a = Interval::new(1, 3, 32);
        let b = Interval::singleton(2, 32);
        let c = a.shl(&b);
        assert_eq!(c.lo(), 4);
        assert_eq!(c.hi(), 12);
    }

    #[test]
    fn bitand_non_negative() {
        let a = Interval::new(0, 15, 32);
        let b = Interval::new(0, 7, 32);
        let c = a.bitand(&b);
        assert_eq!(c.lo(), 0);
        assert_eq!(c.hi(), 7); // min(15, 7)
    }

    // =========================================================================
    // Cast tests
    // =========================================================================

    #[test]
    fn zext_interval() {
        let a = Interval::new(0, 255, 8);
        let b = a.zext(32);
        assert_eq!(b.lo(), 0);
        assert_eq!(b.hi(), 255);
        assert_eq!(b.bits(), 32);
    }

    #[test]
    fn sext_interval() {
        let a = Interval::new(-5, 10, 8);
        let b = a.sext(32);
        assert_eq!(b.lo(), -5);
        assert_eq!(b.hi(), 10);
        assert_eq!(b.bits(), 32);
    }

    #[test]
    fn trunc_fits() {
        let a = Interval::new(0, 100, 32);
        let b = a.trunc(8);
        // Fits in i8 [-128, 127]
        assert_eq!(b.lo(), 0);
        assert_eq!(b.hi(), 100);
        assert_eq!(b.bits(), 8);
    }

    #[test]
    fn trunc_overflow_is_top() {
        let a = Interval::new(0, 500, 32);
        let b = a.trunc(8);
        // 500 > 127 → top for i8
        assert!(b.is_top());
        assert_eq!(b.bits(), 8);
    }

    // =========================================================================
    // Range check tests
    // =========================================================================

    #[test]
    fn within_range() {
        let a = Interval::new(0, 9, 32);
        assert!(a.within_range(0, 10));
        assert!(!a.within_range(0, 9));
        assert!(!a.within_range(1, 10));
    }

    #[test]
    fn may_overlap() {
        let a = Interval::new(5, 15, 32);
        assert!(a.may_overlap(0, 10));
        assert!(a.may_overlap(10, 20));
        assert!(!a.may_overlap(16, 20));
        assert!(!a.may_overlap(0, 5));
    }

    #[test]
    fn contains_zero() {
        let a = Interval::new(-5, 5, 32);
        assert!(a.contains_zero());

        let b = Interval::new(1, 10, 32);
        assert!(!b.contains_zero());

        let c = Interval::new(-10, -1, 32);
        assert!(!c.contains_zero());

        let d = Interval::singleton(0, 32);
        assert!(d.contains_zero());

        let e = Interval::make_bottom(32);
        assert!(!e.contains_zero());
    }

    #[test]
    fn is_singleton_zero() {
        let a = Interval::singleton(0, 32);
        assert!(a.is_singleton_zero());

        let b = Interval::new(0, 1, 32);
        assert!(!b.is_singleton_zero());

        let c = Interval::singleton(1, 32);
        assert!(!c.is_singleton_zero());

        let d = Interval::make_bottom(32);
        assert!(!d.is_singleton_zero());
    }

    // =========================================================================
    // Comparison refinement tests
    // =========================================================================

    #[test]
    fn refine_slt_true_narrows() {
        let a = Interval::new(0, 100, 32);
        let b = Interval::singleton(50, 32);
        let r = a.refine_slt_true(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 49);
    }

    #[test]
    fn refine_slt_false_narrows() {
        let a = Interval::new(0, 100, 32);
        let b = Interval::singleton(50, 32);
        let r = a.refine_slt_false(&b);
        assert_eq!(r.lo(), 50);
        assert_eq!(r.hi(), 100);
    }

    #[test]
    fn refine_eq_true_intersection() {
        let a = Interval::new(0, 10, 32);
        let b = Interval::singleton(5, 32);
        let r = a.refine_eq_true(&b);
        assert_eq!(r, b);
    }

    #[test]
    fn refine_eq_false_shrinks_bounds() {
        let a = Interval::new(5, 10, 32);
        let b = Interval::singleton(5, 32);
        let r = a.refine_eq_false(&b);
        assert_eq!(r.lo(), 6);
        assert_eq!(r.hi(), 10);
    }

    /// RED (plans/213 §2a). `x != c` when `x` is already known to be exactly `c`
    /// is unsatisfiable, so the edge is infeasible and the refinement must be ⊥.
    /// Both refinement arms are guarded by `self.lo < self.hi`, so an EQUAL
    /// singleton falls through to `self.clone()` and the branch survives. This is
    /// the `error-reachable` wall: `(ICmpEq, false)` is exactly what clang emits
    /// for the canonical SV-COMP `if (!(cond)) { reach_error(); }`.
    #[test]
    fn refine_eq_false_equal_singletons_is_bottom() {
        let a = Interval::singleton(0, 32);
        let b = Interval::singleton(0, 32);
        assert!(a.refine_eq_false(&b).is_bottom());
    }

    /// Same shape at a non-zero constant, to pin that the fix is about singleton
    /// EQUALITY and not about zero specifically.
    #[test]
    fn refine_eq_false_equal_singletons_nonzero_is_bottom() {
        let a = Interval::singleton(42, 32);
        let b = Interval::singleton(42, 32);
        assert!(a.refine_eq_false(&b).is_bottom());
    }

    /// GUARD against over-fixing: DISTINCT singletons satisfy `!=` perfectly
    /// well, so the refinement must leave `self` alone rather than go to ⊥.
    /// A ⊥ here would declare a reachable branch dead — a wrong TRUE.
    #[test]
    fn refine_eq_false_distinct_singletons_unchanged() {
        let a = Interval::singleton(3, 32);
        let b = Interval::singleton(5, 32);
        let r = a.refine_eq_false(&b);
        assert!(!r.is_bottom());
        assert_eq!(r.lo(), 3);
        assert_eq!(r.hi(), 3);
    }

    /// GUARD: a singleton refined against a WIDER interval that contains it is
    /// not refinable in the interval domain, but it is certainly not ⊥.
    #[test]
    fn refine_eq_false_singleton_vs_range_unchanged() {
        let a = Interval::singleton(5, 32);
        let b = Interval::new(0, 10, 32);
        let r = a.refine_eq_false(&b);
        assert!(!r.is_bottom());
        assert_eq!(r.lo(), 5);
    }

    // =========================================================================
    // Bit-width edge cases
    // =========================================================================

    #[test]
    fn i8_range() {
        let t = Interval::make_top(8);
        assert_eq!(t.lo(), -128);
        assert_eq!(t.hi(), 127);
    }

    #[test]
    fn i16_range() {
        let t = Interval::make_top(16);
        assert_eq!(t.lo(), -32768);
        assert_eq!(t.hi(), 32767);
    }

    #[test]
    fn i64_range() {
        let t = Interval::make_top(64);
        assert_eq!(t.lo(), i128::from(i64::MIN));
        assert_eq!(t.hi(), i128::from(i64::MAX));
    }

    #[test]
    fn i8_overflow_add_becomes_top() {
        let a = Interval::singleton(127, 8);
        let b = Interval::singleton(1, 8);
        let c = a.add(&b);
        // 127 + 1 = 128 > 127 (i8 max) → top
        assert!(c.is_top());
    }

    // =========================================================================
    // ICmp comparison tests
    // =========================================================================

    // --- icmp_sle (signed ≤) ---

    #[test]
    fn icmp_sle_always_true() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_sle(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sle_always_false() {
        let a = Interval::new(6, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_sle(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 0);
    }

    #[test]
    fn icmp_sle_unknown() {
        let a = Interval::new(3, 8, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_sle(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sle_bottom() {
        let a = Interval::make_bottom(32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_sle(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_sgt (signed >) ---

    #[test]
    fn icmp_sgt_always_true() {
        let a = Interval::new(6, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_sgt(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sgt_always_false() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_sgt(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 0);
    }

    #[test]
    fn icmp_sgt_unknown() {
        let a = Interval::new(3, 8, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_sgt(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sgt_bottom() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::make_bottom(32);
        let r = a.icmp_sgt(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_sge (signed ≥) ---

    #[test]
    fn icmp_sge_always_true() {
        let a = Interval::new(5, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_sge(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sge_always_false() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(6, 10, 32);
        let r = a.icmp_sge(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 0);
    }

    #[test]
    fn icmp_sge_unknown() {
        let a = Interval::new(3, 8, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_sge(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_sge_bottom() {
        let a = Interval::make_bottom(32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_sge(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_eq (equal) ---

    #[test]
    fn icmp_eq_singletons_equal() {
        let a = Interval::singleton(42, 32);
        let b = Interval::singleton(42, 32);
        let r = a.icmp_eq(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_eq_disjoint() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(6, 10, 32);
        let r = a.icmp_eq(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 0);
    }

    #[test]
    fn icmp_eq_overlapping() {
        let a = Interval::new(1, 10, 32);
        let b = Interval::new(5, 15, 32);
        let r = a.icmp_eq(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_eq_bottom() {
        let a = Interval::make_bottom(32);
        let b = Interval::singleton(42, 32);
        let r = a.icmp_eq(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_ne (not equal) ---

    #[test]
    fn icmp_ne_disjoint() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(6, 10, 32);
        let r = a.icmp_ne(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ne_singletons_equal() {
        let a = Interval::singleton(42, 32);
        let b = Interval::singleton(42, 32);
        let r = a.icmp_ne(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 0);
    }

    #[test]
    fn icmp_ne_overlapping() {
        let a = Interval::new(1, 10, 32);
        let b = Interval::new(5, 15, 32);
        let r = a.icmp_ne(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ne_bottom() {
        let a = Interval::singleton(42, 32);
        let b = Interval::make_bottom(32);
        let r = a.icmp_ne(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_ult (unsigned <) ---

    #[test]
    fn icmp_ult_both_positive() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(6, 10, 32);
        let r = a.icmp_ult(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ult_negative_fallback() {
        let a = Interval::new(-5, 5, 32);
        let b = Interval::new(6, 10, 32);
        let r = a.icmp_ult(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ult_bottom() {
        let a = Interval::make_bottom(32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_ult(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_ule (unsigned ≤) ---

    #[test]
    fn icmp_ule_both_positive() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(5, 10, 32);
        let r = a.icmp_ule(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ule_negative_fallback() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::new(-3, 10, 32);
        let r = a.icmp_ule(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ule_bottom() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::make_bottom(32);
        let r = a.icmp_ule(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_ugt (unsigned >) ---

    #[test]
    fn icmp_ugt_both_positive() {
        let a = Interval::new(6, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_ugt(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ugt_negative_fallback() {
        let a = Interval::new(-10, -1, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_ugt(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_ugt_bottom() {
        let a = Interval::make_bottom(32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_ugt(&b);
        assert!(r.is_bottom());
    }

    // --- icmp_uge (unsigned ≥) ---

    #[test]
    fn icmp_uge_both_positive() {
        let a = Interval::new(5, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_uge(&b);
        assert_eq!(r.lo(), 1);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_uge_negative_fallback() {
        let a = Interval::new(-1, 10, 32);
        let b = Interval::new(1, 5, 32);
        let r = a.icmp_uge(&b);
        assert_eq!(r.lo(), 0);
        assert_eq!(r.hi(), 1);
    }

    #[test]
    fn icmp_uge_bottom() {
        let a = Interval::new(1, 5, 32);
        let b = Interval::make_bottom(32);
        let r = a.icmp_uge(&b);
        assert!(r.is_bottom());
    }

    // =========================================================================
    // Display tests
    // =========================================================================

    #[test]
    fn display_formats() {
        assert_eq!(format!("{}", Interval::make_bottom(32)), "⊥");
        assert_eq!(format!("{}", Interval::make_top(32)), "⊤");
        assert_eq!(format!("{}", Interval::singleton(42, 32)), "[42]");
        assert_eq!(format!("{}", Interval::new(1, 10, 32)), "[1, 10]");
    }

    // =========================================================================
    // Bitwise shift improvement tests
    // =========================================================================

    /// CORRECTED (soundness): `[0, 2^31-1] << 30` at 32 bits wraps — `1 << 31` is
    /// `INT_MIN` — so the result must contain negative values. The old answer
    /// `[0, unsigned_max(32)]` held them in an unsigned encoding the signed
    /// refiners misread (`x < 0` became infeasible).
    #[test]
    fn shl_non_negative_overflowing_width_is_top() {
        let a = Interval::new(0, i128::from(i32::MAX), 32);
        let shift = Interval::singleton(30, 32);
        let result = a.shl(&shift);
        assert!(result.contains(i128::from(i32::MIN)));
        assert!(result.hi <= signed_max(32), "canonical (signed) encoding");
    }

    /// `[0, 1] << 31` at 32 bits is `{0, INT_MIN}`: never above `INT_MAX`.
    #[test]
    fn shl_into_sign_bit_is_negative() {
        let a = Interval::new(0, 1, 32);
        let result = a.shl(&Interval::singleton(31, 32));
        assert!(result.contains(i128::from(i32::MIN)));
        assert!(result.hi <= signed_max(32));
    }

    #[test]
    fn shl_small_shift_exact() {
        // [0, 1] << 30 = {0, 2^30} = [0, 1073741824]
        let a = Interval::new(0, 1, 32);
        let shift = Interval::singleton(30, 32);
        let result = a.shl(&shift);
        assert!(!result.is_top());
        assert_eq!(result.lo, 0);
        assert_eq!(result.hi, 1_073_741_824);
    }

    #[test]
    fn shl_then_bitor_not_top() {
        // Simulates: (rand() << 30) | (rand() << 15) | rand()
        // rand() is [0, RAND_MAX] ~ [0, 2^31-1]
        let bit_0_1 = Interval::new(0, 1, 32);

        let shifted_30 = bit_0_1.shl(&Interval::singleton(30, 32));
        assert!(!shifted_30.is_top(), "shift should not be TOP");

        let shifted_15 = bit_0_1.shl(&Interval::singleton(15, 32));
        assert!(!shifted_15.is_top(), "shift should not be TOP");

        let small = Interval::new(0, 65535, 32);
        let combined = shifted_30.bitor(&shifted_15).bitor(&small);
        assert!(!combined.is_top(), "combined OR should not be TOP");
    }

    #[test]
    fn shl_negative_still_uses_signed_range() {
        // Negative values should still use signed range checking
        let a = Interval::new(-1, 1, 32);
        let shift = Interval::singleton(30, 32);
        let result = a.shl(&shift);
        // With negative input, clamp_or_top is used — may be TOP and that's OK
        // Just verify it doesn't panic
        assert_eq!(result.bits, 32);
    }

    // =========================================================================
    // Unsigned / encoding soundness (no-overflow wrong-TRUE o1 and its family)
    // =========================================================================

    /// o1: `unsigned u = (unsigned)nondet_longlong(); int x = (int)(u >> 1);`
    /// `u` is the full 32-bit range — `⊤(i32)` in the signed encoding — so `x`
    /// reaches `INT_MAX` and `x + x` overflows. The old `lshr` read `max(hi, 0)`
    /// and answered `[0, 2^30 - 1]`.
    #[test]
    fn lshr_of_full_unsigned_range_reaches_int_max() {
        let u = Interval::make_top(64).trunc(32);
        let x = u.lshr(&Interval::singleton(1, 32));
        assert_eq!(x.lo(), 0);
        assert_eq!(x.hi(), i128::from(i32::MAX));
        let (_, sum_hi) = x.add_unwrapped(&x);
        assert!(sum_hi > signed_max(32), "x + x must be seen to overflow");
    }

    /// A negative-only interval is a run of large unsigned values.
    #[test]
    fn lshr_of_negative_value_is_large_unsigned() {
        // -2 at 32 bits is 0xFFFFFFFE; >> 1 is 0x7FFFFFFF.
        let x = Interval::singleton(-2, 32).lshr(&Interval::singleton(1, 32));
        assert_eq!(x, Interval::singleton(i128::from(i32::MAX), 32));
    }

    #[test]
    fn lshr_by_zero_keeps_negative_values() {
        let x = Interval::new(-5, -1, 32).lshr(&Interval::singleton(0, 32));
        assert_eq!(x, Interval::new(-5, -1, 32));
    }

    #[test]
    fn lshr_non_constant_shift_of_mixed_sign_is_sound() {
        let x = Interval::new(-1, 1, 32).lshr(&Interval::new(0, 3, 32));
        // 0xFFFFFFFF >> 0 is -1 again, >> 1 is INT_MAX.
        assert!(x.contains(-1));
        assert!(x.contains(i128::from(i32::MAX)));
    }

    #[test]
    fn lshr_shift_out_of_width_is_top() {
        let x = Interval::new(0, 10, 32).lshr(&Interval::singleton(32, 32));
        assert!(x.is_top());
    }

    #[test]
    fn udiv_of_negative_dividend_is_large() {
        // 0xFFFFFFFF /u 2 = 0x7FFFFFFF. The old code said 0.
        let q = Interval::singleton(-1, 32).udiv(&Interval::singleton(2, 32));
        assert_eq!(q, Interval::singleton(i128::from(i32::MAX), 32));
    }

    #[test]
    fn udiv_of_mixed_sign_dividend_covers_int_max() {
        let q = Interval::new(-1, 10, 32).udiv(&Interval::singleton(2, 32));
        assert!(q.contains(i128::from(i32::MAX)));
        assert!(q.contains(0));
    }

    #[test]
    fn udiv_by_negative_divisor_is_small() {
        // x /u 0xFFFFFFFE is 0 or 1.
        let q = Interval::new(0, 100, 32).udiv(&Interval::singleton(-2, 32));
        assert!(q.contains(0));
        assert!(q.leq(&Interval::new(0, 1, 32)));
    }

    #[test]
    fn urem_by_negative_divisor_keeps_large_remainder() {
        // 0xFFFFFFFD %u 0xFFFFFFFE = 0xFFFFFFFD (= -3). The old `|divisor|` read
        // bounded the remainder by 1.
        let r = Interval::singleton(-3, 32).urem(&Interval::singleton(-2, 32));
        assert!(r.contains(-3));
    }

    #[test]
    fn urem_of_negative_dividend_by_small_divisor() {
        let r = Interval::new(-10, -1, 32).urem(&Interval::singleton(8, 32));
        assert_eq!(r, Interval::new(0, 7, 32));
    }

    #[test]
    fn urem_dividend_below_divisor_is_identity() {
        let r = Interval::new(2, 5, 32).urem(&Interval::singleton(8, 32));
        assert_eq!(r, Interval::new(2, 5, 32));
    }

    #[test]
    fn trunc_to_i1_keeps_the_low_bit() {
        assert_eq!(
            Interval::singleton(2, 32).trunc(1),
            Interval::singleton(0, 1)
        );
        assert_eq!(
            Interval::singleton(3, 32).trunc(1),
            Interval::singleton(1, 1)
        );
        assert_eq!(
            Interval::singleton(-1, 32).trunc(1),
            Interval::singleton(1, 1)
        );
        assert_eq!(Interval::new(1, 2, 32).trunc(1), Interval::new(0, 1, 1));
        assert_eq!(Interval::new(0, 1, 32).trunc(1), Interval::new(0, 1, 1));
    }

    #[test]
    fn wrap_to_width_unsigned_encoding_becomes_negative() {
        let w = Interval::singleton(i128::from(u32::MAX), 32).wrap_to_width(32);
        assert_eq!(w, Interval::singleton(-1, 32));
        let straddle = Interval::new(0, 1i128 << 31, 32).wrap_to_width(32);
        assert!(straddle.is_top());
        let high = Interval::new(1i128 << 31, (1i128 << 32) - 1, 32).wrap_to_width(32);
        assert_eq!(high, Interval::new(i128::from(i32::MIN), -1, 32));
    }

    #[test]
    fn wrap_to_width_full_period_is_top() {
        assert!(Interval::new(0, 255, 8).wrap_to_width(8).is_top());
        assert_eq!(
            Interval::new(256, 260, 8).wrap_to_width(8),
            Interval::new(0, 4, 8)
        );
    }

    #[test]
    fn join_wraps_instead_of_clamping() {
        // 0xFFFFFFFF held unsigned must join as -1, not be clamped to INT_MAX.
        let a = Interval::singleton(0, 32);
        let b = Interval::singleton(i128::from(u32::MAX), 32);
        let j = a.join(&b);
        assert!(j.contains(-1));
        assert!(j.contains(0));
    }

    /// `[0,0] ⊔ [1,1]` on `i1` used to clamp to `[0,0]` (signed `i1` max is 0),
    /// dropping `true` — e.g. the phi of `c = a && b`.
    #[test]
    fn bool_join_keeps_true() {
        let f = Interval::singleton(0, 1);
        let t = Interval::singleton(1, 1);
        let j = f.join(&t);
        assert!(j.contains(0) && j.contains(1));
        let w = f.widen(&t);
        assert!(w.contains(0) && w.contains(1));
        // `-1` is the sign-extended IR constant `true`.
        let n = Interval::singleton(-1, 1);
        assert!(f.join(&n).contains(1));
        assert!(t.leq(&n) && n.leq(&t));
    }

    #[test]
    fn bool_narrow_does_not_drop_true() {
        let both = Interval::new(0, 1, 1);
        let n = both.narrow(&Interval::new(-1, 0, 1));
        assert!(n.contains(1), "[-1, 0] at i1 is {{true, false}}");
    }

    #[test]
    fn bitor_never_produces_unsigned_encoding() {
        let a = Interval::new(0, 1i128 << 30, 32);
        let r = a.bitor(&a);
        assert!(r.hi() <= signed_max(32));
        // i1 or: {false} | {true} = {true}.
        let t = Interval::singleton(0, 1).bitor(&Interval::singleton(1, 1));
        assert_eq!(t, Interval::singleton(1, 1));
        let x = Interval::singleton(1, 1).bitxor(&Interval::singleton(-1, 1));
        assert_eq!(x, Interval::singleton(0, 1), "true ^ true = false");
    }

    #[test]
    fn bitand_with_non_negative_mask() {
        let r = Interval::make_top(32).bitand(&Interval::singleton(255, 32));
        assert_eq!(r, Interval::new(0, 255, 32));
    }

    #[test]
    fn zext_negative_uses_source_width() {
        // zext i8 -1 to i32 = 255.
        assert_eq!(
            Interval::singleton(-1, 8).zext(32),
            Interval::singleton(255, 32)
        );
        // zext i32 [-1, 0] to i64 = {0, 0xFFFFFFFF}.
        let z = Interval::new(-1, 0, 32).zext(64);
        assert!(z.contains(i128::from(u32::MAX)) && z.contains(0));
    }

    /// The label is a placeholder (`⊤(i64)` for a loaded `i32`): never mask by
    /// it. The old code produced `[0, i128::MAX]` tagged with the target.
    #[test]
    fn zext_with_untrusted_label_stays_in_target_range() {
        let z = Interval::new(-5, 3, 64).zext(64);
        assert!(z.lo() >= 0 && z.hi() <= signed_max(64));
        assert!(z.contains(3));
    }

    #[test]
    fn bool_casts_use_truth_values() {
        assert_eq!(
            Interval::singleton(-1, 1).zext(32),
            Interval::singleton(1, 32)
        );
        assert_eq!(
            Interval::singleton(1, 1).sext(32),
            Interval::singleton(-1, 32)
        );
        let s = Interval::new(0, 1, 1).sext(32);
        assert!(s.contains(-1) && s.contains(0));
    }

    #[test]
    fn trunc_wraps_exactly() {
        assert_eq!(
            Interval::singleton(255, 32).trunc(8),
            Interval::singleton(-1, 8)
        );
        assert_eq!(
            Interval::new(256, 300, 32).trunc(8),
            Interval::new(0, 44, 8)
        );
    }
}
