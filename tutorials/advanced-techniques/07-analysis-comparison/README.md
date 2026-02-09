# Tutorial: Z3 Analysis Comparison

## What You'll Learn

- How **path-insensitive** analysis provides broad coverage but may over-report
- How **path-sensitive Z3 refinement** filters infeasible false positives
- How **Z3 assertion proving** statically verifies invariants
- Comparing precision vs. cost trade-offs across analysis techniques

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## Background

### The False Positive Problem

Path-insensitive analysis computes a conservative approximation: if a bug
*might* exist on *any* path, it reports it. This leads to false positives
when correlated branches guard both the source and sink:

```c
void process(int flag) {
    char *buf = NULL;
    if (flag) buf = malloc(1024);
    // ... processing ...
    if (flag) free(buf);  // FP: path-insensitive sees malloc without free
}
```

Path-insensitive analysis reports a memory leak because it doesn't track that
`malloc` and `free` are guarded by the same condition.

### Z3 Path Refinement

Z3 refinement adds a second stage:

1. **Stage 1**: Run path-insensitive analysis (fast, may over-report)
2. **Stage 2**: For each finding, extract branch guards along the trace
3. **Stage 3**: Encode guards as Z3 formula, check satisfiability
4. **Classification**:
   - **UNSAT** = infeasible path = false positive (filter it)
   - **SAT** = feasible path = confirmed bug (keep it)
   - **UNKNOWN** = Z3 timed out = keep conservatively

### Z3 Assertion Proving

For `assert(cond)` calls, Z3 can prove whether the condition always holds:

1. Extract dominating branch guards at the assertion location
2. Encode `guards AND NOT(cond)` as Z3 formula
3. If UNSAT: assertion is **proven** (condition always holds)
4. If SAT: assertion **may fail** (Z3 provides counterexample)

## The Program

`vulnerable.c` exercises four analysis categories, each with one false positive
(guarded by correlated branches) and one genuine bug:

### Section 1: Memory Safety

```c
void process_memory(int use_buffer) {
    char *buf = NULL;
    if (use_buffer) buf = malloc(1024);
    // ...
    if (use_buffer) free(buf);  // FP: same guard
}

void process_memory_buggy(int mode) {
    char *buf = malloc(512);
    if (mode == 1) return;  // BUG: forgot to free
    free(buf);
}
```

### Section 2: Typestate (File I/O)

```c
void process_file(const char *path, int validate) {
    FILE *fp = NULL;
    if (validate) fp = fopen(path, "r");
    // ...
    if (validate) fclose(fp);  // FP: same guard
}

void process_file_buggy(const char *path) {
    FILE *fp = fopen(path, "r");
    if (fgets(...) == NULL) return;  // BUG: file leak
    fclose(fp);
}
```

### Section 3: Numeric (Buffer Access)

```c
void process_array(int *data, int size) {
    if (size > 0 && size <= 100) {
        for (int i = 0; i < size; i++)
            data[i] = i * 2;  // FP: guarded by size check
    }
}

int process_multiply(int a, int b) {
    return a * b;  // BUG: unchecked overflow
}
```

### Section 4: Taint (Command Injection)

```c
void process_command(int trusted) {
    char *input = getenv("USER_INPUT");
    if (trusted) {
        // sanitize input
        system(sanitized);  // FP: sanitized on trusted path
    }
}

void process_command_buggy(void) {
    char *input = getenv("USER_INPUT");
    system(input);  // BUG: unsanitized
}
```

## Run the Detector

```bash
python3 detect.py
```

## Expected Output

```
PART A: Path-Insensitive Analysis (Baseline)
============================================
Total findings: 8+
  [warning] leak: ...
  [warning] file_leak: ...
  ...

PART B: Path-Sensitive Analysis (Z3-Refined)
============================================
Confirmed bugs: ~4
  ...
Filtered (false positives): ~4
  ...

PART C: Z3 Assertion Proving
============================
  Total assertions: <N>
  Proven: <N>
  May fail: <N>

PART D: Comparison Table
========================
  Category                       PI     PS  Saved
  ------------------------------ ------ ------ ------
  leak                              2      1      1
  file_leak                         2      1      1
  ...
  ------------------------------ ------ ------ ------
  TOTAL                             8      4      4

CONCLUSION
==========
  Z3 refinement filtered 4 false positive(s)
  out of 8 total finding(s) (50% reduction).
  4 confirmed bug(s) remain for developer review.
```

## Analysis Techniques Compared

| Technique | Coverage | Precision | Cost | Use Case |
|-----------|----------|-----------|------|----------|
| Path-insensitive | High (sound) | Low (FPs) | Fast | Initial triage |
| Path-sensitive Z3 | Same | High | Medium | Reducing FPs |
| Assertion proving | N/A | High | Medium | Verifying invariants |
| Flow-sensitive PTA | Same | Higher | Slow | Pointer precision |
| Context-sensitive PTA | Same | Higher | Slower | Factory patterns |

## API Reference

### Python

```python
# Path-insensitive analysis
findings = proj.check_all()

# Path-sensitive analysis with Z3
ps_result = proj.check_all_path_sensitive(z3_timeout_ms=2000, max_guards=64)
print(ps_result.feasible)    # confirmed bugs
print(ps_result.infeasible)  # filtered false positives
print(ps_result.unknown)     # Z3 timed out

# Assertion proving
result = proj.prove_assertions(z3_timeout_ms=2000, max_guards=64)
print(result.proven)     # always true
print(result.may_fail)   # may fail (with counterexample)
print(result.unknown)    # Z3 timed out
```

## When to Use Each Technique

1. **Start with path-insensitive**: Fast, catches obvious bugs
2. **Apply Z3 refinement**: When FP rate is too high for triage
3. **Use assertion proving**: To verify critical invariants
4. **Enable flow/context sensitivity**: When pointer precision matters

## Limitations

- Z3 refinement adds overhead (one solver call per finding)
- Complex guards may cause Z3 timeouts
- Some false positives cannot be filtered (non-branch-related)
- Assertion proving requires `assert()` calls in source

## Summary

This tutorial demonstrates the power of combining multiple analysis techniques:

- **Broad coverage** from path-insensitive reachability
- **High precision** from Z3 path-sensitive refinement
- **Static verification** from Z3 assertion proving

The key insight: no single technique is best for all cases. SAF provides a
toolkit of analyses that can be combined based on your precision/performance
requirements.
