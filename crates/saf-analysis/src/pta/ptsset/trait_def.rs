//! Points-to set trait definition.
//!
//! Defines the `PtsSet` trait that abstracts over different points-to set
//! representations: `BTreeSet` (baseline), Roaring bitmap (medium scale), and
//! BDD (large scale).

use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

use saf_core::ids::LocId;

use super::indexer::FrozenIndexer;

/// Trait for points-to set implementations.
///
/// Points-to sets store abstract memory locations (`LocId`) that a pointer
/// may reference. This trait enables swapping between representations
/// optimized for different program scales:
///
/// - `BTreePtsSet`: Baseline implementation using `BTreeSet<LocId>`.
///   Best for small programs (<10K allocation sites). O(log n) operations.
///
/// - `RoaringPtsSet`: Roaring bitmap representation for compressed bitmaps.
///   Best for medium programs (10K-100K sites). O(1) membership test.
///
/// - `BddPtsSet`: BDD (Binary Decision Diagram) representation.
///   Best for large programs (>100K sites) with structural sharing.
///
/// All implementations must maintain deterministic iteration order for
/// reproducibility (NFR-DET).
pub trait PtsSet: Clone + Default + Eq + Hash + Send + Sync + Debug {
    /// Create an empty points-to set.
    fn empty() -> Self;

    /// Create a points-to set containing a single location.
    fn singleton(loc: LocId) -> Self;

    /// Insert a location into the set.
    ///
    /// Returns `true` if the location was newly inserted (set changed).
    fn insert(&mut self, loc: LocId) -> bool;

    /// Remove a location from the set.
    ///
    /// Returns `true` if the location was present and removed.
    fn remove(&mut self, loc: LocId) -> bool;

    /// Check if the set contains a location.
    fn contains(&self, loc: LocId) -> bool;

    /// Get the number of locations in the set.
    fn len(&self) -> usize;

    /// Check if the set is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over locations in the set.
    ///
    /// Iteration order must be deterministic (sorted by `LocId`).
    fn iter(&self) -> impl Iterator<Item = LocId>;

    /// Union this set with another, modifying this set in place.
    ///
    /// Returns `true` if this set changed (new elements were added).
    fn union(&mut self, other: &Self) -> bool;

    /// Intersect this set with another, modifying this set in place.
    ///
    /// Returns `true` if this set changed (elements were removed).
    fn intersect(&mut self, other: &Self) -> bool;

    /// Remove elements in `other` from this set.
    ///
    /// Returns `true` if this set changed (elements were removed).
    fn difference(&mut self, other: &Self) -> bool;

    /// Check if this set and another have any common elements.
    fn intersects(&self, other: &Self) -> bool;

    /// Check if this set is a subset of another.
    fn is_subset(&self, other: &Self) -> bool;

    /// Convert to a `BTreeSet<LocId>` for API compatibility.
    ///
    /// This method enables normalization of results regardless of internal
    /// representation, ensuring stable external API.
    fn to_btreeset(&self) -> std::collections::BTreeSet<LocId> {
        self.iter().collect()
    }

    /// Create from a `BTreeSet<LocId>`.
    fn from_btreeset(set: &std::collections::BTreeSet<LocId>) -> Self;

    /// Create a new empty set sharing internal state (indexer/context) with this set.
    ///
    /// For indexed representations (`RoaringPtsSet`, `BddPtsSet`), shares the same
    /// indexer for fast bitwise operations. For `BTreePtsSet`, equivalent to `empty()`.
    #[must_use]
    fn clone_empty(&self) -> Self {
        Self::empty()
    }

    /// Whether this representation benefits from object clustering.
    const BENEFITS_FROM_CLUSTERING: bool = false;

    /// Create an empty set with a pre-seeded indexer ordering.
    ///
    /// Registers `ordered_locs` in the given order so co-occurring locations
    /// get adjacent bit indices. Default ignores the ordering.
    fn with_seeded_ordering(ordered_locs: &[LocId]) -> Self {
        let _ = ordered_locs;
        Self::empty()
    }

    /// Create an empty set with a pre-built frozen indexer.
    ///
    /// For indexed representations, uses the frozen indexer for lock-free operations.
    /// Default implementation ignores the frozen indexer and returns `empty()`.
    fn with_frozen_ordering(frozen: Arc<FrozenIndexer<LocId>>) -> Self {
        let _ = frozen;
        Self::empty()
    }
}
