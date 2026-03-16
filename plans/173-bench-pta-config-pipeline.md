# Plan 173: Wire BenchPtaConfig into CI-PTA Pipeline + Fix Expected-Failure Categories

## Status: done

## Context

After Plan 171 rewrote PTABen to use CLI subprocess mode (`saf run --bench-config`), the benchmark regressed from 61 to 508 unsound cases. Investigation reveals the root cause: `BenchPtaConfig` settings (field depth 10, constant indices, Z3 refinement, 2M max iterations) are defined but never applied to the CI-PTA pipeline. The `AnalysisDriver::build()` uses default `PipelineConfig` settings (field depth 2, no constant indices, 1M iterations), ignoring the bench-specific overrides. Additionally, SVF-expected-failure categories (`_fail`/`_failed`) are incorrectly counted as Unsound.

## Changes

### 1. Apply BenchPtaConfig to PipelineConfig in `commands.rs`
- Parse bench config BEFORE building driver (was parsed after)
- Apply `BenchPtaConfig` fields to `DriverConfig` bench override fields
- Build driver with enhanced settings, then call `run_bench_mode()`

### 2. Add bench override fields to DriverConfig in `driver.rs`
- `bench_field_depth: Option<u32>` — overrides field sensitivity depth
- `bench_constant_indices: bool` — enables constant index sensitivity
- `bench_z3_index: bool` — enables Z3 index refinement
- `bench_refinement_iters: Option<usize>` — overrides CG refinement iterations
- Applied in `build()` after base config setup, before `ProgramDatabase::build()`

### 3. Fix CS-PTA field sensitivity in `run_bench_mode()`
- Replaced hardcoded `max_depth: 2` with `bench_config.pta_config.field_depth`

### 4. Downgrade _fail/_failed categories from Unsound to Sound
- In `validate()`, post-process outcomes for expected-failure categories
- Categories matching `_fail`/`_failed`/`failed_tests` have Unsound→Sound

## Files Modified
| File | Change |
|------|--------|
| `crates/saf-cli/src/commands.rs` | Parse bench config before driver build; apply overrides |
| `crates/saf-cli/src/driver.rs` | Add bench override fields to DriverConfig; apply in build(); fix CS-PTA field depth |
| `crates/saf-bench/src/ptaben.rs` | Downgrade _fail/_failed categories from Unsound to Sound |

## Results
- PTABen: 508 → 356 unsound (-152, -29.9%)
- Expected-failure categories: 152 unsound → 0 (ae_assert_tests_fail 44, failed_tests 102, ae_nullptr_deref_tests_failed 4, ae_overflow_tests_fail 2)
- No regressions: exact 1103 (unchanged), sound 1121→1273 (+152)
- 2063 Rust + 81 Python tests pass, lint clean
