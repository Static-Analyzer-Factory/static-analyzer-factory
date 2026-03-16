# Plan 063: C++ Alias Precision — Fix basic_cpp_tests Unsound Cases

## Problem Statement

54 unsound oracle checks across 12+ test files in PTABen `basic_cpp_tests`.
Five distinct root causes identified, ordered by impact.

## Root Cause Summary

| # | Root Cause | Cases | Pattern | Key Files |
|---|-----------|-------|---------|-----------|
| RC1 | CHA resolves ALL types at a slot | ~30 | MustAlias→Partial, NoAlias→Must | cg_refinement.rs |
| RC2 | Multi-inheritance vtable parsing | ~15 | MustAlias→Partial, NoAlias→Partial | cha_extract.rs |
| RC3 | Destructor member field tracking | 4 | MustAlias→Unknown | cha_extract.rs, cspta |
| RC4 | Function pointer class members | 4 | Unknown | cg_refinement.rs |
| RC5 | Global aggregate array tracking | 2 | Unknown | pta/extract.rs |

## Phase A: Fix Multi-Inheritance Vtable Parsing (RC2)

**Why first:** This is a correctness bug in the frontend. Wrong vtable slots
propagate errors through ALL downstream analyses. Must fix before RC1.

### Task A1: Per-Subobject Vtable Parsing (~150 LOC)

**File:** `crates/saf-frontends/src/llvm/cha_extract.rs`

**Current behavior:** `extract_vtable_slots()` calls
`extract_function_pointers_from_constant()` which recursively flattens
struct fields into a single list, then skips the first 2 entries globally.

**Problem:** Multi-inheritance vtables are `{ [N x ptr], [M x ptr] }` structs
with per-subobject metadata (offset-to-top + RTTI) in each sub-array.
Flattening loses the boundary; only the first pair gets skipped.

**Fix:**

1. In `extract_vtable_slots()`, detect if the vtable initializer is a
   StructValue with multiple ArrayValue fields (multi-inheritance).

2. If single array: current logic (skip 2, extract rest).

3. If struct of arrays: process ONLY the first sub-array (primary vtable).
   The secondary sub-arrays are for base-class subobject thunks — they
   contain the same methods as the primary vtable (possibly with this-pointer
   adjustments). The primary vtable at offset 0 is always the complete one.
   Skip 2 entries from it, extract the rest.

4. Add helper `extract_single_vtable_array()` that takes one array,
   skips offset-to-top + RTTI, returns method slots.

### Task A2: Virtual Inheritance Extra Metadata (~50 LOC)

**File:** `crates/saf-frontends/src/llvm/cha_extract.rs`

**Problem:** Virtual inheritance vtables have 3+ metadata entries before RTTI
(virtual-base-offset, offset-to-top, [extra offsets], RTTI). The parser
skips only 2.

**Fix:**

1. In the single-array extraction path, scan forward from index 0 to find
   the RTTI entry. RTTI pointers are recognizable: they reference `@_ZTI*`
   globals (typeinfo). The entry AFTER the RTTI is the first method slot.

2. Detect RTTI by checking if the function pointer name starts with `_ZTI`.
   Everything before it (inclusive) is metadata; everything after is methods.

3. Fallback: if no `_ZTI` found, use the existing skip-2 heuristic.

### Task A3: Tests for Multi-Inheritance Vtable Parsing (~80 LOC)

**File:** `crates/saf-frontends/tests/` or unit tests in `cha_extract.rs`

Add tests for:
- Single inheritance: `{ [4 x ptr] }` — verify 2 metadata entries skipped
- Diamond inheritance: `{ [3 x ptr], [3 x ptr] }` — verify only primary parsed
- Virtual inheritance: `{ [5 x ptr] }` with 3 metadata entries — verify RTTI detection
- Deep virtual diamond: `{ [5 x ptr], [4 x ptr] }` — verify correct

**Estimated LOC:** ~280

---

## Phase B: Receiver-Type-Aware CHA Resolution (RC1)

**Why second:** This is the highest-impact fix (~30 cases). After Phase A
gives correct vtable slots, this phase ensures virtual calls resolve only
to methods of the receiver's type hierarchy, not all types.

### Task B1: Track Receiver Type in Virtual Call Pattern (~80 LOC)

**File:** `crates/saf-analysis/src/cg_refinement.rs`

**Current:** `match_virtual_call_pattern()` returns `VirtualCallInfo { slot_index }`.
It doesn't capture any information about the receiver object.

**Fix:**

1. Extend `VirtualCallInfo` to include `receiver_value: ValueId` — the
   ValueId of the object whose vptr was loaded (the base pointer of the
   first load in the pattern: `%vptr = load ptr, ptr %obj`).

