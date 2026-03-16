# Plan 024: Call Graph Refinement via CHA + PTA

**Epic:** E10 — Call Graph Refinement
**Status:** approved
**Created:** 2026-01-29

## Goal

Close the call graph construction gap between SAF and SVF/PhASAR by implementing:
1. **Class Hierarchy Analysis (CHA)** — resolve C++ virtual dispatch
2. **PTA-based indirect call resolution** — resolve function pointer calls
3. **Iterative CG refinement** — CHA bootstraps → PTA refines → repeat until stable

After this epic, SAF's call graph construction matches SVF/PhASAR for:
- Direct calls (already done)
- PTA-refined indirect calls (function pointers, callbacks)
- Virtual dispatch resolution (CHA + PTA)
- Iterative refinement (fixed-point CG ↔ PTA loop)

Remaining gaps (deferred): RTA (PhASAR-only feature), demand-driven PTA.

## Background

SAF currently creates `IndirectPlaceholder` nodes for indirect calls but never resolves them. Both `CallGraph::resolve_indirect()` and `ICFG::resolve_indirect()` APIs exist but are never called. The PTA constraint extractor ignores global initializers (missing vtable modeling). There is no type hierarchy extraction from LLVM IR.

## Architecture

### Iterative Refinement Algorithm

```
refine(module, config) -> RefinementResult:
  1. cha = ClassHierarchy::build(&module.type_hierarchy)
  2. cg = CallGraph::build(module)
  3. BOOTSTRAP: for each CallIndirect that matches a virtual call pattern,
     resolve via cha.resolve_virtual() → cg.resolve_indirect()
  4. reachable = compute_reachable(&cg, entry_points)
  5. LOOP (max_iterations):
     a. constraints = extract_constraints(module, reachable)
                    + extract_global_initializers(module)
     b. pta = solve(&constraints, max_iters)
     c. func_loc_map = build_function_location_map(module)
     d. for each CallIndirect in reachable:
        - targets = resolve_via_pta(site, &pta, &func_loc_map)
        - optionally intersect with CHA candidates
        - cg.resolve_indirect(site, &targets)
     e. new_reachable = compute_reachable(&cg, entry_points)
     f. if new_reachable == reachable: break (fixed point)
     g. reachable = new_reachable
  6. icfg = Icfg::build(module, &cg)
  7. return RefinementResult { cg, icfg, pta, cha, iterations, resolved_sites }
```

Convergence guarantee: monotone (only add edges/functions), finite function set. Typical: 1-3 iterations.

### New Modules

| Module | Crate | Purpose |
|--------|-------|---------|
| `air::TypeHierarchyEntry` | saf-core | AIR-level type hierarchy data |
| `llvm/cha_extract.rs` | saf-frontends | Extract vtable/typeinfo from LLVM IR globals |
| `cha.rs` | saf-analysis | Build ClassHierarchy, resolve virtual calls |
| `cg_refinement.rs` | saf-analysis | Orchestrate iterative refinement loop |
| `pta/extract.rs` additions | saf-analysis | Global initializer constraints, reachable-only extraction |
| `pta/func_location.rs` | saf-analysis | ObjId → FunctionId reverse mapping |
| Python bindings | saf-python | `refine_call_graph()`, CHA access |

### AIR Type Hierarchy Extension (saf-core)

```rust
/// A virtual method slot in a class's vtable
pub struct VirtualMethodSlot {
    pub index: usize,
    pub function: Option<FunctionId>, // None = pure virtual
}

/// Type hierarchy entry for a class/struct with virtual methods
pub struct TypeHierarchyEntry {
    pub type_name: String,              // Demangled class name
    pub base_types: Vec<String>,        // Direct base class names
    pub virtual_methods: Vec<VirtualMethodSlot>,
}

// Added to AirModule:
pub type_hierarchy: Vec<TypeHierarchyEntry>,
```

Frontend-agnostic: LLVM frontend extracts from `_ZTV*`/`_ZTI*` globals; future frontends (Clang AST, rust-analyzer) populate from their own metadata. AIR-JSON frontend supports `"type_hierarchy"` array.

### CHA Module (saf-analysis)

```rust
pub struct ClassHierarchy {
    bases: BTreeMap<String, Vec<String>>,
    subclasses: BTreeMap<String, BTreeSet<String>>,  // transitive
    vtables: BTreeMap<String, Vec<Option<FunctionId>>>,
}

impl ClassHierarchy {
    pub fn build(entries: &[TypeHierarchyEntry]) -> Self;
    pub fn resolve_virtual(&self, receiver: &str, slot: usize) -> Vec<FunctionId>;
    pub fn subclasses_of(&self, class: &str) -> &BTreeSet<String>;
    pub fn export(&self) -> serde_json::Value;
}
```

### PTA Extensions

**Function-as-location mapping:**
```rust
pub struct FunctionLocationMap {
    obj_to_func: BTreeMap<ObjId, FunctionId>,
}
```
Built by scanning `AirModule.functions` — each function has an implicit ObjId. When PTA says "value V points to location L with ObjId O", check `obj_to_func[O]` to get FunctionId.

