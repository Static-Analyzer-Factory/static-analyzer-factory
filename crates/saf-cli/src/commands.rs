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

/// Data model for `saf verify` (SV-COMP). Selects clang `-m32`/`-m64` and the
/// LLVM target; passed by `BenchExec` per category.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliDataModel {
    /// 32-bit pointers and `long` (ILP32).
    #[value(name = "ILP32")]
    Ilp32,
    /// 64-bit pointers and `long` (LP64).
    #[value(name = "LP64")]
    Lp64,
}

impl From<CliDataModel> for saf_svcomp::DataModel {
    fn from(v: CliDataModel) -> Self {
        match v {
            CliDataModel::Ilp32 => Self::ILP32,
            CliDataModel::Lp64 => Self::LP64,
        }
    }
}

// ---------------------------------------------------------------------------
// CLI struct definitions
// ---------------------------------------------------------------------------

// Version string surfaced by `saf --version`. Includes the LLVM major.minor
// this binary links against so users can tell LLVM 18 and LLVM 22 images
// apart without running an analysis.
#[cfg(all(feature = "llvm-18", not(feature = "llvm-22")))]
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (LLVM 18.1)");
#[cfg(all(feature = "llvm-22", not(feature = "llvm-18")))]
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (LLVM 22.1)");
#[cfg(not(any(feature = "llvm-18", feature = "llvm-22")))]
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (no LLVM)");

#[derive(Parser)]
#[command(name = "saf", version = VERSION, about = "Static Analyzer Factory")]
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
    /// Verify a C program against an SV-COMP property (blind; prints `true`/`false(p)`/`unknown`).
    Verify(VerifyArgs),
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

/// Arguments for `saf verify` — the blind SV-COMP verifier entry point (plan 192).
#[derive(Args)]
pub struct VerifyArgs {
    /// The C program under verification (`.c` or preprocessed `.i`).
    #[arg(required = true)]
    pub input: PathBuf,

    /// SV-COMP property file (`.prp`). Parsed for the `CHECK(... LTL ...)` form,
    /// never the filename.
    #[arg(long, required = true)]
    pub property: PathBuf,

    /// Data model; selects clang `-m32`/`-m64` and the LLVM target.
    #[arg(long, value_enum, default_value_t = CliDataModel::Lp64)]
    pub data_model: CliDataModel,

    /// Where to write the violation witness (YAML 2.0). `BenchExec` passes `${witness}`.
    #[arg(long, default_value = "witness.yml")]
    pub witness: PathBuf,

    /// Wall-clock budget in seconds. On expiry, print `unknown` and exit 0 — a
    /// graceful UNKNOWN before `BenchExec`'s SIGKILL. Only ever yields UNKNOWN.
    #[arg(long, default_value_t = 850)]
    pub timeout: u64,

    /// Optional full machine-readable report (JSON) to a file; stdout stays verdict-only.
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

/// Run `saf verify` — the blind SV-COMP verifier entry point (plan 192, slice 1).
///
/// Prints exactly one verdict line to stdout — `false(<prop>)`, `true`, or
/// `unknown` — and nothing else (all diagnostics go to stderr). `true` is emitted
/// ONLY for `termination` on a proven loop-free ∧ acyclic-callgraph structural
/// proof (plan 201, R7 — witness-not-required); every other property never emits
/// `true` (a proof of safety maps to `unknown`, so `-32` exposure stays zero). It
/// never reads an expected verdict, and always exits 0 once a verdict is printed.
///
/// Pipeline: pin determinism-affecting env toggles, parse the `.prp` (real
/// `CHECK/LTL` form), compile the C in-tool with clang+`opt -passes=mem2reg`,
/// ingest the IR, and dispatch to the property's `strategy_for` arm, all under a
/// wall-clock watchdog that degrades to `unknown` on timeout. Properties with no
/// wired strategy map to `unknown`.
// The handler is infallible by contract (it always prints a verdict and exits
// 0), but the `Result` return is required by the `Commands` dispatch signature.
#[allow(clippy::unnecessary_wraps)]
pub fn verify(args: &VerifyArgs) -> anyhow::Result<()> {
    // Determinism pins (plan 192 §2.3): make verdicts independent of these
    // verdict-affecting env toggles regardless of the competition environment.
    // Must run before the worker thread is spawned (the unsafe `remove_var`
    // requires no concurrent environment access).
    for var in ["SAF_PTA_FIELD_MINTING", "SAF_DECOMPOSE_POINTER_ARRAYS"] {
        if std::env::var_os(var).is_some() {
            eprintln!("saf verify: ignoring env {var} (pinned off for deterministic verdicts)");
            // SAFETY: called at process start, before any analysis threads are
            // spawned, so there is no concurrent access to the environment.
            unsafe { std::env::remove_var(var) };
        }
    }

    // Parse the property from the `.prp` contents (never the filename). Any read
    // problem is non-fatal: default to the safe `unknown`. The raw text is also
    // the witness `specification` field.
    let prp_text = std::fs::read_to_string(&args.property).unwrap_or_else(|e| {
        eprintln!(
            "saf verify: cannot read property file {}: {e}",
            args.property.display()
        );
        String::new()
    });
    let property = saf_svcomp::Property::from_prp(&prp_text);
    let data_model: saf_svcomp::DataModel = args.data_model.into();

    // Verdict-dispatch table (plan 194): a property is analyzed only if it has a
    // registered strategy; everything else is the safe `unknown`. New properties
    // (memsafety/overflow/…) plug in by adding a `strategy_for` arm.
    let Some(property) = property else {
        eprintln!("saf verify: unrecognized property -> unknown");
        println!("unknown");
        return Ok(());
    };
    if strategy_for(property).is_none() {
        eprintln!(
            "saf verify: property `{}` not analyzed in this build -> unknown",
            property.name()
        );
        println!("unknown");
        return Ok(());
    }

    // Run compile -> ingest -> strategy on a worker thread under a wall-clock
    // budget. On timeout (or a worker panic, which drops the sender) emit the
    // safe `unknown` and exit 0, gracefully beating BenchExec's SIGKILL. The
    // watchdog only ever yields UNKNOWN, so it can never produce an unsound
    // verdict.
    let input = args.input.clone();
    let specification = prp_text.trim().to_string();
    let deadline = std::time::Duration::from_secs(args.timeout);
    let (tx, rx) = std::sync::mpsc::channel();
    let _worker = std::thread::spawn(move || {
        let _ = tx.send(run_verdict(&input, data_model, property, specification));
    });

    let outcome = if let Ok(o) = rx.recv_timeout(deadline) {
        o
    } else {
        eprintln!(
            "saf verify: analysis exceeded {}s budget (or worker failed) -> unknown",
            args.timeout
        );
        unknown_outcome()
    };

    // Write the violation witness ONLY for a `false` verdict received before the
    // deadline — never on timeout/unknown/true — so a witness is emitted only
    // alongside a sound FALSE. The verdict is printed regardless: a `false`
    // whose witness is missing/unwritable is still sound (it just scores 0).
    if outcome.verdict.starts_with("false") {
        // Precedence: prefer a YAML-2.0 violation witness when present, because the
        // `--witness` path is a `.yml` validated against the 2.0 schema. A
        // concurrency FALSE now carries BOTH — a YAML-2.0 target witness (for the
        // re-verification validator panel, which re-derives the interleaving) AND a
        // GraphML-1.0 interleaving witness — so the YAML wins here; GraphML remains
        // the only witness for `no-data-race` (no reach_error to anchor a target),
        // and so is written when no YAML witness exists.
        if let Some(witness) = &outcome.witness {
            match witness.to_yaml_string() {
                Ok(yaml) => match std::fs::write(&args.witness, yaml) {
                    Ok(()) => eprintln!(
                        "saf verify: wrote violation witness to {}",
                        args.witness.display()
                    ),
                    Err(e) => eprintln!(
                        "saf verify: failed to write witness to {}: {e} (verdict still emitted)",
                        args.witness.display()
                    ),
                },
                Err(e) => eprintln!(
                    "saf verify: witness serialization failed: {e:#} (verdict still emitted)"
                ),
            }
        } else if let Some(graphml) = &outcome.graphml {
            // Concurrency witnesses are GraphML 1.0 (R7); write the pre-serialized
            // string verbatim.
            match std::fs::write(&args.witness, graphml) {
                Ok(()) => eprintln!(
                    "saf verify: wrote GraphML violation witness to {}",
                    args.witness.display()
                ),
                Err(e) => eprintln!(
                    "saf verify: failed to write GraphML witness to {}: {e} (verdict still emitted)",
                    args.witness.display()
                ),
            }
        }
    }
    println!("{}", outcome.verdict);
    Ok(())
}

/// Clang / opt binaries matching the LLVM this binary links against, overridable
/// via `$SAF_CLANG` / `$SAF_OPT`.
#[cfg(feature = "llvm-22")]
const DEFAULT_CLANG: &str = "clang-22";
#[cfg(feature = "llvm-22")]
const DEFAULT_OPT: &str = "opt-22";
#[cfg(not(feature = "llvm-22"))]
const DEFAULT_CLANG: &str = "clang-18";
#[cfg(not(feature = "llvm-22"))]
const DEFAULT_OPT: &str = "opt-18";

/// Locate the bundled SV-COMP stub header, independently of the CWD.
///
/// Checks, in order: the `$SAF_SVCOMP_STUBS` override; every ancestor of the
/// running binary joined with `share/saf/stubs/sv-comp-stubs.h` (covers both an
/// installed `<prefix>/bin/saf` -> `<prefix>/share/...` layout and a dev
/// `target/<profile>/saf` -> workspace-root `share/...` layout); and finally a
/// `./share/...` fallback.
fn resolve_svcomp_stub() -> Option<PathBuf> {
    const REL: &str = "share/saf/stubs/sv-comp-stubs.h";

    if let Some(p) = std::env::var_os("SAF_SVCOMP_STUBS") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors() {
            let candidate = ancestor.join(REL);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let cwd_rel = PathBuf::from(REL);
    if cwd_rel.is_file() {
        return Some(cwd_rel);
    }
    None
}

/// Write a one-shot header that neutralizes the stub's `__VERIFIER_assert` MACRO,
/// returning its path for a second `-include`. The stub guards the macro with
/// `#ifndef __VERIFIER_assert`, but `-include`ing the stub first still defines it;
/// `-include`ing THIS header after the stub `#undef`s the macro, so a program that
/// provides its OWN `void __VERIFIER_assert(int)` definition (loop-zilu,
/// nla-digbench, bitvector, recursified families) parses as C instead of hitting a
/// macro-vs-function-definition conflict. Reused by ingestion and the `UBSan` replay
/// compile so a fallback-compiled program also replays.
fn write_assert_neutralizer(dir: &Path) -> anyhow::Result<PathBuf> {
    use anyhow::Context;
    let p = dir.join("saf_undef_assert.h");
    std::fs::write(
        &p,
        "#ifdef __VERIFIER_assert\n#undef __VERIFIER_assert\n#endif\n",
    )
    .with_context(|| "writing __VERIFIER_assert neutralizer header")?;
    Ok(p)
}

/// Run a prepared clang command, returning `Ok(true)` iff it exited successfully.
/// Subprocess stdout is always discarded (the parent's stdout stays verdict-only);
/// `quiet` also discards stderr — used for the first pass of a two-pass compile so
/// a to-be-retried failure is not reported as a fatal diagnostic.
fn clang_emit_ok(mut cmd: std::process::Command, quiet: bool) -> std::io::Result<bool> {
    use std::process::Stdio;
    cmd.stdout(Stdio::null());
    if quiet {
        cmd.stderr(Stdio::null());
    }
    Ok(cmd.status()?.success())
}

/// Compile a C program to mem2reg'd LLVM IR in `dir`, returning the `.ll` path.
///
/// Mirrors the offline SV-COMP recipe: clang emits `-O0` IR with
/// `-disable-O0-optnone` (so `opt`'s mem2reg pass is not a no-op), the data-model
/// flag, and the `-include`d stub header; then `opt -passes=mem2reg` promotes
/// allocas to SSA. Subprocess stdout is discarded so the parent's stdout stays
/// verdict-only; stderr is inherited (diagnostics). A two-pass assert-macro
/// fallback (see [`write_assert_neutralizer`]) rescues programs that define their
/// own `__VERIFIER_assert` function.
fn compile_to_ir(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    stub: &Path,
    dir: &Path,
) -> anyhow::Result<PathBuf> {
    // The default ingestion recipe: no extra clang flags, mem2reg-only promotion.
    // This is the IR every confirmer (unreach/memsafety/overflow/race) sees.
    compile_to_ir_with(input, data_model, stub, dir, &[], "mem2reg")
}

/// Compile a C program to promoted LLVM IR in `dir`, returning the `.ll` path.
///
/// Generalizes [`compile_to_ir`] with two knobs: `extra_clang_args` (appended to
/// the `-emit-llvm` invocation) and `opt_passes` (the `opt -passes=<…>` pipeline
/// run after emission). The default recipe (`compile_to_ir`) passes no extra clang
/// args and `"mem2reg"`. The `termination` re-ingestion (see
/// [`compile_to_ir_termination`]) uses a richer promotion pipeline to expose
/// memory-backed loop induction variables to the ranking synthesizer.
///
/// Mirrors the offline SV-COMP recipe: clang emits `-O0` IR with
/// `-disable-O0-optnone` (so `opt`'s promotion passes are not a no-op), the
/// data-model flag, and the `-include`d stub header. Subprocess stdout is discarded
/// so the parent's stdout stays verdict-only; stderr is inherited (diagnostics). A
/// two-pass assert-macro fallback (see [`write_assert_neutralizer`]) rescues
/// programs that define their own `__VERIFIER_assert` function.
fn compile_to_ir_with(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    stub: &Path,
    dir: &Path,
    extra_clang_args: &[&str],
    opt_passes: &str,
) -> anyhow::Result<PathBuf> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let clang = std::env::var("SAF_CLANG").unwrap_or_else(|_| DEFAULT_CLANG.to_string());
    let opt = std::env::var("SAF_OPT").unwrap_or_else(|_| DEFAULT_OPT.to_string());
    let ir = dir.join("input.ll");
    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));

    // Build the base emit-LLVM invocation; `assert_neutralizer` (when Some) is a
    // second `-include`d header that `#undef`s the stub's `__VERIFIER_assert`
    // MACRO so a program that DEFINES its own `void __VERIFIER_assert(int)`
    // compiles (two-pass fallback below).
    let build_cmd = |assert_neutralizer: Option<&Path>| {
        let mut cmd = Command::new(&clang);
        cmd.args([
            "-g",
            "-S",
            "-emit-llvm",
            "-O0",
            "-Xclang",
            "-disable-O0-optnone",
            "-Wno-everything",
        ])
        .args(extra_clang_args)
        .arg(data_model.clang_flag())
        .arg("-include")
        .arg(stub);
        if let Some(neutralizer) = assert_neutralizer {
            cmd.arg("-include").arg(neutralizer);
        }
        cmd.arg("-I").arg(srcdir).arg(input).arg("-o").arg(&ir);
        cmd
    };

    // Pass 1: the plain compile (stub `__VERIFIER_assert` macro active). Suppress
    // its stderr — a failure here is retried before it is ever reported as fatal.
    let mut ok =
        clang_emit_ok(build_cmd(None), true).with_context(|| format!("failed to spawn {clang}"))?;
    // Pass 2 (assert-macro fallback): only when pass 1 failed. Neutralize the
    // stub's `__VERIFIER_assert` macro so a program that provides its own function
    // definition (loop-zilu / nla-digbench / bitvector / recursified families)
    // parses. Additive: pass 1's committed behavior is unchanged; this only
    // rescues previously-failing ingestions. Pass 2's stderr is shown so a genuine
    // compile error still surfaces.
    if !ok {
        let neutralizer = write_assert_neutralizer(dir)?;
        ok = clang_emit_ok(build_cmd(Some(&neutralizer)), false)
            .with_context(|| format!("failed to spawn {clang}"))?;
    }
    anyhow::ensure!(ok, "{clang} failed to compile {}", input.display());

    let opt_status = Command::new(&opt)
        .arg("-S")
        .arg(format!("-passes={opt_passes}"))
        .arg(&ir)
        .arg("-o")
        .arg(&ir)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn {opt}"))?;
    anyhow::ensure!(opt_status.success(), "{opt} promotion pass failed");

    Ok(ir)
}

/// Re-compile `input` for the `termination` structural proof with a richer
/// promotion pipeline, ingest it, and return the fresh `AirModule`.
///
/// # Why a second ingestion
///
/// The shared [`compile_to_ir`] runs only `mem2reg`, which promotes *static*
/// scalar allocas to SSA. The `termination-memory-alloca` / `-linkedlists`
/// families deliberately place each loop induction variable in a heap/stack cell
/// obtained via the `alloca()` **library call** (`int *i = alloca(sizeof(int))`).
/// Clang lowers that to a dynamically-sized `alloca i8, i64 4` accessed through a
/// bit-cast `i32` pointer — a shape `mem2reg` refuses to promote — so the loop has
/// **no header phi**, the ranking synthesizer sees no induction variable, and the
/// proof abstains on the entire family. Running `instcombine` (canonicalizes the
/// alloca to `[4 x i8]`), then `sroa` (splits the single-scalar cell), then
/// `mem2reg` (promotes to phis) recovers the SSA induction variables the ranker
/// needs.
///
/// # Soundness
///
/// All three passes are semantics-preserving, so the promoted module terminates on
/// a given input **iff** the original does — a structural-termination proof over
/// the promoted module therefore soundly implies the original terminates. To keep
/// that equivalence airtight we additionally pass `-fno-finite-loops`, which strips
/// the C `mustprogress` forward-progress attribute: without it an optimizer is
/// permitted to *delete* a side-effect-free infinite loop, which could turn a
/// genuinely non-terminating program into a terminating one and yield a wrong
/// `true`. None of `instcombine`/`sroa`/`mem2reg` perform loop deletion, and with
/// `mustprogress` gone none is even licensed to; the ranking synthesizer remains
/// the sole termination arbiter.
fn compile_to_ir_termination(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    stub: &Path,
    dir: &Path,
) -> anyhow::Result<saf_core::air::AirModule> {
    let ir = compile_to_ir_with(
        input,
        data_model,
        stub,
        dir,
        &["-fno-finite-loops"],
        "instcombine,sroa,mem2reg",
    )?;
    let bundle = driver::AnalysisDriver::ingest(&[ir], CliFrontend::Llvm)?;
    Ok(bundle.module)
}

/// The result of a verdict computation: the stdout line plus an optional
/// violation witness (present only for a sound `false`).
///
/// A sound `false` may carry EITHER a YAML-2.0 sequential witness (`witness`,
/// the default for unreach-call/memsafety/overflow) OR a raw GraphML-1.0
/// concurrency witness string (`graphml`, required for `no-data-race` — R7). At
/// most one is populated; `graphml` takes precedence when present.
struct VerdictOutcome {
    verdict: String,
    witness: Option<saf_svcomp::ViolationWitness>,
    /// Pre-serialized `GraphML` 1.0 witness (concurrency properties, R7).
    graphml: Option<String>,
}

/// The safe fallback: `unknown` with no witness.
fn unknown_outcome() -> VerdictOutcome {
    VerdictOutcome {
        verdict: "unknown".to_string(),
        witness: None,
        graphml: None,
    }
}

/// Everything a per-property strategy needs. Compile+ingest is shared; execution
/// (native replay) stays in `saf-cli`, while the pure engine lives in `saf-svcomp`.
struct VerifyCtx<'a> {
    input: &'a Path,
    data_model: saf_svcomp::DataModel,
    module: &'a saf_core::air::AirModule,
    meta: &'a saf_svcomp::WitnessMeta,
    stub: &'a Path,
    tempdir: &'a Path,
    clang: &'a str,
}

/// A per-property `propose -> concrete-confirm -> witness` pipeline.
type StrategyFn = fn(&VerifyCtx) -> VerdictOutcome;

/// The verdict-dispatch table (plan 194 extensibility spine). Today only
/// `unreach-call` is wired; memsafety/overflow/… add an arm here, each with its
/// own concrete confirmer, reusing the shared orchestration and witness emitter.
fn strategy_for(property: saf_svcomp::Property) -> Option<StrategyFn> {
    match property {
        saf_svcomp::Property::UnreachCall => Some(unreach_strategy),
        saf_svcomp::Property::ValidMemsafety => Some(memsafety_strategy),
        saf_svcomp::Property::NoOverflow => Some(overflow_strategy),
        saf_svcomp::Property::Termination => Some(termination_strategy),
        saf_svcomp::Property::NoDataRace => Some(no_data_race_strategy),
        _ => None,
    }
}

/// Shared orchestration: compile -> ingest -> build witness metadata -> dispatch
/// to the property strategy. Any internal failure maps to `unknown` (diagnostic
/// on stderr); never errors, so the caller keeps exit code 0.
fn run_verdict(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    property: saf_svcomp::Property,
    specification: String,
) -> VerdictOutcome {
    let Some(strategy) = strategy_for(property) else {
        return unknown_outcome();
    };
    let Some(stub) = resolve_svcomp_stub() else {
        eprintln!("saf verify: could not locate share/saf/stubs/sv-comp-stubs.h -> unknown");
        return unknown_outcome();
    };
    let dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("saf verify: could not create temp dir: {e} -> unknown");
            return unknown_outcome();
        }
    };
    let ir = match compile_to_ir(input, data_model, &stub, dir.path()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("saf verify: compilation failed: {e:#} -> unknown");
            return unknown_outcome();
        }
    };
    let bundle = match driver::AnalysisDriver::ingest(&[ir], CliFrontend::Llvm) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("saf verify: ingestion failed: {e:#} -> unknown");
            return unknown_outcome();
        }
    };
    let clang = std::env::var("SAF_CLANG").unwrap_or_else(|_| DEFAULT_CLANG.to_string());
    let meta = saf_svcomp::WitnessMeta {
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        specification,
        data_model,
        language: saf_svcomp::Language::C,
        input_file: input.to_path_buf(),
    };
    let ctx = VerifyCtx {
        input,
        data_model,
        module: &bundle.module,
        meta: &meta,
        stub: &stub,
        tempdir: dir.path(),
        clang: &clang,
    };
    strategy(&ctx)
}

/// Assemble a violation witness from a lowered waypoint list, logging (but not
/// failing the verdict) on any assembly error.
fn build_witness(
    ctx: &VerifyCtx,
    waypoints: Option<Vec<saf_svcomp::SourceWaypoint>>,
) -> Option<saf_svcomp::ViolationWitness> {
    let waypoints = waypoints?;
    match saf_svcomp::ViolationWitness::assemble(ctx.meta, &waypoints) {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("saf verify: witness assembly failed: {e:#}");
            None
        }
    }
}

/// Build the target-only YAML-2.0 violation witness for a **concurrency**
/// unreach-call FALSE, anchored at the module's `reach_error` call site.
///
/// The concurrency confirmers reproduce the violation by forced-schedule native
/// replay (not a sequential must-reach chain), so there is no block path to lower;
/// the reproducing interleaving is carried by the companion `GraphML` witness. This
/// target-only YAML witness exists so the eval's validator panel — whose
/// re-verification members (`CBMC`, `cpa-witness2test`, `CPAchecker`) re-derive the
/// schedule themselves and only need the violation *location* — can actually score
/// the FALSE: a `GraphML` 1.0 witness is rejected by `WitnessLint`'s 2.0 schema gate
/// before any validator runs, so without this a confirmed concurrency FALSE scores
/// 0. Returns `None` when no `reach_error` call carries a span (the FALSE is still
/// emitted, witnessless). The verdict is already sound, so the witness can never
/// change a right verdict into a wrong one.
fn conc_target_witness(ctx: &VerifyCtx) -> Option<saf_svcomp::ViolationWitness> {
    build_witness(ctx, saf_svcomp::lower_reach_error_target(ctx.module))
}

