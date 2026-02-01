//! Points-to set lattice for Ascent-based PTA.
//!
//! Wraps `FixedBitSet` with set-union as join (least upper bound)
//! and set-intersection as meet (greatest lower bound), implementing
//! the `ascent::lattice::Lattice` trait for use in Ascent relations.
//!
//! Uses a dense bitvector indexed by a shared [`LocIdRegistry`] that maps
//! sparse `LocId` (u128) to dense `u32` indices. The registry is accessed
//! via a thread-local set by [`with_registry`] before Ascent's `prog.run()`.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ascent::lattice::Lattice;
use fixedbitset::FixedBitSet;
use saf_core::ids::LocId;

use crate::pta::registry::{LocIdRegistry, current_registry};

/// Points-to set for Ascent lattice-based PTA.
///
/// Uses `FixedBitSet` (dense bitvector) with set-union as join (least upper
/// bound) and set-intersection as meet (greatest lower bound). The empty set
/// is the lattice bottom element.
///
/// Each set carries an `Arc<LocIdRegistry>` for mapping between dense bit
/// indices and sparse `LocId` values. The registry is obtained from a
/// thread-local on construction and shared across all sets in a solve phase.
#[derive(Clone, Debug)]
pub struct AscentPtsSet {
    bits: FixedBitSet,
    registry: Arc<LocIdRegistry>,
}

impl AscentPtsSet {
    /// Create an empty points-to set (lattice bottom).
    #[must_use]
    pub fn empty() -> Self {
        let registry = current_registry();
        let bits = FixedBitSet::with_capacity(registry.len());
        Self { bits, registry }
    }

    /// Create a points-to set containing a single location.
    #[must_use]
    pub fn singleton(loc: LocId) -> Self {
        let registry = current_registry();
        let mut bits = FixedBitSet::with_capacity(registry.len());
        bits.insert(registry.index_of(loc) as usize);
        Self { bits, registry }
    }

    /// Create a points-to set from an iterator of locations.
    pub fn collect_from(locs: impl IntoIterator<Item = LocId>) -> Self {
        let registry = current_registry();
        let mut bits = FixedBitSet::with_capacity(registry.len());
        for loc in locs {
            bits.insert(registry.index_of(loc) as usize);
        }
        Self { bits, registry }
    }

    /// Check whether the set contains a given location.
    #[must_use]
    pub fn contains(&self, loc: LocId) -> bool {
        if let Some(&idx) = self.registry.try_index_of(loc) {
            self.bits.contains(idx as usize)
        } else {
            false
        }
    }

    /// Return the number of locations in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bits.count_ones(..)
    }

    /// Check whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits.count_ones(..) == 0
    }

    /// Iterate over the locations in the set.
    pub fn iter(&self) -> impl Iterator<Item = LocId> + '_ {
        self.bits.ones().map(|idx| {
            // INVARIANT: indices in bits are always valid registry indices.
            #[allow(clippy::cast_possible_truncation)]
            let idx = idx as u32;
            self.registry.loc_at(idx)
        })
    }

    /// Convert to the standard SAF `BTreeSet<LocId>` for output compatibility.
    #[must_use]
    pub fn into_btreeset(self) -> BTreeSet<LocId> {
        self.iter().collect()
    }
}

impl Default for AscentPtsSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for AscentPtsSet {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl Eq for AscentPtsSet {}

impl Hash for AscentPtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

impl PartialOrd for AscentPtsSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.bits == other.bits {
            Some(std::cmp::Ordering::Equal)
        } else if self.bits.is_subset(&other.bits) {
            Some(std::cmp::Ordering::Less)
        } else if other.bits.is_subset(&self.bits) {
            Some(std::cmp::Ordering::Greater)
        } else {
            None // incomparable sets
        }
    }
}

impl Lattice for AscentPtsSet {
    fn meet_mut(&mut self, other: Self) -> bool {
        let before = self.bits.count_ones(..);
        self.bits.intersect_with(&other.bits);
        self.bits.count_ones(..) != before
    }

