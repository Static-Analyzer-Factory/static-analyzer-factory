# Plan 083: SSA-Native Abstract Interpretation

## Status: Done (partial)

## Goal

Recover the ~45 Exact / ~35 Unsound regressions caused by Plan 081 (global mem2reg enablement). The root cause is that the absint subsystem was designed around `-O0` alloca store→load chains for interval tracking. After mem2reg, promoted locals become SSA phi/copy values with no corresponding Load/Store instructions, causing `loc_memory` to be empty and intervals to degrade to TOP.

**Current baseline (post-Plan 081+082):** 2001 Exact, 304 Unsound
**Pre-mem2reg baseline:** 2046 Exact, 269 Unsound
**Target:** Recover 30+ Exact, reduce Unsound by 25+

## Root Cause Analysis

After mem2reg, a variable like:
```c
int x = 5;
if (cond) x = 10;
assert(x > 3);
```
changes from:
```llvm
; -O0 (alloca pattern)
%x = alloca i32
store i32 5, %x          → loc_memory[x_loc] = [5,5]
...
store i32 10, %x          → loc_memory[x_loc] = [10,10]
%val = load i32, %x        → resolves via loc_memory → [10,10]
```
to:
```llvm
; SSA (after mem2reg)
%x.0 = phi i32 [5, %entry], [10, %if.then]  → values[x.0] = join([5,5],[10,10]) = [5,10]
; No Store/Load → loc_memory is empty
```

The phi handler correctly joins values into `state.values[x.0]`. The checker's `interval_at_inst()` queries `state.get_opt(value)` which reads from `values` — **this works for SSA**. The problem is NOT that SSA values are invisible, but rather:

1. **Branch refinement is lost in fixpoint widening** (12 ae_assert cases): At a loop header, `widen_state()` blows up the refined interval, and `narrow_state()` doesn't fully recover it.
2. **Pass-by-pointer values become TOP** (6 ae_assert cases): Inline analysis propagates caller's `loc_memory` to callee, but it's empty post-mem2reg.
3. **Recursive parameter binding is weaker** (13 ae_recursion cases): SCC fixpoint joins phi values from different iteration depths too aggressively.

## Investigation Findings

### What Works (No Changes Needed)
- **PTA constraint extraction**: Phi → Copy constraints are correct
- **Value-flow builder**: DefUse edges for phi incoming values work
- **MSSA/SVFG**: Sparse skeleton handles zero-memory functions
- **FS-PTA**: Edge-based phi filtering prevents unreachable contamination
- **Checker queries**: `interval_at_inst()` reads from `values` map directly → SSA-compatible
- **Nullness analysis**: Idempotent lattice; no alloca-specific code
- **Phi reachability**: `reached_blocks` filtering correctly skips unreachable predecessors

### What Needs Fixing

| Issue | Cases | Root Cause |
|-------|-------|-----------|
| Branch refinement lost in fixpoint | 12 ae_assert | Widening at loop headers destroys refined intervals |
| Pass-by-pointer TOP | 6 ae_assert | Inline analysis can't propagate SSA values through pointer args |
| Recursive parameter binding | 13 ae_recursion | SCC fixpoint merges phi values too aggressively |
| Overflow size tracking | ~4 ae_overflow | GEP field→base allocation size backtracking incomplete |
| Float→int SSA | 3 ae_assert | `find_float_through_alloca` removed; direct constant lookup doesn't find SSA-propagated floats |

---

## Phase A: Branch Refinement Preservation Through Widening

**Problem:** At a loop header block, the ascending phase calls `widen_state()` which merges the branch-refined state from the loop body with the entry state. The widened result often loses the refinement. The narrowing phase narrows against the joined propagated state, but narrowing only tightens — it can't recover refinements that were lost during widening.

**Example:**
```c
int x = input();          // x ∈ TOP
if (x > 0 && x < 100) {  // branch refines x to [1, 99]
    svf_assert(x > 0);    // needs [1, 99], but fixpoint may have widened to TOP
}
```

The issue: when there's a loop containing the conditional, the loop header phi joins the refined [1,99] with the re-entry edge, and widening blows it to TOP.

