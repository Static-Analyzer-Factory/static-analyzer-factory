# Tutorial: Use-After-Free Detection (CWE-416)

## Overview

This tutorial demonstrates detecting use-after-free vulnerabilities using SAF's taint flow analysis. Use-after-free occurs when a program dereferences a pointer after the memory it points to has been freed.

## The Vulnerability

```c
#include <stdlib.h>

int main(void) {
    // Allocate heap memory
    int *ptr = (int *)malloc(sizeof(int));
    *ptr = 42;

    // Free the memory - ptr is now dangling
    free(ptr);

    // BUG: Dereference after free - undefined behavior
    int value = *ptr;

    return value;
}
```

After `free(ptr)`, the pointer `ptr` becomes "dangling" - it still holds the old address, but the memory is no longer valid. Dereferencing a dangling pointer leads to undefined behavior: data corruption, crashes, or exploitable code execution.

## Detection Technique

SAF detects use-after-free by tracking the heap pointer lifecycle:

| Component | Description |
|-----------|-------------|
| Source | `malloc()` return value (the allocated pointer) |
| Sink | `free()` argument (confirms the pointer is freed) |
| Analysis | Taint flow from malloc return to free argument |

This query confirms that a malloc-allocated pointer reaches `free()`. For full UAF detection, the checker framework also verifies that the same pointer is used AFTER the `free()` call.

## Run the Tutorial

```bash
cd tutorials-new/memory-safety/02-use-after-free
python detect.py
```

Expected output:
```
Found 1 alloc-to-free flow(s):
  [0] finding_id=<hex id>

ValueFlow graph: <N> nodes, <M> edges
```

## Key API Calls

```python
from saf import Project, sources, sinks

# Load the project
proj = Project.open("vulnerable.ll")
q = proj.query()

# Track malloc return value flowing to free's argument
findings = q.taint_flow(
    sources=sources.call("malloc"),
    sinks=sinks.call("free", arg_index=0),
)

# Inspect the ValueFlow graph structure (PropertyGraph format)
graphs = proj.graphs()
vf = graphs.export("valueflow")
print(f"ValueFlow graph: {len(vf['nodes'])} nodes, {len(vf['edges'])} edges")
```

## Understanding the Detection

1. **Taint Source**: The return value of `malloc()` - the pointer to allocated memory
2. **Taint Propagation**: Through assignments and pointer operations
3. **Taint Sink**: The first argument to `free()` - confirms deallocation
4. **Bug Pattern**: Same pointer is used after the `free()` call

The ValueFlow graph captures all data dependencies, allowing SAF to trace how the allocated pointer flows through the program to both `free()` and subsequent dereferences.

## Alternative: Checker Framework

For more comprehensive UAF detection, use the checker framework:

```python
# Use the dedicated UAF checker
uaf_findings = proj.check("use-after-free")
for f in uaf_findings:
    print(f"[{f.severity}] {f.message}")
    print(f"  Source: {f.source}")  # Allocation site
    print(f"  Sink: {f.sink}")      # Dangling use site
```

## Next Steps

- [03-double-free](../03-double-free/) - Detect freeing memory twice
- [04-typestate-memory](../04-typestate-memory/) - Track allocation lifecycle with typestate
