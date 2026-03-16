# Bitvector Lattice for Ascent PTA Solver — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `BTreeSet<LocId>` with a dense bitvector in `AscentPtsSet` to achieve ~100-330x speedup on lattice operations, enabling the Ascent solver to complete large programs like bash that currently time out.

**Architecture:** Build a `LocIdRegistry` that maps sparse `LocId` (u128) to dense `u32` indices. Wrap `fixedbitset::FixedBitSet` in `AscentPtsSet` with an `Arc<LocIdRegistry>` for index-to-LocId reverse mapping. Use a thread-local to make the registry available inside Ascent's generated rule code where `AscentPtsSet::singleton(loc)` is called. Two-phase solving rebuilds the registry between phases since GEP resolution creates new LocIds.

**Tech Stack:** `fixedbitset` crate (SIMD-optimized bitset), `Arc` for shared registry, `thread_local!` + `RefCell` for Ascent integration.

**Design doc:** `docs/plans/2026-02-23-bitvector-lattice-design.md`

---

### Task 1: Add `fixedbitset` dependency

**Files:**
- Modify: `crates/saf-datalog/Cargo.toml`

**Step 1: Add the dependency**

In `crates/saf-datalog/Cargo.toml`, add `fixedbitset` to `[dependencies]`:

```toml
[dependencies]
saf-core = { workspace = true }
saf-analysis = { workspace = true }
ascent = "0.8"
tracing = { workspace = true }
rustc-hash = { workspace = true }
fixedbitset = "0.5"
```

**Step 2: Verify it compiles**

Run: `make fmt && make lint`
Expected: PASS (no code uses it yet)

**Step 3: Commit**

```bash
git add crates/saf-datalog/Cargo.toml Cargo.lock
git commit -m "deps(saf-datalog): add fixedbitset for bitvector lattice"
```

---

### Task 2: Create `LocIdRegistry` with tests

**Files:**
- Create: `crates/saf-datalog/src/pta/registry.rs`
- Modify: `crates/saf-datalog/src/pta/mod.rs` (add module declaration)

**Step 1: Write the failing tests**

Create `crates/saf-datalog/src/pta/registry.rs` with the test module first:

