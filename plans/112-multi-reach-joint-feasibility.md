# Plan 112: Multi-Reach Joint Path-Feasibility Filtering

## Problem

The `multi_reach` solver (used by the double-free checker, CWE-415) reports a finding when an allocation reaches 2+ distinct deallocation sites via BFS on the SVFG. It does **not** check whether the path conditions to different sinks can hold simultaneously.

**False positive example:**
```c
void f(int cond) {
    int *p = malloc(sizeof(int));
    if (cond)
        free(p);   // sink1: guarded by cond != 0
    else
        free(p);   // sink2: guarded by cond == 0
    // NOT a double-free — mutually exclusive branches
}
```

The BFS finds both `free(p)` calls reachable. It reports double-free. But the branch conditions are mutually exclusive — both frees can never execute in the same run.

The existing E18 path-sensitive filter (`pathsens_runner.rs`) checks individual finding traces for Z3 feasibility, but `multi_reach` produces one finding per source with a combined trace. Even if each sink's individual trace is feasible, the key question is whether **both sinks can be reached in the same execution** — i.e., joint feasibility of the path conditions.

## Approach: Post-Filter with Joint Feasibility Check

1. Modify `multi_reach` to store **per-sink traces** (separate trace to each sink) alongside the combined trace.
2. Add a new Z3-based **joint feasibility** filter: for each pair of sink traces, extract path conditions separately, conjoin them, and check Z3 satisfiability.
3. If all sink-pair conjunctions are UNSAT → the finding is a false positive (mutually exclusive branches).
4. Wire into the path-sensitive pipeline and Python API.

## Key Files

| File | Role |
|------|------|
| `crates/saf-analysis/src/checkers/finding.rs` | `CheckerFinding` struct — add `sink_traces` field |
| `crates/saf-analysis/src/checkers/solver.rs` | `multi_reach()` — store per-sink traces |
| `crates/saf-analysis/src/checkers/pathsens_runner.rs` | Add `filter_multi_reach_infeasible()` joint feasibility filter |
| `crates/saf-analysis/src/z3_utils/solver.rs` | Add `check_joint_feasibility()` method |
| `crates/saf-analysis/src/checkers/mod.rs` | Re-export new public API |
| `crates/saf-python/src/checkers.rs` | Expose via Python bindings |
| `tests/programs/c/double_free_exclusive.c` | Test fixture: mutually exclusive double-free (FP) |
| `tests/programs/c/double_free_real.c` | Test fixture: real double-free (TP) |
| `crates/saf-analysis/tests/joint_feasibility_e2e.rs` | Rust E2E tests |
| `python/tests/test_joint_feasibility.py` | Python E2E tests |

## Agent Team Structure

The work is divided into 4 agents + 1 leader. Each agent's task is self-contained and can be completed without reading the full project context. Agents work sequentially (each phase builds on the previous), but their code changes are isolated to specific files.

---

## Phase A — Agent 1: Extend `CheckerFinding` and `multi_reach` Solver

**Goal:** Make `multi_reach` produce per-sink trace data so the joint feasibility filter can extract individual path conditions.

**Context the agent needs:**
- `crates/saf-analysis/src/checkers/finding.rs` (full file, ~295 lines)
- `crates/saf-analysis/src/checkers/solver.rs` lines 279-371 (`multi_reach` function)
- `crates/saf-analysis/src/svfg/mod.rs` — only `SvfgNodeId` type definition

**Changes:**

### A1. Add `sink_traces` field to `CheckerFinding`

In `finding.rs`, add an optional field to `CheckerFinding`:

```rust
pub struct CheckerFinding {
    // ... existing fields unchanged ...

    /// Per-sink traces for MultiReach findings.
    /// Each entry maps a sink node to the trace from the source to that sink.
    /// Only populated for MultiReach mode (e.g., double-free).
    /// Empty for MayReach/MustNotReach findings.
    pub sink_traces: Vec<(SvfgNodeId, Vec<SvfgNodeId>)>,
}
```

Update all existing `CheckerFinding` construction sites in `solver.rs` (both `may_reach` and `must_not_reach` functions) to include `sink_traces: vec![]`.

Update the `sample_finding()` helper in `finding.rs` tests similarly.

Update `FindingExport` to optionally include sink traces:

