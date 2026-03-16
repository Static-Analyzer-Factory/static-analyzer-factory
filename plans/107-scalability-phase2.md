# Plan 107: Scalability Phase 2 — FSPTA Overhaul

**Epic:** Scalability
**Status:** done
**Date:** 2026-02-12
**Depends on:** Plan 106 (Phase 1 quick wins)

## Context

Phase 1 (Plan 106) achieved 3-8x speedup on CI-PTA. The FSPTA solver is now the dominant bottleneck — bunzip2 spends 21.8s in fspta alone. The solver uses `BTreeMap<LocId, BTreeSet<LocId>>` for df_in/df_out state, clones entire maps on every node visit, and doesn't leverage the PtsSet trait abstraction (BitVec/BDD).

Current fspta solver hot paths (`fspta/solver.rs`, ~340 lines):
- `points_to: BTreeMap<ValueId, BTreeSet<LocId>>` — top-level pts
- `df_in/df_out: BTreeMap<SvfgNodeId, BTreeMap<LocId, BTreeSet<LocId>>>` — 3-level nested BTreeMap
- Every `process_node` call clones `df_in.get(&node)` — O(n*m) per node
- Every `propagate_indirect` extends BTreeSets — O(n log n) per edge
- `worklist: BTreeSet<SvfgNodeId>` — O(log n) pop

## Tasks

### Task A: SVFG Optimization Pass (SVFGOPT)

**New file:** `crates/saf-analysis/src/svfg/optimize.rs`
**Modifies:** `crates/saf-analysis/src/svfg/mod.rs`

**Goal:** Remove 30-50% of redundant SVFG nodes before FSPTA solving, reducing graph traversal cost proportionally.

**Changes:**
1. Create `pub fn optimize(svfg: &Svfg) -> Svfg` that:
   - **Pass-through MemPhi elimination:** Remove MemPhi nodes that have exactly 1 incoming indirect edge and 1 outgoing indirect edge (identity pass-through). Reconnect predecessor directly to successor.
   - **Single-operand Phi merging:** Remove MemPhi nodes where all incoming edges come from the same source node. Replace with direct edge from source to all phi successors.
   - **Dead node removal:** Remove nodes with no outgoing edges AND no incoming edges (isolated).
2. Return optimized Svfg with diagnostics (nodes_removed, edges_rewritten).
3. Wire into `FsSvfgBuilder::build()` — call `optimize(&svfg)` before building FsSvfg.
4. Add module to `svfg/mod.rs`: `pub mod optimize;`

**Tests:** Unit tests for each optimization pattern. Integration test verifying node count reduction on store-load-phi patterns.

### Task B: FSPTA PtsSet Generics

**Modifies:** `crates/saf-analysis/src/fspta/solver.rs`, `crates/saf-analysis/src/fspta/mod.rs`

**Goal:** Make the FSPTA solver generic over the `PtsSet` trait, enabling BitVec O(n/64) unions instead of BTreeSet O(n log n). Expected 2-5x speedup on set operations.

**Changes:**
1. **Type aliases in `mod.rs`:**
   - `DfPointsTo` stays as a type alias but becomes generic: `type DfPointsTo<P> = IndexMap<LocId, P, nohash_hasher::BuildNoHashHasher<u128>>;`
   - `FlowSensitivePtaResult` keeps `BTreeMap<ValueId, BTreeSet<LocId>>` for public API (convert at boundaries)
2. **Generify solver functions:**
   - `solve_flow_sensitive_generic<P: PtsSet>(...)` — main generic entry point
   - `process_node<P>`, `process_stores<P>`, `process_load<P>`, `propagate_direct<P>`, `propagate_indirect<P>` — all generic
   - Use `P::new()`, `P::union()`, `P::is_empty()`, `P::contains()`, `P::iter()` instead of BTreeSet methods
