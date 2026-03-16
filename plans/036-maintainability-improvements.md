# Plan 036: Maintainability Improvements (E21)

## Executive Summary

This plan addresses code smells, design issues, and technical debt identified through comprehensive analysis of the SAF codebase. The issues fall into three categories:

1. **Code Duplication** - ID parsing (6 copies), fixture loading (16 copies), diagnostics structs (9+)
2. **API Surface Issues** - Blanket `pub use *` re-exports, stringly-typed interfaces, inconsistent exports
3. **Infrastructure Gaps** - No CI/CD, fragile test paths, ignored API parameters, stub types

The plan is organized into 8 phases, each completable in a single Claude Code session.

---

## Phase 1: Centralize ID Parsing Utilities

**Scope:** Small (1-2h)
**Dependencies:** None
**Risk:** Low

### Problem

The `parse_value_id`, `parse_hex`, and `parse_block_id` functions are duplicated across 6 files in `saf-python`:

| File | Functions |
|------|-----------|
| `z3_refine.rs` | `parse_hex`, `parse_block_id`, `parse_value_id` (most complete) |
| `svfg.rs` | `parse_value_id` |
| `pta.rs` | `parse_value_id` |
| `query.rs` | `parse_value_id` |
| `absint.rs` | `parse_block_id`, `parse_value_id` |
| `cspta.rs` | imports from svfg |

### Solution

1. Create `crates/saf-python/src/id_parse.rs` with all ID parsing functions
2. Export as `pub(crate)` module
3. Replace all duplicates with imports from the new module

### Files to Modify

- **Create:** `crates/saf-python/src/id_parse.rs`
- **Modify:** `crates/saf-python/src/lib.rs` (add `mod id_parse;`)
- **Modify:** `crates/saf-python/src/z3_refine.rs` (remove local defs)
- **Modify:** `crates/saf-python/src/svfg.rs` (use `crate::id_parse`)
- **Modify:** `crates/saf-python/src/pta.rs`
- **Modify:** `crates/saf-python/src/query.rs`
- **Modify:** `crates/saf-python/src/absint.rs`
- **Modify:** `crates/saf-python/src/cspta.rs`

### Verification

```bash
make test  # All Rust + Python tests pass
```

---

## Phase 2: Consolidate Test Fixture Loading

**Scope:** Medium (2-3h)
**Dependencies:** None
**Risk:** Low

### Problem

16 E2E test files duplicate the same `load_e2e_fixture` function with:
- Fragile relative paths: `../../tests/fixtures/llvm/e2e/{name}.ll`
- Identical error handling logic
- No shared test utilities

Files affected:
- `taint_e2e.rs`, `memory_e2e.rs`, `integer_info_e2e.rs`, `oop_e2e.rs`
- `multi_module_e2e.rs`, `ifds_e2e.rs`, `cg_refinement_e2e.rs`
- `mssa_e2e.rs`, `svfg_e2e.rs`, `fspta_e2e.rs`, `checker_e2e.rs`
- `absint_e2e.rs`, `cspta_e2e.rs`, `typestate_e2e.rs`, `pathsens_e2e.rs`
- `z3_enhanced_e2e.rs`

### Solution

1. Create test helper crate: `crates/saf-test-utils/`
2. Use `env!("CARGO_MANIFEST_DIR")` for reliable path resolution
3. Provide `load_ll_fixture(name)` and `fixtures_dir()` helpers
4. Update all E2E tests to use the helper crate

### Files to Create

- `crates/saf-test-utils/Cargo.toml`
- `crates/saf-test-utils/src/lib.rs`

### Files to Modify

- `Cargo.toml` (add workspace member)
- `crates/saf-analysis/Cargo.toml` (add dev-dependency)
- All 16 E2E test files

### Verification

```bash
make test  # Run from project root
cd crates/saf-analysis && cargo test  # Also works from subdirectory
```

---

## Phase 3: Remove Ignored API Parameters

**Scope:** Small (1h)
**Dependencies:** None
**Risk:** Low

### Problem

`Project.open()` accepts `cache_dir` and `config` parameters that are silently ignored:

