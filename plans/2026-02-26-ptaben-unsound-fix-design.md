# PTABen Unsound Fix — Design Document

**Date:** 2026-02-26
**Context:** Plan 171 replaced saf-bench's inline analysis pipeline with subprocess invocations of `saf run --bench-config`. PTABen results: 2883 checks — 971 Exact, 1121 Sound, 61 ToVerify, 640 Unsound. This plan fixes the 640 unsound cases across three tiers.

## Problem Analysis

The 640 unsound cases break down into 4 root causes:

| Root Cause | Tests Affected | Category |
|---|---|---|
| Missing FS-PTA | ~300+ | basic_cpp_tests, failed_tests, fs_tests |
| Buffer overflow validation bug | ~168 | ae_overflow_tests |
| CS-PTA premature termination | ~45 | cs_tests |
| Weak assertion proving + missing assert_eq | ~80+ | ae_assert_tests, ae_assert_tests_fail |
| Memory leak (pre-existing) | 12 | mem_leak_tests |

## Tier 1: Config & Validation Fixes

### 1a. CS-PTA max_iterations

**File:** `crates/saf-cli/src/driver.rs:781`

The CS-PTA solver is configured with `max_iterations: 100`, which is 20,000x lower than the default (2M). This causes premature solver termination on cs_tests.

**Fix:** Use `bench_config.pta_config.max_iterations` instead of hardcoded 100. The `BenchPtaConfig` defaults to 2M.

### 1b. Buffer overflow validation filtering

**File:** `crates/saf-bench/src/ptaben.rs`

`validate_buffer_from_findings()` checks `!findings.is_empty()` against ALL buffer findings for the entire file. A single true overflow causes all `SAFE_BUFACCESS` expectations in the same file to fail.

**Fix:** Pass the expected pointer's hex ID into the validator. Filter `findings.iter().filter(|f| f.ptr == expected_ptr_hex)` before checking emptiness.

The `Expectation::BufferAccess` variant already carries `ptr: ValueId` — thread it through to the validation function.

## Tier 2: Wire Existing APIs

### 2a. FS-PTA (flow-sensitive pointer analysis)

**Problem:** `driver.rs:815` has `fs: None` with comment "FS-PTA blocked — requires SVFG not exposed by ProgramDatabase". All APIs exist and are public — only the SVFG getter on `ProgramDatabase` is `pub(crate)`.

**Changes:**

1. **Expose SVFG** (`crates/saf-analysis/src/database/mod.rs`):
   - Change `pub(crate) fn get_or_build_svfg()` to `pub fn get_or_build_svfg()`
   - Similarly expose `get_or_build_mssa()` if needed (MSSA is consumed during FsSvfg build)

2. **Wire FS-PTA in `run_bench_mode()`** (`crates/saf-cli/src/driver.rs`):
   ```
   if bench_config.analyses.fspta {
       let svfg = self.db.get_or_build_svfg();
       // Build FsSvfg from SVFG + PTA + MSSA + CallGraph
       let fs_svfg = FsSvfgBuilder::new(...).build();
       let fs_config = FsPtaConfig { skip_df_materialization: true, .. };
       let fs_result = solve_flow_sensitive(&module, &fs_svfg, pta, callgraph, &fs_config);
       // For each alias query, compute FS alias result
       // Populate AliasResultEntry.fs
   }
   ```

3. **Update `pick_best_alias()`** to accept FS result as third parameter (currently takes `None`).

**Dependencies:** `solve_flow_sensitive` requires `FsSvfg` which requires `Svfg` + `MemorySsa`. `ProgramDatabase` lazily builds both via `get_or_build_svfg()` / `get_or_build_mssa()`. The FS-PTA solver also needs the `PtaResult` and `CallGraph`, both already exposed.

**Memory consideration:** Use `skip_df_materialization: true` on `FsPtaConfig` to avoid expanding shared version table entries (~2 MB deduped can become ~20 GB materialized on large programs).

### 2b. assert_eq interval wiring

**Problem:** `driver.rs:881-883` has a TODO: "Wire interval-based assert_eq validation once the bench harness emits IntervalQuery entries in BenchConfig." The `svf_assert_eq(a, b)` oracle checks whether the intervals of `a` and `b` overlap.

**Changes:**

