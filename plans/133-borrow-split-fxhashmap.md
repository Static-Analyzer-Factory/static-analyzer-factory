# Plan 133: Borrow Splitting + LocationFactory FxHashMap

**Goal:** Eliminate `.to_vec()` heap allocations in all 4 handler methods via Rust field-level borrow splitting, and speed up GEP lookups by switching LocationFactory maps to FxHashMap.

**Baseline (Plan 132):**
```
Solver: 10.26s
  handle_load:     4.46s (71.5%)
  process_location: 3.83s
  handle_gep:      0.73s (11.7%)
  handle_store:    0.35s
  handle_copy:     0.16s
bash Ander: 10.31s (clean)
bash Total: 27.23s
```

---

## Part A: Borrow Splitting

**Problem:** All 4 handlers call `.to_vec()` on index slices to avoid borrow conflicts with `&mut self` methods (`union_into_value`, `worklist_insert`, `union_into_location`).

**Fix:** Inline `union_into_value`, `worklist_insert`, and `union_into_location` directly in handlers using field-level borrows. Rust allows borrowing disjoint struct fields simultaneously (e.g., `&self.index` + `&mut self.pts`), but not through `&mut self` method calls.

Also:
- Make `resolve_gep_path` a free function (eliminates `&self` borrow in GEP handler)
- Remove `.cloned()` on `self.pts.get(...)` in handle_store (no longer needed since `union_into_location` is inlined — `self.pts` and `self.loc_pts` are disjoint fields)
- Inline `stats_mut()` as `self.stats.get_mut()` for solver-stats

## Part B: LocationFactory FxHashMap

**Problem:** `handle_gep` does `factory.get(loc)` + up to 3 `lookup_approx` calls, each O(log n) BTreeMap lookups. With 3.45M loc iterations, this adds up.

**Fix:** Change `LocationFactory.locations` and `id_map` from `BTreeMap` to `FxHashMap`. `Location` already derives `Hash`.

---

## Files Modified

- `crates/saf-analysis/src/pta/solver.rs` — borrow splitting in all handlers, free functions for GEP resolution
- `crates/saf-analysis/src/pta/location.rs` — FxHashMap for LocationFactory
- `crates/saf-analysis/src/pta/value_origin.rs` — updated function signatures for FxHashMap

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis`
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark

---

## Results

### Profiling (solver-stats)

```
Solver: 9.71s (was 10.26s, -5.4%)
  handle_load:     4.41s (was 4.46s, -1.1%)
  process_location: 3.69s (was 3.83s, -3.7%)
  handle_gep:      0.43s (was 0.73s, -41.1%)  ← FxHashMap O(1) lookups
  handle_store:    0.34s (was 0.35s, -2.9%)
  handle_copy:     0.14s (was 0.16s, -12.5%)
  diff_compute:    0.42s (was 0.44s, -4.5%)
```

Part B (FxHashMap) drove the GEP improvement — `factory.get()` and `factory.lookup_approx()` went from O(log n) BTreeMap to O(1) FxHashMap. With 3.45M loc iterations, this saved 0.30s.

Part A (borrow splitting) eliminated `.to_vec()` allocations and `.cloned()` in all handlers. Modest impact since the slices were typically small (1-10 elements), but cleaner code with zero unnecessary heap allocations.

### CruxBC Benchmark (clean, no solver-stats)

| Program | Plan 132 | Plan 133 | Change |
|---------|----------|----------|--------|
| bash Ander | 10.31s | ~10.0s | **-3.0%** |
| bash Total | 27.23s | ~27.2s | ~flat |

**Cumulative since Plan 126:** Ander 43.02s → ~10.0s (**-76.8%**)

### Validation

| Check | Result |
|-------|--------|
| Tests | 1421 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |
