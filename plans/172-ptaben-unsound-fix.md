# PTABen Unsound Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce PTABen unsound cases from 640 to ~100 by wiring FS-PTA, fixing buffer overflow validation, tuning CS-PTA, wiring assert_eq, and improving absint precision.

**Architecture:** Three tiers: (1) trivial config/validation fixes, (2) wire existing FS-PTA and assert_eq APIs into bench mode, (3) absint precision improvements (loop thresholds, singleton narrowing, branch refinement). All FS-PTA APIs are already public — build the pipeline directly in `run_bench_mode()` without changing ProgramDatabase.

**Tech Stack:** Rust, saf-analysis (absint, fspta, mssa, svfg), saf-cli (driver, bench_types), saf-bench (ptaben)

**Design:** `docs/plans/2026-02-26-ptaben-unsound-fix-design.md`

**IMPORTANT:** All `make` commands (test, lint, fmt, shell) must be run by the main agent only, never by subagents. Subagents do code reading, analysis, and editing only.

---

## Phase 1: Trivial Fixes (Tier 1)

### Task 1: Fix CS-PTA max_iterations

**Files:**
- Modify: `crates/saf-cli/src/driver.rs:778-784`

**Step 1: Fix the hardcoded CS-PTA config**

In `run_bench_mode()`, the CS-PTA config uses `max_iterations: 100` which is 20,000x too low. Change it to use the bench config's value.

Find this block (~line 778):
```rust
let cspta_config = CsPtaConfig {
    k: 2,
    field_sensitivity: FieldSensitivity::StructFields { max_depth: 2 },
    max_iterations: 100,
    max_objects: 100_000,
    pts_config: PtsConfig::default(),
};
```

Change to:
```rust
let cspta_config = CsPtaConfig {
    k: 2,
    field_sensitivity: FieldSensitivity::StructFields { max_depth: 2 },
    max_iterations: bench_config.pta_config.max_iterations,
    max_objects: 200_000,
    pts_config: PtsConfig::default(),
};
```

**Step 2: Commit**

```bash
git add crates/saf-cli/src/driver.rs
git commit -m "fix(bench): use bench config max_iterations for CS-PTA instead of hardcoded 100"
```

### Task 2: Fix buffer overflow validation filtering

**Files:**
- Modify: `crates/saf-bench/src/ptaben.rs:813-815` (call site)
- Modify: `crates/saf-bench/src/ptaben.rs:1120-1149` (validation function)

**Step 1: Update the validation call to pass the pointer**

Find the call site (~line 813):
```rust
Expectation::BufferAccess { kind, .. } => {
    validate_buffer_from_findings(*kind, &result.buffer_findings)
}
```

Change to:
```rust
Expectation::BufferAccess { kind, ptr, .. } => {
    let ptr_hex = ptr.to_hex();
    validate_buffer_from_findings(*kind, &ptr_hex, &result.buffer_findings)
}
```

**Step 2: Update the validation function to filter by pointer**

Find the function (~line 1120):
```rust
fn validate_buffer_from_findings(
    kind: BufferAccessKind,
    findings: &[bench_types::BenchBufferFinding],
) -> Outcome {
    let has_finding = !findings.is_empty();
```

Change to:
```rust
fn validate_buffer_from_findings(
    kind: BufferAccessKind,
    expected_ptr: &str,
    findings: &[bench_types::BenchBufferFinding],
) -> Outcome {
    // Filter findings to only those matching the expected pointer.
    // Without this filter, a single overflow in a file causes ALL
    // SAFE_BUFACCESS expectations to fail as false positives.
    let has_finding = findings.iter().any(|f| f.ptr == expected_ptr);
```

The rest of the function stays the same (it only uses `has_finding`). Update the format string that uses `findings.len()` to use the filtered count:

```rust
BufferAccessKind::Safe => {
    if has_finding {
        let count = findings.iter().filter(|f| f.ptr == expected_ptr).count();
        Outcome::Unsound {
            expected: "No buffer overflow (safe access)".to_string(),
            actual: format!("{count} buffer overflow finding(s) for this pointer"),
        }
    } else {
        Outcome::Exact
    }
}
```

**Step 3: Commit**

```bash
git add crates/saf-bench/src/ptaben.rs
git commit -m "fix(bench): filter buffer overflow findings by pointer in validation"
```

### Task 3: Build, format, lint, test, and verify PTABen (Phase 1 checkpoint)

