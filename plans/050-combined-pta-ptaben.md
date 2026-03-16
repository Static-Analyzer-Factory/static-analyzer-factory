# Plan 050: Combined CS-PTA + FS-PTA for PTABen

**Epic:** E30 - PTABen Precision Improvement
**Status:** Done
**Created:** 2026-02-01

## Problem

PTABen currently uses context-insensitive PTA (CI-PTA), resulting in 132 failures:
- cs_tests: 9 pass, 18 fail (context sensitivity needed)
- fs_tests: 6 pass, 18 fail (flow sensitivity needed)
- path_tests: 1 pass, 20 fail (path sensitivity needed)

SAF already has CS-PTA (k-CFA) and FS-PTA implementations but PTABen doesn't use them.

## Solution

Run both CS-PTA and FS-PTA, combine results for maximum precision.

### Architecture

```
Module → CallGraph → CS-PTA(k=2) → CsPtaResult
                  ↘
                    FS-PTA → FsPtaResult
                              ↓
                    CombinedPtaResult
                              ↓
                    Best alias result per query
```

### Precision Combination Rule

For each alias query, take the most definite answer:

| CS-PTA | FS-PTA | Combined |
|--------|--------|----------|
| No | * | No |
| * | No | No |
| Must | * | Must |
| * | Must | Must |
| Unknown | X | X |
| X | Unknown | X |
| Partial | Partial | Partial |
| Partial | May | Partial |
| May | May | May |

Rationale: If either analysis can prove NoAlias or MustAlias, trust it.

### Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| CS-PTA k | 2 | Balances precision vs cost |
| Field sensitivity | max_depth: 6 | Good for nested structs |
| Constant indices | enabled | Distinguishes a[0] from a[1] |
| Z3 index refinement | enabled | Already used |

## Implementation

### Phase 1: Add CombinedPtaResult

New file: `crates/saf-bench/src/combined_pta.rs`

```rust
pub struct CombinedPtaResult {
    cs_result: CsPtaResult,
    fs_result: FsPtaResult,
}

impl CombinedPtaResult {
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        let cs_alias = self.cs_result.may_alias_any(p, q);
        let fs_alias = self.fs_result.may_alias(p, q);
        combine_alias_results(cs_alias, fs_alias)
    }
}
```

### Phase 2: Update PTABen Harness

Modify `run_test()` in `ptaben.rs`:
1. Build CallGraph first (needed by both CS-PTA and FS-PTA)
2. Run CS-PTA with k=2
3. Run FS-PTA
4. Create CombinedPtaResult for alias queries

### Phase 3: Test and Validate

- Run `make test` for regression check
- Run `make test-ptaben` to measure improvement

### Phase 4: Documentation

- Update PROGRESS.md with results
- Update FUTURE.md if path_tests need further work

## Expected Impact

| Category | Before | After (Est.) |
|----------|--------|--------------|
| cs_tests | 9/27 | 17-21/27 |
| fs_tests | 6/26 | 12-16/26 |
| path_tests | 1/21 | 3-6/21 |
| basic_c_tests | 23/60 | 25-27/60 |
| **Total** | 118/250 | 136-149/250 |

Conservative estimate: +18-31 additional passes (15-26% improvement)

## Non-Goals

- Path-sensitive alias analysis (would need PathSensitiveAliasChecker from E29)
- Memory leak wrapper recognition (separate feature)

## Prerequisites

- MustAlias singleton bug fix (done in this session)

## Files Modified

- `crates/saf-bench/src/lib.rs` - add combined_pta module
- `crates/saf-bench/src/combined_pta.rs` - new file
- `crates/saf-bench/src/ptaben.rs` - use CombinedPtaResult
