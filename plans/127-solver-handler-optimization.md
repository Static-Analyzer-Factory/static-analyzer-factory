# Plan 127: Solver Handler Optimization (FxHash + Clone Elimination + Batching)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce Andersen PTA solver handler time from 15.44s to ~10-12s on bash benchmark via four targeted micro-optimizations that collectively eliminate redundant hashing, cloning, and allocation in the solver's hot paths.

**Architecture:** Four independent optimizations applied to `GenericSolver`:
1. FxHash for all internal IndexMaps (~2-3s savings)
2. Eliminate `process_value` double-clone (~1-1.5s savings)
3. Store handler early-exit on empty pts (~0.3-0.5s savings)
4. GEP handler batching (~0.3-0.5s savings)

**Tech Stack:** Rust, `rustc-hash` crate for FxHash, no new features or APIs.

**Key risk:** FxHash changes map probe sequences, which could theoretically affect iteration-order-dependent behavior. Mitigation: IndexMap preserves insertion order regardless of hasher; all public API types remain BTreeMap/BTreeSet for determinism.

---

## Baseline (from Plan 126 post-optimization profile)

```
Phase breakdown (cumulative wall-clock, 15.44s total):
  handle_load:   10.69s  (69.3%)  -- 2,897,402 locs iterated
  diff_compute:   1.94s  (12.5%)
  handle_gep:     1.19s  ( 7.7%)
  handle_store:   1.12s  ( 7.3%)
  handle_copy:    0.33s  ( 2.2%)
  scc_detect:     0.10s  ( 0.7%)
  lcd_check:      0.04s  ( 0.2%)
  worklist_ops:   0.02s  ( 0.1%)

bash Ander: 43.02s, Total: 60.87s
```

---

### Task 1: FxHash for Solver and ConstraintIndex Maps

**Files:**
- Modify: `Cargo.toml` (workspace) — add `rustc-hash = "2"` to `[workspace.dependencies]`
- Modify: `crates/saf-analysis/Cargo.toml` — add `rustc-hash.workspace = true` to `[dependencies]`
- Modify: `crates/saf-analysis/src/pta/solver.rs`
- Modify: `crates/saf-analysis/src/pta/constraint_index.rs`

**Step 1: Add rustc-hash dependency**

In workspace `Cargo.toml`, add to `[workspace.dependencies]`:
```toml
rustc-hash = "2"
```

In `crates/saf-analysis/Cargo.toml`, add to `[dependencies]`:
```toml
rustc-hash.workspace = true
```

**Step 2: Add type alias in solver.rs**

Near the top of `solver.rs`, after the existing imports, add:

```rust
use rustc_hash::FxBuildHasher;

/// IndexMap with FxHash for fast u128 key lookups.
/// FxHash is ~6x faster than SipHash for u128 keys (2ns vs 13ns per hash).
/// Used only for internal solver maps; public API types remain BTreeMap for determinism.
type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
```

**Step 3: Replace all IndexMap types in GenericSolver**

Change the `GenericSolver` struct fields:

```rust
pub(crate) struct GenericSolver<'a, P: PtsSet> {
    pub(crate) pts: FxIndexMap<ValueId, P>,
    loc_pts: FxIndexMap<LocId, P>,
    prev_pts: FxIndexMap<ValueId, P>,
    rep: FxIndexMap<ValueId, ValueId>,
    topo_order: FxIndexMap<ValueId, u32>,
    load_loc_index: FxIndexMap<LocId, Vec<usize>>,
    // ... rest unchanged
}
```

Also update `GenericPointsToMap`:
```rust
pub type GenericPointsToMap<P> = FxIndexMap<ValueId, P>;
```

Update both constructors (`new` and `new_with_template`) to use `FxIndexMap::with_capacity_and_hasher(cap, FxBuildHasher)` or `FxIndexMap::default()` for initialization.

**Step 4: Replace IndexMap types in constraint_index.rs**

Add the same import and type alias. Change all 5 maps in `ConstraintIndex`:

```rust
pub struct ConstraintIndex {
    pub copy_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    pub load_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    pub store_by_dst_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    pub store_by_src: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
    pub gep_by_src_ptr: FxIndexMap<ValueId, SmallVec<[usize; 4]>>,
}
```

Update `ConstraintIndex::build()` to initialize maps with `FxIndexMap::default()` or `FxIndexMap::with_capacity_and_hasher(...)`.

**Step 5: Fix compilation**

The `FxIndexMap` type needs proper initialization. `IndexMap::default()` uses default hasher, but with a custom hasher you need:
```rust
FxIndexMap::with_hasher(FxBuildHasher)
```
or for with_capacity:
```rust
FxIndexMap::with_capacity_and_hasher(cap, FxBuildHasher)
```

Search for all `IndexMap::new()`, `IndexMap::default()`, `IndexMap::with_capacity()` calls in solver.rs and constraint_index.rs and update them.