/// The `unreach-call` FALSE pipeline (plan 192 §1.6 / slice 1c), now emitting a
/// violation witness alongside each sound FALSE.
///
/// Soundness (unchanged): emit `false` only when `reach_error` is UNCONDITIONALLY
/// reachable (Stage 1, `must_reach_error`) or a concrete native replay reaches it
/// (Stage 2/3). Never emits `true`. The witness is a *side output* of an
/// already-sound verdict — a `false` is still returned when the witness cannot be
/// constructed (e.g. a missing span), it just scores 0 like `unknown`.
// NOTE: staged unreach-call strategy (Stage 1 must-reach → Stage 2/3 replay) kept
// as one cohesive unit; splitting the stages across helpers would obscure the
// fail-closed control flow.
#[allow(clippy::too_many_lines)]
fn unreach_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    use saf_svcomp::Property;

    // Stage 1 (sound, cheap): reach_error UNCONDITIONALLY reached.
    if let Some(chain) = saf_svcomp::must_reach_error(ctx.module) {
        let witness = build_witness(ctx, saf_svcomp::lower_must_reach(ctx.module, &chain));
        if witness.is_none() {
            eprintln!(
                "saf verify: FALSE (must-reach) but witness unconstructible (missing span) -> emitting false without a witness"
            );
        }
        return VerdictOutcome {
            verdict: format!("false({})", Property::UnreachCall.name()),
            witness,
            graphml: None,
        };
    }

    // Concurrency gate: a multithreaded program's reachability is schedule-dependent,
    // so a single native replay of ONE interleaving is not a sound basis for FALSE — a
    // race-free/safe program can be driven to `reach_error` under an arbitrary native
    // schedule (the goblint `race_reach_*_racefree` tasks: `main` spawns 10^4 threads and
    // the assert lives in the thread body). Stage 1 (`must_reach`, sequential
    // unconditional) is already sound above; the sequential replay-confirmed stages
    // below are not schedule-aware, so they must not run on a threaded program.
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if saf_svcomp::fast_paths::reachable_spawns_threads(ctx.module, &callgraph) {
        // Concurrency FALSE finder (lever `conc-seq-m1`): atomic-thread
        // sequentialization. Instead of ONE arbitrary native schedule, replay the
        // ORIGINAL program under a small set of explicit NON-PREEMPTIVE schedules
        // (each thread runs to completion; interleave only at create/join/exit). Every
        // such schedule is a legal SC interleaving, so a `reach_error` it hits is a
        // genuine violation (the schedule is the sole arbiter — R1/R6/R7, GraphML
        // witness). `conc_schedulable` fail-closes on every feature the single-OS-thread
        // model cannot faithfully reproduce (nondet, TLS, condvars, OpenMP, …), so this
        // never confirms on the `_racefree` class. Any miss ⇒ abstain (below).
        if let Some(outcome) = conc_confirm_false(ctx) {
            return outcome;
        }
        // Second concurrency confirmer (lever `conc-replay-confirm`): a cooperative
        // single-token scheduler that forces FINE-GRAINED interleavings (context
        // switches at `__VERIFIER_atomic` boundaries) on the REAL multithreaded
        // binary. Catches interleaving-dependent bugs the whole-thread atomic-thread
        // model above cannot reach (`fib_unsafe`, `triangular`, `reorder`, …). Every
        // replayed run is a real serialized SC interleaving, so a `reach_error` it hits
        // is a genuine violation (R1/R6/R7, GraphML witness); any miss ⇒ abstain.
        if let Some(outcome) = conc_replay_confirm_false(ctx) {
            return outcome;
        }
        // Third concurrency confirmer (lever `conc-shim-firstpass`): a SYSTEMATIC
        // bounded-preemption (CHESS-style) cooperative scheduler that MODELS pthread
        // mutexes and preempts at shared memory accesses. Reaches the mutex-guarded
        // interleaving bugs neither engine above can (`conc_seq` is whole-thread;
        // `conc_replay` refuses any mutex). Every forced schedule is a real serialized
        // SC interleaving with mutual exclusion enforced, so a `reach_error` it hits is a
        // genuine violation (R1/R6/R7, GraphML witness); any miss ⇒ abstain.
        if let Some(outcome) = conc_shim_confirm_false(ctx) {
            return outcome;
        }
        eprintln!(
            "saf verify: a thread spawn is reachable from main (no atomic-thread schedule confirmed) -> unknown"
        );
        return unknown_outcome();
    }

    // Stage 2 + 3: enumerate over-approximate candidates, CONFIRM by concrete
    // native replay. Only a run that actually reaches reach_error is a violation,
    // so the over-approximation's false alarms are filtered out (they do not
    // reproduce), and we still never emit `true`.
    let config = saf_svcomp::PropertyAnalysisConfig {
        conservative: false,
        ..Default::default()
    };
    let candidates = saf_svcomp::enumerate_false_candidates(ctx.module, &config);
    for (idx, candidate) in candidates.iter().take(MAX_REPLAY_CANDIDATES).enumerate() {
        match replay_confirms_false(
            ctx.input,
            ctx.data_model,
            ctx.stub,
            ctx.tempdir,
            ctx.clang,
            idx,
            candidate,
        ) {
            Ok(true) => {
                let witness =
                    build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, candidate));
                if witness.is_none() {
                    eprintln!(
                        "saf verify: FALSE (replay-confirmed) but witness unconstructible -> emitting false without a witness"
                    );
                }
                return VerdictOutcome {
                    verdict: format!("false({})", Property::UnreachCall.name()),
                    witness,
                    graphml: None,
                };
            }
            Ok(false) => {}
            Err(e) => eprintln!("saf verify: replay of candidate {idx} errored: {e:#} -> continue"),
        }
    }

    // Stage 4 (R4, plan 196): INTERPROCEDURAL candidates rooted at `main`. The
    // intraprocedural enumeration above roots at each reach_error's OWN function,
    // so a steering nondet read in `main` (or an intermediate caller) is invisible
    // and never gets pinned. Compose bounded call chains `main -> ... -> F` and
    // confirm through the SAME native-replay gate — the sole arbiter of FALSE, so
    // a spurious composition can only ever yield `unknown`, never a wrong verdict.
    // The replay index is offset past the intraprocedural batch so sentinel/driver
    // temp files never collide.
    let interproc = saf_svcomp::enumerate_false_candidates_interproc(ctx.module, &config);
    if !interproc.is_empty() {
        // R4 addressable-surface signal: reached only when must-reach + the
        // intraprocedural candidates did NOT already confirm, so this counts the
        // tasks where interprocedural composition is the *only* remaining lever.
        eprintln!(
            "saf verify: R4 enumerated {} interprocedural candidate(s) (main -> callee reach_error)",
            interproc.len()
        );
    }
    for (idx, candidate) in interproc.iter().take(MAX_REPLAY_CANDIDATES).enumerate() {
        match replay_confirms_false(
            ctx.input,
            ctx.data_model,
            ctx.stub,
            ctx.tempdir,
            ctx.clang,
            MAX_REPLAY_CANDIDATES + idx,
            candidate,
        ) {
            Ok(true) => {
                let witness =
                    build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, candidate));
                if witness.is_none() {
                    eprintln!(
                        "saf verify: FALSE (interproc replay-confirmed) but witness unconstructible -> emitting false without a witness"
                    );
                }
                return VerdictOutcome {
                    verdict: format!("false({})", Property::UnreachCall.name()),
                    witness,
                    graphml: None,
                };
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!(
                    "saf verify: interproc replay of candidate {idx} errored: {e:#} -> continue"
                );
            }
        }
    }

    // Stage 4b (BMC base case, fixed-k): the Z3 path stages above model each guard
    // operand as a fresh unconstrained variable, so a guard defined by ARITHMETIC of
    // the nondet inputs (`y = x*3+7; if (y==100)`) is proposed with the guard operand
    // pinned but the INPUT `x` left free — the replay then fails. The BMC engine
    // instead symbolically executes from `main` with every value modelled as a
    // bitvector of its width, so the Z3 model gives the actual nondet INPUT vector
    // that makes the arithmetic guard true. Two sub-engines: the acyclic base case
    // (k = 1) and an INCREMENTAL unwinder that grows the loop bound in ONE persistent
    // Z3 context, gating each depth's reach_error check with a `check-sat-assuming`
    // activation literal — so a violation gated behind several loop iterations
    // (`for(i=0;i<20;i++) s+=x; if(s==60) reach_error();`) is reached an order of
    // magnitude deeper per solver budget than re-solving whole paths. Candidates go
    // through the SAME native-replay gate (the sole arbiter), so a spurious/imprecise
    // model can only ever yield `unknown`. Runs before the blind fuzzer because it
    // cracks arithmetic guards the fuzzer's blind/CmpLog search cannot (the input is
    // a preimage of the compared value).
    let bmc = saf_svcomp::enumerate_bmc_candidates(ctx.module, &config, ctx.data_model);
    if !bmc.is_empty() {
        eprintln!(
            "saf verify: BMC enumerated {} candidate(s) (fixed-k + incremental)",
            bmc.len()
        );
    }
    for (idx, candidate) in bmc.iter().take(MAX_REPLAY_CANDIDATES).enumerate() {
        match replay_confirms_false(
            ctx.input,
            ctx.data_model,
            ctx.stub,
            ctx.tempdir,
            ctx.clang,
            2 * MAX_REPLAY_CANDIDATES + idx,
            candidate,
        ) {
            Ok(true) => {
                let witness =
                    build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, candidate));
                if witness.is_none() {
                    eprintln!(
                        "saf verify: FALSE (BMC replay-confirmed) but witness unconstructible -> emitting false without a witness"
                    );
                }
                return VerdictOutcome {
                    verdict: format!("false({})", Property::UnreachCall.name()),
                    witness,
                    graphml: None,
                };
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("saf verify: BMC replay of candidate {idx} errored: {e:#} -> continue");
            }
        }
    }

    // Stage 4c (KLEE-style forward symbolic execution): the fixed-k BMC engine
    // above only unwinds loops to their acyclic base case (k = 1), so a violation
    // reachable only after a loop runs a few iterations (`for(i=0;i<4;i++) s+=x;
    // if(s==40) reach_error();`) is proposed with the accumulator un-grown and the
    // guard UNSAT. The SE engine forward-executes from the error function's entry,
    // forking on Z3-feasible branches and unwinding loops to a bounded per-block
    // visit cap, so the model gives the actual nondet INPUT vector — in execution
    // order, so per-iteration nondet reads each get their own value. Gated to
    // functions with a nondet input AND a CFG cycle (the loop-carried class BMC
    // misses). Candidates go through the SAME native-replay gate (the sole
    // arbiter), so a spurious/imprecise model can only ever yield `unknown`.
    let se = saf_svcomp::enumerate_se_candidates(ctx.module, &config, ctx.data_model);
    if !se.is_empty() {
        eprintln!(
            "saf verify: SE enumerated {} candidate(s) (forward)",
            se.len()
        );
    }
    for (idx, candidate) in se.iter().take(MAX_REPLAY_CANDIDATES).enumerate() {
        match replay_confirms_false(
            ctx.input,
            ctx.data_model,
            ctx.stub,
            ctx.tempdir,
            ctx.clang,
            3 * MAX_REPLAY_CANDIDATES + idx,
            candidate,
        ) {
            Ok(true) => {
                let witness =
                    build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, candidate));
                if witness.is_none() {
                    eprintln!(
                        "saf verify: FALSE (SE replay-confirmed) but witness unconstructible -> emitting false without a witness"
                    );
                }
                return VerdictOutcome {
                    verdict: format!("false({})", Property::UnreachCall.name()),
                    witness,
                    graphml: None,
                };
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("saf verify: SE replay of candidate {idx} errored: {e:#} -> continue");
            }
        }
    }

    let total = candidates.len() + interproc.len() + bmc.len() + se.len();
    if total == 0 {
        eprintln!(
            "saf verify: no FALSE candidate proposed (reach_error not proven reachable) -> unknown"
        );
    } else {
        // Candidates were over-approximated as FALSE but did not reproduce under
        // concrete replay — the soundness filter that keeps false alarms out.
        eprintln!(
            "saf verify: {total} candidate(s) enumerated ({} intraproc + {} interproc + {} bmc + {} se); none reproduced reach_error at runtime -> unknown",
            candidates.len(),
            interproc.len(),
            bmc.len(),
            se.len()
        );
    }

    // Stage 5 (blind byte-stream fuzz): the Z3 stages only reach errors whose
    // guards its linear-arithmetic model can solve; a guard defined by
    // nonlinear/opaque arithmetic (`if (x*x==...)`, bit tricks, hashed indices)
    // leaves reach_error un-proposed. An AFL-style blind mutation loop over a
    // deterministic byte-stream nondet shim (dictionary = the program's own IR
    // constants) searches for an input that drives the ORIGINAL program into
    // reach_error natively. A run that drops the sentinel yields a concrete input
    // sequence, which is RE-CONFIRMED through the same deterministic replay gate
    // (R6) before any verdict — so the fuzzer can only ever propose, never
    // manufacture a wrong FALSE. Runs only when the program has scalar nondet
    // input to fuzz and a reach_error to reach.
    if let Some(candidate) = fuzz_confirm_false(ctx) {
        let witness = build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, &candidate));
        if witness.is_none() {
            eprintln!(
                "saf verify: FALSE (fuzz replay-confirmed) but witness unconstructible -> emitting false without a witness"
            );
        }
        return VerdictOutcome {
            verdict: format!("false({})", Property::UnreachCall.name()),
            witness,
            graphml: None,
        };
    }

    // Stage 6 (CBMC bit-precise oracle): the blind fuzzer and the Z3 stages both
    // stall on modular bit-vector transition systems (the hardware-verification-bv
    // btor2c cluster: masked `SORT_n` arithmetic behind a `for(;;)` step loop) —
    // SAF's linear-integer models over-approximate and blind mutation cannot crack
    // the deep multi-step guards. The now-provisioned CBMC 6.x is a bit-precise
    // SAT-backed BMC: unwinding the step loop a fixed `k` times and solving the
    // resulting propositional formula yields the exact nondet input vector that
    // reaches reach_error. CBMC is used ONLY as an oracle — the vector it reports is
    // parsed into a nondet sequence and RE-CONFIRMED through the SAME native-replay
    // gate (the sole arbiter, R6), so a wrong/over-approximate CBMC model can only
    // ever yield `unknown`. Gated behind a cheap structural pre-filter (loops
    // present AND scalar-integer nondet AND no float/pointer nondet) and run LAST,
    // only when every earlier stage abstained, so its cost is paid rarely. Degrades
    // to a no-op when the CBMC binary is not provisioned ($SAF_CBMC absent).
    match cbmc_confirm_false(ctx) {
        Some(candidate) => {
            let witness = build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, &candidate));
            if witness.is_none() {
                eprintln!(
                    "saf verify: FALSE (CBMC replay-confirmed) but witness unconstructible -> emitting false without a witness"
                );
            }
            VerdictOutcome {
                verdict: format!("false({})", Property::UnreachCall.name()),
                witness,
                graphml: None,
            }
        }
        None => unknown_outcome(),
    }
}

/// Deterministic cap on blind-fuzz iterations (mutation trials), overridable via
/// `$SAF_FUZZ_ITERS`. This — NOT the wall-clock deadline — is the primary bound, so
/// the search (and therefore the verdict) is reproducible across machines; the
/// deadline is only a pathological-slowness safety valve.
///
/// With `SanitizerCoverage` feedback each exec is far more valuable (a new-edge input
/// is kept and its frontier mutated), so the coverage path is given a larger budget
/// to let the incremental frontier-building actually pay off; the blind path keeps
/// the original small budget (a dictionary-steered guard is usually hit in the seed
/// corpus or the first handful of trials). The sv-benchmarks reach tasks are tiny
/// (µs-scale execs), so even the larger budget finishes well inside the deadline.
fn fuzz_iters(cov_enabled: bool) -> usize {
    if let Some(n) = std::env::var("SAF_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
    {
        return n;
    }
    if cov_enabled { 6000 } else { 600 }
}

/// Wall-clock safety cap on the whole blind-fuzz stage (seconds), overridable via
/// `$SAF_FUZZ_TIME`. The iteration cap is the primary (deterministic) bound; this
/// only stops a pathological slow-harness run from eating the task budget.
fn fuzz_time_budget() -> std::time::Duration {
    let secs = std::env::var("SAF_FUZZ_TIME")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(25);
    std::time::Duration::from_secs(secs)
}

/// Iteration cap for the memsafety byte-stream `ASan` mini-fuzz (pass 3), overridable
/// via `$SAF_MEM_FUZZ_ITERS`. Deliberately small: pass 3 runs only on the residue the
/// uniform/split passes leave `unknown` AND that has ≥2 scalar nondet call sites, and a
/// multi-nondet OOB/UAF is either hit within the seed corpus / the first hundreds of
/// dictionary-steered trials or not at all. A tight cap bounds the aggregate added cost
/// (a bloated per-task budget is what regressed the earlier full-fat version of this
/// pass — it timed OTHER tasks out past the eval budget).
fn mem_fuzz_iters() -> usize {
    std::env::var("SAF_MEM_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(600)
}

/// Wall-clock cap (seconds) for the memsafety byte-stream `ASan` mini-fuzz (pass 3),
/// overridable via `$SAF_MEM_FUZZ_TIME`. Kept small to bound the added per-task cost on
/// the (≥2-nondet) tasks the earlier passes did not resolve — a confirming fault traps
/// fast, well under this, and a longer budget only slows the eval for tasks it will not
/// crack anyway.
fn mem_fuzz_time_budget() -> std::time::Duration {
    let secs = std::env::var("SAF_MEM_FUZZ_TIME")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3);
    std::time::Duration::from_secs(secs)
}

/// Blind byte-stream fuzz confirmer for `unreach-call` (Stage 5).
///
/// Compiles the ORIGINAL program with the byte-stream nondet shim ONCE, runs an
/// AFL-style deterministic mutation loop (dictionary = harvested IR constants),
/// and, on the first input that drops the sentinel, reconstructs the concrete
/// nondet sequence and RE-CONFIRMS it via the existing deterministic
/// sequence-replay gate ([`replay_confirms_false`]) on the original program.
/// Returns the confirmed [`saf_svcomp::FalseCandidate`], or `None` (abstain) on any
/// gate miss / compile failure / no reproduction.
// NOTE: the compile-once / seed / mutate / confirm loop is one cohesive unit;
// splitting it across helpers would obscure the fail-closed control flow.
#[allow(clippy::too_many_lines)]
fn fuzz_confirm_false(ctx: &VerifyCtx) -> Option<saf_svcomp::FalseCandidate> {
    use saf_svcomp::fuzz;
    use std::process::{Command, Stdio};

    // Gate: nothing to fuzz without scalar nondet input, nothing to reach without
    // a reach_error site.
    let error_sites = saf_svcomp::reach_error_call_sites(ctx.module);
    let &reach_error_inst = error_sites.first()?;
    if !fuzz::references_scalar_nondet(ctx.module) {
        return None;
    }

    // SOUNDNESS SENTINEL #1 (`scripts/loop/confirmer_contract.md`): abstain when a
    // scalar `__VERIFIER_nondet_*` value is cast to a pointer (`IntToPtr`) and then
    // dereferenced. The blind byte-stream shim drives that nondet with arbitrary
    // bytes, so it can fabricate an invalid pointer, dereference it, and reach
    // `reach_error` on an infeasible path (aws-c-common's
    // `aws_string_new_from_array_harness` does `(void*)__VERIFIER_nondet_ulong()`) —
    // a spurious FALSE. The taint is PRECISE (nondet ⟶ casts ⟶ IntToPtr ⟶ deref),
    // not a blunt any-`inttoptr` gate; symbolic harness-havoc recovers such recall
    // later. Runs before the (expensive) compile so it costs nothing on the common
    // path.
    if fuzz::nondet_taints_int_to_ptr_deref(ctx.module) {
        eprintln!(
            "saf verify: blind fuzz abstains — nondet value cast to a dereferenced pointer \
             (would fabricate an invalid pointer deref) -> unknown"
        );
        return None;
    }

    let dir = ctx.tempdir;
    let sentinel = dir.join("saf_fuzz.sentinel");
    let driver_src = dir.join("saf_fuzz_driver.c");
    let harness = dir.join("saf_fuzz_harness");
    let input_path = dir.join("saf_fuzz.input");
    let log_path = dir.join("saf_fuzz.log");
    let cov_path = dir.join("saf_fuzz.cov");
    let cmplog_path = dir.join("saf_fuzz.cmplog");
    let i2s_path = dir.join("saf_fuzz.i2s");

    if std::fs::write(
        &driver_src,
        fuzz::synthesize_bytestream_driver(&escape_c_string(&sentinel)),
    )
    .is_err()
    {
        return None;
    }

    // Compile the shim + original program ONCE. Two knobs vs a plain native compile:
    // (a) a two-pass __VERIFIER_assert neutralizer (mirrors compile_to_ir) so a benchmark that DEFINES its
    //     own `void __VERIFIER_assert(int)` compiles instead of hitting the stub's function-like macro
    //     ("while loop outside of a function"); (b) -fsanitize-trap=signed-integer-overflow so a fuzz input
    //     that reaches reach_error only via signed-overflow UB TRAPS before the sentinel drops -> not a hit
    //     -> abstain (SV-COMP labels overflow-only reaches safe, e.g. array-fpi/indp2). Additive on both:
    //     a program that already compiled + reaches reach_error without overflow is unchanged; a genuine
    //     failure still returns None (inconclusive), never a wrong verdict.
    let srcdir = ctx.input.parent().unwrap_or_else(|| Path::new("."));
    let build = |neutralizer: Option<&Path>, cov: bool| {
        let mut cmd = Command::new(ctx.clang);
        cmd.args([
            "-O0",
            "-Wno-everything",
            "-fsanitize=signed-integer-overflow",
            "-fsanitize-trap=signed-integer-overflow",
        ]);
        if cov {
            // Standalone SanitizerCoverage: 8-bit edge counters (coverage map) + a PC
            // table (waypoints) + trace-cmp (CmpLog operands). The driver defines the
            // required callbacks; no sanitizer runtime is linked. Purely a search
            // signal — see `fuzz::CoverageMap` / `fuzz::merge_cmplog`.
            cmd.arg("-fsanitize-coverage=inline-8bit-counters,pc-table,trace-cmp");
        }
        cmd.arg(ctx.data_model.clang_flag())
            .arg("-include")
            .arg(ctx.stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(ctx.input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    // Try the coverage-instrumented build first (greybox feedback). If that fails on
    // this toolchain — for any reason — fall back to the plain build so the fuzzer
    // still runs blind: zero regression, coverage feedback is a strict bonus.
    let neutralizer = write_assert_neutralizer(dir).ok();
    let attempt = |cov: bool| {
        if matches!(build(None, cov).status(), Ok(s) if s.success()) {
            return true;
        }
        if let Some(n) = neutralizer.as_ref() {
            return matches!(build(Some(n.as_path()), cov).status(), Ok(s) if s.success());
        }
        false
    };
    let cov_enabled = attempt(true);
    if !cov_enabled && !attempt(false) {
        return None; // link/compile failure -> inconclusive
    }
    if cov_enabled {
        eprintln!("saf verify: blind fuzz using SanitizerCoverage feedback (edge map + CmpLog)");
    }

    // Backward AIR slice from the reach_error criteria (+ __VERIFIER_assume as a
    // secondary criterion) steers the blind search WITHOUT changing what confirms:
    // the guard constants front-load the dictionary (so they survive the cap and
    // are tried first), and multi-guard sequence seeds cover chains of
    // distinct-valued guards that single-value tiling cannot reach. The verdict is
    // still produced by native replay on the ORIGINAL program below (R6). An empty
    // slice yields a dictionary byte-identical to the plain harvest and no sequence
    // seeds, so this never regresses.
    let slice = saf_svcomp::slicing::backward_slice(ctx.module);
    let mut dict = saf_svcomp::slicing::slice_directed_dictionary(ctx.module, &slice);
    let mut corpus = fuzz::seed_corpus(&dict);
    corpus.extend(saf_svcomp::slicing::sequence_seeds(&slice.guard_constants));
    // Fixed seed -> the whole search (and therefore the verdict) is reproducible.
    let mut rng = fuzz::XorShift64::new(0x5AF3_C0DE);
    let per_run = replay_timeout();
    let iters = fuzz_iters(cov_enabled);
    let deadline = std::time::Instant::now() + fuzz_time_budget();
    let mut max_depth = 0usize;
    // Greybox feedback state. `cov` accumulates the AFL-bucketed 8-bit edge map; an
    // input that lights a new bucket is kept as a mutation base. The CmpLog dump is
    // folded back into `dict` so comparison operands become steering constants. Both
    // are pure search heuristics — the verdict is still native replay (R6).
    let mut cov = fuzz::CoverageMap::new();
    // Redqueen input-to-state queue: candidates produced by patching an input byte
    // window to a value the program compared it against. Drained BEFORE havoc — a
    // single I2S step often clears a magic-value / state-machine guard that blind
    // mutation would need millions of execs to hit.
    let mut i2s_pending: std::collections::VecDeque<Vec<u8>> = std::collections::VecDeque::new();
    // Driller-style concolic plateau escape (lever `fuzz-concolic-z3`). When
    // coverage stops growing for `CONCOLIC_STUCK_THRESHOLD` mutation execs, the
    // last input that DID reach new coverage is handed to a concolic executor that
    // Z3-solves the negation of each branch it took, producing inputs that flip the
    // guards blind mutation is stuck on; those are queued as fresh seeds. Bounded to
    // `MAX_CONCOLIC_RUNS` invocations. Pure search steering — native replay (R6)
    // stays the sole FALSE arbiter, so a solved seed can never manufacture a verdict.
    let mut stuck = 0usize;
    let mut concolic_runs = 0usize;
    let mut plateau_seq: Vec<saf_svcomp::NondetCall> = Vec::new();

    // Trial 0..N: the seed corpus first (its entries are tried verbatim before any
    // mutation), then mutations of corpus entries. A run that consumes MORE nondet
    // bytes than any seen (deeper execution) is kept in the corpus — a lightweight,
    // instrumentation-free greybox signal that progressively deepens the search.
    let total = iters + corpus.len();
    for i in 0..total {
        if std::time::Instant::now() >= deadline {
            break;
        }
        let input: Vec<u8> = if i < corpus.len() {
            corpus[i].clone()
        } else if let Some(cand) = i2s_pending.pop_back() {
            // Input-to-state candidates take priority over blind mutation. LIFO
            // (depth-first) so a freshly-cracked stage's follow-up candidates are
            // tried immediately — this chains through multi-stage guards in a handful
            // of execs instead of draining a huge FIFO of stale candidates first.
            cand
        } else {
            // Frontier-biased energy: favour the most-recently-added (deepest /
            // newest-coverage) corpus entries as mutation bases.
            let base = &corpus[rng.below_biased_high(corpus.len())];
            fuzz::mutate(&mut rng, base, &dict)
        };

        if std::fs::write(&input_path, &input).is_err() {
            continue;
        }
        let _ = std::fs::remove_file(&sentinel);
        let _ = std::fs::remove_file(&log_path);
        let _ = std::fs::remove_file(&cov_path);
        let _ = std::fs::remove_file(&cmplog_path);
        let _ = std::fs::remove_file(&i2s_path);

        if run_fuzz_harness(
            &harness,
            &input_path,
            &log_path,
            &cov_path,
            &cmplog_path,
            &i2s_path,
            per_run,
        )
        .is_err()
        {
            continue;
        }

        if sentinel.exists() {
            // Hit: reconstruct the concrete sequence and re-confirm deterministically
            // on the ORIGINAL program through the existing replay gate (R6).
            let log = std::fs::read_to_string(&log_path).unwrap_or_default();
            let nondet_sequence = fuzz::parse_fuzz_log(&log);
            let candidate = saf_svcomp::FalseCandidate {
                reach_error_inst,
                block_path: Vec::new(),
                assignments: std::collections::BTreeMap::new(),
                nondet_sequence,
            };
            match replay_confirms_false(
                ctx.input,
                ctx.data_model,
                ctx.stub,
                ctx.tempdir,
                ctx.clang,
                // Offset the replay index well past the Z3 batches so temp files
                // never collide.
                2 * MAX_REPLAY_CANDIDATES + i,
                &candidate,
            ) {
                Ok(true) => {
                    eprintln!(
                        "saf verify: blind fuzz reached reach_error (trial {i}); re-confirmed -> false(unreach-call)"
                    );
                    return Some(candidate);
                }
                // A sentinel drop that does NOT re-confirm deterministically means
                // the harness state was not faithfully captured; abstain (sound).
                Ok(false) => {
                    eprintln!(
                        "saf verify: fuzz hit at trial {i} did not re-confirm deterministically -> continue"
                    );
                }
                Err(e) => eprintln!("saf verify: fuzz re-confirm errored: {e:#} -> continue"),
            }
        } else {
            // Greybox corpus feedback (runs that did NOT hit the sentinel).
            //
            // With coverage instrumentation: fold this run's edge map; an input that
            // lit a NEW bucket is a coverage frontier and is kept as a mutation base.
            // Merge the CmpLog operands into the dictionary regardless (they steer
            // future mutations past magic-value guards). Without instrumentation, fall
            // back to the original depth (nondet-log-line-count) heuristic.
            if cov_enabled {
                if let Ok(cl) = std::fs::read_to_string(&cmplog_path) {
                    fuzz::merge_cmplog(&mut dict, &cl, MAX_MERGED_DICT);
                }
                let novel = std::fs::read(&cov_path).is_ok_and(|raw| cov.fold(&raw));
                // Only EXPAND the search from inputs that reached somewhere new: keep
                // them as mutation bases, and grow the Redqueen frontier from them
                // (patch input windows to the values just compared against). Gating on
                // novelty keeps the queue tight and walks the fill frontier stage by
                // stage instead of flooding it with redundant candidates.
                if novel {
                    // Coverage advanced: reset the plateau counter and remember this
                    // input's concrete nondet sequence as the concolic base.
                    stuck = 0;
                    if let Ok(log) = std::fs::read_to_string(&log_path) {
                        let seq = fuzz::parse_fuzz_log(&log);
                        if !seq.is_empty() {
                            plateau_seq = seq;
                        }
                    }
                    if i >= corpus.len() && corpus.len() < MAX_FUZZ_CORPUS {
                        corpus.push(input.clone());
                    }
                    if let Ok(dump) = std::fs::read_to_string(&i2s_path) {
                        let pairs = fuzz::parse_i2s(&dump);
                        for cand in
                            fuzz::i2s_candidates(&input, &pairs, MAX_I2S_PER_RUN, I2S_PER_PAIR)
                        {
                            if i2s_pending.len() >= MAX_I2S_PENDING {
                                i2s_pending.pop_front(); // bounded: drop the oldest
                            }
                            i2s_pending.push_back(cand);
                        }
                    }
                } else if i >= corpus.len() {
                    // A mutation exec that reached nowhere new: a coverage plateau is
                    // building. After enough consecutive stalls, invoke the Driller
                    // concolic escape on the last new-coverage seed to solve past the
                    // guards blind mutation cannot flip. Bounded invocations; the
                    // solved inputs are queued as ordinary seeds (native replay stays
                    // the sole arbiter).
                    stuck += 1;
                    if stuck >= CONCOLIC_STUCK_THRESHOLD
                        && concolic_runs < MAX_CONCOLIC_RUNS
                        && !plateau_seq.is_empty()
                    {
                        stuck = 0;
                        concolic_runs += 1;
                        let flips = saf_svcomp::enumerate_concolic_flip_seeds(
                            ctx.module,
                            &plateau_seq,
                            ctx.data_model,
                        );
                        if !flips.is_empty() {
                            eprintln!(
                                "saf verify: concolic plateau escape #{concolic_runs} -> {} flip seed(s) queued",
                                flips.len()
                            );
                        }
                        for fseq in &flips {
                            let seed = fuzz::nondet_seq_to_input(fseq, ctx.data_model);
                            if i2s_pending.len() >= MAX_I2S_PENDING {
                                i2s_pending.pop_front();
                            }
                            i2s_pending.push_back(seed);
                        }
                    }
                }
            } else if i >= corpus.len() {
                let depth = std::fs::read_to_string(&log_path)
                    .map(|l| l.lines().count())
                    .unwrap_or(0);
                if depth > max_depth && corpus.len() < MAX_FUZZ_CORPUS {
                    max_depth = depth;
                    corpus.push(input);
                }
            }
        }
    }

    eprintln!("saf verify: blind fuzz exhausted (no confirmed reach_error) -> unknown");
    None
}

/// The CBMC loop-unwinding bound (`--unwind k`), overridable via `$SAF_CBMC_UNWIND`.
///
/// This — not the outer wall-clock safety valve — is the DETERMINISTIC cost bound
/// (the lever's cost-gate contract): the SAT instance size is a function of `k`, so
/// the search and therefore the verdict is reproducible across machines. A shallow
/// reachable violation is found well inside the safety-valve timeout; a genuinely
/// deep one is missed *consistently* (a recall cost, never a soundness one).
fn cbmc_unwind() -> u32 {
    std::env::var("SAF_CBMC_UNWIND")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|k| *k > 0)
        .unwrap_or(saf_svcomp::DEFAULT_UNWIND)
}

/// Outer wall-clock cap on a single CBMC invocation — a pathological-slowness
/// SAFETY VALVE only, NOT the cost gate (that is [`cbmc_unwind`]). Generous by
/// default so it trips only on a genuine SAT hang; on either a hang or a clean
/// `SUCCESSFUL` the outcome is the same (no trace → abstain), so the timeout never
/// changes a FALSE into anything but an abstain. Overridable via `$SAF_CBMC_TIMEOUT`
/// (seconds).
fn cbmc_timeout() -> std::time::Duration {
    let secs = std::env::var("SAF_CBMC_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(30);
    std::time::Duration::from_secs(secs)
}

/// Resolve the provisioned CBMC install directory (containing `cbmc` +
/// `libminisat.so.2`) from `$SAF_CBMC`, defaulting to the bind-mount path the
/// validator uses. Returns `None` (→ CBMC stage no-ops) when the binary is absent.
fn resolve_cbmc() -> Option<std::path::PathBuf> {
    let home = std::env::var("SAF_CBMC").unwrap_or_else(|_| "/workspace/.svtools/cbmc".to_string());
    let bin = std::path::Path::new(&home).join("cbmc");
    if bin.is_file() {
        Some(std::path::PathBuf::from(home))
    } else {
        None
    }
}

/// CBMC bit-precise oracle confirmer for `unreach-call` (Stage 6, last resort).
///
/// Runs the provisioned CBMC on the ORIGINAL program with a fixed `--unwind k`,
/// `--no-standard-checks` (so only the `reach_error` assertion / no-body failure is
/// a target — R1) and `--stop-on-fail --trace`; parses the counterexample's
/// `__VERIFIER_nondet_*` return values into a concrete input vector (in nondet-call
/// order); and RE-CONFIRMS that vector through the existing native-replay gate
/// ([`replay_confirms_false`]) on the original program. Returns the confirmed
/// [`saf_svcomp::FalseCandidate`], or `None` (abstain) on any gate miss / missing
/// binary / no counterexample / non-reproduction. CBMC is only an oracle — the
/// native replay is the sole arbiter, so a spurious model can never yield a wrong
/// FALSE (R6).
fn cbmc_confirm_false(ctx: &VerifyCtx) -> Option<saf_svcomp::FalseCandidate> {
    use std::process::{Command, Stdio};

    // Gate 1: a reach_error site to reach.
    let error_sites = saf_svcomp::reach_error_call_sites(ctx.module);
    let &reach_error_inst = error_sites.first()?;

    // Gate 2: the cheap, deterministic structural pre-filter (loops present AND
    // scalar-integer nondet AND no float/pointer nondet).
    if !saf_svcomp::cbmc_precheck(ctx.module) {
        return None;
    }

    // Gate 3: the CBMC binary must be provisioned (degrade to no-op otherwise).
    let cbmc_home = resolve_cbmc()?;
    let cbmc_bin = cbmc_home.join("cbmc");

    let unwind = cbmc_unwind();
    // CBMC's own data-model flag (mirrors the task's declared ILP32/LP64, R3).
    let dm_flag = match ctx.data_model {
        saf_svcomp::DataModel::ILP32 => "--ILP32",
        saf_svcomp::DataModel::LP64 => "--LP64",
    };
    let trace_out = ctx.tempdir.join("saf_cbmc.trace");
    let Ok(out) = std::fs::File::create(&trace_out) else {
        return None;
    };

    let mut cmd = Command::new(&cbmc_bin);
    cmd.arg(dm_flag)
        .arg("--unwind")
        .arg(unwind.to_string())
        // Only the property's own violation event is a target: disable CBMC's
        // incidental default checks (overflow / bounds / pointer / div-by-zero) so
        // --stop-on-fail lands on the reach_error assertion, not an unrelated trap
        // (R1). A path that reaches reach_error only via signed overflow still gets
        // rejected downstream: the native replay compiles with
        // -fsanitize-trap=signed-integer-overflow and traps before the sentinel.
        .arg("--no-standard-checks")
        .arg("--stop-on-fail")
        .arg("--trace")
        // Speed: drop functions trivially unreachable from main.
        .arg("--drop-unused-functions")
        .arg(ctx.input)
        .env("LD_LIBRARY_PATH", &cbmc_home)
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(Stdio::null());

    let mut child = harden_replay_spawn(&mut cmd).spawn().ok()?;

    // Poll to the safety-valve timeout; a hang is killed with its whole group.
    let timeout = cbmc_timeout();
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    eprintln!("saf verify: CBMC oracle timed out (unwind {unwind}) -> unknown");
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }

    let trace = std::fs::read_to_string(&trace_out).ok()?;
    // The trace reports raw nondet reads by SOURCE LINE (a simple `x = f();` read is
    // assigned directly to `x` with no `return_value` temp — the load-bearing
    // per-iteration loop inputs), so parse guided by a source-line map of the
    // original program (CBMC runs on the original .c, so its line numbers match).
    let source = std::fs::read_to_string(ctx.input).ok()?;
    let line_map = saf_svcomp::nondet_line_map(&source);
    let nondet_sequence = saf_svcomp::parse_cbmc_trace(&trace, &line_map);
    if nondet_sequence.is_empty() {
        // No counterexample within the bound (SUCCESSFUL / no nondet in the trace)
        // -> abstain.
        return None;
    }
    eprintln!(
        "saf verify: CBMC oracle proposed a {}-value nondet vector (unwind {unwind}); re-confirming natively",
        nondet_sequence.len()
    );

    let candidate = saf_svcomp::FalseCandidate {
        reach_error_inst,
        block_path: Vec::new(),
        assignments: std::collections::BTreeMap::new(),
        nondet_sequence,
    };
    match replay_confirms_false(
        ctx.input,
        ctx.data_model,
        ctx.stub,
        ctx.tempdir,
        ctx.clang,
        // Offset the replay index well past every earlier batch so temp files
        // never collide.
        5 * MAX_REPLAY_CANDIDATES,
        &candidate,
    ) {
        Ok(true) => {
            eprintln!(
                "saf verify: CBMC-proposed vector re-confirmed reach_error -> false(unreach-call)"
            );
            Some(candidate)
        }
        Ok(false) => {
            eprintln!(
                "saf verify: CBMC-proposed vector did not re-confirm deterministically -> unknown"
            );
            None
        }
        Err(e) => {
            eprintln!("saf verify: CBMC re-confirm errored: {e:#} -> unknown");
            None
        }
    }
}

/// Per-schedule wall-clock cap for the concurrency atomic-thread replay. Short: the
/// sv-benchmarks concurrency tasks are µs-scale under a non-preemptive schedule; the
/// only slow case is a busy-wait spin that never terminates (gated out where we can,
/// bounded here otherwise) — a timeout just means "this schedule did not reach the
/// error" → try the next / abstain.
const CONC_SCHED_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);

/// Run the atomic-thread sequentialization harness once under one `SAF_SCHED` order.
/// Returns `Ok(true)` iff the sentinel was dropped (the property's violation event
/// fired). Timeout / normal exit / crash all return `Ok(false)` — the sentinel is the
/// sole confirmer. A runaway (spinning) harness is killed with its whole group.
fn run_conc_schedule(
    harness: &Path,
    sentinel: &Path,
    schedule: saf_svcomp::ConcSchedule,
    timeout: std::time::Duration,
) -> anyhow::Result<bool> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let _ = std::fs::remove_file(sentinel);
    let mut cmd = Command::new(harness);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("SAF_SCHED", schedule.env_value());
    let mut child = harden_replay_spawn(&mut cmd)
        .spawn()
        .with_context(|| "spawning concurrency harness")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(e) => return Err(e).context("waiting on concurrency harness"),
        }
    }
    Ok(sentinel.exists())
}

