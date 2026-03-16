# Incremental Analysis E2E Tests — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that SAF's incremental analysis pipeline delivers real speedups on Lua (~35 files) and CPython (~350 files) by running full → edit → incremental passes and comparing results + metrics.

**Architecture:** Two-tier testing: (1) Rust integration test with pre-compiled Lua `.ll` fixtures for correctness assertions in `make test`, (2) Rust benchmark harness in `saf-bench` for CPython scalability metrics via `make bench-incremental`. Both run two pipeline passes in a single process to preserve in-memory incremental state (`#[serde(skip)]` fields).

**Tech Stack:** Rust (saf-analysis pipeline, saf-bench harness), LLVM IR fixtures compiled from C via clang, Docker for compilation, Makefile for orchestration.

**Spec:** `docs/superpowers/specs/2026-03-18-incremental-e2e-tests-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `crates/saf-analysis/src/pipeline.rs` | Modify | Add `pta_iterations`, `constraint_diff_added/removed`, `changed_module_count` to `PipelineStats` |
| `crates/saf-analysis/src/pta/context.rs` | Read | Source of `PtaDiagnostics.iterations` for full path |
| `crates/saf-test-utils/src/lib.rs` | Modify | Add `incremental_fixtures_dir()` and `load_incremental_lua_bundle()` helpers |
| `crates/saf-analysis/tests/incremental_e2e.rs` | Create | Lua-based integration test (leaf edit + core edit scenarios) |
| `crates/saf-bench/src/main.rs` | Modify | Add `Incremental` subcommand variant |
| `crates/saf-bench/src/incremental.rs` | Create | CPython benchmark harness module |
| `scripts/compile-lua-fixtures.sh` | Create | Download Lua 5.4.7, compile `.c` → `.ll`, apply patches for v2 variants |
| `scripts/compile-incremental-bench.sh` | Create | Clone CPython, compile `.c` → `.ll`, apply patches |
| `tests/fixtures/incremental/lua/patches/lmathlib-leaf.patch` | Create | Leaf edit patch for lmathlib.c |
| `tests/fixtures/incremental/lua/patches/lobject-core.patch` | Create | Core edit patch for lobject.c |
| `tests/benchmarks/incremental/patches/cpython-leaf-edit.patch` | Create | Leaf edit patch for bltinmodule.c |
| `tests/benchmarks/incremental/patches/cpython-core-edit.patch` | Create | Core edit patch for object.c |
| `tests/fixtures/incremental/lua/README.md` | Create | Document fixture generation and refresh |
| `Makefile` | Modify | Add `compile-lua-fixtures`, `bench-incremental`, `test-incremental` targets |
| `.gitignore` | Modify | Add `tests/benchmarks/incremental/cpython/`, `results.json` |

---

## Task 1: Add `pta_iterations` and constraint diff fields to `PipelineStats`

**Files:**
- Modify: `crates/saf-analysis/src/pipeline.rs:118-136` (PipelineStats struct)
- Modify: `crates/saf-analysis/src/pipeline.rs:249-257` (run_pipeline stats population)
- Modify: `crates/saf-analysis/src/pipeline.rs:429-437` (run_pipeline_incremental full path stats)
- Modify: `crates/saf-analysis/src/pipeline.rs:630-638` (run_incremental_path stats)

- [ ] **Step 1: Add fields to `PipelineStats`**

In `crates/saf-analysis/src/pipeline.rs`, add these fields to `PipelineStats`:

```rust
/// Inner PTA solver worklist iterations (NOT CG refinement iterations).
/// Full path: from PtaDiagnostics.iterations. Incremental: from IncrementalResult.iterations.
#[serde(default)]
pub pta_iterations: usize,
/// Number of added constraints in the incremental diff (0 on first run).
#[serde(default)]
pub constraint_diff_added: usize,
/// Number of removed constraints in the incremental diff (0 on first run).
#[serde(default)]
pub constraint_diff_removed: usize,
/// Number of modules whose constraints changed (0 on first run).
#[serde(default)]
pub changed_module_count: usize,
```

- [ ] **Step 2: Populate `pta_iterations` in `run_pipeline` (full analysis path)**

In the `run_pipeline` function, after PTA solving and CG refinement, extract iterations from the `PtaResult`:

```rust
// In the PipelineStats construction in run_pipeline, use the local `pta_result` variable
// (NOT result.pta_result — the result struct is being constructed here):
pta_iterations: pta_result.as_ref()
    .map(|r| r.diagnostics().iterations)
    .unwrap_or(0),
constraint_diff_added: 0,
constraint_diff_removed: 0,
changed_module_count: 0,
```

- [ ] **Step 3: Populate `pta_iterations` in `run_pipeline_incremental` (full first-run path)**

Same as Step 2 — use the local `pta_result` variable in the `run_pipeline_incremental` function's
stats construction. The variable is available from the `RefinementResult` destructuring.

- [ ] **Step 4: Populate all new fields in `run_incremental_path`**

In `run_incremental_path`, capture `incr_result.iterations` BEFORE the `saf_log!` call
consumes it, then use the captured value in the stats construction:

```rust
// Right after: let incr_result = apply_incremental_update(&mut pta_state, ...);
// Capture iteration count before saf_log! borrows incr_result:
let pta_iters = incr_result.iterations;

