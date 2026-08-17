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
        if let Some(graphml) = &outcome.graphml {
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
        } else if let Some(witness) = &outcome.witness {
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
        .args(["-S", "-passes=mem2reg"])
        .arg(&ir)
        .arg("-o")
        .arg(&ir)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn {opt}"))?;
    anyhow::ensure!(opt_status.success(), "{opt} mem2reg pass failed");

    Ok(ir)
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
    // unconditional) is already sound above; the replay-confirmed stages below are not.
    // Abstain when a thread spawn is reachable from `main`, matching the R5/R6 ASan/UBSan
    // confirmers' gate (plan 198). Sound over-approximation: never miss a spawn.
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if saf_svcomp::fast_paths::reachable_spawns_threads(ctx.module, &callgraph) {
        eprintln!(
            "saf verify: a thread spawn is reachable from main (unreach replay is schedule-unsound) -> unknown"
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

    let total = candidates.len() + interproc.len();
    if total == 0 {
        eprintln!(
            "saf verify: no FALSE candidate proposed (reach_error not proven reachable) -> unknown"
        );
    } else {
        // Candidates were over-approximated as FALSE but did not reproduce under
        // concrete replay — the soundness filter that keeps false alarms out.
        eprintln!(
            "saf verify: {total} candidate(s) enumerated ({} intraproc + {} interproc); none reproduced reach_error at runtime -> unknown",
            candidates.len(),
            interproc.len()
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
    match fuzz_confirm_false(ctx) {
        Some(candidate) => {
            let witness = build_witness(ctx, saf_svcomp::lower_candidate(ctx.module, &candidate));
            if witness.is_none() {
                eprintln!(
                    "saf verify: FALSE (fuzz replay-confirmed) but witness unconstructible -> emitting false without a witness"
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

/// Internal cap on blind-fuzz iterations (mutation trials), overridable via
/// `$SAF_FUZZ_ITERS`. Bounds worst-case native run time; a dictionary-steered
/// guard is usually hit in the seed corpus or the first handful of trials.
fn fuzz_iters() -> usize {
    std::env::var("SAF_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(600)
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

    let dir = ctx.tempdir;
    let sentinel = dir.join("saf_fuzz.sentinel");
    let driver_src = dir.join("saf_fuzz_driver.c");
    let harness = dir.join("saf_fuzz_harness");
    let input_path = dir.join("saf_fuzz.input");
    let log_path = dir.join("saf_fuzz.log");

    if std::fs::write(
        &driver_src,
        fuzz::synthesize_bytestream_driver(&escape_c_string(&sentinel)),
    )
    .is_err()
    {
        return None;
    }

    // Compile the shim + original program ONCE (native, no sanitizer).
    let srcdir = ctx.input.parent().unwrap_or_else(|| Path::new("."));
    let compiled = Command::new(ctx.clang)
        .args(["-O0", "-Wno-everything"])
        .arg(ctx.data_model.clang_flag())
        .arg("-include")
        .arg(ctx.stub)
        .arg("-I")
        .arg(srcdir)
        .arg(ctx.input)
        .arg(&driver_src)
        .arg("-o")
        .arg(&harness)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match compiled {
        Ok(s) if s.success() => {}
        _ => return None, // link/compile failure -> inconclusive
    }

    let dict = fuzz::harvest_dictionary(ctx.module);
    let mut corpus = fuzz::seed_corpus(&dict);
    // Fixed seed -> the whole search (and therefore the verdict) is reproducible.
    let mut rng = fuzz::XorShift64::new(0x5AF3_C0DE);
    let per_run = replay_timeout();
    let iters = fuzz_iters();
    let deadline = std::time::Instant::now() + fuzz_time_budget();
    let mut max_depth = 0usize;

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
        } else {
            let base = &corpus[rng.below(corpus.len())];
            fuzz::mutate(&mut rng, base, &dict)
        };

        if std::fs::write(&input_path, &input).is_err() {
            continue;
        }
        let _ = std::fs::remove_file(&sentinel);
        let _ = std::fs::remove_file(&log_path);

        if run_fuzz_harness(&harness, &input_path, &log_path, per_run).is_err() {
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
        } else if i >= corpus.len() {
            // Greybox corpus feedback: keep inputs that reached deeper.
            let depth = std::fs::read_to_string(&log_path)
                .map(|l| l.lines().count())
                .unwrap_or(0);
            if depth > max_depth && corpus.len() < MAX_FUZZ_CORPUS {
                max_depth = depth;
                corpus.push(input);
            }
        }
    }

    eprintln!("saf verify: blind fuzz exhausted (no confirmed reach_error) -> unknown");
    None
}

/// Corpus size cap for the greybox feedback loop — bounds memory and keeps the
/// mutation base-selection distribution stable.
const MAX_FUZZ_CORPUS: usize = 256;

/// Run the byte-stream fuzz harness on one input under a short timeout, feeding
/// `$SAF_FUZZ_INPUT` / `$SAF_FUZZ_LOG`. Success/normal-exit/timeout all return
/// `Ok(())`; the caller inspects the sentinel/log. A runaway harness is killed.
fn run_fuzz_harness(
    harness: &Path,
    input_path: &Path,
    log_path: &Path,
    timeout: std::time::Duration,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let mut child = Command::new(harness)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("SAF_FUZZ_INPUT", input_path)
        .env("SAF_FUZZ_LOG", log_path)
        .spawn()
        .with_context(|| "spawning fuzz harness")?;

    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
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
/// The large POSITIVE probe is `2^30`, NOT `INT_MAX`. When a nondet drives a loop trip
/// count, `for (i=0; i<=x; i++)` (Parts) or `while (z>0) { x=x+1; z=z-1; }` (ESOP2008),
/// the counter/accumulator reaches ~`x`; at `x == INT_MAX` the next `+1` is a spurious
/// `INT_MAX + 1` overflow that SV-COMP's no-overflow benchmarks label TRUE (the
/// termination-* families — 2 full-pool false alarms, indistinguishable in the `UBSan`
/// report from a genuine `x+1`-at-INT_MAX). At `2^30` a loop counter/accumulator stays
/// under `INT_MAX` (no false alarm), while genuine large-value overflows still trap
/// (`2^30 + 2^30`, `2^30 * 2`, `2^30 + 2^30` all exceed `INT_MAX`). The only loss is a
/// direct `x+1`-EXACTLY-at-INT_MAX overflow, which cannot be caught without re-admitting
/// the loop false alarms — soundness (FP=0) is worth more than that ambiguous case.
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
    -1,
    -2_147_483_648,
    2_147_483_648,
];

/// Cap on how many candidates to replay per task — bounds worst-case native
/// compile+run time; a real violation almost always surfaces in the first
/// candidate. Dropped candidates are logged implicitly by not confirming.
const MAX_REPLAY_CANDIDATES: usize = 16;

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
    let status = Command::new(clang)
        .args(["-O0", "-Wno-everything"])
        .arg(data_model.clang_flag())
        .arg("-include")
        .arg(stub)
        .arg("-I")
        .arg(srcdir)
        .arg(input)
        .arg(&driver_src)
        .arg("-o")
        .arg(&harness)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn {clang} for native replay"))?;
    if !status.success() {
        // Link/compile failure (e.g. the task inlines its own reach_error) —
        // inconclusive, not a violation.
        return Ok(false);
    }

    let mut child = Command::new(&harness)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| "spawning replay harness")?;

    let timeout = replay_timeout();
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
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
    let timeout = replay_timeout();

    // One ASan compile-and-mini-fuzz pass. `auto_var_init` toggles the second
    // (uninitialized-variable) pass; see the two-pass rationale at the call below.
    // Returns `Ok(Some(hit))` on the first constant that reproduces a violation,
    // `Ok(None)` if this pass is inconclusive (compile/link failure or no trap).
    let run_pass = |auto_var_init: bool| -> anyhow::Result<Option<saf_svcomp::AsanHit>> {
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
        let status = cmd
            .arg(data_model.clang_flag())
            .arg("-include")
            .arg(stub)
            .arg("-I")
            .arg(srcdir)
            .arg(input)
            .arg(&driver_src)
            .arg("-o")
            .arg(&harness)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .with_context(|| format!("failed to spawn {clang} for ASan replay"))?;
        if !status.success() {
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
        for &k in NONDET_CONSTS {
            // Redirect the child's stderr to a FILE (not a pipe) so a large ASan report
            // cannot deadlock on a full pipe buffer while we poll for the timeout.
            let errfile =
                std::fs::File::create(&errpath).with_context(|| "creating ASan stderr file")?;
            let mut child = Command::new(&harness)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::from(errfile))
                .env("ASAN_OPTIONS", ASAN_OPTS)
                .env("SAF_NONDET_CONST", k.to_string())
                .spawn()
                .with_context(|| "spawning ASan harness")?;

            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            let _ = child.wait();
                            break; // runaway -> parse whatever exists (likely no report)
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(e) => return Err(e).context("waiting on ASan harness"),
                }
            }

            let report = std::fs::read_to_string(&errpath).unwrap_or_default();
            if let Some(hit) = saf_svcomp::parse_asan_report(&report) {
                return Ok(Some(hit)); // first constant that reproduces a violation wins
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
    run_pass(true)
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
    if saf_svcomp::program_structurally_terminates(ctx.module) {
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

/// The `no-data-race` FALSE pipeline (lever `race-find`): finder-gated,
/// `ThreadSanitizer`-confirmed, `GraphML`-witnessed.
///
/// 1. Gate: require an actually-reachable thread spawn (else the program is
///    sequential — no race possible — and we abstain; SAF is FALSE-only).
/// 2. R7 scope gate: abstain on `OpenMP` / relaxed-memory / custom
///    `__VERIFIER_atomic_*` sections a native x86 (TSO/SC) `TSan` replay cannot
///    soundly arbitrate.
/// 3. Propose: run the over-approximate lockset+MHP finder
///    ([`saf_svcomp::find_race_candidates`]). No candidate ⇒ abstain.
/// 4. Confirm: compile the ORIGINAL program with `-fsanitize=thread` and run it
///    under the nondet driver; emit `false(no-data-race)` IFF `TSan` concretely
///    observes a genuine data race (R1). `TSan` is the sole soundness arbiter; the
///    witness is `GraphML` 1.0 (R7).
fn no_data_race_strategy(ctx: &VerifyCtx) -> VerdictOutcome {
    // Gate 1: a race needs a real second thread reachable from main.
    let callgraph = saf_analysis::callgraph::CallGraph::build(ctx.module);
    if !saf_svcomp::fast_paths::reachable_spawns_threads(ctx.module, &callgraph) {
        eprintln!("saf verify: no reachable thread spawn (sequential -> no race) -> unknown");
        return unknown_outcome();
    }

    // Gate 2 (R7): out-of-scope concurrency features TSan-on-x86 cannot soundly
    // arbitrate. Read the source once (also used for the witness hash upstream).
    let source = std::fs::read_to_string(ctx.input).unwrap_or_default();
    if let Some(reason) = tsan_out_of_scope(&source) {
        eprintln!("saf verify: no-data-race out of scope ({reason}) -> unknown");
        return unknown_outcome();
    }

    // Gate 3: the finder must PROPOSE at least one candidate racing pair. An
    // empty result means the over-approximate lockset+MHP analysis proved the
    // program race-free, so we abstain without paying for a TSan run.
    let candidates = saf_svcomp::find_race_candidates(ctx.module);
    if candidates.is_empty() {
        eprintln!("saf verify: lockset+MHP finder found no race candidate -> unknown");
        return unknown_outcome();
    }
    eprintln!(
        "saf verify: finder proposed {} race candidate(s); attempting TSan confirmation",
        candidates.len()
    );

    // Gate 4: TSan is the sole soundness arbiter (HB-based — only reports a race
    // it concretely observes).
    match tsan_confirm(ctx.input, ctx.data_model, ctx.stub, ctx.tempdir, ctx.clang) {
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
            VerdictOutcome {
                verdict: format!("false({})", saf_svcomp::Property::NoDataRace.name()),
                witness: None,
                graphml: Some(graphml),
            }
        }
        Ok(None) => {
            eprintln!("saf verify: TSan replay observed no data race -> unknown");
            unknown_outcome()
        }
        Err(e) => {
            eprintln!("saf verify: TSan replay errored: {e:#} -> unknown");
            unknown_outcome()
        }
    }
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
/// - custom whole-function atomic sections `__VERIFIER_atomic_<name>` (other
///   than the modelled `begin`/`end`) — SV-COMP executes these atomically but
///   `TSan` does not model them, so it would false-alarm a race the semantics
///   forbid.
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
    if references_custom_verifier_atomic(source) {
        return Some("custom __VERIFIER_atomic_* section");
    }
    None
}

/// True if the source references a `__VERIFIER_atomic_<suffix>` identifier whose
/// suffix is NOT the modelled `begin`/`end` — i.e. a custom whole-function
/// atomic that `TSan` cannot arbitrate.
fn references_custom_verifier_atomic(source: &str) -> bool {
    const MARK: &str = "__VERIFIER_atomic_";
    let mut i = 0;
    while let Some(off) = source[i..].find(MARK) {
        let start = i + off + MARK.len();
        let suffix: String = source[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !suffix.is_empty() && suffix != "begin" && suffix != "end" {
            return true;
        }
        // Advance past this occurrence (at least one byte) to avoid looping.
        i = start.max(i + MARK.len());
    }
    false
}

/// Build the `TSan`-replay driver: SV-COMP nondet generators (returning
/// `$SAF_NONDET_CONST`, default 0), `__VERIFIER_assume` as a hard path filter
/// (R4), and `__VERIFIER_atomic_begin/end` modelled as a global RECURSIVE mutex
/// so `TSan` sees the atomic-section synchronization (else it would false-alarm a
/// race the SV-COMP semantics forbid). `rand`/`srand` are pinned for determinism.
fn synthesize_tsan_driver() -> String {
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
    s
}

/// Confirm a `no-data-race` FALSE by `ThreadSanitizer`-instrumented native
/// execution.
///
/// Compiles the ORIGINAL program with `-fsanitize=thread -pthread -g` + the
/// nondet driver, runs it under the [`NONDET_CONSTS`] mini-fuzz capturing stderr
/// to a file, and parses the report ([`saf_svcomp::parse_tsan_report`], which
/// applies R1 — only a genuine `data race` warning confirms). `Ok(Some(hit))` is
/// a confirmed race; `Ok(None)` is inconclusive (no report / compile-link failure
/// / timeout) ⇒ the caller keeps `unknown`. Because `TSan` is happens-before-based,
/// a reported race is a real race for the observed inputs, so confirming is sound;
/// a missing 32-bit `TSan` runtime for an `ILP32` task abstains (R3).
fn tsan_confirm(
    input: &Path,
    data_model: saf_svcomp::DataModel,
    stub: &Path,
    dir: &Path,
    clang: &str,
) -> anyhow::Result<Option<saf_svcomp::RaceHit>> {
    use anyhow::Context;
    use std::process::{Command, Stdio};

    let driver_src = dir.join("saf_tsan_driver.c");
    let harness = dir.join("saf_tsan_harness");
    let errpath = dir.join("saf_tsan_stderr.txt");

    std::fs::write(&driver_src, synthesize_tsan_driver())
        .with_context(|| "writing TSan replay driver")?;

    let srcdir = input.parent().unwrap_or_else(|| Path::new("."));
    let status = Command::new(clang)
        .args([
            "-O0",
            "-g",
            "-fsanitize=thread",
            "-pthread",
            "-Wno-everything",
            // Determinism: redirect rand()/srand() to the driver's __wrap_* stubs.
            "-Wl,--wrap=rand",
            "-Wl,--wrap=srand",
        ])
        .arg(data_model.clang_flag())
        .arg("-include")
        .arg(stub)
        .arg("-I")
        .arg(srcdir)
        .arg(input)
        .arg(&driver_src)
        .arg("-o")
        .arg(&harness)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("failed to spawn {clang} for TSan replay"))?;
    if !status.success() {
        // Compile/link failure (e.g. a missing 32-bit TSan runtime) -> inconclusive.
        return Ok(None);
    }

    let timeout = replay_timeout();
    for &k in NONDET_CONSTS {
        // Redirect the child's stderr to a FILE (not a pipe) so a large TSan
        // report cannot deadlock on a full pipe buffer while we poll the timeout.
        let errfile =
            std::fs::File::create(&errpath).with_context(|| "creating TSan stderr file")?;
        let mut child = Command::new(&harness)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errfile))
            .env("TSAN_OPTIONS", TSAN_OPTS)
            .env("SAF_NONDET_CONST", k.to_string())
            .spawn()
            .with_context(|| "spawning TSan harness")?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
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
    // guesses `K`. Harvested literals are magnitude-capped below 2^30 (so no steered
    // value can widen the loop-counter-to-INT_MAX false-alarm surface) and are still
    // just concrete nondet inputs — soundness is unchanged (a trap is re-triggered on
    // the ORIGINAL program). The fixed spread runs FIRST so committed behavior is a
    // prefix (0 regression); the total is capped at MAX_REPLAY_CANDIDATES.
    let mut candidates: Vec<i64> = OVERFLOW_CONSTS.to_vec();
    for k in saf_svcomp::fast_paths::branch_steering_constants(module) {
        if candidates.len() >= MAX_REPLAY_CANDIDATES {
            break;
        }
        if !candidates.contains(&k) {
            candidates.push(k);
        }
    }

    let timeout = replay_timeout();
    for &k in &candidates {
        let errfile =
            std::fs::File::create(&errpath).with_context(|| "creating UBSan stderr file")?;
        let mut child = Command::new(&harness)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(errfile))
            .env("UBSAN_OPTIONS", UBSAN_OPTS)
            .env("SAF_NONDET_CONST", k.to_string())
            .spawn()
            .with_context(|| "spawning UBSan harness")?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        break; // runaway -> parse whatever exists (likely no report)
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(e).context("waiting on UBSan harness"),
            }
        }

        let report = std::fs::read_to_string(&errpath).unwrap_or_default();
        if let Some(hit) = saf_svcomp::parse_ubsan_overflow(&report) {
            return Ok(Some(hit)); // first constant that reproduces an overflow wins
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
    fn assert_neutralizer_is_byte_stable() {
        // Determinism (NFR-DET): identical content on every write.
        let dir = tempfile::tempdir().expect("tempdir");
        let a = std::fs::read_to_string(write_assert_neutralizer(dir.path()).unwrap()).unwrap();
        let b = std::fs::read_to_string(write_assert_neutralizer(dir.path()).unwrap()).unwrap();
        assert_eq!(a, b);
    }
}
