# Plan 067: Fix Remaining 5 Unsound Cases in basic_c_tests

## Problem Statement

5 unsound oracle checks remain in PTABen `basic_c_tests` after Plan 066.
Current results: 66 Exact, 13 Sound, 22 ToVerify, **5 Unsound**, 8 Skip (114 checks).

| # | Test File | Oracle | Expected | Got | Root Cause |
|---|-----------|--------|----------|-----|------------|
| 1 | ptr-dereference1 | `MUSTALIAS(c,d)` | MustAlias | Partial | RC1: CI-PTA flow-insensitive accumulation |
| 2 | ptr-dereference3 | `MUSTALIAS(s,&z)` | MustAlias | Partial | RC1: CI-PTA flow-insensitive accumulation |
| 3 | spec-equake | 1 of 5 `NOALIAS` | NoAlias | Unknown/May | RC2: Heap array multi-level indirection |
| 4 | spec-gap | `MAYALIAS(*s, &IntComm)` | MayAlias | Unknown | RC3: External allocator + pointer arithmetic + deep deref chain |
| 5 | spec-mesa | `NOALIAS(p,q)` in `begin()` | NoAlias | May | RC4: FP dispatch + interprocedural parameter pollution |

## Detailed Root Cause Analysis

### RC1: ptr-dereference1 and ptr-dereference3 — CI-PTA Accumulation, FS-PTA Returns Unknown

**ptr-dereference1.c:**
```c
int a, b, *c, *d;
c = &a;           // Addr: c → loc(a)
d = &a;           // Addr: d → loc(a)
MUSTALIAS(c, d);  // At this point: c→{a}, d→{a} ← SHOULD be Must
c = &b;           // Addr: c → loc(b)
MAYALIAS(c, d);   // At this point: c→{b}, d→{a} ← SHOULD be May
NOALIAS(&b, d);   // &b→{b}, d→{a} ← SHOULD be No
```

**CI-PTA behavior:** Flow-insensitive — unions all assignments. `c → {loc(a), loc(b)}`, `d → {loc(a)}`. Query `MUSTALIAS(c,d)`: `{a,b}` vs `{a}` → `{a} ⊂ {a,b}` → **Partial** (one is proper subset of other). This is the correct CI-PTA result — it cannot report MustAlias because it doesn't distinguish program points.

**ptr-dereference3.c:**
```c
int *s, *r, t, z;
s = &t;           // Addr: s → loc(t)
r = &z;           // Addr: r → loc(z)
s = r;            // Copy: s = r → s → loc(z)
MUSTALIAS(s, &z); // CI: s→{t,z} vs &z→{z} → Partial (not Must)
```

Same pattern: `s` accumulates both `{t}` and `{z}`, so CI-PTA gives Partial.

**Why combined CS-PTA + FS-PTA doesn't fix this:**
- **CS-PTA (`may_alias_any`):** Uses CI summary (union across all contexts). Since both are in `main()` (single context), CS-PTA reduces to CI-PTA behavior: `s → {t, z}`. Reports **Partial**.
- **FS-PTA:** At -O0, `c`, `d`, `s`, `r` are all stored to alloca slots and loaded back. The oracle arguments are the *loaded* values from alloca. FS-PTA tracks flows on the SVFG, but the issue is that FS-PTA's top-level `pts` map mirrors CI-PTA's result for values that are loaded from alloca. The strong update in FS-PTA applies to *store nodes on the SVFG*, killing old values at the SVFG level. But when the oracle queries use the *loaded* ValueId, FS-PTA's `pts` map for that loaded value will show all values that could flow to it across any execution path. If the SVFG merges both store→load paths (alloca `c` has two stores: `&a` and `&b`), the load's points-to set at the SVFG level is `{a, b}`. **FS-PTA may correctly kill at the SVFG node level, but the `pts` map (exported for queries) may still contain the union.**
- **Result: Combined returns Partial, CI fallback also Partial.**

**Fundamental limitation:** The MUSTALIAS oracle at a *specific program point* is checking a flow-sensitive property. CI-PTA and CS-PTA (with CI summary) are structurally incapable of answering this. FS-PTA *should* handle it, but the current implementation may not export per-program-point query results — it exports a global `pts` map.

**Fix approach:** Enable FS-PTA program-point-aware queries. `FlowSensitivePtaResult` has a `may_alias_at()` method, and `df_in`/`df_out` maps carry per-SVFG-node points-to sets. The fix is:
1. Map the oracle instruction's ValueIds to SVFG nodes
2. Query `df_in`/`df_out` at those SVFG nodes instead of the global `pts` map
3. In `FsPts`, use the per-node data rather than the global summary

