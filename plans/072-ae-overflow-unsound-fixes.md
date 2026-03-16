# Plan 072: ae_overflow_tests Unsound Fixes

## Problem Statement

PTABen `ae_overflow_tests`: 59 unsound out of 223 oracle checks (59 tests, 164 Exact).
Two categories of failure:

### Category A: False Negatives (53 cases) — "No buffer overflow finding"
UNSAFE_BUFACCESS oracles where SAF should detect overflow but doesn't.

### Category B: False Positives (6 cases) — "1 buffer overflow finding(s)"
SAFE_BUFACCESS oracles where SAF incorrectly reports overflow.

## Root Cause Analysis

### RC1: LLVM alloca size not tracked (affects ~40 false negatives)
The LLVM frontend maps ALL `alloca` instructions with `size_bytes: None` and `operands: vec![]`
(`mapping.rs:546`). The LLVM IR contains explicit sizes like `alloca i8, i64 50` (50-byte alloca),
but this information is discarded.

Consequence: `extract_allocation_sizes()` falls through to `Interval::make_top(64)` for all
stack allocations. The memcpy checker skips `is_top()` sizes, so it never compares memcpy size
against alloca capacity.

**Fix:** Extract the alloca size from the LLVM instruction in the frontend and populate
`size_bytes` in AIR. For variable-size allocas (VLAs), add the size operand to the operands list.

### RC2: strcpy/strcat have no byte_count semantic (affects 4 ExtAPI_strcat + ~8 *_cpy tests)
`get_spec_copy_info()` only handles functions with `byte_count` or `max_length` parameter
semantics. `strcpy` and `strcat` are null-terminated operations with no explicit size parameter,
so the memcpy overflow checker never inspects them.

**Fix:** Add a new spec semantic `copies_string` to mark string copy operations where the
copy size is implicitly `strlen(src) + 1`. The checker can then model these as: if the source
string content size (from absint or a heuristic) exceeds destination allocation, report overflow.

**However**, this is hard to do precisely without string length tracking. A simpler intermediate
approach: add a new spec semantic `dst_may_overflow` (or `role: string_copy_unbounded`) that
the checker uses to flag these operations when the destination buffer is smaller than some
threshold. Even simpler: for `strcpy(dst, src)`, if we can determine `src` is a constant string
literal or a buffer with known size, compare against `dst` allocation size.

Actually the simplest correct approach: for these PTABen tests, the source buffers have known
sizes (they're `strlen("worldworld") + strlen("Hello") + 1 = 16` for strcat, or constant-sized
arrays for strcpy). The checker should model `strcpy` as copying `strlen(src) + 1` bytes and
`strcat` as appending `strlen(src) + 1` bytes to the existing content.

### RC3: GEP checker `affected_ptr: None` causes false positive over-matching (affects 6 cases)
The GEP-based buffer overflow checker (`check_buffer_overflow_with_pta_and_result`) sets
`affected_ptr: None` for ALL findings. In `validate_buffer_access`, when `affected_ptr` is
`None`, the filter says `None => true` (conservative match). This means ANY GEP finding in a
function matches ALL SAFE_BUFACCESS oracles in that function, even if the finding is for a
completely different pointer.

Additionally, the PTA-based allocation size resolution (`build_loc_allocation_sizes`) may join
alloc sizes from multiple call sites (e.g., `malloc(400)` in `_bad` and `malloc(800)` in
`goodG2B`), producing imprecise intervals.

**Fix:** Set `affected_ptr: Some(base_operand)` in the GEP checker findings. This allows
precise pointer matching in the validation.

### RC4: CWE129 data-dependent index (affects 3 tests)
Tests like `CWE129_fgets_01` use `data = atoi(inputBuffer)` where `data` comes from user input
(`fgets`). The array access `buffer[data]` with only `data >= 0` check (no upper bound check)
means any positive value can overflow `buffer[10]`.

SAF's abstract interpretation resolves `atoi` return as top (unbounded integer). The GEP
checker should flag `buffer[data]` where the index interval is [0, TOP] and the allocation
is 10 elements. This should work with RC1 fixed (alloca tracking), since `buffer[10]` would
have known size. The GEP checker checks `idx_interval.hi() >= size_hi` — but it skips when
`idx_interval.is_top()` (line 562). **Fix:** For GEP indices that are top with known alloc
size, emit a Warning finding.

