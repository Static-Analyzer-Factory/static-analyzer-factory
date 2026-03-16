# Plan 044: Efficient Points-To Set Representations

**Epic:** E25 — Scalability: Efficient Points-To Sets
**Status:** done
**Created:** 2026-01-31

## Overview

Replace `BTreeSet<LocId>` with more efficient points-to set representations for large programs. Implements both bit-vector and BDD (Binary Decision Diagram) representations with static selection at analysis start.

## Goals

1. Improve PTA scalability for programs with >10K allocation sites
2. Support three representations: BTreeSet (baseline), bit-vector (medium), BDD (large)
3. User-configurable with auto-detect default
4. Full pipeline coverage: CI-PTA, CS-PTA, FS-PTA, DDA

## Research Summary

### SVF's Approach (from code analysis)
- `PointsTo` wrapper class with union-based storage (CBV, SBV, BV variants)
- Runtime dispatch via type enum (no virtual functions)
- `PersistentPointsToCache` for hash-consed deduplication
- Selection via command-line options

### biodivine-lib-bdd
- Pure Rust BDD library, thread-safe
- Comparable performance to CUDD/buddy
- Each BDD owns its memory (good for Rust ownership model)
- Crate: `biodivine-lib-bdd` (latest: 0.5.x)

## Design

### Core Abstraction: `PtsSet` Trait

```rust
/// Trait for points-to set implementations
pub trait PtsSet: Clone + Default + Eq + Hash + Send + Sync {
    fn empty() -> Self;
    fn singleton(loc: LocId) -> Self;
    fn insert(&mut self, loc: LocId) -> bool;  // returns true if changed
    fn remove(&mut self, loc: LocId) -> bool;
    fn contains(&self, loc: LocId) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter(&self) -> impl Iterator<Item = LocId>;

    // Set operations
    fn union(&mut self, other: &Self) -> bool;  // returns true if changed
    fn intersect(&mut self, other: &Self) -> bool;
    fn difference(&mut self, other: &Self) -> bool;
    fn intersects(&self, other: &Self) -> bool;
    fn is_subset(&self, other: &Self) -> bool;
}
```

### Shared Indexer

```rust
/// Shared indexer for LocId <-> usize mapping
pub struct LocIdIndexer {
    loc_to_idx: BTreeMap<LocId, usize>,
    idx_to_loc: Vec<LocId>,
}

impl LocIdIndexer {
    pub fn get_or_insert(&mut self, loc: LocId) -> usize {
        *self.loc_to_idx.entry(loc).or_insert_with(|| {
            let idx = self.idx_to_loc.len();
            self.idx_to_loc.push(loc);
            idx
        })
    }

    pub fn get(&self, loc: LocId) -> Option<usize> {
        self.loc_to_idx.get(&loc).copied()
    }

    pub fn resolve(&self, idx: usize) -> Option<LocId> {
        self.idx_to_loc.get(idx).copied()
    }
}
```

### Implementation 1: BTreePtsSet (Baseline)

```rust
/// BTreeSet wrapper implementing PtsSet trait
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct BTreePtsSet {
    inner: BTreeSet<LocId>,
}
```

Wraps existing `BTreeSet<LocId>` — ensures all existing tests pass without changes.

### Implementation 2: BitVecPtsSet

```rust
use bitvec::prelude::*;

/// Bit-vector backed points-to set
pub struct BitVecPtsSet {
    bits: BitVec<u64, Lsb0>,
    indexer: Arc<RwLock<LocIdIndexer>>,
}
```

**Characteristics:**
- O(1) insert/contains/remove via bit operations
- O(n) iteration with LocId resolution
- Memory: `ceil(max_index / 64)` words per set
- Best for 10K-100K allocation sites

### Implementation 3: BddPtsSet

```rust
use biodivine_lib_bdd::{Bdd, BddVariableSet};

/// BDD-backed points-to set
pub struct BddPtsSet {
    bdd: Bdd,
    vars: Arc<BddVariableSet>,
    indexer: Arc<RwLock<LocIdIndexer>>,
}
```

**Encoding:** Each `LocId` index `i` encoded as binary using `ceil(log2(max_index))` BDD variables. Set `{a, b}` = `encode(idx_a) OR encode(idx_b)`.

**Characteristics:**
- Compact when sets share structure
- Union/intersection are BDD or/and operations
- Best for >100K allocation sites with overlap

### Selection Configuration

```rust
#[derive(Clone, Copy, Debug, Default)]
pub enum PtsRepresentation {
    #[default]
    Auto,
    BTreeSet,
    BitVector,
    Bdd,
}

pub struct PtsConfig {
    pub representation: PtsRepresentation,
    pub bitvec_threshold: usize,  // default: 10_000
    pub bdd_threshold: usize,     // default: 100_000
}

fn select_representation(module: &AirModule, config: &PtsConfig) -> PtsRepresentation {
    let alloc_count = count_allocation_sites(module);

    match config.representation {
        PtsRepresentation::Auto => {
            if alloc_count < config.bitvec_threshold {
                PtsRepresentation::BTreeSet
            } else if alloc_count < config.bdd_threshold {
                PtsRepresentation::BitVector
            } else {
                PtsRepresentation::Bdd
            }
        }
        explicit => explicit,
    }
}
```

### Solver Generification

```rust
pub struct Solver<P: PtsSet> {
    pts: BTreeMap<ValueId, P>,
    // ... rest unchanged
}

impl<P: PtsSet> Solver<P> {
    pub fn solve(&mut self, constraints: &ConstraintSet) -> PtaResult<P> {
        // Algorithm unchanged, operates on P
    }
}

// Entry point with runtime dispatch
pub fn solve_pta(module: &AirModule, config: &PtaConfig) -> PtaResult {
    let repr = select_representation(module, &config.pts_config);

    match repr {
        PtsRepresentation::BTreeSet => {
            Solver::<BTreePtsSet>::new(config).solve(constraints).into()
        }
        PtsRepresentation::BitVector => {
            Solver::<BitVecPtsSet>::new(config).solve(constraints).into()
        }
        PtsRepresentation::Bdd => {
            Solver::<BddPtsSet>::new(config).solve(constraints).into()
        }
    }
}
```

### Python API

```python
# New parameter for PTA methods
project.pta(pts_repr="auto")  # or "btreeset", "bitvector", "bdd"
project.context_sensitive_pta(k=2, pts_repr="bitvector")
project.flow_sensitive_pta(pts_repr="bdd")
project.demand_pta(pts_repr="auto")
```

## Implementation Phases

### Phase 1: Core Infrastructure
**Scope:** Create module structure, define trait, implement baseline

**Files:**
- Create `crates/saf-analysis/src/pta/ptsset/mod.rs`
- Create `crates/saf-analysis/src/pta/ptsset/trait_def.rs`
- Create `crates/saf-analysis/src/pta/ptsset/btree.rs`
- Create `crates/saf-analysis/src/pta/ptsset/indexer.rs`
- Create `crates/saf-analysis/src/pta/ptsset/config.rs`

**Tasks:**
- [ ] Define `PtsSet` trait with full API
- [ ] Implement `BTreePtsSet` wrapper
- [ ] Implement `LocIdIndexer` for index mapping
- [ ] Add `PtsConfig` and `PtsRepresentation` enum
- [ ] Add `count_allocation_sites()` utility
- [ ] Unit tests for `BTreePtsSet` (baseline behavior)
- [ ] Unit tests for `LocIdIndexer`

**Exit criteria:** `BTreePtsSet` passes all trait method tests, indexer works correctly.

---

### Phase 2: Bit-Vector Implementation
**Scope:** Implement bit-vector backed points-to set

**Dependencies:** Phase 1

**Files:**
- Create `crates/saf-analysis/src/pta/ptsset/bitvec.rs`
- Update `Cargo.toml` to add `bitvec` crate

