# Plan 070: Code Quality & Architectural Refactoring

## Motivation

After thorough scrutiny of the entire codebase (~91K LOC across 211 files), 8 exploration agents identified systemic code smells and architectural issues that hinder future development. This plan addresses the highest-impact issues while preserving all existing functionality and PTABen benchmark performance.

## Scope

**In scope**: Code duplication elimination, missing abstractions, performance fixes, dead code cleanup.
**Out of scope**: Crate splitting (too disruptive), Python binding redesign, new features.

---

## Phase A: Eliminate Transfer Function Proliferation (~400 LOC net reduction)

**Problem**: `nullness.rs` has 4 near-identical analysis entry points, 4 near-identical worklist loops, and 4 near-identical transfer functions (2,273 lines total, ~1,000 duplicated). `transfer.rs` has 3 similar variants. `fixpoint.rs` has 4 variants. `interprocedural.rs` has 4 variants.

**Root cause**: Each new capability (PTA, specs, summaries) was added as a new copy of the entire function rather than parameterizing the existing one.

### Tasks

**A1. Create `AnalysisContext` struct for nullness** (`absint/nullness.rs`)
- Define `NullnessContext` struct holding optional PTA, specs, summaries references
- Replace 4 `analyze_nullness*` public functions with single `analyze_nullness_with_context(module, config, ctx: &NullnessContext)`
- Keep the 4 original functions as thin wrappers (deprecated) for backwards compat during transition
- Replace 4 `analyze_function*` private functions with one parameterized on `NullnessContext`

**A2. Unify nullness transfer functions** (`absint/nullness.rs`)
- Replace 4 `transfer_instruction*` functions with single `transfer_nullness_instruction(inst, module, state, ctx: &NullnessContext)`
- The function checks `ctx.pta`, `ctx.specs`, `ctx.summaries` for optional behavior
- Same logic, fewer copies

**A3. Unify interval transfer functions** (`absint/transfer.rs`)
- Replace 3 `transfer_instruction*` functions with single `transfer_instruction_with_context(inst, state, constant_map, module, ctx: &TransferContext)`
- `TransferContext` holds optional PTA, specs, summaries

**A4. Unify fixpoint solvers** (`absint/fixpoint.rs`)
- Replace 4 `solve_abstract_interp*` functions with single parameterized solver
- `FixpointContext` struct holds optional specs, PTA

**A5. Unify interprocedural solvers** (`absint/interprocedural.rs`)
- Replace 4 `solve_interprocedural*` functions with single parameterized solver

**Files**: `crates/saf-analysis/src/absint/nullness.rs`, `transfer.rs`, `fixpoint.rs`, `interprocedural.rs`

---

## Phase B: Consolidate PTA Constraint Extraction (~80 LOC net reduction)

**Problem**: 3 extraction entry points (`extract_constraints`, `extract_constraints_reachable`, `extract_intraprocedural_constraints`) duplicate the same setup steps. CLAUDE.md explicitly warns: "Missing one causes silent failures."

### Tasks

**B1. Extract common constraint setup** (`pta/extract.rs`)
- Create `extract_base_constraints(module, factory, constraints)` helper
- All 3 entry points call it instead of repeating `extract_global_addr_constraints` + `extract_function_addr_constraints`

**B2. Consolidate interprocedural extraction** (`pta/extract.rs`)
- Create `extract_interprocedural_impl(module, reachable: Option<&BTreeSet<FunctionId>>, constraints)`
- Replace `extract_interprocedural()` and `extract_interprocedural_reachable()` as 2-line wrappers

**B3. Extract synthetic ValueId helper** (`pta/extract.rs`)
- Create `make_synthetic_value(kind, global_obj, index)` to replace 3 repeated patterns in `extract_aggregate_elements()`

**Files**: `crates/saf-analysis/src/pta/extract.rs`

---

## Phase C: Unify Function Property Lookup (~50 LOC net reduction)

**Problem**: `is_known_pure_function()` exists in BOTH `transfer.rs` (84 lines, comprehensive) and `nullness.rs` (28 lines, incomplete). The nullness version is missing math, bit manipulation, and overflow intrinsics, causing incorrect impurity assumptions.

### Tasks

**C1. Create shared `function_properties` module** (`absint/function_properties.rs`)
- Move the comprehensive `is_known_pure_function()` from `transfer.rs` here
- Move `returns_first_argument()`, `returns_nonnull()`, `is_known_noreturn()` from `nullness.rs` here
- Add spec-aware wrappers: `is_pure_with_specs()`, `returns_nonnull_with_specs()`, `is_noreturn_with_specs()`

**C2. Update callers** (`nullness.rs`, `transfer.rs`)
- Replace inline `is_known_pure_function()` calls with `function_properties::is_pure()`
- Remove duplicate function definitions

**Files**: `crates/saf-analysis/src/absint/function_properties.rs` (new), `nullness.rs`, `transfer.rs`

---

## Phase D: Add Performance Indices to Graph Types (~60 LOC added)