**Step 6: Verify compilation + tests**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo nextest run -p saf-analysis 2>&1 | tail -5'
```

Expected: all tests pass. No behavioral change.

**Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/saf-analysis/Cargo.toml crates/saf-analysis/src/pta/solver.rs crates/saf-analysis/src/pta/constraint_index.rs
git commit -m "perf(pta): replace SipHash with FxHash for solver and constraint index maps"
```

---

### Task 2: Eliminate process_value Double-Clone

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

**Step 1: Rewrite process_value diff computation**

Replace the current `process_value` method (lines ~954-1017) with a version that avoids cloning the current pts set until we know the diff is non-empty:

```rust
fn process_value(&mut self, v: ValueId) {
    let v = self.find_rep(v);

    #[cfg(feature = "solver-stats")]
    {
        self.stats_mut().process_value_calls += 1;
    }

    // Check if v has any points-to set at all
    let Some(current) = self.pts.get(&v) else { return };

    // Compute diff WITHOUT cloning current
    #[cfg(feature = "solver-stats")]
    let diff_start = SolverStats::start_section();

    let diff = if let Some(prev) = self.prev_pts.get(&v) {
        // Build diff element-by-element: elements in current but not in prev
        let mut d = self.template.clone_empty();
        for loc in current.iter() {
            if !prev.contains(loc) {
                d.insert(loc);
            }
        }
        d
    } else {
        // First time seeing this value — diff IS current (one clone)
        current.clone()
    };

    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(diff_start, &mut self.stats_mut().time_diff);

    if diff.is_empty() {
        #[cfg(feature = "solver-stats")]
        {
            self.stats_mut().empty_diff_skips += 1;
        }
        return;
    }

    // Only NOW clone current into prev (we confirmed diff is non-empty)
    // SAFETY: we know v is in pts because we got `current` from it above
    let current_snapshot = self.pts.get(&v).expect("just checked").clone();
    self.prev_pts.insert(v, current_snapshot);

    // Propagate diff to handlers (stats instrumentation unchanged)
    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_copy_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats_mut().time_copy);

    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_load_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats_mut().time_load);

    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_store_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats_mut().time_store);

    #[cfg(feature = "solver-stats")]
    let t = SolverStats::start_section();
    self.handle_gep_constraints(v, &diff);
    #[cfg(feature = "solver-stats")]
    SolverStats::end_section(t, &mut self.stats_mut().time_gep);
}
```

**Key changes:**
- `current` is a borrowed reference (`&P`), not an owned clone
- Diff built element-by-element using `current.iter()` + `!prev.contains(loc)`
- Clone of current into prev_pts happens AFTER empty-diff check (saves clone on empty diff)
- Net: empty diff = 0 clones (was 2), non-empty diff = 1 clone (was 2)

**Step 2: Handle borrow checker**

The key constraint: `self.pts.get(&v)` returns `&P` borrowing `self.pts`. We then access `self.prev_pts.get(&v)` which borrows `self.prev_pts` — this is fine since they're different fields. But the handlers take `&mut self`, so we must drop the `current` borrow before calling handlers. The code above does this correctly: `diff` is computed and `current` reference is dropped, then `prev_pts.insert` uses a fresh `self.pts.get(&v)`.

**Step 3: Verify compilation + tests**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo nextest run -p saf-analysis 2>&1 | tail -5'
```

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): eliminate double-clone in process_value diff computation"
```

---

### Task 3: Store Handler Early-Exit on Empty Pts

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

**Step 1: Add early-exit checks to handle_store_constraints**

In the dst_ptr role loop, check if src has non-empty pts before cloning:

```rust
fn handle_store_constraints(&mut self, v: ValueId, v_pts: &P) {
    // Role 1: v is dst_ptr
    let dst_indices: Vec<usize> = self.index.stores_by_dst_ptr(v).to_vec();
    for i in dst_indices {
        let src = self.indexed.store[i].src;
        // Early exit: skip clone if src has no pts or empty pts
        let has_pts = self.pts.get(&src).map_or(false, |s| !s.is_empty());
        if !has_pts { continue; }
        if let Some(src_pts) = self.pts.get(&src).cloned() {
            for loc in v_pts.iter() {
                // ... unchanged
            }
        }
    }

    // Role 2: v is src
    let src_indices: Vec<usize> = self.index.stores_by_src(v).to_vec();
    for i in src_indices {
        let dst_ptr = self.indexed.store[i].dst_ptr;
        // Early exit: skip clone if dst_ptr has no pts or empty pts
        let has_pts = self.pts.get(&dst_ptr).map_or(false, |s| !s.is_empty());
        if !has_pts { continue; }
        if let Some(dst_pts) = self.pts.get(&dst_ptr).cloned() {
            for loc in dst_pts.iter() {
                // ... unchanged
            }
        }
    }
}
```

**Why this helps:** In early solver iterations, many values have empty or uninitialized pts. The `.cloned()` call does a full BTreeSet deep-clone even when the set is empty — the early exit avoids this. The `is_empty()` check is O(1).

**Step 2: Verify compilation + tests**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo nextest run -p saf-analysis 2>&1 | tail -5'
```

**Step 3: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): skip store handler clone when pts is empty"
```

