# Benchmark CLI Wrapping Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace saf-bench's duplicated analysis pipeline with subprocess invocations of `saf run --bench-config`, eliminating ~60-70% of ptaben.rs and cruxbc.rs code.

**Architecture:** saf-bench extracts oracle expectations from IR, writes a per-test JSON config file specifying needed queries and analyses, spawns `saf run --bench-config <path> --output <path>`, then validates oracle expectations against the structured JSON results. A temp directory holds all intermediate files and is cleaned up after the run.

**Tech Stack:** Rust, serde_json, tempfile crate, std::process::Command

**Design Doc:** `docs/plans/2026-02-26-benchmark-cli-wrapping-design.md`

---

## Phase 1: CLI Bench-Config Types & Arg

### Task 1: Define BenchConfig and BenchResult serde types

**Files:**
- Create: `crates/saf-cli/src/bench_types.rs`
- Modify: `crates/saf-cli/src/lib.rs` (add `pub mod bench_types;`)

**Step 1: Create bench_types.rs with BenchConfig (input) and BenchResult (output)**

```rust
//! JSON data contract for `--bench-config` mode.
//!
//! `BenchConfig` is written by saf-bench, read by saf-cli.
//! `BenchResult` is written by saf-cli, read by saf-bench.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Input: saf-bench → saf-cli
// ---------------------------------------------------------------------------

/// Per-test configuration written by saf-bench.
#[derive(Debug, Deserialize)]
pub struct BenchConfig {
    /// Alias pointer pairs to query (CI, CS, FS, path-sensitive).
    #[serde(default)]
    pub alias_queries: Vec<AliasQuery>,

    /// Pointer ValueIds to query for nullness.
    #[serde(default)]
    pub nullness_queries: Vec<NullnessQuery>,

    /// Which analysis passes to run.
    #[serde(default)]
    pub analyses: AnalysisFlags,

    /// PTA solver/sensitivity configuration overrides.
    #[serde(default)]
    pub pta_config: BenchPtaConfig,
}

/// A single alias query: check alias(ptr_a, ptr_b).
#[derive(Debug, Deserialize)]
pub struct AliasQuery {
    /// First pointer ValueId (hex string, e.g. "0xabc...").
    pub ptr_a: String,
    /// Second pointer ValueId.
    pub ptr_b: String,
    /// Block containing the oracle call (for path-sensitive refinement).
    #[serde(default)]
    pub oracle_block: Option<String>,
    /// Function containing the oracle call.
    #[serde(default)]
    pub oracle_function: Option<String>,
}

/// A nullness query for a specific load instruction.
#[derive(Debug, Deserialize)]
pub struct NullnessQuery {
    /// Pointer ValueId to check.
    pub ptr: String,
    /// Call site (instruction) where the oracle appears.
    pub call_site: String,
}

/// Flags controlling which analyses to run.
#[derive(Debug, Default, Deserialize)]
pub struct AnalysisFlags {
    #[serde(default)]
    pub checkers: bool,
    #[serde(default)]
    pub z3_prove: bool,
    #[serde(default)]
    pub nullness: bool,
    #[serde(default)]
    pub mta: bool,
    #[serde(default)]
    pub buffer_overflow: bool,
    #[serde(default)]
    pub absint: bool,
    #[serde(default)]
    pub cspta: bool,
    #[serde(default)]
    pub fspta: bool,
}

/// PTA configuration overrides for bench mode.
#[derive(Debug, Deserialize)]
pub struct BenchPtaConfig {
    #[serde(default = "default_field_depth")]
    pub field_depth: u32,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    #[serde(default = "default_true")]
    pub constant_indices: bool,
    #[serde(default = "default_true")]
    pub z3_index_refinement: bool,
    #[serde(default = "default_solver")]
    pub solver: String,
    #[serde(default = "default_entry_strategy")]
    pub entry_point_strategy: String,
    #[serde(default = "default_refinement_iters")]
    pub refinement_max_iterations: usize,
}

fn default_field_depth() -> u32 { 10 }
fn default_max_iterations() -> usize { 2_000_000 }
fn default_true() -> bool { true }
fn default_solver() -> String { "worklist".to_string() }
fn default_entry_strategy() -> String { "all_defined".to_string() }
fn default_refinement_iters() -> usize { 5 }

impl Default for BenchPtaConfig {
    fn default() -> Self {
        Self {
            field_depth: default_field_depth(),
            max_iterations: default_max_iterations(),
            constant_indices: default_true(),
            z3_index_refinement: default_true(),
            solver: default_solver(),
            entry_point_strategy: default_entry_strategy(),
            refinement_max_iterations: default_refinement_iters(),
        }
    }
}

// ---------------------------------------------------------------------------
// Output: saf-cli → saf-bench
// ---------------------------------------------------------------------------

/// Structured result written by saf-cli for saf-bench to parse.
#[derive(Debug, Default, Serialize)]
pub struct BenchResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub stats: BenchStats,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alias_results: Vec<AliasResultEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub checker_findings: Vec<BenchCheckerFinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assertion_results: Vec<AssertionResultEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interval_results: Vec<IntervalResultEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nullness_results: Vec<NullnessResultEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buffer_findings: Vec<BenchBufferFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mta_results: Option<BenchMtaResults>,
}

#[derive(Debug, Default, Serialize)]
pub struct BenchStats {
    pub total_secs: f64,
    pub pta_solve_secs: f64,
    pub refinement_iterations: u32,
    pub defuse_build_secs: f64,
    pub valueflow_build_secs: f64,
}

#[derive(Debug, Serialize)]
pub struct AliasResultEntry {
    pub ptr_a: String,
    pub ptr_b: String,
    /// CI-PTA result: "Must", "May", "No", "Partial", "Unknown"
    pub ci: String,
    /// CS-PTA result (if run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cs: Option<String>,
    /// FS-PTA result (if run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fs: Option<String>,
    /// Path-sensitive result (if run).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ps: Option<String>,
    /// Best (most precise non-Unknown) result across all levels.
    pub best: String,
}

#[derive(Debug, Serialize)]
pub struct BenchCheckerFinding {
    pub check: String,
    /// Allocation site ValueId (hex).
    pub alloc_site: String,
    pub severity: String,
    pub call_sites: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AssertionResultEntry {
    /// Call site (InstId hex) of the assertion oracle.
    pub call_site: String,
    /// "svf_assert" or "svf_assert_eq"
    pub kind: String,
    /// Whether the assertion was proved.
    pub proved: bool,
}

#[derive(Debug, Serialize)]
pub struct IntervalResultEntry {
    pub call_site: String,
    pub left: String,
    pub right: String,
    pub left_interval: String,
    pub right_interval: String,
    pub overlap: bool,
}

#[derive(Debug, Serialize)]
pub struct NullnessResultEntry {
    pub ptr: String,
    pub call_site: String,
    /// true if may be null, false if definitely not null.
    pub may_null: bool,
}

#[derive(Debug, Serialize)]
pub struct BenchBufferFinding {
    pub ptr: String,
    pub function: String,
    pub kind: String,
    pub description: String,
}

#[derive(Debug, Default, Serialize)]
pub struct BenchMtaResults {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub interleaving: Vec<BenchInterleavingEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thread_contexts: Vec<BenchThreadContextEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tct_access: Vec<BenchTctEntry>,
}

#[derive(Debug, Serialize)]
pub struct BenchInterleavingEntry {
    pub thread_id: u64,
    pub call_site: String,
    pub interleaved: bool,
}

#[derive(Debug, Serialize)]
pub struct BenchThreadContextEntry {
    pub thread_id: u64,
    pub exists: bool,
}

#[derive(Debug, Serialize)]
pub struct BenchTctEntry {
    pub thread_id: u64,
    pub call_site: String,
    pub accessible: bool,
}
```

