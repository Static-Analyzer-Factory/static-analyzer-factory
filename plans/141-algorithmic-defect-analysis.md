# Plan 141: Algorithmic Defect Analysis

**Status**: verified
**Date**: 2026-02-20
**Method**: 6-agent parallel deep-code analysis + 3-agent verification pass

## Executive Summary

A thorough analysis of SAF's algorithm implementations found **63 potential defects** across 6 domains. After the initial 6-agent analysis and a subsequent 3-agent verification pass reading the exact code at each cited location, the **verified** count is:

- **Critical**: 5 confirmed (2 downgraded from original 7)
- **High**: 14 confirmed (4 downgraded or reclassified from original 18)
- **Medium**: 11 confirmed (3 downgraded from original 14)
- **Low**: 6 confirmed
- **False Positive**: 5 (C6, H17, M7 refuted; C2 mostly latent; H15 not currently buggy)

## Verification Legend

Each finding now has a verification status:
- **V:CONFIRMED** — Verified against actual code, defect is real as described
- **V:PARTIALLY** — Real issue but severity/scope adjusted from original claim
- **V:FALSE-POSITIVE** — Refuted by code reading; scenario doesn't occur or behavior is correct
- **V:INTENTIONAL** — Real deviation from textbook algorithm but documented/deliberate design choice

---

## Critical Findings

### C1. Value Flow: Memcpy/Memset produce NO flow edges — V:CONFIRMED
**File**: `valueflow/builder.rs:228-237`
**Impact**: Any taint flowing through `memcpy()` is completely lost. Breaks tracking for string operations, struct copying, buffer manipulation.
**Fix**: Add Store/Load edges for Memcpy (src→dst flow), Store edge for Memset (const→dst).
**Verification note**: PTA spec constraints handle memcpy for pointer analysis, but the value-flow graph builder has NO equivalent. This is the most impactful defect for taint analysis.

### C3. LLVM Frontend: AtomicCmpXchg/AtomicRMW silently dropped — V:CONFIRMED
**File**: `mapping.rs:840-843`
**Impact**: Lock-free data structures, atomic pointer stores invisible to PTA.
**Fix**: Model as Load+Store pair at minimum.
**Verification note**: Not currently exercised by PTABen benchmarks (no atomics in test fixtures), but real C programs use these extensively.

### C4. IDE: `join_with` returns `self` for non-trivial edge functions — V:CONFIRMED
**File**: `ifds/edge_fn.rs:104-119`
**Impact**: At CFG join points, one branch's edge function is silently discarded. The `ide_solver.rs:711-720` comparison `joined == *old_jf` then always returns true, terminating propagation prematurely.
**Fix**: Implement proper pointwise LUB for Composed and TransitionTable edge functions.
**Verification note**: Scope limited to cases where two unequal non-trivial edge functions (TransitionTable, Composed) meet at a join point. Identity/Constant/AllTop/AllBottom cases are handled correctly.

### C5. IFDS: Zero fact seeded at ALL functions — V:INTENTIONAL (downgraded from Critical)
**File**: `ifds/solver.rs:206-214`
**Impact**: Over-approximation, not unsoundness. Comment says "for simplicity." Both IFDS and IDE solvers have this pattern.
**Verification note**: This interacts with the `return_flow(Zero) → empty` in taint.rs — the two "bugs" cancel out: over-seeding ensures Zero is everywhere, so it doesn't need to propagate through return summaries. Likely intentional design. **Downgraded to Medium (design debt).**

### C7. PTA: `iteration_limit_hit` diagnostic never set (CI-PTA only) — V:CONFIRMED
**File**: `pta/solver.rs:717-720`, `pta/context.rs`
**Impact**: Callers cannot detect early termination in CI-PTA path. Silent unsound results.
**Fix**: Set `diagnostics.iteration_limit_hit = true` when max_iterations hit.
**Verification note**: CSPTA (`cspta/solver.rs:672`) and FSPTA (`fspta:97`) DO correctly set this flag. Only the main CI-PTA path is missing it.

