# CG Refinement BitVec + Clustering Integration — Plan 124

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the hardcoded `BTreePtsSet` in CG refinement with `BitVecPtsSet` backed by Steensgaard-style object clustering, targeting 5-10x improvement on the Andersen phase for large programs (44s → ~5-10s on bash).

**Architecture:** CG refinement (`cg_refinement.rs:154-155`) currently creates `GenericSolver::<BTreePtsSet>::new(...)`, bypassing all existing clustering and representation-selection infrastructure. The fix: (1) expose `create_template` from `solver.rs`, (2) call it with `ClusteringMode::Auto` to run `approximate_cooccurrence` → hierarchical clustering → pre-seed a `LocIdIndexer` with co-occurring locations as adjacent bit indices, (3) instantiate the solver as `GenericSolver::<BitVecPtsSet>::new_with_template(...)`. All downstream code (`resolve_and_connect`, `to_btreeset` normalization, HVN expansion) is already generic over `PtsSet`.

**Tech Stack:** Rust, existing `clustering.rs` / `ptsset/bitvec.rs` / `solver.rs` infrastructure.

**Key risk:** Plan 110 showed 6x regression when switching to BitVec *without* clustering. Clustering should fix this by making bitvector operations cache-efficient for sparse points-to sets. Rollback threshold: revert if any CruxBC program is >5x slower than baseline.

---

## Baseline (record before implementing)

Run CruxBC benchmarks on all small + big programs. Record Andersen phase times and total times. This is the comparison target.

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled -o /workspace/tests/benchmarks/cruxbc/baseline-124.json'
```

Also run PTABen to record correctness baseline (expect 2252 Exact, 69 Unsound):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/baseline-124.json'
```

---

### Task 1: Expose `create_template` from solver module

**Files:**
- Modify: `crates/saf-analysis/src/pta/solver.rs:328` (visibility change)
- Modify: `crates/saf-analysis/src/pta/mod.rs:144` (re-export)

**Step 1: Change `create_template` visibility**

In `crates/saf-analysis/src/pta/solver.rs`, line 328, change:
```rust
fn create_template<P: PtsSet>(constraints: &ConstraintSet, mode: ClusteringMode) -> P {
```
to:
```rust
pub(crate) fn create_template<P: PtsSet>(constraints: &ConstraintSet, mode: ClusteringMode) -> P {
```

**Step 2: Re-export from pta module**

In `crates/saf-analysis/src/pta/mod.rs`, at line 144 (near the other `pub(crate) use solver::` lines), add:
```rust
pub(crate) use solver::create_template;
```

**Step 3: Verify compilation**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis'`
Expected: compiles with no errors. No behavior change — just visibility.

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/pta/solver.rs crates/saf-analysis/src/pta/mod.rs
git commit -m "refactor(pta): expose create_template as pub(crate)"
```

---

### Task 2: Wire BitVecPtsSet + clustering into CG refinement

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs:30,153-155`

**Step 1: Update imports**

In `crates/saf-analysis/src/cg_refinement.rs`, line 30, change:
```rust
use crate::pta::ptsset::{BTreePtsSet, PtsSet};
```
to:
```rust
use crate::pta::ptsset::{BitVecPtsSet, ClusteringMode, PtsSet};
use crate::pta::create_template;
```

Note: `BTreePtsSet` is still imported for tests (check `#[cfg(test)]` usage). If only used in tests, move the import to the test module. If used nowhere else, remove it entirely.

**Step 2: Replace solver instantiation**

In `crates/saf-analysis/src/cg_refinement.rs`, lines 153-155, change:
```rust
    // 4c. Create solver and run initial fixed point
    let mut solver =
        GenericSolver::<BTreePtsSet>::new(&reduced, &factory).with_constants(&module.constants);
```
to:
```rust
    // 4c. Create solver with clustered BitVec representation
    let template = create_template::<BitVecPtsSet>(&reduced, ClusteringMode::Auto);
    let mut solver = GenericSolver::<BitVecPtsSet>::new_with_template(&reduced, &factory, template)
        .with_constants(&module.constants);
```

**Why `&reduced` (not `&constraints`)?** The clustering should run on the HVN-reduced constraint set, since that's what the solver will actually process. Clustering on the original constraints would include locations that HVN merges away, wasting cluster slots.

**Step 3: Verify compilation**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-analysis'`
Expected: compiles. The rest of `refine()` already works generically:
- `resolve_and_connect(&mut solver, ...)` is generic over `P: PtsSet` (line 374)
- `solver.pts.into_iter().map(|(v, p)| (v, p.to_btreeset()))` works for all `PtsSet` types
- `solver.drain_worklist(...)` and `solver.add_copy_constraint(...)` are on `GenericSolver<P>`

**Step 4: Commit**

```bash
git add crates/saf-analysis/src/cg_refinement.rs
git commit -m "perf(pta): use BitVecPtsSet + clustering in CG refinement (Gap 3+1)"
```

---

### Task 3: Run tests and validate correctness

**Step 1: Run full test suite**

Run: `docker compose run --rm dev sh -c 'cargo nextest run --release 2>&1 | tail -20'`
Expected: all tests pass (baseline: ~1541 Rust tests)

Run: `docker compose run --rm dev sh -c 'cd /workspace && uv run pytest python/tests -q'`
Expected: all Python tests pass (baseline: ~77 tests)

**Step 2: Run PTABen correctness benchmark**

Run (background, takes 30-120s):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-124.json'
```

Expected: 2252 Exact, 69 Unsound (no regression from baseline).

Compare with baseline:
```bash
python3 -c "
import json
for label, path in [('baseline', 'tests/benchmarks/ptaben/baseline-124.json'), ('after', 'tests/benchmarks/ptaben/results-124.json')]:
    with open(path) as f:
        data = json.load(f)
    print(f'{label}: Exact={data[\"summary\"][\"exact\"]}, Unsound={data[\"summary\"][\"unsound\"]}')
"
```

**Gate:** If Unsound increases or Exact decreases, STOP and investigate. Do not proceed to performance benchmarking.

**Step 3: Commit (no code change, just results files for tracking)**

No commit needed — results files are gitignored.

---

### Task 4: Run performance benchmarks and validate

**Step 1: Run CruxBC performance benchmark**

Run (background, takes several minutes):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- cruxbc --compiled-dir tests/benchmarks/cruxbc/.compiled -o /workspace/tests/benchmarks/cruxbc/results-124.json'
```

**Step 2: Compare with baseline**

```bash
python3 -c "
import json
for label, path in [('baseline', 'tests/benchmarks/cruxbc/baseline-124.json'), ('after', 'tests/benchmarks/cruxbc/results-124.json')]:
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
- `big/bash`: Andersen phase from ~44s down to ~5-15s (target: 3-10x improvement)
- Small programs (dc, bc, htop): no more than 5x slower than baseline

**Gate:** If any program is >5x slower than its baseline, REVERT Task 2's change and investigate.

**Step 3: Commit results summary to PROGRESS.md**

---

### Task 5: Lint, format, update progress

**Step 1: Format and lint**

Run: `docker compose run --rm dev sh -c 'cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -20'`
Expected: clean.

**Step 2: Update PROGRESS.md**

Add Plan 124 entry to the Plans Index:
```
| 124 | cg-refinement-bitvec-clustering | performance | done |
```

Add Session Log entry with benchmark results (Andersen phase before/after for bash and small programs).

Update "Next Steps" if this unlocks further optimizations.

**Step 3: Final commit**

```bash
git add plans/124-cg-refinement-bitvec-clustering.md plans/PROGRESS.md
git commit -m "docs: add Plan 124 (CG refinement BitVec + clustering) and results"
```

---

## Rollback Plan

If benchmarks show >5x regression on any program:

1. Revert Task 2: change `BitVecPtsSet` back to `BTreePtsSet`, remove `create_template` call
2. Keep Task 1 (visibility change is harmless)
3. Document findings: what was the clustering quality? Were bitvectors still sparse despite clustering?
4. Investigate: add `tracing::info!` in `create_template` to log cluster count, avg cluster size, and total locations covered vs missed

## Future Work (if this succeeds)

- **Gap 4 (union-find path compression):** Quick follow-up, independent of this change
- **Gap 2 (HCD):** Replace periodic Tarjan with offline cycle annotation
- **Hybrid strategy:** If small programs regress, implement cardinality-based switching (BTreeSet for sets <K elements, BitVec for larger)
