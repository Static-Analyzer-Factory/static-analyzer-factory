# Plan 065: Basic C Alias Precision — Fix basic_c_tests Unsound Cases

## Problem Statement

38 unsound oracle checks across 25 test files in PTABen `basic_c_tests`.
Current results: 49 Exact, 14 Sound, 9 ToVerify, **38 Unsound**, 4 Skip (114 oracle checks total).

Nearly all failures report `got=Unknown`, meaning one or both queried ValueIds
have **no points-to set at all** — they were never tracked by the PTA solver.
The root cause is missing constraint generation for specific IR patterns.

## Root Cause Summary

| # | Root Cause | Cases | Pattern | Key Files |
|---|-----------|-------|---------|-----------|
| RC1 | Memcpy generates no constraints | ~15 | MustAlias/MayAlias→Unknown | pta/extract.rs |
| RC2 | Global aggregate initializer constraints not called | ~8 | MayAlias→Unknown | pta/context.rs |
| RC3 | Indirect call targets not in call graph (interprocedural gaps) | ~8 | MayAlias→Unknown | ptaben.rs, cg_refinement.rs |
| RC4 | MustAlias→Partial for double-pointer dereference chains | 2 | MustAlias→Partial | pta/result.rs |
| RC5 | Complex nested struct + array index + type cast combinations | ~5 | Various→Unknown | pta/extract.rs |

## Detailed Root Cause Analysis

### RC1: `Operation::Memcpy` Generates No PTA Constraints (~15 cases)

**Affected tests:** struct-assignment-direct, struct-assignment-indirect,
struct-assignment-nested, funptr-nested-struct-simple, funptr-nested-struct,
arraycopy1, struct-instance-return, spec-mesa (struct copy via `ctx->API = ctx->Exec`)

**Evidence:** In `extract.rs:563`, `Operation::Memcpy` is in the "no-op" arm:
```rust
| Operation::Memcpy
| Operation::Memset => {}
```

**LLVM IR pattern (struct-assignment-direct.bc):**
```llvm
%7 = getelementptr %struct.s, ptr %2, i32 0, i32 0   ; s1.a
store ptr %5, ptr %7                                   ; s1.a = &x
call void @llvm.memcpy(ptr %3, ptr %2, i64 16, ...)   ; s2 = s1  ← IGNORED
%10 = getelementptr %struct.s, ptr %3, i32 0, i32 0   ; s2.a
%11 = load ptr, ptr %10                                ; load s2.a → Unknown!
call void @MUSTALIAS(ptr %11, ptr %13)                 ; MUSTALIAS(s2.a, s1.a)
```

Since memcpy is skipped, `s2`'s fields are never populated. Any subsequent
load from `s2`'s fields has no points-to set → `AliasResult::Unknown`.

**Fix approach:** Generate a `Copy` constraint from memcpy's src to dst:
`Copy(dst=operands[0], src=operands[1])`. This models that `*dst = *src`
at the base-pointer level. The solver's existing Load/Store propagation
then copies the src object's field contents to the dst.

For struct-level copies, this is sound but conservative: it makes dst
alias with everything src aliases with, which is correct for a memcpy.

### RC2: `extract_global_initializers()` Not Called in PTA Pipeline (~8 cases)

**Affected tests:** funptr-global, funptr-struct, global-call-struct,
global-call-twoparms, global-const-struct, spec-gap, spec-parser, spec-vortex

**Evidence:** In `context.rs:84`, `analyze_with_specs()` calls
`extract_constraints()` but NOT `extract_global_initializers()`. The function
exists and is public (extract.rs:197) but is never invoked in the main PTA
pipeline. It IS called by `cg_refinement.rs` for call graph building, but
the resolved targets aren't propagated back into the main PTA solve.

These tests use global structs initialized with pointer/function-pointer fields:
```c
struct PLHashAllocOps { void *(*allocTable)(void *, ...); };
struct PLHashAllocOps defaultHashAllocOps = { DefaultAllocTable };
```

The aggregate initializer `{ DefaultAllocTable }` contains a function pointer
that needs Store constraints to be visible to PTA. Without
`extract_global_initializers()`, the function pointer values inside global
aggregates are invisible.

**Fix approach:** Call `extract_global_initializers()` in `PtaContext::analyze_with_specs()`
right after `extract_constraints()`. Also handle `Constant::GlobalRef` inside
aggregates (current code only matches `Constant::Int` matching function IDs).

### RC3: Interprocedural Gaps — Indirect Call Targets Not Resolved (~8 cases)

**Affected tests:** funptr-struct, funptr-global, global-call-twoparms,
spec-equake, spec-mesa, spec-gap, heap-linkedlist

**Pattern:** These tests call functions through function pointers stored in
structs. Without the global initializer constraints (RC2), PTA can't resolve
the indirect call targets. Without resolving call targets, interprocedural
constraints (arg→param, return→caller) are missing for those callees.

This is a cascading effect: RC2 blocks function pointer resolution, which
blocks interprocedural tracking, which causes Unknown for values flowing
through indirectly-called functions.

For heap-linkedlist specifically, the function `malloc_list()` is called from
`main()` and modifies the linked list through pointer parameters. The
`p_data1` / `p_data2` values loaded inside `malloc_list` need interprocedural
constraints to flow back to main's oracle checks.

**Fix approach:** This largely resolves with RC2. Additionally, the iterative
refinement loop in `cg_refinement.rs` should propagate newly-resolved targets
back into the PTA constraint set for re-solving.

### RC4: MustAlias→Partial for Double-Pointer Dereference (2 cases)

**Affected tests:** ptr-dereference1, ptr-dereference3

**Pattern (ptr-dereference1.c):**
```c
int *s, *r, *x, **y, t, z, k;
s = &t; r = &z; y = &r;
s = r;           // s = &z
MUSTALIAS(s, &z); // Expected: MustAlias
x = *y;          // x = *(&r) = r = &z
MUSTALIAS(x, r); // Expected: MustAlias
```

PTA reports `Partial` instead of `Must`. This is likely because:
- `s` has pts `{loc(z)}` from the copy `s = r` where `r → {loc(z)}`
- But `s` also had pts `{loc(t)}` from the earlier `s = &t`
- CI-PTA merges both: `s → {loc(t), loc(z)}`
- `&z` → `{loc(z)}` (singleton)
- `{loc(t), loc(z)}` vs `{loc(z)}`: one is a proper subset → `Partial`

For `x = *y`: `y → {loc(r)}`, dereference gives `x → pts(r) = {loc(z)}`.
Then `MUSTALIAS(x, r)` compares `{loc(z)}` vs `{loc(z)}` → should be Must.
If PTA reports Partial, the issue may be that `x`'s set accumulated extra
locations from the Load propagation.

**Fix approach:** This is a flow-sensitivity issue. CI-PTA merges across all
program points, so `s` accumulates both `{loc(t)}` (from `s = &t`) and
`{loc(z)}` (from `s = r`). The combined CS-PTA + FS-PTA should handle this
if flow-sensitive PTA correctly models the kill of the first assignment. Check
if the FS-PTA result is being used for these queries. If the combined result
returns Unknown and falls back to CI-PTA, that explains the Partial result.

### RC5: Complex Patterns — Nested Struct + Array + Type Cast (~5 cases)

**Affected tests:** struct-incompab-typecast-nested (3 oracles),
struct-nested-array2 (2 oracles)

These combine multiple challenging patterns:
- **Type cast between incompatible structs**: `pdst = (DstStruct*)psrc` where
  src and dst have different field layouts. Field paths computed for one type
  don't match the other.
- **Nested arrays in structs**: `p->out3.mid2[2].in1[2]` requires array index
  sensitivity within deeply nested struct fields. The current index collapse
  makes `mid2[1]`, `mid2[2]`, `mid2[3]` all alias to the same location.

**Fix approach:** RC5 tests push the limits of field/index sensitivity.
Struct-incompab-typecast-nested may need type-aware field remapping. The nested
array tests need the existing constant-index sensitivity to work through nested
GEPs, which may be blocked by field depth limits (max_depth: 6 vs actual depth).

---

## Phase A: Memcpy Constraint Generation (RC1) — Highest Impact

### Task A1: Generate Copy Constraint for Memcpy (~30 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs`

**Change:** In `extract_instruction()`, replace the no-op handling of
`Operation::Memcpy` with constraint generation:

