# E23: Algorithm Correctness Verification

## Epic Overview

Systematically verify correctness of SAF's core analysis algorithms by comparing implementations against reference tools (SVF, PhASAR, IKOS) and academic papers. This epic uses critical thinking — reference implementations may have bugs or suboptimal designs, so we evaluate SAF's choices on their merits.

**Goals:**
1. Compare SAF algorithm implementations against canonical references
2. Identify discrepancies (bugs OR intentional improvements)
3. Create test cases that expose any bugs found
4. Fix confirmed bugs
5. Document lessons learned in CLAUDE.md if generalizable
6. Update `docs/tool-comparison.md` and `plans/FUTURE.md` if they contain inaccuracies or ambiguities discovered during verification
7. Add entries to `plans/future.md` (new file) for any new tutorials identified as needed

**Structure:** 4 plans, one per algorithm category:
- Plan 038: PTA Algorithm Verification (8 phases)
- Plan 039: IFDS/IDE Algorithm Verification (6 phases)
- Plan 040: Memory SSA + SVFG Algorithm Verification (7 phases)
- Plan 041: Abstract Interpretation Algorithm Verification (6 phases)

**Total Phases:** 27 (each completable in one Claude Code session)

---

## Plan 038: PTA Algorithm Verification

**Status:** done (35 E2E tests, all phases complete)

### Objective

Verify correctness of SAF's four PTA algorithms:
1. **Andersen CI** (`pta/solver.rs`, `pta/extract.rs`)
2. **k-CFA Context-Sensitive** (`cspta/solver.rs`, `cspta/context.rs`)
3. **Flow-Sensitive SFS** (`fspta/solver.rs`, `fspta/strong_update.rs`)
4. **CHA** (`cha.rs`, `llvm/cha_extract.rs`)

### Reference Implementations

- **SVF**: `Andersen.cpp`, `FlowSensitive.cpp`, `CHGraph.cpp`
- **Papers**: Andersen (1994), Hardekopf & Lin CGO'11 (SFS), Shivers (k-CFA)

### Phase 1: Andersen CI Constraint Extraction Study

**Goal:** Verify constraint extraction produces equivalent constraints to SVF.

**Tasks:**
1. Study SVF's constraint categories (AddrCGEdge, CopyCGEdge, LoadCGEdge, StoreCGEdge, GepCGEdge)
2. Compare SAF's `extract.rs` constraint types against SVF
3. Check edge cases: global initializers, function pointers, array collapsing
4. Create test cases for any missing extraction cases
5. Update `tool-comparison.md` if SAF's constraint model differs from documentation

**Deliverables:**
- Constraint extraction comparison document
- Test cases exposing discrepancies (if any)
- CLAUDE.md update for generalizable patterns

### Phase 2: Andersen CI Worklist Termination & Cycle Detection

**Goal:** Verify worklist algorithm terminates correctly and handles cycles.

**Tasks:**
1. Study SVF's `solveWorklist()` and wave propagation
2. Analyze SAF's `solve()` worklist with BTreeSet ordering
3. Test with cyclic constraint graphs (pointer cycles like `p = &q; q = &p`)
4. Verify `max_iterations` bound is sufficient
5. Document termination guarantees

**Test Program:**
```c
void* p; void* q;
p = &q; q = &p;
void* r = p;  // r should points to {p, q}
```

**Deliverables:**
- Cycle handling tests in `tests/fixtures/pta_verification/`
- Documentation of termination behavior

### Phase 3: k-CFA Context Representation & Propagation

**Goal:** Verify context-sensitive solver creates and propagates contexts correctly.

**Tasks:**
1. Study SVF's context handling in `ContextSensitiveAnalysis`
2. Analyze SAF's `CallSiteContext` = `Vec<InstId>` bounded by k
3. Verify context seeding via `seed_call_site_contexts()` BFS
4. Test context separation for wrapper functions
5. Update `tool-comparison.md` if k-CFA description needs refinement

**Test Program:**
```c
void* wrapper(void* p) { return p; }
void test() {
    void* a = malloc(1);
    void* b = malloc(1);
    void* r1 = wrapper(a);  // context [call1]
    void* r2 = wrapper(b);  // context [call2]
    // With k>=1: r1 should NOT alias r2
}
```

**Deliverables:**
- Context separation tests
- Documentation of context representation

### Phase 4: k-CFA Recursion Handling (SCC Collapse)

**Goal:** Verify recursive function handling doesn't create infinite contexts.

**Tasks:**
1. Study SVF's `SCCDetect()` for recursion
2. Analyze SAF's `compute_scc_functions()` using `tarjan_scc`
3. Test self-recursive and mutually recursive SCCs
4. Verify context collapse doesn't lose precision unnecessarily
5. Update `FUTURE.md` if demand-driven CS-PTA notes are inaccurate

**Test Programs:**
```c
// Self-recursive
int fact(int n) { return n <= 1 ? 1 : n * fact(n-1); }

// Mutual recursion
void ping(void* p) { pong(p); }
void pong(void* p) { ping(p); }
```

**Deliverables:**
- Recursion handling tests
- SCC collapse behavior documentation

### Phase 5: SFS Strong Update Conditions

**Goal:** Verify flow-sensitive strong update conditions match SVF exactly.

**Tasks:**
1. Study SVF's `FlowSensitive.cpp::isStrongUpdate()`:
   - Singleton points-to set
   - Not array-collapsed
   - Not in recursive function
2. Analyze SAF's `can_strong_update()` in `strong_update.rs`
3. Verify all three conditions implemented correctly
4. Test edge cases: arrays, recursive functions, multiple-target pointers
5. Update `tool-comparison.md` Flow-Sensitive PTA section if needed

**Test Programs:**
```c
int arr[10]; int* p = &arr[0];
*p = 1;  // Weak update (array)

int x; int* q = &x;
*q = 2;  // Strong update (singleton, non-array, non-recursive)
```

**Deliverables:**
- Strong update condition tests
- Documentation of any differences from SVF

### Phase 6: SFS IN/OUT Set Propagation

**Goal:** Verify dataflow propagation along SVFG edges.

**Tasks:**
1. Study SVF's `processSVFGNode()` and `propagate()`
2. Analyze SAF's `process_store()`, `process_load()`, `propagate_direct/indirect()`
3. Verify object-labeled indirect edges correctly filter propagation
4. Test PhiFlow edge propagation
5. Check `dfIn`/`dfOut` state management

**Deliverables:**
- Propagation correctness tests
- Edge semantics documentation

### Phase 7: CHA Vtable Parsing & Transitive Subclass

**Goal:** Verify class hierarchy construction from LLVM IR.

**Tasks:**
1. Study SVF's `CHGBuilder.cpp` vtable/typeinfo parsing
2. Analyze SAF's `cha_extract.rs` Itanium ABI handling
3. Test single inheritance, multiple inheritance, pure virtuals
4. Verify BFS transitive subclass computation in `cha.rs`
5. Update `tool-comparison.md` CHA section if needed

**Test Program:**
```cpp
class Base { virtual void foo(); };
class Derived : public Base { void foo() override; };
class GrandChild : public Derived { void foo() override; };
```

**Deliverables:**
- C++ inheritance hierarchy tests
- Vtable parsing documentation

### Phase 8: PTA Integration Tests & Bug Fixes

**Goal:** Create comprehensive test suite and fix any bugs found.

**Tasks:**
1. Create `tests/fixtures/pta_verification/` directory
2. Consolidate all test programs from Phases 1-7
3. Add Rust E2E tests with expected results
4. Fix any bugs discovered in earlier phases
5. Update CLAUDE.md with PTA correctness patterns
6. Review and update `tool-comparison.md` PTA sections for accuracy
7. Add tutorial entries to `plans/future.md` if needed

**Deliverables:**
- Complete PTA verification test suite
- Bug fixes (if any)
- Documentation updates

---

## Plan 039: IFDS/IDE Algorithm Verification

**Status:** done (27 E2E tests, all phases complete)

### Objective

