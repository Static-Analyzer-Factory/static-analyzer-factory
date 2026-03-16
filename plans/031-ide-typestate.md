# Plan 031: IDE Solver + Typestate Analysis

**Epic:** E17 — IDE Solver + Typestate Analysis
**Status:** approved
**Depends on:** E9 (IFDS Framework)

---

## Overview

Extend SAF's IFDS framework (E9) with the IDE (Interprocedural Distributive Environment) algorithm from Sagiv, Reps & Horwitz (TCS'96). IDE generalizes IFDS by associating a *value* from a lattice `L` with each data-flow fact, enabling analyses that track not just "which facts hold" but "what value does each fact have."

The primary client is **typestate analysis** — tracking per-resource state machines (e.g., file: `Uninit → Opened → Closed`). This closes the gap with PhASAR's `IDETypeStateAnalysis` and enables AI agents to author custom typestate checkers declaratively.

## Background & Research

### IDE Algorithm (TCS'96)

IDE extends IFDS with **edge functions** — functions `L → L` attached to every edge in the exploded supergraph. The algorithm has two phases:

1. **Phase 1 (Jump Functions):** Run the IFDS tabulation algorithm, additionally computing a *jump function* `JumpFn(d₁, n, d₂)` = the composed edge function from procedure entry fact `d₁` to fact `d₂` at program point `n`. When propagating, compose edge functions and re-propagate when the join of the new function improves over the existing one.

2. **Phase 2 (Value Propagation):** Propagate actual lattice values top-down through computed jump functions. Seed entry points with `⊤`, then for each reachable `(d₁, n, d₂)` with jump function `f`: `values[(n, d₂)] = join(values[(n, d₂)], f(values[(entry, d₁)]))`.

### PhASAR's IDE Implementation

PhASAR's `IDETabulationProblem` extends `IFDSTabulationProblem` with:
- Four edge function factories: `getNormalEdgeFunction`, `getCallEdgeFunction`, `getReturnEdgeFunction`, `getCallToRetEdgeFunction`
- `EdgeFunction<l_t>` abstraction with `computeTarget()`, `compose()`, `join()`
- Built-in edge functions: `EdgeIdentity`, `AllTop`, `AllBottom`, `ConstantEdgeFunction`
- `IDETypeStateAnalysis` client using `TypeStateDescription` with `getNextState()` delta function

### Heros IDE Implementation (Java, canonical)

Heros' `IDETabulationProblem<N,D,M,V,I>` adds:
- `edgeFunctions()` factory returning `EdgeFunctions<N,D,M,V>`
- `createJoinLattice()` for value merging
- `createAllTopFunction()` for initial edge function
- `IDESolver` with two-phase algorithm

### Typestate via IDE

- **Fact** = resource identity (ValueId of the file pointer / lock / etc.)
- **Value** = typestate (which state the resource is in)
- **Edge functions** = state transition functions from a `TypestateSpec`
- **Findings** = resources in error states or not in accepting states at program exit

## Design

### 1. Lattice Trait

```rust
/// A finite lattice for IDE value domains.
pub trait Lattice: Clone + Ord + Debug {
    fn top() -> Self;
    fn bottom() -> Self;
    fn join(&self, other: &Self) -> Self;
    fn meet(&self, other: &Self) -> Self;
    fn leq(&self, other: &Self) -> bool;
}
```

Simpler than E15's `AbstractDomain` — no widening/narrowing since IDE operates on finite lattices.

### 2. Edge Functions (enum-based, not trait objects)

```rust
/// Built-in edge functions for any Lattice type.
pub enum BuiltinEdgeFn<V: Lattice> {
    /// f(x) = x. Neutral element for composition.
    Identity,
    /// f(x) = ⊤ for all x. Neutral element for join.
    AllTop,
    /// f(x) = ⊥ for all x. Absorbing element.
    AllBottom,
    /// f(x) = c. Ignores input, always returns c.
    Constant(V),
    /// f(x) = g(h(x)). Lazy composition.
    Composed(Box<BuiltinEdgeFn<V>>, Box<BuiltinEdgeFn<V>>),
}
```

