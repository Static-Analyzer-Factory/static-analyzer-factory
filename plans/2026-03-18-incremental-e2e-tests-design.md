# Incremental Analysis E2E Test Design

## Problem

SAF has a full incremental analysis pipeline (`run_pipeline_incremental`) with per-module
constraint caching, SILVA-style diff-based PTA, incremental CG refinement, and selective
value-flow rebuild. However, there are only unit tests for individual components — no E2E
tests that exercise the full pipeline across two runs on real codebases. We need to validate
that incremental analysis actually delivers speedups on large projects.

## Goals

1. Prove incremental analysis works end-to-end on real C projects
2. Measure speedup for two change scenarios: leaf edit (best case) and core edit (worst case)
3. Validate correctness: incremental results match fresh full analysis
4. Collect multi-level metrics: wall-clock time, pipeline stage times, constraint-level diffs

## Non-Goals

- Testing incremental analysis on synthetic/generated code
- Benchmarking against other tools (SVF, Infer, etc.)
- Optimizing the incremental pipeline itself (this is validation only)

## Prerequisites

The following changes are required before implementing the tests:

1. **Add `pta_iterations: usize` to `PipelineStats`** — currently `PipelineStats` has
   `pta_solve_secs` and `refinement_iterations` (CG refinement waves, typically 1-10)
   but no **inner PTA solver worklist iteration count** (typically thousands to tens of
   thousands). `IncrementalResult.iterations` is consumed locally in `run_incremental_path`
   and never surfaced. Thread the iteration count into `PipelineStats`:
   - Full path: source from `PtaResult.diagnostics().iterations`
   - Incremental path: source from `IncrementalResult.iterations`

2. **Expose constraint diff summary in `PipelineStats`** — add
   `constraint_diff_added: usize`, `constraint_diff_removed: usize`, and
   `changed_module_count: usize` fields. Currently the `ConstraintDiff` is computed inside
   `run_incremental_path` and logged but not returned. These fields are 0 on first (full)
   run and populated on incremental runs.

3. **Lua and CPython patch files** — the exact source edits for each scenario must be
   authored and tested. See the "Change Specifications" section for details.

## Design

### Test Projects

| Project | Size | Role | Where |
|---------|------|------|-------|
| Lua 5.4.7 | ~35 `.c` files, ~30K lines | Small, fast, deterministic | Rust integration test (`make test`) |
| CPython v3.13.0 | ~350 `.c` files, ~400K lines | Large, real-world scalability proof | Benchmark harness (`make bench-incremental`) |

### Change Scenarios

Each project is tested with two edit scenarios:

1. **Leaf edit** — modify one function body in a peripheral module. Minimal invalidation
   cascade. Exercises the best-case incremental path.
   - Lua: modify `math_abs` in `lmathlib.c`
   - CPython: modify `builtin_len` in `Python/bltinmodule.c`

2. **Core edit** — modify a widely-called function in a core module. Large invalidation
   cascade but still faster than full re-analysis.
   - Lua: modify `luaO_str2num` in `lobject.c`
   - CPython: modify `_Py_Dealloc` in `Objects/object.c`

### Change Specifications

Edits must create meaningful constraint diffs (not just constant changes). Each patch file
adds or modifies pointer operations to ensure the PTA constraint set actually changes.

**Lua leaf edit** (`lmathlib.c` → `lmathlib_v2.c`):
- Add a local pointer variable and an allocation call inside `math_abs`
- Expected diff: ~2-5 new addr/copy constraints, 0 removed

**Lua core edit** (`lobject.c` → `lobject_v2.c`):
- Add a pointer parameter and indirect call in `luaO_str2num`
- Expected diff: ~10-20 new constraints across addr/copy/load, some removed

**CPython leaf edit** (`Python/bltinmodule.c`):
- Add a local buffer allocation inside `builtin_len`
- Expected diff: small, localized to one function

**CPython core edit** (`Objects/object.c`):
- Add a pointer indirection in `_Py_Dealloc`
- Expected diff: large, cascading through many callers

Patch files are stored in `tests/benchmarks/incremental/patches/` (for CPython) and
`tests/fixtures/incremental/lua/patches/` (for Lua). The Lua patches are also used to
generate the `*_v2.ll` fixtures.

### Part 1: Lua Rust Integration Test

**File:** `crates/saf-analysis/tests/incremental_e2e.rs`

**Fixtures:** `tests/fixtures/incremental/lua/`
- All ~35 Lua `.c` files pre-compiled to `.ll` (pinned to Lua 5.4.7)
- Two modified variants: `lmathlib_v2.ll` (leaf edit), `lobject_v2.ll` (core edit)
- Compiled with `clang -S -emit-llvm -O0` (no `-g` to reduce fixture size; debug info
  is not needed for constraint extraction). Estimated total size: ~3-6MB.