**Fix:** Add **threshold extraction from branch conditions**. The existing `extract_thresholds()` function extracts constants from comparison instructions. Ensure branch condition constants are included as widening thresholds so that `widen_with_thresholds()` stops at the branch boundary instead of jumping to ±∞.

**File:** `crates/saf-analysis/src/absint/threshold.rs`

**Change (~20 LOC):** In `extract_thresholds()`, also extract constants from phi incoming values that are integer constants. After mem2reg, phi nodes directly reference constant values (e.g., `phi [5, %entry], [%x.inc, %body]`). The constant `5` should become a widening threshold.

**Risk:** Low. Adds thresholds, doesn't change algorithm.

---

## Phase B: SSA-Aware Inline Analysis Parameter Propagation

**Problem:** `analyze_callee_inline()` propagates the caller's `loc_memory` to the callee for pointer arguments. After mem2reg, the caller's `loc_memory` is empty for promoted locals. When a callee receives a pointer to the caller's variable, it stores through the pointer, but the result never reaches the caller.

**However:** If the variable's address is taken (`&x`), the alloca survives mem2reg. So this issue only affects cases where the address was taken in a way that mem2reg couldn't promote. These address-taken allocas DO have `loc_memory` entries.

**Re-analysis:** The "pass-by-pointer TOP" cases may actually be about the **callee's** SSA values not flowing back, not the caller's. When the callee has a promoted local that it returns or stores through a pointer, the inline analysis runs a single-pass forward analysis of the callee, which doesn't iterate to fixpoint for loops.

**Fix:** Improve `analyze_callee_inline()` to run a mini-fixpoint for callees that contain loops (detected by back edges).

**File:** `crates/saf-analysis/src/absint/transfer.rs`

**Change (~40 LOC):** In `analyze_callee_inline()`, after the parameter binding:
1. Check if any block in the callee has a back-edge (simple predecessor-in-stack check)
2. If yes, iterate the forward pass up to 3 times until return intervals stabilize
3. If no loops, keep the current single-pass behavior

**Risk:** Low. Adds iteration for loopy callees, keeps current behavior for simple ones.

---

## Phase C: SSA Phi-Constant Propagation in Condition Prover

**Problem:** `best_interval()` in `condition_prover.rs` queries `interval_at_inst()` which checks `state.get_opt(value)`. For SSA phi values that are constants (e.g., the `5` in `phi [5, %entry], [10, %then]`), the interval is correctly computed as `[5, 10]` by the phi handler.

But for **operands of the comparison**, if the operand is a constant ValueId that's only in `module.constants` (not in the abstract state), the fallback chain is:
1. `state.get_opt(value)` → None (constant not in state)
2. `constant_map.get(value)` → Some([5,5]) (constants ARE in the constant map)
3. Returns [5,5]

This should work. Let me verify the actual failing pattern instead.

