# Tutorial: JSON Export

**Difficulty:** Beginner | **Time:** 15 minutes | **Category:** Integration

## What You Will Learn

- How to export all SAF graph types (CFG, CallGraph, DefUse, ValueFlow) to JSON
- How to compute basic graph metrics (node count, edge count, max fan-out)
- How to feed exported JSON to external tools (`jq`, `networkx`, custom dashboards)

## Prerequisites

Complete [Tutorial 01: Schema Discovery](../01-schema-discovery/README.md) before starting this one.

## Why JSON Export?

SAF's internal graph representations are optimized for analysis, but many
downstream workflows need data in a portable format:

- **Visualization**: Feed graph JSON to D3.js, Graphviz, or Cytoscape
- **Data analysis**: Load into Python `networkx` or `pandas` for custom metrics
- **CI/CD integration**: Parse with `jq` in shell scripts for pass/fail decisions
- **AI agents**: Consume graph structure as context for LLM-based reasoning
- **Archival**: Store analysis results alongside code for historical comparison

## The Program

A multi-function C program with clear call relationships and data flow:

```c
char *read_input(void) {
    char *buf = (char *)malloc(256);
    if (!buf) return NULL;
    printf("Enter data: ");
    if (!fgets(buf, 256, stdin)) {
        free(buf);
        return NULL;
    }
    return buf;
}

int validate(const char *data) {
    if (!data) return 0;
    return strlen(data) > 0 && strlen(data) < 200;
}

void process(const char *data) {
    printf("Processing: %s", data);
}

int main(void) {
    char *input = read_input();
    if (!input) return 1;
    if (validate(input)) {
        process(input);
    } else {
        printf("Invalid input\n");
    }
    free(input);
    return 0;
}
```

This program has 4 user-defined functions with different call patterns.

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Available graph types: ['cfg', 'callgraph', 'defuse', 'valueflow']

==================================================
Graph: cfg
==================================================
  Nodes: <N>
  Edges: <N>
  Max fan-out: <N> (node ...)
  Saved to: cfg.json

==================================================
Graph: callgraph
==================================================
  Nodes: 4
  Edges: <N>
  ...
```

Each graph type is exported to a separate JSON file in the tutorial directory.

## Graph Types

| Graph | Nodes | Edges | Use Case |
|-------|-------|-------|----------|
| `cfg` | Basic blocks | Control flow transitions | Loop detection, reachability |
| `callgraph` | Functions | Call sites | Attack surface, dead code |
| `defuse` | Values/instructions | Def-use chains | Data dependency tracking |
| `valueflow` | Value nodes | Value flow edges | Taint analysis, data flow |

## External Tool Integration

### Using `jq` for Command-Line Analysis

All graph types now use the unified PropertyGraph format with `nodes` and `edges` lists:

```bash
# Count functions in the call graph
jq '.nodes | length' callgraph.json

# List function names (properties.name in PropertyGraph)
jq '.nodes[] | .properties.name' callgraph.json

# Find nodes with high fan-out
jq '[.edges[] | .src] | group_by(.) | map({node: .[0], count: length}) | sort_by(-.count) | .[0]' callgraph.json
```

### Using Python `networkx`

```python
import json
import networkx as nx

with open("callgraph.json") as f:
    data = json.load(f)

G = nx.DiGraph()
for node in data["nodes"]:
    G.add_node(node["id"], name=node.get("properties", {}).get("name", ""))
for edge in data["edges"]:
    G.add_edge(edge["src"], edge["dst"], edge_type=edge.get("edge_type", ""))

print("Connected components:", nx.number_weakly_connected_components(G))
```

## Next Steps

- [Tutorial 03: SARIF Reporting](../03-sarif-reporting/README.md) - Generate standards-compliant vulnerability reports
- [Tutorial 04: Batch Scanning](../04-batch-scanning/README.md) - Scan multiple programs in different languages
