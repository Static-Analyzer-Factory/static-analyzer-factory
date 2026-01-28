//! Roaring bitmap backed points-to set implementation.
//!
//! Uses the `roaring` crate for compressed bitmap operations.
//! Best for medium-to-large programs (50K-100K allocation sites).
//! Combines good compression with fast set operations via SIMD-optimized
//! container unions/intersections.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use roaring::RoaringBitmap;
use saf_core::ids::LocId;

use super::indexer::{FrozenLocIdIndexer, LocIdIndexer};
use super::trait_def::PtsSet;

/// Indexer state — either mutable (building) or frozen (solving).
#[derive(Clone, Debug)]
enum IndexerState {
    /// Mutable indexer behind `RwLock` — used during construction.
    Mutable(Arc<RwLock<LocIdIndexer>>),
    /// Frozen indexer — lock-free reads during solving.
    Frozen(Arc<FrozenLocIdIndexer>),
}

/// Points-to set backed by a Roaring bitmap.
///
/// Each element corresponds to a `LocId` via a shared indexer.
/// The indexer maps `LocId` to `u32` indices stored in the bitmap.
///
/// Roaring bitmaps use adaptive container types (arrays, bitsets, runs)
/// per 65536-element chunk, providing good compression for both sparse
/// and dense regions.
///
/// The indexer is shared (via `Arc<RwLock<...>>` or `Arc<FrozenLocIdIndexer>`)
/// to ensure consistent mapping across all roaring sets in an analysis.
#[derive(Clone, Debug)]
pub struct RoaringPtsSet {
    /// The roaring bitmap storing set membership.
    bitmap: RoaringBitmap,
    /// Indexer state — either mutable or frozen.
    indexer: IndexerState,
}

impl RoaringPtsSet {
    /// Create a new empty roaring set with a fresh indexer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bitmap: RoaringBitmap::new(),
            indexer: IndexerState::Mutable(Arc::new(RwLock::new(LocIdIndexer::new()))),
        }
    }

    /// Create a new empty set with a shared mutable indexer.
    #[must_use]
    #[allow(dead_code)]
    pub fn with_indexer(indexer: Arc<RwLock<LocIdIndexer>>) -> Self {
        Self {
            bitmap: RoaringBitmap::new(),
            indexer: IndexerState::Mutable(indexer),
        }
    }

    /// Create a new empty set with a frozen (lock-free) indexer.
    #[must_use]
    #[allow(dead_code)] // Used via PtsSet::with_frozen_ordering trait method
    pub fn with_frozen_indexer(frozen: Arc<FrozenLocIdIndexer>) -> Self {
        Self {
            bitmap: RoaringBitmap::new(),
            indexer: IndexerState::Frozen(frozen),
        }
    }

    /// Get the shared mutable indexer, if using mutable mode.
    #[must_use]
    #[allow(dead_code)]
    pub fn indexer(&self) -> Option<&Arc<RwLock<LocIdIndexer>>> {
        match &self.indexer {
            IndexerState::Mutable(idx) => Some(idx),
            IndexerState::Frozen(_) => None,
        }
    }
}

impl Default for RoaringPtsSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for RoaringPtsSet {
    fn eq(&self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if same_indexer {
            self.bitmap == other.bitmap
        } else {
            self.to_btreeset() == other.to_btreeset()
        }
    }
}

impl Eq for RoaringPtsSet {}

impl Hash for RoaringPtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash the LocIds in sorted order for deterministic hashing
        // (matches BTreePtsSet behavior)
        for loc in self.iter() {
            loc.hash(state);
        }
    }
}

// SAFETY: RoaringBitmap is Send+Sync, Arc<RwLock<...>> is Send+Sync
unsafe impl Send for RoaringPtsSet {}
unsafe impl Sync for RoaringPtsSet {}

impl PtsSet for RoaringPtsSet {
    fn empty() -> Self {
        Self::new()
    }

    fn singleton(loc: LocId) -> Self {
        let mut set = Self::new();
        set.insert(loc);
        set
    }

    fn insert(&mut self, loc: LocId) -> bool {
        let idx = match &self.indexer {
            IndexerState::Mutable(indexer) => {
                let mut indexer = indexer.write().expect("indexer lock poisoned");
                // INVARIANT: indexer indices are < 2^32
                #[allow(clippy::cast_possible_truncation)]
                let idx = indexer.get_or_insert(loc) as u32;
                idx
            }
            IndexerState::Frozen(frozen) => frozen
                .get(loc)
                .expect("LocId not in frozen indexer — all locations must be pre-registered"),
        };
        self.bitmap.insert(idx)
    }

