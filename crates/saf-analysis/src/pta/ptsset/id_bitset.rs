//! Generic bitvec-backed set for any ID type.
//!
//! Uses a shared `Indexer<T>` to map sparse IDs to dense bit positions,
//! enabling O(n/64) bulk set operations (union, intersect, difference).
//! All sets sharing the same indexer can perform bitwise operations directly.

use std::collections::BTreeSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

use bitvec::prelude::*;

use super::indexer::Indexer;

/// A generic bitvec-backed set for ID types.
///
/// Uses a shared `Indexer<T>` to map sparse IDs to dense bit positions,
/// enabling O(n/64) bulk set operations (union, intersect, difference).
/// All sets sharing the same indexer can perform bitwise operations directly.
#[derive(Debug)]
pub struct IdBitSet<T: Eq + Ord + Copy + Hash + fmt::Debug> {
    bits: BitVec<usize, Lsb0>,
    indexer: Arc<RwLock<Indexer<T>>>,
    cached_len: usize,
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> IdBitSet<T> {
    /// Create a new empty set with a fresh indexer.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            bits: BitVec::new(),
            indexer: Arc::new(RwLock::new(Indexer::new())),
            cached_len: 0,
        }
    }

    /// Create a new empty set with the given shared indexer.
    #[must_use]
    pub fn with_indexer(indexer: Arc<RwLock<Indexer<T>>>) -> Self {
        Self {
            bits: BitVec::new(),
            indexer,
            cached_len: 0,
        }
    }

    /// Create an empty set sharing this set's indexer.
    #[must_use]
    pub fn clone_empty(&self) -> Self {
        Self::with_indexer(Arc::clone(&self.indexer))
    }

    /// Get a reference to the shared indexer.
    #[must_use]
    pub fn indexer(&self) -> &Arc<RwLock<Indexer<T>>> {
        &self.indexer
    }

    /// Insert an item. Returns true if newly inserted.
    pub fn insert(&mut self, item: T) -> bool {
        let idx = {
            let mut indexer = self.indexer.write().expect("indexer lock poisoned");
            indexer.get_or_insert(item)
        };

        self.ensure_capacity(idx);

        if self.bits[idx] {
            false
        } else {
            self.bits.set(idx, true);
            self.cached_len += 1;
            true
        }
    }

    /// Remove an item. Returns true if it was present.
    pub fn remove(&mut self, item: T) -> bool {
        let idx = {
            let indexer = self.indexer.read().expect("indexer lock poisoned");
            match indexer.get(item) {
                Some(idx) => idx,
                None => return false,
            }
        };

        if idx < self.bits.len() && self.bits[idx] {
            self.bits.set(idx, false);
            self.cached_len -= 1;
            true
        } else {
            false
        }
    }

    /// Check if item is in the set.
    #[must_use]
    pub fn contains(&self, item: T) -> bool {
        let idx = {
            let indexer = self.indexer.read().expect("indexer lock poisoned");
            match indexer.get(item) {
                Some(idx) => idx,
                None => return false,
            }
        };

        idx < self.bits.len() && self.bits[idx]
    }

    /// Number of elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cached_len
    }

    /// Whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cached_len == 0
    }

    /// Iterate elements in index order (deterministic).
    /// NOTE: index order is insertion order, NOT `T`'s natural `Ord`.
    /// Elements are sorted by `T` for deterministic output.
    pub fn iter(&self) -> impl Iterator<Item = T> {
        let indexer = self.indexer.read().expect("indexer lock poisoned");

        let mut items: Vec<T> = self
            .bits
            .iter_ones()
            .filter_map(|idx| indexer.resolve(idx))
            .collect();

        // Sort for deterministic iteration
        items.sort();
        items.into_iter()
    }

    /// Remove and return the element with the lowest index.
    /// Useful as a worklist pop operation. O(n/64) scan using `first_one()`.
    pub fn pop_first(&mut self) -> Option<T> {
        let first = self.bits.first_one()?;
        let item = {
            let indexer = self.indexer.read().expect("indexer lock poisoned");
            indexer.resolve(first)?
        };
        self.bits.set(first, false);
        self.cached_len -= 1;
        Some(item)
    }

    /// Union: add all elements from other. Returns true if self changed.
    /// Fast path O(n/64) when indexers are shared (`Arc::ptr_eq`).
    pub fn union(&mut self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.indexer, &other.indexer) {
            // Fallback: iterate and insert
            let mut changed = false;
            for item in other.iter() {
                if self.insert(item) {
                    changed = true;
                }
            }
            return changed;
        }

        self.align_to_indexer();
        if other.bits.len() > self.bits.len() {
            self.bits.resize(other.bits.len(), false);
        }

        let old_len = self.cached_len;

        // SIMD-friendly: operate on raw usize words, not individual bits.
        // LLVM auto-vectorizes this loop to AVX2/NEON on 64-bit targets.
        let self_raw = self.bits.as_raw_mut_slice();
        let other_raw = other.bits.as_raw_slice();
        let common = self_raw.len().min(other_raw.len());
        for i in 0..common {
            self_raw[i] |= other_raw[i];
        }

        self.cached_len = self.bits.count_ones();
        self.cached_len > old_len
    }

    /// Intersect: keep only elements also in other. Returns true if self changed.
    pub fn intersect(&mut self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.indexer, &other.indexer) {
            // Fallback: iterate and check membership
            let to_keep: BTreeSet<T> = self.iter().filter(|item| other.contains(*item)).collect();
            let old_len = self.cached_len;
            *self = Self::from_btreeset_with_indexer(&to_keep, Arc::clone(&self.indexer));
            return self.cached_len < old_len;
        }

        let old_len = self.cached_len;

        let self_raw = self.bits.as_raw_mut_slice();
        let other_raw = other.bits.as_raw_slice();
        let common = self_raw.len().min(other_raw.len());

        // AND the common words
        for i in 0..common {
            self_raw[i] &= other_raw[i];
        }
        // Clear bits beyond other's length
        for word in &mut self_raw[common..] {
            *word = 0;
        }

        self.cached_len = self.bits.count_ones();
        self.cached_len < old_len
    }

    /// Difference: remove elements that are in other. Returns true if self changed.
    pub fn difference(&mut self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.indexer, &other.indexer) {
            let to_keep: BTreeSet<T> = self.iter().filter(|item| !other.contains(*item)).collect();
            let old_len = self.cached_len;
            *self = Self::from_btreeset_with_indexer(&to_keep, Arc::clone(&self.indexer));
            return self.cached_len < old_len;
        }

        let old_len = self.cached_len;

        let self_raw = self.bits.as_raw_mut_slice();
        let other_raw = other.bits.as_raw_slice();
        let common = self_raw.len().min(other_raw.len());

        for i in 0..common {
            self_raw[i] &= !other_raw[i];
        }

        self.cached_len = self.bits.count_ones();
        self.cached_len < old_len
    }

    /// Check if any element is in both sets.
    #[must_use]
    pub fn intersects(&self, other: &Self) -> bool {
        if !Arc::ptr_eq(&self.indexer, &other.indexer) {
            if self.len() <= other.len() {
                return self.iter().any(|item| other.contains(item));
            }
            return other.iter().any(|item| self.contains(item));
        }

        let self_raw = self.bits.as_raw_slice();
        let other_raw = other.bits.as_raw_slice();
        let common = self_raw.len().min(other_raw.len());

        for i in 0..common {
            if self_raw[i] & other_raw[i] != 0 {
                return true;
            }
        }
        false
    }

    /// Extend from an iterator of items.
    pub fn extend(&mut self, items: impl IntoIterator<Item = T>) {
        for item in items {
            self.insert(item);
        }
    }

    /// Convert to `BTreeSet` (sorted by `T`'s `Ord`).
    #[must_use]
    pub fn to_btreeset(&self) -> BTreeSet<T> {
        self.iter().collect()
    }

    /// Create from a `BTreeSet`, using a fresh indexer.
    #[must_use]
    pub fn from_btreeset(set: &BTreeSet<T>) -> Self {
        let mut result = Self::empty();
        for &item in set {
            result.insert(item);
        }
        result
    }

    /// Create from a `BTreeSet`, using a shared indexer.
    #[must_use]
    pub fn from_btreeset_with_indexer(set: &BTreeSet<T>, indexer: Arc<RwLock<Indexer<T>>>) -> Self {
        let mut result = Self::with_indexer(indexer);
        for &item in set {
            result.insert(item);
        }
        result
    }

    /// Ensure the bit vector has capacity for the given index.
    fn ensure_capacity(&mut self, idx: usize) {
        if idx >= self.bits.len() {
            self.bits.resize(idx + 1, false);
        }
    }

    /// Align this set's bit vector to match the indexer size.
    fn align_to_indexer(&mut self) {
        let indexer = self.indexer.read().expect("indexer lock poisoned");
        let required_len = indexer.len();
        if self.bits.len() < required_len {
            self.bits.resize(required_len, false);
        }
    }
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> Clone for IdBitSet<T> {
    fn clone(&self) -> Self {
        Self {
            bits: self.bits.clone(),
            indexer: Arc::clone(&self.indexer),
            cached_len: self.cached_len,
        }
    }
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> Default for IdBitSet<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> PartialEq for IdBitSet<T> {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.indexer, &other.indexer) {
            // Same indexer: compare bit patterns directly
            let min_len = self.bits.len().min(other.bits.len());

            if self.bits[..min_len] != other.bits[..min_len] {
                return false;
            }

            if self.bits.len() > min_len && self.bits[min_len..].any() {
                return false;
            }
            if other.bits.len() > min_len && other.bits[min_len..].any() {
                return false;
            }

            true
        } else {
            // Different indexers: compare actual element contents
            self.to_btreeset() == other.to_btreeset()
        }
    }
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> Eq for IdBitSet<T> {}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> Hash for IdBitSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash elements in sorted order for deterministic hashing
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<T: Eq + Ord + Copy + Hash + fmt::Debug> fmt::Display for IdBitSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item:?}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{BlockId, LocId, ValueId};

    #[test]
    fn empty_set() {
        let set = IdBitSet::<LocId>::empty();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut set = IdBitSet::<ValueId>::empty();
        assert!(set.insert(ValueId::new(42)));
        assert!(!set.insert(ValueId::new(42))); // duplicate
        assert!(set.contains(ValueId::new(42)));
        assert!(!set.contains(ValueId::new(99)));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove() {
        let mut set = IdBitSet::<LocId>::empty();
        set.insert(LocId::new(1));
        set.insert(LocId::new(2));
        assert!(set.remove(LocId::new(1)));
        assert!(!set.remove(LocId::new(1))); // already removed
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn union_shared_indexer() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        b.insert(LocId::new(2));
        assert!(a.union(&b));
        assert_eq!(a.len(), 2);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(2)));
    }

    #[test]
    fn union_different_indexer() {
        let mut a = IdBitSet::<LocId>::empty();
        let mut b = IdBitSet::<LocId>::empty();
        a.insert(LocId::new(1));
        b.insert(LocId::new(2));
        assert!(a.union(&b));
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn pop_first() {
        let mut set = IdBitSet::<ValueId>::empty();
        set.insert(ValueId::new(10));
        set.insert(ValueId::new(20));
        let first = set.pop_first();
        assert!(first.is_some());
        assert_eq!(set.len(), 1);
        let second = set.pop_first();
        assert!(second.is_some());
        assert!(set.is_empty());
        assert_eq!(set.pop_first(), None);
    }

    #[test]
    fn clone_empty_shares_indexer() {
        let mut set = IdBitSet::<BlockId>::empty();
        set.insert(BlockId::new(1));
        let empty = set.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(set.indexer(), empty.indexer()));
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut original = BTreeSet::new();
        original.insert(LocId::new(3));
        original.insert(LocId::new(1));
        original.insert(LocId::new(2));
        let bitset = IdBitSet::from_btreeset(&original);
        assert_eq!(bitset.to_btreeset(), original);
    }

    #[test]
    fn equality_same_indexer() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        b.insert(LocId::new(1));
        assert_eq!(a, b);
    }

    #[test]
    fn equality_different_indexer() {
        let mut a = IdBitSet::<LocId>::empty();
        let mut b = IdBitSet::<LocId>::empty();
        a.insert(LocId::new(1));
        b.insert(LocId::new(1));
        assert_eq!(a, b);
    }

    #[test]
    fn extend_from_iter() {
        let mut set = IdBitSet::<LocId>::empty();
        set.extend([LocId::new(1), LocId::new(2), LocId::new(3)]);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn intersect_and_difference() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.extend([LocId::new(1), LocId::new(2), LocId::new(3)]);
        b.extend([LocId::new(2), LocId::new(3), LocId::new(4)]);

        let mut intersected = a.clone();
        intersected.intersect(&b);
        assert_eq!(
            intersected.to_btreeset(),
            [LocId::new(2), LocId::new(3)].into_iter().collect()
        );

        let mut diffed = a.clone();
        diffed.difference(&b);
        assert_eq!(diffed.to_btreeset(), [LocId::new(1)].into_iter().collect());
    }
}
