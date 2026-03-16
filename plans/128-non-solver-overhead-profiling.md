# Plan 128: Non-Solver Overhead Profiling & Optimization

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Identify and optimize the ~29s of non-solver overhead in the Andersen PTA pipeline. The solver itself (14.77s) is only 34% of the total bash Ander time (43.86s). The remaining ~66% is uninstrumented and unoptimized.

**Architecture:** Two-phase approach:
1. **Phase 1 (Profiling):** Add `#[cfg(feature = "solver-stats")]` timing instrumentation to `cg_refinement.rs::refine()` to measure each pipeline step.
2. **Phase 2 (Optimization):** Target the top 1-2 bottlenecks revealed by profiling.

**Tech Stack:** Rust, `std::time::Instant`, existing `solver-stats` feature flag.

**Key risk:** None for Phase 1 (read-only instrumentation). Phase 2 risks depend on what profiling reveals.

---

## Baseline

```
Plan 127 Post-Optimization (bash):
  Solver total:  14.77s  (34% of Ander)
  Non-solver:    ~29.09s (66% of Ander)  ← UNINSTRUMENTED
  bash Ander:    43.86s
  bash Total:    62.58s
```

The non-solver overhead includes everything in `cg_refinement.rs::refine()` outside the solver's `solve()` and `drain_worklist()` calls:

| Step | Code Location | Description |
|------|--------------|-------------|
| `setup` | Lines 132-134 | `FunctionLocationMap::build`, `collect_indirect_call_sites`, `collect_return_values` |
| `extract` | Lines 137-138 | `extract_constraints(module, &mut factory)` — walks all 141k instructions |
| `hvn` | Lines 141-151 | `hvn_preprocess(&mut reduced)` — SCC + signature-based merging |
| `solver_init` | Lines 154-156 | `GenericSolver::new(&reduced, &factory)` — builds ConstraintIndex, IndexedConstraints |
| `solver_solve` | Line 156 | `solver.solve(max_iterations)` — already profiled at ~14.77s |
| `cg_loop` | Lines 162-179 | Online CG refinement: `resolve_and_connect` + `drain_worklist` per iteration |
| `normalize` | Lines 189-193 | `.to_btreeset()` for all pts values — BTreePtsSet → BTreeSet<LocId> |
| `hvn_expand` | Lines 196-200 | Clone pts for all HVN-mapped originals |
| `cha_narrow` | Lines 215-240 | CHA/PTA resolution narrowing |
| `pta_result` | Lines 243-248 | `PtaResult::new(pts, &factory, diagnostics)` |

---

### Task 1: Add Profiling Instrumentation

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs`

**Step 1: Add timing blocks around each step**

Wrap each step in `refine()` with `#[cfg(feature = "solver-stats")]` timing:

```rust
#[cfg(feature = "solver-stats")]
let t_step = Instant::now();

// ... existing code for step ...

#[cfg(feature = "solver-stats")]
let step_time = t_step.elapsed().as_secs_f64();
```

Collect all step times into a local struct or tuple, then print a breakdown table at the end (before the return statement).

**Step 2: Print breakdown table**

At the end of `refine()`, print:

```rust
#[cfg(feature = "solver-stats")]
{
    let total = pta_start.elapsed().as_secs_f64();
    eprintln!("\n=== CG Refinement Pipeline Profile ===");
    eprintln!("  setup:        {:.3}s ({:.1}%)", t_setup, t_setup / total * 100.0);
    eprintln!("  extract:      {:.3}s ({:.1}%)", t_extract, t_extract / total * 100.0);
    // ... etc for each step
    eprintln!("  TOTAL:        {:.3}s", total);
}
```

**Step 3: Verify compilation**

```bash
docker compose run --rm dev sh -c 'cargo check -p saf-analysis --features solver-stats'
```

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/cg_refinement.rs
git commit -m "perf(pta): add pipeline profiling instrumentation to CG refinement"
```

---

### Task 2: Run Profiled Benchmark

**Step 1: Run bash benchmark with solver-stats**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench --features solver-stats -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "big/bash" 2>&1 | tee /tmp/profile-128.txt'
```

**Step 2: Record the breakdown**

Capture the CG Refinement Pipeline Profile output and the solver profile output. Record both in this plan's Results section.

**Step 3: Identify top bottlenecks**

Rank steps by wall-clock time. The top 1-2 non-solver steps are the optimization targets for Phase 2.

---

### Task 3: Design and Implement Optimizations

Based on profiling results. Likely candidates:

