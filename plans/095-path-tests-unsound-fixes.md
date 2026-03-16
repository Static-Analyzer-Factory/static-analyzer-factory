# Plan 095: Path Tests Unsound Fixes

**Epic:** E41 (PTABen Precision Campaign)
**Target:** Fix 3 unsound path_tests cases (path20 × 1, path8 × 2)
**Baseline:** 2241 Exact, 79 Unsound

## Context

Three unsound results in `path_tests`:
1. `path20.ll`: floor=MustAlias, got=Unknown (1 check)
2. `path8.ll`: floor=NoAlias, got=Partial (2 checks)

## Root Cause Analysis

### path20: undef pointer has no points-to set

**C source:** `a = b` where `b` is uninitialized, then `*a = &q; obj = *b`. Since
`a` and `b` alias, `obj` should be `&q` (MustAlias).

**LLVM IR:** `b` is uninitialized → compiled to `undef`. Both store and load use
`ptr undef` as the pointer operand:
```llvm
store ptr %q, ptr undef     ; *undef = &q
%0 = load ptr, ptr undef    ; %0 = *undef
```

**SAF behavior:** The LLVM frontend maps `ptr undef` to a ValueId (cached by repr
string, so both instructions share the same ValueId). But no Addr constraint is
created for it — `undef` has no alloca/global/heap. With empty pts, the store
can't write and the load can't read → `%0` has no pts entry → `Unknown`.

**Fix:** In the per-path flow-sensitive solver (`value_origin.rs`), extend the
Store handler to create a synthetic location when the destination pointer has an
empty pts. Similar to the existing "symbolic object for uninitialized loads" at
lines 842-851, but applied to Store destinations. This lets the store write to the
synthetic location, and the subsequent load through the same pointer reads it back.

### path8: per-path combiner counts unreachable paths

**C source:** Correlated phi store — `*p = q` where `(p,q)` is `(&b,&d)` on path 1
or `(&c,&e)` on path 2. So `*b = &d` and `*c = &e` are exclusive. The NOALIAS
checks verify that `*c ≠ &d` and `*b ≠ &e`.

**Flow-insensitive:** Phi merges uncorrelate the store: both `b` and `c` get
`{&d, &e}` → Partial (instead of NoAlias).

**Per-path fallback:** The function has 3 branches → 8 paths. 7 paths reach the
query point and correctly report NoAlias. 1 path (the loop back-edge `if.else6→c1`)
never reaches `if.end8` where the alias checks are, so the per-path solver returns
`Unknown` for it. `combine_path_alias_results` doesn't filter Unknown (unreachable)
results, so the combination becomes May instead of No → still unsound.

**Fix:** In `combine_path_alias_results`, filter out `Unknown` results before
combining. Paths where the query point is unreachable shouldn't influence the
combined alias result.

## Implementation Plan

### Phase A: Fix path8 combiner (2 checks, Easy)

**File:** `crates/saf-analysis/src/pta/value_origin.rs`

1. In `combine_path_alias_results`, filter out `AliasResult::Unknown` entries
   before combining. If all non-Unknown results agree, return that result.
   If all results are Unknown (no path reaches the query), return Unknown.

2. Add unit test: combining `[No, No, No, Unknown]` → `No`.

### Phase B: Fix path20 undef pointer (1 check, Medium)

**File:** `crates/saf-analysis/src/pta/value_origin.rs`

1. In `process_instruction_flow_sensitive`, for `Operation::Store`:
   When `ptr_pts` is empty (the store destination has no allocation), create a
   synthetic location for the pointer and add it to pts. Then proceed with the
   store normally. This mirrors the existing "uninitialized load" synthetic object
   mechanism (lines 842-851) but applied to stores.

   ```rust
   // Before existing Store handling, add:
   if ptr_pts.is_empty() {
       // Synthetic location for unknown pointers (e.g., undef)
       let syn_obj = ObjId::new(ptr.raw());
       let syn_loc = factory.get_or_create(syn_obj, FieldPath::empty());
       pts.entry(ptr).or_default().insert(syn_loc);
       // Re-read ptr_pts after inserting
       ptr_pts = pts.get(&ptr).cloned().unwrap_or_default();
   }
   ```

2. Add unit test: store through undef pointer + load through same undef →
   per-path solver returns MustAlias.

### Phase C: Verify and regression gate

1. Run `make fmt && make lint` — fix any clippy issues.
2. Run `make test` — all Rust + Python tests pass.
3. Run PTABen path_tests filtered benchmark:
   - path8: both NOALIAS checks → Exact or Sound
   - path20: MUSTALIAS check → Exact or Sound
4. Run full PTABen to verify no regressions.

## Expected Outcome

- path_tests unsound: 3 → 0 (−3)
- Total unsound: 79 → 76 (−3)
- Total exact: 2241 → 2244 (+3) or at least +3 non-unsound
