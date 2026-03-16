# Plan 142: Comprehensive Code Audit — Smells, Design, and Bugs

**Epic:** maintenance
**Status:** verified
**Created:** 2026-02-20
**Method:** 6-agent parallel exploration audit + 4-agent verification pass

## Summary

Full re-audit of the entire SAF codebase found **100 new findings** not covered by Plans 140/141.
After 4-agent verification pass reading actual code at all cited locations:
- **10 High** severity (confirmed implementation bugs with correctness impact)
- **26 Medium** severity (confirmed design issues, performance, or latent bugs)
- **61 Low** severity (code smells, duplication, minor issues; includes downgrades)
- **3 False Positives** removed (B6, NEW-6, Core-1)

### Verification Changes
- **B6** (PTA operand index): **FALSE POSITIVE** — LLVM GEP operands include all indices sequentially, so Field arm advancing `operand_idx` is correct for position tracking
- **NEW-6** (may_reach sink+sanitizer): **FALSE POSITIVE** — sanitizer IS checked before sink in the BFS loop; confirmed by two independent verifiers
- **Core-1** (Constant::Int truncation): **FALSE POSITIVE** — `BigInt` variant exists for values outside i64 range
- **NEW-1** (IDE Phase 2): **DOWNGRADED** High→Medium — Phase 1 composes jump functions transitively, so single-pass Phase 2 may be correct by design
- **CLI-1** (CLI stubs): **DOWNGRADED** High→Medium — missing feature, not a correctness bug
- **G1** (ICFG return edges): **DOWNGRADED** Medium→Low — consistent design choice, not a bug
- **S7** (HVN expansion): **DOWNGRADED** Medium→Low — latent risk only, not active bug
- **D3** (cumulative paths recursion): **DOWNGRADED** Medium→Low — SSA form prevents cycles in valid IR
- **FS-2** (inst_to_func_map): **DOWNGRADED** Medium→Low — `or_insert` is intentionally conservative
- **FE-B3** (comma splitting): **DOWNGRADED** Medium→Low — extremely rare in practice

Key themes (confirmed):
- **Indirect call handling is broken across 4 subsystems** (G2, G4, G5, G6 — all CONFIRMED)
- **Analysis variant correctness bugs** (FS-3, DDA-4 — both CONFIRMED HIGH)
- **Python bindings bypass core optimizations** (PY-1, CS-5 — both CONFIRMED HIGH)
- **Frontend data loss** (FE-B1, FE-B6 — both CONFIRMED HIGH)

---

## Verified High-Severity Findings (10)

### Indirect Call Pipeline (4 related bugs — all V:CONFIRMED)

**G2. ICFG `call_site_map` overwrites with LAST callee only — V:CONFIRMED**
- File: `icfg.rs:208-209`
- `resolve_indirect` calls `self.call_site_map.insert(site, (caller_block, callee_entry))` inside the targets loop. Only the last target survives.
- Fix: Map `InstId -> Vec<(BlockId, BlockId)>` for multi-target sites.

**G4. CallGraph `resolve_indirect` does NOT update `call_sites` map — V:CONFIRMED**
- File: `callgraph.rs:211-228`
- Updates `self.edges` only. `self.call_sites` still maps to `IndirectPlaceholder` after resolution.
- Fix: Update `call_sites` with resolved `Function` nodes.

**G5. SVFG Builder `CallIndirect` arg slice off-by-one — V:CONFIRMED**
- File: `svfg/builder.rs:436-440`
- Uses `&operands[1..]` — skips first real arg and includes callee pointer (last operand).
- Fix: Change to `&operands[..operands.len()-1]` per callee-LAST convention.

**G6. SVFG indirect calls always produce 0 targets after refinement — V:CONFIRMED**
- File: `svfg/builder.rs:471-481`
- `call_site_target()` returns `IndirectPlaceholder` (due to G4), match arm returns `vec![]`.
- Fix: Fall through to `callgraph.edges[caller_node]` when placeholder is returned. Also handle multi-target (current code only wraps single target in vec).

