# PropertyGraph Format

All SAF graph exports use a unified **PropertyGraph** JSON format. This format
is shared across graph types (CFG, call graph, def-use, value-flow, SVFG, PTA)
for consistent downstream processing. SAF also exports **findings** (checker
results) and a **native PTA** format, documented below.

## Schema

```json
{
  "schema_version": "0.1.0",
  "graph_type": "<type>",
  "metadata": {},
  "nodes": [
    {
      "id": "0x...",
      "labels": ["Label1", "Label2"],
      "properties": { "key": "value" }
    }
  ],
  "edges": [
    {
      "src": "0x...",
      "dst": "0x...",
      "edge_type": "EDGE_TYPE",
      "properties": {}
    }
  ]
}
```

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | `string` | Format version (currently `"0.1.0"`) |
| `graph_type` | `string` | One of `cfg`, `callgraph`, `defuse`, `valueflow`, `svfg`, `pta` |
| `metadata` | `object` | Graph-specific metadata (e.g., node/edge counts) |
| `nodes` | `array` | List of node objects |
| `edges` | `array` | List of edge objects |

### Node Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | `string` | Deterministic hex ID (`0x` + 32 hex chars) |
| `labels` | `array[string]` | Node type labels (e.g., `["Function"]`, `["Block", "Entry"]`) |
| `properties` | `object` | Type-specific properties |

### Edge Fields

| Field | Type | Description |
|-------|------|-------------|
| `src` | `string` | Source node ID |
| `dst` | `string` | Destination node ID |
| `edge_type` | `string` | Edge type label |
| `properties` | `object` | Edge-specific properties |

## Graph Types

### Call Graph (`callgraph`)

| Element | Details |
|---------|---------|
| Node labels | `["Function"]` |
| Node properties | `name` (function name), `kind` (`"defined"` or `"external"`) |
| Edge type | `"CALLS"` |

### CFG (`cfg`)

| Element | Details |
|---------|---------|
| Node labels | `["Block"]`, optionally `["Block", "Entry"]` |
| Node properties | `name` (block name), `function` (owning function) |
| Edge type | `"FLOWS_TO"` |

### Def-Use (`defuse`)

| Element | Details |
|---------|---------|
| Node labels | `["Value"]` or `["Instruction"]` |
| Node properties | Varies |
| Edge types | `"DEFINES"`, `"USED_BY"` |

### Value Flow (`valueflow`)

| Element | Details |
|---------|---------|
| Node labels | `["Value"]`, `["Location"]`, or `["UnknownMem"]` |
| Node properties | `kind` (`"Value"`, `"Location"`, `"UnknownMem"`) |
| Edge types | `"Direct"`, `"Store"`, `"Load"`, `"CallArg"`, `"Return"`, `"Transform"` |
| Metadata | `node_count`, `edge_count` |

### SVFG (`svfg`)

The Sparse Value-Flow Graph captures both direct (top-level SSA) and indirect
(memory, via MSSA) value flows. It is the foundation for SVFG-based checkers
such as null-pointer dereference and use-after-free detectors.

| Element | Details |
|---------|---------|
| Node labels | `["Value"]` or `["MemPhi"]` |
| Node properties | `kind` (`"value"` or `"mem_phi"`) |
| Edge types | `DIRECT_DEF`, `DIRECT_TRANSFORM`, `CALL_ARG`, `RETURN`, `INDIRECT_DEF`, `INDIRECT_STORE`, `INDIRECT_LOAD`, `PHI_FLOW` |
| Metadata | `node_count`, `edge_count` |

Edge type descriptions:

| Edge type | Category | Description |
|-----------|----------|-------------|
| `DIRECT_DEF` | Direct | SSA def-use chain (including phi incoming, select, copy) |
| `DIRECT_TRANSFORM` | Direct | Binary/unary/cast/GEP operand to result |
| `CALL_ARG` | Direct | Actual argument to formal parameter |
| `RETURN` | Direct | Callee return value to caller result |
| `INDIRECT_DEF` | Indirect | Store value to load result (clobber is a store) |
| `INDIRECT_STORE` | Indirect | Store value to `MemPhi` node |
| `INDIRECT_LOAD` | Indirect | `MemPhi` node to load result |
| `PHI_FLOW` | Indirect | `MemPhi` to `MemPhi` (nested phi chaining) |

Example:

```json
{
  "schema_version": "0.1.0",
  "graph_type": "svfg",
  "metadata": { "node_count": 3, "edge_count": 2 },
  "nodes": [
    { "id": "0x00000000000000000000000000000001", "labels": ["Value"], "properties": { "kind": "value" } },
    { "id": "0x00000000000000000000000000000064", "labels": ["MemPhi"], "properties": { "kind": "mem_phi" } },
    { "id": "0x00000000000000000000000000000002", "labels": ["Value"], "properties": { "kind": "value" } }
  ],
  "edges": [
    { "src": "0x00000000000000000000000000000001", "dst": "0x00000000000000000000000000000064", "edge_type": "INDIRECT_STORE", "properties": {} },
    { "src": "0x00000000000000000000000000000064", "dst": "0x00000000000000000000000000000002", "edge_type": "INDIRECT_LOAD", "properties": {} }
  ]
}
```

