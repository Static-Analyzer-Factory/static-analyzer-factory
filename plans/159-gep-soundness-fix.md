# Plan 159: Ascent PTA GEP Soundness Fix

## Problem

The Ascent PTA solver had 52 soundness regressions compared to the legacy worklist solver, all traced to incomplete GEP (field-sensitive pointer) resolution. Two bugs:

1. **Bug 1**: Two-phase GEP resolution in `analyze_with_ascent()` runs Phase 1 (solve without GEPs) before callgraph refinement resolves indirect/virtual calls. Values receiving pts from virtual dispatch have no Phase 1 pts, so Phase 2 GEP resolution falls back to conservative copy edges.

2. **Bug 2**: PTABen re-solve path after CG refinement calls `ascent_solve()` directly, which ignores GEP facts entirely.

## Solution

### Fix 1: Iterative GEP Resolution (gep.rs + context.rs)

Replace single-pass two-phase GEP resolution with iterative approach:

1. Added `GepResolutionResult` struct and `try_resolve_gep_facts()` to `gep.rs` — resolves GEPs without copy fallbacks, returns unresolved GEPs for retry
2. Added `iterative_gep_resolve()` helper in `context.rs` — loops up to 10 iterations: resolve GEPs, re-solve if any new resolutions, add copy fallbacks only when no progress
3. Both `analyze_with_ascent()` and `solve_from_constraints()` use the iterative approach

### Fix 2: `solve_from_constraints()` (context.rs)

Extracted Steps 5-9 of `analyze_with_ascent()` into a public `solve_from_constraints()` function for reuse when re-solving from augmented constraints.

### Fix 3: PTABen Re-Solve Path (ptaben.rs)

Replaced the broken re-solve path (raw `ascent_solve` ignoring GEPs) with `solve_from_constraints()` which performs full HVN + SCC + iterative GEP resolution.

## Results

- basic_cpp_tests: 4 → 0 unsound (-4, all destructor alias failures FIXED)
- ae_assert_tests: 13 → 12 unsound (-1)
- Total Ascent unsound: 163 → 158 (-5)
- Legacy baseline: 66 unsound (unchanged)
- 1895 Rust + 81 Python tests pass

## Files Changed

| File | Change |
|------|--------|
| `crates/saf-datalog/src/pta/gep.rs` | Added `GepResolutionResult`, `try_resolve_gep_facts()`, test |
| `crates/saf-datalog/src/pta/context.rs` | Iterative GEP loop, `solve_from_constraints()`, `iterative_gep_resolve()` |
| `crates/saf-datalog/src/pta/mod.rs` | Exported new types and functions |
| `crates/saf-bench/src/ptaben.rs` | Fixed re-solve path to use `solve_from_constraints()` |