/// Concurrency `unreach-call` FALSE confirmer (lever `conc-seq-m1`).
///
/// Compiles the ORIGINAL program with the atomic-thread sequentialization driver
/// ([`saf_svcomp::synthesize_conc_driver`]) ONCE, then replays it under each
/// non-preemptive schedule ([`saf_svcomp::CONC_SCHEDULES`]). The FIRST schedule that
/// drops the sentinel (the property's `reach_error` / `__assert_fail` event, R1) is
/// RE-RUN to require the identical deterministic reproduction (R6) before emitting
/// `false(unreach-call)` + a GraphML-1.0 violation witness (R7). Returns `None`
/// (abstain) on any gate miss / compile failure / no reproduction — the schedule is
/// the sole arbiter, so a spurious model can only ever yield `unknown`.
///
/// Sound: every replayed schedule is a legal sequentially-consistent interleaving of
/// the real program executed natively, so a sentinel drop is a genuine reachable
/// assertion violation. [`saf_svcomp::conc_schedulable`] fail-closes on every feature
/// the non-preemptive single-OS-thread model cannot faithfully reproduce.
// NOTE: compile-once / replay-each-schedule / re-confirm is one cohesive fail-closed
// unit; splitting it would obscure the control flow.
#[allow(clippy::too_many_lines)]
fn conc_confirm_false(ctx: &VerifyCtx) -> Option<VerdictOutcome> {
    use saf_svcomp::Property;
    use std::process::{Command, Stdio};

    // Gate 0: there must be a reach_error site to reach.
    if saf_svcomp::reach_error_call_sites(ctx.module).is_empty() {
        return None;
    }

    // Gate 1: the program must be amenable to atomic-thread sequentialization
    // (reachable spawn + only faithfully-modelled primitives, no forced nondet).
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if !saf_svcomp::conc_schedulable(ctx.module, &callgraph) {
        return None;
    }

    // Gate 2 (R7): abstain on OpenMP / relaxed-memory the native SC replay cannot
    // soundly arbitrate. The symbol-level gate above misses `#pragma omp` (the
    // frontend drops it, so no `omp_*` symbol survives) — catch it at the source.
    let source = std::fs::read_to_string(ctx.input).unwrap_or_default();
    if let Some(reason) = tsan_out_of_scope(&source) {
        eprintln!("saf verify: concurrency FALSE out of scope ({reason}) -> unknown");
        return None;
    }

    let dir = ctx.tempdir;
    let sentinel = dir.join("saf_conc.sentinel");
    let driver_src = dir.join("saf_conc_driver.c");
    let harness = dir.join("saf_conc_harness");

    if std::fs::write(
        &driver_src,
        saf_svcomp::synthesize_conc_driver(&escape_c_string(&sentinel)),
    )
    .is_err()
    {
        return None;
    }

    // Compile the driver + original program ONCE, linking the pthread wrappers. As in
    // the fuzz path: a two-pass __VERIFIER_assert neutralizer handles programs that
    // DEFINE their own `void __VERIFIER_assert(int)` (else the stub's function-like
    // macro breaks the build); -fsanitize-trap=signed-integer-overflow makes a path
    // that reaches reach_error only via signed-overflow UB TRAP before the sentinel
    // drops (R1 — the benchmarks are not UB-free), so it never confirms off the wrong
    // event. `-lpthread` + the three wraps redirect create/join/exit into the driver.
    let srcdir = ctx.input.parent().unwrap_or_else(|| Path::new("."));
    let build = |neutralizer: Option<&Path>| {
        let mut cmd = Command::new(ctx.clang);
        cmd.args([
            "-O0",
            "-Wno-everything",
            "-fsanitize=signed-integer-overflow",
            "-fsanitize-trap=signed-integer-overflow",
        ]);
        cmd.arg(ctx.data_model.clang_flag())
            .arg("-include")
            .arg(ctx.stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(ctx.input)
            .arg(&driver_src)
            .arg("-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit")
            .arg("-lpthread")
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let neutralizer = write_assert_neutralizer(dir).ok();
    let compiled = matches!(build(None).status(), Ok(s) if s.success())
        || neutralizer
            .as_ref()
            .is_some_and(|n| matches!(build(Some(n.as_path())).status(), Ok(s) if s.success()));
    if !compiled {
        eprintln!("saf verify: concurrency harness failed to compile -> unknown");
        return None;
    }

    // Replay under each non-preemptive schedule; the first sentinel drop wins, then
    // re-confirm (R6) before emitting.
    for &schedule in saf_svcomp::CONC_SCHEDULES {
        match run_conc_schedule(&harness, &sentinel, schedule, CONC_SCHED_TIMEOUT) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(e) => {
                eprintln!(
                    "saf verify: concurrency replay ({}) errored: {e:#} -> continue",
                    schedule.label()
                );
                continue;
            }
        }
        // R6: require the identical deterministic reproduction on a second run.
        if !matches!(
            run_conc_schedule(&harness, &sentinel, schedule, CONC_SCHED_TIMEOUT),
            Ok(true)
        ) {
            eprintln!(
                "saf verify: concurrency schedule {} did not re-confirm deterministically -> continue",
                schedule.label()
            );
            continue;
        }
        eprintln!(
            "saf verify: FALSE (concurrency atomic-thread schedule '{}' reached reach_error)",
            schedule.label()
        );
        let programfile = ctx.input.file_name().map_or_else(
            || ctx.input.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let programhash = saf_svcomp::compute_file_hash(ctx.input);
        let architecture = match ctx.data_model {
            saf_svcomp::DataModel::ILP32 => "32bit",
            saf_svcomp::DataModel::LP64 => "64bit",
        };
        let thread_count =
            saf_svcomp::fast_paths::reachable_spawn_call_sites(ctx.module, &callgraph);
        let sites = saf_svcomp::ConcWitnessSites {
            spawn_lines: saf_svcomp::fast_paths::reachable_spawn_startlines(ctx.module, &callgraph),
            error_line: saf_svcomp::fast_paths::error_call_startline(ctx.module),
            entry_function: saf_svcomp::fast_paths::spawn_start_routine_name(
                ctx.module, &callgraph,
            ),
        };
        let graphml = saf_svcomp::conc_graphml_witness(
            ctx.meta.specification.trim(),
            &programfile,
            &programhash,
            architecture,
            thread_count,
            schedule,
            &sites,
        );
        // Also emit a target-only YAML-2.0 violation witness anchored at the
        // reach_error site (see `conc_target_witness`): a re-verification validator
        // (CBMC / cpa-witness2test) re-derives the interleaving itself and only
        // needs the violation location, and — unlike the GraphML witness — a YAML
        // 2.0 witness passes WitnessLint's schema gate so the panel can score it.
        return Some(VerdictOutcome {
            verdict: format!("false({})", Property::UnreachCall.name()),
            witness: conc_target_witness(ctx),
            graphml: Some(graphml),
        });
    }

    eprintln!("saf verify: no atomic-thread schedule reached reach_error -> unknown");
    None
}

/// Per-run timeout for one forced-interleaving replay. Short: a gated atomic-only
/// program that does not reproduce terminates in milliseconds; only a pathological
/// (schedule-induced) spin hits this, and a timeout just means "this schedule did not
/// reach the error" → try the next plan / abstain.
const CONC_REPLAY_RUN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(4);

/// Total wall-clock budget across ALL replay plans for one task. Bounds the per-task
/// cost even if many plans each spin to their per-run timeout; when exhausted we
/// abstain (no verdict lost — the schedule is the sole arbiter).
const CONC_REPLAY_TOTAL_BUDGET: std::time::Duration = std::time::Duration::from_secs(60);

/// Run the cooperative-scheduler replay harness once under one `(policy, seed)` plan.
/// Returns `Ok(true)` iff the sentinel dropped (the property's violation event fired).
/// Timeout / normal exit / crash all return `Ok(false)` — the sentinel is the sole
/// confirmer. A runaway (spinning) harness is killed with its whole group.
fn run_conc_replay_plan(
    harness: &Path,
    sentinel: &Path,
    plan: saf_svcomp::ReplayPlan,
    timeout: std::time::Duration,
) -> anyhow::Result<bool> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let _ = std::fs::remove_file(sentinel);
    let mut cmd = Command::new(harness);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("SAF_REPLAY_POLICY", plan.policy_env())
        .env("SAF_REPLAY_SEED", plan.seed_env().to_string());
    let mut child = harden_replay_spawn(&mut cmd)
        .spawn()
        .with_context(|| "spawning concurrency replay harness")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(e) => return Err(e).context("waiting on concurrency replay harness"),
        }
    }
    Ok(sentinel.exists())
}

/// Fine-grained concurrency `unreach-call` FALSE confirmer (lever `conc-replay-confirm`).
///
/// Runs AFTER [`conc_confirm_false`] abstains. Compiles the ORIGINAL program with the
/// cooperative single-token replay driver ([`saf_svcomp::synthesize_conc_replay_driver`])
/// ONCE, then replays it under each forced-interleaving plan
/// ([`saf_svcomp::replay_plans`] — round-robin, then PCT seeds). The FIRST plan that
/// drops the sentinel (the property's `reach_error` / `__assert_fail` event, R1) is
/// RE-RUN to require the identical deterministic reproduction (R6) before emitting
/// `false(unreach-call)` + a GraphML-1.0 violation witness (R7). Returns `None`
/// (abstain) on any gate miss / compile failure / no reproduction / budget exhaustion —
/// the schedule is the sole arbiter, so a spurious model can only ever yield `unknown`.
///
/// Sound: every replayed run is a real serialized sequentially-consistent interleaving
/// of the original program executed natively (single token; `__VERIFIER_atomic`
/// sections held indivisibly), so a sentinel drop is a genuine reachable violation.
/// [`saf_svcomp::conc_replay_schedulable`] fail-closes on every feature the model
/// cannot faithfully reproduce (nondet, mutex exclusion, condvars, …).
// NOTE: compile-once / replay-each-plan / re-confirm is one cohesive fail-closed unit;
// splitting it would obscure the control flow.
#[allow(clippy::too_many_lines)]
fn conc_replay_confirm_false(ctx: &VerifyCtx) -> Option<VerdictOutcome> {
    use saf_svcomp::Property;
    use std::process::{Command, Stdio};

    // Gate 0: there must be a reach_error site to reach.
    if saf_svcomp::reach_error_call_sites(ctx.module).is_empty() {
        return None;
    }

    // Gate 1: the program must be amenable to the fine-grained replay model (reachable
    // spawn, `__VERIFIER_atomic` mid-body scheduling points, no mutex exclusion, no
    // nondet / condvars / … — see conc_replay_schedulable).
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if !saf_svcomp::conc_replay_schedulable(ctx.module, &callgraph) {
        return None;
    }

    // Gate 2 (R7): abstain on OpenMP / relaxed-memory the SC replay cannot arbitrate
    // (symbol-level gates miss `#pragma omp` — the frontend drops it — so catch it here).
    let source = std::fs::read_to_string(ctx.input).unwrap_or_default();
    if let Some(reason) = tsan_out_of_scope(&source) {
        eprintln!("saf verify: concurrency replay out of scope ({reason}) -> unknown");
        return None;
    }

    let dir = ctx.tempdir;
    let sentinel = dir.join("saf_replay.sentinel");
    let driver_src = dir.join("saf_replay_driver.c");
    let harness = dir.join("saf_replay_harness");

    if std::fs::write(
        &driver_src,
        saf_svcomp::synthesize_conc_replay_driver(&escape_c_string(&sentinel)),
    )
    .is_err()
    {
        return None;
    }

    // Compile the driver + original program ONCE, linking the pthread wrappers. As in
    // the atomic-thread path: a two-pass __VERIFIER_assert neutralizer handles programs
    // that DEFINE their own `void __VERIFIER_assert(int)`;
    // -fsanitize-trap=signed-integer-overflow makes a path that reaches reach_error only
    // via signed-overflow UB TRAP before the sentinel drops (R1 — the benchmarks are not
    // UB-free). The three wraps redirect create/join/exit into the driver.
    let srcdir = ctx.input.parent().unwrap_or_else(|| Path::new("."));
    let build = |neutralizer: Option<&Path>| {
        let mut cmd = Command::new(ctx.clang);
        cmd.args([
            "-O0",
            "-Wno-everything",
            "-fsanitize=signed-integer-overflow",
            "-fsanitize-trap=signed-integer-overflow",
        ]);
        cmd.arg(ctx.data_model.clang_flag())
            .arg("-include")
            .arg(ctx.stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(ctx.input)
            .arg(&driver_src)
            .arg("-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit")
            .arg("-lpthread")
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let neutralizer = write_assert_neutralizer(dir).ok();
    let compiled = matches!(build(None).status(), Ok(s) if s.success())
        || neutralizer
            .as_ref()
            .is_some_and(|n| matches!(build(Some(n.as_path())).status(), Ok(s) if s.success()));
    if !compiled {
        eprintln!("saf verify: concurrency replay harness failed to compile -> unknown");
        return None;
    }

    // Replay under each forced-interleaving plan; the first sentinel drop wins, then
    // re-confirm (R6) before emitting. A total wall-clock budget bounds per-task cost.
    let budget_start = std::time::Instant::now();
    for plan in saf_svcomp::replay_plans() {
        if budget_start.elapsed() >= CONC_REPLAY_TOTAL_BUDGET {
            eprintln!("saf verify: concurrency replay budget exhausted -> unknown");
            break;
        }
        match run_conc_replay_plan(&harness, &sentinel, plan, CONC_REPLAY_RUN_TIMEOUT) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(e) => {
                eprintln!(
                    "saf verify: concurrency replay ({}) errored: {e:#} -> continue",
                    plan.label()
                );
                continue;
            }
        }
        // R6: require the identical deterministic reproduction on a second run.
        if !matches!(
            run_conc_replay_plan(&harness, &sentinel, plan, CONC_REPLAY_RUN_TIMEOUT),
            Ok(true)
        ) {
            eprintln!(
                "saf verify: concurrency replay plan {} did not re-confirm deterministically -> continue",
                plan.label()
            );
            continue;
        }
        eprintln!(
            "saf verify: FALSE (concurrency forced-interleaving plan '{}' reached reach_error)",
            plan.label()
        );
        let programfile = ctx.input.file_name().map_or_else(
            || ctx.input.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let programhash = saf_svcomp::compute_file_hash(ctx.input);
        let architecture = match ctx.data_model {
            saf_svcomp::DataModel::ILP32 => "32bit",
            saf_svcomp::DataModel::LP64 => "64bit",
        };
        let thread_count =
            saf_svcomp::fast_paths::reachable_spawn_call_sites(ctx.module, &callgraph);
        let sites = saf_svcomp::ConcWitnessSites {
            spawn_lines: saf_svcomp::fast_paths::reachable_spawn_startlines(ctx.module, &callgraph),
            error_line: saf_svcomp::fast_paths::error_call_startline(ctx.module),
            entry_function: saf_svcomp::fast_paths::spawn_start_routine_name(
                ctx.module, &callgraph,
            ),
        };
        // Reuse the concurrency GraphML violation-witness emitter; the schedule label
        // records which forced interleaving reached the violation (as provenance, not
        // a threadId).
        let graphml = saf_svcomp::conc_graphml_witness_labeled(
            ctx.meta.specification.trim(),
            &programfile,
            &programhash,
            architecture,
            thread_count,
            &plan.label(),
            &sites,
        );
        return Some(VerdictOutcome {
            verdict: format!("false({})", Property::UnreachCall.name()),
            // YAML-2.0 target witness anchored at reach_error (schema-valid, panel-
            // scorable) alongside the GraphML interleaving witness — see
            // `conc_confirm_false` / `conc_target_witness`.
            witness: conc_target_witness(ctx),
            graphml: Some(graphml),
        });
    }

    eprintln!("saf verify: no forced-interleaving plan reached reach_error -> unknown");
    None
}

/// Per-run timeout for one bounded-preemption shim schedule. Short: a mutex-guarded
/// program that does not reproduce under a given `(p1,p2)` terminates in milliseconds;
/// only a pathological (schedule-induced) spin hits this, and a timeout just means "this
/// schedule did not reach the error" → try the next plan / abstain.
const CONC_SHIM_RUN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Total wall-clock budget across ALL shim schedules for one task. Bounds per-task cost
/// even though the `c<=2` sweep enumerates many `(p1,p2)` pairs; when exhausted we abstain
/// (no verdict lost — the schedule is the sole arbiter).
const CONC_SHIM_TOTAL_BUDGET: std::time::Duration = std::time::Duration::from_secs(45);

/// Run the bounded-preemption shim harness once under one `(p1,p2)` schedule. Returns
/// `Ok(true)` iff the sentinel dropped (the property's violation event fired). Timeout /
/// normal exit / crash all return `Ok(false)` — the sentinel is the sole confirmer. A
/// runaway (spinning) harness is killed with its whole group.
fn run_conc_shim_plan(
    harness: &Path,
    sentinel: &Path,
    plan: saf_svcomp::ShimPlan,
    timeout: std::time::Duration,
) -> anyhow::Result<bool> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let _ = std::fs::remove_file(sentinel);
    let mut cmd = Command::new(harness);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("SAF_SHIM_P1", plan.p1_env())
        .env("SAF_SHIM_P2", plan.p2_env());
    let mut child = harden_replay_spawn(&mut cmd)
        .spawn()
        .with_context(|| "spawning concurrency shim harness")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(e) => return Err(e).context("waiting on concurrency shim harness"),
        }
    }
    Ok(sentinel.exists())
}

