# Tutorial: Buffer Overflow Detection via Interval Analysis (CWE-120)

## Overview

This tutorial demonstrates detecting buffer overflows using abstract interpretation with interval analysis. Unlike taint flow (Tutorial 01), interval analysis computes numeric value ranges at every program point, enabling precise detection of off-by-one errors and other bounds violations.

## What is Interval Analysis?

Interval analysis is a form of abstract interpretation that tracks the possible range of integer values:

```
Variable i: [0, 127]  // i is between 0 and 127 (inclusive)
```

At each program point, we know the range of every integer variable. When checking buffer access `buf[i]`, we can verify:
- Is the lower bound >= 0? (no negative index)
- Is the upper bound < buffer_size? (no overflow)

## The Vulnerability

```c
#define MAX_PATH_LEN 128

// Safe version: respects buffer boundary
int parse_path_safe(const char *request, char *path_buf) {
    int i = 0;
    while (*p && *p != '?' && *p != ' ' && i < MAX_PATH_LEN - 1) {
        path_buf[i] = *p;  // i in [0, 126]
        i++;
        p++;
    }
    path_buf[i] = '\0';  // i in [0, 127] - safe
    return i;
}

// BUGGY version: off-by-one allows writing past buffer end
int parse_path_overflow(const char *request, char *path_buf) {
    int i = 0;
    // BUG: Should be i < MAX_PATH_LEN - 1, not i <= MAX_PATH_LEN
    while (*p && *p != '?' && *p != ' ' && i <= MAX_PATH_LEN) {
        path_buf[i] = *p;  // i in [0, 128] - OVERFLOW!
        i++;
        p++;
    }
    path_buf[i] = '\0';  // i can be 129 - OVERFLOW!
    return i;
}
```

The condition `i <= MAX_PATH_LEN` allows `i` to reach 128, writing past the end of a 128-byte buffer.

## How Interval Analysis Detects This

1. **Initialize**: At function entry, `i = 0`, so interval is `[0, 0]`

2. **Loop analysis with widening**:
   - First iteration: `i` in `[0, 0]`
   - After `i++`: `[1, 1]`
   - After widening at loop header: `[0, infinity]` (widened to ensure convergence)
   - After loop condition `i <= 128`: `[0, 128]`

3. **Narrowing** refines the interval using the loop condition

4. **Check at GEP**: `path_buf[i]` where `i` in `[0, 128]` and buffer size is 128
   - Upper bound 128 >= buffer size 128 -> **OVERFLOW WARNING**

## Key Concepts

### Widening

Loops can iterate indefinitely. Widening ensures the analysis terminates:

```
Iteration 1: i in [0, 0]
Iteration 2: i in [0, 1]
...widening...
After widening: i in [0, +inf]
```

### Threshold Widening

SAF extracts constants from branch conditions as widening thresholds:

```c
while (i <= MAX_PATH_LEN)  // MAX_PATH_LEN = 128 is a threshold
```

This allows widening to `[0, 128]` instead of `[0, +inf]`, improving precision.

### Narrowing

After widening, narrowing recovers precision by re-evaluating constraints:

```
After widening: [0, +inf]
After narrowing with i <= 128: [0, 128]
```

## Run the Tutorial

```bash
cd tutorials-new/buffer-overflow/02-interval-analysis
python detect.py
```

Expected output:
```
Step 3: Running abstract interpretation...
  Blocks analyzed: N
  Widening applications: M
  Narrowing iterations: K
  Converged: True

Step 4: Running buffer overflow checker (CWE-120)...
  Findings: N
  Safe GEP accesses: X
  Warnings (may overflow): Y
  Errors (definite overflow): Z

  Details:
    [WARNING] in parse_path_overflow: Index may exceed buffer size
      Interval: [0, 128]
```

## Key API Calls

```python
import saf

proj = saf.Project.open("vulnerable.ll")

# Run abstract interpretation
result = proj.abstract_interp()

# Get diagnostics
diag = result.diagnostics()
print(f"Blocks analyzed: {diag['blocks_analyzed']}")
print(f"Widening applications: {diag['widening_applications']}")
print(f"Converged: {diag['converged']}")

# Run buffer overflow checker
findings = proj.check_numeric("buffer_overflow")
for f in findings:
    print(f"[{f.severity}] in {f.function}: {f.description}")
    print(f"  Interval: {f.interval}")

# Query specific invariants
inv = result.invariant_at_block(block_id)
print(f"Variable intervals: {inv}")

# Get interval for specific value at specific instruction
interval = result.interval_at_inst(inst_id, value_id)
print(f"Value range: [{interval.lo}, {interval.hi}]")
```

## Severity Levels

| Severity | Meaning | Interval Example |
|----------|---------|------------------|
| `safe` | Index provably in bounds | `[0, 126]` for size-128 buffer |
| `warning` | Index MAY be out of bounds | `[0, 128]` (upper bound == size) |
| `error` | Index DEFINITELY out of bounds | `[129, 256]` (entire range exceeds) |

## Comparing Detection Techniques

| Technique | Detects | Precision | Cost |
|-----------|---------|-----------|------|
| Taint flow (01) | Allocation flows to sink | Low (no bounds) | Fast |
| Interval analysis (02) | Numeric bounds | High (exact ranges) | Medium |
| CS-PTA + Z3 (03) | Complex pointer flows | Highest | Slow |

## When to Use Interval Analysis

Best for:
- Off-by-one errors
- Loop-based array access
- Size calculations
- Index variables in known ranges

Limited for:
- Pointer aliasing (use PTA)
- Indirect array access (use CS-PTA)
- Path-dependent bounds (use Z3)

## Next Steps

- [03-complex-patterns](../03-complex-patterns/) - Combine CS-PTA with Z3 for complex patterns
- [../../integer-issues/](../../integer-issues/) - Use interval analysis for integer overflow detection
