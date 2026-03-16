# Plan 066: Fix Remaining 28 Unsound Cases in basic_c_tests

## Problem Statement

28 unsound oracle checks remain across 19 test files in PTABen `basic_c_tests`.
Current results: 56 Exact, 17 Sound, 9 ToVerify, **28 Unsound**, 4 Skip (114 checks).

All 28 unsound cases report `got=Unknown` (no points-to set) or `got=Partial`/`got=May`
(imprecise set). This plan categorizes each by root cause and proposes targeted fixes.

## Full Unsound Inventory

| # | Test File | Expect | Got | Root Cause |
|---|-----------|--------|-----|------------|
| 1 | arraycopy1 | MayAlias | Unknown | RC1: Array element location |
| 2 | array-constIdx | NoAlias | Unknown | RC1: Array element location |
| 3 | array-varIdx | NoAlias | Unknown | RC1: Array element location |
| 4 | array-varIdx2 | NoAlias | Unknown | RC1: Array element location |
| 5 | array-varIdx2 | MayAlias | Unknown | RC1: Array element location |
| 6 | struct-assignment-nested (oracle 1) | MayAlias | Unknown | RC2: Nested struct memcpy |
| 7 | struct-assignment-nested (oracle 2) | MayAlias | Unknown | RC2: Nested struct memcpy |
| 8 | struct-nested-array2 (oracle 1) | MayAlias | Unknown | RC2: Nested struct memcpy |
| 9 | struct-nested-array2 (oracle 2) | MayAlias | Unknown | RC2: Nested struct memcpy |
| 10 | struct-instance-return | NoAlias | Unknown | RC3: Struct return via sret |
| 11 | funptr-nested-struct-simple (oracle 1) | MayAlias | Unknown | RC4: Memcpy + indirect call |
| 12 | funptr-nested-struct-simple (oracle 2) | MayAlias | Unknown | RC4: Memcpy + indirect call |
| 13 | funptr-nested-struct (oracle 1) | MayAlias | Unknown | RC4: Memcpy + indirect call |
| 14 | funptr-nested-struct (oracle 2) | MayAlias | Unknown | RC4: Memcpy + indirect call |
| 15 | global-call-struct | MayAlias | Unknown | RC5: Double-pointer interprocedural |
| 16 | heap-linkedlist | NoAlias | Unknown | RC6: Heap linked list interprocedural |
| 17 | ptr-dereference1 | MustAlias | Partial | RC7: Flow-insensitive accumulation |
| 18 | ptr-dereference3 | MustAlias | Partial | RC7: Flow-insensitive accumulation |
| 19 | spec-equake (oracle 1) | NoAlias | Unknown | RC8: Multi-level heap array |
| 20 | spec-equake (oracle 2) | NoAlias | Unknown | RC8: Multi-level heap array |
| 21 | spec-equake (oracle 3) | NoAlias | Unknown | RC8: Multi-level heap array |
| 22 | spec-gap | MayAlias | Unknown | RC9: Pointer arithmetic + cast |
| 23 | spec-mesa | NoAlias | May | RC10: Function pointer table dispatch |
| 24 | spec-parser | NoAlias | Unknown | RC9: Pointer arithmetic + cast |
| 25 | spec-vortex | NoAlias | Unknown | RC9: Pointer arithmetic + cast |
| 26 | struct-incompab-typecast-nested (1) | MayAlias | Unknown | RC11: Type-incompatible cast |
| 27 | struct-incompab-typecast-nested (2) | MayAlias | Unknown | RC11: Type-incompatible cast |
| 28 | struct-incompab-typecast-nested (3) | NoAlias | Unknown | RC11: Type-incompatible cast |

## Root Cause Analysis

### RC1: Array Element Location Not Pre-Created (5 cases)

**Tests:** arraycopy1, array-constIdx, array-varIdx, array-varIdx2

**Pattern (arraycopy1.c):**
```c
int a, b;
int* source[2] = {&a, &b};   // Array initializer via GEP+Store
int* x = source[1];           // Load from array element
MAYALIAS(&a, x);              // x should alias &b (or &a conservatively)
```

**LLVM IR:**
```llvm
%4 = alloca [2 x ptr]                              ; source array
%6 = getelementptr [2 x ptr], ptr %4, i64 0, i64 0 ; &source[0]
store ptr %2, ptr %6                                ; source[0] = &a
%7 = getelementptr ptr, ptr %6, i64 1               ; &source[1]  ← GEP from element, not array!
store ptr %3, ptr %7                                ; source[1] = &b
%8 = getelementptr [2 x ptr], ptr %4, i64 0, i64 1 ; &source[1] (for load)
%9 = load ptr, ptr %8                               ; x = source[1]
```

**Root cause:** The LLVM IR has two different GEP patterns accessing the same element:
- Init: `GEP [2 x ptr], %4, 0, 0` then `GEP ptr, %6, 1` (pointer arithmetic from element 0)
- Load: `GEP [2 x ptr], %4, 0, 1` (direct array index)

The first GEP chain creates `source[0]` location, then pointer-increments to reach `source[1]`.
The pointer arithmetic `GEP ptr, %6, 1` is treated as a GEP on the element pointer, not
as accessing `source[1]`. The `precompute_indexed_locations` creates a location for
`source[0,1]` from the load GEP, but the store goes through a different path (pointer
arithmetic on element 0). The two paths create different LocIds for the same memory,
so the Store and Load don't connect.

**For array-constIdx/varIdx:** Similar issue — `%struct.MyStruct` array with GEP indices.
The array GEP `[2 x %struct.MyStruct], ptr, i64 0, i64 N, i32 F` should create locations
for each `(N, F)` combination. If index N is a variable, `IndexSensitivity::ConstantOnly`
collapses all indices to Unknown, losing the distinction between `s[0].f1` and `s[1].f2`.

**Fix:** Two sub-issues:
1. **Pointer arithmetic GEP collapsing:** When a GEP base is itself a GEP result
   (chained GEPs), the solver should compose the paths. Currently the second GEP
   (`GEP ptr, %6, 1`) creates a path relative to `%6`'s location (which is `source[0]`),
   but the path step `[1]` should mean "advance 1 element from source[0]" = `source[1]`.
   The solver needs to merge the base GEP's path with the new GEP's offset.

2. **Array element pre-creation:** `precompute_indexed_locations` should create locations
   for ALL constant-indexed array elements that appear in any GEP, not just the direct
   GEP targets. When `source` is `[2 x ptr]`, we need locations for both `source[0]`
   and `source[1]`.

### RC2: Nested Struct Memcpy — Fields Not Propagated (4 cases)

**Tests:** struct-assignment-nested, struct-nested-array2

**Pattern (struct-assignment-nested.c):**
```c
struct ArrayStruct { char out2; MidArrayStruct out3; int* out4; };
ArrayStruct s1, s2;
s1.out4 = &x;
s1.out3.mid2[1].in1[1] = &y;
s2 = s1;  // ← Compiled as memcpy
MUSTALIAS(s2.out4, &x);     // ← passes (Copy covers base-level fields)
MAYALIAS(s2.out3.mid2[1].in1[1], &y);  // ← FAILS: deeply nested field
```

**Root cause:** The `s2 = s1` becomes `memcpy(&s2, &s1, size)`, modeled as
`Copy(s2_ptr, s1_ptr)`. This makes `s2_ptr.pts = s1_ptr.pts`, so `s2_ptr` and `s1_ptr`
point to the SAME locations (the alloca for `s1`). Subsequent GEPs on `s2` resolve
correctly to `s1`'s field locations because `s2_ptr` now points to `s1`'s object.

