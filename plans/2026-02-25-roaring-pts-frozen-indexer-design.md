# Design: Roaring PTS Default + Frozen Indexer

**Date:** 2026-02-25
**Epic:** E25 — Scalability
**Plan:** 170

## Problem

SAF uses `BTreeSet<LocId>` (via `BTreePtsSet`) as the effective default PTS representation for the Andersen solver. Each entry costs ~64 bytes (48 bytes BTree node overhead + 16 bytes for `u128` LocId). For programs with large average PTS size, this dominates memory:

| Program | Avg PTS | Pointers | PTS Memory (BTree) | SVF Memory |
|---------|---------|----------|--------------------|------------|
| bash    | 317     | 47K      | 954 MB             | 117 MB     |
| tmux    | 278     | 47.7K    | 849 MB             | ~130 MB    |
| htop    | 82      | 9K       | 47 MB              | ~50 MB     |
| unrar   | 56      | 20K      | 72 MB              | ~100 MB    |
| curl    | 4.2     | 14.3K    | 4 MB               | ~100 MB    |

bash consumes 2.6 GB total vs SVF's 1.1 GB. The PTS representation alone accounts for ~954 MB of that gap.

## Solution

Two changes:

1. **Roaring bitmap as default for programs with ≥50K alloc sites** — adaptive compression (array/bitset/run containers per 64K chunk), native SIMD operations, already implemented and tested as `RoaringPtsSet`.

2. **Frozen indexer** — pre-build the `LocId → u32` index from all locations in addr constraints + `LocationFactory` before the solve loop, then share it as `Arc<Indexer>` (immutable, no `RwLock`). Eliminates lock acquisition on every `insert`, `contains`, and `union` call.

### Why Roaring over BitVec

| Aspect | BitVec | Roaring |
|--------|--------|---------|
| Memory (bash, avg 317) | 1.5 KB/set fixed | ~951 B/set (array container) |
| Memory (curl, avg 4.2) | 1.5 KB/set fixed | ~8 B/set (tiny array) |
| Union | O(N/64) word-level | O(containers) SIMD-optimized |
| Sparse handling | Wastes bits on zeros | Array container for <4096 |
| Dense handling | Efficient | Bitset container (same) |
| External dep | bitvec crate | roaring crate (already in Cargo.toml) |

Roaring adapts automatically — no need for a separate density heuristic.

### Why Frozen Indexer

All indexed PTS implementations (`BitVecPtsSet`, `RoaringPtsSet`, `BddPtsSet`) wrap the indexer in `Arc<RwLock<Indexer>>`. Every operation acquires a lock:

- `insert()` → `indexer.write()` (exclusive lock)
- `contains()` → `indexer.read()` (shared lock)
- `iter()` → `indexer.read()` (shared lock)
- `union()` with same indexer → no lock (bitwise only), but `align_to_indexer()` takes read lock

The Andersen solver's hot path (345K value pops, millions of load iterations) makes this lock overhead significant. Since all locations are known before solving (from addr constraints + LocationFactory), the indexer can be built once and frozen.

## Architecture

### Frozen Indexer

```
enum IndexerState<T> {
    Building(Indexer<T>),          // Mutable — during constraint extraction
    Frozen(Arc<FrozenIndexer<T>>), // Immutable — during solve
}

struct FrozenIndexer<T> {
    item_to_idx: FxHashMap<T, u32>,  // Fast O(1) lookup (not BTreeMap)
    idx_to_item: Vec<T>,              // O(1) reverse lookup
}
```

The `PtsSet` trait gains a new associated method:

```rust
fn freeze_indexer(&mut self);  // Transition Building → Frozen
```

Or simpler: the solver pre-builds the indexer, freezes it into `Arc<FrozenIndexer>`, and creates all PTS sets with `with_frozen_indexer()`.

### Auto-Selection Change

Current `PtsConfig::select_by_count()`:
- <50K: FxHash
- 50K-100K: Roaring
- ≥100K: BDD

New:
- <10K alloc sites: FxHash (O(k) wins for small/sparse)
- ≥10K alloc sites: Roaring (adaptive, SIMD, frozen indexer)
- BDD: explicit opt-in only

The threshold drop from 50K to 10K captures programs like bash (which has 5,421 locations but large PTS) that benefit from compressed representation.

### Code Cleanup

- Delete `BitVecPtsSet` (`bitvec.rs`) — redundant with `IdBitSet` which already has word-level ops
- Keep `IdBitSet<T>` for non-PTS uses (worklists, etc.)
- Keep `BddPtsSet` as experimental opt-in

## Precision Impact

None. PTS representation is a storage abstraction — the solver's fixed-point algorithm sees identical abstract state regardless of backing store. All implementations satisfy the same `PtsSet` trait contract (validated by `cross_impl_tests`). Final results are normalized to `BTreeMap<ValueId, BTreeSet<LocId>>` at the API boundary.

## Expected Memory Savings

| Program | Before (BTree PTS) | After (Roaring PTS) | Reduction |
|---------|--------------------|--------------------|-----------|
| bash    | ~954 MB            | ~45 MB             | 21x       |
| tmux    | ~849 MB            | ~42 MB             | 20x       |
| htop    | ~47 MB             | ~1.5 MB            | 31x       |
| unrar   | ~72 MB             | ~2.2 MB            | 33x       |
| curl    | ~4 MB              | ~0.1 MB            | 40x       |

bash total: ~2.6 GB → ~1.7 GB (clobber cache and other structures unchanged).

## Scope

### In Scope
- Frozen indexer infrastructure (new type, pre-build in solver)
- Update `RoaringPtsSet` to use frozen indexer
- Update `PtsConfig` auto-selection thresholds
- Delete `BitVecPtsSet` (consolidate to `IdBitSet`)
- Update Andersen solver to pre-build + freeze indexer
- Validate with PTABen + CruxBC benchmarks

### Out of Scope
- Clobber cache optimization (separate concern, ~400 MB for bash)
- FSPTA version table changes (already optimized with dedup)
- CSPTA/DDA solver changes (follow-up if needed)
- Parallel solver (Phase 5 roadmap)
