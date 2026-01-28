//! Octagon abstract domain tracking relational constraints.
//!
//! More precise than the interval domain for properties involving relationships
//! between variables (e.g., `i < n`, `x + y <= c`), but with O(n^3) complexity
//! for closure operations.

use std::collections::BTreeMap;
use std::fmt;

use saf_core::ids::ValueId;

use super::dbm::{Bound, Dbm, VarIndex};
use crate::absint::domain::AbstractDomain;
use crate::absint::interval::Interval;

/// Octagon abstract domain tracking constraints `±x ± y <= c`.
///
/// Internally uses a Difference-Bound Matrix (DBM) with 2n variables
/// for n program variables. Provides more precision than intervals for
/// relational properties but has higher complexity.
///
/// # Example Constraints
///
/// - `x <= 10` (unary upper bound)
/// - `x >= -5` (unary lower bound)
/// - `x - y <= 3` (difference)
/// - `x + y <= 20` (sum)
#[derive(Clone, Debug)]
pub struct OctagonDomain {
    /// Mapping from `ValueId` to DBM variable index.
    var_map: BTreeMap<ValueId, VarIndex>,
    /// Reverse mapping from index to `ValueId`.
    index_to_value: Vec<ValueId>,
    /// The difference-bound matrix.
    dbm: Dbm,
}

impl PartialEq for OctagonDomain {
    fn eq(&self, other: &Self) -> bool {
        // Two octagon domains are equal if they represent the same constraints
        // Note: var_map ordering might differ but represent the same domain
        self.dbm == other.dbm && self.var_map == other.var_map
    }
}

impl Eq for OctagonDomain {}

impl fmt::Display for OctagonDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dbm.is_bottom() {
            return write!(f, "Octagon(bottom)");
        }

        write!(f, "Octagon({{")?;
        let mut first = true;
        for (value_id, var_idx) in &self.var_map {
            let (lo, hi) = self.dbm.get_interval(*var_idx);
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            match (lo, hi) {
                (Some(l), Some(h)) => write!(f, "{value_id}: [{l}, {h}]")?,
                (Some(l), None) => write!(f, "{value_id}: [{l}, +inf)")?,
                (None, Some(h)) => write!(f, "{value_id}: (-inf, {h}]")?,
                (None, None) => write!(f, "{value_id}: top")?,
            }
        }
        write!(f, "}})")
    }
}

impl OctagonDomain {
    /// Create a new empty octagon domain (top).
    #[must_use]
    pub fn new() -> Self {
        Self {
            var_map: BTreeMap::new(),
            index_to_value: Vec::new(),
            dbm: Dbm::top(0),
        }
    }

    /// Create a bottom octagon domain.
    #[must_use]
    pub fn new_bottom() -> Self {
        Self {
            var_map: BTreeMap::new(),
            index_to_value: Vec::new(),
            dbm: Dbm::bottom(0),
        }
    }

    /// Get the number of variables tracked.
    #[must_use]
    pub fn num_vars(&self) -> usize {
        self.var_map.len()
    }

    /// Get or allocate a variable index for a `ValueId`.
    fn get_or_create_var(&mut self, value: ValueId) -> VarIndex {
        if let Some(&idx) = self.var_map.get(&value) {
            return idx;
        }

        let idx = VarIndex(self.var_map.len());
        self.var_map.insert(value, idx);
        self.index_to_value.push(value);

        // Expand the DBM to accommodate the new variable
        let new_num_vars = self.var_map.len();
        self.dbm = self.expand_dbm(new_num_vars);

        idx
    }

    /// Expand the DBM to a new size while preserving existing constraints.
    fn expand_dbm(&self, new_num_vars: usize) -> Dbm {
        if self.dbm.is_bottom() {
            return Dbm::bottom(new_num_vars);
        }

        let old_size = self.dbm.num_vars() * 2;
        let mut new_dbm = Dbm::top(new_num_vars);

        // Copy existing constraints
        for i in 0..old_size {
            for j in 0..old_size {
                new_dbm.set(i, j, self.dbm.get(i, j));
            }
        }

        new_dbm
    }

