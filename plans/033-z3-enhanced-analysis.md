# Plan 033 — E19: Z3-Enhanced Analysis

## Overview

Extend Z3 SMT solver usage beyond E18's checker framework to enhance precision
across all SAF analysis pillars. Z3 filtering is always **opt-in** via per-method
`z3_refine=True` parameters, preserving backward compatibility.

**Epic:** E19: Z3-Enhanced Analysis
**Depends on:** E18 (Z3 infrastructure), E9 (IFDS), E5 (ValueFlow), E17 (IDE Typestate), E15 (Abstract Interpretation), E4 (PTA)

## Scope — 7 Features Across 3 Categories

### Category A — Trace-based Z3 filtering (reuses E18 pattern)

1. **IFDS taint Z3 refinement** — Reconstruct source→sink witness paths from IFDS
   results, extract guards along paths, filter infeasible taint flows
2. **ValueFlow taint Z3 refinement** — Extract guards along existing BFS taint_flow
   paths, filter infeasible flows

### Category B — Dominator-based Z3 filtering (new technique)

3. **Typestate Z3 refinement** — Collect branch guards that dominate finding
   instructions, verify typestate violations are on feasible paths
4. **Numeric checker Z3 refinement** — Collect dominating guards at GEP/arithmetic
   sites, verify buffer/integer overflow warnings are feasible

### Category C — New Z3-only capabilities

5. **Assertion prover** — Encode `assert()` calls as Z3 queries, prove they hold
   or find counterexamples using dominating guards + interval invariants
6. **Constraint-based alias refinement** — When PTA says may-alias, encode path
   constraints to check if aliasing is feasible on any concrete path
7. **Path-reachability query API** — General Python API: given two program points,
   check if any feasible path connects them via Z3-checked guards

## Architecture

### Shared Infrastructure: `z3_utils` Module

Extract E18's Z3 building blocks into a reusable top-level module:

```
crates/saf-analysis/src/
  z3_utils/
    mod.rs           — Public API, re-exports
    solver.rs        — Z3Solver, FeasibilityResult, PathFeasibilityChecker
                       (moved from checkers/pathsens/z3solver.rs)
    guard.rs         — Guard, PathCondition, OperandInfo, ConditionInfo,
                       ValueLocationIndex, extract_guards()
                       (moved from checkers/pathsens/pathsens.rs)
    dominator.rs     — NEW: DominatingGuards, extract_dominating_guards()
                       Dominator tree computation, dominator-chain guard collection
```

The existing `checkers/pathsens/` becomes a thin wrapper importing from `z3_utils`.

### Dominator-Based Guard Extraction (New Technique)

For point-based analyses (typestate, numeric, assertions) where there is no
source→sink trace, collect guards that **dominate** the finding's block:

1. Compute the dominator tree of the function's CFG using the Cooper-Harvey-Kennedy
   iterative algorithm (already referenced in MSSA phi placement).
2. Walk from the finding's block up to function entry via immediate dominators.
3. At each dominator block with a `CondBr` terminator, determine which branch
   leads toward the finding's block and record the guard.
4. Return a `PathCondition` with these dominating guards.
5. Feed to the same `PathFeasibilityChecker` — same Z3 translation, same
   SAT/UNSAT/timeout classification.

```rust
// New in z3_utils/dominator.rs
pub fn compute_dominators(
    cfg: &Cfg,
    function: &str,
) -> BTreeMap<BlockId, BlockId>  // block → immediate dominator

pub fn extract_dominating_guards(
    block: BlockId,
    function_id: FunctionId,
    cfg: &Cfg,
    dominators: &BTreeMap<BlockId, BlockId>,
    index: &ValueLocationIndex,
) -> PathCondition
```

### API Pattern: Per-Method Z3 Parameters

Every enhanced analysis method follows the same pattern:

```python
# Without Z3 (backward compatible, unchanged):
result = proj.ifds_taint(sources, sinks)

# With Z3 (opt-in):
result = proj.ifds_taint(sources, sinks,
                         z3_refine=True,
                         z3_timeout_ms=1000,
                         max_guards=64)
```