Methods: `compute_target(V) -> V`, `compose_with(&Self) -> Self`, `join_with(&Self) -> Self`.

**Why enum over trait objects:** Identity/AllTop are stateless — no heap allocation. Composition is lazy but deterministic. `Ord` derivable for BTreeMap keys. Matches SAF's preference for concrete types over dynamic dispatch.

### 3. IdeProblem Trait

```rust
/// IDE problem = IFDS problem + edge functions + value lattice.
pub trait IdeProblem: IfdsProblem {
    type Value: Lattice;

    fn normal_edge_fn(&self, inst: &Instruction, src_fact: &Self::Fact,
                       succ_fact: &Self::Fact) -> BuiltinEdgeFn<Self::Value>;
    fn call_edge_fn(&self, call_site: &Instruction, callee: &AirFunction,
                     src_fact: &Self::Fact, dest_fact: &Self::Fact) -> BuiltinEdgeFn<Self::Value>;
    fn return_edge_fn(&self, call_site: &Instruction, callee: &AirFunction,
                       exit_inst: &Instruction, exit_fact: &Self::Fact,
                       ret_fact: &Self::Fact) -> BuiltinEdgeFn<Self::Value>;
    fn call_to_return_edge_fn(&self, call_site: &Instruction,
                               src_fact: &Self::Fact, ret_fact: &Self::Fact) -> BuiltinEdgeFn<Self::Value>;

    fn top_value(&self) -> Self::Value;
    fn bottom_value(&self) -> Self::Value;
}
```

Extends `IfdsProblem` — backward compatible. Existing IFDS clients unchanged.

### 4. IDE Solver

New `solve_ide()` function (does NOT modify existing `solve_ifds()`):

**Phase 1:** Extends IFDS tabulation with jump function tracking:
```rust
jump_fn: BTreeMap<(F, InstId, F), BuiltinEdgeFn<V>>
```

During propagation, compose edge functions along path edges. Re-propagate when `join_with` produces a different (improved) function.

**Phase 2:** Top-down value computation:
```rust
values_at: BTreeMap<(InstId, F), V>
```

Seed entry points, apply jump functions, propagate through call graph.

**Result type:**
```rust
pub struct IdeResult<F, V> {
    pub values: BTreeMap<InstId, BTreeMap<F, V>>,
    pub ifds_result: IfdsResult<F>,
    pub diagnostics: IdeDiagnostics,
}
```

### 5. TypestateSpec (Declarative)

```rust
pub struct TypestateSpec {
    pub name: String,
    pub states: Vec<String>,
    pub initial_state: String,
    pub error_states: Vec<String>,
    pub accepting_states: Vec<String>,
    pub transitions: Vec<TypestateTransition>,
    pub constructors: Vec<String>,
}

pub struct TypestateTransition {
    pub from: String,
    pub call: String,     // function name (glob pattern)
    pub to: String,
}
```

AI agents author typestate checkers as data, not code — consistent with E14's declarative checker approach.

### 6. TypestateLattice

Finite lattice from the state enum. Ordering: `Top > state₁ | state₂ | ... | stateₙ > Bottom`. Join of different concrete states = Top (over-approximation). Error states are concrete lattice elements.

### 7. TypestateIdeProblem

Implements `IdeProblem`:
- **Fact** = `TypestateFact::Zero | TypestateFact::Tracked(ValueId)`
- **Value** = `TypestateValue` (the state lattice)
- **normal_edge_fn**: At constructor calls (e.g., fopen), return `Constant(initial_state)`. At transition calls (e.g., fclose), return a transition edge function. Otherwise `Identity`.
- **call/return edge functions**: Standard interprocedural propagation with `Identity`.
- **Finding generation**: At function exits, check tracked resources — error state or non-accepting state → finding.

### 8. Built-in Typestate Specs

