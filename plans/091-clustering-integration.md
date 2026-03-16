# Plan 091: Integrate Object Clustering into PTA Solver

**Epic:** E25 — Scalability: Efficient Points-To Sets
**Status:** approved
**Created:** 2026-02-09

## Context

The clustering module (`pta/clustering.rs`) is fully implemented but not wired into the solver. Additionally, the solver creates isolated indexers per PtsSet instance, causing BitVec/BDD union operations to use slow O(n) fallbacks instead of O(n/64) bitwise-OR. This plan fixes both.

## Performance Answer

- **Shared indexer (Phase 1)**: Pure speedup. Fixes BitVec/BDD union from O(n) to O(n/64).
- **Clustering (Phase 2)**: Adds preprocessing cost. Auto-disabled for BTreePtsSet. Net faster for medium-large BitVec/BDD programs.

---

## Agent Task Definitions

### Task 1: PtsSet Trait — add `clone_empty`, `BENEFITS_FROM_CLUSTERING`, `with_seeded_ordering`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/ptsset/trait_def.rs`. The file defines a `PtsSet` trait (line 29) with methods like `empty()`, `singleton()`, `insert()`, etc. After the last method `from_btreeset` (line 92), add three new items inside the trait:

```rust
    /// Create a new empty set sharing internal state (indexer/context) with this set.
    ///
    /// For indexed representations (`BitVecPtsSet`, `BddPtsSet`), shares the same
    /// indexer for fast bitwise operations. For `BTreePtsSet`, equivalent to `empty()`.
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
```

No other files. No tests needed (defaults work for all existing impls).

---

### Task 2: BitVecPtsSet — override `clone_empty`, `BENEFITS_FROM_CLUSTERING`, `with_seeded_ordering`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/ptsset/bitvec.rs`. This file defines `BitVecPtsSet` with a shared `indexer: Arc<RwLock<LocIdIndexer>>`. It has `impl PtsSet for BitVecPtsSet` starting at line 145.

Inside that impl block, add these overrides (anywhere in the block, e.g. after `from_btreeset` at line 363):

```rust
    const BENEFITS_FROM_CLUSTERING: bool = true;

    fn clone_empty(&self) -> Self {
        Self::with_indexer(Arc::clone(&self.indexer))
    }

    fn with_seeded_ordering(ordered_locs: &[LocId]) -> Self {
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(ordered_locs.iter().copied());
        Self::with_indexer(Arc::new(RwLock::new(indexer)))
    }
```

These use existing methods: `with_indexer` (line 56) and `LocIdIndexer::register_batch` (indexer.rs line 138).

Add tests at end of the `#[cfg(test)] mod tests` block (line 382):

```rust
    #[test]
    fn clone_empty_shares_indexer() {
        let mut pts = BitVecPtsSet::empty();
        pts.insert(LocId::new(1));
        let empty = pts.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(pts.indexer(), empty.indexer()));
    }

    #[test]
    fn clone_empty_enables_fast_union() {
        let mut a = BitVecPtsSet::empty();
        a.insert(LocId::new(1));
        let mut b = a.clone_empty();
        b.insert(LocId::new(2));
        assert!(Arc::ptr_eq(a.indexer(), b.indexer()));
        a.union(&b);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn with_seeded_ordering_registers_in_order() {
        let locs = vec![LocId::new(300), LocId::new(100), LocId::new(200)];
        let pts = BitVecPtsSet::with_seeded_ordering(&locs);
        let indexer = pts.indexer().read().expect("lock");
        assert_eq!(indexer.get(LocId::new(300)), Some(0));
        assert_eq!(indexer.get(LocId::new(100)), Some(1));
        assert_eq!(indexer.get(LocId::new(200)), Some(2));
    }
```

---

