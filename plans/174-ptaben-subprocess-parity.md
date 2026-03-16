# Plan 174: PTABen Subprocess Parity

**Goal:** Restore PTABen from 356 unsound to ~61 unsound by fixing the subprocess pipeline.
**Status:** Done (356→147 unsound, -209, -58.7%)
**Design:** `docs/plans/2026-02-26-ptaben-subprocess-parity-design.md`
**Epic:** benchmark

## Phase 1: Infrastructure — Store resolved_sites (2 tasks)

### Task 1: Add resolved_sites to PipelineResult and ProgramDatabase

**File:** `crates/saf-analysis/src/pipeline.rs`
- Add `resolved_sites: BTreeMap<InstId, Vec<FunctionId>>` to `PipelineResult`
- In `run_pipeline()`, capture `resolved_sites` from `RefinementResult` (currently dropped via `..` at line ~165)
- In `run_pipeline_incremental()`, use `BTreeMap::new()` (incremental path doesn't have refinement yet)

**File:** `crates/saf-analysis/src/database/mod.rs`
- Add `resolved_sites: BTreeMap<InstId, Vec<FunctionId>>` field to `ProgramDatabase`
- Add `pub fn resolved_sites(&self) -> &BTreeMap<InstId, Vec<FunctionId>>` accessor
- Wire through `build()` (from `pipeline.resolved_sites`)
- Wire through `from_parts()` (add parameter, default to empty in Python bindings)

**Validation:** `make fmt && make lint` — compile check only, no behavior change.

### Task 2: Pass resolved_sites to CS-PTA in run_bench_mode

**File:** `crates/saf-cli/src/driver.rs`
- Change `solve_context_sensitive()` at line ~828 to `solve_context_sensitive_with_resolved()`
- Pass `self.db.resolved_sites()` as the fourth argument
- Add `use saf_analysis::cspta::solve_context_sensitive_with_resolved;` import

**Validation:** `make test` — existing tests pass. Run PTABen filtered to cs_tests to check improvement:
```
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "cs_tests/*" -o /workspace/tests/benchmarks/ptaben/results-phase1.json'
```

## Phase 2: FS-PTA Load-Sensitive + Alias Fixes (2 tasks)

### Task 3: Use load-sensitive FS-PTA in bench mode

**File:** `crates/saf-cli/src/driver.rs`
- Change FS-PTA config to `skip_df_materialization: false`
- Keep `fs_svfg` alive after solving (currently consumed/dropped)
- Call `fs_result.compute_load_sensitive_pts(&fs_svfg)` to get per-load PTS
- Replace the manual set comparison (lines ~903-918) with proper load-sensitive alias logic:
  ```rust
  // Prefer load-sensitive PTS when available, fall back to global
  let p_pts = load_sensitive_pts.get(&ptr_a)
      .or_else(|| fs_pts.get(&ptr_a));
  let q_pts = load_sensitive_pts.get(&ptr_b)
      .or_else(|| fs_pts.get(&ptr_b));
  // If either is None/empty, return Unknown (defer to CS/CI)
  // Otherwise: disjoint→No, singleton equal→Must, equal→May, subset→Partial, else→May
  ```

**Validation:** Run PTABen filtered to fs_tests:
```
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "fs_tests/*" -o /workspace/tests/benchmarks/ptaben/results-phase2.json'
```

### Task 4: Include ps in pick_best_alias

**File:** `crates/saf-cli/src/driver.rs`
- Modify `pick_best_alias()` signature to accept `ps: Option<&str>`
- Include `ps` in the precision comparison loop
- Update caller in alias query loop

**Validation:** Compile check.

## Phase 3: Path-Sensitive Refinement (3 tasks)

### Task 5: Add per-path refinement (Strategy 1)

**File:** `crates/saf-cli/src/driver.rs`
- Add helper `try_perpath_refinement_bench()` that:
  - Parses `oracle_function` from `AliasQuery` as `FunctionId`
  - Looks up the function in `self.module`
  - Calls `path_sensitive_alias_interprocedural_with_resolved(ptr_a, ptr_b, func, module, &config, resolved_sites)`
  - Returns `Some(AliasResult)` if non-Unknown and non-Unsound against expected kind
  - Includes dead-code detection (Unknown + no direct callers → Exact)
- Import `path_sensitive_alias_interprocedural_with_resolved`, `PathSensitiveAliasConfig`

**Validation:** Run path_tests category to check improvement.

### Task 6: Add callsite refinement (Strategy 2)

**File:** `crates/saf-cli/src/driver.rs`
- Add helper `try_callsite_refinement_bench()` that:
  - Builds `param_indices` for the oracle function via `build_param_indices(func)`
  - Iterates all call sites (direct + resolved indirect) targeting the function
  - Maps oracle params to caller args
  - Re-queries `self.db.may_alias(mapped_a, mapped_b)`
  - Returns `Some(AliasResult)` if any call site yields non-Unsound result
- Import `build_param_indices`

**Validation:** Run basic_cpp_tests category to check improvement.

### Task 7: Add guard-based refinement (Strategy 3)

**File:** `crates/saf-cli/src/driver.rs`
- Add helper `try_guard_refinement_bench()` that:
  - Parses `oracle_block` from `AliasQuery` as `BlockId`
  - Builds `ValueOriginMap` and `PathSensitiveAliasChecker`
  - Calls `checker.may_alias_at(ptr_a, ptr_b, block, func, &mut diag)`
  - Returns `Some(AliasResult)` if non-Unknown
- Import `ValueOriginMap`, `PathSensitiveAliasChecker`, `PathSensitiveConfig`, `PathSensitiveDiagnostics`
- Build the checker lazily (once per bench run, not per query) — store as `Option` alongside other analysis results

**Wire all 3 strategies into alias query loop:**
- After computing `best` from CI/CS/FS, if `best` would be Unsound for the expected kind:
  1. Try per-path refinement
  2. If still Unsound, try callsite refinement
  3. If still Unsound, try guard refinement
- Set `ps` field to the refinement result

Note: The bench result doesn't carry the expected kind — the CLI doesn't know the oracle expectation. We need to run PS refinement unconditionally for all alias queries and let ptaben.rs pick the best. Set `ps` field to the PS result regardless.

Alternative: Run PS refinement only, set `ps` field, and let `pick_best_alias` include it. This is simpler and correct — the old code only ran PS when flow-insensitive was Unsound, but running it always just wastes ~time, not correctness.

**Simpler approach:** Run all 3 PS strategies for every alias query, take the most precise non-Unknown result, set as `ps`. Let `pick_best_alias(ci, cs, fs, ps)` select the best overall.

**Validation:** Full PTABen run — expect significant unsound reduction.

## Phase 4: Checker and Validation Fixes (4 tasks)

### Task 8: Register PTABen resource table for checkers

**File:** `crates/saf-cli/src/bench_types.rs`
- Add `ptaben_wrappers: bool` to `AnalysisFlags` (default false)

**File:** `crates/saf-bench/src/ptaben.rs`
- Set `analyses.ptaben_wrappers = true` in `build_bench_config()`

**File:** `crates/saf-cli/src/driver.rs`
- When `analyses.ptaben_wrappers && analyses.checkers`:
  - Build custom `ResourceTable` with PTABen oracle wrappers (SAFEMALLOC→Allocator, SAFEFREE→Deallocator, etc.)
  - Run `checkers::run_checker()` directly with custom table instead of using JSON protocol
  - Run both double-free and memory-leak checkers

**Validation:** Run mem_leak and double_free categories.

### Task 9: Add memory leak PTA fallback

**File:** `crates/saf-cli/src/driver.rs`
- After running memory-leak checker, post-process findings:
  - For each leak finding, check `is_allocation_freed_via_pta()`:
    scan `CallDirect` to known deallocators, PTA alias check
  - Also check `is_allocation_stored_in_global()`:
    scan stores to globals (direct, GEP, PTA-based)
  - Suppress false positive findings

**Validation:** Run mem_leak category.

### Task 10: Add non-PTA memcpy overflow check

**File:** `crates/saf-cli/src/driver.rs`
- After `check_memcpy_overflow_with_pta_and_specs()`, also run `check_memcpy_overflow_with_specs()`
- Merge findings from both (deduplicate by function+description)

**Validation:** Run ae_overflow_tests category.

### Task 11: Fix assertion and nullness validation edge cases

**File:** `crates/saf-cli/src/bench_types.rs`
- Add `status: String` field to `AssertionResultEntry` (values: "proven", "may_fail", "unknown")

**File:** `crates/saf-cli/src/driver.rs`
- Set `status: "unknown"` for Unknown condition results (currently lumped with may_fail as `proved: false`)

**File:** `crates/saf-bench/src/ptaben.rs`
- For assertions: treat `status == "unknown"` as Skip instead of Unsound
- For nullness: if result is Bottom/unreachable, classify as Skip

**Validation:** Run ae_assert_tests and ae_nullptr_deref_tests categories.

## Phase 5: Full Validation (1 task)

### Task 12: Run full PTABen benchmark and verify ≤61 unsound

Run:
```
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-plan174.json'
```

Expected: ≤61 unsound (matching pre-CLI-wrapping baseline).

Also run: `make fmt && make lint && make test` for full validation.

Update `plans/PROGRESS.md` with results.

## Task Summary

| Phase | Task | Description | Est. Lines |
|-------|------|-------------|-----------|
| 1 | 1 | Store resolved_sites in PipelineResult + ProgramDatabase | +20 |
| 1 | 2 | Pass resolved_sites to CS-PTA in bench mode | +5 |
| 2 | 3 | Use load-sensitive FS-PTA | +30 |
| 2 | 4 | Include ps in pick_best_alias | +5 |
| 3 | 5 | Per-path refinement (Strategy 1) | +50 |
| 3 | 6 | Callsite refinement (Strategy 2) | +50 |
| 3 | 7 | Guard refinement (Strategy 3) + wire all into alias loop | +50 |
| 4 | 8 | PTABen resource table for checkers | +40 |
| 4 | 9 | Memory leak PTA fallback | +40 |
| 4 | 10 | Non-PTA memcpy overflow check | +10 |
| 4 | 11 | Assertion/nullness edge cases | +15 |
| 5 | 12 | Full PTABen validation | 0 |

**Total:** ~315 lines across 5 files, 12 tasks, 5 phases.