But the MUSTALIAS oracle on `s2.out4` passes while the MAYALIAS on
`s2.out3.mid2[1].in1[1]` fails. The issue is that the deeply nested path
`out3.mid2[1].in1[1]` exceeds the field depth or the array-index-within-struct
GEP chain is not correctly composed. The struct has 3 levels of nesting with
array indices at each level, producing a path of depth 5+.

**Fix:** Increase `max_depth` from 6 to 8+ for the PTABen harness, and ensure
`precompute_indexed_locations` creates locations for all nested array-in-struct
GEP chains. The GEP chain for `s2.out3.mid2[1].in1[1]` is:
1. GEP %struct.ArrayStruct, 0, 1 → out3
2. GEP %struct.MidArrayStruct, 0, 1, 1 → mid2[1]
3. GEP %struct.InnerArrayStruct, 0, 0, 1 → in1[1]

This is 5 path steps. At -O0, LLVM may split these into separate GEPs that chain,
producing even more steps. The current max_depth=6 should suffice for 5 steps, but
the chained GEP composition may lose intermediate steps.

### RC3: Struct Returned by Value via sret (1 case)

**Test:** struct-instance-return

**Pattern:**
```c
struct MyStruct foo() { struct MyStruct m; m.f1 = &x; return m; }
int main() {
    struct MyStruct m = foo();
    NOALIAS(m.f1, &y);  // ← FAILS: m.f1 is Unknown
}
```

**Root cause:** In LLVM IR at -O0, struct returns are lowered to an `sret` parameter:
```llvm
define void @foo(ptr noalias sret(%struct.MyStruct) %0) {
    ; stores into %0 (the sret pointer)
}
call void @foo(ptr sret(%struct.MyStruct) %m)  ; caller passes stack pointer
```

The sret parameter is the caller's stack slot passed by pointer. SAF needs to:
1. Recognize `sret` attribute on the first parameter
2. Generate interprocedural constraints linking the sret pointer to the caller's alloca
3. Ensure field stores inside `foo()` are visible through the caller's pointer

Currently, `extract_call_constraints` may not handle `sret` parameters correctly —
the first argument is the struct pointer, but standard arg→param Copy constraints
don't model that the callee writes through this pointer to the caller's memory.

