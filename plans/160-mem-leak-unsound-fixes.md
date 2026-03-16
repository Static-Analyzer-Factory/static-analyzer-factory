# Plan 160: Memory Leak Unsound Fixes

**Epic:** Soundness
**Status:** done
**Source:** `docs/debug/ptaben-mem-leak-unsound-analysis.md`

## Goal

Fix 5 unsound memory leak cases in PTABen identified in the analysis document.
Both root causes affect both Legacy and Ascent solvers identically.

## Tasks

### Task 1: RC1 — Fix Incomplete Global Store Detection (2 False Positives)

**Files:** `crates/saf-bench/src/ptaben.rs`
**Tests:** `malloc13.ll`, `sp7.ll`

Enhance `is_allocation_stored_in_global()` to recognize two additional patterns:

1. **GEP-derived global pointers:** If `dest_ptr` is used in a Store and there exists a GEP instruction in the same function whose result is `dest_ptr` and whose base operand (first operand) is a global, treat the store as escaping to a global.

2. **PTA alias overlap with globals:** If the PTA points-to set of `dest_ptr` overlaps with the points-to set of ANY global variable, the store escapes to a global. The current code only checks `global_ids.contains(&dest_ptr)` (direct identity) but misses indirect global access via function returns (sp7.ll pattern).

### Task 2: RC2 — Add Partial Leak Detection via CFG Analysis (3 False Negatives)

**Files:** `crates/saf-analysis/src/checkers/solver.rs`, `crates/saf-analysis/src/checkers/runner.rs`
**Tests:** `sp5a.ll`, `sp8.ll`, `sp9.ll`

The SVFG BFS `must_not_reach` solver can't detect partial leaks because value-flow edges don't exist on paths where the allocation isn't used (e.g., early return). Add CFG-based partial leak detection:

1. **New function `detect_partial_leaks()`** in `solver.rs`:
   - For each allocation source that has NO finding from `must_not_reach` but DOES have a reachable sanitizer in the SVFG:
   - Find the source's containing function and block
   - Find "sanitizer blocks" — blocks in the source function that contain deallocator calls (direct `free()` calls or calls to functions that free the allocation). Use the SVFG BFS to identify which call sites lead to sanitizers.
   - Build the CFG for the containing function
   - BFS on CFG from the allocation block, skipping sanitizer blocks
   - If any exit block is reachable without going through a sanitizer block → partial leak

2. **Wire into `runner.rs`:**
   - In both `run_checkers` and `run_checkers_guarded`, after calling `must_not_reach`, call `detect_partial_leaks()` and extend findings.

### Task 3: Benchmark Validation

Run PTABen mem_leak benchmarks to verify:
- malloc13.ll and sp7.ll now produce Exact (was Unsound)
- sp5a.ll, sp8.ll, sp9.ll now produce Exact or Sound (was Unsound)
- No regressions in other categories

## Results

- mem_leak unsound: 5 → 0 (all 5 fixed)
- Total PTABen unsound: 66 → 61 (legacy)
- 1895 Rust + 81 Python tests pass
- No regressions in any other category
