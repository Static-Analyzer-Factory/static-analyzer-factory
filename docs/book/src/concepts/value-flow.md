# Value Flow

The **ValueFlow graph** is SAF's central data flow representation. It tracks how
values move through a program -- from where they are created to where they are
used -- across function boundaries and through memory operations.

## From Def-Use to ValueFlow

### Def-Use Chains

A **def-use chain** connects a value's definition to its uses within a single
function:

```c
int x = 10;        // definition of x
printf("%d", x);   // use of x
return x;           // another use of x
```

Def-use chains are intraprocedural (within one function) and track SSA values.

### ValueFlow Graph

The ValueFlow graph extends def-use chains with:

- **Interprocedural edges**: Data flowing across function calls (arguments and
  return values)
- **Memory modeling**: Data flowing through stores and loads, resolved by PTA
- **Transform edges**: Data modified by arithmetic, casts, or other operations

## Edge Types

| Edge Type | Meaning | Example |
|-----------|---------|---------|
| **DefUse** | SSA def-use chain (direct assignment) | `y = x` |
| **Store** | Value written to memory | `*p = x` |
| **Load** | Value read from memory | `y = *p` |
| **CallArg** | Value passed as function argument | `foo(x)` |
| **Return** | Value returned from function | `return x` |
| **Transform** | Value modified by an operation | `y = x + 1` |

## Node Types

| Node Kind | Meaning |
|-----------|---------|
| **Value** | An SSA register value |
| **Location** | A memory location (from pointer analysis) |
| **UnknownMem** | Unknown or external memory |

## Example

```c
char *buf = malloc(64);    // Value: malloc return
strcpy(buf, "hello");      // CallArg: buf -> strcpy arg 0
log_message(buf);           // CallArg: buf -> log_message arg 0
free(buf);                  // CallArg: buf -> free arg 0
```

The ValueFlow graph captures all of these flows:

```
malloc() return
    |
    +--[CallArg]--> strcpy arg 0
    +--[CallArg]--> log_message arg 0
    +--[CallArg]--> free arg 0
```

## PropertyGraph Format

```json
{
  "schema_version": "0.1.0",
  "graph_type": "valueflow",
  "metadata": {
    "node_count": 42,
    "edge_count": 58
  },
  "nodes": [
    {
      "id": "0x...",
      "labels": ["Value"],
      "properties": { "kind": "Value" }
    }
  ],
  "edges": [
    {
      "src": "0x...",
      "dst": "0x...",
      "edge_type": "DEFUSE",
      "properties": {}
    }
  ]
}
```

## Exporting with the Python SDK

```python
from saf import Project

proj = Project.open("program.ll")
graphs = proj.graphs()

# Def-use chains
defuse = graphs.export("defuse")
definitions = [e for e in defuse["edges"] if e["edge_type"] == "DEFINES"]
uses = [e for e in defuse["edges"] if e["edge_type"] == "USED_BY"]
print(f"Definitions: {len(definitions)}, Uses: {len(uses)}")

# Full ValueFlow graph
vf = graphs.export("valueflow")
print(f"Nodes: {len(vf['nodes'])}, Edges: {len(vf['edges'])}")

# Count edge types
from collections import Counter
edge_types = Counter(e["edge_type"] for e in vf["edges"])
for kind, count in edge_types.most_common():
    print(f"  {kind}: {count}")
```

## How ValueFlow Enables Analysis

The ValueFlow graph is the foundation for SAF's query capabilities:

| Analysis | How It Uses ValueFlow |
|----------|----------------------|
| **Taint flow** | BFS from source nodes to sink nodes |
| **Memory leak** | Check if allocation nodes reach exit without passing through free |
| **Use-after-free** | Check if freed pointer reaches a dereference |
| **Double free** | Check if freed pointer reaches another free |

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=taint_flow&graph=valueflow" loading="lazy"></iframe>
</div>

## Next Steps

- [Taint Analysis](taint-analysis.md) -- Querying the ValueFlow graph for vulnerabilities
- [Points-To Analysis](points-to.md) -- How PTA makes ValueFlow precise