**Global initializer constraint extraction:**
```rust
fn extract_global_initializers(
    module: &AirModule,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
)
```
Walks `module.globals`, processes `Constant::Aggregate` initializers containing function references. Generates `StoreConstraint`s modeling "vtable slot N stores function F's address".

**Reachable-only extraction:**
```rust
fn extract_constraints_reachable(
    module: &AirModule,
    reachable: &BTreeSet<FunctionId>,
    factory: &mut LocationFactory,
) -> ConstraintSet
```
Like `extract_constraints()` but only processes functions in the reachable set.

### Refinement Config and Result

```rust
pub enum EntryPointStrategy {
    AllDefined,              // All non-declaration functions
    Named(Vec<String>),      // Specific function names (e.g., ["main"])
}

pub struct RefinementConfig {
    pub max_iterations: usize,          // Default: 10
    pub entry_points: EntryPointStrategy,
    pub pta_config: PtaConfig,
    pub field_sensitivity: FieldSensitivity,
}

pub struct RefinementResult {
    pub call_graph: CallGraph,
    pub icfg: Icfg,
    pub pta_result: PtaResult,
    pub cha: Option<ClassHierarchy>,
    pub iterations: usize,
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
}
```

### Python Bindings

```python
project = saf.Project.open("program.ll")
result = project.refine_call_graph(
    entry_points="all",       # or ["main"]
    max_iterations=10,
)

result.iterations               # int
result.call_graph_export()      # dict
result.resolved_sites()         # dict: {inst_id_hex: [func_names]}
result.pta_export()             # dict

cha = result.class_hierarchy()
cha.subclasses_of("Base")       # ["Derived", ...]
cha.resolve_virtual("Base", 2)  # ["Base::method", "Derived::method"]
cha.export()                    # dict
```

## Phases

### Phase 1: AIR Type Hierarchy Extension
- Add `VirtualMethodSlot`, `TypeHierarchyEntry` to `saf-core/src/air.rs`
- Add `type_hierarchy: Vec<TypeHierarchyEntry>` to `AirModule`
- Update AIR-JSON frontend to parse `"type_hierarchy"` from JSON
- Update AIR-JSON serialization to include type hierarchy
- **Tests:** Unit: round-trip serialize/deserialize type hierarchy entries

### Phase 2: LLVM Frontend CHA Extraction
- New file `saf-frontends/src/llvm/cha_extract.rs`
- Parse `_ZTV*` globals: extract function pointers per vtable slot from aggregate initializers
- Parse `_ZTI*` globals: extract base class relationships (single/multiple inheritance)
- Demangle names using `rustc-demangle` or `cpp_demangle` crate
- Populate `AirModule.type_hierarchy` during LLVM ingestion
- **Tests:** Unit: extract CHA from compiled C++ `.ll` fixtures (single inheritance, multiple inheritance, pure virtual)

### Phase 3: CHA Analysis Module
- New file `saf-analysis/src/cha.rs`
- `ClassHierarchy::build()` from `TypeHierarchyEntry` list
- Transitive subclass computation via BFS on inverse of bases relation
- `resolve_virtual(receiver, slot)` returns union of vtable[slot] for receiver + all subclasses
- `export()` returns JSON
- **Tests:** Unit: single inheritance chain, diamond inheritance, pure virtual, empty hierarchy

### Phase 4: PTA Extensions for Indirect Call Resolution
- `FunctionLocationMap`: ObjId → FunctionId reverse mapping
- `extract_global_initializers()`: walk `AirGlobal.init` aggregate constants, generate constraints
- `extract_constraints_reachable()`: filter extraction to reachable functions only
- **Tests:** Unit: global init constraints generated correctly, function location map populated, reachable-only extraction filters correctly

### Phase 5: CG Refinement Orchestration + E2E Tests
- New file `saf-analysis/src/cg_refinement.rs`
- `RefinementConfig`, `RefinementResult` types
- `refine()` function implementing the iterative algorithm
- `compute_reachable()` using `graph_algo` BFS
- Virtual call pattern detection (identify CallIndirect sites that load from vtable GEPs)
- E2E test programs compiled in Docker:

| # | File | Lang | What it tests |
|---|------|------|---------------|
| 1 | `fptr_callback.c` | C | Function pointer callback. `getenv` to `system` through resolved fptr. |
| 2 | `virtual_dispatch.cpp` | C++ | Base pointer, derived override. CHA resolves virtual call. |
| 3 | `multi_inheritance.cpp` | C++ | Two bases, derived overrides both. Multiple vtable handling. |
| 4 | `fptr_struct.c` | C | Fptr stored in struct field. Field-sensitive PTA + resolution. |
| 5 | `iterative_resolve.c` | C | Two-level indirection requiring 2+ iterations to fully resolve. |
| 6 | `trait_object.rs` | Rust | Trait object dispatch via unsafe FFI. Rust vtable resolution. |

