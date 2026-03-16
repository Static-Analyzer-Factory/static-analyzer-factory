# Plan 104: Code Quality Optimization

## Motivation

After 103 plans of feature development (from scaffolding through PTA scalability), the SAF codebase has grown to ~107K LOC across 211 Rust files. While lint compliance is excellent (only 2 clippy warnings + 2 rustfmt diffs), the rapid feature accretion has left structural debt: duplicated analysis entry points, excessive `#[allow]` annotations, dead code behind suppression markers, and overly complex functions. This plan addresses the highest-impact issues while preserving all functionality and the PTABen baseline (2251 Exact, 69 Unsound).

## Scope

**In scope**: Code duplication elimination, dead code cleanup, `#[allow]` reduction, function decomposition, lint fixes, module hygiene.
**Out of scope**: New features, crate splitting, Python API redesign, algorithm changes, performance optimization.

---

## PTABen Baseline (Regression Gate)

| Metric | Value |
|--------|-------|
| Total Exact | 2251 |
| Total Unsound | 69 |
| Total Skip | 92 |

Full category breakdown in `tests/benchmarks/ptaben/baseline-104.json`.

**Gate**: After all changes, Exact must not decrease and Unsound must not increase.

---

## Current Lint Status

| Category | Count |
|----------|-------|
| Clippy warnings | 2 (both in saf-wasm) |
| Rustfmt diffs | 2 (saf-wasm, air_dump_e2e.rs) |
| `#[allow(clippy::too_many_lines)]` | 35 functions |
| `#[allow(clippy::too_many_arguments)]` | 19 functions |
| `#[allow(dead_code)]` | 30+ items |
| `#![allow(...)]` crate-level | 6 crates with blanket allows |
| `.unwrap()` in library code | 258 occurrences across 51 files |

---

## Phase A: Fix All Lint Warnings & Formatting (Quick Wins)

**Risk**: Very Low | **LOC impact**: ~-20

### A1. Fix saf-wasm clippy warnings (2 issues)
- `saf-wasm/src/lib.rs:38` — Remove unused `spec_yamls` field from `WasmConfig`, or wire it through
- `saf-wasm/src/lib.rs:250` — Replace redundant closure `|n| n.to_string()` with `ToString::to_string`

### A2. Fix rustfmt diffs (2 files)
- `saf-wasm/src/lib.rs:292` — Let rustfmt collapse chain expression
- `saf-analysis/tests/air_dump_e2e.rs:40,48` — Let rustfmt reformat if/else and iterator chains

### A3. Narrow crate-level `#![allow]` in saf-python
`saf-python/src/lib.rs` has 12 crate-level `#![allow(...)]` annotations. These hide all instances of those lints across the entire crate:
- `too_many_arguments`, `too_many_lines`, `module_name_repetitions`, `struct_excessive_bools`, `similar_names`, `must_use_candidate` — Move to function/struct level where actually needed
- `unnecessary_wraps`, `needless_pass_by_value`, `unused_self` — Keep crate-level (PyO3 pervasive requirement)
- `doc_markdown`, `missing_errors_doc`, `missing_panics_doc` — Keep crate-level (documentation convention for bindings)

### A4. Narrow crate-level `#![allow]` in saf-analysis
- `saf-analysis/src/lib.rs` has `#![allow(clippy::doc_markdown)]` and `#![allow(clippy::missing_errors_doc)]` — Keep these (pervasive, 500+ public items)

### A5. Narrow crate-level `#![allow]` in saf-bench
- `saf-bench/src/lib.rs` and `main.rs` both have `#![allow(clippy::doc_markdown)]` — Keep (benchmark infra, lower doc priority)

**Files**: `saf-wasm/src/lib.rs`, `saf-analysis/tests/air_dump_e2e.rs`, `saf-python/src/lib.rs`

---

## Phase B: Dead Code Cleanup (~-1,500 LOC)

**Risk**: Low | **LOC impact**: ~-1,500