When `z3_refine=True`, the return type changes to include feasible/infeasible/unknown
classification. When `z3_refine=False` (default), behavior is identical to current.

## Detailed Feature Designs

### Feature 1: IFDS Taint Z3 Refinement

**Challenge:** IFDS results are facts-at-points, not source→sink traces.

**Solution:** Add witness path reconstruction:

```rust
// New in ifds/taint.rs
pub struct TaintWitnessPath {
    pub source_inst: InstId,
    pub sink_inst: InstId,
    pub source_value: ValueId,
    pub sink_value: ValueId,
    pub path: Vec<(InstId, TaintFact)>,  // instruction trace
}

pub fn reconstruct_taint_paths(
    result: &IfdsResult<TaintFact>,
    module: &AirModule,
    sink_functions: &BTreeSet<FunctionId>,
) -> Vec<TaintWitnessPath>

pub struct TaintZ3Result {
    pub feasible: Vec<TaintWitnessPath>,
    pub infeasible: Vec<TaintWitnessPath>,
    pub unknown: Vec<TaintWitnessPath>,
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn filter_taint_paths_z3(
    paths: Vec<TaintWitnessPath>,
    module: &AirModule,
    cfg: &Cfg,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> TaintZ3Result
```

**Witness path reconstruction algorithm:**
1. Scan all instructions for calls to sink functions.
2. For each sink call inst where `Tainted(v)` holds (from IFDS result):
   record as candidate finding.
3. Backward BFS from sink fact through IFDS path edges to find
   the source generation point (where Zero → Tainted transition occurred).
4. The path of (inst, fact) pairs forms the witness.

**Python API:**
```python
#[pyo3(signature = (sources, sinks, sanitizers=None, *, z3_refine=false,
#                    z3_timeout_ms=1000, max_guards=64))]
fn ifds_taint(&self, ..., z3_refine: bool, z3_timeout_ms: u64,
              max_guards: usize) -> PyResult<PyObject>
# Returns PyIfdsResult when z3_refine=false
# Returns PyTaintZ3Result when z3_refine=true
```

### Feature 2: ValueFlow Taint Z3 Refinement

**Simpler** — `taint_flow()` already returns `Vec<Finding>` with `trace: Vec<TraceStep>`.

```rust
// New in valueflow/query.rs
pub struct TaintFlowZ3Result {
    pub feasible: Vec<Finding>,
    pub infeasible: Vec<Finding>,
    pub unknown: Vec<Finding>,
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn taint_flow_z3(
    graph: &ValueFlowGraph,
    sources: &[NodeId], sinks: &[NodeId],
    sanitizers: &[NodeId],
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
    limit: usize,
) -> TaintFlowZ3Result
```

**Guard extraction from ValueFlow traces:**
1. Each `TraceStep` has a `node_id: NodeId`.
2. Map `NodeId::Value(vid)` → block via `ValueLocationIndex`.
3. Walk consecutive trace steps, detect block crossings with `CondBr`.
4. Extract guards exactly as E18 does for SVFG traces.

**Python API:**
```python
#[pyo3(signature = (sources, sinks, sanitizers=None, *, z3_refine=false,
#                    z3_timeout_ms=1000, max_guards=64))]
fn taint_flow(&self, ..., z3_refine: bool, ...) -> PyResult<PyObject>
```

### Feature 3: Typestate Z3 Refinement

**Uses dominator-based guards.**

```rust
// New in ifds/typestate.rs or z3_utils/
pub struct TypestateZ3Result {
    pub feasible: Vec<TypestateFinding>,
    pub infeasible: Vec<TypestateFinding>,
    pub unknown: Vec<TypestateFinding>,
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn filter_typestate_z3(
    findings: &[TypestateFinding],
    module: &AirModule,
    cfg: &Cfg,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> TypestateZ3Result
```