1. **Add interval query type** to `bench_types.rs`:
   ```rust
   pub struct IntervalQuery {
       pub call_site: String,    // InstId hex
       pub left_value: String,   // ValueId hex
       pub right_value: String,  // ValueId hex
   }
   ```
   Add `interval_queries: Vec<IntervalQuery>` to `BenchConfig`.

2. **Extract assert_eq expectations** in `ptaben.rs build_bench_config()`:
   - `Expectation::AssertEq { call_site, left, right, .. }` → `IntervalQuery`

3. **Wire interval analysis** in `driver.rs`:
   - After absint runs, for each interval query: look up intervals for left/right values at the call site instruction
   - Check overlap and populate `IntervalResultEntry` (type already exists in bench_types.rs)

4. **Validate** in `ptaben.rs`: match assert_eq expectations against interval results.

## Tier 3: Absint Precision Improvements

Targeted improvements to the abstract interpretation. Scoped to high-confidence changes — WTO, octagon domain, and Z3 integration are deferred.

### 3a. Increase narrowing iterations in bench mode

**File:** `crates/saf-cli/src/driver.rs`

Change `AbstractInterpConfig::default()` to use `narrowing_iterations: 5` when running bench mode assertions. The config knob already exists; current default is 3. Negligible cost increase (RPO narrowing is fast), more loop-bound stabilization.

### 3b. Loop body threshold scanning

**File:** `crates/saf-analysis/src/absint/fixpoint.rs` (~lines 1371-1433)

Current `extract_loop_thresholds()` only scans loop headers and their predecessors. Gap: doesn't examine loop body blocks (e.g., the `for.inc` block containing `i++`).

**Fix:** Extend scanning to include all blocks in the loop body (blocks dominated by the header that have back-edges to it). This captures increment constants like `+1` used in loop updates, preventing widening past the actual loop bound.

### 3c. Singleton narrowing

**File:** `crates/saf-analysis/src/absint/interval.rs`

Current generalized narrowing doesn't specialize on singleton inputs. If during narrowing one Phi path produces `[5,5]` (singleton) and the current interval is `[0, +inf]`, the narrow result should be `[5,5]`.

**Fix:** In the `narrow()` method, if `other` is a singleton (`lo == hi`), return `other` directly when it's contained within `self`.

### 3d. Branch condition refinement for Phi narrowing

**File:** `crates/saf-analysis/src/absint/fixpoint.rs`

Branch conditions refine values within each branch, but Phi node joins lose this refinement. During the narrowing phase, use stored branch condition info to refine Phi inputs per-predecessor before joining.

Example: After a loop `for (i=0; i<5; i++)`, the exit edge carries the refinement `i >= 5`. Combined with the loop body's `i < 5` → the post-loop Phi should narrow to `[5, 5]`.

## Expected Impact

| Fix | Unsound Eliminated | Confidence |
|---|---|---|
| Tier 1a: CS-PTA tuning | ~30-40 of 45 | High |
| Tier 1b: Buffer overflow filter | ~150-168 of 168 | Very high |
| Tier 2a: FS-PTA wiring | ~250-300 of 300+ | High |
| Tier 2b: assert_eq wiring | ~15-20 (currently skipped) | Medium |
| Tier 3: Absint precision | ~10-20 additional | Medium |
| **Total** | ~455-548 of 640 | |

Remaining ~90-185 unsound after all fixes: pre-existing alias precision issues (61 legacy), deep absint cases (2D arrays, float casting, relational constraints), and path-sensitive PTA (not implemented).

## Files Modified

| File | Change |
|---|---|
| `crates/saf-analysis/src/database/mod.rs` | Expose `get_or_build_svfg()` as `pub` |
| `crates/saf-cli/src/driver.rs` | Wire FS-PTA, assert_eq, CS-PTA tuning, narrowing config |
| `crates/saf-cli/src/bench_types.rs` | Add `IntervalQuery` type, `interval_queries` field |
| `crates/saf-bench/src/ptaben.rs` | Fix buffer overflow validation, extract assert_eq, validate intervals |
| `crates/saf-analysis/src/absint/fixpoint.rs` | Loop body thresholds, Phi narrowing |
| `crates/saf-analysis/src/absint/interval.rs` | Singleton narrowing |