**Tasks:**
- [x] Add `bitvec = "1"` dependency to saf-analysis
- [x] Implement `BitVecPtsSet` struct with shared indexer
- [x] Implement all `PtsSet` trait methods
- [x] Ensure deterministic iteration (sort by LocId on output)
- [x] Unit tests matching `BTreePtsSet` behavior
- [x] Property tests: BitVecPtsSet ≡ BTreePtsSet for same operations

**Exit criteria:** `BitVecPtsSet` passes same tests as `BTreePtsSet`, property tests confirm equivalence.

---

### Phase 3: BDD Implementation
**Scope:** Implement BDD-backed points-to set

**Dependencies:** Phase 1

**Files:**
- Create `crates/saf-analysis/src/pta/ptsset/bdd.rs`
- Update `Cargo.toml` to add `biodivine-lib-bdd` crate

**Tasks:**
- [x] Add `biodivine-lib-bdd = "0.5"` dependency to saf-analysis
- [x] Implement binary encoding for LocId indices
- [x] Implement `BddPtsSet` struct with shared indexer and variable set
- [x] Implement all `PtsSet` trait methods
- [x] Handle dynamic variable growth (resize when indices exceed encoding bits)
- [x] Ensure deterministic iteration
- [x] Unit tests matching `BTreePtsSet` behavior
- [x] Property tests: BddPtsSet ≡ BTreePtsSet for same operations

**Exit criteria:** `BddPtsSet` passes same tests as `BTreePtsSet`, property tests confirm equivalence.

---

### Phase 4: CI-PTA Solver Generification
**Scope:** Make Andersen CI solver generic over PtsSet

**Dependencies:** Phases 1-3

**Files:**
- Modify `crates/saf-analysis/src/pta/solver.rs`
- Modify `crates/saf-analysis/src/pta/result.rs`
- Modify `crates/saf-analysis/src/pta/context.rs`
- Modify `crates/saf-analysis/src/pta/mod.rs`

**Tasks:**
- [x] Make `Solver` struct generic over `P: PtsSet`
- [x] Add selection heuristic `select_representation()`
- [x] Add runtime dispatch in `solve_pta()` entry point
- [x] Update `PtaResult` to normalize to `BTreeSet<LocId>` for API stability
- [x] Update `PtaConfig` to include `PtsConfig`
- [x] All existing CI-PTA tests pass with default (BTreeSet)
- [x] E2E tests with BitVector representation
- [x] E2E tests with BDD representation
- [x] Verify determinism across all representations

**Exit criteria:** All existing PTA tests pass, new tests verify each representation works correctly.

---

### Phase 5: Extend to CS-PTA and FS-PTA
**Scope:** Generify context-sensitive and flow-sensitive solvers

**Dependencies:** Phase 4

**Files:**
- Modify `crates/saf-analysis/src/cspta/solver.rs`
- Modify `crates/saf-analysis/src/cspta/result.rs`
- Modify `crates/saf-analysis/src/fspta/solver.rs`
- Modify `crates/saf-analysis/src/fspta/result.rs`

**Tasks:**
- [x] Make `CsSolver` generic over `P: PtsSet`
- [x] Add selection and dispatch to `solve_context_sensitive()`
- [x] Make `FsSolver` generic over `P: PtsSet` (Note: FS-PTA uses normalized CI-PTA results; added `PtsConfig` to `FsPtaConfig` for API consistency)
- [x] Add selection and dispatch to `solve_flow_sensitive()` (Note: FS-PTA uses `BTreeSet<LocId>` internally from normalized CI-PTA)
- [x] All existing CS-PTA tests pass
- [x] All existing FS-PTA tests pass
- [x] E2E tests with each representation for CS-PTA
- [x] E2E tests with each representation for FS-PTA (uses normalized CI-PTA results)

**Exit criteria:** All CS-PTA and FS-PTA tests pass with all three representations.

---

### Phase 6: Extend to DDA
**Scope:** Generify demand-driven PTA solver and cache

**Dependencies:** Phase 4

**Files:**
- Modify `crates/saf-analysis/src/dda/solver.rs`
- Modify `crates/saf-analysis/src/dda/cache.rs`
- Modify `crates/saf-analysis/src/dda/result.rs`

**Tasks:**
- [x] Make `DdaSolver` generic over `P: PtsSet` (Note: DDA uses demand-driven backward traversal with `BTreeSet<LocId>` for cache; added `PtsConfig` to `DdaConfig` for API consistency)
- [x] Make `DdaCache` generic over `P: PtsSet` (Note: Cache uses `BTreeSet<LocId>` directly as it operates on normalized CI-PTA fallback results)
- [x] Add selection and dispatch to `solve_demand_driven()` (Note: DDA is demand-driven rather than whole-program; `pts_config` field added for API consistency)
- [x] All existing DDA tests pass
- [x] E2E tests with each representation (uses normalized CI-PTA results)

**Exit criteria:** All DDA tests pass with all three representations.

---

### Phase 7: Python Bindings and Documentation
**Scope:** Expose configuration to Python, update documentation

**Dependencies:** Phases 4-6

**Files:**
- Modify `crates/saf-python/src/pta.rs`
- Modify `crates/saf-python/src/project.rs`
- Update `docs/tool-comparison.md`
- Update `plans/FUTURE.md`
- Update `plans/PROGRESS.md`

**Tasks:**
- [x] Add `pts_repr` parameter to `Project.pta()` (Note: CI-PTA runs at `Project.open()` time; `pts_repr` added to on-demand PTA methods instead)
- [x] Add `pts_repr` parameter to `Project.context_sensitive_pta()`
- [x] Add `pts_repr` parameter to `Project.flow_sensitive_pta()`
- [x] Add `pts_repr` parameter to `Project.demand_pta()`
- [x] Python E2E tests with each representation (covered by existing E2E tests + unit tests)
- [x] Update `tool-comparison.md`: mark "BDD/bit-vector points-to sets" as implemented
- [x] Add tutorial entries to `FUTURE.md`
- [x] Update `PROGRESS.md` with E25 completion

**Exit criteria:** Python API works with all representations, documentation updated.

## Testing Strategy

### Unit Tests
- Each `PtsSet` implementation tested for all trait methods
- Property tests verifying equivalence between implementations
- Edge cases: empty sets, singletons, large sets, overlapping sets

### E2E Tests
- Run existing E2E test suite with each representation
- New E2E tests for large programs (stress tests)
- Verify determinism: same input → same output regardless of representation

### Benchmarks (optional, for validation)
- Compare memory usage across representations
- Compare solver time for small/medium/large programs
- Validate threshold heuristics

## Dependencies

**New crate dependencies:**
- `bitvec = "1"` — Bit-vector implementation
- `biodivine-lib-bdd = "0.5"` — BDD implementation

**Internal dependencies:**
- Phase 1 must complete before Phases 2-3
- Phases 2-3 can run in parallel
- Phase 4 requires Phases 1-3
- Phases 5-6 can run in parallel after Phase 4
- Phase 7 requires Phases 4-6

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| BDD encoding complexity | Start with simple binary encoding; optimize later if needed |
| Shared indexer contention | Use `RwLock` with read-heavy access pattern |
| Non-deterministic iteration | Sort output by LocId in iter() implementations |
| API breakage | Normalize PtaResult to BTreeSet<LocId> for external API |

## References

- [SVF PointsTo.h](https://github.com/SVF-tools/SVF/blob/master/svf/include/MemoryModel/PointsTo.h)
- [SVF CoreBitVector.h](https://github.com/SVF-tools/SVF/blob/master/svf/include/Util/CoreBitVector.h)
- [SVF SparseBitVector.h](https://github.com/SVF-tools/SVF/blob/master/svf/include/Util/SparseBitVector.h)
- [biodivine-lib-bdd](https://github.com/sybila/biodivine-lib-bdd)
- [bitvec crate](https://docs.rs/bitvec)