### RC5: Alloca size in elements vs bytes
LLVM `alloca i8, i64 50` allocates 50 bytes. But `alloca [100 x i64]` allocates 100 * 8 = 800
bytes (the type is 800-byte array). The `size_bytes` needs to be the total byte size, not the
element count. For dynamic allocas like `alloca i8, i64 N`, the size is `N * sizeof(element)`.

## Phases

### Phase A: LLVM Alloca Size Extraction (~30 LOC)
**Files:** `crates/saf-frontends/src/llvm/mapping.rs`

1. In the `Alloca` arm of `convert_instruction()`, extract the alloca size:
   - For fixed-size array types like `alloca [100 x i8]`: compute `num_elements * element_size`
     from the LLVM type and set `size_bytes = Some(total)`.
   - For dynamic allocas like `alloca i8, i64 50`: the size operand is the allocation count.
     Compute `count * element_size_in_bytes` and either:
     (a) If the count is a constant, set `size_bytes = Some(count * elem_size)`
     (b) If dynamic, add the count to operands and set `size_bytes = None` (existing fallback)
   - For simple `alloca i32` (single element): set `size_bytes = Some(sizeof(i32))` = 4.

2. This naturally propagates through `extract_allocation_sizes()` which already handles
   `size_bytes: Some(n)` → `Interval::singleton(n, 64)`.

**Expected impact:** Fixes ~35 false negatives where alloca size was unknown.

### Phase B: GEP Checker affected_ptr (~5 LOC)
**Files:** `crates/saf-analysis/src/absint/checker.rs`

1. In `check_buffer_overflow_with_pta_and_result()`, set `affected_ptr: Some(base_operand)`
   for both negative-index and size-exceeded findings (lines 573, 605).

**Expected impact:** Fixes ~6 false positives by enabling precise pointer matching.

### Phase C: Top-index GEP Warning (~10 LOC)
**Files:** `crates/saf-analysis/src/absint/checker.rs`

1. In `check_buffer_overflow_with_pta_and_result()`, after the `is_top()` skip (line 562),
   add a check: if the index is top AND we have a known allocation size for the base pointer,
   emit a Warning finding (unbounded index with bounded buffer).

**Expected impact:** Fixes ~3 CWE129 data-dependent index cases.

### Phase D: String Copy Overflow Detection (~60 LOC)
**Files:** `crates/saf-analysis/src/absint/checker.rs`, `share/saf/specs/libc/string.yaml`

1. Add a new spec semantic `copies_string` for `strcpy`, `strcat`, `wcscpy`, `wcscat`.
   These functions copy an entire null-terminated string without explicit size.

2. In `check_memcpy_overflow_impl()`, add handling for `copies_string` semantic:
   - For `strcpy(dst, src)`: estimate copy size as `strlen(src) + 1`. If `src` is a constant
     string literal (identifiable from global constants in AIR), use its known length. If `src`
     is a buffer with known allocation size, use that as an upper bound.
   - For `strcat(dst, src)`: same as strcpy but the effective size is `strlen(existing) + strlen(src) + 1`.
     As a conservative approximation: if `alloc_size(src) > alloc_size(dst)`, flag as potential overflow.

3. Alternative simpler approach: add `role: string_copy_unbounded` and in the checker, for
   these calls, compare the source buffer's known size against the destination buffer's known
   size. If source > destination, report finding.

**Expected impact:** Fixes ~4 ExtAPI_strcat tests + ~8 strcpy/cpy tests.

### Phase E: Verification
1. Run `ae_overflow_tests` and confirm improvement from 59→~0 unsound.
2. Run full PTABen suite to verify no regressions.
3. Run `make test` for all unit tests.
4. Run `make lint`.

## Expected Results
- Phase A+B+C+D combined: 59 unsound → ~5-10 unsound (some may remain for complex patterns)
- No regressions in other PTABen categories
- ~100 LOC total changes

## Risk Assessment
- **Phase A** (alloca size): Low risk. Only adds information that was previously lost. Existing
  code paths handle `size_bytes: Some(n)` correctly.
- **Phase B** (affected_ptr): Low risk. Only improves precision of finding-to-oracle matching.
- **Phase C** (top-index): Low risk. Only adds new findings for genuinely suspicious patterns.
- **Phase D** (string copy): Medium risk. New analysis capability. Need to ensure no false
  positives on correct code.
