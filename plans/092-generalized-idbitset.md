# Plan 092: Generalized `IdBitSet<T>` — Bitvec for All ID-Based Sets

**Epic:** E25 — Scalability: Efficient Points-To Sets
**Status:** approved
**Created:** 2026-02-09

## Context

Plan 091 integrated object clustering and shared-indexer bitvec operations into PTA. The `BitVecPtsSet` pattern — a shared dense indexer mapping sparse IDs to bit positions for O(n/64) bulk set operations — is highly effective but currently LocId-only. Multiple SAF components use `BTreeSet<SomeId>` in fixpoint loops where bitvec operations would yield 2-20x speedups:

- **MSSA mod/ref**: `BTreeSet<LocId>` with fixpoint `.extend()` unions (O(n log n) → O(n/64))
- **PTA worklists**: `BTreeSet<ValueId/LocId>` with millions of insert/pop_first cycles (allocation-free)
- **Absint fixpoint**: `BTreeSet<BlockId>` for `loop_headers` checked per edge per iteration (O(log n) → O(1))
- **DDA cache**: `BTreeSet<LocId>` cloned on cache hits (O(n) → O(n/64) memcpy)

This plan generalizes the indexer, creates a reusable `IdBitSet<T>`, and wires it into these four components.

---

## Agent Task Definitions

### Task 1: Generalize `Indexer<T>` from `LocIdIndexer`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/ptsset/indexer.rs`. Currently defines `LocIdIndexer` hardcoded to `LocId`. Make it generic.

**Step 1**: Change the import. Replace `use saf_core::ids::LocId;` with nothing (the generic type comes from callers).

**Step 2**: Replace the struct definition (line 38-44):
```rust
#[derive(Debug, Clone, Default)]
pub struct Indexer<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> {
    /// Forward mapping: T → index
    item_to_idx: BTreeMap<T, usize>,
    /// Reverse mapping: index → T
    idx_to_item: Vec<T>,
}
```

**Step 3**: Replace `impl LocIdIndexer` (line 46) with:
```rust
impl<T: Eq + Ord + Copy + std::hash::Hash + std::fmt::Debug> Indexer<T>
```

**Step 4**: In every method, replace `LocId` with `T` in parameter and return types:
- `get_or_insert(&mut self, loc: LocId)` → `get_or_insert(&mut self, item: T)`
- `get(&self, loc: LocId)` → `get(&self, item: T)`
- `resolve(&self, idx: usize) -> Option<LocId>` → `resolve(&self, idx: usize) -> Option<T>`
- `iter()` return type: `Iterator<Item = (usize, LocId)>` → `Iterator<Item = (usize, T)>`
- `locations()` → rename to `items()`, return `Iterator<Item = T>`
- `register_batch(locs: impl IntoIterator<Item = LocId>)` → `register_batch(items: impl IntoIterator<Item = T>)`

Also rename internal field accesses: `loc_to_idx` → `item_to_idx`, `idx_to_loc` → `idx_to_item`.

**Step 5**: Add type alias after the impl block:
```rust
/// Backward-compatible alias for `Indexer<LocId>`.
pub type LocIdIndexer = Indexer<saf_core::ids::LocId>;
```

**Step 6**: Update doc comments and examples to use `Indexer<T>` / `LocIdIndexer`.

**Step 7**: Update tests — they currently use `LocIdIndexer::new()`. Since `LocIdIndexer` is now a type alias, existing test code should compile unchanged. Add one new test:

```rust
    #[test]
    fn generic_indexer_with_value_id() {
        use saf_core::ids::ValueId;
        let mut indexer = Indexer::<ValueId>::new();
        let idx = indexer.get_or_insert(ValueId::new(42));
        assert_eq!(idx, 0);
        assert_eq!(indexer.resolve(0), Some(ValueId::new(42)));
    }
```

