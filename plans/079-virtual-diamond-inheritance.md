# Plan 079: Virtual Diamond Inheritance — Secondary Vtable Parsing + CHA Resolution

## Problem Statement

Two PTABen tests are failing due to unsupported virtual diamond inheritance vtable parsing:
- `failed_tests/virtual-diamond-inheritance-1.bc` — 2 unsound (of 20 oracles)
- `failed_tests/vdiamond-multi-inher.bc` — 4 unsound (of 23 oracles)

Total: **6 unsound oracles** (37 Exact already passing).

## Root Cause Analysis

### LLVM IR Vtable Layout for Virtual Diamond Inheritance

Consider `virtual-diamond-inheritance-1.cpp` with class hierarchy:
```
A ← virtual B ← D
A ← virtual C ← D
```

LLVM generates **multi-sub-array vtables** for D:
```llvm
@_ZTV1D = constant { [7 x ptr], [6 x ptr] } {
  ; Primary vtable (B subobject at offset 0)
  [7 x ptr] [
    ptr null,           ; vbase offset (A)
    ptr null,           ; vbase offset (B→A)
    ptr null,           ; offset-to-top
    ptr @_ZTI1D,        ; RTTI
    ptr @_ZN1A1fEPi,    ; slot 0: A::f
    ptr @_ZN1B1gEPi,    ; slot 1: B::g
    ptr @_ZN1D1lEPi     ; slot 2: D::l
  ],
  ; Secondary vtable (C subobject at offset 8)
  [6 x ptr] [
    ptr inttoptr(i64 -8 to ptr),  ; vbase offset
    ptr inttoptr(i64 -8 to ptr),  ; offset-to-top
    ptr inttoptr(i64 -8 to ptr),  ; offset-to-top (C→A)
    ptr @_ZTI1D,                  ; RTTI
    ptr null,                     ; slot 0: pure virtual / thunk
    ptr @_ZN1C1hEPi               ; slot 1: C::h  ← ONLY SOURCE OF THIS METHOD
  ]
}
```

**Critical observation:** `C::h` appears ONLY in the secondary vtable. The primary vtable does not contain it because C is the second base class, so its methods live in the secondary sub-array.

### How Virtual Calls Are Dispatched

When `pc->h(ptr_h)` is called (where `pc` is a `C*` pointing to the C subobject of D):

```llvm
; pc = &d + 8  (C subobject at byte offset 8)
%add.ptr5 = getelementptr inbounds i8, ptr %d, i64 8
store ptr %add.ptr5, ptr %pc

; Load C's vtable pointer (from secondary vtable)
%vtable8 = load ptr, ptr %8       ; loads D's secondary vptr

; GEP to slot 1 in C's vtable
%vfn9 = getelementptr inbounds ptr, ptr %vtable8, i64 1

; Load and call C::h
%10 = load ptr, ptr %vfn9
call void %10(...)
```

The virtual call pattern matcher correctly matches this as `slot_index = 1`. But CHA resolution searches all root class vtables at slot 1:
- A's vtable slot 1: doesn't exist (A only has slot 0)
- But any root class at slot 1 gets merged

The **core problem** is that D's `ClassHierarchy` vtable only contains the primary sub-array's methods: `{0: A::f, 1: B::g, 2: D::l}`. Method `C::h` is missing entirely because it's in the secondary vtable which is currently ignored.

### Virtual Base Access Pattern (for `pa->f()`)

When casting D* to A* for virtual base access:
```llvm
%vtable = load ptr, ptr %d                   ; load primary vptr
%vbase.offset.ptr = getelementptr i8, ptr %vtable, i64 -32  ; access negative offset
%vbase.offset = load i64, ptr %vbase.offset.ptr              ; load vbase offset
%add.ptr = getelementptr inbounds i8, ptr %d, i64 %vbase.offset  ; adjust pointer
```

This uses a **negative vtable offset** (-32 = -4 pointers × 8 bytes back from vptr position at index 4 of the 7-element primary vtable → index 0, the first null entry, which holds the virtual base offset). The loaded offset adjusts `this` to find the shared A subobject. This pattern is **not a virtual call** — it's a pointer adjustment. The actual `A::f` call at slot 0 through the adjusted pointer works correctly because the primary vtable has `A::f` at slot 0. So this pattern doesn't cause unsound results directly.

### Why the 6 Oracles Fail

The failing oracles are in methods that are resolved via **secondary vtable slots**:

1. **`C::h` in virtual-diamond-inheritance-1** — C::h is at secondary vtable slot 1. CHA doesn't know about this method for class D, so when called through `C*`, CHA resolves slot 1 to `B::g` (from primary vtable) instead of or in addition to `C::h`. This causes the `MUSTALIAS(global_ptr_h, i)` oracle in C::h to get `Partial` (both B::g and C::h are called, merging their `i` parameter points-to sets). The `NOALIAS(global_ptr_h, i)` oracle in `A::f` gets `No` because the virtual base access path confuses PTA.

