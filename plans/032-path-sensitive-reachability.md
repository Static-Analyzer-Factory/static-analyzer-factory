# Plan 032: Path-Sensitive Checker Reachability

**Epic:** E18 — Path-Sensitive Checker Reachability
**Status:** approved
**Depends on:** E14 (Checker Framework), E12 (SVFG)

---

## Overview

Add path-sensitive reachability checking to SAF's E14 checker framework, reducing false positives by filtering findings whose SVFG traces traverse infeasible program paths. Uses a two-stage architecture: Stage 1 runs existing path-insensitive E14 checkers to produce candidate findings; Stage 2 extracts branch guard conditions along each finding's trace, encodes them as Z3 formulas, and checks satisfiability — discarding findings on provably infeasible paths.

This closes the gap identified in `docs/tool-comparison.md` ("Path-sensitive checker reachability — Pinpoint-style guard conditions on SVFG"). Research shows 10x or more FP reduction in academic evaluations (Pinpoint PLDI'18, SMOKE ICSE'19).

## Background & Research

### Pinpoint (PLDI'18, Shi et al.)

Pinpoint uses a Symbolic Expression Graph (SEG) per function — an enriched SVFG where edges carry both data-dependence and control-dependence labels (guard conditions). Guard conditions are extracted from branch predicates. Interprocedural analysis uses "connectors" (call-site parameter/return binding) and memoized per-function summaries. Path feasibility is checked via Z3. Reported ~14-23% false positive rate vs path-insensitive SABER's higher rates. Key innovation: decomposes expensive whole-program PTA into cheap local analysis + on-demand inter-proc queries.

### SMOKE (ICSE'19, Fan et al.)