/// Mutex-aware bounded-preemption concurrency `unreach-call` FALSE confirmer (lever
/// `conc-shim-firstpass`).
///
/// Runs AFTER [`conc_confirm_false`] and [`conc_replay_confirm_false`] both abstain.
/// Compiles the ORIGINAL program with the systematic bounded-preemption driver
/// ([`saf_svcomp::synthesize_conc_shim_driver`]) ONCE — with `SanitizerCoverage` load/store
/// tracing so shared accesses become scheduling points, and `signed-integer-overflow`
/// trapping so a path that reaches `reach_error` only via signed-overflow UB TRAPs before
/// the sentinel (R1 — the benchmarks are not UB-free). Then it replays under each
/// bounded-preemption schedule ([`saf_svcomp::shim_preemption_plans`] — every `c=1` then
/// every `c=2` pair). The FIRST schedule that drops the sentinel (the property's
/// `reach_error` / `__assert_fail` event, R1) is RE-RUN to require the identical
/// deterministic reproduction (R6) before emitting `false(unreach-call)` + a GraphML-1.0
/// violation witness (R7). Returns `None` (abstain) on any gate miss / compile failure / no
/// reproduction / budget exhaustion — the schedule is the sole arbiter, so a spurious model
/// can only ever yield `unknown`.
// NOTE: compile-once / replay-each-plan / re-confirm is one cohesive fail-closed unit;
// splitting it would obscure the control flow.
#[allow(clippy::too_many_lines)]
fn conc_shim_confirm_false(ctx: &VerifyCtx) -> Option<VerdictOutcome> {
    use saf_svcomp::Property;
    use std::process::{Command, Stdio};

    // Gate 0: there must be a reach_error site to reach.
    if saf_svcomp::reach_error_call_sites(ctx.module).is_empty() {
        return None;
    }

    // Gate 1: the program must be amenable to the mutex-aware bounded-preemption model
    // (reachable spawn, no nondet / condvars / …, no __VERIFIER_atomic, no non-default mutex
    // type — see conc_shim_schedulable).
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if !saf_svcomp::conc_shim_schedulable(ctx.module, &callgraph) {
        return None;
    }

    // Gate 2 (R7): abstain on OpenMP / relaxed-memory the SC replay cannot arbitrate
    // (symbol-level gates miss `#pragma omp` — the frontend drops it — so catch it here).
    let source = std::fs::read_to_string(ctx.input).unwrap_or_default();
    if let Some(reason) = tsan_out_of_scope(&source) {
        eprintln!("saf verify: concurrency shim out of scope ({reason}) -> unknown");
        return None;
    }

    let dir = ctx.tempdir;
    let sentinel = dir.join("saf_shim.sentinel");
    let driver_src = dir.join("saf_shim_driver.c");
    let harness = dir.join("saf_shim_harness");

    if std::fs::write(
        &driver_src,
        saf_svcomp::synthesize_conc_shim_driver(&escape_c_string(&sentinel)),
    )
    .is_err()
    {
        return None;
    }

    // Compile the driver + original program ONCE. As in the replay path: a two-pass
    // __VERIFIER_assert neutralizer handles programs that DEFINE their own
    // `void __VERIFIER_assert(int)`; SanitizerCoverage load/store tracing turns shared
    // accesses into scheduling points (the driver defines the callbacks, so no coverage
    // runtime is linked); the six wraps redirect create/join/exit + the mutex API into the
    // driver. If the coverage-instrumented build fails, fall back to a plain build — the
    // scheduler still preempts at the lock/unlock boundaries (fewer points, still sound).
    let srcdir = ctx.input.parent().unwrap_or_else(|| Path::new("."));
    let build = |neutralizer: Option<&Path>, cov: bool| {
        let mut cmd = Command::new(ctx.clang);
        cmd.args([
            "-O0",
            "-Wno-everything",
            "-fsanitize=signed-integer-overflow",
            "-fsanitize-trap=signed-integer-overflow",
        ]);
        if cov {
            cmd.args([
                "-fsanitize-coverage=trace-pc-guard",
                "-mllvm",
                "-sanitizer-coverage-trace-loads=1",
                "-mllvm",
                "-sanitizer-coverage-trace-stores=1",
            ]);
        }
        cmd.arg(ctx.data_model.clang_flag())
            .arg("-include")
            .arg(ctx.stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(ctx.input)
            .arg(&driver_src)
            .arg(
                "-Wl,--wrap=pthread_create,--wrap=pthread_join,--wrap=pthread_exit,\
--wrap=pthread_mutex_lock,--wrap=pthread_mutex_unlock,--wrap=pthread_mutex_trylock",
            )
            .arg("-lpthread")
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let neutralizer = write_assert_neutralizer(dir).ok();
    let try_build = |cov: bool| {
        matches!(build(None, cov).status(), Ok(s) if s.success())
            || neutralizer.as_ref().is_some_and(
                |n| matches!(build(Some(n.as_path()), cov).status(), Ok(s) if s.success()),
            )
    };
    if !try_build(true) && !try_build(false) {
        eprintln!("saf verify: concurrency shim harness failed to compile -> unknown");
        return None;
    }

    // Replay under each bounded-preemption schedule; the first sentinel drop wins, then
    // re-confirm (R6) before emitting. A total wall-clock budget bounds per-task cost.
    let budget_start = std::time::Instant::now();
    for plan in saf_svcomp::shim_preemption_plans() {
        if budget_start.elapsed() >= CONC_SHIM_TOTAL_BUDGET {
            eprintln!("saf verify: concurrency shim budget exhausted -> unknown");
            break;
        }
        match run_conc_shim_plan(&harness, &sentinel, plan, CONC_SHIM_RUN_TIMEOUT) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(e) => {
                eprintln!(
                    "saf verify: concurrency shim ({}) errored: {e:#} -> continue",
                    plan.label()
                );
                continue;
            }
        }
        // R6: require the identical deterministic reproduction on a second run.
        if !matches!(
            run_conc_shim_plan(&harness, &sentinel, plan, CONC_SHIM_RUN_TIMEOUT),
            Ok(true)
        ) {
            eprintln!(
                "saf verify: concurrency shim schedule {} did not re-confirm deterministically -> continue",
                plan.label()
            );
            continue;
        }
        eprintln!(
            "saf verify: FALSE (concurrency bounded-preemption schedule '{}' reached reach_error)",
            plan.label()
        );
        let programfile = ctx.input.file_name().map_or_else(
            || ctx.input.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let programhash = saf_svcomp::compute_file_hash(ctx.input);
        let architecture = match ctx.data_model {
            saf_svcomp::DataModel::ILP32 => "32bit",
            saf_svcomp::DataModel::LP64 => "64bit",
        };
        let thread_count =
            saf_svcomp::fast_paths::reachable_spawn_call_sites(ctx.module, &callgraph);
        let sites = saf_svcomp::ConcWitnessSites {
            spawn_lines: saf_svcomp::fast_paths::reachable_spawn_startlines(ctx.module, &callgraph),
            error_line: saf_svcomp::fast_paths::error_call_startline(ctx.module),
            entry_function: saf_svcomp::fast_paths::spawn_start_routine_name(
                ctx.module, &callgraph,
            ),
        };
        let graphml = saf_svcomp::conc_graphml_witness_labeled(
            ctx.meta.specification.trim(),
            &programfile,
            &programhash,
            architecture,
            thread_count,
            &plan.label(),
            &sites,
        );
        return Some(VerdictOutcome {
            verdict: format!("false({})", Property::UnreachCall.name()),
            // YAML-2.0 target witness anchored at reach_error (schema-valid, panel-
            // scorable) alongside the GraphML interleaving witness — see
            // `conc_confirm_false` / `conc_target_witness`.
            witness: conc_target_witness(ctx),
            graphml: Some(graphml),
        });
    }

    eprintln!("saf verify: no bounded-preemption schedule reached reach_error -> unknown");
    None
}

/// Corpus size cap for the greybox feedback loop — bounds memory and keeps the
/// mutation base-selection distribution stable.
const MAX_FUZZ_CORPUS: usize = 256;

/// Cap on the CmpLog-extended mutation dictionary — bounds the mutation cost while
/// leaving ample room above the static harvest ([`fuzz::MAX_DICT_ENTRIES`]) for
/// dynamically-discovered comparison operands.
const MAX_MERGED_DICT: usize = 1024;

/// Bound on the outstanding Redqueen input-to-state queue (candidates awaiting a
/// run). Keeps the extra execs — and memory — in check on comparison-heavy programs.
const MAX_I2S_PENDING: usize = 4096;

/// Bound on input-to-state candidates harvested from a SINGLE run, so one input that
/// matches many comparison sites cannot monopolise the queue. Sized well above
/// `I2S_PER_PAIR * (typical distinct pairs)` so no comparison site is clipped.
const MAX_I2S_PER_RUN: usize = 256;

/// Per-pair cap on input-to-state match sites (see [`fuzz::i2s_candidates`]): the
/// earliest few matches walk the left-to-right byte-stream fill frontier without a
/// high-multiplicity noise pair crowding out the load-bearing magic-value pair.
const I2S_PER_PAIR: usize = 3;

/// Consecutive new-coverage-free mutation execs that mark a coverage plateau and
/// trigger the Driller concolic escape ([`saf_svcomp::enumerate_concolic_flip_seeds`]).
/// Large enough that `CmpLog` / I2S / havoc get a fair shot first (concolic is the
/// expensive last resort), small enough to fire within the per-task fuzz budget.
const CONCOLIC_STUCK_THRESHOLD: usize = 300;

/// Cap on concolic plateau-escape invocations per task — each is a bounded Z3 budget,
/// but capping the count keeps the total solver cost per task predictable.
const MAX_CONCOLIC_RUNS: usize = 4;

/// Run the byte-stream fuzz harness on one input under a short timeout, feeding
/// `$SAF_FUZZ_INPUT` / `$SAF_FUZZ_LOG`. Success/normal-exit/timeout all return
/// `Ok(())`; the caller inspects the sentinel/log. A runaway harness is killed.
/// Harden a native-replay harness spawn so a hung/spinning sanitizer harness can NEVER outlive us
/// and leak (the 2026-08-19 load runaway that took cd-vm-15 to load 2600+): (a) `PR_SET_PDEATHSIG`
/// so the harness is `SIGKILLed` if this `saf verify` dies mid-replay (the eval RSS watchdog / a
/// supervisor restart) -- the case that previously orphaned harnesses to init for hours; (b) its own
/// process group so a timeout kill (`kill_replay_group`) reaches every descendant (symbolizer, forks).
/// Call immediately before `.spawn()`. Linux-only (SV-COMP runs on Linux).
fn harden_replay_spawn(cmd: &mut std::process::Command) -> &mut std::process::Command {
    use std::os::unix::process::CommandExt;
    // SAFETY: the closure runs in the forked child before exec and only calls async-signal-safe
    // libc functions (prctl / getppid / _exit).
    unsafe {
        cmd.pre_exec(|| {
            libc::prctl(
                libc::PR_SET_PDEATHSIG,
                libc::SIGKILL as libc::c_ulong,
                0,
                0,
                0,
            );
            // Cover the race where the parent already died between fork and prctl.
            if libc::getppid() == 1 {
                libc::_exit(0);
            }
            Ok(())
        });
    }
    cmd.process_group(0)
}

/// SIGKILL a hardened replay harness's ENTIRE process group (leader + descendants), then reap it.
/// Replaces a bare `child.kill()`, which killed only the direct child and orphaned any grandchildren.
fn kill_replay_group(child: &mut std::process::Child) {
    // `harden_replay_spawn` put the child in its own group, so its pgid == child.id().
    // INVARIANT: a Linux pid fits in i32 (default pid_max is 2^22), so the cast never wraps.
    #[allow(clippy::cast_possible_wrap)]
    let pgid = child.id() as i32;
    // SAFETY: kill(2) with a negative pid targets the whole process group; a no-op if already gone.
    unsafe {
        libc::kill(-pgid, libc::SIGKILL);
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn run_fuzz_harness(
    harness: &Path,
    input_path: &Path,
    log_path: &Path,
    cov_path: &Path,
    cmplog_path: &Path,
    i2s_path: &Path,
    timeout: std::time::Duration,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(harness);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("SAF_FUZZ_INPUT", input_path)
        .env("SAF_FUZZ_LOG", log_path)
        // Coverage / CmpLog / I2S feedback channels. Harmless when the harness was
        // built without instrumentation (the dumper writes nothing).
        .env("SAF_FUZZ_COV", cov_path)
        .env("SAF_FUZZ_CMPLOG", cmplog_path)
        .env("SAF_FUZZ_I2S", i2s_path);
    let mut child = harden_replay_spawn(&mut cmd)
        .spawn()
        .with_context(|| "spawning fuzz harness")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Err(e) => return Err(e).context("waiting on fuzz harness"),
        }
    }
    Ok(())
}

/// Constants the memsafety `ASan` mini-fuzz drives every scalar nondet to, in order
/// (0 first — the common unconditional case). A scalar-guarded/scalar-sized fault
/// the zeroed probe misses is reproduced by the matching constant. Sound: each is a
/// valid concrete input, and `__VERIFIER_assume` prunes infeasible ones.
const NONDET_CONSTS: &[i64] = &[0, 1, 2, 42, 255, 256, 1024, 65_535, 2_147_483_647, -1];

/// Curated `(split, base, target)` triples for the memsafety threshold sweep
/// ([`asan_threshold_argv_sweep`]). `split` is how many LEADING scalar-nondet calls
/// return the in-range `base` (the "setup" phase: satisfy an entry guard, pick an
/// array size, fill an array); every LATER call returns the out-of-range `target`
/// (the violation phase: an OOB index, a negative shuffle offset). This reproduces
/// the multi-nondet class a single shared constant cannot — e.g. `i = nondet()` must
/// be in `[0,10)` to ENTER `while (i<10 && a[i]>=0) { i = nondet(); a[i]=0; }` but
/// out of range to index `a[i]` OOB, or `int a[N]` (`N = nondet()` in range) then a
/// later `nondet()` shuffle index that runs `a[r]` negative. Kept short (the run
/// count is `|sweep| × |argv|`, all reusing ONE compiled harness) so the pass stays
/// cheap; each triple is a distinct enter-then-violate shape. Sound: base/target are
/// ordinary `int` values, so any `ASan` mem-error is a genuine feasible execution and a
/// safe program faults for none (R1/R2 still gate the report).
const MEMSAFETY_SPLIT_SWEEP: &[(i64, i64, i64)] = &[
    (1, 0, 10),
    (1, 0, -1),
    (1, 3, -1),
    (1, 1, 100),
    (1, 0, 2_147_483_647),
    (2, 0, 10),
    (2, 3, -1),
];

/// Command-line argument vectors the memsafety threshold sweep runs the harness under.
/// SV-COMP treats `argc`/`argv` as NONDETERMINISTIC inputs (a valid, NUL-terminated
/// argument vector of some length), so a violation reachable via some command line is a
/// genuine violation. The empty vector (`argc == 1`) is the committed no-args behaviour;
/// the two-argument vector (`argc == 3`, argv[1..]=well-formed strings) lets an
/// `if (argc < 2) return;`-gated body execute. libc guarantees the vector is well-formed
/// (argc pointers to NUL-terminated strings, `argv[argc] == NULL`), so any fault is the
/// program's OWN bug — never a malformed-harness artifact — keeping the pass sound.
const MEMSAFETY_SPLIT_ARGV: &[&[&str]] = &[&[], &["a", "b"]];

/// Whole-pass wall-clock cap (seconds) for the memsafety threshold sweep — a hard
/// backstop so the pass can never dominate a currently-`unknown` task's budget even if
/// several runs hit the per-run replay timeout. A genuine confirmation traps fast (well
/// under the per-run timeout), so a confirming task always resolves long before this cap
/// bites; it only bounds wasted work on a non-confirming task (whose verdict is `unknown`
/// either way). Kept small so the added cost on the ~88% of tasks the base sweep already
/// resolves is ZERO (the pass never runs there) and the cost on the remainder is bounded.
const MEMSAFETY_SPLIT_WALL_SECS: u64 = 8;

/// `ASAN_OPTIONS` for the memsafety replay: deterministic exit (no `SIGABRT`/coredump),
/// leaks off (valid-memtrack deferred), printf checks off (SV-COMP does not count
/// libc `printf` string reads; keeps a benign printf artifact from aborting before a
/// real fault — suppressing reports can never add a false alarm, so it stays sound).
// Memory safety for the replay harness (the swap-less host OOM'd when a harness ballooned
// to tens of GB under the mini-fuzz's huge constants). Two ASan-native bounds:
// - `max_allocation_size_mb=1024`: a SINGLE allocation over 1 GB (e.g. the
//   `2_147_483_647` mini-fuzz constant as a `malloc`/VLA size) becomes a
//   "requested-allocation-size-exceeds-maximum" error and exit.
// - `hard_rss_limit_mb=3072`: ASan's own background RSS monitor kills the harness the
//   moment its RESIDENT memory crosses 3 GB — this catches the growth that
//   `max_allocation_size_mb` misses (a VLA / `calloc` / many-small-allocs / `memset`
//   loop that reached 34–49 GB in one run), far more reliably than an external poll.
// Both exit with an UNMAPPED error class that `parse_asan_report` abstains on (⇒
// `unknown`, never a false alarm), while the other mini-fuzz constants still probe the
// task. Keep per-task jobs so jobs × 3 GB stays well under host RAM (≤16 on the 62 GB VM).
const ASAN_OPTS: &str = "exitcode=1:abort_on_error=0:detect_leaks=0:check_printf=0:\
max_allocation_size_mb=1024:hard_rss_limit_mb=3072";

/// A second `ASAN_OPTIONS` variant that DISABLES the heap quarantine
/// (`quarantine_size_mb=0:thread_local_quarantine_size_kb=0`) on top of the same
/// bounds as [`ASAN_OPTS`]. Rationale: `ASan`'s default quarantine holds a freed
/// block out of circulation for a while, so `malloc` after a `free` returns a
/// FRESH address. A whole class of `valid-free` bugs only manifests when the
/// allocator RE-USES a just-freed address — e.g. `p=malloc(); free(p); q=malloc();
/// if ((intptr_t)q==(intptr_t)p) free(q); free(q);` double-frees exactly when `q`
/// lands on `p`'s recycled address (memsafety/`cmp-freed-ptr`), and freelist-LIFO
/// address recycling is a legal allocator behavior the verifier may pick. With the
/// quarantine on, the address never collides and `ASan` sees no bug; with it off the
/// recycle happens promptly and the genuine violation reproduces.
///
/// SOUND for FALSE-only (never adds a false alarm): shrinking the quarantine only
/// changes WHICH concrete addresses `malloc` returns and HOW SOON freed memory is
/// recycled — it never makes a safe program free a live/foreign pointer or index
/// out of bounds (per-allocation redzones are unaffected, so a genuine
/// buffer-overflow is still caught and a safe access still passes). Its only
/// downside is RECALL: a use-after-free READ can land on recycled (unpoisoned)
/// memory and go unreported — which is why this variant runs SECOND, after the
/// full default-quarantine sweep has had its chance to catch exactly those.
const ASAN_OPTS_NO_QUARANTINE: &str = "exitcode=1:abort_on_error=0:detect_leaks=0:check_printf=0:\
max_allocation_size_mb=1024:hard_rss_limit_mb=3072:quarantine_size_mb=0:thread_local_quarantine_size_kb=0";

/// `UBSAN_OPTIONS` for the `no-overflow` replay (plan 199, R6): a deterministic,
/// non-coredumping exit (`halt_on_error=1:abort_on_error=0` — the default
/// `abort_on_error` is platform-dependent, so pin it) plus a symbolized frame #0
/// (`print_stacktrace=1`) for the R1 verifier-abstraction rejection. Do NOT set
/// `external_symbolizer_path` — a bad value breaks symbolization; the runtime
/// auto-finds `llvm-symbolizer`/`addr2line`. The witness location comes from the
/// address-free `runtime error:` line, so stack-frame addresses never leak in.
const UBSAN_OPTS: &str = "halt_on_error=1:abort_on_error=0:print_stacktrace=1";

/// Mini-fuzz constants for the overflow confirmer: the [`NONDET_CONSTS`] spread plus
/// `INT_MIN` and `2^31` — load-bearing for `-INT_MIN`, `INT_MIN - 1`, and `INT_MIN / -1`
/// overflow (Slice-0 caught `id_b3_o2-1.c` only at `INT_MIN`). Kept SEPARATE from
/// `NONDET_CONSTS` so the committed R5 memsafety byte-for-byte behavior is unperturbed.
///
/// The large POSITIVE probes are `2^30` and `1.5e9` — both deliberately BELOW `INT_MAX`.
/// When a nondet drives a loop trip count, `for (i=0; i<=x; i++)` (Parts) or
/// `while (z>0) { x=x+1; z=z-1; }` (ESOP2008), the counter/accumulator reaches ~`x`; at
/// `x == INT_MAX` the next `+1` is a spurious `INT_MAX + 1` overflow that SV-COMP's
/// no-overflow benchmarks label TRUE (the termination-* families — 2 full-pool false
/// alarms, indistinguishable in the `UBSan` report from a genuine `x+1`-at-INT_MAX). The
/// false alarm requires the counter to reach EXACTLY `INT_MAX`, so any probe strictly
/// below `INT_MAX` is safe against it while still triggering genuine large-value overflows.
///
/// Two probes are needed because they catch different overflow shapes:
/// - `2^30` (`1_073_741_824`) catches products/doublings (`2^30 * 2`, `k*nondet()`) and
///   two-operand sums where one operand is small.
/// - `1.5e9` (`1_500_000_000`) catches a **direct two-operand ADDITION of two large nondet
///   operands** (`y = y + x` with `x, y` both near this value — the
///   `AliasDarteFeautrierGonnord`/`ChenFlurMukhopadhyay`/`PodelskiRybalchenko`
///   termination-literature idiom), which needs `2·v > INT_MAX`, i.e. `v > 2^30`. `2^30`
///   itself is too small (`2^30 + (2^30-1) = INT_MAX`, representable — no trap), so these
///   sums were previously missed. `1.5e9` sums to `~3e9 > INT_MAX` (traps) yet leaves a
///   `~6.4e8` margin below `INT_MAX`, so no `+k` loop counter with a realistic step can
///   reach `INT_MAX` under it (verified false-alarm-free across the reasoning TRUE set).
///
/// The only remaining loss is a direct `x+1`-EXACTLY-at-INT_MAX overflow, which cannot be
/// caught without re-admitting the loop false alarms — soundness (FP=0) is worth more.
const OVERFLOW_CONSTS: &[i64] = &[
    0,
    1,
    2,
    42,
    255,
    256,
    1024,
    65_535,
    1_073_741_824,
    1_500_000_000,
    -1,
    -2_147_483_648,
    2_147_483_648,
];

/// Cap on how many candidates to replay per task — bounds worst-case native
/// compile+run time; a real violation almost always surfaces in the first
/// candidate. Dropped candidates are logged implicitly by not confirming.
const MAX_REPLAY_CANDIDATES: usize = 16;

/// Cap on the overflow confirmer's candidate list. Larger than
/// [`MAX_REPLAY_CANDIDATES`] because the overflow sweep layers three sources —
/// the fixed [`OVERFLOW_CONSTS`] spread, the loop-free type-boundary values
/// ([`overflow_boundary_consts`]), and program-literal branch steering — and each
/// boundary value is a high-yield direct-overflow probe that must not be squeezed
/// out by steered literals. Loop-free programs (the only ones the boundary values
/// are added for) have no infinite loops, so the extra native runs stay cheap.
const OVERFLOW_MAX_CANDIDATES: usize = 24;

/// Data-nondet constants swept in the loop-sustaining bool pass (see the
/// `SAF_BOOL_CONST` decoupling in [`synthesize_asan_driver`]). This pass fixes
/// `__VERIFIER_nondet_bool()` to `1` so a `while (nondet_bool()) { … }` loop runs,
/// and sweeps the DATA nondets over this small list. `0` comes first because the
/// dominant idiom is an accumulator whose data variables must start at zero
/// (`if (!(i==0 && j==0)) return; while (nondet_bool()) i += ++x;`); the remaining
/// values cover small-nonzero and large-magnitude initialisations. Kept short so
/// the extra native runs stay bounded even on a TRUE task whose bool-guarded loop
/// is unbounded-but-safe (each candidate then spins to the per-run replay timeout).
const OVERFLOW_BOOL_SWEEP: &[i64] = &[0, 1, -1, 2, 1_073_741_824];

/// Max number of leading scalar-nondet call sites the positional overflow pass
/// targets (see [`ubsan_confirm`]). Bounds the pass to `POS_MAX × |targets| × 2`
/// native runs; small so a many-nondet TRUE task cannot spend the whole budget here.
/// Three positions cover the observed idioms (the overflowing variable is among the
/// first few nondets: a loop bound, an accumulator seed, or a negated operand).
const OVERFLOW_POS_MAX_INDEX: usize = 3;

/// Baselines the positional pass gives every NON-targeted scalar-int nondet: `0` (the
/// dominant `precondition sum==0 && i==0` idiom) and `1` (a precondition needing a
/// small non-zero value, e.g. `n>0 && n<10` with the loop bound as the baseline).
const OVERFLOW_POS_BASELINES: &[i64] = &[0, 1];

/// Whole-pass wall-clock cap (seconds) for the positional overflow sweep — a hard
/// backstop so the pass can never dominate a task's budget even if several runs hit
/// the per-run timeout. The deterministic bound is the run COUNT (positions × targets
/// × baselines); every run that WOULD confirm traps in well under the per-run timeout,
/// so a confirming task is always resolved long before this cap bites — the cap only
/// bounds the wasted work on a non-confirming explosive task (deep recursion / a safe
/// counted loop whose bound is a boundary value), whose verdict is `unknown` either
/// way. Kept modest so an explosive TRUE task cannot spend a big slice of its budget
/// here (the arm-66 cost-regression guard). The typical run is fast: a boundary value
/// rarely lands on a loop's bound, so almost every run either traps in <100 ms or
/// exits immediately — only the rare boundary-into-loop-bound run costs a full per-run
/// timeout. This backstop bounds the pathological all-slow-runs case; a genuine
/// confirmation (its trapping run is fast) always resolves well under it.
const OVERFLOW_POS_WALL_SECS: u64 = 6;

/// Per-run timeout for the positional overflow pass. Short by design: the pass pins
/// `nondet_bool` to 0 so no `while (nondet_bool())` loop is sustained, hence every run
/// either traps immediately (a direct boundary overflow), overflows a quadratic
/// accumulator within tens of thousands of iterations (< 100 ms), or exits — only a
/// boundary value injected into a *safe* counted loop's bound runs long, and this cap
/// bounds that rare case without masking any real trap (250 ms is >2× the slowest
/// productive overflow: a quadratic accumulator overflows within ~65 k iterations).
fn overflow_positional_timeout() -> std::time::Duration {
    std::time::Duration::from_millis(250)
}

/// Type-boundary values injected into a SINGLE targeted nondet call site by the
/// positional overflow pass, in try-order (high-yield first). The 32-bit boundaries
/// always apply; the 64-bit ones are added ONLY under LP64 — under ILP32 the driver's
/// `atol` parses into a 32-bit `long`, so a 64-bit literal would overflow the parse.
fn overflow_positional_targets(data_model: saf_svcomp::DataModel) -> Vec<i64> {
    let mut v = vec![2_147_483_647, -2_147_483_648, -1];
    if matches!(data_model, saf_svcomp::DataModel::LP64) {
        v.push(i64::MAX);
        v.push(i64::MIN);
    }
    v
}

/// Signed-overflow type-BOUNDARY nondet candidates, appended to the overflow
/// mini-fuzz sweep ONLY when every reachable loop is provably ranked
/// ([`saf_svcomp::fast_paths::module_reachable_loops_all_ranked`], which subsumes the
/// loop-free case).
///
/// [`OVERFLOW_CONSTS`] deliberately caps its large positive probe at `2^30` (not
/// `INT_MAX`) so a nondet-driven loop counter cannot be pushed to a *spurious*
/// `+1`-at-`INT_MAX` trap on a TRUE `termination-*` task. When every reachable loop
/// is **ranked** that hazard is gone: `loops_are_ranked` admits a loop only when its
/// induction variable's per-iteration update provably stays inside the type range
/// (so no counter can reach `INT_MAX` and overflow on the next step), so a
/// near-boundary input can only trigger a *genuine direct* overflow (`x+1` at
/// `INT_MAX`, `x*2`, a counted-loop sink `sum += INT_MAX`, or a recursive
/// `addition(m+1, …)` at `m == INT_MAX`). `UBSan` stays the sole arbiter (R2) and
/// re-triggers deterministically (R6); a value that does not actually overflow
/// simply yields no report.
///
/// The 32-bit boundaries always apply. The 64-bit boundaries are added ONLY under
/// LP64 (where `long`/`int64_t` sinks are 64-bit, closing the 64-bit direct-overflow
/// gap) — under ILP32 the driver's `atol` parses into a 32-bit `long`, so a 64-bit
/// literal would overflow the parse itself; excluding it keeps the driver defined.
fn overflow_boundary_consts(data_model: saf_svcomp::DataModel) -> Vec<i64> {
    // 32-bit boundaries: INT_MAX and its immediate neighbours. INT_MIN itself is
    // already in OVERFLOW_CONSTS; INT_MIN+1 lets `-x` / `x-1` near the low boundary
    // trap, and INT_MAX-1 covers `x+2` / `2*x` just under the high boundary.
    let mut v = vec![2_147_483_647, 2_147_483_646, -2_147_483_647];
    if matches!(data_model, saf_svcomp::DataModel::LP64) {
        // 64-bit boundaries for `long`/`long long`/`int64_t` sinks.
        v.extend_from_slice(&[
            9_223_372_036_854_775_807,  // LONG_MAX
            9_223_372_036_854_775_806,  // LONG_MAX - 1
            -9_223_372_036_854_775_807, // LONG_MIN + 1
        ]);
    }
    v
}

/// Yield-ordering key for an overflow mini-fuzz candidate: sorts by DESCENDING
/// magnitude, with `0` placed last.
///
/// A large-magnitude input drives a *direct* signed overflow on the first few
/// operations (`k*nondet()`, `nondet()+nondet()`, `nondet()*nondet()`), whereas a
/// small input rarely overflows a straight-line computation. Ordering the sweep so
/// the high-magnitude probes run FIRST is the load-bearing fix for
/// infinite/long-running loops (`while (1) { acc += 2*nondet(); }`, the CIL
/// `while(1)` state machines): every *non*-overflowing constant spins that loop to
/// the per-candidate replay timeout, so a productive constant buried behind a dozen
/// small ones is never reached before the task-level budget is spent — and the
/// confirmable FALSE is lost. Sorting is SOUND and DETERMINISTIC: it changes only
/// the ORDER of an unchanged constant SET; `UBSan` remains the sole R2 arbiter and
/// the trap is re-triggered on the original program (R6).
fn overflow_yield_rank(v: i64) -> (u8, std::cmp::Reverse<u64>) {
    // (bucket, key): bucket 0 = nonzero (sorted by descending magnitude), 1 = zero
    // (last). `unsigned_abs` avoids the `i64::MIN` `abs` overflow.
    if v == 0 {
        (1, std::cmp::Reverse(0))
    } else {
        (0, std::cmp::Reverse(v.unsigned_abs()))
    }
}

/// Assemble the overflow confirmer's mini-fuzz candidate list from three sources —
/// the fixed [`OVERFLOW_CONSTS`] spread, the type-boundary probes
/// ([`overflow_boundary_consts`], only when every reachable loop is provably ranked),
/// and program-literal branch steering — then order them by DESCENDING overflow
/// yield ([`overflow_yield_rank`]) so the high-magnitude, direct-overflow probes run
/// first. Deduplicated (first occurrence wins after the stable sort); bounded by
/// [`OVERFLOW_MAX_CANDIDATES`]. Deterministic (fixed spread + `BTreeSet`-ordered
/// steering, then a total order on the values).
fn overflow_replay_candidates(
    module: &saf_core::air::AirModule,
    data_model: saf_svcomp::DataModel,
) -> Vec<i64> {
    let mut candidates: Vec<i64> = OVERFLOW_CONSTS.to_vec();
    if saf_svcomp::fast_paths::module_reachable_loops_all_ranked(module) {
        for b in overflow_boundary_consts(data_model) {
            if !candidates.contains(&b) {
                candidates.push(b);
            }
        }
    }
    for k in saf_svcomp::fast_paths::branch_steering_constants(module) {
        if candidates.len() >= OVERFLOW_MAX_CANDIDATES {
            break;
        }
        if !candidates.contains(&k) {
            candidates.push(k);
        }
    }
    // Order by descending overflow yield so long-running loops reach a productive
    // constant before the per-candidate replay budget is exhausted (see
    // `overflow_yield_rank`). `sort_by_key` is a stable sort over a deduplicated,
    // deterministic input, so the result is a fixed total order.
    candidates.sort_by_key(|&v| overflow_yield_rank(v));
    candidates
}

/// Assemble the mini-fuzz candidate constant list for a native-replay confirmer:
/// the fixed `fixed` spread FIRST (so a confirmer's committed behavior is a
/// byte-for-byte prefix — 0 regression), then the program's OWN integer comparison /
/// switch literals appended via branch-steering (cpa-witness2test-style input
/// steering). A guard-gated fault (`if (nondet() == K) …`) is unreachable when the
/// fixed spread never guesses the guard constant `K`; feeding the program's own `K`
/// reaches it. Every steered value is just another concrete nondet input the verifier
/// may choose (so soundness is unchanged — the sanitizer remains the sole arbiter and
/// `__VERIFIER_assume` still prunes infeasible paths), harvested literals are
/// magnitude-capped and de-duplicated, and the total is bounded by
/// `MAX_REPLAY_CANDIDATES` to keep the per-task replay budget finite.
fn replay_candidates(fixed: &[i64], module: &saf_core::air::AirModule) -> Vec<i64> {
    let mut candidates: Vec<i64> = fixed.to_vec();
    for k in saf_svcomp::fast_paths::branch_steering_constants(module) {
        if candidates.len() >= MAX_REPLAY_CANDIDATES {
            break;
        }
        if !candidates.contains(&k) {
            candidates.push(k);
        }
    }
    candidates
}

/// Scalar-integer nondet functions and their C return types, in a fixed order.
/// The replay driver always defines all of them (so the native link succeeds
/// regardless of which the program references); the ones with model values
/// replay their sequence, the rest return `0`.
const SCALAR_NONDET: &[(&str, &str)] = &[
    ("__VERIFIER_nondet_int", "int"),
    ("__VERIFIER_nondet_uint", "unsigned int"),
    ("__VERIFIER_nondet_long", "long"),
    ("__VERIFIER_nondet_ulong", "unsigned long"),
    ("__VERIFIER_nondet_longlong", "long long"),
    ("__VERIFIER_nondet_ulonglong", "unsigned long long"),
    ("__VERIFIER_nondet_short", "short"),
    ("__VERIFIER_nondet_ushort", "unsigned short"),
    ("__VERIFIER_nondet_char", "char"),
    ("__VERIFIER_nondet_uchar", "unsigned char"),
    ("__VERIFIER_nondet_bool", "_Bool"),
    ("__VERIFIER_nondet_size_t", "size_t"),
];

/// Internal per-candidate replay timeout (seconds), overridable via
/// `$SAF_VERIFY_REPLAY_TIMEOUT`. Short by design so a runaway harness cannot eat
/// the whole wall-clock budget; on expiry the candidate is inconclusive.
fn replay_timeout() -> std::time::Duration {
    let secs = std::env::var("SAF_VERIFY_REPLAY_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(10);
    std::time::Duration::from_secs(secs)
}

/// Per-candidate replay timeout (seconds) for the `valid-memsafety` `ASan` sweep —
/// the base passes ([`asan_confirm`]), the threshold/argv pass
/// ([`asan_threshold_argv_sweep`]) and the byte-stream pass 3 ([`asan_fuzz_pass`]).
/// Overridable via `$SAF_MEMSAFETY_REPLAY_TIMEOUT`.
///
/// Deliberately SHORTER than the shared [`replay_timeout`] (4 s vs 10 s), because the
/// memsafety confirmer runs a MULTI-RUN sweep (two base passes × two `ASAN_OPTIONS` ×
/// the constant spread, then the threshold and byte-stream passes) that must all fit
/// inside the per-task wall-clock budget (60 s in the loop eval). An `ASan` memory
/// violation traps essentially instantly once the faulting access executes; the only
/// runs that approach the cap are NON-faulting ones that spin in a long/│unbounded loop
/// under a pathological constant (e.g. an array size of `INT_MAX`, or a self-referential
/// `while` that never advances). With the shared 10 s cap those wasted runs starve the
/// LATER passes (the uninitialised-variable pass 2, the threshold sweep, pass 3) that
/// would confirm the violation quickly — so a task that is trivially confirmable in
/// isolation times out at 60 s. A 4 s cap bounds each wasted run, leaving budget for the
/// confirming pass; measured, it recovers cost-bound tasks (e.g.
/// `array-memsafety/bubblesort_unsafe`, `termination-crafted/NonTermination3-1`) with no
/// observed recall loss on a broad sample.
///
/// SOUND (fail-closed): a shorter timeout can only kill a run EARLIER, so the strictly
/// worse case is a missing report ⇒ abstain (`unknown`). It can never turn a clean run
/// into a fault, so it never adds a false alarm — the risk is bounded to recall, and
/// only for the rare violation whose faulting access is reached only after >4 s of
/// native execution (billions of iterations), which cannot fit a 60 s multi-run sweep
/// anyway.
fn memsafety_replay_timeout() -> std::time::Duration {
    let secs = std::env::var("SAF_MEMSAFETY_REPLAY_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(4);
    std::time::Duration::from_secs(secs)
}

/// Escape a filesystem path for embedding in a C string literal.
fn escape_c_string(p: &Path) -> String {
    let mut out = String::new();
    for c in p.to_string_lossy().chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}

/// Build the replay driver C source for `candidate`.
///
/// It defines every SV-COMP special function the program may reference (so the
/// native link succeeds): the scalar-integer nondet generators replay the model
/// sequence (a per-function counter over a fixed array; exhausted ⇒ `0`);
/// pointer/float/double nondet return `0`/`NULL`; `__VERIFIER_assume` blocks
/// assumed-false paths at runtime (so an assume-pruned error can never be a
/// false confirmation); and `reach_error`/`__VERIFIER_error` drop the sentinel
/// file and `_exit`.
fn synthesize_driver(candidate: &saf_svcomp::FalseCandidate, sentinel: &Path) -> String {
    use std::fmt::Write as _;

    let mut s = String::new();
    let _ = writeln!(s, "/* slice-1c concrete-replay driver (generated) */");
    let _ = writeln!(
        s,
        "#define __SAF_SENTINEL \"{}\"",
        escape_c_string(sentinel)
    );
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdio.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");

    for (fname, cty) in SCALAR_NONDET {
        let suffix = fname.trim_start_matches("__VERIFIER_nondet_");
        let values: Vec<i64> = candidate
            .nondet_sequence
            .iter()
            .filter(|n| n.func_name == *fname)
            .map(|n| n.value)
            .collect();
        // A length-0 C array is illegal, so emit a dummy element when empty; the
        // count gate (`__saf_n`) makes it unreachable.
        let elems = if values.is_empty() {
            "0".to_string()
        } else {
            values
                .iter()
                .map(|v| format!("{v}LL"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(s, "static long long __saf_arr_{suffix}[] = {{ {elems} }};");
        let _ = writeln!(
            s,
            "static unsigned long __saf_n_{suffix} = {};",
            values.len()
        );
        let _ = writeln!(s, "static unsigned long __saf_i_{suffix} = 0;");
        let _ = writeln!(
            s,
            "{cty} __VERIFIER_nondet_{suffix}(void) {{ return (__saf_i_{suffix} < __saf_n_{suffix}) ? ({cty})__saf_arr_{suffix}[__saf_i_{suffix}++] : ({cty})0; }}"
        );
    }

    // Non-integer / pointer nondet: legal defaults so the program links + runs.
    s.push_str("void* __VERIFIER_nondet_pointer(void) { return (void*)0; }\n");
    s.push_str("float __VERIFIER_nondet_float(void) { return 0.0f; }\n");
    s.push_str("double __VERIFIER_nondet_double(void) { return 0.0; }\n");
    s.push_str("void __VERIFIER_atomic_begin(void) { }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { }\n");

    // Honour assumptions at runtime: assume(false) blocks the path WITHOUT a hit.
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");

    // Error sinks: dropping the sentinel is the sole evidence of a violation.
    s.push_str(
        "__attribute__((noreturn)) static void __saf_hit(void) { FILE* f = fopen(__SAF_SENTINEL, \"w\"); if (f) { fputc('1', f); fclose(f); } _exit(0); }\n",
    );
    // reach_error / __VERIFIER_error are WEAK: the canonical sv-benchmarks task
    // DEFINES its own `reach_error(){ __assert_fail(...); }`, which must win the
    // link (a strong symbol overrides our weak one); when the task only declares
    // reach_error, our weak definition supplies it. __assert_fail is overridden
    // (a safe libc override — libc is a shared object, so no multiple-definition)
    // so the task-defined `reach_error -> __assert_fail` path still trips the
    // sentinel. `abort` is deliberately NOT overridden: it is too generic to
    // attribute to the property soundly, so reach_error variants calling it
    // directly stay `unknown` (sound, at some recall cost).
    s.push_str("__attribute__((weak)) void reach_error(void) { __saf_hit(); }\n");
    s.push_str("__attribute__((weak)) void __VERIFIER_error(void) { __saf_hit(); }\n");
    s.push_str(
        "__attribute__((noreturn)) void __assert_fail(const char* a, const char* b, unsigned int c, const char* d) { (void)a; (void)b; (void)c; (void)d; __saf_hit(); }\n",
    );

    s
}

/// Confirm a FALSE candidate by concrete native execution.
///
/// Synthesizes the replay driver, compiles the ORIGINAL program natively
/// together with it (`clang -O0`, no `-emit-llvm`), and runs it under a short
/// timeout. The candidate is confirmed iff the run drops the sentinel — an
/// irrefutable, real execution reaching `reach_error`. Any other outcome (normal
/// exit, crash, timeout, compile/link failure) is inconclusive ⇒ `Ok(false)` ⇒
/// the caller keeps `unknown`. Never emits to stdout (subprocess I/O is nulled).
fn replay_confirms_false(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    stub: &Path,
    dir: &Path,
    clang: &str,
    idx: usize,
    candidate: &saf_svcomp::FalseCandidate,
) -> anyhow::Result<bool> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let sentinel = dir.join(format!("saf_reach_{idx}.sentinel"));
    let driver_src = dir.join(format!("saf_driver_{idx}.c"));
    let harness = dir.join(format!("saf_harness_{idx}"));

    std::fs::write(&driver_src, synthesize_driver(candidate, &sentinel))
        .with_context(|| "writing replay driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));
    // Two knobs vs a plain native compile (see fuzz_confirm_false): (a) a two-pass __VERIFIER_assert
    // neutralizer so a benchmark that DEFINES its own assert compiles; (b) -fsanitize-trap=signed-integer-
    // overflow so a candidate whose path reaches reach_error only via signed-overflow UB TRAPS before the
    // sentinel drops -> Ok(false) (inconclusive), NOT a wrong FALSE. Additive: a program that already
    // compiled + reaches reach_error without overflow is unchanged; a genuine link/compile failure still
    // returns Ok(false). stderr suppressed on both passes so the transient pass-1 macro error no longer
    // leaks into task diagnostics.
    let build = |neutralizer: Option<&Path>| {
        let mut cmd = Command::new(clang);
        cmd.args([
            "-O0",
            "-Wno-everything",
            "-fsanitize=signed-integer-overflow",
            "-fsanitize-trap=signed-integer-overflow",
        ])
        .arg(data_model.clang_flag())
        .arg("-include")
        .arg(stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let mut ok = build(None)
        .status()
        .with_context(|| format!("failed to spawn {clang} for native replay"))?
        .success();
    if !ok {
        let neutralizer = write_assert_neutralizer(dir)?;
        ok = build(Some(neutralizer.as_path()))
            .status()
            .with_context(|| format!("failed to spawn {clang} for native replay"))?
            .success();
    }
    if !ok {
        // Link/compile failure (e.g. the task inlines its own reach_error) —
        // inconclusive, not a violation.
        return Ok(false);
    }

    let mut cmd = Command::new(&harness);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = harden_replay_spawn(&mut cmd)
        .spawn()
        .with_context(|| "spawning replay harness")?;

    let timeout = replay_timeout();
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    kill_replay_group(&mut child);
                    return Ok(false); // runaway program → inconclusive
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(e) => return Err(e).context("waiting on replay harness"),
        }
    }

    // The sentinel is written only by our reach_error/__VERIFIER_error override,
    // so its presence is an irrefutable witness that the real run reached the
    // error call.
    Ok(sentinel.exists())
}

/// The `valid-memsafety` FALSE pipeline (plan 197, R5): confirmer-first.
///
/// Compiles the ORIGINAL program with `-fsanitize=address` and runs it under a
/// zeroed-nondet driver; emits `false(<sub-property>)` iff `AddressSanitizer` reports
/// a violation in the program's OWN code (R1) with a high-fidelity class (R2). `ASan`
/// is the sole arbiter, the witness-target source, and the sub-property classifier
/// — SAF's over-approximate memory checkers are not consulted. Never emits `true`;
/// a program whose violation does not reproduce (or is out of scope) → `unknown`.
fn memsafety_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    match asan_confirm(
        ctx.input,
        ctx.data_model,
        ctx.module,
        ctx.stub,
        ctx.tempdir,
        ctx.clang,
    ) {
        Ok(Some(hit)) => {
            let witness = build_witness(ctx, Some(saf_svcomp::lower_memsafety_hit(&hit)));
            if witness.is_none() {
                eprintln!(
                    "saf verify: FALSE (ASan {}) but witness unconstructible -> emitting false without a witness",
                    hit.subproperty
                );
            }
            VerdictOutcome {
                verdict: saf_svcomp::memsafety_verdict(hit.subproperty),
                witness,
                graphml: None,
            }
        }
        Ok(None) => {
            eprintln!("saf verify: ASan replay reproduced no memsafety violation -> unknown");
            unknown_outcome()
        }
        Err(e) => {
            eprintln!("saf verify: ASan replay errored: {e:#} -> unknown");
            unknown_outcome()
        }
    }
}

/// Build the `ASan`-replay driver: defines the SV-COMP nondet generators (all
/// returning the default `0`/`NULL` — Slice 1 is an UNSTEERED probe) and honours
/// `__VERIFIER_assume`. Unlike [`synthesize_driver`], it installs NO `reach_error`
/// sentinel and does NOT override `malloc`/`free`/`memcpy` (`ASan` intercepts those,
/// and a memory-safety fault is intrinsic to the program).
fn synthesize_asan_driver() -> String {
    use std::fmt::Write as _;

    let mut s = String::new();
    let _ = writeln!(s, "/* R5 ASan-replay driver (generated) */");
    s.push_str("#include <stddef.h>\n");
    s.push_str("#include <stdlib.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    // Every scalar nondet returns the constant chosen at RUNTIME via $SAF_NONDET_CONST
    // (the multi-constant mini-fuzz — one binary run under many constants), default 0.
    s.push_str(
        "static long __saf_c(void) { const char *e = getenv(\"SAF_NONDET_CONST\"); return e ? atol(e) : 0; }\n",
    );
    // Positional per-call targeting (overflow confirmer's positional pass): a global
    // call counter lets ONE nondet call site return a distinct `SAF_TARGET_VAL` while
    // every OTHER scalar-int nondet returns the small `SAF_BASE_VAL` (default 0). This
    // synthesizes the DIFFERENT-value-per-call inputs a single shared
    // `SAF_NONDET_CONST` cannot — e.g. a precondition pins `sum==0 && i==0` while the
    // loop bound `n` must be large to overflow the accumulator. When `SAF_TARGET_IDX`
    // is UNSET (every existing pass: memsafety, the primary overflow sweep, the bool
    // pass) `__saf_nv()` returns exactly `__saf_c()`, so committed behaviour is
    // unchanged. Sound: each returned value is a legal value of the nondet's type, so
    // any resulting UBSan trap is a genuine feasible execution (R2/R6); the counter
    // side effect is unobservable when untargeted.
    s.push_str("static long __saf_idx = 0;\n");
    // SAF_SPLIT_IDX threshold mode (memsafety confirmer's positional/threshold pass):
    // when set, the FIRST `SAF_SPLIT_IDX` scalar-nondet calls return `SAF_BASE_VAL`
    // (the in-range "setup" value that satisfies an entry guard / fills an array) and
    // EVERY LATER call returns `SAF_TARGET_VAL` (the out-of-range value that drives the
    // violation). This synthesizes the DIFFERENT-value-per-phase inputs a single shared
    // `SAF_NONDET_CONST` cannot — e.g. `int i = nondet(); ...; while (guard(i)) { i =
    // nondet(); a[i] = 0; }` needs `i` in range to ENTER the loop but out of range to
    // index OOB, and `int a[N]` with `N = nondet()` then a later `nondet()` shuffle index.
    // Checked BEFORE `SAF_TARGET_IDX`, so when `SAF_SPLIT_IDX` is UNSET this is
    // byte-identical to the committed behaviour (the overflow positional pass and every
    // other pass never set it). Sound: each returned value is a legal value of the
    // nondet's type, so any resulting ASan mem-error is a genuine feasible execution
    // (R1/R2 in `parse_asan_report` still gate), and a safe program faults for none.
    s.push_str(
        "static long __saf_nv(void) {\n\
         \x20 long idx = __saf_idx++;\n\
         \x20 const char *sp = getenv(\"SAF_SPLIT_IDX\");\n\
         \x20 if (sp) {\n\
         \x20   const char *bv = getenv(\"SAF_BASE_VAL\"); const char *tv = getenv(\"SAF_TARGET_VAL\");\n\
         \x20   return idx < atol(sp) ? (bv ? atol(bv) : 0) : (tv ? atol(tv) : 0);\n\
         \x20 }\n\
         \x20 const char *ti = getenv(\"SAF_TARGET_IDX\");\n\
         \x20 if (!ti) return __saf_c();\n\
         \x20 if (idx == atol(ti)) { const char *tv = getenv(\"SAF_TARGET_VAL\"); return tv ? atol(tv) : 0; }\n\
         \x20 const char *bv = getenv(\"SAF_BASE_VAL\"); return bv ? atol(bv) : 0;\n\
         }\n",
    );
    for (fname, cty) in SCALAR_NONDET {
        if *fname == "__VERIFIER_nondet_bool" {
            // Loop-sustaining bool decoupling (overflow confirmer): when `SAF_BOOL_CONST`
            // is set, `nondet_bool()` returns that fixed 0/1 value INDEPENDENT of
            // `SAF_NONDET_CONST`. This lets an accumulator loop `while (nondet_bool())
            // { … }` be driven to run even when the program's data nondets must take a
            // DIFFERENT value (e.g. `if (!(i==0 && j==0)) return; while (nondet_bool())
            // { i += ++x; }` needs `i==0` AND the guard true — impossible with one shared
            // constant). Every returned bool is a value `__VERIFIER_nondet_bool` is
            // allowed to return, so a resulting overflow is a genuine feasible execution
            // and UBSan stays the sole sound arbiter (R2, R6). When neither
            // `SAF_BOOL_CONST` nor `SAF_TARGET_IDX` is set — the ASan/memsafety path and
            // the primary overflow sweep — this is byte-identical to the previous
            // `(_Bool)__saf_c()` behaviour. In the positional pass (`SAF_TARGET_IDX`
            // set) the bool is pinned to 0 unless it IS the targeted call, so no
            // `while (nondet_bool())` loop is sustained and every positional run
            // terminates fast. The counter is advanced in every branch so a bool call
            // occupies its execution-order index (keeping int/bool targeting aligned).
            s.push_str(
                "_Bool __VERIFIER_nondet_bool(void) {\n\
                 \x20 const char *b = getenv(\"SAF_BOOL_CONST\");\n\
                 \x20 if (b) { __saf_idx++; return (_Bool)(atoi(b) & 1); }\n\
                 \x20 const char *ti = getenv(\"SAF_TARGET_IDX\");\n\
                 \x20 if (!ti) { __saf_idx++; return (_Bool)__saf_c(); }\n\
                 \x20 long idx = __saf_idx++;\n\
                 \x20 if (idx == atol(ti)) { const char *tv = getenv(\"SAF_TARGET_VAL\"); return (_Bool)((tv ? atol(tv) : 0) & 1); }\n\
                 \x20 return (_Bool)0;\n\
                 }\n",
            );
            continue;
        }
        let suffix = fname.trim_start_matches("__VERIFIER_nondet_");
        let _ = writeln!(
            s,
            "{cty} __VERIFIER_nondet_{suffix}(void) {{ return ({cty})__saf_nv(); }}"
        );
    }
    s.push_str("void* __VERIFIER_nondet_pointer(void) { return (void*)0; }\n");
    s.push_str("float __VERIFIER_nondet_float(void) { return 0.0f; }\n");
    s.push_str("double __VERIFIER_nondet_double(void) { return 0.0; }\n");
    s.push_str("void __VERIFIER_atomic_begin(void) { }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { }\n");
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");
    // Determinism (reproducibility is an SV-COMP requirement): Juliet's `*_rand_*`
    // variants call `srand(time(NULL))` and derive buffer indices/sizes from `rand()`,
    // so a native replay's verdict flips run-to-run. Intercept both via the linker's
    // `--wrap` (see the `-Wl,--wrap=rand/srand` compile flags): `rand()` is tied to the
    // mini-fuzz constant (`SAF_NONDET_CONST`), so the constant sweep deterministically
    // probes rand-driven indices instead of a time seed; `srand()` becomes a no-op.
    s.push_str("int __wrap_rand(void) { return (int)__saf_c(); }\n");
    s.push_str("void __wrap_srand(unsigned s) { (void)s; }\n");
    s
}

/// Confirm a `valid-memsafety` FALSE by `AddressSanitizer`-instrumented native
/// execution.
///
/// Compiles the ORIGINAL program with `-fsanitize=address -g` + the zeroed-nondet
/// driver, runs it under a short timeout capturing stderr to a file, and parses the
/// report ([`saf_svcomp::parse_asan_report`], which applies R1/R2). `Ok(Some(hit))`
/// is a confirmed violation; `Ok(None)` is inconclusive (no report / abstained /
/// compile-link failure / timeout) ⇒ the caller keeps `unknown`. Multithreaded
/// programs are out of R5 scope (schedule-dependent memory safety) and abstain up
/// front.
///
/// Two replay passes run in sequence, both through the same R1/R2 report gate:
/// pass 1 is the committed stack-garbage replay; pass 2 (only when pass 1 is
/// inconclusive) adds `-ftrivial-auto-var-init=pattern` so a deref of an
/// uninitialized local reproduces deterministically. Pass 2 is sound-additive — an
/// uninitialized read is nondeterministic under SV-COMP semantics, so it can only
/// confirm a violation the program genuinely has.
// NOTE: a single cohesive replay pipeline — driver synthesis, the compile/link closure
// (with its assert-macro fallback), the two base sweeps, and the threshold+argv sweep —
// whose stages share the harness path, candidate list, and timeout; splitting it would
// scatter that shared setup across helpers and obscure the pass ordering.
#[allow(clippy::too_many_lines)]
fn asan_confirm(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    module: &saf_core::air::AirModule,
    stub: &Path,
    dir: &Path,
    clang: &str,
) -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    // Concurrency is out of scope (sequential valid-deref/valid-free); a threaded
    // program's memory safety can be schedule-dependent, so abstain rather than risk
    // a schedule-specific false alarm. The gate fires only on an ACTUALLY-reachable
    // thread spawn (plan 198) — NOT the mere presence of a `pthread_create` symbol,
    // which is dead scaffolding in the sv-benchmarks Juliet reservoir (the sink runs
    // in `main`; `pthread_create` is unreachable). No reachable spawn ⇒ the execution
    // is sequential ⇒ ASan's single run is schedule-independent ⇒ confirming is sound.
    let callgraph = saf_analysis::callgraph::CallGraph::build(module);
    if saf_svcomp::fast_paths::reachable_spawns_threads(module, &callgraph) {
        eprintln!("saf verify: a thread spawn is reachable from main (out of scope) -> unknown");
        return Ok(None);
    }

    let driver_src = dir.join("saf_asan_driver.c");
    let harness = dir.join("saf_asan_harness");
    let errpath = dir.join("saf_asan_stderr.txt");

    std::fs::write(&driver_src, synthesize_asan_driver())
        .with_context(|| "writing ASan replay driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));
    let timeout = memsafety_replay_timeout();

    // Mini-fuzz candidate constants: the fixed NONDET_CONSTS spread followed by the
    // program's own branch-steering literals (see `replay_candidates`). A guard-gated
    // memory fault (`if (nondet() == K) buf[BIG] = 0;`) that the fixed spread never
    // reaches is reproduced by feeding the program's own guard constant `K`. Sound: the
    // steered values are ordinary concrete inputs, ASan stays the sole arbiter, and
    // R1/R2 in `parse_asan_report` still gate the report — a safe program never faults.
    let candidates = replay_candidates(NONDET_CONSTS, module);

    // One ASan compile-and-mini-fuzz pass. `auto_var_init` toggles the second
    // (uninitialized-variable) pass; see the two-pass rationale at the call below.
    // Returns `Ok(Some(hit))` on the first constant that reproduces a violation,
    // `Ok(None)` if this pass is inconclusive (compile/link failure or no trap).
    let run_pass = |auto_var_init: bool| -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
        // Build the ASan compile+link; `assert_neutralizer` (when Some) is the two-pass
        // assert-macro fallback (see `write_assert_neutralizer`) so a program that
        // defines its own `void __VERIFIER_assert(int)` — the `loops`/`array-examples`/
        // `memsafety` reasoning idiom — links its replay binary instead of failing on
        // the stub's `__VERIFIER_assert` MACRO. `auto_var_init` toggles the second
        // (uninitialized-variable) pass.
        let build_compile = |assert_neutralizer: Option<&Path>| {
            let mut cmd = Command::new(clang);
            cmd.args([
                "-O0",
                "-g",
                "-fsanitize=address",
                "-fno-sanitize-recover=address",
                "-Wno-everything",
                // Determinism: redirect rand()/srand() to the driver's __wrap_* stubs.
                "-Wl,--wrap=rand",
                "-Wl,--wrap=srand",
            ]);
            if auto_var_init {
                // Pass 2: make every otherwise-uninitialized local a fixed, non-canonical
                // bit pattern (0xAA…) instead of whatever stack garbage happened to be
                // there. This is deterministic (reproducibility, R6) and SOUND for
                // `valid-memsafety`: an uninitialized read is nondeterministic under
                // SV-COMP semantics, so a program that dereferences (or indexes with) an
                // indeterminate value ALREADY violates the property — the pattern is one
                // concrete value the verifier may pick. It never converts a safe program
                // into a violation (a safe program does not dereference indeterminate
                // memory). R1/R2 in `parse_asan_report` still gate the report.
                cmd.arg("-ftrivial-auto-var-init=pattern");
            }
            cmd.arg(data_model.clang_flag()).arg("-include").arg(stub);
            if let Some(neutralizer) = assert_neutralizer {
                cmd.arg("-include").arg(neutralizer);
            }
            cmd.arg("-I")
                .arg(srcdir)
                .arg(input)
                .arg(&driver_src)
                .arg("-o")
                .arg(&harness)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            cmd
        };
        let mut linked = build_compile(None)
            .status()
            .with_context(|| format!("failed to spawn {clang} for ASan replay"))?
            .success();
        if !linked {
            // Assert-macro fallback: `#undef` the stub's `__VERIFIER_assert` macro via a
            // second `-include` so a program providing its own definition compiles
            // (mirrors ingestion + the UBSan replay). Purely additive — a program that
            // already linked is byte-for-byte unchanged.
            let neutralizer = write_assert_neutralizer(dir)?;
            linked = build_compile(Some(&neutralizer))
                .status()
                .with_context(|| format!("failed to spawn {clang} for ASan replay"))?
                .success();
        }
        if !linked {
            // Compile/link failure (e.g. a missing 32-bit ASan runtime) -> inconclusive.
            return Ok(None);
        }

        // Multi-constant mini-fuzz (Slice 2): the nondet generators return
        // $SAF_NONDET_CONST, so one binary is run under a spread of constants — a
        // scalar-guarded/scalar-sized fault (e.g. `if (nondet()==42) OOB`, or a
        // nondet-sized alloc/index) that the zeroed probe misses is reproduced by the
        // matching constant. Sound: each constant is a valid concrete input the verifier
        // may choose, and __VERIFIER_assume still prunes infeasible ones. 0 first (the
        // common unconditional case); confirm on the FIRST trap.
        //
        // check_printf=0: SV-COMP valid-memsafety does not count a libc printf("%s")
        // string read (ASan's printf_common interceptor from Juliet's printLine on a
        // non-terminated buffer). Suppressing it — in addition to the R1 frame filter —
        // stops ASan aborting at a benign printf artifact BEFORE the real fault; it can
        // never add a false alarm (only suppresses reports), so it stays sound. R2 still
        // abstains on any ambiguous secondary fault a suppressed intended-fault exposes.
        // Sweep the mini-fuzz constants twice: first under the default
        // (quarantine-on) `ASAN_OPTS`, then under `ASAN_OPTS_NO_QUARANTINE`. The
        // default sweep runs to completion FIRST so the committed behavior is
        // byte-for-byte unchanged for any task it already confirms; the
        // no-quarantine sweep is purely ADDITIVE, reproducing the address-recycle
        // class of `valid-free` bugs the quarantine masks (see the const doc). Both
        // reuse the SAME compiled binary — only the runtime option string differs —
        // so this adds no compile cost. R1/R2 in `parse_asan_report` still gate.
        for opts in [ASAN_OPTS, ASAN_OPTS_NO_QUARANTINE] {
            for &k in &candidates {
                // Redirect the child's stderr to a FILE (not a pipe) so a large ASan
                // report cannot deadlock on a full pipe buffer while we poll for the
                // timeout.
                let errfile =
                    std::fs::File::create(&errpath).with_context(|| "creating ASan stderr file")?;
                let mut cmd = Command::new(&harness);
                cmd.stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::from(errfile))
                    .env("ASAN_OPTIONS", opts)
                    .env("SAF_NONDET_CONST", k.to_string());
                let mut child = harden_replay_spawn(&mut cmd)
                    .spawn()
                    .with_context(|| "spawning ASan harness")?;

                let start = std::time::Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) => {
                            if start.elapsed() >= timeout {
                                kill_replay_group(&mut child);
                                break; // runaway -> parse whatever exists (likely no report)
                            }
                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(e) => return Err(e).context("waiting on ASan harness"),
                    }
                }

                let report = std::fs::read_to_string(&errpath).unwrap_or_default();
                if let Some(hit) = saf_svcomp::parse_asan_report(&report) {
                    return Ok(Some(hit)); // first (opts, constant) that reproduces wins
                }
            }
        }
        Ok(None)
    };

    // Pass 1 is the committed, byte-for-byte-unchanged R5 replay (stack garbage as-is).
    if let Some(hit) = run_pass(false)? {
        return Ok(Some(hit));
    }
    // Pass 2 (additive, uninitialized-variable replay): only when pass 1 is
    // inconclusive. Many array/string reasoning tasks (`array-memsafety`,
    // `ldv-memsafety`, …) dereference or index through a local that `main` never
    // initializes — `int *a; foo(a, n);` or `char *s1; cstrcat(s1, s2);`. With real
    // stack garbage the fault is nondeterministic (may or may not SEGV run-to-run);
    // pattern-init makes the indeterminate pointer a fixed wild address that SEGVs
    // deterministically, so ASan reproduces the genuine `valid-deref` violation. It
    // can only ADD confirmations, never a false alarm (see the pass-2 rationale above).
    if let Some(hit) = run_pass(true)? {
        return Ok(Some(hit));
    }

    // Threshold + argv positional sweep (additive; only reached when BOTH base passes
    // are inconclusive, so it costs nothing on the tasks the simple sweep already
    // resolves). Reuses the harness the last `run_pass` compiled — a valid ASan binary
    // whether it is the pass-1 (stack-garbage) or pass-2 (pattern-init) build — so it
    // adds NO compile cost, only a small, wall-capped set of extra native runs. It
    // recovers the multi-nondet enter-then-violate class (an early nondet must be
    // in-range to reach the sink, a later nondet out-of-range to violate) and the
    // `argc`-gated class (`if (argc < 2) return;`), neither of which the single-shared-
    // constant sweep can reach. Sound: the threshold values and the well-formed argv are
    // legal nondet inputs, ASan stays the sole arbiter, and R1/R2 still gate the report.
    if harness.exists() {
        if let Some(hit) = asan_threshold_argv_sweep(&harness, &errpath)? {
            return Ok(Some(hit));
        }
    }

    // Pass 3 (byte-stream coverage-guided greybox mini-fuzz; additive, only reached when
    // every earlier pass is inconclusive). The uniform/positional-split passes give every
    // scalar nondet call the SAME value (or one leading/trailing split), so a violation
    // gated behind two-or-more INDEPENDENTLY-valued nondet inputs (`n=nondet()` sizes an
    // array; a LATER `idx=nondet()` indexes it OOB) stays unreached. Pass 3 compiles a
    // separate ASan harness with a byte-stream nondet shim (each call draws its own bytes
    // from an AFL-mutated buffer) and runs a deterministic mutation loop; the FIRST input
    // that makes ASan trap in the program's own code — through the SAME unmodified R1/R2
    // `parse_asan_report` gate — is the confirmed FALSE. Sound: ASan is the sole arbiter,
    // the byte-stream runs the ORIGINAL program, and the hit is re-triggered on the same
    // input before emitting (R6). It costs one extra compile + a wall-capped set of native
    // runs, borne only by tasks the cheaper passes already left `unknown`.
    asan_fuzz_pass(input, data_model, module, stub, dir, clang)
}

/// Byte-stream depth-guided greybox `ASan` mini-fuzz (memsafety pass 3, lever
/// `mem-fuzz-covguided`). Compiles the ORIGINAL program with the
/// [`saf_svcomp::fuzz::synthesize_bytestream_asan_shim`] driver ONCE, then runs a
/// deterministic AFL-style mutation loop (dictionary = harvested IR constants + AFL
/// "interesting" values; seed corpus tiles each). Each input is fed via `$SAF_FUZZ_INPUT`;
/// the harness's `ASan` stderr is parsed by the UNMODIFIED [`saf_svcomp::parse_asan_report`]
/// R1/R2 gate. On the first gated hit the SAME input is replayed and required to reproduce
/// the identical hit (R6) before returning `Ok(Some(hit))`; otherwise `Ok(None)` (abstain).
///
/// Gate: skip unless the program has **≥2 scalar `__VERIFIER_nondet_*` call sites**.
/// That is precisely the class the cheaper passes cannot reach — the uniform sweep gives
/// every call the same shared constant and the threshold pass one leading/trailing split,
/// so a violation needing two or more call sites to each take a DISTINCT value stays
/// unreached. With fewer than two sites the byte stream can only reproduce the single
/// value the earlier passes already swept, so pass 3 adds nothing; skipping there also
/// keeps its (one extra compile + wall-capped native runs) cost off the large majority of
/// memsafety tasks — the aggregate-cost blowup that must be avoided.
///
/// Sound & fail-closed: `ASan` is the sole FALSE arbiter (the shim installs no sentinel
/// sink — an assertion failure is not a memory-safety violation), every byte-stream value
/// is a legal input of its nondet type (R5), `__VERIFIER_assume` hard-rejects infeasible
/// paths (R4), the compile honours the task's data model (R3), and the search is fully
/// deterministic (fixed `XorShift64` seed) so the verdict is reproducible.
// NOTE: the compile-once / seed / mutate / confirm loop is one cohesive fail-closed unit,
// mirroring `fuzz_confirm_false`; splitting it would scatter the shared harness setup.
#[allow(clippy::too_many_lines)]
fn asan_fuzz_pass(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    module: &saf_core::air::AirModule,
    stub: &Path,
    dir: &Path,
    clang: &str,
) -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
    use anyhow::Context;
    use saf_svcomp::fuzz;
    use std::process::{Command, Stdio};

    // Gate: pass 3 targets ONLY the multi-independent-nondet class. With fewer than two
    // scalar nondet call sites the byte stream reproduces the same single value the
    // uniform / threshold passes already swept, so it can add nothing — and skipping keeps
    // its extra compile + native-run cost off the memsafety majority (see the fn doc).
    if fuzz::count_scalar_nondet_call_sites(module) < 2 {
        return Ok(None);
    }

    // SOUNDNESS GATE (fail-closed): abstain when a dynamically-sized stack allocation
    // (VLA / `alloca(n)` ⇒ `Alloca { size_bytes: None }`) is reachable from main. Under
    // SV-COMP's UNBOUNDED-abstract-stack model such an allocation is safe for any size,
    // but a coverage-guided byte-stream search that drives the nondet size large exhausts
    // the concrete 8 MB native stack — AddressSanitizer then reports a stack-exhaustion
    // fault (`stack-overflow` / stack-region `SEGV` / `dynamic-stack-buffer-overflow`)
    // that is an artifact of the bounded native stack, NOT a violation of the program
    // under test. Emitting `false` on it is a false alarm on a correct-TRUE task
    // (`array-memsafety/{openbsd_cmemchr,subseq}-alloca-*` are exactly this shape). Only
    // the aggressive fuzz pass can wander into the exhaustion regime, so it alone gates
    // out; the cheaper passes (which never drive the size to a stack-exhausting magnitude)
    // still handle a genuine violation in such a program. Fixed-size allocas are unaffected.
    let callgraph = saf_analysis::callgraph::CallGraph::build(module);
    if saf_svcomp::fast_paths::reachable_has_dynamic_alloca(module, &callgraph) {
        eprintln!(
            "saf verify: byte-stream ASan fuzz skipped — reachable dynamic alloca (native \
             stack-exhaustion false-alarm class) -> unknown"
        );
        return Ok(None);
    }

    let driver_src = dir.join("saf_asan_fuzz_driver.c");
    let harness = dir.join("saf_asan_fuzz_harness");
    let errpath = dir.join("saf_asan_fuzz_stderr.txt");
    let input_path = dir.join("saf_asan_fuzz.input");
    let log_path = dir.join("saf_asan_fuzz.log");

    std::fs::write(&driver_src, fuzz::synthesize_bytestream_asan_shim())
        .with_context(|| "writing ASan byte-stream fuzz driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));

    // Compile the shim + original program ONCE with PLAIN ASan — NO SanitizerCoverage.
    // The instrumented double-compile + slower instrumented binary is exactly the
    // aggregate-cost blowup that regressed the earlier full-fat version of this pass; the
    // slice-directed dictionary + execution-depth-proxy corpus feedback steer the search
    // adequately without it. Mirrors `asan_confirm`'s flags (data model per R3, rand/srand
    // linker-wrap for determinism) plus the two-pass `__VERIFIER_assert`-macro neutralizer
    // fallback so own-assert programs still link.
    let build = |neutralizer: Option<&Path>| {
        let mut cmd = Command::new(clang);
        cmd.args([
            "-O0",
            "-g",
            "-fsanitize=address",
            "-fno-sanitize-recover=address",
            "-Wno-everything",
            "-Wl,--wrap=rand",
            "-Wl,--wrap=srand",
        ]);
        cmd.arg(data_model.clang_flag()).arg("-include").arg(stub);
        if let Some(n) = neutralizer {
            cmd.arg("-include").arg(n);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let mut linked = matches!(build(None).status(), Ok(s) if s.success());
    if !linked {
        if let Ok(n) = write_assert_neutralizer(dir) {
            linked = matches!(build(Some(&n)).status(), Ok(s) if s.success());
        }
    }
    if !linked {
        return Ok(None); // link/compile failure -> inconclusive
    }

    // Slice-directed steering (identical to the unreach fuzzer): the backward slice from
    // the `__VERIFIER_assume` guards front-loads their constants into the dictionary and
    // lays multi-guard chains as sequence seeds — so a multi-nondet OOB gated behind
    // distinct-valued guards (`assume(i>=40); assume(n<=15); a[i]=…`) is steered toward,
    // which single-value tiling cannot reach. An empty slice reduces to the plain harvest.
    let slice = saf_svcomp::slicing::backward_slice(module);
    let dict = saf_svcomp::slicing::slice_directed_dictionary(module, &slice);
    let mut corpus = fuzz::seed_corpus(&dict);
    corpus.extend(saf_svcomp::slicing::sequence_seeds(&slice.guard_constants));
    // Fixed seed -> the whole search (and therefore the verdict) is reproducible.
    let mut rng = fuzz::XorShift64::new(0x5AF3_C0DE);
    let per_run = memsafety_replay_timeout();
    let iters = mem_fuzz_iters();
    let deadline = std::time::Instant::now() + mem_fuzz_time_budget();
    let mut max_depth = 0usize;

    // A one-shot ASan run of `input` capturing stderr to `errpath` (and the depth-proxy
    // log), returning the gated hit (if any). Reused for the trial and the R6 re-confirm
    // so both go through the identical run + parse path.
    let run_once = |the_input: &[u8]| -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
        std::fs::write(&input_path, the_input).with_context(|| "writing ASan fuzz input")?;
        let errfile =
            std::fs::File::create(&errpath).with_context(|| "creating ASan fuzz stderr file")?;
        let mut cmd = Command::new(&harness);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errfile))
            .env("ASAN_OPTIONS", ASAN_OPTS)
            .env("SAF_FUZZ_INPUT", &input_path)
            .env("SAF_FUZZ_LOG", &log_path);
        let mut child = harden_replay_spawn(&mut cmd)
            .spawn()
            .with_context(|| "spawning ASan fuzz harness")?;
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= per_run {
                        kill_replay_group(&mut child);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Err(e) => return Err(e).context("waiting on ASan fuzz harness"),
            }
        }
        let report = std::fs::read_to_string(&errpath).unwrap_or_default();
        Ok(saf_svcomp::parse_asan_report(&report))
    };

    // Trials: the seed corpus verbatim first, then dictionary-steered mutations.
    let total = iters + corpus.len();
    for i in 0..total {
        if std::time::Instant::now() >= deadline {
            break;
        }
        let the_input: Vec<u8> = if i < corpus.len() {
            corpus[i].clone()
        } else {
            // Frontier-biased energy: favour the newest (deepest) corpus bases.
            let base = &corpus[rng.below_biased_high(corpus.len())];
            fuzz::mutate(&mut rng, base, &dict)
        };
        let _ = std::fs::remove_file(&log_path);

        let Ok(hit) = run_once(&the_input) else {
            continue;
        };
        let Some(hit) = hit else {
            // Greybox corpus feedback without instrumentation: keep inputs that drove the
            // program DEEPER (consumed more nondet bytes), a lightweight execution-depth
            // proxy that biases mutation toward the deep multi-nondet violation states.
            if i >= corpus.len() && corpus.len() < MAX_FUZZ_CORPUS {
                let depth = std::fs::read_to_string(&log_path)
                    .map(|l| l.lines().count())
                    .unwrap_or(0);
                if depth > max_depth {
                    max_depth = depth;
                    corpus.push(the_input);
                }
            }
            continue;
        };

        // R6: re-trigger the violation deterministically on the SAME input before
        // emitting. A genuine memory bug reproduces byte-identically (same sub-property,
        // file, and line); anything that does not is treated as flaky and abstained.
        match run_once(&the_input) {
            Ok(Some(hit2)) if hit2 == hit => {
                eprintln!(
                    "saf verify: byte-stream ASan fuzz reproduced {} (trial {i}); re-confirmed -> false",
                    hit.subproperty
                );
                return Ok(Some(hit));
            }
            _ => {
                eprintln!(
                    "saf verify: ASan fuzz hit at trial {i} did not re-confirm deterministically -> continue"
                );
            }
        }
    }

    eprintln!("saf verify: byte-stream ASan fuzz exhausted (no confirmed violation) -> unknown");
    Ok(None)
}

/// Reuse an already-compiled `ASan` harness to sweep the memsafety threshold cases
/// ([`MEMSAFETY_SPLIT_SWEEP`]) under each argument vector ([`MEMSAFETY_SPLIT_ARGV`]),
/// returning the first `(split, base, target, argv)` combination that reproduces a
/// gated `ASan` mem-error. `Ok(None)` if none does (⇒ the caller keeps `unknown`). The
/// whole pass is wall-capped by [`MEMSAFETY_SPLIT_WALL_SECS`]; individual runs by
/// [`replay_timeout`]. Only `SAF_SPLIT_IDX`/`SAF_BASE_VAL`/`SAF_TARGET_VAL` are set
/// (never `SAF_NONDET_CONST`), so the driver's threshold branch governs every scalar
/// nondet and `__VERIFIER_nondet_bool` stays `false` (no unbounded bool loop is spun).
fn asan_threshold_argv_sweep(
    harness: &Path,
    errpath: &Path,
) -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let per_run = memsafety_replay_timeout();
    let pass_deadline =
        std::time::Instant::now() + std::time::Duration::from_secs(MEMSAFETY_SPLIT_WALL_SECS);

    for &(split, base, target) in MEMSAFETY_SPLIT_SWEEP {
        for argv in MEMSAFETY_SPLIT_ARGV {
            if std::time::Instant::now() >= pass_deadline {
                return Ok(None); // whole-pass wall cap — bound wasted work on a non-confirming task
            }
            let errfile = std::fs::File::create(errpath)
                .with_context(|| "creating ASan threshold stderr file")?;
            let mut cmd = Command::new(harness);
            cmd.args(*argv)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::from(errfile))
                .env("ASAN_OPTIONS", ASAN_OPTS)
                .env("SAF_SPLIT_IDX", split.to_string())
                .env("SAF_BASE_VAL", base.to_string())
                .env("SAF_TARGET_VAL", target.to_string());
            let mut child = harden_replay_spawn(&mut cmd)
                .spawn()
                .with_context(|| "spawning ASan threshold harness")?;

            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed() >= per_run {
                            kill_replay_group(&mut child);
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(e) => return Err(e).context("waiting on ASan threshold harness"),
                }
            }

            let report = std::fs::read_to_string(errpath).unwrap_or_default();
            if let Some(hit) = saf_svcomp::parse_asan_report(&report) {
                return Ok(Some(hit)); // first threshold/argv combination that reproduces wins
            }
        }
    }
    Ok(None)
}