2. **`C::h` in vdiamond-multi-inher** — Same root cause. E has 3 sub-arrays.

3. **`D::l` in vdiamond-multi-inher** — D is the third base of E, its vtable is the third sub-array `[3 x ptr]`. D::l at slot 0 of that sub-array is invisible to CHA.

## Approach: Parse All Vtable Sub-Arrays with Slot Mapping

The fix requires two changes:

1. **Parse secondary vtable sub-arrays** in `cha_extract.rs` to discover methods that only appear there (like `C::h`, `D::l`).
2. **Register these methods** in the class's vtable entry so CHA resolution can find them.

### Key Insight: Per-Class Vtable Accumulation

For a class like D with hierarchy `A ← virtual {B, C} ← D`:
- Primary vtable has: `A::f (slot 0), B::g (slot 1), D::l (slot 2)`
- Secondary vtable has: `C::h (slot 1)`

The **combined vtable** for class D should be: `{0: A::f, 1: B::g, 2: D::l}` from primary, PLUS register that `C::h` is accessible at some slot. However, the slot indices in secondary vtables are relative to the secondary subobject, not the primary. We can't simply merge them by slot index.

**Better approach:** Instead of tracking per-slot indices, extract all method function IDs from ALL sub-arrays and add them as additional vtable entries with unique slot indices. CHA resolution for a class with both primary and secondary methods will then find all methods. The slot index is only used by `match_virtual_call_pattern()` and `resolve_virtual()` — we need to ensure the secondary methods are visible.

**Simplest correct approach:** For each secondary sub-array, extract its methods (skipping metadata) and append them to the class's vtable at slot indices continuing after the primary vtable's last slot. This way:
- Primary: slots 0, 1, 2 → A::f, B::g, D::l
- Secondary (C subobject): original slots 0, 1 → null, C::h → appended as slots 3, 4 → null, C::h

Then CHA resolves:
- Slot 0: A::f (primary) ✓
- Slot 1: B::g (primary) AND C::h won't be at slot 1 — this doesn't work.

**The actual problem is more subtle.** When `pc->h(ptr_h)` is called through C*, the vtable load gets the secondary vtable pointer (not primary). The GEP uses slot 1 relative to the secondary vtable. So CHA needs to understand that for a call through the **C subobject**, slot 1 maps to `C::h`, not `B::g`.

### Refined Approach: Secondary Vtable Registration

Since SAF's virtual call pattern currently resolves calls by slot index against ALL root classes' vtables, and this works for the primary vtable, the fix is to register secondary vtable methods so they can be found.

Two sub-problems:
1. **Extracting methods from secondary vtables** — straightforward extension of `extract_primary_vtable_pointers`
2. **Making CHA resolve them at the correct slot** — the C subobject virtual call uses slot 1, and C::h IS at slot 1 in C's own vtable (since C extends A virtually, C's vtable is `{0: A::f, 1: C::h}`).

**Key insight:** The secondary vtable for C in D's `_ZTV1D` replicates C's own vtable layout. So C::h at slot 1 in the secondary vtable **matches** C::h at slot 1 in a standalone C's vtable. The problem is that `extract_vtable_slots()` for D only looks at the primary sub-array and produces `{0: A::f, 1: B::g, 2: D::l}`. It never populates C's vtable with C::h.

**But C does have its own standalone vtable** (`_ZTV1C` is NOT present in the IR — let me verify). Looking at the actual IR: there is NO `@_ZTV1C` global and NO `@_ZTV1B` global. Only `@_ZTV1A`, `@_ZTV1D`, construction vtables `@_ZTC1D0_1B` and `@_ZTC1D8_1C`. So B and C don't have standalone vtables — their methods only exist in D's multi-sub-array vtable and the construction vtables.

**This is the critical issue.** B and C have no standalone `_ZTV` globals because they are abstract-like (have virtual bases). Their methods are only discoverable from:
1. D's secondary vtable sub-arrays
2. Construction vtables (`_ZTC*`) — intermediate vtables used during construction

### Solution: Parse Construction Vtables (_ZTC*)

The `_ZTC*` globals are **construction vtables** (VTT entries). They represent the vtable that a subobject uses during construction before the complete object's vtable is installed.

```
@_ZTC1D0_1B = { [6 x ptr] } — B-in-D construction vtable at offset 0
@_ZTC1D8_1C = { [6 x ptr], [4 x ptr] } — C-in-D construction vtable at offset 8
```