Verify correctness of SAF's IFDS/IDE framework:
1. **IFDS Solver** (`ifds/solver.rs`)
2. **IDE Solver** (`ifds/ide_solver.rs`)

### Reference Implementations

- **PhASAR**: `IFDSSolver.h`, `IDESolver.h`
- **Heros (Java)**: Canonical IDE implementation
- **Papers**: Reps/Horwitz/Sagiv POPL'95 (IFDS), Sagiv/Reps/Horwitz TCS'96 (IDE)

### Phase 1: Zero-Fact Handling Study

**Goal:** Verify Lambda (zero fact) generation at all program points.

**Tasks:**
1. Study PhASAR's `submitInitialSeeds()` zero-fact seeding
2. Analyze SAF's `zero_value()` in `IfdsProblem` trait
3. Verify zero reaches all reachable instructions
4. Test conditional branches where zero must propagate to both arms
5. Update `tool-comparison.md` IFDS section if zero-fact handling differs

**Key Invariant:** Lambda must hold at every reachable program point.

**Test Program:**
```c
void sink(int);
void test(int x) {
    if (x) sink(1);  // Lambda should reach here
    else sink(2);    // Lambda should reach here too
}
```

**Deliverables:**
- Zero-fact propagation tests
- Seeding behavior documentation

### Phase 2: Summary Edge Memoization

**Goal:** Verify summary edges are correctly computed and reused.

**Tasks:**
1. Study PhASAR's `processExit()` summary edge creation
2. Analyze SAF's `summary_edges` map in `solver.rs`
3. Verify summaries applied immediately when call site revisited
4. Test interprocedural scenarios with multiple callers
5. Check context-insensitive summary correctness

**Deliverables:**
- Summary edge tests
- Memoization correctness documentation

### Phase 3: Edge Function Composition Order

**Goal:** Verify IDE edge function composition follows correct order.

**Tasks:**
1. Study Heros `f.composeWith(g)` = "apply g first, then f"
2. Analyze SAF's `compose_with()` in `edge_fn.rs`
3. Verify `Composed { first, second }` applies in correct order
4. Test composition with Identity, Constant, AllTop functions
5. Update `FUTURE.md` IDE extension notes if composition semantics unclear

**Key Identity:**
```
f.composeWith(g)(x) = f(g(x))
```

**Deliverables:**
- Composition order tests
- Edge function semantics documentation

### Phase 4: Jump Function Computation (Phase 1)

**Goal:** Verify IDE Phase 1 correctly computes jump functions.

**Tasks:**
1. Study TCS'96 algorithm: JumpFn[(d1, n, d2)] = composed edge function
2. Analyze SAF's `ide_propagate()` jump function updates
3. Verify join (not meet) used when new path found
4. Test re-propagation when jump function improves
5. Check monotonicity of jump function lattice

**Deliverables:**
- Jump function computation tests
- Phase 1 semantics documentation

### Phase 5: Value Propagation (Phase 2)

**Goal:** Verify IDE Phase 2 correctly propagates values.

**Tasks:**
1. Study TCS'96: MFP[(n, d)] = join over all paths of f(top)
2. Analyze SAF's Phase 2 in `ide_solver.rs`
3. Verify entry facts seeded with `top_value()`
4. Test value propagation through jump functions
5. Check join at merge points

**Key Invariant:** Entry facts get TOP, non-entry facts start as BOTTOM.

**Deliverables:**
- Value propagation tests
- Phase 2 semantics documentation

### Phase 6: IFDS/IDE Integration Tests & Bug Fixes

**Goal:** Create comprehensive test suite.

**Tasks:**
1. Create `tests/fixtures/ifds_verification/` directory
2. Add taint analysis tests with known-correct results
3. Add typestate analysis tests with known-correct results
4. Fix any bugs discovered in earlier phases
5. Update CLAUDE.md with IFDS/IDE patterns
6. Review `tool-comparison.md` IFDS/IDE sections for accuracy
7. Add tutorial entries to `plans/future.md` if needed

**Deliverables:**
- Complete IFDS/IDE verification test suite
- Bug fixes (if any)
- Documentation updates

---