```rust
pub struct FindingExport {
    // ... existing fields ...

    /// Per-sink traces (only for MultiReach findings).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sink_traces: Vec<SinkTraceExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkTraceExport {
    pub sink: String,
    pub trace: Vec<String>,
}
```

Update the `From<&CheckerFinding> for FindingExport` impl.

### A2. Modify `multi_reach` to populate `sink_traces`

In `solver.rs`, modify `multi_reach()`:
- After BFS finds 2+ sinks, reconstruct individual traces for **each** sink using the existing `reconstruct_trace()` helper.
- Store them in the finding's `sink_traces` field.
- Keep the existing `trace` field as the combined trace (for backward compatibility).

```rust
// In the 2+ sinks branch:
let mut per_sink: Vec<(SvfgNodeId, Vec<SvfgNodeId>)> = reached_sinks
    .iter()
    .map(|&sink| (sink, reconstruct_trace(&parent, source, sink)))
    .collect();

findings.push(CheckerFinding {
    // ... existing fields ...
    trace: combined_trace,  // unchanged for backward compat
    sink_traces: per_sink,
});
```

### A3. Update unit tests

Update the existing `multi_reach_finds_double_free`, `multi_reach_triple_free` tests in `solver.rs` to assert that `sink_traces` is populated:

```rust
assert_eq!(findings[0].sink_traces.len(), 2); // or 3 for triple
```

Add a new test: `multi_reach_stores_individual_traces` that verifies each `sink_traces` entry has the correct sink node and a valid non-empty trace from source.

**Verification:** `cargo nextest run -p saf-analysis -E 'test(multi_reach)'` passes.

---

## Phase B — Agent 2: Add Joint Feasibility Z3 Check

**Goal:** Add a `check_joint_feasibility()` method to the Z3 solver that checks whether two path conditions can hold simultaneously.

**Context the agent needs:**
- `crates/saf-analysis/src/z3_utils/solver.rs` (full file, ~462 lines)
- `crates/saf-analysis/src/z3_utils/guard.rs` lines 1-50 (`Guard`, `PathCondition`, `ValueLocationIndex` types only)

**Changes:**

### B1. Add `check_joint_feasibility` to `PathFeasibilityChecker`

In `z3_utils/solver.rs`, add:

```rust
/// Check whether two path conditions can hold simultaneously.
///
/// Used for MultiReach filtering: if an allocation reaches two sinks
/// through mutually exclusive branches, the conjunction of their
/// path conditions is UNSAT, proving the finding is a false positive.
///
/// Returns:
/// - `Feasible` if the conjoined guards from BOTH paths are satisfiable
///   (both sinks CAN be reached in the same execution)
/// - `Infeasible` if the conjunction is UNSAT (mutually exclusive paths)
/// - `Unknown` on timeout
pub fn check_joint_feasibility(
    &self,
    pc_a: &PathCondition,
    pc_b: &PathCondition,
    index: &ValueLocationIndex,
) -> FeasibilityResult {
    if pc_a.is_empty() || pc_b.is_empty() {
        // If either path has no guards, we can't prove mutual exclusivity.
        return FeasibilityResult::Feasible;
    }

    let solver = z3::Solver::new();
    let mut params = z3::Params::new();
    #[allow(clippy::cast_possible_truncation)]
    params.set_u32("timeout", self.timeout_ms as u32);
    solver.set_params(&params);

    let mut var_cache: BTreeMap<ValueId, z3::ast::Int> = BTreeMap::new();

    // Assert all guards from path A
    for guard in &pc_a.guards {
        if let Some(expr) = self.translate_guard(guard, index, &mut var_cache) {
            solver.assert(&expr);
        }
    }

    // Assert all guards from path B (sharing the same variable namespace)
    for guard in &pc_b.guards {
        if let Some(expr) = self.translate_guard(guard, index, &mut var_cache) {
            solver.assert(&expr);
        }
    }

    match solver.check() {
        z3::SatResult::Sat => FeasibilityResult::Feasible,
        z3::SatResult::Unsat => FeasibilityResult::Infeasible,
        z3::SatResult::Unknown => FeasibilityResult::Unknown,
    }
}
```

Key design: **shared `var_cache`** — the same `ValueId` (e.g., the branch condition variable `cond`) maps to the same Z3 variable in both paths, so `cond != 0` (path A) AND `cond == 0` (path B) correctly yields UNSAT.

