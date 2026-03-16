# CLI Feature Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire all saf-analysis features into saf-cli, achieving full feature parity with the Python SDK (28 features across 8 commands).

**Architecture:** New `driver.rs` wraps `ProgramDatabase` with frontend ingestion, extended analyses (CSPTA, FSPTA, DDA, IFDS, absint, Z3, typestate, combined), and output formatting. New `help.rs` provides 11 guide topics. Existing `commands.rs` is rewritten with full arg structs. `saf-cli/Cargo.toml` gains `saf-frontends` and `saf-analysis` dependencies.

**Tech Stack:** Rust, clap (CLI), saf-core/saf-frontends/saf-analysis crates, serde_json/serde_yaml (output).

**Design:** `docs/plans/2026-02-25-cli-feature-parity-design.md`

**IMPORTANT:** All builds/tests run in Docker. The main agent must run `make fmt && make lint` and `make test`. Subagents must NEVER call `make` commands.

---

## Phase 1: Foundation — Dependencies + Driver Skeleton + `saf run` (Core Pipeline)

### Task 1.1: Add saf-frontends and saf-analysis dependencies to saf-cli

**Files:**
- Modify: `crates/saf-cli/Cargo.toml`

**Step 1: Add dependencies**

Add `saf-frontends` and `saf-analysis` as workspace dependencies:

```toml
[dependencies]
saf-core = { workspace = true }
saf-frontends = { workspace = true }
saf-analysis = { workspace = true }
clap = { workspace = true }
anyhow = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
glob = { workspace = true }
tracing-subscriber = { workspace = true }
```

**Step 2: Verify it compiles**

Run: `make fmt && make lint`
Expected: PASS (no new code yet, just dependency additions)

**Step 3: Commit**

```bash
git add crates/saf-cli/Cargo.toml
git commit -m "build: add saf-frontends and saf-analysis deps to saf-cli"
```

---

### Task 1.2: Create driver.rs skeleton with DriverConfig and AnalysisDriver

**Files:**
- Create: `crates/saf-cli/src/driver.rs`
- Modify: `crates/saf-cli/src/main.rs` (add `mod driver;`)

**Step 1: Create driver.rs with config types and empty AnalysisDriver**

Create `crates/saf-cli/src/driver.rs` with:

- `PtaVariant` enum: `Andersen`, `CsPta`, `FsPta`, `Dda`
- `PtaSolverKind` enum: `Worklist`, `Datalog`
- `CheckerSelection` enum: `All`, `None`, `Specific(Vec<String>)`
- `Z3Config` struct: `prove`, `refine_alias`, `check_reachability`, `timeout_ms`
- `OutputFormat` enum: `Human`, `Json`, `Sarif`
- `OutputConfig` struct: `format`, `path`, `verbose`, `diagnostics`
- `DriverConfig` struct: all fields from design doc
- `AnalysisOutput` struct: `stats`, `checker_findings`, `numeric_findings`, etc.
- `AnalysisDriver` struct: `db: ProgramDatabase`, `module: Arc<AirModule>`, `specs: SpecRegistry`

All analysis methods on `AnalysisDriver` should be stubs (`todo!()`) at this stage except:
- `ingest()` — use `LlvmFrontend::ingest()` / `AirJsonFrontend` based on `CliFrontend`
- `build()` — call `ingest()` then `ProgramDatabase::build()`

Reference `crates/saf-bench/src/main.rs:439-445` for the ingestion pattern:
```rust
let frontend = LlvmFrontend::default();
let config = Config::default();
let bundle = frontend.ingest(&[path.as_path()], &config)?;
```

Reference `crates/saf-analysis/src/database/mod.rs:66-78` for `ProgramDatabase::build()`.

Reference `crates/saf-analysis/src/pipeline.rs:57-95` for `PipelineConfig` and `PipelineConfig::from_mode()`.