- Rust E2E tests: assert refined CG contains expected edges, verify iteration count, verify taint flows found post-refinement
- **Tests:** 6 E2E Rust tests + unit tests for refinement loop convergence

### Phase 6: Python Bindings + Python E2E + Tutorial
- `PyRefinementResult`: iterations, call_graph_export, resolved_sites, pta_export
- `PyCha`: subclasses_of, resolve_virtual, export
- `Project.refine_call_graph()` method
- Python E2E tests for all 6 test programs
- Tutorial: `tutorials/graphs/05-cg-refinement/` with C++ virtual dispatch example
- **Tests:** 6+ Python E2E tests + 1 tutorial with detect.py

## Test Programs (Source Code Sketches)

### 1. fptr_callback.c
```c
#include <stdlib.h>
#include <stdio.h>

void dangerous_sink(const char *cmd) { system(cmd); }

void dispatch(void (*handler)(const char *), const char *data) {
    handler(data);  // indirect call resolved via PTA
}

int main() {
    const char *input = getenv("USER_CMD");
    dispatch(dangerous_sink, input);
    return 0;
}
```

### 2. virtual_dispatch.cpp
```cpp
#include <cstdlib>

class Processor {
public:
    virtual void process(const char *data) = 0;
    virtual ~Processor() = default;
};

class UnsafeProcessor : public Processor {
public:
    void process(const char *data) override {
        system(data);  // sink
    }
};

void run_processor(Processor *p, const char *data) {
    p->process(data);  // virtual call resolved via CHA
}

int main() {
    const char *input = getenv("CMD");
    UnsafeProcessor proc;
    run_processor(&proc, input);
    return 0;
}
```

### 3. multi_inheritance.cpp
```cpp
#include <cstdlib>
#include <cstdio>

class Logger {
public:
    virtual void log(const char *msg) { puts(msg); }
    virtual ~Logger() = default;
};

class Executor {
public:
    virtual void exec(const char *cmd) = 0;
    virtual ~Executor() = default;
};

class Service : public Logger, public Executor {
public:
    void log(const char *msg) override { puts(msg); }
    void exec(const char *cmd) override { system(cmd); }
};

void run(Executor *e, const char *cmd) {
    e->exec(cmd);  // virtual call via second base
}

int main() {
    const char *input = getenv("CMD");
    Service svc;
    run(&svc, input);
    return 0;
}
```

### 4. fptr_struct.c
```c
#include <stdlib.h>

typedef void (*handler_fn)(const char *);

struct Plugin {
    handler_fn handle;
    const char *name;
};

void dangerous_handler(const char *s) { system(s); }

void invoke_plugin(struct Plugin *p, const char *data) {
    p->handle(data);  // indirect via struct field
}

int main() {
    struct Plugin p;
    p.handle = dangerous_handler;
    p.name = "danger";
    const char *input = getenv("INPUT");
    invoke_plugin(&p, input);
    return 0;
}
```

### 5. iterative_resolve.c
```c
#include <stdlib.h>

typedef void (*sink_fn)(const char *);

void final_sink(const char *s) { system(s); }

void trampoline(sink_fn fn, const char *data) {
    fn(data);  // 2nd indirect call resolved in iteration 2
}

typedef void (*dispatch_fn)(sink_fn, const char *);

void setup(dispatch_fn *out) {
    *out = trampoline;  // store function pointer
}

int main() {
    dispatch_fn f;
    setup(&f);
    // Iteration 1: resolve setup() to discover f = trampoline
    // Iteration 2: resolve f(final_sink, ...) to trampoline then final_sink
    const char *input = getenv("CMD");
    f(final_sink, input);
    return 0;
}
```

### 6. trait_object.rs
```rust
use std::os::raw::c_char;

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn system(cmd: *const c_char) -> i32;
}

trait Handler {
    fn handle(&self, data: *const c_char);
}

struct UnsafeHandler;

impl Handler for UnsafeHandler {
    fn handle(&self, data: *const c_char) {
        unsafe { system(data); }
    }
}

fn dispatch(handler: &dyn Handler, data: *const c_char) {
    handler.handle(data);  // trait object dispatch
}

fn main() {
    let handler = UnsafeHandler;
    let input = unsafe { getenv(b"CMD\0".as_ptr() as *const c_char) };
    dispatch(&handler, input);
}
```

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| CHA bootstraps before PTA | Without CHA, first PTA iteration misses code reachable only through virtual calls |
| Iterative refinement (not single-pass) | Catches transitively reachable code through multi-level indirection |
| AIR-level type hierarchy (not LLVM-specific) | Frontend-agnostic per NFR-EXT-001; other frontends can populate same structure |
| Solver re-runs from scratch each iteration | Andersen CI is fast; incremental solver adds complexity without clear benefit |
| Separate FunctionLocationMap | Clean reverse mapping without polluting PTA's Location type |
| Global initializer processing as additive pass | Existing extraction unchanged; new constraints added separately |
| BTreeMap/BTreeSet throughout | Determinism per NFR-DET-001 |
| CHA + PTA intersection for virtual calls | More precise than either alone |
