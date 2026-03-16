# Roaring PTS Default + Frozen Indexer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Switch the default PTS representation from FxHash/BTreeSet to Roaring bitmap with a frozen (lock-free) indexer, reducing bash PTS memory from ~954 MB to ~45 MB.

**Architecture:** Pre-build a `FrozenIndexer` from all `LocId`s in addr constraints before solving, wrap it in `Arc` (no `RwLock`), and thread it through `RoaringPtsSet` via the existing `clone_empty()` / `with_seeded_ordering()` pattern. Update auto-selection to prefer Roaring for programs with ≥10K alloc sites.

**Tech Stack:** Rust, `roaring` crate (already in Cargo.toml), `bitvec` crate (kept for `IdBitSet`), existing `PtsSet` trait infrastructure.

**Design doc:** `docs/plans/2026-02-25-roaring-pts-frozen-indexer-design.md`

---

## Phase 1: Frozen Indexer Infrastructure

### Task 1: Create `FrozenIndexer<T>`

**Files:**
- Modify: `crates/saf-analysis/src/pta/ptsset/indexer.rs`
- Test: inline `#[cfg(test)] mod tests` in same file

A frozen indexer is an immutable snapshot of `Indexer<T>`. It uses `FxHashMap` for O(1) forward lookup (vs `BTreeMap`'s O(log n) in the mutable indexer) and `Vec<T>` for O(1) reverse lookup. No locks needed since it's immutable.

**Step 1: Write failing test**

Add to the existing `tests` module in `indexer.rs`:

```rust
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
```

**Step 2: Run test — expected FAIL** (`freeze` method not found)

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis frozen_indexer_basic'
```

**Step 3: Implement `FrozenIndexer<T>`**

Add to `indexer.rs`, above the existing `Indexer<T>` impl:

```rust
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
```

Add a `freeze()` method to `Indexer<T>`:

```rust
/// Freeze this indexer into an immutable `FrozenIndexer`.
///
/// The frozen indexer uses `FxHashMap` for O(1) lookups (vs `BTreeMap`'s O(log n))
/// and requires no locks for concurrent reads.
#[must_use]
pub fn freeze(&self) -> FrozenIndexer<T> {
    let item_to_idx: FxHashMap<T, u32> = self
        .item_to_idx
        .iter()
        .map(|(&item, &idx)| (item, idx as u32))
        .collect();
    FrozenIndexer {
        item_to_idx,
        idx_to_item: self.idx_to_item.clone(),
    }
}
```

**Step 4: Run test — expected PASS**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis frozen_indexer_basic'
```

**Step 5: Add more frozen indexer tests, then commit**

```rust
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
```

Update the `pub use` in `ptsset/mod.rs` to also export `FrozenIndexer`:

```rust
pub use indexer::{FrozenIndexer, Indexer, LocIdIndexer};
```

Add type alias:

```rust
// in indexer.rs, after LocIdIndexer alias
pub type FrozenLocIdIndexer = FrozenIndexer<saf_core::ids::LocId>;
```

```bash
git add crates/saf-analysis/src/pta/ptsset/indexer.rs crates/saf-analysis/src/pta/ptsset/mod.rs
git commit -m "feat(ptsset): add FrozenIndexer for lock-free PTS operations"
```

---

### Task 2: Add frozen indexer support to `RoaringPtsSet`

**Files:**
- Modify: `crates/saf-analysis/src/pta/ptsset/roaring_pts.rs`
- Test: inline tests in same file

Currently `RoaringPtsSet` wraps `Arc<RwLock<LocIdIndexer>>`. We add a dual-mode enum: either locked (for building) or frozen (for solving). The frozen path eliminates all lock acquisition.

**Step 1: Write failing test**

Add to existing tests in `roaring_pts.rs`:

```rust
#[test]
fn frozen_roaring_basic_ops() {
    // Build indexer, freeze, create sets
    let mut indexer = LocIdIndexer::new();
    indexer.get_or_insert(LocId::new(1));
    indexer.get_or_insert(LocId::new(2));
    indexer.get_or_insert(LocId::new(3));
    let frozen = Arc::new(indexer.freeze());

    let mut pts = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
    assert!(pts.insert(LocId::new(1)));
    assert!(pts.insert(LocId::new(2)));
    assert!(!pts.insert(LocId::new(1))); // duplicate
    assert_eq!(pts.len(), 2);
    assert!(pts.contains(LocId::new(1)));
    assert!(pts.contains(LocId::new(2)));
    assert!(!pts.contains(LocId::new(3)));
}
```

**Step 2: Run test — expected FAIL** (`with_frozen_indexer` not found)

**Step 3: Implement dual-mode indexer in `RoaringPtsSet`**

Replace the `indexer` field with an enum:

```rust
use super::indexer::{FrozenIndexer, FrozenLocIdIndexer};

/// Indexer state — either mutable (building) or frozen (solving).
#[derive(Clone, Debug)]
enum IndexerState {
    /// Mutable indexer behind RwLock — used during construction.
    Mutable(Arc<RwLock<LocIdIndexer>>),
    /// Frozen indexer — lock-free reads during solving.
    Frozen(Arc<FrozenLocIdIndexer>),
}
```

Update `RoaringPtsSet`:

```rust
pub struct RoaringPtsSet {
    bitmap: RoaringBitmap,
    indexer: IndexerState,
}
```

Add `with_frozen_indexer()` constructor and update all methods to dispatch on `IndexerState`. The key methods:

- `insert()`: `Frozen` path calls `frozen.get(loc)` (returns `Option<u32>`, panics if not found — all locs pre-registered)
- `contains()`: `Frozen` path calls `frozen.get(loc)` — O(1), no lock
- `iter()`: `Frozen` path calls `frozen.resolve(idx)` — O(1) per element, no lock
- `union()`: Both frozen → check `Arc::ptr_eq` on frozen arcs, then bitmap `|=`
- `clone_empty()`: Propagates `IndexerState` to new set
- `from_btreeset()`: Uses mutable path (not performance-critical)

**Important**: For the `Frozen` path in `insert()`, if `frozen.get(loc)` returns `None`, the location wasn't pre-registered. This should never happen during normal solving (verified by agent research). Use `expect("LocId not in frozen indexer — all locations must be pre-registered")`.

**Step 4: Run test — expected PASS**

**Step 5: Add frozen union test**

```rust
#[test]
fn frozen_roaring_union() {
    let mut indexer = LocIdIndexer::new();
    for i in 0..10 {
        indexer.get_or_insert(LocId::new(i));
    }
    let frozen = Arc::new(indexer.freeze());

    let mut a = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
    let mut b = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
    a.insert(LocId::new(1));
    a.insert(LocId::new(2));
    b.insert(LocId::new(2));
    b.insert(LocId::new(3));

    assert!(a.union(&b));
    assert_eq!(a.len(), 3);
    assert!(a.contains(LocId::new(1)));
    assert!(a.contains(LocId::new(2)));
    assert!(a.contains(LocId::new(3)));
}

#[test]
fn frozen_roaring_clone_empty_shares_frozen() {
    let mut indexer = LocIdIndexer::new();
    indexer.get_or_insert(LocId::new(1));
    let frozen = Arc::new(indexer.freeze());

    let mut pts = RoaringPtsSet::with_frozen_indexer(Arc::clone(&frozen));
    pts.insert(LocId::new(1));
    let empty = pts.clone_empty();
    assert!(empty.is_empty());
    // Both should use the same frozen indexer
}
```

```bash
git add crates/saf-analysis/src/pta/ptsset/roaring_pts.rs
git commit -m "feat(ptsset): add frozen indexer support to RoaringPtsSet"
```

---

## Phase 2: Wire Frozen Indexer Into Solver

### Task 3: Pre-build and freeze indexer in `create_template()`

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs` (lines 283-342, `create_template()`)

The `create_template()` function already processes all addr constraints for clustering. We extend it to also collect ALL `LocId`s, build a `FrozenIndexer`, and create the template with it.

**Step 1: Modify `create_template()` to collect all LocIds and freeze**

The function currently returns `P`. For Roaring specifically, we want it to use the frozen indexer. The cleanest approach: add a new `PtsSet` trait method `with_frozen_indexer_and_ordering()` that both seeds the ordering AND uses a frozen indexer.

Actually, simpler: add a new method to the trait:

```rust
// In trait_def.rs
/// Create an empty set with a pre-built frozen indexer.
///
/// For indexed representations, uses the frozen indexer for lock-free operations.
/// Default implementation ignores the frozen indexer and returns `empty()`.
fn with_frozen_ordering(frozen: Arc<FrozenIndexer<LocId>>) -> Self {
    let _ = frozen;
    Self::empty()
}
```

Then in `create_template()`, collect all LocIds from constraints, freeze them, and call `P::with_frozen_ordering()`:

```rust
pub(crate) fn create_template<P: PtsSet>(constraints: &ConstraintSet, mode: ClusteringMode) -> P {
    // Collect ALL LocIds from addr constraints
    let all_locs: Vec<LocId> = constraints.addr.iter().map(|a| a.loc).collect();

    let should_cluster = match mode { ... };
    if should_cluster && !constraints.addr.is_empty() {
        // ... existing clustering code ...
        // Use ordered (clustered) locs for indexer ordering
        let ordered: Vec<LocId> = result.clusters.iter().flatten().copied().collect();

        // Build frozen indexer from clustered ordering
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(ordered.iter().copied());
        // Also register any locs NOT in clusters (edge case)
        indexer.register_batch(all_locs.iter().copied());
        let frozen = Arc::new(indexer.freeze());
        P::with_frozen_ordering(frozen)
    } else {
        // No clustering — register all locs in natural order
        let mut indexer = LocIdIndexer::new();
        indexer.register_batch(all_locs.iter().copied());
        let frozen = Arc::new(indexer.freeze());
        P::with_frozen_ordering(frozen)
    }
}
```

**Step 2: Implement `with_frozen_ordering()` for `RoaringPtsSet`**

```rust
fn with_frozen_ordering(frozen: Arc<FrozenIndexer<LocId>>) -> Self {
    Self {
        bitmap: RoaringBitmap::new(),
        indexer: IndexerState::Frozen(frozen),
    }
}
```

**Step 3: Test — run existing solver tests to verify no regressions**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis solver'
```

All existing solver tests use `BTreePtsSet` / `FxHashPtsSet` which ignore the frozen indexer (trait default returns `empty()`), so they should pass unchanged.

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/pta/ptsset/trait_def.rs crates/saf-analysis/src/pta/solver.rs
git commit -m "feat(solver): pre-build frozen indexer in create_template()"
```

---

### Task 4: Update `PtsConfig` auto-selection to prefer Roaring

**Files:**
- Modify: `crates/saf-analysis/src/pta/ptsset/config.rs` (lines 250-263, `select_by_count()`)
- Modify: `crates/saf-analysis/src/pta/ptsset/config.rs` (test at line 342)

**Step 1: Change the auto-selection thresholds**

Current:
```rust
fn select_by_count(&self, alloc_count: usize) -> PtsRepresentation {
    if alloc_count >= self.bdd_threshold {       // >= 100K → BDD
        PtsRepresentation::Bdd
    } else if alloc_count >= self.roaring_threshold { // >= 50K → Roaring
        PtsRepresentation::Roaring
    } else {
        PtsRepresentation::FxHash              // < 50K → FxHash
    }
}
```

New:
```rust
fn select_by_count(&self, alloc_count: usize) -> PtsRepresentation {
    if alloc_count >= self.bdd_threshold {
        PtsRepresentation::Bdd
    } else if alloc_count >= self.roaring_threshold {
        PtsRepresentation::Roaring
    } else {
        PtsRepresentation::FxHash
    }
}
```

And change the default `roaring_threshold` from 50,000 to 10,000:

```rust
impl Default for PtsConfig {
    fn default() -> Self {
        Self {
            representation: PtsRepresentation::Auto,
            bitvec_threshold: 10_000,
            roaring_threshold: 10_000,  // was 50_000
            bdd_threshold: 100_000,
            clustering: ClusteringMode::Auto,
        }
    }
}
```

**Step 2: Update the config test**

The test `select_by_count_custom_thresholds` uses explicit thresholds (not defaults), so it should still pass. But update the `pts_config_default` test:

```rust
#[test]
fn pts_config_default() {
    let config = PtsConfig::default();
    assert_eq!(config.representation, PtsRepresentation::Auto);
    assert_eq!(config.bitvec_threshold, 10_000);
    assert_eq!(config.roaring_threshold, 10_000);
    assert_eq!(config.bdd_threshold, 100_000);
}
```

**Step 3: Run config tests**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis pts_config'
```

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/pta/ptsset/config.rs
git commit -m "feat(ptsset): lower Roaring auto-select threshold to 10K alloc sites"
```

---

## Phase 3: Delete BitVecPtsSet + Consolidate

### Task 5: Replace BitVecPtsSet references with RoaringPtsSet

**Files to modify** (all `BitVecPtsSet` import/usage sites):
- `crates/saf-analysis/src/pta/solver.rs` — lines 33, 187-195, 238-265, 1454, 1768, 1831
- `crates/saf-analysis/src/pta/solver_repr_tests.rs` — lines 15, 78, 121, 172, 221, 257, 281
- `crates/saf-analysis/src/pta/ptsset/cross_impl_tests.rs` — all `BitVecPtsSet` → `RoaringPtsSet`
- `crates/saf-analysis/src/pta/ptsset/edge_case_tests.rs` — BitVec-specific tests → Roaring equivalents
- `crates/saf-analysis/src/pta/ptsset/mod.rs` — line 68 (remove `pub use bitvec::BitVecPtsSet`)
- `crates/saf-analysis/src/cspta/solver.rs` — line 20, 321
- `crates/saf-analysis/src/fspta/solver.rs` — comment only (line 12)
- `crates/saf-analysis/src/pta/mod.rs` — comment only

**Step 1: Update `solver.rs`**

In the imports (line 33), remove `BitVecPtsSet` from the import list.

In the `solve_with_index_config` dispatch (line 187-195), change `PtsRepresentation::BitVector` to dispatch to `RoaringPtsSet` instead:

```rust
PtsRepresentation::BitVector => {
    // BitVector now dispatches to Roaring (BitVecPtsSet removed)
    let (generic_result, limit_hit) = solve_generic_with_options::<RoaringPtsSet>(
        &reduced, factory, max_iterations, constants, index_sensitivity, pts_config.clustering,
    );
    (normalize_result(generic_result), limit_hit)
}
```

Change `solve_bitvec()` (lines 238-265) to use `RoaringPtsSet`:

```rust
pub fn solve_bitvec(
    constraints: &ConstraintSet,
    factory: &LocationFactory,
    max_iterations: usize,
    module: Option<&AirModule>,
) -> (GenericPointsToMap<RoaringPtsSet>, HvnResult) {
    // ... same body, but uses RoaringPtsSet ...
    let (generic_result, _) = solve_generic_with_options::<RoaringPtsSet>(...);
    (generic_result, hvn_result)
}
```

Update solver unit tests (line 1768, 1831) to use `RoaringPtsSet`.

**Step 2: Update `solver_repr_tests.rs`**

Replace all `BitVecPtsSet` with `RoaringPtsSet` in imports and test bodies.

**Step 3: Update `cross_impl_tests.rs`**

Replace `BitVecPtsSet` with `RoaringPtsSet` throughout. The test structure stays the same — it validates BTree, Roaring, and BDD produce identical results.

Update `create_all_impls()`:
```rust
fn create_all_impls() -> (BTreePtsSet, RoaringPtsSet, BddPtsSet) {
    let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
    let context = Arc::new(RwLock::new(BddContext::new(16)));

    let btree = BTreePtsSet::empty();
    let roaring = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
    let bdd = BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));

    (btree, roaring, bdd)
}
```

**Step 4: Update `edge_case_tests.rs`**

Replace the 3 BitVec-specific tests with Roaring equivalents:
- `bitvec_large_sparse_set` → `roaring_large_sparse_set`
- `bitvec_capacity_growth` → remove (Roaring handles capacity internally)
- `bitvec_shared_indexer_different_sets` → `roaring_shared_indexer_different_sets`

Also replace BitVecPtsSet in the clone and zero/max LocId tests.

**Step 5: Update `cspta/solver.rs`**

Replace `BitVecPtsSet` import and dispatch.

**Step 6: Update `ptsset/mod.rs`**

Remove line 68 (`pub use bitvec::BitVecPtsSet;`).

**Step 7: Run all tests**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis'
```

