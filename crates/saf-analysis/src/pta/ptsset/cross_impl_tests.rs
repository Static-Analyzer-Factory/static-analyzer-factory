//! Cross-implementation property tests for points-to set representations.
//!
//! These tests verify that all three `PtsSet` implementations (`BTreePtsSet`,
//! `RoaringPtsSet`, `BddPtsSet`) produce identical results for the same
//! sequence of operations.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::{Arc, RwLock};

    use proptest::prelude::*;
    use saf_core::ids::LocId;

    use crate::pta::ptsset::{
        BTreePtsSet, BddContext, BddPtsSet, LocIdIndexer, PtsSet, RoaringPtsSet,
    };

    /// Helper to create all three implementations with shared resources.
    fn create_all_impls() -> (BTreePtsSet, RoaringPtsSet, BddPtsSet) {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        let btree = BTreePtsSet::empty();
        let roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let bdd = BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        (btree, roaring, bdd)
    }

    /// Assert that all three sets have the same contents (as `BTreeSet<LocId>`).
    fn assert_all_equal(btree: &BTreePtsSet, roaring: &RoaringPtsSet, bdd: &BddPtsSet) {
        let btree_set = btree.to_btreeset();
        let roaring_set = roaring.to_btreeset();
        let bdd_set = bdd.to_btreeset();

        assert_eq!(
            btree_set, roaring_set,
            "BTree and Roaring sets differ: {:?} vs {:?}",
            btree_set, roaring_set
        );
        assert_eq!(
            btree_set, bdd_set,
            "BTree and BDD sets differ: {:?} vs {:?}",
            btree_set, bdd_set
        );

        // Also verify len() consistency
        assert_eq!(
            btree.len(),
            roaring.len(),
            "len() differs between BTree and BitVec"
        );
        assert_eq!(
            btree.len(),
            bdd.len(),
            "len() differs between BTree and BDD"
        );

        // Verify is_empty() consistency
        assert_eq!(
            btree.is_empty(),
            roaring.is_empty(),
            "is_empty() differs between BTree and BitVec"
        );
        assert_eq!(
            btree.is_empty(),
            bdd.is_empty(),
            "is_empty() differs between BTree and BDD"
        );
    }

    // ---------------------------------------------------------------------------
    // Property Tests
    // ---------------------------------------------------------------------------

    proptest! {
        /// All implementations produce identical results for insert sequences.
        #[test]
        fn all_impls_equivalent_insert_sequence(
            ops in prop::collection::vec(0u32..1000, 0..100)
        ) {
            let (mut btree, mut roaring, mut bdd) = create_all_impls();

            for val in ops {
                let loc = LocId::new(u128::from(val));

                let btree_changed = btree.insert(loc);
                let roaring_changed = roaring.insert(loc);
                let bdd_changed = bdd.insert(loc);

                // Insert return values should match
                assert_eq!(
                    btree_changed, roaring_changed,
                    "insert() return differs for {:?}",
                    loc
                );
                assert_eq!(
                    btree_changed, bdd_changed,
                    "insert() return differs for {:?}",
                    loc
                );
            }

            assert_all_equal(&btree, &roaring, &bdd);
        }

        /// All implementations produce identical results for union operations.
        #[test]
        fn all_impls_equivalent_union(
            set_a in prop::collection::vec(0u32..1000, 0..50),
            set_b in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            // Create set A
            let mut btree_a = BTreePtsSet::empty();
            let mut roaring_a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_a = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_a {
                let loc = LocId::new(u128::from(*val));
                btree_a.insert(loc);
                roaring_a.insert(loc);
                bdd_a.insert(loc);
            }

            // Create set B
            let mut btree_b = BTreePtsSet::empty();
            let mut roaring_b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_b = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_b {
                let loc = LocId::new(u128::from(*val));
                btree_b.insert(loc);
                roaring_b.insert(loc);
                bdd_b.insert(loc);
            }

            // Perform union
            let btree_changed = btree_a.union(&btree_b);
            let roaring_changed = roaring_a.union(&roaring_b);
            let bdd_changed = bdd_a.union(&bdd_b);

            // Union return values should match
            assert_eq!(
                btree_changed, roaring_changed,
                "union() return differs between BTree and BitVec"
            );
            assert_eq!(
                btree_changed, bdd_changed,
                "union() return differs between BTree and BDD"
            );

            assert_all_equal(&btree_a, &roaring_a, &bdd_a);
        }

        /// All implementations produce identical results for intersect operations.
        #[test]
        fn all_impls_equivalent_intersect(
            set_a in prop::collection::vec(0u32..1000, 0..50),
            set_b in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            // Create set A
            let mut btree_a = BTreePtsSet::empty();
            let mut roaring_a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_a = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_a {
                let loc = LocId::new(u128::from(*val));
                btree_a.insert(loc);
                roaring_a.insert(loc);
                bdd_a.insert(loc);
            }

            // Create set B
            let mut btree_b = BTreePtsSet::empty();
            let mut roaring_b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_b = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_b {
                let loc = LocId::new(u128::from(*val));
                btree_b.insert(loc);
                roaring_b.insert(loc);
                bdd_b.insert(loc);
            }

            // Perform intersection
            let btree_changed = btree_a.intersect(&btree_b);
            let roaring_changed = roaring_a.intersect(&roaring_b);
            let bdd_changed = bdd_a.intersect(&bdd_b);

            // Intersect return values should match
            assert_eq!(
                btree_changed, roaring_changed,
                "intersect() return differs between BTree and BitVec"
            );
            assert_eq!(
                btree_changed, bdd_changed,
                "intersect() return differs between BTree and BDD"
            );

            assert_all_equal(&btree_a, &roaring_a, &bdd_a);
        }

        /// All implementations produce identical results for difference operations.
        #[test]
        fn all_impls_equivalent_difference(
            set_a in prop::collection::vec(0u32..1000, 0..50),
            set_b in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            // Create set A
            let mut btree_a = BTreePtsSet::empty();
            let mut roaring_a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_a = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_a {
                let loc = LocId::new(u128::from(*val));
                btree_a.insert(loc);
                roaring_a.insert(loc);
                bdd_a.insert(loc);
            }

            // Create set B
            let mut btree_b = BTreePtsSet::empty();
            let mut roaring_b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_b = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_b {
                let loc = LocId::new(u128::from(*val));
                btree_b.insert(loc);
                roaring_b.insert(loc);
                bdd_b.insert(loc);
            }

            // Perform difference (A - B)
            let btree_changed = btree_a.difference(&btree_b);
            let roaring_changed = roaring_a.difference(&roaring_b);
            let bdd_changed = bdd_a.difference(&bdd_b);

            // Difference return values should match
            assert_eq!(
                btree_changed, roaring_changed,
                "difference() return differs between BTree and BitVec"
            );
            assert_eq!(
                btree_changed, bdd_changed,
                "difference() return differs between BTree and BDD"
            );

            assert_all_equal(&btree_a, &roaring_a, &bdd_a);
        }

        /// All implementations agree on `contains()` queries.
        #[test]
        fn all_impls_equivalent_contains(
            elements in prop::collection::vec(0u32..1000, 0..50),
            queries in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let (mut btree, mut roaring, mut bdd) = create_all_impls();

            // Insert elements
            for val in &elements {
                let loc = LocId::new(u128::from(*val));
                btree.insert(loc);
                roaring.insert(loc);
                bdd.insert(loc);
            }

            // Query contains
            for val in &queries {
                let loc = LocId::new(u128::from(*val));

                let btree_contains = btree.contains(loc);
                let roaring_contains = roaring.contains(loc);
                let bdd_contains = bdd.contains(loc);

                assert_eq!(
                    btree_contains, roaring_contains,
                    "contains({:?}) differs between BTree and BitVec",
                    loc
                );
                assert_eq!(
                    btree_contains, bdd_contains,
                    "contains({:?}) differs between BTree and BDD",
                    loc
                );
            }
        }

        /// All implementations agree on `is_subset()` predicate.
        #[test]
        fn all_impls_equivalent_is_subset(
            set_a in prop::collection::vec(0u32..1000, 0..30),
            set_b in prop::collection::vec(0u32..1000, 0..30)
        ) {
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            // Create set A
            let mut btree_a = BTreePtsSet::empty();
            let mut roaring_a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_a = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_a {
                let loc = LocId::new(u128::from(*val));
                btree_a.insert(loc);
                roaring_a.insert(loc);
                bdd_a.insert(loc);
            }

            // Create set B
            let mut btree_b = BTreePtsSet::empty();
            let mut roaring_b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_b = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_b {
                let loc = LocId::new(u128::from(*val));
                btree_b.insert(loc);
                roaring_b.insert(loc);
                bdd_b.insert(loc);
            }

            // Check subset relationship both ways
            let btree_a_subset_b = btree_a.is_subset(&btree_b);
            let roaring_a_subset_b = roaring_a.is_subset(&roaring_b);
            let bdd_a_subset_b = bdd_a.is_subset(&bdd_b);

            assert_eq!(
                btree_a_subset_b, roaring_a_subset_b,
                "is_subset() differs between BTree and BitVec"
            );
            assert_eq!(
                btree_a_subset_b, bdd_a_subset_b,
                "is_subset() differs between BTree and BDD"
            );

            let btree_b_subset_a = btree_b.is_subset(&btree_a);
            let roaring_b_subset_a = roaring_b.is_subset(&roaring_a);
            let bdd_b_subset_a = bdd_b.is_subset(&bdd_a);

            assert_eq!(
                btree_b_subset_a, roaring_b_subset_a,
                "is_subset() (reverse) differs between BTree and BitVec"
            );
            assert_eq!(
                btree_b_subset_a, bdd_b_subset_a,
                "is_subset() (reverse) differs between BTree and BDD"
            );
        }

        /// All implementations agree on `intersects()` predicate.
        #[test]
        fn all_impls_equivalent_intersects(
            set_a in prop::collection::vec(0u32..1000, 0..30),
            set_b in prop::collection::vec(0u32..1000, 0..30)
        ) {
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            // Create set A
            let mut btree_a = BTreePtsSet::empty();
            let mut roaring_a = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_a = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_a {
                let loc = LocId::new(u128::from(*val));
                btree_a.insert(loc);
                roaring_a.insert(loc);
                bdd_a.insert(loc);
            }

            // Create set B
            let mut btree_b = BTreePtsSet::empty();
            let mut roaring_b = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd_b = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for val in &set_b {
                let loc = LocId::new(u128::from(*val));
                btree_b.insert(loc);
                roaring_b.insert(loc);
                bdd_b.insert(loc);
            }

            let btree_intersects = btree_a.intersects(&btree_b);
            let roaring_intersects = roaring_a.intersects(&roaring_b);
            let bdd_intersects = bdd_a.intersects(&bdd_b);

            assert_eq!(
                btree_intersects, roaring_intersects,
                "intersects() differs between BTree and BitVec"
            );
            assert_eq!(
                btree_intersects, bdd_intersects,
                "intersects() differs between BTree and BDD"
            );
        }

        /// All implementations produce identical iteration order (sorted by `LocId`).
        #[test]
        fn all_impls_equivalent_iteration_order(
            elements in prop::collection::vec(0u32..1000, 0..50)
        ) {
            let (mut btree, mut roaring, mut bdd) = create_all_impls();

            // Insert in random order
            for val in &elements {
                let loc = LocId::new(u128::from(*val));
                btree.insert(loc);
                roaring.insert(loc);
                bdd.insert(loc);
            }

            // Collect iteration results
            let btree_iter: Vec<LocId> = btree.iter().collect();
            let roaring_iter: Vec<LocId> = roaring.iter().collect();
            let bdd_iter: Vec<LocId> = bdd.iter().collect();

            assert_eq!(
                btree_iter, roaring_iter,
                "Iteration order differs between BTree and BitVec"
            );
            assert_eq!(
                btree_iter, bdd_iter,
                "Iteration order differs between BTree and BDD"
            );

            // Verify the iteration order is sorted
            let mut sorted = btree_iter.clone();
            sorted.sort();
            assert_eq!(
                btree_iter, sorted,
                "Iteration order should be sorted by LocId"
            );
        }

        /// All implementations handle `remove()` consistently.
        #[test]
        fn all_impls_equivalent_remove(
            elements in prop::collection::vec(0u32..1000, 0..50),
            removals in prop::collection::vec(0u32..1000, 0..30)
        ) {
            let (mut btree, mut roaring, mut bdd) = create_all_impls();

            // Insert elements
            for val in &elements {
                let loc = LocId::new(u128::from(*val));
                btree.insert(loc);
                roaring.insert(loc);
                bdd.insert(loc);
            }

            // Remove elements
            for val in &removals {
                let loc = LocId::new(u128::from(*val));

                let btree_removed = btree.remove(loc);
                let roaring_removed = roaring.remove(loc);
                let bdd_removed = bdd.remove(loc);

                assert_eq!(
                    btree_removed, roaring_removed,
                    "remove() return differs for {:?}",
                    loc
                );
                assert_eq!(
                    btree_removed, bdd_removed,
                    "remove() return differs for {:?}",
                    loc
                );
            }

            assert_all_equal(&btree, &roaring, &bdd);
        }

        /// `from_btreeset` and `to_btreeset` are inverse operations.
        #[test]
        fn from_to_btreeset_roundtrip(
            elements in prop::collection::vec(0u32..1000, 0..50)
        ) {
            // Create a BTreeSet
            let mut original: BTreeSet<LocId> = BTreeSet::new();
            for val in &elements {
                original.insert(LocId::new(u128::from(*val)));
            }

            // Roundtrip through each implementation
            let btree = BTreePtsSet::from_btreeset(&original);
            assert_eq!(btree.to_btreeset(), original, "BTree roundtrip failed");

            // Note: BitVec and BDD require shared indexer for from_btreeset,
            // so we test via singleton + insert pattern instead
            let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
            let context = Arc::new(RwLock::new(BddContext::new(16)));

            let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
            let mut bdd = BddPtsSet::with_context_and_indexer(
                Arc::clone(&context),
                Arc::clone(&indexer),
            );

            for loc in &original {
                roaring.insert(*loc);
                bdd.insert(*loc);
            }

            assert_eq!(roaring.to_btreeset(), original, "Roaring contents differ");
            assert_eq!(bdd.to_btreeset(), original, "BDD contents differ");
        }
    }

    // ---------------------------------------------------------------------------
    // Unit Tests for Edge Cases
    // ---------------------------------------------------------------------------

    #[test]
    fn empty_sets_all_equal() {
        let (btree, roaring, bdd) = create_all_impls();
        assert_all_equal(&btree, &roaring, &bdd);
        assert!(btree.is_empty());
        assert!(roaring.is_empty());
        assert!(bdd.is_empty());
    }

    #[test]
    fn singleton_all_equal() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        let loc = LocId::new(42);

        let btree = BTreePtsSet::singleton(loc);
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        roaring.insert(loc);
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        bdd.insert(loc);

        assert_all_equal(&btree, &roaring, &bdd);
        assert_eq!(btree.len(), 1);
        assert!(btree.contains(loc));
    }

    #[test]
    fn union_with_empty_unchanged() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        let loc = LocId::new(100);

        let mut btree = BTreePtsSet::singleton(loc);
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        roaring.insert(loc);
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        bdd.insert(loc);

        let empty_btree = BTreePtsSet::empty();
        let empty_roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let empty_bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        // Union with empty should not change the set
        assert!(!btree.union(&empty_btree));
        assert!(!roaring.union(&empty_roaring));
        assert!(!bdd.union(&empty_bdd));

        assert_all_equal(&btree, &roaring, &bdd);
        assert_eq!(btree.len(), 1);
    }

    #[test]
    fn intersect_with_empty_clears() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        let loc = LocId::new(100);

        let mut btree = BTreePtsSet::singleton(loc);
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        roaring.insert(loc);
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        bdd.insert(loc);

        let empty_btree = BTreePtsSet::empty();
        let empty_roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let empty_bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        // Intersect with empty should clear the set
        assert!(btree.intersect(&empty_btree));
        assert!(roaring.intersect(&empty_roaring));
        assert!(bdd.intersect(&empty_bdd));

        assert_all_equal(&btree, &roaring, &bdd);
        assert!(btree.is_empty());
    }
}