### B2. Add unit tests for joint feasibility

Add tests in the existing `mod tests` in `z3_utils/solver.rs`:

```rust
#[test]
fn joint_feasibility_mutually_exclusive() {
    // Path A: x == 0 (then branch)
    // Path B: x == 0 (else branch) → effectively x != 0
    // Joint: x == 0 AND x != 0 → UNSAT
    let checker = make_checker();
    let x = ValueId::new(1);
    let cond = ValueId::new(100);
    let index = make_index_with_conditions(vec![(
        cond, BinaryOp::ICmpEq,
        OperandInfo::Value(x), OperandInfo::IntConst(0),
    )]);

    let pc_a = PathCondition {
        guards: vec![Guard {
            block: BlockId::new(1), function: FunctionId::new(1),
            condition: cond, branch_taken: true,  // x == 0
        }],
    };
    let pc_b = PathCondition {
        guards: vec![Guard {
            block: BlockId::new(1), function: FunctionId::new(1),
            condition: cond, branch_taken: false,  // x != 0
        }],
    };

    assert_eq!(
        checker.check_joint_feasibility(&pc_a, &pc_b, &index),
        FeasibilityResult::Infeasible
    );
}

#[test]
fn joint_feasibility_compatible_paths() {
    // Path A: x > 5 (then branch)
    // Path B: x > 10 (then branch)
    // Joint: x > 5 AND x > 10 → SAT (e.g., x = 11)
    let checker = make_checker();
    let x = ValueId::new(1);
    let cond1 = ValueId::new(100);
    let cond2 = ValueId::new(101);
    let index = make_index_with_conditions(vec![
        (cond1, BinaryOp::ICmpSgt, OperandInfo::Value(x), OperandInfo::IntConst(5)),
        (cond2, BinaryOp::ICmpSgt, OperandInfo::Value(x), OperandInfo::IntConst(10)),
    ]);

    let pc_a = PathCondition {
        guards: vec![Guard {
            block: BlockId::new(1), function: FunctionId::new(1),
            condition: cond1, branch_taken: true,
        }],
    };
    let pc_b = PathCondition {
        guards: vec![Guard {
            block: BlockId::new(2), function: FunctionId::new(1),
            condition: cond2, branch_taken: true,
        }],
    };

    assert_eq!(
        checker.check_joint_feasibility(&pc_a, &pc_b, &index),
        FeasibilityResult::Feasible
    );
}

#[test]
fn joint_feasibility_empty_path_conservative() {
    let checker = make_checker();
    let index = make_index_with_conditions(vec![]);
    let pc_a = PathCondition::empty();
    let pc_b = PathCondition::empty();

    assert_eq!(
        checker.check_joint_feasibility(&pc_a, &pc_b, &index),
        FeasibilityResult::Feasible
    );
}
```

**Verification:** `cargo nextest run -p saf-analysis -E 'test(joint_feasibility)'` passes.

---

## Phase C — Agent 3: Joint Feasibility Filter in Path-Sensitive Pipeline

**Goal:** Add a `filter_multi_reach_infeasible()` function that uses the Z3 joint feasibility check to filter MultiReach false positives, and wire it into the path-sensitive pipeline.

**Context the agent needs:**
- `crates/saf-analysis/src/checkers/pathsens_runner.rs` (full file, ~708 lines)
- `crates/saf-analysis/src/checkers/finding.rs` lines 14-30 (`CheckerFinding` struct with new `sink_traces` field — after Phase A)
- `crates/saf-analysis/src/z3_utils/solver.rs` lines 75-111 (`check_joint_feasibility` signature — after Phase B)
- `crates/saf-analysis/src/z3_utils/guard.rs` lines 1-50 (types)
- `crates/saf-analysis/src/checkers/mod.rs` (re-exports)

**Changes:**

### C1. Add `filter_multi_reach_infeasible()` in `pathsens_runner.rs`

Add a new public function after the existing `filter_temporal_infeasible()`:

```rust
/// Filter MultiReach (double-free) findings by joint path feasibility.
///
/// For each MultiReach finding with per-sink traces, extracts path conditions
/// for each sink trace, then checks if any pair of sink paths can hold
/// simultaneously. If ALL sink pairs are mutually exclusive (UNSAT), the
/// finding is a false positive.
///
/// This filter addresses the case where an allocation reaches 2+ free calls
/// on mutually exclusive branches (e.g., if/else), which is not a double-free.
pub fn filter_multi_reach_infeasible(
    result: PathSensitiveResult,
    module: &AirModule,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(config.z3_timeout_ms);

    let mut feasible = Vec::new();
    let mut infeasible = result.infeasible;
    let mut unknown = Vec::new();
    let mut joint_filtered = 0usize;

    for finding in result.feasible {
        if finding.sink_traces.len() < 2 {
            // Not a MultiReach finding or only one sink trace — keep as-is
            feasible.push(finding);
            continue;
        }

        match check_sink_pair_feasibility(&finding, &index, &checker, config) {
            FeasibilityResult::Infeasible => {
                // All sink pairs are mutually exclusive — false positive
                joint_filtered += 1;
                infeasible.push(finding);
            }
            FeasibilityResult::Feasible => {
                // At least one pair of sinks can co-execute — real finding
                feasible.push(finding);
            }
            FeasibilityResult::Unknown => {
                // Conservatively keep
                unknown.push(finding);
            }
        }
    }

    // Also check the unknown bucket from earlier stages
    for finding in result.unknown {
        if finding.sink_traces.len() < 2 {
            unknown.push(finding);
            continue;
        }

        match check_sink_pair_feasibility(&finding, &index, &checker, config) {
            FeasibilityResult::Infeasible => {
                joint_filtered += 1;
                infeasible.push(finding);
            }
            _ => {
                unknown.push(finding);
            }
        }
    }

    if joint_filtered > 0 {
        tracing::debug!(
            "Joint feasibility filter: removed {} mutually-exclusive MultiReach findings",
            joint_filtered
        );
    }

    let mut diagnostics = result.diagnostics;
    diagnostics.feasible_count = feasible.len();
    diagnostics.infeasible_count = infeasible.len();
    diagnostics.unknown_count = unknown.len();

    PathSensitiveResult {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}

/// Check if any pair of sink traces in a MultiReach finding can co-execute.
///
/// Returns:
/// - `Infeasible` if ALL pairs are mutually exclusive
/// - `Feasible` if at least one pair can co-execute
/// - `Unknown` if any pair times out and no pair is proven feasible
fn check_sink_pair_feasibility(
    finding: &CheckerFinding,
    index: &ValueLocationIndex,
    checker: &PathFeasibilityChecker,
    config: &PathSensitiveConfig,
) -> FeasibilityResult {
    let traces = &finding.sink_traces;
    let mut any_unknown = false;

    for i in 0..traces.len() {
        for j in (i + 1)..traces.len() {
            let pc_a = extract_guards(&traces[i].1, index);
            let pc_b = extract_guards(&traces[j].1, index);

            if pc_a.guards.len() + pc_b.guards.len() > config.max_guards_per_trace {
                any_unknown = true;
                continue;
            }

            match checker.check_joint_feasibility(&pc_a, &pc_b, index) {
                FeasibilityResult::Feasible => {
                    // This pair can co-execute — finding is real
                    return FeasibilityResult::Feasible;
                }
                FeasibilityResult::Unknown => {
                    any_unknown = true;
                }
                FeasibilityResult::Infeasible => {
                    // This pair is mutually exclusive — continue checking others
                }
            }
        }
    }

    if any_unknown {
        FeasibilityResult::Unknown
    } else {
        // All pairs are mutually exclusive
        FeasibilityResult::Infeasible
    }
}
```

### C2. Wire into `run_checkers_path_sensitive()`

In `pathsens_runner.rs`, modify `run_checkers_path_sensitive()` to apply the joint filter after the existing individual-trace filter:

```rust
pub fn run_checkers_path_sensitive(
    specs: &[CheckerSpec],
    module: &AirModule,
    svfg: &Svfg,
    table: &ResourceTable,
    config: &PathSensitiveConfig,
) -> PathSensitiveResult {
    // Stage 1: path-insensitive
    let stage1 = runner::run_checkers(specs, module, svfg, table, &config.solver_config);

    if !config.enabled {
        // ... existing disabled path (unchanged) ...
    }

    // ... existing caller context build (unchanged) ...

    // Stage 2: individual-trace path-sensitive filtering (existing)
    let result = filter_infeasible_with_context(&stage1.findings, module, config, caller_ctx.as_ref());

    // Stage 3: joint feasibility filtering for MultiReach findings (NEW)
    filter_multi_reach_infeasible(result, module, config)
}
```