**For ErrorState findings:**
- Dominating guards at the error-triggering instruction must be satisfiable.

**For NonAcceptingAtExit findings (leaks):**
- Dominating guards at the function exit block must be satisfiable
  AND no dominating guard forces execution through the closing call.

**Python API:**
```python
#[pyo3(signature = (spec, *, z3_refine=false, z3_timeout_ms=1000, max_guards=64))]
fn typestate(&self, spec: &str, z3_refine: bool, ...) -> PyResult<PyObject>
```

### Feature 4: Numeric Checker Z3 Refinement

**Uses dominator-based guards + interval constraints.**

```rust
pub struct NumericZ3Result {
    pub confirmed: Vec<NumericFinding>,   // SAT → real bug
    pub refuted: Vec<NumericFinding>,     // UNSAT → false positive
    pub uncertain: Vec<NumericFinding>,   // Timeout → keep warning
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn check_numeric_z3(
    module: &AirModule,
    cfg: &Cfg,
    checker: NumericCheckerKind,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> NumericZ3Result
```

**Z3 encoding for numeric findings:**

For buffer overflow: `dominating_guards AND (index < 0 OR index >= alloc_size)`
For integer overflow: `dominating_guards AND (result > MAX_VAL OR result < MIN_VAL)`

- SAT → the overflow is feasible (confirmed bug).
- UNSAT → the overflow cannot happen given path constraints (false positive from
  widening or imprecise join).
- Timeout → keep as Warning.

**Python API:**
```python
#[pyo3(signature = (name, *, z3_refine=false, z3_timeout_ms=1000, max_guards=64))]
fn check_numeric(&self, name: &str, z3_refine: bool, ...) -> PyResult<PyObject>
```

### Feature 5: Assertion Prover

```rust
pub struct AssertionFinding {
    pub function: String,
    pub inst: InstId,
    pub condition_desc: String,     // human-readable assertion text
    pub status: AssertionStatus,    // Proven, MayFail, Unknown
    pub counterexample: Option<BTreeMap<String, i64>>,  // Z3 model for MayFail
}

pub enum AssertionStatus {
    Proven,    // UNSAT(NOT cond) → assertion always holds
    MayFail,   // SAT(NOT cond) → counterexample exists
    Unknown,   // Timeout
}

pub struct AssertionResult {
    pub proven: Vec<AssertionFinding>,
    pub may_fail: Vec<AssertionFinding>,
    pub unknown: Vec<AssertionFinding>,
    pub diagnostics: AssertionDiagnostics,
}

pub fn prove_assertions(
    module: &AirModule,
    cfg: &Cfg,
    absint_result: Option<&AbstractInterpResult>,
    z3_timeout_ms: u64,
    max_guards: usize,
    assert_functions: &[String],
) -> AssertionResult
```

**Algorithm:**
1. Scan AIR for calls to assertion functions.
2. Identify the assert condition: the `CondBr` that guards the call to
   `__assert_fail` / `abort`. The condition is the ICmp feeding the branch.
3. Collect dominating guards at the assertion block.
4. If `absint_result` provided: add interval constraints for all values with
   known bounds (`lo ≤ v ≤ hi`) in the abstract state at the assertion point.
5. Encode: `dominating_guards AND interval_constraints AND NOT(assertion_cond)`.
6. SAT → assertion may fail; extract Z3 model as counterexample.
   UNSAT → assertion always holds (proven). Timeout → unknown.

**Python API:**
```python
#[pyo3(signature = (*, z3_timeout_ms=1000, max_guards=64,
#                    assert_functions=None, use_intervals=false))]
fn prove_assertions(&self, ...) -> PyAssertionResult
```

### Feature 6: Constraint-Based Alias Refinement

