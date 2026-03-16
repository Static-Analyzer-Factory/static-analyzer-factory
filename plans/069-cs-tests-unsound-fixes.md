# Plan 069: cs_tests Unsound Fixes

## Status: done (Phases A-C; Phase D deferred)
## Epic: E37 (Context-Sensitive Alias Precision)

## Current State

PTABen cs_tests: 27 tests, 94 oracle checks → **49 Exact, 11 Sound, 2 ToVerify, 32 Unsound**.

The 32 unsound checks span 11 test files:

| Test | Unsound | Pattern | Actual | Expected |
|------|---------|---------|--------|----------|
| cs0 | 4 | Return pass-through identity fn | Partial | Must/No |
| cs4 | 4 | Return + double-pointer store | Partial/May | Must/No |
| cs7 | 2 | Double-pointer `*q = *p` | No | Must |
| cs11 | 4 | Double-pointer `*a = b` | Partial | Must/No |
| cs17 | 4 | Chained bar+foo, double-pointer | Partial | Must/No |
| cs19 | 4 | Double-pointer with overwrite | Partial/May | Must/No |
| cs20 | 4 | Double-pointer via two callers | Partial | Must/No |
| funcpoiner | 2 | Function pointer indirect call | Partial | Must |
| recur5 | 1 | Recursive with flow-sensitive store | Partial | Must |
| recur6 | 1 | Recursive with global overwrite | Partial | Must |
| recur8 | 1 | Mutual recursion + malloc | Partial | Must |

## Root Cause Analysis

### Bug 1: Return propagation broadcasts to ALL callers (~4-8 unsound)

**File:** `crates/saf-analysis/src/cspta/solver.rs`, `collect_return_updates()` (line 767)

When a return value is updated in context [CS1], `collect_return_updates` iterates
ALL call sites to this function (`calls_to[func]`) and propagates to ALL of them.
It computes the caller context via `cv.ctx.pop()`, discarding the popped call site
ID (line 790: `let (popped, _) = cv.ctx.pop()`).

**Consequence:** If `foo` is called at CS1 and CS2, and `foo`'s return in context [CS1]
has pts={&a}, that {&a} also leaks to call2 (the CS2 call site). Both call results
get the union of both contexts' return values.

**Affected tests:** cs0 (identity function called twice), funcpoiner (indirect call twice).

### Bug 2: Cross-context Store/Load invisibility (~24 unsound)

**File:** `crates/saf-analysis/src/cspta/solver.rs`, `loc_pts` field (line 352)

`loc_pts` is keyed by `(LocId, CallSiteContext)`. When `foo` stores to a caller's
memory in context [CS1], the store goes to `loc_pts((caller_loc, [CS1]))`. But when
main later loads from `caller_loc` in context [], it reads from
`loc_pts((caller_loc, []))` which is a DIFFERENT entry that doesn't contain the callee's store.

**Example (cs7):** `foo(a,b)` does `*q = *p` where p→x_alloca, q→y_alloca. The load
`*p` reads `loc_pts((x_alloca, [CS1]))` which is EMPTY because `x = &x1` was stored
in main's context [].

**Affected tests:** cs4, cs7, cs11, cs17, cs19, cs20, funcpoiner (all use double-pointer
store/load patterns through function calls).

### Bug 3: No heap cloning across contexts (~4 unsound)

**File:** `crates/saf-analysis/src/cspta/solver.rs`, `seed_function_in_context()` (line 823)

`seed_function_in_context` seeds the SAME `LocId` for each alloca regardless of context.
This means callee-local allocations (like `-O0` retval allocas) share a single abstract
location across all calling contexts.

Even after fixing Bug 2 with context-insensitive loc_pts, callee-local allocas would
merge: `retval_loc` content = union of all callers' data.

**Example (cs0):** `foo(p)` and `foo(q)` both store to `retval_loc`. Without cloning,
the load gets {&a, &b} in both contexts.

**Affected tests:** cs0 (identity function using retval alloca at -O0).

### Bug 4: SCC context collapse for recursive functions (~3 unsound)

**File:** `crates/saf-analysis/src/cspta/solver.rs`, `callee_context()` (line 805)

Recursive functions are detected via SCC analysis and collapsed to empty context.
ALL instances of a recursive function share the same context, losing all
context-sensitivity. The MUSTALIAS checks in recur5/6/8 require distinguishing
specific recursive call instances.

**Affected tests:** recur5, recur6, recur8.

### Bug 5: Flow-insensitive accumulation (underlying limitation)

Andersen's inclusion-based analysis never kills previous stores. In `cs7`:
`y = &y1; foo(a,b)` where foo overwrites y — even with all CS-PTA fixes, Andersen's
retains {y1} in y's pts. The path-sensitive solver handles this as a fallback.

## Fix Strategy

The fixes are ordered to maximize incremental testability.

### Phase A: Fix return propagation (Bug 1) — ~15 LOC

**File:** `crates/saf-analysis/src/cspta/solver.rs`

In `collect_return_updates()`, use the popped call site ID to filter which call
sites receive the return value:

```rust
fn collect_return_updates(&self, cv: &CtxValue, v_pts: &P, updates: &mut Vec<(CtxValue, P)>) {
    // ... existing lookup ...
    for &idx in site_indices {
        let site = &self.call_sites[idx];
        let Some(dst) = site.dst else { continue; };

        let caller_ctx = if self.scc_functions.contains(&func_id) {
            CallSiteContext::empty()
        } else {
            let (popped, popped_site) = cv.ctx.pop();
            // Only propagate to the call site that created this context
            if let Some(site_id) = popped_site {
                if site.inst_id != site_id {
                    continue;  // Skip: this call site didn't create this context
                }
            }
            popped
        };

        updates.push((CtxValue { value: dst, ctx: caller_ctx }, v_pts.clone()));
    }
}
```

