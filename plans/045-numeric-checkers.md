# Plan 045: Division-by-Zero and Shift-Count Checkers

**Epic:** E26 — Additional Numeric Checkers
**Status:** done
**Created:** 2026-01-31
**Completed:** 2026-01-31

## Overview

Add two numeric safety checkers to achieve IKOS parity for basic numeric analysis:

1. **Division-by-Zero Checker (CWE-369)** — Detect division/remainder operations where divisor may be zero
2. **Shift-Count Checker (CWE-682)** — Detect shift operations with invalid shift counts

Both checkers build on E15's abstract interpretation infrastructure (interval domain, fixpoint iterator) and follow the same pattern as existing buffer/integer overflow checkers.

## Background

### IKOS Reference

IKOS implements these checkers with the following behavior:

**Division-by-Zero:**
- Operations: `UDiv`, `SDiv`, `URem`, `SRem` (all 4)
- Logic: Query divisor interval, check if it contains zero
- Severity: Error (definitely zero), Warning (may be zero), Ok (cannot be zero)

**Shift-Count:**
- Operations: `Shl`, `LShr`, `AShr` (all 3)
- Conditions: Shift count < 0 OR shift count >= bit_width
- Severity: Error (definitely invalid), Warning (may be invalid), Ok (valid)

### SAF Infrastructure

- `AbstractDomain` trait + `Interval` implementation (E15)
- `solve_abstract_interp()` fixpoint iterator with widening/narrowing
- `NumericCheckerKind` enum, `NumericFinding` struct, `NumericCheckResult`
- Existing checkers: `check_buffer_overflow()`, `check_integer_overflow()`
- Python bindings: `check_numeric()`, `check_all_numeric()`

## Phases

### Phase 1: Division-by-Zero Checker (Rust)

**Goal:** Implement the division-by-zero checker with Rust E2E tests.

**Tasks:**

1. Add `DivisionByZero` variant to `NumericCheckerKind` enum in `checker.rs`
   - CWE: 369
   - Name: "division_by_zero"

2. Implement `check_division_by_zero()` function:
   ```rust
   pub fn check_division_by_zero(
       module: &AirModule,
       config: &AbstractInterpConfig,
   ) -> NumericCheckResult
   ```
   - Match operations: `SDiv`, `UDiv`, `SRem`, `URem`
   - Query divisor interval (second operand)
   - Check if interval contains zero:
     - Singleton `[0, 0]` → Error
     - Contains zero but not singleton → Warning
   - Skip if divisor is top or bottom

3. Add helper method to `Interval` if needed:
   ```rust
   pub fn contains_zero(&self) -> bool
   pub fn is_singleton_zero(&self) -> bool
   ```

4. Add `check_division_by_zero_with_result()` for shared fixpoint reuse

5. Update `check_all_numeric()` to include division-by-zero

6. Create test fixture `tests/fixtures/e2e/div_by_zero.c`:
   ```c
   // definite_div_zero: x / 0
   // possible_div_zero: x / y where y in [0, 10]
   // safe_div: guarded with if (y != 0)
   // rem_by_zero: x % 0
   // unsigned variants
   ```

7. Compile fixture: `clang -S -emit-llvm -O0 div_by_zero.c -o div_by_zero.ll`

8. Write Rust E2E tests in `crates/saf-analysis/tests/`:
   - `test_div_by_zero_definite` — expects Error
   - `test_div_by_zero_possible` — expects Warning
   - `test_div_by_zero_safe` — expects no finding
   - `test_rem_by_zero` — check remainder operations
   - `test_div_by_zero_unsigned` — check unsigned variants

**Acceptance:**
- `make test` passes
- Division-by-zero findings correct for all test cases

---

### Phase 2: Shift-Count Checker (Rust)

**Goal:** Implement the shift-count checker with Rust E2E tests.

**Tasks:**

1. Add `ShiftCount` variant to `NumericCheckerKind` enum:
   - CWE: 682
   - Name: "shift_count"

2. Implement `check_shift_count()` function:
   ```rust
   pub fn check_shift_count(
       module: &AirModule,
       config: &AbstractInterpConfig,
   ) -> NumericCheckResult
   ```
   - Match operations: `Shl`, `LShr`, `AShr`
   - Get bit width from first operand's type
   - Query shift count interval (second operand)
   - Check two conditions:
     - `shift_count.hi() >= bit_width` → out of bounds
     - `shift_count.lo() < 0` → negative (signed interpretation)
   - Severity:
     - Entire range invalid → Error
     - Partial range invalid → Warning

3. Add `check_shift_count_with_result()` for shared fixpoint reuse

4. Update `check_all_numeric()` to include shift-count

5. Create test fixture `tests/fixtures/e2e/shift_count.c`:
   ```c
   // shift_too_large: x << 32 (32-bit int)
   // shift_negative: x << -1
   // shift_variable_unsafe: x << n where n in [0, 40]
   // shift_variable_safe: x << n where n in [0, 31]
   // all three shift types: shl, lshr, ashr
   ```

6. Compile fixture to `.ll`

7. Write Rust E2E tests:
   - `test_shift_too_large` — expects Error
   - `test_shift_negative` — expects Error/Warning
   - `test_shift_variable_unsafe` — expects Warning
   - `test_shift_variable_safe` — expects no finding
   - `test_shift_all_types` — check shl, lshr, ashr

**Acceptance:**
- `make test` passes
- Shift-count findings correct for all test cases

---

### Phase 3: Python Bindings + E2E Tests

**Goal:** Expose new checkers to Python and add Python E2E tests.

**Tasks:**

1. Update `run_check_numeric()` in `crates/saf-python/src/absint.rs`:
   - Add "division_by_zero" dispatch
   - Add "shift_count" dispatch
   - Update error message to list all available checkers

2. Update `run_check_all_numeric()` to use the updated Rust function

3. Write Python E2E tests in `python/tests/`:
   - `test_check_division_by_zero_definite`
   - `test_check_division_by_zero_possible`
   - `test_check_division_by_zero_safe`
   - `test_check_shift_count_too_large`
   - `test_check_shift_count_negative`
   - `test_check_shift_count_safe`
   - `test_check_all_numeric_includes_new_checkers`

4. Verify Python API returns correct finding properties:
   - `checker` — "division_by_zero" or "shift_count"
   - `severity` — "error" or "warning"
   - `cwe` — 369 or 682
   - `description` — human-readable message

5. Test determinism: run each check twice, verify identical output

**Acceptance:**
- `make test` passes (both Rust and Python)
- All 4 checker types work via `check_numeric(name)`
- `check_numeric("all")` includes all 4 checkers

---

### Phase 4: Documentation Updates

**Goal:** Update documentation to reflect E26 completion.

**Tasks:**

1. Update `docs/tool-comparison.md`:
   - In "Bug Detection / Checkers" table (Section 4):
     - Change "Division-by-zero" row: SAF column from "No" to "**Yes**"
     - Change "Shift-count checker" row: SAF column from "No" to "**Yes**"
   - Update "Remaining Gaps — Tier 3 (Checkers)" section:
     - Remove division-by-zero and shift-count from the list
   - Update "vs IKOS" summary section to note these are now implemented
   - Update numeric checker count in summary

2. Update `plans/FUTURE.md`:
   - Add tutorial entries under "Future Tutorials" section:
     ```
     ### From E26: Additional Numeric Checkers

     1. **Division-by-Zero Detection** — Finding divisions that may crash.
        Demonstrates `check_numeric("division_by_zero")` with guarded
        and unguarded examples.

     2. **Shift-Count Validation** — Detecting undefined shift behavior.
        Demonstrates `check_numeric("shift_count")` with bit-width
        and negative shift examples.
     ```