> **Note:** The SVFG also has a native (non-PropertyGraph) export format with an
> `SvfgExport` schema that includes a `diagnostics` object with construction
> statistics (`direct_edge_count`, `indirect_edge_count`, `mem_phi_count`,
> `skipped_call_clobbers`, `skipped_live_on_entry`). The PropertyGraph format
> shown above is what `saf export svfg` produces.

### PTA (`pta`) — PropertyGraph Format

The points-to analysis can be exported as a PropertyGraph. Pointer values become
nodes and locations become nodes, connected by `POINTS_TO` edges.

| Element | Details |
|---------|---------|
| Node labels | `["Pointer"]` or `["Location"]` |
| Node properties | Location nodes carry `obj` (object ID hex) and optionally `path` (field path) |
| Edge type | `"POINTS_TO"` |

Example:

```json
{
  "schema_version": "0.1.0",
  "graph_type": "pta",
  "metadata": {},
  "nodes": [
    { "id": "0x00000000000000000000000000000001", "labels": ["Pointer"], "properties": {} },
    { "id": "0x00000000000000000000000000000064", "labels": ["Location"], "properties": { "obj": "0x000000000000000000000000000000c8", "path": [".0"] } }
  ],
  "edges": [
    { "src": "0x00000000000000000000000000000001", "dst": "0x00000000000000000000000000000064", "edge_type": "POINTS_TO", "properties": {} }
  ]
}
```

### PTA Native Export

The PTA also has a richer native export format (not PropertyGraph) that includes
analysis configuration, all abstract locations, and diagnostics. This is the
format returned by `PtaResult::export()` in the Rust API:

```json
{
  "schema_version": "0.1.0",
  "config": {
    "enabled": true,
    "field_sensitivity": "struct_fields(max_depth=2)",
    "max_objects": 100000,
    "max_iterations": 100
  },
  "locations": [
    {
      "id": "0x...",
      "obj": "0x...",
      "path": [".0", "[2]"]
    }
  ],
  "points_to": [
    {
      "value": "0x...",
      "locations": ["0x...", "0x..."]
    }
  ],
  "diagnostics": {
    "iterations": 12,
    "iteration_limit_hit": false,
    "collapse_warning_count": 0,
    "constraint_count": 150,
    "location_count": 45
  }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `config` | `object` | PTA configuration used for the analysis run |
| `locations` | `array` | All abstract locations with object ID and field path |
| `points_to` | `array` | Points-to sets: each entry maps a `value` to its `locations` |
| `diagnostics` | `object` | Solver statistics (iterations, limits, constraint/location counts) |

### Findings Export

The findings export (`saf export findings`) produces a JSON array of checker
findings. This is **not** a PropertyGraph -- it is a flat list of diagnostic
results from all enabled checkers.

Each finding has the following structure:

```json
[
  {
    "check": "null_deref",
    "severity": "error",
    "cwe": 476,
    "message": "Pointer may be null when dereferenced",
    "path": [
      { "location": "main:5", "event": "NULL assigned" },
      { "location": "main:10", "event": "pointer dereferenced" }
    ],
    "object": "p"
  }
]
```

| Field | Type | Description |
|-------|------|-------------|
| `check` | `string` | Name of the checker that produced this finding |
| `severity` | `string` | One of `info`, `warning`, `error`, `critical` |
| `cwe` | `number?` | CWE ID if applicable (omitted when not set) |
| `message` | `string` | Human-readable description of the issue |
| `path` | `array` | Trace events from source to sink (omitted when empty) |
| `object` | `string?` | Affected object name if applicable (omitted when not set) |

Each entry in `path` is a `PathEvent`:

| Field | Type | Description |
|-------|------|-------------|
| `location` | `string` | Source location description |
| `event` | `string` | What happened at this point |
| `state` | `string?` | Typestate label (omitted when not applicable) |

## Determinism

All PropertyGraph exports are deterministic:

- Nodes are sorted by `(node_kind, referenced_id_hex)`
- Edges are sorted by `(edge_kind, src_id_hex, dst_id_hex, label_hash_hex)`
- No timestamps or wall-clock-dependent data
- Identical inputs produce byte-identical JSON output

## Working with PropertyGraph

### Python

```python
import json
from saf import Project

proj = Project.open("program.ll")
graphs = proj.graphs()
cg = graphs.export("callgraph")

# Access nodes and edges directly
for node in cg["nodes"]:
    print(node["properties"]["name"])

for edge in cg["edges"]:
    print(f"{edge['src']} -> {edge['dst']}")

# Save to file
with open("callgraph.json", "w") as f:
    json.dump(cg, f, indent=2)
```

### jq

```bash
# List function names
jq '.nodes[] | .properties.name' callgraph.json

# Count edges by type
jq '[.edges[] | .edge_type] | group_by(.) | map({type: .[0], count: length})' graph.json

# Find high fan-out nodes
jq '[.edges[] | .src] | group_by(.) | map({node: .[0], out: length}) | sort_by(-.out) | .[0:5]' callgraph.json
```

### NetworkX

```python
import json
import networkx as nx

with open("callgraph.json") as f:
    data = json.load(f)

G = nx.DiGraph()
for node in data["nodes"]:
    G.add_node(node["id"], **node.get("properties", {}))
for edge in data["edges"]:
    G.add_edge(edge["src"], edge["dst"], edge_type=edge.get("edge_type", ""))

print(f"Nodes: {G.number_of_nodes()}")
print(f"Edges: {G.number_of_edges()}")
print(f"Components: {nx.number_weakly_connected_components(G)}")
```
