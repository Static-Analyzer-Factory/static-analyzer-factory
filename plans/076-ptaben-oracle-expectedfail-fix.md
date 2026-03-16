# Plan 076: Fix PTABen Oracle Verification — Expected-Imprecision Oracles

## Status: Done

## Problem

Audit of all PTABen expected-imprecision oracle types found three bugs where validation unconditionally returns `Sound` without querying actual analysis results. The correct pattern is to run analysis and return `Exact` when SAF beats SVF, `Sound` when SAF has the same imprecision.

### Full Audit Results

| Oracle Type | Variant | Extracted? | Validation | Bug? |
|---|---|---|---|---|
| Alias `EXPECTEDFAIL_MAYALIAS` | `ExpectedFailMayAlias` | Yes | Was early return → Fixed | **BUG** |
| Alias `EXPECTEDFAIL_NOALIAS` | `ExpectedFailNoAlias` | Yes | Was early return → Fixed | **BUG** |
| MemLeak `NFRLEAKFP` | `NeverFreeFP` | Yes | Checks `leak_finding_count == 0` | Correct |
| MemLeak `PLKLEAKFP` | `PartialLeakFP` | Yes | Checks `leak_finding_count == 0` | Correct |
| MemLeak `LEAKFN` | `FalseNegative` | Yes | Was unconditional `Sound` → Fixed | **BUG** |
| DoubleFree `SAFEMALLOCFP` | `FalsePositive` | Yes | Checks `df_finding_count == 0` | Correct |
| DoubleFree `DOUBLEFREEMALLOCFN` | `FalseNegative` | Yes | Was unconditional `Sound` → Fixed | **BUG** |
| NullCheck | `FalsePositive` | No (dead code) | Checks nullness properly | Correct (unreachable) |
| NullCheck | `FalseNegative` | No (dead code) | Checks nullness properly | Correct (unreachable) |

### Bug Pattern

All three bugs followed the same pattern — `FalseNegative` / `ExpectedFail` variants:
```rust
SomeKind::FalseNegative => {
    Outcome::Sound  // ← bypasses analysis entirely
}
```

Correct logic:
```rust
SomeKind::FalseNegative => {
    if finding_count > 0 {
        Outcome::Exact  // SAF found the bug SVF missed
    } else {
        Outcome::Sound  // Same imprecision as SVF
    }
}
```

## Changes

### Phase A: Fix alias EXPECTEDFAIL early return (`ptaben.rs`)

Removed the early return in `validate_alias()` that unconditionally returned `Sound` for `ExpectedFailMayAlias`/`ExpectedFailNoAlias`. The existing `check_alias_result()` already had correct dead-code handling.

### Phase B: Fix memleak FalseNegative (`ptaben.rs`)

Changed `MemLeakKind::FalseNegative` from unconditional `Sound` to checking `leak_finding_count > 0` → `Exact` (SAF beat SVF) vs `Sound` (same imprecision).

### Phase C: Fix double-free FalseNegative (`ptaben.rs`)

Changed `DoubleFreeKind::FalseNegative` from unconditional `Sound` to checking `df_finding_count > 0` → `Exact` (SAF beat SVF) vs `Sound` (same imprecision).

### Phase D: Report expected-imprecision wins (`lib.rs`, `main.rs`, `report.rs`)

1. `lib.rs`: Added `ExpectedFailDetail` struct with `file`, `expectation`, `result` fields. Added `expectedfail_total`, `expectedfail_exact`, and `expectedfail_details` to `SuiteSummary`.
2. `main.rs`: Count all expected-imprecision oracles in `build_summary()` — alias `ExpectedFail*`, memleak `NeverFreeFP`/`PartialLeakFP`/`FalseNegative`, double-free `FalsePositive`/`FalseNegative`. Collect `ExpectedFailDetail` for each win with `describe_expectedfail_win()` helper.
3. `report.rs`: Print "SAF vs SVF: X of Y expected-imprecision oracle(s) correctly resolved" summary line followed by per-oracle detail lines.

## Files Modified

| File | Change |
|------|--------|
| `crates/saf-bench/src/ptaben.rs` | Fixed 3 early-return bugs (alias, memleak FN, double-free FN) |
| `crates/saf-bench/src/lib.rs` | Added `ExpectedFailDetail`, `expectedfail_*` fields to `SuiteSummary` |
| `crates/saf-bench/src/main.rs` | Count + collect expected-imprecision outcomes with `describe_expectedfail_win()` |
| `crates/saf-bench/src/report.rs` | Print expected-imprecision summary with per-oracle details |

## Verification

- `make lint` — clippy + rustfmt clean
- `make test` — all 1,372 Rust tests pass, 72 Python tests pass

## Sample Output

```
SAF vs SVF: 4 of 5 expected-imprecision oracle(s) correctly resolved
  basic_c_tests/field-ptr-arith-constIdx.bc - Proved NoAlias (SVF reports MayAlias)
  basic_c_tests/struct-incompab-typecast.bc - Proved NoAlias (SVF reports MayAlias)
  basic_c_tests/struct-incompab-typecast.bc - Proved NoAlias (SVF reports MayAlias)
  basic_c_tests/struct-instance-return.bc - Proved NoAlias (SVF reports MayAlias)

SAF vs SVF: 8 of 11 expected-imprecision oracle(s) correctly resolved
  mem_leak/malloc26.bc - No false positive (SVF incorrectly reports leak)
  mem_leak/malloc33.bc - Found leak (SVF misses it)
  mem_leak/malloc61.bc - No false positive (SVF incorrectly reports leak)
  ...
```