**Step 1:** Run `make fmt && make lint && make test` to verify Phase 1 changes compile and pass.

Expected: 2071+ tests pass, lint clean.

**Step 2:** Run PTABen benchmark to measure Phase 1 improvement (run in background, takes 30-120s):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-phase1.json'
```

Parse results and compare against Plan 171 baseline (640 Unsound). Expected improvement: ~200+ fewer unsound from buffer overflow filtering + CS-PTA convergence.

---

## Phase 2: Wire FS-PTA (Tier 2a)

### Task 4: Build FS-PTA pipeline in run_bench_mode()

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (run_bench_mode, ~line 774-820)

**Context:** All FS-PTA APIs are already public. We don't need to change ProgramDatabase visibility. The pipeline is: Build CFGs → Clone PTA → Build MSSA → Build SVFG → Build FsSvfg → Run solver.

**Step 1: Add imports at the top of driver.rs**

Add these imports (merge with existing use blocks):
```rust
use saf_analysis::cfg::Cfg;
use saf_analysis::fspta::{FsPtaConfig, FsSvfg, solve_flow_sensitive};
use saf_analysis::fspta::builder::FsSvfgBuilder;
use saf_analysis::mssa::MemorySsa;
use saf_analysis::svfg::SvfgBuilder;
```

Check which of these are already imported. Only add what's missing.

**Step 2: Add FS-PTA pipeline after CS-PTA block**

After the CS-PTA result is computed (~line 792), add the FS-PTA pipeline. Insert BEFORE the alias query loop (before `for query in &bench_config.alias_queries`):

```rust
// Optionally run FS-PTA for flow-sensitive alias precision
let fspta_result = if bench_config.analyses.fspta {
    if let Some(pta) = self.db.pta_result() {
        // Build CFGs for all defined functions
        let cfgs: std::collections::BTreeMap<saf_core::ids::FunctionId, Cfg> = self
            .module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();

        // MSSA needs an owned PtaResult
        let mssa_pta = pta.clone();
        let mut mssa = MemorySsa::build(
            &self.module,
            &cfgs,
            mssa_pta,
            self.db.call_graph(),
        );

        // Build SVFG
        let (svfg, _program_points) = SvfgBuilder::new(
            &self.module,
            self.db.defuse(),
            self.db.call_graph(),
            pta,
            &mut mssa,
        )
        .build();

        // Build FsSvfg (object-labeled SVFG for flow-sensitive solving)
        let fs_svfg = FsSvfgBuilder::new(
            &self.module,
            &svfg,
            pta,
            &mut mssa,
            self.db.call_graph(),
        )
        .build();

        // Run FS-PTA solver
        let fs_config = FsPtaConfig {
            skip_df_materialization: true,
            ..FsPtaConfig::default()
        };
        Some(solve_flow_sensitive(
            &self.module,
            &fs_svfg,
            pta,
            self.db.call_graph(),
            &fs_config,
        ))
    } else {
        None
    }
} else {
    None
};
```

**Step 3: Wire FS-PTA alias results into the query loop**

In the alias query loop, replace the hardcoded `fs: None` and update `pick_best_alias`. Find (~line 807-818):

```rust
// Pick best (most precise non-Unknown) result
let best = pick_best_alias(&ci_str, cs_str.as_deref(), None);

result.alias_results.push(AliasResultEntry {
    ptr_a: query.ptr_a.clone(),
    ptr_b: query.ptr_b.clone(),
    ci: ci_str,
    cs: cs_str,
    fs: None, // FS-PTA blocked — requires SVFG not exposed by ProgramDatabase
    ps: None,
    best,
});
```

Replace with:
```rust
// FS-PTA alias query (if run) — uses global top-level pts
let fs_str = fspta_result.as_ref().map(|fs| {
    let p_pts = fs.points_to(ptr_a);
    let q_pts = fs.points_to(ptr_b);
    if p_pts.is_empty() || q_pts.is_empty() {
        "Unknown".to_string()
    } else if p_pts.is_disjoint(q_pts) {
        "NoAlias".to_string()
    } else if p_pts == q_pts && p_pts.len() == 1 {
        "MustAlias".to_string()
    } else if p_pts == q_pts {
        "MayAlias".to_string()
    } else if p_pts.is_subset(q_pts) || q_pts.is_subset(p_pts) {
        "PartialAlias".to_string()
    } else {
        "MayAlias".to_string()
    }
});