    fn join_mut(&mut self, other: Self) -> bool {
        let before = self.bits.count_ones(..);
        self.bits.union_with(&other.bits);
        self.bits.count_ones(..) != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::PtaFacts;
    use crate::pta::registry::{LocIdRegistry, with_registry};
    use saf_core::ids::ValueId;

    fn setup_registry(locs: &[LocId]) -> Arc<LocIdRegistry> {
        let val = ValueId::derive(b"ptr");
        let facts = PtaFacts {
            addr_of: locs.iter().map(|l| (val, *l)).collect(),
            ..Default::default()
        };
        Arc::new(LocIdRegistry::from_facts(&facts))
    }

    fn loc(name: &[u8]) -> LocId {
        LocId::derive(name)
    }

    #[test]
    fn empty_set_has_zero_length() {
        let locs = [loc(b"a"), loc(b"b")];
        let reg = setup_registry(&locs);
        with_registry(reg, || {
            let s = AscentPtsSet::empty();
            assert!(s.is_empty());
            assert_eq!(s.len(), 0);
        });
    }

    #[test]
    fn singleton_contains_one_element() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s = AscentPtsSet::singleton(la);
            assert_eq!(s.len(), 1);
            assert!(s.contains(la));
            assert!(!s.contains(lb));
        });
    }

    #[test]
    fn join_mut_computes_union() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::singleton(lb);

            let changed = s1.join_mut(s2);
            assert!(changed);
            assert_eq!(s1.len(), 2);
            assert!(s1.contains(la));
            assert!(s1.contains(lb));
        });
    }

    #[test]
    fn join_mut_returns_false_when_no_change() {
        let la = loc(b"a");
        let reg = setup_registry(&[la]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::singleton(la);

            let changed = s1.join_mut(s2);
            assert!(!changed);
            assert_eq!(s1.len(), 1);
        });
    }

    #[test]
    fn meet_mut_computes_intersection() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::collect_from(vec![la, lb]);
            let s2 = AscentPtsSet::collect_from(vec![lb, lc]);

            let changed = s1.meet_mut(s2);
            assert!(changed);
            assert_eq!(s1.len(), 1);
            assert!(s1.contains(lb));
        });
    }

    #[test]
    fn meet_mut_returns_false_when_no_change() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::collect_from(vec![la, lb]);

            // s1 ∩ s2 = {a} = s1, so no change
            let changed = s1.meet_mut(s2);
            assert!(!changed);
        });
    }

    #[test]
    fn partial_ord_subset_ordering() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let s_a = AscentPtsSet::singleton(la);
            let s_ab = AscentPtsSet::collect_from(vec![la, lb]);
            let s_c = AscentPtsSet::singleton(lc);

            // {a} < {a, b}
            assert!(s_a < s_ab);
            // {a, b} > {a}
            assert!(s_ab > s_a);
            // {a} and {c} are incomparable
            assert_eq!(s_a.partial_cmp(&s_c), None);
            // Equal sets
            assert_eq!(
                s_a.partial_cmp(&s_a.clone()),
                Some(std::cmp::Ordering::Equal)
            );
        });
    }

    #[test]
    fn iter_yields_correct_loc_ids() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let s = AscentPtsSet::collect_from(vec![la, lc]);
            let mut result: Vec<LocId> = s.iter().collect();
            result.sort();

            let mut expected = vec![la, lc];
            expected.sort();

            assert_eq!(result, expected);
        });
    }

    #[test]
    fn into_btreeset_round_trips() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s = AscentPtsSet::collect_from(vec![la, lb]);
            let btree = s.into_btreeset();
            assert_eq!(btree.len(), 2);
            assert!(btree.contains(&la));
            assert!(btree.contains(&lb));
        });
    }

    #[test]
    fn hash_consistent_for_equal_sets() {
        use std::collections::hash_map::DefaultHasher;

        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s1 = AscentPtsSet::collect_from(vec![la, lb]);
            let s2 = AscentPtsSet::collect_from(vec![lb, la]); // same set, different order

            let hash = |s: &AscentPtsSet| {
                let mut h = DefaultHasher::new();
                s.hash(&mut h);
                h.finish()
            };

            assert_eq!(hash(&s1), hash(&s2));
        });
    }

    #[test]
    fn default_is_empty() {
        let la = loc(b"a");
        let reg = setup_registry(&[la]);
        with_registry(reg, || {
            let s = AscentPtsSet::default();
            assert!(s.is_empty());
            assert_eq!(s.len(), 0);
        });
    }

    #[test]
    fn clone_produces_independent_copy() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s1 = AscentPtsSet::singleton(la);
            let mut s2 = s1.clone();
            s2.join_mut(AscentPtsSet::singleton(lb));

            assert_eq!(s1.len(), 1);
            assert_eq!(s2.len(), 2);
        });
    }
}
