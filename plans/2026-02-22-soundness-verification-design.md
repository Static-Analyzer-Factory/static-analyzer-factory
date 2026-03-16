# SAF Core Library Soundness Verification — Design Document

**Date:** 2026-02-22
**Goal:** Verify correctness of SAF's full analysis pipeline (PTA, CG, CFG, MSSA, SVFG, absint) through a three-pillar strategy combining formal proofs, property-based testing, and oracle-based empirical testing.

## Motivation

SAF has 1,782 Rust tests, proptest-based property tests, and benchmark suites (PTABen, Juliet). These provide strong **structural verification** (determinism, graph invariants, termination, cross-implementation equivalence). However, **soundness verification** — proving that the analysis computes correct results — relies primarily on statistical benchmark comparisons. There is no formal specification of what each analysis stage should compute, no oracle-based ground truth beyond benchmarks, and no differential testing infrastructure.

Plans 141-142 identified 99 defects (36 confirmed critical/high) through code audits, demonstrating that soundness bugs exist and are not caught by the current test suite.

## Architecture: Three-Pillar Verification

```
┌─────────────────────────────────────────────────────────────┐
│                  Verification Strategy                       │
├──────────────────┬──────────────────┬───────────────────────┤
│  Pillar 1: Kani  │  Pillar 2:       │  Pillar 3:            │
│  Formal Proofs   │  Property Tests  │  Oracle + Differential│
│  (bounded model  │  (proptest,      │  Testing              │
│   checking)      │   statistical)   │  (empirical)          │
├──────────────────┼──────────────────┼───────────────────────┤
│ PtsSet algebra   │ Constraint       │ Hand-crafted C progs  │
│ Constraint rules │ extraction       │ with known-correct    │
│ Temporal filter  │ completeness     │ analysis results      │
│ Spec registry    │                  │                       │
│ ID generation    │ PTA solver       │ SVF/Clang SA as       │
│                  │ monotonicity     │ reference (anomaly    │
│                  │ (large inputs)   │ detection, NOT ground │
│                  │                  │ truth)                │
│                  │ MSSA def-use     │                       │
│                  │ chain properties │                       │
│                  │                  │                       │
│                  │ SVFG reachability│                       │
│                  │ invariants       │                       │
├──────────────────┼──────────────────┼───────────────────────┤
│ Proof: exhaustive│ Statistical:     │ Empirical: ground     │
│ for bounded      │ 1000s of random  │ truth from human      │
│ inputs           │ inputs           │ analysis              │
│                  │                  │                       │
│ Zero production  │ Runs with normal │ Runs in Docker via    │
│ overhead —       │ cargo nextest    │ compile + analyze +   │
│ #[cfg(kani)]     │                  │ compare harness       │
└──────────────────┴──────────────────┴───────────────────────┘
```

## Pillar 1: Kani Formal Proofs

### What is Kani

Kani is a bit-precise model checker for Rust (developed by AWS). It uses `#[kani::proof]` harnesses with `kani::any()` to exhaustively verify properties over all possible inputs within bounds. It catches panics, overflows, assertion failures, and custom properties.

### Zero Production Impact

All Kani code is gated behind `#[cfg(kani)]` — completely stripped from normal `cargo build`, `cargo test`, and release builds. The `kani-verifier` crate is a dev-dependency only. The only cost is CI time for the verification job.

### Scope: Isolated, Bounded Components

Kani works on modules that can be verified exhaustively without LLVM dependencies or unbounded state spaces.

#### PtsSet Equivalence (5 implementations)

Prove that all 5 `PtsSet` implementations (BTree, BitVec, BDD, FxHash, Roaring) produce identical results for any sequence of operations.

```rust
/// PROPERTY: All PtsSet implementations agree on contains() after identical operations
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(50)]
fn proof_ptsset_cross_impl_equivalence() {
    let ops: [u32; 8] = kani::any();  // 8 arbitrary insert values
    let mut btree = BTreePtsSet::new();
    let mut bitvec = BitVecPtsSet::new();
    // ... all 5 impls
    for &val in &ops {
        btree.insert(val); bitvec.insert(val); // ...
    }
    let query: u32 = kani::any();
    assert_eq!(btree.contains(query), bitvec.contains(query));
    // ... all pairs
}
```

Properties to prove:
- `insert(x); contains(x) == true` (for all x, all impls)
- `union(A,B) == union(B,A)` (commutativity)
- `union(A, union(B,C)) == union(union(A,B), C)` (associativity)
- All 5 impls agree on `contains()` for identical operation sequences
- `insert(x); insert(x)` is idempotent (set size unchanged)

#### Constraint Rule Soundness

Prove that the PTA solver correctly processes each constraint type on small graphs.

```rust
/// PROPERTY: Addr constraint guarantees membership
/// If Addr(p, x) is in the constraint set, then x ∈ pts(p) after solving
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(20)]
fn proof_addr_constraint_soundness() {
    let p: ValueId = kani::any();
    let x: ValueId = kani::any();
    kani::assume(p != x);
    let mut constraints = ConstraintSet::new();
    constraints.addr.push(AddrConstraint { ptr: p, obj: x });
    let result = solve(constraints);
    assert!(result.pts(p).contains(x), "Addr(p,x) must put x in pts(p)");
}
```

Properties to prove:
- **Addr soundness:** `Addr(p, x)` → `x ∈ pts(p)`
- **Copy transitivity:** `Copy(p, q)` + `x ∈ pts(q)` → `x ∈ pts(p)`
- **Store/Load propagation:** `Store(p, q)` + `Load(r, p)` + `x ∈ pts(q)` → `x ∈ pts(r)` (when pts(p) overlaps)
- **Monotonicity:** Adding constraints never shrinks any points-to set
- **Fixed-point:** One more solver iteration produces no changes

#### Temporal Filter Logic

Prove ordering properties on small CFGs.

Properties to prove:
- If A strictly dominates B, then `can_happen_after(B, A) == true`
- If A and B are in the same basic block and A precedes B, temporal ordering is respected
- Symmetry: `can_happen_after(A, B) && can_happen_after(B, A)` only when A and B are in different branches

#### Spec Registry Invariants

Properties to prove:
- YAML spec lookup is deterministic (same function name → same spec)
- Derived spec overlay takes priority over YAML base
- `LookupResult::NotFound` when no spec exists
- Depth guard prevents infinite recursion in computed bounds

#### ID Generation

Properties to prove:
- Domain separation: `FunctionId::derive(x) != BlockId::derive(x)` for all x
- Determinism: `make_id(domain, data) == make_id(domain, data)` always
- Collision resistance: `make_id(d, x) != make_id(d, y)` when `x != y` (for bounded inputs)

### Kani Infrastructure

