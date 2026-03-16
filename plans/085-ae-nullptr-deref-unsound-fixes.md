# Plan 085: ae_nullptr_deref_tests Unsound Fixes

## Status: Done

## Context

ae_nullptr_deref_tests had **35 unsound** cases. Root cause: void function calls (including `free()`) were completely skipped in the nullness transfer function because the LLVM frontend assigns dummy `dst` values to ALL calls (even void ones), so the `inst.dst.is_none()` guard never triggered for `CallDirect`.

**Baseline**: ae_nullptr_deref_tests: 115 Exact, 35 Unsound, 0 Skip.
**Result**: ae_nullptr_deref_tests: 147 Exact, 3 Unsound, 0 Skip.
**Overall**: 2046 Exact (+32), 263 Unsound (−32). Zero regressions.

## Phase A: Deallocation Effect in `transfer_call_direct` (~40 LOC)

**File**: `crates/saf-analysis/src/absint/nullness.rs`

Added `apply_deallocation_effect()` function called after standard call handling in `transfer_call_direct` when the callee is a deallocator. Marks the first argument SSA value as `MaybeNull`, stores `MaybeNull` in ValueId-based memory, and propagates to PTA loc_memory.

Also added void call handler block (for `CallDirect` and `CallIndirect`) before the `dst` guard for correctness on any future void calls that truly have `dst = None`.

**Fixed 32 cases**: All ExtAPI tests (memcpy/memset/strcpy/strcat + free → UNSAFE_LOAD) and dangleptr tests (free → UNSAFE_LOAD).

## Phase B: Interprocedural Parameter Nullness Seeding (~35 LOC)

**File**: `crates/saf-analysis/src/absint/nullness.rs`

Two-pass approach in `analyze_nullness_with_context()`:
1. First pass: analyze all functions with default `MaybeNull` parameters
2. Collect caller argument nullness via `collect_caller_param_nullness()`
3. Second pass: re-analyze functions whose parameter nullness tightened to `NotNull`

No visible impact on current benchmarks — the 3 remaining unsound are caused by other issues.

## Phase C: Deallocation Function Detection (~15 LOC)

**File**: `crates/saf-analysis/src/absint/function_properties.rs`

Added `is_known_deallocation_function()` (hardcoded: free, _ZdlPv, _ZdaPv, etc.) and `is_deallocation_with_specs()` (checks `Role::Deallocator` in spec first).

## Remaining 3 Unsound

- `dangleptr_safe_free_and_reassign`: free→reassign in SSA (phi precision)
- `dangleptr_safe_branch`: free→branch reassign (path-sensitivity)
- `safe_ptr_array_access`: array element tracking

## Files Modified

| File | Change |
|------|--------|
| `crates/saf-analysis/src/absint/nullness.rs` | Phase A + Phase B |
| `crates/saf-analysis/src/absint/function_properties.rs` | Phase C |