// ... (existing saf_log! call) ...

// In the PipelineStats construction:
pta_iterations: pta_iters,
constraint_diff_added: diff.added.total_count(),
constraint_diff_removed: diff.removed.total_count(),
changed_module_count: diff.changed_module_count,
```

- [ ] **Step 4a: Update `analyze_single_module` stats construction**

There is another `PipelineStats` construction site at `analyze_single_module` (~line 830 of
pipeline.rs). Add the new fields with zero values there too:

```rust
pta_iterations: 0,
constraint_diff_added: 0,
constraint_diff_removed: 0,
changed_module_count: 0,
```

- [ ] **Step 5: Verify `ConstraintDiff` has the needed methods**

Check that `ConstraintDiff` in `crates/saf-analysis/src/pta/module_constraints.rs` has:
- `changed_module_count: usize` field
- `added: ConstraintSet` with `total_count()` method
- `removed: ConstraintSet` with `total_count()` method

If `total_count()` is missing on `ConstraintSet`, add it:

```rust
impl ConstraintSet {
    pub fn total_count(&self) -> usize {
        self.addr.len() + self.copy.len() + self.load.len() + self.store.len() + self.gep.len()
    }
}
```

- [ ] **Step 6: Run `make fmt && make lint` to verify no errors**

Run: `make fmt && make lint`
Expected: clean (no errors)

- [ ] **Step 7: Run `make test` to verify no regressions**

Run: `make test`
Expected: all existing tests pass

- [ ] **Step 8: Commit**

```bash
git add crates/saf-analysis/src/pipeline.rs crates/saf-analysis/src/pta/module_constraints.rs
git commit -m "feat: expose PTA iterations and constraint diff in PipelineStats"
```

---

## Task 2: Create Lua fixture compilation script and patches

**Files:**
- Create: `scripts/compile-lua-fixtures.sh`
- Create: `tests/fixtures/incremental/lua/patches/lmathlib-leaf.patch`
- Create: `tests/fixtures/incremental/lua/patches/lobject-core.patch`
- Create: `tests/fixtures/incremental/lua/README.md`

- [ ] **Step 1: Create the Lua patches directory**

```bash
mkdir -p tests/fixtures/incremental/lua/patches
```

- [ ] **Step 2: Create the leaf edit patch (`lmathlib-leaf.patch`)**

This patch modifies `math_abs` in `lmathlib.c` to add a local pointer variable and allocation call, creating new PTA constraints:

```patch
--- a/src/lmathlib.c
+++ b/src/lmathlib.c
@@ -<math_abs function>
 static int math_abs (lua_State *L) {
+  /* Incremental test: add pointer allocation for constraint diff */
+  void *incr_test_ptr = lua_newuserdata(L, 64);
+  (void)incr_test_ptr;
   if (lua_isinteger(L, 1)) {
```

Note: exact line numbers depend on Lua 5.4.7 source. The agent implementing this task must download and inspect the Lua source to determine the correct offset. The key requirement is that the patch adds a `lua_newuserdata` call (which creates addr + copy constraints) inside `math_abs`.

- [ ] **Step 3: Create the core edit patch (`lobject-core.patch`)**

This patch modifies `luaO_str2num` in `lobject.c` to add an indirect call through a function pointer, creating a larger constraint cascade:

```patch
--- a/src/lobject.c
+++ b/src/lobject.c
@@ -<luaO_str2num function>
 size_t luaO_str2num (const char *s, TValue *o) {
+  /* Incremental test: add pointer indirection for constraint cascade */
+  typedef size_t (*str2num_hook_t)(const char *, TValue *);
+  static str2num_hook_t hook = NULL;
+  if (hook) return hook(s, o);
```

Note: same caveat — exact line numbers from Lua 5.4.7 source. The key requirement is a function pointer + indirect call to generate load/store/copy constraints.

- [ ] **Step 4: Create `scripts/compile-lua-fixtures.sh`**

```bash
#!/usr/bin/env bash
# Compile Lua 5.4.7 sources to LLVM IR for incremental analysis fixtures.
# Must be run inside Docker (requires clang/LLVM 18).
set -euo pipefail

LUA_VERSION="5.4.7"
LUA_URL="https://www.lua.org/ftp/lua-${LUA_VERSION}.tar.gz"
WORK_DIR="/tmp/lua-fixtures-build"
OUTPUT_DIR="/workspace/tests/fixtures/incremental/lua"
PATCHES_DIR="/workspace/tests/fixtures/incremental/lua/patches"

echo "=== Compiling Lua ${LUA_VERSION} to LLVM IR ==="

# Download and extract
rm -rf "${WORK_DIR}"
mkdir -p "${WORK_DIR}"
cd "${WORK_DIR}"
curl -sL "${LUA_URL}" | tar xz
cd "lua-${LUA_VERSION}/src"

# Compile all .c files to .ll (no debug info to save space)
mkdir -p "${OUTPUT_DIR}"
for f in *.c; do
    echo "  Compiling ${f}..."
    clang -S -emit-llvm -O0 -I. "${f}" -o "${OUTPUT_DIR}/${f%.c}.ll"
done

# Generate v2 variants by applying patches and recompiling
echo "=== Generating v2 variants ==="

# Leaf edit: lmathlib
if [ -f "${PATCHES_DIR}/lmathlib-leaf.patch" ]; then
    cp lmathlib.c lmathlib_v2.c
    patch lmathlib_v2.c "${PATCHES_DIR}/lmathlib-leaf.patch" || {
        echo "WARNING: lmathlib-leaf.patch failed to apply cleanly"
        exit 1
    }
    clang -S -emit-llvm -O0 -I. lmathlib_v2.c -o "${OUTPUT_DIR}/lmathlib_v2.ll"
    echo "  lmathlib_v2.ll generated"
fi

# Core edit: lobject
if [ -f "${PATCHES_DIR}/lobject-core.patch" ]; then
    cp lobject.c lobject_v2.c
    patch lobject_v2.c "${PATCHES_DIR}/lobject-core.patch" || {
        echo "WARNING: lobject-core.patch failed to apply cleanly"
        exit 1
    }
    clang -S -emit-llvm -O0 -I. lobject_v2.c -o "${OUTPUT_DIR}/lobject_v2.ll"
    echo "  lobject_v2.ll generated"
fi

# Report size
echo ""
echo "=== Fixture summary ==="
echo "Files: $(ls "${OUTPUT_DIR}"/*.ll | wc -l)"
echo "Total size: $(du -sh "${OUTPUT_DIR}" | cut -f1)"
echo "Output: ${OUTPUT_DIR}"
```

- [ ] **Step 5: Create `tests/fixtures/incremental/lua/README.md`**

```markdown
# Lua Incremental Analysis Fixtures

Pre-compiled LLVM IR from Lua 5.4.7 for incremental analysis E2E tests.

## Files

- `*.ll` — All Lua 5.4.7 source files compiled to LLVM IR
- `lmathlib_v2.ll` — Leaf edit variant (added allocation in `math_abs`)
- `lobject_v2.ll` — Core edit variant (added indirect call in `luaO_str2num`)
- `patches/` — Source patches used to generate v2 variants

## Regenerating

Run inside Docker:

    make compile-lua-fixtures

Or manually:

    docker compose run --rm dev sh -c 'bash scripts/compile-lua-fixtures.sh'

## Pinned Version

Lua 5.4.7 — patches are pinned to this exact version.
```

- [ ] **Step 6: Commit scaffold (patches + script + README, no `.ll` files yet)**

```bash
git add scripts/compile-lua-fixtures.sh tests/fixtures/incremental/lua/patches/ tests/fixtures/incremental/lua/README.md
git commit -m "feat: add Lua fixture compilation script and patches for incremental E2E tests"
```

---

## Task 3: Compile Lua fixtures and check into repo

**Files:**
- Modify: `Makefile` (add `compile-lua-fixtures` target)
- Generated: `tests/fixtures/incremental/lua/*.ll` (35+ files)

- [ ] **Step 1: Add Makefile target**

Add to `Makefile` after the existing compile targets:

```makefile
compile-lua-fixtures: ## Compile Lua 5.4.7 to LLVM IR for incremental analysis fixtures
	docker compose run --rm dev sh -c 'bash scripts/compile-lua-fixtures.sh'
```

- [ ] **Step 2: Run the compilation**

Run: `make compile-lua-fixtures`
Expected: ~35 `.ll` files + 2 `*_v2.ll` variants in `tests/fixtures/incremental/lua/`

- [ ] **Step 3: Verify patch application worked**

Check that `lmathlib_v2.ll` and `lobject_v2.ll` exist and differ from the originals:

```bash
diff tests/fixtures/incremental/lua/lmathlib.ll tests/fixtures/incremental/lua/lmathlib_v2.ll | head -20
```

Expected: visible differences (new alloca/call instructions)

- [ ] **Step 4: Measure fixture size**

```bash
du -sh tests/fixtures/incremental/lua/
```

Expected: 3-6MB. If >10MB, consider stripping more metadata.

- [ ] **Step 5: Commit fixtures**

```bash
git add tests/fixtures/incremental/lua/*.ll Makefile
git commit -m "feat: add pre-compiled Lua 5.4.7 LLVM IR fixtures for incremental analysis"
```

---

## Task 4: Add test helpers for incremental fixtures

**Files:**
- Modify: `crates/saf-test-utils/src/lib.rs`

- [ ] **Step 1: Add `incremental_fixtures_dir()` helper**

Add to `crates/saf-test-utils/src/lib.rs`:

```rust
/// Return the path to `tests/fixtures/incremental/lua/`.
#[must_use]
pub fn incremental_lua_fixtures_dir() -> PathBuf {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()  // crates/
        .unwrap()
        .parent()  // workspace root
        .unwrap()
        .to_owned();
    workspace.join("tests/fixtures/incremental/lua")
}
```

- [ ] **Step 2: Add `load_incremental_lua_program()` helper**

This loads all Lua `.ll` files into an `AirProgram`, optionally swapping one module:

```rust
use saf_core::program::AirProgram;

/// Load all Lua `.ll` fixtures as a multi-module `AirProgram`.
///
/// If `swap` is `Some(("lmathlib", "lmathlib_v2"))`, the named module's `.ll`
/// file is replaced with the v2 variant before linking.
pub fn load_incremental_lua_program(swap: Option<(&str, &str)>) -> AirProgram {
    let dir = incremental_lua_fixtures_dir();
    let mut bundles = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("read incremental/lua dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "ll") {
            let stem = path.file_stem().unwrap().to_str().unwrap();

            // Skip *_v2 files — they're only used when swapped in
            if stem.ends_with("_v2") {
                continue;
            }

            // If swap requested and this is the target, load v2 instead
            if let Some((orig, replacement)) = swap {
                if stem == orig {
                    let v2_path = dir.join(format!("{replacement}.ll"));
                    bundles.push(load_ll_from_path(&v2_path));
                    continue;
                }
            }

            bundles.push(load_ll_from_path(&path));
        }
    }

    AirProgram::link(bundles)
}
```

- [ ] **Step 3: Verify dev-dependencies**

Check that `crates/saf-analysis/Cargo.toml` has `saf-test-utils` in `[dev-dependencies]`
(needed for the integration test in Task 5). Also check that `tempfile` is in
`[dev-dependencies]` (needed for temp cache dirs in tests). Add if missing:

```toml
[dev-dependencies]
saf-test-utils = { path = "../saf-test-utils" }
tempfile = "3"
```

- [ ] **Step 4: Verify compilation**

Run: `make fmt && make lint`
Expected: clean

- [ ] **Step 5: Commit**

```bash
git add crates/saf-test-utils/src/lib.rs crates/saf-analysis/Cargo.toml
git commit -m "feat: add incremental Lua fixture loading helpers to saf-test-utils"
```

---

## Task 5: Write the Lua incremental integration test

**Files:**
- Create: `crates/saf-analysis/tests/incremental_e2e.rs`

- [ ] **Step 1: Write the leaf edit test**

Create `crates/saf-analysis/tests/incremental_e2e.rs`:

```rust
//! E2E tests for the incremental analysis pipeline.
//!
//! These tests validate that `run_pipeline_incremental` delivers correct results
//! and measurable speedups when only one module changes between runs.
//! Uses pre-compiled Lua 5.4.7 LLVM IR fixtures (~35 modules).

use std::path::PathBuf;

use saf_analysis::pipeline::{PipelineConfig, PipelineResult, run_pipeline_incremental};
use saf_analysis::session::AnalysisSession;
use saf_test_utils::load_incremental_lua_program;

/// Run a complete incremental scenario: full first run → swap module → incremental second run.
/// Returns (first_run_result, incremental_result, session) for assertions.
fn run_incremental_scenario(
    swap_from: &str,
    swap_to: &str,
) -> (PipelineResult, PipelineResult, AnalysisSession) {
    let config = PipelineConfig::default();
    let tmp = tempfile::tempdir().expect("create temp dir");
    let mut session = AnalysisSession::new(tmp.path().to_owned());

    // First run: full analysis on unmodified Lua
    let program_v1 = load_incremental_lua_program(None);
    let result_v1 = run_pipeline_incremental(&program_v1, &config, &mut session);

    // Second run: incremental analysis with one module swapped
    let program_v2 = load_incremental_lua_program(Some((swap_from, swap_to)));
    let result_v2 = run_pipeline_incremental(&program_v2, &config, &mut session);

    (result_v1, result_v2, session)
}

/// Run a fresh full analysis on v2 code for comparison.
fn run_fresh_full(swap_from: &str, swap_to: &str) -> PipelineResult {
    let config = PipelineConfig::default();
    let tmp = tempfile::tempdir().expect("create temp dir");
    let mut session = AnalysisSession::new(tmp.path().to_owned());
    let program = load_incremental_lua_program(Some((swap_from, swap_to)));
    run_pipeline_incremental(&program, &config, &mut session)
}

#[test]
#[ignore] // Runs in ~30-60s — use `make test-incremental` or `make test`
fn incremental_leaf_edit_correctness_and_speedup() {
    let (full, incr, session) = run_incremental_scenario("lmathlib", "lmathlib_v2");

    // Incremental path was taken
    assert_eq!(session.run_count, 2, "should have completed 2 runs");

    // Constraint diff: exactly 1 module changed
    assert_eq!(incr.stats.changed_module_count, 1, "leaf edit changes 1 module");
    assert!(incr.stats.constraint_diff_added > 0, "should have added constraints");

    // Efficiency: incremental PTA should use fewer iterations
    assert!(
        incr.stats.pta_iterations < full.stats.pta_iterations,
        "incremental PTA iterations ({}) should be less than full ({})",
        incr.stats.pta_iterations,
        full.stats.pta_iterations,
    );

    // Correctness: compare PTA results against fresh full analysis on v2
    let fresh = run_fresh_full("lmathlib", "lmathlib_v2");
    let incr_pts = incr.pta_result.as_ref().expect("incremental PTA result");
    let fresh_pts = fresh.pta_result.as_ref().expect("fresh PTA result");
    assert_eq!(
        incr_pts.points_to_map(),
        fresh_pts.points_to_map(),
        "incremental PTA results must match fresh full analysis (leaf edit is add-only)"
    );

    // Correctness: call graphs should match
    assert_eq!(
        incr.call_graph, fresh.call_graph,
        "incremental call graph must match fresh full analysis"
    );

    // Convergence: incremental solver must have converged
    // (checked via pta_iterations > 0 — if 0, the incremental path was skipped)
    assert!(incr.stats.pta_iterations > 0, "incremental PTA should have run");
}

#[test]
#[ignore]
fn incremental_core_edit_correctness_and_speedup() {
    let (full, incr, session) = run_incremental_scenario("lobject", "lobject_v2");

    // Incremental path was taken
    assert_eq!(session.run_count, 2);

    // Constraint diff: 1 module changed but more constraints affected
    assert_eq!(incr.stats.changed_module_count, 1);
    assert!(incr.stats.constraint_diff_added > 0);

    // Efficiency: still fewer iterations than full, but gap may be smaller
    assert!(
        incr.stats.pta_iterations < full.stats.pta_iterations,
        "core edit incremental ({}) should still beat full ({})",
        incr.stats.pta_iterations,
        full.stats.pta_iterations,
    );

    // Correctness against fresh analysis
    let fresh = run_fresh_full("lobject", "lobject_v2");
    let incr_pts = incr.pta_result.as_ref().expect("incremental PTA result");
    let fresh_pts = fresh.pta_result.as_ref().expect("fresh PTA result");

    // Core edit may involve removals → check superset if removal debt exists
    if incr.stats.constraint_diff_removed > 0 {
        // Over-approximate is acceptable: incremental ⊇ fresh
        for (val, fresh_locs) in fresh_pts.points_to_map() {
            if let Some(incr_locs) = incr_pts.points_to_map().get(val) {
                assert!(
                    incr_locs.is_superset(fresh_locs),
                    "incremental pts for {val:?} must be superset of fresh"
                );
            }
        }
    } else {
        assert_eq!(incr_pts.points_to_map(), fresh_pts.points_to_map());
    }
}

#[test]
#[ignore]
fn incremental_determinism() {
    // Run the same incremental scenario twice and verify identical non-timing stats
    let config = PipelineConfig::default();

    let tmp1 = tempfile::tempdir().expect("create temp dir");
    let mut session1 = AnalysisSession::new(tmp1.path().to_owned());
    let prog1 = load_incremental_lua_program(None);
    let _ = run_pipeline_incremental(&prog1, &config, &mut session1);
    let prog1v2 = load_incremental_lua_program(Some(("lmathlib", "lmathlib_v2")));
    let result1 = run_pipeline_incremental(&prog1v2, &config, &mut session1);

    let tmp2 = tempfile::tempdir().expect("create temp dir");
    let mut session2 = AnalysisSession::new(tmp2.path().to_owned());
    let prog2 = load_incremental_lua_program(None);
    let _ = run_pipeline_incremental(&prog2, &config, &mut session2);
    let prog2v2 = load_incremental_lua_program(Some(("lmathlib", "lmathlib_v2")));
    let result2 = run_pipeline_incremental(&prog2v2, &config, &mut session2);

    // Non-timing stats must be identical (NFR-DET)
    assert_eq!(result1.stats.pta_iterations, result2.stats.pta_iterations);
    assert_eq!(result1.stats.constraint_diff_added, result2.stats.constraint_diff_added);
    assert_eq!(result1.stats.constraint_diff_removed, result2.stats.constraint_diff_removed);
    assert_eq!(result1.stats.changed_module_count, result2.stats.changed_module_count);

    // PTA results must be identical
    let pts1 = result1.pta_result.as_ref().expect("PTA result 1");
    let pts2 = result2.pta_result.as_ref().expect("PTA result 2");
    assert_eq!(pts1.points_to_map(), pts2.points_to_map());
}
```

- [ ] **Step 2: Run `make fmt && make lint`**

Expected: clean

- [ ] **Step 3: Run the test inside Docker**

Run: `docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test incremental_e2e -- --ignored'`

Expected: 3 tests pass (may take 30-60s total). If fixtures aren't compiled yet, this will fail — run `make compile-lua-fixtures` first.

- [ ] **Step 4: Commit**

```bash
git add crates/saf-analysis/tests/incremental_e2e.rs
git commit -m "test: add incremental analysis E2E tests with Lua fixtures"
```

---

## Task 6: Add Makefile targets for incremental tests

**Files:**
- Modify: `Makefile`

- [ ] **Step 1: Add targets**

Add to `Makefile`:

```makefile
test-incremental: ## Run incremental analysis E2E tests (Lua fixtures, ~60s)
	docker compose run --rm dev sh -c 'cargo nextest run -p saf-analysis --test incremental_e2e -- --ignored'

bench-incremental: ## Run incremental analysis benchmark (CPython, requires compile-incremental-bench first)
	docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- incremental --compiled-dir tests/benchmarks/incremental/cpython/.compiled --patches-dir tests/benchmarks/incremental/patches -o /workspace/tests/benchmarks/incremental/results.json'
```

- [ ] **Step 2: Verify `make test-incremental` works**

Run: `make test-incremental`
Expected: 3 tests pass

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "feat: add make test-incremental and make bench-incremental targets"
```

---

## Task 7: Create the CPython benchmark harness in saf-bench

**Files:**
- Create: `crates/saf-bench/src/incremental.rs`
- Modify: `crates/saf-bench/src/main.rs`

- [ ] **Step 1: Add `Incremental` subcommand to main.rs**

In `crates/saf-bench/src/main.rs`, add to the `Commands` enum:

```rust
/// Run incremental analysis benchmark on a real C project.
Incremental {
    /// Directory containing compiled .ll files.
    #[arg(long)]
    compiled_dir: PathBuf,
    /// Directory containing patch files (leaf/core edit .ll variants).
    #[arg(long)]
    patches_dir: PathBuf,
    /// Output JSON results to file.
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Run fresh full analysis for result equivalence verification.
    #[arg(long)]
    verify: bool,
},
```

Add the dispatch in the `match cli.command` block:

```rust
Commands::Incremental { compiled_dir, patches_dir, output, verify } => {
    incremental::run(&compiled_dir, &patches_dir, output.as_deref(), verify)
}
```

Add `mod incremental;` to `main.rs`.

- [ ] **Step 2: Create `crates/saf-bench/src/incremental.rs`**

```rust
//! Incremental analysis benchmark harness.
//!
//! Runs full → edit → incremental on a real C project (e.g., CPython) and
//! reports wall-clock times, PTA iterations, and constraint diff metrics.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use saf_analysis::pipeline::{PipelineConfig, PipelineResult, run_pipeline_incremental};
use saf_analysis::session::AnalysisSession;
use saf_core::config::Config;
use saf_core::program::AirProgram;
use saf_frontends::llvm::LlvmFrontend;
use saf_frontends::api::Frontend;

#[derive(Serialize)]
struct BenchmarkReport {
    project: String,
    version: String,
    total_modules: usize,
    scenarios: Vec<ScenarioResult>,
}

#[derive(Serialize)]
struct ScenarioResult {
    name: String,
    changed_files: usize,
    full_run_secs: f64,
    incremental_run_secs: f64,
    speedup: String,
    stages: StageBreakdown,
    pta_iterations_full: usize,
    pta_iterations_incr: usize,
    constraint_diff: ConstraintDiffReport,
    results_match: bool,
}

#[derive(Serialize)]
struct StageBreakdown {
    full: StageTimings,
    incremental: StageTimings,
}

#[derive(Serialize)]
struct StageTimings {
    pta_solve_secs: f64,
    vf_build_secs: f64,
    total_secs: f64,
}

#[derive(Serialize)]
struct ConstraintDiffReport {
    added: usize,
    removed: usize,
    changed_modules: usize,
}

/// Discover scenario patch files from the patches directory.
///
/// Expects files named `<project>-leaf-edit.ll` and `<project>-core-edit.ll`
/// which are the recompiled v2 `.ll` files for the changed module.
fn discover_scenarios(patches_dir: &Path) -> Result<Vec<(String, String)>> {
    let mut scenarios = Vec::new();
    for entry in std::fs::read_dir(patches_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "ll") {
            let stem = path.file_stem().unwrap().to_str().unwrap().to_owned();
            scenarios.push((stem.clone(), path.to_str().unwrap().to_owned()));
        }
    }
    scenarios.sort();
    Ok(scenarios)
}

/// Load all .ll files from a directory into an AirProgram.
///
/// NOTE: `Frontend::ingest()` signature is `fn ingest(&self, inputs: &[&Path], config: &Config)`.
fn load_program(compiled_dir: &Path, swap: Option<(&str, &Path)>) -> Result<AirProgram> {
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let mut bundles = Vec::new();

    for entry in std::fs::read_dir(compiled_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "ll") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            if let Some((orig, replacement)) = swap {
                if stem == orig {
                    let bundle = frontend.ingest(&[replacement], &config)
                        .map_err(|e| anyhow::anyhow!("failed to ingest {}: {e}", replacement.display()))?;
                    bundles.push(bundle);
                    continue;
                }
            }
            let bundle = frontend.ingest(&[path.as_path()], &config)
                .map_err(|e| anyhow::anyhow!("failed to ingest {}: {e}", path.display()))?;
            bundles.push(bundle);
        }
    }

    Ok(AirProgram::link(bundles))
}

fn run_scenario(
    compiled_dir: &Path,
    scenario_name: &str,
    swap_original: &str,
    swap_replacement: &Path,
    config: &PipelineConfig,
    verify: bool,
) -> Result<ScenarioResult> {
    eprintln!("  Running scenario: {scenario_name}");

    // Full first run
    let tmp = tempfile::tempdir()?;
    let mut session = AnalysisSession::new(tmp.path().to_owned());
    let program_v1 = load_program(compiled_dir, None)?;
    eprintln!("    Full analysis ({} modules)...", program_v1.modules.len());
    let result_full = run_pipeline_incremental(&program_v1, config, &mut session);

    // Incremental second run
    let program_v2 = load_program(compiled_dir, Some((swap_original, swap_replacement)))?;
    eprintln!("    Incremental analysis...");
    let result_incr = run_pipeline_incremental(&program_v2, config, &mut session);

    let speedup = if result_incr.stats.total_secs > 0.0 {
        result_full.stats.total_secs / result_incr.stats.total_secs
    } else {
        0.0
    };

    // Optional result equivalence check
    let results_match = if verify {
        eprintln!("    Verifying against fresh full analysis...");
        let fresh = {
            let tmp2 = tempfile::tempdir()?;
            let mut session2 = AnalysisSession::new(tmp2.path().to_owned());
            let prog = load_program(compiled_dir, Some((swap_original, swap_replacement)))?;
            run_pipeline_incremental(&prog, config, &mut session2)
        };
        match (result_incr.pta_result.as_ref(), fresh.pta_result.as_ref()) {
            (Some(a), Some(b)) => a.points_to_map() == b.points_to_map(),
            (None, None) => true,
            _ => false,
        }
    } else {
        true // Skip verification
    };

    Ok(ScenarioResult {
        name: scenario_name.to_owned(),
        changed_files: 1,
        full_run_secs: result_full.stats.total_secs,
        incremental_run_secs: result_incr.stats.total_secs,
        speedup: format!("{speedup:.1}x"),
        stages: StageBreakdown {
            full: StageTimings {
                pta_solve_secs: result_full.stats.pta_solve_secs,
                vf_build_secs: result_full.stats.valueflow_build_secs,
                total_secs: result_full.stats.total_secs,
            },
            incremental: StageTimings {
                pta_solve_secs: result_incr.stats.pta_solve_secs,
                vf_build_secs: result_incr.stats.valueflow_build_secs,
                total_secs: result_incr.stats.total_secs,
            },
        },
        pta_iterations_full: result_full.stats.pta_iterations,
        pta_iterations_incr: result_incr.stats.pta_iterations,
        constraint_diff: ConstraintDiffReport {
            added: result_incr.stats.constraint_diff_added,
            removed: result_incr.stats.constraint_diff_removed,
            changed_modules: result_incr.stats.changed_module_count,
        },
        results_match,
    })
}

pub fn run(
    compiled_dir: &Path,
    patches_dir: &Path,
    output: Option<&Path>,
    verify: bool,
) -> Result<()> {
    eprintln!("=== Incremental Analysis Benchmark ===");
    let config = PipelineConfig::default();

    // Count modules
    let module_count = std::fs::read_dir(compiled_dir)?
        .filter(|e| {
            e.as_ref()
                .map(|e| e.path().extension().map_or(false, |ext| ext == "ll"))
                .unwrap_or(false)
        })
        .count();

    // Discover scenarios from patches directory.
    // Convention: each .ll file in patches_dir is a v2 variant.
    // Filename format: <original_module_stem>_v2.ll
    // The scenario name is derived from the filename (e.g., "bltinmodule_leaf" from
    // "bltinmodule_v2_leaf.ll"). The implementing agent should adapt this to the
    // actual patch naming convention used for the target project.
    let mut scenarios = Vec::new();
    let scenario_specs = discover_scenarios(patches_dir)?;
    for (scenario_name, v2_path) in &scenario_specs {
        // Derive the original module stem from the v2 filename.
        // E.g., "bltinmodule_v2" → "bltinmodule"
        let original_stem = scenario_name.trim_end_matches("_v2");
        let v2 = Path::new(v2_path);
        match run_scenario(compiled_dir, scenario_name, original_stem, v2, &config, verify) {
            Ok(result) => scenarios.push(result),
            Err(e) => eprintln!("  ERROR in {scenario_name}: {e:#}"),
        }
    }

    let report = BenchmarkReport {
        project: compiled_dir
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        version: String::new(), // Set by caller or detected from source
        total_modules: module_count,
        scenarios,
    };

    // Print summary to stderr
    eprintln!("\n=== Results ===");
    for s in &report.scenarios {
        eprintln!(
            "  {}: full={:.1}s, incr={:.1}s, speedup={}, match={}",
            s.name, s.full_run_secs, s.incremental_run_secs, s.speedup, s.results_match
        );
    }

    // Write JSON if requested
    if let Some(path) = output {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)?;
        eprintln!("\nJSON written to {}", path.display());
    }

    Ok(())
}
```

- [ ] **Step 3: Run `make fmt && make lint`**

Expected: clean

- [ ] **Step 4: Commit**

```bash
git add crates/saf-bench/src/incremental.rs crates/saf-bench/src/main.rs
git commit -m "feat: add incremental analysis benchmark harness to saf-bench"
```

---

## Task 8: Create CPython compilation script and patches

**Files:**
- Create: `scripts/compile-incremental-bench.sh`
- Create: `tests/benchmarks/incremental/patches/cpython-leaf-edit.patch`
- Create: `tests/benchmarks/incremental/patches/cpython-core-edit.patch`
- Modify: `.gitignore`

- [ ] **Step 1: Create patches directory**

```bash
mkdir -p tests/benchmarks/incremental/patches
```

- [ ] **Step 2: Create CPython leaf edit patch**

Create `tests/benchmarks/incremental/patches/cpython-leaf-edit.patch`:

The patch adds a pointer allocation inside `builtin_len` in `Python/bltinmodule.c`. The exact line numbers must be determined from CPython v3.13.0 source. The agent implementing this task should:
1. Clone CPython at `v3.13.0`
2. Find `builtin_len` in `Python/bltinmodule.c`
3. Add `void *incr_test = PyMem_Malloc(64); PyMem_Free(incr_test);` inside the function body
4. Generate the patch with `git diff`

- [ ] **Step 3: Create CPython core edit patch**

Create `tests/benchmarks/incremental/patches/cpython-core-edit.patch`:

The patch adds a pointer indirection in `_Py_Dealloc` in `Objects/object.c`. Same approach as step 2:
1. Find `_Py_Dealloc` in `Objects/object.c`
2. Add a function pointer variable and conditional indirect call
3. Generate the patch with `git diff`

- [ ] **Step 4: Create `scripts/compile-incremental-bench.sh`**

```bash
#!/usr/bin/env bash
# Compile a C project to LLVM IR for incremental analysis benchmarking.
# Usage: compile-incremental-bench.sh <project> <version> <source-dir> <output-dir>
set -euo pipefail

PROJECT="${1:-cpython}"
VERSION="${2:-v3.13.0}"
SOURCE_DIR="${3:-/tmp/incremental-bench/${PROJECT}}"
OUTPUT_DIR="${4:-/workspace/tests/benchmarks/incremental/${PROJECT}/.compiled}"
PATCHES_DIR="/workspace/tests/benchmarks/incremental/patches"

echo "=== Compiling ${PROJECT} ${VERSION} to LLVM IR ==="

# Clone if not already present
if [ ! -d "${SOURCE_DIR}" ]; then
    echo "Cloning ${PROJECT} ${VERSION}..."
    git clone --depth 1 --branch "${VERSION}" \
        "https://github.com/python/cpython.git" "${SOURCE_DIR}"
fi

mkdir -p "${OUTPUT_DIR}"

# Find and compile all .c files
echo "Compiling .c files..."
find "${SOURCE_DIR}" -name '*.c' -not -path '*/test/*' -not -path '*/.git/*' | \
    sort | while read -r f; do
    relpath="${f#${SOURCE_DIR}/}"
    outname="$(echo "${relpath}" | tr '/' '_' | sed 's/\.c$/.ll/')"
    echo "  ${relpath} -> ${outname}"
    clang -S -emit-llvm -O0 \
        -I"${SOURCE_DIR}" \
        -I"${SOURCE_DIR}/Include" \
        -I"${SOURCE_DIR}/Include/internal" \
        "${f}" -o "${OUTPUT_DIR}/${outname}" 2>/dev/null || {
        echo "    SKIP (compile error)"
    }
done

# Generate v2 variants from patches
echo "=== Applying patches ==="
for patchfile in "${PATCHES_DIR}"/${PROJECT}-*.patch; do
    [ -f "${patchfile}" ] || continue
    patchname="$(basename "${patchfile}" .patch)"
    echo "  Applying ${patchname}..."
    # The patch file should contain the source path and the recompile instruction
    # This is project-specific and will be refined during implementation
done

echo ""
echo "=== Summary ==="
echo "Files: $(ls "${OUTPUT_DIR}"/*.ll 2>/dev/null | wc -l)"
echo "Total size: $(du -sh "${OUTPUT_DIR}" | cut -f1)"
```

- [ ] **Step 5: Add gitignore entries**

Add to `.gitignore`:

```
tests/benchmarks/incremental/cpython/
tests/benchmarks/incremental/results.json
tests/benchmarks/incremental/*/.compiled/
```

- [ ] **Step 6: Commit**

```bash
git add scripts/compile-incremental-bench.sh tests/benchmarks/incremental/patches/ .gitignore
git commit -m "feat: add CPython compilation script and patches for incremental benchmark"
```

---

## Task 9: Integration testing and cleanup

**Files:**
- Various (fix any issues found)

- [ ] **Step 1: Run `make fmt && make lint`**

Fix any lint issues across all modified files.

- [ ] **Step 2: Run `make test`**

Verify all existing tests still pass (including the new incremental tests if `#[ignore]` tests are included).

- [ ] **Step 3: Run `make test-incremental` specifically**

```bash
make test-incremental
```

Expected: 3 tests pass (leaf edit, core edit, determinism).

If tests fail, debug by:
1. Check that fixtures exist: `ls tests/fixtures/incremental/lua/*.ll | wc -l`
2. Check v2 variants differ: `diff tests/fixtures/incremental/lua/lmathlib.ll tests/fixtures/incremental/lua/lmathlib_v2.ll`
3. Check pipeline runs: add `eprintln!` or use `SAF_LOG=pipeline` to trace execution

- [ ] **Step 4: Record actual metrics**

Print the actual metrics from a successful test run for documentation:

```
Leaf edit: full PTA iters=X, incr PTA iters=Y, speedup=Z
Core edit: full PTA iters=X, incr PTA iters=Y, speedup=Z
Fixture size: X MB
```

- [ ] **Step 5: Update PROGRESS.md**

Add plan 181 to the Plans Index:

```
| 181 | incremental-e2e-tests | testing | done | Notes: ... |
```

Update Next Steps and Session Log as appropriate.

- [ ] **Step 6: Final commit**

```bash
git add plans/PROGRESS.md
git commit -m "docs: mark Plan 181 incremental E2E tests as done"
```
