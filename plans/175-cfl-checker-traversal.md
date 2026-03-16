# Plan 175: Context-Sensitive SVFG Checker Traversal (CFL-Reachability)

**Design:** `docs/plans/2026-02-27-cfl-checker-traversal-design.md`

**Epic:** Checker Precision

**Goal:** Eliminate false-positive checker findings caused by interprocedurally unrealizable paths in context-insensitive SVFG traversal.

## Phase 1: Edge Annotation (3 tasks)

### Task 1: Extend `SvfgEdgeKind` with call-site `InstId`

**File:** `crates/saf-analysis/src/svfg/mod.rs`

Change:
```rust
CallArg,              →  CallArg { call_site: InstId },
Return,               →  Return { call_site: InstId },
```

Update `is_direct()` and `name()` to use `CallArg { .. }` and `Return { .. }` patterns.

Update `SvfgEdgeKind` to remove `Copy` derive if it has one (InstId is Copy so this should still work).

Update serde: the `rename_all = "snake_case"` attribute will serialize `CallArg { call_site: ... }` as `{ "call_arg": { "call_site": "0x..." } }`. Verify this is acceptable or add custom serde.

Update all tests in `mod.rs` that reference `SvfgEdgeKind::CallArg` or `SvfgEdgeKind::Return`.

**Tests:** Existing tests updated to compile. Serde round-trip test updated.

### Task 2: Update SVFG builder to pass call-site `InstId`

**File:** `crates/saf-analysis/src/svfg/builder.rs`

In `add_call_edges()`, the `call_site: InstId` parameter already exists. Change:
```rust
SvfgEdgeKind::CallArg       →  SvfgEdgeKind::CallArg { call_site }
SvfgEdgeKind::Return        →  SvfgEdgeKind::Return { call_site }
```

Update test `call_arg_and_return_edges()` to match on `CallArg { .. }` / `Return { .. }`.

**Tests:** `call_arg_and_return_edges` passes with new edge variants.

### Task 3: Update remaining consumers

**Files:**
- `crates/saf-analysis/src/svfg/optimize.rs` — update test edge construction
- `crates/saf-analysis/src/svfg/export.rs` — verify `name()` and serde still work
- `crates/saf-analysis/src/dda/solver.rs` — update `classify_node()` pattern matches; optionally simplify by reading `call_site` directly from the edge instead of `find_call_site_for_param()`

**Tests:** All existing tests compile and pass.

**Validation gate:** `make fmt && make lint && make test` — all pass.

## Phase 2: CallString Relocation (2 tasks)

### Task 4: Create `svfg/context.rs` with `CallString`

**File:** `crates/saf-analysis/src/svfg/context.rs` (new)

Move `CallString` from `dda/types.rs` to `svfg/context.rs`. Keep exact same API:
- `empty()`, `push(InstId)`, `pop()`, `matches(InstId)`, `top()`, `depth()`, `sites()`
- `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default` derives

Add `pub mod context;` to `svfg/mod.rs` and `pub use context::CallString;`.

### Task 5: Update DDA to import from new location

**File:** `crates/saf-analysis/src/dda/types.rs`

Replace `CallString` definition with re-export:
```rust
pub use crate::svfg::context::CallString;
```

Keep `Dpm`, `DdaConfig`, `Budget`, `DdaCache`, `DdaDiagnostics` in `dda/types.rs` (they are DDA-specific).

Move `CallString` tests to `svfg/context.rs`.

**Tests:** All DDA tests pass unchanged.

**Validation gate:** `make fmt && make lint && make test` — all pass.

## Phase 3: CFL-Aware Solvers (4 tasks)

### Task 6: Add `max_context_depth` to `SolverConfig`

**File:** `crates/saf-analysis/src/checkers/solver.rs`

```rust
pub struct SolverConfig {
    pub max_depth: usize,
    pub max_context_depth: usize,  // default: 3
}
```

Default implementation sets `max_context_depth: 3`. Value 0 means disabled (context-insensitive, existing behavior).

### Task 7: CFL-aware `may_reach`

**File:** `crates/saf-analysis/src/checkers/solver.rs`

Changes to `may_reach()`:
1. BFS queue element: `(SvfgNodeId, usize, CallString)` — add context
2. Visited set: `BTreeSet<(SvfgNodeId, CallString)>`
3. Edge loop: match on `SvfgEdgeKind` to push/pop/skip context
4. Parent map: keyed by `(SvfgNodeId, CallString)` for trace reconstruction

CFL rules (from design):
- `CallArg { call_site }`: push if below k-limit, else keep current context
- `Return { call_site }`: pop if matches top, skip if mismatched, allow if empty
- Other edges: context unchanged

When `config.max_context_depth == 0`: skip all CFL logic (backward compatible).

**Tests:**
- Unit test: mismatched return edge → unreachable
- Unit test: matched return edge → reachable
- Unit test: empty context return → allowed (conservative)

### Task 8: CFL-aware `multi_reach`

**File:** `crates/saf-analysis/src/checkers/solver.rs`

Same CFL-aware BFS pattern as Task 7 applied to `multi_reach()`. The sink collection and 2+ threshold logic remain unchanged.

**Tests:**
- Unit test: two allocations through identity function, freed once each → 0 findings
- Unit test: one allocation through identity function, freed twice → 1 finding

### Task 9: CFL-aware `must_not_reach` and `may_reach_guarded`

**File:** `crates/saf-analysis/src/checkers/solver.rs`

Apply same CFL pattern to `must_not_reach()` and `may_reach_guarded()`. For `may_reach_guarded`, context tracking is orthogonal to guard tracking — both are accumulated during BFS.

**Tests:** Existing tests pass with CFL enabled (default k=3).

**Validation gate:** `make fmt && make lint && make test` — all pass.

## Phase 4: Integration Testing (2 tasks)

### Task 10: E2E tests with C test programs

Compile and run checkers on:
- `tests/mytests/checks/df_identity_fn.c` — expect 0 double-free (was 2 false positives)
- `tests/mytests/checks/df_identity_real.c` — expect 1 double-free (should still detect)
- `tests/mytests/checks/df0.c` — existing test, expect same result
- `tests/mytests/checks/df1.c` — existing test, expect same result
- Existing checker e2e tests — no regressions

### Task 11: Update PROGRESS.md

Update `plans/PROGRESS.md`:
- Add Plan 175 to Plans Index with status "done"
- Update session log
- Note CLAUDE.md update if needed (SVFG checker known issues section)

**Validation gate:** Full `make test` pass, no regressions.

## Summary

| Phase | Tasks | Estimated scope |
|-------|-------|----------------|
| 1. Edge Annotation | 1-3 | ~13 call sites, enum change |
| 2. CallString Relocation | 4-5 | Move + re-export, ~100 lines |
| 3. CFL-Aware Solvers | 6-9 | Core change, ~200 lines new logic |
| 4. Integration Testing | 10-11 | E2E verification, docs |

**Total:** 11 tasks, 4 phases.
