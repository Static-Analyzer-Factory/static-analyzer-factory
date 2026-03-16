# Plan 064: PTABen complex_tests Oracle Annotations

## Problem

The `complex_tests` category in PTABen (49 source files, 48 compiled `.bc` files) is **automatically skipped** because the test files contain no oracle annotations (`MUSTALIAS`, `MAYALIAS`, `NOALIAS`, etc.). Unlike `basic_c_tests` which `#include "aliascheck.h"` and call oracle macros, the complex_tests were originally designed for SVF's instrumentation-based comparison (`compare.sh` / `instrument.sh`) — not for static oracle validation.

These tests exercise important pointer analysis patterns that SAF should be evaluated against:
- Pointer swapping through function calls (context sensitivity)
- Heap allocation through wrapper chains
- Function pointers
- Recursive data structures (linked lists)
- Multi-level indirection (triple pointers)
- Path-sensitive allocation
- Global variable side effects

## Approach

Add oracle annotations (`MUSTALIAS`/`MAYALIAS`/`NOALIAS`) to each complex_test source file, recompile to `.bc`, and let the existing PTABen harness validate them. This requires:

1. **Annotating source files** — Add `#include "aliascheck.h"` and oracle calls at key program points
2. **Recompiling** — Run `clang -c -emit-llvm` inside Docker to produce new `.bc` files
3. **No harness changes** — The existing PTABen infrastructure already handles all oracle types

## Analysis of Each Test File

### Tier 1: Simple Swap Tests (should pass with current SAF)

These test basic interprocedural pointer tracking through swap functions. SAF's CI-PTA + field sensitivity should handle most.

| File | Pattern | Expected Oracles | Required Feature |
|------|---------|-----------------|------------------|
| `swap.c` | Local swap via function call | `MUSTALIAS(pa,&a)`, `MUSTALIAS(pb,&b)` (precise) or `MAYALIAS` (conservative) | Basic interprocedural PTA |
| `swap1.c` | Swap through call chain | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Context sensitivity |
| `swap-global.c` | Swap via global pointers | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Global tracking |
| `swap-global1.c` | Global swap, no params | `MAYALIAS(p1,&a)`, `MAYALIAS(p1,&b)` | Global side effects |
| `swap-global2.c` | Global swap via call chain | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Interprocedural globals |
| `swap-struct.c` | Swap struct fields | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Field sensitivity |
| `swap-struct1.c` | Swap struct via pointer | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Field + interprocedural |
| `test-globalstruct.c` | Const struct init | `MUSTALIAS(p,&g1)` | Constant init tracking |
| `test.c` | Struct cast + array | `MAYALIAS` for cast pointers | Type casting + array |
| `test1.c` | Return value chain | `MAYALIAS` for return aliases | Return value tracking |

### Tier 2: Heap + Function Pointer Tests (need context sensitivity)

| File | Pattern | Expected Oracles | Required Feature |
|------|---------|-----------------|------------------|
| `swap-heap.c` | Swap malloc'd ptrs through wrapper chain | `MAYALIAS(pa,pb)` (CI) or `NOALIAS` (CS) | Heap + malloc wrapper |
| `swap-heap1.c` | Out-param + return malloc | `MAYALIAS` for heap aliases | Param alias tracking |
| `swap-heap2.c` | Global + heap swap | `MAYALIAS` for cross-domain | Global-heap interaction |
| `swap-heap3.c` | Same as heap2 | Same | Same |
| `swap-heap4.c` | Double malloc in one fn | `NOALIAS` (CS) between mallocs | Context-sensitive heap |
| `swap-funcptr.c` | Function pointer to swap | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Function pointer resolution |
| `swap-funcptr1.c` | Registered function pointer | `MAYALIAS` | Interprocedural fn ptr |
| `swap-funcptr2.c` | Same as funcptr1 | Same | Same |
| `swap-array.c` | Array of pointers swapped | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Array + swap |

### Tier 3: Deep Context/Flow Sensitivity (challenging)