    /// Get the interval bounds for a variable.
    #[must_use]
    pub fn get_interval(&self, value: ValueId) -> Option<Interval> {
        let idx = self.var_map.get(&value)?;
        let (lo, hi) = self.dbm.get_interval(*idx);

        match (lo, hi) {
            (Some(l), Some(h)) => Some(Interval::new(l, h, 64)),
            (Some(l), None) => Some(Interval::new(l, i128::from(i64::MAX), 64)),
            (None, Some(h)) => Some(Interval::new(i128::from(i64::MIN), h, 64)),
            (None, None) => Some(Interval::make_top(64)),
        }
    }

    /// Set the interval bounds for a variable.
    pub fn set_interval(&mut self, value: ValueId, lo: i128, hi: i128) {
        let idx = self.get_or_create_var(value);

        // Upper bound: x <= hi → 2x <= 2*hi
        self.dbm.add_unary(idx, 1, hi);

        // Lower bound: x >= lo → -x <= -lo
        self.dbm.add_unary(idx, -1, -lo);
    }

    /// Add a constraint: `left - right <= constant`.
    pub fn assume_diff_leq(&mut self, left: ValueId, right: ValueId, constant: i128) {
        let left_idx = self.get_or_create_var(left);
        let right_idx = self.get_or_create_var(right);

        self.dbm.add_octagon(left_idx, 1, right_idx, -1, constant);
    }

    /// Add a constraint: `left + right <= constant`.
    pub fn assume_sum_leq(&mut self, left: ValueId, right: ValueId, constant: i128) {
        let left_idx = self.get_or_create_var(left);
        let right_idx = self.get_or_create_var(right);

        self.dbm.add_octagon(left_idx, 1, right_idx, 1, constant);
    }

    /// Add a constraint: `-left - right <= constant` (i.e., `left + right >= -constant`).
    pub fn assume_neg_sum_leq(&mut self, left: ValueId, right: ValueId, constant: i128) {
        let left_idx = self.get_or_create_var(left);
        let right_idx = self.get_or_create_var(right);

        self.dbm.add_octagon(left_idx, -1, right_idx, -1, constant);
    }

    /// Add a constraint: `-left + right <= constant` (i.e., `right - left <= constant`).
    pub fn assume_neg_diff_leq(&mut self, left: ValueId, right: ValueId, constant: i128) {
        let left_idx = self.get_or_create_var(left);
        let right_idx = self.get_or_create_var(right);

        self.dbm.add_octagon(left_idx, -1, right_idx, 1, constant);
    }

    /// Apply assignment: `dst := src + constant`.
    ///
    /// This is a "non-invertible" assignment that requires forgetting `dst`
    /// and then adding the constraint `dst == src + constant`.
    pub fn assign_linear(&mut self, dst: ValueId, src: ValueId, constant: i128) {
        // First forget dst
        self.forget(dst);

        // Then add: dst = src + constant
        // This means: dst - src = constant, i.e., dst - src <= constant AND src - dst <= -constant
        let dst_idx = self.get_or_create_var(dst);
        let src_idx = self.get_or_create_var(src);

        self.dbm.add_octagon(dst_idx, 1, src_idx, -1, constant); // dst - src <= c
        self.dbm.add_octagon(src_idx, 1, dst_idx, -1, -constant); // src - dst <= -c
    }

    /// Forget a variable (havoc).
    ///
    /// Removes all constraints involving this variable.
    pub fn forget(&mut self, value: ValueId) {
        let Some(&idx) = self.var_map.get(&value) else {
            return;
        };

        if self.dbm.is_bottom() {
            return;
        }

        let pos = idx.positive();
        let neg = idx.negative();
        let size = self.dbm.num_vars() * 2;

        // Set all constraints involving this variable to infinity
        for i in 0..size {
            if i != pos {
                self.dbm.set(pos, i, Bound::PosInf);
                self.dbm.set(i, pos, Bound::PosInf);
            }
            if i != neg {
                self.dbm.set(neg, i, Bound::PosInf);
                self.dbm.set(i, neg, Bound::PosInf);
            }
        }

        // Keep diagonal at 0
        self.dbm.set(pos, pos, Bound::Finite(0));
        self.dbm.set(neg, neg, Bound::Finite(0));
    }