```rust
pub enum AliasRefinement {
    ConfirmedAlias,   // SAT → aliasing is feasible
    NoAlias,          // UNSAT → aliasing is infeasible on any feasible path
    Unknown,          // Timeout
}

pub struct AliasRefinementResult {
    pub result: AliasRefinement,
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn refine_alias(
    p: ValueId,
    q: ValueId,
    at_inst: InstId,  // program point where aliasing matters
    module: &AirModule,
    cfg: &Cfg,
    pta: &PtaResult,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> AliasRefinementResult
```

**Algorithm:**
1. Get points-to sets: `pts(p)` and `pts(q)` from PTA.
2. If `pts(p) ∩ pts(q) = ∅`: return `NoAlias` (no Z3 needed).
3. For each overlapping location `loc ∈ pts(p) ∩ pts(q)`:
   - Find the allocation instruction for `loc`.
   - Collect dominating guards at `at_inst`.
   - Collect dominating guards at the allocation instruction.
   - Encode: `all_guards AND (p points to loc) AND (q points to loc)`.
4. If ANY encoding is SAT: `ConfirmedAlias`. If ALL are UNSAT: `NoAlias`.

**Python API:**
```python
#[pyo3(signature = (p, q, at_inst, *, z3_timeout_ms=1000, max_guards=64))]
fn refine_alias(&self, p: &str, q: &str, at_inst: &str, ...) -> PyAliasRefinementResult
```

### Feature 7: Path-Reachability Query API

```rust
pub enum PathReachability {
    Reachable(Vec<BlockId>),    // SAT, with witness path
    Unreachable,                // All paths UNSAT
    Unknown,                    // Timeout / too many paths
}

pub struct PathReachabilityResult {
    pub result: PathReachability,
    pub paths_checked: usize,
    pub diagnostics: Z3FilterDiagnostics,
}

pub fn check_path_reachable(
    from_inst: InstId,
    to_inst: InstId,
    module: &AirModule,
    cfg: &Cfg,
    z3_timeout_ms: u64,
    max_guards: usize,
    max_paths: usize,
) -> PathReachabilityResult
```

**Algorithm:**
1. Find blocks containing `from_inst` and `to_inst`.
2. Enumerate CFG paths from `from_block` to `to_block` (BFS with depth limit).
3. For each path, extract branch guards at every `CondBr` along the path.
4. Z3 check: is the conjunction of all guards satisfiable?
5. If any path is SAT → `Reachable(witness_path)`.
6. If all paths UNSAT → `Unreachable`.
7. If max_paths exceeded or timeout → `Unknown`.

**Python API:**
```python
#[pyo3(signature = (from_inst, to_inst, *, z3_timeout_ms=1000,
#                    max_guards=64, max_paths=100))]
fn check_path_reachable(&self, from_inst: &str, to_inst: &str, ...) -> PyPathReachabilityResult
```

## Implementation Phases

### Phase 1: Shared Infrastructure

**Goal:** Extract reusable Z3 module, add dominator computation.

1. Create `crates/saf-analysis/src/z3_utils/mod.rs` — public API.
2. Move `Z3Solver`, `FeasibilityResult`, `PathFeasibilityChecker` from
   `checkers/pathsens/z3solver.rs` → `z3_utils/solver.rs`.
3. Move `Guard`, `PathCondition`, `OperandInfo`, `ConditionInfo`,
   `ValueLocationIndex`, `extract_guards()` from `checkers/pathsens/pathsens.rs`
   → `z3_utils/guard.rs`.
4. Add shared `Z3FilterDiagnostics` type (reusable across all features).
5. Refactor `checkers/pathsens/` to import from `z3_utils`. Run E18 tests — no
   behavior change.
6. Add `compute_dominators()` to `graph_algo` module.
   - Cooper-Harvey-Kennedy iterative algorithm.
   - Unit tests: linear CFG, diamond CFG, loop CFG, nested loops.
7. Add `extract_dominating_guards()` to `z3_utils/dominator.rs`.
   - Walk dominator chain, collect CondBr guards.
   - Unit tests: single guard, multiple guards, no guards (entry block).
8. Verify: all E18 Rust and Python tests still pass.

### Phase 2: IFDS Taint Z3 Refinement

