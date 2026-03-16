# Call Graphs

A **call graph** is a directed graph representing the calling relationships
between functions in a program. It answers the question: "which functions can
call which other functions?"

## Structure

- **Nodes** represent functions
- **Edges** represent call sites (function A calls function B)

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Entry point** | Function with no callers (e.g., `main`) |
| **Leaf function** | Function that calls no other user-defined functions |
| **Fan-out** | Number of distinct functions a function calls |
| **Fan-in** | Number of distinct callers a function has |
| **Strongly connected component** | Group of functions that (transitively) call each other (recursion) |

### Example

```
main
  |-- log_error          (shared utility)
  \-- parse_request
        |-- validate_input
        |     \-- log_error
        |-- process_data
        \-- send_response
```

Here, `main` is the entry point, and `log_error` has fan-in of 2 (called by both
`main` and `validate_input`). `process_data`, `send_response`, and `log_error`
are leaf functions.

## Direct vs Indirect Calls

SAF distinguishes between:

- **Direct calls**: The target function is known statically (e.g., `foo()`)
- **Indirect calls**: The target is a function pointer (e.g., `fptr()`)

For indirect calls, SAF uses points-to analysis to resolve the possible targets.
The call graph is refined as PTA discovers which functions a pointer may refer to.

## SAF's PropertyGraph Format

```json
{
  "schema_version": "0.1.0",
  "graph_type": "callgraph",
  "nodes": [
    {
      "id": "0x...",
      "labels": ["Function"],
      "properties": {
        "name": "main",
        "kind": "defined"
      }
    }
  ],
  "edges": [
    {
      "src": "0x...",
      "dst": "0x...",
      "edge_type": "CALLS",
      "properties": {}
    }
  ]
}
```

- Nodes have `labels: ["Function"]` and `properties.name` for the function name
- The `properties.kind` field indicates `"defined"` (has a body) or `"external"`
- Edges have `edge_type: "CALLS"`

## Exporting with the Python SDK

```python
from saf import Project

proj = Project.open("program.ll")
graphs = proj.graphs()
cg = graphs.export("callgraph")

nodes = cg["nodes"]
edges = cg["edges"]

# Build adjacency
callees = {}
callers = {}
for node in nodes:
    nid = node["id"]
    name = node["properties"]["name"]
    callees[nid] = []
    callers[nid] = []

for edge in edges:
    callees[edge["src"]].append(edge["dst"])
    callers[edge["dst"]].append(edge["src"])

# Find entry points and leaves
for node in nodes:
    nid = node["id"]
    name = node["properties"]["name"]
    if not callers[nid]:
        print(f"Entry point: {name}")
    if not callees[nid]:
        print(f"Leaf: {name}")
```

## Why Call Graphs Matter

Call graphs are essential for:

- **Attack surface analysis**: Entry points and functions that process external input
- **Dead code detection**: Functions with no callers may be unused
- **Interprocedural analysis**: Taint and value flow follow call edges
- **Modular analysis**: Analyze one function at a time, using summaries for callees
- **Dependency analysis**: Understanding how changes propagate through a codebase

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=indirect_call&graph=callgraph" loading="lazy"></iframe>
</div>

## Next Steps

- [Points-To Analysis](points-to.md) -- Resolving indirect call targets
- [Value Flow](value-flow.md) -- Data flow across function calls