| File | Pattern | Expected Oracles | Required Feature |
|------|---------|-----------------|------------------|
| `swap4.c` | 4 sequential swaps | `MUSTALIAS` (flow-sens) or `MAYALIAS` (flow-insens) | Flow sensitivity |
| `swap4-context.c` | Deep call chain (h→f,f,g) | `MAYALIAS` | Deep context sensitivity |
| `swap4-context1.c` | 8-level call chain | `MAYALIAS` | Very deep CS (k≥4) |
| `swap4-contextindirect.c` | Context + indirection | `MAYALIAS` | CS + multi-level deref |
| `cond-swap.c` | Conditional swap | `MAYALIAS(pa,&a)`, `MAYALIAS(pa,&b)` | Path sensitivity |
| `swap-structindirect.c` | Double deref struct swap | `MAYALIAS` | Deep deref + field |
| `swap-indirect.c` | Triple pointer swap | `MAYALIAS` | 3-level deref tracking |
| `swap-indirect1.c` | Triple ptr + factored fn | `MAYALIAS` | Same + interprocedural |
| `swap-indirect2.c` | Triple ptr + nested calls | `MAYALIAS` | Same + context |
| `swap-recursion.c` | Recursive swap | `MAYALIAS` | Recursion handling |

### Tier 4: Path + Data Structure Tests (most challenging)

| File | Pattern | Expected Oracles | Required Feature |
|------|---------|-----------------|------------------|
| `test-path.c` | Path-dependent malloc | `NOALIAS(pa,pb)` (path-sens) | Path sensitivity |
| `test1-path.c` | Param-driven path | `MAYALIAS` | Path + interprocedural |
| `test2-path.c` | Conditional return value | `MAYALIAS` | Path + return tracking |
| `test3-path.c` | Path merge with alloc | `MAYALIAS(pa,pb)` | Path + heap |
| `test-indirect.c` | Multi-level deref modify | `MAYALIAS` | Deep deref + call chain |
| `test-indirect1.c` | Global via indirection | `MAYALIAS` | Multi-level + global |
| `test-cond.c` | Conditional global assign | `MAYALIAS` | Flow-sens + global |
| `test-clone.c` | Recursive array malloc | `MAYALIAS` | Array + recursion + heap |
| `test-clone1.c` | Static var + conditional malloc | `MAYALIAS` | Static + heap + recursion |
| `test2.c`-`test8.c` | Various ptr arithmetic | `MAYALIAS` | Pointer arithmetic |
| `test-recursive.c` | Linked list construction | `MAYALIAS` for list node aliases | Recursive heap |
| `test-recursive0.c`-`test-recursive2.c` | List variants | `MAYALIAS` | Recursive heap |
| `test-recursiveglobal.c`-`2.c` | Global + recursion | `MAYALIAS` | Global + recursion |
| `test-linklist.c` | List construct + destroy | `MAYALIAS` for list aliases | Heap + recursion |
| `test-linklist1.c` | Doubly-linked list + walk | `MAYALIAS` | Field-sens + recursion |

## Implementation Plan

### Phase 1: Add Oracle Annotations to Source Files (~48 files)

For each file, add:
1. `#include "aliascheck.h"` at the top
2. Oracle calls at the END of `main()` (after all pointer operations complete)
3. Use conservative `MAYALIAS` where context/flow/path sensitivity would be needed for precision
4. Use `MUSTALIAS` only where the alias is unconditional regardless of analysis precision
5. Use `NOALIAS` where pointers provably never alias (e.g., two distinct stack arrays)

**Key annotation principle**: Oracle annotations should reflect the *ground truth* of what the program actually does, not what a specific analyzer can prove. For pointer swaps:
- `swap(&p1, &p2)` unconditionally swaps → after swap, `p1` points to what `p2` pointed to → `MUSTALIAS(p1, &b)` if `p2` was `&b`
- But if we can't statically determine the swap happened (e.g., conditional swap), use `MAYALIAS`
- Two distinct stack allocations are always `NOALIAS`: `NOALIAS(&a, &b)`

**Annotation strategy per pattern:**