### Analysis Variant Bugs (2 — V:CONFIRMED)

**FS-3. `flow_sensitive_pts_at` reinterprets ValueId bits as LocId — V:CONFIRMED**
- File: `fspta/mod.rs:379`
- `LocId::new(value.raw())` — semantic ID space confusion. Makes `may_alias_at` always fall back to flow-insensitive PTA.
- Fix: Look up `pts.get(&value)` for the pointer's targets, then query df_in for those locations.

**DDA-4. Empty-context return traversal violates CFL-reachability — V:CONFIRMED**
- File: `dda/solver.rs:593-604`
- Comment says "wild return" — propagates to ALL predecessors when context is empty. Introduces spurious paths.
- Fix: Stop traversal on unmatched returns, or fall back to CI-PTA results.

### Frontend Bugs (2 — V:CONFIRMED)

**FE-B1. `convert_call_instruction` always sets `has_result = true` for void calls — V:CONFIRMED**
- File: `llvm/mapping.rs:1080-1104`
- All return paths unconditionally return `true` as has_result. Creates phantom ValueIds for void functions.
- Fix: Check callee return type, set `has_result = false` for void.

**FE-B6. `llvm.ptr.annotation` classified as `Skip` but returns a pointer — V:CONFIRMED**
- File: `llvm/intrinsics.rs:87-88`
- `Skip` emits no AIR instruction and no ValueId. Uses of the annotation result see undefined pointer.
- Fix: Change to `PassThrough` (same as `llvm.launder.invariant.group`).

### Python Binding Bugs (2 — V:CONFIRMED)

**CS-5. Python CSPTA context reconstruction uses wrong k — V:CONFIRMED**
- File: `saf-python/cspta.rs:206-207`
- Uses `hex_strings.len() as u32` as k limit. Can create contexts longer than solver's configured k.
- Fix: Use `self.config.k`.

**PY-1. DDA cache never reused across Python calls — V:CONFIRMED**
- File: `saf-python/dda.rs:139-150`
- Each query method calls `self.create_solver()` which creates a new `DdaPta` with empty cache.
- Fix: Store solver or cache in `PyDdaPtaResult`.

---

## Verified Medium-Severity Findings (26)

### Core (2 — V:CONFIRMED)
| ID | File | Title |
|----|------|-------|
| Core-2 | air.rs:44-50 | `Constant::Float` loses f32 precision on roundtrip |
| Core-10 | registry.rs:163-183 | Disabled pattern spec suppresses lower-priority enabled matches |

Core-1 (`Constant::Int`): **V:FALSE POSITIVE** — `BigInt` variant handles >i64.
Core-3 (`function()` fallback): **V:PARTIALLY** — real but Low risk; moved to Low.

### Frontend (3 — V:CONFIRMED)
| ID | File | Title |
|----|------|-------|
| FE-B4 | cha_extract.rs:431-456 | `extract_base_classes` may include self via string parsing |
| FE-S1 | mapping.rs:361-388 | `extract_global_name_from_repr` and `extract_global_ref_name` duplicate ~30 lines |
| FE-S2 | mapping.rs:1817-1858 | `parse_constant_gep` is a fragile hand-rolled LLVM IR string parser |

FE-B3 (comma splitting): **V:CONFIRMED but DOWNGRADED to Low** — extremely rare in practice.

### PTA (4 — V:CONFIRMED)
| ID | File | Title |
|----|------|-------|
| S1 | context.rs + solver.rs | Duplicated GEP path resolution logic |
| S2 | solver.rs + context.rs | Duplicated pointer-arithmetic merge (coverage gap) |
| S4 | config.rs:82-84 | `max_objects` config field never enforced |
| S5 | solver.rs:100-116 | `solve_with_config` discards `iteration_limit_hit` |
| D2 | solver.rs:1243-1247 | `load_loc_index` deduplication O(n) per entry |

S7 (HVN expansion): **V:PARTIALLY — DOWNGRADED to Low** — latent risk only.
D3 (recursion): **V:PARTIALLY — DOWNGRADED to Low** — SSA prevents cycles in valid IR.