The `build()` method must:
1. Call `ingest()` to get `AirBundle`
2. Build `PipelineConfig` from `DriverConfig` (mode, PTA settings, specs)
3. Set `PtaConfig` fields: `field_sensitivity`, `max_iterations`, `pts_config` with PTS representation
4. Load `SpecRegistry` from `specs_path` or default
5. Call `ProgramDatabase::build(module, &pipeline_config)`
6. Return `AnalysisDriver { db, module, specs }`

**Step 2: Add `mod driver;` to main.rs**

Add `mod driver;` to `crates/saf-cli/src/main.rs`.

**Step 3: Verify it compiles**

Run: `make fmt && make lint`
Expected: PASS (some dead-code warnings expected for unused types — that's fine)

**Step 4: Commit**

```bash
git add crates/saf-cli/src/driver.rs crates/saf-cli/src/main.rs
git commit -m "feat(cli): add driver.rs skeleton with DriverConfig and AnalysisDriver"
```

---

### Task 1.3: Rewrite RunArgs and implement `saf run` (core pipeline + basic output)

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Rewrite RunArgs in commands.rs**

Replace the existing `RunArgs` struct with the full version from the design:

```rust
#[derive(Args)]
pub struct RunArgs {
    /// Input files to analyze.
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Frontend to use for ingestion.
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,

    /// Analysis mode.
    #[arg(long, value_enum, default_value_t = CliAnalysisMode::Precise)]
    pub mode: CliAnalysisMode,

    /// Additional spec files or directories.
    #[arg(long)]
    pub specs: Option<PathBuf>,

    /// PTA variant.
    #[arg(long, value_enum, default_value_t = CliPtaVariant::Andersen)]
    pub pta: CliPtaVariant,

    /// PTA solver backend.
    #[arg(long, value_enum, default_value_t = CliPtaSolver::Worklist)]
    pub solver: CliPtaSolver,

    /// PTS representation.
    #[arg(long, value_enum, default_value_t = CliPtsRepr::Auto)]
    pub pts_repr: CliPtsRepr,

    /// k-CFA depth (cspta only).
    #[arg(long, default_value_t = 2)]
    pub pta_k: u32,

    /// Field sensitivity level.
    #[arg(long, value_enum, default_value_t = CliFieldSensitivity::StructFields)]
    pub field_sensitivity: CliFieldSensitivity,

    /// Maximum PTA iterations.
    #[arg(long)]
    pub max_pta_iterations: Option<usize>,

    /// Checkers to run.
    #[arg(long, default_value = "all")]
    pub checkers: String,

    /// Enable Z3 path-sensitive checker filtering.
    #[arg(long)]
    pub path_sensitive: bool,

    /// Run builtin typestate spec.
    #[arg(long)]
    pub typestate: Option<String>,

    /// Run custom typestate spec from YAML.
    #[arg(long)]
    pub typestate_custom: Option<PathBuf>,

    /// Prove assertions via Z3.
    #[arg(long)]
    pub z3_prove: bool,

    /// Refine alias results via Z3.
    #[arg(long)]
    pub z3_refine_alias: bool,

    /// Check path reachability via Z3.
    #[arg(long)]
    pub z3_check_reachability: bool,

    /// Z3 solver timeout in ms.
    #[arg(long, default_value_t = 5000)]
    pub z3_timeout: u64,

    /// Run combined PTA + abstract interpretation.
    #[arg(long)]
    pub combined: bool,

    /// Run IFDS taint analysis with config file.
    #[arg(long)]
    pub ifds_taint: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Human)]
    pub format: CliOutputFormat,

    /// Write output to file.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Include checker/PTA diagnostics.
    #[arg(long)]
    pub diagnostics: bool,

    /// Show timing, resource table, stats.
    #[arg(long)]
    pub verbose: bool,

    /// Start JSON protocol server on stdin/stdout.
    #[arg(long)]
    pub serve: bool,
}
```

Add the required CLI enum wrappers (`CliPtaVariant`, `CliPtaSolver`, `CliPtsRepr`, `CliFieldSensitivity`, `CliOutputFormat`) with `ValueEnum` derives and `Display` impls, following the existing `CliFrontend`/`CliAnalysisMode` pattern at the top of `commands.rs`.

**Step 2: Implement `DriverConfig::from_run_args()`**

In `driver.rs`, add a method that converts `RunArgs` to `DriverConfig`, mapping each CLI enum to its analysis-crate counterpart.

**Step 3: Implement `run()` command**

Replace the `run()` stub in `commands.rs`:

```rust
pub fn run(args: &RunArgs) -> anyhow::Result<()> {
    let config = DriverConfig::from_run_args(args);

    if args.serve {
        let driver = AnalysisDriver::build(&args.inputs, &config)?;
        return driver.serve();
    }

    let driver = AnalysisDriver::build(&args.inputs, &config)?;
    let output = driver.analyze(&config)?;
    driver.format_output(&output, &config.output)
}
```

**Step 4: Implement `AnalysisDriver::analyze()` — checkers only for now**

In `driver.rs`, implement `analyze()` to run builtin SVFG checkers via `ProgramDatabase`:
- Parse `CheckerSelection` from the `--checkers` string
- If `All`: use `db.handle_check_all()` pattern (see `crates/saf-analysis/src/database/handler.rs:140-161`)
- If `Specific`: run each named checker via `db.run_svfg_check()`
- If `None`: skip checkers

Reference `crates/saf-analysis/src/checkers/spec.rs:385-407` for `builtin_checkers()` and `builtin_checker_names()`.

**Step 5: Implement `AnalysisDriver::format_output()` — human format**

For `OutputFormat::Human`:
- Print pipeline stats (timing): `driver.db.stats()`
- Print findings as a table: checker name, severity, message, location
- If `--verbose`: print resource table, PTA diagnostics

For `OutputFormat::Json`:
- Serialize `AnalysisOutput` as JSON

For `OutputFormat::Sarif`:
- Use `checkers::export_findings_sarif()` — reference `crates/saf-analysis/src/checkers/finding.rs:219`

Handle `--output` path vs stdout.

**Step 6: Implement `AnalysisDriver::serve()`**

Simple stdin/stdout JSON-line protocol loop:
```rust
pub fn serve(&self) -> anyhow::Result<()> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let response = self.db.handle_request(&line)
            .unwrap_or_else(|e| format!("{{\"status\":\"error\",\"error\":{{\"code\":\"SERIALIZE\",\"message\":\"{e}\"}}}}"));
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }
    Ok(())
}
```

**Step 7: Verify it compiles and run basic test**

Run: `make fmt && make lint`

Then test with an existing fixture inside Docker:
```bash
docker compose run --rm dev sh -c 'cargo run -p saf-cli -- run tests/fixtures/llvm/e2e/simple.ll --verbose 2>&1 | head -30'
```
Expected: Pipeline runs, prints stats and any findings found.

**Step 8: Commit**

```bash
git add crates/saf-cli/src/commands.rs crates/saf-cli/src/driver.rs
git commit -m "feat(cli): implement saf run with full pipeline, checkers, and serve mode"
```

---

## Phase 2: Index, Export, Query, Schema Commands

### Task 2.1: Implement `saf index`

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Update IndexArgs**

Add `--output` flag to `IndexArgs`:
```rust
#[derive(Args)]
pub struct IndexArgs {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,
    #[arg(long)]
    pub output: Option<PathBuf>,
}
```

**Step 2: Implement `index()` command**

```rust
pub fn index(args: &IndexArgs) -> anyhow::Result<()> {
    let bundle = AnalysisDriver::ingest(&args.inputs, args.frontend.into())?;
    let json = serde_json::to_string_pretty(&bundle)?;
    if let Some(ref path) = args.output {
        std::fs::write(path, &json)?;
        eprintln!("Wrote AIR-JSON to {}", path.display());
    } else {
        println!("{json}");
    }
    Ok(())
}
```

Reference: `AirBundle` derives `Serialize` — see `crates/saf-core/src/air.rs`.

**Step 3: Verify**

Run: `make fmt && make lint`

Test: `docker compose run --rm dev sh -c 'cargo run -p saf-cli -- index tests/fixtures/llvm/e2e/simple.ll | head -20'`
Expected: AIR-JSON output to stdout.

**Step 4: Commit**

```bash
git commit -m "feat(cli): implement saf index command"
```

---

### Task 2.2: Implement `saf export`

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Rewrite ExportArgs with full target list**

```rust
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliExportTarget {
    Cfg,
    Callgraph,
    Defuse,
    Valueflow,
    Svfg,
    Findings,
    Pta,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliExportFormat {
    Json,
    Sarif,
    Dot,
    Html,
}

#[derive(Args)]
pub struct ExportArgs {
    #[arg(required = true, value_enum)]
    pub target: CliExportTarget,
    #[arg(long, value_enum, default_value_t = CliExportFormat::Json)]
    pub format: CliExportFormat,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(required = true, long)]
    pub input: Vec<PathBuf>,
    #[arg(long)]
    pub function: Option<String>,
}
```

**Step 2: Implement `AnalysisDriver::export_graph()`**

In `driver.rs`, implement graph export for each target:
- `Cfg`: use `db.cfg(func_id)` then `Cfg::to_pg()` — requires `--function` flag. Reference `crates/saf-analysis/src/cfg.rs`.
- `Callgraph`: use `db.call_graph().to_pg(module)`. Reference `crates/saf-analysis/src/callgraph/mod.rs`.
- `Defuse`: use `db.defuse().to_pg(module)`. Reference `crates/saf-analysis/src/defuse.rs`.
- `Valueflow`: use `to_property_graph(db.valueflow(), module)`. Reference `crates/saf-analysis/src/valueflow/mod.rs`.
- `Svfg`: build SVFG via `db.get_or_build_svfg()`, then serialize. Note: SVFG may not have a `to_pg()` method — check and adapt.
- `Findings`: run all checkers, then format as JSON/SARIF.
- `Pta`: use `db.pta_result().unwrap().to_pg()`. Reference `crates/saf-analysis/src/pta/result.rs`.

For output formats:
- `Json`: `serde_json::to_string_pretty(&property_graph)`
- `Dot`: `property_graph.to_dot()` — reference `crates/saf-analysis/src/export.rs:180`
- `Html`: `property_graph.to_html()` — reference `crates/saf-analysis/src/export.rs:238`
- `Sarif`: only valid for `findings` target

**Step 3: Implement `export()` command in commands.rs**

Build the driver, call `export_graph()`, write output.

**Step 4: Verify**

Run: `make fmt && make lint`

Test: `docker compose run --rm dev sh -c 'cargo run -p saf-cli -- export callgraph --input tests/fixtures/llvm/e2e/simple.ll | head -20'`
Expected: JSON PropertyGraph of call graph.

**Step 5: Commit**

```bash
git commit -m "feat(cli): implement saf export with all targets and formats"
```

---

### Task 2.3: Implement `saf query`

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Rewrite QueryArgs as subcommand**

```rust
#[derive(Args)]
pub struct QueryArgs {
    #[command(subcommand)]
    pub command: QueryCommand,
    #[arg(long, required = true)]
    pub input: Vec<PathBuf>,
}

#[derive(Subcommand)]
pub enum QueryCommand {
    /// Points-to set for a value.
    #[command(name = "points-to")]
    PointsTo {
        /// Value ID (hex, e.g. 0x1234...).
        #[arg(required = true)]
        pointer: String,
    },
    /// May-alias check between two pointers.
    Alias {
        #[arg(required = true)]
        p: String,
        #[arg(required = true)]
        q: String,
    },
    /// Data-flow reachability.
    Flows {
        #[arg(required = true)]
        source: String,
        #[arg(required = true)]
        sink: String,
    },
    /// Taint-flow query.
    Taint {
        #[arg(required = true)]
        source: String,
        #[arg(required = true)]
        sink: String,
    },
    /// CG reachability from functions.
    Reachable {
        #[arg(required = true)]
        func_ids: Vec<String>,
    },
}
```

**Step 2: Implement query dispatch**

In `commands.rs`, implement `query()` that builds a `DriverConfig` (default settings), creates `AnalysisDriver::build()`, then dispatches to the appropriate `db` method:

- `PointsTo`: parse hex → `ValueId`, call `db.points_to(vid)`, print locations
- `Alias`: parse two hex IDs, call `db.may_alias(p, q)`, print result
- `Flows`/`Taint`: delegate to `db.handle_request()` with a JSON query (reuse the protocol)
- `Reachable`: parse hex → `FunctionId`, call `db.cg_reachable_from()`, print

Reference `crates/saf-core/src/ids.rs` for hex parsing of IDs (look for `from_hex` or equivalent).

**Step 3: Verify and commit**

Run: `make fmt && make lint`

```bash
git commit -m "feat(cli): implement saf query with points-to, alias, flows, taint, reachable"
```

---

### Task 2.4: Implement `saf schema`

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Add SchemaArgs**

```rust
#[derive(Args)]
pub struct SchemaArgs {
    /// List available checkers.
    #[arg(long)]
    pub checkers: bool,
    /// List available frontends.
    #[arg(long)]
    pub frontends: bool,
    /// Output format.
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Human)]
    pub format: CliOutputFormat,
}
```

Add `Schema(SchemaArgs)` variant to `Commands` enum (replacing the unit `Schema` variant).

**Step 2: Implement `schema()` command**

This is a static command — no analysis needed. Use:
- `CheckCatalog::new()` for checker list — reference `crates/saf-analysis/src/database/catalog.rs`
- `builtin_checker_names()` for checker names — reference `crates/saf-analysis/src/checkers/spec.rs:407`
- Frontend list is static: `["llvm", "air-json"]`
- Query types: `["points-to", "alias", "flows", "taint", "reachable"]`

For human output, print a formatted table. For JSON, serialize and print.

**Step 3: Verify and commit**

```bash
git commit -m "feat(cli): implement saf schema command"
```

---

## Phase 3: Extended Analyses

### Task 3.1: Wire up CSPTA, FSPTA, DDA in driver.rs

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Implement CSPTA path in `analyze()`**

When `config.pta_variant == PtaVariant::CsPta`:
```rust
use saf_analysis::cspta::{solve_context_sensitive, CsPtaConfig};

let cs_config = CsPtaConfig {
    k: config.pta_k,
    field_sensitivity: /* from config */,
    max_iterations: config.max_pta_iterations.unwrap_or(100_000),
    ..CsPtaConfig::default()
};
let cs_result = solve_context_sensitive(self.db.module(), self.db.call_graph(), &cs_config);
```

Include CS-PTA diagnostics in output.

**Step 2: Implement FSPTA path**

When `config.pta_variant == PtaVariant::FsPta`:
- Requires SVFG and MSSA — use `db.get_or_build_svfg()` for SVFG
- MSSA needs to be built separately. Reference `crates/saf-analysis/src/mssa/mod.rs` for `MemorySsa::build()`
- Call `solve_flow_sensitive()` from `saf_analysis::fspta`

Note: `ProgramDatabase` builds the core pipeline with Andersen by default. FSPTA is run _after_ the core pipeline as a refinement step, using the CI-PTA result as input.

**Step 3: Implement DDA path**

When `config.pta_variant == PtaVariant::Dda`:
- Create `DdaPta` instance, configure and run
- Reference `crates/saf-analysis/src/dda/mod.rs`

**Step 4: Verify and commit**

```bash
git commit -m "feat(cli): wire CSPTA, FSPTA, DDA analysis variants"
```

---

### Task 3.2: Wire up abstract interpretation (numeric checkers)

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Handle `--checkers numeric` and `--checkers all`**

When `CheckerSelection::All` or `Specific` contains `"numeric"`:
```rust
use saf_analysis::absint::{check_all_numeric, NumericFinding};

let numeric = check_all_numeric(self.db.module());
output.numeric_findings = numeric;
```

Also support individual numeric checker names: `"buffer_overflow"`, `"integer_overflow"`, `"division_by_zero"`.

Reference `crates/saf-analysis/src/absint/mod.rs` for all `check_*` functions.

**Step 2: Include numeric findings in output formatting**

In `format_output()`, add a section for numeric findings (separate from SVFG findings):
```
Numeric Findings (abstract interpretation):
  [warning] integer_overflow at func() file:line — message
```

**Step 3: Verify and commit**

```bash
git commit -m "feat(cli): wire abstract interpretation numeric checkers"
```

---

### Task 3.3: Wire up IFDS taint analysis

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Implement `--ifds-taint` flag handling**

When `config.ifds_taint_config` is set:
1. Read config YAML from the path
2. Build `TaintIfdsProblem` — reference `crates/saf-analysis/src/ifds/taint.rs`
3. Build ICFG from `db.icfg()`
4. Call `solve_ifds(&problem, &icfg, db.call_graph(), &IfdsConfig::default())`
5. Store result in `output.ifds_results`

Reference `crates/saf-python/src/ifds.rs:151` for the Python SDK pattern.

**Step 2: Add IFDS results to output formatting**

Print taint flows found by IFDS.

**Step 3: Verify and commit**

```bash
git commit -m "feat(cli): wire IFDS taint analysis"
```

---

### Task 3.4: Wire up typestate analysis

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Implement `--typestate` and `--typestate-custom` handling**

For `--typestate <spec-name>`:
```rust
use saf_analysis::ifds::{builtin_typestate_spec, TypestateIdeProblem, solve_ide, IfdsConfig};

let spec = builtin_typestate_spec(spec_name)
    .ok_or_else(|| anyhow!("Unknown typestate spec: '{spec_name}'"))?;
let problem = TypestateIdeProblem::new(&spec, db.module());
let result = solve_ide(&problem, db.icfg(), db.call_graph(), &IfdsConfig::default());
```

For `--typestate-custom <path.yaml>`: load YAML into `TypestateSpec`, same flow.

Reference `crates/saf-python/src/ide.rs:241-275` for the Python SDK pattern.

**Step 2: Add typestate findings to output**

**Step 3: Verify and commit**

```bash
git commit -m "feat(cli): wire typestate analysis (builtin + custom)"
```

---

### Task 3.5: Wire up Z3 features

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

Note: Z3 features require `feature = "z3-solver"`. Guard these with `#[cfg(feature = "z3-solver")]`.

**Step 1: Implement `--path-sensitive`**

When `config.path_sensitive`:
```rust
use saf_analysis::checkers::{run_checkers_path_sensitive, PathSensitiveConfig};

let ps_config = PathSensitiveConfig { /* z3_timeout, etc. */ };
let ps_result = run_checkers_path_sensitive(&specs, module, svfg, &table, &ps_config);
// Use ps_result.feasible as the finding set instead of the unfiltered results
```

**Step 2: Implement `--z3-prove`, `--z3-refine-alias`, `--z3-check-reachability`**

Stub these with `anyhow::bail!("Z3 feature requires z3-solver feature")` when `z3-solver` is not enabled. When enabled, delegate to the Z3 functions from `saf_analysis::z3_utils`.

Reference `crates/saf-python/src/z3_refine.rs` for the Python SDK patterns.

**Step 3: Verify and commit**

```bash
git commit -m "feat(cli): wire Z3 features (path-sensitive, prove, refine, reachability)"
```

---

### Task 3.6: Wire up combined analysis

**Files:**
- Modify: `crates/saf-cli/src/driver.rs`

**Step 1: Implement `--combined`**

When `config.combined`:
```rust
use saf_analysis::combined::{analyze_combined, CombinedAnalysisConfig};

let combined_config = CombinedAnalysisConfig {
    pta: /* from driver config */,
    absint: AbstractInterpConfig::default(),
    enable_refinement: true,
    max_refinement_iterations: 3,
    ..CombinedAnalysisConfig::default()
};
let result = analyze_combined(self.db.module(), &combined_config);
```

Print combined analysis results (PTA stats + absint results + summaries).

**Step 2: Verify and commit**

```bash
git commit -m "feat(cli): wire combined PTA + absint analysis"
```

---

## Phase 4: Incremental Execution + Help System

### Task 4.1: Wire up incremental analysis execution

**Files:**
- Modify: `crates/saf-cli/src/commands.rs`

**Step 1: Replace the incremental execution stub**

The current `incremental()` function in `commands.rs:299-323` bails with "not yet implemented: incremental analysis execution". Replace with actual execution using:

```rust
use saf_analysis::pipeline::run_pipeline_incremental;
use saf_analysis::session::AnalysisSession;
use saf_core::program::AirProgram;
```

The implementation should:
1. Ingest files via `LlvmFrontend::ingest()` / `AirJsonFrontend`
2. Build `AirProgram` from modules
3. Load or create `AnalysisSession` from cache dir
4. Call `run_pipeline_incremental()` with session + program + config
5. Save updated session
6. Print results (stats, findings)

Reference `crates/saf-python/src/project.rs:1137` for the Python SDK's `analyze()` method which does exactly this.

**Step 2: Verify and commit**

```bash
git commit -m "feat(cli): wire up incremental analysis execution"
```

---

### Task 4.2: Create help.rs with all 11 topics

**Files:**
- Create: `crates/saf-cli/src/help.rs`
- Modify: `crates/saf-cli/src/commands.rs` (add `Help` command)
- Modify: `crates/saf-cli/src/main.rs` (add `mod help;`)

**Step 1: Add Help command to commands.rs**

```rust
#[derive(Args)]
pub struct HelpArgs {
    /// Help topic.
    pub topic: Option<String>,
}

// In Commands enum:
/// Get detailed help on a topic.
Help(HelpArgs),
```

Add dispatch in `main.rs`:
```rust
Commands::Help(args) => help::print_help(args.topic.as_deref()),
```

**Step 2: Create help.rs with all topics**

Create `crates/saf-cli/src/help.rs` with:

```rust
pub fn print_help(topic: Option<&str>) -> anyhow::Result<()> {
    match topic {
        None                => print_overview(),
        Some("run")         => print_run_guide(),
        Some("checkers")    => print_checkers_guide(),
        Some("pta")         => print_pta_guide(),
        Some("typestate")   => print_typestate_guide(),
        Some("taint")       => print_taint_guide(),
        Some("z3")          => print_z3_guide(),
        Some("export")      => print_export_guide(),
        Some("specs")       => print_specs_guide(),
        Some("incremental") => print_incremental_guide(),
        Some("examples")    => print_examples(),
        Some(other)         => anyhow::bail!("Unknown help topic: '{other}'\n\nRun 'saf help' for available topics."),
    }
}
```

Each topic function prints static content using `println!()`. Content as specified in the design doc:

- **Overview**: list all commands + topics
- **Checkers**: 9 builtin checkers with names, CWE IDs, descriptions, example commands, custom checker format
- **PTA**: Andersen/CSPTA/FSPTA/DDA trade-offs, solver backends (worklist/datalog), PTS representations (auto/btreeset/fxhash/roaring/bdd), field sensitivity
- **Typestate**: what typestate is, builtin specs, custom YAML format, example
- **Taint**: VF taint vs IFDS taint, selector format, IFDS config, example
- **Z3**: assertion proving, alias refinement, path reachability, timeout tuning
- **Export**: all targets, all formats, examples
- **Specs**: YAML format, discovery paths, how specs affect PTA + checkers
- **Incremental**: cache dir, change detection, invalidation, `--plan`, summary export
- **Examples**: common usage patterns (quick scan, CI, taint, custom checker, incremental, serve)

Reference existing checker names from `crates/saf-analysis/src/checkers/spec.rs:385-407`.
Reference CWE IDs from the `CheckerSpec` definitions in `spec.rs`.

**Step 3: Verify and commit**

Run: `make fmt && make lint`

Test: `docker compose run --rm dev sh -c 'cargo run -p saf-cli -- help checkers'`

```bash
git commit -m "feat(cli): add help command with 11 guide topics"
```

---

## Phase 5: Testing + Polish

### Task 5.1: Add CLI smoke tests

**Files:**
- Modify: `crates/saf-cli/tests/smoke.rs` (or create if not exists)

**Step 1: Check existing smoke test**

Read `crates/saf-cli/tests/smoke.rs` to see existing test patterns. The crate has `assert_cmd` and `predicates` as dev-dependencies.

**Step 2: Add smoke tests for each command**

Using `assert_cmd`:
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_overview() {
    Command::cargo_bin("saf").unwrap()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available commands"));
}

#[test]
fn help_checkers() {
    Command::cargo_bin("saf").unwrap()
        .args(["help", "checkers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("memory-leak"));
}

#[test]
fn schema_human() {
    Command::cargo_bin("saf").unwrap()
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("Checkers"));
}

#[test]
fn schema_json() {
    Command::cargo_bin("saf").unwrap()
        .args(["schema", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"));
}
```

Note: Tests that require LLVM (run, index, export, query) can only run inside Docker. These tests should be `#[ignore]` with a note, or use AIR-JSON fixtures instead. Check if any AIR-JSON test fixtures exist at `tests/fixtures/` that can be used without LLVM.

**Step 3: Verify**

Run: `make test`

**Step 4: Commit**

```bash
git commit -m "test(cli): add smoke tests for help, schema, and basic commands"
```

---

### Task 5.2: Update PROGRESS.md

**Files:**
- Modify: `plans/PROGRESS.md`

**Step 1: Add plan 167 to Plans Index**

Add row to the Plans Index table:
```
| 167 | cli-feature-parity | cli | done | Notes: ... |
```

**Step 2: Update Next Steps**

Remove the plan 166 follow-up bullet about CLI wiring (it's now done). Add any remaining follow-ups.

**Step 3: Add Session Log entry**

Append to Session Log with summary of work done.

**Step 4: Commit**

```bash
git commit -m "docs: update PROGRESS.md with plan 167 completion"
```

---

## Summary

| Phase | Tasks | What it delivers |
|-------|-------|-----------------|
| 1 | 1.1-1.3 | Dependencies + driver skeleton + `saf run` (pipeline + checkers + serve) |
| 2 | 2.1-2.4 | `saf index`, `saf export`, `saf query`, `saf schema` |
| 3 | 3.1-3.6 | CSPTA, FSPTA, DDA, absint, IFDS taint, typestate, Z3, combined |
| 4 | 4.1-4.2 | Incremental execution + help system (11 topics) |
| 5 | 5.1-5.2 | Smoke tests + PROGRESS.md update |

Total: **17 tasks** across 5 phases. Each task is independently committable. Phase 1 delivers a usable `saf run` command. Later phases add breadth.