**If `extract` dominates (~10-15s):**
- Constraint extraction walks 141k instructions. Options:
  - Profile sub-steps (alloca, store, load, call, gep constraint generation)
  - Parallelize with rayon (per-function extraction is independent)
  - Cache extracted constraints (they don't change across CG iterations)

**If `normalize` + `hvn_expand` dominate (~5-10s):**
- `.to_btreeset()` creates a new BTreeSet for every value. Options:
  - Avoid normalization entirely — keep `FxIndexMap<ValueId, BTreePtsSet>` and convert `PtaResult` to accept it
  - Lazy normalization — only convert when a specific value is queried
  - Parallel normalization with rayon

**If `cg_loop` dominates (~5-10s):**
- Each iteration scans all indirect call sites. Options:
  - Index indirect sites by their function pointer `ValueId` for O(1) lookup
  - Track which pointers' pts changed and only re-check those sites
  - Delta-based resolution (only examine sites whose pointer pts grew)

**If `hvn` dominates (~5-10s):**
- HVN builds full adjacency graph + runs Tarjan SCC. Options:
  - Use FxHash for HVN internal maps
  - Incremental HVN that avoids full graph rebuild

---

### Task 4: Benchmark and Validate

**Step 1: Format and lint**
```bash
make fmt && make lint
```

**Step 2: Full test suite**
```bash
docker compose run --rm dev sh -c 'cargo nextest run --release -p saf-analysis 2>&1 | tail -10'
```

**Step 3: PTABen regression check**
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-128.json'
```

Expected: 69 Unsound (no regression).

**Step 4: CruxBC benchmark**
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small,big" -o /workspace/tests/benchmarks/cruxbc/results-128.json'
```

---

## Results

### Phase 1: Pipeline Profiling (before optimization)

```
=== CG Refinement Pipeline Profile ===
  setup:          0.004s (  0.0%)  [FuncLocMap + indirect sites + returns]
  extract:        0.025s (  0.1%)  [extract_constraints, 92370 constraints]
  hvn:            0.041s (  0.1%)  [HVN preprocess, 984 classes, 6482 removed]
  solver_init:    0.006s (  0.0%)  [GenericSolver::new + ConstraintIndex]
  solver_solve:  42.753s ( 99.5%)  [solver.solve() initial fixed point]
  cg_loop:        0.028s (  0.1%)  [1 iterations, resolve+drain]
  normalize:      0.099s (  0.2%)  [to_btreeset for 48503 values]
  hvn_expand:     0.010s (  0.0%)  [clone pts for 4546 HVN mappings]
  TOTAL:         42.970s  (solver: 42.781s, non-solver: 0.189s)
```

Key finding: **Non-solver overhead is only 0.189s (0.4%)**, not ~29s. The "missing time" was inside the solver but not attributed to any handler.

### Deeper Solver Profiling

```
Deeper instrumentation:
  process_value total:    14.86s  (handler sum: 14.21s, prev_pts clone: 0.68s)
  process_location:       27.64s  (2056 calls, 13.4ms each)
```

**`process_location` was 64.7% of solver time** — no diff-based propagation, re-unioned entire `loc_pts` each call.

### Phase 2: Diff-Based process_location (after optimization)

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| process_location | 27.64s | 12.09s | **-56.3%** |
| solver_solve | 42.73s | 26.45s | **-38.1%** |
| bash Ander | 42.95s | 26.66s | **-37.9%** |
| bash Total | 61.20s | 43.74s | **-28.5%** |
| PTABen Unsound | 69 | 69 | no change |
| Tests | 1417 | 1417 | no change |

### Full CruxBC Benchmark (post-optimization)

```
Program               Funcs    Insts      Load   CG Ref    Ander    Total
big/bash               2071   141.1k    17.79s    0.22s   27.61s   45.62s
big/libcurl.so         1234    92.5k     8.06s    0.02s    0.08s    8.17s
small/bc                145    10.5k     0.17s    0.00s    0.02s    0.30s
small/bunzip2           108    18.7k     0.61s    0.00s    0.02s    0.75s
small/dc                113     6.0k     0.05s    0.00s    0.00s    0.07s
small/htop              407    21.1k     0.84s    0.01s    0.28s    1.58s
small/libbz2.so          64    16.3k     0.58s    0.00s    0.01s    0.66s
```

## Rollback Plan

Phase 1 (profiling) is behind `solver-stats` feature flag — zero impact on normal builds. Phase 2 optimizations will be in separate commits, each independently revertible.
