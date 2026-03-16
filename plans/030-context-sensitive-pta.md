# Plan 030: Context-Sensitive Points-To Analysis

**Epic:** E16 — Context-Sensitive PTA
**Status:** approved
**Created:** 2026-01-30

## Overview

Build a whole-program k-call-site-sensitive Andersen pointer analysis (k-CFA) for SAF. This is the next step in the PTA precision chain after flow-sensitive PTA (E13), and is SVF's key differentiator for precision. Context sensitivity separates the analysis of a function called from different call sites, dramatically reducing false positives from utility functions, wrappers, allocators, and container helpers.

**Prerequisite fix:** Interprocedural PTA parameter passing (the known limitation from E10) must be implemented first — without modeling argument→parameter and return→caller copy constraints at call sites, context sensitivity has no interprocedural information to separate.

**Configurable k:** k=1 (1-call-site context, 30-50% false positive reduction), k=2 (two-level wrappers, higher precision), k=3 (deep call chains, maximum precision). Default: k=1.

## Research Summary

### Design Influences

- **SVF `CondPTAImpl` / `ContextCond`**: SVF's context-sensitive PTA uses `ContextCond` — a list of call-site IDs as the context string. Each pointer is parameterized by the context of its containing method. Objects are parameterized by the context of the allocating method. Indirect calls handled on-the-fly during analysis. SVF also supports demand-driven context sensitivity (SUPA, TSE'18).
- **Andersen + Contexts (k-CFA)**: Classic k-CFA maintains a context stack of bounded depth k. At call sites, push the call-site ID; at function entry, truncate to k. All values within a function are qualified by the function's current context.
- **"Return of CFA" (POPL 2022)**: Call-site sensitivity can match or beat object sensitivity even for OO programs, especially with context tunneling. For C/C++ (no receiver objects), call-site sensitivity is the natural choice.
- **DSA (Lattner et al.)**: Practical context-sensitive analysis for C/C++ via heap cloning + unification. Key engineering insight: collapse context within recursive SCCs to avoid infinite chains.
- **Selective sensitivity**: Apply context sensitivity only to functions that benefit (small utilities, wrappers, allocators). Heuristic-based selection avoids explosion on large programs.

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| k-call-site sensitivity (k-CFA) | Natural for C/C++ (no receiver objects). Matches SVF's `ContextCond`. Recent research (POPL 2022) shows it can beat object sensitivity. |
| Configurable k=1,2,3 | k=1 covers single-level wrappers (malloc→my_malloc). k=2 covers two-level chains. k=3 matches SVF's typical settings. |
| Interprocedural parameter passing as Phase 1 | Without argument→parameter copy constraints, context sensitivity has nothing to separate. This fixes the known E10 limitation. |
| Context = `Vec<InstId>` bounded by k | Deterministic (Ord-compatible). Empty = CI. Push call-site InstId at call, truncate oldest if > k. |
| SCC collapse: recursive functions get empty context | Functions in call-graph SCCs collapse to empty context to avoid infinite chains. Matches DSA/SVF approach. |
| New `cspta/` module (not modifying existing `pta/`) | Preserves existing CI PTA API. CS-PTA is a separate analysis that consumes CI results as initialization. |
| Context-insensitive summary available | CS results can be collapsed (union all contexts) for downstream consumers. Existing checkers, SVFG, flow-sensitive PTA can use the summary without modification. |
| Heap cloning: allocation-site context | HeapAlloc objects qualified by allocating function's context. Distinguishes `malloc` called from site A vs site B. |

## Architecture

### Module Structure

```
crates/saf-analysis/src/cspta/
  mod.rs          — Public API: solve_context_sensitive(), CsPtaConfig, CsPtaResult
  context.rs      — CallSiteContext type, context operations (push, truncate, merge)
  interproc.rs    — Interprocedural constraint generation (arg→param, return→caller)
  solver.rs       — Context-sensitive worklist solver
  export.rs       — JSON export with context information
```

### Data Types