```rust
//! Location ID registry for dense bitvector indexing.
//!
//! Maps sparse `LocId` (u128 BLAKE3 hashes) to dense `u32` indices
//! for use in `FixedBitSet`-backed points-to sets. The registry is
//! built once from all `LocId`s in `PtaFacts` and shared via `Arc`.

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_core::ids::LocId;

use crate::facts::PtaFacts;

/// Maps sparse `LocId` values to dense sequential `u32` indices.
///
/// Built once per analysis phase from the `LocId`s appearing in `PtaFacts`.
/// Shared across all `AscentPtsSet` instances via `Arc<LocIdRegistry>`.
#[derive(Debug, Clone)]
pub struct LocIdRegistry {
    to_index: BTreeMap<LocId, u32>,
    to_loc: Vec<LocId>,
}

impl LocIdRegistry {
    /// Build a registry from all `LocId`s appearing in the given facts.
    ///
    /// Collects unique `LocId`s from `addr_of` facts (the only source of
    /// location IDs), assigns them dense sequential indices starting at 0,
    /// and builds both forward and reverse mappings.
    #[must_use]
    pub fn from_facts(facts: &PtaFacts) -> Self {
        todo!()
    }

    /// Look up the dense index for a `LocId`.
    ///
    /// # Panics
    ///
    /// Panics if `loc` was not in the facts used to build this registry.
    #[must_use]
    pub fn index_of(&self, loc: LocId) -> u32 {
        todo!()
    }

    /// Look up the `LocId` for a dense index.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds.
    #[must_use]
    pub fn loc_at(&self, idx: u32) -> LocId {
        todo!()
    }

    /// Number of registered locations (= bitvector capacity).
    #[must_use]
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ValueId;

    fn make_loc(name: &[u8]) -> LocId {
        LocId::derive(name)
    }

    fn make_val(name: &[u8]) -> ValueId {
        ValueId::derive(name)
    }

    #[test]
    fn empty_facts_produce_empty_registry() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());
    }

    #[test]
    fn round_trip_index_to_loc() {
        let loc_a = make_loc(b"a");
        let loc_b = make_loc(b"b");
        let val = make_val(b"ptr");

        let facts = PtaFacts {
            addr_of: vec![(val, loc_a), (val, loc_b)],
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 2);

        // Round-trip: loc -> index -> loc
        let idx_a = reg.index_of(loc_a);
        let idx_b = reg.index_of(loc_b);
        assert_ne!(idx_a, idx_b);
        assert_eq!(reg.loc_at(idx_a), loc_a);
        assert_eq!(reg.loc_at(idx_b), loc_b);
    }

    #[test]
    fn duplicate_locs_are_deduplicated() {
        let loc = make_loc(b"same");
        let v1 = make_val(b"p1");
        let v2 = make_val(b"p2");

        let facts = PtaFacts {
            addr_of: vec![(v1, loc), (v2, loc)],
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn indices_are_sequential_from_zero() {
        let locs: Vec<LocId> = (0..5).map(|i| make_loc(format!("loc{i}").as_bytes())).collect();
        let val = make_val(b"ptr");

        let facts = PtaFacts {
            addr_of: locs.iter().map(|l| (val, *l)).collect(),
            ..Default::default()
        };
        let reg = LocIdRegistry::from_facts(&facts);

        assert_eq!(reg.len(), 5);

        let mut indices: Vec<u32> = locs.iter().map(|l| reg.index_of(*l)).collect();
        indices.sort_unstable();
        assert_eq!(indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    #[should_panic]
    fn index_of_unknown_loc_panics() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        reg.index_of(make_loc(b"unknown"));
    }

    #[test]
    #[should_panic]
    fn loc_at_out_of_bounds_panics() {
        let facts = PtaFacts::default();
        let reg = LocIdRegistry::from_facts(&facts);
        reg.loc_at(0);
    }
}
```

**Step 2: Add module to `mod.rs`**

In `crates/saf-datalog/src/pta/mod.rs`, add:

```rust
pub mod registry;
```

And add to re-exports:

```rust
pub use registry::LocIdRegistry;
```

**Step 3: Run tests to verify they fail**

Run: `make test` (or just the saf-datalog tests)
Expected: FAIL — all tests fail with `todo!()`

**Step 4: Implement `LocIdRegistry`**

Replace the `todo!()` bodies:

```rust
impl LocIdRegistry {
    #[must_use]
    pub fn from_facts(facts: &PtaFacts) -> Self {
        let mut to_loc: Vec<LocId> = facts
            .addr_of
            .iter()
            .map(|(_, loc)| *loc)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        // BTreeSet already sorts, so to_loc is in deterministic order.

        let to_index: BTreeMap<LocId, u32> = to_loc
            .iter()
            .enumerate()
            .map(|(i, loc)| {
                // INVARIANT: PTA programs have < 2^32 abstract locations.
                #[allow(clippy::cast_possible_truncation)]
                let idx = i as u32;
                (*loc, idx)
            })
            .collect();

        Self { to_index, to_loc }
    }

    #[must_use]
    pub fn index_of(&self, loc: LocId) -> u32 {
        self.to_index[&loc]
    }

    #[must_use]
    pub fn loc_at(&self, idx: u32) -> LocId {
        self.to_loc[idx as usize]
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.to_loc.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.to_loc.is_empty()
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `make test`
Expected: All registry tests PASS

**Step 6: Commit**

```bash
git add crates/saf-datalog/src/pta/registry.rs crates/saf-datalog/src/pta/mod.rs
git commit -m "feat(saf-datalog): add LocIdRegistry for sparse-to-dense LocId mapping"
```

---

### Task 3: Add thread-local registry setup

**Files:**
- Modify: `crates/saf-datalog/src/pta/registry.rs`

**Step 1: Write the failing test for thread-local access**

Add to the test module in `registry.rs`:

```rust
    #[test]
    fn with_registry_provides_access() {
        let loc = make_loc(b"loc");
        let val = make_val(b"ptr");
        let facts = PtaFacts {
            addr_of: vec![(val, loc)],
            ..Default::default()
        };
        let reg = Arc::new(LocIdRegistry::from_facts(&facts));

        with_registry(reg.clone(), || {
            let current = current_registry();
            assert_eq!(current.len(), 1);
            assert_eq!(current.index_of(loc), 0);
        });
    }

    #[test]
    #[should_panic(expected = "PTS_REGISTRY not set")]
    fn current_registry_panics_without_setup() {
        let _ = current_registry();
    }

    #[test]
    fn with_registry_cleans_up_after_closure() {
        let facts = PtaFacts {
            addr_of: vec![(make_val(b"p"), make_loc(b"l"))],
            ..Default::default()
        };
        let reg = Arc::new(LocIdRegistry::from_facts(&facts));

        with_registry(reg, || {
            // Registry available here
            assert_eq!(current_registry().len(), 1);
        });

        // After with_registry, thread-local should be cleared
        // (panics if we try to access)
        let result = std::panic::catch_unwind(|| current_registry());
        assert!(result.is_err());
    }