```rust
Operation::Memcpy => {
    // memcpy(dst, src, size): model as Copy(dst, src)
    // This makes dst's abstract location contain everything src's does.
    // operands[0] = dst pointer, operands[1] = src pointer
    if inst.operands.len() >= 2 {
        constraints.copy.insert(CopyConstraint {
            dst: inst.operands[0],
            src: inst.operands[1],
        });
    }
}
```

**Note on dst ValueId:** Memcpy instructions in AIR may not have a `dst`
ValueId (they're void-returning). The copy constraint here uses the
*pointer operands* (not `inst.dst`), which is correct: it models that the
*pointed-to* memory of operands[0] receives the *pointed-to* memory of
operands[1]. The solver processes Copy constraints by propagating the src's
points-to set to the dst, and for pointer-to-pointer operations this
transitively copies through the memory.

**Wait** — a simple `Copy(dst_ptr, src_ptr)` would make `dst_ptr` point to
whatever `src_ptr` points to (the struct objects). This is wrong: memcpy
copies the *contents*, not the pointer value. We need a different approach.

**Correct approach — Load+Store pair:**
```rust
Operation::Memcpy => {
    // memcpy(dst, src, size): model as *dst = *src
    // This copies the contents of src's pointed-to memory to dst's.
    // We model this as: tmp = load src; store tmp to dst
    if inst.operands.len() >= 2 {
        // Create a synthetic tmp value for the loaded content
        let tmp = ValueId::derive_child(inst.id.raw(), b"memcpy_tmp");
        constraints.load.insert(LoadConstraint {
            dst: tmp,
            src_ptr: inst.operands[1], // src
        });
        constraints.store.insert(StoreConstraint {
            dst_ptr: inst.operands[0], // dst
            src: tmp,
        });
    }
}
```

This models `tmp = *src; *dst = tmp`, which correctly copies the pointed-to
contents rather than the pointer values themselves. For struct copies, this
propagates field contents at the base-object level.

**Limitation:** This models the copy at a single level of indirection. For
nested struct pointers (pointer-to-pointer fields), full field-by-field copy
would be more precise. But the Load+Store model is sound (conservative) and
handles the majority of cases.

### Task A2: Test Memcpy Constraint Generation (~50 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs` (unit tests)

Add unit test that creates an AIR module with a Memcpy instruction and
verifies that Load + Store constraints are generated.

**Estimated LOC:** ~80

---

## Phase B: Global Aggregate Initializer Integration (RC2)

### Task B1: Call `extract_global_initializers` in PTA Pipeline (~5 LOC)

**File:** `crates/saf-analysis/src/pta/context.rs`

**Change:** Add call after `extract_constraints()` at line 84:

```rust
let mut constraints = extract_constraints(module, &mut self.factory);

// Extract constraints from global aggregate initializers (vtables, function
// pointer tables, struct initializers with pointer fields)
super::extract::extract_global_initializers(module, &mut self.factory, &mut constraints);
```

### Task B2: Handle `Constant::GlobalRef` in Aggregate Initializers (~20 LOC)

**File:** `crates/saf-analysis/src/pta/extract.rs`

**Change:** In `extract_aggregate_elements()`, add handling for
`Constant::GlobalRef` alongside the existing `Constant::Int` match:

```rust
Constant::GlobalRef(target_id) => {
    // Global pointer field: e.g., struct { .f2 = &x }
    let field_loc = factory.get_or_create(global_obj, field_path.clone());
    let field_ptr = ValueId::derive_child(global_obj.raw(), &i.to_le_bytes());
    constraints.addr.insert(AddrConstraint {
        ptr: field_ptr,
        loc: field_loc,
    });
    constraints.store.insert(StoreConstraint {
        dst_ptr: field_ptr,
        src: *target_id,
    });
}
```

This generates Store constraints modeling `global.field = &target`, making
pointer fields in global struct initializers visible to PTA.

### Task B3: Test Global Struct Initializer Constraints (~40 LOC)

Add unit test with a global struct whose `Aggregate` initializer contains
`Constant::GlobalRef` entries. Verify Store constraints are generated.

**Estimated LOC:** ~65

---

## Phase C: Interprocedural Refinement for Indirect Calls (RC3)

### Task C1: Wire CG Refinement Results into PTABen Harness (~40 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

The PTABen harness already runs `cg_refinement::refine()` (line 427) which
resolves indirect calls. But the resolved call targets are only used for
building the call graph — NOT fed back into the PTA constraint set.

**Change:** After the refinement loop resolves indirect calls, use the
`resolved_sites` to add interprocedural constraints for the newly-discovered
call targets. This means:

1. From `refinement_result.resolved_sites`, for each `(call_inst_id, targets)`:
   - Create `CopyConstraint` for each arg→param pair
   - Create `CopyConstraint` for return→caller pair
2. Re-run PTA with the augmented constraints

**Alternative approach:** Simply ensure that the combined CS-PTA already uses
the refined call graph (it does via `solve_context_sensitive_with_resolved()`
which takes `&resolved_sites`). The issue may be that the resolved sites from
CG refinement don't include function-pointer-in-struct targets because
`extract_global_initializers` is not called. If Phase B fixes the constraint
gap, this cascading issue may resolve automatically.

**Decision:** Implement Phase B first. If the funptr tests still fail after B,
then implement this phase.

### Task C2: Validate Cascading Fix (~10 LOC)

After Phase B, re-run PTABen to check if funptr tests pass. If they do,
mark this phase as resolved by Phase B.

**Estimated LOC:** ~50 (or 0 if Phase B suffices)

---

## Phase D: Fix MustAlias→Partial for Flow-Insensitive Accumulation (RC4)

### Task D1: Investigate FS-PTA Fallback for ptr-dereference Tests (~30 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

The combined PTA falls back to CI-PTA when combined returns Unknown. The
ptr-dereference tests exercise `s = &t; s = r;` where CI-PTA accumulates
both targets. FS-PTA should kill the first assignment.

**Investigation steps:**
1. Add debug logging to show whether combined PTA returns Unknown (triggering
   CI-PTA fallback) or Partial directly
2. Check if FS-PTA handles the ptr-dereference pattern correctly
3. If FS-PTA returns Unknown for these tests, the issue is that the SSA values
   used as oracle arguments don't match the flow-sensitive value naming

**Fix approach:** If FS-PTA returns Must correctly, ensure the combined result
takes precedence over CI-PTA. If FS-PTA also returns Partial, this is a
fundamental CI limitation and the tests should be marked as known-imprecision
(not a bug).

**Estimated LOC:** ~30

---

## Phase E: Complex Pattern Improvements (RC5)

### Task E1: Increase Field Depth for Nested Array Structs (~5 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

struct-nested-array2 uses 3-level nesting with arrays: `out3.mid2[N].in1[N]`.
The current max_depth is 6. Check if this is sufficient for the actual GEP
chain (struct→field→array→struct→field→array = 6 steps). If the GEP chain
exceeds max_depth, locations collapse to base object.

If needed, increase `max_depth` to 8 for the PTABen harness.

### Task E2: Type-Incompatible Cast Field Remapping (Deferred)

struct-incompab-typecast-nested casts between structs with different field
layouts. This requires type-aware field path remapping which is architecturally
complex. Defer to a future plan.

**Estimated LOC:** ~5

---

## Implementation Order

```
Phase A (memcpy constraints)        ← highest impact, 15 cases
  ↓
Phase B (global initializer wiring) ← high impact, 8+ cases
  ↓
Phase C (check cascading fix)       ← validate B fixes funptr tests
  ↓
Phase D (ptr-dereference)           ← investigate 2 cases
  ↓
Phase E (field depth)               ← small tweak, 2-5 cases
```

## Expected Results

| Phase | Root Cause | Cases Fixed | Remaining Unsound |
|-------|-----------|-----------|-------------------|
| A | Memcpy constraints | ~15 | ~23 |
| B | Global initializers | ~8 | ~15 |
| C | Cascading (via B) | ~3 | ~12 |
| D | Flow-insensitive accumulation | 0-2 | ~10-12 |
| E | Field depth | 0-2 | ~8-10 |

After all phases: **~25-30 of 38 unsound cases fixed**, from 49 Exact to ~75-80 Exact.

Remaining ~8-10 unsound cases will be:
- spec-equake (multi-level array indirection with loop-based indexing)
- struct-incompab-typecast-nested (type-incompatible casts, deferred)
- heap-linkedlist (loop-carried field updates, needs strong update + flow sensitivity)
- Other SPEC tests requiring deep interprocedural analysis

## Total Estimated LOC: ~230
