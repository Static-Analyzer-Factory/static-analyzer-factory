# Plan 047: PTABen Full Integration

## Overview

Complete the PTABen benchmark integration (Plan 046) by:
1. Integrating SAF's existing checkers (double-free, memory-leak) with PTABen validation
2. Fixing ValueId tracking for alias oracle validation
3. Adding abstract interpretation oracle support (`svf_assert`, `UNSAFE_LOAD`)

## Current State (Plan 046 Results)

```
PTABen Benchmark Results
=========================

Category                    Pass   Fail   Skip  Total  Issue
-------------------------------------------------------------------------------------
double_free                    1     43      0     44  Checker exists but not integrated
mem_leak                      36     20     25     81  Checker exists but not integrated
basic_c_tests                  2      2     56     60  ValueId tracking broken
cs_tests                       0      3     24     27  ValueId tracking broken
fs_tests                       0      0     26     26  ValueId tracking broken
path_tests                     0      0     21     21  ValueId tracking broken
complex_tests                  0      0     49     49  No oracle calls found
ae_assert_tests                0      0     84     84  svf_assert not implemented
ae_nullptr_deref_tests         0      0     79     79  UNSAFE_LOAD not implemented
ae_* (other)                   0      0     93     93  Various AE oracles
mta                            0      0     59     59  MTA not implemented (deferred)
-------------------------------------------------------------------------------------
TOTAL                         39     68    545    652
```

## Goals

1. **Checker Integration** — Wire SAF's double-free and memory-leak checkers into PTABen validation
2. **ValueId Fix** — Store actual ValueIds (not debug strings) for alias oracle validation
3. **AE Oracles** — Support `svf_assert` and `UNSAFE_LOAD` oracles from PTABen
4. **Target Results** — 400+ pass, <50 fail, <200 skip (60%+ pass rate)

## Design

### Phase 1: Checker Integration

**Problem**: The current `validate_expectation` function skips double-free and mem-leak validation:
```rust
Expectation::DoubleFree { kind, location } => {
    match kind {
        DoubleFreeKind::Safe | DoubleFreeKind::FalsePositive => Outcome::Pass,
        _ => Outcome::Skip { reason: "Double-free checker not integrated" },
    }
}
```

**Solution**: Run SAF's SVFG-based checkers and match findings to oracle allocation sites.

**Key Insight from SVF**: PTABen oracles like `DOUBLEFREEMALLOC(n)` are wrapper functions around `malloc`. The allocation site is the *return value* of the oracle function call, not the argument. To validate:
1. Find all calls to oracle functions (`DOUBLEFREEMALLOC`, `PLKMALLOC`, etc.)
2. Track the return ValueId of each oracle call (the allocated pointer)
3. Run SAF's checker
4. Check if the oracle's allocation site appears in checker findings

**Data Flow**:
```
Source: DOUBLEFREEMALLOC(n)
        ↓ returns ptr (this is the allocation site)
        ... code uses ptr ...
        free(ptr)
        free(ptr)  ← double-free

Checker: Reports finding at the first free() call
Oracle Match: Did the finding's source trace back to our DOUBLEFREEMALLOC call?
```

### Phase 2: Alias ValueId Fix

**Problem**: Alias expectations store ValueIds as debug strings:
```rust
ptr_a: args.first().map(|v| format!("{v:?}")).unwrap_or_default(),
```
This loses the actual ValueId, making PTA queries impossible.

