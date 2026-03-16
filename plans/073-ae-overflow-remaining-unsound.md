# Plan 073: ae_overflow_tests Remaining Unsound Fixes

## Problem

43 UNSAFE_BUFACCESS oracles remain unsound in PTABen `ae_overflow_tests`, all reporting "No buffer overflow finding in function (memcpy tracking not supported)". Two checker functions produce findings:
1. `check_buffer_overflow_with_pta_and_result()` — GEP-based, **uses PTA** ✓
2. `check_memcpy_overflow_impl()` — memcpy/spec-based, **does NOT use PTA** ✗

The core issue: `check_memcpy_overflow_impl` uses `build_pointer_origins()` (simple intraprocedural store→load chain) instead of PTA for pointer resolution. At `-O0`, LLVM generates store/load pairs through alloca'd addresses that break this heuristic. PTA already handles these patterns correctly.

## Root Cause Summary (43 tests)

| Category | Count | Pattern | Root Cause |
|----------|-------|---------|------------|
| A: Loop GEP | 8 | `for(i<100) data[i]=src[i]` | Loop index → TOP after widening, GEP checker skips TOP |
| B: memcpy/memmove | 12 | `memcpy(dst,src,strlen()+1)` | No PTA in memcpy checker + size from strlen is TOP |
| C: strcpy/strcat | 9 | `strcpy(small_dst, large_src)` | No PTA in memcpy checker (pointer origin broken) |
| D: strncpy | 2 | `strncpy(dst50, src, 99)` | No PTA in memcpy checker |
| E: snprintf/swprintf | 3 | `swprintf(dst50, 100, ...)` | `buffer_size` semantic unrecognized + swprintf missing |
| F: Data-dependent idx | 5 | `buf[atoi(input)]` | GEP checker skips TOP indices |
| G: wcscpy/wcscat | 4 | `wcscpy(small, large)` | Missing specs for wide-char functions |

No SVF-specific handling needed — all fixes are generic static analysis improvements.

## Implementation

### Phase A: PTA Integration for Memcpy Checker (~80 LOC)
**Impact: ~23 tests (Categories B, C, D)**

**Files:**
- `crates/saf-analysis/src/absint/checker.rs`
- `crates/saf-analysis/src/absint/mod.rs`
- `crates/saf-bench/src/ptaben.rs`

**Changes:**
1. Create `check_memcpy_overflow_with_pta_and_specs(module, config, pta, specs)` — new public function that passes PTA to the implementation
2. Create `check_memcpy_overflow_with_pta_impl(module, result, pta, specs)` — mirrors `check_memcpy_overflow_impl` but replaces:
   - `build_pointer_origins()` + `find_dest_allocation_size()` → `build_loc_allocation_sizes()` + `find_allocation_size_with_pta()`
   - These PTA-based functions already exist and are used by the GEP checker
3. Apply PTA resolution to BOTH sides of `copies_string` comparisons (dst and src)
4. Export new function in `mod.rs`
5. Wire in PTABen harness (ptaben.rs:622-623): replace `check_memcpy_overflow_with_specs` with PTA-aware variant

### Phase B: Wide-Char Specs (~40 lines YAML)
**Impact: 4 tests (Category G)**

**File:** `share/saf/specs/libc/string.yaml`

Add `wcscpy` (param 1: `copies_string`), `wcscat` (param 1: `copies_string`), `wcsncpy` (param 2: `max_length`), `wcsncat` (param 2: `max_length`). Mirror existing strcpy/strcat/strncpy/strncat patterns.

### Phase C: TOP Index Warning (~20 LOC)
**Impact: 5 tests (Category F) + ~8 tests (Category A)**

**File:** `crates/saf-analysis/src/absint/checker.rs`

In `check_buffer_overflow_with_pta_and_result()`, change the TOP skip at line 562:
- When `idx_interval.is_top()` AND allocation size is known → emit `Warning` finding
- Set `affected_ptr: Some(base_operand)` for oracle matching
- This catches both data-dependent indices (CWE129) and loop variables widened to TOP (CWE805/CWE131 loops)

### Phase D: snprintf/swprintf Handling (~5 LOC + ~15 lines YAML)
**Impact: 3 tests (Category E)**

**Files:**
- `crates/saf-analysis/src/absint/checker.rs`
- `share/saf/specs/libc/stdio.yaml`

1. In `get_spec_copy_info()`: add `|| p.semantic.as_deref() == Some("buffer_size")` to the semantic check
2. Add `swprintf` spec to stdio.yaml with param 1 as `buffer_size`
3. For wide-char `buffer_size` (swprintf): multiply size by 4 (sizeof wchar_t) before comparing against byte-sized allocation. Use function name heuristic (`sw` prefix or `wchar` in name).

### Phase E: Source Overread Detection (~20 LOC)
**Impact: ~5 tests (CWE126 subset of Category B)**

**File:** `crates/saf-analysis/src/absint/checker.rs`

In the PTA-aware memcpy checker, after checking dest overflow, also check source overread:
- For memcpy/memmove: resolve source operand via PTA, get source allocation size
- If `copy_size > source_alloc_size` → emit Warning/Error with CWE 126 and `affected_ptr: Some(src_operand)`

### Phase F: TOP Size Fallback (~15 LOC)
**Impact: remaining strlen-based tests**

**File:** `crates/saf-analysis/src/absint/checker.rs`

When memcpy/memmove size interval is TOP (e.g., from `strlen()`):
- Instead of skipping, compare `src_alloc` vs `dest_alloc` directly (like `copies_string` already does)
- If `src_alloc.lo() > dest_alloc.hi()` → emit Warning
- Conservative: only fires when source minimum exceeds destination maximum

## Dependency Order

```
Phase A (PTA integration) ─┬─ Phase B (wide-char specs)
                           ├─ Phase D (snprintf/swprintf)
                           ├─ Phase E (source overread)
                           └─ Phase F (TOP size fallback)
Phase C (TOP index) ─── independent
```

## Expected Results

| Phase | Tests Fixed | Cumulative Unsound |
|-------|------------|-------------------|
| A | ~23 | ~20 |
| B | ~4 | ~16 |
| C | ~13 | ~3 |
| D | ~3 | ~0 |
| E+F | 0-3 | ~0 |

**Target: 43 → ~0 unsound**. Total ~195 LOC.

## Verification

After each phase:
```bash
# Inside make shell:
cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "ae_overflow_tests/*"

# Full regression check:
cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled

# Unit tests + lint:
# (run via make test && make lint from host)
```