```rust
/// A call-site context: sequence of up to k call-site InstIds.
/// Empty context = context-insensitive (global scope / collapsed SCC).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallSiteContext {
    sites: Vec<InstId>,  // bounded by k
}

/// Context-sensitive PTA configuration.
pub struct CsPtaConfig {
    pub k: u32,                          // context depth (1, 2, or 3)
    pub field_sensitivity: FieldSensitivity,
    pub max_iterations: usize,
    pub max_objects: usize,
}

/// Context-qualified value: a (ValueId, Context) pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CtxValue {
    pub value: ValueId,
    pub ctx: CallSiteContext,
}

/// Context-sensitive points-to map.
pub type CsPointsToMap = BTreeMap<CtxValue, BTreeSet<LocId>>;

/// Result of context-sensitive PTA.
pub struct CsPtaResult {
    cs_pts: CsPointsToMap,                           // full context-sensitive map
    ci_summary: BTreeMap<ValueId, BTreeSet<LocId>>,   // collapsed CI summary
    locations: BTreeMap<LocId, Location>,
    diagnostics: CsPtaDiagnostics,
}
```

### Solver Algorithm

```
Input: AirModule, CallGraph, CsPtaConfig
Output: CsPtaResult

1. Extract base constraints (Addr, Copy, Load, Store, Gep) — same as CI
2. Extract interprocedural constraints:
   For each CallDirect { callee } instruction at call site cs:
     For each (actual_arg[i], formal_param[i]) pair:
       Record InterCallConstraint { site: cs, arg: actual_arg, param: formal_param }
     For callee's return value (if any):
       Record InterRetConstraint { site: cs, ret_val: callee_return, caller_dst: call_result }
   For each CallIndirect instruction:
     Resolve callees from CI PTA results (or CG refinement)
     Generate same constraints for each resolved callee
3. Identify SCC functions on call graph (using existing Tarjan SCC)
4. Initialize CS solver:
   - Seed worklist from Addr constraints with empty context
   - For each function, compute its context set:
     If in SCC: always empty context
     Otherwise: contexts from call sites (push site ID, truncate to k)
5. Fixed-point iteration:
   Pop (value, ctx) from worklist
   Process constraints:
     - Intraprocedural (Copy, Load, Store, Gep): same context propagation
     - Call site cs calling function f:
       caller_ctx → callee_ctx = push(caller_ctx, cs)  [truncate to k]
       Copy arg[i]@caller_ctx → param[i]@callee_ctx
     - Return from function f at call site cs:
       callee_ctx → caller_ctx (pop cs from context)
       Copy ret@callee_ctx → result@caller_ctx
   Until fixed point
6. Build CI summary: for each (value, ctx) → locs, union locs into value → locs
7. Compute diagnostics
```

### Heap Cloning

When `HeapAlloc` at instruction I is reached in context C:
- Object ID = `make_id("heap_clone", I.raw() ++ C.sites[..].raw())`
- This creates distinct abstract objects for `malloc()` called from different contexts
- Allows distinguishing "buffer allocated in `create_input()` vs `create_output()`"

## Implementation Phases

### Phase 1: Interprocedural Parameter Passing (fixes E10 limitation)

**Goal:** Model argument→parameter and return→caller copy constraints at call sites in PTA constraint extraction.