### Task 3: BddPtsSet — override `clone_empty`, `BENEFITS_FROM_CLUSTERING`, `with_seeded_ordering`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/ptsset/bdd.rs`. This file defines `BddPtsSet` with `context: Arc<RwLock<BddContext>>` and `indexer: Arc<RwLock<LocIdIndexer>>`. It has `impl PtsSet for BddPtsSet`. It has `with_context_and_indexer` (line 234) and `BddContext::bits_needed` (line 72, currently private).

First, make `bits_needed` accessible by changing line 72 from `fn bits_needed` to `pub(crate) fn bits_needed`.

Then add these overrides inside `impl PtsSet for BddPtsSet`:

```rust
    const BENEFITS_FROM_CLUSTERING: bool = true;

    fn clone_empty(&self) -> Self {
        Self::with_context_and_indexer(Arc::clone(&self.context), Arc::clone(&self.indexer))
    }

    fn with_seeded_ordering(ordered_locs: &[LocId]) -> Self {
        let num_vars = if ordered_locs.is_empty() {
            16
        } else {
            BddContext::bits_needed(ordered_locs.len()).max(1)
        };
        let context = Arc::new(RwLock::new(BddContext::new(num_vars)));
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(ordered_locs.iter().copied());
        Self::with_context_and_indexer(context, Arc::new(RwLock::new(indexer)))
    }
```

Add test in the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn clone_empty_shares_context_and_indexer() {
        let mut pts = BddPtsSet::empty();
        pts.insert(LocId::new(1));
        let empty = pts.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(pts.context(), empty.context()));
        assert!(Arc::ptr_eq(pts.indexer(), empty.indexer()));
    }
```

---

### Task 4: Add `approximate_cooccurrence()` to clustering module

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/clustering.rs`. This module already has `CooccurrenceMatrix`, `cluster_objects`, etc. It currently has `use saf_core::ids::LocId;` (line 24) and `use std::collections::{BTreeMap, BTreeSet};` (line 22).

Add these imports at the top (after existing imports):
```rust
use saf_core::ids::ValueId;
use super::constraint::{AddrConstraint, ConstraintSet, CopyConstraint};
```

Add the function before the `// Tests` section (before line 509):

```rust
/// Compute approximate co-occurrence by propagating `Addr` `LocId`s through Copy constraints.
///
/// Runs a bounded worklist pass (Copy-only, no Load/Store/GEP) to build approximate
/// points-to sets, then records co-occurrence for all multi-element sets.
/// Cost: O(V + E_copy * iterations), much cheaper than the full solver.
pub fn approximate_cooccurrence(constraints: &ConstraintSet) -> CooccurrenceMatrix {
    let mut pts: BTreeMap<ValueId, BTreeSet<LocId>> = BTreeMap::new();

    // Seed from Addr constraints
    for addr in &constraints.addr {
        pts.entry(addr.ptr).or_default().insert(addr.loc);
    }

    // Build copy adjacency: src -> [dst, ...]
    let mut copy_edges: BTreeMap<ValueId, Vec<ValueId>> = BTreeMap::new();
    for copy in &constraints.copy {
        copy_edges.entry(copy.src).or_default().push(copy.dst);
    }

    // Worklist propagation (bounded to prevent divergence on cycles)
    let mut worklist: BTreeSet<ValueId> = pts.keys().copied().collect();
    let budget = 100 * pts.len().max(1);
    let mut steps = 0;
    while let Some(v) = worklist.pop_first() {
        steps += 1;
        if steps > budget {
            break;
        }

        let v_pts = match pts.get(&v) {
            Some(s) => s.clone(),
            None => continue,
        };

        if let Some(dsts) = copy_edges.get(&v) {
            for &dst in dsts {
                let dst_pts = pts.entry(dst).or_default();
                let old_len = dst_pts.len();
                dst_pts.extend(&v_pts);
                if dst_pts.len() > old_len {
                    worklist.insert(dst);
                }
            }
        }
    }

    // Record co-occurrence from multi-element sets
    let mut matrix = CooccurrenceMatrix::new();
    for set in pts.values() {
        if set.len() >= 2 {
            matrix.record_points_to_set(set);
        }
    }
    matrix
}
```