For `@_ZTC1D8_1C`:
```llvm
{ [6 x ptr] [
    ptr inttoptr(i64 -8 to ptr),  ; vbase offset
    ptr inttoptr(i64 -8 to ptr),  ; offset-to-top
    ptr null,                     ; (padding/offset)
    ptr @_ZTI1C,                  ; RTTI → class C
    ptr null,                     ; slot 0 (A::f placeholder)
    ptr @_ZN1C1hEPi               ; slot 1 → C::h ✓
  ],
  [4 x ptr] [...]
}
```

This construction vtable tells us: **C has slot 1 → C::h**.

### Alternative (Simpler): Parse Secondary Sub-Arrays of Complete Vtables

Instead of dealing with `_ZTC*`, we can extract methods from ALL sub-arrays of the complete vtable `@_ZTV1D`. Each secondary sub-array contains RTTI (`@_ZTI1D`) and methods. For each sub-array:
1. Find the RTTI entry → determine metadata count
2. Extract method slots after RTTI
3. Register these methods against the appropriate class

But which class should own these slots? The RTTI always points to D (the complete class), not to C. We need a different signal to associate slot 1 of the secondary sub-array with C::h.

**Simplest correct solution:** Instead of trying to track which base class owns which secondary vtable, just collect ALL method function pointers from ALL sub-arrays and ensure they appear in the class's method set. The key is that `C::h` needs to be **discoverable** by CHA for the slot index used at the call site.