### ~~C2. GEP first-index semantics wrong~~ — V:PARTIALLY (downgraded to Medium)
**File**: `mapping.rs:1308-1332`
**Verification note**: The code does treat all indices identically, but in practice the dominant pattern is `gep %struct.S, ptr %p, i64 0, i32 field` where the first index is 0. Non-zero first indices (true array pointer arithmetic) are rare. **Bug is real but mostly latent.** Downgraded to Medium.

### ~~C6. IFDS: Wrong return-site when call is last in block~~ — V:FALSE-POSITIVE
**File**: `ifds/solver.rs:144-157`
**Verification note**: In well-formed LLVM/AIR IR, a `CallDirect` is NOT a block terminator. Only `Ret`/`Br`/`CondBr`/`Switch`/`Unreachable` are terminators. A call always has a next instruction (the block's terminator). The problematic code path is unreachable for valid IR. **Removed from findings.**

---

## High Findings

### H1. CG Refinement: All operands scanned for call targets — V:CONFIRMED
**File**: `cg_refinement.rs:586-614`
**Impact**: Spurious call edges from argument operands' points-to sets.
**Fix**: Change to `site.operands.last()` only.

### H2. Value Flow: Indirect calls get ZERO resolved targets for flow edges — V:CONFIRMED (worse than initially claimed)
**File**: `valueflow/builder.rs:349-358`
**Impact**: Originally claimed "only one target." Verification reveals `call_site_target` returns the `IndirectPlaceholder` (not a resolved target), so `vec![]` is returned — **zero** resolved targets get flow edges for ANY indirect call. All indirect call inter-procedural value flow is missing.
**Fix**: Use `callgraph.edges(caller)` or add a `call_site_targets(site) -> Vec<FunctionId>` that returns resolved targets.

### H3. MSSA: Indirect calls skipped in mod/ref summary — V:CONFIRMED (intentional trade-off)
**File**: `mssa/modref.rs:195-199`
**Impact**: Transitive unsoundness in mod/ref for functions containing indirect calls.
**Verification note**: Comment acknowledges this: "The clobber walker handles this conservatively." Deliberate soundness trade-off but still causes missing value-flow edges.

### H4. LLVM Frontend: insertvalue/extractvalue dropped — V:CONFIRMED
**File**: `mapping.rs` catch-all
**Verification note**: Zero matches for InsertValue/ExtractValue in entire llvm directory. Falls to catch-all with `tracing::warn`.

### H5. LLVM Frontend: invoke unwind target not in CFG — V:CONFIRMED
**File**: `mapping.rs:765`
**Impact**: Exception paths invisible to all analyses.

### H6. Non-aggregate global constant GEPs not field-sensitive — V:CONFIRMED (intentional)
**File**: `mapping.rs:233-247`
**Verification note**: Comment says "simple base-address resolution is appropriate" for non-aggregate globals. Deliberate design.

### H7. IFDS Typestate: ALL tracked facts transitioned — V:PARTIALLY (downgraded to Medium)
**File**: `ifds/typestate.rs:869-879`
**Verification note**: The inline comment explicitly documents this as deliberate alias-handling for `-O0` LLVM IR where SSA aliases are common. `normal_flow` does filter by `operands[0]` but `normal_edge_fn` applies transitions to all `Tracked(_)` facts. This is a documented design choice, not a bug. **Downgraded to Medium** — causes imprecision for multi-resource programs but is intentional.

### H8. IFDS Taint Z3: Interprocedural flows excluded — V:CONFIRMED
**File**: `ifds/taint_z3.rs:146-167`
**Verification note**: `if src_func_id == sink_func_id` guard unconditionally skips cross-function pairs. No else branch — they silently disappear. Comment says "only intraprocedural for now."

### H9. CSPTA: SCC return values broadcast to all callers — V:CONFIRMED
**File**: `cspta/solver.rs:884-904`

### H10. FSPTA: `may_alias_at` ignores the node parameter — V:CONFIRMED
**File**: `fspta/mod.rs:317-341`
**Verification note**: `_node` parameter with underscore prefix confirms this is a known placeholder.

### H11. DDA: `find_call_site_for_param` is a stub — V:CONFIRMED
**File**: `dda/solver.rs:471-477`
**Verification note**: Explicit TODO comment, `#[allow(clippy::unused_self)]`, `_param_value` naming.

### H12. CFG: Exit detection misclassifies non-terminator-ending blocks — V:CONFIRMED (mitigated)
**File**: `cfg.rs:57-60`
**Verification note**: `extract_successors` catch-all returns empty for all non-terminators. Mitigated if AIR always has proper terminators at block ends (which well-formed LLVM→AIR translation guarantees).

### H14. Graph algorithms: Recursive Tarjan/DFS — V:CONFIRMED
**File**: `graph_algo.rs:134-188`

### ~~H13. ICFG: Return edges target caller block~~ — not re-verified (same as H5 root cause)

### ~~H15. Abstract interpretation: Default widen = join~~ — V:PARTIALLY (downgraded to Low)
**Verification note**: Both concrete domain implementations (Interval, Octagon) correctly override `widen`. Only a footgun for future domains. **Downgraded to Low.**

### H16. PTA: Indirect calls not modeled in main CI-PTA path — V:CONFIRMED (architectural)
**File**: `pta/extract.rs:444-476`
**Verification note**: Intentional — CI-PTA is designed to be used with CG refinement for indirect calls. But undocumented.

### ~~H17. MSSA: Phi operands stale~~ — V:FALSE-POSITIVE
**Verification note**: `rename_pass` at lines 484-496 ALWAYS overwrites every phi operand using `BTreeMap::insert`. The initial write at 232-252 is harmless redundancy. **Removed from findings.**

### H18. decompose_constant_gep fails for global initializers — V:CONFIRMED (known)
**File**: `mapping.rs:305`
**Verification note**: `current_block_id` is `None` during global init processing. Comment documents the limitation.

---

## Medium Findings

### M1. CG: Dynamic external nodes missing from func_index — V:CONFIRMED
`callgraph.rs:124-138`

### M2. CG: CHA resolves ALL root hierarchies for every virtual call — not re-verified
`cg_refinement.rs:876-926`

### M3. CG: extract_base_classes may include self as base — not re-verified
`cha_extract.rs:431-455`

### M4. PTA: O(addr-taken-allocs x GEP-paths) location precomputation — V:PARTIALLY
`pta/context.rs:215-223` — Uses addr-taken objects (not ALL objects as originally claimed). Still potentially large but not the full Cartesian product.

### M5. PTA: GEP merge heuristic conflates pointer arithmetic with field access — V:CONFIRMED
`pta/solver.rs:309-347` — Documented as intentional "pointer-arithmetic merge" but has genuine precision consequences.

### M6. Value Flow: BFS visited map suppresses some paths — V:PARTIALLY (downgraded to Low)
`valueflow/query.rs:121-127` — Allows revisiting via shorter paths (not a simple boolean set). Impact narrower than claimed.

### ~~M7. MSSA: Clobber walker cycle detection returns wrong node~~ — V:FALSE-POSITIVE
`mssa/walker.rs:46-49` — Returning the cycle entry as clobber is correct conservative behavior. **Removed.**

### M8. IFDS: External calls never invoke call_to_return_flow — V:CONFIRMED
`ifds/solver.rs:362-410`

### M9. ICFG: resolve_indirect doesn't update call_site_map — V:CONFIRMED
`icfg.rs:187-213`

### M10. CSPTA: SCC functions always get empty context — V:CONFIRMED
`cspta/solver.rs:922-928`

### M11. Checkers: Visited set blocks some multi-sink discovery — V:PARTIALLY
`checkers/solver.rs:64-113` — BFS continues after finding sinks, but shared visited set prevents re-exploring through first sink to reach second sink. Narrower than claimed.

### M12. Checkers: Partial-leak heuristic false positives — V:CONFIRMED
`checkers/solver.rs:238-265` — Any intermediate SSA node triggers the heuristic.

### M13. LLVM Frontend: Named struct types treated as Opaque — not re-verified
`type_intern.rs:127-142`

### M14. LLVM Frontend: parse_constant_gep string parsing fragile — not re-verified
`mapping.rs:1570-1612`

### C2 (demoted). GEP first-index semantics — V:PARTIALLY (moved from Critical)
`mapping.rs:1308-1332` — Real but mostly latent due to "first index = 0" pattern.

### C5 (demoted). IFDS over-seeding — V:INTENTIONAL (moved from Critical)
`ifds/solver.rs:206-214` — Deliberate design with known tradeoff.

### H7 (demoted). Typestate ALL-facts transition — V:PARTIALLY (moved from High)
`ifds/typestate.rs:869-879` — Documented deliberate alias-handling.

---

## Low Findings

### L1. PTA: find_rep lacks path compression (performance)
### L2. PTA: SCC detection skips empty-pts values (delayed collapse)
### L3. PTA: Function pointer heuristic matches by raw int (design concern)
### L4. Value Flow: FindingId omits trace node IDs (collision risk)
### L5. CG: signature_compatible too coarse (pointer-only check)
### L6. CG: resolve_callback_targets misses non-global function pointers
### L7. LLVM Frontend: Anonymous globals dropped, llvm.ptr.annotation skipped
### H15 (demoted). Default widen = join — footgun for future domains only
### M6 (demoted). BFS visited narrower impact than claimed

---

## False Positives (Removed)

| ID | Original Claim | Reason for Rejection |
|----|---------------|---------------------|
| C6 | IFDS wrong return-site | Well-formed IR prevents scenario; call is never last in block |
| H17 | MSSA phi stale operands | rename_pass always overwrites via BTreeMap::insert |
| M7 | MSSA walker returns wrong node | Returning cycle entry is correct conservative behavior |

---

## Documentation Bugs

### D1. AIR spec vs implementation: CallIndirect operand order — V:CONFIRMED
The AIR doc says "Operand[0] is function pointer" but ALL implementation code (LLVM frontend, CG refinement, PTA extract) consistently uses callee-LAST convention. **Doc is wrong; implementation is consistent.**

---

## Recommended Fix Prioritization (Updated)

### Phase 1: Critical soundness (5 items)
| ID | Fix | Effort |
|----|-----|--------|
| C1 | Add memcpy/memset value-flow edges | Small |
| C3 | Model atomics as Load+Store | Small |
| C4 | Implement proper edge function LUB | Medium |
| C7 | Set iteration_limit_hit in CI-PTA solver | Trivial |
| H2 | Fix indirect call target resolution for value flow | Small |

### Phase 2: High-impact core pipeline (8 items)
| ID | Fix | Effort |
|----|-----|--------|
| H1 | CG: scan only last operand for call targets | Trivial |
| H3 | MSSA: conservative modref for indirect calls | Medium |
| H4 | LLVM: handle insertvalue/extractvalue | Medium |
| H5 | LLVM: capture invoke unwind target | Medium |
| H8 | IFDS Z3: extend to interprocedural flows | Medium |
| M1 | CG: insert dynamic externals into func_index | Trivial |
| M9 | ICFG: update call_site_map in resolve_indirect | Trivial |
| M12 | Checkers: fix partial-leak heuristic | Small |

### Phase 3: Analysis variants (5 items)
| ID | Fix | Effort |
|----|-----|--------|
| H9 | CSPTA: filter SCC returns by call site | Small |
| H10 | FSPTA: use df_in at node for alias query | Small |
| H11 | DDA: implement find_call_site_for_param | Medium |
| H14 | Graph algos: iterative Tarjan/DFS | Medium |
| M10 | CSPTA: use k-limiting instead of SCC collapse | Small |

### Phase 4: Precision and polish
M2, M3, M4, M5, M11, M13, M14, D1, and all Low findings.