Two-stage approach: Stage 1 does fast path-insensitive reachability (like SAF's current E14), Stage 2 applies path-sensitive checking only to candidates from Stage 1 using a dedicated lightweight constraint solver. FP rate: 24.4%. 27-106x faster than SABER. The two-stage design is the inspiration for SAF's approach.

### SVF/SABER Current Implementation

SVF's SABER module uses `SaberCondAllocator` to assign Z3 conditions to basic block edges, and `ProgSlice` to propagate conditions along SVFG paths. Conditions are composed via AND/OR/NOT and checked via Z3's `solver.check()`. The key data structure is `BBToCondMap` mapping blocks to accumulated path conditions.

### Infer Pulse

Uses a custom arithmetic solver (not Z3) for path conditions during symbolic execution. Demonstrates that lightweight solvers suffice for many cases, but SAF will use Z3 via the mature `z3` Rust crate for full expressiveness.

### Design Decision: Z3 via `z3` crate

The [`z3` crate](https://github.com/prove-rs/z3.rs) provides high-level Rust bindings (latest: v0.12, actively maintained). Using the `bundled` cargo feature statically compiles Z3, requiring no system dependency. This gives full SMT solving capability without C++ FFI complexity.

## Design

### Architecture: Two-Stage Pipeline

```
Stage 1 (existing E14 — unchanged):
  run_checkers(specs, module, svfg, table, config) → Vec<CheckerFinding>
    Each finding has: source_node, sink_node, trace: Vec<SvfgNodeId>

Stage 2 (new — path feasibility filter):
  For each CheckerFinding:
    1. Walk the trace [source → ... → sink]
    2. At each node, map SvfgNodeId → ValueId → BlockId
    3. Check if block's terminator is CondBr
    4. If so, extract the guard: (condition_value, branch_taken)
    5. Translate the ICmp/BinaryOp condition to Z3 AST
    6. Conjoin all guards into a Z3 formula
    7. Check SAT:
       - SAT → finding is feasible, keep
       - UNSAT → finding is infeasible (false positive), discard
       - UNKNOWN/timeout → conservatively keep
```

**Soundness property:** Stage 2 only removes findings, never adds. UNSAT means no execution can take that path, so filtering preserves soundness.

### 1. Guard Extraction

New types in `checkers/pathsens.rs`:

```rust
/// A guard condition extracted from a CondBr terminator
pub struct Guard {
    pub block: BlockId,
    pub function: FunctionId,
    pub condition: ValueId,     // The ICmp/BinaryOp result feeding CondBr
    pub branch_taken: bool,     // true = then branch, false = else branch
}

/// Collected guards along a checker trace
pub struct PathCondition {
    pub guards: Vec<Guard>,
}
```

**`extract_guards(trace, module) → PathCondition`:**
- For each consecutive pair `(node_i, node_j)` in the trace
- Map each `SvfgNodeId::Value(vid)` to its containing block
- If the block of `node_i` differs from `node_j`'s block (control flow edge)
- Check if `node_i`'s block has a `CondBr` terminator
- If `node_j`'s block is `then_target`: record `Guard { condition, branch_taken: true }`
- If `node_j`'s block is `else_target`: record `Guard { condition, branch_taken: false }`
- Skip `MemPhi` nodes and same-block transitions (no guard)

### 2. Z3 Translation

New module `checkers/z3solver.rs`:

```rust
pub struct PathFeasibilityChecker<'ctx> {
    ctx: &'ctx z3::Context,
    solver: z3::Solver<'ctx>,
    var_cache: BTreeMap<ValueId, z3::ast::Dynamic<'ctx>>,
    timeout_ms: u64,
}

pub enum FeasibilityResult {
    Feasible,       // SAT
    Infeasible,     // UNSAT — false positive
    Unknown,        // Timeout or undecidable
}
```

**AIR → Z3 translation table:**

| AIR Construct | Z3 Representation |
|---|---|
| `Constant::Int(v)` | `z3::ast::Int::from_i64(ctx, v)` |
| `Constant::Null` | `z3::ast::Int::from_i64(ctx, 0)` (pointer-as-int) |
| `ICmpEq(a, b)` | `a._eq(&b)` |
| `ICmpNe(a, b)` | `a._eq(&b).not()` |
| `ICmpSlt(a, b)` | `a.lt(&b)` |
| `ICmpSle(a, b)` | `a.le(&b)` |
| `ICmpSgt(a, b)` | `a.gt(&b)` |
| `ICmpSge(a, b)` | `a.ge(&b)` |
| `ICmpUlt/Ule/Ugt/Uge` | Same as signed (conservative) |
| Unknown/opaque operand | Fresh `z3::ast::Int::new_const(ctx, vid_hex)` |
| Guard with `branch_taken: true` | condition as-is |
| Guard with `branch_taken: false` | `condition.not()` |

**Procedure:**
1. Create Z3 context once per `run_checkers_path_sensitive()` call
2. For each finding, reset solver, translate `PathCondition` guards to Z3 AST
3. Assert conjunction of all guards
4. Call `solver.check()` with timeout
5. Return `FeasibilityResult`

### 3. Path-Sensitive Runner

```rust
pub fn run_checkers_path_sensitive(
    specs: &[&CheckerSpec],
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult

pub struct PathSensitiveConfig {
    pub solver_config: SolverConfig,       // existing E14 config
    pub z3_timeout_ms: u64,                // per-finding timeout (default: 1000)
    pub max_guards_per_trace: usize,       // skip Z3 if too many guards (default: 64)
    pub enabled: bool,                     // toggle path-sensitivity
}

pub struct PathSensitiveResult {
    pub feasible: Vec<CheckerFinding>,
    pub infeasible: Vec<CheckerFinding>,
    pub unknown: Vec<CheckerFinding>,
    pub diagnostics: PathSensitiveDiagnostics,
}

pub struct PathSensitiveDiagnostics {
    pub total_findings: usize,
    pub feasible_count: usize,
    pub infeasible_count: usize,
    pub unknown_count: usize,
    pub guards_extracted: usize,
    pub z3_calls: usize,
    pub z3_timeouts: usize,
}
```

**Also: standalone post-filter:**

```rust
pub fn filter_infeasible(
    findings: &[CheckerFinding],
    module: &AirModule,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult
```

### 4. Python Bindings

New Python API additions:

```python
# Path-sensitive versions of check methods
result = project.check_all_path_sensitive(z3_timeout_ms=1000)
result = project.check_path_sensitive("memory-leak", z3_timeout_ms=1000)

# Post-filter existing findings
ps_result = project.filter_infeasible(findings, z3_timeout_ms=1000)

# Result object
ps_result.feasible       # List[CheckerFinding]
ps_result.infeasible     # List[CheckerFinding]
ps_result.unknown        # List[CheckerFinding]
ps_result.diagnostics    # dict
```

Existing `check()` / `check_all()` remain unchanged (path-insensitive). Path-sensitive checking is opt-in.

### 5. Scope Boundaries

**In scope:**
- Guard extraction from `CondBr` terminators along SVFG traces
- Integer comparison conditions (`ICmpEq/Ne/Slt/Sle/Sgt/Sge/Ult/Ule/Ugt/Uge`)
- Null pointer checks (pointer == NULL modeled as int == 0)
- Z3-based satisfiability checking with timeout
- All 9 existing E14 checkers benefit automatically

**Out of scope (future extensions):**
- `Switch` statement guards (uncommon in typical LLVM IR)
- Loop-dependent guard reasoning (loop-carried values treated as unconstrained — conservative)
- Interprocedural guard correlation (guards in different functions treated independently)
- Guard conditions involving function return values (would need interprocedural summary)
- Floating-point comparisons (`FCmp*` — Z3 supports but adds complexity)

## Implementation Phases

### Phase 1 — Z3 Integration Scaffolding
- Add `z3 = { version = "0.12", features = ["bundled"] }` to `saf-analysis/Cargo.toml`
- Update `Dockerfile` if needed for Z3 build dependencies (cmake, python3)
- Write smoke test: create Z3 context, assert `x > 0 AND x < 0`, check UNSAT
- Verify `make test` passes with Z3 linked

### Phase 2 — Guard Extraction
- Implement `Guard`, `PathCondition` types in `checkers/pathsens.rs`
- Implement `extract_guards(trace, module) → PathCondition`
- Build `ValueId → (FunctionId, BlockId)` lookup from `AirModule`
- Unit tests:
  - Extract guards from module with `CondBr` on trace path
  - No guards when trace stays within one block
  - Skip `MemPhi` nodes gracefully
  - Multiple guards from multi-branch trace

### Phase 3 — Z3 Translation and Feasibility Checking
- Implement `PathFeasibilityChecker` in `checkers/z3solver.rs`
- AIR operand → Z3 AST translation (constants, ICmp variants, unknown → fresh var)
- `check_feasibility(path_condition) → FeasibilityResult`
- Unit tests:
  - Contradictory guards (`x == NULL` AND `x != NULL`) → Infeasible
  - Compatible guards (`x > 0` AND `x < 10`) → Feasible
  - Empty guards → Feasible
  - Timeout → Unknown
  - Multiple guards with mixed constraints

### Phase 4 — Path-Sensitive Runner
- Implement `PathSensitiveConfig`, `PathSensitiveResult`, `PathSensitiveDiagnostics`
- Implement `run_checkers_path_sensitive()` — calls `run_checkers()` then filters via Phase 2+3
- Implement `filter_infeasible()` — standalone post-filter
- Export types and functions from `checkers/mod.rs`
- Unit tests:
  - Full pipeline with hand-crafted AIR module
  - Diagnostics counters correct
  - Config toggle (enabled=false skips Z3)
  - `max_guards_per_trace` limit respected

### Phase 5 — E2E Tests
- 6 source programs compiled to LLVM IR in Docker:
  1. `ps_null_guard.c` — `malloc` + null check before dereference. Path-insensitive reports null-deref; path-sensitive filters it (null path doesn't reach deref).
  2. `ps_correlated_branch.c` — Freed flag prevents UAF. `free(p); freed=1; if (!freed) use(p);`. Path-insensitive reports UAF; path-sensitive filters.
  3. `ps_true_positive.c` — Genuine UAF: `free(p); *p = 1;`. Both modes report it.
  4. `ps_error_path_leak.c` — `fopen` returns NULL on error path, `fclose` on success path. Path-insensitive may report leak; path-sensitive filters the null-return path.
  5. `ps_multi_condition.c` — Multiple branches, bug path IS feasible. Verifies no over-filtering.
  6. `ps_cpp_raii_guard.cpp` — C++ with guarded resource cleanup. Path-sensitive filters FP on guarded path.
- Rust E2E tests asserting:
  - Correct feasible/infeasible/unknown classification for each program
  - True positives preserved across both modes
  - Determinism (repeated runs produce identical results)

### Phase 6 — Python Bindings
- `PyPathSensitiveResult` with `feasible`, `infeasible`, `unknown`, `diagnostics` properties
- `PyPathSensitiveDiagnostics` as dict
- `Project.check_all_path_sensitive(z3_timeout_ms=...)` method
- `Project.check_path_sensitive(checker_name, z3_timeout_ms=...)` method
- `Project.filter_infeasible(findings, z3_timeout_ms=...)` method
- Python E2E tests mirroring Rust E2E coverage

### Phase 7 — Tutorials
- `tutorials/checkers/07-path-sensitive-basics/`
  - `vulnerable.c` — Program with null-check-guarded dereference (FP for path-insensitive) AND a genuine UAF (TP)
  - `detect.py` — Compiles `vulnerable.c` → LLVM IR via `clang-18`, loads via `saf.Project.open()`, runs `check_all()` and `check_all_path_sensitive()`, prints side-by-side comparison showing FP filtered
  - `detect.rs` — Same pipeline via Rust API
  - `README.md` — Explains path sensitivity concept, annotated source walkthrough, shows expected output for both modes, explains guard extraction and Z3 checking
- `tutorials/checkers/08-path-sensitive-comparison/`
  - `vulnerable.c` — Program with 3+ findings: mix of TPs and FPs across different checkers (leak + UAF + null-deref). At least one FP is filtered by path sensitivity.
  - `detect.py` — Compiles source → LLVM IR, runs both modes, prints findings table with feasibility classification and diagnostics summary (guards extracted, Z3 calls, timeouts, FPs filtered)
  - `detect.rs` — Same via Rust API
  - `README.md` — Explains multi-checker comparison, shows the two-stage pipeline, discusses when path sensitivity helps and its limitations
- Both tutorials verified end-to-end in Docker via `make shell` → `python3 detect.py`

### Phase 8 — Documentation Updates
- Update `docs/tool-comparison.md`:
  - Mark "Path-sensitive checker reachability" as implemented in Bug Detection table
  - Update impact text for Section 4 (Bug Detection)
  - Update Summary section
- Update `plans/FUTURE.md`:
  - Move "Path-Sensitive Checker Reachability" entry to implemented status
  - Add future extension points: Switch guards, loop reasoning, interprocedural guards, FCmp support
- Update `plans/PROGRESS.md`:
  - Add E18 to Epics list
  - Add Plan 032 to Plans Index
  - Update Next Steps
  - Append to Session Log

## References

- Shi et al., "Pinpoint: Fast and Precise Sparse Value Flow Analysis for Million Lines of Code," PLDI 2018
- Fan et al., "SMOKE: Scalable Path-Sensitive Memory Leak Detection for Millions of Lines of Code," ICSE 2019
- Sui et al., "SVF: Interprocedural Static Value-Flow Analysis in LLVM," CC 2016
- SVF SABER implementation: `SaberCondAllocator`, `ProgSlice`, `SrcSnkDDA`
- Z3 Rust bindings: https://github.com/prove-rs/z3.rs
- Yao et al., "Falcon: Fused Path-Sensitive Sparse Data Dependence Analysis," PLDI 2024