**Goal:** Witness path reconstruction + Z3 filtering for IFDS taint.

1. Add `TaintWitnessPath` struct in `ifds/taint.rs`.
2. Implement `reconstruct_taint_paths()`:
   - Scan for sink call instructions where tainted fact holds.
   - Backward BFS through IFDS path edges to source generation.
   - Unit tests: simple path, interprocedural path, no path found.
3. Implement `filter_taint_paths_z3()`:
   - Map witness path instructions to blocks.
   - Extract guards along block sequence.
   - Z3 feasibility check per path.
   - Unit tests: feasible path, infeasible (contradictory guards), empty guards.
4. Add `TaintZ3Result` and `PyTaintZ3Result` types.
5. Update Python `ifds_taint()` to accept `z3_refine`, `z3_timeout_ms`, `max_guards`.
6. E2E tests: 2-3 C source programs compiled to LLVM IR:
   - `ifds_z3_correlated_branch.c` — taint flow through correlated branches
     (same flag guards source read and sink write → FP filtered)
   - `ifds_z3_genuine_taint.c` — genuine taint flow that Z3 confirms as feasible
   - `ifds_z3_interproc.c` — interprocedural taint with infeasible call sequence

### Phase 3: ValueFlow Taint Z3 Refinement

**Goal:** Z3 filtering for ValueFlow BFS taint paths.

1. Implement `taint_flow_z3()` in `valueflow/query.rs`:
   - Take existing `Finding` with trace, map `TraceStep` node IDs to blocks.
   - Extract guards between consecutive trace steps crossing block boundaries.
   - Z3 feasibility check per finding.
2. Add `TaintFlowZ3Result` and Python `PyTaintFlowZ3Result`.
3. Update Python `taint_flow()` to accept Z3 parameters.
4. E2E tests: 2 C source programs:
   - `vf_z3_sanitized_path.c` — taint sanitized on one branch but ValueFlow
     reports flow on merged path → Z3 filters
   - `vf_z3_confirmed_flow.c` — genuine taint confirmed by Z3

### Phase 4: Typestate Z3 Refinement

**Goal:** Dominator-based Z3 filtering for typestate findings.

1. Implement `filter_typestate_z3()`:
   - For ErrorState: collect dominating guards at finding inst, check feasibility.
   - For NonAcceptingAtExit: collect dominating guards at exit block, check that
     path to exit without closing call is feasible.
2. Add `TypestateZ3Result` and Python `PyTypestateZ3Result`.
3. Update Python `typestate()` and `typestate_custom()` to accept Z3 parameters.
4. E2E tests: 2-3 C/C++ source programs:
   - `ts_z3_guarded_close.c` — file opened and closed under same guard → leak FP filtered
   - `ts_z3_genuine_leak.c` — genuine file leak confirmed by Z3
   - `ts_z3_error_path.cpp` — C++ RAII with error-guarded resource → FP filtered

### Phase 5: Numeric Checker Z3 Refinement

**Goal:** Z3 verification for interval-based numeric findings.

1. Implement `check_numeric_z3()`:
   - Run standard interval analysis.
   - For Warning-severity findings: encode `dominating_guards AND overflow_condition`.
   - SAT → confirmed, UNSAT → refuted, timeout → uncertain.
2. Add `NumericZ3Result` and Python `PyNumericZ3Result`.
3. Update Python `check_numeric()` and `check_all_numeric()` to accept Z3 params.
4. E2E tests: 2-3 C source programs:
   - `num_z3_loop_widening.c` — loop counter widened to [0, TOP] but guard
     ensures idx < N → buffer overflow FP refuted by Z3
   - `num_z3_genuine_overflow.c` — unchecked multiplication confirmed by Z3
   - `num_z3_branch_narrowing.c` — branch narrows value but interval doesn't
     capture it → Z3 refutes overflow warning

### Phase 6: Assertion Prover

**Goal:** Prove or disprove `assert()` calls using Z3.