**Step 8: Commit**

```bash
git add -A
git commit -m "refactor(ptsset): replace BitVecPtsSet references with RoaringPtsSet"
```

---

### Task 6: Delete `bitvec.rs`

**Files:**
- Delete: `crates/saf-analysis/src/pta/ptsset/bitvec.rs`
- Modify: `crates/saf-analysis/src/pta/ptsset/mod.rs` — remove `mod bitvec;`

**Step 1: Remove the module declaration and file**

In `mod.rs`, remove `mod bitvec;` (line 48).

Delete the file `bitvec.rs`.

**Step 2: Verify it compiles**

```bash
docker compose run --rm dev sh -c 'cargo build -p saf-analysis'
```

**Step 3: Run full test suite**

```bash
docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis'
```

**Step 4: Lint**

```bash
make fmt && make lint
```

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/pta/ptsset/bitvec.rs crates/saf-analysis/src/pta/ptsset/mod.rs
git commit -m "refactor(ptsset): delete BitVecPtsSet (consolidated to RoaringPtsSet)"
```

---

## Phase 4: Benchmark Validation

### Task 7: Validate PTABen precision unchanged

**Files:** none (benchmark run only)

PTABen tests verify the solver produces correct alias answers. The representation change should produce byte-identical `PointsToMap` results. We need to confirm: same exact/unsound counts as before.

**Step 1: Run PTABen with Roaring auto-selection**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-roaring.json'
```