- `kani-verifier` added as dev-dependency to `saf-core` and `saf-analysis`
- Proof harnesses in `#[cfg(kani)] mod proofs { ... }` blocks alongside source
- Each proof has a doc comment explaining the property in plain English
- CI job: `cargo kani --package saf-core && cargo kani --package saf-analysis` (analysis subset that doesn't require LLVM)

## Pillar 2: Property Tests (proptest)

### Scope: Pipeline-Level, Statistical

For stages where Kani can't scale due to state space (large constraint graphs, full MSSA/SVFG construction), use proptest to statistically verify properties over thousands of random inputs.

#### Constraint Extraction Completeness

Extend existing `proptest_arb.rs` generators to verify:
- Every `alloca` instruction produces an `AddrConstraint`
- Every `store` instruction produces a `StoreConstraint`
- Every `load` instruction produces a `LoadConstraint`
- Every pointer-typed `cast` produces a `CopyConstraint`
- No constraint references an undefined `ValueId`

#### PTA Solver Monotonicity (Large Inputs)

- Generate random constraint sets with 100-1000 nodes
- Verify: solving `constraints ∪ {new}` produces a superset of solving `constraints`
- Verify: two runs produce identical output (determinism)
- Verify: fixed-point convergence within iteration limit

#### MSSA Def-Use Chain Properties

- Every memory use has exactly one reaching definition
- Phi nodes are placed at dominance frontiers of memory definitions
- Def-use chains respect control flow (no def reaches a use that is unreachable from the def)

#### SVFG Value-Flow Invariants

- Every value-flow edge connects compatible types
- No orphan nodes (every node has at least one edge or is a root/leaf)
- Reachability: if source S flows to sink T via direct edges, there exists a path S→...→T in the SVFG

### Proptest Infrastructure

- Extend `crates/saf-analysis/src/proptest_arb.rs` with new generators
- New property test files per stage in `crates/saf-analysis/src/`
- CI runs with 10,000 iterations (default 256 is too few for soundness confidence)
- Failed cases are automatically minimized and saved as regression fixtures

## Pillar 3: Oracle + Differential Testing

### Hand-Crafted Oracle Suite

Small C programs with human-verified expected analysis results. Tests the core library's graph/analysis outputs, NOT checker verdicts (checkers are applications of the library).

#### Directory Structure

```
tests/verification/oracle/
├── pta/                          # Points-to analysis
│   ├── simple_alias.c
│   ├── simple_alias.expected.yaml
│   ├── conditional_alias.c
│   ├── conditional_alias.expected.yaml
│   └── ...
├── callgraph/                    # Call graph construction
├── cfg/                          # Control flow graph
├── mssa/                         # Memory SSA
├── svfg/                         # Value-flow graph
└── harness/                      # Test harness code
    └── oracle_runner.rs
```

#### Expected Results Format (YAML)

```yaml
# tests/verification/oracle/pta/conditional_alias.expected.yaml
description: "Conditional pointer assignment creates phi-node aliasing"
layer: pta
difficulty: basic

expectations:
  points_to:
    - pointer: "p"
      at_line: 12
      must_contain: [x, y]       # Soundness: these MUST be in pts(p)
      may_only_contain: [x, y]   # Precision: nothing else should be

  alias:
    - pair: [p, "&x"]
      relation: may_alias
    - pair: [p, "&y"]
      relation: may_alias
    - pair: [p, "&z"]
      relation: no_alias         # z is never assigned to p
```

Key design: expectations use **human-readable variable/function names**, not internal IDs. The test harness maps names to IDs via LLVM debug info.

#### Soundness vs. Imprecision Distinction

The harness reports separately:
- **Unsoundness** (missing items in `must_contain`) — this is a bug, test FAILS
- **Imprecision** (extra items beyond `may_only_contain`) — this is acceptable, test WARNS
- **Correct** (exact match) — test PASSES

#### Oracle Corpus: Corner Case Coverage

**PTA (20 programs):**

| Category | Count | Corner cases |
|----------|-------|-------------|
| Basic aliasing | 3 | address-of, copy, phi-node conditional assignment |
| Multi-level pointers | 3 | `**p`, pointer-to-pointer, dereference chains |
| Struct/field access | 3 | `p->field`, nested structs, field-sensitive vs insensitive |
| Heap & arrays | 3 | malloc, array elements, realloc |
| Function pointers | 3 | simple fptr, fptr arrays, fptr through struct fields |
| Interprocedural | 3 | parameter passing, return values, context-sensitive (same fn, different args) |
| Edge cases | 2 | void* casting, globals, recursive data structures |

**Call Graph (10 programs):**

| Category | Count | Corner cases |
|----------|-------|-------------|
| Direct | 2 | simple calls, mutually recursive |
| Indirect (PTA-resolved) | 4 | fptr call, fptr array dispatch, fptr as param, fptr from return |
| Edge cases | 4 | unreachable functions, external/library calls, varargs, callbacks |

**CFG (8 programs):**

| Category | Count | Corner cases |
|----------|-------|-------------|
| Branching | 3 | if/else, nested, switch/case |
| Loops | 3 | for/while/do-while, break/continue, nested |
| Edge cases | 2 | goto, unreachable code after return, empty blocks |

**MSSA (8 programs):**

| Category | Count | Corner cases |
|----------|-------|-------------|
| Basic defs | 2 | simple store/load, overwrite |
| Phi nodes | 3 | conditional store, loop-carried dep, aliased stores through different ptrs |
| Interprocedural | 3 | store in callee visible to caller, memcpy/memset, global mutation |

**SVFG (8 programs):**

| Category | Count | Corner cases |
|----------|-------|-------------|
| Direct flow | 2 | assignment chain, return value |
| Indirect flow | 3 | store→load, aliased pointer flow, struct field flow |
| Interprocedural | 3 | param→use, caller→callee→return, global-mediated flow |

**Total: 54 oracle programs**, each 10-30 lines of C.

#### Oracle Harness

The harness:
1. Compiles each `.c` file to LLVM IR (via clang in Docker)
2. Loads IR through SAF frontend
3. Runs the relevant analysis (PTA, CG build, CFG build, MSSA, SVFG)
4. Maps human-readable names from YAML to internal IDs via debug info
5. Compares analysis results against expectations
6. Produces a human-readable report distinguishing soundness failures from imprecision

#### Human-Readable Report

```
=== Oracle Verification Report ===

PTA (20 programs):
  [PASS] simple_alias.c          pts(p) = {x} ✓
  [PASS] conditional_alias.c     pts(p) = {x, y} ✓
  [WARN] struct_field.c          pts(p->next) expected {node2}, got {node2, node3}
                                  ↳ imprecision (not unsoundness)
  [FAIL] multi_level.c           pts(**pp) expected {x, y}, got {x}
                                  ↳ UNSOUND: missing {y}
  ...

Call Graph (10 programs):
  [PASS] direct_call.c           calls(main) = {foo, bar} ✓
  ...

Overall: 48/54 pass | 4 imprecision | 2 unsoundness
```

### Differential Testing (Future Iteration)

Compare SAF against SVF and Clang Static Analyzer on shared corpus. These tools are **references, not ground truth** — divergences are flagged for human review, not automatically treated as bugs.

Deferred to second iteration to keep first iteration focused.

## Make Commands

```makefile
# ─── Pillar 1: Kani Formal Proofs ─────────────────────────
make verify-kani           # Run ALL Kani proofs (saf-core + saf-analysis)
make verify-kani-core      # Run Kani proofs for saf-core only
make verify-kani-analysis  # Run Kani proofs for saf-analysis only

# ─── Pillar 2: Property Tests (proptest) ──────────────────
make verify-props          # Run ALL property tests (10,000 iterations)
make verify-props-quick    # Run property tests (256 iterations, faster)

# ─── Pillar 3: Oracle Suite ───────────────────────────────
make verify-oracle         # Run full oracle suite (compile C + verify)
make verify-oracle LAYER=pta       # Single layer
make verify-oracle LAYER=callgraph # Other: cfg, mssa, svfg
make compile-oracle        # Compile oracle C files to LLVM IR only

# ─── Combined ─────────────────────────────────────────────
make verify                # Run ALL three pillars
make verify-quick          # Kani(core) + props(256 iters) + oracle
```

## First Iteration Scope

| Pillar | Scope | Effort |
|--------|-------|--------|
| Kani | PtsSet equivalence proofs + constraint rule soundness | 1-2 weeks |
| Proptest | Constraint extraction completeness | 1 week |
| Oracle | 54 hand-crafted C programs (PTA, CG, CFG, MSSA, SVFG) + harness | 2-3 weeks |
| Make commands | `verify-kani`, `verify-props`, `verify-oracle`, `verify` | 1-2 days |
| **Total** | | **5-7 weeks** |

## Future Iterations

- **Iteration 2:** Differential testing against SVF + Clang SA
- **Iteration 3:** Abstract interpretation oracle (interval/nullness at program points)
- **Iteration 4:** Fuzzing harness (random LLVM IR generation + crash detection)
- **Iteration 5:** Kani expansion to MSSA phi placement, SVFG edge construction
