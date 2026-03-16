# Plan 130: handle_load Shared Accumulation + Store-Side Filtering

**Goal:** Reduce `handle_load` cost (10.60s, 73.8% of handler time) and eliminate wasted `process_location` calls.

**Baseline (Plan 129 post-optimization):**
```
handle_load:    10.60s (73.8%)  — 2,897,402 locs iterated
process_location: 12.35s (2056 calls, 84 early-exit)
bash Ander:  26.79s
bash Total:  43.57s
```

---

## Part A: Shared Accumulation in handle_load

The accumulated set (union of `loc_pts[loc]` for each `loc` in `v_pts`) is **identical** for all K load constraints sharing the same `src_ptr = v`. Currently computed K times; hoist to compute once.

**Before:** O(K * M) loc_pts lookups per call
**After:** O(M + K) — compute once, union into each dst

### Implementation

Restructure `handle_load_constraints`:
1. First pass: compute `accumulated` over all `v_pts` locations (same as current inner loop)
2. Second pass: for each load constraint, union `accumulated` into `dst`

## Part B: Store-Side loc_worklist Filtering

**Current flow:** `handle_store` → `union_into_location` → if changed, add to `loc_worklist` → later, `process_location` pops and checks `load_loc_index` (84 early-exits per Plan 129).

**Optimization:** Don't add to `loc_worklist` if no `load_loc_index` entry exists. No retroactive insertion needed in `handle_load` — it already propagates `loc_pts[loc]` content directly via the accumulated set. Future stores will use the normal path since `load_loc_index` now has entries.

### Implementation

1. In `handle_store_constraints`: after `union_into_location`, check `load_loc_index.contains_key(&loc)` before `loc_worklist.insert(loc)`
2. ~~In `handle_load_constraints`: retroactively add to loc_worklist~~ — NOT needed. handle_load directly propagates via accumulated set. Retroactive insertion caused 15K wasted diff-empty process_location calls in initial implementation; removed.

---

## Files Modified

- `crates/saf-analysis/src/pta/solver.rs` — handle_load + handle_store rewrite

## Validation

1. `make fmt && make lint`
2. `cargo nextest run --release -p saf-analysis` (1417 tests)
3. PTABen regression (69 Unsound expected)
4. CruxBC benchmark

---

## Results

### Profiling (solver-stats)

```
handle_load:       10.62s — 2,733,579 locs iterated (was 2,897,402, -5.6%)
process_location:  12.21s — 1,853 calls (was 2,056, -9.9%), 0 early-exit, 0 diff-empty
```

Part A saved ~164K loc iterations (shared accumulation across K>1 constraints).
Part B eliminated 203 wasted process_location calls.

### CruxBC Benchmark (clean, no solver-stats)

| Program | Plan 129 | Plan 130 | Change |
|---------|----------|----------|--------|
| bash Ander | 26.79s | 25.98s | **-3.0%** |
| bash Total | 43.57s | 43.81s | ~0% (Load noise) |
| libcurl | 7.90s | 7.85s | -0.6% |
| Small programs | unchanged | unchanged | no change |

**Cumulative since Plan 128:** Ander 27.61s → 25.98s (**-5.9%**)

### Validation

| Check | Result |
|-------|--------|
| Tests | 1417 pass, 5 skip |
| PTABen | 69 Unsound (no regression) |
| `make fmt && make lint` | clean |

### Design Iteration Note

Initial Part B included retroactive `loc_worklist` insertion in `handle_load` when creating new `load_loc_index` entries. This caused process_location to spike from 2,056 to 17,324 calls (15,373 diff-empty — wasted work). Root cause: handle_load already propagates `loc_pts[loc]` content directly via the accumulated set, making retroactive insertion redundant. Removed the retroactive insertion; process_location calls dropped to 1,853.
