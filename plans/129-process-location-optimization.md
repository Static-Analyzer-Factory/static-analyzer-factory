# Plan 129: Further process_location Optimization

**Goal:** Reduce `process_location` cost from 12.09s (45.7% of solver) through targeted micro-optimizations.

**Baseline (Plan 128 post-optimization):**
```
process_location:  12.09s (45.7% of solver 26.45s)
  2,056 calls, avg 5.9ms each
  bash Ander:    26.66s
  bash Total:    43.74s
```

---

## Optimizations

### Opt 1: Early-exit before diff computation

Check `load_loc_index.contains_key(&loc)` before any diff computation or cloning. If no load constraints reference this location, there's nothing to propagate — skip everything.

Many locations are written to (via Store) but never loaded from, or not yet discovered by the load handler. This avoids wasted work on those locations.

### Opt 2: SmallVec for load indices

Replace `self.load_loc_index.get(&loc).cloned().unwrap_or_default()` (heap-allocating Vec clone) with `SmallVec<[usize; 8]>` copied from slice. Most locations have few load constraints (typically 1-4), so the SmallVec's inline storage avoids heap allocation entirely.

### Opt 3: Incremental prev_loc_pts update

**First visit:** After propagation, MOVE diff into `prev_loc_pts` instead of cloning current again. Since diff IS current on first visit, this saves one full PtsSet clone.

**Subsequent visits:** Union diff into existing `prev_loc_pts` entry — O(|diff|) instead of O(|current|) for a full clone. Correctness: `loc_pts` is monotone, so `prev ∪ diff = prev ∪ (current \ prev) = current`.

### Opt 4: Profiling instrumentation

Add `solver-stats` counters for:
- `process_location_early_exits` — skipped due to no load indices
- `process_location_first_visits` — first-time visit (full diff = current)
- `process_location_diff_empty` — diff was empty, skipped propagation

---

## Files Modified

- `crates/saf-analysis/src/pta/solver.rs` — `process_location` rewrite
- `crates/saf-analysis/src/pta/solver_stats.rs` — new counters + print

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis` (1417 tests)
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark with `--features solver-stats`

---

## Results

### Profiling (solver-stats)

```
process_location:  12.35s (2056 calls, 84 early-exit, 792 first-visit, 0 diff-empty)
```

- 84 early-exits (4.1%): locations written to but never loaded from
- 792 first-visits (38.5%): saved one BTreeSet clone each via move
- 1,180 subsequent visits: saved full BTreeSet clone via incremental union

### CruxBC Benchmark (clean, no solver-stats)

| Program | Plan 128 | Plan 129 | Change |
|---------|----------|----------|--------|
| bash Ander | 27.61s | 26.79s | **-3.0%** |
| bash Total | 45.62s | 43.57s | **-4.5%** |
| libcurl Total | 8.17s | 7.90s | -3.3% |
| Small programs | unchanged | unchanged | no change |

### Validation

| Check | Result |
|-------|--------|
| Tests | 1417 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |
