# Plan 080: Abstract Interpretation loc_memory Precision

## Problem

~78 PTABen tests across ae_assert_tests (32 unsound), ae_assert_tests_fail (46 unsound),
and ae_recursion_tests (16 unsound) fail because the abstract interpreter loses interval
precision when values flow through alloca store/load patterns at -O0.

At -O0 (no optimizations), clang generates alloca/store/load for every variable:
```llvm
%x = alloca i32
store i32 5, ptr %x       ; loc_memory[alloca_x] = [5,5]
%val = load i32, ptr %x   ; should get [5,5]
```

The loc_memory tracking exists but breaks down in several specific scenarios.

## Root Cause Analysis

Investigation identified **5 distinct root causes** across the 78+ unsound tests:

### RC1: Inline Analysis Results Wiped by Invalidation (~10 tests)

**Pattern**: `foo(&a)` where `foo` is a small function that modifies `*p`.

**Code path** (transfer.rs):
1. Line 369: `compute_call_return_with_summaries(inst, callee, state, ...)` runs
2. Inside that function (line 891-929): inline analysis succeeds, callee's loc_memory
   is propagated back to caller's state via `state.store_loc(loc, interval)`
3. Function returns the return interval value
4. Back in the CallDirect handler, lines 433-481: memory invalidation runs
5. Line 459: `state.invalidate_locs(&reachable_locs)` **wipes the same locations**
   that inline analysis just propagated

**The bug**: `compute_call_return_with_summaries` mutates `state` (propagating
callee effects) but then the caller's invalidation code unconditionally wipes
argument-reachable locations. The `return result` on line 938 exits the inner
function, NOT the CallDirect handler.

**Affected tests**: BASIC_funcall_ref_0, BASIC_funcall_ref_1, BASIC_ptr_func_0/1/4/6

**Fix**: Skip loc_memory invalidation for argument-reachable locations when inline
analysis succeeded and propagated results.

### RC2: GEP Field Path Computation Errors (~6 tests)

**Pattern**: Array-of-struct or struct-with-array access yields **wrong concrete
values** (not TOP).

Examples:
- `a[0].b == 11` gets `[21,21]` (wrong element)
- `a[1] == 1` gets `[2,2]` (off-by-one)
- `a.b[1] == 5` gets `[1,1]` (wrong field entirely)

**Root cause**: Two issues in GEP target resolution:

A. `find_location_with_index()` in pta_integration.rs matches only the LAST step
   of a location's path. For nested paths like `[Index(0), Field(1), Index(1)]`,
   searching for index 1 can match `[Index(1)]` instead — a completely different
   location at the wrong nesting level.

B. `merge_gep_with_base_path()` in solver.rs adds child index to parent's last
   step, which is correct for pointer arithmetic but wrong for nested array/struct
   indexing where the GEP adds a new nesting level.

**Affected tests**: BASIC_array_struct_0, BASIC_array_varIdx_1, BASIC_struct_array_0,
cwe121_struct_alloc, BASIC_arraycopy2, BASIC_arraycopy3

### RC3: Branch Refinement Lost Through Alloca Indirection (~12 tests)

**Pattern**: `if (a > 5) { svf_assert(a > 5); }` — branch refines the loaded SSA
value but the value is re-loaded from alloca inside the true block, getting the
un-refined interval from loc_memory.

`propagate_refinement_to_loc_memory` was added to fix this, but it only looks at
Load instructions in the **current block** (the block containing the branch). If the
assertion is in a **successor block**, the refinement doesn't propagate.

Additionally, during fixpoint iteration at loop headers, widening can widen the
refined loc_memory entry back to TOP, losing the refinement from the previous
iteration.

**Affected tests**: INTERVAL_test_2/9/12/13/16/19/20/49, BASIC_bi_div_0,
BASIC_bi_mix_0, BASIC_br_nd_1, LOOP_for01

### RC4: Float-to-Int Cast and ZExt Bugs (~3 tests)

**Pattern**: `int si = (int)(-3.14)` yields full i32 range instead of `[-3,-3]`.
`unsigned char c = 255; int x = c + 1` yields `[1,1]` instead of `[256,256]`.

**Root cause**: FPToSI/FPToUI constant propagation only checks `Constant::Float`
in the module's constant map, but doesn't handle float constants embedded in
instructions. The ZExt issue is a truncation bug where the source value isn't
properly zero-extended before arithmetic.

**Affected tests**: CAST_fptosi, CAST_fptoui, CAST_zext

### RC5: External Function Memory Effects (~2 tests)

**Pattern**: `memset(buf, 'A', 3); assert(buf[0] == 'A')` — memset transfer function
stores the interval but subsequent GEP-indexed load can't find the right location.

**Root cause**: memset/memcpy transfer functions store to PTA-resolved locations,
but the load uses a GEP result that resolves to a different LocId (element-level vs
base-level).

**Affected tests**: CWE127_har_alloc, cwe126_char_alloc

## Phases

### Phase A: Fix Inline Analysis Invalidation (~15 LOC)

**Goal**: Prevent loc_memory invalidation from wiping inline analysis results.
**Expected improvement**: +6-10 Exact in ae_assert_tests.

In `transfer.rs`, the `CallDirect` handler:

1. Add a boolean flag to `compute_call_return_with_summaries` signature:
   `inline_succeeded: &mut bool`

2. Set `*inline_succeeded = true` when inline analysis runs and propagates
   loc_memory (around line 925-929).

3. In the CallDirect invalidation section (lines 444-460), skip
   `state.invalidate_locs(&reachable_locs)` when `inline_succeeded` is true.
   The inline analysis already computed the precise callee effects — invalidation
   would destroy that precision.

**Alternative (simpler)**: Have `compute_call_return_with_summaries` return
`(Interval, bool)` where the bool indicates inline analysis success. Use that
to guard the invalidation.

### Phase B: Fix GEP Field Path Resolution (~40 LOC)

**Goal**: Fix wrong concrete values from array/struct GEP indexing.
**Expected improvement**: +4-6 Exact in ae_assert_tests.

B1. In `pta_integration.rs::find_location_with_index()`:
   - Add path depth matching: the candidate location's path length should match
     the expected depth (base location's path length + 1 for one-level indexing)
   - When multiple candidates match at the last step, prefer the one whose path
     is a prefix extension of the base location's path

B2. In `solver.rs::merge_gep_with_base_path()`:
   - Add a guard: only merge (add indices) when the GEP is a single-step
     pointer-arithmetic offset (both base and GEP have the same path prefix)
   - For nested struct/array access, use path extension instead of index addition

### Phase C: Improve Branch Refinement Propagation (~30 LOC)

**Goal**: Make branch-refined intervals survive into successor blocks' loc_memory.
**Expected improvement**: +8-12 Exact across ae_assert + ae_assert_fail.

C1. After `refine_branch_condition()` in fixpoint.rs (ascending phase, line ~330):
   - Call `propagate_refinement_to_loc_memory()` using the **successor block**
     (not the current block) to also catch loads in the successor
   - Alternatively: scan ALL Load instructions in the current function that load
     from the same alloca, and update loc_memory for those locations

C2. In the narrowing phase: same treatment — propagate branch refinement to
   loc_memory after narrowing refinement.

C3. During fixpoint joining at loop headers: when widening would push a refined
   loc_memory entry to TOP, check if the entry was refined by a dominating branch
   condition. If so, cap the widening at the branch-implied bound instead of TOP.

### Phase D: Fix Cast Operations (~15 LOC)

**Goal**: Correct FPToSI/FPToUI/ZExt transfer functions.
**Expected improvement**: +3 Exact in ae_assert_tests.

D1. In `transfer.rs` FPToSI/FPToUI handler:
   - Check if the source operand's defining instruction is a float constant
     (scan module instructions, not just the constant map)
   - Convert float constant to integer singleton directly

D2. In `transfer.rs` ZExt handler:
   - When source is a singleton from a smaller bit width, zero-extend properly:
     compute `value & ((1 << source_bits) - 1)` before storing

### Phase E: Memset/Memcpy Element-Level Store (~20 LOC)

**Goal**: Make memset/memcpy intervals accessible through GEP-indexed loads.
**Expected improvement**: +2 Exact in ae_assert_tests.

In `transfer.rs` memset handler:
   - After storing the interval to PTA base locations, also store to all
     element-level LocIds that share the same base object
   - Use `pta.locations_of_object(obj)` to find all sub-locations

## Expected Results

| Phase | ae_assert Δ | ae_assert_fail Δ | ae_recursion Δ | Total Exact Δ |
|-------|-------------|-------------------|----------------|---------------|
| A     | +6-10       | +6-10             | +2-4           | +14-24        |
| B     | +4-6        | +2-4              | 0              | +6-10         |
| C     | +8-12       | +10-15            | +2-4           | +20-31        |
| D     | +3          | +0-1              | 0              | +3-4          |
| E     | +2          | +0-1              | 0              | +2-3          |

**Conservative total: +45-72 Exact, corresponding Unsound reduction.**

Phase A has the highest ROI (simplest fix, big impact). Phase C has the highest
absolute improvement but is the most complex.

## Risks

- Phase C (loop header refinement) could introduce unsoundness if the branch
  condition doesn't actually dominate the loop header. Must verify dominance
  before applying the cap.
- Phase B changes could affect PTA precision for other categories (basic_c_tests,
  path_tests). Run full benchmark to check for regressions.
- Phase A should be safe since inline analysis already computes exact callee effects.

## Verification

After each phase:
1. Run `cargo nextest run --workspace --exclude saf-python` (Rust tests)
2. Run filtered PTABen:
   ```bash
   cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_assert_tests/*" -o results-ae-assert.json
   cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_assert_tests_fail/*" -o results-ae-assert-fail.json
   ```
3. Run full PTABen to check for regressions:
   ```bash
   cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o results-full.json
   ```
