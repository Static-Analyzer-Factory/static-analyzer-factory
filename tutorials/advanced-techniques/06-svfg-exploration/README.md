# Tutorial: Sparse Value-Flow Graph (SVFG)

## What You'll Learn

- How SAF builds a Sparse Value-Flow Graph that unifies register and memory data flow
- What direct vs indirect edges represent
- How Memory SSA clobber analysis precisely links stores to loads
- How to use the SVFG API in both Python and Rust

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## The Program

A C program with a use-after-free pattern. Memory is allocated, written to
via one pointer, freed, then read through an alias:

```c
void test(void) {
    int *buf = malloc(sizeof(int));
    int *alias = buf;          // alias points to same allocation

    int tainted = source();
    *buf = tainted;            // store tainted to heap

    free(buf);                 // free

    int leaked = *alias;       // use-after-free: read via alias
    sink(leaked);              // tainted data reaches sink
}
```

### Why SVFG Matters

Without the SVFG, SAF's ValueFlow graph uses a single "unknown memory"
node to route all memory operations. This is sound (doesn't miss flows)
but imprecise (connects unrelated stores and loads).

The SVFG replaces this with precise `value -> value` edges computed via
Memory SSA clobber analysis:

```
Without SVFG:  store_val -> [unknown_mem] -> load_result   (all stores/loads connected)
With SVFG:     store_val --IndirectDef--> load_result       (only aliasing pairs)
```

### Edge Types

| Edge Kind | Category | Description |
|-----------|----------|-------------|
| `direct_def` | Direct | SSA def-use (phi, select, copy) |
| `direct_transform` | Direct | Binary/unary/cast/GEP operand to result |
| `call_arg` | Direct | Actual argument to formal parameter |
| `return` | Direct | Callee return value to caller result |
| `indirect_def` | Indirect | Store value to load result (direct clobber) |
| `indirect_store` | Indirect | Store value to MemPhi node |
| `indirect_load` | Indirect | MemPhi node to load result |
| `phi_flow` | Indirect | MemPhi to MemPhi (nested merge) |

### How Indirect Edges Work

When a load reads from memory, Memory SSA identifies which store last
wrote to that location (the "clobber"). The SVFG creates edges based on
the clobber type:

1. **Direct clobber (store):** `store_val --IndirectDef--> load_result`
2. **Phi clobber (merge point):** `store_val --IndirectStore--> MemPhi --IndirectLoad--> load_result`
3. **Nested phi:** `MemPhi --PhiFlow--> MemPhi`

## The Pipeline

```
vulnerable.c -> clang-18 -> LLVM IR (.ll) -> LLVM frontend -> AIR
  -> DefUse graph
  -> Call graph
  -> Points-to analysis (PTA)
  -> Memory SSA (MSSA)
  -> SVFG (4-phase construction)
  -> Queries + Export
```

## Run the Detector

### Python

```bash
python3 detect.py
```

Expected output (approximate):

```
SVFG Construction Results:
  Nodes: <N>
  Edges: <N>

  Direct edges:   <N>
  Indirect edges: <N>
  MemPhi nodes:   <N>

  Export schema version: 0.1.0
  Value nodes:    <N>
  MemPhi nodes:   <N>

  Edge breakdown:
    call_arg: <N>
    direct_def: <N>
    direct_transform: <N>
    indirect_def: <N>
    return: <N>
```

### Rust

```bash
cargo run --features llvm-18 --example detect_svfg
```

## Understanding the Code

### Python (`detect.py`)

```python
from saf import Project

proj = Project.open(str(llvm_ir))
svfg = proj.svfg()

# Inspect counts and diagnostics
print(svfg.node_count, svfg.edge_count)
diag = svfg.diagnostics()

# Export to JSON-serializable dict
export = svfg.export()
for node in export["nodes"]:
    print(node["kind"], node["id"])

for edge in export["edges"]:
    print(edge["src"], "--", edge["kind"], "-->", edge["dst"])
```

Key APIs:

| Method | Returns | Description |
|--------|---------|-------------|
| `proj.svfg()` | `Svfg` | Build the SVFG |
| `svfg.node_count` | `int` | Total node count |
| `svfg.edge_count` | `int` | Total edge count |
| `svfg.diagnostics()` | `dict` | Construction statistics |
| `svfg.reachable(from_id, to_id)` | `bool` | Forward reachability check |
| `svfg.forward_reachable(from_id)` | `list[str]` | All reachable value node IDs |
| `svfg.value_flow_path(from, to)` | `list[str]` or `None` | Path between values |
| `svfg.export()` | `dict` | Full JSON export |

### Rust (`detect.rs`)

```rust
use saf_analysis::svfg::SvfgBuilder;

let svfg = SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();

// Query
let reachable = svfg.reachable(from_value, to_value);
let path = svfg.value_flow_path(from, to, 1000);

// Export
let export = svfg.export();
```

## Relationship to Other Features

| Feature | Without SVFG | With SVFG |
|---------|-------------|-----------|
| Memory flow | Single unknown_mem node | Precise store-to-load edges |
| Taint tracking | Sound but imprecise | Precise through memory |
| Leak detection | Not possible | Graph reachability on SVFG |
| UAF detection | Not possible | Graph reachability on SVFG |

## Next Steps

The SVFG is the foundation for SABER-style memory safety checkers (leak, UAF,
double-free). Continue to the **Analysis Comparison** tutorial to see how
Z3-enhanced analysis combines multiple techniques for maximum precision.