```

**Step 2: Run tests to verify they fail**

Run: `make test`
Expected: FAIL — `with_registry` and `current_registry` not defined

**Step 3: Implement thread-local setup**

Add to `registry.rs`, above the `impl LocIdRegistry` block:

```rust
use std::cell::RefCell;

thread_local! {
    static PTS_REGISTRY: RefCell<Option<Arc<LocIdRegistry>>> = const { RefCell::new(None) };
}

/// Run a closure with the given `LocIdRegistry` available via [`current_registry`].
///
/// Sets the thread-local registry before the closure and clears it after.
/// This is the entry point for making the registry available inside Ascent's
/// generated rule code, where `AscentPtsSet::singleton(loc)` needs to look
/// up the dense index for a `LocId`.
///
/// # Panics
///
/// Panics if the thread-local is already set (nested calls are not supported).
pub fn with_registry<R>(registry: Arc<LocIdRegistry>, f: impl FnOnce() -> R) -> R {
    PTS_REGISTRY.with(|r| {
        let prev = r.borrow().clone();
        assert!(prev.is_none(), "nested with_registry calls are not supported");
        *r.borrow_mut() = Some(registry);
    });
    let result = f();
    PTS_REGISTRY.with(|r| {
        *r.borrow_mut() = None;
    });
    result
}

