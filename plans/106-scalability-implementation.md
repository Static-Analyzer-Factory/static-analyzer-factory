# Plan 106: Scalability Phase 1 — Quick Wins Implementation

**Epic:** Scalability
**Status:** done
**Date:** 2026-02-11
**Depends on:** Plan 105 (research synthesis)

## Context

Plan 105 identified a 5-phase scalability roadmap. This plan implements Phase 1 (quick wins) targeting 3-8x cumulative speedup via 5 changes. Current baseline: bunzip2 27s total (21.8s fspta), bash/libcurl timeout.

## Tasks

### Task A: Fix CG Refinement PtsConfig Bug (1C)

**File:** `crates/saf-analysis/src/cg_refinement.rs:136`

**Bug:** `solve_with_config()` call hardcodes `&PtsConfig::default()` instead of `&config.pta_config.pts_config`. BitVec/BDD representations never activated in CG refinement loop.

**Fix:** Change line 136 from:
```rust
&PtsConfig::default(),
```
to:
```rust
&config.pta_config.pts_config,
```

One-line fix. No API changes. Existing tests must still pass.

### Task B: SmallVec for Constraint Index

**Files:** `crates/saf-analysis/src/pta/constraint_index.rs`, workspace `Cargo.toml`, `crates/saf-analysis/Cargo.toml`

**Goal:** Replace `Vec<usize>` with `SmallVec<[usize; 4]>` in `ConstraintIndex` map values. ~70% of constraint lists have 1-3 entries, avoiding heap allocation.

**Changes:**
1. Add `smallvec = "1"` to workspace `Cargo.toml` `[workspace.dependencies]`
2. Add `smallvec = { workspace = true }` to `crates/saf-analysis/Cargo.toml` `[dependencies]`
3. In `constraint_index.rs`:
   - Add `use smallvec::{smallvec, SmallVec};`
   - Change all 5 map types from `BTreeMap<ValueId, Vec<usize>>` to `BTreeMap<ValueId, SmallVec<[usize; 4]>>`
   - Update `build()` method: change local variable types to match
   - Getter methods work unchanged (SmallVec implements `Deref<[T]>`)
4. Update tests if needed (`.len()` and `.is_empty()` work on SmallVec)

### Task C: IndexMap + nohash-hasher Migration (PTA Hot Paths)

**Files:** `crates/saf-analysis/src/pta/solver.rs`, `crates/saf-analysis/src/pta/constraint_index.rs`, workspace `Cargo.toml`, `crates/saf-analysis/Cargo.toml`

**Goal:** Replace BTreeMap with IndexMap (preserves insertion-order determinism) in PTA hot paths. O(1) vs O(log n) lookup.

**Changes:**
1. Add to workspace `Cargo.toml`:
   - `indexmap = "2"`
   - `nohash-hasher = "0.2"`
2. Add to `crates/saf-analysis/Cargo.toml`:
   - `indexmap = { workspace = true }`
   - `nohash-hasher = { workspace = true }`
3. **constraint_index.rs** — Replace all 5 `BTreeMap<ValueId, SmallVec<...>>` with `IndexMap<ValueId, SmallVec<...>, nohash_hasher::BuildNoHashHasher<u128>>`:
   - ValueId is a u128 newtype, so nohash-hasher works directly IF ValueId implements `nohash_hasher::IsEnabled`. Otherwise use `IndexMap<ValueId, SmallVec<...>>` with default hasher.
   - Need to check if ValueId implements `Hash`. If not, add `Hash` derive to ValueId.
4. **solver.rs** — Replace hot-path BTreeMaps in `GenericSolver`:
   - `pts: BTreeMap<ValueId, P>` → `IndexMap<ValueId, P>`
   - `loc_pts: BTreeMap<LocId, P>` → `IndexMap<LocId, P>`
   - `prev_pts: BTreeMap<ValueId, P>` → `IndexMap<ValueId, P>`
   - `rep: BTreeMap<ValueId, ValueId>` → `IndexMap<ValueId, ValueId>`
   - `topo_order: BTreeMap<ValueId, u32>` → `IndexMap<ValueId, u32>`
   - Update `BTreeMap::new()` → `IndexMap::default()` in constructors
   - `Entry::Vacant` in `solve()` → use IndexMap's entry API (same interface)
   - `compute_topo_order()` local variables also use IndexMap
5. **Type aliases** at top of solver.rs:
   - `PointsToMap` stays as `BTreeMap<ValueId, BTreeSet<LocId>>` (public API)
   - `GenericPointsToMap<P>` → `IndexMap<ValueId, P>` (internal)
   - `ConstantsTable` stays as `BTreeMap<ValueId, Constant>` (external dependency)
6. **Worklist** `BTreeSet<(u32, ValueId)>` stays as-is (needs ordered pop_first)

**IMPORTANT:** ValueId and LocId must implement `Hash` + `Eq` for IndexMap. Check `saf_core::ids` — these are u128 newtypes and likely already derive Hash.

### Task D: Wire Steensgaard → Clustering Seed (1A + 4F)

**Files:** `crates/saf-analysis/src/pta/solver.rs`, `crates/saf-analysis/src/pta/mod.rs`, `crates/saf-analysis/Cargo.toml`

**Goal:** When clustering is enabled, optionally use Steensgaard pre-analysis to generate better clustering seeds instead of the current copy-only `approximate_cooccurrence()`.

**Changes:**
1. Enable `experimental` feature by default (or in the specific build path) so steensgaard module is available
2. In `solver.rs` `create_template()`: after the existing `approximate_cooccurrence()` call, add an alternative path that uses Steensgaard results to seed clustering when available
3. Steensgaard produces equivalence classes (unified pts) — feed these as cooccurrence evidence to the clustering pipeline
4. Config: Add `use_steensgaard_seed: bool` to `PtsConfig` (default false for now)

**Note:** This is lower priority than A/B/C. If time is short, just enable the experimental feature and add the config plumbing without changing the default behavior.

### Task E: Wave Propagation Restructure

**Files:** `crates/saf-analysis/src/pta/solver.rs`

**Goal:** Restructure main `solve()` loop for 2-phase wave propagation: process all nodes at current topological level before advancing to next level. Currently nodes are processed in strict topo order (BTreeSet pop_first) but without explicit wave boundaries.

**Changes:**
1. In `solve()`, after initializing topo_order, group initial worklist nodes by topo rank
2. Process all nodes at rank R before moving to rank R+1 (wave front)
3. When processing a node generates new worklist entries at the SAME rank, process them in the current wave
4. When processing generates entries at HIGHER ranks, defer to next wave
5. Location worklist processing stays between value waves

**Expected benefit:** 1.2-1.5x speedup by reducing redundant re-processing of nodes that get updated multiple times from different predecessors at the same level.

## Implementation Strategy

- **Task A**: Trivial one-line fix, do first to unblock BitVec/BDD in CG refinement
- **Task B**: SmallVec migration — independent, safe, no API changes
- **Task C**: IndexMap migration — depends on B (SmallVec types in constraint_index), biggest impact
- **Task D**: Steensgaard wiring — independent, optional enhancement
- **Task E**: Wave propagation — independent, solver-internal restructure

## Success Criteria

1. All existing tests pass (1465 Rust + 72 Python)
2. PTABen: 2251 Exact, 69 Unsound (no regression)
3. CruxBC benchmarks show measurable speedup (target: bunzip2 <15s)
4. `make fmt && make lint` clean