Add tests inside the existing `#[cfg(test)] mod tests` block (after line ~706):

```rust
    #[test]
    fn approximate_cooccurrence_basic() {
        use super::super::constraint::{AddrConstraint, CopyConstraint};

        let mut constraints = ConstraintSet::default();
        let v1 = saf_core::ids::ValueId::new(1);
        let v2 = saf_core::ids::ValueId::new(2);
        let l1 = loc(100);
        let l2 = loc(200);

        constraints.addr.insert(AddrConstraint { ptr: v1, loc: l1 });
        constraints.addr.insert(AddrConstraint { ptr: v2, loc: l2 });
        constraints.copy.insert(CopyConstraint { dst: v2, src: v1 });

        let matrix = approximate_cooccurrence(&constraints);
        // v2 gets {l1, l2} after copy propagation, so l1 and l2 co-occur
        assert!(matrix.get_count(l1, l2) > 0);
    }

    #[test]
    fn approximate_cooccurrence_cycle_terminates() {
        use super::super::constraint::{AddrConstraint, CopyConstraint};

        let mut constraints = ConstraintSet::default();
        let v1 = saf_core::ids::ValueId::new(1);
        let v2 = saf_core::ids::ValueId::new(2);
        constraints.addr.insert(AddrConstraint { ptr: v1, loc: loc(100) });
        constraints.copy.insert(CopyConstraint { dst: v2, src: v1 });
        constraints.copy.insert(CopyConstraint { dst: v1, src: v2 });
        let _matrix = approximate_cooccurrence(&constraints); // must not hang
    }

    #[test]
    fn approximate_cooccurrence_empty() {
        let constraints = ConstraintSet::default();
        let matrix = approximate_cooccurrence(&constraints);
        assert_eq!(matrix.num_pairs(), 0);
    }
```

Also add `pub use clustering::approximate_cooccurrence;` to `crates/saf-analysis/src/pta/mod.rs` in the clustering re-exports (line 42-44).

---

### Task 5: Add `ClusteringMode` to PtsConfig and PtaConfig

**Self-contained instructions for agent:**

**File 1**: `crates/saf-analysis/src/pta/ptsset/config.rs`

After the `PtsRepresentation` enum (line 51), add:

```rust
/// Controls whether object clustering is applied as a preprocessing step.
///
/// Clustering groups frequently co-occurring locations into consecutive
/// bit positions, improving cache locality for bit-vector/BDD operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusteringMode {
    /// Enable for `BitVector`/`Bdd`, skip for `BTreeSet`.
    #[default]
    Auto,
    /// Always run clustering.
    Enabled,
    /// Never run clustering.
    Disabled,
}
```

Add `pub clustering: ClusteringMode,` to `PtsConfig` struct (after `bdd_threshold` field, line 98). Update `PtsConfig::default()` to include `clustering: ClusteringMode::Auto`. Update the convenience builders (`btreeset()`, `bitvector()`, `bdd()`) to include `clustering: ClusteringMode::Auto`.

**File 2**: `crates/saf-analysis/src/pta/ptsset/mod.rs` — Add `ClusteringMode` to the re-export on the `config` line (line 68): `pub use config::{ClusteringMode, PtsConfig, PtsRepresentation, count_allocation_sites};`

**File 3**: `crates/saf-analysis/src/pta/config.rs` — The `PtaConfig` struct contains `pub pts_config: PtsConfig`. No new field needed on PtaConfig itself since `pts_config.clustering` is sufficient. Just ensure the existing code compiles with the new field.

**File 4**: `crates/saf-analysis/src/cspta/solver.rs` — `CsPtaConfig` (line 30) has `pub pts_config: PtsConfig`. Same as above — no new field needed.

---

