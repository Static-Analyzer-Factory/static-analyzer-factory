# Plan 086: PTABen Unified Improvement Campaign

## Status: approved
## Epic: E41 (PTABen Precision Campaign)

## Baseline

PTABen (846 files, 2883 oracle checks): **2046 Exact, 363 Sound, 79 ToVerify, 263 Unsound, 132 Skip.**

## Root Cause Synthesis

Four analysts independently investigated all 263 unsound cases across 16 categories. After merging duplicates and overlapping root causes, 17 distinct root causes reduce to 8 implementation phases ordered by impact-to-effort ratio.

### Cross-Analyst Overlap Resolution

| Merged Root Cause | Source Analysts | Original IDs | Combined Cases |
|---|---|---|---|
| CHA over-resolution / parameter pollution | Alias RC-1A/C + Other RC-2a/b | 7 + 7 | 14 |
| SVFG interprocedural edge gaps | Checker RC-1 + RC-2 | 30 + 18 | 48 |
| Memcpy overflow harness wiring | Absint RC-1 | 31 | 31 |
| MTA context-sensitive thread discovery | Other RC-1a/b | 6 + 11 | 17 |
| Branch/switch refinement lost at join | Absint RC-2 | 23 | 23 |
| Recursive summary returns TOP | Absint RC-3 | 29 | 29 |
| Overflow FP from CI-PTA conflation | Absint RC-5 | 16 | 16 |
| Various category-specific | Multiple | Various | 85 |

### Full Root Cause Inventory (263 unsound)

| # | Root Cause | Cases | Difficulty | Phase |
|---|-----------|-------|------------|-------|
| U1 | Memcpy/loop overflow not wired into harness | 31 | Easy | A |
| U2 | MTA indirect fork + context-sensitive indexing | 17 | Medium | B |
| U3 | Overflow FP: CI-PTA cross-function conflation | 16 | Medium | C |
| U4 | Nested struct FP / global const init (Unknown PTS) | 8 | Medium | D |
| U5 | Static member return alloca tracking | 5 | Medium | D |
| U6 | Null/pointer-as-integer assertion (noreturn, trunc) | 5 | Easy-Medium | E |
| U7 | CHA over-resolution / multi-inheritance param pollution | 14 | Hard | F |
| U8 | CS-PTA context pollution / SCC collapse | 6 | Hard | F |
| U9 | Nullness: array element + struct field tracking | 4 | Medium | E |
| U10 | Interprocedural array/pointer value TOP | 10 | Medium | G |
| U11 | Loop induction variable narrowing | 5 | Medium | G |
| U12 | Branch/switch refinement lost at join | 23 | Hard | H |
| U13 | Recursive function summary TOP | 29 | Hard | H |
| U14 | SVFG interprocedural wrapper edge gaps | 30 | High | I |
| U15 | PartialLeak: conditional free not detected | 18 | High | I |
| U16 | Path-sensitive alias: loop back-edge / interproc | 3 | Very Hard | defer |
| U17 | Complex relational / global / string reasoning | 27 | Architectural | defer |
| U18 | Per-allocation NeverFree sanitizer filtering | 1 | Low-Medium | E |
| U19 | Nullness path-sensitivity (dangleptr branch) | 3 | High | defer |
| U20 | Array element null merge (over-precision) | 1 | Medium | defer |

**Addressable total: ~197 cases (Phases A-I)**
**Deferred: ~34 cases (require architectural changes or new domains)**
**Overlap note**: Some cases are shared across U-numbers (e.g., double_free 3 cases counted in U14 are MemLeak oracles within double_free tests).

---

## Phase A: Wire Memcpy Overflow Checker into PTABen Harness

**Impact: ~31 cases (ae_overflow 29 + ae_overflow_fail 2)**
**Effort: ~40 LOC, Easy**
**Risk: None (harness-only change)**

### Problem

The existing `check_memcpy_overflow_with_pta_and_specs()` function detects buffer overflows via memcpy/memmove/strncpy size vs destination allocation size. However, the PTABen harness at `ptaben.rs:1776` only calls `check_buffer_overflow_with_pta_and_result()` (GEP-based checker). All 31 unsound cases show "memcpy tracking not supported".

### Tasks

