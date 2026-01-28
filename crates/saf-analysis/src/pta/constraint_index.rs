//! Pre-built index for fast constraint lookup by `ValueId`.
//!
//! The solver worklist pops a `ValueId` whose points-to set changed and
//! needs to find all constraints that reference that value. Without an
//! index, every worklist pop scans all constraints of each type. This
//! module builds reverse maps so lookup is O(1) amortized.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use saf_core::ids::ValueId;
use smallvec::SmallVec;

/// `IndexMap` with `FxHash` for fast u128 key lookups in constraint indices.
type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

use super::constraint::{
    ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};

// =============================================================================
// Indexed constraints (Vec storage for O(1) index access)
// =============================================================================

/// Constraints stored as `Vec`s for O(1) indexed access.
///
/// The `ConstraintIndex` stores `usize` indices into these `Vec`s.
/// Built from the `BTreeSet`s in `ConstraintSet` to allow direct
/// element access by position.
#[derive(Debug, Clone)]
pub struct IndexedConstraints {
    /// Copy constraints as a `Vec`.
    pub copy: Vec<CopyConstraint>,
    /// Load constraints as a `Vec`.
    pub load: Vec<LoadConstraint>,
    /// Store constraints as a `Vec`.
    pub store: Vec<StoreConstraint>,
    /// GEP constraints as a `Vec`.
    pub gep: Vec<GepConstraint>,
}

// =============================================================================
// Constraint index
// =============================================================================

/// Index for O(1) constraint lookup by source/destination `ValueId`.
///
/// Built once before solving; each `handle_*` method in the solver
/// looks up only the constraints relevant to the current worklist value
/// instead of scanning all constraints.
///
/// The `usize` values stored in each map are indices into the
/// corresponding `Vec` in `IndexedConstraints`.
#[derive(Debug, Clone)]
pub struct ConstraintIndex {
    /// Copy constraints where `src == v`.
    pub copy_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    /// Load constraints where `src_ptr == v`.
    pub load_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    /// Store constraints where `dst_ptr == v`.
    pub store_by_dst_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    /// Store constraints where `src == v`.
    pub store_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    /// GEP constraints where `src_ptr == v`.
    pub gep_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
}

impl ConstraintIndex {
    /// Build an index from a `ConstraintSet`.
    ///
    /// Converts the `BTreeSet` constraints into `Vec`s (for O(1) index
    /// access) and builds reverse maps keyed by the relevant `ValueId`
    /// field in each constraint.
    #[must_use]
    pub fn build(constraints: &ConstraintSet) -> (Self, IndexedConstraints) {
        // Convert BTreeSets → Vecs
        let copy: Vec<CopyConstraint> = constraints.copy.iter().cloned().collect();
        let load: Vec<LoadConstraint> = constraints.load.iter().cloned().collect();
        let store: Vec<StoreConstraint> = constraints.store.iter().cloned().collect();
        let gep: Vec<GepConstraint> = constraints.gep.iter().cloned().collect();

        // Build reverse-index maps
        let mut copy_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>> = FxIndexMap::default();
        for (i, c) in copy.iter().enumerate() {
            copy_by_src.entry(c.src).or_default().push(i);
        }

        let mut load_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>> = FxIndexMap::default();
        for (i, c) in load.iter().enumerate() {
            load_by_src_ptr.entry(c.src_ptr).or_default().push(i);
        }

        let mut store_by_dst_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>> = FxIndexMap::default();
        let mut store_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>> = FxIndexMap::default();
        for (i, c) in store.iter().enumerate() {
            store_by_dst_ptr.entry(c.dst_ptr).or_default().push(i);
            store_by_src.entry(c.src).or_default().push(i);
        }

        let mut gep_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>> = FxIndexMap::default();
        for (i, c) in gep.iter().enumerate() {
            gep_by_src_ptr.entry(c.src_ptr).or_default().push(i);
        }

        let index = Self {
            copy_by_src,
            load_by_src_ptr,
            store_by_dst_ptr,
            store_by_src,
            gep_by_src_ptr,
        };

        let indexed = IndexedConstraints {
            copy,
            load,
            store,
            gep,
        };

        (index, indexed)
    }

    /// Look up copy constraint indices where `src == value`.
    #[must_use]
    pub fn copies_by_src(&self, value: ValueId) -> &[usize] {
        self.copy_by_src.get(&value).map_or(&[], |v| v.as_slice())
    }

    /// Look up load constraint indices where `src_ptr == value`.
    #[must_use]
    pub fn loads_by_src_ptr(&self, value: ValueId) -> &[usize] {
        self.load_by_src_ptr
            .get(&value)
            .map_or(&[], |v| v.as_slice())
    }

