# Control Flow Graphs

A **control flow graph** (CFG) represents the possible execution paths within a
function. An **interprocedural control flow graph** (ICFG) extends this to the
whole program by connecting function calls to their targets.

## CFG Basics

A CFG is a directed graph where:

- **Nodes** are basic blocks -- straight-line sequences of instructions with no
  branches except at the end
- **Edges** represent possible control flow between blocks

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Entry block** | Where function execution begins (no predecessors) |
| **Exit block** | Ends with a return instruction (no successors) |
| **Branch block** | Has multiple successors (if/while/switch) |
| **Merge block** | Has multiple predecessors (join point after a branch) |
| **Back-edge** | Edge to an earlier block, indicating a loop |

### Example

```c
int abs(int x) {
    if (x < 0)       // Block 0: entry, branch
        x = -x;      // Block 1: negate
    return x;         // Block 2: merge, exit
}
```

The CFG for this function has three blocks:

```
[Block 0: entry] --true--> [Block 1: negate]
       |                         |
       +---false--> [Block 2: return] <--+
```

Block 2 is a merge point (two predecessors) and an exit point (return).

## ICFG

The ICFG connects CFGs across functions by adding call and return edges:

- **Call edge**: From a call instruction to the callee's entry block
- **Return edge**: From the callee's return to the instruction after the call

```
main:
  [Block 0] --call--> validate: [entry]
                                  ...
                      validate: [exit] --return--> [Block 1]
```

The ICFG enables interprocedural analysis -- tracking data flow across function
boundaries.

## SAF's PropertyGraph Format

SAF exports CFGs in the unified PropertyGraph JSON format:

```json
{
  "schema_version": "0.1.0",
  "graph_type": "cfg",
  "nodes": [
    {
      "id": "0x...",
      "labels": ["Block", "Entry"],
      "properties": {
        "name": "entry",
        "function": "main"
      }
    }
  ],
  "edges": [
    {
      "src": "0x...",
      "dst": "0x...",
      "edge_type": "FLOWS_TO",
      "properties": {}
    }
  ]
}
```

- Nodes have `labels` including `"Block"` and optionally `"Entry"` for entry blocks
- The `properties.function` field groups blocks by function
- Edges have `edge_type: "FLOWS_TO"`

## Exporting with the Python SDK

```python
from saf import Project

proj = Project.open("program.ll")
graphs = proj.graphs()
cfg = graphs.export("cfg")

# Group blocks by function
from collections import defaultdict
by_function = defaultdict(list)
for node in cfg["nodes"]:
    fn = node["properties"]["function"]
    by_function[fn].append(node)

for fn, blocks in by_function.items():
    entry = [b for b in blocks if "Entry" in b["labels"]]
    print(f"{fn}: {len(blocks)} blocks, entry={entry[0]['id']}")
```

## Why CFGs Matter

CFGs are the foundation for:

- **Reachability analysis**: Is a particular code path possible?
- **Loop detection**: Back-edges in the CFG reveal loops
- **Dominance**: Which blocks must execute before others?
- **Dead code**: Blocks with no path from entry are unreachable
- **Taint propagation**: Data flows follow control flow paths

<div class="saf-widget">
  <iframe src="../../playground/?embed=true&split=true&example=complex_cfg&graph=cfg" loading="lazy"></iframe>
</div>

## Next Steps

- [Call Graphs](callgraph.md) -- Function-level structure
- [Value Flow](value-flow.md) -- Data flow built on top of CFGs
