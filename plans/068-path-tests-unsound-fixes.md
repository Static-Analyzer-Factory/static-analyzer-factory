# Plan 068: Path-Tests Unsound Fixes

## Problem Statement

PTABen `path_tests` has **42 unsound** oracle checks across 19 test files (out of 90 total oracle checks). All but 2 failures are "expected=NoAlias, got=Partial" — SAF's flow/context-insensitive PTA merges pointer assignments from exclusive branches, producing overlapping points-to sets where path-sensitive reasoning would prove disjointness. The remaining 2 are "expected=MustAlias, got=Partial/No" from pointer convergence patterns (`a=b; *a=&q; obj=*b`).

## Current Results

```
path_tests: 10 Exact, 36 Sound, 2 ToVerify, 42 Unsound, 0 Skip (90 oracles)
```

## Root Cause Analysis

### Pattern 1: Branch-correlated pointer assignments (38 of 42 unsound)

```c
if (cond) { p = &a; q = &c; }     // Branch 1: (p→a, q→c)
else      { p = &b; q = &d; }     // Branch 2: (p→b, q→d)
*p = q;
NOALIAS(a, &d);   // True at runtime: branch 1 ⟹ a gets c, never d
                   // SAF: pts(a) ⊇ {c,d} because *p ∈ {&a,&b}, q ∈ {&c,&d}
```

CI/CS/FS-PTA all compute `pts(*p) = pts(&a) ∪ pts(&b)` and `pts(q) = {&c, &d}`. The store `*p = q` propagates `q`'s entire pts to ALL locations `p` may point to, losing the branch correlation. The result is `pts(a) = {c, d}` and `pts(b) = {c, d}`, causing `a` and `d` to have overlapping field paths → `Partial` instead of `NoAlias`.

**Affected tests:** path2 (2), path3 (3), path4 (2), path5 (2), path6 (2), path7 (2), path8 (2), path9 (2), path10 (1), path11 (2), path13 (2), path14 (3), path15 (1), path16 (2), path17 (1), path18 (3), path19 (6), path21 (2).

### Pattern 2: Pointer convergence / synchronization (4 of 42 unsound)

```c
// path3: p = q = &a; if(a) { p = q = &b; }
// *p = m; n = *q; MUSTALIAS(n, m)
// path20: a = b; *a = &q; obj = *b; MUSTALIAS(obj, &q)
// path21: *p = &g; b = *p; ... MUSTALIAS(r, q)
```

Here `p` and `q` are always assigned together (synchronized), so `*p = m; n = *q` should yield `n == m` (MustAlias). But SAF computes `pts(n) ⊇ pts(a) ∪ pts(b)` = multi-element set → `Partial` instead of `Must`.

**Affected tests:** path3 (1 MustAlias), path4 (1 MustAlias), path20 (1 MustAlias), path21 (1 MustAlias).

### Why existing path-sensitive infrastructure doesn't help

SAF has `PathSensitiveAliasChecker` (Plan 049/E29) with guard extraction, path enumeration, and Z3 feasibility. However:

1. **`alias_under_path()` is a stub** (line 287-290 of `path_sensitive.rs`): returns flow-insensitive alias result without filtering points-to sets by path conditions
2. **Not wired into PTABen**: the harness calls `pta_result.may_alias()` / `combined_pta.may_alias()` directly, never the path-sensitive checker
3. **The checker operates on global PTS**: Even if wired in, the global points-to sets already contain merged data from all branches — path conditions can't un-merge them

## Proposed Solution

### Approach: Conditional points-to set filtering via Z3

Instead of trying to make PTA itself path-sensitive (which would require a fundamentally different algorithm like SSA-based sparse analysis), we add a **post-hoc refinement stage** that filters the overly-conservative PTS for a specific alias query at a specific program point.

**Key insight:** For each `NOALIAS(a, d)` query at a program point Q:
1. Extract all dominating branch conditions reaching Q
2. For each `LocId L` in `pts(a)`, determine which branch assigns `a → L`
3. For each `LocId M` in `pts(d)`, determine which branch assigns `d → M`
4. Use Z3 to check if any path exists where BOTH `a → L` AND `d → M` are assigned
5. If no feasible path exists for ANY (L, M) overlap → prove `NoAlias`