    /// Look up store constraint indices where `dst_ptr == value`.
    #[must_use]
    pub fn stores_by_dst_ptr(&self, value: ValueId) -> &[usize] {
        self.store_by_dst_ptr
            .get(&value)
            .map_or(&[], |v| v.as_slice())
    }

    /// Look up store constraint indices where `src == value`.
    #[must_use]
    pub fn stores_by_src(&self, value: ValueId) -> &[usize] {
        self.store_by_src.get(&value).map_or(&[], |v| v.as_slice())
    }

    /// Look up GEP constraint indices where `src_ptr == value`.
    #[must_use]
    pub fn geps_by_src_ptr(&self, value: ValueId) -> &[usize] {
        self.gep_by_src_ptr
            .get(&value)
            .map_or(&[], |v| v.as_slice())
    }

    // =========================================================================
    // Incremental constraint addition
    // =========================================================================

    /// Add a copy constraint incrementally.
    ///
    /// Appends the constraint to the `IndexedConstraints` copy vec and
    /// updates the reverse index.
    pub fn add_copy(&mut self, indexed: &mut IndexedConstraints, c: CopyConstraint) {
        let idx = indexed.copy.len();
        self.copy_by_src.entry(c.src).or_default().push(idx);
        indexed.copy.push(c);
    }

    /// Add a load constraint incrementally.
    ///
    /// Appends the constraint to the `IndexedConstraints` load vec and
    /// updates the reverse index.
    #[allow(dead_code)] // Public API for future online CG load/store handling
    pub fn add_load(&mut self, indexed: &mut IndexedConstraints, c: &LoadConstraint) {
        let idx = indexed.load.len();
        self.load_by_src_ptr.entry(c.src_ptr).or_default().push(idx);
        indexed.load.push(c.clone());
    }

    /// Add a store constraint incrementally.
    ///
    /// Appends the constraint to the `IndexedConstraints` store vec and
    /// updates both reverse indexes (`store_by_dst_ptr` and `store_by_src`).
    #[allow(dead_code)] // Public API for future online CG load/store handling
    pub fn add_store(&mut self, indexed: &mut IndexedConstraints, c: &StoreConstraint) {
        let idx = indexed.store.len();
        self.store_by_dst_ptr
            .entry(c.dst_ptr)
            .or_default()
            .push(idx);
        self.store_by_src.entry(c.src).or_default().push(idx);
        indexed.store.push(c.clone());
    }