### Task 6: CI-PTA solver — add template, replace `P::empty()`, wire clustering

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/solver.rs`. This is the main Andersen CI solver.

**Step 1**: Add import at top (after line 21):
```rust
use super::ptsset::ClusteringMode;
```

**Step 2**: Add `template: P` field to `GenericSolver` struct (line 225-242). Add it after `index_sensitivity`:
```rust
    /// Template for creating empty sets that share indexer state.
    template: P,
```

**Step 3**: Update `GenericSolver::new()` (line 245) to initialize template:
```rust
    fn new(constraints: &'a ConstraintSet, factory: &'a LocationFactory) -> Self {
        Self {
            pts: BTreeMap::new(),
            loc_pts: BTreeMap::new(),
            worklist: BTreeSet::new(),
            loc_worklist: BTreeSet::new(),
            constraints,
            factory,
            constants: None,
            index_sensitivity: IndexSensitivity::default(),
            template: P::empty(),
        }
    }
```

**Step 4**: Add `new_with_template()`:
```rust
    fn new_with_template(
        constraints: &'a ConstraintSet,
        factory: &'a LocationFactory,
        template: P,
    ) -> Self {
        Self {
            pts: BTreeMap::new(),
            loc_pts: BTreeMap::new(),
            worklist: BTreeSet::new(),
            loc_worklist: BTreeSet::new(),
            constraints,
            factory,
            constants: None,
            index_sensitivity: IndexSensitivity::default(),
            template,
        }
    }
```

**Step 5**: Refactor `union_into_value` (line 468) and `union_into_location` (line 474) to avoid borrow conflict with entry API:

```rust
    fn union_into_value(&mut self, v: ValueId, locs: &P) -> bool {
        if !self.pts.contains_key(&v) {
            self.pts.insert(v, self.template.clone_empty());
        }
        self.pts.get_mut(&v).expect("just inserted").union(locs)
    }

    fn union_into_location(&mut self, loc: LocId, locs: &P) -> bool {
        if !self.loc_pts.contains_key(&loc) {
            self.loc_pts.insert(loc, self.template.clone_empty());
        }
        self.loc_pts.get_mut(&loc).expect("just inserted").union(locs)
    }
```

**Step 6**: In `solve()` method, replace the Addr initialization (line 270-279):

Replace:
```rust
            if self
                .pts
                .entry(addr.ptr)
                .or_insert_with(P::empty)
                .insert(addr.loc)
```
With:
```rust
            if !self.pts.contains_key(&addr.ptr) {
                self.pts.insert(addr.ptr, self.template.clone_empty());
            }
            if self.pts.get_mut(&addr.ptr).expect("just inserted").insert(addr.loc)
```

**Step 7**: In `handle_load_constraints` (line 338), replace `.or_insert_with(P::empty)`:
Replace:
```rust
                    self.pts.entry(load.dst).or_insert_with(P::empty);
```
With:
```rust
                    if !self.pts.contains_key(&load.dst) {
                        self.pts.insert(load.dst, self.template.clone_empty());
                    }
