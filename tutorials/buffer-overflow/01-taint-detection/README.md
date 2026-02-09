# Tutorial: Heap Buffer Overflow via Taint Flow (CWE-120)

## Overview

This tutorial demonstrates detecting heap buffer overflows using taint flow analysis. By tracking heap allocations through pointer arithmetic to memory access functions, we can identify when pointer arithmetic may produce out-of-bounds addresses.

## The Vulnerability

```c
#include <stdlib.h>
#include <stdio.h>

int main(void) {
    // Allocate a small buffer (16 bytes)
    char *buf = (char *)malloc(16);

    // BUG: Pointer arithmetic goes past the allocation boundary
    char *oob_ptr = buf + 256;

    // BUG: Accessing out-of-bounds heap memory
    puts(oob_ptr);

    free(buf);
    return 0;
}
```

The pointer `buf` starts valid, but `buf + 256` points well beyond the 16-byte allocation. When `puts()` reads from this out-of-bounds pointer, it accesses memory past the buffer - a heap buffer over-read.

## Detection Technique

Taint flow analysis tracks data from sources to sinks:

| Component | Description |
|-----------|-------------|
| Source | `malloc()` return value (the base pointer of the allocation) |
| Sink | `puts()` argument (the string pointer being dereferenced) |
| Propagation | Pointer arithmetic preserves taint through transform edges |

### Transform Edges

When tainted data is modified (arithmetic, casts, string operations), SAF preserves the taint through **transform edges**. For `buf + 256`:

```
malloc() return (tainted)
    |
    | [Transform: binary_op(add)]
    v
buf + 256 (still tainted - derived from allocation)
    |
    | [CallArg]
    v
puts() argument (tainted data reaches sink)
```

## Run the Tutorial

```bash
cd tutorials-new/buffer-overflow/01-taint-detection
python detect.py
```

Expected output:
```
Step 3: Running taint flow analysis...
  Source: malloc() return value (heap base pointer)
  Sink: puts() argument (string pointer)

  Found 1 buffer overflow flow(s):

  [0] finding_id=0x...
      trace steps: N
        step 0: ...
        step 1: ...
```

## Key API Calls

```python
from saf import Project, sources, sinks

# Load the project
proj = Project.open("vulnerable.ll")
q = proj.query()

# Track malloc result flowing through arithmetic to puts
findings = q.taint_flow(
    sources=sources.call("malloc"),
    sinks=sinks.call("puts", arg_index=0),
)

# Inspect the trace to see how data flows
for f in findings:
    if f.trace:
        for step in f.trace.steps:
            print(step)  # Shows each edge in the flow

# Explore the ValueFlow graph directly (PropertyGraph format)
graphs = proj.graphs()
vf = graphs.export("valueflow")
print(f"Nodes: {len(vf['nodes'])}, Edges: {len(vf['edges'])}")

# Edge types are in the edge_type field
for edge in vf["edges"]:
    kind = edge.get("edge_type", "unknown")
```

## Trace Steps

Each trace step represents one hop in the ValueFlow graph:

| Edge Type | Description |
|-----------|-------------|
| `DefUse` | Direct data flow (assignment) |
| `Transform` | Data-modifying operation (arithmetic, cast) |
| `CallArg` | Flow into function argument |
| `Return` | Flow from function return |
| `Store` | Write to memory |
| `Load` | Read from memory |

## Limitations of Taint-Based Detection

Taint flow analysis detects that an allocation-derived pointer reaches a dangerous sink, but it does not verify the actual bounds:

| What Taint Analysis Detects | What It Does NOT Detect |
|----------------------------|-------------------------|
| Pointer flows from malloc to puts | Whether the offset exceeds allocation size |
| Transform edge exists (arithmetic) | The specific offset value (256 vs 10) |
| Data dependency chain | Whether access is actually out-of-bounds |

For precise bounds checking, use interval analysis (Tutorial 02) or combine with Z3 (Tutorial 03).

## Next Steps

- [02-interval-analysis](../02-interval-analysis/) - Precise off-by-one detection via abstract interpretation
- [03-complex-patterns](../03-complex-patterns/) - Advanced patterns with CS-PTA + Z3