### Graph (3 — V:CONFIRMED)
| ID | File | Title |
|----|------|-------|
| G3 | callgraph.rs:171-177 | `callers_of` is O(n) full scan with no reverse-edge map |
| G7 | svfg/optimize.rs:136-151 | `collect_reachable_past_removed` recursive without cycle guard |
| G13 | callgraph.rs:334-340 | `add_callback_edges` silently drops edges for functions not in edges map |

G1 (return edges): **V:CONFIRMED as design choice — DOWNGRADED to Low**.

### Dataflow (5 — includes downgrades from High)
| ID | File | Title |
|----|------|-------|
| NEW-1 | ifds/ide_solver.rs:633-676 | IDE Phase 2 single-pass (V:PARTIALLY — downgraded from High; Phase 1 composition may make this correct) |
| NEW-3 | taint.rs + typestate.rs | `matches_name` duplicated verbatim across IFDS problem files |
| NEW-4 | solver.rs + ide_solver.rs | IFDS/IDE solvers duplicate ~120 lines of helper maps |
| NEW-5 | typestate.rs | Sentinel `(FunctionId::new(0), BlockId::new(0))` as unknown location |
| CLI-1 | saf-cli/commands.rs:225-243 | All main CLI commands are stubs (V:CONFIRMED — downgraded from High; missing feature, not bug) |

NEW-6 (`may_reach` sink+sanitizer): **V:FALSE POSITIVE** — sanitizer checked before sink; confirmed by 2 verifiers.

### Advanced (4 — V:CONFIRMED)
| ID | File | Title |
|----|------|-------|
| CS-1 | cspta/context.rs:36 | `push` uses O(n) `remove(0)` for k-limiting |
| CS-2 | cspta/solver.rs:1249 | `find_or_approximate_location` does 3 full linear scans |
| CS-3 | cspta/solver.rs:1207 | `process_global_location` scans entire cs_pts per global location change |
| WASM-2 | saf-wasm/lib.rs:329 | `func_id_to_node` map is tautological (maps id→id) |
| MI-1 | module_index.rs:80-86 | ModuleIndex missing function parameters in `value_to_inst` |

FS-2 (inst_to_func_map): **V:FALSE POSITIVE (mostly)** — `or_insert` is intentionally conservative; downgraded to Low.

---

## Low-Severity Findings (60)

### Core (10): Core-4 through Core-9, Core-11 through Core-18
Code smells: redundant guard in `FunctionSpec::merge`, `parse_hex_id` accepts short-form IDs, `function_by_name`/`global_by_name` O(n) scans, duplicate `AirFunctionData` deserialization struct, no schema_version validation, stray `is_default` location, duplicate pattern spec warning never emitted, `discovery_paths` non-deterministic, `TaintSpec::merge` allows unbounded duplicates, `SchemaError::Yaml` re-wrapped losing type, empty `error`/`deterministic` modules.

### Frontend (4): FE-B8, FE-S3 through FE-S8
`convert_invoke_instruction` ignores `has_result`, `inst_counter` naming misleading, `decompose_arrays: bool` should be enum, `HEAP_ALLOC_FUNCTIONS` linear scan, `pending_instructions` not drained at block end, `air_json.rs` global counters, `extract_function_symbol` stub.

### PTA (6): S3, S6, D1, D4, D5, B1, B3, B4, B5
`extract_instruction` god function, magic 50K SCC trigger, sequential counter LocIds, `PtaContext` silently resets, unnecessary `.min()` in HVN, precompute Index+Field gap, LCD subset check, spurious function field locations, `solve_bitvec` discards limit flag.

### Graph (6): G8 through G12, G14
SVFG pass-through phi loses direct edges, DefUse phi double-counting, `toposort` redundant self-loop check, CFG `BlockId::new(0)` sentinel, SVFG edge_type naming inconsistency, dead `defuse` field.

