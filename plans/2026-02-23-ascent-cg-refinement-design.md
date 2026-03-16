# Design: Ascent-Based CG Refinement

**Date:** 2026-02-23
**Epic:** Scalability
**Status:** Proposed

## Problem

The current Ascent pipeline runs **two full PTA solves** on bash (CruxBC):

1. Legacy worklist PTA inside `refine()` for CG refinement — **16.7s**
2. Ascent solver standalone on **unrefined** constraints — **18.4s**

Total: **38.2s**. The legacy PTA result is discarded when Ascent is selected.

## Solution

Replace the legacy worklist solver inside `refine()` with iterative Ascent calls, selected at runtime via `PtaSolver` enum. Split `refine()` into solver-agnostic phases and a solver-specific PTA+CG loop.

## Architecture

### Split `refine()` into three phases

**Phase 1 — `refine_prepare()`** (solver-agnostic):
- Build CHA + initial CallGraph
- Bootstrap virtual calls via CHA
- Collect indirect call sites, return values, function-location map
- Extract ALL constraints + spec constraints
- HVN preprocessing
- Returns `RefinementPrepared` intermediate state

**Phase 2 — Solver-specific PTA + CG loop** (caller-orchestrated):
- **Legacy path**: `GenericSolver::new()` → `solve()` → `resolve_and_connect()` → `drain_worklist()` loop (existing code, extracted into `refine_legacy()`)
- **Ascent path**: new `refine_with_ascent()` in `saf-datalog` — iterative `ascent_solve_incremental()` → resolve targets from pts → add copy constraints → re-solve loop

**Phase 3 — `refine_finalize()`** (solver-agnostic):
- Normalize PTA results (HVN expansion)
- CHA/PTA resolution narrowing
- Build PtaResult + ICFG
- Returns `RefinementResult`

### New types

```rust
// In saf-core (move from saf-bench)
pub enum PtaSolver {
    /// Worklist-based imperative solver
    #[default]
    Ascent,
    /// Legacy worklist solver
    Legacy,
}

// In saf-analysis::cg_refinement
pub struct RefinementPrepared {
    pub cg: CallGraph,
    pub cha: Option<ClassHierarchy>,
    pub cha_resolved_sites: BTreeSet<InstId>,
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    pub constraints: ConstraintSet,
    pub reduced: ConstraintSet,       // post-HVN
    pub hvn_result: HvnResult,
    pub factory: LocationFactory,
    pub func_loc_map: FunctionLocationMap,
    pub indirect_sites: Vec<IndirectCallSite>,
    pub return_values: BTreeMap<FunctionId, Vec<ValueId>>,
    pub constraint_counts: [usize; 5],
    pub post_hvn_constraint_counts: [usize; 5],
}

// Solver-agnostic PTA loop output
pub struct PtaSolveResult {
    pub pts: PointsToMap,
    pub factory: LocationFactory,
    pub resolved_calls: BTreeMap<InstId, BTreeSet<FunctionId>>,
    pub iterations: usize,
    pub pta_solve_secs: f64,
    pub iteration_limit_hit: bool,
}
```

### Ascent CG refinement loop

New function in `saf-datalog::pta::context`:

```rust
pub fn refine_with_ascent(
    module: &AirModule,
    prepared: &mut RefinementPrepared,
    max_iterations: usize,
) -> PtaSolveResult
```

Algorithm:
1. Convert `prepared.reduced` (post-HVN constraints) to Datalog facts
2. Apply SCC detection + rewriting
3. Run initial Ascent solve (with iterative GEP resolution)
4. Resolve indirect call targets from points-to results
5. If new targets found: add interprocedural copy constraints to **facts** (not full constraint reconversion), re-run Ascent solve **without** re-doing HVN/SCC
6. Repeat steps 4-5 until fixpoint or `max_iterations`
7. Expand HVN + SCC back to original ValueIds
8. Return `PtaSolveResult`

### Incremental re-solve (skip HVN/SCC)

Key optimization: on CG refinement iterations 2+, we only add `(dst, src)` copy facts. These don't create new HVN equivalences (HVN merges values with identical constraint signatures — interprocedural copies have unique dst/src pairs) or new copy-graph SCCs (the new edges connect previously-disconnected subgraphs).

New function in `saf-datalog::pta::solver`:

```rust
pub fn ascent_solve_incremental(
    facts: &PtaFacts,
    additional_copies: &[(ValueId, ValueId)],
) -> PointsToMap
```

This appends `additional_copies` to the fact set and re-runs the Ascent fixpoint without HVN/SCC preprocessing. The GEP resolution phase is also skipped (GEP facts don't change between CG iterations — only copy constraints are added).

### Solver-agnostic indirect call resolution

Extract the "read PTS → discover targets → collect copy constraints" logic from `resolve_and_connect()` into a solver-agnostic function:

```rust
pub fn resolve_indirect_calls_from_pts(
    pts: &PointsToMap,
    factory: &LocationFactory,
    indirect_sites: &[IndirectCallSite],
    func_loc_map: &FunctionLocationMap,
    module: &AirModule,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
    resolved_calls: &mut BTreeMap<InstId, BTreeSet<FunctionId>>,
    cg: &mut CallGraph,
) -> Vec<CopyConstraint>
```

This replaces the current `resolve_and_connect()` which is tightly coupled to `GenericSolver<P>`. The legacy path wraps this with solver PTS access; the Ascent path calls it directly with `PointsToMap`.

### RefinementConfig change

```rust
pub struct RefinementConfig {
    pub max_iterations: usize,
    pub entry_points: EntryPointStrategy,
    pub pta_config: PtaConfig,
    pub field_sensitivity: FieldSensitivity,
    pub pta_solver: PtaSolver,  // NEW — default Ascent
}
```

### Convenience wrapper

For callers that don't need fine-grained control (e2e tests, pipeline.rs), provide:

```rust
pub fn refine(module, config, specs) -> RefinementResult
```

This calls `refine_prepare()`, dispatches to legacy or Ascent based on `config.pta_solver`, then calls `refine_finalize()`. Existing callers are unchanged.

### Dependency flow

```
saf-core:  PtaSolver enum, PtaConfig
saf-analysis:  refine_prepare(), refine_finalize(), refine() wrapper,
               resolve_indirect_calls_from_pts(), legacy PTA loop
saf-datalog:   refine_with_ascent() — calls resolve_indirect_calls_from_pts()
               from saf-analysis (existing dependency direction)
saf-bench:     calls refine_prepare() + refine_with_ascent() + refine_finalize()
               for profiling, or just refine() for simplicity
```

No circular dependencies. `saf-datalog` already depends on `saf-analysis`.

## Expected Performance

**bash CruxBC (Ascent path):**
- Current: 3s load + 16.7s legacy PTA + 18.4s Ascent = **38.2s**
- Expected: 3s load + ~20s Ascent-with-CG-refinement = **~23s**
- Breakdown: 1st full solve ~18s + 1-2 incremental re-solves ~2-4s each
- **~40% improvement**

**PTABen (Ascent path):**
- Current: 65 unsound (no CG refinement in Ascent path)
- Expected: ≤65 unsound (CG refinement adds interprocedural edges → more precise)
- May close some of the 4-unsound gap vs legacy (61)

## Validation

1. PTABen with `--solver datalog`: verify ≤65 unsound (no regression)
2. PTABen with `--solver worklist`: verify 61 unsound (legacy unchanged)
3. CruxBC bash with `--solver datalog`: measure wall-clock improvement
4. `make test`: all 1895 Rust + 81 Python tests pass