When `pc->h()` is dispatched, the slot index is 1 (relative to C's vtable). CHA resolves slot 1 across all root classes. Currently:
- A at slot 1: nothing (A only has slot 0)
- Result: empty → no resolution → falls to PTA

If we parse the secondary sub-array and register `C::h` at slot 1 of D's vtable, we'd overwrite `B::g` (also at slot 1). That's wrong.

**The fundamental issue:** D's complete vtable has different methods at the same slot index depending on which subobject vtable is being accessed. Slot 1 in the primary vtable is `B::g`, slot 1 in the secondary vtable is `C::h`.

### Final Approach: Subobject-Aware Vtable Slots

We need to parse each vtable sub-array independently and register the discovered methods to the appropriate base class in the CHA. The key mapping is:

1. **Primary sub-array** → methods belong to the most-derived class (D) and its primary base chain (B → A)
2. **Secondary sub-array N** → methods belong to the Nth non-primary base class

For virtual diamond inheritance:
- D's primary sub-array: methods for B-subobject chain → register to D
- D's secondary sub-array: methods for C-subobject → register to C

**How to determine which class a secondary sub-array belongs to:**
Use the construction vtable names `_ZTC<D><offset>_<Base>`:
- `_ZTC1D0_1B` → B at offset 0 in D
- `_ZTC1D8_1C` → C at offset 8 in D

Or simply use the fact that in the typeinfo, D's bases are listed in order `[B, C]`, matching the sub-array order `[primary=B, secondary=C]`.

**Or even simpler:** Extract methods from each secondary sub-array, and for EACH method found, search for which base class defines it (by scanning all `_ZTC*` and base class hierarchies). This is over-engineered.

### Simplest Correct Approach: Register All Methods to the Complete Class

The simplest approach that fixes these tests:

**Phase A — Extract ALL sub-array methods:**
Modify `extract_vtable_slots()` to process ALL sub-arrays of a struct-of-arrays vtable (not just the first one). For each sub-array, find RTTI, skip metadata, collect method slots. Use a **merged slot map** where secondary sub-array methods get unique slot indices beyond the primary's range.

**Phase B — Populate intermediate class vtables from construction vtables:**
Parse `_ZTC*` globals (construction vtables) to discover methods for intermediate classes (B, C) that don't have their own `_ZTV*` globals. The `_ZTC` name encodes the base class (`_ZTC1D8_1C` → class C). Extract vtable slots from these and register them as C's vtable entries.

This way:
- C gets vtable `{0: A::f (or null), 1: C::h}` from `_ZTC1D8_1C`
- CHA resolves slot 1 on root class A → finds C::h in C's vtable (C is a subclass of A)

## Implementation Plan

### Phase A: Construction Vtable Parsing (~80 LOC)

**File:** `crates/saf-frontends/src/llvm/cha_extract.rs`

**Task A1: Parse `_ZTC*` globals** (~50 LOC)

Add a new pass in `extract_type_hierarchy()` to scan for `_ZTC*` globals.

1. Identify `_ZTC*` globals (construction vtables)
2. Demangle the name to extract the base class: `_ZTC<derived><offset>_<base>` → base class name
3. Call `extract_vtable_slots()` on the construction vtable global
4. If the base class doesn't already have vtable entries in `entries_map`, add them
5. If it does, merge (don't overwrite) — the construction vtable may have placeholders (null) for slots that the base class fills in from its own `_ZTV`

The name demangling for `_ZTC` follows this pattern:
- `_ZTC` prefix
- Derived class name (length-prefixed, e.g., `1D`)
- Offset in decimal (e.g., `8`)
- `_` separator
- Base class name (length-prefixed, e.g., `1C`)

**Task A2: Demangle `_ZTC` names** (~30 LOC)

Add `demangle_construction_vtable_name()` that extracts the base class name from a `_ZTC*` global name:
```
_ZTC1D0_1B → Some("B")
_ZTC1D8_1C → Some("C")
```

Parse: strip `_ZTC`, parse derived class name, skip digits (offset), skip `_`, parse base class name.

### Phase B: Secondary Sub-Array Extraction (~60 LOC)

**File:** `crates/saf-frontends/src/llvm/cha_extract.rs`

**Task B1: Extract ALL sub-arrays from multi-inheritance vtables** (~40 LOC)

Create `extract_all_vtable_subarrays()` that processes a struct-of-arrays vtable and returns a `Vec<Vec<VirtualMethodSlot>>` — one slot list per sub-array.

For each sub-array:
1. Parse function pointers from the array
2. Find RTTI index, compute metadata count
3. Extract method slots after metadata

**Task B2: Merge secondary sub-array methods into class vtable** (~20 LOC)

In `extract_type_hierarchy()`, after processing primary vtable:
- For each secondary sub-array, extract its methods
- Skip null/thunk entries (entries where the function name starts with `_ZThn` are thunks)
- For non-null, non-thunk entries found in secondary sub-arrays, check if the method is already present in the class's vtable; if not, add it at an extended slot index

### Phase C: Test and Verify (~20 LOC)

**Task C1: Unit tests for `_ZTC` demangling** (~10 LOC)

Add tests for `demangle_construction_vtable_name()`:
- `_ZTC1D0_1B` → `Some("B")`
- `_ZTC1D8_1C` → `Some("C")`
- `_ZTC1E0_1B` → `Some("B")`
- `_ZTC1E8_1C` → `Some("C")`

**Task C2: Run PTABen and verify** (~10 LOC)

Run the benchmark filter for both test files. Verify:
- virtual-diamond-inheritance-1: 2 unsound → 0 unsound (20/20 Exact)
- vdiamond-multi-inher: 4 unsound → 0 unsound (23/23 Exact)
- No regressions in other categories (basic_cpp_tests, basic_c_tests, etc.)

## Expected Results

| Test | Before | After |
|------|--------|-------|
| virtual-diamond-inheritance-1 | 18 Exact, 2 Unsound | 20 Exact, 0 Unsound |
| vdiamond-multi-inher | 19 Exact, 4 Unsound | 23 Exact, 0 Unsound |
| **Total improvement** | | **+6 Exact, -6 Unsound** |

## Risk Assessment

- **Low risk:** Changes are additive — we're adding new vtable entries for classes that currently have none. No existing vtable entries are modified.
- **Regression risk:** Construction vtable parsing might pick up thunks (`_ZThn*` functions) as virtual methods. Need to filter these out.
- **Scope:** ~160 LOC across 1 file (`cha_extract.rs`), contained change.

## File Changes

| File | Change |
|------|--------|
| `crates/saf-frontends/src/llvm/cha_extract.rs` | Add `_ZTC*` parsing, `demangle_construction_vtable_name()`, secondary sub-array extraction |

## Key Technical Details

### Object Layout (virtual-diamond-inheritance-1, class D)

```
D object (16 bytes):
  offset 0: vptr → D's primary vtable [A::f, B::g, D::l]  (B subobject)
  offset 8: vptr → D's secondary vtable [null, C::h]       (C subobject)
  (shared A subobject accessed via vbase offset in vtable metadata)
```

### Constructor Vtable Store Pattern

```llvm
; D's constructor stores vptrs at subobject offsets:
store ptr GEP(@_ZTV1D, 0, 0, 4), ptr %this1           ; primary vptr at offset 0
store ptr GEP(@_ZTV1D, 0, 1, 4), ptr (this1 + 8)      ; secondary vptr at offset 8
```

### Virtual Call Through C Subobject

```llvm
; pc points to C subobject (offset 8 in D)
%vtable8 = load ptr, ptr %8         ; loads secondary vptr
%vfn9 = getelementptr ptr, ptr %vtable8, i64 1  ; slot 1
%10 = load ptr, ptr %vfn9           ; loads C::h
call void %10(...)                  ; dispatches C::h
```

CHA sees slot 1, resolves across root classes:
- Before fix: A has no slot 1, B has slot 1 = B::g → resolves to {B::g} → wrong
- After fix: C has slot 1 = C::h (from construction vtable parsing) → resolves to {B::g, C::h} → both targets in PTA → correct parameter flow → MustAlias