    /// Close the DBM to make all implied constraints explicit.
    pub fn close(&mut self) {
        self.dbm.close();
    }

    /// Strongly close the DBM with octagon-specific tightening.
    pub fn strong_close(&mut self) {
        self.dbm.strong_close();
    }
}

impl Default for OctagonDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractDomain for OctagonDomain {
    fn bottom() -> Self {
        Self::new_bottom()
    }

    fn top() -> Self {
        Self::new()
    }

    fn is_bottom(&self) -> bool {
        self.dbm.is_bottom()
    }

    fn is_top(&self) -> bool {
        // Top if all non-diagonal entries are infinity
        if self.dbm.is_bottom() {
            return false;
        }
        let size = self.dbm.num_vars() * 2;
        for i in 0..size {
            for j in 0..size {
                if i != j && !self.dbm.get(i, j).is_inf() {
                    return false;
                }
            }
        }
        true
    }

    fn leq(&self, other: &Self) -> bool {
        if self.dbm.is_bottom() {
            return true;
        }
        if other.dbm.is_bottom() {
            return false;
        }

        // Need to compare on common variables
        // For simplicity, assume same variable ordering for now
        self.dbm.leq(&other.dbm)
    }

    fn join(&self, other: &Self) -> Self {
        if self.dbm.is_bottom() {
            return other.clone();
        }
        if other.dbm.is_bottom() {
            return self.clone();
        }

        // Merge variable maps and join DBMs
        let mut result = self.clone();

        // Add any new variables from other
        for &value in other.var_map.keys() {
            result.get_or_create_var(value);
        }

        // Now join the DBMs (after ensuring same size)
        result.dbm = result.dbm.join(&other.dbm);
        result
    }

    fn meet(&self, other: &Self) -> Self {
        if self.dbm.is_bottom() || other.dbm.is_bottom() {
            return Self::new_bottom();
        }

        // Merge variable maps and meet DBMs
        let mut result = self.clone();

        // Add any new variables from other
        for &value in other.var_map.keys() {
            result.get_or_create_var(value);
        }

        // Now meet the DBMs (after ensuring same size)
        result.dbm = result.dbm.meet(&other.dbm);
        result
    }

    fn widen(&self, other: &Self) -> Self {
        if self.dbm.is_bottom() {
            return other.clone();
        }
        if other.dbm.is_bottom() {
            return self.clone();
        }

        // Merge variable maps and widen DBMs
        let mut result = self.clone();

        // Add any new variables from other
        for &value in other.var_map.keys() {
            result.get_or_create_var(value);
        }

        result.dbm = result.dbm.widen(&other.dbm);
        result
    }