1. Add `AssertionFinding`, `AssertionStatus`, `AssertionResult` types.
2. Implement assertion condition extraction:
   - Scan for calls to configurable assertion functions.
   - Map assertion call → controlling `CondBr` → ICmp condition.
3. Implement `prove_assertions()`:
   - Collect dominating guards.
   - Optionally add interval constraints from abstract interpretation.
   - Encode: `guards AND intervals AND NOT(assertion_cond)`.
   - Extract Z3 model as counterexample for MayFail cases.
4. Add Python `PyAssertionResult`, `PyAssertionFinding`.
5. Add `Project.prove_assertions()` Python method.
6. E2E tests: 2-3 C source programs:
   - `assert_z3_provable.c` — assertions that are always true (modular arithmetic,
     validated input) → Z3 proves them
   - `assert_z3_failing.c` — assertions that can fail (unchecked user input) →
     Z3 finds counterexample
   - `assert_z3_with_intervals.c` — assertion provable only with interval
     invariants (loop bound) → demonstrates interval + Z3 synergy

### Phase 7: Constraint-Based Alias Refinement

**Goal:** Z3-based may-alias refinement.

1. Add `AliasRefinement`, `AliasRefinementResult` types.
2. Implement `refine_alias()`:
   - Get PTA points-to sets intersection.
   - For overlapping locations: collect dominating guards at use point and
     allocation points.
   - Z3 check: is aliasing feasible under combined guards?
3. Add Python `PyAliasRefinementResult`.
4. Add `Project.refine_alias()` Python method.
5. E2E tests: 2 C source programs:
   - `alias_z3_disjoint_paths.c` — two pointers assigned on mutually exclusive
     branches → PTA says may-alias, Z3 proves no-alias
   - `alias_z3_confirmed.c` — genuine aliasing confirmed by Z3

### Phase 8: Path-Reachability Query API

**Goal:** General feasible path-reachability queries.

1. Add `PathReachability`, `PathReachabilityResult` types.
2. Implement `check_path_reachable()`:
   - Enumerate CFG paths (BFS with depth/count limits).
   - Extract guards per path.
   - Z3 check per path; return first SAT as witness.
3. Add Python `PyPathReachabilityResult`.
4. Add `Project.check_path_reachable()` Python method.
5. E2E tests: 2 C source programs:
   - `reach_z3_infeasible.c` — two points connected by CFG path but guarded
     by contradictory conditions → Z3 proves unreachable
   - `reach_z3_feasible.c` — reachable path confirmed with witness

### Phase 9: Tutorials (3 tutorials + README.md each)

**Tutorial 09:** `tutorials/checkers/09-z3-taint-refinement/`
- `vulnerable.c` — HTTP request handler with type-dependent sanitization.
  GET/POST branches have different sanitization; path-insensitive reports
  cross-branch flow (FP). Z3 filters it. Plus genuine taint on error path.
- `detect.py` — End-to-end: compile → load → IFDS taint → Z3-refined IFDS
  taint → compare results.
- `detect.rs` — Rust E2E: same pipeline using Rust API directly.
- `README.md` — Explains the scenario, what Z3 adds, how to run both scripts.

**Tutorial 10:** `tutorials/checkers/10-z3-assertion-prover/`
- `vulnerable.c` — Ring buffer with assert() calls. Some provable (modular
  arithmetic ensures bounds), some may fail (unchecked user input).
- `detect.py` — End-to-end: compile → load → prove_assertions() → show
  proven/may_fail/unknown with counterexamples.
- `detect.rs` — Rust E2E: same pipeline using Rust API directly.
- `README.md` — Explains assertion proving concept, how to run both scripts.

**Tutorial 11:** `tutorials/checkers/11-z3-analysis-comparison/`
- `vulnerable.c` — File processing pipeline exercising all analysis types:
  correlated malloc/free (leak FP), guarded file ops (typestate FP), loop
  with widened index (numeric FP), validated-path taint (taint FP). Plus one
  genuine bug per category.