```rust
#[pyo3(signature = (path, *, cache_dir=None, config=None, vf_mode="fast"))]
fn open(..., cache_dir: Option<&str>, config: Option<&Bound<'_, PyAny>>, ...) {
    let _ = cache_dir;  // Ignored!
    let _ = config;     // Ignored!
}
```

This misleads users into thinking these parameters work.

### Solution

Remove unused parameters from the public API and document their absence:

1. Remove `cache_dir` and `config` from `Project.open()` signature
2. Update docstring to clarify these are future planned features
3. Add extension point to `docs/FUTURE.md`

### Files to Modify

- `crates/saf-python/src/project.rs`
- `docs/FUTURE.md`

### Verification

```bash
make test  # No tutorials use these parameters
```

---

## Phase 4: Remove GraphStore Stub

**Scope:** Small (30min)
**Dependencies:** None
**Risk:** Low

### Problem

`saf_core::graph::GraphStore` is a complete stub that was never implemented:

```rust
pub struct GraphStore {
    // TODO(E3): Add typed graph storage with deterministic iteration.
}
```

E3 was completed using separate typed graph structs (Cfg, CallGraph, etc.) without GraphStore. The stub adds confusion about the intended architecture.

### Solution

1. Verify `GraphStore` is not used anywhere in the codebase
2. Remove `crates/saf-core/src/graph.rs`
3. Remove `pub mod graph` from `crates/saf-core/src/lib.rs`

### Files to Modify

- **Remove:** `crates/saf-core/src/graph.rs`
- **Modify:** `crates/saf-core/src/lib.rs`

### Verification

```bash
cargo build --workspace
make test
```

---

## Phase 5: Explicit Module Exports

**Scope:** Medium (2h)
**Dependencies:** None
**Risk:** Medium (public API change)

### Problem

`saf-analysis/src/lib.rs` uses blanket re-exports that expose entire modules:

```rust
mod pta;
pub use pta::*;  // Exposes ~20 types/functions

mod valueflow;
pub use valueflow::*;  // Exposes ~20 types/functions
```

This makes API surface uncontrollable — any new `pub` item in submodules automatically becomes part of the crate API, risking SemVer breakage.

### Solution

Replace blanket re-exports with explicit ones. The `pta/mod.rs` and `valueflow/mod.rs` already have good explicit exports:

```rust
// pta/mod.rs already exports explicitly:
pub use config::{FieldSensitivity, PtaConfig};
pub use constraint::{AddrConstraint, ConstraintSet, ...};
// etc.
```

Change `lib.rs` from `pub use pta::*` to explicitly list what's re-exported:

```rust
pub use pta::{
    FieldSensitivity, PtaConfig, AddrConstraint, ConstraintSet,
    CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
    PtaAnalysisResult, PtaContext, PtaDiagnostics,
    PtaExport, export_constraints,
    extract_constraints, extract_constraints_reachable,
    extract_global_initializers, extract_intraprocedural_constraints,
    FunctionLocationMap,
    CollapseWarning, FieldPath, Location, LocationFactory, PathStep,
    AliasResult, PtaResult,
    PointsToMap, solve,
};
```

### Files to Modify

- `crates/saf-analysis/src/lib.rs`

### Verification

```bash
make test
cargo doc --no-deps -p saf-analysis  # Review generated docs
```

---

## Phase 6: Add Basic CI/CD Pipeline

**Scope:** Medium (2h)
**Dependencies:** None
**Risk:** Low (additive)

### Problem

No CI/CD pipeline exists. Tests only run manually via `make test`. No automated:
- Test execution on PR
- Lint checks (clippy, rustfmt)
- Matrix testing (LLVM versions)

### Solution

Create `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]

env:
  APT_MIRROR: ""  # Use default mirrors, not China mirror

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build test image
        run: docker build --target test --build-arg APT_MIRROR="" -t saf-test .
      - name: Run tests
        run: docker run saf-test

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build dev image
        run: docker build --target dev --build-arg APT_MIRROR="" -t saf-dev .
      - name: Run lints
        run: docker run saf-dev make lint
```

### Files to Create

- `.github/workflows/ci.yml`

### Files to Modify

- `Dockerfile` (ensure APT_MIRROR defaults to empty for CI)