**Difficulty:** Medium. The infrastructure exists (`may_alias_at`, `df_in/df_out`), but the PTABen harness would need to pass program-point information (instruction IDs) down to the alias query, and the `FsPts` wrapper would need SVFG node mapping. This is a non-trivial integration change (~150-200 LOC).

**Alternative fix:** In the `FsPts::may_alias()` function, instead of checking the global `pts` map, iterate over the per-value SVFG nodes and find the most precise result across nodes. For loads from a single alloca, the last store node's `df_out` gives the flow-sensitive answer. This is simpler but requires SVFG node→ValueId mapping.

**Simpler alternative:** Change the combined PTA to check if FS-PTA's `df_out` at any store of the relevant alloca proves a singleton result, and use that to refine the global `pts` answer.

---

### RC2: spec-equake — Multi-Level Heap Array Indirection

**Pattern:**
```c
double ***disp = malloc(3 * sizeof(double**));
double ***K = malloc(3 * sizeof(double**));
// ... nested allocation loops ...
NOALIAS(disp, K);           // Oracle 1: globals should be NoAlias → PASSES
NOALIAS(disp, v);           // Oracle 2 → PASSES
NOALIAS(disp[1], K[Anext]); // Oracle 3 → FAILS (1 of 5 unsound)
NOALIAS(disp[1], v[i]);     // Oracle 4 → check
NOALIAS(disp[1][col], v[i]);// Oracle 5 → check
```

According to the Plan 066 session log, 4 of 5 spec-equake oracles were fixed. The remaining 1 unsound oracle is likely one of the deeper indirection checks (`disp[1]` vs `K[Anext]` or `disp[1][col]` vs `v[i]`).

**Root cause analysis:**
- `disp` is stored in a global variable alloca. `disp[1]` requires: load global → GEP to element 1 → load element → get pointer.
- `K[Anext]` requires: load global → GEP with *variable index* `Anext` → load element.
- With variable index `Anext`, the GEP produces a field path with `IndexExpr::Unknown` (under ConstantOnly sensitivity) or a collapsed field path.
- The loaded values from `disp[1]` and `K[Anext]` are heap-allocated `double**` pointers. `disp`'s inner malloc is at a *different call site* from `K`'s inner malloc, so they produce different `ObjId`s. If PTA correctly tracks through the load chain, `disp[1] → malloc_disp_inner` and `K[Anext] → malloc_K_inner`, which are disjoint. NoAlias should follow.
- **But:** At -O0, the chain is: `%disp_alloca = alloca ptr` → `store @malloc_outer, %disp_alloca` → `%disp_val = load %disp_alloca` → `%gep = GEP %disp_val, 0, 1` → `%inner = load %gep` → then `%inner` is what the oracle checks against.
- The problem is the intermediate loads. `%disp_val` (the outer malloc result) points to the outer heap object. Then `GEP %disp_val, 0, 1` gives field 1 of the outer heap object. Then `load` from that field gives whatever was stored there (the inner malloc result from the nested loop: `disp[1] = malloc(5000*sizeof(double*))`.
- **The store `disp[1] = malloc(...)` in the init loop** uses a variable index `disptplus`. At -O0 this becomes: `%idx = load %disptplus_alloca` → `%gep = GEP %disp_val, 0, %idx` → `store %inner_malloc, %gep`. The store goes to a variable-indexed location. Later, the oracle reads from constant index 1. With collapsed index sensitivity, these map to the same `Unknown` index location. But the *stored* value includes all three inner mallocs from the loop iterations (they all store to `disp[*]`). This means `load disp[1]` gets the union of all `disp[0]`, `disp[1]`, `disp[2]` inner mallocs — but they're all the *same* abstract heap object (single malloc call site inside the loop body).
- Similarly, `K[Anext]` gets the same-site abstract heap object for K's inner mallocs.
- **These are different call sites** (one for `disp`'s inner malloc at line 34, one for `K`'s inner malloc at line 44). So they produce different ObjIds. **This should give NoAlias.**
- The failure might be that the variable-indexed GEP chain doesn't connect the load to the store (different field paths due to constant vs variable index), causing `Unknown` instead of NoAlias.

**Fix approach:** This requires connecting variable-indexed stores to constant-indexed loads through the same collapsed array index. When `IndexSensitivity::ConstantOnly` is used and a GEP has a variable index, it collapses to `Unknown`. A store with `GEP %disp, 0, %idx` (Unknown index) should put the value at the collapsed field location. A load from `GEP %disp, 0, 1` (constant index 1) should *also* check the collapsed field location. Currently, constant index 1 resolves to a *specific* field location (`Field { index: 1 }`), which is different from the collapsed `Index(Unknown)` location.

The fix: When a constant-indexed load doesn't find its exact field location populated, fall back to check the collapsed `Index(Unknown)` location. This requires the `find_or_approximate_location` logic to also try the Unknown-index collapsed location as an approximation.

**Difficulty:** Hard. Changing `find_or_approximate_location` to consider collapsed-index locations risks over-approximation elsewhere. This is a precision/soundness tradeoff for heap array handling.

---

### RC3: spec-gap — External Allocator + Pointer Arithmetic + Deep Deref Chain

**Code flow:**
```c
char *SyGetmem(long size);  // External allocator (no body)

void InitGasman() {
    HdFree = (TypHandle)SyGetmem(SyMemory);  // HdFree = external alloc
    FreeHandle = (TypHandle)((TypHandle*)((FreeHandle)->ptr));
}

TypHandle NewBag() {
    d = ((TypHandle*)((HdFree)->ptr)) + needed/(sizeof(TypHandle)) - 1;
    s = ((TypHandle*)((HdFree)->ptr)) - 1;
    e = (FirstBag-1);
    while (e <= s) *d-- = *s--;
    MAYALIAS(*s, &IntComm);  // ← THE ORACLE
    // ...
}

void InstIntFunc(void (*func)()) {
    TypHandle hdDef = NewBag();
    *(void(**)())((TypHandle*)((hdDef)->ptr)) = func;  // stores func into hdDef->ptr
}

int main() {
    InitGasman();             // HdFree = SyGetmem(...)
    InstIntFunc(IntComm);     // stores IntComm into hdDef->ptr
}
```

**The oracle:** `MAYALIAS(*s, &IntComm)` — checks if `*s` (dereferenced pointer in the memory pool) may alias with `&IntComm` (address of the function).

**Why this is hard:**
1. `SyGetmem` is an **external function with no body**. SVF has a summary (`EFT_ALLOC`) for it, but SAF doesn't recognize `SyGetmem` as an allocator. Without modeling it, `HdFree` has no points-to set after the call → all downstream dereferences of `HdFree->ptr` produce `Unknown`.
2. Even if `SyGetmem` is modeled as an allocator, the pointer arithmetic `((TypHandle*)((HdFree)->ptr)) + needed/(sizeof(TypHandle)) - 1` involves symbolic arithmetic that PTA can't track.
3. The function `InstIntFunc` stores `IntComm` into `hdDef->ptr` through a complex cast chain. For `*s` to alias `&IntComm`, PTA would need to trace: `s` → dereferenced memory pool → same pool that `InstIntFunc` wrote `IntComm` into.
4. The SVF comment in the source says `SyGetmem` is a "summarized lib function" — SVF knows it's an allocator. SAF's spec system could handle this with a YAML spec for `SyGetmem`, but this is a single-test-specific function.

**Fix approach:** Add a function spec for `SyGetmem` (role: allocator) to the spec registry, either as a YAML file or inline in the PTABen harness. This would give `HdFree` a heap object to dereference. However, even with that fix, the pointer arithmetic and deep dereference chain may still produce Unknown due to unmodeled arithmetic offsets.

**Difficulty:** Hard. The test exercises: (1) external allocator modeling, (2) pointer arithmetic with symbolic offsets, (3) deep pointer-to-pointer dereference chains with casts. Even SVF needs its summary database to handle this. Fixing (1) alone is insufficient — (2) and (3) would still likely fail.

---

### RC4: spec-mesa — Function Pointer Dispatch + Interprocedural Parameter Pollution

**Code flow:**
```c
void begin(int *p, int *q) { NOALIAS(p,q); }   // ← THE ORACLE
void end(int *p, int *q) { MAYALIAS(p,q); }
void render(int *p, int *q) { MAYALIAS(p,q); }

// ctx->Exec = {begin, end, render}
// ctx->API = ctx->Exec (struct copy)
// CC = ctx (global)

void draw(int *p, int *q, int *r) {
    (*CC->API.Begin)(p, q);           // calls begin(p,q)
    if (p) q = r;                      // q may become r
    (*CC->API.Render)(q, r);           // calls render(q_or_r, r)
    (*CC->API.End)(p, r);              // calls end(p, r)
    // else branch: (*CC->API.End)(q, p);  // calls end(q, p)
}

int main() {
    draw(&x, &y, &z);   // p=&x, q=&y, r=&z
}
```

**The failing oracle:** `NOALIAS(p, q)` inside `begin()`. Called as `begin(&x, &y)` from `draw()`. Since `&x` ≠ `&y`, this should be NoAlias.

**Why it fails:**
The function `begin` is called through a function pointer `(*CC->API.Begin)(p, q)`. For the oracle to pass:
1. PTA must resolve `CC->API.Begin` to the function `begin`.
2. Interprocedural constraints must link `draw`'s `p` (= `&x`) to `begin`'s `p`, and `draw`'s `q` (= `&y`) to `begin`'s `q`.
3. The alias query on `begin`'s parameters must return NoAlias.

**The problem is parameter pollution.** `begin`, `end`, and `render` all have the same signature `(int*, int*)`. If CG refinement resolves `CC->API.Begin`, `.End`, and `.Render` to the correct functions, then:
- `begin` is called with `(p, q)` = `(&x, &y)`: parameters are distinct → NoAlias ✓
- `render` is called with `(q_or_r, r)`: parameters may be `(&y,&z)` or `(&z,&z)` → May ✓
- `end` is called with `(p, r)` = `(&x, &z)` or `(q, p)` = `(&y_or_z, &x)` → May ✓

But if CG refinement **over-resolves** (resolves all three indirect calls to *all three* functions), then `begin`'s parameter `p` gets values from: `(&x, &y_or_z, &x)` and `q` gets `(&y, &z, &z_or_x)`. This pollutes `begin`'s parameters with values from all call sites, causing May instead of No.

**Root cause chain:**
1. `CC->API.Begin` is loaded through: global `CC` → dereference → field `API` → field `Begin` → load function pointer.
2. `CC->API` was set via `ctx->API = ctx->Exec` (memcpy/struct copy). This makes `API.Begin` point to the same location as `Exec.Begin` = `begin`.
3. But `CC->API.End` and `CC->API.Render` point to `end` and `render` respectively.
4. If the GEP field paths for `API.Begin` (field 0), `API.End` (field 1), `API.Render` (field 2) are correctly distinguished, each indirect call should resolve to exactly one target.
5. **If field paths collapse** (e.g., max_depth too low, or the struct copy merges all fields), then all three indirect calls resolve to `{begin, end, render}`, polluting parameters.

The struct `api_table` has 3 function pointer fields. The struct copy `ctx->API = ctx->Exec` is compiled as memcpy. Our memcpy modeling uses `Copy(dst_ptr, src_ptr)`, making `API_ptr` point to `Exec`'s object. GEPs on `API_ptr` then traverse `Exec`'s field structure. If the field paths are correctly maintained (3 distinct fields at indices 0, 1, 2), each field load resolves to one function.

**Likely issue:** The struct copy makes `API_ptr → Exec_obj`. Then `GEP API_ptr, 0, 0` (Begin field) → `Exec_obj, field 0` → load → `begin`. And `GEP API_ptr, 0, 1` (End field) → `Exec_obj, field 1` → load → `end`. This SHOULD work if field sensitivity is correctly handled through the memcpy Copy constraint.

But wait: the memcpy is at the *struct level* (`ctx->API = ctx->Exec`). In LLVM IR, this is `memcpy(GEP ctx, 0, 0, GEP ctx, 0, 1, sizeof(api_table))` — copying from `ctx->Exec` (field 1 of `context`) to `ctx->API` (field 0 of `context`). The Copy constraint makes the *API sub-struct pointer* point to the *Exec sub-struct's object/location*. But the function pointer stores were done on `Exec` via `init_exec_pointers(&ctx->Exec)` which stores `begin` at `Exec.Begin` = field path `[1, 0]` of `context`, `end` at `[1, 1]`, `render` at `[1, 2]`.

After memcpy, `API_ptr` (field path `[0]`) points to the same object as `Exec_ptr` (field path `[1]`). So loading from `API.Begin` = `GEP API_ptr, 0, 0` gives field `[0]` relative to the Exec sub-object, which is `[1, 0]` relative to the `context` object = `begin`. This should work.

**Alternative hypothesis:** The indirect call resolution doesn't resolve correctly because `CC->API.Begin` goes through a *global variable* and multi-level dereference chain that PTA doesn't fully track. `CC` is a global pointer, set via `change_context(ctx)` → `CC = ctx`. If `CC`'s points-to set doesn't include the heap-allocated context, the chain breaks.

To trace: `main` → `make_current(mesa)` → `change_context(mesa->ctx)` → `CC = ctx`. Then `draw(&x, &y, &z)` → `(*CC->API.Begin)(p,q)`.

For `CC` to point to the context: `CC` is a global. `change_context` stores its argument to `CC`. The argument is `mesa->ctx` = the result of `create_context()` = a heap-allocated `context`. This requires:
1. `mesa->ctx` field write/read tracked
2. `create_context()` return value tracked
3. Interprocedural arg→param for `make_current` → `change_context`
4. Store `CC = ctx` tracked

This is a deep interprocedural chain (3 function calls deep). If any link breaks, `CC` has empty/Unknown pts → the entire function pointer resolution fails → falls back to conservative resolution → parameter pollution.

**Fix approach:** Two potential fixes:
A. **Verify and fix the interprocedural chain** — ensure all links in `main() → make_current() → change_context() → CC = ctx` work. CG refinement should resolve all direct calls. The heap allocation from `create_context()` should produce a return value constraint. This is a debugging task.
B. **Context-sensitive indirect call resolution** — ensure that when `CC->API.Begin` resolves to `begin`, the parameter constraints use the *specific call site's* arguments, not the union across all call sites where `begin` appears. This is a context-sensitivity issue — CS-PTA with k=2 should handle it, but the CI summary (used by `may_alias_any`) unions contexts.

**Difficulty:** Hard. Requires both correct multi-level-deref function pointer resolution AND context-sensitive parameter tracking to avoid pollution.

---

## Fix Categories by Difficulty

### Potentially Fixable (2 cases, ~200 LOC)

| Case | Test | Approach | Risk |
|------|------|----------|------|
| RC1 | ptr-dereference1/3 | FS-PTA program-point-aware query in combined PTA | Medium — infrastructure exists but integration is non-trivial |

### Hard / Deferred (3 cases)

| Case | Test | Why Hard |
|------|------|----------|
| RC2 | spec-equake | Variable-index vs constant-index location mismatch for heap arrays; requires collapsed-index fallback in load resolution |
| RC3 | spec-gap | External allocator not modeled + symbolic pointer arithmetic + deep cast chain |
| RC4 | spec-mesa | Deep interprocedural function pointer resolution chain (3 levels) + context-sensitive parameter isolation |

---

## Phase A: FS-PTA Program-Point Queries for ptr-dereference1/3 (RC1)

### Approach

The core issue is that `FsPts::may_alias()` uses a global `pts` map that contains the flow-insensitive union. The fix is to make the FS-PTA query flow-sensitive by using the per-SVFG-node `df_in`/`df_out` maps.

### Task A1: Map Oracle ValueIds to SVFG Nodes (~50 LOC)

**Files:** `crates/saf-bench/src/combined_pta.rs`, `crates/saf-bench/src/ptaben.rs`

For each alias oracle, we know the instruction ID where the oracle is called (from the `call_inst_id` in the expectation). The oracle's arguments are loaded from alloca *before* the oracle call. We need to find the SVFG *load node* corresponding to each argument, then query `df_in` at that node.

**Approach:**
1. Store the SVFG node→ValueId mapping in `FsPts` alongside the global `pts` map.
2. When querying alias at a specific program point, look up the closest SVFG node for each ValueId and use its `df_in`/`df_out` set.

**Challenge:** The ValueIds used in oracle arguments are the *loaded* values from alloca (SSA values at the call site). Mapping these to SVFG nodes requires the SVFG node→ValueId association that `FlowSensitivePtaResult` already tracks via `node_value` map.

### Task A2: Add Flow-Sensitive Query to FsPts (~60 LOC)

**File:** `crates/saf-bench/src/combined_pta.rs`

Add a method `may_alias_flow_sensitive(p, q, fs_result)` that:
1. Finds all SVFG nodes associated with `p` and `q`
2. For each node, checks `df_in` or `df_out` to get the flow-sensitive points-to set
3. Takes the intersection (most precise) across SVFG nodes for each value
4. Computes alias result from the flow-sensitive sets

### Task A3: Wire Flow-Sensitive Query into PTABen Harness (~40 LOC)

**File:** `crates/saf-bench/src/ptaben.rs`

Pass the `FlowSensitivePtaResult` (not just the global `pts` map) into the combined result, and use it for flow-sensitive queries.

### Task A4: Verify with ptr-dereference1/3

Run filtered PTABen and confirm both tests pass.

**Estimated LOC:** ~150
**Risk:** Medium. FS-PTA may not have nodes for the specific ValueIds used in the oracle (the oracle arguments are SSA values, not alloca addresses). If the FS-PTA tracks at the alloca level but oracle queries use loaded values, there's a mapping gap. Need to investigate the actual SVFG structure for ptr-dereference1.

---

## Phase B: spec-equake Collapsed-Index Fallback (RC2) — Deferred

### Why Deferred

The remaining spec-equake oracle involves variable-indexed GEPs in loops storing to `disp[disptplus]` and constant-indexed loads from `disp[1]`. The store uses an `Unknown` collapsed index; the load uses a concrete `Field { index: 1 }`. These create different locations in the PTA factory, so the store and load don't connect.

Fixing this requires a principled approach to collapsed-index fallback: when a constant-indexed load finds no value at its specific field location, it should also check the collapsed `Index(Unknown)` location as a fallback. This is architecturally sound but requires careful handling to avoid over-approximation for struct fields (where constant indices should remain precise).

**Potential approach:** Add a `try_collapsed_fallback` flag to `find_or_approximate_location` that, when the exact field path isn't found, tries replacing the last constant index with `Unknown`. Only enable this for array-type GEPs (not struct field GEPs).

**Estimated LOC:** ~80
**Risk:** High — could cause over-approximation in struct field handling if not carefully scoped.

---

## Phase C: spec-gap External Allocator (RC3) — Deferred

### Why Deferred

`SyGetmem` is a test-specific external allocator. Even adding a function spec for it (role: allocator) would only fix the first link in the chain. The subsequent pointer arithmetic `((TypHandle*)((HdFree)->ptr)) + needed/(sizeof(TypHandle)) - 1` involves symbolic arithmetic that PTA cannot model. The deep dereference chain through type casts makes this one of the hardest patterns.

SVF handles this through its ExtAPI database (`EFT_ALLOC` summary for `SyGetmem`). SAF would need either:
1. An equivalent external function API database, or
2. A way to import SVF's function summaries

Neither is in scope for this plan.

---

## Phase D: spec-mesa Multi-Level FP Resolution (RC4) — Deferred

### Why Deferred

The test requires:
1. Tracking `CC` (global) through 3-level interprocedural call chain: `main → make_current → change_context → CC = ctx`
2. Resolving `CC->API.Begin` to `begin` through field-sensitive dereference + struct copy
3. Context-sensitive parameter tracking to isolate `begin`'s parameters from `end`/`render`'s

Even if function pointer resolution works (resolving each `API.field` to one function), the CI summary used for alias queries unions parameters across all call contexts. Context-sensitive queries (`may_alias` with specific `CallSiteContext`) would need the oracle to specify which context to query in — but the PTABen oracle has no context annotation.

The proper fix requires:
- **Per-call-site parameter isolation** in the CS-PTA query: when checking `begin(p,q)`, only consider the parameter values from call sites that actually call `begin`, not from sites calling `end` or `render`.
- This is equivalent to using CS-PTA's per-context query rather than the CI summary (`may_alias_any`).

**Estimated LOC:** ~200 (for context-aware oracle queries)
**Risk:** High — architectural change to how PTABen queries alias results.

---

## Implementation Order

```
Phase A (FS-PTA flow-sensitive query) ← 2 cases, medium difficulty
  ↓
Phases B/C/D: Deferred
```

## Expected Results

| Phase | Cases Fixed | New Exact | Remaining Unsound |
|-------|-----------|-----------|-------------------|
| Baseline | — | 66 | 5 |
| A (if successful) | 2 | 68 | 3 |
| B (deferred) | 1 | 69 | 2 |
| C (deferred) | 0-1 | 69 | 2 |
| D (deferred) | 0-1 | 69-70 | 1-2 |

### Realistic Assessment

- **ptr-dereference1/3:** Fixable with FS-PTA integration, but requires non-trivial SVFG→ValueId mapping work. ~60% chance of success in a single session.
- **spec-equake (1 oracle):** Requires collapsed-index fallback. Architecturally feasible but needs careful scoping. Deferred.
- **spec-gap:** Fundamentally requires external function modeling + pointer arithmetic. Not fixable without significant new infrastructure. Deferred.
- **spec-mesa:** Requires both correct FP resolution AND context-sensitive parameter queries. Both may already work partially. Investigation needed before committing to a fix.

## Total Estimated LOC for Phase A: ~150