**Expected impact:** Fixes cs0's return-value merging. Tests that only fail due
to return broadcasting will improve.

### Phase B: Context-insensitive loc_pts with heap cloning (Bug 2 + Bug 3) — ~120 LOC

**File:** `crates/saf-analysis/src/cspta/solver.rs`

This is the core fix. Two changes:

**B1: Make loc_pts context-insensitive for non-cloned locations.**

Change `loc_pts: BTreeMap<(LocId, CallSiteContext), P>` to use a hybrid approach:
- `global_loc_pts: BTreeMap<LocId, P>` — context-insensitive for all locations
- `cloned_loc_pts: BTreeMap<(LocId, CallSiteContext), P>` — context-sensitive for cloned callee locals

Store/Load logic:
- **Store:** If target loc is a cloned loc, write to `cloned_loc_pts((loc, ctx))`.
  Also write to `global_loc_pts(original_loc)` for cross-context visibility.
  If target is non-cloned, write to `global_loc_pts(loc)`.
- **Load:** If target loc is a cloned loc, read from `cloned_loc_pts((loc, ctx))`.
  If empty/missing, fall back to `global_loc_pts(original_loc)`.
  If target is non-cloned, read from `global_loc_pts(loc)`.

**B2: Heap cloning for callee-local allocations.**

When `seed_function_in_context` is called for a non-SCC function in a non-empty context:
- For each Addr constraint `(ptr, loc)` in the function:
  - Create a new cloned `LocId` (derive from original + context hash)
  - Record mapping: `clone_map: BTreeMap<(LocId, CallSiteContext), LocId>`
  - Seed `pts((ptr, ctx)) = {cloned_loc}` instead of original loc

When processing constraints in a context, resolve LocIds through the clone map:
- If a loc has a clone for the current context, use the clone
- Otherwise use the original (caller/global location)

**Implementation detail:** The clone map must also handle field locations. When
a LocId is cloned, all its field-sensitive children must also be cloned. This
requires iterating the factory's locations for the same ObjId and creating
corresponding clones.

**Expected impact:** Fixes all double-pointer tests (cs4, cs7, cs11, cs17, cs19, cs20)
and the return-value alloca merging (cs0). Combined with Phase A, should fix 26-29
of 32 unsound checks.

### Phase C: Path-sensitive solver improvements for remaining cases — ~30 LOC

**File:** `crates/saf-analysis/src/pta/value_origin.rs`

After Phases A+B, some tests may still be unsound due to flow-insensitive accumulation
(Bug 5). The path-sensitive solver already handles this via strong updates and callee
inlining. Two improvements:

**C1: Allow one level of nested CallDirect in callee inlining.**

Currently, `process_instruction_flow_sensitive` skips ALL `CallDirect` in callee bodies
(line 570). For tests like cs17 that chain `bar` → `foo`, this means foo's body
isn't processed when bar is inlined. Allow one level of recursion:

```rust
// Add depth parameter
fn process_instruction_flow_sensitive(
    inst: &Instruction,
    module: &AirModule,
    ...,
    inline_depth: usize,  // new parameter
) -> bool {
    // ...
    Operation::CallDirect { callee } => {
        if inline_depth < 2 {  // allow nesting up to depth 2
            // inline callee body with depth+1
        }
    }
}
```

**C2: Process callee blocks in topological order for strong updates.**

Currently callee blocks are iterated in arbitrary order. For strong updates to work
correctly, blocks must be processed in CFG order (entry first, then successors).

**Expected impact:** Fixes remaining flow-insensitive accumulation cases.

### Phase D: Recursive function improvements (Bug 4) — ~60 LOC (optional)

**File:** `crates/saf-analysis/src/cspta/solver.rs`

For recur5/6/8, instead of full SCC collapse to empty context, allow bounded
context depth for self-recursive functions:

- For non-mutual recursion (self-loop only): allow up to k contexts before
  collapsing. This means the first k recursive calls get distinct contexts,
  and deeper calls collapse.
- For mutual recursion (multi-function SCC): keep collapsing to empty context
  (too complex for bounded unrolling).

Implementation: In `callee_context()`, instead of checking `scc_functions.contains(callee)`,
check if the current context already contains k instances of the same call site.
If so, return empty; otherwise push normally.

**Expected impact:** Fixes recur5, recur6. recur8 (mutual recursion) may remain
unsound.

## Expected Results

| Phase | Fixes | Remaining |
|-------|-------|-----------|
| Before | — | 32 unsound |
| Phase A | cs0 return merging (partial) | ~30 unsound |
| Phase B | cs0-cs20 + funcpoiner (Bugs 2+3) | ~5 unsound |
| Phase C | Flow-insensitive stragglers | ~3 unsound |
| Phase D | recur5, recur6 | ~1 unsound |

**Target: 32 → 1-3 unsound** (recur8 mutual recursion likely remains).

## Files Modified

- `crates/saf-analysis/src/cspta/solver.rs` — Phases A, B, D (~200 LOC)
- `crates/saf-analysis/src/pta/value_origin.rs` — Phase C (~30 LOC)
- `crates/saf-analysis/src/cspta/context.rs` — Add context hash helper (~10 LOC)

## Test Plan

1. Run `cargo test -p saf-analysis` after each phase
2. Run PTABen cs_tests filter after each phase to verify improvement
3. Run full PTABen suite to check for regressions
4. Run full `make test` before committing
