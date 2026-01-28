//! Edge case unit tests for points-to set implementations.
//!
//! Tests specific edge cases and boundary conditions for each `PtsSet`
//! implementation that might not be covered by property tests.

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use saf_core::ids::LocId;

    use crate::pta::ptsset::{
        BTreePtsSet, BddContext, BddPtsSet, LocIdIndexer, PtsSet, RoaringPtsSet,
    };

    // ---------------------------------------------------------------------------
    // Roaring Edge Cases
    // ---------------------------------------------------------------------------

    #[test]
    fn roaring_large_sparse_set() {
        // Insert elements at widely spaced indices to test sparse bit vector behavior
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let mut pts = RoaringPtsSet::with_indexer(Arc::clone(&indexer));

        // Insert at indices that would create a sparse bit vector
        let loc_0 = LocId::new(0);
        let loc_1000 = LocId::new(1000);
        let loc_10000 = LocId::new(10000);

        pts.insert(loc_0);
        pts.insert(loc_1000);
        pts.insert(loc_10000);

        assert_eq!(pts.len(), 3);
        assert!(pts.contains(loc_0));
        assert!(pts.contains(loc_1000));
        assert!(pts.contains(loc_10000));

        // Verify iteration returns correct elements
        let collected: Vec<LocId> = pts.iter().collect();
        assert_eq!(collected.len(), 3);
        assert!(collected.contains(&loc_0));
        assert!(collected.contains(&loc_1000));
        assert!(collected.contains(&loc_10000));
    }

    #[test]
    fn roaring_shared_indexer_different_sets() {
        // Two sets sharing the same indexer should work correctly
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts1 = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let mut pts2 = RoaringPtsSet::with_indexer(Arc::clone(&indexer));

        let loc_a = LocId::new(100);
        let loc_b = LocId::new(200);
        let loc_c = LocId::new(300);

        // Insert different elements into each set
        pts1.insert(loc_a);
        pts1.insert(loc_b);

        pts2.insert(loc_b);
        pts2.insert(loc_c);

        // Verify each set has the correct contents
        assert_eq!(pts1.len(), 2);
        assert!(pts1.contains(loc_a));
        assert!(pts1.contains(loc_b));
        assert!(!pts1.contains(loc_c));

        assert_eq!(pts2.len(), 2);
        assert!(!pts2.contains(loc_a));
        assert!(pts2.contains(loc_b));
        assert!(pts2.contains(loc_c));

        // Union should work correctly
        let changed = pts1.union(&pts2);
        assert!(changed);
        assert_eq!(pts1.len(), 3);
        assert!(pts1.contains(loc_a));
        assert!(pts1.contains(loc_b));
        assert!(pts1.contains(loc_c));
    }

    // ---------------------------------------------------------------------------
    // BDD Edge Cases
    // ---------------------------------------------------------------------------

    #[test]
    fn bdd_dynamic_variable_growth() {
        // Start with small context and grow beyond initial encoding bits
        let context = Arc::new(RwLock::new(BddContext::new(4))); // 4 bits = 16 values max initially
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        // Insert more than 16 elements to force variable growth
        for i in 0..32 {
            pts.insert(LocId::new(i));
        }

        assert_eq!(pts.len(), 32);

        // Verify all elements are present
        for i in 0..32 {
            assert!(
                pts.contains(LocId::new(i)),
                "Missing element {} after growth",
                i
            );
        }
    }

    #[test]
    fn bdd_encoding_boundary_values() {
        // Test at 2^n boundaries where encoding might have edge cases
        let context = Arc::new(RwLock::new(BddContext::new(16)));
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        // Boundary values (note: 65535 == (1<<16)-1 and 65536 == 1<<16, so 6 unique)
        let boundaries = [0u128, 1, 255, 256, 65535, 65536];

        for val in boundaries {
            pts.insert(LocId::new(val));
        }

        assert_eq!(pts.len(), boundaries.len());

        for val in boundaries {
            assert!(
                pts.contains(LocId::new(val)),
                "Missing boundary value {}",
                val
            );
        }
    }

    #[test]
    fn bdd_shared_context_different_sets() {
        // Two sets sharing the same context and indexer should work correctly
        let context = Arc::new(RwLock::new(BddContext::new(16)));
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts1 =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        let mut pts2 =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        let loc_a = LocId::new(100);
        let loc_b = LocId::new(200);
        let loc_c = LocId::new(300);

        pts1.insert(loc_a);
        pts1.insert(loc_b);

        pts2.insert(loc_b);
        pts2.insert(loc_c);

        // Verify independent storage
        assert_eq!(pts1.len(), 2);
        assert_eq!(pts2.len(), 2);

        assert!(pts1.contains(loc_a));
        assert!(!pts2.contains(loc_a));

        // Union should work
        let changed = pts1.union(&pts2);
        assert!(changed);
        assert_eq!(pts1.len(), 3);
    }

    #[test]
    fn bdd_set_operations_preserve_structure() {
        // Test that BDD operations maintain valid BDD structure
        let context = Arc::new(RwLock::new(BddContext::new(16)));
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        let mut pts1 =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        let mut pts2 =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        // Create overlapping sets
        for i in 0..50 {
            pts1.insert(LocId::new(i));
        }
        for i in 25..75 {
            pts2.insert(LocId::new(i));
        }

        // Clone before operations
        let pts1_original = pts1.clone();

        // Test union
        pts1.union(&pts2);
        assert_eq!(pts1.len(), 75); // 0-74

        // Test intersection
        let mut pts_intersect = pts1_original.clone();
        pts_intersect.intersect(&pts2);
        assert_eq!(pts_intersect.len(), 25); // 25-49

        // Test difference
        let mut pts_diff = pts1_original.clone();
        pts_diff.difference(&pts2);
        assert_eq!(pts_diff.len(), 25); // 0-24
    }

    // ---------------------------------------------------------------------------
    // Indexer Edge Cases
    // ---------------------------------------------------------------------------

    #[test]
    fn indexer_concurrent_reads() {
        // Verify RwLock behavior with multiple readers
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));

        // Pre-populate indexer
        {
            let mut idx = indexer.write().unwrap();
            for i in 0..100 {
                idx.get_or_insert(LocId::new(i));
            }
        }

        // Multiple concurrent reads should work
        let idx1 = indexer.read().unwrap();
        let idx2 = indexer.read().unwrap();

        assert_eq!(idx1.len(), 100);
        assert_eq!(idx2.len(), 100);

        // Both readers should see the same data
        for i in 0..100 {
            let loc = LocId::new(i);
            assert_eq!(idx1.get(loc), idx2.get(loc));
        }
    }

    #[test]
    fn indexer_deterministic_ordering() {
        // Verify indexer assigns indices in insertion order
        let mut indexer = LocIdIndexer::new();

        // Insert in non-sequential order
        let locs = [
            LocId::new(100),
            LocId::new(1),
            LocId::new(50),
            LocId::new(25),
        ];

        for loc in &locs {
            indexer.get_or_insert(*loc);
        }

        // Indices should be assigned in insertion order
        assert_eq!(indexer.get(locs[0]), Some(0));
        assert_eq!(indexer.get(locs[1]), Some(1));
        assert_eq!(indexer.get(locs[2]), Some(2));
        assert_eq!(indexer.get(locs[3]), Some(3));
    }

    // ---------------------------------------------------------------------------
    // Clone Semantics
    // ---------------------------------------------------------------------------

    #[test]
    fn clone_is_independent() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        // BTreePtsSet
        let mut btree = BTreePtsSet::empty();
        btree.insert(LocId::new(1));
        let mut btree_clone = btree.clone();
        btree_clone.insert(LocId::new(2));

        assert_eq!(btree.len(), 1);
        assert_eq!(btree_clone.len(), 2);

        // RoaringPtsSet
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        roaring.insert(LocId::new(1));
        let mut roaring_clone = roaring.clone();
        roaring_clone.insert(LocId::new(2));

        assert_eq!(roaring.len(), 1);
        assert_eq!(roaring_clone.len(), 2);

        // BddPtsSet
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
        bdd.insert(LocId::new(1));
        let mut bdd_clone = bdd.clone();
        bdd_clone.insert(LocId::new(2));

        assert_eq!(bdd.len(), 1);
        assert_eq!(bdd_clone.len(), 2);
    }

    // ---------------------------------------------------------------------------
    // Zero/Max LocId Values
    // ---------------------------------------------------------------------------

    #[test]
    fn handles_zero_loc_id() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        let loc_zero = LocId::new(0);

        let mut btree = BTreePtsSet::empty();
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        btree.insert(loc_zero);
        roaring.insert(loc_zero);
        bdd.insert(loc_zero);

        assert!(btree.contains(loc_zero));
        assert!(roaring.contains(loc_zero));
        assert!(bdd.contains(loc_zero));

        assert_eq!(btree.len(), 1);
        assert_eq!(roaring.len(), 1);
        assert_eq!(bdd.len(), 1);
    }

    #[test]
    fn handles_large_loc_id() {
        let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
        let context = Arc::new(RwLock::new(BddContext::new(16)));

        // Large but not max to avoid potential overflow issues
        let loc_large = LocId::new(u128::MAX / 2);

        let mut btree = BTreePtsSet::empty();
        let mut roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
        let mut bdd =
            BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

        btree.insert(loc_large);
        roaring.insert(loc_large);
        bdd.insert(loc_large);

        assert!(btree.contains(loc_large));
        assert!(roaring.contains(loc_large));
        assert!(bdd.contains(loc_large));
    }
}
