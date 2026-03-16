# Plan 084 — ae_assert_tests Unsound Fixes (Round 3)

## Goal

Reduce ae_assert_tests unsound from **35 → ≤5** by addressing 5 root causes identified through source-code analysis of all failing tests.

## Current State

PTABen ae_assert_tests: **51 Exact, 35 Unsound, 23 Skip** (103 test files, 109 oracle checks).

After Plans 074, 078, 080, 081, 083 — the remaining 35 unsound cases cluster into 5 categories with distinct root causes.

## Root-Cause Analysis

### Category 1: Array/Struct Field Index Confusion (4 cases)
**Tests:** `BASIC_array_struct_0`, `BASIC_array_varIdx_1`, `BASIC_struct_array_0`, `cwe121_struct_alloc`

The test stores different values into distinct array-of-struct elements, then loads a specific element. SAF loads the **wrong element's value** (e.g., `a[1].b == 21` instead of `a[0].b == 11`).

**Root cause:** After mem2reg, array-of-struct element stores go through GEP chains with multiple indices. The `loc_memory` store writes to a `LocId` derived from PTA's field-indexed object decomposition, but the load resolves to a *different* `LocId` for the same logical element — because the GEP index path computation diverges between the store path and the load path. The inline analysis (which handles `svf_assert` arguments) resolves GEP targets via PTA, but PTA's field-indexed `LocId` mapping conflates elements when the outer array index is variable or when struct padding shifts field offsets.

### Category 2: Interprocedural Value Flow Through Pointer Stores (8 cases)
**Tests:** `BASIC_funcall_ref_0`, `BASIC_funcall_ref_1`, `BASIC_ptr_func_0`, `BASIC_ptr_func_4`, `BASIC_ptr_func_6`, `BASIC_arraycopy1`, `BASIC_arraycopy2`, `BASIC_arraycopy3`

A callee writes `*p = val` or the test reads from an array of pointers. SAF reports TOP for the value after the call returns.

**Root cause:** Two sub-issues:
- **Inline analysis size limit:** Functions like `swap()` (3 blocks, ~15 instructions) fit the limit (≤5 blocks, ≤40 instructions), but after mem2reg the block count may change. More critically, `BASIC_ptr_func_4/6` use indirect calls (`q(&y)` where `q = c`) which the inline analysis doesn't follow — it only handles `CallDirect`. After inline analysis succeeds for direct calls, `loc_memory` IS propagated back (Plan 078/080), but for indirect calls or when all args are TOP, the summary fallback returns TOP.
- **Global array initialization:** `BASIC_arraycopy1/2/3` initialize arrays with `{&a, &b}` or `{'A','B'}` at global scope or via aggregate constants. The absint doesn't track constant aggregate element values — it sees the GEP-indexed load but has no `loc_memory` entry for the array element because the initializer was never processed as a Store instruction.

### Category 3: Branch Condition Narrowing Lost in Fixpoint (8 cases)
**Tests:** `BASIC_br_nd_0`, `INTERVAL_test_2`, `INTERVAL_test_9`, `INTERVAL_test_12`, `INTERVAL_test_13` (×2), `INTERVAL_test_16`, `INTERVAL_test_49`

After `if (a > 5)`, the variable `a` should be `[6, MAX]` inside the branch. SAF reports `[-MIN, MAX]`.

**Root cause:** Branch refinement IS applied (Plan 078), but the refined value is lost during fixpoint iteration. The issue is that after mem2reg, the refined SSA ValueId flows through phi nodes at join points. When the fixpoint's ascending phase widens at loop headers or joins at merge points, the refined interval is joined with TOP from the other path (or from the initial state), producing TOP. The `propagate_refinement_to_loc_memory` mechanism was designed for `-O0` alloca patterns and works by scanning Load instructions — but after mem2reg there are no Loads for promoted locals. The refinement sets the SSA ValueId directly in state, but that refined state is then joined/widened away at the next block entry.

The core problem: branch refinement narrows a ValueId's interval, but the **block entry state join** at the successor overwrites it. The refinement must survive the join — which requires either (a) per-edge state tracking or (b) re-applying refinement after each join at blocks dominated by the branch.

### Category 4: Switch State Tracking (10 cases)
**Tests:** `BASIC_switch` through `BASIC_switch10`

All switch tests report both operands of the assertion comparison as full-range TOP.

