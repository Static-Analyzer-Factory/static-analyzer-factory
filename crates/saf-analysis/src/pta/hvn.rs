//! Hash-based Value Numbering (HVN) pre-processing for PTA constraint reduction.
//!
//! HVN identifies pointer values that must have identical points-to solutions
//! based on their constraint signatures. Values with the same signature are
//! merged into equivalence classes, and constraints are rewritten to use a
//! single representative per class.
//!
//! This reduces the number of values the solver needs to track and the number
//! of constraints it needs to propagate, improving scalability for large
//! programs.
//!
//! # Algorithm
//!
//! 1. Scan all constraints to compute a signature for each `ValueId`:
//!    - `AddrOnly(BTreeSet<LocId>)` — value only appears in Addr constraints
//!    - `CopyTarget(BTreeSet<ValueId>)` — value is dst of Copy from these sources
//!    - `Complex` — value appears in Load/Store/Gep
//!
//! 2. Group values with identical signatures into equivalence classes.
//!
//! 3. For each class with >1 member, pick the minimum `ValueId` as representative.
//!
//! 4. Rewrite all constraints in-place, replacing non-representative values
//!    with their representative. Deduplication is automatic via `BTreeSet`.
//!
//! 5. Remove self-constraints (e.g., `copy(a, a)` after rewriting).

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{LocId, ValueId};

use super::constraint::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};

// =============================================================================
// HVN result
// =============================================================================

/// Result of HVN pre-processing.
#[derive(Debug, Clone)]
pub struct HvnResult {
    /// Mapping from non-representative values to their representative.
    pub mapping: BTreeMap<ValueId, ValueId>,
    /// Number of equivalence classes found with >1 member.
    pub num_classes: usize,
    /// Number of constraints removed by deduplication and self-constraint elimination.
    pub removed: usize,
}

// =============================================================================
// Value signatures
// =============================================================================

/// Signature describing a value's role in the constraint system.
///
/// Two values with the same signature will have identical points-to solutions
/// and can be merged.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ValueSignature {
    /// Value only appears in `Addr` constraints pointing to these locations.
    AddrOnly(BTreeSet<LocId>),
    /// Value is the destination of `Copy` constraints from exactly these sources,
    /// and does not appear in any Load/Store/Gep constraint.
    CopyTarget(BTreeSet<ValueId>),
    /// Value appears in Load, Store, or Gep constraints (cannot be safely merged).
    Complex,
}

// =============================================================================
// HVN algorithm
// =============================================================================

/// Run HVN pre-processing on a constraint set.
///
/// Identifies values with identical constraint signatures, merges them into
/// equivalence classes, rewrites constraints in-place, and removes duplicates
/// and self-constraints.
///
/// Returns an `HvnResult` containing the mapping (for expanding results after
/// solving) and statistics.
pub fn hvn_preprocess(constraints: &mut ConstraintSet) -> HvnResult {
    let original_count = constraints.total_count();

    // Phase 1: Compute signatures
    let signatures = compute_signatures(constraints);

    // Phase 2: Group by signature, build mapping
    let mapping = build_mapping(&signatures);

    let num_classes = count_classes(&mapping);

    if mapping.is_empty() {
        return HvnResult {
            mapping,
            num_classes: 0,
            removed: 0,
        };
    }

    // Phase 3: Rewrite constraints in-place
    rewrite_constraints(constraints, &mapping);

    let new_count = constraints.total_count();
    let removed = original_count.saturating_sub(new_count);

    HvnResult {
        mapping,
        num_classes,
        removed,
    }
}