### Dataflow (8): NEW-2, NEW-7 through NEW-14
Dead sanitizer check in taint source arm, branch refinement duplication in absint, O(n) worklist contains, recursive DFS stack overflow risk, `solve_function_with_params` duplicates 200 lines, `flows()`/`taint_flow()` duplicate BFS, stray docstring, typestate `normal_edge_fn` operand mismatch, `param_derived` single-block limitation.

### Advanced (6): CS-4, FS-1, DDA-1 through DDA-3, MI-1, CLI-1, CLI-2, MSSA-1, WASM-1, WASM-3
Eager location cloning, inflated diagnostics, brute-force allocation check, full param scan, array heuristic magic number, ModuleIndex missing params, CLI stubs, WASM stringly-typed config, merge hardcodes "cfg".

---

## Prioritized Implementation Phases (post-verification)

### Phase 1: Indirect Call Pipeline Fix (G2, G4, G5, G6)
**Impact:** Fixes a chain of 4 CONFIRMED bugs that make indirect call handling broken across ICFG, CallGraph, and SVFG. Single most impactful fix cluster — restores correctness for all indirect call analysis.

1. G4: Update `call_sites` map in `resolve_indirect` with resolved Function nodes
2. G2: Change `call_site_map` to `Vec<(BlockId, BlockId)>` for multi-target sites
3. G5: Fix `CallIndirect` arg slice to `&operands[..operands.len()-1]`
4. G6: Fall through to `callgraph.edges` when `call_site_target` returns placeholder; handle multi-target

### Phase 2: Analysis Variant Correctness (FS-3, DDA-4)
**Impact:** Fixes CONFIRMED fundamental correctness bugs in FSPTA and DDA.

1. FS-3: Rewrite `flow_sensitive_pts_at` to use `pts.get(&value)` then query df_in for targets
2. DDA-4: Stop traversal on empty-context unmatched Return edges

### Phase 3: Frontend & Spec Correctness (FE-B1, FE-B6, Core-10)
**Impact:** Fixes CONFIRMED silent data corruption in void call handling, intrinsic classification, and spec lookup.

1. FE-B1: Check callee return type, set `has_result = false` for void
2. FE-B6: Change `llvm.ptr.annotation` from Skip to PassThrough
3. Core-10: Change disabled pattern match from `return None` to `continue`

### Phase 4: Python Binding Fixes (CS-5, PY-1)
**Impact:** Restores correctness/performance in Python bindings.

1. CS-5: Use `self.config.k` for context reconstruction
2. PY-1: Store DDA cache in `PyDdaPtaResult` for reuse across calls

### Phase 5: Performance Hot Spots (D2, CS-1, CS-2, CS-3, G3)
**Impact:** Fixes CONFIRMED O(n^2) patterns in solver hot paths and CSPTA.

1. D2: Change `load_loc_index` values from `Vec<usize>` to `FxHashSet<usize>`
2. CS-1: Use `VecDeque` or rotate for k-limiting in `CallSiteContext::push`
3. CS-2: Add `find_by_obj_path` to LocationFactory
4. CS-3: Add reverse index `value_to_contexts` in CSPTA solver
5. G3: Add `reverse_edges` map to CallGraph

### Phase 6: Duplication Cleanup (deferred)
**Impact:** Reduces maintenance burden by consolidating ~500 lines of duplicated code.

- S1/S2: Extract shared GEP path resolution
- NEW-3: Extract shared `matches_name` helper
- NEW-4: Extract shared `IcfgIndex` from IFDS/IDE solvers
- NEW-7/NEW-10: Extract shared fixpoint iteration in absint
- NEW-11: Extract generic BFS in value-flow queries

### Phase 7: Remaining Medium/Low (deferred)
All remaining medium and low severity items including: NEW-1 (IDE Phase 2 — investigate if Phase 1 composition makes it correct), CLI-1 (implement CLI commands), S4 (enforce or remove max_objects), S5 (propagate iteration_limit_hit), NEW-5 (replace sentinel with Option), MI-1 (add param_value_to_func to ModuleIndex), G7 (add visited set to SVFG optimize recursion), and all Low items.
