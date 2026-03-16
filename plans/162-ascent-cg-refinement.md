# Ascent CG Refinement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate the duplicate legacy PTA solve when using the Ascent solver by integrating Ascent into the CG refinement loop, achieving ~40% wall-clock improvement on bash CruxBC.

**Architecture:** Split `refine()` into `refine_prepare()` (solver-agnostic setup) + solver-specific PTA loop + `refine_finalize()` (solver-agnostic cleanup). Add `PtaSolver` to `RefinementConfig` for runtime dispatch. New `refine_with_ascent()` in `saf-datalog` runs iterative Ascent solves with incremental copy constraint addition (skipping HVN/SCC on re-solves).

**Tech Stack:** Rust, Ascent Datalog, saf-analysis, saf-datalog, saf-core, saf-bench

**Design:** `docs/plans/2026-02-23-ascent-cg-refinement-design.md`

---

### Task 1: Move `PtaSolver` to `saf-core`

`PtaSolver` is currently defined in `saf-bench::ptaben` but it's a domain concept needed by `saf-analysis` and `saf-datalog`. Move it to `saf-core::config`.

**Files:**
- Modify: `crates/saf-core/src/config.rs` — add `PtaSolver` enum
- Modify: `crates/saf-bench/src/ptaben.rs:79-87` — remove `PtaSolver` enum, re-import from `saf_core`
- Modify: `crates/saf-bench/src/cruxbc.rs` — update import
- Modify: `crates/saf-bench/src/main.rs` — update import

**Step 1:** Add to `crates/saf-core/src/config.rs`:

```rust
/// Which PTA solver backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PtaSolver {
    /// Worklist-based imperative solver.
    Legacy,
    /// Datalog fixpoint solver (Ascent).
    #[default]
    Ascent,
}
```

**Step 2:** In `crates/saf-bench/src/ptaben.rs`, delete the `PtaSolver` enum (lines 79-87) and add `use saf_core::config::PtaSolver;`.

**Step 3:** Update imports in `cruxbc.rs` and `main.rs` to use `saf_core::config::PtaSolver`.

**Step 4:** Run `make fmt && make lint` to verify.

**Step 5:** Commit: `refactor: move PtaSolver enum to saf-core`

---

### Task 2: Add `PtaSolver` to `RefinementConfig`

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs:53-72` — add `pta_solver` field to `RefinementConfig` and `Default`

**Step 1:** Add field to `RefinementConfig`:

```rust
pub struct RefinementConfig {
    pub max_iterations: usize,
    pub entry_points: EntryPointStrategy,
    pub pta_config: PtaConfig,
    pub field_sensitivity: FieldSensitivity,
    /// Which PTA solver backend to use for the refinement loop.
    pub pta_solver: PtaSolver,
}
```

Add `use saf_core::config::PtaSolver;` to imports.

**Step 2:** Update `Default for RefinementConfig` to include `pta_solver: PtaSolver::default()`.

**Step 3:** Run `make fmt && make lint`. Fix any callers that construct `RefinementConfig` with struct literal (they'll need `pta_solver` or `..Default::default()`). Check: `crates/saf-bench/src/cruxbc.rs:349-353`, `crates/saf-bench/src/ptaben.rs` (any explicit construction), `crates/saf-python/src/cg_refinement.rs`, `crates/saf-analysis/src/pipeline.rs`, `crates/saf-analysis/src/passes/pta_pass.rs`, e2e tests.

**Step 4:** Commit: `feat: add pta_solver field to RefinementConfig`

---

### Task 3: Extract solver-agnostic indirect call resolution

Extract the "read PTS → discover targets → collect copy constraints" logic from `resolve_and_connect()` into a new public function that works with `&PointsToMap` + `&LocationFactory` (no solver dependency).

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs`
  - Make `IndirectCallSite`, `FunctionLocationMap`, `collect_indirect_call_sites`, `collect_return_values`, `collect_interprocedural_copies` public (needed by `saf-datalog`)
  - Add new `resolve_indirect_calls_from_pts()` public function

**Step 1:** Change visibility of helper types from private to `pub`:

```rust
pub struct IndirectCallSite { ... }
// Also: FunctionLocationMap is already in crate::pta, verify it's pub
pub fn collect_indirect_call_sites(module: &AirModule) -> Vec<IndirectCallSite> { ... }
pub fn collect_return_values(module: &AirModule) -> BTreeMap<FunctionId, Vec<ValueId>> { ... }
```

Note: `collect_interprocedural_copies` stays private — it's used internally by both `resolve_and_connect` and the new function.

**Step 2:** Add new solver-agnostic resolution function:

```rust
/// Resolve indirect calls from a points-to map (solver-agnostic).
///
/// Reads `pts` to discover new indirect call targets, adds them to `cg`,
/// and returns the interprocedural copy constraints that need to be added
/// to the solver's constraint set.
///
/// Returns an empty vec if no new targets were found.
pub fn resolve_indirect_calls_from_pts(
    pts: &crate::pta::PointsToMap,
    factory: &LocationFactory,
    indirect_sites: &[IndirectCallSite],
    func_loc_map: &FunctionLocationMap,
    module: &AirModule,
    return_values: &BTreeMap<FunctionId, Vec<ValueId>>,
    resolved_calls: &mut BTreeMap<InstId, BTreeSet<FunctionId>>,
    cg: &mut CallGraph,
) -> Vec<CopyConstraint> {
    let mut new_copies: Vec<CopyConstraint> = Vec::new();
    let mut found_new = false;

    for site in indirect_sites {
        if let Some(&callee_val) = site.operands.last() {
            if let Some(pts_set) = pts.get(&callee_val) {
                for &loc_id in pts_set {
                    if let Some(loc) = factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site, fid, module, return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }

        // Also check dst value's points-to set
        if let Some(dst) = site.dst {
            if let Some(pts_set) = pts.get(&dst) {
                for &loc_id in pts_set {
                    if let Some(loc) = factory.get(loc_id) {
                        if let Some(fid) = func_loc_map.get(loc.obj) {
                            if let Some(sig) = site.expected_signature {
                                if !module
                                    .function(fid)
                                    .is_none_or(|f| signature_compatible(sig, f, module))
                                {
                                    continue;
                                }
                            }
                            if resolved_calls.entry(site.inst_id).or_default().insert(fid) {
                                new_copies.extend(collect_interprocedural_copies(
                                    site, fid, module, return_values,
                                ));
                                found_new = true;
                            }
                        }
                    }
                }
            }
        }
    }

    if found_new {
        // Update CG edges for all newly resolved calls
        for (inst_id, targets) in &*resolved_calls {
            let target_vec: Vec<FunctionId> = targets.iter().copied().collect();
            cg.resolve_indirect(*inst_id, &target_vec);
        }
    }

    new_copies
}
```

**Step 3:** Refactor `resolve_and_connect()` to call `resolve_indirect_calls_from_pts()` internally, converting solver PTS to `PointsToMap` for the call. This avoids duplicating the resolution logic. The legacy-specific part is just the `solver.add_copy_constraint()` + `solver.recompute_topo_order()` at the end.

Note: this conversion may be expensive (building a full `BTreeMap<ValueId, BTreeSet<LocId>>` from the solver's hash-based PTS). If so, keep the existing `resolve_and_connect` as-is and just have both paths share `collect_interprocedural_copies` / `signature_compatible` / `collect_indirect_call_sites` etc. The new function is for the Ascent path only.

**Step 4:** Add the new types/functions to `cg_refinement` module's public exports. Check `crates/saf-analysis/src/lib.rs` to see what's re-exported.

**Step 5:** `make fmt && make lint && make test`

**Step 6:** Commit: `refactor: extract solver-agnostic indirect call resolution`

---

### Task 4: Split `refine()` into prepare/finalize + legacy loop

Refactor `refine()` into three functions. Keep `refine()` as a convenience wrapper.

**Files:**
- Modify: `crates/saf-analysis/src/cg_refinement.rs`

**Step 1:** Define `RefinementPrepared`:

```rust
/// Intermediate state after solver-agnostic preparation.
///
/// Passed to a solver-specific PTA loop, then to [`refine_finalize`].
pub struct RefinementPrepared {
    pub cg: CallGraph,
    pub cha: Option<ClassHierarchy>,
    pub cha_resolved_sites: BTreeSet<InstId>,
    pub resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    pub constraints: ConstraintSet,
    pub reduced: ConstraintSet,
    pub hvn_result: HvnResult,
    pub factory: LocationFactory,
    pub func_loc_map: FunctionLocationMap,
    pub indirect_sites: Vec<IndirectCallSite>,
    pub return_values: BTreeMap<FunctionId, Vec<ValueId>>,
    pub constraint_counts: [usize; 5],
    pub post_hvn_constraint_counts: [usize; 5],
}
```

**Step 2:** Define `PtaSolveResult`:

```rust
/// Output from the solver-specific PTA loop.
pub struct PtaSolveResult {
    pub pts: crate::pta::PointsToMap,
    pub factory: LocationFactory,
    pub resolved_calls: BTreeMap<InstId, BTreeSet<FunctionId>>,
    pub iterations: usize,
    pub pta_solve_secs: f64,
    pub iteration_limit_hit: bool,
}
```

**Step 3:** Extract `refine_prepare()` — steps 1-4a (CHA, initial CG, bootstrap, constraint extraction, HVN). This is lines 116-199 of current `refine()`.

```rust
/// Solver-agnostic preparation: CHA, initial CG, constraint extraction, HVN.
pub fn refine_prepare(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementPrepared { ... }
```

**Step 4:** Extract `refine_finalize()` — steps 4e-5 (HVN expansion, CHA narrowing, PtaResult, ICFG).

```rust
/// Solver-agnostic finalization: HVN expansion, CHA narrowing, ICFG.
pub fn refine_finalize(
    module: &AirModule,
    prepared: RefinementPrepared,
    solve_result: PtaSolveResult,
) -> RefinementResult { ... }
```

**Step 5:** Extract `refine_legacy()` — the legacy worklist PTA loop (steps 4b-4d). This wraps the existing `GenericSolver` + `resolve_and_connect` loop.

```rust
/// Run the legacy worklist PTA solver for CG refinement.
fn refine_legacy(
    module: &AirModule,
    prepared: &mut RefinementPrepared,
    max_iterations: usize,
) -> PtaSolveResult { ... }
```

**Step 6:** Rewrite `refine()` as a wrapper:

```rust
pub fn refine(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementResult {
    let mut prepared = refine_prepare(module, config, specs);

    let solve_result = match config.pta_solver {
        PtaSolver::Legacy => refine_legacy(module, &mut prepared, config.max_iterations),
        PtaSolver::Ascent => {
            // Ascent path will be added in Task 5.
            // For now, fall back to legacy.
            refine_legacy(module, &mut prepared, config.max_iterations)
        }
    };

    refine_finalize(module, prepared, solve_result)
}
```

**Step 7:** `make fmt && make lint && make test` — all tests should pass identically since behavior is unchanged.

**Step 8:** Commit: `refactor: split refine() into prepare/finalize + legacy loop`

---

### Task 5: Implement `refine_with_ascent()` in `saf-datalog`

The core new code: Ascent-based CG refinement with incremental re-solves (skipping HVN/SCC).

**Files:**
- Modify: `crates/saf-datalog/src/pta/context.rs` — add `refine_with_ascent()`
- Modify: `crates/saf-datalog/src/pta/solver.rs` — add `ascent_solve_with_extra_copies()`
- Modify: `crates/saf-analysis/src/cg_refinement.rs` — wire Ascent path in `refine()`

**Step 1:** Add incremental solve function to `crates/saf-datalog/src/pta/solver.rs`:

```rust
/// Solve with additional copy constraints appended to existing facts.
///
/// Skips HVN/SCC preprocessing — used for CG refinement iterations where
/// only interprocedural copy constraints are added. The additional copies
/// don't create new HVN equivalences (unique dst/src pairs) or copy-graph
/// SCCs (connect previously-disconnected subgraphs).
pub fn ascent_solve_with_extra_copies(
    facts: &PtaFacts,
    extra_copies: &[(ValueId, ValueId)],
) -> PointsToMap {
    let mut augmented = facts.clone();
    augmented.copy.extend_from_slice(extra_copies);

    let registry = Arc::new(LocIdRegistry::from_facts(&augmented));
    with_registry(registry, || ascent_solve(&augmented))
}
```

**Step 2:** Add `refine_with_ascent()` to `crates/saf-datalog/src/pta/context.rs`:

```rust
use saf_analysis::cg_refinement::{
    IndirectCallSite, PtaSolveResult, RefinementPrepared,
    resolve_indirect_calls_from_pts,
};

/// Run Ascent-based PTA with CG refinement.
///
/// Replaces the legacy worklist solver inside the CG refinement loop.
/// First iteration runs the full Ascent pipeline (SCC + iterative GEP).
/// Subsequent iterations only add interprocedural copy facts and re-solve
/// (skipping HVN/SCC/GEP — only copy constraints change).
pub fn refine_with_ascent(
    module: &AirModule,
    prepared: &mut RefinementPrepared,
    max_iterations: usize,
) -> PtaSolveResult {
    let pta_start = std::time::Instant::now();
    let index_sensitivity = /* from config, default InsensitiveAll */;

    // --- First iteration: full Ascent pipeline (SCC + GEP) ---
    let mut dl_facts = constraint_set_to_facts_ref(
        &prepared.reduced, &module.constants, index_sensitivity,
    );

    let scc_result = detect_scc(&dl_facts.copy);
    rewrite_facts_with_scc(&mut dl_facts, &scc_result.representatives);

    let original_geps = std::mem::take(&mut dl_facts.gep);

    let current_pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };

    let mut pts = iterative_gep_resolve(
        &mut dl_facts, &mut prepared.factory, original_geps, current_pts,
    );

    // Expand HVN + SCC for the resolution pass
    prepared.hvn_result.expand_results(&mut pts);
    for (original, rep) in &scc_result.representatives {
        if let Some(rep_pts) = pts.get(rep).cloned() {
            pts.insert(*original, rep_pts);
        }
    }

    let mut resolved_calls = BTreeMap::new();
    let mut iterations = 1;

    // --- CG refinement loop ---
    for _wave in 0..max_iterations {
        let new_copies = resolve_indirect_calls_from_pts(
            &pts,
            &prepared.factory,
            &prepared.indirect_sites,
            &prepared.func_loc_map,
            module,
            &prepared.return_values,
            &mut resolved_calls,
            &mut prepared.cg,
        );

        if new_copies.is_empty() {
            break;
        }

        iterations += 1;

        // Convert CopyConstraints to (dst, src) tuples for Datalog facts
        let extra: Vec<(ValueId, ValueId)> = new_copies
            .iter()
            .map(|c| (c.dst, c.src))
            .collect();

        // Incremental re-solve: append copy facts, skip HVN/SCC/GEP
        let registry = Arc::new(LocIdRegistry::from_facts_and_copies(&dl_facts, &extra));
        let raw_pts = with_registry(registry.clone(), || {
            ascent_solve_with_extra_copies(&dl_facts, &extra)
        });

        // Persist the new copies into dl_facts for next iteration
        dl_facts.copy.extend(extra);

        // Expand HVN + SCC
        pts = raw_pts;
        prepared.hvn_result.expand_results(&mut pts);
        for (original, rep) in &scc_result.representatives {
            if let Some(rep_pts) = pts.get(rep).cloned() {
                pts.insert(*original, rep_pts);
            }
        }
    }

    PtaSolveResult {
        pts,
        factory: std::mem::take(&mut prepared.factory),
        resolved_calls,
        iterations,
        pta_solve_secs: pta_start.elapsed().as_secs_f64(),
        iteration_limit_hit: false, // Ascent always converges
    }
}
```

Note: `LocIdRegistry::from_facts_and_copies()` may need to be added — it should register LocIds from both the existing facts and the extra copy values. Alternatively, since copy constraints only reference existing ValueIds (not LocIds), the existing `from_facts()` may suffice — verify.

**Step 3:** Wire the Ascent path in `crates/saf-analysis/src/cg_refinement.rs`:

```rust
PtaSolver::Ascent => {
    saf_datalog::pta::refine_with_ascent(module, &mut prepared, config.max_iterations)
}
```

This requires adding `saf-datalog` as a dependency of `saf-analysis`. **Check if this creates a circular dependency.** If so, use a different approach: have the caller (saf-bench, saf-cli, saf-python) call `refine_prepare()` + `refine_with_ascent()` + `refine_finalize()` directly, keeping `saf-analysis` free of `saf-datalog` dependency.

**Alternative (no circular dep):** `refine()` wrapper only handles `Legacy`. For `Ascent`, callers use the split API directly. Add a helper:

```rust
// In saf-datalog::pta::context
pub fn refine_ascent(
    module: &AirModule,
    config: &RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> RefinementResult {
    let mut prepared = saf_analysis::cg_refinement::refine_prepare(module, config, specs);
    let solve_result = refine_with_ascent(module, &mut prepared, config.max_iterations);
    saf_analysis::cg_refinement::refine_finalize(module, prepared, solve_result)
}
```

**Step 4:** `make fmt && make lint && make test`

**Step 5:** Commit: `feat: implement Ascent-based CG refinement loop`

---

### Task 6: Wire Ascent CG refinement into callers

Update all callers to use the new unified pipeline.

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs:346-396` — use `refine_prepare` + dispatch + `refine_finalize` for profiling
- Modify: `crates/saf-bench/src/ptaben.rs` — pass `PtaSolver` through to `RefinementConfig`
- Modify: `crates/saf-analysis/src/pipeline.rs:151` — pass `PtaSolver` from config
- Modify: `crates/saf-python/src/cg_refinement.rs` — pass `PtaSolver`

**Step 1:** Update `cruxbc.rs` to use split API for fine-grained profiling:

```rust
let refinement_config = RefinementConfig {
    entry_points: EntryPointStrategy::AllDefined,
    max_iterations: 5,
    pta_solver: solver, // <-- pass through from CLI
    ..RefinementConfig::default()
};

let refinement_result = match solver {
    PtaSolver::Ascent => {
        saf_datalog::pta::refine_ascent(module, &refinement_config, Some(&specs))
    }
    PtaSolver::Legacy => {
        refine(module, &refinement_config, Some(&specs))
    }
};
```

Remove the separate `analyze_with_ascent()` call — Ascent now runs inside the refinement.

**Step 2:** Update `ptaben.rs` — the PTABen harness currently calls `refine()` then optionally re-solves with Ascent. Change it to pass `PtaSolver` through `RefinementConfig` and remove the separate Ascent re-solve.

**Step 3:** Update `pipeline.rs` — add `PtaSolver` to `PipelineConfig` or use `RefinementConfig.pta_solver`. The pipeline should respect the solver choice.

**Step 4:** Update Python bindings — pass `pta_solver` from `Project.open()` kwargs through to `RefinementConfig`.

**Step 5:** Remove the now-unused standalone `analyze_with_ascent()` call path from `cruxbc.rs`. Keep `analyze_with_ascent()` itself in `saf-datalog` (it's still useful for standalone PTA without CG refinement).

**Step 6:** Remove profiling instrumentation added to `cruxbc.rs` (the `t_stats` / gap profiling code in the uncommitted diff).

**Step 7:** `make fmt && make lint && make test`

**Step 8:** Commit: `feat: wire Ascent CG refinement into all callers`

---

### Task 7: Benchmark validation

Run PTABen + CruxBC to validate soundness and measure performance improvement.

**Files:**
- No code changes

**Step 1:** Run PTABen with Ascent solver (background, 30-120s):

```bash
docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo run --release -p saf-bench -- ptaben \
   --compiled-dir tests/benchmarks/ptaben/.compiled \
   --solver datalog \
   -o /workspace/tests/benchmarks/ptaben/results.json'
```

Expected: ≤65 unsound (ideally closer to legacy's 61 now that Ascent has CG refinement).

**Step 2:** Run PTABen with legacy solver (verify no regression):

```bash
docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo run --release -p saf-bench -- ptaben \
   --compiled-dir tests/benchmarks/ptaben/.compiled \
   --solver worklist \
   -o /workspace/tests/benchmarks/ptaben/results-legacy.json'
```

Expected: 61 unsound (unchanged).

**Step 3:** Run CruxBC bash with Ascent solver:

```bash
docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo run --release -p saf-bench -- cruxbc \
   --compiled-dir tests/benchmarks/cruxbc/.compiled \
   --filter bash --solver datalog'
```

Expected: ~20-25s (down from 38.2s).

**Step 4:** Run CruxBC bash with legacy solver (verify no regression):

```bash
docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
  'cargo run --release -p saf-bench -- cruxbc \
   --compiled-dir tests/benchmarks/cruxbc/.compiled \
   --filter bash --solver worklist'
```

Expected: ~20s (unchanged — legacy PTA + no Ascent overhead).

**Step 5:** Run full test suite:

```bash
make fmt && make lint && make test
```

Expected: 1895+ Rust tests pass, 81 Python tests pass.

**Step 6:** If PTABen Ascent unsound count improved (e.g., 65→62), update PROGRESS.md notes. If CruxBC shows significant improvement, document the before/after.

**Step 7:** Commit: `docs: update benchmark results after Ascent CG refinement`

---

### Task 8: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

**Step 1:** Add plan 162 to Plans Index with status and notes.

**Step 2:** Update Next Steps — remove the "Plan 157 blocked on Tasks 11-14" note about performance gap (now resolved), update Ascent solver gap numbers.

**Step 3:** Add session log entry with benchmark results.

**Step 4:** Commit: `docs: update PROGRESS.md with Plan 162`