This runs with default `PtsConfig::Auto`, which now selects Roaring for ≥10K alloc sites. PTABen programs are small enough that FxHash will still be selected — but that's fine, it validates no regressions.

**Step 2: Force Roaring and re-run**

To explicitly test the Roaring path, temporarily override the threshold or add a CLI flag. Alternatively, verify via the solver unit tests which explicitly use `solve_generic::<RoaringPtsSet>(...)`.

**Step 3: Compare results**

Expected: 61 unsound (legacy), same exact counts as baseline. If any difference, investigate.

---

### Task 8: Benchmark CruxBC memory improvement

**Files:** none (benchmark run only)

This is the key validation — run the full CruxBC benchmark suite and compare peak memory (RSS) for bash, tmux, and other programs.

**Step 1: Run CruxBC benchmark**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc'
```

Run in background (takes 30-120s per program).

**Step 2: Compare bash memory**

Expected: bash peak RSS drops from ~2.6 GB to ~1.7 GB (PTS savings ~880 MB, clobber cache unchanged). Monitor the printed peak memory column.

**Step 3: Compare all programs**

Expected: No regressions on small programs (htop, unrar, curl) — they still use FxHash via auto-select. tmux should also see memory reduction (similar density to bash).

**Step 4: Document results in commit message**

```bash
git add -A  # any config adjustments from tuning
git commit -m "bench: validate Roaring PTS memory savings on CruxBC

