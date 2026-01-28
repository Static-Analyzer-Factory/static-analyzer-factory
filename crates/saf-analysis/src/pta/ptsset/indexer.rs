//! Generic ID indexer for bit-vector and BDD representations.
//!
//! Provides bidirectional mapping between any ID type `T` and dense `usize` indices.
//! This is needed for bit-vector (where each bit position represents an item)
//! and BDD (where each item is encoded in binary) representations.

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;

/// Immutable snapshot of an `Indexer<T>`.
///
/// Provides O(1) forward lookup via `FxHashMap` and O(1) reverse lookup via `Vec`.
/// No synchronization needed — wrap in `Arc` for shared access.
#[derive(Debug, Clone)]
pub struct FrozenIndexer<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> {
    item_to_idx: FxHashMap<T, u32>,
    idx_to_item: Vec<T>,
}

impl<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> FrozenIndexer<T> {
    /// Look up the index for an item. O(1) amortized.
    #[must_use]
    #[inline]
    pub fn get(&self, item: T) -> Option<u32> {
        self.item_to_idx.get(&item).copied()
    }

    /// Resolve an index back to its item. O(1).
    #[must_use]
    #[inline]
    pub fn resolve(&self, idx: u32) -> Option<T> {
        self.idx_to_item.get(idx as usize).copied()
    }

    /// Number of indexed items.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.idx_to_item.len()
    }

    /// Whether the indexer is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.idx_to_item.is_empty()
    }
}

/// Bidirectional mapping between items of type `T` and dense `usize` indices.
///
/// This indexer is used by bit-vector and BDD points-to set implementations
/// to map abstract IDs to compact integer indices suitable for
/// their internal representations.
///
/// # Properties
///
/// - Indices are assigned sequentially starting from 0
/// - Each `T` maps to exactly one index
/// - Mapping is deterministic (same insertion order = same indices)
/// - Thread-safe when wrapped in appropriate synchronization primitives
///
/// # Example
///
/// ```ignore
/// use saf_core::ids::LocId;
/// use saf_analysis::pta::ptsset::LocIdIndexer;
///
/// let mut indexer = LocIdIndexer::new();
/// let idx1 = indexer.get_or_insert(LocId::new(100));
/// let idx2 = indexer.get_or_insert(LocId::new(200));
///
/// assert_eq!(idx1, 0);
/// assert_eq!(idx2, 1);
/// assert_eq!(indexer.resolve(0), Some(LocId::new(100)));
/// ```
#[derive(Debug, Clone)]
pub struct Indexer<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> {
    /// Forward mapping: T → index
    item_to_idx: BTreeMap<T, usize>,
    /// Reverse mapping: index → T
    idx_to_item: Vec<T>,
}

impl<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> Default for Indexer<T> {
    fn default() -> Self {
        Self {
            item_to_idx: BTreeMap::new(),
            idx_to_item: Vec::new(),
        }
    }
}

impl<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> Indexer<T> {
    /// Create a new empty indexer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an indexer with pre-allocated capacity.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            item_to_idx: BTreeMap::new(),
            idx_to_item: Vec::with_capacity(capacity),
        }
    }

    /// Get or create an index for an item.
    ///
    /// If the item is already indexed, returns its existing index.
    /// Otherwise, assigns the next available index.
    pub fn get_or_insert(&mut self, item: T) -> usize {
        if let Some(&idx) = self.item_to_idx.get(&item) {
            return idx;
        }

        let idx = self.idx_to_item.len();
        self.idx_to_item.push(item);
        self.item_to_idx.insert(item, idx);
        idx
    }

    /// Get the index for an item, if it exists.
    #[must_use]
    pub fn get(&self, item: T) -> Option<usize> {
        self.item_to_idx.get(&item).copied()
    }

    /// Resolve an index back to its item.
    #[must_use]
    pub fn resolve(&self, idx: usize) -> Option<T> {
        self.idx_to_item.get(idx).copied()
    }

    /// Get the number of indexed items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.idx_to_item.len()
    }

    /// Check if the indexer is empty.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn is_empty(&self) -> bool {
        self.idx_to_item.is_empty()
    }

    /// Iterate over all (index, T) pairs.
    #[allow(dead_code)] // Public API for external use
    pub fn iter(&self) -> impl Iterator<Item = (usize, T)> + '_ {
        self.idx_to_item
            .iter()
            .enumerate()
            .map(|(i, &item)| (i, item))
    }

    /// Iterate over all items in index order.
    #[allow(dead_code)] // Public API for external use
    pub fn items(&self) -> impl Iterator<Item = T> + '_ {
        self.idx_to_item.iter().copied()
    }

    /// Iterate over all items in index order (alias for backward compatibility).
    #[allow(dead_code)] // Public API for external use
    pub fn locations(&self) -> impl Iterator<Item = T> + '_ {
        self.items()
    }

    /// Get the maximum index (len - 1), or None if empty.
    #[must_use]
    #[allow(dead_code)] // Public API for external use
    pub fn max_index(&self) -> Option<usize> {
        if self.idx_to_item.is_empty() {
            None
        } else {
            Some(self.idx_to_item.len() - 1)
        }
    }

    /// Clear all mappings.
    #[allow(dead_code)] // Public API for external use
    pub fn clear(&mut self) {
        self.item_to_idx.clear();
        self.idx_to_item.clear();
    }

    /// Register multiple items at once, returning the starting index.
    ///
    /// This is more efficient than calling `get_or_insert` repeatedly
    /// when you need to register many items.
    #[allow(dead_code)] // Public API for external use
    pub fn register_batch(&mut self, items: impl IntoIterator<Item = T>) -> usize {
        let start = self.idx_to_item.len();
        for item in items {
            self.get_or_insert(item);
        }
        start
    }

    /// Freeze this indexer into an immutable `FrozenIndexer`.
    ///
    /// The frozen indexer uses `FxHashMap` for O(1) lookups (vs `BTreeMap`'s O(log n))
    /// and requires no locks for concurrent reads.
    #[must_use]
    pub fn freeze(&self) -> FrozenIndexer<T> {
        let item_to_idx: FxHashMap<T, u32> = self
            .item_to_idx
            .iter()
            .map(|(&item, &idx)| {
                // INVARIANT: indexer indices are < 2^32 (programs won't exceed 4B locations)
                #[allow(clippy::cast_possible_truncation)]
                let idx32 = idx as u32;
                (item, idx32)
            })
            .collect();
        FrozenIndexer {
            item_to_idx,
            idx_to_item: self.idx_to_item.clone(),
        }
    }
}

