# Tutorial 3: Def-Use Chains and ValueFlow

This tutorial introduces data flow analysis: tracking how values move
through your program from where they are created to where they are used.

## What You Will Learn

- What def-use chains are and how they work
- What the ValueFlow graph is and why it matters
- Different types of data flow edges
- How these graphs enable taint analysis

## The Program

We analyze a program where a single `malloc` allocation flows to multiple uses:

```c
int main(void) {
    // Definition: allocate buffer
    char *buf = (char *)malloc(64);

    // Use 1: write data
    strcpy(buf, "hello world");

    // Use 2: pass to function
    log_message(buf);

    // Use 3: print directly
    printf("Data: %s\n", buf);

    // Use 4: free
    free(buf);

    return 0;
}
```

## What are Def-Use Chains?

A **def-use chain** connects:
- **Definition**: where a value is created (e.g., `malloc` return)
- **Uses**: where that value is read (e.g., passed to `strcpy`, `printf`, `free`)

This is intraprocedural (within one function) and tracks SSA values.

## What is the ValueFlow Graph?

The **ValueFlow graph** extends def-use chains with:
- **Interprocedural edges**: data flowing across function calls
- **Memory modeling**: tracking data through stores and loads
- **Pointer analysis**: understanding where pointers point

### Edge Types

| Edge Kind | Meaning |
|-----------|---------|
| Direct | Value assigned directly (SSA copy) |
| Store | Value written to memory |
| Load | Value read from memory |
| CallArg | Value passed as function argument |
| Return | Value returned from function |

### Node Types

| Node Kind | Meaning |
|-----------|---------|
| Value | An SSA register value |
| Location | A memory location (from pointer analysis) |
| UnknownMem | Unknown or external memory |

## Run the Tutorial

```bash
cd tutorials-new/getting-started/03-defuse-valueflow
python detect.py
```

Expected output:
```
Step 1: Compiling C to LLVM IR...

Step 2: Loading project...

==================================================
PART A: Def-Use Chains
==================================================

Def-Use Summary:
  Definitions: <N>
  Uses: <N>

Values with multiple uses: <N>
  (These are values that flow to multiple places)
    ...: 4 uses

Defined but unused: <N> values

==================================================
PART B: ValueFlow Graph
==================================================

ValueFlow Summary:
  Nodes: <N>
  Edges: <N>

Edge types (data flow mechanisms):
    CallArg: <N> - value passed as function argument
    Direct: <N> - value assigned directly
    Load: <N> - value read from memory
    Store: <N> - value written to memory
    ...
```

## Understanding the Code

### Exporting Def-Use Chains

```python
defuse = graphs.export("defuse")

# PropertyGraph format:
# {"schema_version": "0.1.0", "graph_type": "defuse",
#  "nodes": [{"id": ..., "labels": ["Value"] or ["Instruction"], "properties": {...}}, ...],
#  "edges": [{"src": ..., "dst": ..., "edge_type": "DEFINES" or "USED_BY", "properties": {}}, ...]}
defuse_edges = defuse.get("edges", [])
definitions = [e for e in defuse_edges if e.get("edge_type") == "DEFINES"]
uses = [e for e in defuse_edges if e.get("edge_type") == "USED_BY"]
```

### Exporting the ValueFlow Graph

```python
vf = graphs.export("valueflow")

# PropertyGraph format:
# {"schema_version": "0.1.0", "graph_type": "valueflow",
#  "metadata": {"node_count": N, "edge_count": N},
#  "nodes": [{"id": ..., "labels": [...], "properties": {"kind": ...}}, ...],
#  "edges": [{"src": ..., "dst": ..., "edge_type": "Direct"|"Store"|..., "properties": {}}, ...]}
nodes = vf.get("nodes", [])
edges = vf.get("edges", [])
```

## Why These Graphs Matter

Data flow graphs are the foundation for:

1. **Taint analysis**: Follow edges from tainted sources to sinks
2. **Constant propagation**: Track constant values through the program
3. **Dead code elimination**: Find values that are never used
4. **Use-after-free detection**: Track freed pointers to later uses
5. **Information flow**: Ensure sensitive data does not leak

## The `buf` Variable Journey

In our example program, the `buf` pointer flows through:

```
malloc() return value
    |
    v
buf = ... (Direct assignment)
    |
    +--> strcpy(buf, ...) (CallArg to arg 0)
    |
    +--> log_message(buf) (CallArg to arg 0)
    |         |
    |         v
    |    printf(..., buf) inside log_message (nested CallArg)
    |
    +--> printf(..., buf) (CallArg to arg 1)
    |
    +--> free(buf) (CallArg to arg 0)
```

The ValueFlow graph captures all of these flows, enabling SAF to answer
questions like "does user input reach this free() call?"

## Exercises

1. **Find the malloc flow**: Modify `detect.py` to find all edges where
   the source node contains "malloc" - these show where allocated memory flows
2. **Dead value analysis**: Find all defined values with zero uses
3. **Cross-function flow**: Count how many `CallArg` edges exist vs `Direct` edges

## What's Next?

Continue to [Tutorial 4: Your First Checker](../04-your-first-checker/README.md)
to learn how SAF's checker framework uses these graphs to find bugs.