### B1. Feature-gate experimental PTA modules
`pta/steensgaard.rs` (641 LOC) and `pta/incremental.rs` (756 LOC) are fully implemented but never used (accessed only via `mod` declarations, gated with implicit `dead_code`). These are 1,397 lines of dead code.
- Add `experimental` feature to `saf-analysis/Cargo.toml` (disabled by default)
- Gate both modules behind `#[cfg(feature = "experimental")]`
- Remove `#[allow(dead_code)]` annotations inside them

### B2. Remove dead LLVM adapter methods
`saf-frontends/src/llvm/adapter.rs` has `#[allow(dead_code)]` on the entire trait impl (line 21) plus several individual functions (line 54, 156). Check each method:
- If never called and no planned use: remove
- If part of a trait contract: document intent

### B3. Remove dead debug_info fields
`saf-frontends/src/llvm/debug_info.rs` has 4 `#[allow(dead_code)]` fields (lines 26, 34, 40, 61) marked "Will be used when debug info extraction is implemented." Since debug info is now extracted via `!DILocation` metadata (Plan 101), either wire these fields or remove them.

### B4. Clean up dead Python binding code
- `saf-python/src/dda.rs` has 7 `#[allow(dead_code)]` fields (lines 33-45)
- `saf-python/src/z3_refine.rs` has 4 `#[allow(dead_code)]` items (lines 833-882)
- `saf-python/src/absint.rs` has 1 `#[allow(dead_code)]` (line 316)
- `saf-python/src/combined.rs` has 1 `#[allow(dead_code)]` (line 151) — labeled "Public API for use without custom config" so may be intentional

### B5. Remove dead code in saf-analysis
- `saf-analysis/src/pta/clustering.rs` — entire module is `#![allow(dead_code)]` with additional cast allows (3 lines of crate-level allows)
- `saf-analysis/src/proptest_arb.rs` — `#![allow(dead_code)]` (line 5), "Utility functions for future tests"
- `saf-analysis/src/cspta/solver.rs:408` — dead convenience constructor
- `saf-analysis/src/ifds/ide_solver.rs:814` — dead code
- `saf-analysis/src/cg_refinement.rs:317` — dead code
- `saf-analysis/src/valueflow/builder.rs:22` — dead field "Reserved for future use"
- `saf-analysis/src/mta/discovery.rs:19`, `mta/mhp.rs:17,20` — dead fields

For each: verify truly unused with grep, then remove (or document if intentionally reserved).

### B6. Clean up dead SVComp fields
- `saf-bench/src/svcomp/property.rs:1100,1104` — 2 `#[allow(dead_code)]` items

**Files**: All crates, focused on `#[allow(dead_code)]` sites

---

## Phase C: Absint Entry Point Unification (~-600 LOC)

**Risk**: Medium | **LOC impact**: ~-600

This is the highest-impact structural improvement. The absint subsystem has proliferated entry points through Plans 071-103, creating maintenance burden.

### C1. Unify fixpoint solver entry points
Currently 5 public functions in `fixpoint.rs`:
- `solve_abstract_interp()` (line 104)
- `solve_abstract_interp_with_specs()` (line 119)
- `solve_abstract_interp_with_pta()` (line 138)
- `solve_abstract_interp_with_pta_and_summaries()` (line 164)
- `solve_abstract_interp_with_context()` (line 187)

**Fix**: Keep `solve_abstract_interp_with_context()` as the single implementation. Convert the other 4 to thin wrappers that construct a context and delegate. The wrappers are ~5 lines each; the core logic is in `with_context`. This is mostly already done — verify and clean up any remaining duplication in the internal impl.

### C2. Unify interprocedural solver entry points
Currently 5 public functions in `interprocedural.rs`:
- `solve_interprocedural()` (line 862)
- `solve_interprocedural_with_specs()` (line 878)
- `solve_interprocedural_with_pta()` (line 901)
- `solve_interprocedural_with_pta_and_specs()` (line 943)
- `solve_interprocedural_with_context()` (line 466)

**Fix**: Same pattern — ensure `with_context` is the single implementation, others are thin wrappers.

### C3. Unify transfer function entry points
Currently 4 public functions in `transfer.rs`:
- `transfer_instruction()` (line 1295)
- `transfer_instruction_with_pta()` (line 1318)
- `transfer_instruction_with_pta_and_summaries()` (line 1345)
- `transfer_instruction_with_context()` (line 174)