/// The `no-overflow` FALSE pipeline (plan 199, R6): confirmer-first, propose-free.
/// Mirrors [`memsafety_strategy`], swapping the arbiter ASan→UBSan. No sub-property,
/// so the verdict is always `false(no-overflow)` (no classifier).
fn overflow_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    match ubsan_confirm(
        ctx.input,
        ctx.data_model,
        ctx.module,
        ctx.stub,
        ctx.tempdir,
        ctx.clang,
    ) {
        Ok(Some(hit)) => {
            let witness = build_witness(ctx, Some(saf_svcomp::lower_overflow_hit(&hit)));
            if witness.is_none() {
                eprintln!(
                    "saf verify: FALSE (UBSan signed-overflow) but witness unconstructible -> emitting false without a witness"
                );
            }
            VerdictOutcome {
                verdict: saf_svcomp::overflow_verdict(),
                witness,
                graphml: None,
            }
        }
        Ok(None) => {
            eprintln!("saf verify: UBSan replay reproduced no signed-integer overflow -> unknown");
            unknown_outcome()
        }
        Err(e) => {
            eprintln!("saf verify: UBSan replay errored: {e:#} -> unknown");
            unknown_outcome()
        }
    }
}

/// The `termination` TRUE strategy (plan 201, R7 — SAF's first sound-TRUE arm).
///
/// A purely-static structural proof over the already-ingested AIR — no
/// compile-of-original, no execution, no witness (termination TRUE is
/// witness-not-required in SV-COMP 2026). Emits a bare `true` iff
/// [`saf_svcomp::program_structurally_terminates`] holds (loop-free reachable CFGs
/// ∧ acyclic reachable call graph ∧ no reachable indirect call ∧ allowlisted
/// externals); otherwise `unknown`. Never emits `false`.
fn termination_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    // Path 1: the shared mem2reg-only module (current behavior, preserved as a
    // floor so this arm never regresses an already-emitted `true`).
    let mut proven = saf_svcomp::program_structurally_terminates(ctx.module);

    // Path 2 (additive): re-ingest with a richer promotion pipeline that exposes
    // memory-backed loop induction variables (the `alloca()`-library-call families)
    // to the ranking synthesizer. Both proofs are independently sound, so a `true`
    // from either is sound; OR-ing them is strictly recall-additive. Any failure of
    // the second ingestion falls through to Path 1's verdict.
    if !proven {
        match compile_to_ir_termination(ctx.input, ctx.data_model, ctx.stub, ctx.tempdir) {
            Ok(promoted) => proven = saf_svcomp::program_structurally_terminates(&promoted),
            Err(e) => eprintln!(
                "saf verify: termination promotion re-ingest failed: {e:#} -> using mem2reg module"
            ),
        }
    }

    if proven {
        VerdictOutcome {
            verdict: saf_svcomp::termination_verdict().to_string(),
            witness: None,
            graphml: None,
        }
    } else {
        unknown_outcome()
    }
}

/// `ThreadSanitizer` runtime options for the `no-data-race` confirmer: stop at the
/// first race so the report is captured promptly; do not `abort()` (which would
/// mangle the report); a distinct exit code aids debugging (the verdict is driven
/// by the parsed report, not the exit code).
const TSAN_OPTS: &str = "halt_on_error=1:abort_on_error=0:exitcode=66";

/// The `no-data-race` strategy: attempt a sound structural **TRUE** proof first
/// (lever `race-free-true-prover`, verdict-only, no witness) and — if that
/// abstains — fall back to the concrete FALSE confirmation pipeline (finder-gated,
/// `ThreadSanitizer`-confirmed, `GraphML`-witnessed via [`try_no_data_race_false`]).
///
/// TRUE is tried first **for cost**, not because it is less careful: the prover
/// ([`saf_svcomp::program_is_race_free`]) fails **closed** behind a cheap
/// structural gate (unresolved spawns / reachable indirect calls / un-modelled
/// externals) that abstains *before* it pays for pointer analysis — so on the
/// large CIL-expanded driver tasks it costs almost nothing. When it *does* prove a
/// program race-free it short-circuits the whole (expensive) finder + `TSan`
/// replay, which the prior FALSE-first ordering ran on every race-free task; that
/// duplicated analysis is exactly what timed sibling tasks out. The prover is
/// sound and over-approximate (maximal concurrency, sound may-alias,
/// under-approximate locks), so it only ever emits `true` when no unprotected
/// conflicting access pair can exist — a wrong TRUE (−32) cannot arise from an
/// over-approximate finding.
fn no_data_race_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    let source = std::fs::read_to_string(ctx.input).unwrap_or_default();

    // TRUE prover first (cheap-gated). Gate on the same out-of-scope source
    // features (OpenMP / relaxed-memory) plus inline asm, since those can hide
    // concurrency or memory effects the AIR-level proof would not see (a wrong TRUE).
    if race_true_out_of_scope(&source).is_none() {
        if saf_svcomp::program_is_race_free(ctx.module) {
            eprintln!("saf verify: no-data-race structurally proven race-free -> true");
            return race_true_outcome();
        }
        // Retry after a `-fgnu89-inline` re-ingest when a reachable helper is a C99
        // **bare `inline`** function (`inline void f(...)`): the default compile
        // emits NO out-of-line body for such a definition, so the frontend sees a
        // bodyless declaration and the prover abstains (`non-inert-external:f`)
        // even though the program is race-free. `-fgnu89-inline` makes clang emit
        // the out-of-line definition, exposing the real body. This is sound: it
        // only makes MORE code visible to the (fail-closed) prover — the body is the
        // program's actual code, and an opaque declaration already forces abstain,
        // so we can only move a task from `unknown` to prove-or-abstain, never to a
        // wrong `true`. Gated on an actual bare-inline definition in the source, so
        // programs without one (incl. the large CIL driver reservoir, whose
        // undefined callees are kernel externals) pay no extra compile.
        if let Some(module2) = race_gnu89_reingest_if_beneficial(ctx, &source) {
            if saf_svcomp::program_is_race_free(&module2) {
                eprintln!(
                    "saf verify: no-data-race proven race-free after -fgnu89-inline re-ingest -> true"
                );
                return race_true_outcome();
            }
        }
    }

    // FALSE pipeline (concrete TSan confirmation).
    if let Some(outcome) = try_no_data_race_false(ctx, &source) {
        return outcome;
    }

    eprintln!("saf verify: no-data-race neither proven TRUE nor confirmed FALSE -> unknown");
    unknown_outcome()
}

/// The proven-race-free verdict outcome (bare `true`, no witness — R7).
fn race_true_outcome() -> VerdictOutcome {
    VerdictOutcome {
        verdict: saf_svcomp::race_true_verdict().to_string(),
        witness: None,
        graphml: None,
    }
}

/// Re-ingest `ctx.input` with `-fgnu89-inline` and return the fresh module **iff**
/// doing so could plausibly help the no-data-race TRUE prover — i.e. some function
/// the default ingest left as a bodyless declaration is defined in the C source as
/// a C99 **bare `inline`** helper (which the default compile emits no out-of-line
/// body for). Returns `None` (skip the extra compile) when no such helper exists,
/// or on any compile/ingest error (fail-closed — the caller then just abstains from
/// the TRUE proof).
fn race_gnu89_reingest_if_beneficial(
    ctx: &VerifyCtx,
    source: &str,
) -> Option<saf_core::air::AirModule> {
    let undefined = saf_svcomp::undefined_userfn_names(ctx.module);
    if !undefined
        .iter()
        .any(|name| source_has_bare_inline_def(source, name))
    {
        return None;
    }
    let ir = match compile_to_ir_with(
        ctx.input,
        ctx.data_model,
        ctx.stub,
        ctx.tempdir,
        &["-fgnu89-inline"],
        "mem2reg",
    ) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("saf verify: -fgnu89-inline re-ingest compile failed: {e:#}");
            return None;
        }
    };
    match driver::AnalysisDriver::ingest(&[ir], CliFrontend::Llvm) {
        Ok(b) => Some(b.module),
        Err(e) => {
            eprintln!("saf verify: -fgnu89-inline re-ingest failed: {e:#}");
            None
        }
    }
}

/// Does the C `source` define `name` with a C99 **bare `inline`** specifier — i.e.
/// `inline <ret> name(...)` NOT qualified by `static`/`extern` and not the GNU
/// `__inline` spelling? Such a definition emits no out-of-line body under the
/// default (C99) inline semantics, so `name` reaches the frontend as a bodyless
/// declaration; `-fgnu89-inline` restores the out-of-line body.
///
/// Conservative and cost-only: a miss merely skips the (recall-adding) re-ingest
/// and a spurious hit only wastes one compile — neither can affect a verdict.
fn source_has_bare_inline_def(source: &str, name: &str) -> bool {
    let nb = name.as_bytes();
    if nb.is_empty() {
        return false;
    }
    let bytes = source.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while let Some(rel) = source[i..].find(name) {
        let pos = i + rel;
        i = pos + nb.len();
        // Whole-word match for `name`.
        if pos > 0 && is_ident(bytes[pos - 1]) {
            continue;
        }
        // Must be a call/decl form `name(` (allowing whitespace before `(`).
        let mut j = pos + nb.len();
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'(' {
            continue;
        }
        // Scan back to the enclosing statement/declaration boundary and inspect the
        // qualifier/return-type prefix for a bare `inline`.
        let start = source[..pos].rfind([';', '{', '}']).map_or(0, |b| b + 1);
        if prefix_has_bare_inline(&source[start..pos]) {
            return true;
        }
    }
    false
}

