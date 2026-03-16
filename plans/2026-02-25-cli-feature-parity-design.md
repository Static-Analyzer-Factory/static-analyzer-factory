# SAF CLI Feature Parity Design

**Date:** 2026-02-25
**Goal:** Wire up all saf-analysis features into saf-cli, achieving feature parity with the Python SDK.

## Problem

The `saf-cli` binary has 7 subcommands, but only 2 work (`specs`, partial `incremental`). The remaining 5 (`index`, `run`, `query`, `export`, `schema`) bail with "not yet implemented". Meanwhile, the Python SDK (`saf-python`) exposes the full analysis stack: pipeline execution, 9 builtin checkers, CSPTA, FSPTA, DDA, IFDS taint, abstract interpretation, Z3-based proving/refinement, typestate analysis, graph export, and JSON protocol. The CLI needs all of these.

The root cause is simple: `saf-cli/Cargo.toml` only depends on `saf-core`. It lacks dependencies on `saf-frontends` (for ingestion) and `saf-analysis` (for the pipeline and all analyses).

## Architecture

### New files

```
crates/saf-cli/src/
  main.rs        # (existing) CLI entry point
  commands.rs    # (existing) arg structs + command dispatch — extended with new commands
  driver.rs      # (new) analysis driver wrapping ProgramDatabase
  help.rs        # (new) detailed help guides by topic
```

### Dependency changes

```toml
# saf-cli/Cargo.toml — add:
saf-frontends = { workspace = true }
saf-analysis = { workspace = true }
```

### Data flow

```
CLI args ──→ DriverConfig
                │
Input files ──→ AnalysisDriver::build()
                │  1. Frontend::ingest() → AirBundle
                │  2. PipelineConfig from DriverConfig
                │  3. ProgramDatabase::build()
                │
                ▼
         AnalysisDriver
         ├── db: ProgramDatabase        (core pipeline results)
         ├── module: Arc<AirModule>     (shared with db)
         └── specs: SpecRegistry
                │
                ├── .run_checkers()     → Vec<CheckerFinding>
                ├── .run_cspta(k)       → CsPtaResult
                ├── .run_fspta()        → FsPtaResult
                ├── .run_absint()       → Vec<NumericFinding>
                ├── .run_ifds_taint()   → IfdsResult
                ├── .run_typestate()    → TypestateResult
                ├── .run_z3_prove()     → Vec<AssertionResult>
                ├── .run_z3_refine()    → RefinedAliasResult
                ├── .run_z3_reach()     → ReachabilityResult
                ├── .export_graph()     → PropertyGraph
                ├── .query_*()          → query results
                ├── .serve()            → stdin/stdout JSON protocol loop
                └── .schema()           → Schema
```

## Command Structure

### `saf run <files>` — Primary analysis command

The main command: ingests files, runs the pipeline, executes checkers, prints results.

```
saf run <files...>
    # Input
    --frontend <llvm|air-json>              default: llvm
    --mode <fast|precise>                   default: precise
    --specs <path>                          additional spec files/dirs

    # PTA variant
    --pta <andersen|cspta|fspta|dda>        default: andersen
    --solver <worklist|datalog>             default: worklist
    --pts-repr <auto|btreeset|fxhash|roaring|bdd>  default: auto
    --pta-k <N>                             k-CFA depth (cspta only, default: 2)
    --field-sensitivity <none|struct-fields> default: struct-fields
    --max-pta-iterations <N>                iteration cap

    # Checkers
    --checkers <list>                       all | none | comma-separated names
    --path-sensitive                        Z3-based infeasible path filtering
    --typestate <spec-name>                 builtin typestate spec
    --typestate-custom <path.yaml>          custom typestate spec

    # Z3 features
    --z3-prove                              prove assertions via Z3
    --z3-refine-alias                       refine alias results via Z3
    --z3-check-reachability                 check path reachability via Z3
    --z3-timeout <ms>                       Z3 solver timeout (default: 5000)

    # Combined analysis
    --combined                              PTA + abstract interpretation

    # Taint
    --ifds-taint <config.yaml>              IFDS taint analysis with config

    # Output
    --format <human|json|sarif>             default: human
    --output <path>                         write to file instead of stdout
    --diagnostics                           include checker/PTA diagnostics
    --verbose                               timing, resource table, stats

    # Server mode
    --serve                                 JSON protocol server (stdin/stdout)
```

Default behavior (no flags): runs full pipeline with Andersen PTA, all builtin checkers, human-readable output showing timing stats and findings.

### `saf index <files>` — Ingest only

Runs frontend ingestion without analysis. Outputs AIR-JSON.

```
saf index <files...>
    --frontend <llvm|air-json>
    --output <path>                         write AIR-JSON (default: stdout)
```

### `saf query <type>` — Query analysis results

Runs specific queries against an analyzed program.

```
saf query points-to <pointer-id>
saf query alias <id-a> <id-b>
saf query flows <source> <sink>
saf query taint <source> <sink>
saf query reachable <func-ids...>

    --input <files...>                      analyze these files first
    --cache <dir>                           use cached incremental session
```

### `saf export <target>` — Export graphs/findings

Exports specific analysis artifacts.

```
saf export cfg
saf export callgraph
saf export defuse
saf export valueflow
saf export svfg
saf export findings
saf export pta

    --format <json|sarif|dot|html>          default: json (sarif for findings only)
    --output <path>
    --input <files...>                      analyze these files first
    --function <id>                         scope to function (cfg only)
```

### `saf schema` — Print analysis schema

```
saf schema
    --checkers                              list available checkers
    --frontends                             list available frontends
    --format <human|json>
```

### `saf specs` — Manage function specifications (already implemented)

### `saf incremental <files>` — Incremental analysis

Existing `--plan`, `--clean`, `--export-summaries` flags stay. The stub execution path gets wired up to actually run `run_pipeline_incremental()`.

### `saf help <topic>` — Detailed help guides

```
saf help                                    overview of all commands
saf help run                                full pipeline walkthrough
saf help checkers                           builtin checkers with CWE IDs
saf help pta                                PTA variants and trade-offs
saf help typestate                          typestate analysis guide
saf help taint                              taint analysis (VF vs IFDS)
saf help z3                                 Z3 features guide
saf help export                             export targets and formats
saf help specs                              spec file format
saf help incremental                        incremental workflow
saf help examples                           common usage patterns
```

## Feature Parity Matrix

Complete mapping of Python SDK features to CLI:

| # | Feature | Python SDK | CLI Surface |
|---|---------|-----------|------------|
| 1 | Full pipeline | `Project.open()` | `saf run <files>` |
| 2 | Incremental pipeline | `Project.analyze()` | `saf incremental <files>` |
| 3 | Graph export (CFG, CG, defuse, VF) | `Project.graphs()` | `saf export <target>` |
| 4 | SARIF export | `export_findings_sarif()` | `saf export findings --format sarif` |
| 5 | Points-to / alias queries | `Project.query()` | `saf query points-to/alias` |
| 6 | Builtin checkers (9 types) | `Project.check_all()` | `saf run` (default) |
| 7 | Custom checker specs | `Project.check("name")` | `saf run --checkers custom:path.yaml` |
| 8 | Path-sensitive filtering (Z3) | `run_check_path_sensitive_with_svfg()` | `saf run --path-sensitive` |
| 9 | Abstract interp / numeric | `Project.check_all_numeric()` | `saf run --checkers numeric` |
| 10 | IFDS taint analysis | `run_ifds_taint()` | `saf run --ifds-taint config.yaml` |
| 11 | Context-sensitive PTA (k-CFA) | `Project.context_sensitive_pta(k)` | `saf run --pta cspta --pta-k 2` |
| 12 | Flow-sensitive PTA | `Project.flow_sensitive_pta()` | `saf run --pta fspta` |
| 13 | Demand-driven analysis (DDA) | `saf_analysis::dda` | `saf run --pta dda` |
| 14 | SVFG export | `Project.svfg()` | `saf export svfg` |
| 15 | CG refinement | `Project.refine_call_graph()` | via pipeline (always runs) |
| 16 | Z3 assertion proving | `run_prove_assertions()` | `saf run --z3-prove` |
| 17 | Z3 alias refinement | `run_refine_alias()` | `saf run --z3-refine-alias` |
| 18 | Z3 path reachability | `run_check_path_reachable()` | `saf run --z3-check-reachability` |
| 19 | Typestate analysis | `Project.typestate(spec)` | `saf run --typestate <spec>` |
| 20 | Combined analysis (PTA + absint) | `Project.analyze_combined()` | `saf run --combined` |
| 21 | Schema / catalog | `Project.schema()` | `saf schema` |
| 22 | JSON protocol server | `Project.request(json)` | `saf run --serve` |
| 23 | Function specs | `SpecRegistry` | `saf specs` (existing) |
| 24 | Summary export | via incremental | `saf incremental --export-summaries` (existing) |
| 25 | Resource table | `Project.resource_table()` | `saf run --verbose` |
| 26 | Checker diagnostics | `Project.checker_diagnostics()` | `saf run --diagnostics` |
| 27 | PTA solver selection | `Project.open(pta_solver=...)` | `saf run --solver worklist/datalog` |
| 28 | PTS representation | `Project.open(pts_repr=...)` | `saf run --pts-repr auto/btreeset/...` |

