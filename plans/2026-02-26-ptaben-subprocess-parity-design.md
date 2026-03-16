# PTABen Subprocess Parity — Design Document

**Date:** 2026-02-26
**Plan:** 174
**Status:** Approved

## Problem

Plan 171 replaced PTABen's in-process analysis with CLI subprocess wrapping
(`saf run --bench-config`). This reduced code duplication but introduced 295
unsound regressions (61 → 356) because the subprocess pipeline lacks several
analysis features the old in-process code provided.

## Root Cause Analysis

An agent team analyzed 5000+ lines of old vs new code and identified 11
behavioral differences. The top 7 account for nearly all regressions:

| # | Root Cause | Impact | Category |
|---|-----------|--------|----------|
| 1 | Path-sensitive alias refinement missing | ~66 | path_tests, basic_cpp_tests |
| 2 | CS-PTA missing resolved indirect call sites | ~45 | cs_tests |
| 3 | FS-PTA using global PTS not load-sensitive | ~26 | fs_tests |
| 4 | Checker resource table missing (PTABen wrappers) | ~12 | mem_leak, double_free |
| 5 | Memory leak PTA fallback missing | ~12 | mem_leak |
| 6 | Buffer overflow PTA alias matching lost | ~10 | ae_overflow_tests |
| 7 | Missing non-PTA memcpy overflow check | ~5 | ae_overflow_tests |
| 8 | basic_cpp_tests: virtual dispatch + no refinement | ~120 | basic_cpp_tests |

Note: basic_cpp_tests (120 unsound) is a combination of issues 1 and 2.

## Approach

Fix the subprocess pipeline (`run_bench_mode()` in driver.rs) so it produces
the same analysis results as the old in-process code. No architectural changes
— we keep the subprocess model from Plan 171.

### Alternative approaches considered

**Hybrid (subprocess + in-process refinement):** Rejected because it partially
reverses Plan 171's dependency cleanup, requiring saf-bench to re-depend on
saf-analysis for path-sensitive refinement.

**Revert to in-process:** Rejected because it abandons the CLI wrapping
architecture and re-introduces duplicated pipeline code.

## Detailed Design

### 1. Store resolved_sites in ProgramDatabase

**Files:** `crates/saf-analysis/src/pipeline.rs`, `crates/saf-analysis/src/database/mod.rs`

The CG refinement step produces `resolved_sites: BTreeMap<InstId, Vec<FunctionId>>`
mapping indirect call instructions to their resolved callee targets. Currently
this is dropped via `..` in `run_pipeline()`.

Changes:
- Add `resolved_sites` field to `PipelineResult`
- Capture from `RefinementResult` in `run_pipeline()` (line ~158)
- Add `resolved_sites` field to `ProgramDatabase`
- Add `pub fn resolved_sites(&self)` accessor
- Wire through `ProgramDatabase::build()` and `from_parts()`

### 2. Pass resolved_sites to CS-PTA

**File:** `crates/saf-cli/src/driver.rs`

Change line ~828 from:
```rust
solve_context_sensitive(self.db.module(), self.db.call_graph(), &cspta_config)
```
to:
```rust
solve_context_sensitive_with_resolved(
    self.db.module(), self.db.call_graph(), &cspta_config, self.db.resolved_sites()
)
```

### 3. Fix FS-PTA to use load-sensitive PTS

**File:** `crates/saf-cli/src/driver.rs`

The current code uses `skip_df_materialization: true` and queries global
top-level PTS. The old code used `compute_load_sensitive_pts()` for per-load
flow-sensitive PTS.

Changes:
- Set `skip_df_materialization: false` in bench mode FS-PTA config
- Keep the `FsSvfg` alive (currently dropped after solve)
- Call `fs_result.compute_load_sensitive_pts(&fs_svfg)` to build per-load PTS
- Build `FsPts::with_load_sensitive(global_pts, load_pts)` (from combined_pta.rs)
- Use `FsPts::may_alias()` for the FS alias string instead of manual set comparison

This requires importing `FsPts` from `saf_bench::combined_pta`. Since saf-cli
cannot depend on saf-bench (circular), we either:
- (a) Move `FsPts` to saf-cli's bench_types.rs, or
- (b) Inline the load-sensitive alias logic directly in driver.rs