/// Get the current thread-local `LocIdRegistry`.
///
/// Returns a clone of the `Arc` — cheap (atomic refcount bump).
///
/// # Panics
///
/// Panics if called outside a [`with_registry`] scope.
#[must_use]
pub fn current_registry() -> Arc<LocIdRegistry> {
    PTS_REGISTRY.with(|r| {
        r.borrow()
            .as_ref()
            .expect("PTS_REGISTRY not set — call with_registry() first")
            .clone()
    })
}
```

**Step 4: Run tests to verify they pass**

Run: `make test`
Expected: All registry + thread-local tests PASS

**Step 5: Commit**

```bash
git add crates/saf-datalog/src/pta/registry.rs
git commit -m "feat(saf-datalog): add thread-local LocIdRegistry for Ascent integration"
```

---

### Task 4: Rewrite `AscentPtsSet` with bitvector backend

**Files:**
- Modify: `crates/saf-datalog/src/pta/pts_lattice.rs` (full rewrite of internals)

This is the core task. The public API stays the same, but the internals change from `BTreeSet<LocId>` to `FixedBitSet` + `Arc<LocIdRegistry>`.

**Step 1: Write new tests for bitvector behavior**

Add these tests to the existing test module in `pts_lattice.rs`. They test bitvector-specific behavior alongside the existing tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::PtaFacts;
    use crate::pta::registry::{LocIdRegistry, with_registry};
    use saf_core::ids::ValueId;
    use std::sync::Arc;

    fn setup_registry(locs: &[LocId]) -> Arc<LocIdRegistry> {
        let val = ValueId::derive(b"ptr");
        let facts = PtaFacts {
            addr_of: locs.iter().map(|l| (val, *l)).collect(),
            ..Default::default()
        };
        Arc::new(LocIdRegistry::from_facts(&facts))
    }

    fn loc(name: &[u8]) -> LocId {
        LocId::derive(name)
    }

    #[test]
    fn empty_set_has_zero_length() {
        let locs = [loc(b"a"), loc(b"b")];
        let reg = setup_registry(&locs);
        with_registry(reg, || {
            let s = AscentPtsSet::empty();
            assert!(s.is_empty());
            assert_eq!(s.len(), 0);
        });
    }

    #[test]
    fn singleton_contains_one_element() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s = AscentPtsSet::singleton(la);
            assert_eq!(s.len(), 1);
            assert!(s.contains(la));
            assert!(!s.contains(lb));
        });
    }

    #[test]
    fn join_mut_computes_union() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::singleton(lb);

            let changed = s1.join_mut(s2);
            assert!(changed);
            assert_eq!(s1.len(), 2);
            assert!(s1.contains(la));
            assert!(s1.contains(lb));
        });
    }

    #[test]
    fn join_mut_returns_false_when_no_change() {
        let la = loc(b"a");
        let reg = setup_registry(&[la]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::singleton(la);

            let changed = s1.join_mut(s2);
            assert!(!changed);
            assert_eq!(s1.len(), 1);
        });
    }

    #[test]
    fn meet_mut_computes_intersection() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::collect_from(vec![la, lb]);
            let s2 = AscentPtsSet::collect_from(vec![lb, lc]);

            let changed = s1.meet_mut(s2);
            assert!(changed);
            assert_eq!(s1.len(), 1);
            assert!(s1.contains(lb));
        });
    }

    #[test]
    fn meet_mut_returns_false_when_no_change() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let mut s1 = AscentPtsSet::singleton(la);
            let s2 = AscentPtsSet::collect_from(vec![la, lb]);

            // s1 ∩ s2 = {a} = s1, so no change
            let changed = s1.meet_mut(s2);
            assert!(!changed);
        });
    }

    #[test]
    fn partial_ord_subset_ordering() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let s_a = AscentPtsSet::singleton(la);
            let s_ab = AscentPtsSet::collect_from(vec![la, lb]);
            let s_c = AscentPtsSet::singleton(lc);

            // {a} < {a, b}
            assert!(s_a < s_ab);
            // {a, b} > {a}
            assert!(s_ab > s_a);
            // {a} and {c} are incomparable
            assert_eq!(s_a.partial_cmp(&s_c), None);
            // Equal sets
            assert_eq!(s_a.partial_cmp(&s_a.clone()), Some(std::cmp::Ordering::Equal));
        });
    }

    #[test]
    fn iter_yields_correct_loc_ids() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let lc = loc(b"c");
        let reg = setup_registry(&[la, lb, lc]);
        with_registry(reg, || {
            let s = AscentPtsSet::collect_from(vec![la, lc]);
            let mut result: Vec<LocId> = s.iter().collect();
            result.sort();

            let mut expected = vec![la, lc];
            expected.sort();

            assert_eq!(result, expected);
        });
    }

    #[test]
    fn into_btreeset_round_trips() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s = AscentPtsSet::collect_from(vec![la, lb]);
            let btree = s.into_btreeset();
            assert_eq!(btree.len(), 2);
            assert!(btree.contains(&la));
            assert!(btree.contains(&lb));
        });
    }

    #[test]
    fn hash_consistent_for_equal_sets() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s1 = AscentPtsSet::collect_from(vec![la, lb]);
            let s2 = AscentPtsSet::collect_from(vec![lb, la]); // same set, different order

            let hash = |s: &AscentPtsSet| {
                let mut h = DefaultHasher::new();
                s.hash(&mut h);
                h.finish()
            };

            assert_eq!(hash(&s1), hash(&s2));
        });
    }

    #[test]
    fn default_is_empty() {
        let la = loc(b"a");
        let reg = setup_registry(&[la]);
        with_registry(reg, || {
            let s = AscentPtsSet::default();
            assert!(s.is_empty());
            assert_eq!(s.len(), 0);
        });
    }

    #[test]
    fn clone_produces_independent_copy() {
        let la = loc(b"a");
        let lb = loc(b"b");
        let reg = setup_registry(&[la, lb]);
        with_registry(reg, || {
            let s1 = AscentPtsSet::singleton(la);
            let mut s2 = s1.clone();
            s2.join_mut(AscentPtsSet::singleton(lb));

            assert_eq!(s1.len(), 1);
            assert_eq!(s2.len(), 2);
        });
    }
}
```

