# Plan 132: GEP Hoist + Load Iteration Merge

**Goal:** Reduce handle_gep and handle_load cost by eliminating redundant work in inner loops.

**Baseline (Plan 131 post-optimization):**
```
Solver: 10.78s
  handle_load:     4.60s (42.7%)  — 2,733,579 locs iterated
  process_location: 4.16s (38.6%)  — 1,853 calls
  handle_gep:      0.74s  (6.9%)  — 3,451,275 locs iterated
  diff_compute:    0.45s  (4.2%)
  handle_store:    0.38s  (3.5%)
  handle_copy:     0.16s  (1.5%)
  normalize:       0.14s  (1.3%)
bash Ander: 10.86s (clean, no solver-stats)
bash Total: 28.05s
```

---

## Part A: Hoist `resolve_gep_path` in `handle_gep_constraints`

`resolve_gep_path(&gep_path, &gep_index_operands)` is called inside the inner `for loc in v_pts.iter()` loop. The inputs depend only on the GEP constraint, not on which location is being processed. With 3.45M loc iterations, this is millions of redundant calls.

**Fix:** Move `resolve_gep_path` before the inner loop, compute once per GEP constraint.

## Part C: Merge double `v_pts` iteration in `handle_load_constraints`

`v_pts` is iterated twice: Phase 1 (accumulate `loc_pts`) and Phase 4 (update `load_loc_index`). Merging these into a single pass eliminates redundant FxHashSet iteration.

**Fix:** Merge `load_loc_index` registration into the Phase 1 loop body.

---

## Files Modified

- `crates/saf-analysis/src/pta/solver.rs` — both changes

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis` (1421+ tests)
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark

---

## Results

### Profiling (solver-stats)

```
Solver: 10.26s (was 10.78s, -4.8%)
  handle_load:     4.46s (was 4.60s, -3.0%)
  process_location: 3.83s (was 4.16s, -7.9%)
  handle_gep:      0.73s (was 0.74s, -1.4%)
  handle_store:    0.35s (was 0.38s, -7.9%)
  diff_compute:    0.44s (was 0.45s, -2.2%)
  handle_copy:     0.16s (unchanged)
  normalize:       0.14s (unchanged)
```

Part A (GEP hoist) gave modest improvement — `resolve_gep_path` was cheap per call; the real GEP cost is in `factory.get()` and `lookup_approx()` BTreeMap lookups.

Part C (load iteration merge) helped more — `load_loc_index` entries registered earlier means `process_location` finds relevant constraints sooner, reducing wasted work.

### CruxBC Benchmark (clean, no solver-stats)

| Program | Plan 131 | Plan 132 | Change |
|---------|----------|----------|--------|
| bash Ander | 10.86s | 10.31s | **-5.1%** |
| bash Total | 28.05s | 27.23s | **-2.9%** |

**Cumulative since Plan 126:** Ander 43.02s → 10.31s (**-76.0%**)

### Validation

| Check | Result |
|-------|--------|
| Tests | 1421 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |
