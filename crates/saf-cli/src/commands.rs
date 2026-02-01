/// CLI command definitions for SAF (FR-CLI-001, FR-CLI-002).
use std::fmt;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use saf_core::manifest::CacheManifest;
use saf_core::spec::{SpecFile, SpecRegistry};
use saf_core::summary::FunctionSummary;

use crate::driver;

// ---------------------------------------------------------------------------
// CLI-local enums (thin wrappers around saf-core types, keeping clap out of core)
// ---------------------------------------------------------------------------

/// Frontend selection for CLI ingestion.
///
/// Wraps [`saf_core::config::Frontend`] with `ValueEnum` support.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFrontend {
    /// LLVM bitcode / IR frontend.
    Llvm,
    /// AIR-JSON frontend.
    #[value(name = "air-json")]
    AirJson,
}

impl fmt::Display for CliFrontend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliFrontend> for saf_core::config::Frontend {
    fn from(v: CliFrontend) -> Self {
        match v {
            CliFrontend::Llvm => Self::Llvm,
            CliFrontend::AirJson => Self::AirJson,
        }
    }
}

/// Analysis mode selection for the CLI.
///
/// Wraps [`saf_core::config::AnalysisMode`] with `ValueEnum` support.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliAnalysisMode {
    /// Fast mode: fewer iterations, less precision.
    Fast,
    /// Precise mode: full fixed-point iteration.
    Precise,
}

impl fmt::Display for CliAnalysisMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliAnalysisMode> for saf_core::config::AnalysisMode {
    fn from(v: CliAnalysisMode) -> Self {
        match v {
            CliAnalysisMode::Fast => Self::Fast,
            CliAnalysisMode::Precise => Self::Precise,
        }
    }
}

/// PTA algorithm variant.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliPtaVariant {
    /// Andersen's inclusion-based analysis.
    Andersen,
    /// Context-sensitive PTA (k-CFA).
    Cspta,
    /// Flow-sensitive PTA.
    Fspta,
    /// Demand-driven alias analysis.
    Dda,
}

impl fmt::Display for CliPtaVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliPtaVariant> for driver::PtaVariant {
    fn from(v: CliPtaVariant) -> Self {
        match v {
            CliPtaVariant::Andersen => Self::Andersen,
            CliPtaVariant::Cspta => Self::CsPta,
            CliPtaVariant::Fspta => Self::FsPta,
            CliPtaVariant::Dda => Self::Dda,
        }
    }
}

/// PTA solver backend.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliPtaSolver {
    /// Worklist-based imperative solver.
    Worklist,
    /// Datalog fixpoint solver (Ascent).
    Datalog,
}

impl fmt::Display for CliPtaSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliPtaSolver> for driver::PtaSolverKind {
    fn from(v: CliPtaSolver) -> Self {
        match v {
            CliPtaSolver::Worklist => Self::Worklist,
            CliPtaSolver::Datalog => Self::Datalog,
        }
    }
}

/// Points-to set representation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliPtsRepr {
    /// Auto-select based on program size.
    Auto,
    /// `BTreeSet` baseline.
    Btreeset,
    /// `FxHashSet` for fast operations.
    Fxhash,
    /// Roaring bitmap.
    Roaring,
    /// Binary Decision Diagram.
    Bdd,
}

impl fmt::Display for CliPtsRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliPtsRepr> for driver::PtsRepr {
    fn from(v: CliPtsRepr) -> Self {
        match v {
            CliPtsRepr::Auto => Self::Auto,
            CliPtsRepr::Btreeset => Self::BTreeSet,
            CliPtsRepr::Fxhash => Self::FxHash,
            CliPtsRepr::Roaring => Self::Roaring,
            CliPtsRepr::Bdd => Self::Bdd,
        }
    }
}

/// Field sensitivity level.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliFieldSensitivity {
    /// Track struct fields (depth 2).
    #[value(name = "struct-fields")]
    StructFields,
    /// Track array indices.
    #[value(name = "array-index")]
    ArrayIndex,
    /// No field sensitivity.
    Flat,
}

impl fmt::Display for CliFieldSensitivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliFieldSensitivity> for driver::CliFieldSensitivityKind {
    fn from(v: CliFieldSensitivity) -> Self {
        match v {
            CliFieldSensitivity::StructFields => Self::StructFields,
            CliFieldSensitivity::ArrayIndex => Self::ArrayIndex,
            CliFieldSensitivity::Flat => Self::Flat,
        }
    }
}

/// Output format for analysis results.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliOutputFormat {
    /// Human-readable text output.
    Human,
    /// JSON output.
    Json,
    /// SARIF 2.1.0 output.
    Sarif,
}

impl fmt::Display for CliOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl From<CliOutputFormat> for driver::OutputFormat {
    fn from(v: CliOutputFormat) -> Self {
        match v {
            CliOutputFormat::Human => Self::Human,
            CliOutputFormat::Json => Self::Json,
            CliOutputFormat::Sarif => Self::Sarif,
        }
    }
}

/// Incremental analysis precision mode.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum IncrementalMode {
    /// Sound mode: conservative over-approximation, no missed behaviors.
    Sound,
    /// Best-effort mode: faster, may miss some behaviors.
    #[value(name = "best-effort")]
    BestEffort,
}

impl fmt::Display for IncrementalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

/// Export target (graph or artifact).
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliExportTarget {
    /// Control-flow graph.
    Cfg,
    /// Call graph.
    Callgraph,
    /// Def-use graph.
    Defuse,
    /// Value-flow graph.
    Valueflow,
    /// SVFG (Sparse Value-Flow Graph).
    Svfg,
    /// Analysis findings.
    Findings,
    /// Points-to analysis results.
    Pta,
}

impl fmt::Display for CliExportTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

/// Export output format for the `export` subcommand.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliExportFormat {
    /// JSON format.
    Json,
    /// SARIF format (findings only).
    Sarif,
    /// Graphviz DOT format.
    Dot,
    /// Interactive HTML visualization.
    Html,
}

impl fmt::Display for CliExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

// ---------------------------------------------------------------------------
// CLI struct definitions
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "saf", version, about = "Static Analyzer Factory")]
pub struct Cli {
    /// Output errors as JSON (NFR-OBS-001).
    #[arg(long)]
    pub json_errors: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum Commands {
    /// Index input files via a frontend to produce AIR.
    Index(IndexArgs),
    /// Run analysis passes on indexed AIR.
    Run(RunArgs),
    /// Query analysis results.
    Query(QueryArgs),
    /// Export graphs or findings.
    Export(ExportArgs),
    /// Print the SAF schema (supported frontends, queries, checkers).
    Schema(SchemaArgs),
    /// Manage function specifications.
    Specs(SpecsArgs),
    /// Run incremental analysis on one or more input files.
    Incremental(IncrementalArgs),
    /// Show help for a topic (run, checkers, pta, typestate, taint, z3, export, specs, incremental, examples).
    Help(HelpArgs),
}

#[derive(Args)]
pub struct IndexArgs {
    /// Input files to index.
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Frontend to use for ingestion.
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,

    /// Write AIR-JSON output to file instead of stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

// NOTE: CLI arg structs naturally accumulate bool flags for feature toggles.
// Splitting into substructs would hurt CLI ergonomics.
#[allow(clippy::struct_excessive_bools)]
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

    /// k-CFA depth (`cspta` only).
    #[arg(long, default_value_t = 2)]
    pub pta_k: u32,

    /// Field sensitivity level.
    #[arg(long, value_enum, default_value_t = CliFieldSensitivity::StructFields)]
    pub field_sensitivity: CliFieldSensitivity,

    /// Maximum PTA iterations.
    #[arg(long)]
    pub max_pta_iterations: Option<usize>,

    /// Checkers to run (comma-separated, or "all" / "none").
    #[arg(long, default_value = "all")]
    pub checkers: String,

    /// Enable Z3 path-sensitive checker filtering.
    #[arg(long)]
    pub path_sensitive: bool,

    /// Run built-in typestate spec.
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

    /// Path to bench-config JSON file (benchmark mode).
    /// When set, reads analysis queries and configuration from this file
    /// and writes structured results to --output.
    #[arg(long)]
    pub bench_config: Option<PathBuf>,
}

#[derive(Args)]
pub struct QueryArgs {
    /// The query to execute.
    #[command(subcommand)]
    pub command: QueryCommand,

    /// Input files to analyze.
    #[arg(long, required = true)]
    pub input: Vec<PathBuf>,

    /// Frontend to use for ingestion.
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,
}

/// Available query commands.
#[derive(Subcommand)]
pub enum QueryCommand {
    /// Points-to set for a value.
    #[command(name = "points-to")]
    PointsTo {
        /// Value ID (hex, e.g. `0x00ab...`).
        #[arg(required = true)]
        pointer: String,
    },
    /// May-alias check between two pointers.
    Alias {
        /// First pointer value ID (hex).
        #[arg(required = true)]
        p: String,
        /// Second pointer value ID (hex).
        #[arg(required = true)]
        q: String,
    },
    /// Data-flow reachability.
    Flows {
        /// Source value ID (hex).
        #[arg(required = true)]
        source: String,
        /// Sink value ID (hex).
        #[arg(required = true)]
        sink: String,
    },
    /// Taint-flow query.
    Taint {
        /// Source value ID (hex).
        #[arg(required = true)]
        source: String,
        /// Sink value ID (hex).
        #[arg(required = true)]
        sink: String,
    },
    /// CG reachability from functions.
    Reachable {
        /// Function IDs (hex).
        #[arg(required = true)]
        func_ids: Vec<String>,
    },
}

#[derive(Args)]
pub struct ExportArgs {
    /// Graph or artifact to export.
    #[arg(required = true, value_enum)]
    pub target: CliExportTarget,

    /// Output format.
    #[arg(long, value_enum, default_value_t = CliExportFormat::Json)]
    pub format: CliExportFormat,

    /// Write output to file instead of stdout.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Input files to analyze.
    #[arg(long, required = true)]
    pub input: Vec<PathBuf>,

    /// Filter to a specific function (for CFG export).
    #[arg(long)]
    pub function: Option<String>,

    /// Frontend to use for ingestion.
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,
}

#[derive(Args)]
pub struct IncrementalArgs {
    /// Input files to analyze.
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Frontend to use for ingestion.
    #[arg(long, value_enum, default_value_t = CliFrontend::Llvm)]
    pub frontend: CliFrontend,

    /// Precision mode for incremental analysis.
    #[arg(long, value_enum, default_value_t = IncrementalMode::BestEffort)]
    pub mode: IncrementalMode,

    /// Cache directory for incremental state.
    #[arg(long, default_value = ".saf-cache")]
    pub cache_dir: PathBuf,

    /// Dry-run: show what would be recomputed without running analysis.
    #[arg(long)]
    pub plan: bool,

    /// Clear the cache before analysis.
    #[arg(long)]
    pub clean: bool,

    /// Export computed summaries as YAML to the given path.
    #[arg(long)]
    pub export_summaries: Option<PathBuf>,
}

#[derive(Args)]
pub struct HelpArgs {
    /// Help topic to display (e.g., run, checkers, pta, typestate, taint, z3, export, specs, incremental, examples).
    pub topic: Option<String>,
}

#[derive(Args)]
pub struct SchemaArgs {
    /// List available checkers only.
    #[arg(long)]
    pub checkers: bool,

    /// List available frontends only.
    #[arg(long)]
    pub frontends: bool,

    /// Output format.
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Human)]
    pub format: CliOutputFormat,
}

#[derive(Args)]
pub struct SpecsArgs {
    #[command(subcommand)]
    pub command: SpecsCommand,
}

#[derive(Subcommand)]
pub enum SpecsCommand {
    /// List loaded function specifications.
    List {
        /// Show detailed information for each spec.
        #[arg(long)]
        verbose: bool,
    },
    /// Validate spec files.
    Validate {
        /// Path to spec file or directory to validate.
        #[arg(required = true)]
        path: String,
    },
    /// Look up the spec for a function.
    Lookup {
        /// Function name to look up.
        #[arg(required = true)]
        name: String,
    },
}

/// Build a `RunArgs` with defaults for commands that need a `DriverConfig`
/// but don't expose all `saf run` options (e.g., `query`, `export`).
fn default_run_args(inputs: &[PathBuf], frontend: CliFrontend) -> RunArgs {
    RunArgs {
        inputs: inputs.to_vec(),
        frontend,
        mode: CliAnalysisMode::Precise,
        specs: None,
        pta: CliPtaVariant::Andersen,
        solver: CliPtaSolver::Worklist,
        pts_repr: CliPtsRepr::Auto,
        pta_k: 2,
        field_sensitivity: CliFieldSensitivity::StructFields,
        max_pta_iterations: None,
        checkers: "none".to_string(),
        path_sensitive: false,
        typestate: None,
        typestate_custom: None,
        z3_prove: false,
        z3_refine_alias: false,
        z3_check_reachability: false,
        z3_timeout: 5000,
        combined: false,
        ifds_taint: None,
        format: CliOutputFormat::Human,
        output: None,
        diagnostics: false,
        verbose: false,
        serve: false,
        bench_config: None,
    }
}

/// Run `saf index` — ingest input files and emit AIR-JSON.
pub fn index(args: &IndexArgs) -> anyhow::Result<()> {
    use crate::driver::AnalysisDriver;

    let bundle = AnalysisDriver::ingest(&args.inputs, args.frontend)?;
    let json = serde_json::to_string_pretty(&bundle)?;

    if let Some(ref path) = args.output {
        std::fs::write(path, &json)?;
        eprintln!("Wrote AIR-JSON to {}", path.display());
    } else {
        println!("{json}");
    }
    Ok(())
}

pub fn run(args: &RunArgs) -> anyhow::Result<()> {
    use crate::driver::{AnalysisDriver, DriverConfig};
    use anyhow::Context;

    let mut config = DriverConfig::from_run_args(args);

    // Bench mode: parse config first, apply PTA overrides, then build driver
    if let Some(ref bench_config_path) = config.bench_config.clone() {
        let bench_config: saf_cli::bench_types::BenchConfig = {
            let data = std::fs::read_to_string(bench_config_path).with_context(|| {
                format!(
                    "Failed to read bench config: {}",
                    bench_config_path.display()
                )
            })?;
            serde_json::from_str(&data).with_context(|| "Failed to parse bench config JSON")?
        };

        // Apply BenchPtaConfig overrides to DriverConfig before building
        let bench_pta = &bench_config.pta_config;
        config.bench_field_depth = Some(bench_pta.field_depth);
        config.bench_constant_indices = bench_pta.constant_indices;
        config.bench_z3_index = bench_pta.z3_index_refinement;
        config.max_pta_iterations = Some(bench_pta.max_iterations);
        config.bench_refinement_iters = Some(bench_pta.refinement_max_iterations);

        let build_start = std::time::Instant::now();
        let mut driver = AnalysisDriver::build(&args.inputs, &config, args.frontend)?;
        let build_secs = build_start.elapsed().as_secs_f64();
        // Register PTABen wrapper functions in resource table so the SVFG
        // checker's `filter_wrapper_internal_sources` works correctly.
        if bench_config.analyses.ptaben_wrappers {
            driver.register_ptaben_wrappers();
        }
        let mut result = driver.run_bench_mode(&bench_config)?;
        // frontend_secs = build time minus pipeline time (pipeline is measured separately)
        result.stats.frontend_secs = (build_secs - result.stats.total_secs).max(0.0);
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
    AnalysisDriver::format_output(&output, &config.output, &driver_result.db)
}

/// Parse a hex string (with or without `0x` prefix) into a `ValueId`.
fn parse_value_id(s: &str) -> anyhow::Result<saf_core::ids::ValueId> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    let raw = u128::from_str_radix(hex_str, 16)?;
    Ok(saf_core::ids::ValueId::new(raw))
}

/// Parse a hex string (with or without `0x` prefix) into a `FunctionId`.
fn parse_function_id(s: &str) -> anyhow::Result<saf_core::ids::FunctionId> {
    let hex_str = s.strip_prefix("0x").unwrap_or(s);
    let raw = u128::from_str_radix(hex_str, 16)?;
    Ok(saf_core::ids::FunctionId::new(raw))
}

/// Run `saf query` — execute a query against the analysis database.
pub fn query(args: &QueryArgs) -> anyhow::Result<()> {
    use crate::driver::{AnalysisDriver, DriverConfig};

    // Build with defaults (precise mode, Andersen PTA, all defaults)
    let default_run_args = default_run_args(&args.input, args.frontend);
    let config = DriverConfig::from_run_args(&default_run_args);
    let driver = AnalysisDriver::build(&args.input, &config, args.frontend)?;

    let resolver = driver.db.display_resolver();

    match &args.command {
        QueryCommand::PointsTo { pointer } => {
            let vid = parse_value_id(pointer)?;
            let label = resolver.resolve(vid.raw());
            let locs = driver.db.points_to(vid);
            if locs.is_empty() {
                println!(
                    "No points-to targets found for {} ({})",
                    vid.to_hex(),
                    label
                );
            } else {
                println!(
                    "Points-to set for {} ({}) — {} targets:",
                    vid.to_hex(),
                    label,
                    locs.len()
                );
                for loc in &locs {
                    let loc_label = resolver.resolve(loc.raw());
                    println!("  {} ({})", loc.to_hex(), loc_label);
                }
            }
        }
        QueryCommand::Alias { p, q } => {
            let p_id = parse_value_id(p)?;
            let q_id = parse_value_id(q)?;
            let p_label = resolver.resolve(p_id.raw());
            let q_label = resolver.resolve(q_id.raw());
            let result = driver.db.may_alias(p_id, q_id);
            println!(
                "Alias({} [{}], {} [{}]) = {result:?}",
                p_id.to_hex(),
                p_label,
                q_id.to_hex(),
                q_label
            );
        }
        QueryCommand::Flows { source, sink } => {
            let req = serde_json::json!({
                "action": "flows",
                "source": source,
                "sink": sink,
            });
            let resp = driver
                .db
                .handle_request(&req.to_string())
                .map_err(|e| anyhow::anyhow!("JSON protocol error: {e}"))?;
            println!("{resp}");
        }
        QueryCommand::Taint { source, sink } => {
            let req = serde_json::json!({
                "action": "taint_flow",
                "source": source,
                "sink": sink,
            });
            let resp = driver
                .db
                .handle_request(&req.to_string())
                .map_err(|e| anyhow::anyhow!("JSON protocol error: {e}"))?;
            println!("{resp}");
        }
        QueryCommand::Reachable { func_ids } => {
            let mut fids = Vec::new();
            for s in func_ids {
                fids.push(parse_function_id(s)?);
            }
            let reachable = driver.db.cg_reachable_from(&fids);
            println!("Reachable functions ({}):", reachable.len());
            for fid in &reachable {
                let label = resolver.resolve(fid.raw());
                println!("  {} ({})", fid.to_hex(), label);
            }
        }
    }

    Ok(())
}

/// Run `saf export` — export a graph or artifact.
pub fn export(args: &ExportArgs) -> anyhow::Result<()> {
    use crate::driver::{AnalysisDriver, DriverConfig};

    let mut default_run = default_run_args(&args.input, args.frontend);
    // For findings export, we need checkers enabled.
    if matches!(args.target, CliExportTarget::Findings) {
        default_run.checkers = "all".to_string();
    }
    let config = DriverConfig::from_run_args(&default_run);
    let driver = AnalysisDriver::build(&args.input, &config, args.frontend)?;

    let output_text = match args.target {
        CliExportTarget::Findings => {
            // Run all checkers and format as JSON or SARIF.
            let analysis_output = driver.analyze(&config)?;
            match args.format {
                CliExportFormat::Json => serde_json::to_string_pretty(&analysis_output.findings)?,
                CliExportFormat::Sarif => {
                    // Delegate to the same SARIF formatter used by `saf run --format sarif`.
                    AnalysisDriver::format_sarif_string(&analysis_output)?
                }
                CliExportFormat::Dot | CliExportFormat::Html => {
                    anyhow::bail!(
                        "Findings can only be exported as JSON or SARIF, not {}",
                        args.format
                    )
                }
            }
        }
        target => {
            // Graph exports: build the PropertyGraph, then format.
            let pg = build_property_graph(&driver, target, args.function.as_deref())?;
            match args.format {
                CliExportFormat::Json => serde_json::to_string_pretty(&pg)?,
                CliExportFormat::Dot => pg.to_dot(),
                CliExportFormat::Html => pg.to_html(),
                CliExportFormat::Sarif => {
                    anyhow::bail!("SARIF format is only valid for findings export")
                }
            }
        }
    };

    if let Some(ref path) = args.output {
        std::fs::write(path, &output_text)?;
        eprintln!("Wrote {} export to {}", args.target, path.display());
    } else {
        print!("{output_text}");
    }

    Ok(())
}

/// Build a `PropertyGraph` for the given export target.
fn build_property_graph(
    driver: &crate::driver::AnalysisDriver,
    target: CliExportTarget,
    function_filter: Option<&str>,
) -> anyhow::Result<saf_analysis::export::PropertyGraph> {
    let module = driver.db.module();

    match target {
        CliExportTarget::Cfg => {
            let func = if let Some(name) = function_filter {
                module
                    .functions
                    .iter()
                    .find(|f| f.name == name)
                    .ok_or_else(|| anyhow::anyhow!("Function '{name}' not found in module"))?
            } else {
                // Default to the first non-declaration function, or main.
                module
                    .functions
                    .iter()
                    .find(|f| f.name == "main" && !f.is_declaration)
                    .or_else(|| module.functions.iter().find(|f| !f.is_declaration))
                    .ok_or_else(|| {
                        anyhow::anyhow!("No functions found. Use --function to specify.")
                    })?
            };
            let cfg = driver.db.cfg(func.id);
            Ok(cfg.to_pg(func, &module.source_files, None))
        }
        CliExportTarget::Callgraph => Ok(driver.db.call_graph().to_pg(module, None)),
        CliExportTarget::Defuse => Ok(driver.db.defuse().to_pg(module, None)),
        CliExportTarget::Valueflow => Ok(saf_analysis::to_property_graph(
            driver.db.valueflow(),
            module,
            None,
        )),
        CliExportTarget::Svfg => {
            anyhow::bail!(
                "SVFG export is not available via CLI. \
                 Use `saf run --serve` and the JSON protocol instead."
            )
        }
        CliExportTarget::Pta => {
            let pta = driver
                .db
                .pta_result()
                .ok_or_else(|| anyhow::anyhow!("PTA did not produce results"))?;
            Ok(pta.to_pg(None))
        }
        CliExportTarget::Findings => {
            // Handled by the caller; this arm is unreachable in practice.
            anyhow::bail!("Findings export is handled separately")
        }
    }
}

/// Run `saf schema` — print the discovery schema.
pub fn schema(args: &SchemaArgs) -> anyhow::Result<()> {
    use saf_analysis::database::catalog::CheckCatalog;

    let catalog = CheckCatalog::new();

    if args.checkers {
        return print_checkers(&catalog, args.format);
    }
    if args.frontends {
        return print_frontends(args.format);
    }

    // Print everything.
    match args.format {
        CliOutputFormat::Human => {
            print_checkers_human(&catalog);
            println!();
            print_frontends_human();
            println!();
            println!("Queries:");
            println!("  points-to, alias, flows, taint, reachable");
            println!();
            println!("Export targets:");
            println!("  cfg, callgraph, defuse, valueflow, svfg, findings, pta");
        }
        CliOutputFormat::Json => {
            let entries: Vec<_> = catalog.entries().values().collect();
            let schema = serde_json::json!({
                "checkers": entries,
                "frontends": ["llvm", "air-json"],
                "queries": ["points-to", "alias", "flows", "taint", "reachable"],
                "export_targets": ["cfg", "callgraph", "defuse", "valueflow", "svfg", "findings", "pta"],
            });
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        CliOutputFormat::Sarif => {
            anyhow::bail!("SARIF format is not applicable for schema output")
        }
    }

    Ok(())
}

/// Print checker catalog in the requested format.
fn print_checkers(
    catalog: &saf_analysis::database::catalog::CheckCatalog,
    format: CliOutputFormat,
) -> anyhow::Result<()> {
    match format {
        CliOutputFormat::Human => print_checkers_human(catalog),
        CliOutputFormat::Json => {
            let entries: Vec<_> = catalog.entries().values().collect();
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        CliOutputFormat::Sarif => {
            anyhow::bail!("SARIF format is not applicable for checker listing")
        }
    }
    Ok(())
}

/// Print checkers as a human-readable table.
fn print_checkers_human(catalog: &saf_analysis::database::catalog::CheckCatalog) {
    println!("Checkers:");
    println!("  {:<28} {:<6} {:<10} Category", "Name", "CWE", "Severity");
    for entry in catalog.entries().values() {
        let cwe = entry.cwe.map_or_else(|| "-".to_string(), |c| c.to_string());
        println!(
            "  {:<28} {:<6} {:<10} {}",
            entry.name,
            cwe,
            format!("{:?}", entry.severity).to_lowercase(),
            entry.category
        );
    }
}

/// Print frontends in the requested format.
fn print_frontends(format: CliOutputFormat) -> anyhow::Result<()> {
    match format {
        CliOutputFormat::Human => print_frontends_human(),
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&["llvm", "air-json"])?);
        }
        CliOutputFormat::Sarif => {
            anyhow::bail!("SARIF format is not applicable for frontend listing")
        }
    }
    Ok(())
}

/// Print frontends as human-readable text.
fn print_frontends_human() {
    println!("Frontends:");
    println!("  llvm, air-json");
}

pub fn incremental(args: &IncrementalArgs) -> anyhow::Result<()> {
    // Handle --clean: remove cache directory
    if args.clean && args.cache_dir.exists() {
        std::fs::remove_dir_all(&args.cache_dir)?;
        println!("Cleared cache directory: {}", args.cache_dir.display());
    }

    // Handle --plan: dry-run showing what would be recomputed
    if args.plan {
        return incremental_plan(args);
    }

    // Handle --export-summaries: export cached summaries as YAML
    if let Some(ref output_path) = args.export_summaries {
        return export_summaries(&args.cache_dir, output_path);
    }

    // Run full incremental analysis pipeline.
    incremental_run(args)
}

fn incremental_run(args: &IncrementalArgs) -> anyhow::Result<()> {
    use crate::driver::AnalysisDriver;
    use saf_analysis::pipeline::{PipelineConfig, run_pipeline_incremental};
    use saf_analysis::session::AnalysisSession;
    use saf_core::config::AnalysisMode;
    use saf_core::program::AirProgram;

    let mode = match args.mode {
        IncrementalMode::Sound => AnalysisMode::Precise,
        IncrementalMode::BestEffort => AnalysisMode::Fast,
    };

    // 1. Ingest each input file as a separate bundle
    let mut bundles = Vec::with_capacity(args.inputs.len());
    for input in &args.inputs {
        let bundle = AnalysisDriver::ingest(&[input.clone()], args.frontend)?;
        bundles.push(bundle);
    }

    // 2. Link bundles into a program
    let program = AirProgram::link(bundles);
    eprintln!(
        "Linked {} module(s) into program {}",
        program.modules.len(),
        program.id.to_hex()
    );

    // 3. Load or create session
    let mut session = AnalysisSession::load(&args.cache_dir);
    eprintln!(
        "Session loaded (run #{}, cache: {})",
        session.run_count + 1,
        args.cache_dir.display()
    );

    // 4. Configure pipeline
    let pipeline_config = PipelineConfig::from_mode(mode);

    // 5. Run incremental pipeline
    let result = run_pipeline_incremental(&program, &pipeline_config, &mut session);

    // 6. Save session state for next run
    session.save().map_err(|e| {
        anyhow::anyhow!(
            "Failed to save session to {}: {e}",
            args.cache_dir.display()
        )
    })?;

    // 7. Print results
    println!("=== Incremental Analysis Results ===");
    println!("  Mode:              {}", args.mode);
    println!("  Modules:           {}", program.modules.len());
    println!(
        "  Def-use build:     {:.3}s",
        result.stats.defuse_build_secs
    );
    println!("  PTA solve:         {:.3}s", result.stats.pta_solve_secs);
    println!(
        "  CG refinement:     {} iterations",
        result.stats.refinement_iterations
    );
    println!(
        "  Value-flow build:  {:.3}s",
        result.stats.valueflow_build_secs
    );
    println!("  Total:             {:.3}s", result.stats.total_secs);

    if let Some(ref pta) = result.pta_result {
        println!("  PTA values:        {}", pta.value_count());
    }
    let total_cg_edges: usize = result
        .call_graph
        .edges
        .values()
        .map(std::collections::BTreeSet::len)
        .sum();
    println!("  Call graph edges:  {total_cg_edges}");

    println!("\nSession saved to {}", args.cache_dir.display());

    Ok(())
}

/// Dry-run: compute and display the invalidation plan without executing.
fn incremental_plan(args: &IncrementalArgs) -> anyhow::Result<()> {
    use std::collections::BTreeMap;

    let manifest = CacheManifest::load(&args.cache_dir);

    // Compute current file fingerprints
    let mut current_fingerprints = BTreeMap::new();
    for input in &args.inputs {
        if !input.exists() {
            anyhow::bail!("Input file not found: {}", input.display());
        }
        let contents = std::fs::read(input)?;
        let fingerprint = saf_core::id::id_to_hex(saf_core::id::make_id("file", &contents));
        let path_key = input.display().to_string();
        current_fingerprints.insert(path_key, fingerprint);
    }

    let diff = manifest.diff(&current_fingerprints);

    println!("Incremental analysis plan:");
    println!("  Mode: {}", args.mode);
    println!("  Cache: {}", args.cache_dir.display());
    println!("  Inputs: {} files", args.inputs.len());
    println!();

    if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
        println!("  No changes detected. All modules up to date.");
        println!("  Recompute steps: none");
    } else {
        if !diff.added.is_empty() {
            println!("  Added modules ({}):", diff.added.len());
            for name in &diff.added {
                println!("    + {name}");
            }
        }
        if !diff.removed.is_empty() {
            println!("  Removed modules ({}):", diff.removed.len());
            for name in &diff.removed {
                println!("    - {name}");
            }
        }
        if !diff.changed.is_empty() {
            println!("  Changed modules ({}):", diff.changed.len());
            for name in &diff.changed {
                println!("    ~ {name}");
            }
        }

        let total_changed = diff.added.len() + diff.removed.len() + diff.changed.len();
        println!();
        println!("  Planned recompute steps:");
        println!("    1. Re-extract constraints for {total_changed} module(s)");
        println!("    2. Run incremental PTA");
        println!("    3. Rebuild call graph");
        println!("    4. Rebuild value-flow for affected functions");
        println!("    5. Re-run checkers");
    }

    Ok(())
}

/// Export cached analysis-computed summaries as YAML.
///
/// Reads individual `{cache_dir}/summaries/*.json` files produced by
/// previous analysis runs and writes a single YAML file that can be
/// shipped alongside a library and loaded by downstream consumers as specs.
fn export_summaries(cache_dir: &Path, output_path: &Path) -> anyhow::Result<()> {
    let summaries_dir = cache_dir.join("summaries");
    if !summaries_dir.exists() {
        anyhow::bail!(
            "No summaries found in {}. Run an incremental analysis first.",
            summaries_dir.display()
        );
    }

    let mut summaries: Vec<FunctionSummary> = Vec::new();

    for entry in std::fs::read_dir(&summaries_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let contents = std::fs::read_to_string(&path)?;
        match serde_json::from_str::<FunctionSummary>(&contents) {
            Ok(summary) => summaries.push(summary),
            Err(e) => {
                eprintln!("Warning: skipping {}: {}", path.display(), e);
            }
        }
    }

    if summaries.is_empty() {
        anyhow::bail!("No valid summaries found in {}.", summaries_dir.display());
    }

    // Sort by function ID for deterministic output
    summaries.sort_by_key(|s| s.function_id);

    let yaml = serde_yaml::to_string(&summaries)?;
    std::fs::write(output_path, &yaml)?;

    println!(
        "Exported {} summaries to {}",
        summaries.len(),
        output_path.display()
    );

    Ok(())
}

pub fn specs(args: &SpecsArgs) -> anyhow::Result<()> {
    match &args.command {
        SpecsCommand::List { verbose } => {
            let registry = SpecRegistry::load()?;

            // Show loaded paths
            let paths = registry.loaded_paths();
            if paths.is_empty() {
                println!("No spec files loaded.");
                println!("\nSpec discovery paths:");
                println!("  1. <binary>/../share/saf/specs/*.yaml (shipped defaults)");
                println!("  2. ~/.saf/specs/*.yaml (user global)");
                println!("  3. ./saf-specs/*.yaml (project local)");
                println!("  4. $SAF_SPECS_PATH/*.yaml (explicit override)");
                return Ok(());
            }

            println!("Loaded spec files:");
            for path in paths {
                println!("  {}", path.display());
            }

            // Show warnings
            let warnings = registry.warnings();
            if !warnings.is_empty() {
                println!("\nWarnings:");
                for warning in warnings {
                    println!("  {warning}");
                }
            }

            // Show specs
            println!(
                "\nFunction specs ({} exact, {} patterns):",
                registry.len(),
                registry.patterns().count()
            );

            if *verbose {
                for spec in registry.iter() {
                    println!("\n  {}:", spec.name);
                    if let Some(role) = &spec.role {
                        println!("    role: {role:?}");
                    }
                    if spec.is_pure() {
                        println!("    pure: true");
                    }
                    if spec.is_noreturn() {
                        println!("    noreturn: true");
                    }
                    if !spec.params.is_empty() {
                        println!("    params: {} defined", spec.params.len());
                    }
                    if spec.returns.is_some() {
                        println!("    returns: defined");
                    }
                    if spec.taint.is_some() {
                        println!("    taint: defined");
                    }
                }
            } else {
                for spec in registry.iter() {
                    let role = spec
                        .role
                        .as_ref()
                        .map(|r| format!(" ({r:?})"))
                        .unwrap_or_default();
                    println!("  {}{}", spec.name, role);
                }
            }

            Ok(())
        }

        SpecsCommand::Validate { path } => {
            let path = PathBuf::from(path);
            if path.is_dir() {
                // Validate all YAML files in directory
                let pattern = path.join("**/*.yaml");
                let pattern_str = pattern.display().to_string();
                let entries: Vec<_> = glob::glob(&pattern_str)?.filter_map(Result::ok).collect();

                if entries.is_empty() {
                    println!("No .yaml files found in {}", path.display());
                    return Ok(());
                }

                let mut all_ok = true;
                for file in entries {
                    match SpecFile::load(&file) {
                        Ok(spec_file) => {
                            println!("✓ {} ({} specs)", file.display(), spec_file.specs.len());
                        }
                        Err(e) => {
                            println!("✗ {}: {}", file.display(), e);
                            all_ok = false;
                        }
                    }
                }

                if all_ok {
                    println!("\nAll spec files are valid.");
                } else {
                    anyhow::bail!("Some spec files have errors");
                }
            } else {
                // Validate single file
                match SpecFile::load(&path) {
                    Ok(spec_file) => {
                        println!("✓ {} ({} specs)", path.display(), spec_file.specs.len());
                        for spec in &spec_file.specs {
                            println!("  - {}", spec.name);
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("{}: {}", path.display(), e);
                    }
                }
            }

            Ok(())
        }

        SpecsCommand::Lookup { name } => {
            let registry = SpecRegistry::load()?;

            if let Some(spec) = registry.lookup(name) {
                println!("Spec for '{name}':");
                // Pretty-print the spec as YAML
                let yaml = serde_yaml::to_string(spec)?;
                println!("{yaml}");
            } else {
                println!("No spec found for '{name}'");
                println!("\nNote: Analysis will use conservative assumptions for this function.");
            }

            Ok(())
        }
    }
}