2. In `match_virtual_call_pattern()`, after matching the Load→GEP→Load→Call
   chain, also extract the GEP's base pointer (the vptr), then trace back
   one more Load to find the object pointer. Record this as `receiver_value`.

3. Also trace back to check that the GEP base is a LOADED vptr (not a
   direct struct field access). This distinguishes virtual calls from
   function-pointer-in-class calls (RC4).

### Task B2: Resolve Per-Receiver-Type Instead of All Types (~100 LOC)

**File:** `crates/saf-analysis/src/cg_refinement.rs`

**Current code (lines 342-347):**
```rust
for entry in &module.type_hierarchy {
    let resolved = cha.resolve_virtual(&entry.type_name, vcall_info.slot_index);
    targets.extend(resolved);
}
```

This resolves slot N across ALL types. Two A-unrelated classes with
slot-0 methods both get included.

**Fix — two-tier approach:**

**Tier 1 (PTA-based, precise):** After the initial PTA solve in the
refinement loop, use the PTA result to look up the receiver's points-to
set. For each pointed-to location, determine the allocated type (from
`HeapAlloc` site or global type). Resolve the virtual call only for
those types and their subclasses.

Add to `RefinementResult`: `receiver_types: BTreeMap<InstId, BTreeSet<String>>`
mapping each virtual call to its possible receiver types.

**Tier 2 (CHA-only fallback, sound):** When no PTA result is available
(bootstrap phase), use the receiver ValueId to trace back to the
allocation site. If the allocation is `new B` (HeapAlloc), determine the
type from debug info or vtable store. Fall back to resolving for only the
declared static type's hierarchy.

For the bootstrap (pre-PTA) phase, a simpler heuristic: instead of
iterating ALL type hierarchy entries, only resolve for entries where the
class name appears in the receiver's transitive base chain. Use
`ClassHierarchy::bases()` to limit scope.

### Task B3: PTA-Based Devirtualization in Refinement Loop (~120 LOC)

**File:** `crates/saf-analysis/src/cg_refinement.rs`

**Current:** The refinement loop (lines 115-163) does PTA-based resolution
for indirect calls via `resolve_indirect_calls_via_pta()`, but this only
checks points-to sets of the function pointer operand. It doesn't use
receiver type information.

**Fix:**

1. After solving PTA in each iteration, for each virtual call site that
   was CHA-resolved, refine the target set using the receiver's points-to
   set.

2. For each allocation site in `pts(receiver)`, check if the allocated
   object's vtable store points to a known vtable global. Map this to a
   class name. Only keep targets from `cha.resolve_virtual(class, slot)`.

3. Add `refine_virtual_targets()` function that takes `(pts, resolved_sites,
   cha, module)` and returns a pruned `resolved_sites` map.

4. Call this after `resolve_indirect_calls_via_pta()` in each iteration.

**Estimated LOC:** ~300

---

## Phase C: Function Pointer Member Discrimination (RC4)

### Task C1: Distinguish Vtable GEP from Struct Field GEP (~60 LOC)

**File:** `crates/saf-analysis/src/cg_refinement.rs`

**Problem:** `match_virtual_call_pattern()` matches ANY Load→GEP→Load→Call
chain, including function pointer member access `a->pf(args)`.

The LLVM IR for a virtual call:
```llvm
%vptr = load ptr, ptr %obj       ; load vtable pointer FROM object
%slot = gep ptr, %vptr, i64 N    ; index INTO loaded vtable pointer
%fn = load ptr, ptr %slot
call %fn(...)
```

The LLVM IR for a function pointer member:
```llvm
%field = gep %class.A, ptr %obj, i32 0, i32 1  ; GEP into object struct
%fn = load ptr, ptr %field                        ; load function pointer
call %fn(...)
```

Key difference: the GEP base for virtual calls is a **loaded pointer**
(the vptr), while for member access it's the **object pointer itself**.
Also, the GEP type for virtual calls is `ptr` (flat pointer indexing),
while for member access it's a named struct type (`%class.A`).

**Fix:**

1. In `match_virtual_call_pattern()`, after finding the GEP instruction,
   check that the GEP instruction's first operand (the base pointer) is
   itself the result of a Load instruction (i.e., it's a loaded vptr,
   not a direct object pointer).

2. The chain must be: Load(vptr from obj) → GEP(slot in vptr) →
   Load(fn from slot) → CallIndirect. Currently only the last 3 steps
   are validated.

3. When the pattern DOESN'T match (function pointer member), do NOT
   fall back to CHA all-methods resolution. Instead, leave the
   CallIndirect unresolved for CHA and let PTA-based resolution
   handle it in the iterative refinement loop.

**Estimated LOC:** ~60

---