**Root cause:** `refine_switch_edge()` (Plan 074) narrows only the **switch discriminant** to a singleton on each case edge. But the discriminant is typically a loaded local variable (after mem2reg: an SSA value), not the variables modified inside switch cases. The actual issue is the same as Category 3 — values modified in case arms are SSA values that flow through phi nodes at the switch exit, and the fixpoint join/widen at those phi nodes produces TOP. Additionally, for switch tests where `cond = 'a'` is known, the analysis should prune infeasible arms, but `refine_switch_edge()` only narrows the discriminant — it doesn't mark the state as unreachable for impossible cases (e.g., `cond` is `[97,97]` so only the `case 'a'` edge is feasible).

### Category 5: Loop Induction Variable Analysis (5 cases)
**Tests:** `LOOP_for01`, `LOOP_for_call`, `INTERVAL_test_20`, `INTERVAL_test_19`, `INTERVAL_test_49` (also in Cat 3)

Loop variables are TOP or bottom after the loop, failing post-loop assertions.

**Root cause:** The widening strategy jumps to type bounds too aggressively. For `for (i = 0; i < 5; i++)`, the threshold extractor (Plan 083) should capture `5` from the comparison, but after mem2reg the loop comparison's constant operand may not be in `module.constants` — it could be an inline operand in the IR. The fixpoint widens `i` from `[0,0]` → `[0,1]` → `[0,MAX]` because no threshold stops it at `5`. After loop exit, narrowing should recover `i = [5,5]` but the narrowing phase operates on joined states that are already at TOP.

