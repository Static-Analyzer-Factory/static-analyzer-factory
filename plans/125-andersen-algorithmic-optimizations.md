# Plan 125: Andersen PTA Algorithmic Optimizations

> **Status: REVERTED** — All 3 optimizations implemented and passed correctness tests but did not achieve performance target. Changes reverted.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce Andersen PTA phase from ~44s to ~10-15s on `big/bash` through three independent algorithmic improvements: union-find path compression (Gap 4), Hybrid Cycle Detection (Gap 2), and extended offline variable substitution HRU (Gap 5). These address the remaining gaps from Plan 123 after Plan 124 proved representation switching alone is insufficient.

**Architecture:** All three optimizations operate on the existing `GenericSolver` in `solver.rs` and the HVN pre-processor in `hvn.rs`. They are independent and composable — each reduces the total work the solver performs rather than changing data structure representation.

**Tech Stack:** Rust, existing PTA solver infrastructure.

**Key risk:** HRU (Gap 5) is the most complex change. If constraint demotion introduces soundness bugs, PTABen regression gate catches it. Each task is independently revertible.

---

## Results (2026-02-19)

All three optimizations (Cell\<ValueId\> path compression, HCD offline table, HD extended HVN) were implemented, tested for correctness, and benchmarked. Each was found to cause net performance regression. All solver changes reverted. Only clippy lint fixes retained.

| Configuration | Andersen (s) | Notes |
|---|---|---|
| Baseline (pre-Plan-125) | ~44-48 | Varies with Docker/host load |
| Cell\<ValueId\> + canonicalization | ~58 | Cell overhead on hot find\_rep path |
| + redirect\_node + expand\_solver\_merges | ~51 | Redirect causes more constraint processing |
| All reverted (current) | ~48 | Back to baseline |

**Key findings:**
1. **Cell\<ValueId\> overhead dominates:** 16-byte Cell\<u128\> on every find\_rep call (hottest path) causes ~30% regression. Path compression doesn't compensate because Tarjan SCC already keeps chains short.
2. **redirect\_node causes more work:** Transferring constraint index entries from merged→rep node makes previously-orphaned constraints discoverable, increasing total work per process\_value call by ~15%.
3. **expand\_solver\_merges is expensive:** Post-solve cloning of BTreeSet\<LocId\> for all merged nodes costs ~7s on bash.
4. **Canonicalization find\_rep calls on hot paths:** Extra IndexMap lookups in union\_into\_value, worklist\_insert, store/load handlers add ~15% overhead.
5. **HCD + diff-based solver incompatibility:** Eager cycle merging before convergence creates large diffs that cascade through the worklist. SVF avoids this because it doesn't use diff-based propagation.
6. **HD ≈ HVN for sparse workloads:** Extended offline graph with REF nodes and Load/Store demotion provides no benefit when pts sets have 1-3 locations.
7. **Pre-existing canonicalization "bug" is benign:** Without find\_rep in union\_into\_value/worklist\_insert, some locs after SCC merge may go to orphaned pts entries. PTABen shows no measurable impact (same 69 Unsound).

---

## Baseline (record before implementing)

Run CruxBC benchmarks on small + big programs:

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small,big" -o /workspace/tests/benchmarks/cruxbc/baseline-125.json'
```

Also run PTABen correctness baseline (expect 2236 Exact, 69 Unsound):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/baseline-125.json'
```

---

### Task 1: Union-Find Path Compression (Gap 4)

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs`

**Step 1: Change `rep` map to use `Cell<ValueId>`**

In `solver.rs`, change the `rep` field type from:
```rust
rep: IndexMap<ValueId, ValueId>,
```
to:
```rust
rep: IndexMap<ValueId, Cell<ValueId>>,
```

Add `use std::cell::Cell;` to imports.

**Step 2: Update `find_rep` with path halving**

Replace the current `find_rep`:
```rust
fn find_rep(&self, mut v: ValueId) -> ValueId {
    while let Some(&r) = self.rep.get(&v) {
        if r == v {
            break;
        }
        v = r;
    }
    v
}
```

With path-halving path compression:
```rust
fn find_rep(&self, mut v: ValueId) -> ValueId {
    // Find root
    let mut root = v;
    while let Some(cell) = self.rep.get(&root) {
        let r = cell.get();
        if r == root {
            break;
        }
        root = r;
    }
    // Path compression: point all intermediate nodes directly to root
    while v != root {
        let Some(cell) = self.rep.get(&v) else { break };
        let next = cell.get();
        if next == root {
            break;
        }
        cell.set(root);
        v = next;
    }
    root
}
```

**Step 3: Update all `rep` write sites**

Search for all `self.rep.insert(...)` and `self.rep.get(...)` call sites and update to use `Cell` API:
- `self.rep.insert(v, v)` → `self.rep.insert(v, Cell::new(v))`
- `self.rep.insert(other, rep)` → `self.rep.insert(other, Cell::new(rep))`
- Read sites: `self.rep.get(&v).map(|c| c.get())` — but `find_rep` already encapsulates this

Find all write sites with: `grep -n 'self\.rep\.insert' crates/saf-analysis/src/pta/solver.rs`

**Step 4: Verify compilation and tests**

```bash
docker compose run --rm dev sh -c 'cargo nextest run --release -p saf-analysis 2>&1 | tail -5'
```

Expected: all tests pass. No behavior change — same results, just faster path traversal.

**Step 5: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): add union-find path compression via Cell<ValueId> (Gap 4)"
```

---

### Task 2: Hybrid Cycle Detection (Gap 2)

**Files:**
- Create: `crates/saf-analysis/src/pta/hcd.rs` (~100-150 lines)
- Modify: `crates/saf-analysis/src/pta/mod.rs` (add module)
- Modify: `crates/saf-analysis/src/pta/solver.rs` (integrate HCD table)

**Step 1: Implement the HCD offline pre-pass**

Create `crates/saf-analysis/src/pta/hcd.rs`:

```rust
//! Hybrid Cycle Detection (HCD) for Andersen's pointer analysis.
//!
//! Implements the offline pre-pass from Hardekopf & Lin (PLDI 2007).
//! Builds a lookup table mapping dereferenced variables to cycle anchors,
//! enabling eager cycle collapse during online solving.

use indexmap::IndexMap;
use saf_core::ids::ValueId;
use crate::pta::constraint::{ConstraintSet, LoadConstraint, StoreConstraint};
```

The module exposes one public function:

```rust
/// Build the HCD lookup table from the initial constraint set.
///
/// Returns a map from `ValueId` (a variable that appears dereferenced in
/// Load/Store) to its cycle anchor `ValueId`. During solving, when node `n`
/// is processed and `n ∈ hcd_table`, every location in `pts(n)` should be
/// collapsed with `hcd_table[n]`.
pub(crate) fn build_hcd_table(constraints: &ConstraintSet) -> IndexMap<ValueId, ValueId>
```

Algorithm:
1. Collect all variables that appear dereferenced: `src` in Load constraints (`dst ⊇ *src`), `dst` in Store constraints (`*dst ⊇ src`). These get virtual REF nodes.
2. Build directed graph with node IDs as `u32` indices. Real variable nodes: 0..N. REF nodes: N..2N. Edges:
   - Copy `dst ⊇ src` → edge `src_idx → dst_idx`
   - Load `dst ⊇ *src` → edge `ref_idx(src) → dst_idx`
   - Store `*dst ⊇ src` → edge `src_idx → ref_idx(dst)`
3. Run iterative Tarjan SCC on this graph.
4. For each SCC with >1 node that contains at least one REF node `ref_idx(a)`:
   - Pick any non-REF node `b` as anchor
   - Insert `(a, b)` into the result map (using original `ValueId`s)
5. Return the map.

Add unit tests:
- Simple cycle: `a = *b; *a = b` → `(b, a)` or `(a, b)` in table
- No cycle: `a = *b; c = *d` → empty table
- Mixed: cycle involving one Load + one Copy chain

**Step 2: Register the module**

In `crates/saf-analysis/src/pta/mod.rs`, add:
```rust
mod hcd;
pub(crate) use hcd::build_hcd_table;
```

**Step 3: Integrate into solver**

In `solver.rs`, add a field to `GenericSolver`:
```rust
hcd_table: IndexMap<ValueId, ValueId>,
```

Initialize it in `new()` and `new_with_template()`:
```rust
hcd_table: hcd::build_hcd_table(constraints),
```

In `process_value()`, add the HCD check at the top (after `find_rep`, before diff computation):
```rust
fn process_value(&mut self, v: ValueId) {
    let v = self.find_rep(v);

    // HCD: eagerly collapse cycle members
    if let Some(&anchor) = self.hcd_table.get(&v) {
        let anchor = self.find_rep(anchor);
        if anchor != v {
            if let Some(pts_clone) = self.pts.get(&v).cloned() {
                for loc in pts_clone.iter() {
                    let loc_rep = self.find_rep(loc);
                    if loc_rep != anchor {
                        let (rep, other) = if anchor < loc_rep {
                            (anchor, loc_rep)
                        } else {
                            (loc_rep, anchor)
                        };
                        self.merge_nodes(rep, other);
                    }
                }
                self.worklist_insert(self.find_rep(anchor));
            }
        }
    }

    // ... rest of process_value unchanged
```

**Step 4: Increase periodic Tarjan threshold**

Change line 625 from:
```rust
if value_pops % 50_000 == 0 {
```
to:
```rust
if value_pops % 500_000 == 0 {
```

HCD now handles structural cycles eagerly; periodic Tarjan is just a safety net.

**Step 5: Verify**

```bash
docker compose run --rm dev sh -c 'cargo nextest run --release -p saf-analysis 2>&1 | tail -5'
```

Run PTABen (background):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-125-hcd.json'
```

Expected: 2236 Exact, 69 Unsound (no regression).

**Step 6: Commit**

```bash
git add crates/saf-analysis/src/pta/hcd.rs crates/saf-analysis/src/pta/mod.rs crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): add Hybrid Cycle Detection for eager cycle collapse (Gap 2)"
```

---

### Task 3: Extended Offline Variable Substitution — HD (Gap 5)

**Files:**
- Modify: `crates/saf-analysis/src/pta/hvn.rs` (major rewrite)
- Modify: `crates/saf-analysis/src/pta/solver.rs` (minor: accept demoted constraints)

**Step 1: Extend the offline graph with REF nodes**

In `hvn.rs`, replace the current three-category signature enum with a proper offline graph:

```rust
/// Node in the HVN/HD offline constraint graph.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum OfflineNode {
    /// A real program variable.
    Var(ValueId),
    /// Virtual dereference node for a variable (*v).
    Ref(ValueId),
    /// Address-of node for a location.
    Adr(LocId),
}
```

Build the offline graph from constraints:
- Addr `ptr ⊇ {loc}` → edge `Adr(loc) → Var(ptr)` + edge `Var(loc) → Ref(ptr)` + mark `loc` as indirect
- Copy `dst ⊇ src` → edge `Var(src) → Var(dst)` + edge `Ref(src) → Ref(dst)`
- Load `dst ⊇ *src` → edge `Ref(src) → Var(dst)` (instead of marking dst Complex!)
- Store `*dst ⊇ src` → edge `Var(src) → Ref(dst)` (instead of marking dst Complex!)

**Step 2: SCC + topological label assignment**

Run Tarjan SCC on the offline graph. Collapse pure-Var SCCs (same as current HVN). For label assignment in topological order:
- Indirect nodes (Ref nodes, address-taken locations) → fresh unique label
- Direct Var nodes → hash of predecessor labels (same as current HVN, but now predecessors can include Ref nodes instead of being `Complex`)

Two Var nodes with the same label are pointer-equivalent and can be merged.

**Step 3: Constraint demotion**

After label assignment, scan Load and Store constraints:
- Load `dst ⊇ *src_ptr`: if `src_ptr`'s label equals some `Adr(X)` label → demote to Copy `dst ⊇ X`
- Store `*dst_ptr ⊇ src`: if `dst_ptr`'s label equals some `Adr(X)` label → demote to Copy `X ⊇ src`

Return demoted constraints as additional Copy constraints in the `HvnResult`.

**Step 4: Update `HvnResult` to include demoted constraints**

Add to `HvnResult`:
```rust
pub demoted_loads: Vec<CopyConstraint>,  // Load→Copy demotions
pub demoted_stores: Vec<CopyConstraint>, // Store→Copy demotions
```

In the solver's `solve_with_hvn` (or wherever HVN is called), add demoted constraints to the reduced constraint set and remove the original Load/Store constraints that were demoted.

**Step 5: Preserve existing HVN tests + add new ones**

Existing tests should still pass (Var-only merging is unchanged). Add tests for:
- Load from a single-target pointer → demoted to Copy
- Store to a single-target pointer → demoted to Copy
- Two loads from the same pointer → their dst values merged
- Chain: `a = &x; b = *a; c = *a` → b and c are pointer-equivalent

**Step 6: Verify**

Full test suite:
```bash
docker compose run --rm dev sh -c 'cargo nextest run --release 2>&1 | tail -5'
```

PTABen (background):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-125-hru.json'
```

Expected: 2236 Exact, 69 Unsound (no regression). Log the constraint reduction percentage — target: 60-77% (up from 20-40%).

**Step 7: Commit**

```bash
git add crates/saf-analysis/src/pta/hvn.rs crates/saf-analysis/src/pta/solver.rs
git commit -m "perf(pta): extend HVN to HD with REF nodes and Load/Store demotion (Gap 5)"
```

---

### Task 4: Run performance benchmarks and validate

**Step 1: Run CruxBC performance benchmark**

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled --filter "small,big" -o /workspace/tests/benchmarks/cruxbc/results-125.json'
```

**Step 2: Compare with baseline**

```bash
python3 -c "
import json
for label, path in [('baseline', 'tests/benchmarks/cruxbc/baseline-125.json'), ('after', 'tests/benchmarks/cruxbc/results-125.json')]:
    with open(path) as f:
        data = json.load(f)
    print(f'\n=== {label} ===')
    for p in data.get('programs', data.get('results', [])):
        name = p.get('name', p.get('program', '?'))
        ander = p.get('pta_solve_secs', p.get('andersen_secs', '?'))
        total = p.get('total_secs', '?')
        print(f'  {name}: Ander={ander:.2f}s, Total={total:.2f}s')
"
```

**Expected improvements:**
- `big/bash`: Andersen from ~44s down to ~10-15s (3-5x improvement)
- Small programs: no regression (already fast)

**Gate:** If any program is >2x slower than baseline, investigate which gap caused it and revert selectively.

---

### Task 5: Format, lint, update PROGRESS.md

**Step 1: Format and lint**

```bash
make fmt && make lint
```

**Step 2: Update PROGRESS.md**

Add Plan 125 entry to Plans Index:
```
| 125 | andersen-algorithmic-optimizations | performance | done |
```

Add Session Log entry with benchmark results (Andersen phase before/after, constraint reduction percentage).

Update "Next Steps" based on results.

**Step 3: Final commit**

```bash
git add plans/125-andersen-algorithmic-optimizations.md plans/PROGRESS.md
git commit -m "docs: add Plan 125 (Andersen algorithmic optimizations) and results"
```

---

## Rollback Plan

Each gap is independently revertible:

- **Gap 4 (path compression):** Revert `Cell<ValueId>` back to plain `ValueId` in `rep` map
- **Gap 2 (HCD):** Remove `hcd_table` field and the `process_value` check; restore 50K Tarjan trigger
- **Gap 5 (HRU):** Revert `hvn.rs` to the simple 3-category signature; remove constraint demotion

## Future Work (if this succeeds)

- **HU extension:** Add offline points-to set union propagation to HD for more merging opportunities
- **Location Equivalence (LE):** Merge co-pointed-to objects (dual of pointer equivalence)
- **Gap 6 (PWC):** Positive weight cycle handling with stride-based field representation
- **Hybrid PtsSet:** With fewer constraints post-HRU, revisit cardinality-based BTreeSet→BitVec switching