**Solution** (following SVF's pattern):
1. Store actual `ValueId` in the `Expectation` struct
2. Extract from instruction operands (args[0], args[1] for MAYALIAS(p,q))
3. Query PTA directly with `pta.may_alias(ptr_a, ptr_b)`

**Key Change**: The `Expectation::Alias` variant changes from:
```rust
Alias { kind: AliasKind, ptr_a: String, ptr_b: String }
```
to:
```rust
Alias { kind: AliasKind, ptr_a: ValueId, ptr_b: ValueId }
```

### Phase 3: Abstract Interpretation Oracles

**Oracle: `svf_assert(condition)`**

PTABen's AE tests use `svf_assert(bool)` as an assertion prover oracle:
```c
extern void svf_assert(bool);
int main() {
    int a[3][3];
    a[2][2] = 8;
    int b = 2, c = 2;
    svf_assert(a[b][c] == 8);  // Should be provable
}
```

**Validation Logic**:
1. Find all calls to `svf_assert`
2. Run SAF's abstract interpretation analysis
3. At each call site, evaluate the condition operand's interval:
   - `[n, m] where n > 0`: Pass (always true)
   - `[0, 0]`: Fail (always false)
   - `[0, m] where m > 0`: Fail (may be false)

**Oracle: `UNSAFE_LOAD(ptr)` / `SAFE_LOAD(ptr)`**

PTABen's null-pointer tests mark expected unsafe/safe pointer dereferences:
```c
extern void UNSAFE_LOAD(void *ptr);
int main() {
    int *arr = NULL;
    UNSAFE_LOAD(arr);  // Should trigger null warning
}
```

**Validation Logic**:
1. Find all calls to `UNSAFE_LOAD` and `SAFE_LOAD`
2. Run SAF's null-deref checker
3. Match oracle pointers against checker findings

### New Expectation Types

```rust
pub enum Expectation {
    /// Alias relationship (Phase 2: fixed with actual ValueIds)
    Alias {
        kind: AliasKind,
        ptr_a: ValueId,
        ptr_b: ValueId,
    },

    /// Memory leak expectation (Phase 1: integrated with checker)
    MemLeak {
        kind: MemLeakKind,
        /// The allocation site ValueId (return value of oracle function)
        alloc_site: ValueId,
        /// Call instruction ID for location tracking
        call_site: InstructionId,
    },

    /// Double-free expectation (Phase 1: integrated with checker)
    DoubleFree {
        kind: DoubleFreeKind,
        /// The allocation site ValueId
        alloc_site: ValueId,
        call_site: InstructionId,
    },

    /// Abstract interpretation assertion (Phase 3)
    Assert {
        /// The condition operand ValueId
        condition: ValueId,
        /// Instruction ID for state lookup
        call_site: InstructionId,
        /// Whether this test is in ae_assert_tests_fail (expect failure)
        expect_failure: bool,
    },

    /// Null pointer check oracle (Phase 3)
    NullCheck {
        kind: NullCheckKind,
        /// The pointer being checked
        ptr: ValueId,
        call_site: InstructionId,
    },

    // ... existing variants ...
}
```

## Implementation Phases

### Phase 1: Double-Free Checker Integration (1 session)

**Files to modify**:
- `crates/saf-bench/src/ptaben.rs` — Main validation logic
- `crates/saf-bench/src/lib.rs` — Update Expectation types if needed

**Tasks**:
1. Add SVFG building to the validation pipeline
2. Implement `validate_double_free()` that:
   - Runs SAF's `double-free` checker
   - Collects flagged allocation sites from findings
   - Matches against `DOUBLEFREEMALLOC`/`SAFEMALLOC`/etc. expectations
3. Extract allocation site ValueId (return value of oracle call)
4. Test with `double_free/df*.bc` tests

**Verification**: Run `make test-ptaben` and confirm double_free category improves from 1 pass to 40+ pass.

### Phase 2: Memory-Leak Checker Integration (1 session)

**Files to modify**:
- `crates/saf-bench/src/ptaben.rs`

**Tasks**:
1. Implement `validate_mem_leak()` following same pattern as Phase 1
2. Handle all MemLeakKind variants:
   - `Safe` → no finding expected
   - `NeverFree` → finding expected
   - `PartialLeak` → finding expected (may-reach)
   - `ContextLeak` → context-sensitive finding expected
   - `*FP` / `FN` variants → expected imprecision
3. Test with `mem_leak/malloc*.bc` tests

**Verification**: Run `make test-ptaben` and confirm mem_leak category improves from 36 pass to 70+ pass.

### Phase 3: Alias ValueId Tracking Fix (1 session)

**Files to modify**:
- `crates/saf-bench/src/lib.rs` — Change `Expectation::Alias` type
- `crates/saf-bench/src/ptaben.rs` — Fix extraction and validation

**Tasks**:
1. Change `ptr_a`/`ptr_b` from `String` to `ValueId`
2. Update `alias_expectation()` to extract actual ValueIds from operands
3. Implement `validate_alias()` that queries `pta.may_alias()`
4. Handle all AliasKind variants per SVF semantics:
   - `MayAlias` → `alias() != NoAlias`
   - `NoAlias` → `alias() == NoAlias`
   - `MustAlias` → `alias() == Must` (SAF uses May)
   - `PartialAlias` → `alias() == May`
   - `ExpectedFail*` → expected imprecision

**Verification**: Run `make test-ptaben` and confirm basic_c_tests/cs_tests/fs_tests/path_tests categories show significant improvement.

### Phase 4: svf_assert Oracle Support (1 session)

**Files to modify**:
- `crates/saf-bench/src/lib.rs` — Add `Expectation::Assert`
- `crates/saf-bench/src/ptaben.rs` — Add oracle handler and validation

**Tasks**:
1. Add `svf_assert` to oracle handlers
2. Extract condition operand ValueId
3. Run abstract interpretation analysis on the module
4. Evaluate condition interval at call site
5. Handle `ae_assert_tests_fail` category (expect imprecision)

**Verification**: Run `make test-ptaben` and confirm ae_assert_tests category shows improvement.

### Phase 5: UNSAFE_LOAD Oracle Support (1 session)

**Files to modify**:
- `crates/saf-bench/src/lib.rs` — Add `Expectation::NullCheck`
- `crates/saf-bench/src/ptaben.rs` — Add oracle handlers and validation

**Tasks**:
1. Add `UNSAFE_LOAD` and `SAFE_LOAD` oracle handlers
2. Run null-deref checker
3. Match findings against oracle expectations
4. Handle `ae_nullptr_deref_tests_failed` category

**Verification**: Run `make test-ptaben` and confirm ae_nullptr_deref_tests category shows improvement.

### Phase 6: Documentation and Cleanup (1 session)

**Tasks**:
1. Update `docs/future.md` with remaining gaps
2. Update `docs/tool-comparison.md` with PTABen results
3. Update `plans/PROGRESS.md`
4. Add unit tests for new validation logic
5. Final full benchmark run and summary

## Dependencies

- `saf-analysis::checkers` — Double-free and memory-leak checkers (already exists)
- `saf-analysis::pta` — Pointer analysis (already exists)
- `saf-analysis::absint` — Abstract interpretation (already exists, needs integration)
- `saf-analysis::svfg` — SVFG builder (already exists)

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Allocation site tracking may be complex due to LLVM optimizations | Start with O0-compiled tests; add debug info tracking if needed |
| Abstract interpretation may not handle all C constructs in AE tests | Skip unsupported patterns; document limitations |
| Some PTABen tests may have bugs or unclear oracles | Document known issues; focus on high-confidence tests |

## Success Criteria

| Metric | Before (P046) | Target (P047) |
|--------|---------------|---------------|
| Total Pass | 39 | 400+ |
| Total Fail | 68 | <50 |
| Total Skip | 545 | <200 |
| Pass Rate | 6% | 60%+ |
| double_free Pass | 1 | 40+ |
| mem_leak Pass | 36 | 70+ |
| Alias Tests Pass | 2 | 100+ |
| AE Tests Pass | 0 | 100+ |

## Related Documents

- Plan 046: PTABen Benchmark Integration (infrastructure setup)
- Plan 028: Checker Framework (SVFG-based checkers)
- Plan 029: Abstract Interpretation (interval domain)
- `docs/tool-comparison.md` — Feature gap analysis

## References

- [SVF PointerAnalysis.cpp](https://github.com/SVF-tools/SVF/blob/master/svf/lib/MemoryModel/PointerAnalysis.cpp) — Oracle validation logic
- [PTABen aliascheck.h](https://github.com/SVF-tools/Test-Suite/blob/master/aliascheck.h) — Alias oracle functions
- [IKOS intrinsic.h](https://github.com/NASA-SW-VnV/ikos/blob/master/analyzer/include/ikos/analyzer/intrinsic.h) — IKOS oracle functions