**Test flow:**

```
1. Load all .ll files → AirProgram (multi-module)
2. Create AnalysisSession with temp cache dir
3. Run run_pipeline_incremental (first run — full analysis)
4. Record: wall-clock, PTA iterations, constraint counts
5. Swap lmathlib.ll → lmathlib_v2.ll, rebuild AirProgram
6. Run run_pipeline_incremental (second run — incremental, same session object)
7. Assert:
   a. Incremental path was taken (session.run_count == 2)
   b. Constraint diff is small (only 1 module changed, diff.changed_module_count == 1)
   c. Constraint diff is non-trivial (diff.added.total_count() > 0)
   d. PTA iterations << first run iterations
   e. PTA results match fresh full analysis on v2 code
8. Run determinism check: run_pipeline_incremental again with identical v2 input,
   verify PipelineStats (non-timing fields) and PtaResult are identical
9. Repeat steps 5-8 for lobject_v2.ll (core edit)
   a. Constraint diff is larger (diff.added.total_count() > leaf edit's)
   b. Still faster than full re-analysis (pta_solve_secs < full run's)
   c. PTA results still match fresh full analysis
```

Note: both runs use the **same `AnalysisSession` instance** in-process, which is required
for the incremental path. The in-memory fields (`incremental_pta_state`,
`previous_call_graph`, `previous_pta_result`, etc.) are `#[serde(skip)]` and would be
lost across process restarts.

**Assertions:**
- Correctness: `incremental_pta_result.points_to_map() == fresh_full_pta_result.points_to_map()`
  and `incremental_cg.edges == fresh_cg.edges` (direct field comparison on `BTreeMap`).
  Note: for edits that only add constraints (no removals), strict equality is expected.
  For edits with removals, incremental may over-approximate due to removal debt — use
  superset check (`incremental_pts ⊇ fresh_pts`) and verify `removal_debt < threshold`.
- Efficiency: `run2_stats.pta_iterations < run1_stats.pta_iterations * 0.5` for leaf edit
- Constraint diff: `run2_stats.changed_module_count == 1` and
  `run2_stats.constraint_diff_added > 0` (sourced from new `PipelineStats` fields)
- Convergence: `incremental_result.converged == true`
- Determinism: repeated incremental run produces identical non-timing stats and PTA results

**Scenario independence:** Steps 5-8 (leaf edit) and step 9 (core edit) should each start
from a fresh session to ensure independent measurements. Create a new `AnalysisSession`,
run a full first pass, then the incremental second pass for each scenario separately.

**Timeout:** This test may take 30-60s with two pipeline runs per scenario. Use the
`#[ignore]` attribute and run via `make test` which includes ignored tests, or add a
dedicated `make test-incremental` target.

### Part 2: CPython Benchmark Harness

**Architecture:** A Rust benchmark binary in `saf-bench` (alongside the existing PTABen
harness), not separate CLI invocations. This is required because the incremental state
(`incremental_pta_state`, `previous_call_graph`, etc.) is in-memory only — `AnalysisSession`
serialization via `#[serde(skip)]` drops these fields, so two separate `saf incremental`
process invocations would both take the full analysis path.

**Makefile target:** `make bench-incremental`

**Workflow:**

1. **Clone** CPython at pinned commit (`v3.13.0`) into
   `tests/benchmarks/incremental/cpython/` (gitignored)
2. **Compile** all ~350 `.c` files to `.ll` inside Docker using
   `clang -S -emit-llvm -O0` via `scripts/compile-incremental-bench.sh`
3. **Run benchmark harness** inside Docker:
   ```
   cargo run --release -p saf-bench -- incremental \
     --compiled-dir tests/benchmarks/incremental/cpython/.compiled \
     --patches-dir tests/benchmarks/incremental/patches \
     -o tests/benchmarks/incremental/results.json
   ```
4. The harness performs for each scenario (leaf edit, core edit):
   a. Load all `.ll` files → `AirProgram`
   b. Create `AnalysisSession` with temp cache dir
   c. Run `run_pipeline_incremental` (full, first run) — record metrics
   d. Swap the patched module's `.ll` file, rebuild `AirProgram`
   e. Run `run_pipeline_incremental` (incremental, same session) — record metrics
   f. Run fresh full analysis on v2 for result equivalence (optional, `--verify` flag)
   g. Write scenario metrics to JSON
5. **Report** structured JSON to `tests/benchmarks/incremental/results.json`

**JSON output format:**