/// Compute a `ValueSignature` for each `ValueId` appearing in the constraint set.
fn compute_signatures(constraints: &ConstraintSet) -> BTreeMap<ValueId, ValueSignature> {
    let mut sigs: BTreeMap<ValueId, ValueSignature> = BTreeMap::new();

    // Process Addr constraints: accumulate locations for each ptr
    for addr in &constraints.addr {
        let entry = sigs
            .entry(addr.ptr)
            .or_insert_with(|| ValueSignature::AddrOnly(BTreeSet::new()));
        if let ValueSignature::AddrOnly(locs) = entry {
            locs.insert(addr.loc);
        }
        // If already Complex, leave it as Complex
    }

    // Process Copy constraints: track dst as CopyTarget, src is unaffected
    for copy in &constraints.copy {
        // Mark dst as CopyTarget (unless already Complex)
        let entry = sigs
            .entry(copy.dst)
            .or_insert_with(|| ValueSignature::CopyTarget(BTreeSet::new()));
        match entry {
            ValueSignature::CopyTarget(srcs) => {
                srcs.insert(copy.src);
            }
            ValueSignature::AddrOnly(_) => {
                // Value has both Addr and Copy — promote to Complex
                *entry = ValueSignature::Complex;
            }
            ValueSignature::Complex => {}
        }
    }

    // Process Load constraints: mark both dst and src_ptr as Complex
    for load in &constraints.load {
        sigs.insert(load.dst, ValueSignature::Complex);
        sigs.insert(load.src_ptr, ValueSignature::Complex);
    }

    // Process Store constraints: mark both dst_ptr and src as Complex
    for store in &constraints.store {
        sigs.insert(store.dst_ptr, ValueSignature::Complex);
        sigs.insert(store.src, ValueSignature::Complex);
    }

    // Process Gep constraints: mark dst, src_ptr, and index_operands as Complex
    for gep in &constraints.gep {
        sigs.insert(gep.dst, ValueSignature::Complex);
        sigs.insert(gep.src_ptr, ValueSignature::Complex);
        for &idx in &gep.index_operands {
            sigs.insert(idx, ValueSignature::Complex);
        }
    }

    sigs
}

/// Group values by identical signature and build a mapping from non-representative
/// values to their representative (minimum `ValueId` in each group).
fn build_mapping(signatures: &BTreeMap<ValueId, ValueSignature>) -> BTreeMap<ValueId, ValueId> {
    // Group values by signature (only non-Complex signatures can be merged)
    let mut groups: BTreeMap<&ValueSignature, Vec<ValueId>> = BTreeMap::new();

    for (vid, sig) in signatures {
        if matches!(sig, ValueSignature::Complex) {
            continue;
        }
        groups.entry(sig).or_default().push(*vid);
    }

    let mut mapping = BTreeMap::new();

    for members in groups.values() {
        if members.len() <= 1 {
            continue;
        }
        // Pick representative: minimum ValueId (members are already sorted since
        // we iterate a BTreeMap keyed by ValueId)
        let rep = members
            .iter()
            .copied()
            .min()
            .expect("group has at least 2 members");

        for &vid in members {
            if vid != rep {
                mapping.insert(vid, rep);
            }
        }
    }

    mapping
}

/// Count the number of distinct equivalence classes with >1 member.
fn count_classes(mapping: &BTreeMap<ValueId, ValueId>) -> usize {
    let reps: BTreeSet<ValueId> = mapping.values().copied().collect();
    reps.len()
}

/// Apply a value mapping: return the representative if one exists, otherwise the original.
fn map_value(v: ValueId, mapping: &BTreeMap<ValueId, ValueId>) -> ValueId {
    mapping.get(&v).copied().unwrap_or(v)
}

