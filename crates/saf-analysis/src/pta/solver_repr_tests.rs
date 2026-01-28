//! Solver integration tests for points-to set representations.
//!
//! These tests verify that the PTA solver produces identical results regardless
//! of which `PtsSet` implementation is used (`BTreePtsSet`, `RoaringPtsSet`, `BddPtsSet`).

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use saf_core::ids::{LocId, ObjId, ValueId};

    use crate::FieldSensitivity;
    use crate::pta::constraint::{AddrConstraint, ConstraintSet, CopyConstraint};
    use crate::pta::location::{FieldPath, LocationFactory};
    use crate::pta::ptsset::{BTreePtsSet, BddPtsSet, PtsSet, RoaringPtsSet};
    use crate::pta::solver::{GenericPointsToMap, solve_generic};

    /// Helper to create a `LocationFactory` with default field sensitivity.
    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    /// Helper to create test constraints with addresses and copies.
    fn make_test_constraints() -> (ConstraintSet, LocationFactory, Vec<LocId>) {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        // Create some values
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);

        // Create locations
        let loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
        let loc2 = factory.get_or_create(ObjId::new(200), FieldPath::empty());
        let loc3 = factory.get_or_create(ObjId::new(300), FieldPath::empty());

        // v1 = &loc1 (address constraint)
        constraints
            .addr
            .insert(AddrConstraint { ptr: v1, loc: loc1 });

        // v2 = &loc2
        constraints
            .addr
            .insert(AddrConstraint { ptr: v2, loc: loc2 });

        // v3 = v1 (copy constraint)
        constraints.copy.insert(CopyConstraint { dst: v3, src: v1 });

        (constraints, factory, vec![loc1, loc2, loc3])
    }

    /// Convert a `GenericPointsToMap` to normalized `BTreeMap<ValueId, Vec<LocId>>`.
    fn normalize_pts_map<P: PtsSet>(pts: &GenericPointsToMap<P>) -> BTreeMap<ValueId, Vec<LocId>> {
        pts.iter()
            .map(|(vid, ptsset)| {
                let mut locs: Vec<LocId> = ptsset.to_btreeset().into_iter().collect();
                locs.sort();
                (*vid, locs)
            })
            .collect()
    }

    // ---------------------------------------------------------------------------
    // CI-PTA Representation Equivalence Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn ci_pta_btree_roaring_equivalent() {
        let (constraints, factory, _) = make_test_constraints();

        // Solve with BTree
        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        // Solve with BitVec
        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        // Normalize and compare
        let btree_normalized = normalize_pts_map(&btree_result);
        let roaring_normalized = normalize_pts_map(&roaring_result);

        assert_eq!(
            btree_normalized, roaring_normalized,
            "BTree and Roaring PTA results differ"
        );
    }

    #[test]
    fn ci_pta_btree_bdd_equivalent() {
        let (constraints, factory, _) = make_test_constraints();

        // Solve with BTree
        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        // Solve with BDD
        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        // Normalize and compare
        let btree_normalized = normalize_pts_map(&btree_result);
        let bdd_normalized = normalize_pts_map(&bdd_result);

        assert_eq!(
            btree_normalized, bdd_normalized,
            "BTree and BDD PTA results differ"
        );
    }

    #[test]
    fn ci_pta_all_representations_equivalent() {
        let (constraints, factory, _) = make_test_constraints();

        // Solve with all three representations
        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        // Normalize all
        let btree_normalized = normalize_pts_map(&btree_result);
        let roaring_normalized = normalize_pts_map(&roaring_result);
        let bdd_normalized = normalize_pts_map(&bdd_result);

        assert_eq!(
            btree_normalized, roaring_normalized,
            "BTree and Roaring differ"
        );
        assert_eq!(btree_normalized, bdd_normalized, "BTree and BDD differ");
        assert_eq!(roaring_normalized, bdd_normalized, "Roaring and BDD differ");
    }

    // ---------------------------------------------------------------------------
    // Complex Constraint Pattern Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn solver_cyclic_constraints_all_repr_equivalent() {
        // Test with cyclic constraint patterns
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);

        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        // v1 = &loc
        constraints.addr.insert(AddrConstraint { ptr: v1, loc });

        // v2 = v1
        constraints.copy.insert(CopyConstraint { dst: v2, src: v1 });

        // v3 = v2
        constraints.copy.insert(CopyConstraint { dst: v3, src: v2 });

        // v1 = v3 (creates cycle)
        constraints.copy.insert(CopyConstraint { dst: v1, src: v3 });

        // Solve with all representations
        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let btree_normalized = normalize_pts_map(&btree_result);
        let roaring_normalized = normalize_pts_map(&roaring_result);
        let bdd_normalized = normalize_pts_map(&bdd_result);

        assert_eq!(btree_normalized, roaring_normalized);
        assert_eq!(btree_normalized, bdd_normalized);

        // All three values should point to loc
        for locs in btree_normalized.values() {
            assert!(locs.contains(&loc), "Value should point to loc");
        }
    }

    #[test]
    fn solver_multiple_allocations_all_repr_equivalent() {
        // Test with many allocations
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        // Create 50 allocations
        let mut values = Vec::new();
        let mut locations = Vec::new();

        for i in 0..50 {
            let v = ValueId::new(i as u128);
            let loc = factory.get_or_create(ObjId::new((i + 100) as u128), FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr: v, loc });
            values.push(v);
            locations.push(loc);
        }

        // Add some copy constraints
        for i in 1..50 {
            constraints.copy.insert(CopyConstraint {
                dst: values[i],
                src: values[i - 1],
            });
        }

        // Solve with all representations
        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 100_000);

        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 100_000);

        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 100_000);

        let btree_normalized = normalize_pts_map(&btree_result);
        let roaring_normalized = normalize_pts_map(&roaring_result);
        let bdd_normalized = normalize_pts_map(&bdd_result);

        assert_eq!(btree_normalized, roaring_normalized);
        assert_eq!(btree_normalized, bdd_normalized);

        // Last value should point to all previous locations
        let last_pts = btree_normalized
            .get(&values[49])
            .expect("last value should have pts");
        assert_eq!(
            last_pts.len(),
            50,
            "last value should point to all 50 locations"
        );
    }

    // ---------------------------------------------------------------------------
    // Empty and Minimal Tests
    // ---------------------------------------------------------------------------

    #[test]
    fn solver_empty_constraints_all_repr() {
        let factory = make_factory();
        let constraints = ConstraintSet::default();

        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        assert!(btree_result.is_empty());
        assert!(roaring_result.is_empty());
        assert!(bdd_result.is_empty());
    }

    #[test]
    fn solver_single_addr_all_repr() {
        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        let v = ValueId::new(1);
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        constraints.addr.insert(AddrConstraint { ptr: v, loc });

        let btree_result: GenericPointsToMap<BTreePtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let roaring_result: GenericPointsToMap<RoaringPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let bdd_result: GenericPointsToMap<BddPtsSet> =
            solve_generic(&constraints, &factory, 10_000);

        let btree_normalized = normalize_pts_map(&btree_result);
        let roaring_normalized = normalize_pts_map(&roaring_result);
        let bdd_normalized = normalize_pts_map(&bdd_result);

        assert_eq!(btree_normalized, roaring_normalized);
        assert_eq!(btree_normalized, bdd_normalized);

        assert_eq!(btree_result.len(), 1);
        let pts = btree_result.get(&v).expect("v should have pts");
        assert!(pts.contains(loc));
    }
}