**Fix**: Ensure `with_context` is the single implementation. Mark others as `#[deprecated]` if they have external callers, or remove if internal-only.

### C4. Unify nullness analysis entry points
Currently 5 public functions in `nullness.rs`:
- `analyze_nullness()` (line 464)
- `analyze_nullness_with_pta()` (line 475)
- `analyze_nullness_with_pta_and_specs()` (line 496)
- `analyze_nullness_with_pta_specs_and_summaries()` (line 519)
- `analyze_nullness_with_context()` (line 408)

**Fix**: Same pattern.

### C5. Audit all callers
After unification, grep all callers to ensure they use the context-based API. Update tests that call the convenience wrappers to use `with_context` directly if the wrappers are removed.

**Files**: `crates/saf-analysis/src/absint/{fixpoint,interprocedural,transfer,nullness}.rs`

---

## Phase D: Function Decomposition — Reduce `too_many_lines` (~0 net LOC)

**Risk**: Low-Medium | **LOC impact**: ~0 (reorganization, not deletion)

35 functions carry `#[allow(clippy::too_many_lines)]`. Most are legitimate (algorithm implementations that shouldn't be split). Triage:

### D1. Decomposable functions (should be split)
Review each `#[allow(clippy::too_many_lines)]` site. For each, determine:
- **Split-worthy**: Function handles multiple distinct concerns that can be extracted
- **Algorithmic unity**: Function implements a single algorithm where splitting hurts readability

Likely candidates for decomposition:
- `absint/checker.rs:569` and `:1278` — checker functions that handle multiple operation types could extract per-operation handlers
- `absint/escape.rs:168` — escape analysis with distinct phases
- `cg_refinement.rs:96` — CG refinement with clear phases

### D2. Functions that should keep `#[allow]` but add `// NOTE:` comments
For functions where the allow is justified, ensure each has a `// NOTE:` comment explaining why:
```rust
// NOTE: This function implements the IFDS tabulation algorithm as a single
// cohesive unit. Splitting would obscure the algorithm structure.
#[allow(clippy::too_many_lines)]
fn solve_ifds() { ... }
```

### D3. Reduce `too_many_arguments` where possible
19 functions have `#[allow(clippy::too_many_arguments)]`. For each:
- If arguments share a common theme, group into a context/config struct
- If PyO3-constrained, document with `// NOTE: PyO3 requires ...`
- `absint/interprocedural.rs:2695` — likely needs a parameter struct
- `z3_utils/condition_prover.rs` — 4 functions, may benefit from a `ProverContext` struct

**Files**: All files with `too_many_lines`/`too_many_arguments` annotations

---

## Phase E: `.unwrap()` Audit in Library Code (~-100 unwraps)

**Risk**: Low | **LOC impact**: ~+50 (replacing unwrap with proper handling)

CLAUDE.md says "No `.unwrap()` in library code." Currently 258 occurrences across 51 files in saf-analysis alone. Triage:

### E1. Categorize all `.unwrap()` calls
For each occurrence:
- **In tests**: OK, leave as-is
- **After `.get(key)` that was just inserted**: Replace with `.expect("key was just inserted")`
- **On `Regex::new(literal)`: Replace with `expect("valid regex")`
- **Genuine error paths**: Replace with `?` or proper error handling
- **In `BTreeMap::entry().or_insert()` chains**: Usually fine, verify

### E2. Priority fixes
Focus on library code paths (not tests, not benchmarks):
- `saf-analysis/src/pta/mod_ref.rs` — 12 occurrences (highest density)
- `saf-analysis/src/cfg.rs` — 11 occurrences
- `saf-analysis/src/selector/resolve.rs` — 11 occurrences
- `saf-analysis/src/valueflow/builder.rs` — 12 occurrences
- `saf-analysis/src/absint/transfer.rs` — 9 occurrences

Replace unwraps with `expect()` + invariant comment where the unwrap is justified, or with `?` where it can propagate.

**Files**: All files with `.unwrap()` in non-test code

---

## Phase F: PTA Constraint Extraction Consolidation (~-60 LOC)

**Risk**: Low | **LOC impact**: ~-60

### F1. Extract common constraint setup
Three entry points (`extract_constraints`, `extract_constraints_reachable`, `extract_intraprocedural_constraints`) in `pta/extract.rs` repeat the same setup. As CLAUDE.md warns: "Missing one causes silent failures in CG refinement or CS-PTA."

**Fix**: Create `extract_base_constraints()` helper called by all three. When a new constraint type is added, it only needs to go in one place.

### F2. Consolidate interprocedural extraction
Create `extract_interprocedural_impl(module, reachable: Option<&BTreeSet<FunctionId>>, ...)` to replace the two near-identical functions.

**Files**: `crates/saf-analysis/src/pta/extract.rs`

---

## Phase G: Module Hygiene & Documentation (~+100 LOC docs)

**Risk**: Very Low | **LOC impact**: ~+100

### G1. Add module-level documentation
Several key modules lack top-level `//!` documentation:
- `saf-analysis/src/absint/mod.rs` — should document the absint framework architecture
- `saf-analysis/src/pta/mod.rs` — should document PTA variants and relationships
- `saf-analysis/src/cspta/` — should document CS-PTA vs CI-PTA distinction

### G2. Review `pub` visibility
Check for items that are `pub` but only used within the crate — these should be `pub(crate)` to reduce the public API surface.

### G3. Consistent `mod.rs` vs file-based modules
Ensure consistent use of either `mod.rs` pattern or Rust 2018 file-based modules across crates.

**Files**: Various `mod.rs` and `lib.rs` files

---

## Execution Order

```
Phase A (30 min)  — Quick lint fixes, immediate clean state
   ↓
Phase B (2 hrs)   — Dead code removal, largest LOC impact
   ↓
Phase C (3 hrs)   — Absint unification, highest structural impact
   ↓
Phase D (2 hrs)   — Function decomposition, readability
   ↓
Phase E (2 hrs)   — Unwrap audit, safety improvement
   ↓
Phase F (1 hr)    — PTA extraction consolidation
   ↓
Phase G (1 hr)    — Documentation & hygiene
```

Phases A and B can be parallelized. C must come before D (C may change function structure). E, F, G are independent of each other and of C/D.

---

## Estimated Impact Summary

| Phase | Net LOC Change | Risk | Primary Benefit |
|-------|---------------|------|----------------|
| A (Lint fixes) | -20 | Very Low | Clean `make lint` |
| B (Dead code) | -1,500 | Low | Remove maintenance burden of unused code |
| C (Absint unify) | -600 | Medium | Eliminate triple/quadruple maintenance trap |
| D (Decompose) | 0 | Low-Medium | Readability, reduce `#[allow]` count |
| E (Unwrap audit) | +50 | Low | Safety, explicit invariants |
| F (PTA extract) | -60 | Low | Prevent silent constraint failures |
| G (Documentation) | +100 | Very Low | Maintainability for new contributors |
| **Total** | **~-2,030** | | |

---

## Verification Plan

After ALL changes:
1. `make fmt && make lint` — Zero warnings, zero formatting diffs
2. `make test` — All Rust + Python tests pass
3. **PTABen regression gate**:
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben \
     --compiled-dir tests/benchmarks/ptaben/.compiled \
     -o /workspace/tests/benchmarks/ptaben/post-104.json'
   ```
   - Exact >= 2251 (must not decrease)
   - Unsound <= 69 (must not increase)
4. Verify no new `#[allow(dead_code)]` was introduced
5. Verify `#[allow(clippy::too_many_lines)]` count decreased (target: <25, from 35)

---

## What NOT to Change

1. **Algorithm implementations** — Don't refactor working PTA/IFDS/absint algorithms for aesthetic reasons
2. **PTA solver hot paths** — Don't change constraint indexing, SCC, or propagation code from Plan 103
3. **Test fixtures** — Don't reorganize LLVM IR fixtures
4. **Python API surface** — Don't change public Python function signatures
5. **AIR schema** — Don't modify AirBundle/AirModule/Instruction types
6. **Determinism invariants** — Don't replace BTreeMap/BTreeSet anywhere
7. **Config contract** — Don't split or modify the Config struct (it's the SRS Section 6 contract)