### Verification

- Push to branch, verify GitHub Actions run successfully

---

## Phase 7: Improve Error Types

**Scope:** Medium (2-3h)
**Dependencies:** Phases 1-6
**Risk:** Medium

### Problem

Error enums are minimalist with only generic variants:

```rust
// saf-core/src/error.rs
pub enum CoreError {
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

// saf-analysis/src/error.rs
pub enum AnalysisError {
    #[error("not implemented: {0}")]
    NotImplemented(String),
}
```

This provides poor error context for users and makes programmatic error handling impossible.

### Solution

Expand error types with specific variants:

```rust
pub enum AnalysisError {
    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("selector resolution failed: {0}")]
    SelectorResolution(String),

    #[error("analysis did not converge after {0} iterations")]
    Convergence(usize),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("missing function: {0}")]
    MissingFunction(String),
}
```

### Files to Modify

- `crates/saf-core/src/error.rs`
- `crates/saf-analysis/src/error.rs`
- `crates/saf-frontends/src/error.rs`
- Update callers that use `NotImplemented` for specific errors

### Verification

```bash
make test
make lint
```

---

## Phase 8: Reduce Unsafe .unwrap() Calls

**Scope:** Large (4h+)
**Dependencies:** Phases 1-7
**Risk:** Medium

### Problem

174 `.unwrap()` calls in `saf-analysis/src/` library code violate CLAUDE.md coding conventions ("No `.unwrap()` in library code").

### Solution

Triage by severity and fix systematically:

1. **Critical:** unwraps on user-provided data → convert to `?` with proper errors
2. **High:** unwraps that can fail with valid inputs → add Result return type
3. **Low:** unwraps on internal invariants → convert to `expect("reason")`

Focus on high-occurrence files first:
- `valueflow/builder.rs` (12 occurrences)
- `selector/resolve.rs` (11 occurrences)
- `pta/extract.rs` (9 occurrences)
- `cfg.rs` (9 occurrences)

### Strategy

For each file:
1. Replace user-input unwraps with `?` and proper error types (from Phase 7)
2. Replace invariant unwraps with `expect("message explaining invariant")`
3. Add `#[must_use]` where appropriate

### Files to Modify

~36 files in `crates/saf-analysis/src/` (systematic pass)

### Verification

```bash
make test
make lint  # No new clippy warnings
grep -r "\.unwrap()" crates/saf-analysis/src/ | wc -l  # Should decrease significantly
```

---

## Summary

| Phase | Task | Scope | Risk | Est. Time |
|-------|------|-------|------|-----------|
| 1 | Centralize ID parsing | Small | Low | 1-2h |
| 2 | Consolidate fixture loading | Medium | Low | 2-3h |
| 3 | Remove ignored parameters | Small | Low | 1h |
| 4 | Remove GraphStore stub | Small | Low | 30min |
| 5 | Explicit module exports | Medium | Medium | 2h |
| 6 | Add CI/CD pipeline | Medium | Low | 2h |
| 7 | Improve error types | Medium | Medium | 2-3h |
| 8 | Reduce unwrap() calls | Large | Medium | 4h+ |

**Recommended Order:** 1 → 2 → 3 → 4 → 6 → 5 → 7 → 8

Phases 1-4 and 6 are independent and low-risk. Phase 5 involves API changes. Phases 7-8 should come last as they're larger and benefit from earlier cleanup.

---

## Post-Implementation Updates

After completing all phases:

1. **Update `plans/PROGRESS.md`:**
   - Add E21 to Epics list
   - Add Plan 036 to Plans Index
   - Update Session Log

2. **Update `docs/tool-comparison.md`:**
   - No feature changes (this is internal cleanup)

3. **Update `docs/FUTURE.md`:**
   - Add note about cache_dir/config parameters as future work
   - Remove GraphStore TODO reference

---

## Verification Checklist

After each phase:
- [ ] All Rust tests pass (`make test-rust` or `cargo nextest run`)
- [ ] All Python tests pass (`make test-python`)
- [ ] All tutorials work (`make shell` → run sample tutorials)
- [ ] Clippy clean (`make lint`)
- [ ] No regressions in existing functionality
