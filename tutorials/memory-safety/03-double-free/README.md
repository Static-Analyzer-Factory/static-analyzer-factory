# Tutorial: Double-Free Detection (CWE-415)

## Overview

This tutorial demonstrates detecting double-free vulnerabilities using SAF's SVFG-based checker framework. A double-free occurs when `free()` is called twice on the same pointer, corrupting the heap allocator's internal data structures.

## The Vulnerability

```c
typedef struct {
    char *data;
    int size;
} Buffer;

void buffer_free(Buffer *buf) {
    if (buf) {
        free(buf->data);
        free(buf);
    }
}

int process_data(void) {
    Buffer *buf = buffer_create(256);
    if (!buf) return -1;

    strcpy(buf->data, "Hello, World!");

    // First free - correct
    buffer_free(buf);

    // BUG: Double-free - buf was already freed above
    buffer_free(buf);

    return 0;
}
```

The second call to `buffer_free(buf)` passes a pointer that was already freed. This corrupts heap metadata, leading to:
- Program crashes
- Data corruption
- Exploitable security conditions (heap exploitation)

## Detection Technique

SAF's double-free checker tracks the lifecycle of freed pointers:

| Component | Description |
|-----------|-------------|
| Source | First `free()` call (pointer becomes invalid) |
| Sink | Second `free()` call (double-free point) |
| Mode | `may_reach` - freed pointer reaching another free |
| Finding | Same pointer passed to `free()` multiple times |

## Lifecycle Tracking

The checker framework models memory lifecycle as a state machine:

```
[Allocated] --free()--> [Freed] --free()--> [ERROR: Double-Free]
                           |
                           +--use--> [ERROR: Use-After-Free]
```

Double-free detection focuses on the `[Freed] --free()--> [ERROR]` transition.

## Run the Tutorial

```bash
cd tutorials-new/memory-safety/03-double-free
python detect.py
```

Expected output:
```
Step 3: Running double-free checker (CWE-415)...
  Findings: N

  [0] Double-free: freed memory may be freed again
      Severity: error
      Checker:  double-free
      Source:   <first free call site>
      Sink:     <second free call site>
```

## Key API Calls

```python
import saf

# Load the project
proj = saf.Project.open("vulnerable.ll")

# Run the double-free checker
findings = proj.check("double-free")

for f in findings:
    print(f.message)     # Description
    print(f.source)      # First free() call
    print(f.sink)        # Second free() call (the bug)
    print(f.trace)       # Path between the two free() calls
```

## Comparing Memory Safety Checkers

| Checker | Pattern | Mode |
|---------|---------|------|
| memory-leak | alloc -> exit (no free) | must_not_reach |
| use-after-free | free -> use | may_reach |
| double-free | free -> free | may_reach |

All three checkers use the same underlying SVFG infrastructure but with different source/sink patterns and reachability modes.

## Real-World Patterns

Double-free often occurs in these scenarios:

1. **Error handling paths**: Different error handlers both call cleanup
2. **Complex ownership**: Unclear who is responsible for freeing
3. **Wrapper functions**: Like `buffer_free()` being called multiple times
4. **Conditional cleanup**: Free inside and outside a conditional

## Next Steps

- [04-typestate-memory](../04-typestate-memory/) - Track full memory lifecycle with typestate analysis
- [05-path-sensitive](../05-path-sensitive/) - Reduce false positives with Z3-based path sensitivity
