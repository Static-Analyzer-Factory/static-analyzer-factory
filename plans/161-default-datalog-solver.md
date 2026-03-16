# Plan 161: Make Ascent (Datalog) the Default PTA Solver

## Context

The Ascent Datalog PTA solver is now within 1.28x of the legacy worklist solver on bash (21.4s vs 16.7s) and has near-identical soundness (65 vs 61 unsound on PTABen). It should become the default solver everywhere: CLI benchmarks, Python SDK, and WASM playground. User-facing naming should use "worklist" / "datalog" (not "legacy").

## Changes

### 1. Rename solver options + change default in CLI

**File: `crates/saf-bench/src/ptaben.rs`**
- Change `#[default]` from `Legacy` to `Ascent`
- Update doc comments: "worklist" / "datalog" as user-facing names

**File: `crates/saf-bench/src/main.rs`** (2 places: ptaben + cruxbc subcommands)
- Change `--solver` default from `"legacy"` to `"datalog"`
- Update help text: `"worklist"` or `"datalog"` (default)
- Parsing: `"worklist" | "legacy"` → Legacy, `"datalog" | "ascent"` → Ascent (keep old names as aliases)

### 2. Add Ascent solver to Python API

**File: `crates/saf-python/Cargo.toml`**
- Add `saf-datalog = { workspace = true }`

**File: `crates/saf-python/src/project.rs`**
- Add `pta_solver: &str` param to `Project.open()` (default `"datalog"`)
- Parse: `"worklist"` → worklist, `"datalog"` → Ascent, else error
- In `build_analysis()`: if datalog, run `saf_datalog::pta::analyze_with_ascent()` after `run_pipeline()` and replace the PTA result

### 3. Add Ascent solver to WASM playground (default)

**File: `crates/saf-wasm/Cargo.toml`**
- Add `saf-datalog = { path = "../saf-datalog" }`

**File: `crates/saf-wasm/src/lib.rs`**
- Add `pta_solver: Option<String>` to `AnalysisConfig` struct (default: `"datalog"`)
- In `run_analysis()`: after `run_pipeline()`, run `analyze_with_ascent()` and replace the PTA result in the ProgramDatabase and exports (unless user explicitly passes `"worklist"`)
- `saf-datalog` already supports wasm32 (sequential fallback via `ascent!` macro)

### 4. No Makefile changes needed

Makefile targets don't pass `--solver` — they rely on CLI default. Changing CLI default to `"datalog"` switches them automatically.

## Files to Modify

1. `crates/saf-bench/src/ptaben.rs` — `#[default]` on `PtaSolver` enum
2. `crates/saf-bench/src/main.rs` — `--solver` default + parsing (2 subcommands)
3. `crates/saf-python/Cargo.toml` — add `saf-datalog` dep
4. `crates/saf-python/src/project.rs` — `pta_solver` param in `open()` + `build_analysis()`
5. `crates/saf-wasm/Cargo.toml` — add `saf-datalog` dep
6. `crates/saf-wasm/src/lib.rs` — `pta_solver` in config + Ascent path

## Verification

1. `make fmt && make lint` — clean
2. `make test` — all Rust + Python tests pass
3. PTABen default uses Ascent; `--solver worklist` still accessible
4. Python: `Project.open("test.ll")` uses Ascent; `pta_solver="worklist"` uses legacy
5. WASM: rebuild playground, verify analysis still works with Ascent as default
