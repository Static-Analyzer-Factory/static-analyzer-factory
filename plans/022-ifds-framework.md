# Plan 022: IFDS Framework

**Epic**: E9 — IFDS/IDE Data-Flow Framework
**Status**: approved
**Scope**: Generic IFDS solver + taint-as-IFDS client + Python bindings + E2E tests + 1 tutorial

---

## Overview

Add a generic IFDS (Interprocedural Finite Distributive Subset) solver to SAF. This is the single highest-impact addition identified in the tool comparison — it transforms SAF from a single-purpose taint tool into a general-purpose data-flow analysis framework.

IFDS encodes interprocedural data-flow problems as graph reachability on an "exploded supergraph" where each node is `(program_point, data_flow_fact)`. The tabulation algorithm by Reps, Horwitz, and Sagiv (POPL'95) solves these problems precisely in polynomial time.

### Prerequisites

SAF already has everything IFDS needs:
- ICFG (block-level with call/return edges)
- CallGraph (direct calls, external declarations, indirect placeholders)
- AIR instruction model (typed operations, ValueId-based SSA)
- Graph algorithms (DFS, BFS, SCC, toposort, reverse post-order)

### Design Decisions

1. **Instruction-level within blocks**: IFDS iterates instructions sequentially within each block. The block-level ICFG provides inter-block edges. No instruction-level CFG needed.
2. **Zero fact (Λ) built into framework**: The tautology fact is fundamental to the tabulation algorithm. The problem trait includes `zero_value()`.
3. **Module reference in trait**: Flow functions need to inspect instruction operands, function names, etc. The problem trait includes `module()`.
4. **Forward-only for this plan**: Backward IFDS deferred to future work.
5. **Keep existing BFS taint**: IFDS taint is a parallel implementation, not a replacement. Both coexist for comparison.

---

## File Layout

```
crates/saf-analysis/src/ifds/
  mod.rs              — Public re-exports
  problem.rs          — IfdsProblem trait
  solver.rs           — Tabulation algorithm (solve_ifds)
  config.rs           — IfdsConfig, limits
  result.rs           — IfdsResult, IfdsDiagnostics
  export.rs           — JSON export for IFDS results
  taint.rs            — Taint-as-IFDS client (TaintIfdsProblem)

crates/saf-python/src/ifds.rs  — Python bindings for IFDS

tutorials/taint/07-ifds-taint/
  detect.py           — Python tutorial script
  detect.rs           — Rust tutorial script (optional)
  vulnerable.c        — C source with inter-procedural taint
  README.md           — Tutorial guide
```

---

## Phase 1: Core IFDS Framework (TDD)

### Task 1.1: IfdsProblem trait + types

Create `crates/saf-analysis/src/ifds/problem.rs`:

```rust
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use saf_core::air::{AirModule, AirFunction, Instruction};
use saf_core::ids::FunctionId;

/// A data-flow problem expressible in the IFDS framework.
///
/// IFDS (Interprocedural Finite Distributive Subset) problems operate on
/// a finite domain D with powerset lattice and distributive flow functions.
/// The solver reduces the problem to graph reachability on an exploded supergraph.
///
/// # Type Parameters
///
/// The `Fact` associated type is the domain D. It must be:
/// - `Ord` — for deterministic BTreeSet storage
/// - `Clone` — facts are copied during propagation
/// - `Debug` — for diagnostics and export
///
/// # Flow Function Categories
///
/// - `normal_flow` — non-call intra-procedural statements
/// - `call_flow` — caller → callee entry (argument passing)
/// - `return_flow` — callee exit → caller return point
/// - `call_to_return_flow` — facts that bypass the callee (locals not passed as args)
pub trait IfdsProblem {
    /// The data-flow fact type (domain D).
    type Fact: Ord + Clone + std::fmt::Debug;

    /// The zero (tautology) fact Λ.
    ///
    /// This special fact is always propagated and represents "this program point
    /// is reachable." Edges from Λ generate new facts; edges to Λ kill facts.
    fn zero_value(&self) -> Self::Fact;

    /// Reference to the AIR module being analyzed.
    fn module(&self) -> &AirModule;

    /// Initial seeds: functions and facts alive at their entry points.
    ///
    /// The zero fact is automatically seeded at all reachable function entries;
    /// this method should return additional problem-specific initial facts.
    fn initial_seeds(&self) -> BTreeMap<FunctionId, BTreeSet<Self::Fact>>;

    /// Normal (non-call) intra-procedural flow function.
    ///
    /// Given an instruction and an incoming fact, returns the set of facts
    /// that hold after the instruction executes.
    ///
    /// To propagate the fact unchanged, include it in the returned set.
    /// To kill the fact, return an empty set.
    /// To generate new facts, include them in the returned set.
    fn normal_flow(
        &self,
        inst: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;

    /// Call flow function: facts propagated from call site to callee entry.
    ///
    /// Typically maps tainted arguments to callee parameters.
    fn call_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;

    /// Return flow function: facts propagated from callee exit to caller return site.
    ///
    /// Typically maps tainted return values back to the call result.
    fn return_flow(
        &self,
        call_site: &Instruction,
        callee: &AirFunction,
        exit_inst: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;

    /// Call-to-return flow function: facts that skip the callee.
    ///
    /// Handles facts about caller-local state that isn't affected by the call
    /// (e.g., local variables not passed as arguments).
    fn call_to_return_flow(
        &self,
        call_site: &Instruction,
        fact: &Self::Fact,
    ) -> BTreeSet<Self::Fact>;
}
```

**Tests** (write first):
- Trait is object-safe enough for our use (we use it via generics, not dyn)
- A minimal "identity" problem that propagates all facts unchanged compiles and type-checks

### Task 1.2: IfdsConfig + IfdsDiagnostics

Create `crates/saf-analysis/src/ifds/config.rs`:

```rust
/// Configuration for the IFDS solver.
#[derive(Debug, Clone)]
pub struct IfdsConfig {
    /// Maximum worklist iterations before aborting (default: 1_000_000).
    pub max_iterations: usize,
    /// Maximum facts per program point (default: 10_000).
    pub max_facts_per_point: usize,
}

impl Default for IfdsConfig {
    fn default() -> Self {
        Self {
            max_iterations: 1_000_000,
            max_facts_per_point: 10_000,
        }
    }
}
```

Create `crates/saf-analysis/src/ifds/result.rs`:

```rust
use std::collections::{BTreeMap, BTreeSet};
use saf_core::ids::{FunctionId, InstId};

/// Result of an IFDS analysis.
#[derive(Debug, Clone)]
pub struct IfdsResult<F: Ord + Clone> {
    /// Facts holding at each instruction (after the instruction executes).
    pub facts: BTreeMap<InstId, BTreeSet<F>>,
    /// Summary edges per function: (entry_fact, exit_fact) pairs.
    pub summaries: BTreeMap<FunctionId, BTreeSet<(F, F)>>,
    /// Solver diagnostics.
    pub diagnostics: IfdsDiagnostics,
}

/// Solver diagnostics for performance monitoring.
#[derive(Debug, Clone, Default)]
pub struct IfdsDiagnostics {
    /// Total worklist iterations performed.
    pub iterations: usize,
    /// Total path edges explored.
    pub path_edges_explored: usize,
    /// Total summary edges created.
    pub summary_edges_created: usize,
    /// Peak number of facts at any single program point.
    pub facts_at_peak: usize,
    /// Whether the solver hit a configured limit.
    pub reached_limit: bool,
}

impl<F: Ord + Clone> IfdsResult<F> {
    /// Get facts at a specific instruction.
    pub fn facts_at(&self, inst: InstId) -> Option<&BTreeSet<F>> {
        self.facts.get(&inst)
    }

    /// Check if a specific fact holds at a specific instruction.
    pub fn holds_at(&self, inst: InstId, fact: &F) -> bool {
        self.facts.get(&inst).map_or(false, |fs| fs.contains(fact))
    }
}
```

**Tests**:
- `IfdsConfig::default()` produces sane values
- `IfdsResult::holds_at` returns correct boolean
- `IfdsResult::facts_at` returns None for unknown instructions

### Task 1.3: Tabulation Solver

Create `crates/saf-analysis/src/ifds/solver.rs`:

The core algorithm (simplified pseudocode):

```
WorkList = {initial path edges from seeds}
PathEdges = {}
SummaryEdges = {}

while WorkList is not empty:
    take (s_p, d1, n, d2) from WorkList   // path edge: d1@entry → d2@n

    if n is a call site:
        // 1. Call-to-return: bypass callee
        for d3 in call_to_return_flow(n, d2):
            propagate path edge (s_p, d1, return_site(n), d3)

        // 2. Call flow: enter callee
        for each callee:
            for d3 in call_flow(n, callee, d2):
                propagate path edge (s_callee, d3, s_callee, d3)

        // 3. Apply existing summaries
        for (d3, d4) in SummaryEdges[callee]:
            if d3 reachable from d2:
                propagate path edge (s_p, d1, return_site(n), d4)

    elif n is an exit node:
        // Create summary edge
        for each caller of this function:
            for d3 in return_flow(caller_call_site, callee, n, d2):
                add (d1_at_entry, d3) to SummaryEdges[callee]
                // Re-propagate to all callers with matching entry facts
                for (s_c, d4, call_site, d5) in PathEdges where d5 maps to d1:
                    propagate path edge (s_c, d4, return_site, d3)

    else:  // normal statement
        for d3 in normal_flow(n, d2):
            propagate path edge (s_p, d1, successor(n), d3)
```

The actual implementation handles:
- Instruction-level iteration within blocks
- Identifying call sites (CallDirect/CallIndirect operations)
- Identifying exit nodes (Ret/Unreachable terminators)
- Looking up callees via CallGraph
- Looking up callee functions via AirModule
- Deterministic iteration via BTreeSet worklist

**Tests** (write first, run against the solver):

1. **Linear flow**: Single function, 3 instructions, fact propagates through
2. **Kill flow**: Fact killed at second instruction, doesn't reach third
3. **Generate flow**: Zero fact generates new fact at call to source function
4. **Diamond CFG**: Fact propagates through both branches, merges at join
5. **Loop**: Fact propagates through loop body, reaches exit
6. **Inter-procedural**: Caller passes fact through call, callee propagates, returns to caller
7. **Summary reuse**: Same callee called twice from different sites, summary computed once
8. **Zero fact propagation**: Zero fact reaches all reachable program points
9. **Max iterations limit**: Solver stops when hitting limit, diagnostics show `reached_limit`
10. **Max facts limit**: Solver caps facts per point

### Task 1.4: Export

Create `crates/saf-analysis/src/ifds/export.rs`:

```rust
use serde::{Serialize, Deserialize};

/// Exportable IFDS result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfdsExport {
    /// Facts per instruction: inst_hex → [fact_strings].
    pub facts: BTreeMap<String, Vec<String>>,
    /// Summary edges per function.
    pub summaries: Vec<IfdsSummaryExport>,
    /// Solver diagnostics.
    pub diagnostics: IfdsDiagnosticsExport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfdsSummaryExport {
    pub function: String,  // hex
    pub edges: Vec<(String, String)>,  // (entry_fact, exit_fact) as debug strings
}
```

The `IfdsResult` gets an `export()` method that takes a fact formatter:

```rust
impl<F: Ord + Clone + std::fmt::Debug> IfdsResult<F> {
    pub fn export(&self) -> IfdsExport { ... }
}
```

**Tests**:
- Export produces valid JSON
- Export is deterministic (same result on repeated calls)

### Task 1.5: Module wiring

Update `crates/saf-analysis/src/ifds/mod.rs` and `crates/saf-analysis/src/lib.rs` to export:
- `IfdsProblem` trait
- `IfdsConfig`, `IfdsDiagnostics`
- `IfdsResult`
- `solve_ifds()` function
- `IfdsExport`

---

## Phase 2: Taint-as-IFDS Client

### Task 2.1: TaintFact type + TaintIfdsProblem

Create `crates/saf-analysis/src/ifds/taint.rs`:

```rust
/// A taint data-flow fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaintFact {
    /// The zero (tautology) fact.
    Zero,
    /// A tainted value.
    Tainted(ValueId),
}

/// Taint analysis as an IFDS problem.
///
/// Sources: function return values matching source patterns.
/// Sinks: arguments to functions matching sink patterns.
/// Propagation: through copies, casts, binary ops, phi, GEP, call args, returns.
pub struct TaintIfdsProblem<'a> {
    module: &'a AirModule,
    sources: Vec<Selector>,
    sinks: Vec<Selector>,
    sanitizers: Vec<Selector>,
}
```

Flow function implementations:
- **normal_flow**: Propagate `Tainted(v)` through Copy, Cast, BinaryOp, Select, Phi, GEP. Kill at sanitizer calls. Zero fact generates `Tainted(return_val)` at source calls.
- **call_flow**: Map `Tainted(arg_i)` to callee's param_i.
- **return_flow**: Map `Tainted(return_val)` back to caller's call result.
- **call_to_return_flow**: Pass through facts not related to call arguments.

### Task 2.2: Taint-as-IFDS unit tests

Test programs (AIR fixtures built from C source):

1. **Simple source→sink**: `getenv("PATH")` assigned to `x`, `system(x)` — verify `Tainted(x)` reaches system's argument
2. **Inter-procedural**: `get_input()` → `helper(x)` → `system(helper_result)` — verify taint propagates through call chain
3. **Sanitizer kill**: `getenv()` → `sanitize()` → `system()` — verify taint killed at sanitizer
4. **Multiple sources**: Two independent taint flows in same program
5. **No false flow**: Clean data path that doesn't touch any source — verify no spurious taint

---

## Phase 3: End-to-End Tests

### Task 3.1: C source programs for IFDS taint E2E

Create test source programs:

**`tests/fixtures/sources/ifds_simple_taint.c`**:
```c
#include <stdlib.h>
// getenv → system (direct, single function)
int main() {
    char *path = getenv("PATH");
    system(path);
    return 0;
}
```

**`tests/fixtures/sources/ifds_interprocedural_taint.c`**:
```c
#include <stdlib.h>
// getenv → helper → system (across function boundary)
char *process(char *input) {
    return input;  // pass-through
}
int main() {
    char *data = getenv("USER");
    char *result = process(data);
    system(result);
    return 0;
}
```

**`tests/fixtures/sources/ifds_sanitized_taint.c`**:
```c
#include <stdlib.h>
#include <string.h>
// getenv → sanitize (overwrite) → system (safe)
int main() {
    char *input = getenv("PATH");
    char safe[256];
    strcpy(safe, "/usr/bin/ls");  // overwrite with safe value
    system(safe);
    return 0;
}
```

### Task 3.2: Rust E2E integration tests

Create `crates/saf-analysis/tests/ifds_e2e.rs`:
- Compile C sources → .ll via clang-18
- Load via LlvmFrontend
- Build ICFG + CallGraph
- Run taint-as-IFDS
- Assert expected taint facts at sink instructions

### Task 3.3: Python E2E integration tests

Create `python/tests/test_ifds.py`:
- Load .ll files via `saf.Project.open()`
- Run IFDS taint analysis via Python bindings
- Assert expected results

---

## Phase 4: Python SDK Bindings

### Task 4.1: IFDS Python bindings

Create `crates/saf-python/src/ifds.rs`:
- `PyIfdsResult` — wraps IfdsResult with Python-accessible methods
- `PyIfdsDiagnostics` — diagnostics as Python dict
- Expose `ifds_taint()` method on `Project`:

```python
# Usage:
project = saf.Project.open("program.ll")
sources = saf.function_return("getenv")
sinks = saf.arg_to("system", 0)
result = project.ifds_taint(sources, sinks)
# result.facts_at(inst_id) → set of tainted values
# result.has_taint_at_sink() → bool
# result.export() → dict
```

### Task 4.2: Python tests

Test the Python bindings end-to-end using the same C source programs from Phase 3.

---

## Phase 5: Tutorial

### Task 5.1: IFDS taint tutorial

Create `tutorials/taint/07-ifds-taint/`:

- `vulnerable.c` — Inter-procedural taint flow (getenv → helper → system)
- `detect.py` — Python script using `project.ifds_taint()`
- `README.md` — Explains IFDS concept, compares to BFS taint, shows how inter-procedural precision improves
- `detect.rs` (optional) — Rust version

The tutorial should demonstrate:
1. How IFDS tracks taint through function calls precisely
2. How summary edges avoid re-analyzing callees
3. Comparing IFDS results with existing BFS taint results

---

## Phase 6: Documentation & Tracking Updates

### Task 6.1: Update tool-comparison.md

Mark IFDS as implemented (non-gapped):
- Section 2 table: IFDS solver → **Yes** for SAF
- Update impact statement

### Task 6.2: Update FUTURE.md

Add extension points:
- IDE solver (environment transformers, value domains)
- Backward IFDS
- Typestate, uninit vars, constant propagation as future IFDS clients

### Task 6.3: Update PROGRESS.md

- Add E9 epic
- Add Plan 022 to index
- Update Next Steps

---

## Implementation Order

```
Phase 1: Core IFDS Framework (TDD)
  1.1 IfdsProblem trait + types
  1.2 IfdsConfig + IfdsDiagnostics + IfdsResult
  1.3 Tabulation solver
  1.4 Export
  1.5 Module wiring

Phase 2: Taint-as-IFDS Client
  2.1 TaintFact + TaintIfdsProblem
  2.2 Taint-as-IFDS unit tests

Phase 3: End-to-End Tests
  3.1 C source programs
  3.2 Rust E2E integration tests
  3.3 Python E2E integration tests

Phase 4: Python SDK Bindings
  4.1 IFDS Python bindings
  4.2 Python tests

Phase 5: Tutorial
  5.1 IFDS taint tutorial

Phase 6: Documentation & Tracking
  6.1 Update tool-comparison.md
  6.2 Update FUTURE.md
  6.3 Update PROGRESS.md
```

Each phase depends on the previous one. Within phases, tasks are sequential (TDD: test → implement → verify).

---

## References

- Reps, Horwitz, Sagiv: "Precise Interprocedural Dataflow Analysis via Graph Reachability" (POPL'95)
- Sagiv, Reps, Horwitz: "Precise Interprocedural Dataflow Analysis with Applications to Constant Propagation" (TCS'96) — IDE paper
- Bodden: "Inter-procedural Data-flow Analysis with IFDS/IDE and Soot" (SOAP'12) — practical implementation guide
- PhASAR IFDS implementation: https://github.com/secure-software-engineering/phasar