**Step 2: Rewrite `AscentPtsSet` internals**

Replace the entire `pts_lattice.rs` content (keeping the module doc comment updated):

```rust
//! Points-to set lattice for Ascent-based PTA.
//!
//! Wraps `FixedBitSet` with set-union as join (least upper bound)
//! and set-intersection as meet (greatest lower bound), implementing
//! the `ascent::lattice::Lattice` trait for use in Ascent relations.
//!
//! Uses a dense bitvector indexed by a shared [`LocIdRegistry`] that maps
//! sparse `LocId` (u128) to dense `u32` indices. The registry is accessed
//! via a thread-local set by [`with_registry`] before Ascent's `prog.run()`.

use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ascent::lattice::Lattice;
use fixedbitset::FixedBitSet;
use saf_core::ids::LocId;

use crate::pta::registry::{LocIdRegistry, current_registry};

/// Points-to set for Ascent lattice-based PTA.
///
/// Uses `FixedBitSet` (dense bitvector) with set-union as join (least upper
/// bound) and set-intersection as meet (greatest lower bound). The empty set
/// is the lattice bottom element.
///
/// Each set carries an `Arc<LocIdRegistry>` for mapping between dense bit
/// indices and sparse `LocId` values. The registry is obtained from a
/// thread-local on construction and shared across all sets in a solve phase.
#[derive(Clone, Debug)]
pub struct AscentPtsSet {
    bits: FixedBitSet,
    registry: Arc<LocIdRegistry>,
}

impl AscentPtsSet {
    /// Create an empty points-to set (lattice bottom).
    #[must_use]
    pub fn empty() -> Self {
        let registry = current_registry();
        let bits = FixedBitSet::with_capacity(registry.len());
        Self { bits, registry }
    }

    /// Create a points-to set containing a single location.
    #[must_use]
    pub fn singleton(loc: LocId) -> Self {
        let registry = current_registry();
        let mut bits = FixedBitSet::with_capacity(registry.len());
        bits.insert(registry.index_of(loc) as usize);
        Self { bits, registry }
    }

    /// Create a points-to set from an iterator of locations.
    pub fn collect_from(locs: impl IntoIterator<Item = LocId>) -> Self {
        let registry = current_registry();
        let mut bits = FixedBitSet::with_capacity(registry.len());
        for loc in locs {
            bits.insert(registry.index_of(loc) as usize);
        }
        Self { bits, registry }
    }

    /// Check whether the set contains a given location.
    #[must_use]
    pub fn contains(&self, loc: LocId) -> bool {
        // Use get() to avoid panic if loc is not in registry
        // (can happen if querying for a loc from a different phase)
        if let Some(&idx) = self.registry.try_index_of(loc) {
            self.bits.contains(idx as usize)
        } else {
            false
        }
    }

    /// Return the number of locations in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bits.count_ones(..)
    }

    /// Check whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bits.count_ones(..) == 0
    }

    /// Iterate over the locations in the set.
    pub fn iter(&self) -> impl Iterator<Item = LocId> + '_ {
        self.bits.ones().map(|idx| {
            // INVARIANT: indices in bits are always valid registry indices.
            #[allow(clippy::cast_possible_truncation)]
            let idx = idx as u32;
            self.registry.loc_at(idx)
        })
    }

    /// Convert to the standard SAF `BTreeSet<LocId>` for output compatibility.
    #[must_use]
    pub fn into_btreeset(self) -> BTreeSet<LocId> {
        self.iter().collect()
    }
}

impl Default for AscentPtsSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for AscentPtsSet {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl Eq for AscentPtsSet {}

impl Hash for AscentPtsSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bits.hash(state);
    }
}

impl PartialOrd for AscentPtsSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.bits == other.bits {
            Some(std::cmp::Ordering::Equal)
        } else if self.bits.is_subset(&other.bits) {
            Some(std::cmp::Ordering::Less)
        } else if other.bits.is_subset(&self.bits) {
            Some(std::cmp::Ordering::Greater)
        } else {
            None // incomparable sets
        }
    }
}

impl Lattice for AscentPtsSet {
    fn meet_mut(&mut self, other: Self) -> bool {
        let before = self.bits.count_ones(..);
        self.bits.intersect_with(&other.bits);
        self.bits.count_ones(..) != before
    }

    fn join_mut(&mut self, other: Self) -> bool {
        let before = self.bits.count_ones(..);
        self.bits.union_with(&other.bits);
        self.bits.count_ones(..) != before
    }
}
```

