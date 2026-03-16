# Plan 131: Incremental prev_pts + FxHashSet PtsSet

**Goal:** Reduce solver time by replacing BTreeSet (O(log n) ops) with FxHashSet (O(1) ops) for internal PTA computation, and applying incremental prev_pts update in process_value.

**Baseline (Plan 130 post-optimization):**
```
handle_load:       10.36s (73.7%, 2,733,579 locs iterated)
process_location:  11.90s (1,853 calls, 6.4ms avg)
diff_compute:       1.04s
prev_pts_clone:     0.72s
bash Ander:        25.98s (clean, no solver-stats)
bash Total:        43.81s
```

---

## Part A: Incremental prev_pts in process_value

Apply Plan 129's proven pattern: first visit MOVEs diff into prev_pts (zero clone), subsequent visits UNIONs diff into prev (O(|diff|) vs O(|current|) clone). Saves 0.72s clone cost.

## Part B: FxHashSet PtsSet

New `FxHashPtsSet` wrapping `FxHashSet<LocId>`:
- `contains()`: O(1) vs O(log n) — speeds up diff computation
- `union()` via extend: O(m) vs O(m log n) — speeds up handle_load + process_location
- `insert()`: O(1) vs O(log n) — speeds up diff building
- Determinism: solver reaches same unique least fixed point regardless of iteration order. Output normalized via `to_btreeset()`.

---

## Files Modified

- `crates/saf-analysis/src/pta/ptsset/fxhash.rs` — new FxHashPtsSet
- `crates/saf-analysis/src/pta/ptsset/mod.rs` — re-export
- `crates/saf-analysis/src/pta/ptsset/config.rs` — FxHash variant
- `crates/saf-analysis/src/pta/solver.rs` — incremental prev_pts
- `crates/saf-analysis/src/cg_refinement.rs` — use FxHashPtsSet
- `crates/saf-analysis/src/cspta/solver.rs` — FxHash dispatch arm
- `crates/saf-analysis/src/pta/ptsset/config_tests.rs` — roundtrip test

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis` (1422 tests — +5 from FxHashPtsSet tests)
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark

---

## Results

### Profiling (solver-stats)

```
Solver: 11.54s (was 26.86s, -57%)
  handle_load:       4.93s (was 10.36s, -52%)
  process_location:  4.23s (was 11.90s, -64%)
  handle_store:      0.45s (was 1.08s, -58%)
  handle_gep:        0.90s (was 1.08s, -17%)
  diff_compute:      0.51s (was 1.04s, -51%)
  prev_pts_clone:    0.10s (was 0.72s, -86%)
  handle_copy:       0.19s (was 0.35s, -46%)
  normalize:         0.14s (was 0.10s, +52% — FxHash→BTreeSet conversion)
```

Part A (incremental prev_pts) saved 0.62s (0.72s → 0.10s, -86%).
Part B (FxHashSet) saved ~12s across all handler phases via O(1) contains/insert vs O(log n).

### CruxBC Benchmark (clean, no solver-stats)

| Program | Plan 130 | Plan 131 | Change |
|---------|----------|----------|--------|
| bash Ander | 25.98s | 10.86s | **-58.2%** |
| bash Total | 43.81s | 28.05s | **-36.0%** |
| libcurl Total | 7.85s | 7.14s | -9.0% |
| htop Total | 1.52s | 1.37s | -9.9% |
| Small programs | unchanged | unchanged | ~0% |

**Cumulative since Plan 126:** Ander 43.02s → 10.86s (**-74.8%**)
**Cumulative since Plan 128:** Ander 27.61s → 10.86s (**-60.7%**)

### Validation

| Check | Result |
|-------|--------|
| Tests | 1422 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |
