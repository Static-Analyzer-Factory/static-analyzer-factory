# Plan 103: PTA Solver Scalability — Constraint Indexing, SCC, Diff Propagation

## Context

CruxBC benchmarks (Plan 102) reveal SAF's Andersen solver is 56-964x slower than SVF on
medium programs (bc: 81s vs 1.45s, bunzip2: 1620s vs 1.68s). Investigation by 3 agents
identified 6 root causes (see below). This plan fixes P0–P5.

**Root causes (ordered by impact):**

| ID | Issue | Est. Speedup |
|----|-------|-------------|
| P0 | Linear scan of ALL constraints per worklist pop | 50-100x |
| P1 | No online SCC detection / cycle collapsing | 10-50x |
| P2 | Full pts-set clone per pop (no diff propagation) | 5-20x |
| P3 | Linear scan of ALL locations in `find_or_approximate_location` | 2-10x |
| P4 | `cg_refinement.rs` hardcodes `BTreePtsSet` via `solve()` | 2-5x |
| P5 | Arbitrary worklist order (no topological awareness) | 2-3x |

**Key constraint:** Nearly all fixes touch `solver.rs` (1186 lines). To enable parallel
agents, we extract new types into **new files** (no file conflicts) and wire them in
during a sequential leader integration phase.

## Architecture

```
Phase 1 (3 agents parallel — new files only, no solver.rs edits):
  Agent 1: crates/saf-analysis/src/pta/constraint_index.rs  (P0 data structure)
  Agent 2: location.rs — add lookup_by_obj_path() method     (P3 data structure)
  Agent 3: cg_refinement.rs — switch solve() → solve_with_config() (P4)

Phase 2 (leader — solver.rs integration):
  Wire P0: replace linear scans with ConstraintIndex lookups
  Wire P3: replace find_or_approximate_location 3x scan with factory lookup
  Wire P2: add diff-set tracking, propagate only new elements

Phase 3 (2 agents sequential — solver.rs, one at a time):
  Agent 4: solver.rs — online SCC detection + node merging (P1)
  Agent 5: solver.rs — topological worklist ordering (P5)

Verification: make fmt && make lint && make test-rust
Benchmark: make test-cruxbc (dc + bc on small set)
```

## Agent Tasks

### Agent 1: Create `ConstraintIndex` (P0 — new file)

**File to create:** `crates/saf-analysis/src/pta/constraint_index.rs`

**Context (do NOT read solver.rs — too large):** The solver currently stores constraints
in `ConstraintSet` which has 5 flat `BTreeSet` fields: `addr`, `copy`, `load`, `store`,
`gep`. The solver iterates ALL constraints of each type on every worklist pop to find
matching ones. We need a pre-built index that maps `ValueId → [constraint indices]`.

**Read these files only:**
- `crates/saf-analysis/src/pta/constraint.rs` (299 lines — the constraint types)
- `crates/saf-analysis/src/pta/mod.rs` (96 lines — add module declaration)

**Specification:**

```rust
//! Pre-built index for fast constraint lookup by ValueId.

use std::collections::BTreeMap;
use saf_core::ids::ValueId;
use super::constraint::ConstraintSet;

/// Index for O(1) constraint lookup by source/destination ValueId.
///
/// Built once before solving; each `handle_*` method looks up only
/// the constraints relevant to the current worklist value.
pub struct ConstraintIndex {
    /// Copy constraints where `src == v`.
    pub copy_by_src: BTreeMap<ValueId, Vec<usize>>,
    /// Load constraints where `src_ptr == v`.
    pub load_by_src_ptr: BTreeMap<ValueId, Vec<usize>>,
    /// Store constraints where `dst_ptr == v`.
    pub store_by_dst_ptr: BTreeMap<ValueId, Vec<usize>>,
    /// Store constraints where `src == v`.
    pub store_by_src: BTreeMap<ValueId, Vec<usize>>,
    /// GEP constraints where `src_ptr == v`.
    pub gep_by_src_ptr: BTreeMap<ValueId, Vec<usize>>,
}

impl ConstraintIndex {
    /// Build index from a constraint set.
    ///
    /// Converts BTreeSet constraints to Vec (for O(1) index access)
    /// and builds reverse maps keyed by the relevant ValueId field.
    pub fn build(constraints: &ConstraintSet) -> (Self, IndexedConstraints) { ... }
}

/// Constraints stored as Vecs for O(1) indexed access.
///
/// The `ConstraintIndex` stores `usize` indices into these Vecs.
pub struct IndexedConstraints {
    pub copy: Vec<super::constraint::CopyConstraint>,
    pub load: Vec<super::constraint::LoadConstraint>,
    pub store: Vec<super::constraint::StoreConstraint>,
    pub gep: Vec<super::constraint::GepConstraint>,
}
```

The `build()` method:
1. Convert each `BTreeSet` in `ConstraintSet` into a `Vec` (for O(1) index access)
2. Iterate each Vec, building the reverse-index maps
3. Return both the index and the indexed Vecs

**Also:** Add `pub mod constraint_index;` to `crates/saf-analysis/src/pta/mod.rs`.

---

### Agent 2: Add `LocationFactory` lookup methods (P3 — edit existing file)

**File to edit:** `crates/saf-analysis/src/pta/location.rs` (793 lines)

**Context:** The solver's `find_or_approximate_location(obj, path)` does 3 linear scans
of `factory.all_locations()` to find: exact match → parent path → base object. The
`LocationFactory` already has `id_map: BTreeMap<Location, LocId>` but the solver doesn't
use it. We need direct lookup methods.

**Read this file only:** `crates/saf-analysis/src/pta/location.rs`

**Specification:** Add these methods to `impl LocationFactory`:

```rust
/// Look up a location by object + exact field path.  O(log n).
pub fn lookup(&self, obj: ObjId, path: &FieldPath) -> Option<LocId> {
    let key = Location { obj, path: path.clone() };
    self.id_map.get(&key).copied()
}

/// Look up with fallback: exact → parent → base.  O(log n) each step.
///
/// Replaces the solver's 3x linear scan in `find_or_approximate_location`.
pub fn lookup_approx(&self, obj: ObjId, path: &FieldPath) -> Option<LocId> {
    // 1. Exact match
    if let Some(id) = self.lookup(obj, path) {
        return Some(id);
    }
    // 2. Parent path (truncate last step)
    if !path.steps.is_empty() {
        let parent = path.truncate(path.depth() - 1);
        if let Some(id) = self.lookup(obj, &parent) {
            return Some(id);
        }
    }
    // 3. Base object (empty path)
    self.lookup(obj, &FieldPath::empty())
}
```

**Note:** Check that `FieldPath` has an `empty()` constructor (or use `FieldPath { steps: vec![] }`).
Check that `Location` derives `Ord` (it does, since it's a `BTreeMap` key in `id_map`).

Add a unit test verifying `lookup_approx` returns exact > parent > base fallback.

---

### Agent 3: Use `solve_with_config` in CG refinement (P4 — edit existing file)

**File to edit:** `crates/saf-analysis/src/cg_refinement.rs` (989 lines)

**Context:** The `refine()` function calls `solve()` which hardcodes `BTreePtsSet`
(O(n log n) union). It should use `solve_with_config()` which auto-selects `BitVecPtsSet`
for medium programs (O(n/64) union via bitwise OR).

**Read these files only:**
- `crates/saf-analysis/src/cg_refinement.rs`
- `crates/saf-analysis/src/pta/solver.rs` lines 1-130 only (public API signatures)
- `crates/saf-analysis/src/pta/ptsset/config.rs` (PtsConfig type)

**Specification:**

1. Find all calls to `solve()` in `cg_refinement.rs` (likely 1-2 calls)
2. Replace with `solve_with_config()`, passing `PtsConfig::auto()` (or `PtsConfig::default()`)
3. Add the necessary `use` import for `PtsConfig`
4. The function signature of `solve_with_config` is:
   ```rust
   pub fn solve_with_config(
       constraints: &ConstraintSet,
       factory: &LocationFactory,
       max_iterations: usize,
       pts_config: &PtsConfig,
   ) -> PointsToMap
   ```
5. Verify the return type is still `PointsToMap` (it is — `solve_with_config` normalizes)

This is a minimal, low-risk change.

---

### Leader: Wire P0 + P3 + P2 into solver.rs (Phase 2)

**After agents 1-3 complete, the leader integrates into `solver.rs`.**

**Files to edit:** `crates/saf-analysis/src/pta/solver.rs`

**Read:** solver.rs (full), plus Agent 1's `constraint_index.rs`

**P0 integration — replace linear scans:**

1. In `GenericSolver::new()`, build the `ConstraintIndex`:
   ```rust
   let (index, indexed) = ConstraintIndex::build(constraints);
   ```
   Store `index` and `indexed` as fields on `GenericSolver`.

2. Rewrite `handle_copy_constraints`:
   ```rust
   fn handle_copy_constraints(&mut self, v: ValueId, v_pts: &P) {
       if let Some(indices) = self.index.copy_by_src.get(&v) {
           for &i in indices {
               let copy = &self.indexed.copy[i];
               if self.union_into_value(copy.dst, v_pts) {
                   self.worklist.insert(copy.dst);
               }
           }
       }
   }
   ```

3. Same pattern for `handle_load_constraints` (lookup `load_by_src_ptr`),
   `handle_store_constraints` (lookup both `store_by_dst_ptr` and `store_by_src`),
   `handle_gep_constraints` (lookup `gep_by_src_ptr`).

4. In `process_location`, replace the linear scan of load constraints with
   a reverse lookup. Add `load_by_dst` index if needed (loads where a location
   is in the src_ptr's pts — this requires a different indexing strategy;
   for now, keep the linear scan in `process_location` as it's less hot).

**P3 integration — replace `find_or_approximate_location`:**

Replace the body of `find_or_approximate_location` with:
```rust
self.factory.lookup_approx(obj, path)
```

**P2 integration — diff-based propagation:**

1. Add a `prev_pts: BTreeMap<ValueId, P>` field to `GenericSolver` (stores pts snapshot
   from last time each value was processed).

2. In `process_value`, compute diff instead of full clone:
   ```rust
   fn process_value(&mut self, v: ValueId) {
       let current = match self.pts.get(&v) {
           Some(s) => s.clone(),
           None => return,
       };
       // Compute diff: elements in current but not in prev
       let diff = if let Some(prev) = self.prev_pts.get(&v) {
           current.difference(prev)
       } else {
           current.clone()
       };
       if diff.is_empty() {
           return; // Nothing new to propagate
       }
       self.prev_pts.insert(v, current);
       self.handle_copy_constraints(v, &diff);
       self.handle_load_constraints(v, &diff);
       self.handle_store_constraints(v, &diff);
       self.handle_gep_constraints(v, &diff);
   }
   ```

3. This requires `PtsSet` trait to have a `difference(&self, other: &Self) -> Self` method.
   Check if it exists; if not, add it to the trait with a default implementation using
   `iter()` + `contains()`. The `is_empty()` method should already exist on the trait.

**Run `make fmt && make lint` after integration.**

---

### Agent 4: Online SCC Detection + Node Merging (P1 — edit solver.rs)

**Runs AFTER leader completes Phase 2.** Agent 4 receives solver.rs in its post-Phase-2 state.

**File to edit:** `crates/saf-analysis/src/pta/solver.rs`

**Context:** Cycles in the constraint graph cause the solver to repeatedly propagate the
same information. SVF uses online Tarjan's SCC detection to find and collapse cycles
during solving. When an SCC is found, all nodes merge to a single representative:
pts sets are unioned, all edges redirect to the rep, sub-nodes are removed.

**Read these files only:**
- `crates/saf-analysis/src/pta/solver.rs` (post-Phase-2 version)
- `crates/saf-analysis/src/pta/constraint_index.rs` (from Agent 1)

**Specification:**

1. Add SCC detection state to `GenericSolver`:
   ```rust
   /// Maps each merged node to its representative.
   rep: BTreeMap<ValueId, ValueId>,
   ```

2. Add a `find_rep(&self, v: ValueId) -> ValueId` method (path-compressed union-find):
   ```rust
   fn find_rep(&self, mut v: ValueId) -> ValueId {
       while let Some(&r) = self.rep.get(&v) {
           if r == v { break; }
           v = r;
       }
       v
   }
   ```

3. At the START of `process_value`, canonicalize: `let v = self.find_rep(v);`

4. Add `detect_and_collapse_cycles()` method, called periodically (e.g., every 1000
   worklist pops). Use a simplified approach:
   - Build a directed graph from the copy constraints (src → dst where both have non-empty pts)
   - Run Tarjan's SCC on this graph
   - For each SCC with >1 node: merge all pts sets into the representative,
     update `self.rep` for all non-rep nodes, add rep to worklist

5. In `merge_nodes(rep, other)`:
   - Union `pts[other]` into `pts[rep]`
   - Union `loc_pts` entries if applicable
   - Update `ConstraintIndex`: for each constraint referencing `other`,
     logically redirect to `rep` (or just rely on `find_rep` at lookup time)
   - Remove `other` from worklist, add `rep`

**Keep it simple.** A periodic Tarjan's (every N pops) is good enough for a first pass.
Full online cycle detection (during edge insertion) can come later.

**Do NOT read constraint_index.rs from disk** — the leader will have already integrated
it. Just use the `self.index` and `self.indexed` fields as documented.

---

### Agent 5: Topological Worklist Ordering (P5 — edit solver.rs)

**Runs AFTER Agent 4 completes.** Agent 5 receives solver.rs in its post-P1 state.

**File to edit:** `crates/saf-analysis/src/pta/solver.rs`

**Context:** SAF uses `IdBitSet` (arbitrary deterministic order) for the worklist. SVF
uses wave propagation: process nodes in topological order of the constraint graph, so
information flows "downhill" and each node is processed after its inputs stabilize.

**Read this file only:** `crates/saf-analysis/src/pta/solver.rs` (post-P1 version)

**Specification:**

1. Before the main solve loop, compute a topological ordering of the copy constraint
   graph (using the `ConstraintIndex`):
   ```rust
   fn compute_topo_order(&self) -> BTreeMap<ValueId, u32> {
       // Build adjacency list from copy constraints (src → dst)
       // Run Kahn's algorithm (BFS) to produce topo order
       // Return map: ValueId → priority (lower = process first)
   }
   ```

2. Replace the worklist from `IdBitSet<ValueId>` with a priority-based worklist
   that pops the lowest-topo-order value first. Options:
   - `BTreeMap<(u32, ValueId), ()>` keyed by (topo_priority, value_id)
   - Or keep `IdBitSet` but override pop to select by topo order (may be expensive)
   - Simplest: `BTreeSet<(u32, ValueId)>` — deterministic and topo-aware

3. When SCC collapsing changes the graph (Agent 4), the topo order may become stale.
   This is acceptable — stale topo order is still better than arbitrary order. A full
   re-computation can be added later if needed.

4. The `loc_worklist` for location processing can remain as `IdBitSet` (locations don't
   have meaningful topo order in copy constraints).

**Keep the change minimal.** If the priority worklist is too complex, a simpler approach
is to process the initial worklist in topo order and fall back to FIFO for later rounds.

---

## Verification

After all phases:

1. `make fmt && make lint` — clean (ignore pre-existing saf-wasm errors)
2. `make test-rust` — all existing tests pass (no regressions)
3. `make test-cruxbc` — benchmark dc and bc:
   - dc should remain ~0.1-0.3s
   - bc should drop from 81s to <5s (target: <2s)
   - bunzip2 should drop from 1620s to <30s (target: <5s)

## Non-goals

- Hash-consed points-to sets (P7) — separate plan
- HVN pre-solving (P8) — separate plan
- Incremental CG refinement warm-start — separate plan
- PWC field collapsing — separate plan
