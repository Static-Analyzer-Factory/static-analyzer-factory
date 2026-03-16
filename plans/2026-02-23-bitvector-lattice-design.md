# Bitvector Lattice for Ascent PTA Solver

**Date:** 2026-02-23
**Plan:** 158-bitvector-lattice
**Status:** design

## Problem

The Ascent PTA solver uses `BTreeSet<LocId>` for points-to sets via `AscentPtsSet`.
On the `bash` CruxBC benchmark (47K pointers, avg pts size 321, max 623),
`BTreeSet::extend()` in `join_mut()` performs O(k * log n) insertions per union,
while the legacy solver's bitvector does O(n/64) per union. Result: Ascent's
phase 2 times out at 300s+ while legacy completes in 16s.

## Profiling Data (bash)

| Phase | Current (BTreeSet) | Expected (BitVec) |
|-------|-------------------|-------------------|
| `join_mut` (union) | ~10us (300 inserts x 33ns) | ~30ns (2KB OR) |
| `PartialOrd` (subset) | ~2us | ~30ns |
| `clone` | ~3us (300 allocs) | ~300ns (2KB memcpy) |
| Phase 2 total | >300s (timeout) | ~1-5s (estimated) |
| Memory per set | ~11KB | ~1.9KB |
| Total memory (47K sets) | ~520MB | ~92MB |

## Design

### Architecture

```
LocIdRegistry (shared, built once from facts)
  to_index: BTreeMap<LocId, u32>  -- sparse LocId -> dense index
  to_loc: Vec<LocId>              -- dense index -> LocId

AscentPtsSet (one per pointer)
  bits: FixedBitSet               -- dense bitvector over registry indices
  registry: Arc<LocIdRegistry>    -- shared reference to mapping

Thread-local PTS_REGISTRY          -- set before prog.run(), read by singleton()
```

### Key Decision: Thread-local Registry

Ascent's `Lattice::join_mut(&mut self, other: Self) -> bool` has no parameter for
external state. The Ascent program calls `AscentPtsSet::singleton(loc)` inside rule
bodies where we can't pass a registry reference.

Solution: Store `Arc<LocIdRegistry>` in a thread-local before `prog.run()`.
`singleton()` reads it to create sets with the correct Arc reference. After creation,
each set is self-contained (carries its own Arc).

Alternatives considered:
- **Pre-built singletons**: Requires changing Ascent relation types, bloats addr_of
  relation from 2.6MB to 157MB. Rejected.
- **Global static (OnceLock)**: Not re-entrant, tests with different registries must
  be serialized. Rejected for library crate.

### Trait Requirements

`AscentPtsSet` must implement for Ascent's generated code:
- `Lattice` (from ascent_base) -- requires `PartialOrd + Sized`
- `Clone` -- for propagating sets in copy/store/load rules
- `Debug` -- for Ascent's debug output
- `Default` -- for lattice bottom (empty set)
- `Eq + PartialEq` -- for convergence detection
- `Hash` -- for Ascent's internal relation indexing

### Bitvector Backend

Use `fixedbitset` crate (SIMD-optimized, used by petgraph):
- `union_with()`: SIMD bitwise OR
- `intersection_with()`: SIMD bitwise AND
- `is_subset()`: efficient subset check
- `ones()`: iterate set bit indices

If `fixedbitset` doesn't implement `Hash`, implement manually by hashing the
underlying block slice.

### API Changes

```rust
// pts_lattice.rs -- new internal structure
pub struct AscentPtsSet {
    bits: FixedBitSet,
    registry: Arc<LocIdRegistry>,
}

// New module: registry.rs
pub struct LocIdRegistry {
    to_index: BTreeMap<LocId, u32>,
    to_loc: Vec<LocId>,
}

impl LocIdRegistry {
    /// Build from all LocIds appearing in PtaFacts.
    pub fn from_facts(facts: &PtaFacts) -> Self;
}
```

Public API on `AscentPtsSet` stays the same:
- `empty()` -- reads thread-local for capacity
- `singleton(loc)` -- reads thread-local for registry + index
- `contains(loc)`, `len()`, `is_empty()`, `iter()`, `into_btreeset()`
- `Lattice::join_mut()`, `Lattice::meet_mut()`

### Integration Points

1. **context.rs (`analyze_with_ascent`)**: Build `LocIdRegistry` from facts after
   GEP resolution, set thread-local, call `ascent_solve()`, clear thread-local.

2. **solver.rs (`ascent_solve`)**: No changes needed. The Ascent program definition
   is byte-identical. `AscentPtsSet::singleton(*loc)` works transparently.

3. **Result extraction**: `pts.iter()` yields `LocId` (mapped back from indices).
   `pts.into_btreeset()` works unchanged.

### Thread-local Setup

```rust
// In context.rs
pub fn analyze_with_ascent(...) -> PtaAnalysisResult {
    // ... build facts ...
    let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
    crate::pta::pts_lattice::with_registry(registry, || {
        let preliminary_pts = ascent_solve(&dl_facts);
        // ... GEP resolution may add new LocIds ...
        // Registry must include ALL LocIds that will appear during solving
        let pts = ascent_solve(&dl_facts);
        pts
    })
}
```

**Important**: The registry must be built AFTER GEP resolution adds new field
locations, so it includes all LocIds that will appear during phase 2 solving.
For the two-phase approach, rebuild the registry between phases.

### Default Implementation

`Default` for `AscentPtsSet` is needed by Ascent's generated code. Two options:
1. Read thread-local in `Default::default()` to get registry + capacity
2. Start with an empty/zero-capacity BitVec and grow on first `join_mut`

Option 1 is cleaner and avoids reallocation.

### Hash Implementation

`Hash` for `AscentPtsSet` should hash ONLY the bit content, not the registry Arc.
Two sets with the same bits and different Arc pointers (but same registry content)
should hash identically. In practice, all sets share the same Arc during a single
analysis run.

## Files Changed

| File | Change |
|------|--------|
| `crates/saf-datalog/Cargo.toml` | Add `fixedbitset` dependency |
| `crates/saf-datalog/src/pta/pts_lattice.rs` | Replace BTreeSet with FixedBitSet + Arc<Registry> |
| `crates/saf-datalog/src/pta/registry.rs` | New: LocIdRegistry + thread-local setup |
| `crates/saf-datalog/src/pta/context.rs` | Build registry, wrap solve calls in with_registry |
| `crates/saf-datalog/src/pta/mod.rs` | Export registry module |

## Testing

- All existing `saf-datalog` tests pass unchanged (54 tests)
- Add unit tests for `LocIdRegistry` (round-trip mapping)
- Add unit tests for `AscentPtsSet` bitvector operations (union, intersection, subset)
- Benchmark comparison: bash cruxbc with BTreeSet vs BitVec