3. **Dispatch function:**
   - `solve_flow_sensitive(...)` auto-selects representation based on `config.pts_config`:
     - Small programs (<10K locs): BTreePtsSet
     - Medium (10K-100K): BitVecPtsSet
     - Large (>100K): BddPtsSet
   - Uses same pattern as CI-PTA solver (`solve_with_config`)
4. **Shared indexer:** For BitVec/BDD, create indexer from Andersen pre-analysis locations and share across all PtsSet instances.

### Task C: FSPTA Hot Path Optimization (IndexMap + Diff Propagation)

**Modifies:** `crates/saf-analysis/src/fspta/solver.rs`, `crates/saf-analysis/src/fspta/mod.rs`

**Goal:** Replace O(log n) BTreeMap lookups with O(1) IndexMap, and implement diff-based propagation to avoid redundant work. Combined with im-rc for cheap DfPointsTo cloning.

**Changes:**
1. **IndexMap migration in solver:**
   - `points_to: BTreeMap<ValueId, P>` → `IndexMap<ValueId, P>`
   - `df_in: BTreeMap<SvfgNodeId, DfPointsTo<P>>` → `IndexMap<SvfgNodeId, DfPointsTo<P>>`
   - `df_out: BTreeMap<SvfgNodeId, DfPointsTo<P>>` → `IndexMap<SvfgNodeId, DfPointsTo<P>>`
   - `inst_to_func: BTreeMap<ValueId, FunctionId>` → `IndexMap<ValueId, FunctionId>`
2. **Diff-based indirect propagation:**
   - In `propagate_indirect()`, track which objects actually changed (not just "any change")
   - Only propagate changed objects to successors, not the entire df_out map
   - Reduces per-edge propagation from O(|objects|) to O(|changed_objects|)
3. **Add `im-rc` for persistent DfPointsTo cloning:**
   - Add `im-rc = "15"` to workspace `Cargo.toml`
   - Replace `DfPointsTo<P>` inner map: `BTreeMap<LocId, P>` → `im::OrdMap<LocId, P>` for O(log n) structural cloning
   - This is the single biggest win: in `process_stores`, `let in_map = df_in.get(&node).cloned()` goes from O(n*m) to O(log n)
4. **Worklist optimization:**
   - Replace `BTreeSet<SvfgNodeId>` with `VecDeque<SvfgNodeId>` + `HashSet` for O(1) insert/pop

### Task D: FsSvfg Builder Optimization

**Modifies:** `crates/saf-analysis/src/fspta/builder.rs`

**Goal:** Speed up FsSvfg construction by eliminating O(n) linear scans.

**Changes:**
1. **Index store instructions:** In `locs_for_store()`, currently scans ALL instructions in ALL functions to find one InstId. Pre-build `BTreeMap<InstId, (ValueId, ValueId)>` (already exists as `store_map`) and use it for O(log n) lookup.
2. **Cache PTA queries:** Pre-compute `pta.points_to(ptr)` results to avoid redundant PTA lookups in `compute_indirect_edge_objects`.

## Implementation Strategy

- **Task A + Task D**: Fully independent of each other and of B/C. Can be done in parallel.
- **Task B**: Core generics change. Must be done before Task C (which uses generic types).
- **Task C**: Depends on Task B for generic DfPointsTo type.
- **Order:** A ‖ D ‖ B → C → wire + test

## Agent Team Plan

1. **Agent `svfg-opt`**: Task A (SVFG optimization — new file, independent)
2. **Agent `builder-opt`**: Task D (FsSvfg builder optimization — builder.rs only)
3. **Agent `fspta-solver`**: Tasks B+C (PtsSet generics + IndexMap + diff propagation — solver.rs + mod.rs)
4. **Main agent**: Dependencies (im-rc, indexmap for fspta), wiring, integration testing

## Success Criteria

1. All existing tests pass (1473 Rust + 72 Python)
2. PTABen: 2251 Exact, 69 Unsound (no regression)
3. CruxBC: bunzip2 fspta time reduced from ~21.8s baseline
4. `make fmt && make lint` clean
5. SVFG node count reduction ≥20% on bunzip2