**Step 8**: In `crates/saf-analysis/src/pta/ptsset/mod.rs`, update re-exports (line 70):
```rust
pub use indexer::{Indexer, LocIdIndexer};
```

No other files should need changes — all existing code uses `LocIdIndexer` which is now a type alias.

---

### Task 2: Create `IdBitSet<T>` — Generic Bitvec Set

**Self-contained instructions for agent:**

Create a new file `crates/saf-analysis/src/pta/ptsset/id_bitset.rs`. This is a general-purpose bitvec-backed set for any ID type, modeled after `BitVecPtsSet` in the same directory (read `bitvec.rs` for reference patterns).

**API surface** (implement all of these):

```rust
use std::collections::BTreeSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use bitvec::prelude::*;
use super::indexer::Indexer;

/// A generic bitvec-backed set for ID types.
///
/// Uses a shared `Indexer<T>` to map sparse IDs to dense bit positions,
/// enabling O(n/64) bulk set operations (union, intersect, difference).
/// All sets sharing the same indexer can perform bitwise operations directly.
#[derive(Debug)]
pub struct IdBitSet<T: Eq + Ord + Copy + Hash + fmt::Debug> {
    bits: BitVec<u64, Lsb0>,
    indexer: Arc<RwLock<Indexer<T>>>,
    cached_len: usize,
}
```

**Required methods:**

```rust
impl<T: Eq + Ord + Copy + Hash + fmt::Debug> IdBitSet<T> {
    /// Create a new empty set with a fresh indexer.
    pub fn empty() -> Self;

    /// Create a new empty set with the given shared indexer.
    pub fn with_indexer(indexer: Arc<RwLock<Indexer<T>>>) -> Self;

    /// Create an empty set sharing this set's indexer.
    pub fn clone_empty(&self) -> Self;

    /// Get a reference to the shared indexer.
    pub fn indexer(&self) -> &Arc<RwLock<Indexer<T>>>;

    /// Insert an item. Returns true if newly inserted.
    pub fn insert(&mut self, item: T) -> bool;

    /// Remove an item. Returns true if it was present.
    pub fn remove(&mut self, item: T) -> bool;

    /// Check if item is in the set.
    pub fn contains(&self, item: T) -> bool;

    /// Number of elements.
    pub fn len(&self) -> usize;

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool;

    /// Iterate elements in index order (deterministic).
    /// NOTE: index order is insertion order, NOT T's natural Ord.
    /// If sorted-by-T iteration is needed, collect and sort.
    pub fn iter(&self) -> impl Iterator<Item = T>;

    /// Remove and return the element with the lowest index.
    /// Useful as a worklist pop operation. O(n/64) scan using first_one().
    pub fn pop_first(&mut self) -> Option<T>;

    /// Union: add all elements from other. Returns true if self changed.
    /// Fast path O(n/64) when indexers are shared (Arc::ptr_eq).
    pub fn union(&mut self, other: &Self) -> bool;

    /// Intersect: keep only elements also in other. Returns true if self changed.
    pub fn intersect(&mut self, other: &Self) -> bool;

    /// Difference: remove elements that are in other. Returns true if self changed.
    pub fn difference(&mut self, other: &Self) -> bool;

    /// Check if any element is in both sets.
    pub fn intersects(&self, other: &Self) -> bool;

    /// Extend from an iterator of items.
    pub fn extend(&mut self, items: impl IntoIterator<Item = T>);

    /// Convert to `BTreeSet` (sorted by T's Ord).
    pub fn to_btreeset(&self) -> BTreeSet<T>;

    /// Create from a `BTreeSet`, using a fresh indexer.
    pub fn from_btreeset(set: &BTreeSet<T>) -> Self;

    /// Create from a `BTreeSet`, using a shared indexer.
    pub fn from_btreeset_with_indexer(set: &BTreeSet<T>, indexer: Arc<RwLock<Indexer<T>>>) -> Self;
}
```