// Pick best (most precise non-Unknown) result
let best = pick_best_alias(&ci_str, cs_str.as_deref(), fs_str.as_deref());

result.alias_results.push(AliasResultEntry {
    ptr_a: query.ptr_a.clone(),
    ptr_b: query.ptr_b.clone(),
    ci: ci_str,
    cs: cs_str,
    fs: fs_str,
    ps: None,
    best,
});
```

**Step 4: Commit**

```bash
git add crates/saf-cli/src/driver.rs
git commit -m "feat(bench): wire FS-PTA pipeline into run_bench_mode for flow-sensitive alias"
```

### Task 5: Build, format, lint, test, and verify PTABen (Phase 2 checkpoint)

**Step 1:** Run `make fmt && make lint && make test` to verify FS-PTA wiring compiles and all tests pass.

Expected: 2071+ tests pass, lint clean. Any import issues should be fixed now.

**Step 2:** Run PTABen benchmark to measure FS-PTA improvement (run in background, takes 30-120s):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-phase2.json'
```

Parse results and compare against Phase 1 results. Expected improvement: ~300+ fewer unsound from FS-PTA wiring (basic_cpp_tests, failed_tests, fs_tests categories should improve dramatically).

---

## Phase 3: Wire assert_eq (Tier 2b)

### Task 6: Add IntervalQuery to BenchConfig

**Files:**
- Modify: `crates/saf-cli/src/bench_types.rs`

**Step 1: Add the IntervalQuery struct**

After the `NullnessQuery` struct (~line 54), add:

```rust
/// An interval query for `svf_assert_eq(a, b)` — checks if intervals overlap.
#[derive(Debug, Serialize, Deserialize)]
pub struct IntervalQuery {
    /// Call site (`InstId` hex) of the `svf_assert_eq` oracle.
    pub call_site: String,
    /// Left operand `ValueId` (hex).
    pub left_value: String,
    /// Right operand `ValueId` (hex).
    pub right_value: String,
}
```

**Step 2: Add interval_queries field to BenchConfig**

In the `BenchConfig` struct, after `nullness_queries`, add:

```rust
/// Interval queries for `svf_assert_eq` validation.
#[serde(default)]
pub interval_queries: Vec<IntervalQuery>,
```

**Step 3: Commit**

```bash
git add crates/saf-cli/src/bench_types.rs
git commit -m "feat(bench): add IntervalQuery type to BenchConfig for assert_eq support"
```

### Task 7: Extract assert_eq expectations in build_bench_config

**Files:**
- Modify: `crates/saf-bench/src/ptaben.rs:669-730` (build_bench_config function)
- Modify: `crates/saf-bench/src/ptaben.rs:23-25` (imports)

**Step 1: Add import**

In the imports at line 23-25, add `IntervalQuery` to the import from `saf_cli::bench_types`:

```rust
use saf_cli::bench_types::{
    self, AliasQuery, AnalysisFlags, BenchConfig, BenchPtaConfig, BenchResult, IntervalQuery,
    NullnessQuery,
};
```

**Step 2: Add interval query extraction**

In `build_bench_config()`, the function already iterates expectations and extracts alias/nullness queries. Add a `let mut interval_queries = Vec::new();` alongside the existing query vectors, then add a match arm for `AssertEq`:

After the existing `Expectation::NullCheck` arm (~line 699-704), add:

```rust
Expectation::AssertEq {
    left,
    right,
    call_site,
} => {
    interval_queries.push(IntervalQuery {
        call_site: format!("0x{:032x}", call_site.raw()),
        left_value: format!("0x{:032x}", left.raw()),
        right_value: format!("0x{:032x}", right.raw()),
    });
}
```

**Step 3: Add interval_queries to the BenchConfig construction**

In the `BenchConfig { ... }` return value (~line 709), add:
```rust
interval_queries,
```

**Step 4: Commit**

```bash
git add crates/saf-bench/src/ptaben.rs
git commit -m "feat(bench): extract assert_eq expectations into interval queries"
```

### Task 8: Wire interval analysis in driver.rs

**Files:**
- Modify: `crates/saf-cli/src/driver.rs:881-883` (the TODO section)

**Step 1: Replace the TODO with interval query logic**

Find the TODO block (~line 881-883):
```rust
// ── 4. Interval queries (assert_eq) ──────────────────────────────
// TODO: Wire interval-based assert_eq validation once the bench harness
// emits IntervalQuery entries in BenchConfig.
```

Replace with:
```rust
// ── 4. Interval queries (assert_eq) ──────────────────────────────
if !bench_config.interval_queries.is_empty() {
    // Reuse the interprocedural result if z3_prove already ran it,
    // otherwise run absint now.
    let interp_for_intervals = if bench_config.analyses.z3_prove {
        None // Already ran above, will reuse
    } else if let Some(pta) = self.db.pta_result() {
        let absint_config = saf_analysis::absint::AbstractInterpConfig::default();
        let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();
        Some(saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
            &self.module,
            &absint_config,
            pta,
            Some(&specs),
        ))
    } else {
        None
    };

    // We need a reference to whichever interp result we have.
    // If z3_prove ran, we need to re-run (the previous result was consumed).
    // Simplification: always run if we have PTA.
    if let Some(pta) = self.db.pta_result() {
        let absint_config = saf_analysis::absint::AbstractInterpConfig::default();
        let specs = saf_core::spec::SpecRegistry::load().unwrap_or_default();
        let interp_result = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
            &self.module,
            &absint_config,
            pta,
            Some(&specs),
        );

        for iq in &bench_config.interval_queries {
            let call_site = bench_parse_inst_id(&iq.call_site)?;
            let left_val = bench_parse_value_id(&iq.left_value)?;
            let right_val = bench_parse_value_id(&iq.right_value)?;

            // Query intervals at the call site instruction (32-bit default)
            let left_iv = interp_result.interval_at_inst(call_site, left_val, 32);
            let right_iv = interp_result.interval_at_inst(call_site, right_val, 32);

            // Check overlap: if intervals share any value, assert_eq could pass
            let overlap = !left_iv.is_bottom()
                && !right_iv.is_bottom()
                && left_iv.lo <= right_iv.hi
                && right_iv.lo <= left_iv.hi;

            result.interval_results.push(IntervalResultEntry {
                call_site: iq.call_site.clone(),
                left: iq.left_value.clone(),
                right: iq.right_value.clone(),
                left_interval: format!("[{}, {}]", left_iv.lo, left_iv.hi),
                right_interval: format!("[{}, {}]", right_iv.lo, right_iv.hi),
                overlap,
            });
        }
    }
}
```

**Note:** Need to add `IntervalResultEntry` to the imports at the top of `run_bench_mode()`:
```rust
use saf_cli::bench_types::{
    AliasResultEntry, AssertionResultEntry, BenchBufferFinding, BenchCheckerFinding,
    BenchMtaResults, BenchResult, BenchStats, BenchThreadContextEntry, IntervalResultEntry,
    NullnessResultEntry,
};
```

Also need to add `bench_parse_inst_id` if it doesn't exist yet — check if it does. If not, add a helper near `bench_parse_value_id`:
```rust
fn bench_parse_inst_id(hex: &str) -> anyhow::Result<InstId> {
    let stripped = hex.strip_prefix("0x").unwrap_or(hex);
    let raw = u128::from_str_radix(stripped, 16)
        .map_err(|e| anyhow::anyhow!("Invalid InstId hex '{hex}': {e}"))?;
    Ok(InstId::from_raw(raw))
}
```

**Step 2: Commit**

```bash
git add crates/saf-cli/src/driver.rs
git commit -m "feat(bench): wire interval queries for assert_eq in run_bench_mode"
```

### Task 9: Validate assert_eq expectations in ptaben.rs

**Files:**
- Modify: `crates/saf-bench/src/ptaben.rs:787-792` (AssertEq validation)

**Step 1: Replace the skip with actual validation**

Find (~line 787):
```rust
Expectation::AssertEq { .. } => {
    // AssertEq requires interval queries which are not yet wired in CLI bench mode
    Outcome::Skip {
        reason: "AssertEq validation via CLI not yet implemented".to_string(),
    }
}
```

Replace with:
```rust
Expectation::AssertEq {
    left,
    right,
    call_site,
} => {
    let cs_hex = format!("0x{:032x}", call_site.raw());
    let entry = result
        .interval_results
        .iter()
        .find(|r| r.call_site == cs_hex);
    match entry {
        Some(e) => {
            if e.overlap {
                Outcome::Exact
            } else {
                Outcome::Unsound {
                    expected: "Intervals should overlap (assert_eq)".to_string(),
                    actual: format!(
                        "No overlap: left={}, right={}",
                        e.left_interval, e.right_interval
                    ),
                }
            }
        }
        None => Outcome::Skip {
            reason: "Interval result not found in CLI output".to_string(),
        },
    }
}
```

**Step 2: Commit**

```bash
git add crates/saf-bench/src/ptaben.rs
git commit -m "feat(bench): validate assert_eq expectations against interval results"
```

### Task 10: Build, format, lint, test, and verify PTABen (Phase 3 checkpoint)

**Step 1:** Run `make fmt && make lint && make test` to verify assert_eq wiring compiles and all tests pass.

Expected: 2071+ tests pass, lint clean.

**Step 2:** Run PTABen benchmark to measure assert_eq improvement (run in background, takes 30-120s):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-phase3.json'
```

Parse results and compare against Phase 2 results. Expected improvement: ~15-20 fewer skips (assert_eq tests that were skipped are now testable). Some may show as Exact, some as Unsound (depends on absint precision).

---

## Phase 4: Absint Precision Improvements (Tier 3)

### Task 11: Increase narrowing iterations in bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (z3_prove and interval query sections)

**Step 1: Use more narrowing iterations for bench assertions**

In the z3_prove section (~line 843), replace:
```rust
let absint_config = saf_analysis::absint::AbstractInterpConfig::default();
```

With:
```rust
let absint_config = saf_analysis::absint::AbstractInterpConfig {
    narrowing_iterations: 5,
    ..saf_analysis::absint::AbstractInterpConfig::default()
};
```

Do the same for the interval query section if it has its own `AbstractInterpConfig::default()`.

Also do the same for the nullness section (~line 889) and buffer overflow section (~line 929) — all bench mode absint should use the higher narrowing count.

**Step 2: Commit**

```bash
git add crates/saf-cli/src/driver.rs
git commit -m "fix(bench): increase narrowing iterations to 5 in bench mode absint"
```

### Task 12: Extend loop body threshold scanning

**Files:**
- Modify: `crates/saf-analysis/src/absint/fixpoint.rs:1380-1446`

**Step 1: Extend block scanning to include loop body**

The current `extract_loop_bound_constants()` only scans the header block and its predecessors. This misses constants in loop body blocks (e.g., the increment block `for.inc` containing `i + 1`).

Find the block scanning setup (~line 1401-1407):
```rust
// Collect blocks to scan: the header itself and its predecessors
let mut blocks_to_scan = vec![header_id];
if let Some(preds) = cfg.predecessors_of(header_id) {
    for pred_id in preds {
        blocks_to_scan.push(*pred_id);
    }
}
```

Replace with:
```rust
// Collect blocks to scan: the header, predecessors, and successor
// blocks (the loop body). Loop bodies contain increment constants
// (e.g., `i + 1`) that are useful widening thresholds.
let mut blocks_to_scan = vec![header_id];
if let Some(preds) = cfg.predecessors_of(header_id) {
    for pred_id in preds {
        blocks_to_scan.push(*pred_id);
    }
}
// Also scan successors of the header (loop body entry blocks)
// and their successors (one level deep covers most loop bodies).
let header_succs: Vec<BlockId> = cfg
    .successors_of(header_id)
    .map(|s| s.to_vec())
    .unwrap_or_default();
for succ_id in &header_succs {
    if !blocks_to_scan.contains(succ_id) {
        blocks_to_scan.push(*succ_id);
    }
    // One more level: scan successors of loop body blocks
    if let Some(inner_succs) = cfg.successors_of(*succ_id) {
        for inner_succ in inner_succs {
            if !blocks_to_scan.contains(inner_succ) {
                blocks_to_scan.push(*inner_succ);
            }
        }
    }
}
```

Also extend to scan Add/Sub operations in addition to ICmp, since loop increments use BinaryOp::Add:

Find the operation match (~line 1414):
```rust
if let Operation::BinaryOp { kind } = &inst.op {
    if matches!(
        kind,
        BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
            ...
    ) {
```

Extend the match to include arithmetic operations:
```rust
if let Operation::BinaryOp { kind } = &inst.op {
    if matches!(
        kind,
        BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSge
            | BinaryOp::ICmpEq
            | BinaryOp::ICmpNe
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpUle
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUge
            | BinaryOp::Add
            | BinaryOp::Sub
    ) {
```

**Step 2: Commit**

```bash
git add crates/saf-analysis/src/absint/fixpoint.rs
git commit -m "feat(absint): extend loop threshold scanning to loop body blocks and arithmetic ops"
```

### Task 13: Singleton narrowing improvement

**Files:**
- Modify: `crates/saf-analysis/src/absint/interval.rs:990-1023`

**Step 1: Add singleton specialization to narrow()**

Find the narrow method (~line 990):
```rust
fn narrow(&self, other: &Self) -> Self {
    if self.bottom {
        return Self::make_bottom(self.bits);
    }
    if other.bottom {
        return Self::make_bottom(self.bits);
    }

    let min_bound = signed_min(self.bits);
    let max_bound = signed_max(self.bits);
```

After the bottom checks and before the min_bound/max_bound lines, add:
```rust
    // Singleton specialization: if `other` is a singleton contained
    // within `self`, narrow directly to the singleton. This helps
    // post-loop assertions like `assert(i == 5)` where the loop
    // exit produces the exact bound value.
    if other.is_singleton() && other.lo >= self.lo && other.hi <= self.hi {
        return other.clone();
    }
```

**Step 2: Commit**

```bash
git add crates/saf-analysis/src/absint/interval.rs
git commit -m "feat(absint): add singleton specialization to narrowing for post-loop precision"
```

### Task 14: Build, format, lint, test, and verify PTABen (Phase 4 checkpoint)

**Step 1:** Run `make fmt && make lint && make test` to verify all Tier 3 changes compile and pass.

Expected: 2071+ tests pass, lint clean.

**Step 2:** Run PTABen benchmark to measure absint precision improvement (run in background, takes 30-120s):

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-phase4.json'
```

Parse results and compare against Phase 3 results. Expected improvement: ~10-20 fewer unsound from better narrowing (ae_assert_tests, ae_recursion_tests categories).

---

## Phase 5: Validation & Benchmarking

### Task 15: Run PTABen benchmark

Run the full PTABen benchmark inside Docker to measure improvement.

```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results-plan172.json'
```

Run this in the background (takes 30-120s). Read the JSON results file after completion.

Parse and report the results:
```bash
python3 -c "
import json
with open('tests/benchmarks/ptaben/results-plan172.json') as f:
    data = json.load(f)
print(f'Total: {data[\"summary\"][\"total\"]}')
print(f'Exact: {data[\"summary\"][\"exact\"]}')
print(f'Sound: {data[\"summary\"][\"sound\"]}')
print(f'ToVerify: {data[\"summary\"][\"to_verify\"]}')
print(f'Unsound: {data[\"summary\"][\"unsound\"]}')
print(f'Skip: {data[\"summary\"][\"skip\"]}')
print()
for cat in data['by_category']:
    if cat['unsound'] > 0:
        print(f'{cat[\"category\"]}: Exact={cat[\"exact\"]}, Sound={cat[\"sound\"]}, Unsound={cat[\"unsound\"]}, Skip={cat[\"skip\"]}')
"
```

Compare against Plan 171 baseline: 971 Exact, 1121 Sound, 61 ToVerify, 640 Unsound.

Target: Unsound < 150 (eliminate ~500+).

### Task 16: Fix any issues found in benchmark results

If specific categories still show unexpected unsound results, investigate and fix. Common issues:
- Import errors in driver.rs (fix compilation)
- Interval bit-width mismatch (use instruction's bit-width, not hardcoded 32)
- FS-PTA solver timeout on specific test files

### Task 17: Update PROGRESS.md and commit

**Files:**
- Modify: `plans/PROGRESS.md`

Add Plan 172 to the Plans Index, update Next Steps, and append to Session Log with:
- Before/after unsound numbers
- Per-category breakdown of improvements
- Any remaining unsound with root cause classification

Final commit:
```bash
git add plans/PROGRESS.md plans/172-ptaben-unsound-fix.md docs/plans/2026-02-26-ptaben-unsound-fix-design.md
git commit -m "docs: add Plan 172 PTABen unsound fix to progress tracking"
```

---

## Summary

| Phase | Tasks | What |
|-------|-------|------|
| 1 | 1-3 | CS-PTA tuning + buffer overflow filter |
| 2 | 4-5 | Wire FS-PTA pipeline |
| 3 | 6-10 | Wire assert_eq intervals |
| 4 | 11-14 | Absint precision (narrowing, loop thresholds, singleton) |
| 5 | 15-17 | Benchmark validation + docs |

**Expected outcome:** Unsound from 640 → ~100-150