## `driver.rs` Design

### Types

```rust
/// Configuration for the analysis driver, built from CLI args.
pub struct DriverConfig {
    // Input
    pub frontend: CliFrontend,
    pub mode: AnalysisMode,

    // PTA
    pub pta_variant: PtaVariant,
    pub solver: PtaSolverKind,
    pub pts_repr: PtsRepresentation,
    pub field_sensitivity: FieldSensitivity,
    pub max_pta_iterations: Option<usize>,
    pub pta_k: u32,

    // Specs
    pub specs_path: Option<PathBuf>,

    // Checkers
    pub checkers: CheckerSelection,
    pub path_sensitive: bool,

    // Extended analyses
    pub combined: bool,
    pub ifds_taint_config: Option<PathBuf>,
    pub typestate: Option<TypestateSpec>,

    // Z3
    pub z3: Z3Config,

    // Output
    pub output: OutputConfig,
}

pub enum PtaVariant { Andersen, CsPta, FsPta, Dda }
pub enum PtaSolverKind { Worklist, Datalog }
pub enum CheckerSelection { All, None, Specific(Vec<String>) }

pub struct Z3Config {
    pub prove: bool,
    pub refine_alias: bool,
    pub check_reachability: bool,
    pub timeout_ms: u64,
}

pub struct OutputConfig {
    pub format: OutputFormat,
    pub path: Option<PathBuf>,
    pub verbose: bool,
    pub diagnostics: bool,
}

/// Collected analysis results from a single run.
pub struct AnalysisOutput {
    pub stats: PipelineStats,
    pub checker_findings: Vec<CheckerFinding>,
    pub numeric_findings: Vec<NumericFinding>,
    pub z3_results: Option<Z3Results>,
    pub ifds_results: Option<IfdsResult>,
    pub typestate_results: Option<TypestateResult>,
    pub diagnostics: Option<Diagnostics>,
}

/// The analysis driver.
pub struct AnalysisDriver {
    db: ProgramDatabase,
    module: Arc<AirModule>,
    specs: SpecRegistry,
}
```

### Methods

```rust
impl AnalysisDriver {
    /// Ingest files via the selected frontend.
    pub fn ingest(inputs: &[PathBuf], frontend: CliFrontend) -> Result<AirBundle>;

    /// Ingest + run pipeline → build driver.
    pub fn build(inputs: &[PathBuf], config: &DriverConfig) -> Result<Self>;

    /// Run all requested analyses based on config flags.
    pub fn analyze(&self, config: &DriverConfig) -> Result<AnalysisOutput>;

    /// Format and write results to stdout/file.
    pub fn format_output(&self, output: &AnalysisOutput, config: &OutputConfig) -> Result<()>;

    /// Export a specific graph as PropertyGraph.
    pub fn export_graph(&self, target: ExportTarget) -> Result<PropertyGraph>;

    /// Run JSON protocol server on stdin/stdout.
    pub fn serve(&self) -> Result<()>;

    /// Print analysis schema.
    pub fn print_schema(format: OutputFormat) -> Result<()>;

    // Query methods delegate to ProgramDatabase
    pub fn query_points_to(&self, pointer: ValueId) -> Vec<LocId>;
    pub fn query_alias(&self, p: ValueId, q: ValueId) -> AliasResult;
}
```