For **MustAlias** queries:
1. Check that on ALL feasible paths, both pointers have the SAME singleton target
2. If every feasible path produces the same target → prove `MustAlias`

### Phase A: Value-Origin Tracking (~300 LOC)

**Goal:** For each pointer value, track which branch conditions control which points-to set entries.

**Task A1: `ValueOriginMap` data structure**

New module `crates/saf-analysis/src/pta/value_origin.rs`:

```rust
/// Maps (ValueId, LocId) → set of branch conditions under which this entry exists
pub struct ValueOriginMap {
    /// For each value, maps each location to the set of branch conditions
    /// that cause this value to point to this location.
    origins: BTreeMap<ValueId, BTreeMap<LocId, Vec<BranchCondition>>>,
}

pub struct BranchCondition {
    /// The comparison instruction (ICmp operand of CondBr)
    pub condition: ValueId,
    /// Which branch was taken (true/false)
    pub branch_taken: bool,
    /// Block where the branch occurs
    pub block: BlockId,
    /// Function containing the branch
    pub function: FunctionId,
}
```

**Task A2: Build `ValueOriginMap` from AIR**

Walk the AIR module's control flow to associate each Store instruction with its dominating branch conditions:

1. For each function, build CFG and dominator tree
2. For each Store `*p = q`, find all dominating `CondBr` terminators
3. Record: "value `p` gets location `L` from `q`'s pts under conditions [C1=true, C2=false, ...]"
4. For Phi nodes, each incoming value inherits the condition of its incoming edge

**Complexity:** O(|instructions| * |dominator_depth|) — same as existing guard extraction.

### Phase B: Path-Filtered Alias Query (~250 LOC)

**Goal:** Implement `alias_under_path()` properly using `ValueOriginMap`.

**Task B1: Implement path-filtered alias computation**

In `path_sensitive.rs`, replace the stub `alias_under_path()`:

```rust
fn alias_under_path(&self, p: ValueId, q: ValueId, path: &PathCondition) -> AliasResult {
    // Filter pts(p) to entries consistent with this path's conditions
    let p_filtered = self.filter_pts_for_path(p, path);
    let q_filtered = self.filter_pts_for_path(q, path);

    // Compute alias on filtered sets
    compute_alias_from_sets(&p_filtered, &q_filtered, self.locations)
}

fn filter_pts_for_path(&self, v: ValueId, path: &PathCondition) -> BTreeSet<LocId> {
    let Some(origins) = self.origin_map.get(v) else {
        return self.pts.get(&v).cloned().unwrap_or_default();
    };

    origins.iter()
        .filter(|(_, conditions)| conditions_consistent_with_path(conditions, path))
        .map(|(&loc, _)| loc)
        .collect()
}
```

**Task B2: Condition consistency check**

A `(ValueId, LocId)` entry is consistent with a path if none of its required branch conditions contradict the path's guard assignments. If the origin has condition `C=true` but the path has `C=false`, that entry is filtered out.

### Phase C: Wire into PTABen Harness (~150 LOC)

**Goal:** Use path-sensitive alias queries for `path_tests` oracle validation.

**Task C1: Locate oracle query points**

For each alias oracle call (e.g., `NOALIAS(a, d)`), determine:
- The block containing the oracle call
- The function containing it
- The oracle instruction's position (for dominator extraction)

These are already available via `extract_expectations()` in `ptaben.rs` — each `Expectation::Alias` stores the `ptr_a` and `ptr_b` ValueIds. We need to also store the oracle call's `BlockId` and `FunctionId`.

**Task C2: Update `Expectation::Alias` with location info**

Add `oracle_block: Option<BlockId>` and `oracle_function: Option<FunctionId>` fields to `Expectation::Alias`. Populate during `extract_expectations()` by recording the containing block/function of each oracle call instruction.

**Task C3: Path-sensitive alias validation**

In `validate_alias()`, when oracle location info is available:
1. Create `PathSensitiveAliasChecker` with the origin map
2. Call `may_alias_at(ptr_a, ptr_b, oracle_block, oracle_function)`
3. Use path-sensitive result instead of flow-insensitive

