# Tutorial: Complex Buffer Overflow Patterns with CS-PTA + Z3

## Overview

This tutorial demonstrates detecting complex buffer overflow patterns that require combining multiple analysis techniques:
- **Context-Sensitive PTA (CS-PTA)**: Distinguishes allocations at different call sites
- **Z3 Path Sensitivity**: Filters path-dependent false positives
- **Interval Analysis**: Precise numeric bounds checking

## The Challenge

Simple buffer overflows can be detected with taint flow or interval analysis alone. But real-world code has patterns that defeat simpler analyses:

### Pattern 1: Wrapper Functions with Different Sizes

```c
DynamicBuffer *create_buffer(size_t capacity) {
    buf->data = malloc(capacity);
    buf->capacity = capacity;
    return buf;
}

void process_input(...) {
    DynamicBuffer *meta = create_buffer(32);     // Small buffer
    DynamicBuffer *content = create_buffer(1024); // Large buffer

    buffer_write(meta, input, len);  // Overflow risk if len > 31
}
```

**Problem**: Context-insensitive PTA conflates `meta->data` and `content->data` because both come from `create_buffer()`. It cannot distinguish their different capacities.

**Solution**: Context-sensitive PTA (k-CFA) creates separate contexts for `create_buffer(32)` and `create_buffer(1024)`, preserving the capacity information.

### Pattern 2: Path-Dependent Overflow

```c
int conditional_overflow(int flag, size_t size) {
    char stack_buf[64];
    char *heap_buf = malloc(128);

    if (flag) {
        // Stack buffer: overflow if size > 64
        memset(stack_buf, 'A', size);
    } else {
        // Heap buffer: safe for size < 128
        memset(heap_buf, 'B', size);
    }
}
```

**Problem**: Path-insensitive analysis reports overflow for both branches because it doesn't know which path is taken.

**Solution**: Z3 path sensitivity encodes the branch condition (`flag == 1` or `flag == 0`) and verifies feasibility of each finding's path.

## How the Combination Works

```
Source Code
    |
    v
[LLVM Frontend] -> AIR (Analysis IR)
    |
    v
[Context-Sensitive PTA (k=2)]
    |-- Distinguishes allocations by call context
    |-- meta->data: context [call1] -> capacity 32
    |-- content->data: context [call2] -> capacity 1024
    |
    v
[SVFG + Checkers]
    |-- Finds potential overflows
    |-- May include false positives (path-insensitive)
    |
    v
[Z3 Path Sensitivity]
    |-- Extracts branch guards along each finding's trace
    |-- Encodes guards as Z3 formulas
    |-- SAT -> feasible (real bug)
    |-- UNSAT -> infeasible (false positive)
    |
    v
[Interval Analysis]
    |-- Computes numeric bounds at each program point
    |-- Checks: is index in [0, buffer_size)?
    |
    v
Confirmed Findings
```

## Run the Tutorial

```bash
cd tutorials-new/buffer-overflow/03-complex-patterns
python detect.py
```

Expected output:
```
TECHNIQUE 1: Context-Insensitive PTA
====================================
Running standard numeric checkers...
  Findings: N
  [warning] buffer_overflow in buffer_write: ...

TECHNIQUE 2: Context-Sensitive PTA (k=2)
========================================
  Contexts created: M
  Iterations: K
  Converged: True

TECHNIQUE 3: Z3-Based Path Sensitivity
======================================
Running path-insensitive checkers first...
  Path-insensitive findings: N

Applying Z3 path-sensitive filtering...
  Feasible (real bugs):    X
  Infeasible (filtered):   Y
  Unknown (conservative):  Z
```

## Key API Calls

```python
import saf

proj = saf.Project.open("vulnerable.ll")

# Context-sensitive PTA
cs_result = proj.context_sensitive_pta(k=2)
diag = cs_result.diagnostics()
print(f"Contexts: {diag['contexts_created']}")

# For a specific pointer, get context-qualified points-to
contexts = cs_result.contexts_for(value_id)
for ctx in contexts:
    pts = cs_result.points_to_in_context(value_id, ctx)
    print(f"Context {ctx}: points to {pts}")

# Context-insensitive summary (union across all contexts)
pts = cs_result.points_to(value_id)

# Path-sensitive analysis
pi_findings = proj.check_all()
ps_result = proj.check_all_path_sensitive()
for f in ps_result.feasible:
    print(f"REAL BUG: {f.message}")
for f in ps_result.infeasible:
    print(f"FALSE POSITIVE: {f.message}")

# Combine with interval analysis
ai_result = proj.abstract_interp()
numeric_findings = proj.check_all_numeric()
```

## When to Use Each Technique

| Technique | Best For | Overhead |
|-----------|----------|----------|
| CI-PTA + taint flow | Direct flows, simple patterns | Low |
| Interval analysis | Loop bounds, off-by-one | Medium |
| CS-PTA (k=2) | Wrapper functions, factories | Medium-High |
| Z3 path sensitivity | Branch-dependent bugs | High |
| CS-PTA + Z3 + intervals | Complex real-world patterns | Highest |

## Choosing k for CS-PTA

| k value | Context depth | Precision | Cost |
|---------|---------------|-----------|------|
| k=1 | 1 call site | Low | Low |
| k=2 | 2 call sites | Medium | Medium |
| k=3 | 3 call sites | High | High |

For most wrapper patterns, k=2 is sufficient. Deeply nested factory patterns may need k=3.

## The Bugs in This Tutorial

1. **Off-by-one in buffer_write()**:
   ```c
   // BUG: Should be <= not <
   if (buf->size + len < buf->capacity)
   ```

2. **Path-dependent stack overflow**:
   ```c
   if (flag) {
       memset(stack_buf, 'A', user_size);  // Overflow if size > 64
   }
   ```

3. **Memory leak on error path**:
   ```c
   if (buffer_write(meta, ...) < 0) {
       printf("Error\n");
       // BUG: Don't free meta and content before returning
   }
   ```

## Next Steps

- [../../memory-safety/05-path-sensitive/](../../memory-safety/05-path-sensitive/) - More Z3 examples for memory safety
- [../../advanced-techniques/](../../advanced-techniques/) - Explore Z3-enhanced analysis across all domains
