# Tutorial: Context-Sensitive PTA, Virtual Dispatch, and Call Graph Refinement

## What You'll Learn

- How **context-sensitive PTA (k-CFA)** distinguishes allocations across call sites
- Why context-insensitive analysis produces spurious aliases for factory functions
- How **Class Hierarchy Analysis (CHA)** resolves C++ virtual calls
- How **call graph refinement** iteratively improves precision using PTA
- Comparing k=1 vs k=2 context sensitivity trade-offs

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## Background

### The Factory Function Problem

Consider a factory function that allocates objects:

```c
struct Pair *make_pair(int x, int y) {
    struct Pair *p = malloc(sizeof(struct Pair));
    p->x = x;
    p->y = y;
    return p;
}

int main(void) {
    struct Pair *a = make_pair(0, 0);  // call site 1
    struct Pair *b = make_pair(1, 1);  // call site 2
    // Do a and b alias?
}
```

**Context-insensitive (Andersen):** Merges all call sites to `make_pair()`. The
`malloc` inside returns a single abstract object, so `a` and `b` both point to
it. Result: `may_alias(a, b) = true` (false positive).

**Context-sensitive (k=1):** Each call site gets its own context. The `malloc`
in context `[site1]` is distinct from `malloc` in context `[site2]`. Result:
`may_alias(a, b) = false` (correct).

### Virtual Dispatch and CHA

C++ virtual calls are lowered to indirect calls through vtables:

```cpp
class Shape { virtual double area() = 0; };
class Circle : public Shape { double area() override; };
class Rectangle : public Shape { double area() override; };

void compute(Shape *s) {
    s->area();  // indirect call through vtable
}
```

**Class Hierarchy Analysis (CHA)** extracts the inheritance relationships from
vtable/typeinfo globals in LLVM IR. For a virtual call `s->area()`:

1. Determine the declared type of `s` (Shape*)
2. Find all subclasses of Shape (Circle, Rectangle)
3. Resolve to the overriding methods in each subclass

### Call Graph Refinement

CHA provides an initial approximation, but PTA can further refine it:

1. **CHA bootstrap**: Resolve virtual calls using type hierarchy
2. **PTA loop**: Use points-to information to resolve remaining indirect calls
3. **Iterate**: New call edges may expose new code, triggering more analysis
4. **Fixed point**: Stop when no new edges are discovered

## The Program

The tutorial program (`program.c`) demonstrates the factory pattern:

```c
struct Pair *make_pair(int x, int y) {
    struct Pair *p = malloc(sizeof(struct Pair));
    p->x = x;
    p->y = y;
    return p;
}

int main(void) {
    struct Pair *a = make_pair(0, 0);  // Call site 1
    struct Pair *b = make_pair(1, 1);  // Call site 2

    // CI PTA: a and b may alias (same abstract malloc)
    // CS PTA: a and b don't alias (different contexts)

    print_pair(a);
    print_pair(b);
    free(a);
    free(b);
    return 0;
}
```

## Run the Detector

```bash
python3 detect.py
```

## Expected Output

```
PART A: Context-Insensitive PTA (Andersen)
==========================================
  Values tracked: <N>
  Abstract locations: <N>
  Points-to entries: <N>

PART B: Context-Sensitive PTA (k=1)
===================================
  Iterations: <N>
  Converged: True
  Unique contexts: <N>
  ...

PART C: Context-Sensitive PTA (k=2)
===================================
  Iterations: <N>
  Unique contexts: <N>
  (k=2 >= k=1 contexts: True)

PART D: Call Graph Refinement (CHA + PTA)
=========================================
  Refinement iterations: <N>
  Call graph nodes: <N>
  Call graph edges: <N>
  ...

Summary
=======
  Andersen (context-insensitive):
    - Merges all call sites to factory functions
    - All returned pointers may alias each other

  k=1 Context-Sensitive:
    - Each call site gets its own context
    - Pointers from different sites don't alias

  Call Graph Refinement:
    - CHA bootstraps virtual call resolution
    - PTA refines indirect call targets
```

## Algorithm Details

### k-CFA Context Sensitivity

SAF implements k-call-site-sensitive pointer analysis:

- **Context** = sequence of up to k call site IDs
- **Push**: When entering a function, append current call site to context
- **Truncate**: If context exceeds k elements, drop the oldest
- **SCC collapse**: Recursive functions share the empty context (prevents blowup)

| k | Precision | Cost | Use Case |
|---|-----------|------|----------|
| 1 | Medium    | Low  | Most programs |
| 2 | High      | Medium | Wrappers calling wrappers |
| 3+ | Very high | High | Deep factory chains |

### CHA + PTA Refinement Loop

```
1. Build initial call graph (direct calls only)
2. Extract class hierarchy from _ZTV*/_ZTI* globals
3. CHA: resolve virtual calls → add edges
4. PTA: resolve indirect calls → add edges
5. If new edges added, goto 4
6. Return refined call graph
```

## API Reference

### Python

```python
# Context-insensitive PTA
pta = proj.pta_result()

# Context-sensitive PTA
cs1 = proj.context_sensitive_pta(k=1)
cs2 = proj.context_sensitive_pta(k=2)

# Diagnostics
diag = cs1.diagnostics()
print(diag["context_count"], diag["iterations"])

# Export
export = cs1.export()  # includes "contexts" and "ci_summary"

# Call graph refinement
result = proj.refine_call_graph(entry_points="main", max_iterations=10)
cg = result.call_graph_export()
cha = result.class_hierarchy()
sites = result.resolved_sites()
```

### Rust

```rust
use saf_analysis::cspta::{CsPtaConfig, CsSolver};
use saf_analysis::cg_refinement::{refine, RefinementConfig};

// Context-sensitive PTA
let config = CsPtaConfig { k: 1, ..Default::default() };
let cs_result = CsSolver::new(&module, &pta, &cg, config).solve();

// Call graph refinement
let refine_config = RefinementConfig { max_iterations: 10, ..Default::default() };
let result = refine(&module, refine_config);
```

## Trade-offs

| Aspect | CI (Andersen) | CS (k=1) | CS (k=2) |
|--------|---------------|----------|----------|
| Precision | Low | Medium | High |
| Memory | O(V * L) | O(V * L * C) | O(V * L * C^2) |
| Time | Fast | Medium | Slower |
| False positives | Many | Fewer | Fewest |

Where V = values, L = locations, C = call sites.

## Next Steps

Continue to the **Memory SSA** tutorial to see how precise store-to-load
disambiguation enables advanced analyses like SVFG and flow-sensitive PTA.
