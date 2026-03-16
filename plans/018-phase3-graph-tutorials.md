# Plan 018: Phase 3 — Graph Exploration Tutorials

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8
**Prerequisite:** Plan 016 (directories restructured)

## Goal

Create 4 tutorials in `tutorials/graphs/` that teach users how to export and interpret SAF's graph representations: call graph, CFG, def-use graph, and value-flow graph.

**Audience:** Static analysis students/researchers who want to understand program structure.

## Tutorials

### graphs/01-callgraph-visualization

**Source:** ~50 lines C, 4-5 functions calling each other:
```
main → parse_request → validate_input → process_data → send_response
main → log_error (shared utility)
```

**detect.py:**
- `proj.graphs().export("callgraph")`
- Print callers/callees for each function
- Identify entry points (no incoming edges) and leaf functions (no outgoing edges)

**Teaches:** Call graph export, interpreting callgraph JSON structure (nodes = functions, edges = call sites).

### graphs/02-cfg-exploration

**Source:** ~40 lines C++, function with if/else, while loop, early return:
```cpp
int classify(int x) {
    if (x < 0) return -1;
    int count = 0;
    while (x > 0) { count++; x /= 2; }
    if (count > 10) return 2;
    return 1;
}
```

**detect.py:**
- `proj.graphs().export("cfg", function="classify")` (or equivalent)
- Print basic blocks and their edges
- Identify loop back-edges (edge where target block ID < source block ID, or appears in a cycle)

**Teaches:** CFG structure, basic blocks, branch/loop patterns in the IR.

### graphs/03-defuse-chains

**Source:** ~30 lines C:
```c
char *buf = malloc(64);   // def
strcpy(buf, input);       // use 1
log_message(buf);         // use 2
free(buf);                // use 3
```

**detect.py:**
- `proj.graphs().export("defuse")`
- For a given value, show all its uses
- Trace the def-use chain from allocation to each consumer

**Teaches:** Def-use graph export, understanding SSA value relationships.

### graphs/04-valueflow-graph

**Source:** ~60 lines C, multi-function with pointers:
- Data flows through 3 functions
- Stored to and loaded from a struct field
- Shows transform, store, load, call_arg, return edge types

**detect.py:**
- `proj.graphs().export("valueflow")`
- Count nodes/edges by kind
- Print full flow path from input to output

**Teaches:** ValueFlow graph structure, edge types (DefUse, Transform, Store, Load, CallArg, Return), interprocedural flow.

## Implementation Order

Implement tutorials sequentially: 01 → 02 → 03 → 04. Each is a safe checkpoint. If a tutorial exposes a bug in graph export, fix the bug or document it before proceeding.

## Each Tutorial Contains

- Source file (`program.c` or `program.cpp`)
- `detect.py` — Python exploration script
- `detect.rs` — Rust equivalent
- `README.md` — Concept explanation, code walkthrough, output interpretation

## Verification

For each tutorial: `python detect.py` (in Docker) produces meaningful graph output matching the README description.

## On Completion

Update `PROGRESS.md`:
- Set plan 018 status to `done`
- Update task checklist: T3, T4, T5, T6 → `done` (update individually as each is completed)
- Update "Next Steps" to point to plan 019
- Update `tutorials/graphs/README.md` from placeholder to real overview