**Step 3: Add `try_index_of` to `LocIdRegistry`**

In `registry.rs`, add this method to `impl LocIdRegistry`:

```rust
    /// Try to look up the dense index for a `LocId`.
    ///
    /// Returns `None` if the `LocId` is not in the registry.
    #[must_use]
    pub fn try_index_of(&self, loc: LocId) -> Option<&u32> {
        self.to_index.get(&loc)
    }
```

**Step 4: Run tests to verify they pass**

Run: `make fmt && make test`
Expected: All `pts_lattice` tests PASS, all existing `saf-datalog` tests PASS

Note: The existing tests in `context.rs` (like `ascent_analyze_single_alloca`) will also exercise the new code path, since they call `analyze_with_ascent` which calls `ascent_solve` which uses `AscentPtsSet`. These tests will FAIL until Task 5 sets up the registry in `context.rs`.

**Step 5: Commit**

```bash
git add crates/saf-datalog/src/pta/pts_lattice.rs crates/saf-datalog/src/pta/registry.rs
git commit -m "feat(saf-datalog): replace BTreeSet with FixedBitSet in AscentPtsSet

Bitvector union is O(n/64) vs BTreeSet's O(n log n), providing
~100-330x speedup on lattice operations for large programs."
```

---

### Task 5: Update `context.rs` to build and install registry

**Files:**
- Modify: `crates/saf-datalog/src/pta/context.rs`

This wires up the registry into the two-phase solve pipeline. The registry must be rebuilt between phases because GEP resolution creates new `LocId`s.

**Step 1: Update imports**

At the top of `context.rs`, add:

```rust
use std::sync::Arc;
use crate::pta::registry::{LocIdRegistry, with_registry};
```

**Step 2: Wrap the two-phase solve in `with_registry` calls**

Replace the solve section of `analyze_with_ascent` (from Step 8 onward). The key change: build a `LocIdRegistry` from facts before each `ascent_solve` call, and wrap each call in `with_registry`.

Replace the code from "Step 8: Two-phase demand-driven GEP resolution" through "Step 9: Final Ascent fixpoint solve":