    fn remove(&mut self, loc: LocId) -> bool {
        let idx = match &self.indexer {
            IndexerState::Mutable(indexer) => {
                let indexer = indexer.read().expect("indexer lock poisoned");
                // INVARIANT: indexer indices are < 2^32
                #[allow(clippy::cast_possible_truncation)]
                match indexer.get(loc) {
                    Some(idx) => idx as u32,
                    None => return false,
                }
            }
            IndexerState::Frozen(frozen) => match frozen.get(loc) {
                Some(idx) => idx,
                None => return false,
            },
        };
        self.bitmap.remove(idx)
    }

    fn contains(&self, loc: LocId) -> bool {
        let idx = match &self.indexer {
            IndexerState::Mutable(indexer) => {
                let indexer = indexer.read().expect("indexer lock poisoned");
                // INVARIANT: indexer indices are < 2^32
                #[allow(clippy::cast_possible_truncation)]
                match indexer.get(loc) {
                    Some(idx) => idx as u32,
                    None => return false,
                }
            }
            IndexerState::Frozen(frozen) => match frozen.get(loc) {
                Some(idx) => idx,
                None => return false,
            },
        };
        self.bitmap.contains(idx)
    }

    fn len(&self) -> usize {
        // INVARIANT: number of points-to targets fits in usize
        #[allow(clippy::cast_possible_truncation)]
        let len = self.bitmap.len() as usize;
        len
    }

