# Tutorial 1: Detecting Integer Overflow (CWE-190)

## What You'll Learn

- How integer overflow vulnerabilities occur in C programs
- How SAF's abstract interpretation framework detects overflow
- How interval analysis tracks numeric value ranges through computations
- The difference between safe, warning, and error severity levels

## Prerequisites

> Complete [SETUP.md](../../SETUP.md) before starting this tutorial.

## The Vulnerability

Integer overflow occurs when an arithmetic operation produces a result that
exceeds the maximum (or falls below the minimum) value that can be represented
in the target integer type. In C, signed integer overflow is undefined behavior,
while unsigned overflow wraps around (modular arithmetic).

### Why It Matters

Integer overflow can lead to:
- **Buffer overflows**: When an overflowed size is used for allocation
- **Logic errors**: When comparisons produce unexpected results
- **Security bypasses**: When overflow affects security checks

For example, if `width * height * bytes_per_pixel` overflows to a small value,
`malloc()` allocates too little memory, leading to a heap buffer overflow when
the full image data is written.

### Vulnerable Code

```c
/**
 * BUGGY: Compute image buffer size from dimensions.
 *
 * This function multiplies width * height * bytes_per_pixel without
 * checking for overflow. For large images (e.g., 65536 x 65536 x 4),
 * the result wraps around in 32-bit arithmetic.
 */
int compute_image_size(int width, int height, int bpp) {
    /* BUG: This multiplication can overflow for large dimensions.
     * For width=65536, height=65536, bpp=4:
     *   65536 * 65536 = 2^32 = 0 (wraps around in 32-bit)
     *   0 * 4 = 0
     * Caller then allocates 0 bytes but writes the full image! */
    int size = width * height * bpp;
    return size;
}
```

The `vulnerable.c` file also contains:
- `safe_add()`: Demonstrates bounds checking before arithmetic
- `sum_pixels()`: Loop-based accumulation that can overflow
- `allocate_image()`: Uses the buggy `compute_image_size()` result

## How SAF Detects It

SAF uses **abstract interpretation** with **interval analysis** to detect
integer overflow:

1. **Interval Abstraction**: Each integer variable is approximated by an
   interval `[min, max]`. For example, after `if (x >= 0 && x <= 100)`,
   we know `x` is in `[0, 100]`.

2. **Abstract Arithmetic**: SAF computes the "unwrapped" result of each
   arithmetic operation. For multiplication, `[a, b] * [c, d]` produces
   `[min(ac, ad, bc, bd), max(ac, ad, bc, bd)]`.

3. **Overflow Detection**: SAF checks whether the unwrapped result fits
   within the operand's signed range `[-(2^(n-1)), 2^(n-1) - 1]` for
   n-bit integers.

4. **Severity Levels**:
   - **Safe**: The interval is entirely within bounds
   - **Warning**: The interval may exceed bounds (inconclusive)
   - **Error**: The interval is entirely outside bounds (definite overflow)

### The Analysis Pipeline

```
vulnerable.c -> clang-18 -> LLVM IR -> abstract_interp() -> check_numeric("integer_overflow")
```

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
  Blocks analyzed: 15
  Widening applications: 2
  Converged: True
  Functions analyzed: 5

Step 4: Running integer overflow checker (CWE-190)...
  Findings: 8
  Warnings (may overflow): 6
  Errors (definite overflow): 0

  Details:
    [WARNING] in compute_image_size: multiplication may overflow
      Interval: [-2147483648, 2147483647]
    [WARNING] in sum_pixels: addition may overflow
      Interval: [-2147483648, 2147483647]
    ...

============================================================
SUMMARY: Integer Overflow Analysis (CWE-190)
  Arithmetic operations checked: 8
  Potential issues found: 6
  POTENTIAL INTEGER OVERFLOW DETECTED in vulnerable.c
============================================================
```

## Understanding the Code

Here is `detect.py` with a walkthrough:

```python
import subprocess
from pathlib import Path
import saf

def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # 1. Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g0",
         "-Xclang", "-disable-O0-optnone",
         "-fno-discard-value-names",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # 2. Load the compiled IR
    proj = saf.Project.open(str(llvm_ir))

    # 3. Run abstract interpretation to compute intervals
    result = proj.abstract_interp()
    diag = result.diagnostics()
    # diag contains: blocks_analyzed, widening_applications, converged, etc.

    # 4. Run the integer overflow checker
    findings = proj.check_numeric("integer_overflow")

    # 5. Analyze results by severity
    warnings = [f for f in findings if f.severity == "warning"]
    errors = [f for f in findings if f.severity == "error"]

    for f in findings:
        if f.severity != "safe":
            print(f"[{f.severity.upper()}] in {f.function}: {f.description}")
            print(f"  Interval: {f.interval}")

if __name__ == "__main__":
    main()
```

### Key API Points

- `proj.abstract_interp()`: Runs interval analysis over the entire program
- `proj.check_numeric("integer_overflow")`: Checks all arithmetic operations
- `proj.check_all_numeric()`: Runs all numeric checkers at once
- Each finding has `.severity`, `.function`, `.description`, and `.interval`

## How `safe_add()` Avoids False Positives

```c
int safe_add(int a, int b) {
    if (a >= 0 && a <= 100 && b >= 0 && b <= 100) {
        return a + b;  // SAF knows a+b is in [0, 200] - no overflow
    }
    return -1;
}
```

SAF's interval analysis tracks the constraints from the `if` condition. After
the checks, it knows `a` and `b` are both in `[0, 100]`, so `a + b` is in
`[0, 200]`, which cannot overflow a 32-bit integer. This demonstrates
**path-sensitive** analysis.

## Widening for Loops

The `sum_pixels()` function contains a loop:

```c
int sum_pixels(const unsigned char *data, int count) {
    int total = 0;
    for (int i = 0; i < count; i++) {
        total += data[i];  // Can overflow for large count
    }
    return total;
}
```

Abstract interpretation uses **widening** to ensure termination on loops.
Without widening, the analysis might iterate forever trying to compute the
exact interval. With widening, after a few iterations, the interval jumps
to `[-2147483648, 2147483647]` (full range), which triggers a warning.

## Exercises

1. **Modify the bounds**: Change `safe_add()` to allow inputs up to 10000.
   Does SAF still report it as safe?

2. **Add an overflow check**: Modify `compute_image_size()` to check for
   overflow before multiplying. Re-run the detector to see if the warning
   disappears.

3. **Explore `check_all_numeric()`**: This runs all numeric checkers. What
   other checker types does SAF provide?

## Next Steps

Continue to [Tutorial 2: Size Calculation Overflow](../02-size-calculation/README.md)
to see how integer overflow in allocation size calculations leads to buffer
overflows.