```rust
    // Step 8: Two-phase demand-driven GEP resolution
    //
    // Phase 1: Solve without GEPs to get preliminary points-to.
    // This captures locations reachable via store/load chains that
    // single-pass addr_of-only resolution would miss.
    let gep_facts = std::mem::take(&mut dl_facts.gep);

    let preliminary_pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };

    // Phase 2: Resolve GEP facts using the full preliminary points-to map.
    // Field locations are created demand-driven here — only for objects that
    // a pointer actually points to — avoiding the O(objects × GEPs) blowup
    // of eager precomputation.
    dl_facts.gep = gep_facts;
    crate::pta::gep::resolve_gep_facts_with_pts(&mut dl_facts, &mut factory, &preliminary_pts);

    // Step 9: Final Ascent fixpoint solve with resolved GEP facts.
    // Rebuild registry since GEP resolution added new LocIds to addr_of.
    let pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };
```

**Step 3: Run tests to verify everything passes**

Run: `make fmt && make test`
Expected: ALL 54+ saf-datalog tests PASS, including:
- `ascent_analyze_empty_module`
- `ascent_analyze_disabled_returns_empty`
- `ascent_analyze_single_alloca`
- `ascent_analyze_gep_creates_field_locations`
- All registry tests
- All pts_lattice tests
- All GEP tests
- All solver tests

**Step 4: Commit**

```bash
git add crates/saf-datalog/src/pta/context.rs
git commit -m "feat(saf-datalog): wire LocIdRegistry into two-phase Ascent pipeline

Builds registry from facts before each ascent_solve() call and
installs it via thread-local with_registry(). Rebuilds between
phases since GEP resolution creates new LocIds."
```

---

### Task 6: Update `mod.rs` exports and run lint

**Files:**
- Modify: `crates/saf-datalog/src/pta/mod.rs`

**Step 1: Verify exports are complete**

Ensure `mod.rs` exports the new public items:

```rust
pub mod registry;
// ... existing modules ...

pub use registry::{LocIdRegistry, with_registry, current_registry};
```

**Step 2: Run full lint + test suite**

Run: `make fmt && make lint && make test`
Expected: ALL pass with zero warnings

**Step 3: Commit**

```bash
git add crates/saf-datalog/src/pta/mod.rs
git commit -m "feat(saf-datalog): export LocIdRegistry and registry helpers"
```

---

### Task 7: Benchmark comparison on CruxBC bash

**Files:** None (read-only verification)

**Step 1: Run Ascent solver on bash with bitvector**

Run inside Docker (background, 120s timeout):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-cruxbc --filter "bash" --solver ascent -o /workspace/tests/benchmarks/cruxbc/ascent-bitvec.json'
```

**Step 2: Compare with legacy results**

Read `tests/benchmarks/cruxbc/legacy-results.json` and `tests/benchmarks/cruxbc/ascent-bitvec.json`.
Compare:
- Solver time (legacy ~16s, target: <30s for Ascent)
- Points-to set sizes (should be similar)
- Memory usage if available

**Step 3: Document results**

Update `plans/PROGRESS.md` with benchmark comparison results.

---

## Edge Cases and Risks

1. **Empty registry**: `Default::default()` for `AscentPtsSet` calls `current_registry()`. If called outside `with_registry` scope, it panics. Mitigated by: only `analyze_with_ascent` creates sets, and it always wraps in `with_registry`.

2. **Phase mismatch**: If a set from phase 1 is accidentally used in phase 2 (different registry), `contains()` returns false for unknown locs (safe). `join_mut` on mismatched registries would produce incorrect results. Mitigated by: phase 1 results are extracted to `PointsToMap` (BTreeSet) before phase 2 starts.

3. **FixedBitSet size mismatch**: If two sets have different capacities (from different registries), `union_with` may panic or produce incorrect results. Mitigated by: same concern as #2, each phase uses its own registry.

4. **`ascent_par!` thread-local propagation**: The parallel solver uses Rayon threads. Thread-locals set on the main thread are NOT visible in Rayon worker threads. For now, this is fine because `parallel` feature is not enabled by default. When enabling parallel: use `rayon::ThreadPoolBuilder::build_scoped()` or set the thread-local in each worker thread.