### C3. Update re-exports in `mod.rs`

In `checkers/mod.rs`, add `filter_multi_reach_infeasible` to the path-sensitive re-exports:

```rust
#[cfg(feature = "z3-solver")]
pub use pathsens_runner::{
    PathSensitiveConfig, PathSensitiveDiagnostics, PathSensitiveResult,
    filter_infeasible, filter_multi_reach_infeasible,
    filter_temporal_infeasible, run_checkers_path_sensitive,
};
```

### C4. Add unit tests

Add tests in `pathsens_runner.rs::tests`:

- `joint_filter_removes_exclusive_double_free`: Build a module + SVFG where allocation reaches two frees on mutually exclusive branches. Construct a `CheckerFinding` with `sink_traces` populated. Verify `filter_multi_reach_infeasible` moves it to `infeasible`.
- `joint_filter_keeps_real_double_free`: Build a case where both frees are on the same (unconditional) path. Verify the finding stays in `feasible`.
- `joint_filter_skips_non_multireach`: Verify findings with empty `sink_traces` pass through unchanged.

**Verification:** `cargo nextest run -p saf-analysis -E 'test(joint_filter) | test(multi_reach_infeasible)'` passes.

---

## Phase D — Agent 4: E2E Tests and Python Bindings

**Goal:** Create C test fixtures that demonstrate the false positive scenario, add Rust E2E tests using the full pipeline, and ensure the Python API exposes the improved results.

**Context the agent needs:**
- `crates/saf-python/src/checkers.rs` lines 365-411 (path-sensitive Python wrappers)
- `crates/saf-analysis/tests/` — existing E2E test patterns (read one for reference)
- `tests/programs/c/` — existing C test programs (read one for reference)
- CLAUDE.md "Compiling C to LLVM IR" section

**Changes:**

### D1. Create C test fixture: mutually exclusive double-free (false positive)

File: `tests/programs/c/double_free_exclusive.c`
```c
#include <stdlib.h>

// Two frees on mutually exclusive branches — NOT a double-free.
// The checker should NOT report this after joint feasibility filtering.
void exclusive_free(int cond) {
    int *p = (int *)malloc(sizeof(int));
    if (cond)
        free(p);
    else
        free(p);
}

int main() {
    exclusive_free(1);
    return 0;
}
```

### D2. Create C test fixture: real double-free (true positive)

File: `tests/programs/c/double_free_real.c`
```c
#include <stdlib.h>

// Two frees on the same path — a REAL double-free.
// The checker MUST still report this after joint feasibility filtering.
void real_double_free() {
    int *p = (int *)malloc(sizeof(int));
    free(p);
    free(p);  // bug: second free of same allocation
}

int main() {
    real_double_free();
    return 0;
}
```

### D3. Compile fixtures to LLVM IR (inside Docker)

```bash
make shell
clang -S -emit-llvm -g -O0 tests/programs/c/double_free_exclusive.c \
    -o tests/fixtures/llvm/e2e/double_free_exclusive.ll
clang -S -emit-llvm -g -O0 tests/programs/c/double_free_real.c \
    -o tests/fixtures/llvm/e2e/double_free_real.ll
```

### D4. Add Rust E2E test

File: `crates/saf-analysis/tests/joint_feasibility_e2e.rs`

```rust
//! E2E tests for joint path-feasibility filtering of MultiReach findings.

mod common;
use common::load_ll_fixture;

#[test]
fn double_free_exclusive_filtered() {
    // Mutually exclusive frees — joint feasibility should filter this out
    let module = load_ll_fixture("double_free_exclusive.ll");
    // Build full pipeline: PTA → MSSA → SVFG → checkers → path-sensitive
    // Run check_all_path_sensitive (double-free checker included)
    // Assert: no feasible double-free findings
    // Assert: 1 infeasible double-free finding (filtered by joint feasibility)
}

#[test]
fn double_free_real_kept() {
    // Sequential frees — joint feasibility should NOT filter this
    let module = load_ll_fixture("double_free_real.ll");
    // Run check_all_path_sensitive
    // Assert: 1 feasible double-free finding (real bug)
}
```

