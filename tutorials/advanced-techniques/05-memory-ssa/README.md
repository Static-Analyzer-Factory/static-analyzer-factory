# Tutorial: Memory SSA

## What You'll Learn

- What **Memory SSA** is and why it matters for static analysis
- How **def-use chains** for memory operations enable precise disambiguation
- How **interprocedural mod/ref summaries** track callee effects
- How to use the Memory SSA API in Python and Rust

## Prerequisites

Complete the setup instructions in the main tutorials README before starting.

## Background: Why Memory SSA?

Previous tutorials used points-to analysis (PTA) to determine *what* a pointer
can point to. Memory SSA answers a different question: **which store is
actually read by a given load?**

Consider:

```c
*p = source();   // S1: tainted data
*q = 99;         // S2: unrelated store
int x = *p;      // L1: which store does this read?
```

Without Memory SSA, an analyzer sees three memory operations and doesn't know
which store reaches L1. With PTA telling us `p -> {a}` and `q -> {b}`, Memory
SSA can determine that S2 writes to a *different* location than L1 reads, so
**L1's clobbering def is S1, not S2**.

Memory SSA extends this to interprocedural cases:

```c
*p = source();   // S1: tainted data
modify(p);       // C1: mod/ref says modify() writes to {a}
int x = *p;      // L1: clobber is C1, not S1
```

The `modify(p)` call is a **memory Def** because its mod/ref summary says it
may modify the location `{a}`. Memory SSA correctly identifies C1 as the
clobbering def for L1, meaning the load reads the value written by `modify()`,
not the original tainted value from `source()`.

## The Program

```c
extern int source(void);
extern void sink(int x);

void modify(int *p) {
    *p = 42;
}

void test(void) {
    int a, b;
    int *p = &a;
    int *q = &b;

    *p = source();   // S1: tainted store to a
    *q = 99;         // S2: store to b (unrelated)
    modify(p);       // C1: overwrites *p with 42 (kills the taint)
    int x = *p;      // L1: load from a -- clobber is C1, not S1
    sink(x);         // sink receives 42, not source() result
}
```

### Memory SSA Analysis

| Access | Kind | Location | Clobber |
|--------|------|----------|---------|
| S1 | Def (store) | loc_a | LiveOnEntry |
| S2 | Def (store) | loc_b | LiveOnEntry |
| C1 | Def (call) | loc_a | S1 |
| L1 | Use (load) | loc_a | C1 (not S1) |

The key insight: **L1 reads from C1 (the call to `modify()`), not from S1
(the tainted `source()` call)**. This means the sink receives `42`, not
tainted data. A taint analysis using Memory SSA would correctly report no
taint flow, while one without Memory SSA might false-positive.

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Memory SSA built successfully
  Total memory accesses: <N>
  Schema version: 0.1.0
  Functions with MSSA: <N>
  Functions with mod/ref: <N>

  modify() mod/ref summary:
    may_mod locations: <N>
    may_ref locations: <N>
    (modify() writes to <N> memory location(s))

  test() mod/ref summary:
    may_mod locations: <N>
    may_ref locations: <N>
    (test() transitively includes modify()'s effects)

Memory SSA analysis complete.
  The call to modify(p) is recognized as a memory Def,
  which clobbers the earlier store *p = source().
  This means the load x = *p reads from modify(),
  not from the tainted source() call.
```

## Understanding the Code

### Python API

```python
from saf import Project

proj = Project.open("vulnerable.ll")

# Build Memory SSA
mssa = proj.memory_ssa()
print(f"Memory accesses: {mssa.access_count}")

# Export to JSON
export = mssa.export()
# export = {
#   "schema_version": "0.1.0",
#   "access_count": N,
#   "functions": { func_id: { "live_on_entry": ..., "accesses": [...], "phis": {...} } },
#   "mod_ref": { func_id: { "may_mod": [...], "may_ref": [...] } }
# }

# Query mod/ref for a specific function
air = proj.air()
modify_fn = air.get_function("modify")
summary = mssa.mod_ref(modify_fn.id)
# summary = {"may_mod": ["0x..."], "may_ref": []}
```

### Rust API

```rust
use saf_analysis::mssa::MemorySsa;

let mssa = MemorySsa::build(&module, &cfgs, pta_result, &callgraph);

// Query mod/ref
if let Some(summary) = mssa.mod_ref(modify_fn.id) {
    println!("modify() may write to {} locations", summary.may_mod.len());
}

// Export to JSON
let export = mssa.export();
let json = serde_json::to_string_pretty(&export).unwrap();
```

### Key Concepts

1. **Skeleton**: Every memory instruction gets a `MemoryAccess` (Def for
   stores/calls, Use for loads). These form a single def chain per function.

2. **Phi Nodes**: At control flow join points (if/else merge, loop headers),
   Memory SSA inserts Phi nodes that merge the reaching definitions from
   each predecessor.

3. **Clobber Walker**: When asked "which Def does this Use read?", the walker
   traverses the def chain backward, consulting PTA to skip Defs that write
   to different locations.

4. **Mod/Ref Summaries**: For each function, Memory SSA computes which
   locations it may modify (write) and reference (read), including transitive
   effects through callees. This is computed bottom-up on the call graph.

## Next Steps

Memory SSA is a building block for more advanced analyses. Continue to the
**SVFG Exploration** tutorial to see how the Sparse Value-Flow Graph unifies
Memory SSA's def-use chains with value-flow edges.