### Phase D: Handle Interprocedural Cases (~200 LOC)

**Goal:** Handle path19, path18, path10, path11 which involve function calls.

Several tests pass pointers through function calls where the branch correlation is:
- `main()` branches and sets `p` and `f` → calls `foo(p)` → `foo` stores `*p = f`
- The correlation is: `p→&q` and `f→&d` are on the SAME branch in `main()`
- Context-insensitive analysis merges both call sites of `foo()`

**Task D1: Caller-context guard propagation for alias queries**

Extend the origin map construction to cross function boundaries:
1. When processing a CallDirect, propagate the caller's dominating conditions to the callee's parameter origins
2. When a callee stores `*param = value`, the origin includes both the callee's local conditions AND the caller's conditions at the call site

This reuses the existing `interprocedural.rs` guard propagation infrastructure from Plan 032 (E18).

**Task D2: Context-sensitive origin merging**

For each call site, build a per-context origin map:
- Call site 1 in branch 1: `param → &q` with condition `C=true`
- Call site 2 in branch 2: `param → &r` with condition `C=false`

The callee's stores inherit these per-call-site conditions.

### Phase E: Handle MustAlias Convergence (~100 LOC)

**Goal:** Fix path3, path4, path20, path21 MustAlias cases.

These require proving that despite multiple branches, two pointers always end up pointing to the same location.

**Task E1: Path-unanimous MustAlias detection**

After computing alias results for all feasible paths (in `combine_path_results`):
- If ALL feasible paths produce `Must` → combined = `Must`
- This already works in `combine_path_results` when `all_same`

The key is making `alias_under_path()` return `Must` for convergence patterns:
- `a = b; *a = &q; obj = *b;` → on every path, `a` and `b` point to the same thing, so `*b` loads what `*a` stored → `Must`
- With filtered PTS: `pts_filtered(obj) = {&q}` and `pts_filtered(&q) = {&q}` → singleton equal → `Must`

## Expected Results

| Phase | Tests Fixed | Unsound Reduction |
|-------|------------|-------------------|
| A+B (intra-procedural) | path2-9, path13-17, path20 | ~28 → ~14 |
| C (wiring) | Same as A+B | harness integration |
| D (interprocedural) | path10, path11, path18, path19 | ~14 → ~2 |
| E (MustAlias convergence) | path3, path4, path20, path21 | ~2 → ~0 |

**Target:** 42 → ~0-5 unsound (from 10 to ~38-42 Exact).

The 2 `ToVerify` cases (path10 MayAlias→NoAlias, path4 MayAlias→NoAlias) should become Exact after path-sensitive analysis correctly proves NoAlias.

## Risks & Mitigations

1. **Performance:** Origin map construction is O(|insts| * |dom_depth|). For PTABen micro-benchmarks this is negligible. For large programs, the path-sensitive queries could be slow with many guards. **Mitigation:** Existing max_paths=16 limit, Z3 timeout=500ms.

2. **Soundness:** Filtering PTS entries can only REMOVE entries (more precise), never ADD them. If the filter is conservative (keeping entries when unsure), soundness is preserved. **Mitigation:** If origin tracking can't determine the condition for an entry, keep it (don't filter).

3. **Origin tracking through memory:** Stores like `*p = q` create indirect origins. The origin of the loaded value depends on what `p` points to AND what was stored. **Mitigation:** For Phase A/B, track origins through direct assignments and Phi nodes only. Memory-indirect origins get "unknown condition" (no filtering). Phase D extends to interprocedural.

4. **LLVM -O0 complexity:** Unoptimized code uses alloca+store+load for everything, adding indirection layers. **Mitigation:** The origin tracker follows store-load chains through allocas using the PTA result to resolve which alloca a load reads from.

## Implementation Order

A → B → C → test → D → E → test

Phases A+B+C form a self-contained improvement for intraprocedural cases. Phase D extends to interprocedural. Phase E is a small enhancement to the combination logic.

## LOC Estimate

~1000 LOC total (300 + 250 + 150 + 200 + 100).