**Actual issue:** After mem2reg, some float constants that were stored to alloca then loaded before `fptosi` are now SSA phi values. Plan 082 removed `find_float_through_alloca()`. But the direct constant lookup (`module.constants.get(&operand)`) should find float constants that are direct operands. The issue is when a float flows through a phi:
```llvm
%f.phi = phi double [-3.14, %entry], [undef, %dead]
%i = fptosi double %f.phi to i32
```
Here `%f.phi` is NOT in `module.constants` (it's a phi result), and the operand's interval is TOP because floats aren't tracked in the interval domain.

**Fix:** In the `FPToSI`/`FPToUI` handler, trace through SSA phi/copy chains to find the underlying float constant.

**File:** `crates/saf-analysis/src/absint/transfer.rs`

**Change (~15 LOC):** Add `find_float_constant_ssa()` helper that follows the SSA def-use chain (phi incoming values, copy sources, cast sources) up to 5 hops to find a `Constant::Float` in `module.constants`. Replace the removed `find_float_through_alloca()` with this SSA-aware version.

**Risk:** Low. Targeted constant propagation through SSA chains.

---

## Phase D: Recursive SCC Parameter Precision

**Problem:** `compute_recursive_scc_summaries_with_pta()` in `interprocedural.rs` initializes SCC member summaries with bottom return intervals, then iterates. The iteration seeds parameters from external callers via `collect_external_call_args()`. After mem2reg, the caller's argument intervals may be wider because the caller's fixpoint also had widening issues.

**Specific pattern:** Recursive functions where the base case tests a parameter (e.g., `if (n == 0) return 1; else return n * f(n-1)`). After mem2reg, `n` is a phi value that gets widened at the loop header. The recursive SCC solver sees TOP for `n`, making the return interval TOP.

**Fix:** In the SCC parameter seeding, when the caller's argument is a direct constant or narrow interval, bypass the widened phi and use the concrete value.

**File:** `crates/saf-analysis/src/absint/interprocedural.rs`

**Change (~25 LOC):** In `collect_external_call_args()`, when extracting argument intervals, also check `module.constants` for constant arguments. If the argument ValueId is a constant, use the constant value directly instead of looking up the (potentially widened) abstract state.

**Risk:** Low. More precise seeding, no behavioral change for non-constant arguments.

---

## Phase E: Narrowing Phase Enhancement

**Problem:** The narrowing phase (`narrow_state()`) iterates in reverse postorder, applying `old.narrow(new)` to each value entry. This correctly tightens over-widened intervals. However:

1. The narrowing phase uses the same `propagate_refinement_to_loc_memory()` which is a no-op for promoted locals (no Load instructions).
2. Branch refinements are applied during narrowing (lines 468-494 in fixpoint.rs), but at merge points the join operation `old.join(&propagated_state)` before narrowing widens again.

**Fix:** In the narrowing phase, when propagating a branch-refined state to a successor, use `old.narrow(&propagated_state)` directly instead of `old.join(&propagated_state)` followed by `narrow_state()`.

**File:** `crates/saf-analysis/src/absint/fixpoint.rs`

**Change (~10 LOC):** At narrowing line 515-516:
```rust
// Current:
let joined = old_state.join(&propagated_state);
let narrowed = narrow_state(&old_state, &joined, thresholds);
// Proposed:
let narrowed = narrow_state(&old_state, &propagated_state, thresholds);
```
This avoids the intermediate join that re-widens the state before narrowing.

**Risk:** Medium. This changes narrowing semantics. Need to verify that `narrow(old, propagated)` is sound — specifically that `propagated ≤ old` is guaranteed at this point (it should be, since narrowing only runs after ascending phase stabilized).

---

## Phase F: Remove Dead `propagate_refinement_to_loc_memory` for SSA Blocks

**Problem:** `propagate_refinement_to_loc_memory()` is called twice per branch edge (once for current block, once for successor block). After Plan 082 Phase C, it already returns early when no Load instructions exist. But the two calls themselves add overhead — scanning for Load instructions in both blocks on every branch edge, every fixpoint iteration.

**Fix:** Hoist the "has any Load" check to the caller so it's not done twice per block per iteration.

**File:** `crates/saf-analysis/src/absint/fixpoint.rs`

**Change (~10 LOC):** Cache which blocks have Load instructions at the start of the fixpoint. Only call `propagate_refinement_to_loc_memory()` for blocks that actually have Loads.

**Risk:** None. Pure optimization.

---

## Verification

1. `make test` — all 1,373 Rust tests and 72 Python tests pass
2. `make lint` — clean
3. PTABen: Run full suite, target Exact ≥ 2030 (recovering ~30 from 2001), Unsound ≤ 280 (reducing ~24 from 304)
4. Spot-check regressed categories: `ae_assert_tests`, `ae_recursion_tests`, `ae_overflow_tests`
5. Verify no regressions in recovered categories (path_tests, basic_cpp_tests)

## Files Modified

| File | Phase | Changes |
|------|-------|---------|
| `crates/saf-analysis/src/absint/threshold.rs` | A | Extract phi-constant thresholds |
| `crates/saf-analysis/src/absint/transfer.rs` | B, C | Mini-fixpoint for loopy callees; SSA float constant propagation |
| `crates/saf-analysis/src/absint/interprocedural.rs` | D | Constant-aware SCC parameter seeding |
| `crates/saf-analysis/src/absint/fixpoint.rs` | E, F | Narrowing enhancement; cached Load-block check |

## Estimated: ~120 LOC added/modified across 4 files