```json
{
  "project": "cpython",
  "version": "v3.13.0",
  "total_modules": 350,
  "scenarios": [
    {
      "name": "leaf_edit",
      "changed_files": 1,
      "full_run_secs": 45.2,
      "incremental_run_secs": 8.1,
      "speedup": "5.6x",
      "stages": {
        "full": { "pta_solve_secs": 30.1, "cg_refinement_secs": 2.3, "vf_build_secs": 10.5 },
        "incremental": { "pta_solve_secs": 3.2, "cg_refinement_secs": 0.8, "vf_build_secs": 3.1 }
      },
      "pta_iterations_full": 12340,
      "pta_iterations_incr": 890,
      "constraint_diff": { "added": 12, "removed": 10, "changed_modules": 1 },
      "results_match": true
    },
    {
      "name": "core_edit",
      "changed_files": 1,
      "full_run_secs": 45.2,
      "incremental_run_secs": 22.4,
      "speedup": "2.0x",
      "stages": {
        "full": { "pta_solve_secs": 30.1, "cg_refinement_secs": 2.3, "vf_build_secs": 10.5 },
        "incremental": { "pta_solve_secs": 15.0, "cg_refinement_secs": 1.8, "vf_build_secs": 4.2 }
      },
      "pta_iterations_full": 12340,
      "pta_iterations_incr": 6200,
      "constraint_diff": { "added": 450, "removed": 420, "changed_modules": 1 },
      "results_match": true
    }
  ]
}
```

**Notes:**
- Not part of `make test` — run manually or in CI nightly
- Patches pinned to the exact CPython commit
- Fresh comparison run is optional (`--verify` flag) since correctness is validated by
  the Lua integration test; the CPython benchmark focuses on scalability metrics
- Uses `run_in_background: true` and `--release` build since CPython analysis may take minutes

### Part 3: Shared Infrastructure

**Compile scripts:**
- `scripts/compile-lua-fixtures.sh` — one-time script to compile Lua sources to `.ll`
  fixtures. Downloads Lua 5.4.7 tarball, compiles inside Docker with
  `clang -S -emit-llvm -O0`, outputs to `tests/fixtures/incremental/lua/`. Also applies
  patches and compiles `*_v2.ll` variants.
- `scripts/compile-incremental-bench.sh` — parameterized script for clone + compile.
  Called by `make bench-incremental`. Handles: git clone at pinned commit, compile all
  `.c` → `.ll`, apply patches and recompile changed files.

**Metrics collection:**
- Rust test: extract metrics directly from `PipelineResult.stats` (needs new
  `pta_iterations` field; existing `pta_solve_secs`, `valueflow_build_secs`, `total_secs`
  are already available)
- CPython benchmark: the `saf-bench incremental` harness collects metrics directly from
  `PipelineResult` in-process — no CLI flag needed for this path.

**Future work (not required for this plan):**
- `--metrics-output <path>` flag on `saf incremental` CLI for ad-hoc user workflows

**Fixture management:**
- Lua `.ll` files checked into repo (~35 files, ~3-6MB estimated without debug info)
- CPython `.ll` files gitignored, generated on-demand
- `README.md` in `tests/fixtures/incremental/` documenting how to refresh fixtures
- Measure actual fixture size after initial compilation; reconsider if >10MB

**Makefile targets:**
- `make compile-lua-fixtures` — regenerate Lua `.ll` fixtures inside Docker
- `make bench-incremental` — full CPython benchmark workflow
- `make bench-incremental PROJECT=lua` — benchmark-style test against Lua for quick validation
- `make test-incremental` — (optional) run just the Lua Rust integration test

### Directory Layout

```
tests/
  fixtures/
    incremental/
      lua/
        *.ll                 (35 files, checked in)
        lmathlib_v2.ll       (leaf edit variant, checked in)
        lobject_v2.ll        (core edit variant, checked in)
        patches/
          lmathlib-leaf.patch
          lobject-core.patch
        README.md
  benchmarks/
    incremental/
      cpython/               (gitignored, cloned on-demand)
      patches/
        cpython-leaf-edit.patch
        cpython-core-edit.patch
      results.json           (gitignored, generated)
scripts/
  compile-lua-fixtures.sh
  compile-incremental-bench.sh
crates/
  saf-analysis/
    tests/
      incremental_e2e.rs     (Rust integration test)
  saf-bench/
    src/
      incremental.rs         (benchmark harness module)
```

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Lua/CPython update breaks fixtures | Pin to exact version; `compile-lua-fixtures.sh` makes refresh easy |
| CPython analysis is too slow or OOMs | Set timeout; use `--release` build; can fall back to a subset of files |
| Incremental PTA removal debt causes over-approximation | Fresh comparison validates equivalence; document known imprecision |
| `.ll` fixtures bloat repo | Compile without `-g`; measure actual size; CPython is on-demand only |
| Patch files break on version update | Patches pinned to exact commit; updating version = updating patches |
| Cross-process state loss | Benchmark harness runs both passes in single process; documented in spec |
| Lua integration test is too slow for `make test` | Use `#[ignore]` or dedicated `make test-incremental` target |
| Fixture size exceeds estimate | Measure after compilation; strip debug info; reconsider if >10MB |