/// Is there a whole-word `inline` keyword in a declaration-prefix `head` that is a
/// C99 *bare* inline (no `static`/`extern`/`__inline` in the same prefix)?
fn prefix_has_bare_inline(head: &str) -> bool {
    if head.contains("static") || head.contains("extern") {
        return false;
    }
    let bytes = head.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut i = 0;
    while let Some(rel) = head[i..].find("inline") {
        let pos = i + rel;
        i = pos + 6;
        let before_ok = pos == 0 || !is_ident(bytes[pos - 1]);
        let after = pos + 6;
        let after_ok = after >= bytes.len() || !is_ident(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// The FALSE half of [`no_data_race_strategy`] (lever `race-find`). Returns
/// `Some(false-outcome)` iff `ThreadSanitizer` concretely observes a genuine data
/// race; `None` (abstain from FALSE) otherwise, letting the caller attempt the TRUE
/// proof.
///
/// 1. Gate: require an actually-reachable thread spawn (else sequential — no race).
/// 2. R7 scope gate: abstain on `OpenMP` / relaxed-memory a native x86 (TSO/SC)
///    `TSan` replay cannot soundly arbitrate.
/// 3. Propose: the over-approximate lockset+MHP finder must yield ≥1 candidate.
/// 4. Confirm: compile the ORIGINAL program with `-fsanitize=thread`; emit
///    `false(no-data-race)` IFF `TSan` observes a genuine race (R1). `TSan` is the
///    sole soundness arbiter; the witness is `GraphML` 1.0 (R7).
fn try_no_data_race_false(ctx: &VerifyCtx, source: &str) -> Option<VerdictOutcome> {
    // Gate 1: a race needs a real second thread reachable from main.
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if !saf_svcomp::fast_paths::reachable_spawns_threads(ctx.module, &callgraph) {
        eprintln!("saf verify: no reachable thread spawn (sequential -> no race) -> unknown");
        return None;
    }

    // Gate 2 (R7): out-of-scope concurrency features TSan-on-x86 cannot soundly
    // arbitrate.
    if let Some(reason) = tsan_out_of_scope(source) {
        eprintln!("saf verify: no-data-race FALSE out of scope ({reason}) -> unknown");
        return None;
    }

    // Gate 3: the finder must PROPOSE at least one candidate racing pair. An
    // empty result means the over-approximate lockset+MHP analysis proved the
    // program race-free, so we abstain without paying for a TSan run.
    let candidates = saf_svcomp::find_race_candidates(ctx.module);
    if candidates.is_empty() {
        eprintln!("saf verify: lockset+MHP finder found no race candidate -> unknown");
        return None;
    }
    eprintln!(
        "saf verify: finder proposed {} race candidate(s); attempting TSan confirmation",
        candidates.len()
    );

    // Gate 4: TSan is the sole soundness arbiter (HB-based — only reports a race
    // it concretely observes).
    match tsan_confirm(ctx.input, ctx.module, ctx.stub, ctx.tempdir, ctx.clang) {
        Ok(Some(hit)) => {
            let programfile = ctx.input.file_name().map_or_else(
                || ctx.input.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            let programhash = saf_svcomp::compute_file_hash(ctx.input);
            let architecture = match ctx.data_model {
                saf_svcomp::DataModel::ILP32 => "32bit",
                saf_svcomp::DataModel::LP64 => "64bit",
            };
            let graphml = saf_svcomp::race_graphml_witness(
                ctx.meta.specification.trim(),
                &programfile,
                &programhash,
                architecture,
                &hit,
            );
            Some(VerdictOutcome {
                verdict: format!("false({})", saf_svcomp::Property::NoDataRace.name()),
                witness: None,
                graphml: Some(graphml),
            })
        }
        Ok(None) => {
            eprintln!("saf verify: TSan replay observed no data race -> unknown");
            None
        }
        Err(e) => {
            eprintln!("saf verify: TSan replay errored: {e:#} -> unknown");
            None
        }
    }
}

/// Source-level out-of-scope gate for the structural `no-data-race` **TRUE** proof.
/// Returns `Some(reason)` for features that could hide concurrency or memory effects
/// the AIR-level analysis cannot see — an unsound `true` risk:
///
/// - everything [`tsan_out_of_scope`] rejects (`OpenMP` pragmas, whose parallel
///   regions the frontend drops; relaxed-memory atomics), and
/// - **effectful** inline assembly, whose memory effects the frontend drops
///   entirely (the mapping layer lowers an inline-asm call to nothing, so a
///   store/load performed by asm is invisible to the access scan).
///
/// A bare asm **symbol-rename label** (`extern int f(...) __asm__("f64");`) is NOT
/// effectful — it only renames the link symbol and emits no instruction — so it
/// must not gate. Preprocessed SV-COMP `.i` files pull in glibc headers riddled
/// with such labels (`__sigsetjmp`, `fopen64`, …); the previous blunt substring
/// gate abstained on almost every threaded `.i` file for a label it never needed to
/// fear. [`contains_effectful_inline_asm`] distinguishes the two precisely.
fn race_true_out_of_scope(source: &str) -> Option<&'static str> {
    if let Some(reason) = tsan_out_of_scope(source) {
        return Some(reason);
    }
    if contains_effectful_inline_asm(source) {
        return Some("inline asm");
    }
    None
}

/// Does `source` contain an inline-asm **statement** that could read/write memory
/// or otherwise perturb concurrency — as opposed to a pure asm **symbol-rename
/// label** (`__asm__("name")`, only string literals in the parens)?
///
/// Sound-conservative: returns `true` (⇒ abstain) on ANY asm usage that is not a
/// provably-inert label. An asm occurrence is a label iff it is `asm`/`__asm__`/
/// `__asm` immediately followed (after whitespace) by `(` whose content, up to the
/// matching close paren, is **only** string literals — each a valid assembler
/// **symbol name** (`[A-Za-z0-9_.$@]`, or empty) — and whitespace.
///
/// A symbol-rename label (`__asm__("" "__sigsetjmp")`) has exactly this form and
/// emits no instruction, so it is inert. It is syntactically indistinguishable from
/// basic asm *except* by string content: a real asm template needs operands or a
/// multi-token instruction, which require spaces / `;` / `%` / `,` — none of which
/// are legal in a symbol name. So any string carrying such a character (⇒ a genuine
/// instruction template), a `volatile`/`goto` qualifier before the paren, an
/// extended-asm `:` operand section, or a missing/short paren is treated as
/// effectful. (A degenerate single-mnemonic basic asm like `asm("nop")` classifies
/// as a label — sound for the data-race model: with no operand it cannot name any
/// memory.)
fn contains_effectful_inline_asm(source: &str) -> bool {
    let bytes = source.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    // Characters legal in an assembler symbol name (the only content a rename label
    // may carry). Anything else inside the string ⇒ an instruction template.
    let is_symbol_char =
        |b: u8| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'$' | b'@');
    let mut i = 0;
    while i < bytes.len() {
        // Match the longest asm keyword at `i`, on a word boundary.
        let kw_len = if bytes[i..].starts_with(b"__asm__") {
            7
        } else if bytes[i..].starts_with(b"__asm") {
            5
        } else if bytes[i..].starts_with(b"asm") {
            3
        } else {
            i += 1;
            continue;
        };
        let boundary_before = i == 0 || !is_ident(bytes[i - 1]);
        let after = i + kw_len;
        let boundary_after = after >= bytes.len() || !is_ident(bytes[after]);
        if !(boundary_before && boundary_after) {
            i += 1;
            continue;
        }
        // Skip whitespace after the keyword.
        let mut j = after;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        // A qualifier (`volatile`/`goto`) or anything other than an opening paren
        // means this is not a pure `asm("label")` — treat as effectful.
        if j >= bytes.len() || bytes[j] != b'(' {
            return true;
        }
        // Scan the parenthesised group: a label has ONLY string literals + ws.
        j += 1; // consume '('
        loop {
            if j >= bytes.len() {
                return true; // unterminated — fail closed
            }
            let c = bytes[j];
            if c.is_ascii_whitespace() {
                j += 1;
            } else if c == b'"' {
                // Scan a string literal, honouring `\"` escapes. A non-symbol char
                // inside ⇒ a real instruction template ⇒ effectful.
                j += 1;
                while j < bytes.len() && bytes[j] != b'"' {
                    if bytes[j] == b'\\' {
                        return true; // escape ⇒ instruction template, not a symbol
                    }
                    if !is_symbol_char(bytes[j]) {
                        return true;
                    }
                    j += 1;
                }
                if j >= bytes.len() {
                    return true; // unterminated string — fail closed
                }
                j += 1; // consume closing quote
            } else if c == b')' {
                break; // only strings + ws seen ⇒ inert label
            } else {
                return true; // any other token ⇒ effectful asm
            }
        }
        i = j + 1;
    }
    false
}

/// R7 scope gate for `no-data-race`: returns `Some(reason)` when the program
/// uses concurrency features that a native x86 (TSO/SC) `TSan` replay cannot
/// soundly arbitrate, so the strategy must abstain (fail-closed).
///
/// - `#pragma omp` — SAF's frontend drops `OpenMP` pragmas and native replay
///   cannot reproduce the `OpenMP` runtime.
/// - non-seq_cst C11 atomic orderings — weak-memory behaviour is not observable
///   under x86 TSO, so a weak-memory-only race would be missed and, conversely,
///   `TSan`'s treatment of relaxed atomics as synchronization could hide a real
///   SV-COMP race. Out of scope either way.
///
/// Custom whole-function atomic sections `__VERIFIER_atomic_<name>` (other than
/// the modelled `begin`/`end`) are NOT abstained here: they are modelled in the
/// `TSan` replay driver as a global recursive mutex acquired around the whole
/// function body via `-finstrument-functions` (see
/// [`custom_verifier_atomic_fns`] / [`synthesize_tsan_driver`]), which matches
/// SV-COMP's "the function executes atomically" semantics (atomic↔atomic never
/// races; atomic↔plain still races). Only genuinely-unarbitratable features stay
/// out of scope.
fn tsan_out_of_scope(source: &str) -> Option<&'static str> {
    if source.contains("#pragma omp") {
        return Some("OpenMP");
    }
    if source.contains("memory_order_relaxed")
        || source.contains("memory_order_consume")
        || source.contains("memory_order_acquire")
        || source.contains("memory_order_release")
        || source.contains("memory_order_acq_rel")
    {
        return Some("relaxed-memory atomics");
    }
    None
}

/// Collect the full identifiers of every custom whole-function atomic the source
/// references — `__VERIFIER_atomic_<suffix>` where `<suffix>` is NOT the modelled
/// `begin`/`end`. Deterministic (`BTreeSet`-ordered).
///
/// SV-COMP defines a function named `__VERIFIER_atomic_<name>` to execute
/// atomically (its whole body runs without interleaving). The returned set is
/// modelled in the `TSan` driver: each such function's entry/exit is wrapped in a
/// global recursive mutex via `-finstrument-functions` so `TSan` sees the
/// serialization. An atomic function that is only declared (never defined) or is
/// `static` yields an unresolved `extern` symbol at link time → the confirm link
/// fails → the strategy abstains (fail-closed, sound).
fn custom_verifier_atomic_fns(source: &str) -> std::collections::BTreeSet<String> {
    const MARK: &str = "__VERIFIER_atomic_";
    let mut out = std::collections::BTreeSet::new();
    let mut i = 0;
    while let Some(off) = source[i..].find(MARK) {
        let start = i + off + MARK.len();
        let suffix: String = source[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !suffix.is_empty() && suffix != "begin" && suffix != "end" {
            out.insert(format!("{MARK}{suffix}"));
        }
        // Advance past this occurrence (at least one byte) to avoid looping.
        i = start.max(i + MARK.len());
    }
    out
}

/// Build the `TSan`-replay driver: SV-COMP nondet generators (returning
/// `$SAF_NONDET_CONST`, default 0), `__VERIFIER_assume` as a hard path filter
/// (R4), and `__VERIFIER_atomic_begin/end` modelled as a global RECURSIVE mutex
/// so `TSan` sees the atomic-section synchronization (else it would false-alarm a
/// race the SV-COMP semantics forbid). `rand`/`srand` are pinned for determinism.
///
/// `atomic_fns` is the set of custom whole-function atomics
/// (`__VERIFIER_atomic_<name>`) the program defines. When non-empty the driver
/// also emits `-finstrument-functions` entry/exit hooks (`__cyg_profile_func_*`)
/// that acquire/release the SAME global recursive mutex around each such
/// function's body, giving them SV-COMP whole-function atomicity under `TSan`.
/// The hooks are marked `no_instrument_function` to avoid self-recursion, and
/// identify the atomic functions by comparing the runtime frame address
/// (`this_fn`) against each function's taken address — `extern void f();` is a
/// signature-agnostic forward declaration, so no signature knowledge is needed.
fn synthesize_tsan_driver(atomic_fns: &std::collections::BTreeSet<String>) -> String {
    use std::fmt::Write as _;

    let mut s = String::new();
    s.push_str("/* no-data-race TSan-replay driver (generated) */\n");
    s.push_str("#define _GNU_SOURCE 1\n");
    s.push_str("#include <stddef.h>\n#include <stdlib.h>\n#include <pthread.h>\n");
    s.push_str("extern void _exit(int) __attribute__((noreturn));\n");
    s.push_str(
        "static long __saf_c(void) { const char *e = getenv(\"SAF_NONDET_CONST\"); return e ? atol(e) : 0; }\n",
    );
    for (fname, cty) in SCALAR_NONDET {
        let suffix = fname.trim_start_matches("__VERIFIER_nondet_");
        let _ = writeln!(
            s,
            "{cty} __VERIFIER_nondet_{suffix}(void) {{ return ({cty})__saf_c(); }}"
        );
    }
    s.push_str("void* __VERIFIER_nondet_pointer(void) { return (void*)0; }\n");
    s.push_str("float __VERIFIER_nondet_float(void) { return 0.0f; }\n");
    s.push_str("double __VERIFIER_nondet_double(void) { return 0.0; }\n");
    s.push_str("void __VERIFIER_assume(int c) { if (!c) _exit(0); }\n");
    // Atomic sections modelled as a global recursive mutex (visible to TSan).
    s.push_str("static pthread_mutex_t __saf_atomic = PTHREAD_RECURSIVE_MUTEX_INITIALIZER_NP;\n");
    s.push_str("void __VERIFIER_atomic_begin(void) { pthread_mutex_lock(&__saf_atomic); }\n");
    s.push_str("void __VERIFIER_atomic_end(void) { pthread_mutex_unlock(&__saf_atomic); }\n");
    s.push_str("int __wrap_rand(void) { return (int)__saf_c(); }\n");
    s.push_str("void __wrap_srand(unsigned s) { (void)s; }\n");

    // Custom whole-function atomics: serialize each on the SAME global recursive
    // mutex via -finstrument-functions hooks (compiled in by the caller). The
    // recursive mutex composes with a nested __VERIFIER_atomic_begin/end.
    if !atomic_fns.is_empty() {
        s.push_str("/* custom __VERIFIER_atomic_* whole-function atomicity */\n");
        for fname in atomic_fns {
            let _ = writeln!(s, "extern void {fname}();");
        }
        s.push_str("static void *const __saf_atomic_fns[] = {");
        for fname in atomic_fns {
            let _ = write!(s, " (void*){fname},");
        }
        s.push_str(" };\n");
        s.push_str(
            "__attribute__((no_instrument_function)) static int __saf_is_atomic_fn(void *f) {\n\
             unsigned i; for (i = 0; i < sizeof(__saf_atomic_fns)/sizeof(__saf_atomic_fns[0]); i++)\n\
             if (__saf_atomic_fns[i] == f) return 1; return 0; }\n",
        );
        s.push_str(
            "__attribute__((no_instrument_function)) void __cyg_profile_func_enter(void *f, void *c) {\n\
             (void)c; if (__saf_is_atomic_fn(f)) pthread_mutex_lock(&__saf_atomic); }\n",
        );
        s.push_str(
            "__attribute__((no_instrument_function)) void __cyg_profile_func_exit(void *f, void *c) {\n\
             (void)c; if (__saf_is_atomic_fn(f)) pthread_mutex_unlock(&__saf_atomic); }\n",
        );
    }
    s
}

/// Confirm a `no-data-race` FALSE by `ThreadSanitizer`-instrumented native
/// execution.
///
/// Compiles the ORIGINAL program with `-fsanitize=thread -pthread -g` + the
/// nondet driver, runs it under the [`replay_candidates`] mini-fuzz (the fixed
/// [`NONDET_CONSTS`] spread followed by the program's own branch-steering
/// literals) capturing stderr to a file, and parses the report
/// ([`saf_svcomp::parse_tsan_report`], which applies R1 — only a genuine `data
/// race` warning confirms). `Ok(Some(hit))` is a confirmed race; `Ok(None)` is
/// inconclusive (no report / compile-link failure / timeout) ⇒ the caller keeps
/// `unknown`. Because `TSan` is happens-before-based, a reported race is a real
/// race for the observed inputs, so confirming is sound.
///
/// # Input steering (R5-legal nondet choices)
///
/// Many concurrency FALSEs gate their racing path on a nondet input range — e.g.
/// `n = __VERIFIER_nondet_uint(); assume_abort_if_not(n >= 5 && n <= 10);` before
/// spawning `n` racing threads. The fixed spread (`0,1,2,42,255,…`) contains no
/// value in `[5,10]`, so the program aborts before any thread is created and the
/// race is never observed. Feeding the program's OWN comparison literals (`5`,
/// `10`) into the sweep via [`replay_candidates`] reaches the spawn. Each steered
/// value is just another concrete nondet input the verifier may legally choose
/// (R5); `TSan` remains the sole arbiter, so this only widens *reachability* and
/// cannot manufacture a race the program does not have.
///
/// # Data model (R3 exception — soundly compile LP64 even for ILP32 tasks)
///
/// The replay is ALWAYS compiled under LP64 (`-m64`), regardless of the task's
/// declared data model. Clang ships **no 32-bit `ThreadSanitizer` runtime**
/// (`-fsanitize=thread` + `-m32` fails: "unsupported option for target
/// `i386-pc-linux-gnu`"), so honoring an `ILP32` task's `-m32` here would make
/// every ILP32 concurrency task fail to compile and abstain — and the entire
/// SV-COMP `no-data-race` FALSE reservoir is ILP32.
///
/// This substitution is SOUND for the `no-data-race` property specifically: a
/// data race is defined by the thread synchronization / happens-before structure
/// (thread create/join, mutexes, atomics), whose semantics are identical under
/// ILP32 and LP64. Pointer/`long` width cannot introduce a race the source lacks,
/// nor synchronize away a real one. So a race `TSan` concretely observes under
/// LP64 is a genuine race of the original program under either model. (The
/// emitted witness still declares the task's REAL architecture upstream, so the
/// validator re-analyzes under the declared model.)
fn tsan_confirm(
    input: &Path,
    module: &saf_core::air::AirModule,
    stub: &Path,
    dir: &Path,
    clang: &str,
) -> anyhow::Result<Option<saf_svcomp::RaceHit>> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let driver_src = dir.join("saf_tsan_driver.c");
    let harness = dir.join("saf_tsan_harness");
    let errpath = dir.join("saf_tsan_stderr.txt");

    // Custom whole-function atomics (`__VERIFIER_atomic_<name>`) are modelled in
    // the driver via `-finstrument-functions` hooks. When the program uses any,
    // we must both emit their entry/exit serialization AND compile with
    // instrumentation enabled so the hooks actually fire.
    let source = std::fs::read_to_string(input).unwrap_or_default();
    let atomic_fns = custom_verifier_atomic_fns(&source);
    let instrument: &[&str] = if atomic_fns.is_empty() {
        &[]
    } else {
        &["-finstrument-functions"]
    };

    std::fs::write(&driver_src, synthesize_tsan_driver(&atomic_fns))
        .with_context(|| "writing TSan replay driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));
    // Shared compile of ORIGINAL program + driver under TSan. `extra` carries the
    // inline-linkage flag on the first attempt; on failure we retry without it (a
    // rare program with both an `inline` and an external definition would
    // duplicate-define under GNU89 semantics — the fallback preserves the prior
    // C99 behavior so we never regress a previously-compiling task).
    let compile = |extra: &[&str]| -> anyhow::Result<bool> {
        let mut cmd = Command::new(clang);
        cmd.args([
            "-O0",
            "-g",
            "-fsanitize=thread",
            "-pthread",
            "-Wno-everything",
            // A data race is data-model-independent and clang has no 32-bit TSan
            // runtime, so always compile the replay LP64 (see the doc comment).
            "-m64",
            // Determinism: redirect rand()/srand() to the driver's __wrap_* stubs.
            "-Wl,--wrap=rand",
            "-Wl,--wrap=srand",
        ]);
        cmd.args(instrument);
        cmd.args(extra)
            .arg("-include")
            .arg(stub)
            .arg("-I")
            .arg(srcdir)
            .arg(input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        Ok(cmd
            .status()
            .with_context(|| format!("failed to spawn {clang} for TSan replay"))?
            .success())
    };

    // Emit out-of-line definitions for C99 `inline` helpers. Many SV-COMP
    // concurrency programs (e.g. `pthread-ext`) define lock helpers as bare
    // `inline void acquire_lock() { … }`; under C99 semantics at -O0 clang emits
    // an *inline definition* with no external symbol, so a non-inlined call
    // link-fails ("undefined reference to acquire_lock") and the whole confirm
    // aborts — indistinguishable from "no race observed". GNU89 inline semantics
    // always emit a definition, fixing the link. This is a pure linkage change
    // (identical function bodies) and cannot alter the race.
    if !compile(&["-fgnu89-inline"])? && !compile(&[])? {
        // Compile/link failure (e.g. a missing 32-bit TSan runtime) -> inconclusive.
        return Ok(None);
    }

    let timeout = replay_timeout();
    // Fixed spread FIRST (committed byte-for-byte behavior is a prefix), then the
    // program's own guard literals for nondet-input-gated racing paths.
    let candidates = replay_candidates(NONDET_CONSTS, module);
    for &k in &candidates {
        // Redirect the child's stderr to a FILE (not a pipe) so a large TSan
        // report cannot deadlock on a full pipe buffer while we poll the timeout.
        let errfile =
            std::fs::File::create(&errpath).with_context(|| "creating TSan stderr file")?;
        let mut cmd = Command::new(&harness);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errfile))
            .env("TSAN_OPTIONS", TSAN_OPTS)
            .env("SAF_NONDET_CONST", k.to_string());
        let mut child = harden_replay_spawn(&mut cmd)
            .spawn()
            .with_context(|| "spawning TSan harness")?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        kill_replay_group(&mut child);
                        break; // runaway (e.g. a thread blocked forever) -> parse what exists
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(e).context("waiting on TSan harness"),
            }
        }

        let report = std::fs::read_to_string(&errpath).unwrap_or_default();
        if let Some(hit) = saf_svcomp::parse_tsan_report(&report) {
            return Ok(Some(hit)); // first constant that reproduces a race wins
        }
    }
    Ok(None)
}

/// Confirm a `no-overflow` FALSE by UBSan-instrumented native execution (plan 199, R6).
///
/// Compiles the ORIGINAL program with `-fsanitize=signed-integer-overflow -g` + the
/// nondet driver, runs it under the [`OVERFLOW_CONSTS`] mini-fuzz capturing stderr to a
/// file, and parses the report ([`saf_svcomp::parse_ubsan_overflow`], which applies R1).
/// `Ok(Some(hit))` is a confirmed signed overflow; `Ok(None)` is inconclusive (no report
/// / abstained / compile-link failure / timeout) ⇒ the caller keeps `unknown`. Reuses
/// [`synthesize_asan_driver`] (sanitizer-agnostic) unchanged.
///
/// Threaded programs abstain up front (conservative default, plan 199 D5): an overflow
/// is schedule-independent so confirming a threaded task is sound, but the concurrency
/// reservoir is small and low-recall, so the R5 reachability gate is kept until a gated
/// slice measures it worth dropping.
// NOTE: this is one cohesive confirmer — compile (with the assert-macro fallback),
// then two mini-fuzz sweeps (primary + the loop-sustaining bool pass) sharing a
// spawn/wait closure. Splitting it would scatter the tightly-coupled harness/errpath
// state across helpers for no readability gain.
#[allow(clippy::too_many_lines)]
fn ubsan_confirm(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    module: &saf_core::air::AirModule,
    stub: &Path,
    dir: &Path,
    clang: &str,
) -> anyhow::Result<Option<saf_svcomp::OverflowHit>> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let callgraph = saf_analysis::callgraph::CallGraph::build(module);
    if saf_svcomp::fast_paths::reachable_spawns_threads(module, &callgraph) {
        eprintln!("saf verify: a thread spawn is reachable from main (out of R6 scope) -> unknown");
        return Ok(None);
    }

    let driver_src = dir.join("saf_ubsan_driver.c");
    let harness = dir.join("saf_ubsan_harness");
    let errpath = dir.join("saf_ubsan_stderr.txt");

    std::fs::write(&driver_src, synthesize_asan_driver())
        .with_context(|| "writing UBSan replay driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));
    // Build the sanitizer compile+link; `assert_neutralizer` (when Some) is the
    // two-pass assert-macro fallback (see `write_assert_neutralizer`) so a program
    // that defines its own `__VERIFIER_assert` links its replay binary.
    let build_replay = |assert_neutralizer: Option<&Path>| {
        let mut cmd = Command::new(clang);
        cmd.args([
            "-O0",
            "-g",
            "-fsanitize=signed-integer-overflow",
            "-fno-sanitize-recover=signed-integer-overflow",
            "-Wno-everything",
            // Determinism: redirect rand()/srand() to the driver's __wrap_* stubs.
            "-Wl,--wrap=rand",
            "-Wl,--wrap=srand",
        ])
        .arg(data_model.clang_flag())
        .arg("-include")
        .arg(stub);
        if let Some(neutralizer) = assert_neutralizer {
            cmd.arg("-include").arg(neutralizer);
        }
        cmd.arg("-I")
            .arg(srcdir)
            .arg(input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        cmd
    };
    let mut linked = build_replay(None)
        .status()
        .with_context(|| format!("failed to spawn {clang} for UBSan replay"))?
        .success();
    if !linked {
        // Assert-macro fallback: mirror the ingestion two-pass so a task whose IR
        // was only obtainable with the neutralizer also produces a replay binary.
        let neutralizer = write_assert_neutralizer(dir)?;
        linked = build_replay(Some(&neutralizer))
            .status()
            .with_context(|| format!("failed to spawn {clang} for UBSan replay"))?
            .success();
    }
    if !linked {
        // Compile/link failure (e.g. a task defining its own nondet) -> inconclusive.
        return Ok(None);
    }

    // Multi-constant mini-fuzz: one binary run under OVERFLOW_CONSTS (0 first). A
    // scalar-guarded/scalar-sized overflow the zeroed probe misses is reproduced by the
    // matching constant. Sound: each constant is a valid concrete input, and
    // __VERIFIER_assume still prunes infeasible ones. Confirm on the FIRST trap.
    //
    // Branch-steering (cpa-witness2test-style input steering): after the fixed spread,
    // append the program's OWN integer comparison / switch literals so a guard-gated
    // overflow (`if (x == K) INT_MAX + x;`) is reached when the fixed spread never
    // guesses `K` (see `replay_candidates`). Harvested literals are magnitude-capped
    // below 2^30 (so no steered value can widen the loop-counter-to-INT_MAX false-alarm
    // surface) and are still just concrete nondet inputs — soundness is unchanged (a
    // trap is re-triggered on the ORIGINAL program).
    //
    // Ranked-loop boundary injection: when every reachable loop is provably ranked
    // (or the program is loop-free), `overflow_replay_candidates` also appends the
    // type-boundary values (INT_MAX, near-INT_MAX/INT_MIN, plus the 64-bit
    // boundaries under LP64). A ranked loop's counter provably cannot reach INT_MAX
    // and overflow on the next step, so a boundary input can only trigger a genuine
    // DIRECT overflow — this unlocks counted-loop sinks without re-admitting the
    // termination-* loop false alarm (see `overflow_boundary_consts`).
    let candidates = overflow_replay_candidates(module, data_model);

    let timeout = replay_timeout();
    // Run the harness ONCE under the extra environment `env` (added to the fixed
    // `UBSAN_OPTIONS`), waiting up to `run_timeout`, and parse the captured stderr into
    // an overflow hit (`None` = no trap / timeout). Shared by every sweep below so the
    // spawn/wait/parse logic lives in one place.
    let run_child = |env: &[(&str, String)],
                     run_timeout: std::time::Duration|
     -> anyhow::Result<Option<saf_svcomp::OverflowHit>> {
        let errfile =
            std::fs::File::create(&errpath).with_context(|| "creating UBSan stderr file")?;
        let mut cmd = Command::new(&harness);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errfile))
            .env("UBSAN_OPTIONS", UBSAN_OPTS);
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = harden_replay_spawn(&mut cmd)
            .spawn()
            .with_context(|| "spawning UBSan harness")?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= run_timeout {
                        kill_replay_group(&mut child);
                        break; // runaway -> parse whatever exists (likely no report)
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(e).context("waiting on UBSan harness"),
            }
        }

        let report = std::fs::read_to_string(&errpath).unwrap_or_default();
        Ok(saf_svcomp::parse_ubsan_overflow(&report))
    };

    // Run one mini-fuzz sweep: for each data constant `k` in `sweep`, run the harness
    // with `SAF_NONDET_CONST=k` (and, when `bool_const` is set, `SAF_BOOL_CONST` too),
    // returning the first constant that reproduces a signed overflow.
    let run_sweep = |sweep: &[i64],
                     bool_const: Option<&str>|
     -> anyhow::Result<Option<saf_svcomp::OverflowHit>> {
        for &k in sweep {
            let mut env: Vec<(&str, String)> = vec![("SAF_NONDET_CONST", k.to_string())];
            if let Some(bc) = bool_const {
                env.push(("SAF_BOOL_CONST", bc.to_string()));
            }
            if let Some(hit) = run_child(&env, timeout)? {
                return Ok(Some(hit)); // first constant that reproduces an overflow wins
            }
        }
        Ok(None)
    };

    // Primary sweep: nondet_bool tracks SAF_NONDET_CONST (the committed behaviour).
    if let Some(hit) = run_sweep(&candidates, None)? {
        return Ok(Some(hit));
    }

    // Loop-sustaining bool pass: when the program has a `__VERIFIER_nondet_bool()`
    // (so it may have a `while (nondet_bool()) { … }` guard the primary sweep cannot
    // both keep true AND supply the right data value for), fix the bool to `1` and
    // re-sweep the data nondets. Any resulting UBSan trap is a genuine feasible
    // execution — `1` is a legal `nondet_bool` return and the data value is a legal
    // nondet input — so soundness is unchanged (UBSan is the sole R2 arbiter, the trap
    // re-triggers on the ORIGINAL program per R6). Gated on the presence of nondet_bool
    // so the extra native runs never touch a program that cannot benefit.
    if saf_svcomp::fuzz::references_nondet_bool(module) {
        if let Some(hit) = run_sweep(OVERFLOW_BOOL_SWEEP, Some("1"))? {
            return Ok(Some(hit));
        }
    }

    // Positional boundary injection (multi-nondet loop class): the uniform sweeps above
    // give EVERY scalar nondet the same value, so they cannot satisfy a program whose
    // precondition pins some nondets (`sum==0 && i==0`, `x!=y`) while a DIFFERENT nondet
    // must be large/specific to overflow (the loop bound `n`, an accumulator seed near
    // INT_MAX, a negated `y=-1`). This pass gives ONE targeted nondet call site a
    // type-boundary value (`SAF_TARGET_IDX`/`SAF_TARGET_VAL`) while every other
    // scalar-int nondet takes a small baseline (`SAF_BASE_VAL` ∈ {0,1}); `nondet_bool`
    // is pinned to 0 so no `while (nondet_bool())` loop is sustained, keeping every run
    // fast. Sweep the target position × boundary value × baseline, returning on the
    // first UBSan trap. Sound: every combination is a concrete feasible execution
    // (UBSan sole arbiter R2, re-triggered on the original program R6; `__VERIFIER_assume`
    // still prunes infeasible paths R4). Gated on ≥2 scalar-nondet call sites — with a
    // single site the positional value equals the uniform sweep already tried — and
    // bounded by a per-run timeout, a leading-position cap, and a whole-pass wall clock.
    let nondet_sites = saf_svcomp::fuzz::count_scalar_nondet_call_sites(module);
    if nondet_sites >= 2 {
        let positions = nondet_sites.min(OVERFLOW_POS_MAX_INDEX);
        let targets = overflow_positional_targets(data_model);
        let pos_timeout = overflow_positional_timeout();
        let pos_deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(OVERFLOW_POS_WALL_SECS);
        for &val in &targets {
            for idx in 0..positions {
                for &base in OVERFLOW_POS_BASELINES {
                    if std::time::Instant::now() >= pos_deadline {
                        return Ok(None); // whole-pass wall cap -> inconclusive
                    }
                    let env = [
                        ("SAF_TARGET_IDX", idx.to_string()),
                        ("SAF_TARGET_VAL", val.to_string()),
                        ("SAF_BASE_VAL", base.to_string()),
                    ];
                    if let Some(hit) = run_child(&env, pos_timeout)? {
                        return Ok(Some(hit));
                    }
                }
            }
        }
    }
    Ok(None)
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
            let llvm_name = format!("llvm-{}", saf_frontends::LLVM_VERSION);
            println!(
                "{}",
                serde_json::to_string_pretty(&[llvm_name.as_str(), "air-json"])?
            );
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
    println!("  llvm ({}), air-json", saf_frontends::LLVM_VERSION);
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