## Plan 040: Memory SSA + SVFG Algorithm Verification

**Status:** done (35 E2E tests, all phases complete)

### Objective

Verify correctness of SAF's Memory SSA and SVFG algorithms:
1. **Memory SSA Skeleton** (`mssa/builder.rs`)
2. **Mod/Ref Summaries** (`mssa/modref.rs`)
3. **Clobber Walker** (`mssa/walker.rs`)
4. **SVFG Builder** (`svfg/builder.rs`)
5. **SVFG Queries** (`svfg/query.rs`)

### Reference Implementations

- **SVF**: `MemSSA.cpp`, `SVFGBuilder.cpp`
- **LLVM**: `MemorySSA.cpp`
- **Papers**: Cytron TOPLAS'91 (SSA), Sui CC'16 (SVF)

### Phase 1: Phi Placement via Iterated Dominance Frontier

**Goal:** Verify phi nodes placed at correct locations.

**Tasks:**
1. Study LLVM MemorySSA phi placement
2. Analyze SAF's `compute_dominance_frontiers()` and `iterated_dominance_frontier()`
3. Verify DF+ (iterated) used, not just DF
4. Test diamond CFG, loop CFG, complex control flow
5. Update `tool-comparison.md` Memory SSA section if needed

**Key Invariant:** DF+(S) = fixed point of DF(S) ∪ DF(DF(S)) ∪ ...

**Test Program:**
```c
// Diamond: phi at merge
if (cond) *p = 1; else *p = 2;
int x = *p;  // phi merges both stores
```

**Deliverables:**
- Phi placement tests
- IDF implementation documentation

### Phase 2: Dominator Computation Correctness

**Goal:** Verify Cooper-Harvey-Kennedy dominator algorithm.

**Tasks:**
1. Compare against LLVM's dominator tree (for reference)
2. Analyze SAF's `compute_idom()` in `builder.rs`
3. Verify `intersect()` helper for common ancestor
4. Test linear, diamond, loop, nested loop CFGs
5. Check irreducible CFG handling (if applicable)

**Key Invariants:**
- Entry block dominates all blocks
- No block dominates itself (except entry self-loop)
- Immediate dominator unique per block

**Deliverables:**
- Dominator computation tests
- Edge case documentation

### Phase 3: Mod/Ref Summary SCC Handling

**Goal:** Verify interprocedural mod/ref reaches fixed point.

**Tasks:**
1. Study SVF's mod/ref computation
2. Analyze SAF's `compute_mod_ref()` SCC iteration
3. Verify fixed-point convergence for mutual recursion
4. Test `MAX_ITERATIONS` bound sufficiency
5. Update `FUTURE.md` interprocedural notes if inaccurate

**Test Program:**
```c
void a(int* p) { *p = 1; b(p); }
void b(int* p) { a(p); }  // mutual recursion
```

**Deliverables:**
- SCC mod/ref tests
- Convergence guarantee documentation

### Phase 4: Clobber Query Precision

**Goal:** Verify clobber walker returns all potential clobbers.

**Tasks:**
1. Study LLVM's `getClobberingMemoryAccess()`
2. Analyze SAF's `clobber_for()` backward walk
3. Verify PTA consultation for disambiguation
4. Test aliased vs non-aliased store/load pairs
5. Check caching correctness

**Key Invariant:** If store S may write location L that load D reads, S must be in clobber set.

**Deliverables:**
- Clobber precision tests
- May-alias integration documentation

### Phase 5: Store-to-Load SVFG Edge Construction

**Goal:** Verify indirect edges connect aliased store/load pairs.

**Tasks:**
1. Study SVF's `connectIndirectSVFGEdges()`
2. Analyze SAF's SVFG Phase 3 clobber-based edge construction
3. Verify every load has edge from its clobber stores
4. Test alias-based edge filtering
5. Update `tool-comparison.md` SVFG section if needed

**Deliverables:**
- Store-to-load edge tests
- Edge construction documentation

### Phase 6: Memory Phi Edge Construction

**Goal:** Verify memory phi nodes have correct incoming edges.

