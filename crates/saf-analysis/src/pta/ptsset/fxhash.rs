//! `FxHashSet`-based points-to set implementation.
//!
//! Uses `rustc-hash`'s `FxHashSet` for O(1) insert/contains/union operations.
//! ~14x faster than `BTreePtsSet` for programs with ~12K allocation sites
//! (log(12000) ≈ 14). Iteration order is non-deterministic, but the solver's
//! fixed-point result is order-independent. Output is normalized to
//! `BTreeSet<LocId>` at the API boundary for determinism (NFR-DET).

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use rustc_hash::FxHashSet;
use saf_core::ids::LocId;

use super::trait_def::PtsSet;

/// Points-to set backed by `FxHashSet<LocId>`.
///
/// Provides O(1) amortized insert, contains, and per-element union operations,
/// compared to O(log n) for `BTreePtsSet`. Best for internal solver computation
/// where iteration order does not affect the fixed-point result.
#[derive(Clone, Debug, Default)]
pub struct FxHashPtsSet {
    inner: FxHashSet<LocId>,
}

impl FxHashPtsSet {
    /// Create a new empty points-to set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl PartialEq for FxHashPtsSet {
    fn eq(&self, other: &Self) -> bool {
        self.inner.len() == other.inner.len()
            && self.inner.iter().all(|loc| other.inner.contains(loc))
    }
}

impl Eq for FxHashPtsSet {}

impl Hash for FxHashPtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Order-independent hash: XOR all element hashes.
        // This is correct because the set is unordered.
        let mut xor_hash: u64 = 0;
        for loc in &self.inner {
            let mut h = rustc_hash::FxHasher::default();
            loc.hash(&mut h);
            xor_hash ^= h.finish();
        }
        xor_hash.hash(state);
    }
}

impl PtsSet for FxHashPtsSet {
    fn empty() -> Self {
        Self::new()
    }

    fn singleton(loc: LocId) -> Self {
        let mut set = Self::new();
        set.inner.insert(loc);
        set
    }

    fn insert(&mut self, loc: LocId) -> bool {
        self.inner.insert(loc)
    }

    fn remove(&mut self, loc: LocId) -> bool {
        self.inner.remove(&loc)
    }

    fn contains(&self, loc: LocId) -> bool {
        self.inner.contains(&loc)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = LocId> {
        self.inner.iter().copied()
    }

    fn union(&mut self, other: &Self) -> bool {
        let old_len = self.inner.len();
        self.inner.extend(other.inner.iter().copied());
        self.inner.len() > old_len
    }

    fn intersect(&mut self, other: &Self) -> bool {
        let old_len = self.inner.len();
        self.inner.retain(|loc| other.inner.contains(loc));
        self.inner.len() < old_len
    }

    fn difference(&mut self, other: &Self) -> bool {
        let old_len = self.inner.len();
        self.inner.retain(|loc| !other.inner.contains(loc));
        self.inner.len() < old_len
    }

    fn intersects(&self, other: &Self) -> bool {
        if self.inner.len() <= other.inner.len() {
            self.inner.iter().any(|loc| other.inner.contains(loc))
        } else {
            other.inner.iter().any(|loc| self.inner.contains(loc))
        }
    }

    fn is_subset(&self, other: &Self) -> bool {
        self.inner.is_subset(&other.inner)
    }

    fn to_btreeset(&self) -> BTreeSet<LocId> {
        self.inner.iter().copied().collect()
    }

    fn from_btreeset(set: &BTreeSet<LocId>) -> Self {
        Self {
            inner: set.iter().copied().collect(),
        }
    }
}

impl FromIterator<LocId> for FxHashPtsSet {
    fn from_iter<T: IntoIterator<Item = LocId>>(iter: T) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        let mut pts = FxHashPtsSet::empty();
        assert!(pts.is_empty());

        assert!(pts.insert(LocId::new(1)));
        assert!(pts.insert(LocId::new(2)));
        assert!(!pts.insert(LocId::new(1))); // duplicate
        assert_eq!(pts.len(), 2);
        assert!(pts.contains(LocId::new(1)));
        assert!(!pts.contains(LocId::new(3)));
    }

    #[test]
    fn union_and_difference() {
        let mut a = FxHashPtsSet::empty();
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = FxHashPtsSet::empty();
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));

        assert!(a.union(&b));
        assert_eq!(a.len(), 3);

        assert!(a.difference(&b));
        assert_eq!(a.len(), 1);
        assert!(a.contains(LocId::new(1)));
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut pts = FxHashPtsSet::empty();
        pts.insert(LocId::new(3));
        pts.insert(LocId::new(1));
        pts.insert(LocId::new(2));

        let btree = pts.to_btreeset();
        assert_eq!(btree.len(), 3);
        // BTreeSet is sorted
        let sorted: Vec<_> = btree.iter().copied().collect();
        assert_eq!(sorted, vec![LocId::new(1), LocId::new(2), LocId::new(3)]);

        let pts2 = FxHashPtsSet::from_btreeset(&btree);
        assert_eq!(pts, pts2);
    }

    #[test]
    fn equality_is_order_independent() {
        let mut a = FxHashPtsSet::empty();
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));
        a.insert(LocId::new(3));

        let mut b = FxHashPtsSet::empty();
        b.insert(LocId::new(3));
        b.insert(LocId::new(1));
        b.insert(LocId::new(2));

        assert_eq!(a, b);
    }

    #[test]
    fn singleton() {
        let pts = FxHashPtsSet::singleton(LocId::new(42));
        assert_eq!(pts.len(), 1);
        assert!(pts.contains(LocId::new(42)));
    }
}
