# Plan 019: Phase 4 — Pointer Analysis Tutorials

**Parent:** Plan 015 (Tutorial Reorganization)
**Epic:** E8
**Prerequisite:** Plan 016 (directories restructured)

## Goal

Create 5 tutorials in `tutorials/pta/` that teach users how to inspect and use SAF's pointer analysis results directly. Progresses from basic aliasing to complex OOP patterns to a large stress-test program.

**Audience:** Static analysis students/researchers who want to understand pointer analysis.

## Tutorials

### pta/01-pointer-aliasing

**Source:** ~40 lines C with structs and pointer fields:
```c
int x = 10, y = 20;
int *p = &x;
int *q = &x;   // aliases p
int *r = &y;   // does not alias p
```

**detect.py:**
- `proj.pta_result().points_to(ptr_id)` for each pointer
- `proj.pta_result().may_alias(p_id, q_id)` → true
- `proj.pta_result().no_alias(p_id, r_id)` → true
- Print points-to sets for each pointer

**Teaches:** What points-to sets are, alias queries, basic PTA concepts.

### pta/02-indirect-calls

**Source:** ~60 lines C, function pointer dispatch table:
```c
typedef int (*handler_fn)(const char *);
handler_fn handlers[] = { handle_get, handle_post, handle_delete };
void dispatch(int method, const char *body) {
    handlers[method](body);  // indirect call — PTA resolves targets
}
```

**detect.py:**
- Export callgraph showing PTA resolved the indirect call to 3 concrete targets
- Cross-reference `pta_result().points_to()` on the function pointer with callgraph edges

**Teaches:** How PTA resolves indirect calls, connection between PTA and call graph construction.

### pta/03-cpp-virtual-dispatch

**Source:** ~80 lines C++, class hierarchy:
```cpp
struct Shape { virtual double area() = 0; };
struct Circle : Shape { double area() override; };
struct Rect : Shape { double area() override; };
void print_area(Shape *s) { printf("%f", s->area()); }
```

**detect.py:**
- Show how PTA + vtable resolution resolves `s->area()` to both `Circle::area` and `Rect::area`
- Inspect callgraph for resolved virtual targets

**Teaches:** Virtual dispatch resolution through pointer analysis, how C++ vtables appear in LLVM IR.

**Risk:** LLVM lowers vtables to GEPs and indirect calls. PTA must resolve these — may need `fast` mode or may not resolve fully. Document what SAF can and cannot do here.

### pta/04-field-sensitive-structs

**Source:** ~120 lines C, nested structs and linked list:
```c
struct Node { int data; struct Node *next; };
// Build a 3-node list
// Show that PTA tracks: node1.next → node2, node2.next → node3
// Show that node1.data does NOT alias node1.next (field sensitivity)
```

**detect.py:**
- Query `points_to()` for struct fields separately
- Demonstrate field sensitivity vs field insensitivity
- Show `max_depth` config effect (what happens when depth is exceeded)

**Teaches:** Field-sensitive pointer analysis, how `StructFields { max_depth }` config works, what "collapse to parent" means.

**Risk:** Linked list traversal may exceed `max_depth` — tutorial should explicitly demonstrate and explain this.

### pta/05-large-cpp-program

**Source:** ~500+ lines C++, simplified plugin system:
- **Factory pattern:** `PluginRegistry` holds a map of creator functions. Plugins are registered by name, instantiated via string lookup.
- **Observer pattern:** `EventBus` dispatches events to registered listener callbacks.
- **Callbacks:** Mix of raw function pointers and virtual method calls.
- ~10 classes, ~20 functions, multiple allocation sites, deep call chains.

**detect.py:**
- Run full pipeline
- Report PTA statistics: `pta_result().value_count`, `pta_result().location_count`
- Export callgraph and count resolved indirect calls vs unresolved
- Print timing/scale information

**Teaches:** SAF at scale — can it handle a complex, pointer-heavy OOP program? What does PTA produce for a realistic codebase?

**Risk:** This is the first large program. May expose performance issues or analysis bugs. That's the point — document what works and what doesn't.

## Implementation Order

Implement sequentially: 01 → 02 → 03 → 04 → 05. Each is a safe checkpoint. Update PROGRESS.md task checklist after each one.

Tutorials 01-02 are low risk. Tutorial 03 (virtual dispatch) is medium risk. Tutorials 04-05 are higher risk and may expose bugs.

## Each Tutorial Contains

- Source file (`program.c` or `program.cpp`)
- `detect.py` — Python exploration script
- `detect.rs` — Rust equivalent
- `README.md` — Concept explanation, code walkthrough, output interpretation

## Verification

For each tutorial: `python detect.py` (in Docker) produces meaningful PTA output matching the README description.

## On Completion

Update `PROGRESS.md`:
- Set plan 019 status to `done`
- Update task checklist: T7, T8, T9, T10, T11 → `done` (update individually)
- Update "Next Steps" to point to plan 020
- Update `tutorials/pta/README.md` from placeholder to real overview
- If bugs found, document in `FUTURE.md`
