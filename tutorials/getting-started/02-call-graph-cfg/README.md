# Tutorial 2: Call Graphs and Control Flow Graphs

This tutorial introduces two fundamental program representations:
the **call graph** and the **control flow graph (CFG)**.

## What You Will Learn

- What a call graph is and why it matters
- What a CFG is and how it represents function structure
- How to export and analyze both graphs with SAF
- Key graph concepts: entry points, leaf functions, branches, merges

## The Program

We analyze a multi-function C program that models a request-processing pipeline:

```
main
  |-- log_error          (shared error utility)
  \-- parse_request
        |-- validate_input
        |     \-- log_error
        |-- process_data
        \-- send_response
```

## What is a Call Graph?

A **call graph** is a directed graph where:
- **Nodes** represent functions
- **Edges** represent call sites (function A calls function B)

Key concepts:
- **Entry point**: function with no callers (e.g., `main`)
- **Leaf function**: function that calls no other user-defined functions
- **Fan-out**: number of functions a function calls
- **Fan-in**: number of callers a function has

## What is a CFG?

A **control flow graph (CFG)** is a directed graph for each function where:
- **Nodes** represent basic blocks (straight-line code sequences)
- **Edges** represent possible control flow between blocks

Key concepts:
- **Entry block**: where function execution begins
- **Exit block**: ends with a return (no successors within function)
- **Branch block**: has multiple successors (if/while/switch)
- **Merge block**: has multiple predecessors (join point)
- **Back-edge**: edge to an earlier block (indicates a loop)

## Run the Tutorial

```bash
cd tutorials-new/getting-started/02-call-graph-cfg
python detect.py
```

Expected output:
```
Step 1: Compiling C to LLVM IR...

Step 2: Loading project...
  Available graphs: ['cfg', 'callgraph', 'defuse', 'valueflow']

==================================================
PART A: Call Graph
==================================================

Call Graph Summary:
  Functions: 6
  Call edges: <N>

Function relationships:
  main: 0 caller(s), 2 callee(s) [ENTRY POINT]
  parse_request: 1 caller(s), 3 callee(s)
  validate_input: 1 caller(s), 1 callee(s)
  log_error: 2 caller(s), 0 callee(s) [LEAF]
  process_data: 1 caller(s), 0 callee(s) [LEAF]
  send_response: 1 caller(s), 0 callee(s) [LEAF]

==================================================
PART B: Control Flow Graph (CFG)
==================================================

CFG for 6 function(s)

  Function: main
    Basic blocks: <N>
    Entry block: ...
    Exit blocks: <N>
    Branch points: <N>
    Merge points: <N>
  ...
```

## Understanding the Code

### Exporting the Call Graph

```python
graphs = proj.graphs()
callgraph = graphs.export("callgraph")

# PropertyGraph format:
# {"schema_version": "0.1.0", "graph_type": "callgraph",
#  "nodes": [{"id": ..., "labels": ["Function"],
#             "properties": {"name": ..., "kind": ...}}, ...],
#  "edges": [{"src": ..., "dst": ..., "edge_type": "CALLS", "properties": {}}, ...]}
nodes = callgraph.get("nodes", [])
edges = callgraph.get("edges", [])

# Access function name via properties
for node in nodes:
    name = node.get("properties", {}).get("name", "")
```

### Exporting the CFG

```python
cfg = graphs.export("cfg")

# PropertyGraph format:
# {"schema_version": "0.1.0", "graph_type": "cfg",
#  "nodes": [{"id": ..., "labels": ["Block", "Entry"],
#             "properties": {"name": ..., "function": ...}}, ...],
#  "edges": [{"src": ..., "dst": ..., "edge_type": "FLOWS_TO", "properties": {}}, ...]}
#
# Group blocks by properties.function, identify entry/exit from labels,
# and derive successors from edges where src matches the block id.
cfg_nodes = cfg.get("nodes", [])
cfg_edges = cfg.get("edges", [])
```

## Why These Graphs Matter

Call graphs and CFGs are the foundation for:

1. **Reachability analysis**: Can function X be called from main?
2. **Dead code detection**: Functions with no callers may be unused
3. **Taint propagation**: Data flows follow both CFG and call graph edges
4. **Attack surface**: Entry points and functions processing external input
5. **Loop detection**: Back-edges in the CFG reveal loops

## Exercises

1. **Count back-edges**: Modify `detect.py` to detect loop back-edges by
   checking if any successor block ID appears earlier in the CFG
2. **Find hot spots**: Which function has the highest fan-in? This is often
   a shared utility worth careful analysis
3. **Visualize**: Export the call graph as DOT format and render with Graphviz

## What's Next?

Continue to [Tutorial 3: Def-Use and ValueFlow](../03-defuse-valueflow/README.md)
to learn how SAF tracks data flow within and across functions.