### Command wiring (in `commands.rs`)

Each command function is thin — builds a `DriverConfig` from CLI args, then calls driver methods:

```rust
pub fn run(args: &RunArgs) -> Result<()> {
    let config = DriverConfig::from(args);
    if args.serve {
        let driver = AnalysisDriver::build(&args.inputs, &config)?;
        return driver.serve();
    }
    let driver = AnalysisDriver::build(&args.inputs, &config)?;
    let output = driver.analyze(&config)?;
    driver.format_output(&output, &config.output)
}

pub fn index(args: &IndexArgs) -> Result<()> {
    let bundle = AnalysisDriver::ingest(&args.inputs, args.frontend)?;
    let json = serde_json::to_string_pretty(&bundle)?;
    // write to args.output or stdout
}

pub fn export(args: &ExportArgs) -> Result<()> {
    let config = DriverConfig::from(args);
    let driver = AnalysisDriver::build(&args.inputs, &config)?;
    let graph = driver.export_graph(args.target)?;
    // serialize in requested format
}

pub fn query(args: &QueryArgs) -> Result<()> {
    let config = DriverConfig::from(args);
    let driver = AnalysisDriver::build(&args.inputs, &config)?;
    match args.command {
        QueryCommand::PointsTo { id } => { /* driver.query_points_to() */ }
        QueryCommand::Alias { p, q } => { /* driver.query_alias() */ }
        // ...
    }
}

pub fn schema(args: &SchemaArgs) -> Result<()> {
    AnalysisDriver::print_schema(args.format)
}
```

## `help.rs` Design

Static content organized by topic. Each topic function prints a formatted guide.

```rust
pub fn print_help(topic: Option<&str>) -> Result<()> {
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
        Some(other)         => bail!("Unknown topic '{other}'. Run 'saf help' for topics."),
    }
}
```

### Topics

| Topic | Covers |
|-------|--------|
| (none) | All commands with one-line descriptions, available help topics |
| `run` | Pipeline stages, flag groups, default behavior |
| `checkers` | 9 builtin checkers with CWE IDs, custom checker YAML format, path-sensitive filtering |
| `pta` | Andersen vs CSPTA vs FSPTA vs DDA trade-offs, solver backends, PTS representations, field sensitivity |
| `typestate` | What typestate is, builtin specs, custom spec format, example workflow |
| `taint` | Value-flow taint vs IFDS taint trade-offs, selector format, IFDS config |
| `z3` | Assertion proving, alias refinement, path reachability, timeout tuning |
| `export` | All targets (cfg, callgraph, defuse, valueflow, svfg, findings, pta), formats (JSON, SARIF, DOT, HTML) |
| `specs` | YAML format, discovery paths, how specs affect PTA + checkers |
| `incremental` | Cache directory, change detection, invalidation, `--plan`, summary export |
| `examples` | Quick scan, CI integration, taint analysis, custom checker, incremental, server mode |

## Reuse from Existing Code

- **Frontend ingestion**: Same `LlvmFrontend::ingest()` / `AirJsonFrontend::ingest()` pattern as `saf-bench/main.rs:load_bitcode()`
- **Core pipeline**: `ProgramDatabase::build()` (wraps `run_pipeline()`)
- **Checkers**: `ProgramDatabase::run_svfg_check()` and `handle_check_all()` already run all 9 builtin checkers
- **Graph export**: `ProgramDatabase::export_graphs()` already produces PropertyGraphs
- **JSON protocol**: `ProgramDatabase::handle_request()` already handles the full protocol
- **Schema**: `ProgramDatabase::schema()` already returns the discovery schema
- **SVFG**: `ProgramDatabase::get_or_build_svfg()` handles lazy SVFG construction
- **Incremental pipeline**: `run_pipeline_incremental()` + `AnalysisSession` already implemented

Extended analyses (CSPTA, FSPTA, absint, IFDS, Z3, typestate) follow patterns from `saf-python/src/project.rs` methods.