**Tasks:**
1. Study SVF's memory phi representation
2. Analyze SAF's Phase 2 phi flow edges
3. Verify each memory phi has operand from each predecessor
4. Test PhiFlow edge connectivity
5. Check phi operand update after rename pass

**Deliverables:**
- Memory phi edge tests
- Phi edge semantics documentation

### Phase 7: MSSA/SVFG Integration Tests & Bug Fixes

**Goal:** Comprehensive test suite.

**Tasks:**
1. Create `tests/fixtures/mssa_verification/` directory
2. Add store/load disambiguation tests
3. Add interprocedural memory flow tests
4. Fix any bugs discovered in earlier phases
5. Update CLAUDE.md with MSSA/SVFG patterns
6. Review `tool-comparison.md` Memory SSA and SVFG sections
7. Add tutorial entries to `plans/future.md` if needed

**Deliverables:**
- Complete MSSA/SVFG verification test suite
- Bug fixes (if any)
- Documentation updates

---

## Plan 041: Abstract Interpretation Algorithm Verification

**Status:** done (55 E2E tests, all phases complete)

### Objective

Verify correctness of SAF's abstract interpretation framework:
1. **Interval Domain** (`absint/interval.rs`)
2. **Fixpoint Iterator** (`absint/fixpoint.rs`)

### Reference Implementations

- **IKOS**: `interval.hpp`, `fixpoint.cpp`
- **SPARTA**: `AbstractDomain.h`
- **Papers**: Cousot POPL'77, Navas APLAS'12 (wrapped intervals)

### Phase 1: Widening Timing at Loop Headers

**Goal:** Verify widening applied correctly to ensure convergence.

**Tasks:**
1. Study IKOS `InterleavedFwdFixpointIterator` widening strategy
2. Analyze SAF's `detect_loop_headers()` via DFS back-edge detection
3. Verify widening only at loop headers
4. Test simple loop, nested loops, multiple exit loops
5. Update `tool-comparison.md` AI section if needed

**Key Invariant:** Without widening at loop headers, infinite ascending chains possible.

**Test Program:**
```c
int x = 0;
while (x < 100) x++;
// Without widening: [0,0], [0,1], ... (never converges)
// With widening: [0,0], [0,+inf] (converges)
```

**Deliverables:**
- Widening timing tests
- Loop header detection documentation

### Phase 2: Threshold Widening Correctness

**Goal:** Verify threshold widening improves precision.

**Tasks:**
1. Study IKOS threshold extraction from ICmp constants
2. Analyze SAF's `extract_thresholds()` in `threshold.rs`
3. Verify `widen_with_thresholds()` uses threshold instead of infinity
4. Test precision improvement from thresholds
5. Update `FUTURE.md` threshold notes if inaccurate

**Test Program:**
```c
int x = 0;
while (x < 100) x++;  // threshold 100 extracted
// With threshold: [0,0], [0,100] (precise)
```

**Deliverables:**
- Threshold widening tests
- Threshold extraction documentation

### Phase 3: Wrapped vs Signed Interval Semantics

**Goal:** Verify interval arithmetic respects LLVM IR semantics.

