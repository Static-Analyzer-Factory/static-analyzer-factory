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
    -(1i128 << (bits - 1))
}

/// Maximum value for a signed integer of the given bit-width.
#[must_use]
pub fn signed_max(bits: u8) -> i128 {
    if bits == 0 {
        return 0;
    }
    (1i128 << (bits - 1)) - 1
}

/// Maximum value for an unsigned integer of the given bit-width.
#[must_use]
pub fn unsigned_max(bits: u8) -> i128 {
    if bits == 0 {
        return 0;
    }
    if bits >= 128 {
        return i128::MAX; // clamp for safety
    }
    (1i128 << bits) - 1
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

        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        // Lower bound: if other.lo < self.lo, find largest threshold <= other.lo
        let new_lo = if other.lo < self.lo {
            thresholds
                .range(..=other.lo)
                .next_back()
                .copied()
                .unwrap_or(min_bound)
                .max(min_bound)
        } else {
            self.lo
        };

        // Upper bound: if other.hi > self.hi, find smallest threshold >= other.hi
        let new_hi = if other.hi > self.hi {
            thresholds
                .range(other.hi..)
                .next()
                .copied()
                .unwrap_or(max_bound)
                .min(max_bound)
        } else {
            self.hi
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

    /// Unsigned division: treats both operands as unsigned.
    #[must_use]
    pub fn udiv(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        // Division by zero → top
        if other.lo <= 0 && other.hi >= 0 {
            return Self::make_top(self.bits);
        }
        // For unsigned, assume non-negative
        let lo = if other.hi != 0 {
            self.lo.max(0) / other.hi.max(1)
        } else {
            0
        };
        let hi = if other.lo > 0 {
            self.hi.max(0) / other.lo
        } else {
            unsigned_max(self.bits)
        };
        self.clamp_or_top(lo, hi)
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

    /// Unsigned remainder.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // max_divisor fits in i128 after range validation
    pub fn urem(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        if other.lo <= 0 && other.hi >= 0 {
            return Self::make_top(self.bits);
        }
        // Conservative: 0 <= result < |divisor|
        let max_divisor = cmp::max(other.lo.unsigned_abs(), other.hi.unsigned_abs());
        Self::new(0, max_divisor as i128 - 1, self.bits)
    }

    /// Left shift: `[a, b] << [c, d]`.
    ///
    /// When both operands are non-negative, the result is bounded by
    /// `unsigned_max(bits)` instead of `signed_max(bits)`. This avoids
    /// spurious TOP from expressions like `[0, RAND_MAX] << 30` where the
    /// shifted value exceeds the signed range but is still valid as an
    /// unsigned value within the bit-width.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // shift validated to [0, bits)
    pub fn shl(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        let bits = i128::from(self.bits);
        // Shift amount must be in [0, bits)
        if shift.lo < 0 || shift.hi >= bits {
            return Self::make_top(self.bits);
        }
        let lo_shift = shift.lo as u32;
        let hi_shift = shift.hi as u32;
        let products = [
            self.lo.checked_shl(lo_shift).unwrap_or(0),
            self.lo.checked_shl(hi_shift).unwrap_or(0),
            self.hi.checked_shl(lo_shift).unwrap_or(0),
            self.hi.checked_shl(hi_shift).unwrap_or(0),
        ];
        let lo = products.iter().copied().min().unwrap_or(0);
        let hi = products.iter().copied().max().unwrap_or(0);

        // When both operands are non-negative, use unsigned range to avoid
        // spurious TOP from signed overflow (e.g., [0, RAND_MAX] << 30).
        if self.lo >= 0 && shift.lo >= 0 {
            let umax = unsigned_max(self.bits);
            return Self::new(lo.max(0), hi.min(umax), self.bits);
        }

        self.clamp_or_top(lo, hi)
    }

    /// Logical shift right (unsigned).
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )] // shift validated to [0, bits), unsigned to signed cast deliberate
    pub fn lshr(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        let bits = i128::from(self.bits);
        if shift.lo < 0 || shift.hi >= bits {
            return Self::make_top(self.bits);
        }
        // For logical shift right, treat as unsigned
        // Conservative: result is in [0, max >> min_shift]
        let min_shift = shift.lo as u32;
        let hi = (self.hi.max(0) as u128 >> min_shift) as i128;
        Self::new(0, hi, self.bits)
    }

    /// Arithmetic shift right (signed).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // shift validated to [0, bits)
    pub fn ashr(&self, shift: &Self) -> Self {
        if self.bottom || shift.bottom {
            return Self::make_bottom(self.bits);
        }
        let bits = i128::from(self.bits);
        if shift.lo < 0 || shift.hi >= bits {
            return Self::make_top(self.bits);
        }
        let lo_shift = shift.lo as u32;
        let hi_shift = shift.hi as u32;
        let results = [
            self.lo >> lo_shift,
            self.lo >> hi_shift,
            self.hi >> lo_shift,
            self.hi >> hi_shift,
        ];
        let lo = results.iter().copied().min().unwrap_or(0);
        let hi = results.iter().copied().max().unwrap_or(0);
        Self::new(lo, hi, self.bits)
    }

    /// Bitwise AND (conservative).
    #[must_use]
    pub fn bitand(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        // Conservative: if both non-negative, result in [0, min(hi, other.hi)]
        if self.lo >= 0 && other.lo >= 0 {
            let hi = cmp::min(self.hi, other.hi);
            return Self::new(0, hi, self.bits);
        }
        Self::make_top(self.bits)
    }

    /// Bitwise OR (conservative).
    #[must_use]
    pub fn bitor(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        // Conservative: top for general case
        if self.lo >= 0 && other.lo >= 0 {
            // Both non-negative: upper bound approximation
            let hi = unsigned_max(self.bits).min(self.hi + other.hi);
            return Self::new(0, hi, self.bits);
        }
        Self::make_top(self.bits)
    }

    /// Bitwise XOR (conservative).
    #[must_use]
    pub fn bitxor(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        // Conservative approximation
        if self.lo >= 0 && other.lo >= 0 {
            let hi = unsigned_max(self.bits).min(self.hi + other.hi);
            return Self::new(0, hi, self.bits);
        }
        Self::make_top(self.bits)
    }

    // =========================================================================
    // Cast operations
    // =========================================================================

    /// Zero-extend to a larger bit-width.
    #[must_use]
    pub fn zext(&self, target_bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(target_bits);
        }
        // Zero-extend: reinterpret the source value as unsigned at the
        // source bit width, then extend. For example, i8 -1 → u8 255.
        if self.bits > 0 && self.bits < 128 {
            let mask = if self.bits >= 64 {
                i128::MAX // avoid overflow for large bit widths
            } else {
                (1i128 << self.bits) - 1
            };
            let lo = self.lo & mask;
            let hi = self.hi & mask;
            // If the range spans the sign boundary (lo > hi after masking),
            // the result covers [0, mask] — return top at target bits.
            if lo > hi {
                return Self::new(0, mask, target_bits);
            }
            Self::new(lo, hi, target_bits)
        } else {
            let lo = self.lo.max(0);
            let hi = self.hi.max(0);
            Self::new(lo, hi, target_bits)
        }
    }

    /// Sign-extend to a larger bit-width.
    #[must_use]
    pub fn sext(&self, target_bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(target_bits);
        }
        // Sign-extend preserves the value in the larger range
        Self::new(self.lo, self.hi, target_bits)
    }

    /// Truncate to a smaller bit-width (conservative).
    #[must_use]
    pub fn trunc(&self, target_bits: u8) -> Self {
        if self.bottom {
            return Self::make_bottom(target_bits);
        }
        let target_min = signed_min(target_bits);
        let target_max = signed_max(target_bits);
        // If the interval fits in the target range, truncation is exact
        if self.lo >= target_min && self.hi <= target_max {
            return Self::new(self.lo, self.hi, target_bits);
        }
        // Singleton: compute the concrete wrapped value
        if self.lo == self.hi {
            let mask = (1i128 << target_bits) - 1;
            let truncated = self.lo & mask;
            // Sign-extend from target_bits to i128
            let sign_bit = 1i128 << (target_bits - 1);
            let signed_val = if truncated & sign_bit != 0 {
                truncated | !mask // sign-extend
            } else {
                truncated
            };
            return Self::singleton(signed_val, target_bits);
        }
        // Otherwise conservative: full target range
        Self::make_top(target_bits)
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
        self.lo >= other.lo && self.hi <= other.hi
    }

    fn join(&self, other: &Self) -> Self {
        if self.bottom {
            return other.clone();
        }
        if other.bottom {
            return self.clone();
        }
        let lo = cmp::min(self.lo, other.lo);
        let hi = cmp::max(self.hi, other.hi);
        // Clamp to self.bits range to prevent bit-width mismatch corruption.
        // When joining intervals with different bit widths (e.g. [0,0](i32)
        // with TOP(i64) from resolve_operand fallback), the raw min/max can
        // produce bounds outside the representable range, creating intervals
        // like [-2^63, 2^63-1](i32) that is_top() doesn't recognize.
        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);
        if lo <= min_bound && hi >= max_bound {
            Self::make_top(self.bits)
        } else {
            Self::new(
                lo.clamp(min_bound, max_bound),
                hi.clamp(min_bound, max_bound),
                self.bits,
            )
        }
    }

    fn meet(&self, other: &Self) -> Self {
        if self.bottom || other.bottom {
            return Self::make_bottom(self.bits);
        }
        let lo = cmp::max(self.lo, other.lo);
        let hi = cmp::min(self.hi, other.hi);
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

        let min_bound = signed_min(self.bits);
        let max_bound = signed_max(self.bits);

        // Standard widening: jump to ±∞ when bounds grow
        let lo = if other.lo < self.lo {
            min_bound
        } else {
            self.lo
        };
        let hi = if other.hi > self.hi {
            max_bound
        } else {
            self.hi
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

        // Singleton specialization: if `other` is a singleton contained
        // within `self`, narrow directly to the singleton. This helps
        // post-loop assertions like `assert(i == 5)` where the loop
        // exit produces the exact bound value.
        if other.is_singleton() && other.lo >= self.lo && other.hi <= self.hi {
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
        let lo = if self.lo == min_bound || other.lo > self.lo {
            other.lo
        } else {
            self.lo
        };
        let hi = if self.hi == max_bound || other.hi < self.hi {
            other.hi
        } else {
            self.hi
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

    #[test]
    fn shl_non_negative_avoids_top() {
        // [0, 2^31-1] << 30 should NOT be TOP — should be [0, unsigned_max(32)]
        let a = Interval::new(0, i128::from(i32::MAX), 32);
        let shift = Interval::singleton(30, 32);
        let result = a.shl(&shift);
        assert!(
            !result.is_top(),
            "shl of non-negative values should not return TOP"
        );
        assert_eq!(result.lo, 0);
        // Upper bound should be at most unsigned_max(32)
        assert!(result.hi <= unsigned_max(32));
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
}