/// Backward-compatible alias for `Indexer<LocId>`.
pub type LocIdIndexer = Indexer<saf_core::ids::LocId>;

/// Backward-compatible alias for `FrozenIndexer<LocId>`.
pub type FrozenLocIdIndexer = FrozenIndexer<saf_core::ids::LocId>;

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::LocId;

    #[test]
    fn new_indexer_is_empty() {
        let indexer = LocIdIndexer::new();
        assert!(indexer.is_empty());
        assert_eq!(indexer.len(), 0);
        assert_eq!(indexer.max_index(), None);
    }

    #[test]
    fn get_or_insert_new() {
        let mut indexer = LocIdIndexer::new();
        let loc = LocId::new(100);
        let idx = indexer.get_or_insert(loc);

        assert_eq!(idx, 0);
        assert_eq!(indexer.len(), 1);
    }

    #[test]
    fn get_or_insert_existing() {
        let mut indexer = LocIdIndexer::new();
        let loc = LocId::new(100);

        let idx1 = indexer.get_or_insert(loc);
        let idx2 = indexer.get_or_insert(loc);

        assert_eq!(idx1, idx2);
        assert_eq!(indexer.len(), 1);
    }

    #[test]
    fn sequential_indices() {
        let mut indexer = LocIdIndexer::new();

        let idx1 = indexer.get_or_insert(LocId::new(100));
        let idx2 = indexer.get_or_insert(LocId::new(200));
        let idx3 = indexer.get_or_insert(LocId::new(300));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(idx3, 2);
    }

    #[test]
    fn get_existing() {
        let mut indexer = LocIdIndexer::new();
        let loc = LocId::new(100);
        indexer.get_or_insert(loc);

        assert_eq!(indexer.get(loc), Some(0));
    }

    #[test]
    fn get_nonexistent() {
        let indexer = LocIdIndexer::new();
        assert_eq!(indexer.get(LocId::new(100)), None);
    }

    #[test]
    fn resolve_existing() {
        let mut indexer = LocIdIndexer::new();
        let loc = LocId::new(100);
        indexer.get_or_insert(loc);

        assert_eq!(indexer.resolve(0), Some(loc));
    }

    #[test]
    fn resolve_nonexistent() {
        let indexer = LocIdIndexer::new();
        assert_eq!(indexer.resolve(0), None);
    }

    #[test]
    fn resolve_out_of_bounds() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));

        assert_eq!(indexer.resolve(0), Some(LocId::new(100)));
        assert_eq!(indexer.resolve(1), None);
        assert_eq!(indexer.resolve(100), None);
    }

    #[test]
    fn iter_returns_all_pairs() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));
        indexer.get_or_insert(LocId::new(200));
        indexer.get_or_insert(LocId::new(300));

        let pairs: Vec<_> = indexer.iter().collect();
        assert_eq!(
            pairs,
            vec![
                (0, LocId::new(100)),
                (1, LocId::new(200)),
                (2, LocId::new(300)),
            ]
        );
    }

    #[test]
    fn items_returns_all() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));
        indexer.get_or_insert(LocId::new(200));

        let items: Vec<_> = indexer.items().collect();
        assert_eq!(items, vec![LocId::new(100), LocId::new(200)]);
    }

    #[test]
    fn locations_returns_all_locs() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));
        indexer.get_or_insert(LocId::new(200));

        let locs: Vec<_> = indexer.locations().collect();
        assert_eq!(locs, vec![LocId::new(100), LocId::new(200)]);
    }

    #[test]
    fn max_index() {
        let mut indexer = LocIdIndexer::new();
        assert_eq!(indexer.max_index(), None);

        indexer.get_or_insert(LocId::new(100));
        assert_eq!(indexer.max_index(), Some(0));

        indexer.get_or_insert(LocId::new(200));
        assert_eq!(indexer.max_index(), Some(1));
    }

    #[test]
    fn clear() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));
        indexer.get_or_insert(LocId::new(200));

        indexer.clear();

        assert!(indexer.is_empty());
        assert_eq!(indexer.get(LocId::new(100)), None);
    }

    #[test]
    fn register_batch() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(50)); // Pre-existing

        let locs = vec![LocId::new(100), LocId::new(200), LocId::new(300)];
        let start = indexer.register_batch(locs);

        assert_eq!(start, 1); // Started after the pre-existing entry
        assert_eq!(indexer.len(), 4);
        assert_eq!(indexer.get(LocId::new(100)), Some(1));
        assert_eq!(indexer.get(LocId::new(200)), Some(2));
        assert_eq!(indexer.get(LocId::new(300)), Some(3));
    }

    #[test]
    fn register_batch_with_duplicates() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100)); // Pre-existing

        let locs = vec![LocId::new(100), LocId::new(200), LocId::new(100)];
        indexer.register_batch(locs);

        // Should only have 2 unique locations
        assert_eq!(indexer.len(), 2);
    }

    #[test]
    fn with_capacity() {
        let indexer = LocIdIndexer::with_capacity(100);
        assert!(indexer.is_empty());
        // Capacity is an internal detail, but creation should work
    }

    #[test]
    fn deterministic_ordering() {
        // Create two indexers with same insertions
        let mut indexer1 = LocIdIndexer::new();
        let mut indexer2 = LocIdIndexer::new();

        let locs = vec![LocId::new(300), LocId::new(100), LocId::new(200)];
        for &loc in &locs {
            indexer1.get_or_insert(loc);
            indexer2.get_or_insert(loc);
        }

        // Same insertion order should give same indices
        for &loc in &locs {
            assert_eq!(indexer1.get(loc), indexer2.get(loc));
        }
    }

    #[test]
    fn generic_indexer_with_value_id() {
        use saf_core::ids::ValueId;
        let mut indexer = Indexer::<ValueId>::new();
        let idx = indexer.get_or_insert(ValueId::new(42));
        assert_eq!(idx, 0);
        assert_eq!(indexer.resolve(0), Some(ValueId::new(42)));
    }

    #[test]
    fn frozen_indexer_basic() {
        let mut indexer = LocIdIndexer::new();
        indexer.get_or_insert(LocId::new(100));
        indexer.get_or_insert(LocId::new(200));
        indexer.get_or_insert(LocId::new(300));

        let frozen = indexer.freeze();
        assert_eq!(frozen.get(LocId::new(100)), Some(0));
        assert_eq!(frozen.get(LocId::new(200)), Some(1));
        assert_eq!(frozen.get(LocId::new(300)), Some(2));
        assert_eq!(frozen.get(LocId::new(999)), None);
        assert_eq!(frozen.resolve(0), Some(LocId::new(100)));
        assert_eq!(frozen.resolve(3), None);
        assert_eq!(frozen.len(), 3);
    }

    #[test]
    fn frozen_indexer_empty() {
        let indexer = LocIdIndexer::new();
        let frozen = indexer.freeze();
        assert!(frozen.is_empty());
        assert_eq!(frozen.len(), 0);
        assert_eq!(frozen.get(LocId::new(1)), None);
        assert_eq!(frozen.resolve(0), None);
    }

    #[test]
    fn frozen_indexer_from_register_batch() {
        let mut indexer = LocIdIndexer::new();
        let locs = vec![LocId::new(300), LocId::new(100), LocId::new(200)];
        indexer.register_batch(locs);
        let frozen = indexer.freeze();
        // register_batch preserves insertion order
        assert_eq!(frozen.get(LocId::new(300)), Some(0));
        assert_eq!(frozen.get(LocId::new(100)), Some(1));
        assert_eq!(frozen.get(LocId::new(200)), Some(2));
    }
}
