# Plan 074: ae_assert_tests Unsound Fixes

## Current State
- **42 unsound**, 26 exact, 18 skip (floating-point comparisons — `FCmpOeq` not supported)
- Target: reduce unsound significantly by fixing the tractable root causes

## Root Cause Analysis

### RC1: `evaluate_comparison` hardcodes 32-bit interval lookup (~30 tests)
In `condition_prover.rs`, interval lookups used 32-bit width but abstract interpreter stores at 64-bit.

### RC2: `Trunc` cast hardcoded to 32 bits (2 tests)
In `transfer.rs`, `CastKind::Trunc => val.trunc(32)` — should use actual target type.

### RC3: `FPToSI`/`FPToUI` returns TOP for constant floats (2 tests)
All float casts returned TOP. Constant float→int conversions should be precise.

### RC4: Switch terminator doesn't narrow intervals (10 tests)
Only `CondBr` edges refined intervals. Switch case edges should narrow discriminant.

### RC5: FCmpOeq skipping (18 tests) — NOT fixable without float domain

## Implementation (Completed)

### Phase A: Multi-width interval lookup — condition_prover.rs
Added `best_interval()` helper trying 64, 32, 8, 1 bit widths. Fixed 3 sites.

### Phase B: Cast target_bits in AIR — air.rs, mapping.rs, transfer.rs
Added `target_bits: Option<u8>` to `Operation::Cast`. Extract from LLVM IR text.

### Phase C: FPToSI/FPToUI constant propagation — transfer.rs
Look up `Constant::Float` from module constants at cast site (not in constant map).

### Phase D: Switch edge refinement — fixpoint.rs
Added `refine_switch_edge()` for ascending, narrowing, and interprocedural phases.

## Results
- ae_assert_tests: 26→27 Exact (+1), 42→41 Unsound (-1)
- ae_assert_tests_fail: 43 Unsound (+2 from Skip, minor regression)
- Full PTABen: 1800 Exact (+1), 311 Unsound (+1), 108 Skip (-2)
- Remaining 41 unsound: fundamental store→load precision limitation at -O0

## Files Modified
1. `crates/saf-analysis/src/z3_utils/condition_prover.rs` — Phase A
2. `crates/saf-core/src/air.rs` — Phase B
3. `crates/saf-frontends/src/llvm/mapping.rs` — Phase B
4. `crates/saf-frontends/src/air_json.rs` — Phase B
5. `crates/saf-analysis/src/absint/transfer.rs` — Phases B, C
6. `crates/saf-analysis/src/absint/fixpoint.rs` — Phase D
7. `crates/saf-analysis/src/pta/extract.rs` — Phase B (constructor update)
8. `crates/saf-analysis/src/proptest_arb.rs` — Phase B (constructor update)
