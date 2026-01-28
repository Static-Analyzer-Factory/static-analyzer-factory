//! BTreeSet-based points-to set implementation.
//!
//! This is the baseline implementation wrapping `BTreeSet<LocId>`.
//! It provides deterministic iteration order and O(log n) operations.
//! Best for small to medium programs (<10K allocation sites).

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use saf_core::ids::LocId;

use super::trait_def::PtsSet;

/// Points-to set backed by `BTreeSet<LocId>`.
///
/// This is the baseline implementation that wraps the standard library's
/// `BTreeSet`. It provides:
/// - Deterministic iteration order (sorted by `LocId`)
/// - O(log n) insert, remove, and contains operations
/// - Simple implementation with no external dependencies
///
/// Use this for small programs or when determinism is the primary concern.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BTreePtsSet {
    inner: BTreeSet<LocId>,
}

impl BTreePtsSet {
    /// Create a new empty points-to set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the underlying `BTreeSet`.
    #[must_use]
    pub fn inner(&self) -> &BTreeSet<LocId> {
        &self.inner
    }

    /// Convert into the underlying `BTreeSet`.
    #[must_use]
    pub fn into_inner(self) -> BTreeSet<LocId> {
        self.inner
    }
}

impl Hash for BTreePtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash elements in deterministic order (BTreeSet iteration is sorted)
        for loc in &self.inner {
            loc.hash(state);
        }
    }
}

impl PtsSet for BTreePtsSet {
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
        // Check if any element is in both sets
        // Use the smaller set for iteration for efficiency
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
        self.inner.clone()
    }

    fn from_btreeset(set: &BTreeSet<LocId>) -> Self {
        Self { inner: set.clone() }
    }
}

impl From<BTreeSet<LocId>> for BTreePtsSet {
    fn from(inner: BTreeSet<LocId>) -> Self {
        Self { inner }
    }
}

impl From<BTreePtsSet> for BTreeSet<LocId> {
    fn from(pts: BTreePtsSet) -> Self {
        pts.inner
    }
}

impl FromIterator<LocId> for BTreePtsSet {
    fn from_iter<T: IntoIterator<Item = LocId>>(iter: T) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for BTreePtsSet {
    type Item = LocId;
    type IntoIter = std::collections::btree_set::IntoIter<LocId>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

#[allow(clippy::into_iter_without_iter)]
impl<'a> IntoIterator for &'a BTreePtsSet {
    type Item = &'a LocId;
    type IntoIter = std::collections::btree_set::Iter<'a, LocId>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_inner() {
        let mut pts = BTreePtsSet::empty();
        pts.insert(LocId::new(1));

        let inner = pts.into_inner();
        assert_eq!(inner.len(), 1);
        assert!(inner.contains(&LocId::new(1)));
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut pts = BTreePtsSet::empty();
        pts.insert(LocId::new(1));
        pts.insert(LocId::new(2));

        let btree = pts.to_btreeset();
        assert_eq!(btree.len(), 2);

        let pts2 = BTreePtsSet::from_btreeset(&btree);
        assert_eq!(pts, pts2);
    }
}
