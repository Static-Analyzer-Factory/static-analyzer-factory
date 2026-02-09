# Tutorial: Pointer Aliasing and Indirect Call Resolution

## What You'll Learn

- What **points-to analysis** (PTA) is and why it matters for static analysis
- How **address-of** operations create points-to relations
- How to query **may-alias** and **no-alias** relationships between pointers
- How PTA resolves **indirect calls** through function pointers
- How resolved indirect calls appear in the **call graph**

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## Background: What Is Pointer Analysis?

Pointer analysis (PTA) is a static analysis that determines which memory
locations each pointer variable may reference at runtime. The result is a
**points-to set** for every pointer-typed value in the program.

For example, given `int *p = &x;`, PTA computes `points_to(p) = {x}`.

Two pointers **may alias** if their points-to sets overlap. Two pointers
**definitely don't alias** (no-alias) if their points-to sets are disjoint.

### Why Aliasing Matters

Alias information is critical for:

- **Taint analysis**: determining whether a tainted value can reach a sink
  through pointer indirection
- **Optimization**: deciding whether two memory accesses can be reordered
- **Bug detection**: finding use-after-free, double-free, and dangling pointer bugs

### Indirect Call Resolution

A **direct call** names the target function explicitly: `printf("hello")`.
The analyzer knows exactly which function is called.

An **indirect call** calls through a pointer: `handler(data)`. The analyzer
cannot determine the target without knowing what `handler` points to.

Without PTA, a static analyzer must either:
- **Skip** the indirect call (missing bugs behind it)
- **Assume** it can call anything (massive false positives)

PTA provides the middle ground: it resolves `handler` to its possible targets,
enabling precise analysis through indirect call sites.

## The Program

```c
int main(void) {
    int x = 10;
    int y = 20;
    int z = 30;

    int *p = &x;      // p points to x
    int *q = &x;      // q also points to x -- aliases p
    int *r = &y;      // r points to y -- does NOT alias p
    int *s = p;       // s copies p -- aliases both p and q

    *s = 42;          // modifies x through aliased pointer
    *r = 99;          // modifies y through non-aliased pointer
    return 0;
}
```

This program creates four pointers with the following alias relationships:

| Pair   | Alias? | Why |
|--------|--------|-----|
| p, q   | Yes    | Both point to `x` via address-of |
| p, s   | Yes    | `s` copies `p`, so `s` also points to `x` |
| q, s   | Yes    | Transitive: both point to `x` |
| p, r   | No     | `p` points to `x`, `r` points to `y` -- disjoint |
| q, r   | No     | `q` points to `x`, `r` points to `y` -- disjoint |

## How PTA Computes Points-To Sets

SAF implements Andersen's inclusion-based pointer analysis. The algorithm:

1. **Scans** all instructions to find pointer operations:
   - `&x` (address-of) creates a constraint: `x in points_to(p)`
   - `q = p` (copy) creates a constraint: `points_to(p) subset of points_to(q)`
   - `*p = q` (store) and `q = *p` (load) create indirect constraints
2. **Iterates** until all constraints reach a fixed point (no more changes)
3. **Produces** a points-to map: `value_id -> {location_id, ...}`

## How PTA Resolves Indirect Calls

When function pointers are used:

```c
typedef int (*handler_fn)(const char *);

handler_fn get_handler(int method) {
    if (method == 0) return handle_get;
    if (method == 1) return handle_post;
    return handle_delete;
}

void dispatch(int method, const char *data) {
    handler_fn handler = get_handler(method);
    int status = handler(data);  // indirect call
}
```

1. PTA tracks the return value of `get_handler`:
   - When `method == 0`: return value points to `handle_get`
   - When `method == 1`: return value points to `handle_post`
   - Otherwise: return value points to `handle_delete`
2. At the call site `handler(data)`, PTA looks up the points-to set of
   `handler` and finds `{handle_get, handle_post, handle_delete}`.
3. The call graph builder adds edges from `dispatch` to all three targets.

## The Pipeline

```
program.c -> clang-18 -> LLVM IR (.ll) -> LLVM frontend -> AIR -> PTA -> points-to sets
                                                                      -> call graph
```

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
PART A: Basic Pointer Aliasing
==============================
PTA Statistics:
  Values tracked: <N>
  Abstract locations: <N>
  Points-to entries: <N>

Points-to sets (first 10):
  0x1234567890abcdef... -> 1 location(s)
  ...

Alias queries:
  may_alias(0x..., 0x...) = True/False
  no_alias(0x..., 0x...) = True/False

PART B: Indirect Call Resolution
================================
Call Graph: <N> functions, <N> call edges

Functions:
  main
  ...

Call edges:
  main -> printf
  ...
```

## Key API Methods

- `proj.pta_result()` returns the PTA result object with alias queries
- `pta.value_count` / `pta.location_count` give summary statistics
- `pta.export()` returns the full points-to map as a Python dict
- `pta.may_alias(id_a, id_b)` returns `True` if the two values may point to
  overlapping locations
- `pta.no_alias(id_a, id_b)` returns `True` if the two values definitely point
  to disjoint locations
- `proj.graphs().export("callgraph")` exports the call graph including
  PTA-resolved indirect call edges

## Next Steps

Continue to the **Field Sensitivity** tutorial to see how PTA tracks individual
struct fields separately for higher precision.