Three built-in specs shipped with SAF:

**file_io** (matches PhASAR's `CSTDFILEIOTypeStateDescription`):
```
States: uninit, opened, closed, error
Constructors: fopen, fdopen
Transitions:
  uninit  + fopen/fdopen → opened
  opened  + fread/fwrite/fgetc/fgets/fputc/fputs/fprintf/fscanf/fflush/fseek/ftell/rewind/feof/ferror → opened (stay)
  opened  + fclose → closed
  closed  + fclose → error   (double-close)
  closed  + fread/fwrite/... → error   (use-after-close)
  uninit  + fclose → error   (close-before-open)
  uninit  + fread/fwrite/... → error   (use-before-open)
Accepting: uninit, closed
```

**mutex_lock:**
```
States: uninit, locked, unlocked, error
Transitions: pthread_mutex_lock→locked, pthread_mutex_unlock→unlocked,
             (locked)pthread_mutex_lock→error, (unlocked)pthread_mutex_unlock→error
Constructors: pthread_mutex_init
Accepting: uninit, unlocked
```

**memory_alloc:**
```
States: unallocated, allocated, freed, error
Transitions: malloc/calloc/realloc→allocated, free→freed,
             (freed)free→error, (freed)use→error
Constructors: malloc, calloc, realloc
Accepting: unallocated, freed
```

## E2E Test Programs

| # | File | Language | Scenario |
|---|------|----------|----------|
| 1 | `typestate_file_leak.c` | C | fopen without fclose on one path → leak |
| 2 | `typestate_double_close.c` | C | fclose called twice → error state |
| 3 | `typestate_use_after_close.c` | C | fread after fclose → error state |
| 4 | `typestate_correct.c` | C | All paths close file → zero findings |
| 5 | `typestate_lock.cpp` | C++ | pthread mutex lock/unlock mismatch |
| 6 | `typestate_rust_file.rs` | Rust | unsafe FFI file handle misuse |

## Tutorials

### Tutorial 1: `tutorials/checkers/05-typestate-file-io/`

Config parser in C that opens files in multiple paths. Bugs: double-close on "invalid" path, missing fclose on success path. Uses `Project.typestate("file_io")`.

### Tutorial 2: `tutorials/checkers/06-typestate-lock/`

Multi-threaded C program with pthread mutex misuse — lock acquired but not released on error path. Uses `Project.typestate("mutex_lock")`.

## Python API

```python
# Built-in typestate spec
findings = project.typestate("file_io")

# Custom typestate spec
findings = project.typestate_custom({
    "name": "socket_io",
    "states": ["uninit", "connected", "closed", "error"],
    "initial_state": "uninit",
    "error_states": ["error"],
    "accepting_states": ["closed", "uninit"],
    "transitions": [
        {"from": "uninit", "call": "connect", "to": "connected"},
        {"from": "connected", "call": "send", "to": "connected"},
        {"from": "connected", "call": "close", "to": "closed"},
    ],
    "constructors": ["socket"],
})

# Low-level IDE access (for advanced users)
result = project.ide_solve(problem_config)
value = result.value_at(inst_id, fact)
```

## Implementation Phases

### Phase 1: Lattice trait + built-in edge functions
- `ifds/lattice.rs`: `Lattice` trait definition
- `ifds/edge_fn.rs`: `BuiltinEdgeFn<V>` enum with Identity/AllTop/AllBottom/Constant/Composed
- Methods: `compute_target()`, `compose_with()`, `join_with()`, `is_identity()`, `is_top()`
- Derive `Ord` for deterministic BTreeMap usage
- ~15 unit tests: compose laws, join lattice laws, constant propagation, determinism

### Phase 2: IdeProblem trait
- `ifds/ide_problem.rs`: `IdeProblem` trait extending `IfdsProblem`
- Four edge function factory methods
- `top_value()`, `bottom_value()` lattice accessors
- ~5 unit tests: trait implementability with a trivial test problem

### Phase 3: IDE solver — Phase 1 (jump functions)
- `ifds/ide_solver.rs`: `solve_ide()` function
- Reuses IFDS pre-computation infrastructure (helper maps, ICFG)
- Adds `jump_fn: BTreeMap<(F, InstId, F), BuiltinEdgeFn<V>>`
- Compose edge functions during propagation
- Re-propagate when join produces improved function
- ~8 unit tests: linear composition, diamond merge, interprocedural, loop convergence

### Phase 4: IDE solver — Phase 2 (value propagation)
- `ifds/ide_solver.rs` (continued): Phase 2 top-down BFS
- `ifds/ide_result.rs`: `IdeResult<F, V>`, `IdeDiagnostics`, `IdeExport`
- `values_at()` and `value_at()` query methods
- JSON export
- ~6 unit tests: single-function values, interprocedural, merge-point join

### Phase 5: TypestateSpec + TypestateLattice
- `ifds/typestate.rs`: `TypestateSpec`, `TypestateTransition`, `TypestateLattice`
- Spec validation (no duplicate states, valid transitions, constructors exist)
- Lattice from state enum: Top/Bottom/State(idx)
- Built-in specs: `file_io`, `mutex_lock`, `memory_alloc`
- ~10 unit tests: spec validation, lattice ordering, delta function

### Phase 6: TypestateIdeProblem
- `ifds/typestate.rs` (continued): `TypestateFact`, `TypestateIdeProblem`
- Implements both `IfdsProblem` (fact flow) and `IdeProblem` (edge functions)
- Constructor detection, transition matching, error state detection
- Finding generation at function exits
- ~8 unit tests: constructor flow, transitions, interprocedural resource tracking

### Phase 7: E2E tests
- 6 source programs (4 C, 1 C++, 1 Rust) compiled to LLVM IR
- `tests/typestate_e2e.rs`: ~12 Rust E2E tests
- Tests: findings count, error state identification, zero-findings on correct programs, determinism, export format
- TDD: write test expectations first, then verify

### Phase 8: Python bindings
- `saf-python/src/ide.rs`: `PyIdeResult`, `PyTypestateFinding`, `PyTypestateSpec`
- `project.rs`: `Project.typestate()`, `Project.typestate_custom()`, `Project.ide_solve()`
- `python/tests/test_ide.py`: ~15 Python E2E tests
- Tests: API surface, built-in specs, custom specs, diagnostics, export

### Phase 9: Tutorials + documentation
- `tutorials/checkers/05-typestate-file-io/`: vulnerable.c + detect.py + detect.rs + README.md
- `tutorials/checkers/06-typestate-lock/`: vulnerable.cpp + detect.py + detect.rs + README.md
- Update `docs/tool-comparison.md`: mark IDE solver and typestate as implemented
- Update `plans/PROGRESS.md`: E17 done, session log
- Update `plans/FUTURE.md`: mark IDE as implemented, add future extensions

## Future Extensions (not in this epic)

- **Constant propagation via IDE**: Value domain = abstract constants, edge functions for arithmetic
- **Linear constant propagation**: `x := a*y + b` — edge functions as affine transforms
- **Sparse IDE**: Sparsification for scalability (SparseHeros approach)
- **Library summaries**: Pre-computed typestate summaries for standard libraries
- **Backward IDE**: Demand-driven "what can reach this error state?"

## References

- Sagiv, Reps, Horwitz: "Precise Interprocedural Dataflow Analysis with Applications to Constant Propagation" (TCS'96)
- PhASAR: `IDETabulationProblem`, `IDETypeStateAnalysis`, `TypeStateDescription` (https://github.com/secure-software-engineering/phasar)
- Heros: `IDESolver`, `EdgeFunction`, `IDETabulationProblem` (https://github.com/Sable/heros)
- PhASAR Wiki: "Writing an IDE Analysis" (https://github.com/secure-software-engineering/phasar/wiki/Writing-an-IDE-analysis)