**Fix:** When CG refinement resolves `foo` as the target, generate constraints that
link the sret actual argument to the formal parameter. The callee's Stores through
the sret parameter should naturally propagate back. The issue is likely that `foo`
is not in the call graph (it's a direct call with sret, which should work), or the
sret pointer's field locations aren't pre-created.

### RC4: Memcpy + Indirect Call Chain (4 cases)

**Tests:** funptr-nested-struct-simple, funptr-nested-struct

**Pattern (funptr-nested-struct-simple.c):**
```c
struct interesting { int dummy; void (*f1)(int*); void (*f2)(int*); };
struct nested_ptr { int dummy; struct interesting* ptr; };
struct interesting i1 = {0, f1, f2};
struct nested_ptr n1 = {0, &i1};

void test_ptr() {
    struct interesting local;
    memcpy(&local, n1.ptr, sizeof(struct interesting));
    local.f1(&g);   // indirect call through copied function pointer
    local.f2(&g);
}
```

**Root cause:** Chain of dependencies:
1. Global `i1` has aggregate initializer `{0, @f1, @f2}` → needs `extract_global_initializers`
2. Global `n1` has aggregate initializer `{0, @i1}` → needs GlobalRef in aggregates
3. `n1.ptr` is loaded via `GEP + Load` on constant global address
4. `memcpy(&local, n1.ptr, size)` → Copy(local_ptr, loaded_ptr)
5. `GEP local, 0, 1` + `Load` → gets function pointer from local.f1
6. `CallIndirect %fn(&g)` → needs CG refinement to resolve to f1/f2
7. Inside `f1(int* param)`: `MAYALIAS(param, &g)` needs arg→param constraint

This is a cascade: global initializer → memcpy → indirect call → interprocedural.
The global initializer must put function pointers `@f1`, `@f2` into `i1`'s fields.
Then memcpy copies `i1`'s field contents into `local`. Then the indirect call through
`local.f1` must resolve to the actual function `f1`.

The current Copy constraint for memcpy makes `local_ptr` point to `i1`'s locations.
GEP on `local` should then find `i1`'s field locations. But the global aggregate
initializer for `i1` creates Store constraints with field paths — these paths must
match the GEP paths used when reading `local.f1`.

**Fix:** Ensure global initializer field paths are consistent with GEP field paths.
The aggregate `{0, @f1, @f2}` should create Store constraints at field indices 0, 1, 2.
GEP `%struct.interesting, 0, 1` accesses field index 1 (f1). If the aggregate uses
flat indices (0, 1, 2) while GEP uses struct field indices (0, 1, 2), they should match.
Investigate whether `extract_aggregate_elements` creates the same `FieldPath` as
the GEP extraction.

### RC5: Double-Pointer Interprocedural (1 case)

**Test:** global-call-struct

**Pattern:**
```c
struct MyStruct global = {"abcdefg", 20, &x};
void foo(int** pp, int** qq) { *pp = &a; *qq = &b; }
void bar(int** pp, int** qq) { *pp = &x; *qq = &x; }
int main() {
    int *p, *q;
    bar(&p, &q);
    MAYALIAS(p, q);           // ← passes
    MAYALIAS(global.f2, *qq); // ← FAILS: global struct field
}
```

**Root cause:** `global.f2` is a pointer field in a global struct with aggregate
initializer `{"abcdefg", 20, &x}`. The third element `&x` is stored at field index 2.
`*qq` is the result of dereferencing `qq` which was set to `&q` and then `q` was set
to `&x` inside `bar()`. So `*qq = *(&q) = q = &x`, and `global.f2 = &x`.

The issue is that `global.f2` has `Unknown` points-to set. The global struct initializer
`{"abcdefg", 20, &x}` needs to generate a Store constraint for the third field.
Currently `extract_aggregate_elements` handles `Constant::Int` matching function IDs
and `Constant::GlobalRef`. A plain `&x` in C becomes `@x` in LLVM IR, which should be
a `Constant::GlobalRef` in AIR. If it's not being captured, the initializer processing
is incomplete.

**Fix:** Check whether `&x` in the global initializer is represented as `Constant::GlobalRef`
in AIR. If it's a raw integer (address), it won't match the GlobalRef handler. May need
to handle additional constant patterns in `extract_aggregate_elements`.

### RC6: Heap Linked List Interprocedural (1 case)

**Test:** heap-linkedlist

**Pattern:**
```c
struct Node { int *data; struct Node *next; };
void malloc_list(struct Node* head) {
    for (int i = 0; i < 3; i++) {
        struct Node* n = malloc(sizeof(struct Node));
        n->data = malloc(sizeof(int));
        n->next = head->next;
        head->next = n;
    }
}
int main() {
    struct Node head; head.next = NULL;
    malloc_list(&head);
    int* p_data1 = head.next->data;
    int* p_next = head.next->next;
    NOALIAS(p_next, p_data1);  // ← FAILS: Unknown
}
```

**Root cause:** Interprocedural constraint propagation through `malloc_list`. The
function takes `head` by pointer and modifies it. The caller needs to see the
modifications through the pointer. At -O0, the parameter is stored to an alloca
and all accesses go through load/store chains. The function stores to `head->next`
and allocates heap nodes. For the oracle, `p_data1 = head.next->data` and
`p_next = head.next->next` are different fields of the same heap node, so they
should be NoAlias.

The issue is that the heap allocations inside `malloc_list` create abstract locations,
but the field stores (`n->data`, `n->next`) go through GEPs on the malloc result.
The caller's `head.next->data` dereferences the pointer chain. This requires:
1. Interprocedural arg→param constraints for `&head` → `head_param`
2. Store constraints for `head_param->next = n`
3. The caller's load `head.next` must resolve to the same locations

With CG refinement and reachable function analysis, `malloc_list` should be in scope.
The likely issue is that -O0 load/store chains lose the pointer identity across
function boundaries.

**Fix:** This is a fundamental limitation of CI-PTA with -O0 code. The alloca→store→load
chains inside `malloc_list` require flow sensitivity to track that `head->next` was
updated. CS-PTA should handle this if `malloc_list` is in the call graph. Investigate
whether CS-PTA returns a result or falls back to CI.

### RC7: Flow-Insensitive Accumulation → Partial (2 cases)

**Tests:** ptr-dereference1, ptr-dereference3

**Pattern (ptr-dereference1.c):**
```c
int *s, *r, *x, **y, t, z, k;
s = &t; r = &z;
y = &r;
s = r;                // s now points to z (kills previous &t)
MUSTALIAS(s, &z);     // ← CI-PTA reports Partial (s → {t, z}, &z → {z})
```

**Root cause:** CI-PTA merges across all program points. `s = &t` creates `s → {loc(t)}`,
then `s = r` adds `s → {loc(z)}`. Final: `s → {loc(t), loc(z)}`.
`&z → {loc(z)}`. Comparison: `{loc(t), loc(z)}` vs `{loc(z)}` = Partial (not Must).

This is a well-known limitation of context-insensitive, flow-insensitive PTA.
FS-PTA should kill the first assignment and produce `s → {loc(z)}` at the oracle point.

**Fix:** The combined CS-PTA + FS-PTA in the harness should handle this. Investigate
whether FS-PTA returns `Unknown` (causing CI-PTA fallback) or if the combined result
correctly computes Must but the fallback overrides it. The issue may be that the combined
PTA queries FS-PTA first, gets Unknown (FS-PTA uses different ValueIds), then falls
back to CI-PTA which reports Partial.

### RC8: Multi-Level Heap Array (3 cases)

**Test:** spec-equake

**Pattern:**
```c
double ***disp = (double***)malloc(...);
for (i...) disp[i] = (double**)malloc(...);
for (i...) for (j...) disp[i][j] = (double*)malloc(...);
// Similarly for K, v
NOALIAS(disp, K);           // Different globals → should be NoAlias
NOALIAS(disp[1], K[Anext]); // Array elements of different allocations
NOALIAS(disp[1][col], v[i]); // Deeply nested array elements
```

**Root cause:** Three-level pointer arrays with heap allocation in loops. Each
`disp[i]` is a separate `malloc` result, and `disp[i][j]` is another. The oracles
check that elements from different top-level arrays don't alias.

The issue is that `disp` and `K` are global variables with the same pattern: allocated
via malloc, then array-indexed. The global variables themselves (`@disp`, `@K`) should
have different base objects, so `disp` vs `K` at the top level should be NoAlias
(different globals). But the oracle arguments may be the loaded values (the malloc
results stored in the globals), not the global addresses themselves.

At -O0, `disp` is accessed via: `load @disp_alloca` → gives the `malloc` result.
Two different `malloc` calls create different heap objects, so they should be NoAlias.
The issue is likely that multiple `malloc` calls inside the same function share the
same instruction (loop body), creating a single abstract heap location for all
iterations. So `disp[i]` and `K[j]` may both collapse to the same abstract location.

**Fix:** Heap allocation site abstraction. In loops, each `malloc` call creates one
abstract location per call site. `disp`'s inner `malloc` and `K`'s inner `malloc`
are different call sites, so they should be different objects. But the variable-index
array stores `disp[i] = malloc_result` and loads `disp[1]` may not connect through
the collapsed array index. This requires array index tracking for heap-allocated arrays,
which is beyond current field sensitivity (operates on struct fields, not heap-pointer
arrays).

**Classification:** Hard — requires heap array index sensitivity, likely deferred.

### RC9: Pointer Arithmetic + Complex Casts (3 cases)

**Tests:** spec-gap, spec-parser, spec-vortex

These are complex SPEC benchmark patterns with:
- Pointer arithmetic (`ptr + offset`)
- Type casting between incompatible pointer types
- Multi-level indirection through heap-allocated handle structures
- Custom allocator wrappers (spec-parser's `xalloc`)

**Classification:** Hard — requires pointer arithmetic modeling, type-cast-aware
field mapping, or custom allocator specifications. Likely deferred.

### RC10: Function Pointer Table Dispatch Imprecision (1 case)

**Test:** spec-mesa

**Pattern:**
```c
struct api_table { void (*Begin)(); void (*End)(); void (*Render)(); };
struct context { struct api_table API; struct api_table Exec; };
// ctx->Exec.Render = render_vb; ctx->API = ctx->Exec;
// Then: ctx->API.Render(x, y) dispatches to render_vb
NOALIAS(&x, &y);  // got=May (over-approximate due to dispatch imprecision)
```

**Root cause:** The oracle expects NoAlias between two distinct stack variables, but
SAF reports May. This is not a "no points-to set" problem — SAF has results but they
are too conservative. The function pointer dispatch through `ctx->API.Render` resolves
to a function that takes pointer parameters. If CHA or PTA resolves the indirect call
to multiple possible targets, the parameter aliasing becomes over-approximate.

**Fix:** This is a precision issue in function pointer resolution, not a missing
constraint. The memcpy `ctx->API = ctx->Exec` copies the function pointer table.
If the Copy constraint makes `API`'s function pointers alias with `Exec`'s (correct),
then indirect calls through `API.Render` should resolve to the same targets as
`Exec.Render`. The May result suggests the resolved call has too many targets,
causing parameter flow from unrelated functions to pollute the alias analysis.

### RC11: Type-Incompatible Struct Cast (3 cases)

**Test:** struct-incompab-typecast-nested

**Pattern:**
```c
struct SrcStruct { int* f1[10]; int* f2[10]; InnerStruct f3[5]; int* f4; };
struct DstStruct { int* f1[10]; int* f2[20]; InnerStruct f3[5]; };
SrcStruct* psrc = &src;
DstStruct* pdst = (DstStruct*)psrc;
MAYALIAS(pdst->f1[9], &x);  // ← field access through cast pointer
```

**Root cause:** `pdst` and `psrc` point to the same memory, but through different type
lenses. The field paths are different: `DstStruct.f1[9]` has a different byte offset
than `SrcStruct.f1[9]` only if the struct layouts differ before field f1. In this case,
`f1` is the first field in both, so `pdst->f1[9]` and `psrc->f1[9]` access the same memory.

The issue is that `pdst = (DstStruct*)psrc` is a bitcast in LLVM IR. The GEP on `pdst`
uses `DstStruct` field indices, which don't match `SrcStruct` field indices in the PTA's
field path representation. Since PTA uses struct-index-based field paths (not byte-offset
based), the same memory location gets different field paths depending on which type is used,
and they don't connect.

**Classification:** Hard — requires byte-offset-based field sensitivity or type-cast-aware
field path remapping. Deferred.

---

## Fix Categories by Difficulty

### Fixable (14 cases, ~180 LOC)

| Phase | Root Cause | Cases | Approach |
|-------|-----------|-------|----------|
| A | RC1: Array element location | 5 | GEP chain composition + array element pre-creation |
| B | RC2: Nested struct memcpy | 4 | Increase max_depth + GEP chain composition |
| C | RC3: Struct return via sret | 1 | sret parameter handling in interprocedural constraints |
| D | RC4: Memcpy + indirect call | 4 | Fix global initializer field path consistency |

### Investigatable (4 cases, ~50 LOC)

| Phase | Root Cause | Cases | Approach |
|-------|-----------|-------|----------|
| E | RC5: Double-pointer interprocedural | 1 | Debug global struct initializer constraint |
| F | RC7: Flow-insensitive → Partial | 2 | Investigate FS-PTA fallback |
| G | RC10: FP table dispatch | 1 | Debug CHA resolution precision |

### Deferred (10 cases)

| Root Cause | Cases | Reason |
|-----------|-------|--------|
| RC6: Heap linked list | 1 | -O0 alloca chain + interprocedural flow sensitivity |
| RC8: Multi-level heap array | 3 | Heap array index sensitivity needed |
| RC9: Pointer arithmetic + cast | 3 | Pointer arithmetic modeling |
| RC11: Type-incompatible cast | 3 | Byte-offset field sensitivity needed |

---

## Phase A: Array Element Location Pre-Creation (RC1)

### Task A1: Chained GEP Path Composition (~60 LOC)

**Files:** `crates/saf-analysis/src/pta/solver.rs`, `crates/saf-analysis/src/pta/context.rs`

**Problem:** When LLVM generates chained GEPs like:
```llvm
%6 = GEP [2 x ptr], %4, 0, 0    ; base of array
%7 = GEP ptr, %6, 1               ; pointer arithmetic: element 1
```

The second GEP's base (`%6`) already has a location from the first GEP.
The second GEP adds path `[1]` relative to element 0, which should compose
to `[0, 1]` (array base, element 1) or resolve to the same location as
`GEP [2 x ptr], %4, 0, 1`.

**Approach:** In `precompute_indexed_locations`, when processing a GEP constraint
whose `src_ptr` is itself the `dst` of another GEP constraint, compose the paths:
1. Build a map: `value → (base_obj, cumulative_path)` from GEP chains
2. For chained GEPs, extend the base GEP's path with the new GEP's path
3. Pre-create locations for all composed paths

This is partially implemented in Phase 2 of `precompute_indexed_locations` (lines
226-249 in context.rs). Verify it handles the arraycopy1 pattern and fix if not.

### Task A2: Array Element Location for All Constant Indices (~40 LOC)

**File:** `crates/saf-analysis/src/pta/context.rs`

**Problem:** For `int* source[2] = {&a, &b}`, the array has 2 elements. The initializer
stores to `source[0]` and `source[1]` via two separate GEP+Store sequences. The load
uses `GEP ..., i64 1`. If the store's GEP chain doesn't create a location at the same
path as the load's GEP, the Store and Load don't connect.

**Approach:** After Phase 1 of `precompute_indexed_locations`, for each array-type GEP
(where the base type is an array), also pre-create locations for elements 0..N where N
is any constant index seen in GEPs on that object. This ensures both the store path and
load path find the same pre-existing location.

### Task A3: Verify with arraycopy1, array-constIdx tests

Run PTABen filtered to these tests and confirm they pass.

**Estimated LOC:** ~100

---

## Phase B: Nested Struct Depth + GEP Composition (RC2)

### Task B1: Increase max_depth for PTABen (~5 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

Change `max_depth: 6` to `max_depth: 10` in the PTABen PTA config. The deeply nested
struct tests (struct-assignment-nested, struct-nested-array2) have paths of depth 5-7.
With memcpy aliasing the dst to the src object, the GEP chain from the dst traverses
the src's field hierarchy, which can be 7+ steps deep.

### Task B2: Verify Nested Field Location Creation

Check that `precompute_indexed_locations` creates locations for the full GEP chains
in struct-assignment-nested. If paths like `[1, 1, 1, 0, 1]` (out3.mid2[1].in1[1])
are being truncated at max_depth, increasing the depth should fix it.

**Estimated LOC:** ~5

---

## Phase C: Struct Return via sret (RC3)

### Task C1: Investigate sret Parameter Handling (~30 LOC)

**File:** `crates/saf-frontends/src/llvm/mapping.rs`

At -O0, LLVM lowers struct returns to `sret` parameters:
```llvm
define void @foo(ptr noalias sret(%struct.MyStruct) %0)
call void @foo(ptr sret(%struct.MyStruct) %result_alloca)
```

Check how the LLVM frontend translates this:
1. Does it recognize `sret` as a special parameter attribute?
2. Is the sret pointer argument included in the AIR call's operands?
3. Are the stores inside `foo` to `%0` visible as writes to the caller's memory?

If the sret parameter is just treated as a regular pointer argument with regular
arg→param Copy constraints, the callee's stores through the sret pointer should
propagate to the caller's alloca via Store constraints. This should work with
standard CI-PTA interprocedural handling. Debug why it doesn't.

**Estimated LOC:** ~30

---

## Phase D: Global Initializer Field Path Consistency (RC4)

### Task D1: Verify Aggregate Field Path Matches GEP Path (~40 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs`

For global `i1 = {0, @f1, @f2}` of type `%struct.interesting = { i32, ptr, ptr }`:
- `extract_aggregate_elements` should create Store at field paths [0], [1], [2]
- GEP `%struct.interesting, ptr, i32 0, i32 1` accesses field 1

Check that the field paths from aggregate extraction match the field paths from GEP
extraction. The aggregate uses flat element indices (0, 1, 2) while the GEP uses
struct field indices. For a flat struct these are identical, but for nested structs
with arrays they may diverge.

### Task D2: Debug funptr-nested-struct-simple End-to-End

Add diagnostic output to trace:
1. What constraints are generated for global `i1` initialization
2. What the Copy constraint from memcpy produces
3. Whether the GEP on `local.f1` finds the correct field location
4. Whether the indirect call resolves to `f1`/`f2`

**Estimated LOC:** ~40

---

## Phase E: Debug Global Struct Initializer (RC5)

### Task E1: Check Aggregate Initializer for global-call-struct (~20 LOC)

The global `MyStruct global = {"abcdefg", 20, &x}` has a pointer field `&x`.
Verify that `extract_aggregate_elements` processes this correctly:
1. Element 0: `"abcdefg"` — string constant, no pointer
2. Element 1: `20` — integer, no pointer
3. Element 2: `&x` — should be `Constant::GlobalRef(@x)`

If element 2 is not a GlobalRef (e.g., it's a raw integer or a different constant
type), the Store constraint for the pointer field won't be generated.

**Estimated LOC:** ~20

---

## Phase F: Flow-Sensitive PTA for ptr-dereference (RC7)

### Task F1: Debug Combined PTA Fallback (~30 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

The combined CS-PTA + FS-PTA should handle `ptr-dereference1`. Add debug logging:
1. Does CS-PTA return a result for the oracle ValueIds?
2. Does FS-PTA return a result?
3. Which result is used for the alias query?

If FS-PTA returns `Unknown` (ValueIds not in its scope), the fallback to CI-PTA
explains the Partial result. The fix would be to ensure FS-PTA covers the oracle
ValueIds or to improve the fallback logic.

**Estimated LOC:** ~30

---

## Implementation Order

```
Phase A (array elements)          ← 5 cases, highest impact
  ↓
Phase D (global init field paths) ← 4 cases, cascading fix for funptr
  ↓
Phase B (nested struct depth)     ← 4 cases, small change
  ↓
Phase E (global struct debug)     ← 1 case, diagnostic
  ↓
Phase C (sret struct return)      ← 1 case
  ↓
Phase F (FS-PTA fallback)         ← 2 cases
```

## Expected Results

| Phase | Cases Fixed | Cumulative Exact |
|-------|-----------|-----------------|
| Baseline | — | 56 |
| A | 5 | 61 |
| D | 4 | 65 |
| B | 4 | 69 |
| E | 1 | 70 |
| C | 1 | 71 |
| F | 2 | 73 |
| **Total fixable** | **17** | **73** |

Remaining 11 unsound after all phases:
- spec-equake (3) — heap array index sensitivity
- spec-gap, spec-parser, spec-vortex (3) — pointer arithmetic
- struct-incompab-typecast-nested (3) — type-incompatible cast
- heap-linkedlist (1) — interprocedural flow sensitivity
- spec-mesa (1) — function pointer dispatch precision

## Total Estimated LOC: ~225