**Problem**: `CallGraph::resolve_indirect()` does O(n) linear scan through `self.nodes` to find target by FunctionId. `ICFG::successors()` does O(n) scan of all `inter_edges`. These are hot paths in CG refinement.

### Tasks

**D1. Add `FunctionId -> CallGraphNode` index to `CallGraph`** (`callgraph.rs`)
- Add `func_index: BTreeMap<FunctionId, CallGraphNode>` field
- Populate during `build()`
- Use in `resolve_indirect()` and `add_callback_edges()` instead of `.iter().find()`

**D2. Add successor index to `Icfg`** (`icfg.rs`)
- Add `inter_successors: BTreeMap<BlockId, BTreeSet<(BlockId, IcfgEdge)>>` field
- Populate when edges are inserted
- Use in `successors()` instead of linear scan

**Files**: `crates/saf-analysis/src/callgraph.rs`, `crates/saf-analysis/src/icfg.rs`

---

## Phase E: Clean Up Dead Code & Code Hygiene (~100 LOC removed)

### Tasks

**E1. Feature-gate dead code modules** (`pta/steensgaard.rs`, `pta/incremental.rs`)
- Move `#![allow(dead_code)]` modules behind `#[cfg(feature = "experimental")]`
- Add `experimental` feature to Cargo.toml (disabled by default)
- These are 645 + 758 = 1,403 lines of fully implemented but unused code

**E2. Remove redundant export sorting** (`svfg/export.rs`)
- BTreeSet already guarantees order; remove explicit re-sorting

**E3. Remove unused LLVM adapter trait methods** (`llvm/adapter.rs`)
- `get_called_function()` and `get_called_value()` are defined but never called
- Remove or document with intent

**E4. Consolidate LLVM 17/18 adapter duplication** (`llvm/llvm17.rs`, `llvm/llvm18.rs`)
- These files are 99% identical (only class name and version string differ)
- Extract shared implementation into a generic function or macro

**Files**: `crates/saf-analysis/src/pta/steensgaard.rs`, `pta/incremental.rs`, `svfg/export.rs`, `crates/saf-frontends/src/llvm/adapter.rs`, `llvm17.rs`, `llvm18.rs`

---

## Phase F: Improve Solver Code Organization (~50 LOC refactored)

**Problem**: `GenericSolver::process_value()` is 104 lines with 4 constraint types interleaved, complex nesting, and unconditional cloning.

### Tasks

**F1. Extract constraint-type handlers** (`pta/solver.rs`)
- Create `handle_copy_constraints()`, `handle_load_constraints()`, `handle_store_constraints()`, `handle_gep_constraints()`
- `process_value()` becomes a 10-line dispatcher

**F2. Extract GEP merge logic** (`pta/solver.rs`)
- Create `merge_gep_with_base_path(base_path, gep_path) -> Option<FieldPath>` helper
- Replaces 20-line nested conditional in process_value

**Files**: `crates/saf-analysis/src/pta/solver.rs`

---

## Phase G: Test Infrastructure Cleanup (~30 LOC)

### Tasks

**G1. Add AIR-JSON fixture helpers to `saf-test-utils`**
- Add `load_air_json_fixture(name) -> AirBundle` and `load_air_json_module(name) -> AirModule`
- Replace 3 duplicate implementations in test files

**Files**: `crates/saf-test-utils/src/lib.rs`, `crates/saf-analysis/tests/graph_integration.rs`, `pta_integration.rs`, `valueflow_integration.rs`

---

## Estimated Impact

| Phase | Net LOC Change | Risk | Benefit |
|-------|---------------|------|---------|
| A (Transfer unification) | -400 | Medium | Eliminates ~1,300 lines of duplication, fixes nullness purity bug |
| B (PTA extraction) | -80 | Low | Eliminates triple-maintenance trap |
| C (Function properties) | -50 | Low | Fixes inconsistent purity checking |
| D (Performance indices) | +60 | Low | 5-10% CG refinement speedup |
| E (Dead code cleanup) | -100 | Low | Cleaner codebase |
| F (Solver refactor) | 0 | Low | Better readability |
| G (Test helpers) | -30 | Very Low | Less test boilerplate |
| **Total** | **~-600** | | |

## Verification Plan

1. `make test` — All Rust + Python tests must pass (currently 1,091 Rust + 248 Python)
2. `make lint` — Clippy must pass cleanly
3. **PTABen benchmark**: Run full suite, verify no regressions from current baseline:
   - 1,772 Exact (must not decrease)
   - 337 Unsound (must not increase)
4. Spot-check: Run `cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled` inside Docker to confirm benchmark numbers

## Execution Order

A1 -> A2 -> A3 -> A4 -> A5 (sequential, each builds on previous)
B1 -> B2 -> B3 (sequential within phase)
C1 -> C2 (sequential)
D1, D2 (independent, can be parallel)
E1, E2, E3, E4 (independent, can be parallel)
F1 -> F2 (sequential)
G1 (independent)

Phases A-G are independent of each other and can be done in any order. Recommended order: C (quick win, fixes real bug), B (quick win), D (performance), A (largest change), E, F, G.