3. Update `plans/PROGRESS.md`:
   - Mark E26 as done
   - Add plan 045 to Plans Index with status "done"
   - Update "Next Steps" section
   - Add to Session Log

**Acceptance:**
- All documentation accurately reflects E26 implementation
- No broken references or stale information

---

## Test Fixtures

### div_by_zero.c

```c
#include <stdint.h>

// Definite division by zero - Error
int definite_div_zero(int x) {
    return x / 0;
}

// Possible division by zero - Warning
int possible_div_zero(int x, int y) {
    // y is in [0, 10] after this check
    if (y >= 0 && y <= 10) {
        return x / y;  // y could be 0
    }
    return 0;
}

// Safe division - guarded
int safe_div(int x, int y) {
    if (y != 0) {
        return x / y;  // Safe: y != 0
    }
    return 0;
}

// Remainder by zero - Error
int rem_by_zero(int x) {
    return x % 0;
}

// Unsigned division by zero
uint32_t unsigned_div_zero(uint32_t x) {
    return x / 0u;
}

// Safe after narrowing
int safe_after_check(int x, int y) {
    if (y > 0) {
        return x / y;  // Safe: y > 0
    }
    return 1;
}
```

### shift_count.c

```c
#include <stdint.h>

// Shift count equals bit width - Error
int shift_equals_width(int x) {
    return x << 32;  // UB: 32 >= 32
}

// Shift count exceeds bit width - Error
int shift_exceeds_width(int x) {
    return x << 64;  // UB: 64 >= 32
}

// Variable shift that may exceed - Warning
int shift_variable_unsafe(int x, int n) {
    if (n >= 0 && n <= 40) {
        return x << n;  // Warning: n could be >= 32
    }
    return 0;
}

// Variable shift that is safe
int shift_variable_safe(int x, int n) {
    if (n >= 0 && n < 32) {
        return x << n;  // Safe: n in [0, 31]
    }
    return 0;
}

// Logical shift right
uint32_t lshr_too_large(uint32_t x) {
    return x >> 32;  // Error
}

// Arithmetic shift right
int ashr_too_large(int x) {
    return x >> 32;  // Error
}

// Negative shift count (if signed operand)
int shift_negative(int x, int n) {
    if (n >= -5 && n <= 5) {
        return x << n;  // Warning: n could be negative
    }
    return 0;
}

// Safe shift with mask
int shift_masked(int x, int n) {
    int safe_n = n & 31;  // Mask to 0-31
    return x << safe_n;   // Safe
}
```

---

## CWE Mapping

| Checker | CWE | Description |
|---------|-----|-------------|
| Division-by-Zero | CWE-369 | Divide By Zero |
| Shift-Count | CWE-682 | Incorrect Calculation |

---

## Dependencies

- E15: Abstract Interpretation (interval domain, fixpoint iterator) — **done**
- Existing checker infrastructure (`NumericCheckerKind`, `NumericFinding`) — **done**

---

## Risks

1. **Interval precision** — If intervals are too wide (top), checkers produce no findings. Mitigation: test with `-O0` to preserve program structure.

2. **Bit-width extraction** — Need to get bit width from instruction operands. LLVM IR types are available via AIR metadata.

3. **Signed vs unsigned shifts** — LLVM IR shift count is typically unsigned. Need to handle signed interpretation for negative detection.

---

## References

- IKOS division_by_zero.cpp: https://github.com/NASA-SW-VnV/ikos/blob/master/analyzer/src/checker/division_by_zero.cpp
- IKOS shift_count.cpp: https://github.com/NASA-SW-VnV/ikos/blob/master/analyzer/src/checker/shift_count.cpp
- CWE-369: https://cwe.mitre.org/data/definitions/369.html
- CWE-682: https://cwe.mitre.org/data/definitions/682.html
- SAF E15 Abstract Interpretation: plans/029-abstract-interpretation.md