- `detect.py` — End-to-end: compile → run all 4 analyses with and without
  Z3 → side-by-side comparison table.
- `detect.rs` — Rust E2E: same multi-analysis comparison using Rust API.
- `README.md` — Comprehensive comparison guide, how to run both scripts.

All tutorials:
- Include both `detect.py` (Python API) and `detect.rs` (Rust API)
- Compile from C source via `clang-18` inside Docker
- No pre-built fixtures or artificial test data
- Follow established tutorial pattern (01–08)

### Phase 10: Documentation Updates

1. Update `plans/PROGRESS.md`:
   - Add E19 to Epics list
   - Add Plan 033 to Plans Index
   - Update Next Steps
   - Add Session Log entry
2. Update `plans/FUTURE.md`:
   - Update relevant extension point entries
   - Add log entry
3. Update `docs/tool-comparison.md`:
   - Update Data-Flow Analysis section (Z3 refinement capability)
   - Update Bug Detection section (path-sensitive across all checkers)
   - Update Summary section
   - Mark new capabilities as non-gapped

## Test Strategy

### Unit Tests (per phase)
- Dominator computation: 4-6 CFG topologies
- Dominating guard extraction: various dominator chain shapes
- Witness path reconstruction: simple/interprocedural/empty cases
- Z3 encoding for each feature's constraint shapes

### E2E Tests (from compiled source)
Each feature has 2-3 purpose-built C/C++ source programs compiled via
`clang-18 -S -emit-llvm -O0`. Tests verify:
- Z3 correctly classifies feasible vs infeasible findings
- Results are deterministic (run twice, compare)
- Backward-compatible (z3_refine=false gives same result as before)
- Python API works end-to-end

### Total estimated tests
- ~15-20 unit tests (dominator + guard extraction + each feature's encoding)
- ~15-20 Rust E2E tests (from compiled C/C++ programs)
- ~20-25 Python E2E tests (Python bindings for all 7 features)
- 3 tutorials verified end-to-end in Docker

## Python API Summary

| Method | New Parameters | Z3 Result Type |
|--------|---------------|----------------|
| `ifds_taint()` | `z3_refine, z3_timeout_ms, max_guards` | `TaintZ3Result` |
| `taint_flow()` | `z3_refine, z3_timeout_ms, max_guards` | `TaintFlowZ3Result` |
| `typestate()` | `z3_refine, z3_timeout_ms, max_guards` | `TypestateZ3Result` |
| `typestate_custom()` | `z3_refine, z3_timeout_ms, max_guards` | `TypestateZ3Result` |
| `check_numeric()` | `z3_refine, z3_timeout_ms, max_guards` | `NumericZ3Result` |
| `check_all_numeric()` | `z3_refine, z3_timeout_ms, max_guards` | `NumericZ3Result` |
| `prove_assertions()` | `z3_timeout_ms, max_guards, assert_functions, use_intervals` | `AssertionResult` (new) |
| `refine_alias()` | `z3_timeout_ms, max_guards` | `AliasRefinementResult` (new) |
| `check_path_reachable()` | `z3_timeout_ms, max_guards, max_paths` | `PathReachabilityResult` (new) |

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Per-method z3 parameters (not config object) | Consistent with E18 pattern; explicit, simple, no new types |
| Extract z3_utils from checkers/pathsens/ | Avoids code duplication; single Z3 integration point |
| Dominator-based guards for point analyses | Natural generalization; dominators = "must be true on all paths to point" |
| Cooper-Harvey-Kennedy dominators | Simple iterative algorithm; already referenced in MSSA code |
| IFDS witness path reconstruction | Necessary to bridge IFDS facts-at-points to guard extraction |
| Optional interval integration for assertions | Synergy between E15 and E19; intervals tighten Z3 constraints |
| Conservative defaults (z3_refine=false) | Backward compatible; Z3 is always opt-in |
| Counterexample extraction for assertions | Z3 model provides concrete failing inputs — valuable for debugging |