**Tasks:**
1. Study IKOS wrapped intervals (Navas APLAS'12)
2. Analyze SAF's `Interval { lo, hi, bits }` representation
3. Verify `add()`, `sub()`, `mul()` handle overflow correctly
4. Test wrapping at 2^bits boundary
5. Compare against mathematical (non-wrapping) intervals

**Key LLVM semantics:** `add` is modular (wraps), `add nsw`/`nuw` have UB on overflow.

**Test Program:**
```c
uint8_t x = 255;
x = x + 1;  // Should wrap to 0, not overflow to 256
```

**Deliverables:**
- Wrapped arithmetic tests
- Overflow handling documentation

### Phase 4: Narrowing Iteration Count

**Goal:** Verify narrowing recovers precision after widening.

**Tasks:**
1. Study IKOS narrowing strategy and iteration count
2. Analyze SAF's `config.narrowing_iterations` (default: 2)
3. Test precision recovery after widening
4. Experiment with different iteration counts
5. Update `FUTURE.md` narrowing notes if needed

**Test Program:**
```c
int x = 0;
while (x < 100) x++;
// After widening: [0, +inf]
// After narrowing: [0, 100] at exit
```

**Deliverables:**
- Narrowing iteration tests
- Optimal iteration count analysis

### Phase 5: Join/Meet Lattice Correctness

**Goal:** Verify lattice operations satisfy lattice laws.

**Tasks:**
1. Study IKOS/SPARTA `AbstractDomain` trait requirements
2. Analyze SAF's `AbstractDomain` trait in `domain.rs`
3. Verify lattice laws with property-based tests:
   - Commutativity: `a.join(b) = b.join(a)`
   - Associativity: `a.join(b.join(c)) = a.join(b).join(c)`
   - Idempotence: `a.join(a) = a`
   - Absorption: `a.join(a.meet(b)) = a`
4. Test bottom/top identity properties
5. Update `tool-comparison.md` AI framework section

**Deliverables:**
- Property-based lattice law tests
- Lattice implementation documentation

### Phase 6: AbsInt Integration Tests & Bug Fixes

**Goal:** Comprehensive test suite.

**Tasks:**
1. Create `tests/fixtures/absint_verification/` directory
2. Add buffer overflow detection tests with known results
3. Add integer overflow detection tests with known results
4. Fix any bugs discovered in earlier phases
5. Update CLAUDE.md with AI patterns
6. Review `tool-comparison.md` Abstract Interpretation sections
7. Add tutorial entries to `plans/future.md` if needed

**Deliverables:**
- Complete AI verification test suite
- Bug fixes (if any)
- Documentation updates

---

## Cross-Cutting Concerns

### Documentation Updates

Each plan's final phase must review and update:
1. **`docs/tool-comparison.md`** — Mark incorrectly documented features, fix ambiguities, update gap analysis
2. **`plans/FUTURE.md`** — Fix inaccurate extension point descriptions, update status of deferred features
3. **`plans/future.md`** (new) — Centralized list of future tutorials to create
4. **`CLAUDE.md`** — Add generalizable lessons learned to coding conventions

### Test Organization

All verification tests go in `tests/fixtures/<category>_verification/`:
- `pta_verification/` — PTA algorithm tests
- `ifds_verification/` — IFDS/IDE tests
- `mssa_verification/` — Memory SSA and SVFG tests
- `absint_verification/` — Abstract interpretation tests

### Critical Thinking Guidelines

When comparing SAF against reference implementations:
1. **Don't assume reference is correct** — SVF, PhASAR, IKOS may have bugs too
2. **Evaluate trade-offs** — SAF may intentionally differ for determinism (NFR-DET) or simplicity
3. **Document intentional differences** — If SAF's approach is better, document why
4. **Create regression tests** — For any bug found, add a test that fails before the fix

---

## Summary

| Plan | Algorithms | Phases | Reference |
|------|------------|--------|-----------|
| 038 | Andersen CI, k-CFA, SFS, CHA | 8 | SVF |
| 039 | IFDS, IDE | 6 | PhASAR, Heros |
| 040 | Memory SSA (5 sub-algorithms) | 7 | SVF, LLVM |
| 041 | Interval AI, Fixpoint | 6 | IKOS, SPARTA |

**Total:** 27 phases across 4 plans

**Critical Files:**
- `crates/saf-analysis/src/pta/solver.rs` — Andersen CI
- `crates/saf-analysis/src/cspta/solver.rs` — k-CFA
- `crates/saf-analysis/src/fspta/solver.rs` — Flow-sensitive
- `crates/saf-analysis/src/ifds/solver.rs` — IFDS
- `crates/saf-analysis/src/ifds/ide_solver.rs` — IDE
- `crates/saf-analysis/src/mssa/builder.rs` — Memory SSA
- `crates/saf-analysis/src/svfg/builder.rs` — SVFG
- `crates/saf-analysis/src/absint/fixpoint.rs` — Fixpoint iterator
- `crates/saf-analysis/src/absint/interval.rs` — Interval domain