**Trait implementations:**
- `Clone` — clone bits + Arc::clone indexer + copy cached_len
- `Default` — delegate to `empty()`
- `PartialEq` / `Eq` — same indexer: compare bits (O(n/64)); different indexer: compare `to_btreeset()` (O(n log n))
- `Hash` — iterate sorted elements and hash them (same as `BitVecPtsSet`)
- `Send + Sync` — automatically satisfied since Arc<RwLock<_>> is Send+Sync
- `fmt::Display` — show as `{item1, item2, ...}` (sorted)

**Implementation notes:**

**CRITICAL — SIMD-friendly bulk operations via raw `u64` slices:**

The existing `BitVecPtsSet::union()` in `bitvec.rs` has a performance bug: it operates bit-by-bit (`for i in 0..min_len { if other.bits[i] && !self.bits[i] { ... } }`), which is O(n) not O(n/64) and prevents auto-vectorization. **Do NOT copy that pattern.** Instead, use raw `u64` word-level operations that LLVM auto-vectorizes to SIMD (AVX2/NEON):

```rust
fn union(&mut self, other: &Self) -> bool {
    if !Arc::ptr_eq(&self.indexer, &other.indexer) {
        // Fallback: iterate and insert
        let mut changed = false;
        for item in other.iter() {
            if self.insert(item) { changed = true; }
        }
        return changed;
    }

    self.align_to_indexer();
    if other.bits.len() > self.bits.len() {
        self.bits.resize(other.bits.len(), false);
    }

    let old_len = self.cached_len;

    // SIMD-friendly: operate on raw u64 words, not individual bits.
    // LLVM auto-vectorizes this loop to AVX2 (4x u64) or NEON (2x u64).
    let self_raw = self.bits.as_raw_mut_slice();
    let other_raw = other.bits.as_raw_slice();
    let common = self_raw.len().min(other_raw.len());
    for i in 0..common {
        self_raw[i] |= other_raw[i];
    }

    self.cached_len = self.bits.count_ones();
    self.cached_len > old_len
}
```

Apply the same raw-slice pattern for `intersect` (use `&=`), `difference` (use `&= !`), and `intersects` (use `& != 0` early exit):

```rust
// intersect: keep only common elements
for i in 0..common {
    self_raw[i] &= other_raw[i];
}
// clear bits beyond other's length
for word in &mut self_raw[common..] {
    *word = 0;
}

// difference: remove elements in other
for i in 0..common {
    self_raw[i] &= !other_raw[i];
}

// intersects: early-exit check
for i in 0..common {
    if self_raw[i] & other_raw[i] != 0 { return true; }
}
```

**Other implementation notes:**
- `ensure_capacity(idx)`: grow `self.bits` to `idx + 1` if needed, filling with `false`
- `align_to_indexer()`: grow bits to match indexer length before bitwise ops
- `pop_first()`: `self.bits.first_one()` → resolve index → clear bit → decrement cached_len. The `first_one()` method internally scans u64 words with hardware `ctz` (count trailing zeros) — effectively O(n/64).
- `iter()`: collect `bits.iter_ones()` → resolve each → sort by T (for determinism)
- `count_ones()`: the bitvec crate's implementation uses `u64::count_ones()` which compiles to a single `popcnt` instruction on x86, making the recount after bulk ops very fast.

