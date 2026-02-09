# Tutorial 2: Allocation Size Calculation Overflow

## What You'll Learn

- How integer overflow in size calculations leads to buffer overflows
- Common vulnerable patterns in allocation code
- How to implement safe allocation size checking
- Using `check_all_numeric()` to find multiple issue types

## Prerequisites

> Complete [Tutorial 1: Basic Overflow](../01-basic-overflow/README.md) first.

## The Vulnerability

Allocation size overflow is a critical vulnerability pattern where integer
arithmetic used to calculate allocation sizes overflows, resulting in an
undersized buffer. When the program writes the expected amount of data to
this undersized buffer, a heap buffer overflow occurs.

### Why It's Dangerous

This vulnerability is particularly severe because:
1. **Silent failure**: The allocation "succeeds" with a smaller buffer
2. **Delayed crash**: The overflow happens later during data writes
3. **Exploitable**: Attackers can control the overflow to corrupt heap metadata
4. **Common**: Image/video processing, network protocols, and file formats
   frequently perform multi-factor size calculations

### Vulnerable Patterns

```c
/* Pattern 1: Simple multiplication overflow */
void *allocate_pixel_buffer(int width, int height) {
    size_t size = width * height;  /* Overflow here */
    return malloc(size);
}

/* Pattern 2: Multi-factor overflow */
void *allocate_rgba_buffer(int width, int height, int channels, int bpc) {
    int total = width * height * channels * bpc;  /* Multiple overflow points */
    return malloc((size_t)total);
}

/* Pattern 3: Array element count */
struct Pixel *allocate_pixel_array(int count) {
    return malloc(count * sizeof(struct Pixel));  /* count * 4 can overflow */
}

/* Pattern 4: Reallocation growth */
void *grow_buffer(void *old, size_t old_size, int factor) {
    size_t new_size = old_size * factor;  /* Overflow */
    return realloc(old, new_size);
}

/* Pattern 5: Stride calculation */
void *allocate_image_with_stride(int w, int h, int bpp, int align) {
    int stride = ((w * bpp + align - 1) / align) * align;  /* Overflow */
    int total = stride * h;  /* Another overflow */
    return malloc(total);
}
```

## Safe Implementation

```c
void *allocate_pixel_buffer_safe(int width, int height) {
    /* Check for negative inputs */
    if (width <= 0 || height <= 0) {
        return NULL;
    }

    /* Use 64-bit arithmetic for the multiplication */
    uint64_t size = (uint64_t)width * (uint64_t)height;

    /* Check result fits in size_t and is reasonable */
    if (size > SIZE_MAX || size > (1ULL << 30)) {
        return NULL;
    }

    return malloc((size_t)size);
}
```

The safe version:
1. Validates inputs are positive
2. Uses wider integer types for intermediate calculations
3. Checks the result fits in the target type
4. Enforces a reasonable maximum size

## How SAF Detects It

SAF uses the same interval analysis as Tutorial 1, but focuses on values
flowing into allocation functions:

1. **Track multiplication operands**: For `width * height`, SAF computes
   the intervals of both operands
2. **Compute product interval**: The product of two intervals may exceed
   32-bit signed range
3. **Check allocation argument**: If the interval flowing into `malloc()`
   could have wrapped, report a warning

### Example Analysis

For `allocate_pixel_buffer(int width, int height)`:
- Parameters start with full range: `[-2147483648, 2147483647]`
- `width * height` product: interval analysis detects this can overflow
- Value flows to `malloc()`: SAF reports potential allocation size overflow

For `allocate_pixel_buffer_safe()`:
- After `if (width <= 0 || ...)`: `width` narrows to `[1, 2147483647]`
- After casting to `uint64_t`: arithmetic cannot overflow 64-bit
- After `if (size > SIZE_MAX || ...)`: `size` is bounded
- SAF knows the value is safe when it reaches `malloc()`

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Step 1: Compiling C source to LLVM IR...
  Compiled: vulnerable.c -> vulnerable.ll

Step 2: Loading via LLVM frontend...
  Project loaded: <saf.Project object>

Step 3: Running abstract interpretation...
  Blocks analyzed: 25
  Widening applications: 0
  Converged: True
  Functions analyzed: 7

Step 4: Running all numeric checkers...
  Total findings: 18
    buffer_overflow: 0 non-safe finding(s)
    integer_overflow: 12 non-safe finding(s)

Step 5: Integer overflow analysis (CWE-190)...
  Arithmetic operations checked: 18
  Potential overflow issues: 12

  Findings by function:
    allocate_image_with_stride: 4 warning(s), 0 error(s)
    allocate_pixel_array: 1 warning(s), 0 error(s)
    allocate_pixel_buffer: 1 warning(s), 0 error(s)
    allocate_rgba_buffer: 3 warning(s), 0 error(s)
    grow_buffer: 1 warning(s), 0 error(s)

Step 6: Detailed findings in allocation functions...

  allocate_pixel_buffer:
    [WARNING] multiplication may overflow
      Interval: [-2147483648, 2147483647]

  allocate_rgba_buffer:
    [WARNING] multiplication may overflow
      Interval: [-2147483648, 2147483647]
    ...

Step 7: Checking the safe implementation...
  allocate_pixel_buffer_safe: No overflow findings (properly checked)

============================================================
SUMMARY: Allocation Size Overflow Analysis
  Functions analyzed: 7
  Arithmetic operations: 18
  Potential overflows: 12

  POTENTIAL ALLOCATION SIZE OVERFLOW DETECTED
  The following functions have overflow risks:
    - allocate_image_with_stride: 4 issue(s)
    - allocate_pixel_array: 1 issue(s)
    - allocate_pixel_buffer: 1 issue(s)
    - allocate_rgba_buffer: 3 issue(s)
    - grow_buffer: 1 issue(s)

  Impact: Integer overflow in size calculations can
  cause undersized allocations, leading to heap buffer
  overflows when the full expected data is written.
============================================================
```

## Understanding the Detection Script

```python
# Run all numeric checkers at once
all_findings = proj.check_all_numeric()

# Categorize by checker type
by_checker: dict[str, list] = {}
for f in all_findings:
    by_checker.setdefault(f.checker, []).append(f)

# Focus on integer overflow specifically
overflow_findings = proj.check_numeric("integer_overflow")

# Group by function to identify problematic code
by_function: dict[str, list] = {}
for f in non_safe_overflow:
    by_function.setdefault(f.function, []).append(f)
```

Key differences from Tutorial 1:
- Uses `check_all_numeric()` to run multiple checkers
- Groups findings by function for easier triage
- Compares buggy vs. safe implementations

## Real-World Impact

This vulnerability pattern has caused numerous CVEs:

- **CVE-2022-0543**: Redis Lua sandbox escape via integer overflow
- **CVE-2021-22555**: Linux kernel netfilter integer overflow
- **CVE-2020-8597**: pppd buffer overflow via size calculation

Image and video codecs are particularly affected because they commonly
multiply user-controlled dimensions.

## Exercises

1. **Add more checks**: Modify `allocate_rgba_buffer()` to be safe. Verify
   SAF no longer reports warnings for it.

2. **Test edge cases**: What happens with `width=1, height=INT_MAX`? Does
   the check in `allocate_rgba_buffer()` (`if (total <= 0)`) catch this?

3. **Explore multiplication chains**: Add a 5-factor multiplication. How
   does SAF handle longer chains?

4. **Cross-check with buffer overflow**: Does `check_numeric("buffer_overflow")`
   report any issues? Why or why not?

## Next Steps

This concludes the Integer Issues category. Continue to:
- [Resource Safety](../../resource-safety/) for file and lock tracking
- [Memory Safety](../../memory-safety/) for use-after-free detection
- [Buffer Overflow](../../buffer-overflow/) for spatial memory safety