bash: ~2.6G → ~X.XG (PTS 954 MB → ~45 MB)
tmux: ~2.0G → ~X.XG
Other programs: unchanged (FxHash auto-selected)"
```

---

## Phase 5: Cleanup and Documentation

### Task 9: Update module docs and PROGRESS.md

**Files:**
- Modify: `crates/saf-analysis/src/pta/ptsset/mod.rs` — update module doc to reflect Roaring as primary
- Modify: `plans/PROGRESS.md` — add plan 170 to index, update session log, mark complete

**Step 1: Update `mod.rs` doc comment**

Replace the opening doc comment to document the new hierarchy:
- `FxHashPtsSet`: Default for small programs (<10K alloc sites)
- `RoaringPtsSet`: Default for medium/large programs (≥10K alloc sites)
- `BddPtsSet`: Experimental, explicit opt-in for >100K sites
- `BTreePtsSet`: Baseline, for debugging/deterministic inspection
- `IdBitSet<T>`: Generic bitvec set for non-PTS uses (worklists, etc.)

**Step 2: Update PROGRESS.md**

Add to Plans Index:
```
| 170 | roaring-pts-frozen-indexer | scalability | done | Notes: Switched PTS default from FxHash/BTreeSet to Roaring bitmap with frozen indexer. Deleted BitVecPtsSet. Roaring auto-selected for ≥10K alloc sites. bash PTS: 954 MB → ~45 MB. Plan: `plans/170-roaring-pts-frozen-indexer.md`. Design: `docs/plans/2026-02-25-roaring-pts-frozen-indexer-design.md`. |
```

Append to Session Log.

Update Key Decisions table:
```
| PtsSet | Roaring default (≥10K), FxHash (<10K), frozen indexer for lock-free solving |
```

**Step 3: Commit**

```bash
git add plans/PROGRESS.md crates/saf-analysis/src/pta/ptsset/mod.rs
git commit -m "docs: update PROGRESS.md and module docs for Plan 170"
```

---

## Task Dependencies

```
Task 1 (FrozenIndexer) ──→ Task 2 (Roaring frozen support) ──→ Task 3 (Wire into solver)
                                                                       │
Task 4 (Config thresholds) ────────────────────────────────────────────┤
                                                                       │
Task 5 (Replace BitVecPtsSet refs) ──→ Task 6 (Delete bitvec.rs) ─────┤
                                                                       │
                                                             Task 7 (PTABen validation)
                                                             Task 8 (CruxBC benchmark)
                                                                       │
                                                             Task 9 (Docs + PROGRESS.md)
```

Tasks 1-3 are sequential (each depends on the prior). Task 4 is independent. Tasks 5-6 depend on Task 2 (for `with_frozen_indexer()`). Tasks 7-8 depend on Tasks 3+4+6. Task 9 is last.

**Parallelism opportunity:** Tasks 4, 5 can run in parallel once Task 2 is done.
