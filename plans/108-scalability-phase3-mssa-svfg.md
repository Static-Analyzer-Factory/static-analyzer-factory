# Plan 108: Scalability Phase 3 — MSSA/SVFG Scalability

**Epic:** Scalability
**Status:** done
**Date:** 2026-02-12

## Context

Phases 1-2 (Plans 106-107) delivered PTA solver speedups (32-112x) and FSPTA overhaul (PtsSet generics, SVFG optimization, IndexMap migration). Phase 3 targets MSSA/SVFG construction scalability and constraint preprocessing.

Three independent improvements:
1. **Memory Region Partitioning** — Classify locations by type/region for faster MSSA clobber disambiguation
2. **Roaring Bitmaps** — New PtsSet implementation for sparse sets (2-10x memory, 1.5-3x speed)
3. **HVN Pre-processing** — Hash-based value numbering to reduce constraint graph 20-40%

## Phase 3A: Roaring Bitmaps PtsSet (Agent A)

**Goal:** Add `RoaringPtsSet` as a fourth PtsSet implementation using the `roaring` crate.

### Tasks
1. Add `roaring = "0.10"` to workspace Cargo.toml and saf-analysis Cargo.toml
2. Create `crates/saf-analysis/src/pta/ptsset/roaring_pts.rs`:
   - `RoaringPtsSet` struct wrapping `roaring::RoaringBitmap` + shared `Arc<RwLock<LocIdIndexer>>`
   - Implement full `PtsSet` trait (following `BitVecPtsSet` pattern)
   - `BENEFITS_FROM_CLUSTERING = true`
   - `clone_empty()` shares indexer
   - `with_seeded_ordering()` pre-registers locations
   - Deterministic iteration (Roaring iterates in sorted order naturally)
3. Add `Roaring` variant to `PtsRepresentation` enum in `config.rs`
4. Wire into dispatch in `solver.rs` `solve_with_index_config()`
5. Add to auto-selection thresholds in `PtsConfig` (between BitVec and BDD)
6. Re-export from `ptsset/mod.rs`
7. Add unit tests (same pattern as bitvec.rs tests)
8. Add solver equivalence test (like `solver_all_representations_equivalent`)

### Key Pattern to Follow
- `BitVecPtsSet` in `bitvec.rs` — exact same architecture (shared indexer via Arc<RwLock<LocIdIndexer>>)
- Hash/Eq/Send/Sync implementations matching existing pattern
- O(1) membership via Roaring's compressed bitmap

## Phase 3B: HVN Pre-processing (Agent B)

**Goal:** Add offline Hash-based Value Numbering to merge equivalent pointer variables before solving, reducing constraint count by 20-40%.

### Algorithm (from Hardekopf & Lin, PLDI 2007)
1. For each value, compute a hash from its constraint pattern:
   - Values with only Addr constraints: hash = set of pointed-to locations
   - Values with only Copy constraints from same sources: hash = sources
   - Values with no constraints (non-pointer): mark as "non-pointer"
2. Values with identical hashes are in the same equivalence class
3. Merge each class into a single representative
4. Rewrite constraints to use representatives
5. Remove redundant constraints

### Tasks
1. Create `crates/saf-analysis/src/pta/hvn.rs`:
   - `pub fn hvn_preprocess(constraints: &mut ConstraintSet)` — main entry point
   - Build "pointer equivalence" graph from constraints
   - Hash-based partitioning into equivalence classes
   - Constraint rewriting using representatives
   - Return mapping `BTreeMap<ValueId, ValueId>` (original → representative)
2. Wire into solver pipeline:
   - Call from `solve_with_index_config()` before `ConstraintIndex::build()`
   - Apply mapping to results after solving (expand representatives back)
3. Add unit tests:
   - Equivalent copy chains merged
   - Addr-only values with same target merged
   - Non-pointer values eliminated
   - Results equivalent with/without HVN

### Key Constraints
- Must maintain determinism (use BTreeMap for hashing partitions)
- Must not change analysis results (only optimization)
- Must handle all 5 constraint types correctly

## Phase 3C: Memory Region Partitioning (Agent C)

**Goal:** Classify memory locations into regions to accelerate MSSA clobber disambiguation.

### Design
Memory regions partition locations by type compatibility:
- **Stack** — `Alloca`-derived locations
- **Heap** — `HeapAlloc`-derived locations
- **Global** — Global variable locations
- **Unknown** — External/unclassified

Within each category, further partition by base type (struct types don't alias non-struct types).

### Tasks
1. Add `MemoryRegion` enum to `crates/saf-analysis/src/pta/location.rs`:
   ```rust
   pub enum MemoryRegion {
       Stack,
       Heap,
       Global,
       Unknown,
   }
   ```
2. Add `region` field to `Location` struct and `LocationFactory`:
   - `get_or_create()` accepts region parameter
   - `get_or_create_with_region()` for backwards compat
3. Update constraint extraction (`extract.rs`) to pass region when creating locations:
   - `Alloca` → `Stack`
   - `HeapAlloc` → `Heap`
   - `Global` → `Global`
   - External/unknown → `Unknown`
4. Add region query to `LocationFactory`:
   - `pub fn region(&self, loc: LocId) -> MemoryRegion`
   - `pub fn same_region(&self, a: LocId, b: LocId) -> bool`
5. Use region in MSSA clobber walker (`mssa/walker.rs`):
   - Early return in `is_clobber()` when store region ≠ query region
   - Skip clobber walk entirely for cross-region pairs
6. Use region in mod/ref summaries (`mssa/modref.rs`):
   - Per-region may_mod/may_ref sets for faster intersection
7. Add tests for region classification and clobber disambiguation

### Key Constraint
- `Unknown` region must alias with ALL other regions (sound over-approximation)
- Field children inherit parent region
- Must not change existing `get_or_create()` signature — add defaulting or new method

## Team Structure

| Agent | Task | Files (new) | Files (modify) |
|-------|------|-------------|----------------|
| A | Roaring Bitmaps | `ptsset/roaring_pts.rs` | `Cargo.toml` (workspace + saf-analysis), `ptsset/mod.rs`, `ptsset/config.rs`, `solver.rs` |
| B | HVN Pre-processing | `pta/hvn.rs` | `pta/mod.rs`, `solver.rs` |
| C | Memory Region Partitioning | — | `pta/location.rs`, `pta/extract.rs`, `mssa/walker.rs`, `mssa/modref.rs` |
| Leader | Integration + testing | — | `plans/PROGRESS.md` |

All three agents work in parallel. Leader runs `make fmt && make lint && make test` after integration.

## Expected Impact

| Improvement | Expected Speedup | Where |
|-------------|-----------------|-------|
| Memory Region Partitioning | 2-5x MSSA clobber | Large programs with mixed heap/stack/global |
| Roaring Bitmaps | 2-10x memory, 1.5-3x speed | Sparse PTA sets (medium-large programs) |
| HVN Pre-processing | 20-40% fewer constraints | Constraint solving phase |
| **Cumulative** | **~2-5x overall** | MSSA/SVFG construction + PTA solving |