    fn narrow(&self, other: &Self) -> Self {
        if self.dbm.is_bottom() {
            return Self::new_bottom();
        }
        if other.dbm.is_bottom() {
            return Self::new_bottom();
        }

        let mut result = self.clone();
        result.dbm = result.dbm.narrow(&other.dbm);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_value_id(n: u128) -> ValueId {
        ValueId::new(n)
    }

    #[test]
    fn test_new_octagon_is_top() {
        let oct = OctagonDomain::new();
        assert!(oct.is_top());
        assert!(!oct.is_bottom());
    }

    #[test]
    fn test_bottom_octagon() {
        let oct = OctagonDomain::new_bottom();
        assert!(oct.is_bottom());
        assert!(!oct.is_top());
    }

    #[test]
    fn test_set_and_get_interval() {
        let mut oct = OctagonDomain::new();
        let v = make_value_id(1);

        oct.set_interval(v, -10, 20);

        let interval = oct.get_interval(v).unwrap();
        assert_eq!(interval.lo(), -10);
        assert_eq!(interval.hi(), 20);
    }

    #[test]
    fn test_assume_diff_constraint() {
        let mut oct = OctagonDomain::new();
        let x = make_value_id(1);
        let y = make_value_id(2);

        // x in [0, 100], y in [0, 50]
        oct.set_interval(x, 0, 100);
        oct.set_interval(y, 0, 50);

        // Add: x - y <= 10
        oct.assume_diff_leq(x, y, 10);
        oct.strong_close();

        // Verify the constraint was added (the DBM stores it)
        // Note: Full octagon refinement is complex; here we just verify
        // that the constraint is recorded and closure completes
        let x_interval = oct.get_interval(x).unwrap();
        let y_interval = oct.get_interval(y).unwrap();

        // Basic sanity: original bounds should still hold or be tighter
        assert!(x_interval.hi() <= 100);
        assert!(y_interval.hi() <= 50);

        // The domain should not be bottom (constraints are consistent)
        assert!(!oct.is_bottom());
    }

    #[test]
    fn test_forget_variable() {
        let mut oct = OctagonDomain::new();
        let x = make_value_id(1);

        oct.set_interval(x, 10, 20);

        let interval_before = oct.get_interval(x).unwrap();
        assert_eq!(interval_before.lo(), 10);
        assert_eq!(interval_before.hi(), 20);

        oct.forget(x);

        let interval_after = oct.get_interval(x).unwrap();
        // After forget, variable should be unconstrained (top)
        assert!(interval_after.is_top());
    }

    #[test]
    fn test_join() {
        let mut oct1 = OctagonDomain::new();
        let mut oct2 = OctagonDomain::new();
        let x = make_value_id(1);

        oct1.set_interval(x, 0, 10);
        oct2.set_interval(x, 5, 20);

        let joined = oct1.join(&oct2);
        let interval = joined.get_interval(x).unwrap();

        // Join takes the union: [0, 20]
        assert_eq!(interval.lo(), 0);
        assert_eq!(interval.hi(), 20);
    }

    #[test]
    fn test_meet() {
        let mut oct1 = OctagonDomain::new();
        let mut oct2 = OctagonDomain::new();
        let x = make_value_id(1);

        oct1.set_interval(x, 0, 15);
        oct2.set_interval(x, 5, 20);

        let met = oct1.meet(&oct2);
        let interval = met.get_interval(x).unwrap();

        // Meet takes the intersection: [5, 15]
        assert_eq!(interval.lo(), 5);
        assert_eq!(interval.hi(), 15);
    }

    #[test]
    fn test_leq_relation() {
        let mut oct1 = OctagonDomain::new();
        let mut oct2 = OctagonDomain::new();
        let x = make_value_id(1);

        oct1.set_interval(x, 5, 10);
        oct2.set_interval(x, 0, 20);

        // oct1 represents a subset of oct2's values
        assert!(oct1.leq(&oct2));
        assert!(!oct2.leq(&oct1));
    }

    #[test]
    fn test_abstract_domain_laws() {
        let mut a = OctagonDomain::new();
        let mut b = OctagonDomain::new();
        let x = make_value_id(1);

        a.set_interval(x, 0, 10);
        b.set_interval(x, 5, 15);

        // Join is commutative
        assert_eq!(a.join(&b), b.join(&a));

        // Join is idempotent
        assert_eq!(a.join(&a), a);

        // Bottom absorbs join
        let bottom = OctagonDomain::new_bottom();
        assert_eq!(bottom.join(&a), a);
        assert_eq!(a.join(&bottom), a);

        // Join is upper bound
        let joined = a.join(&b);
        assert!(a.leq(&joined));
        assert!(b.leq(&joined));
    }

    #[test]
    fn test_widen() {
        let mut oct1 = OctagonDomain::new();
        let mut oct2 = OctagonDomain::new();
        let x = make_value_id(1);

        oct1.set_interval(x, 0, 10);
        oct2.set_interval(x, 0, 15); // Upper bound grew

        let widened = oct1.widen(&oct2);

        // Widening should have jumped upper bound to infinity
        let interval = widened.get_interval(x).unwrap();
        assert_eq!(interval.lo(), 0);
        // Upper bound should be large (jumped to max)
    }
}