**Tests** (add in `#[cfg(test)] mod tests` at the end of the file):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{LocId, ValueId, BlockId};

    #[test]
    fn empty_set() {
        let set = IdBitSet::<LocId>::empty();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut set = IdBitSet::<ValueId>::empty();
        assert!(set.insert(ValueId::new(42)));
        assert!(!set.insert(ValueId::new(42))); // duplicate
        assert!(set.contains(ValueId::new(42)));
        assert!(!set.contains(ValueId::new(99)));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove() {
        let mut set = IdBitSet::<LocId>::empty();
        set.insert(LocId::new(1));
        set.insert(LocId::new(2));
        assert!(set.remove(LocId::new(1)));
        assert!(!set.remove(LocId::new(1))); // already removed
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn union_shared_indexer() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        b.insert(LocId::new(2));
        assert!(a.union(&b));
        assert_eq!(a.len(), 2);
        assert!(a.contains(LocId::new(1)));
        assert!(a.contains(LocId::new(2)));
    }

    #[test]
    fn union_different_indexer() {
        let mut a = IdBitSet::<LocId>::empty();
        let mut b = IdBitSet::<LocId>::empty();
        a.insert(LocId::new(1));
        b.insert(LocId::new(2));
        assert!(a.union(&b));
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn pop_first() {
        let mut set = IdBitSet::<ValueId>::empty();
        set.insert(ValueId::new(10));
        set.insert(ValueId::new(20));
        let first = set.pop_first();
        assert!(first.is_some());
        assert_eq!(set.len(), 1);
        let second = set.pop_first();
        assert!(second.is_some());
        assert!(set.is_empty());
        assert_eq!(set.pop_first(), None);
    }

    #[test]
    fn clone_empty_shares_indexer() {
        let mut set = IdBitSet::<BlockId>::empty();
        set.insert(BlockId::new(1));
        let empty = set.clone_empty();
        assert!(empty.is_empty());
        assert!(Arc::ptr_eq(set.indexer(), empty.indexer()));
    }

    #[test]
    fn to_and_from_btreeset() {
        let mut original = BTreeSet::new();
        original.insert(LocId::new(3));
        original.insert(LocId::new(1));
        original.insert(LocId::new(2));
        let bitset = IdBitSet::from_btreeset(&original);
        assert_eq!(bitset.to_btreeset(), original);
    }

    #[test]
    fn equality_same_indexer() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.insert(LocId::new(1));
        b.insert(LocId::new(1));
        assert_eq!(a, b);
    }

    #[test]
    fn equality_different_indexer() {
        let mut a = IdBitSet::<LocId>::empty();
        let mut b = IdBitSet::<LocId>::empty();
        a.insert(LocId::new(1));
        b.insert(LocId::new(1));
        assert_eq!(a, b);
    }

    #[test]
    fn extend_from_iter() {
        let mut set = IdBitSet::<LocId>::empty();
        set.extend([LocId::new(1), LocId::new(2), LocId::new(3)]);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn intersect_and_difference() {
        let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
        let mut a = IdBitSet::with_indexer(Arc::clone(&indexer));
        let mut b = IdBitSet::with_indexer(Arc::clone(&indexer));
        a.extend([LocId::new(1), LocId::new(2), LocId::new(3)]);
        b.extend([LocId::new(2), LocId::new(3), LocId::new(4)]);

        let mut intersected = a.clone();
        intersected.intersect(&b);
        assert_eq!(intersected.to_btreeset(), [LocId::new(2), LocId::new(3)].into_iter().collect());

        let mut diffed = a.clone();
        diffed.difference(&b);
        assert_eq!(diffed.to_btreeset(), [LocId::new(1)].into_iter().collect());
    }
}
```

**File 2**: Update `crates/saf-analysis/src/pta/ptsset/mod.rs`:
- Add `mod id_bitset;` after the existing module declarations (line 52)
- Add `pub use id_bitset::IdBitSet;` to the re-exports section

---

### Task 3: PTA CI Solver Worklists → `IdBitSet`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/pta/solver.rs`. Read the full file first. Replace the BTreeSet worklists with IdBitSet for allocation-free insert/pop.

**Step 1**: Add import at top:
```rust
use super::ptsset::{IdBitSet, Indexer};
```

**Step 2**: In `GenericSolver` struct, replace the worklist fields:
```rust
    // Old:
    worklist: BTreeSet<ValueId>,
    loc_worklist: BTreeSet<LocId>,
    // New:
    worklist: IdBitSet<ValueId>,
    loc_worklist: IdBitSet<LocId>,
```

**Step 3**: In `GenericSolver::new()`, initialize with `IdBitSet::empty()`:
```rust
    worklist: IdBitSet::empty(),
    loc_worklist: IdBitSet::empty(),
```

**Step 4**: In `new_with_template()`, same initialization.

**Step 5**: Find all `self.worklist.insert(...)` calls and verify they work unchanged — `IdBitSet::insert()` has the same signature `insert(item) -> bool`. No changes needed.

**Step 6**: Find all `self.worklist.pop_first()` calls. `IdBitSet::pop_first()` has the same return type `Option<T>`. No changes needed.

**Step 7**: The `is_empty()` check in the loop termination — `IdBitSet::is_empty()` works the same. No changes needed.

**Step 8**: Remove `BTreeSet` from the worklist-related imports if no longer needed for worklists (but keep it if used elsewhere in the file, e.g., for constraint iteration).

**All existing tests must pass unchanged.** The worklist pop order may differ (index order vs ValueId order), but the PTA fixpoint is unique so results are identical.

---

### Task 4: MSSA Mod/Ref Summaries → `IdBitSet<LocId>`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/mssa/modref.rs`. Read the full file first. Replace `BTreeSet<LocId>` in `ModRefSummary` with `IdBitSet<LocId>` for O(n/64) fixpoint unions.

**Step 1**: Add imports:
```rust
use std::sync::{Arc, RwLock};
use crate::pta::ptsset::{IdBitSet, Indexer};
```

**Step 2**: Change `ModRefSummary` fields:
```rust
pub struct ModRefSummary {
    pub may_mod: IdBitSet<LocId>,
    pub may_ref: IdBitSet<LocId>,
}
```

Remove the `PartialEq, Eq` derives (IdBitSet implements `PartialEq` + `Eq` already, so derive still works — but verify; if not, implement manually).

**Step 3**: Update `ModRefSummary::empty()` to use a shared indexer. Add a parameter:
```rust
impl ModRefSummary {
    pub fn empty() -> Self {
        Self {
            may_mod: IdBitSet::empty(),
            may_ref: IdBitSet::empty(),
        }
    }

    /// Create an empty summary sharing the given indexer for fast unions.
    pub fn with_indexer(indexer: &Arc<RwLock<Indexer<LocId>>>) -> Self {
        Self {
            may_mod: IdBitSet::with_indexer(Arc::clone(indexer)),
            may_ref: IdBitSet::with_indexer(Arc::clone(indexer)),
        }
    }
}
```

**Step 4**: In `compute_mod_ref()`, create a shared indexer at the top and use it for all summaries:
```rust
pub fn compute_mod_ref(...) -> BTreeMap<FunctionId, ModRefSummary> {
    let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
    let mut summaries = BTreeMap::new();
    // ...
```

Replace `ModRefSummary::empty()` calls with `ModRefSummary::with_indexer(&indexer)`.

**Step 5**: In `compute_function_summary()`, accept the indexer and use it:
```rust
fn compute_function_summary(
    module: &AirModule,
    func_id: FunctionId,
    pta: &PtaResult,
    existing: &BTreeMap<FunctionId, ModRefSummary>,
    indexer: &Arc<RwLock<Indexer<LocId>>>,
) -> ModRefSummary {
    let mut summary = ModRefSummary::with_indexer(indexer);
    // ...
```

The `.extend(pta.points_to(ptr))` calls work because `IdBitSet::extend()` accepts `impl IntoIterator<Item = T>` and `pta.points_to()` returns an iterator of LocId.

For `summary.may_mod.extend(&callee_summary.may_mod)` — this won't work directly because `extend` takes `IntoIterator<Item = T>`, not `&IdBitSet`. Replace with:
```rust
summary.may_mod.union(&callee_summary.may_mod);
summary.may_ref.union(&callee_summary.may_ref);
```
This is the key win: `union()` is O(n/64) with shared indexer.

**Step 6**: Update all callers of `compute_function_summary` to pass the indexer.

**Step 7**: Update consumers in other files. Search for `may_mod` and `may_ref` usage across the codebase:

- `mssa/walker.rs` (~line 155): `summary.may_mod.contains(&loc)` — `IdBitSet::contains()` takes `T` by value not reference. Change to `summary.may_mod.contains(loc)` (remove the `&`).
- `saf-python/src/mssa.rs` (~line 118): `summary.may_mod.iter()` — works unchanged (IdBitSet has `.iter()`). But check if the Python code uses `.iter()` or `for x in &summary.may_mod`. If the latter, change to `.iter()`.
- `mssa/mod.rs` (~line 90): just stores the result — works unchanged.
- `mssa/export.rs`: check for any may_mod/may_ref iteration and update if needed.

**Step 8**: Update tests in `modref.rs`. The test `make_simple_pta` builds `BTreeSet<LocId>` for the PTA — these stay as BTreeSet (PTA result type is unchanged). Tests that check `s.may_mod.is_empty()` or `s.may_mod.is_superset()` — `IdBitSet` has `is_empty()` but may not have `is_superset()`. For the superset check, convert: `s.may_mod.to_btreeset().is_superset(&callee_s.may_mod.to_btreeset())` or add a helper. Alternatively, check via: every element in callee is in caller.

---

### Task 5: Absint Fixpoint `loop_headers` → `IdBitSet<BlockId>`

**Self-contained instructions for agent:**

Edit file `crates/saf-analysis/src/absint/fixpoint.rs`. Read the full file first. Replace `loop_headers: BTreeSet<BlockId>` with `IdBitSet<BlockId>` for O(1) membership tests in the tight fixpoint loop.

**Step 1**: Add imports at the top of the file:
```rust
use crate::pta::ptsset::{IdBitSet, Indexer};
use std::sync::{Arc, RwLock};
```

**Step 2**: Find the `detect_loop_headers` function (or however loop headers are computed — it's at the bottom of the file, ~line 1256). Change its return type from `BTreeSet<BlockId>` to `IdBitSet<BlockId>`:

```rust
fn detect_loop_headers(cfg: &Cfg) -> IdBitSet<BlockId> {
    let mut headers = IdBitSet::<BlockId>::empty();
    // ... same DFS logic, but use headers.insert(block) instead of BTreeSet insert
    headers
}
```

**Step 3**: Find all `loop_headers.contains(...)` calls in the file (should be in the ascending phase ~line 518 and narrowing phase ~line 907). These work unchanged — `IdBitSet` has `.contains()`.

**Step 4**: Also convert the `reached` set. Find all occurrences of:
```rust
let reached: BTreeSet<BlockId> = block_entry_states
    .iter()
    .filter(|(_, s)| !s.is_unreachable())
    .map(|(id, _)| *id)
    .collect();
```
Replace with:
```rust
let mut reached = IdBitSet::<BlockId>::empty();
for (id, s) in block_entry_states.iter() {
    if !s.is_unreachable() {
        reached.insert(*id);
    }
}
```

There are multiple occurrences (~5) — replace all of them.

**Step 5**: Update `apply_transfer` signature and `TransferContext` struct. Find `TransferContext` in `transfer.rs`:
```rust
// In transfer.rs:
pub reached_blocks: Option<&'a BTreeSet<BlockId>>,
// Change to:
pub reached_blocks: Option<&'a IdBitSet<BlockId>>,
```

Add the necessary import in `transfer.rs`:
```rust
use crate::pta::ptsset::IdBitSet;
```

The `reached.contains(block_id)` call in transfer.rs works unchanged.

**Step 6**: Also update any other function in fixpoint.rs that builds `BTreeSet<BlockId>` for the `reached` set (search for all `BTreeSet<BlockId>` in the file). The DFS helper functions (`dfs_find_back_edges`, `reverse_postorder`) may use `BTreeSet<BlockId>` for their internal `visited` sets — leave these as `BTreeSet` since they're one-time computations, not in the hot path.

**All existing tests must pass unchanged.**

---

### Task 6: DDA Cache → `IdBitSet<LocId>`

**Self-contained instructions for agent:**

Edit files in `crates/saf-analysis/src/dda/`. Read `types.rs` and `solver.rs` fully first.

**Step 1**: In `types.rs`, add import:
```rust
use crate::pta::ptsset::{IdBitSet, Indexer};
use std::sync::{Arc, RwLock};
```

**Step 2**: Change `DdaCache` struct:
```rust
pub struct DdaCache {
    tl_cache: std::collections::BTreeMap<Dpm, IdBitSet<saf_core::ids::LocId>>,
    /// Shared indexer for all cached sets.
    loc_indexer: Arc<RwLock<Indexer<saf_core::ids::LocId>>>,
    visited: BTreeSet<Dpm>,
}
```

**Step 3**: Update `DdaCache::new()`:
```rust
pub fn new() -> Self {
    Self {
        tl_cache: std::collections::BTreeMap::new(),
        loc_indexer: Arc::new(RwLock::new(Indexer::new())),
        visited: BTreeSet::new(),
    }
}
```

**Step 4**: Update `get()` return type:
```rust
pub fn get(&self, dpm: &Dpm) -> Option<&IdBitSet<saf_core::ids::LocId>> {
    self.tl_cache.get(dpm)
}
```

**Step 5**: Update `insert()`:
```rust
pub fn insert(&mut self, dpm: Dpm, pts: BTreeSet<saf_core::ids::LocId>) {
    let bitset = IdBitSet::from_btreeset_with_indexer(&pts, Arc::clone(&self.loc_indexer));
    self.tl_cache.insert(dpm, bitset);
}
```

This preserves the existing `insert(Dpm, BTreeSet<LocId>)` signature so callers don't change. Internally converts to `IdBitSet` with the shared indexer. If you find callers already have access to IdBitSet, you can add an `insert_bitset` method too.

**Step 6**: Update `clear()`:
```rust
pub fn clear(&mut self) {
    self.tl_cache.clear();
    self.visited.clear();
    // Keep the indexer — it's reused across queries
}
```

**Step 7**: In `solver.rs`, find where cache results are consumed. The consumer likely calls `.get()` and then iterates or converts the result. Since `get()` now returns `Option<&IdBitSet<LocId>>`, the consumer needs to use `.iter()` or `.to_btreeset()` where it previously had `&BTreeSet<LocId>`.

Search for all `cache.get(` calls and update:
- If the consumer iterates: use `.iter()` (works on both types)
- If the consumer needs `BTreeSet<LocId>`: call `.to_btreeset()`
- If the consumer does set operations: use IdBitSet methods

**All existing tests must pass unchanged.**

---

## Execution Order

```
Group 1 (sequential): Tasks 1 → 2
  Task 2 depends on Task 1 (IdBitSet uses Indexer<T>)
Group 2 (parallel): Tasks 3, 4, 5, 6
  All depend on Tasks 1-2 being complete
  No dependencies between them (different files)
```

## Verification (main agent only)

After all tasks complete:

1. `make fmt && make lint`
2. `make test`
3. PTABen regression (run in background):
   ```bash
   docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-idbitset.json'
   ```
4. Compare against baseline: 2239 Exact, 392 Sound, 80 Unsound, 93 Skip. Any regression is a blocker.

## Key Conventions

- `BTreeMap`/`BTreeSet` everywhere for determinism (NFR-DET) — IdBitSet preserves this via deterministic indexer
- No `.unwrap()` in library code — use `.expect("message")`
- Doc comments: backtick type names (clippy `doc_markdown`)
- Function-level `#[allow]` with comments, not crate-level