For `INTERVAL_test_20` and `LOOP_for01/for_call`, the phi-based condition tracking reports "operand interval is bottom" — meaning the phi node's back-edge incoming value wasn't computed (the widened state at the loop header didn't propagate the phi incoming from the loop body).

## Plan

### Phase A: SSA-Aware Refinement Persistence (~60 LOC, 3 files)
**Target:** Categories 3 + 4 (18 cases)

**Problem:** Branch/switch refinements set ValueId intervals in the propagated state, but these are lost when the state is joined with the successor's existing entry state.

**Approach:** After computing the refined propagated state for a branch/switch edge, the fixpoint currently does:
```rust
let new_state = if loop_headers.contains(succ_id) {
    widen_state(&old_state, &propagated_state, thresholds)
} else {
    old_state.join(&propagated_state)
};
```
The join with `old_state` destroys the refinement.

**Fix:** Track **conditional refinements** separately. When a ValueId was refined by a branch condition:
1. After `refine_branch_condition()`, record which ValueIds were narrowed and their refined intervals in a `BTreeMap<(BlockId, ValueId), Interval>` (edge refinement map).
2. After the join/widen step at successor entry, **re-apply** refinements for all ValueIds where the refinement is tighter than the joined result AND the block is dominated by the branching block.
3. For this, maintain a `block_refinements: BTreeMap<BlockId, Vec<(ValueId, Interval)>>` that maps each block to refinements inherited from its dominating branch.
4. At each block's entry, after computing the joined state, intersect (meet) with any inherited refinements.

**Implementation:**
- `fixpoint.rs`: After `refine_branch_condition()`, compute the set of refined ValueIds by comparing pre/post state. Store in a per-block refinement map. After join/widen at successor, apply `state.set(id, state.get(id).meet(&refinement))` for each inherited refinement.
- **Dominator check:** Only apply refinement to blocks dominated by the branch source. Use a simple heuristic: if the branch has exactly one feasible successor (other path is bottom), all blocks reachable from that successor inherit the refinement.
- For switch: when the discriminant is a singleton (or narrow interval), mark non-matching case edges as unreachable (bottom state), which prunes infeasible paths and prevents join pollution.

**Expected impact:** Fix ~18 of the 35 unsound cases.

### Phase B: Constant Aggregate Element Tracking (~40 LOC, 1 file)
**Target:** Category 2, sub-issue "global array initialization" (3 cases: `arraycopy1/2/3`)

**Problem:** Global or local arrays initialized with constant aggregates (e.g., `int* source[2] = {&a, &b}` or `char source[2] = {'A','B'}`) have no Store instruction in the IR — the initialization is via a constant aggregate. When the absint encounters a GEP+Load from these arrays, `loc_memory` has no entry.

**Approach:** At the start of fixpoint analysis for a function, scan for Alloca instructions whose initializers (or associated Store instructions) reference `Constant::Aggregate` or `Constant::Array` in `module.constants`. For each element with a known scalar value, pre-populate `loc_memory` at the corresponding PTA `LocId`.

**Implementation:**
- `fixpoint.rs` or `transfer.rs`: New helper `seed_aggregate_constants()` that:
  1. Iterates function instructions looking for Store where the stored value is a constant aggregate (check `module.constants`).
  2. For each aggregate element that is an integer/float constant, resolve the element's `LocId` via PTA (using the GEP-indexed field path).
  3. Store the constant value as a singleton interval in the initial state's `loc_memory`.
- Also handle global variable initializers by scanning `module.globals` at analysis start.
- For pointer-valued elements (`{&a, &b}`), this doesn't directly help interval tracking, but for scalar elements (`{'A','B'}`, `{1,2}`), it gives the load a concrete interval.

**Expected impact:** Fix 3 cases (`arraycopy1/2/3`).

### Phase C: Interprocedural Loc-Memory Summary Propagation (~80 LOC, 2 files)
**Target:** Category 2, sub-issue "pointer stores across call boundaries" (5 cases: `funcall_ref_0/1`, `ptr_func_0/4/6`)

**Problem:** For functions that modify `*p` (where `p` is a parameter), the inline analysis handles this for small direct-call callees. But:
- `BASIC_ptr_func_4/6` use **indirect calls** (`q(&y)` where `q` is a function pointer) — inline analysis only handles `CallDirect`.
- `BASIC_ptr_func_0` uses `swap(&a, &b)` where after mem2reg the pointer arguments are SSA values; the inline analysis runs but the double-indirection (`*p = *q; *q = t`) may lose precision in single-pass mode.

**Approach:** Extend the interprocedural summary to include **memory side-effect summaries** alongside return-value summaries. For each function, track `param_store_effects: BTreeMap<usize, Interval>` — "parameter index i's pointee is set to interval V".

**Implementation:**
- `interprocedural.rs`: After solving a function's intraprocedural fixpoint, scan Return blocks' exit states for `loc_memory` entries that correspond to parameter pointer targets. For each param `p_i`, if `pts(p_i) = {loc}` (singleton) and `state.loc_memory[loc]` has a non-TOP interval, record the summary `(i, interval)`.
- `transfer.rs`: In `compute_call_return_with_summaries`, when inline analysis is skipped (too large, indirect call, all-TOP args), check if the callee's memory summary is available. If so, for each `(param_idx, interval)` in the summary, resolve the actual argument's PTA target and store the interval in caller's `loc_memory`.
- For **indirect calls**: resolve the callee via PTA's indirect call resolution (`callgraph.callees_of(call_site)`), then apply the same memory summary mechanism.

**Expected impact:** Fix 5 cases (`funcall_ref_0/1`, `ptr_func_0/4/6`).

### Phase D: Loop Widening Threshold Improvement (~30 LOC, 1 file)
**Target:** Category 5, loop-related (3 cases: `LOOP_for01`, `LOOP_for_call`, `INTERVAL_test_20`)

**Problem:** The threshold extractor misses some loop bound constants after mem2reg, causing widening to jump to type bounds.

**Approach:**
1. **Scan loop condition operands:** In addition to the current threshold extraction from module constants and phi incoming values, scan the actual loop header's terminator condition operands. For `for (i = 0; i < 5; i++)`, the loop header's `CondBr` condition is `ICmpSlt(i_phi, 5)` — extract `5` as a threshold.
2. **Loop-exit narrowing enhancement:** After the ascending phase widens to `[0, MAX]` and the loop exits (condition is false), the narrowing phase should use the loop condition to bound the variable. If the loop condition is `i < 5` and exit is on the false edge, then `i >= 5`. Combined with `i = [0, MAX]`, this gives `i = [5, MAX]`. Further narrowing with the actual loop increment can tighten this to `[5, 5]` for simple counted loops.
3. **Phi back-edge propagation:** Ensure the narrowing phase properly propagates the narrowed phi incoming value from the loop body back through the back-edge. The "operand interval is bottom" error suggests the back-edge state isn't computed.

**Implementation:**
- `threshold.rs`: Add extraction of ICmp operands from loop header terminators (block terminators that have back-edges).
- `fixpoint.rs`: In the narrowing phase, when processing a loop header phi, ensure the incoming from the back-edge predecessor carries the narrowed (not bottom) interval.
- `fixpoint.rs`: After the narrowing phase, for blocks that follow loop exits, apply the negated loop condition as a refinement (same mechanism as Phase A's refinement persistence).

**Expected impact:** Fix 3 cases.

### Phase E: GEP Field Path Resolution for Array-of-Struct (4 cases)
**Target:** Category 1 (`BASIC_array_struct_0`, `BASIC_array_varIdx_1`, `BASIC_struct_array_0`, `cwe121_struct_alloc`)

**Problem:** When a struct array element is accessed (e.g., `a[0].b`), the GEP produces a two-level index path: `[array_idx, field_idx]`. PTA decomposes the object into field-indexed `LocId`s, but the store and load paths may produce different `LocId`s for the same logical element.

**Approach:** This is the deferred Plan 080 Phase B (GEP field path depth matching). The previous attempt regressed because the depth+prefix check was too strict for sibling array element navigation.

**Implementation:**
- `transfer.rs` (in `transfer_load_with_pta`): When loading from a GEP target `LocId` that has no `loc_memory` entry, try a **sibling resolution** strategy: find all `LocId`s in `loc_memory` that share the same base ObjId, compute their field index, and compare with the load's expected field index. If there's exactly one match, use its value (strong read). If multiple match, join them (weak read).
- The field index can be extracted from the LocId's position in PTA's field-indexed decomposition, or by computing it from the GEP operands.

**Deferred:** If this regresses other tests (as Plan 080 Phase B did), scope down to only activating sibling resolution when the access is provably to an array-of-struct with constant index.

**Expected impact:** Fix 4 cases.

### Phase F: INTERVAL_test_19 Modulo Precision (~15 LOC, 1 file)
**Target:** `INTERVAL_test_19` (1 case)

**Problem:** `a % 2` produces interval `[-1, 1]` instead of `[0, 1]` (for non-negative `a`). The test does `if (i >= 0) { foo(&i); ... }` where `foo` uses `a = *i % 2` and a switch on `a`.

**Root cause:** The signed remainder implementation doesn't exploit the non-negativity of the dividend. `srem(a, 2)` where `a >= 0` should give `[0, 1]`, not `[-1, 1]`.

**Implementation:**
- `interval.rs` (in `srem` handler): If the dividend is non-negative (`lo >= 0`), the result of `srem` is also non-negative. Clamp result `lo` to `max(result.lo, 0)`.

**Expected impact:** Fix 1 case (enables the switch-case logic to prove the assertion).

## Phase Ordering & Dependencies

```
Phase F (independent, trivial)  ─┐
Phase B (independent)            ├─→ Run benchmarks
Phase D (independent)            │
Phase A (foundational)          ─┘
Phase C (depends on A for full benefit, but independently useful)
Phase E (independent, risk of regression — do last)
```

Recommended execution order: **A → F → B → D → C → E**, with PTABen validation after each phase.

## Risk Assessment

| Phase | Risk | Mitigation |
|-------|------|------------|
| A | Over-refinement (marking feasible branches as unreachable) | Only apply refinements from branches with one infeasible side; validate with full PTABen |
| B | Incorrect aggregate element indexing | Limit to simple scalar constants; validate GEP-to-LocId mapping against PTA |
| C | Memory summary too coarse (merges contexts) | Only apply when PTA target is singleton (strong update semantics) |
| D | Widening instability | New thresholds only added, never removed; existing convergence guarantees preserved |
| E | Regression in other test categories (Plan 080 Phase B history) | Gate behind array-of-struct access pattern; run full PTABen before/after |
| F | None (pure precision improvement in well-isolated function) | Trivial change |

## Expected Results

| Phase | Cases Fixed | Running Unsound |
|-------|-----------|-----------------|
| Baseline | — | 35 |
| A | ~18 (Cat 3 + Cat 4) | ~17 |
| F | ~1 (INTERVAL_test_19) | ~16 |
| B | ~3 (arraycopy1/2/3) | ~13 |
| D | ~3 (LOOP_for01, for_call, test_20) | ~10 |
| C | ~5 (funcall_ref_0/1, ptr_func_0/4/6) | ~5 |
| E | ~4 (array_struct_0, array_varIdx_1, struct_array_0, cwe121) | ~1 |

Target: **35 → ≤5 Unsound** (may have 1-2 stubborn cases from edge interactions).

## Success Criteria

- ae_assert_tests unsound ≤ 5
- No regressions in other PTABen categories (±0 unsound in ae_overflow, ae_recursion, basic_c, basic_cpp, path, cs, fs tests)
- All Rust tests pass
- `make lint` clean