```

**Step 8**: In `handle_gep_constraints` (line 388), replace `P::singleton(field_loc)`:
Replace:
```rust
                        if let Some(field_loc) = field_loc {
                            let new_pts = P::singleton(field_loc);
                            if self.union_into_value(gep.dst, &new_pts) {
```
With:
```rust
                        if let Some(field_loc) = field_loc {
                            let mut new_pts = self.template.clone_empty();
                            new_pts.insert(field_loc);
                            if self.union_into_value(gep.dst, &new_pts) {
```

**Step 9**: Add `create_template` helper function (before `GenericSolver`):
```rust
/// Create a solver template, optionally pre-seeded with cluster ordering.
fn create_template<P: PtsSet>(constraints: &ConstraintSet, mode: ClusteringMode) -> P {
    let should_cluster = match mode {
        ClusteringMode::Disabled => false,
        ClusteringMode::Enabled => true,
        ClusteringMode::Auto => P::BENEFITS_FROM_CLUSTERING,
    };
    if should_cluster && !constraints.addr.is_empty() {
        let matrix = super::clustering::approximate_cooccurrence(constraints);
        if matrix.num_pairs() > 0 {
            let config = super::clustering::ClusteringConfig::default();
            let result = super::clustering::cluster_objects(&matrix, &config);
            let ordered: Vec<saf_core::ids::LocId> =
                result.clusters.iter().flatten().copied().collect();
            P::with_seeded_ordering(&ordered)
        } else {
            P::empty()
        }
    } else {
        P::empty()
    }
}
```

**Step 10**: Update `solve_generic_with_options` (line 159) to accept and use clustering:
```rust
fn solve_generic_with_options<P: PtsSet>(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    constants: Option<&ConstantsTable>,
    index_sensitivity: IndexSensitivity,
    clustering: ClusteringMode,
) -> GenericPointsToMap<P> {
    let template = create_template::<P>(constraints, clustering);
    let mut solver = GenericSolver::<P>::new_with_template(constraints, factory, template)
        .with_index_sensitivity(index_sensitivity);
    if let Some(c) = constants {
        solver = solver.with_constants(c);
    }
    solver.solve(max_iterations);
    solver.pts
}
```

**Step 11**: Update all callers of `solve_generic_with_options` in `solve_with_index_config` (line 101) to pass clustering. The function already receives `pts_config: &PtsConfig` — use `pts_config.clustering`:

Add `use super::ptsset::ClusteringMode;` if not already imported.

In each dispatch arm (lines 126, 136, 146), add the clustering arg:
```rust
            let generic_result = solve_generic_with_options::<BTreePtsSet>(
                constraints, factory, max_iterations, constants, index_sensitivity,
                pts_config.clustering,
            );
```
(Same for BitVecPtsSet and BddPtsSet arms.)

**All existing tests in `solver.rs` must pass unchanged.**

---

### Task 7: CS-PTA solver — add template, replace `P::empty()` calls

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/cspta/solver.rs`. This is the context-sensitive k-CFA solver. It has a `GenericCsSolver<P: PtsSet>` struct.

Read the full file first to understand the struct layout and all `P::empty()` call sites. Then:

1. Add `template: P` field to `GenericCsSolver`
2. Initialize it via `P::empty()` in the constructor (the clustering integration for CS-PTA can be added later; for now, just use `P::empty()` to get the shared-indexer benefit)
3. Replace **every** `P::empty()` call with `self.template.clone_empty()`
4. Replace **every** `P::singleton(x)` with `{ let mut s = self.template.clone_empty(); s.insert(x); s }`
5. Replace **every** `.unwrap_or_else(P::empty)` with `.unwrap_or_else(|| self.template.clone_empty())`

Search for the patterns: `P::empty`, `P::singleton`, to find all call sites.

**All existing CS-PTA tests must pass unchanged.**

---

## Execution Order

```
Group 1 (parallel): Tasks 1, 2, 3, 4, 5
  └─ No dependencies between them
Group 2 (sequential): Task 6
  └─ Depends on Tasks 1-5 being complete
Group 3 (sequential): Task 7
  └─ Depends on Tasks 1-2 (needs clone_empty in trait + BitVec)
  └─ Can run in parallel with Task 6
```

In practice: run Tasks 1-5 in parallel, then run Tasks 6 and 7 in parallel.

## Verification (main agent only — no subagent)

After all tasks complete:

1. `make fmt && make lint`
2. `make test`
3. PTABen regression (run in background, 30-120s):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-clustering.json'
   ```
4. Compare against baseline: 2239 Exact, 392 Sound, 80 Unsound, 93 Skip. Any regression is a blocker.

## Key Conventions

- `BTreeMap`/`BTreeSet` everywhere (determinism)
- No `.unwrap()` in library code — use `.expect("message")`
- Doc comments: backtick type names (clippy `doc_markdown`)
- Function-level `#[allow]` with comments, not crate-level
