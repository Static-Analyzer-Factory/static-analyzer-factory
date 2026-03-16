# Plan 082: Alloca Workaround Cleanup (Post-mem2reg)

## Status: Done

## Goal

Remove dead alloca-specific workaround code that is no longer needed after global mem2reg enablement (Plan 081). Simplify dual-path code (alloca + SSA) to SSA-only where the alloca path is dead.

**Important constraint:** Address-taken locals survive mem2reg. Code that handles memory operations for address-taken allocas (structs passed by pointer, arrays, globals) must be preserved. Only code that traces **promoted** locals through store→load→alloca chains is dead.

## Changes

### Phase A: Remove `build_param_indices()` alloca tracing
**File:** `crates/saf-analysis/src/pta/value_origin.rs`

Removed Step 2 (~24 LOC) from `build_param_indices()` that traced `-O0` alloca store→load chains for parameter mapping. After mem2reg, Step 1 (direct params) and Step 2/renamed (SSA phi/copy/cast propagation) cover all cases. Updated doc comment.

### Phase B: Remove `find_float_through_alloca()`
**File:** `crates/saf-analysis/src/absint/transfer.rs`

Removed the `find_float_through_alloca()` function (~40 LOC) and its call site in the FPToSI/FPToUI handler. After mem2reg, float constants are direct operands — the existing `module.constants` lookup handles this.

### Phase C: Simplify `propagate_refinement_to_loc_memory()`
**File:** `crates/saf-analysis/src/absint/transfer.rs`

Added early-return optimization when block has no Load instructions (~5 LOC). Added clarifying comment noting it's primarily active for address-taken locals post-mem2reg.

### Phase D: Simplify `set_value()` alloca tracing
**File:** `crates/saf-analysis/src/absint/transfer.rs`

Added SSA-direct path: if PTA knows what `val_id` points to, store the constraint directly to those locations without Load tracing (~8 LOC). Kept Load tracing as fallback for address-taken locals.

### Phase E: Remove alloca-size operand fallbacks in checker
**File:** `crates/saf-analysis/src/absint/checker.rs`

Removed operand-based alloca size resolution fallbacks in `extract_allocation_sizes()` and `build_loc_allocation_sizes()` (~12 LOC removed). The `size_bytes` field in AIR (set by the frontend) handles address-taken allocas.

### Phase F: Add legacy comments to `memory` field
**File:** `crates/saf-analysis/src/absint/state.rs`

Added legacy comment to the `memory` field explaining it's the pre-PTA memory model, retained for non-PTA checker entry points. Full removal deferred.

## Verification

- `make lint` — clean
- `make test` — 1,373 Rust tests pass (5 skipped), 72 Python tests pass
- PTABen: Exact=2001, Unsound=304 (identical to post-Plan-081 baseline)

## Files Modified

| File | Changes |
|------|---------|
| `crates/saf-analysis/src/pta/value_origin.rs` | Remove alloca tracing from `build_param_indices()` |
| `crates/saf-analysis/src/absint/transfer.rs` | Remove `find_float_through_alloca()`, simplify `propagate_refinement_to_loc_memory()`, simplify `set_value()` |
| `crates/saf-analysis/src/absint/checker.rs` | Remove alloca-size operand fallbacks |
| `crates/saf-analysis/src/absint/state.rs` | Add legacy comments to `memory` field |

## Net: ~80 LOC removed, ~15 LOC added