**Swap tests**: After swap, the pointers are exchanged. Ground truth:
```c
// Before: p1 = &a, p2 = &b
swap(&p1, &p2);
// After: p1 = &b, p2 = &a
pa = p2;  // pa = &a
pb = p1;  // pb = &b
MUSTALIAS(pa, &a);  // pa definitely points to a
MUSTALIAS(pb, &b);  // pb definitely points to b
NOALIAS(pa, pb);    // pa and pb point to different things
```

**Heap tests**: Two malloc calls return different objects:
```c
p1 = my_malloc(10);
p2 = my_malloc(20);
swap(&p1, &p2);
pa = p2; pb = p1;
// Ground truth: pa = malloc(10), pb = malloc(20)
NOALIAS(pa, pb);  // Different heap objects
```

**Multiple swaps / conditional**: May lose precision:
```c
swap(&p1, &p2, flag);  // Conditional swap
pa = p1;
// Ground truth: pa may be &a or &b depending on flag
MAYALIAS(pa, &a);
MAYALIAS(pa, &b);
```

### Phase 2: Recompile All complex_tests

Inside Docker (`make shell`):
```bash
cd tests/benchmarks/ptaben
for f in src/complex_tests/*.c; do
  name=$(basename "$f" .c)
  clang-18 -c -emit-llvm -g -O0 \
    -include aliascheck.h \
    "$f" -o ".compiled/complex_tests/${name}.bc"
done
```

### Phase 3: Validate and Adjust

Run PTABen filtered to complex_tests:
```bash
cargo run --release -p saf-bench -- ptaben \
  --compiled-dir tests/benchmarks/ptaben/.compiled \
  --filter "complex_tests/*"
```

Review results and classify:
- **Exact**: SAF matches ground truth
- **Sound**: SAF is more conservative than ground truth (MAYALIAS where MUSTALIAS expected)
- **Unsound**: SAF is less conservative (NOALIAS where MAYALIAS expected) — indicates a bug

### Phase 4: Identify and Fix SAF Gaps (if any unsound results)

Potential issues that may surface:
1. **Swap through function calls**: CI-PTA merges all contexts → may lose must-alias precision
2. **Function pointer resolution**: Need PTA to resolve `(*p)(&p1, &p2)` to `swap`
3. **Triple indirection**: Deep dereference chains may not track correctly
4. **Recursive heap structures**: Linked list nodes may collapse to single abstract object
5. **Global variable side effects**: `swap()` modifying globals needs interprocedural tracking

## Expected Results

With conservative annotations (using `MAYALIAS` for most swap results):
- **Tier 1** (~10 tests): ~8-10 Exact (basic cases SAF handles)
- **Tier 2** (~9 tests): ~5-7 Exact (heap/funcptr partially supported)
- **Tier 3** (~11 tests): ~3-6 Exact (deep CS/FS needed, SAF may be conservative)
- **Tier 4** (~18 tests): ~5-10 Exact (data structures partially supported)

**Total expected**: ~20-35 Exact out of 48 tests, with the remainder being Sound (conservative) or Skip (no applicable oracle points).

## SAF Features Needed for Full Precision

These are features that would improve results beyond conservative MAYALIAS:

1. **Strong updates for singletons** — When PTA proves a store target is a singleton, use MUSTALIAS instead of MAYALIAS for subsequent loads. SAF already has this in CS-PTA.

2. **Flow-sensitive swap tracking** — FS-PTA should track pointer state changes across swap calls. SAF has FS-PTA but may not handle interprocedural strong updates.

3. **Recursive summary widening** — For linked list tests, need to converge on a sound approximation of recursive heap structures. SAF has recursive SCC summaries.

4. **Function pointer resolution in call graph** — SAF already resolves function pointers via PTA, but complex_tests exercise this with global function pointers.

## Effort Estimate

- Phase 1 (annotations): ~48 files × ~5-15 lines each = ~400-700 lines of annotation
- Phase 2 (recompile): One compilation script, run in Docker
- Phase 3 (validate): Run benchmark, review results
- Phase 4 (fixes): Depends on findings, likely 0-300 LOC

## No Harness Changes Required

The existing PTABen harness already handles all oracle types used in the annotations. No changes to `ptaben.rs` are needed — just annotated source files and recompiled `.bc` files.