    fn is_empty(&self) -> bool {
        self.bitmap.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = LocId> {
        let mut locs: Vec<LocId> = match &self.indexer {
            IndexerState::Mutable(indexer) => {
                let indexer = indexer.read().expect("indexer lock poisoned");
                self.bitmap
                    .iter()
                    .filter_map(|idx32| indexer.resolve(idx32 as usize))
                    .collect()
            }
            IndexerState::Frozen(frozen) => self
                .bitmap
                .iter()
                .filter_map(|idx32| frozen.resolve(idx32))
                .collect(),
        };
        // Sort for deterministic iteration (matches BTreePtsSet)
        locs.sort();
        locs.into_iter()
    }

    fn union(&mut self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_indexer {
            let mut changed = false;
            for loc in other.iter() {
                if self.insert(loc) {
                    changed = true;
                }
            }
            return changed;
        }
        let old_len = self.bitmap.len();
        self.bitmap |= &other.bitmap;
        self.bitmap.len() > old_len
    }

    fn intersect(&mut self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_indexer {
            let to_keep: BTreeSet<LocId> = self.iter().filter(|loc| other.contains(*loc)).collect();
            let old_len = self.len();
            *self = Self::from_btreeset(&to_keep);
            return self.len() < old_len;
        }
        let old_len = self.bitmap.len();
        self.bitmap &= &other.bitmap;
        self.bitmap.len() < old_len
    }

    fn difference(&mut self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_indexer {
            let to_keep: BTreeSet<LocId> =
                self.iter().filter(|loc| !other.contains(*loc)).collect();
            let old_len = self.len();
            *self = Self::from_btreeset(&to_keep);
            return self.len() < old_len;
        }
        let old_len = self.bitmap.len();
        self.bitmap -= &other.bitmap;
        self.bitmap.len() < old_len
    }

    fn intersects(&self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_indexer {
            if self.len() <= other.len() {
                return self.iter().any(|loc| other.contains(loc));
            }
            return other.iter().any(|loc| self.contains(loc));
        }
        !RoaringBitmap::is_disjoint(&self.bitmap, &other.bitmap)
    }

    fn is_subset(&self, other: &Self) -> bool {
        let same_indexer = match (&self.indexer, &other.indexer) {
            (IndexerState::Mutable(a), IndexerState::Mutable(b)) => Arc::ptr_eq(a, b),
            (IndexerState::Frozen(a), IndexerState::Frozen(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !same_indexer {
            return self.iter().all(|loc| other.contains(loc));
        }
        self.bitmap.is_subset(&other.bitmap)
    }

    fn to_btreeset(&self) -> BTreeSet<LocId> {
        self.iter().collect()
    }

    fn from_btreeset(set: &BTreeSet<LocId>) -> Self {
        let mut result = Self::new();
        for &loc in set {
            result.insert(loc);
        }
        result
    }

    const BENEFITS_FROM_CLUSTERING: bool = true;

    fn clone_empty(&self) -> Self {
        Self {
            bitmap: RoaringBitmap::new(),
            indexer: self.indexer.clone(),
        }
    }

    fn with_seeded_ordering(ordered_locs: &[LocId]) -> Self {
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(ordered_locs.iter().copied());
        Self::with_indexer(Arc::new(RwLock::new(indexer)))
    }

    fn with_frozen_ordering(frozen: Arc<FrozenLocIdIndexer>) -> Self {
        Self {
            bitmap: RoaringBitmap::new(),
            indexer: IndexerState::Frozen(frozen),
        }
    }
}

impl From<BTreeSet<LocId>> for RoaringPtsSet {
    fn from(set: BTreeSet<LocId>) -> Self {
        Self::from_btreeset(&set)
    }
}

impl FromIterator<LocId> for RoaringPtsSet {
    fn from_iter<T: IntoIterator<Item = LocId>>(iter: T) -> Self {
        let mut result = Self::new();
        for loc in iter {
            result.insert(loc);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_insert_contains_remove() {
        let mut pts = RoaringPtsSet::empty();
        assert!(pts.is_empty());

        assert!(pts.insert(LocId::new(1)));
        assert!(pts.insert(LocId::new(2)));
        assert!(!pts.insert(LocId::new(1))); // duplicate
        assert_eq!(pts.len(), 2);

        assert!(pts.contains(LocId::new(1)));
        assert!(pts.contains(LocId::new(2)));
        assert!(!pts.contains(LocId::new(3)));

        assert!(pts.remove(LocId::new(1)));
        assert!(!pts.remove(LocId::new(1))); // already removed
        assert_eq!(pts.len(), 1);
        assert!(!pts.contains(LocId::new(1)));
        assert!(pts.contains(LocId::new(2)));
    }

    #[test]
    fn union_same_indexer() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));

        assert!(a.union(&b));
        assert_eq!(a.len(), 3);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(2)));
        assert!(a.contains(LocId::new(3)));

        // Union with subset should not change
        assert!(!a.union(&b));
    }

    #[test]
    fn union_different_indexer() {
        let mut a = RoaringPtsSet::empty();
        a.insert(LocId::new(1));

        let mut b = RoaringPtsSet::empty();
        b.insert(LocId::new(2));

        assert!(a.union(&b));
        assert_eq!(a.len(), 2);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(2)));
    }

    #[test]
    fn intersect() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));
        a.insert(LocId::new(3));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));
        b.insert(LocId::new(4));

        assert!(a.intersect(&b));
        assert_eq!(a.len(), 2);
        assert!(a.contains(LocId::new(2)));
        assert!(a.contains(LocId::new(3)));
        assert!(!a.contains(LocId::new(1)));
    }

    #[test]
    fn difference() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));
        a.insert(LocId::new(3));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(2));
        b.insert(LocId::new(4));

        assert!(a.difference(&b));
        assert_eq!(a.len(), 2);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(3)));
        assert!(!a.contains(LocId::new(2)));
    }

    #[test]
    fn deterministic_iteration() {
        let mut pts = RoaringPtsSet::empty();
        // Insert in non-sorted order (by LocId)
        pts.insert(LocId::new(300));
        pts.insert(LocId::new(100));
        pts.insert(LocId::new(200));

        let collected: Vec<LocId> = pts.iter().collect();
        let mut sorted = collected.clone();
        sorted.sort();
        assert_eq!(
            collected, sorted,
            "iteration should be in sorted LocId order"
        );
    }

    #[test]
    fn clone_empty_shares_indexer() {
        let mut pts = RoaringPtsSet::empty();
        pts.insert(LocId::new(1));
        let empty = pts.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(
            pts.indexer().unwrap(),
            empty.indexer().unwrap()
        ));
    }

    #[test]
    fn with_seeded_ordering_registers_in_order() {
        let locs = vec![LocId::new(300), LocId::new(100), LocId::new(200)];
        let pts = RoaringPtsSet::with_seeded_ordering(&locs);
        let indexer = pts.indexer().unwrap().read().expect("lock");
        assert_eq!(indexer.get(LocId::new(300)), Some(0));
        assert_eq!(indexer.get(LocId::new(100)), Some(1));
        assert_eq!(indexer.get(LocId::new(200)), Some(2));
    }

    #[test]
    fn with_shared_indexer() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts1 = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        pts1.insert(LocId::new(100));

        let mut pts2 = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        pts2.insert(LocId::new(200));

        // Both should share the same indexer
        assert!(Arc::ptr_eq(
            pts1.indexer().unwrap(),
            pts2.indexer().unwrap()
        ));

        // Indexer should have both entries
        let indexer = indexer.read().unwrap();
        assert_eq!(indexer.len(), 2);
    }

    #[test]
    fn large_set() {
        let mut pts = RoaringPtsSet::empty();
        for i in 0..1000 {
            pts.insert(LocId::new(i * 3)); // Non-contiguous IDs
        }

        assert_eq!(pts.len(), 1000);

        // Verify all are present
        for i in 0..1000 {
            assert!(pts.contains(LocId::new(i * 3)));
            assert!(!pts.contains(LocId::new(i * 3 + 1)));
        }
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut pts = RoaringPtsSet::empty();
        pts.insert(LocId::new(1));
        pts.insert(LocId::new(2));

        let btree = pts.to_btreeset();
        assert_eq!(btree.len(), 2);

        let pts2 = RoaringPtsSet::from_btreeset(&btree);
        assert_eq!(pts.len(), pts2.len());
        assert!(pts2.contains(LocId::new(1)));
        assert!(pts2.contains(LocId::new(2)));
    }

    #[test]
    fn iter_empty_set() {
        let pts = RoaringPtsSet::empty();
        let collected: Vec<_> = pts.iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn clone_empty_enables_fast_union() {
        let mut a = RoaringPtsSet::empty();
        a.insert(LocId::new(1));
        let mut b = a.clone_empty();
        b.insert(LocId::new(2));
        assert!(Arc::ptr_eq(a.indexer().unwrap(), b.indexer().unwrap()));
        a.union(&b);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn intersects_check() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));

        assert!(a.intersects(&b));

        let mut c = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        c.insert(LocId::new(4));

        assert!(!a.intersects(&c));
    }

    #[test]
    fn is_subset_check() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(1));
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));

        assert!(a.is_subset(&b));
        assert!(!b.is_subset(&a));
    }

    #[test]
    fn singleton_test() {
        let pts = RoaringPtsSet::singleton(LocId::new(42));
        assert_eq!(pts.len(), 1);
        assert!(pts.contains(LocId::new(42)));
    }

    #[test]
    fn equality_same_indexer() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        b.insert(LocId::new(1));
        b.insert(LocId::new(2));

        assert_eq!(a, b);
    }

    #[test]
    fn equality_different_indexer() {
        let mut a = RoaringPtsSet::empty();
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));

        let mut b = RoaringPtsSet::empty();
        b.insert(LocId::new(1));
        b.insert(LocId::new(2));

        assert_eq!(a, b);
    }

    #[test]
    fn from_iterator() {
        let locs = vec![LocId::new(1), LocId::new(2), LocId::new(3)];
        let pts: RoaringPtsSet = locs.into_iter().collect();
        assert_eq!(pts.len(), 3);
        assert!(pts.contains(LocId::new(1)));
        assert!(pts.contains(LocId::new(2)));
        assert!(pts.contains(LocId::new(3)));
    }

    #[test]
    fn frozen_roaring_basic_ops() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(1));
        indexer.get_or_insert(LocId::new(2));
        indexer.get_or_insert(LocId::new(3));
        let frozen = Arc::new(indexer.freeze());

        let mut pts = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
        assert!(pts.insert(LocId::new(1)));
        assert!(pts.insert(LocId::new(2)));
        assert!(!pts.insert(LocId::new(1))); // duplicate
        assert_eq!(pts.len(), 2);
        assert!(pts.contains(LocId::new(1)));
        assert!(pts.contains(LocId::new(2)));
        assert!(!pts.contains(LocId::new(3)));
    }

    #[test]
    fn frozen_roaring_union() {
        let mut indexer = LocIdIndexer::new();
        for i in 0..10 {
            indexer.get_or_insert(LocId::new(i));
        }
        let frozen = Arc::new(indexer.freeze());

        let mut a = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
        let mut b = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
        a.insert(LocId::new(1));
        a.insert(LocId::new(2));
        b.insert(LocId::new(2));
        b.insert(LocId::new(3));

        assert!(a.union(&b));
        assert_eq!(a.len(), 3);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(2)));
        assert!(a.contains(LocId::new(3)));
    }

    #[test]
    fn frozen_roaring_clone_empty_shares_frozen() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(1));
        let frozen = Arc::new(indexer.freeze());

        let mut pts = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
        pts.insert(LocId::new(1));
        let empty = pts.clone_empty();
        assert!(empty.is_empty());
    }
}