    /// Add a GEP constraint incrementally.
    ///
    /// Appends the constraint to the `IndexedConstraints` gep vec and
    /// updates the reverse index.
    #[allow(dead_code)] // Public API for future online CG GEP handling
    pub fn add_gep(&mut self, indexed: &mut IndexedConstraints, c: &GepConstraint) {
        let idx = indexed.gep.len();
        self.gep_by_src_ptr.entry(c.src_ptr).or_default().push(idx);
        indexed.gep.push(c.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::super::constraint::AddrConstraint;
    use super::super::location::FieldPath;
    use super::*;
    use saf_core::ids::{LocId, ValueId};

    fn make_constraint_set() -> ConstraintSet {
        let mut cs = ConstraintSet::default();

        // Addr constraints (not indexed, but part of the set)
        cs.addr.insert(AddrConstraint {
            ptr: ValueId::new(1),
            loc: LocId::new(100),
        });

        // Copy: v2 = v1, v3 = v1
        cs.copy.insert(CopyConstraint {
            dst: ValueId::new(2),
            src: ValueId::new(1),
        });
        cs.copy.insert(CopyConstraint {
            dst: ValueId::new(3),
            src: ValueId::new(1),
        });
        // Copy: v4 = v2
        cs.copy.insert(CopyConstraint {
            dst: ValueId::new(4),
            src: ValueId::new(2),
        });

        // Load: v5 = *v1
        cs.load.insert(LoadConstraint {
            dst: ValueId::new(5),
            src_ptr: ValueId::new(1),
        });

        // Store: *v2 = v3, *v2 = v4
        cs.store.insert(StoreConstraint {
            dst_ptr: ValueId::new(2),
            src: ValueId::new(3),
        });
        cs.store.insert(StoreConstraint {
            dst_ptr: ValueId::new(2),
            src: ValueId::new(4),
        });

        // GEP: v6 = gep(v1, field 0)
        cs.gep.insert(GepConstraint {
            dst: ValueId::new(6),
            src_ptr: ValueId::new(1),
            path: FieldPath::field(0),
            index_operands: vec![],
        });

        cs
    }

    #[test]
    fn build_indexes_copy_by_src() {
        let cs = make_constraint_set();
        let (index, indexed) = ConstraintIndex::build(&cs);

        // v1 is src of 2 copy constraints
        let indices = index.copies_by_src(ValueId::new(1));
        assert_eq!(indices.len(), 2);
        for &i in indices {
            assert_eq!(indexed.copy[i].src, ValueId::new(1));
        }

        // v2 is src of 1 copy constraint
        let indices = index.copies_by_src(ValueId::new(2));
        assert_eq!(indices.len(), 1);
        assert_eq!(indexed.copy[indices[0]].src, ValueId::new(2));

        // v99 has no copy constraints
        assert!(index.copies_by_src(ValueId::new(99)).is_empty());
    }

    #[test]
    fn build_indexes_load_by_src_ptr() {
        let cs = make_constraint_set();
        let (index, indexed) = ConstraintIndex::build(&cs);

        let indices = index.loads_by_src_ptr(ValueId::new(1));
        assert_eq!(indices.len(), 1);
        assert_eq!(indexed.load[indices[0]].src_ptr, ValueId::new(1));

        assert!(index.loads_by_src_ptr(ValueId::new(99)).is_empty());
    }

    #[test]
    fn build_indexes_store_by_dst_ptr() {
        let cs = make_constraint_set();
        let (index, indexed) = ConstraintIndex::build(&cs);

        let indices = index.stores_by_dst_ptr(ValueId::new(2));
        assert_eq!(indices.len(), 2);
        for &i in indices {
            assert_eq!(indexed.store[i].dst_ptr, ValueId::new(2));
        }

        assert!(index.stores_by_dst_ptr(ValueId::new(99)).is_empty());
    }

    #[test]
    fn build_indexes_store_by_src() {
        let cs = make_constraint_set();
        let (index, indexed) = ConstraintIndex::build(&cs);

        let indices = index.stores_by_src(ValueId::new(3));
        assert_eq!(indices.len(), 1);
        assert_eq!(indexed.store[indices[0]].src, ValueId::new(3));

        let indices = index.stores_by_src(ValueId::new(4));
        assert_eq!(indices.len(), 1);
        assert_eq!(indexed.store[indices[0]].src, ValueId::new(4));
    }

    #[test]
    fn build_indexes_gep_by_src_ptr() {
        let cs = make_constraint_set();
        let (index, indexed) = ConstraintIndex::build(&cs);

        let indices = index.geps_by_src_ptr(ValueId::new(1));
        assert_eq!(indices.len(), 1);
        assert_eq!(indexed.gep[indices[0]].src_ptr, ValueId::new(1));

        assert!(index.geps_by_src_ptr(ValueId::new(99)).is_empty());
    }

    #[test]
    fn build_with_empty_constraint_set() {
        let cs = ConstraintSet::default();
        let (index, indexed) = ConstraintIndex::build(&cs);

        assert!(index.copy_by_src.is_empty());
        assert!(index.load_by_src_ptr.is_empty());
        assert!(index.store_by_dst_ptr.is_empty());
        assert!(index.store_by_src.is_empty());
        assert!(index.gep_by_src_ptr.is_empty());

        assert!(indexed.copy.is_empty());
        assert!(indexed.load.is_empty());
        assert!(indexed.store.is_empty());
        assert!(indexed.gep.is_empty());
    }

    #[test]
    fn indexed_constraints_preserve_count() {
        let cs = make_constraint_set();
        let (_index, indexed) = ConstraintIndex::build(&cs);

        assert_eq!(indexed.copy.len(), cs.copy.len());
        assert_eq!(indexed.load.len(), cs.load.len());
        assert_eq!(indexed.store.len(), cs.store.len());
        assert_eq!(indexed.gep.len(), cs.gep.len());
    }

    #[test]
    fn add_copy_incremental() {
        let cs = make_constraint_set();
        let (mut index, mut indexed) = ConstraintIndex::build(&cs);
        let original_len = indexed.copy.len();

        let new_copy = CopyConstraint {
            dst: ValueId::new(10),
            src: ValueId::new(20),
        };
        index.add_copy(&mut indexed, new_copy);

        assert_eq!(indexed.copy.len(), original_len + 1);
        assert_eq!(indexed.copy[original_len].src, ValueId::new(20));
        assert_eq!(indexed.copy[original_len].dst, ValueId::new(10));

        let indices = index.copies_by_src(ValueId::new(20));
        assert!(indices.contains(&original_len));
    }

    #[test]
    fn add_load_incremental() {
        let cs = make_constraint_set();
        let (mut index, mut indexed) = ConstraintIndex::build(&cs);
        let original_len = indexed.load.len();

        let new_load = LoadConstraint {
            dst: ValueId::new(11),
            src_ptr: ValueId::new(21),
        };
        index.add_load(&mut indexed, &new_load);

        assert_eq!(indexed.load.len(), original_len + 1);
        assert_eq!(indexed.load[original_len].src_ptr, ValueId::new(21));
        assert_eq!(indexed.load[original_len].dst, ValueId::new(11));

        let indices = index.loads_by_src_ptr(ValueId::new(21));
        assert!(indices.contains(&original_len));
    }

    #[test]
    fn add_store_incremental() {
        let cs = make_constraint_set();
        let (mut index, mut indexed) = ConstraintIndex::build(&cs);
        let original_len = indexed.store.len();

        let new_store = StoreConstraint {
            dst_ptr: ValueId::new(12),
            src: ValueId::new(22),
        };
        index.add_store(&mut indexed, &new_store);

        assert_eq!(indexed.store.len(), original_len + 1);
        assert_eq!(indexed.store[original_len].dst_ptr, ValueId::new(12));
        assert_eq!(indexed.store[original_len].src, ValueId::new(22));

        // Verify both reverse indexes
        let by_dst = index.stores_by_dst_ptr(ValueId::new(12));
        assert!(by_dst.contains(&original_len));

        let by_src = index.stores_by_src(ValueId::new(22));
        assert!(by_src.contains(&original_len));
    }

    #[test]
    fn add_gep_incremental() {
        let cs = make_constraint_set();
        let (mut index, mut indexed) = ConstraintIndex::build(&cs);
        let original_len = indexed.gep.len();

        let new_gep = GepConstraint {
            dst: ValueId::new(13),
            src_ptr: ValueId::new(23),
            path: FieldPath::field(1),
            index_operands: vec![],
        };
        index.add_gep(&mut indexed, &new_gep);

        assert_eq!(indexed.gep.len(), original_len + 1);
        assert_eq!(indexed.gep[original_len].src_ptr, ValueId::new(23));
        assert_eq!(indexed.gep[original_len].dst, ValueId::new(13));

        let indices = index.geps_by_src_ptr(ValueId::new(23));
        assert!(indices.contains(&original_len));
    }

    #[test]
    fn incremental_add_preserves_existing() {
        let cs = make_constraint_set();
        let (mut index, mut indexed) = ConstraintIndex::build(&cs);

        // Record original state
        let orig_copy_count = indexed.copy.len();
        let orig_copies_by_v1 = index.copies_by_src(ValueId::new(1)).len();

        // Add new constraints
        index.add_copy(
            &mut indexed,
            CopyConstraint {
                dst: ValueId::new(50),
                src: ValueId::new(51),
            },
        );
        index.add_load(
            &mut indexed,
            &LoadConstraint {
                dst: ValueId::new(52),
                src_ptr: ValueId::new(53),
            },
        );
        index.add_store(
            &mut indexed,
            &StoreConstraint {
                dst_ptr: ValueId::new(54),
                src: ValueId::new(55),
            },
        );
        index.add_gep(
            &mut indexed,
            &GepConstraint {
                dst: ValueId::new(56),
                src_ptr: ValueId::new(57),
                path: FieldPath::field(1),
                index_operands: vec![],
            },
        );

        // Original copy constraints still accessible via index
        let copies_v1 = index.copies_by_src(ValueId::new(1));
        assert_eq!(copies_v1.len(), orig_copies_by_v1);
        for &i in copies_v1 {
            assert_eq!(indexed.copy[i].src, ValueId::new(1));
        }

        // Original copy vec entries untouched (first orig_copy_count entries)
        for c in &indexed.copy[..orig_copy_count] {
            assert!(c.src == ValueId::new(1) || c.src == ValueId::new(2));
        }

        // Original load still accessible
        let loads_v1 = index.loads_by_src_ptr(ValueId::new(1));
        assert_eq!(loads_v1.len(), 1);
        assert_eq!(indexed.load[loads_v1[0]].src_ptr, ValueId::new(1));

        // Original stores still accessible
        let stores_v2 = index.stores_by_dst_ptr(ValueId::new(2));
        assert_eq!(stores_v2.len(), 2);
        for &i in stores_v2 {
            assert_eq!(indexed.store[i].dst_ptr, ValueId::new(2));
        }

        // Original GEP still accessible
        let geps_v1 = index.geps_by_src_ptr(ValueId::new(1));
        assert_eq!(geps_v1.len(), 1);
        assert_eq!(indexed.gep[geps_v1[0]].src_ptr, ValueId::new(1));
    }
}