## Phase D: Destructor Member Field Tracking (RC3)

### Task D1: Constructor Store Tracking for Member Fields (~100 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs`

**Problem:** In `destructor-1.cpp`, the constructor stores `i` to
`this->aptr` (a member field). Later, the destructor's virtual call
dispatches to `f()`, which loads `this->aptr` and checks
`MUSTALIAS(global_ptr, aptr)`. PTA returns Unknown because it can't
trace through the constructor's field store.

**The data flow chain:**
```
main: B(i) where i = &global_obj
  → B::B(int *i): stores i to this->bptr (GEP field 1)
  → ~B(): calls virtual f(this)
  → B::f(): loads this->bptr (GEP field 1), checks MUSTALIAS
```

**Fix:**

The CS-PTA already has field-sensitive constraints. The issue is that
the constructor's store to `this->bptr` must create a `Store` constraint
with the correct field path, and the load in `B::f()` must create a
`Load` constraint with the same field path.

Check if the current field-sensitive extraction handles `GEP` into the
`this` pointer correctly. The GEP `%gep = getelementptr %class.B, ptr
%this, i32 0, i32 1` should produce a `FieldPath` of `[0, 1]`. The
subsequent `store ptr %i, ptr %gep` should create a `Store` constraint:
`*(%this + field[0,1]) ← %i`.

If this already works (it should, given field sensitivity up to depth 6),
the issue may be that the **constructor is not in the resolved call
targets** (it's a direct call, not virtual) or the CS-PTA context doesn't
connect the constructor's `this` to the destructor's `this`.

Investigate and fix the context chain: `main` calls `B::B(obj)` (direct),
`delete b` calls `~B(obj)` → virtual `f(obj)`. The `this` pointer must
be the same across all three. If CS-PTA creates different context-qualified
versions, they may not connect.

### Task D2: Validate Destructor Virtual Dispatch Chain (~50 LOC)

**File:** `crates/saf-analysis/tests/` (E2E test)

Create a focused E2E test with:
```cpp
class A { A(int *i): m(i) {} virtual void f() { /* check m */ } int *m; };
```

Compile with `-O0`, verify that CS-PTA connects:
1. Constructor store: `this->m = i`
2. Virtual method load: `this->m`
3. Alias query: `MUSTALIAS(global_ptr, this->m)`

**Estimated LOC:** ~150

---

## Phase E: Global Aggregate Array Tracking (RC5)

### Task E1: Aggregate GlobalRef Initializer Constraints (~80 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs`

**Problem:** `extract_aggregate_elements()` handles `Constant::Int` values
that match function IDs (lines 257-260) and recurses for nested
`Constant::Aggregate`. But it does NOT handle `Constant::GlobalRef` inside
aggregates — which is what `global-obj-in-array.bc` uses for the
`TableEntry` array containing object pointers.

The global:
```cpp
TableEntry theTable[] = { {1, &a1}, {2, &a2}, {3, &a3}, {0, 0} };
```

In AIR, this becomes `Constant::Aggregate` with elements containing
`Constant::Int` (for num field) and `Constant::GlobalRef` (for the
object pointer field).

**Fix:**

1. In `extract_aggregate_elements()`, add a case for `Constant::GlobalRef(vid)`:
   - Create a field location for the aggregate field
   - Create a `Store` constraint: `*(global_obj + field_path) ← vid`
   - This ensures PTA knows the aggregate's pointer field points to the
     referenced global object.

2. Also handle `Constant::Null` in aggregates (the `{0, 0}` sentinel):
   skip or create a null constraint.

### Task E2: Test Global Object Array (~40 LOC)

Create a test with global array of structs containing object pointers.
Verify PTA resolves virtual calls through array-loaded object pointers.

**Estimated LOC:** ~120

---

## Implementation Order

```
Phase A (vtable parsing)     ← correctness foundation
  ↓
Phase B (receiver-aware CHA) ← highest impact
  ↓
Phase C (fn-ptr member)      ← quick fix, prevents false matches
  ↓
Phase D (destructor members)  ← investigation + targeted fix
  ↓
Phase E (global aggregates)   ← smallest impact, standalone
```

## Expected Results

| Phase | Cases Fixed | New Exact | Remaining Unsound |
|-------|-----------|-----------|-------------------|
| A | ~15 | +15 | ~39 |
| B | ~25 | +25 | ~14 |
| C | ~4 | +4 | ~10 |
| D | ~4 | +4 | ~6 |
| E | ~2 | +2 | ~4 |

After all phases: **~50 of 54 unsound cases fixed**, from 66 Exact to ~116 Exact.
Remaining ~4 may require deeper analysis (e.g., STL container tracking).

## Total Estimated LOC: ~910