**A1: Wire memcpy checker into PTABen UNSAFE/SAFE_BUFACCESS validation (~30 LOC in ptaben.rs)**

In the `validate_buffer_overflow()` function, after calling `check_buffer_overflow_with_pta_and_result()`, also call `check_memcpy_overflow_with_pta_and_specs()`. Merge findings from both checkers. For UNSAFE oracles: report Exact if either checker finds an overflow. For SAFE oracles: report Unsound only if both checkers agree on an overflow (or if either reports one -- keep conservative).

**A2: Handle loop-copy patterns (~10 LOC)**

For `CWE805_*_loop_*` tests, the overflow occurs through `for(i=0; i<100; i++) dest[i]=src[i]` patterns. The GEP-based checker should already detect these (GEP index exceeds allocation). Verify that the GEP checker catches loop-indexed overflows, and if not, ensure loop bounds are extracted as widened intervals that exceed allocation sizes.

### Expected Outcome

ae_overflow unsound: 47 -> ~16 (31 fixed by wiring, 16 remain as FPs from Phase C).
ae_overflow_fail unsound: 2 -> 0.

---

## Phase B: MTA Context-Sensitive Thread Discovery

**Impact: ~17 cases (mta)**
**Effort: ~80 LOC, Medium**
**Risk: Low (MTA module is isolated)**

### Problem

Two bugs in `discovery.rs`:
1. **Indirect fork**: When PTA resolves `pthread_create`'s function argument to multiple targets, only one thread is created per call site instead of one per target.
2. **Context-sensitive indexing**: The visited set uses `(thread_id, func_id)` as key, preventing re-exploration of the same function from different call sites. Oracle expects distinct thread IDs per calling context.

### Tasks

**B1: Fork thread IDs per resolved target (~30 LOC in discovery.rs)**

In `discover_threads_bfs()`, when processing a `pthread_create` call with an indirect function argument, resolve via PTA and create one `ThreadInfo` per resolved target function. Assign incrementing thread IDs. Currently picks first target only.

**B2: Call-site-sensitive visited set (~30 LOC in discovery.rs)**

Change the visited set key from `(ThreadId, FunctionId)` to `(ThreadId, FunctionId, InstId)` where `InstId` is the call instruction that entered the function. This allows re-exploration of the same function when called from different sites within the same thread.

**B3: Context chain in ThreadInfo (~20 LOC)**

Store the call site chain in `ThreadInfo` so that the oracle matching (`CXT_THREAD(id, "cs1.foo1,cs2.foo2")`) can validate against the discovered context path.

### Expected Outcome

mta unsound: 17 -> 0.

---

## Phase C: Overflow False Positive Reduction

**Impact: ~16 cases (ae_overflow)**
**Effort: ~60 LOC, Medium**
**Risk: Low (checker precision improvement)**

### Problem

For SAFE_BUFACCESS oracles, SAF incorrectly reports buffer overflow findings due to CI-PTA conflating `_bad()` and `_good()` function pointer targets. The allocation size from `malloc(10)` in `_bad()` and `malloc(40)` in `_good()` gets joined to `[10,40]`, and findings from `_bad()` match the SAFE oracle in `_good()`.

### Tasks

**C1: Function-scoped finding matching (~30 LOC in ptaben.rs)**

When validating SAFE_BUFACCESS oracles, only match findings whose affected pointer is in the same function as the oracle call site. Reject cross-function finding matches.

**C2: Per-function allocation size isolation (~30 LOC in checker.rs)**

In `check_buffer_overflow_with_pta_and_result()`, when building `loc_alloc_sizes`, use context-sensitive PTA if available to get per-function allocation sizes. Alternatively, filter allocation sites by function reachability from the GEP instruction.

### Expected Outcome

ae_overflow unsound: ~16 -> ~0 (after Phase A).

---

## Phase D: PTA Constraint Extraction for Missing Patterns

**Impact: ~13 cases (basic_c 8 + basic_cpp 5)**
**Effort: ~200 LOC, Medium**
**Risk: Low-Medium (PTA solver changes, test carefully)**

### Problem

13 cases return "Unknown" meaning one or both queried ValueIds have no points-to set. Two distinct gaps:

1. **Nested struct function pointer in global initializers (8 basic_c)**: `extract_global_initializers()` doesn't recursively decompose nested `Constant::Aggregate` containing `Constant::GlobalRef` for function pointers at depth > 1.
2. **Static member function return through alloca (5 basic_cpp)**: Static member functions returning pointers lose identity through the -O0 `retval` alloca chain. The loaded return value at the call site has no PTS.

### Tasks

**D1: Recursive global initializer decomposition (~80 LOC in extract.rs)**

In `extract_aggregate_elements()`, handle nested `Constant::Aggregate` by recursively decomposing. For each `Constant::GlobalRef` at any nesting depth, create a Store constraint linking the global's field location to the referenced function/global address.

Also handle `Constant::GlobalRef` inside struct fields of struct fields (depth 2+). Create GEP paths matching the nesting: `[Field{0}, Field{1}]` for a function pointer in the second field of the first field.

**D2: Static return value constraint linking (~80 LOC in extract.rs or value_origin.rs)**

For static member functions returning pointers, trace the return value chain: `CallDirect` instruction's result ValueId <- function's return instructions <- Store to retval alloca <- Load from retval alloca. Add Copy constraints linking the call-site result to the callee's return value.

Alternatively, extend `build_param_indices()` to also build a return-value-to-call-site mapping for static member functions, then use this in Strategy 2 of `validate_alias()`.

**D3: Collapsed-index fallback for heap arrays (~40 LOC in solver.rs or result.rs)**

When a constant-indexed load (e.g., `disp[1]`) finds no field location at `[Field{0}, Field{1}]`, check if a collapsed `Index(Unknown)` location exists for the same object. If so, use its PTS as a conservative answer. This fixes the remaining spec-equake oracle.

### Expected Outcome

basic_c unsound: 8 -> ~0.
basic_cpp unsound: 12 -> ~7 (5 static-return fixed; 7 CHA issues remain for Phase F).

---

## Phase E: Nullness and Assertion Precision Improvements

**Impact: ~10 cases (ae_nullptr_deref 3-4 + ae_assert_fail 5 + mem_leak 1)**
**Effort: ~100 LOC, Easy-Medium**
**Risk: Low**

### Problem

Three sub-issues:
1. **Pointer-as-nonzero**: `svf_assert(ptr != NULL)` where `ptr = &value` fails because the condition prover has no integer interval for pointer values.
2. **Boolean through trunc**: `trunc i8 to i1` loses boolean value tracking.
3. **Noreturn function modeling**: `exit(0)` / `abort()` after assert means the assert is only reachable if condition is true.
4. **Array/struct field nullness**: PTA loc_memory can track per-location nullness.
5. **Per-allocation sanitizer filtering**: NeverFree detection with shared module sanitizers.

### Tasks

**E1: Pointer address as non-zero interval (~15 LOC in transfer.rs)**

When processing `Alloca` or `GlobalRef` address operations, store `Interval::new(1, i64::MAX, 64)` for the result ValueId. This makes `ICmpNe(ptr, 0)` provable for stack/global addresses.

**E2: Boolean trunc preservation (~15 LOC in transfer.rs or interval.rs)**

In `Trunc` handler: when source interval is `[0,1]` or `[1,1]` and target_bits is 1, preserve the value as `[0,1]` or `[1,1]` instead of computing `trunc` which may lose precision.

**E3: Noreturn function specs (~20 LOC in function_properties.rs + fixpoint.rs)**

Add `exit`, `abort`, `_exit`, `_Exit` to `is_known_noreturn_function()`. In the fixpoint solver, when a block ends with a call to a noreturn function, don't propagate its state to successors.

**E4: Loc-memory nullness propagation (~30 LOC in nullness.rs)**

When a Store writes a `NotNull` value (e.g., `&foo`, `malloc()` return) to a PTA location, record the nullness in `loc_memory`-style per-location tracking. Subsequent Loads from the same location inherit `NotNull`.

**E5: Per-allocation sanitizer filtering (~20 LOC in solver.rs)**