#[cfg(test)]
mod verify_tests {
    use super::*;

    #[test]
    fn assert_neutralizer_undefs_the_macro() {
        // The fallback header must `#undef __VERIFIER_assert` (guarded so it is a
        // no-op when the program never triggered the stub macro), so a program
        // that defines its own `void __VERIFIER_assert(int)` parses. Guarding the
        // `#undef` keeps the header safe to `-include` unconditionally.
        let dir = tempfile::tempdir().expect("tempdir");
        let p = write_assert_neutralizer(dir.path()).expect("write header");
        let body = std::fs::read_to_string(&p).expect("read header");
        assert!(body.contains("#undef __VERIFIER_assert"), "{body}");
        assert!(body.contains("#ifdef __VERIFIER_assert"), "{body}");
        // Deterministic path/name so a second `-include` is stable across runs.
        assert!(p.ends_with("saf_undef_assert.h"), "{}", p.display());
    }

    #[test]
    fn no_quarantine_opts_disable_quarantine_and_keep_base_bounds() {
        // The address-recycle `valid-free` variant (memsafety/`cmp-freed-ptr`) only
        // reproduces when the quarantine is off, so both keys must be present and
        // zeroed. It must ALSO keep every safety/determinism bound of the default
        // `ASAN_OPTS` (leaks off, printf checks off, the RSS/allocation caps) so the
        // second sweep does not regress determinism or reintroduce the OOM risk.
        assert!(ASAN_OPTS_NO_QUARANTINE.contains("quarantine_size_mb=0"));
        assert!(ASAN_OPTS_NO_QUARANTINE.contains("thread_local_quarantine_size_kb=0"));
        for bound in [
            "detect_leaks=0",
            "check_printf=0",
            "max_allocation_size_mb=1024",
            "hard_rss_limit_mb=3072",
            "abort_on_error=0",
        ] {
            assert!(ASAN_OPTS.contains(bound), "default missing {bound}");
            assert!(
                ASAN_OPTS_NO_QUARANTINE.contains(bound),
                "no-quarantine missing {bound}"
            );
        }
        // The default sweep (run FIRST) must NOT disable the quarantine — that is what
        // preserves its committed use-after-free recall before the additive variant.
        assert!(!ASAN_OPTS.contains("quarantine_size_mb=0"));
    }

    #[test]
    fn assert_neutralizer_is_byte_stable() {
        // Determinism (NFR-DET): identical content on every write.
        let dir = tempfile::tempdir().expect("tempdir");
        let a = std::fs::read_to_string(write_assert_neutralizer(dir.path()).unwrap()).unwrap();
        let b = std::fs::read_to_string(write_assert_neutralizer(dir.path()).unwrap()).unwrap();
        assert_eq!(a, b);
    }

    // --- replay_candidates (branch-steering merge) -------------------------

    use saf_core::air::{
        AirBlock, AirFunction, AirModule, BinaryOp, Constant, Instruction, Operation,
    };
    use saf_core::id::make_id;
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};
    use std::collections::BTreeMap;

    /// A defined `main` whose body is a sequence of `icmp eq x, K` instructions, one
    /// per constant in `guard_consts` — the branch-steering harvester reads these.
    fn module_with_guard_constants(guard_consts: &[i64]) -> AirModule {
        let bid = BlockId(make_id("block", b"main_entry"));
        let mut block = AirBlock::new(bid);
        let mut constants: BTreeMap<ValueId, Constant> = BTreeMap::new();
        for (i, &k) in guard_consts.iter().enumerate() {
            let cvid = ValueId(make_id("value", format!("c{i}").as_bytes()));
            let xvid = ValueId(make_id("value", format!("x{i}").as_bytes()));
            constants.insert(cvid, Constant::int(k, 32));
            block.instructions.push(Instruction {
                id: InstId(make_id("inst", format!("i{i}").as_bytes())),
                op: Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                operands: vec![xvid, cvid],
                dst: Some(ValueId(make_id("value", format!("r{i}").as_bytes()))),
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            });
        }
        let main = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut module = AirModule::new(ModuleId(make_id("module", b"test")));
        module.functions.push(main);
        module.constants = constants;
        module
    }

    #[test]
    fn memsafety_replay_timeout_defaults_shorter_than_the_shared_replay_timeout() {
        // The memsafety ASan sweep runs a multi-pass, multi-constant set of native runs
        // that must all fit the per-task wall-clock budget, so its per-run cap is
        // deliberately shorter than the shared default (4 s vs 10 s). Keeping it strictly
        // below the shared cap is the invariant this change relies on; assert it so a
        // future default bump cannot silently re-starve the later confirming passes.
        // (Assumes neither `$SAF_MEMSAFETY_REPLAY_TIMEOUT` nor `$SAF_VERIFY_REPLAY_TIMEOUT`
        // is set — the default in every CI/eval run; the env override path is exercised
        // in production, not here, to avoid process-global env mutation in the test.)
        assert_eq!(
            memsafety_replay_timeout(),
            std::time::Duration::from_secs(4)
        );
        assert!(
            memsafety_replay_timeout() < replay_timeout(),
            "memsafety per-run cap must stay below the shared replay cap"
        );
    }

    #[test]
    fn replay_candidates_is_a_prefix_of_the_fixed_spread() {
        // No program guard literals -> the candidate list is EXACTLY the fixed spread
        // (the committed behavior is byte-for-byte preserved: 0 regression).
        let module = module_with_guard_constants(&[]);
        assert_eq!(
            replay_candidates(NONDET_CONSTS, &module),
            NONDET_CONSTS.to_vec()
        );
        assert_eq!(
            replay_candidates(OVERFLOW_CONSTS, &module),
            OVERFLOW_CONSTS.to_vec()
        );
    }

    #[test]
    fn replay_candidates_appends_novel_guard_literals_after_the_fixed_spread() {
        // A guard constant absent from the fixed spread is appended AFTER it, so the
        // fixed spread still runs first and a guard-gated fault becomes reachable.
        let module = module_with_guard_constants(&[500]);
        let got = replay_candidates(NONDET_CONSTS, &module);
        assert!(
            got.starts_with(NONDET_CONSTS),
            "fixed spread must be the prefix"
        );
        assert_eq!(*got.last().unwrap(), 500);
    }

    #[test]
    fn replay_candidates_dedups_literals_already_in_the_fixed_spread() {
        // A guard constant that equals a fixed-spread value adds nothing (no duplicate
        // replay run) — 42 is already in NONDET_CONSTS.
        let module = module_with_guard_constants(&[42]);
        assert_eq!(
            replay_candidates(NONDET_CONSTS, &module),
            NONDET_CONSTS.to_vec()
        );
    }

    #[test]
    fn tsan_sweep_includes_range_guard_literals_absent_from_the_fixed_spread() {
        // Regression for the no-data-race input-steering gap: a program that gates
        // its racing thread spawns behind `assume_abort_if_not(n >= 5 && n <= 10)`
        // aborts under every fixed NONDET_CONSTS value (none lands in [5,10]), so the
        // TSan replay never observes the race. The confirmer now derives its sweep
        // from `replay_candidates(NONDET_CONSTS, module)`, which appends the program's
        // OWN comparison literals (5 and 10) — reaching the spawn so TSan can arbitrate.
        let module = module_with_guard_constants(&[5, 10]);
        let sweep = replay_candidates(NONDET_CONSTS, &module);
        assert!(
            sweep.starts_with(NONDET_CONSTS),
            "committed fixed spread stays the prefix (0 regression)"
        );
        assert!(
            sweep.contains(&5),
            "range-lower guard literal must be steered"
        );
        assert!(
            sweep.contains(&10),
            "range-upper guard literal must be steered"
        );
        // None of the fixed constants satisfy 5 <= n <= 10, so steering is essential.
        assert!(
            !NONDET_CONSTS.iter().any(|&k| (5..=10).contains(&k)),
            "the fixed spread alone cannot reach the [5,10]-gated spawn"
        );
    }

    #[test]
    fn replay_candidates_is_capped_at_max_replay_candidates() {
        // Many distinct guard literals -> the list is bounded (per-task replay budget).
        let many: Vec<i64> = (1000..1100).collect();
        let module = module_with_guard_constants(&many);
        let got = replay_candidates(NONDET_CONSTS, &module);
        assert!(got.len() <= MAX_REPLAY_CANDIDATES);
        assert!(
            got.starts_with(NONDET_CONSTS),
            "fixed spread must be preserved under the cap"
        );
    }

    // --- overflow loop-free boundary injection -----------------------------

    /// A defined `main` with a two-block CFG back-edge (`b0 -> b1 -> b0`) — a
    /// reachable CFG loop, so `module_reachable_is_loop_free` is false. Includes the
    /// `guard_consts` as `icmp` literals in `b0` (branch-steering fodder).
    fn module_with_loop(guard_consts: &[i64]) -> AirModule {
        let b0 = BlockId(make_id("block", b"main_b0"));
        let b1 = BlockId(make_id("block", b"main_b1"));
        let mut block0 = AirBlock::new(b0);
        let mut constants: BTreeMap<ValueId, Constant> = BTreeMap::new();
        for (i, &k) in guard_consts.iter().enumerate() {
            let cvid = ValueId(make_id("value", format!("c{i}").as_bytes()));
            let xvid = ValueId(make_id("value", format!("x{i}").as_bytes()));
            constants.insert(cvid, Constant::int(k, 32));
            block0.instructions.push(Instruction {
                id: InstId(make_id("inst", format!("i{i}").as_bytes())),
                op: Operation::BinaryOp {
                    kind: BinaryOp::ICmpEq,
                },
                operands: vec![xvid, cvid],
                dst: Some(ValueId(make_id("value", format!("r{i}").as_bytes()))),
                span: None,
                symbol: None,
                result_type: None,
                extensions: BTreeMap::new(),
            });
        }
        // b0 terminator: unconditional branch to b1.
        block0.instructions.push(Instruction {
            id: InstId(make_id("inst", b"b0_term")),
            op: Operation::Br { target: b1 },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        // b1 terminator: unconditional branch back to b0 (the back-edge).
        let mut block1 = AirBlock::new(b1);
        block1.instructions.push(Instruction {
            id: InstId(make_id("inst", b"b1_term")),
            op: Operation::Br { target: b0 },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        });
        let main = AirFunction {
            id: FunctionId(make_id("func", b"main")),
            name: "main".to_string(),
            params: Vec::new(),
            blocks: vec![block0, block1],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let mut module = AirModule::new(ModuleId(make_id("module", b"test_loop")));
        module.functions.push(main);
        module.constants = constants;
        module
    }

    #[test]
    fn overflow_positional_targets_are_width_specific() {
        let ilp32 = overflow_positional_targets(saf_svcomp::DataModel::ILP32);
        let lp64 = overflow_positional_targets(saf_svcomp::DataModel::LP64);
        // 32-bit boundaries + the negated-value probe present under both models.
        assert!(ilp32.contains(&2_147_483_647), "INT_MAX under ILP32");
        assert!(ilp32.contains(&-1), "-1 (negation seed) under ILP32");
        // ILP32 must NOT carry a 64-bit literal (the driver's atol would overflow).
        assert!(!ilp32.contains(&i64::MAX));
        // LP64 adds the 64-bit boundaries on top.
        assert!(lp64.contains(&i64::MAX), "LONG_MAX under LP64");
        assert!(lp64.contains(&i64::MIN), "LONG_MIN under LP64");
        assert!(lp64.len() > ilp32.len());
    }

    #[test]
    fn overflow_boundary_consts_are_width_specific() {
        let ilp32 = overflow_boundary_consts(saf_svcomp::DataModel::ILP32);
        let lp64 = overflow_boundary_consts(saf_svcomp::DataModel::LP64);
        // 32-bit boundaries always present; INT_MAX is the load-bearing one.
        assert!(ilp32.contains(&2_147_483_647), "INT_MAX under ILP32");
        // ILP32 must NOT carry a 64-bit literal (the driver's atol would overflow).
        assert!(!ilp32.contains(&9_223_372_036_854_775_807));
        // LP64 adds the 64-bit boundaries on top of the 32-bit ones.
        assert!(lp64.contains(&2_147_483_647), "INT_MAX under LP64");
        assert!(
            lp64.contains(&9_223_372_036_854_775_807),
            "LONG_MAX under LP64"
        );
        assert!(lp64.len() > ilp32.len());
    }

    #[test]
    fn overflow_candidates_inject_boundary_when_loop_free() {
        // A single-block main (no back-edge) is loop-free -> INT_MAX is injected.
        let module = module_with_guard_constants(&[]);
        let got = overflow_replay_candidates(&module, saf_svcomp::DataModel::LP64);
        // Same constant SET as before (order is now by yield): every fixed spread value
        // is still probed, plus the boundary probes.
        for &v in OVERFLOW_CONSTS {
            assert!(
                got.contains(&v),
                "fixed spread value {v} must still be probed"
            );
        }
        assert!(
            got.contains(&2_147_483_647),
            "loop-free program must get the INT_MAX boundary probe"
        );
        assert!(
            got.contains(&9_223_372_036_854_775_807),
            "LP64 loop-free program must get the 64-bit boundary probe"
        );
        assert!(got.len() <= OVERFLOW_MAX_CANDIDATES);
        // Yield order: the largest magnitude runs first, `0` runs last.
        assert_eq!(
            got.first(),
            Some(&9_223_372_036_854_775_807),
            "highest-magnitude probe (LONG_MAX) runs first"
        );
        assert_eq!(got.last(), Some(&0), "the 0 probe runs last");
    }

    #[test]
    fn overflow_candidates_omit_boundary_when_looping() {
        // A program with a reachable CFG loop must NOT get the boundary probes — this
        // is the gate that keeps the termination-* loop-counter false alarm out.
        let module = module_with_loop(&[]);
        let got = overflow_replay_candidates(&module, saf_svcomp::DataModel::LP64);
        // Same SET as the committed fixed spread (no boundary), only reordered by yield.
        let mut sorted_got = got.clone();
        sorted_got.sort_unstable();
        let mut expect = OVERFLOW_CONSTS.to_vec();
        expect.sort_unstable();
        assert_eq!(
            sorted_got, expect,
            "looping program keeps exactly the committed fixed spread (set)"
        );
        assert!(
            !got.contains(&2_147_483_647),
            "no INT_MAX boundary under a loop"
        );
    }

    #[test]
    fn overflow_consts_carry_a_sound_two_operand_addition_probe() {
        // The two-operand-ADDITION probe (`y = y + x`, both operands ~v) must satisfy
        // two properties simultaneously:
        //   (1) `2·v` overflows `int` (`2·v > INT_MAX`) so the direct sum traps, and
        //   (2) `v < INT_MAX` with margin, so a `+k` loop counter cannot reach `INT_MAX`
        //       under it (the documented termination-* false-alarm boundary is EXACTLY
        //       `INT_MAX`).
        const INT_MAX: i64 = 2_147_483_647;
        let addition_probes: Vec<i64> = OVERFLOW_CONSTS
            .iter()
            .copied()
            .filter(|&v| v > 0 && v < INT_MAX && 2 * v > INT_MAX)
            .collect();
        assert!(
            addition_probes.iter().any(|&v| v == 1_500_000_000),
            "OVERFLOW_CONSTS must include the 1.5e9 two-operand-addition probe"
        );
        for &v in &addition_probes {
            assert!(2 * v > INT_MAX, "probe {v} must make a direct sum overflow");
            assert!(
                v < INT_MAX,
                "probe {v} must stay below INT_MAX (counter safety)"
            );
            // A comfortable margin below INT_MAX so no realistic `+k` step reaches it.
            assert!(
                INT_MAX - v > 100_000_000,
                "probe {v} must keep a wide margin below INT_MAX"
            );
        }
    }

    #[test]
    fn overflow_candidates_are_ordered_by_descending_yield() {
        // The sweep is ordered so high-magnitude direct-overflow probes run first and
        // the (non-overflowing) `0` probe runs last — the load-bearing ordering that
        // lets long-running loops confirm before the per-candidate budget is spent.
        let module = module_with_guard_constants(&[777]);
        let got = overflow_replay_candidates(&module, saf_svcomp::DataModel::LP64);
        // Magnitudes are non-increasing until the final `0`.
        let nonzero: Vec<i64> = got.iter().copied().filter(|&v| v != 0).collect();
        for w in nonzero.windows(2) {
            assert!(
                w[0].unsigned_abs() >= w[1].unsigned_abs(),
                "candidates must be ordered by descending magnitude: {w:?}"
            );
        }
        assert_eq!(got.last(), Some(&0), "0 is probed last");
        // The boundary probe still precedes the smaller branch-steered guard literal.
        let pos_boundary = got.iter().position(|&v| v == 2_147_483_647).unwrap();
        let pos_guard = got.iter().position(|&v| v == 777).unwrap();
        assert!(pos_boundary < pos_guard);
    }

    #[test]
    fn custom_verifier_atomic_fns_collects_named_atomics() {
        let src = "\
void __VERIFIER_atomic_begin(void);
void __VERIFIER_atomic_end(void);
void __VERIFIER_atomic_acquire(void) { }
void worker(void) { __VERIFIER_atomic_inc(&g); __VERIFIER_atomic_acquire(); }
";
        let got = custom_verifier_atomic_fns(src);
        // begin/end are modelled separately and must NOT be collected.
        assert!(!got.contains("__VERIFIER_atomic_begin"));
        assert!(!got.contains("__VERIFIER_atomic_end"));
        // Named custom atomics are collected (deduplicated, sorted).
        assert!(got.contains("__VERIFIER_atomic_acquire"));
        assert!(got.contains("__VERIFIER_atomic_inc"));
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn custom_verifier_atomic_fns_empty_when_only_begin_end() {
        let src = "void f(void){__VERIFIER_atomic_begin();g++;__VERIFIER_atomic_end();}";
        assert!(custom_verifier_atomic_fns(src).is_empty());
    }

    #[test]
    fn tsan_no_longer_abstains_on_custom_atomics() {
        // Custom whole-function atomics are now modelled, not abstained. Only
        // OpenMP / relaxed-memory remain out of scope.
        assert_eq!(
            tsan_out_of_scope("void __VERIFIER_atomic_foo(void){}"),
            None
        );
        assert_eq!(tsan_out_of_scope("#pragma omp parallel"), Some("OpenMP"));
        assert_eq!(
            tsan_out_of_scope("atomic_load_explicit(&x, memory_order_acquire)"),
            Some("relaxed-memory atomics")
        );
    }

    #[test]
    fn race_true_out_of_scope_gates_omp_relaxed_and_asm() {
        // The TRUE prover inherits the FALSE scope gate plus inline-asm rejection.
        assert_eq!(race_true_out_of_scope("int main(){return 0;}"), None);
        assert_eq!(
            race_true_out_of_scope("#pragma omp parallel"),
            Some("OpenMP")
        );
        assert_eq!(
            race_true_out_of_scope("atomic_load_explicit(&x, memory_order_acquire)"),
            Some("relaxed-memory atomics")
        );
        assert_eq!(
            race_true_out_of_scope("__asm__ volatile(\"mfence\")"),
            Some("inline asm")
        );
        assert_eq!(
            race_true_out_of_scope("asm volatile(\"nop\")"),
            Some("inline asm")
        );
        // An asm symbol-rename LABEL is inert (link alias only) — must NOT gate.
        assert_eq!(
            race_true_out_of_scope(
                "extern int f(int) __asm__(\"\" \"__sigsetjmp\") __attribute__((__nothrow__));"
            ),
            None
        );
    }

    #[test]
    fn effectful_inline_asm_distinguishes_labels_from_statements() {
        // Symbol-rename labels (only string literals in parens) are inert.
        assert!(!contains_effectful_inline_asm(
            "void g(void) __asm__(\"g64\");"
        ));
        assert!(!contains_effectful_inline_asm(
            "extern int __sigsetjmp_cancel(void*) __asm__ (\"\" \"__sigsetjmp\");"
        ));
        assert!(!contains_effectful_inline_asm("int main(){return 0;}"));
        // `asm` appearing as a substring of an identifier must not match.
        assert!(!contains_effectful_inline_asm(
            "int wasm; struct { int basmati; } s;"
        ));
        // Real inline-asm statements (volatile / goto / extended `:` operands / a
        // bare mnemonic) are effectful ⇒ gate.
        assert!(contains_effectful_inline_asm(
            "asm volatile(\"mfence\" ::: \"memory\");"
        ));
        assert!(contains_effectful_inline_asm(
            "__asm__ __volatile__(\"pause\");"
        ));
        assert!(contains_effectful_inline_asm(
            "__asm__ (\"addl %1,%0\" : \"=r\"(out) : \"r\"(in));"
        ));
        assert!(contains_effectful_inline_asm(
            "asm goto(\"jmp %l0\" ::::lbl);"
        ));
        // A multi-token / operand-bearing template carries non-symbol chars
        // (space, `;`, `%`) ⇒ effectful, even without a colon section.
        assert!(contains_effectful_inline_asm("asm(\"rep; nop\"); int x;"));
        assert!(contains_effectful_inline_asm(
            "__asm__(\"movl $0, (%eax)\");"
        ));
        // A degenerate single-mnemonic basic asm has no operand ⇒ names no memory ⇒
        // classified as an inert label (sound for the data-race model).
        assert!(!contains_effectful_inline_asm("__asm(\"nop\");"));
    }

    #[test]
    fn bare_inline_definition_is_detected_for_gnu89_reingest() {
        // A C99 bare `inline` helper (dropped to a bodyless declaration by the
        // default compile) is detected — this is the pthread-ext fmaxsym/stack/inc
        // shape.
        let src = "pthread_mutex_t m;\ninline void findMax(int offset)\n{\n  pthread_mutex_lock(&m);\n}\n";
        assert!(source_has_bare_inline_def(src, "findMax"));
        assert!(source_has_bare_inline_def(
            "inline unsigned inc() { return 0; }",
            "inc"
        ));
        assert!(source_has_bare_inline_def(
            "inline int push(int d) {\n return d;\n}",
            "push"
        ));

        // `static inline` / `extern inline` DO emit an out-of-line body under the
        // default compile, so re-ingesting would not help — must NOT match.
        assert!(!source_has_bare_inline_def(
            "static inline int helper(void) { return 1; }",
            "helper"
        ));
        assert!(!source_has_bare_inline_def(
            "extern inline int helper(void) { return 1; }",
            "helper"
        ));
        // The GNU `__inline` spelling (glibc's `extern __inline`) must not match.
        assert!(!source_has_bare_inline_def(
            "extern __inline int __attribute__((__gnu_inline__)) foo(void) { return 0; }",
            "foo"
        ));
        // A plain (non-inline) definition or an unrelated name must not match.
        assert!(!source_has_bare_inline_def(
            "void findMax(int o) { }",
            "findMax"
        ));
        assert!(!source_has_bare_inline_def(
            "inline void findMax(int o) { }",
            "other"
        ));
        // A substring name must not spuriously match (`Max` inside `findMax`).
        assert!(!source_has_bare_inline_def(
            "inline void findMax(int o) { }",
            "Max"
        ));
        // The `inline` of a PRIOR definition must not leak across the `}` boundary
        // to a following plain definition.
        assert!(!source_has_bare_inline_def(
            "inline void a(void) { }\nvoid b(void) { }",
            "b"
        ));
    }

    #[test]
    fn driver_models_custom_atomics_with_instrument_hooks() {
        let mut fns = std::collections::BTreeSet::new();
        fns.insert("__VERIFIER_atomic_acquire".to_string());
        fns.insert("__VERIFIER_atomic_release".to_string());
        let d = synthesize_tsan_driver(&fns);
        // Forward declares each atomic and takes its address into the table.
        assert!(d.contains("extern void __VERIFIER_atomic_acquire();"));
        assert!(d.contains("(void*)__VERIFIER_atomic_release"));
        // Emits the instrument-functions hooks locking the SAME global mutex.
        assert!(d.contains("__cyg_profile_func_enter"));
        assert!(d.contains("__cyg_profile_func_exit"));
        assert!(d.contains("pthread_mutex_lock(&__saf_atomic)"));
        assert!(d.contains("no_instrument_function"));
        // Deterministic: identical inputs → identical bytes.
        assert_eq!(d, synthesize_tsan_driver(&fns));
    }

    #[test]
    fn driver_omits_hooks_when_no_custom_atomics() {
        let d = synthesize_tsan_driver(&std::collections::BTreeSet::new());
        // begin/end are still modelled, but no instrument-functions hooks are
        // emitted so the common case is unperturbed.
        assert!(d.contains("__VERIFIER_atomic_begin"));
        assert!(!d.contains("__cyg_profile_func_enter"));
    }

    #[test]
    fn asan_driver_emits_split_threshold_branch() {
        // The memsafety threshold sweep drives the driver through SAF_SPLIT_IDX: the
        // first `split` scalar-nondet calls return SAF_BASE_VAL, later ones SAF_TARGET_VAL.
        let d = synthesize_asan_driver();
        assert!(d.contains("SAF_SPLIT_IDX"), "{d}");
        // The threshold branch is checked BEFORE the existing single-index targeting so
        // the committed SAF_TARGET_IDX behaviour is preserved when SAF_SPLIT_IDX is unset.
        let split_at = d.find("SAF_SPLIT_IDX").expect("split env present");
        let target_idx_at = d.find("SAF_TARGET_IDX").expect("target-idx env present");
        assert!(
            split_at < target_idx_at,
            "split branch must precede target-idx"
        );
        // Deterministic generation.
        assert_eq!(d, synthesize_asan_driver());
    }

    #[test]
    fn memsafety_split_sweep_is_enter_then_violate() {
        // Every triple sets up with at least one in-range leading call (`split >= 1`) and
        // then violates: the target is an OOB index/offset (negative, or well past any
        // small in-range base), never equal to a plausible in-bounds value like the base.
        assert!(!MEMSAFETY_SPLIT_SWEEP.is_empty());
        for &(split, base, target) in MEMSAFETY_SPLIT_SWEEP {
            assert!(split >= 1, "need a setup phase: {split}");
            assert_ne!(
                base, target,
                "base and target must differ to exercise two phases"
            );
        }
        // The no-args vector (committed behaviour) and a well-formed multi-arg vector
        // (to pass an `argc < 2` gate) are both swept.
        assert!(MEMSAFETY_SPLIT_ARGV.iter().any(|a| a.is_empty()));
        assert!(MEMSAFETY_SPLIT_ARGV.iter().any(|a| a.len() >= 2));
    }
}