Note: follow the patterns in existing E2E test files (e.g., `crates/saf-analysis/tests/checker_e2e.rs`) for the pipeline construction boilerplate.

### D5. Add Python E2E test

File: `python/tests/test_joint_feasibility.py`

```python
"""E2E tests for joint path-feasibility filtering of MultiReach findings."""
import saf

def test_exclusive_double_free_filtered(e2e_project):
    """Mutually exclusive frees should be filtered as false positive."""
    proj = saf.Project.open(str(e2e_project / "double_free_exclusive.ll"))
    result = proj.check_all_path_sensitive()
    df_feasible = [f for f in result.feasible if f.checker == "double-free"]
    df_infeasible = [f for f in result.infeasible if f.checker == "double-free"]
    assert len(df_feasible) == 0, "Exclusive frees should be filtered"
    assert len(df_infeasible) >= 1, "Should appear in infeasible bucket"

def test_real_double_free_kept(e2e_project):
    """Sequential frees should remain as feasible finding."""
    proj = saf.Project.open(str(e2e_project / "double_free_real.ll"))
    result = proj.check_all_path_sensitive()
    df_feasible = [f for f in result.feasible if f.checker == "double-free"]
    assert len(df_feasible) >= 1, "Real double-free must be reported"
```

### D6. Verify Python `CheckerFinding` exposes sink traces (optional enhancement)

In `crates/saf-python/src/checkers.rs`, add an optional `sink_traces` getter to `PyCheckerFinding`:

```rust
/// Per-sink traces for MultiReach findings (list of dicts with 'sink' and 'trace').
/// Empty for non-MultiReach findings.
#[getter]
fn sink_traces(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
    let list = PyList::empty(py);
    for (sink, trace) in &self.inner.sink_traces {
        let d = PyDict::new(py);
        d.set_item("sink", sink.to_hex())?;
        let trace_ids: Vec<String> = trace.iter().map(SvfgNodeId::to_hex).collect();
        d.set_item("trace", trace_ids)?;
        list.append(d)?;
    }
    Ok(list.into())
}
```

Also add `sink_traces` to the `to_dict()` method.

**Verification:** `make test` passes (both Rust and Python).

---

## Phase E — Leader: Integration Review and Diagnostics

**Goal:** Review all phases, run full test suite, update diagnostics, and update PROGRESS.md.

**Leader tasks (lightweight):**

1. **Review Phase A-D code** for correctness and consistency.
2. **Run `make fmt && make lint`** — fix any clippy warnings.
3. **Run `make test`** — verify all existing + new tests pass.
4. **Update `PathSensitiveDiagnostics`** in `pathsens_runner.rs`:
   - Add `joint_feasibility_filtered: usize` field to track how many MultiReach findings were removed by the joint filter.
   - Update `filter_multi_reach_infeasible` to populate this counter.
   - Expose in Python via `diagnostics` getter.
5. **Update `plans/PROGRESS.md`**:
   - Add Plan 112 to Plans Index with status "done".
   - Append session log entry.
6. **Update `plans/FUTURE.md`**: Remove or mark the relevant entry about multi-reach false positives if one exists.
7. **Commit** all changes.

---

## Execution Order

```
Phase A (Agent 1) → Phase B (Agent 2) → Phase C (Agent 3) → Phase D (Agent 4) → Phase E (Leader)
```

Phases A and B are independent in code (different files) and could run in parallel, but B's tests need A's `sink_traces` type to exist. To keep things simple and avoid merge conflicts, run sequentially.

## Estimated Scope

| Phase | Agent | Files Changed | ~LOC Added | Context Needed |
|-------|-------|---------------|------------|----------------|
| A | Agent 1 | finding.rs, solver.rs | ~60 | ~400 lines |
| B | Agent 2 | z3_utils/solver.rs | ~80 | ~500 lines |
| C | Agent 3 | pathsens_runner.rs, mod.rs | ~120 | ~800 lines |
| D | Agent 4 | 2 C files, 2 test files, checkers.rs | ~120 | ~600 lines |
| E | Leader | PROGRESS.md, pathsens_runner.rs | ~20 | Review only |

Total: ~400 LOC across 9 files.