Option (b) is simpler — ~20 lines of inline code matching the old `FsPts::may_alias()`.

### 4. Add path-sensitive alias refinement

**File:** `crates/saf-cli/src/driver.rs`

Port the 3 refinement strategies from old ptaben.rs. These run when the
combined CI+CS+FS result would be Unsound for an alias query, and the query
has `oracle_block` and `oracle_function` set.

**Strategy 1 — Per-path refinement:**
- Parse `oracle_function` from `AliasQuery`
- Call `path_sensitive_alias_interprocedural_with_resolved(ptr_a, ptr_b, func, module, &ps_config, resolved_sites)`
- If result is non-Unknown and non-Unsound, use it
- Dead code detection: if result is Unknown and function has no direct callers, report Exact

**Strategy 2 — Callsite refinement:**
- Build `param_indices` for the oracle function via `build_param_indices(func)`
- For each call site that targets the function, map oracle params to caller args
- Re-query CI-PTA with mapped values
- If any call site yields non-Unsound result, use it

**Strategy 3 — Guard refinement:**
- Build `PathSensitiveAliasChecker` with `ValueOriginMap`
- Call `may_alias_at(ptr_a, ptr_b, block, func, &mut diag)`
- If result is non-Unknown and non-Unsound, use it

Return the path-sensitive result as the `ps` field in `AliasResultEntry`.
Include `ps` in `pick_best_alias()`.

### 5. Register PTABen resource table for checkers

**File:** `crates/saf-cli/src/driver.rs`, `crates/saf-cli/src/bench_types.rs`

Add `ptaben_wrappers: bool` field to `AnalysisFlags`. When true, register
PTABen oracle wrapper functions as allocators/deallocators before running
SVFG checkers. This enables the checker to track memory through wrapper
functions like `SAFEMALLOC`, `DOUBLEFREEMALLOC`, etc.

The resource table registration mirrors the old `create_ptaben_resource_table()`.

Since the checker currently runs via the JSON protocol (`check_all`), we need
an alternative: run the checker directly using `checkers::run_checker()` with
a custom resource table, instead of going through the protocol. The protocol
doesn't support custom resource tables.

### 6. Fix memory leak validation with PTA fallback

**File:** `crates/saf-cli/src/driver.rs`

After running the memory-leak checker, post-process findings to suppress false
positives: for each leak finding, scan all `CallDirect` instructions for known
deallocator functions and check if any deallocator argument may-alias the
allocation site via CI-PTA. If so, suppress the finding (the allocation IS
freed through that path — the SVFG BFS just couldn't trace through wrappers).

Also check if the allocation is stored in a global variable (direct store,
GEP-derived store, or PTA-based store-to-global detection).

### 7. Fix buffer overflow validation

**File:** `crates/saf-cli/src/driver.rs`

- Add `check_memcpy_overflow_with_specs()` (non-PTA variant) alongside the
  existing PTA variant, merging findings from both
- For each buffer finding, include the PTA points-to set of the affected
  pointer in the finding (or at minimum, include it as metadata so ptaben.rs
  can do PTA-based matching)

Simpler alternative: include a `pts: Vec<String>` field on `BenchBufferFinding`
listing the LocIds the pointer may point to. Then ptaben.rs can do overlap
matching.

### 8. Minor validation fixes

**File:** `crates/saf-bench/src/ptaben.rs`

- For nullness validation: if the subprocess returns a query but the pointer
  is unreachable (would be Bottom in old code), handle gracefully
- For assertion validation: distinguish Unknown conditions from MayFail —
  add an `unknown: bool` field to `AssertionResultEntry` so ptaben.rs can
  classify Unknown as Skip instead of Unsound

## Expected Outcome

356 unsound → ~61 unsound, matching the pre-CLI-wrapping baseline.

## Files Changed

| File | Type | Est. Lines |
|------|------|-----------|
| `crates/saf-analysis/src/pipeline.rs` | modify | +5 |
| `crates/saf-analysis/src/database/mod.rs` | modify | +15 |
| `crates/saf-cli/src/driver.rs` | modify | +200 |
| `crates/saf-cli/src/bench_types.rs` | modify | +10 |
| `crates/saf-bench/src/ptaben.rs` | modify | +20 |

Total: ~250 lines added across 5 files.