**Changes:**
- Add `InterCallConstraint` and `InterRetConstraint` types to `pta/constraint.rs`
- Extend `extract_instruction()` in `pta/extract.rs` to generate interprocedural constraints at `CallDirect` and `CallIndirect` sites
- Requires resolving callee function parameters from the `AirModule` function list
- For `CallIndirect`: use call graph (if available) or skip (CI PTA can't resolve without prior analysis)
- Update existing CI solver to process interprocedural constraints
- **Tests:** Unit tests for constraint extraction at call sites; E2E test with interprocedural pointer flow

### Phase 2: Context Type and Operations

**Goal:** Define `CallSiteContext` with deterministic ordering and k-bounded operations.

**Files:** `crates/saf-analysis/src/cspta/context.rs`

**Types:**
- `CallSiteContext` — Vec<InstId> with push/truncate/is_empty/len
- `CallSiteContext::push(site: InstId, k: u32) -> CallSiteContext` — append and truncate oldest
- `CallSiteContext::empty() -> CallSiteContext`
- `CallSiteContext::pop() -> (CallSiteContext, Option<InstId>)`
- Implement `Ord` for deterministic BTreeMap keys

**Tests:** Unit tests for push, truncate, pop, ordering, equality, empty context

### Phase 3: CS-PTA Configuration and Result Types

**Goal:** Define configuration and result types for context-sensitive analysis.

**Files:** `crates/saf-analysis/src/cspta/mod.rs`, `crates/saf-analysis/src/cspta/config.rs`, `crates/saf-analysis/src/cspta/result.rs`

**Types:**
- `CsPtaConfig` — k, field_sensitivity, max_iterations, max_objects, serde support
- `CtxValue` — (ValueId, CallSiteContext) pair with Ord
- `CsPtaResult` — context-sensitive and CI-summary points-to maps, location storage, diagnostics
  - `points_to(ptr, ctx) -> Vec<LocId>` — context-specific query
  - `points_to_any(ptr) -> Vec<LocId>` — CI summary query
  - `may_alias(p, p_ctx, q, q_ctx) -> AliasResult` — context-qualified alias
  - `may_alias_any(p, q) -> AliasResult` — CI summary alias
  - `contexts_for(value) -> Vec<CallSiteContext>` — enumerate contexts
  - `diagnostics() -> &CsPtaDiagnostics`
- `CsPtaDiagnostics` — iteration count, context count, max pts set size, SCC function count

**Tests:** Unit tests for result queries, diagnostics, empty results

### Phase 4: Context-Sensitive Solver

**Goal:** Implement the k-CFA worklist solver.

**Files:** `crates/saf-analysis/src/cspta/solver.rs`

**Algorithm:**
1. Take `AirModule`, `CallGraph`, `CsPtaConfig` as input
2. Extract base + interprocedural constraints
3. Compute SCCs on call graph for recursive function detection
4. Build function→parameters map and call-site→callee map from AIR
5. Initialize context-qualified Addr constraints with empty context
6. Worklist iteration with context-qualified propagation:
   - Intraprocedural edges: same context
   - Call edges: push call-site ID, truncate to k
   - Return edges: inherit caller's context
   - SCC functions: always use empty context
7. Fixed-point convergence
8. Build CI summary by unioning across contexts

**Key implementation details:**
- Worklist: `BTreeSet<CtxValue>` for deterministic pop order
- Points-to map: `BTreeMap<CtxValue, BTreeSet<LocId>>`
- Location points-to: `BTreeMap<(LocId, CallSiteContext), BTreeSet<LocId>>`
- Heap cloning: deterministic ObjId derivation from (InstId, Context)

**Tests:** Unit tests with hand-crafted constraints showing context separation works; test SCC collapse; test k=1 vs k=2 precision differences

### Phase 5: Export

**Goal:** JSON export of context-sensitive PTA results.

**Files:** `crates/saf-analysis/src/cspta/export.rs`

**Schema:**
```json
{
  "schema_version": "0.1.0",
  "config": { "k": 1, ... },
  "diagnostics": { ... },
  "contexts": [
    { "value": "0x...", "context": ["0x...", ...], "points_to": ["0x...", ...] }
  ],
  "ci_summary": {
    "points_to": [
      { "value": "0x...", "locations": ["0x...", ...] }
    ]
  }
}
```

**Tests:** Export roundtrip, determinism

### Phase 6: E2E Tests (C/C++/Rust)

**Goal:** End-to-end tests from real source code demonstrating context-sensitive precision improvements.

**Test programs** (6 programs, compiled to LLVM IR):

1. **`cspta_wrapper_dispatch.c`** — `my_alloc()` wrapper called from two sites; k=1 distinguishes the two allocations
2. **`cspta_identity_function.c`** — Identity function `id(p)` called with different pointers; CI merges, CS separates
3. **`cspta_nested_wrappers.c`** — Two-level wrapper chain (`alloc_buffer` → `safe_malloc` → `malloc`); k=2 needed for full separation
4. **`cspta_recursive_list.c`** — Recursive linked-list traversal; SCC collapse ensures termination
5. **`cspta_cpp_factory.cpp`** — C++ factory pattern with multiple `new` calls through a shared constructor; CS separates allocations per factory call
6. **`cspta_rust_generic.rs`** — Rust unsafe code with generic-like wrapper functions; CS distinguishes call sites

**Rust E2E tests:** ~12 tests covering build, context separation, SCC handling, precision comparison (CS vs CI), determinism, k=1 vs k=2

### Phase 7: Python Bindings

**Goal:** Expose CS-PTA to Python API.

**Files:** `crates/saf-python/src/cspta.rs`, updates to `project.rs`

**Python API:**
```python
# On Project
result = project.context_sensitive_pta(k=1)  # or k=2, k=3

# On CsPtaResult
result.points_to("0x...", context=["0x..."])     # context-specific
result.points_to_any("0x...")                     # CI summary
result.may_alias("0x...", ctx1, "0x...", ctx2)    # context-qualified
result.may_alias_any("0x...", "0x...")             # CI summary
result.contexts_for("0x...")                       # enumerate contexts
result.diagnostics                                 # analysis stats
result.export()                                    # JSON dict
```

**Python E2E tests:** ~12 tests for basic usage, context queries, CI summary, diagnostics, export

### Phase 8: Tutorial

**Goal:** Tutorial demonstrating CS-PTA precision improvement on a real-world pattern.

**Directory:** `tutorials/pta/08-context-sensitive-pta/`

**Scenario:** A memory pool allocator with wrapper functions — `pool_alloc()` called from `create_request()` and `create_response()`. Without CS, both point to the same abstract location. With CS, they're separated, enabling the checker framework to distinguish request vs response memory.

**Files:**
- `vulnerable.c` — Pool allocator with use-after-free that CS-PTA can disambiguate
- `detect.py` — Python script comparing CI vs CS results, showing precision improvement
- `detect.rs` — Rust equivalent
- `README.md` — Walkthrough explaining context sensitivity and its impact

### Phase 9: Documentation Updates

**Goal:** Update all tracking documents.

**Updates:**
- `docs/tool-comparison.md`: Mark Context-Sensitive PTA as implemented in SAF; update Impact text in Section 1; update Summary section
- `plans/PROGRESS.md`: Add E16 epic, Plan 030, update Next Steps, append Session Log
- `plans/FUTURE.md`: Update Context-Sensitive PTA entry to reflect implementation; add Demand-Driven Context-Sensitive PTA as next future extension; note interprocedural parameter passing is now fixed

## Test Strategy

### Unit Tests (Phases 2-5)
- Context operations: push, truncate, pop, ordering (~10 tests)
- Config serialization, result queries (~8 tests)
- Solver: hand-crafted constraints with context separation (~12 tests)
- Export: determinism, schema compliance (~4 tests)

### E2E Tests (Phase 6-7)
- 6 source programs (4 C, 1 C++, 1 Rust) compiled to LLVM IR
- ~12 Rust E2E tests (context separation, SCC, precision comparison, determinism)
- ~12 Python E2E tests (API surface, context queries, CI summary, export)

### Tutorial Verification (Phase 8)
- Full Docker E2E: compile → analyze → compare CI vs CS results

**Total estimated tests:** ~58 new tests

## Dependencies

- **Existing:** PTA (E4), CallGraph (E3), CG Refinement (E10), LLVM frontend (E2)
- **New:** Interprocedural parameter passing (Phase 1 of this plan)
- **No new external dependencies** — uses existing BTreeMap/BTreeSet, BLAKE3

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Context explosion at high k | Default k=1; k=3 documented as slow for large programs |
| Recursive functions cause infinite contexts | SCC collapse to empty context (Phase 4) |
| Interprocedural parameter passing increases CI PTA analysis time | New constraints are proportional to #call-sites × #params — bounded |
| Heap cloning creates too many objects | Bound heap-cloned objects by max_objects config |
| Existing tests break from interprocedural parameter changes | Phase 1 updates existing PTA tests to expect richer results |