---

### Task 4: GEP Handler Batching

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

**Step 1: Rewrite handle_gep_constraints to accumulate per-constraint**

Replace the current per-location pattern (create empty set, insert 1, union) with accumulation:

```rust
fn handle_gep_constraints(&mut self, v: ValueId, v_pts: &P) {
    let indices: Vec<usize> = self.index.geps_by_src_ptr(v).to_vec();
    for i in indices {
        let gep_path = self.indexed.gep[i].path.clone();
        let gep_index_operands = self.indexed.gep[i].index_operands.clone();
        let gep_dst = self.indexed.gep[i].dst;

        // Accumulate ALL field locations for this GEP constraint into one set
        let mut accumulated = self.template.clone_empty();
        for loc in v_pts.iter() {
            #[cfg(feature = "solver-stats")]
            {
                self.stats_mut().gep_locs_iterated += 1;
            }
            if let Some(base_loc) = self.factory.get(loc) {
                let resolved_path = self.resolve_gep_path(&gep_path, &gep_index_operands);

                let merged = merge_gep_with_base_path(base_loc, &resolved_path);
                let field_loc = merged
                    .as_ref()
                    .and_then(|p| self.factory.lookup_approx(base_loc.obj, p))
                    .or_else(|| {
                        let new_path = base_loc.path.extend(&resolved_path);
                        self.factory.lookup_approx(base_loc.obj, &new_path)
                    });

                if let Some(field_loc) = field_loc {
                    accumulated.insert(field_loc);
                }
            }
        }

        // Single union into dst instead of one per location
        if !accumulated.is_empty() && self.union_into_value(gep_dst, &accumulated) {
            self.worklist_insert(gep_dst);
        }
    }
}
```

**Key change:** Instead of creating a fresh empty PtsSet and calling `union_into_value` per location, we accumulate all field_locs into one set and call `union_into_value` once per GEP constraint. This eliminates N-1 empty set allocations per constraint (where N = |v_pts|).

**Step 2: Verify compilation + tests**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis && cargo nextest run -p saf-analysis 2>&1 | tail -5'
```

**Step 3: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): batch GEP handler to accumulate field locs before union"
```

---

### Task 5: Benchmark and Validate

**Step 1: Format and lint**

```bash
make fmt && make lint
```

**Step 2: Run full test suite**

```bash
docker compose run --rm dev sh -c 'cargo nextest run --release 2>&1 | tail -10'
```

Expected: all tests pass.

**Step 3: PTABen regression check**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-127.json'
```

Expected: 69 Unsound (no regression).

**Step 4: Profile benchmark (solver-stats)**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench --features solver-stats -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "big/bash" -o /workspace/tests/benchmarks/cruxbc/profile-127.json 2>&1 | tee /tmp/profile-127.txt | tail -40'
```

Record the new handler breakdown and compare with baseline.

**Step 5: Full CruxBC benchmark**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small,big" -o /workspace/tests/benchmarks/cruxbc/results-127.json'
```

Verify no regression on small programs.

**Step 6: Update plan with results and PROGRESS.md**

---

### Task 6: Python Tests and Final Cleanup

**Step 1: Python test suite**

```bash
docker compose run --rm dev sh -c 'cd /workspace && uv run pytest python/tests 2>&1 | tail -10'
```

**Step 2: Final commit if any cleanup needed**

**Step 3: Update PROGRESS.md**

Add Plan 127 entry and update Next Steps.

---

## Results

```
Plan 127 Solver Profile (bash, post-optimization):
  Phase breakdown (cumulative wall-clock, 14.77s total):
    handle_load:    10.74s  (72.7%)
    handle_gep:      1.20s  ( 8.2%)
    diff_compute:    1.14s  ( 7.7%)  ← -41% from 1.94s (clone elimination)
    handle_store:    1.13s  ( 7.6%)
    handle_copy:     0.38s  ( 2.6%)
    scc_detect:      0.13s  ( 0.9%)

  bash Ander: 43.86s, Total: 62.58s
```

| Metric | Plan 126 | Plan 127 | Change |
|--------|----------|----------|--------|
| Solver total | 15.44s | 14.77s | **-4.3%** |
| diff_compute | 1.94s | 1.14s | **-41.3%** |
| handle_load | 10.69s | 10.74s | ~0% |
| handle_store | 1.12s | 1.13s | ~0% |
| handle_gep | 1.19s | 1.20s | ~0% |
| bash Ander | 43.02s | 43.86s | ~0% |
| PTABen Unsound | 69 | 69 | no change |
| Tests | 1417 | 1417 | no change |

## Rollback Plan

Each optimization is in a separate commit. Revert any individual commit if it causes regression:
- Task 1+2 (FxHash + diff clone): `git revert 49f828f` — falls back to SipHash, restores double-clone
- Task 3 (store early-exit): `git revert c7c1ea7` — restores unconditional clone
- Task 4 (GEP batching): `git revert 8b8d225` — restores per-location union