/// Rewrite all constraints in-place using the mapping, then deduplicate and
/// remove self-constraints.
fn rewrite_constraints(constraints: &mut ConstraintSet, mapping: &BTreeMap<ValueId, ValueId>) {
    // Rewrite Addr constraints
    let addr: BTreeSet<AddrConstraint> = constraints
        .addr
        .iter()
        .map(|a| AddrConstraint {
            ptr: map_value(a.ptr, mapping),
            loc: a.loc,
        })
        .collect();
    constraints.addr = addr;

    // Rewrite Copy constraints (and remove self-copies)
    let copy: BTreeSet<CopyConstraint> = constraints
        .copy
        .iter()
        .map(|c| CopyConstraint {
            dst: map_value(c.dst, mapping),
            src: map_value(c.src, mapping),
        })
        .filter(|c| c.dst != c.src)
        .collect();
    constraints.copy = copy;

    // Rewrite Load constraints
    let load: BTreeSet<LoadConstraint> = constraints
        .load
        .iter()
        .map(|l| LoadConstraint {
            dst: map_value(l.dst, mapping),
            src_ptr: map_value(l.src_ptr, mapping),
        })
        .collect();
    constraints.load = load;

    // Rewrite Store constraints
    let store: BTreeSet<StoreConstraint> = constraints
        .store
        .iter()
        .map(|s| StoreConstraint {
            dst_ptr: map_value(s.dst_ptr, mapping),
            src: map_value(s.src, mapping),
        })
        .collect();
    constraints.store = store;

    // Rewrite Gep constraints
    let gep: BTreeSet<GepConstraint> = constraints
        .gep
        .iter()
        .map(|g| GepConstraint {
            dst: map_value(g.dst, mapping),
            src_ptr: map_value(g.src_ptr, mapping),
            path: g.path.clone(),
            index_operands: g
                .index_operands
                .iter()
                .map(|&v| map_value(v, mapping))
                .collect(),
        })
        .collect();
    constraints.gep = gep;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Two values with Addr to the same location should be merged.
    #[test]
    fn test_addr_only_merge() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let loc = LocId::new(100);

        // Both v1 and v2 point to the same location
        cs.addr.insert(AddrConstraint { ptr: v1, loc });
        cs.addr.insert(AddrConstraint { ptr: v2, loc });

        let result = hvn_preprocess(&mut cs);

        // v2 should be mapped to v1 (min ValueId)
        assert_eq!(result.mapping.get(&v2), Some(&v1));
        assert!(!result.mapping.contains_key(&v1));
        assert_eq!(result.num_classes, 1);

        // After rewriting, only one Addr constraint should remain
        assert_eq!(cs.addr.len(), 1);
        assert!(cs.addr.iter().any(|a| a.ptr == v1 && a.loc == loc));
    }

    /// Two values that are Copy destinations from the same source should be merged.
    #[test]
    fn test_copy_target_merge() {
        let mut cs = ConstraintSet::default();
        let src = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);

        // Both v2 and v3 are copies of src
        cs.copy.insert(CopyConstraint { dst: v2, src });
        cs.copy.insert(CopyConstraint { dst: v3, src });

        let result = hvn_preprocess(&mut cs);

        // v3 should be mapped to v2 (min of {v2, v3})
        assert_eq!(result.mapping.get(&v3), Some(&v2));
        assert!(!result.mapping.contains_key(&v2));
        assert_eq!(result.num_classes, 1);

        // After rewriting, only one Copy constraint should remain: v2 = src
        assert_eq!(cs.copy.len(), 1);
        assert!(cs.copy.iter().any(|c| c.dst == v2 && c.src == src));
    }

    /// Values involved in Load/Store/Gep should NOT be merged, even if they
    /// have similar structure.
    #[test]
    fn test_complex_not_merged() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);
        let v4 = ValueId::new(4);

        // v1 and v2 appear in Load constraints
        cs.load.insert(LoadConstraint {
            dst: v1,
            src_ptr: v3,
        });
        cs.load.insert(LoadConstraint {
            dst: v2,
            src_ptr: v4,
        });

        let result = hvn_preprocess(&mut cs);

        // No merging should occur — all values are Complex
        assert!(result.mapping.is_empty());
        assert_eq!(result.num_classes, 0);
        assert_eq!(cs.load.len(), 2);
    }

    /// Self-copy constraints (`copy(v, v)`) created by rewriting are removed.
    ///
    /// Uses `rewrite_constraints` directly to test the self-copy filter,
    /// since the HVN signature rules make it hard to organically produce
    /// a self-copy through the full pipeline.
    #[test]
    fn test_self_constraint_removal() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);

        // copy(v2, v1) — after mapping v2 → v1, becomes copy(v1, v1) = self-copy
        cs.copy.insert(CopyConstraint { dst: v2, src: v1 });
        // copy(v3, v2) — after mapping v2 → v1, becomes copy(v3, v1) = valid
        cs.copy.insert(CopyConstraint { dst: v3, src: v2 });

        // Manually build a mapping: v2 → v1
        let mut mapping = BTreeMap::new();
        mapping.insert(v2, v1);

        rewrite_constraints(&mut cs, &mapping);

        // Self-copy removed, only copy(v3, v1) remains
        assert_eq!(cs.copy.len(), 1);
        assert!(cs.copy.iter().any(|c| c.dst == v3 && c.src == v1));
    }

    /// Total constraint count should decrease after HVN.
    #[test]
    fn test_constraint_reduction() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);
        let loc = LocId::new(100);

        // v1 and v2 both point to loc (AddrOnly with same sig)
        cs.addr.insert(AddrConstraint { ptr: v1, loc });
        cs.addr.insert(AddrConstraint { ptr: v2, loc });

        // copy(v3, v2) — v2 as src doesn't affect its signature
        // After merge v2 → v1: copy(v3, v1)
        cs.copy.insert(CopyConstraint { dst: v3, src: v2 });

        // copy(v3, v1) — duplicates the rewritten constraint above
        cs.copy.insert(CopyConstraint { dst: v3, src: v1 });

        let original_count = cs.total_count(); // 2 addr + 2 copy = 4

        let result = hvn_preprocess(&mut cs);

        let new_count = cs.total_count(); // 1 addr + 1 copy = 2
        assert!(new_count < original_count);
        assert_eq!(result.removed, original_count - new_count);
    }

    /// Solving with and without HVN should produce equivalent normalized results
    /// (after expanding the mapping).
    #[test]
    fn test_results_equivalent() {
        use super::super::config::FieldSensitivity;
        use super::super::location::{FieldPath, LocationFactory};
        use super::super::solver::{PointsToMap, solve};
        use saf_core::ids::ObjId;

        let mut factory = LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 });

        // Build a constraint set with mergeable values
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);
        let v4 = ValueId::new(4);
        let obj_a = ObjId::new(100);
        let obj_b = ObjId::new(200);
        let loc_a = factory.get_or_create(obj_a, FieldPath::empty());
        let loc_b = factory.get_or_create(obj_b, FieldPath::empty());

        let mut cs_orig = ConstraintSet::default();

        // v1 -> loc_a, v2 -> loc_a (same AddrOnly sig)
        cs_orig.addr.insert(AddrConstraint {
            ptr: v1,
            loc: loc_a,
        });
        cs_orig.addr.insert(AddrConstraint {
            ptr: v2,
            loc: loc_a,
        });

        // v3 -> loc_b
        cs_orig.addr.insert(AddrConstraint {
            ptr: v3,
            loc: loc_b,
        });

        // v4 = copy(v1), also v4 = copy(v3) → v4 points to {loc_a, loc_b}
        cs_orig.copy.insert(CopyConstraint { dst: v4, src: v1 });
        cs_orig.copy.insert(CopyConstraint { dst: v4, src: v3 });

        // Solve without HVN
        let result_no_hvn: PointsToMap = solve(&cs_orig, &factory, 1_000_000);

        // Solve with HVN
        let mut cs_hvn = cs_orig.clone();
        let hvn_result = hvn_preprocess(&mut cs_hvn);
        let mut result_hvn: PointsToMap = solve(&cs_hvn, &factory, 1_000_000);

        // Expand: copy representative's pts to original values
        for (original, rep) in &hvn_result.mapping {
            if let Some(pts) = result_hvn.get(rep).cloned() {
                result_hvn.insert(*original, pts);
            }
        }

        // Compare for each value in the original result
        for (vid, pts_no_hvn) in &result_no_hvn {
            let pts_hvn = result_hvn.get(vid);
            assert_eq!(
                Some(pts_no_hvn),
                pts_hvn,
                "Points-to sets differ for {:?}: no_hvn={:?}, hvn={:?}",
                vid,
                pts_no_hvn,
                pts_hvn,
            );
        }
    }

    /// Values with different Addr signatures should NOT be merged.
    #[test]
    fn test_different_addr_not_merged() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let loc_a = LocId::new(100);
        let loc_b = LocId::new(200);

        cs.addr.insert(AddrConstraint {
            ptr: v1,
            loc: loc_a,
        });
        cs.addr.insert(AddrConstraint {
            ptr: v2,
            loc: loc_b,
        });

        let result = hvn_preprocess(&mut cs);
        assert!(result.mapping.is_empty());
        assert_eq!(cs.addr.len(), 2);
    }

    /// Empty constraint set should produce no changes.
    #[test]
    fn test_empty_constraints() {
        let mut cs = ConstraintSet::default();
        let result = hvn_preprocess(&mut cs);
        assert!(result.mapping.is_empty());
        assert_eq!(result.num_classes, 0);
        assert_eq!(result.removed, 0);
    }

    /// Values with Addr + Copy signatures should be Complex and not merged.
    #[test]
    fn test_addr_plus_copy_is_complex() {
        let mut cs = ConstraintSet::default();
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let v3 = ValueId::new(3);
        let loc = LocId::new(100);

        // v1 has Addr
        cs.addr.insert(AddrConstraint { ptr: v1, loc });
        // v1 also is Copy dst → becomes Complex
        cs.copy.insert(CopyConstraint { dst: v1, src: v2 });

        // v3 also has Addr to same loc
        cs.addr.insert(AddrConstraint { ptr: v3, loc });
        // v3 also is Copy dst → becomes Complex
        cs.copy.insert(CopyConstraint { dst: v3, src: v2 });

        let result = hvn_preprocess(&mut cs);
        // Both v1 and v3 are Complex — should not merge
        assert!(result.mapping.is_empty());
    }
}