In `must_not_reach` solver, when checking the no-sanitizer fallback, verify the sanitizer is reachable via a path that carries the same allocation pointer (not a different allocation's pointer). Use PTA may-alias to check if the sanitizer's argument may alias the allocation.

### Expected Outcome

ae_nullptr_deref unsound: 3 -> ~1 (safe_ptr_array_access may need deeper work).
ae_nullptr_deref_failed unsound: 4 -> ~2.
ae_assert_fail unsound: 44 -> ~39 (5 RC-7 cases fixed).
mem_leak unsound: 49 -> ~48 (1 NeverFree fixed).

---

## Phase F: CHA Precision and CS-PTA Context Improvements

**Impact: ~20 cases (basic_cpp 7 + failed_tests 7 + cs_tests 6)**
**Effort: ~300 LOC, Hard**
**Risk: Medium (CHA changes affect all C++ tests; regression-test carefully)**

### Problem

CHA resolves virtual calls to ALL methods at the same vtable slot across ALL classes in the type hierarchy, regardless of receiver type. This causes parameter pollution when methods from unrelated classes share the same slot index.

CS-PTA has remaining context pollution issues: cs18 (4 cases) needs full context-sensitive loc_pts, and recur6/cs9 need SCC context handling.

### Tasks

**F1: Receiver-type-aware CHA resolution (~120 LOC in cg_refinement.rs)**

In `resolve_virtual_calls_via_cha()`, when resolving a virtual call on receiver `obj`, use PTA to determine `obj`'s allocation type(s). Filter CHA targets to only methods from those types and their subtypes, not all classes at the same slot.

Fallback to current behavior (all types) when receiver type is unknown.

**F2: Per-receiver-type parameter binding in Strategy 2 (~60 LOC in ptaben.rs)**

In `validate_alias()` Strategy 2, when tracing per-call-site arguments for virtual dispatch, narrow the resolved targets by receiver type (same as F1) before mapping oracle params to caller args.

**F3: CS-PTA full context-sensitive loc_pts for non-SCC functions (~80 LOC in cspta/solver.rs)**

For cs18: remove the CI loc_pts fallback for non-recursive functions. Use `(LocId, CallerContext)` lookup when the store was in a callee context, falling back to `(LocId, [])` only for globals and address-taken variables.

**F4: CS-PTA bounded SCC unrolling (~40 LOC in cspta/solver.rs)**

For recur6: allow 1 level of recursive context before collapsing to empty context. This gives the first recursive call its own context, while deeper recursion collapses. Modest improvement for simple recursive patterns.

### Expected Outcome

basic_cpp unsound: ~7 -> ~0.
failed_tests unsound: 7 -> ~0.
cs_tests unsound: 6 -> ~2 (cs9 may remain due to indirect call complexity).

---

## Phase G: Absint Interprocedural and Loop Precision

**Impact: ~15 cases (ae_assert 10 + ae_assert_fail 5)**
**Effort: ~200 LOC, Medium-Hard**
**Risk: Medium (fixpoint changes need convergence testing)**

### Problem

Two sub-issues:
1. **Interprocedural array element access**: `getValue(arr, idx)` returns TOP because inline analysis can't resolve specific array element through parameter GEP+Load.
2. **Loop induction variable**: Narrowing phase doesn't propagate through loop back-edges correctly.

### Tasks

**G1: Inline analysis GEP index binding (~80 LOC in transfer.rs)**

In `analyze_callee_inline()`, when the callee has a GEP with index from a parameter, bind the parameter to the caller's concrete argument interval. Use this to select the specific PTA location for the array element, then load from that location's interval in `loc_memory`.

**G2: Loop-exit condition refinement (~50 LOC in fixpoint.rs)**

After the ascending phase (widening) completes for a loop, apply the negated loop condition to the loop exit edge. For `while (i < 5)`, the exit edge gets `i >= 5`. Apply this as a refinement to the state flowing out of the loop.

**G3: Narrowing back-edge propagation fix (~70 LOC in fixpoint.rs)**

In the narrowing phase, ensure the narrowed state from the loop header propagates through the loop body blocks and back through the back-edge phi incoming value. Currently the back-edge incoming is bottom because the narrowing doesn't re-process loop body blocks.

### Expected Outcome

ae_assert unsound: 29 -> ~19 (10 RC-4 cases addressed, 19 RC-2 remain for Phase H).
ae_assert_fail unsound: ~39 -> ~34 (5 RC-6 loop cases partially addressed).

---

## Phase H: Branch Refinement and Recursive Summary (Hard)

**Impact: ~52 cases (ae_assert 19-23 + ae_recursion 29)**
**Effort: ~400 LOC, Hard**
**Risk: High (architectural changes to fixpoint; risk of regressions in all ae_* categories)**

### Problem

1. **Branch/switch refinement lost at join (23 cases)**: After mem2reg, branch conditions refine SSA ValueIds directly, but at multi-predecessor join points, the refined state is destroyed by the join/widen operation. Prior Plans 074/078/080/084 all made incremental progress but the core issue remains.

2. **Recursive summary returns TOP (29 cases)**: Multiple sub-patterns each need different fixes (scanf TOP input, nested recursion mc91, global side effects, aggressive widening).

### Tasks

**H1: Dominator-based refinement propagation (~150 LOC in fixpoint.rs)**

Compute the dominator tree for the function CFG. After join/widen at a block entry, re-apply refinements from ALL dominating branch conditions (not just immediate predecessors). A branch condition at block B0 that dominates block B5 means B5 is only reachable through that branch, so the refinement is valid at B5.

This handles switch cases (all dominated by the switch block), nested if-else (inner blocks dominated by outer condition), and sequential assertions after branches.

**H2: Return-value clamping detection for recursive functions (~80 LOC in interprocedural.rs)**

For `recursive_id` pattern: detect "if (ret > N) return N" clamp patterns in recursive functions. Extract the clamp bound as a widening threshold for the return interval. Apply it during the recursive SCC fixpoint.

**H3: Nested recursion bounded unrolling (~100 LOC in interprocedural.rs)**

For `recursive_mc91` pattern: allow 1-2 levels of nested recursive call evaluation before widening. Evaluate `mc91(mc91(p+11))` by computing the inner call's interval first, then using that as input to the outer call.

**H4: Global variable summary tracking (~70 LOC in interprocedural.rs)**

For `recursive_afterrec` pattern: extend `FunctionSummary` with `global_effects: BTreeMap<ValueId, Interval>` to track which globals a function modifies and to what value. Apply these effects at call sites during interprocedural analysis.

### Expected Outcome

ae_assert unsound: ~19 -> ~0 (switch/branch cases fixed by dominator refinement).
ae_recursion unsound: 29 -> ~10 (recursive_id 10, recursive_afterrec 5, recursive_addition 4 fixable; mc91 8 + demo 1 likely remain).

---

## Phase I: SVFG-Based Checker Improvements (High Effort)

**Impact: ~48 cases (mem_leak 48 + double_free 3)**
**Effort: ~500+ LOC, High**
**Risk: High (SVFG and solver changes affect all checker-based categories)**

### Problem

81% of checker unsound (48/59) stems from SVFG interprocedural edge gaps:
- RC-1 (30 cases): Wrapper function flows, struct field stores, global variable flows, multi-level call chains not connected in SVFG.
- RC-2 (18 cases): PartialLeak patterns where conditional free on one path is not detected as missing on other paths.

### Tasks

**I1: PTA-augmented SVFG edges for allocator flows (~150 LOC in svfg/builder.rs)**

For each `HeapAlloc` source (malloc/SAFEMALLOC), query PTA to find all `Deallocator` (free/SAFEFREE) call sites whose argument may-aliases the allocation. Add synthetic `IndirectDef` edges in the SVFG connecting the allocation's return value to the deallocator's argument. This bridges the interprocedural gap without full SVFG edge reconstruction.

**I2: PartialLeak detection in must_not_reach solver (~120 LOC in solver.rs)**

Enhance the `must_not_reach` solver to detect partial leaks:
1. Run BFS from allocation source. Track both sanitized paths (those that reach a free) and unsanitized paths (those that reach a function exit).
2. If BOTH sanitized and unsanitized exits exist, generate a PartialLeak finding.
3. Currently the solver only generates findings when NO sanitizer is reachable or when an unsanitized exit is found; it misses the "some paths sanitized, some not" pattern.

**I3: Loop-body allocation-deallocation linking (~80 LOC in svfg/builder.rs)**

For the 3 double_free MemLeak cases: `for(i=0; i<1; i++) { p=SAFEMALLOC(); SAFEFREE(p); }` — the SVFG phi node at the loop header merges the pre-loop (no allocation) and post-free (freed) states. Ensure the SVFG connects the in-loop SAFEMALLOC to the in-loop SAFEFREE through the loop body, not just through the loop header phi.

**I4: Consider IFDS-based leak checker (architectural, optional)**

The checker framework uses BFS on SVFG which is fundamentally limited for interprocedural must-not-reach reasoning. An IFDS-based approach with "allocated-not-freed" as a data-flow fact would handle all these patterns naturally. This is a larger architectural change but would be more robust long-term.

### Expected Outcome

mem_leak unsound: 49 -> ~5-10 (significant improvement but some complex patterns may remain).
double_free unsound: 3 -> 0.

---

## Deferred (Architectural / Out of Scope for Plan 086)

| Root Cause | Cases | Reason for Deferral |
|-----------|-------|---------------------|
| U16: Path-sensitive alias loop back-edge | 3 | Requires loop-aware path enumeration beyond per-path re-solving |
| U17: Complex relational/global/string | 27 | Requires octagon/polyhedra domain, string domain, symbolic reasoning |
| U19: Nullness path-sensitivity | 3 | Requires coupling nullness with interval analysis or conditional state |
| U20: Array element null merge | 1 | Same root cause as U9 but inverse direction; needs per-index tracking |

---

## Phase Execution Order

| Phase | Effort | Impact | Net Cases Fixed | Dependencies |
|-------|--------|--------|----------------|--------------|
| A | ~40 LOC | Easy | ~31 | None |
| B | ~80 LOC | Medium | ~17 | None |
| C | ~60 LOC | Medium | ~16 | After Phase A |
| D | ~200 LOC | Medium | ~13 | None |
| E | ~100 LOC | Easy-Medium | ~10 | None |
| F | ~300 LOC | Hard | ~20 | None |
| G | ~200 LOC | Medium-Hard | ~15 | None |
| H | ~400 LOC | Hard | ~40-52 | After G |
| I | ~500+ LOC | High | ~48 | None |

**Phases A-E can run in parallel** (independent code areas).
**Phase C depends on Phase A** (to assess remaining FPs after wiring).
**Phase H depends on Phase G** (loop fixes inform branch refinement).

### Projected Outcomes (Cumulative)

| After Phase | Exact | Unsound | Delta |
|-------------|-------|---------|-------|
| Baseline | 2046 | 263 | - |
| A | ~2077 | ~232 | +31 E, -31 U |
| A+B | ~2094 | ~215 | +17 E, -17 U |
| A+B+C | ~2110 | ~199 | +16 E, -16 U |
| A+B+C+D | ~2123 | ~186 | +13 E, -13 U |
| A+B+C+D+E | ~2133 | ~176 | +10 E, -10 U |
| +F | ~2153 | ~156 | +20 E, -20 U |
| +G | ~2168 | ~141 | +15 E, -15 U |
| +H | ~2208 | ~101 | +40 E, -40 U |
| +I | ~2256 | ~53 | +48 E, -48 U |

**Conservative target: Phases A-F = ~2153 Exact, ~156 Unsound (107 improvement, -41% unsound)**
**Aggressive target: All phases = ~2256 Exact, ~53 Unsound (210 improvement, -80% unsound)**

---

## Regression Risk Assessment

| Phase | Regression Risk | Mitigation |
|-------|----------------|------------|
| A | None | Harness-only, additive |
| B | None | MTA module isolated |
| C | Low | Tighter matching, may cause some SAFE oracles to flip to Skip |
| D | Low-Medium | PTA constraint changes affect solver convergence; run full PTABen |
| E | Low | Targeted per-category changes |
| F | **Medium** | CHA changes affect ALL C++ test categories; must verify basic_cpp, failed_tests, cs_tests, complex_tests |
| G | **Medium** | Fixpoint changes affect ALL ae_* categories; convergence testing essential |
| H | **High** | Architectural fixpoint changes; risk regressions in ae_overflow, ae_assert, ae_recursion simultaneously |
| I | **High** | SVFG/solver changes affect mem_leak, double_free; may introduce FPs |

### Recommended Checkpoint Strategy

- Run full PTABen after each phase
- Accept phase if net Unsound decreases and no category regresses by more than 2 cases
- Phase H specifically: implement H1 first (dominator refinement), benchmark, then proceed with H2-H4
- Phase I: implement I1 first (PTA-augmented edges), benchmark separately from I2 (PartialLeak)