**Step 2: Add `pub mod bench_types;` to saf-cli's lib.rs**

Check `crates/saf-cli/src/lib.rs` — add the module declaration.

**Step 3: Verify it compiles**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-cli 2>&1' | tail -5`
Expected: Compiles (may warn about dead code, that's OK at this point)

**Step 4: Commit**

```
feat(cli): add BenchConfig/BenchResult types for benchmark mode
```

---

### Task 2: Add --bench-config CLI arg to RunArgs

**Files:**
- Modify: `crates/saf-cli/src/commands.rs` (~line 437, in RunArgs)
- Modify: `crates/saf-cli/src/driver.rs` (~line 142, in DriverConfig; ~line 196, in from_run_args)

**Step 1: Add `bench_config` field to RunArgs**

In `commands.rs`, add after the `ifds_taint` field (around line 436):

```rust
    /// Path to bench-config JSON file (benchmark mode).
    /// When set, the CLI reads analysis queries and configuration from
    /// this file and writes structured results to --output.
    #[arg(long)]
    pub bench_config: Option<PathBuf>,
```

**Step 2: Add `bench_config` field to DriverConfig**

In `driver.rs`, add after `specs_path` (around line 150):

```rust
    pub bench_config: Option<PathBuf>,
```

**Step 3: Map the field in `DriverConfig::from_run_args()`**

In `driver.rs`, add the mapping in `from_run_args()` (around line 200):

```rust
            bench_config: args.bench_config.clone(),
```

**Step 4: Verify it compiles**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-cli 2>&1' | tail -5`

**Step 5: Commit**

```
feat(cli): add --bench-config flag to saf run
```

---

### Task 3: Implement bench mode execution path in driver

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (add `run_bench_mode` method)
- Modify: `crates/saf-cli/src/commands.rs` (alter `run()` to check bench_config)

**Step 1: Add bench mode entry in `commands.rs` run()**

In `commands::run()` (around line 674), after `AnalysisDriver::build()`, check for bench_config:

```rust
pub fn run(args: &RunArgs) -> anyhow::Result<()> {
    let config = DriverConfig::from_run_args(args);

    // Bench mode: read config, run analyses, write structured result
    if let Some(ref bench_config_path) = config.bench_config {
        let bench_config: crate::bench_types::BenchConfig = {
            let data = std::fs::read_to_string(bench_config_path)
                .with_context(|| format!("Failed to read bench config: {}", bench_config_path.display()))?;
            serde_json::from_str(&data)
                .with_context(|| "Failed to parse bench config JSON")?
        };
        let driver = AnalysisDriver::build(&args.inputs, &config, args.frontend)?;
        let result = driver.run_bench_mode(&bench_config)?;
        let json = serde_json::to_string_pretty(&result)?;
        if let Some(ref path) = config.output.path {
            std::fs::write(path, &json)?;
        } else {
            print!("{json}");
        }
        return Ok(());
    }

    let driver_result = AnalysisDriver::build(&args.inputs, &config, args.frontend)?;

    if args.serve {
        return driver_result.serve();
    }

    let output = driver_result.analyze(&config)?;
    AnalysisDriver::format_output(&output, &config.output)
}
```

**Step 2: Add stub `run_bench_mode` to AnalysisDriver**

In `driver.rs`, add a new method on `AnalysisDriver`:

```rust
    /// Run in benchmark mode: execute analyses specified by `BenchConfig`
    /// and return structured `BenchResult`.
    pub fn run_bench_mode(
        &self,
        bench_config: &crate::bench_types::BenchConfig,
    ) -> anyhow::Result<crate::bench_types::BenchResult> {
        use crate::bench_types::*;

        let stats = self.db.stats();
        let mut result = BenchResult {
            success: true,
            error: None,
            stats: BenchStats {
                total_secs: stats.total_secs,
                pta_solve_secs: stats.pta_solve_secs,
                refinement_iterations: stats.refinement_iterations as u32,
                defuse_build_secs: stats.defuse_build_secs,
                valueflow_build_secs: stats.valueflow_build_secs,
            },
            ..Default::default()
        };

        // TODO: Phase 2 will wire each analysis here
        // - alias queries
        // - checker findings
        // - assertion results
        // - interval results
        // - nullness results
        // - buffer findings
        // - MTA results

        Ok(result)
    }
```

**Step 3: Verify it compiles**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-cli 2>&1' | tail -5`

**Step 4: Commit**

```
feat(cli): add bench mode execution path with stub run_bench_mode
```

---

## Phase 2: Wire Analyses into CLI Bench Mode

### Task 4: Wire alias queries into bench mode

This is the most complex piece. The CLI needs to run CI-PTA alias queries (and optionally CS-PTA, FS-PTA, path-sensitive) for each pointer pair from the bench config.

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add alias query execution**

In `run_bench_mode`, after the stats block, add:

```rust
        // ── Alias queries ──────────────────────────────────────
        if !bench_config.alias_queries.is_empty() {
            let pta_result = self.db.pta_result();

            // Optionally run CS-PTA for higher precision
            let combined_pta = if bench_config.analyses.cspta || bench_config.analyses.fspta {
                // Build CS-PTA + optionally FS-PTA
                // Use the same config as ptaben.rs: k=2, field_depth from bench_config
                let cs_config = saf_analysis::cspta::CsPtaConfig {
                    k: 2,
                    field_sensitivity: saf_analysis::FieldSensitivity::StructFields {
                        max_depth: bench_config.pta_config.field_depth as usize,
                    },
                };
                let resolved_sites = self.db.resolved_sites();
                let specs = self.db.specs();
                let cs_result = saf_analysis::cspta::solve_context_sensitive_with_resolved(
                    &self.module,
                    &cs_config,
                    pta_result,
                    self.db.callgraph(),
                    resolved_sites,
                    specs.as_ref(),
                );

                if bench_config.analyses.fspta {
                    // Build FS-PTA on top of CS-PTA
                    let defuse = self.db.defuse();
                    let cfgs = self.db.cfgs();
                    let fs_builder = saf_analysis::fspta::FsSvfgBuilder::new(
                        &self.module, defuse, cfgs, pta_result, self.db.callgraph(),
                    );
                    let fs_svfg = fs_builder.build();
                    let fs_config = saf_analysis::fspta::FsPtaConfig::default();
                    let fs_result = saf_analysis::fspta::solve_flow_sensitive(
                        &fs_svfg, pta_result, &fs_config,
                    );
                    Some(saf_analysis::cspta::CombinedPtaResult::with_load_sensitive(
                        cs_result, Some(fs_result),
                    ))
                } else {
                    Some(saf_analysis::cspta::CombinedPtaResult::new(cs_result))
                }
            } else {
                None
            };

            for query in &bench_config.alias_queries {
                let ptr_a = parse_value_id(&query.ptr_a)?;
                let ptr_b = parse_value_id(&query.ptr_b)?;

                let ci = pta_result.may_alias(ptr_a, ptr_b);
                let ci_str = format_alias_result(ci);

                let (cs_str, fs_str) = if let Some(ref combined) = combined_pta {
                    let combined_result = combined.may_alias(ptr_a, ptr_b);
                    // CS and FS are merged in CombinedPtaResult — report the combined
                    if matches!(combined_result, saf_analysis::pta::AliasResult::Unknown) {
                        (None, None)
                    } else {
                        (Some(format_alias_result(combined_result)), None)
                    }
                } else {
                    (None, None)
                };

                let best = cs_str.as_deref()
                    .unwrap_or(&ci_str)
                    .to_string();

                result.alias_results.push(AliasResultEntry {
                    ptr_a: query.ptr_a.clone(),
                    ptr_b: query.ptr_b.clone(),
                    ci: ci_str,
                    cs: cs_str,
                    fs: fs_str,
                    ps: None, // TODO: path-sensitive in Task 5
                    best,
                });
            }
        }
```

**Step 2: Add helper functions at the bottom of driver.rs**

```rust
/// Parse a hex ValueId string ("0x...") into a `ValueId`.
fn parse_value_id(hex: &str) -> anyhow::Result<saf_core::id::ValueId> {
    saf_core::id::ValueId::from_hex(hex)
        .ok_or_else(|| anyhow::anyhow!("Invalid ValueId hex: {hex}"))
}

/// Format an `AliasResult` as a string for JSON output.
fn format_alias_result(r: saf_analysis::pta::AliasResult) -> String {
    match r {
        saf_analysis::pta::AliasResult::Must => "Must".to_string(),
        saf_analysis::pta::AliasResult::May => "May".to_string(),
        saf_analysis::pta::AliasResult::No => "No".to_string(),
        saf_analysis::pta::AliasResult::Partial => "Partial".to_string(),
        saf_analysis::pta::AliasResult::Unknown => "Unknown".to_string(),
    }
}
```

**Note:** The exact APIs for `self.db.pta_result()`, `self.db.cfgs()`, `self.db.resolved_sites()`, `self.db.specs()` etc. need to be verified against `ProgramDatabase`. Check `crates/saf-analysis/src/database.rs` for the actual accessor names and adjust. The code above shows the intent — exact method names may differ.

**Step 3: Verify it compiles**

Run: `docker compose run --rm dev sh -c 'cargo check -p saf-cli 2>&1' | tail -20`

**Step 4: Commit**

```
feat(cli): wire alias queries into bench mode (CI + CS + FS)
```

---

### Task 5: Wire checker findings into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add checker execution after alias queries**

In `run_bench_mode`, add:

```rust
        // ── Checker findings (SVFG) ─────────────────────────────
        if bench_config.analyses.checkers {
            // Run all SVFG checkers via the protocol handler
            let checker_request = r#"{"action":"run_checkers","checker_names":[]}"#;
            if let Ok(response) = self.db.handle_request(checker_request) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) {
                    if let Some(findings) = parsed.get("findings").and_then(|f| f.as_array()) {
                        for finding in findings {
                            result.checker_findings.push(BenchCheckerFinding {
                                check: finding.get("check")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                alloc_site: finding.get("object_id")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                severity: finding.get("severity")
                                    .and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                call_sites: finding.get("path")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter()
                                        .filter_map(|e| e.get("inst_id").and_then(|v| v.as_str()).map(String::from))
                                        .collect())
                                    .unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
```

**Note:** The exact JSON protocol format for `handle_request` must be verified against `crates/saf-analysis/src/database/protocol.rs`. The code above shows the intent — adapt the field names based on the actual protocol response schema.

**Step 2: Verify it compiles**

**Step 3: Commit**

```
feat(cli): wire SVFG checker findings into bench mode
```

---

### Task 6: Wire assertion proving into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add Z3 assertion proving**

```rust
        // ── Assertion proving (Z3) ────────────────────────────────
        if bench_config.analyses.z3_prove || bench_config.analyses.absint {
            let absint_config = saf_analysis::absint::AbstractInterpConfig::default();
            let pta_result = self.db.pta_result();
            let specs = self.db.specs().cloned().unwrap_or_default();

            let interprocedural = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
                &self.module,
                &absint_config,
                pta_result,
                Some(&specs),
            );

            if bench_config.analyses.z3_prove {
                let condition_result = saf_analysis::z3_utils::prove_conditions_interprocedural(
                    &self.module,
                    &interprocedural,
                    "svf_assert",
                );

                for finding in &condition_result.proven {
                    result.assertion_results.push(AssertionResultEntry {
                        call_site: format!("0x{:032x}", finding.inst.raw()),
                        kind: "svf_assert".to_string(),
                        proved: true,
                    });
                }
                for finding in condition_result.may_fail.iter().chain(&condition_result.unknown) {
                    result.assertion_results.push(AssertionResultEntry {
                        call_site: format!("0x{:032x}", finding.inst.raw()),
                        kind: "svf_assert".to_string(),
                        proved: false,
                    });
                }
            }

            // Store interprocedural result for interval and nullness queries
            // (used in Tasks 7 and 8)
        }
```

**Note:** The interprocedural result is needed by interval queries (Task 7) and nullness (Task 8). Extract it to a local variable shared across these blocks. The exact scope management will depend on how the full `run_bench_mode` is structured — consider computing interprocedural once and sharing.

**Step 2: Commit**

```
feat(cli): wire Z3 assertion proving into bench mode
```

---

### Task 7: Wire interval queries (assert_eq) into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add interval query execution using the interprocedural result from Task 6**

The `InterproceduralResult` provides `interval_at_inst(inst, value, bits)`. For `svf_assert_eq(a, b)`, the harness needs to know if the intervals of `a` and `b` overlap.

```rust
        // ── Interval queries (assert_eq) ────────────────────────
        // Uses interprocedural result from assertion proving above.
        // The bench config doesn't explicitly list assert_eq queries —
        // instead, the harness extracts AssertEq expectations and passes
        // them as part of the analysis flags. The CLI outputs all
        // interval information it can compute.
        //
        // For now, the interprocedural result is available if absint was run.
        // Specific per-value interval queries will be added when the harness
        // generates them in the bench config.
```

**Note:** This task requires understanding how `validate_assert_eq` in ptaben.rs uses the `AbstractInterpResult`. The exact query interface (passing specific ValueId pairs for interval comparison) will be defined when we rewrite the PTABen harness in Phase 3. For now, ensure the interprocedural result is computed and available; the detailed interval extraction will be added alongside the harness rewrite.

**Step 2: Commit**

```
feat(cli): prepare interval query support in bench mode
```

---

### Task 8: Wire nullness analysis into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add nullness analysis**

```rust
        // ── Nullness analysis ───────────────────────────────────
        if bench_config.analyses.nullness && !bench_config.nullness_queries.is_empty() {
            let nullness_config = saf_analysis::absint::NullnessConfig::default();
            let pta_result = self.db.pta_result();
            let specs = self.db.specs().cloned().unwrap_or_default();

            // Need interprocedural summaries for nullness
            let absint_config = saf_analysis::absint::AbstractInterpConfig::default();
            let interprocedural = saf_analysis::absint::solve_interprocedural_with_pta_and_specs(
                &self.module, &absint_config, pta_result, Some(&specs),
            );
            let summaries = interprocedural.summaries();

            let pta_integration = saf_analysis::PtaIntegration::new(
                &pta_result.pts, &pta_result.factory,
            );

            let nullness_result = saf_analysis::absint::analyze_nullness_with_pta_specs_and_summaries(
                &self.module, &nullness_config, &pta_integration, &specs, summaries,
            );

            for query in &bench_config.nullness_queries {
                let ptr = parse_value_id(&query.ptr)?;
                let call_site = parse_inst_id(&query.call_site)?;

                let nullness = nullness_result.nullness_at(call_site, ptr);
                let may_null = matches!(
                    nullness,
                    saf_analysis::absint::Nullness::Null | saf_analysis::absint::Nullness::MaybeNull
                );

                result.nullness_results.push(NullnessResultEntry {
                    ptr: query.ptr.clone(),
                    call_site: query.call_site.clone(),
                    may_null,
                });
            }
        }
```

**Step 2: Add `parse_inst_id` helper**

```rust
fn parse_inst_id(hex: &str) -> anyhow::Result<saf_core::id::InstId> {
    saf_core::id::InstId::from_hex(hex)
        .ok_or_else(|| anyhow::anyhow!("Invalid InstId hex: {hex}"))
}
```

**Note:** Verify that `PtaIntegration::new()` and `interprocedural.summaries()` exist with these exact signatures. The ptaben.rs code at line 720 shows the actual calling convention. Adjust field access as needed.

**Step 3: Commit**

```
feat(cli): wire nullness analysis into bench mode
```

---

### Task 9: Wire buffer overflow analysis into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add buffer overflow checking**

```rust
        // ── Buffer overflow ─────────────────────────────────────
        if bench_config.analyses.buffer_overflow {
            let pta_result = self.db.pta_result();
            let specs = self.db.specs().cloned().unwrap_or_default();

            let mut findings = saf_analysis::absint::check_buffer_overflow_with_pta(
                &self.module, pta_result,
            );
            findings.extend(saf_analysis::absint::check_memcpy_overflow_with_pta_and_specs(
                &self.module, pta_result, &specs,
            ));
            findings.extend(saf_analysis::absint::check_memcpy_overflow_with_specs(
                &self.module, &specs,
            ));

            for finding in &findings {
                result.buffer_findings.push(BenchBufferFinding {
                    ptr: format!("0x{:032x}", finding.value_id.map(|v| v.raw()).unwrap_or(0)),
                    function: finding.function.clone(),
                    kind: format!("{:?}", finding.checker),
                    description: finding.description.clone(),
                });
            }
        }
```

**Note:** Verify the `NumericFinding` struct has `value_id`, `function`, `checker`, `description` fields. Check `crates/saf-analysis/src/absint/mod.rs` for the exact definition. The ptaben.rs code at lines 672-706 shows the calling pattern.

**Step 2: Commit**

```
feat(cli): wire buffer overflow analysis into bench mode
```

---

### Task 10: Wire MTA analysis into bench mode

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (extend `run_bench_mode`)

**Step 1: Add MTA (multithreading) analysis**

```rust
        // ── MTA analysis ──────────────────────────────────────
        if bench_config.analyses.mta {
            let pta_result = self.db.pta_result();
            let callgraph = self.db.callgraph();

            // Build ICFG for thread analysis
            let icfg = saf_analysis::icfg::Icfg::build(&self.module, callgraph);
            let mta_config = saf_analysis::mta::MtaConfig::default();

            let mta = saf_analysis::mta::MtaAnalysis::with_pta(
                &self.module, callgraph, &icfg, mta_config,
                &pta_result.pts, &pta_result.factory,
            );
            let mta_result = mta.analyze();

            let mut mta_output = BenchMtaResults::default();

            // Export thread contexts
            let threads = mta_result.threads();
            for (&thread_id, _ctx) in threads {
                mta_output.thread_contexts.push(BenchThreadContextEntry {
                    thread_id: thread_id.raw(),
                    exists: true,
                });
            }

            // TODO: Interleaving and TCT queries require specific
            // instruction/thread pairs from the bench config.
            // These will be wired when the harness generates them.

            result.mta_results = Some(mta_output);
        }
```

**Note:** Verify `MtaAnalysis::with_pta` signature against the exploration results. The ptaben.rs code at lines 731-749 shows `MtaAnalysis::with_pta(module, callgraph, &icfg, config, &pta_result.pts, &pta_result.factory)`. Also verify `ThreadId::raw()` returns a u64.

**Step 2: Commit**

```
feat(cli): wire MTA analysis into bench mode
```

---

### Task 11: Add PTA config overrides from bench-config

**Files:**
- Modify: `crates/saf-cli/src/driver.rs` (modify `build()` to apply bench PTA config)
- Modify: `crates/saf-cli/src/commands.rs` (add new CLI args)

**Step 1: Add `--constant-indices` and `--z3-index-refinement` args to RunArgs**

In `commands.rs`, add:

```rust
    /// Enable constant-index sensitivity for PTA.
    #[arg(long, default_value_t = false)]
    pub constant_indices: bool,

    /// Enable Z3-based index refinement for PTA.
    #[arg(long, default_value_t = false)]
    pub z3_index_refinement: bool,

    /// Field sensitivity depth (numeric, e.g. 10).
    #[arg(long)]
    pub field_depth: Option<u32>,
```

**Step 2: Add fields to DriverConfig and mapping**

In `driver.rs`:

```rust
    pub constant_indices: bool,
    pub z3_index_refinement: bool,
    pub field_depth: Option<u32>,
```

And in `from_run_args`:

```rust
    constant_indices: args.constant_indices,
    z3_index_refinement: args.z3_index_refinement,
    field_depth: args.field_depth,
```

**Step 3: Apply in `build()`**

In `AnalysisDriver::build()`, after PTS config (around line 330):

```rust
        // Apply constant-index and Z3 refinement settings
        if config.constant_indices {
            pipeline_config.refinement.pta_config = pipeline_config
                .refinement.pta_config.with_constant_indices();
        }
        if config.z3_index_refinement {
            pipeline_config.refinement.pta_config = pipeline_config
                .refinement.pta_config.with_z3_index_refinement(true);
        }
        if let Some(depth) = config.field_depth {
            pipeline_config.refinement.pta_config.field_sensitivity =
                FieldSensitivity::StructFields { max_depth: depth as usize };
        }
```

**Step 4: In bench mode, apply BenchPtaConfig overrides**

In `commands::run()`, when bench mode is active, apply overrides from BenchPtaConfig to the DriverConfig before calling `build()`:

```rust
    if let Some(ref bench_config_path) = config.bench_config {
        // ... parse bench_config ...
        // Override PTA config from bench config
        config.constant_indices = bench_config.pta_config.constant_indices;
        config.z3_index_refinement = bench_config.pta_config.z3_index_refinement;
        config.field_depth = Some(bench_config.pta_config.field_depth);
        config.max_pta_iterations = Some(bench_config.pta_config.max_iterations);
        // ... build driver ...
    }
```

**Step 5: Verify and commit**

```
feat(cli): add PTA config overrides (constant-indices, z3-index-refinement, field-depth)
```

---

## Phase 3: Rewrite saf-bench to Use Subprocess

### Task 12: Add tempfile dependency and subprocess runner to saf-bench

**Files:**
- Modify: `crates/saf-bench/Cargo.toml` (add `tempfile` dependency)
- Create: `crates/saf-bench/src/runner.rs` (subprocess management)

**Step 1: Add tempfile dependency**

In `Cargo.toml`:
```toml
tempfile = "3"
```

**Step 2: Create runner.rs with subprocess utilities**

```rust
//! Subprocess runner for invoking `saf run --bench-config`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use saf_cli::bench_types::{BenchConfig, BenchResult};

/// Find the `saf` binary. Prefers the same target directory as our own binary.
fn find_saf_binary() -> Result<PathBuf> {
    // Look for saf in the same directory as this binary
    let self_path = std::env::current_exe()?;
    let dir = self_path.parent().unwrap_or(Path::new("."));
    let saf_path = dir.join("saf");
    if saf_path.exists() {
        return Ok(saf_path);
    }
    // Fall back to PATH
    Ok(PathBuf::from("saf"))
}

/// Run `saf run --bench-config` as a subprocess.
///
/// Writes `config` to `config_path`, invokes `saf`, reads result from
/// `result_path`. Returns the parsed `BenchResult`.
pub fn run_saf_bench(
    input: &Path,
    config: &BenchConfig,
    config_path: &Path,
    result_path: &Path,
) -> Result<BenchResult> {
    // Write bench config
    let config_json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path, &config_json)?;

    let saf = find_saf_binary()?;
    let status = Command::new(&saf)
        .arg("run")
        .arg("--input")
        .arg(input)
        .arg("--bench-config")
        .arg(config_path)
        .arg("--output")
        .arg(result_path)
        .status()
        .with_context(|| format!("Failed to execute: {}", saf.display()))?;

    if !status.success() {
        // Process failed — return error result
        return Ok(BenchResult {
            success: false,
            error: Some(format!("saf exited with status {status}")),
            ..Default::default()
        });
    }

    // Parse result
    let result_json = std::fs::read_to_string(result_path)
        .with_context(|| format!("Failed to read result: {}", result_path.display()))?;
    let result: BenchResult = serde_json::from_str(&result_json)
        .with_context(|| "Failed to parse bench result JSON")?;
    Ok(result)
}
```

**Note:** `BenchConfig` needs `Serialize` derive (not just `Deserialize`) since saf-bench writes it. Add `#[derive(Debug, Serialize, Deserialize)]` to `BenchConfig` and all its input types in bench_types.rs.

**Step 3: Add `pub mod runner;` to saf-bench's lib.rs or main.rs module list**

**Step 4: Verify and commit**

```
feat(bench): add subprocess runner for saf CLI invocation
```

---

### Task 13: Add saf-cli as a dependency of saf-bench (for types only)

**Files:**
- Modify: `crates/saf-bench/Cargo.toml`

**Step 1: Add saf-cli as a dependency for the shared types**

```toml
saf-cli = { path = "../saf-cli" }
```

This gives saf-bench access to `BenchConfig`, `BenchResult`, and all the serde types. saf-bench only imports the types — it does NOT call `AnalysisDriver` directly.

**Step 2: Verify and commit**

```
feat(bench): add saf-cli dependency for BenchConfig/BenchResult types
```

---

### Task 14: Rewrite PTABen to use subprocess wrapping

This is the largest task. It replaces the inline analysis pipeline in `ptaben.rs::validate()` with subprocess invocation.

**Files:**
- Modify: `crates/saf-bench/src/ptaben.rs` (major rewrite of `validate()`)

**Step 1: Rewrite validate() to use subprocess**

The new flow:
1. Extract expectations (unchanged)
2. Build `BenchConfig` from expectations (new)
3. Spawn `saf run --bench-config` via `runner::run_saf_bench()` (new)
4. Validate each expectation against `BenchResult` (adapted)

Replace the analysis pipeline (lines ~437-749) with:

```rust
fn validate(&self, test: &TestCase, bundle: &AirBundle) -> Result<TestResult> {
    let start = std::time::Instant::now();
    let expectations = self.extract_expectations(bundle)?;

    if expectations.is_empty() {
        return Ok(TestResult {
            test: test.clone(),
            outcomes: vec![],
            elapsed: start.elapsed(),
        });
    }

    // Determine which analyses are needed
    let needs_alias = expectations.iter().any(|e| matches!(e, Expectation::Alias { .. }));
    let needs_svfg = expectations.iter().any(|e| matches!(e,
        Expectation::MemLeak { .. } | Expectation::DoubleFree { .. }));
    let needs_assertions = expectations.iter().any(|e| matches!(e, Expectation::Assert { .. }));
    let needs_assert_eq = expectations.iter().any(|e| matches!(e, Expectation::AssertEq { .. }));
    let needs_null_check = expectations.iter().any(|e| matches!(e, Expectation::NullCheck { .. }));
    let needs_buffer_access = expectations.iter().any(|e| matches!(e, Expectation::BufferAccess { .. }));
    let needs_mta = expectations.iter().any(|e| matches!(e,
        Expectation::InterleavingAccess { .. }
        | Expectation::ThreadContext { .. }
        | Expectation::TctAccess { .. }));

    // Build bench config
    let bench_config = build_bench_config(
        &expectations, needs_alias, needs_svfg, needs_assertions,
        needs_assert_eq, needs_null_check, needs_buffer_access, needs_mta,
        self.solver,
    );

    // Run saf CLI
    let config_path = self.temp_dir.path().join(format!("{}-config.json", test.name));
    let result_path = self.temp_dir.path().join(format!("{}-result.json", test.name));
    let bench_result = runner::run_saf_bench(
        &test.path, &bench_config, &config_path, &result_path,
    )?;

    if !bench_result.success {
        return Ok(TestResult {
            test: test.clone(),
            outcomes: vec![Outcome::Skip {
                reason: bench_result.error.unwrap_or_else(|| "Unknown error".to_string()),
            }],
            elapsed: start.elapsed(),
        });
    }

    // Validate each expectation against CLI results
    let outcomes = expectations.iter().map(|exp| {
        validate_expectation_from_bench_result(exp, &bench_result)
    }).collect();

    Ok(TestResult {
        test: test.clone(),
        outcomes,
        elapsed: start.elapsed(),
    })
}
```

**Step 2: Implement `build_bench_config()`**

This function translates extracted expectations into a `BenchConfig`:

```rust
fn build_bench_config(
    expectations: &[Expectation],
    needs_alias: bool,
    needs_svfg: bool,
    needs_assertions: bool,
    needs_assert_eq: bool,
    needs_null_check: bool,
    needs_buffer_access: bool,
    needs_mta: bool,
    solver: PtaSolver,
) -> BenchConfig {
    use saf_cli::bench_types::*;

    let mut alias_queries = Vec::new();
    let mut nullness_queries = Vec::new();

    for exp in expectations {
        match exp {
            Expectation::Alias { ptr_a, ptr_b, oracle_block, oracle_function, .. } => {
                alias_queries.push(AliasQuery {
                    ptr_a: format!("0x{:032x}", ptr_a.raw()),
                    ptr_b: format!("0x{:032x}", ptr_b.raw()),
                    oracle_block: oracle_block.map(|b| format!("0x{:032x}", b.raw())),
                    oracle_function: oracle_function.map(|f| format!("0x{:032x}", f.raw())),
                });
            }
            Expectation::NullCheck { ptr, call_site, .. } => {
                nullness_queries.push(NullnessQuery {
                    ptr: format!("0x{:032x}", ptr.raw()),
                    call_site: format!("0x{:032x}", call_site.raw()),
                });
            }
            _ => {}
        }
    }

    BenchConfig {
        alias_queries,
        nullness_queries,
        analyses: AnalysisFlags {
            checkers: needs_svfg,
            z3_prove: needs_assertions,
            nullness: needs_null_check,
            mta: needs_mta,
            buffer_overflow: needs_buffer_access,
            absint: needs_assert_eq || needs_assertions,
            cspta: needs_alias,
            fspta: needs_alias,
        },
        pta_config: BenchPtaConfig {
            solver: match solver {
                PtaSolver::Worklist => "worklist".to_string(),
                PtaSolver::Datalog => "datalog".to_string(),
            },
            ..BenchPtaConfig::default()
        },
    }
}
```

**Step 3: Implement `validate_expectation_from_bench_result()`**

This replaces the old `validate_expectation()` that took raw analysis structures. It now reads from the JSON result:

```rust
fn validate_expectation_from_bench_result(
    exp: &Expectation,
    result: &BenchResult,
) -> Outcome {
    match exp {
        Expectation::Alias { kind, ptr_a, ptr_b, .. } => {
            let ptr_a_hex = format!("0x{:032x}", ptr_a.raw());
            let ptr_b_hex = format!("0x{:032x}", ptr_b.raw());
            let alias_entry = result.alias_results.iter().find(|r|
                r.ptr_a == ptr_a_hex && r.ptr_b == ptr_b_hex
            );
            match alias_entry {
                Some(entry) => {
                    let actual = parse_alias_string(&entry.best);
                    check_alias_result(*kind, actual)
                }
                None => Outcome::Skip {
                    reason: "Alias query result not found in CLI output".to_string(),
                },
            }
        }
        Expectation::MemLeak { kind, alloc_site, .. } => {
            let alloc_hex = format!("0x{:032x}", alloc_site.raw());
            validate_mem_leak_from_findings(*kind, &alloc_hex, &result.checker_findings)
        }
        Expectation::DoubleFree { kind, alloc_site, .. } => {
            let alloc_hex = format!("0x{:032x}", alloc_site.raw());
            validate_double_free_from_findings(*kind, &alloc_hex, &result.checker_findings)
        }
        Expectation::Assert { kind, call_site } => {
            let cs_hex = format!("0x{:032x}", call_site.raw());
            let entry = result.assertion_results.iter().find(|r| r.call_site == cs_hex);
            match entry {
                Some(e) => validate_assert_from_result(*kind, e.proved),
                None => Outcome::Skip { reason: "Assertion result not found".to_string() },
            }
        }
        Expectation::NullCheck { kind, ptr, call_site } => {
            let ptr_hex = format!("0x{:032x}", ptr.raw());
            let cs_hex = format!("0x{:032x}", call_site.raw());
            let entry = result.nullness_results.iter().find(|r|
                r.ptr == ptr_hex && r.call_site == cs_hex
            );
            match entry {
                Some(e) => validate_null_from_result(*kind, e.may_null),
                None => Outcome::Skip { reason: "Nullness result not found".to_string() },
            }
        }
        Expectation::BufferAccess { kind, ptr, call_site, .. } => {
            let ptr_hex = format!("0x{:032x}", ptr.raw());
            validate_buffer_from_findings(*kind, &ptr_hex, &result.buffer_findings)
        }
        Expectation::InterleavingAccess { .. }
        | Expectation::ThreadContext { .. }
        | Expectation::TctAccess { .. } => {
            // MTA validation from bench result
            // TODO: wire when MTA output is fully populated
            Outcome::Skip { reason: "MTA validation via CLI not yet implemented".to_string() }
        }
        _ => Outcome::Skip { reason: "Oracle type not supported in CLI mode".to_string() },
    }
}
```

**Note:** Helper functions like `validate_mem_leak_from_findings`, `validate_double_free_from_findings`, `validate_assert_from_result`, `validate_null_from_result`, `validate_buffer_from_findings`, and `parse_alias_string` need to be implemented. These map the string-based CLI output back to the same validation logic currently in `validate_mem_leak()`, `validate_double_free()`, etc. The existing validation functions can be adapted to work with string IDs.

**Step 4: Add `temp_dir: tempfile::TempDir` to PTABen struct**

Update the `PTABen` constructor to create a temp dir:

```rust
pub struct PTABen {
    solver: PtaSolver,
    temp_dir: tempfile::TempDir,
}

impl PTABen {
    pub fn new(solver: PtaSolver) -> Result<Self> {
        Ok(Self {
            solver,
            temp_dir: tempfile::TempDir::new()?,
        })
    }
}
```

The `TempDir` is automatically cleaned up when `PTABen` is dropped.

**Step 5: Remove old analysis pipeline code**

Delete the old inline analysis pipeline (roughly lines 437-749 of the current ptaben.rs). All the imports for `PtaContext`, `RefinementConfig`, `MemorySsa`, `SvfgBuilder`, `MtaAnalysis`, etc. can be removed.

**Step 6: Verify and commit**

```
refactor(bench): rewrite PTABen to use saf CLI subprocess wrapping
```

---

### Task 15: Rewrite CruxBC to use subprocess wrapping

**Files:**
- Modify: `crates/saf-bench/src/cruxbc.rs`

**Step 1: Replace inline analysis pipeline with subprocess call**

CruxBC is simpler — it mainly cares about:
- Did the analysis complete successfully?
- How long did it take?
- What are the pipeline stats?

The bench config for CruxBC:
```rust
let bench_config = BenchConfig {
    alias_queries: vec![],
    nullness_queries: vec![],
    analyses: AnalysisFlags {
        checkers: true,
        fspta: run_fspta,
        ..Default::default()
    },
    pta_config: BenchPtaConfig::default(),
};
```

Replace the inline pipeline (~lines 450-590) with `runner::run_saf_bench()`, then extract timing from `bench_result.stats`.

**Step 2: Remove old imports**

Remove `LlvmFrontend`, `RefinementConfig`, `PtaContext`, `MemorySsa`, `SvfgBuilder`, `FsSvfgBuilder`, etc.

**Step 3: Verify and commit**

```
refactor(bench): rewrite CruxBC to use saf CLI subprocess wrapping
```

---

### Task 16: Rewrite SV-COMP/Juliet to use subprocess wrapping

**Files:**
- Modify: `crates/saf-bench/src/svcomp/mod.rs`
- Modify: `crates/saf-bench/src/juliet.rs`

**Step 1: Replace inline analysis in SV-COMP**

SV-COMP needs abstract interpretation + buffer overflow + nullness findings. The bench config:
```rust
let bench_config = BenchConfig {
    analyses: AnalysisFlags {
        absint: true,
        buffer_overflow: true,
        nullness: true,
        ..Default::default()
    },
    ..Default::default()
};
```

**Step 2: Replace inline analysis in Juliet**

Same pattern as SV-COMP.

**Step 3: Verify and commit**

```
refactor(bench): rewrite SV-COMP and Juliet to use saf CLI subprocess
```

---

### Task 17: Remove dead code and clean up saf-bench

**Files:**
- Modify: `crates/saf-bench/src/main.rs` (remove `load_bitcode` if unused)
- Modify: `crates/saf-bench/Cargo.toml` (remove unused saf-analysis direct dependencies if possible)

**Step 1: Remove `load_bitcode` from main.rs**

If all benchmark suites now use the subprocess runner, the `load_bitcode` function (~line 470) is dead code. Remove it.

**Note:** saf-bench still needs `saf-frontends` for oracle extraction (parsing LLVM IR to find oracle calls). But it no longer needs direct saf-analysis dependencies for PTA, MSSA, SVFG, etc. Check which deps can be removed.

**Step 2: Run full test suite**

Run: `make fmt && make lint` then `make test`

**Step 3: Run PTABen benchmark to verify results match**

Run (in background):
```bash
docker compose run --rm dev sh -c 'cargo run --release -p saf-bench -- ptaben --compiled-dir tests/benchmarks/ptaben/.compiled -o /workspace/tests/benchmarks/ptaben/results.json'
```

Compare unsound/exact/skip counts against the previous baseline (61 unsound from context).

**Step 4: Commit**

```
refactor(bench): remove dead analysis pipeline code from saf-bench
```

---

## Phase 4: Verification & Documentation

### Task 18: End-to-end verification

**Step 1: Run all benchmarks**

- PTABen: verify same unsound count as baseline (61)
- CruxBC: verify all programs complete with reasonable timing
- Run `make test` for all Rust + Python tests

**Step 2: Update CLAUDE.md if needed**

If benchmark commands changed, update the PTABen/CruxBC sections.

**Step 3: Final commit**

```
docs: update benchmark documentation for CLI wrapping
```

---

## Dependency Graph

```
Task 1 (types) → Task 2 (CLI arg) → Task 3 (stub) → Tasks 4-11 (wire analyses, parallel)
                                                    → Task 12 (runner) → Task 13 (dep) → Tasks 14-16 (rewrite suites)
                                                                                        → Task 17 (cleanup) → Task 18 (verify)
```

Tasks 4-11 can be done in any order (each adds one analysis to bench mode).
Tasks 14-16 can be done in any order (each rewrites one benchmark suite).
Task 17 depends on all rewrites being complete.
Task 18 is the final verification.
