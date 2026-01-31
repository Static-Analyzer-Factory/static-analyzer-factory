//! CruxBC scalability benchmark runner.
//!
//! Runs real-world C programs through SAF's full analysis pipeline and measures
//! performance at each phase. Programs are categorized by size:
//!
//! All categories run the full pipeline: CG refinement -> Andersen -> MSSA+SVFG -> FS-PTA.
//! The driver temporarily frees heavy graphs during the FS-PTA chain to avoid OOM.
#![allow(clippy::doc_markdown)]

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use saf_cli::bench_types::{AnalysisFlags, BenchConfig, BenchPtaConfig};
use saf_core::config::PtaSolver;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::runner;

// =============================================================================
// Configuration
// =============================================================================

/// Configuration for the CruxBC benchmark runner.
pub struct CruxBcConfig {
    /// Directory containing compiled `.ll`/`.bc` files organized by category.
    pub compiled_dir: PathBuf,
    /// Optional filter for category prefix or substring match.
    pub filter: Option<String>,
    /// PTA solver to use (Legacy or Ascent).
    pub solver: PtaSolver,
}

// =============================================================================
// Runner
// =============================================================================

/// CruxBC scalability benchmark runner.
pub struct CruxBcRunner {
    config: CruxBcConfig,
}

impl CruxBcRunner {
    /// Create a new runner with the given configuration.
    #[must_use]
    pub fn new(config: CruxBcConfig) -> Self {
        Self { config }
    }

    /// Run the benchmark suite and return a summary.
    ///
    /// Each program is analyzed in a separate child process to ensure the OS
    /// reclaims all memory between analyses. The child invokes `saf-bench
    /// cruxbc-single` and writes `ProgramResult` JSON to stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if the current executable path cannot be determined.
    pub fn run(&self) -> Result<CruxBcSummary> {
        let overall_start = Instant::now();

        let programs = discover_programs(&self.config.compiled_dir, self.config.filter.as_deref());
        eprintln!(
            "Discovered {} programs in {}",
            programs.len(),
            self.config.compiled_dir.display()
        );

        let exe = std::env::current_exe().context("failed to determine current executable")?;
        let solver_str = format!("{:?}", self.config.solver).to_lowercase();

        let results: Vec<ProgramResult> = programs
            .iter()
            .map(|prog| run_isolated(&exe, prog, &solver_str))
            .collect();

        Ok(CruxBcSummary {
            suite: "cruxbc".to_string(),
            solver: format!("{:?}", self.config.solver),
            programs: results,
            total_secs: overall_start.elapsed().as_secs_f64(),
        })
    }
}

// =============================================================================
// Result types
// =============================================================================

/// Summary of an entire CruxBC benchmark run.
#[derive(Debug, Serialize)]
pub struct CruxBcSummary {
    /// Suite identifier.
    pub suite: String,
    /// PTA solver used for this run.
    pub solver: String,
    /// Per-program results.
    pub programs: Vec<ProgramResult>,
    /// Total wall-clock time in seconds.
    pub total_secs: f64,
}

/// Result for a single program in the benchmark.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramResult {
    /// Program name (filename without extension).
    pub name: String,
    /// Category (directory name, e.g., "small", "big", "extra").
    pub category: String,
    /// IR statistics after loading.
    pub ir_stats: IrStats,
    /// Phase timings in seconds (keys: "frontend", "cg_refine", "andersen", "mssa_svfg", "fs_pta").
    pub phases: BTreeMap<String, f64>,
    /// Total analysis time in seconds (sum of phases).
    pub total_secs: f64,
    /// Peak RSS in megabytes (VmHWM on Linux, 0 on other platforms).
    #[serde(default)]
    pub peak_rss_mb: usize,
    /// Error message if any phase panicked or failed.
    pub error: Option<String>,
    /// Analysis statistics (constraint counts, PTA metrics, CG stats).
    pub stats: AnalysisStats,
}

/// IR statistics for a loaded program.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IrStats {
    /// Number of defined (non-declaration) functions.
    pub functions: usize,
    /// Total number of instructions across all defined functions.
    pub instructions: usize,
    /// Number of global variables.
    pub globals: usize,
}

/// Analysis statistics for comparison with SVF.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnalysisStats {
    // -- Constraints (pre-HVN) --
    /// Number of address-of constraints.
    pub addr_constraints: usize,
    /// Number of copy constraints.
    pub copy_constraints: usize,
    /// Number of load constraints.
    pub load_constraints: usize,
    /// Number of store constraints.
    pub store_constraints: usize,
    /// Number of GEP (field access) constraints.
    pub gep_constraints: usize,
    // -- Constraints (post-HVN, D7) --
    /// Total constraints after HVN preprocessing.
    pub post_hvn_total_constraints: usize,
    // -- Solver --
    /// PTA solver iterations.
    pub solve_iterations: usize,
    // -- Pointers (D1) --
    /// Values with points-to entries (SAF-style denominator).
    pub pta_pointers: usize,
    /// All pointer-typed values in module (SVF-comparable).
    pub total_pointer_values: usize,
    // -- Locations (D2) --
    /// Unique base objects (`ObjId` count).
    pub obj_count: usize,
    /// Field-expanded locations (`(ObjId, FieldPath)` count).
    pub field_location_count: usize,
    // -- Points-to sizes (D3) --
    /// Average points-to set size (denominator = `pta_pointers`).
    pub avg_pts_size: f64,
    /// Average points-to set size, SVF-style (denominator = `total_pointer_values`).
    pub avg_pts_size_svf: f64,
    /// Maximum points-to set size.
    pub max_pts_size: usize,
    // -- Call graph (D4, D5) --
    /// Call graph nodes excluding `IndirectPlaceholder` (SVF-comparable).
    pub cg_nodes: usize,
    /// Call graph edges per unique caller-callee pair (SAF-style).
    pub cg_edges: usize,
    /// Call graph edges per call site (SVF-comparable).
    pub cg_callsite_edges: usize,
    /// Number of indirect call sites resolved.
    pub indirect_calls_resolved: usize,
    /// Number of `CallIndirect` sites in the AIR (before resolution).
    pub ind_call_sites: usize,
}

// =============================================================================
// Internal: program discovery
// =============================================================================

/// A discovered program to benchmark.
pub struct CruxBcProgram {
    /// Path to the .ll/.bc file.
    pub path: PathBuf,
    /// Category (directory name, e.g., "small", "big", "extra").
    pub category: String,
    /// Program name (filename without extension).
    pub name: String,
}

/// Walk `compiled_dir` for `.ll` and `.bc` files and extract category/name.
fn discover_programs(compiled_dir: &Path, filter: Option<&str>) -> Vec<CruxBcProgram> {
    let mut programs = Vec::new();

    for entry in WalkDir::new(compiled_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ll" && ext != "bc" {
            continue;
        }

        // Extract category from the first path component relative to compiled_dir
        let rel = path.strip_prefix(compiled_dir).unwrap_or(path);
        let category = rel
            .parent()
            .and_then(|p| p.components().next())
            .map_or_else(
                || "uncategorized".to_string(),
                |c| c.as_os_str().to_string_lossy().to_string(),
            );

        let name = path.file_stem().map_or_else(
            || "unknown".to_string(),
            |s| s.to_string_lossy().to_string(),
        );

        // Apply filter: comma-separated substring match on category/name
        if let Some(f) = filter {
            let full = format!("{category}/{name}");
            let matched = f
                .split(',')
                .any(|part| full.contains(part) || category.starts_with(part));
            if !matched {
                continue;
            }
        }

        programs.push(CruxBcProgram {
            path: path.to_path_buf(),
            category,
            name,
        });
    }

    // Sort for deterministic order
    programs.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });

    programs
}

// =============================================================================
// Internal: single-program analysis
// =============================================================================

/// Run all analysis phases on a single program, catching panics.
pub fn run_single_program(prog: &CruxBcProgram, solver: PtaSolver) -> ProgramResult {
    let label = format!("{}/{}", prog.category, prog.name);
    eprintln!("  Analyzing {label} ({solver:?}) ...");

    let result = catch_unwind(AssertUnwindSafe(|| run_program_inner(prog, solver)));

    match result {
        Ok(Ok(pr)) => {
            eprintln!("  {label}: {:.2}s", pr.total_secs);
            pr
        }
        Ok(Err(e)) => {
            let msg = format!("{e:#}");
            eprintln!("  {label}: ERROR {msg}");
            ProgramResult {
                name: prog.name.clone(),
                category: prog.category.clone(),
                ir_stats: IrStats::default(),
                phases: BTreeMap::new(),
                total_secs: 0.0,
                peak_rss_mb: 0,
                error: Some(msg),
                stats: AnalysisStats::default(),
            }
        }
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                (*s).to_string()
            } else {
                "unknown panic".to_string()
            };
            eprintln!("  {label}: PANIC {msg}");
            ProgramResult {
                name: prog.name.clone(),
                category: prog.category.clone(),
                ir_stats: IrStats::default(),
                phases: BTreeMap::new(),
                total_secs: 0.0,
                peak_rss_mb: 0,
                error: Some(format!("panic: {msg}")),
                stats: AnalysisStats::default(),
            }
        }
    }
}

/// Run a single program in an isolated child process.
///
/// Spawns `saf-bench cruxbc-single` as a subprocess. On success, parses the
/// `ProgramResult` JSON from stdout. On failure (crash, OOM, timeout),
/// returns a `ProgramResult` with the error message.
fn run_isolated(exe: &Path, prog: &CruxBcProgram, solver: &str) -> ProgramResult {
    let label = format!("{}/{}", prog.category, prog.name);
    eprintln!("  Analyzing {label} (isolated, {solver}) ...");

    let start = Instant::now();

    let output = std::process::Command::new(exe)
        .arg("cruxbc-single")
        .arg("--path")
        .arg(&prog.path)
        .arg("--category")
        .arg(&prog.category)
        .arg("--solver")
        .arg(solver)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output();

    let elapsed = start.elapsed().as_secs_f64();

    match output {
        Ok(out) if out.status.success() => {
            match serde_json::from_slice::<ProgramResult>(&out.stdout) {
                Ok(pr) => {
                    eprintln!("  {label}: {:.2}s", pr.total_secs);
                    pr
                }
                Err(e) => {
                    let msg = format!("failed to parse child JSON: {e}");
                    eprintln!("  {label}: ERROR {msg}");
                    error_result(&prog.name, &prog.category, msg)
                }
            }
        }
        Ok(out) => {
            // On Unix, OOM-killed processes have no exit code but signal 9
            #[cfg(unix)]
            let msg = {
                use std::os::unix::process::ExitStatusExt;
                if out.status.signal() == Some(9) {
                    format!("killed by OOM (SIGKILL, {elapsed:.1}s)")
                } else if let Some(sig) = out.status.signal() {
                    format!("killed by signal {sig} ({elapsed:.1}s)")
                } else {
                    let code = out.status.code().unwrap_or(-1);
                    format!("child exited with code {code}")
                }
            };
            #[cfg(not(unix))]
            let msg = {
                let code = out.status.code().unwrap_or(-1);
                format!("child exited with code {code}")
            };
            eprintln!("  {label}: ERROR {msg}");
            error_result(&prog.name, &prog.category, msg)
        }
        Err(e) => {
            let msg = format!("failed to spawn child: {e}");
            eprintln!("  {label}: ERROR {msg}");
            error_result(&prog.name, &prog.category, msg)
        }
    }
}

/// Create a `ProgramResult` representing an error.
fn error_result(name: &str, category: &str, error: String) -> ProgramResult {
    ProgramResult {
        name: name.to_string(),
        category: category.to_string(),
        ir_stats: IrStats::default(),
        phases: BTreeMap::new(),
        total_secs: 0.0,
        peak_rss_mb: 0,
        error: Some(error),
        stats: AnalysisStats::default(),
    }
}

/// Inner analysis logic — delegates to `saf run --bench-config` via subprocess.
fn run_program_inner(prog: &CruxBcProgram, solver: PtaSolver) -> Result<ProgramResult> {
    let analysis_start = Instant::now();

    // Build bench config — full pipeline (Andersen + MSSA/SVFG + FS-PTA) for all targets,
    // matching SVF's wpa -fspta benchmark configuration.
    let bench_config = BenchConfig {
        alias_queries: vec![],
        nullness_queries: vec![],
        interval_queries: vec![],
        interleaving_queries: vec![],
        tct_queries: vec![],
        analyses: AnalysisFlags {
            checkers: false,
            fspta: true,
            fspta_skip_df: true,
            ..Default::default()
        },
        pta_config: BenchPtaConfig {
            solver: match solver {
                PtaSolver::Worklist => "worklist".to_string(),
                PtaSolver::Datalog => "datalog".to_string(),
            },
            ..BenchPtaConfig::default()
        },
    };

    // Use temp files for config and result
    let temp_dir = tempfile::TempDir::new()?;
    let config_path = temp_dir.path().join("config.json");
    let result_path = temp_dir.path().join("result.json");

    let bench_result =
        runner::run_saf_bench(&prog.path, &bench_config, &config_path, &result_path)?;

    let total_secs = analysis_start.elapsed().as_secs_f64();

    if !bench_result.success {
        return Ok(ProgramResult {
            name: prog.name.clone(),
            category: prog.category.clone(),
            ir_stats: IrStats::default(),
            phases: BTreeMap::new(),
            total_secs,
            peak_rss_mb: 0,
            error: bench_result.error,
            stats: AnalysisStats::default(),
        });
    }

    // Map BenchResult stats to ProgramResult
    let mut phases = BTreeMap::new();
    phases.insert("total".to_string(), bench_result.stats.total_secs);
    phases.insert("frontend".to_string(), bench_result.stats.frontend_secs);
    phases.insert("andersen".to_string(), bench_result.stats.pta_solve_secs);
    if let Some(v) = bench_result.stats.mssa_svfg_secs {
        phases.insert("mssa_svfg".to_string(), v);
    }
    if let Some(v) = bench_result.stats.fspta_secs {
        phases.insert("fs_pta".to_string(), v);
    }
    if let Some(v) = bench_result.stats.cfg_build_secs {
        phases.insert("cfg_build".to_string(), v);
    }
    if let Some(v) = bench_result.stats.pta_clone_secs {
        phases.insert("pta_clone".to_string(), v);
    }
    if let Some(v) = bench_result.stats.defuse_local_secs {
        phases.insert("defuse_local".to_string(), v);
    }

    // Map IR stats from BenchResult
    let ir_stats = bench_result
        .ir_stats
        .map(|ir| IrStats {
            functions: ir.functions,
            instructions: ir.instructions,
            globals: ir.globals,
        })
        .unwrap_or_default();

    // Map analysis stats from BenchResult
    let stats = bench_result
        .analysis_stats
        .map(|s| AnalysisStats {
            addr_constraints: s.addr_constraints,
            copy_constraints: s.copy_constraints,
            load_constraints: s.load_constraints,
            store_constraints: s.store_constraints,
            gep_constraints: s.gep_constraints,
            post_hvn_total_constraints: s.post_hvn_total_constraints,
            solve_iterations: s.solve_iterations,
            pta_pointers: s.pta_pointers,
            total_pointer_values: s.total_pointer_values,
            obj_count: s.obj_count,
            field_location_count: s.field_location_count,
            avg_pts_size: s.avg_pts_size,
            avg_pts_size_svf: s.avg_pts_size_svf,
            max_pts_size: s.max_pts_size,
            cg_nodes: s.cg_nodes,
            cg_edges: s.cg_edges,
            cg_callsite_edges: s.cg_callsite_edges,
            indirect_calls_resolved: s.indirect_calls_resolved,
            ind_call_sites: s.ind_call_sites,
        })
        .unwrap_or_default();

    Ok(ProgramResult {
        name: prog.name.clone(),
        category: prog.category.clone(),
        ir_stats,
        phases,
        total_secs,
        peak_rss_mb: bench_result.peak_rss_mb,
        error: None,
        stats,
    })
}

// =============================================================================
// Human-readable output
// =============================================================================

/// Format an instruction count with `k` suffix for readability.
fn format_count(n: usize) -> String {
    if n >= 1000 {
        #[allow(clippy::cast_precision_loss)]
        let k = n as f64 / 1000.0;
        format!("{k:.1}k")
    } else {
        n.to_string()
    }
}

/// Print a human-readable summary table to stderr.
pub fn print_human(summary: &CruxBcSummary) {
    eprintln!();
    eprintln!("=== CruxBC Scalability Benchmark ({}) ===", summary.solver);
    eprintln!(
        "Programs: {} | Time: {:.1}s",
        summary.programs.len(),
        summary.total_secs
    );
    eprintln!();

    // Header
    eprintln!(
        "{:<20} {:>6} {:>8}  {:>8} {:>8} {:>8} {:>9} {:>8} {:>8}",
        "Program", "Funcs", "Insts", "Load", "CG Ref", "Ander", "MSSA+VFG", "FS-PTA", "Total"
    );
    eprintln!("{}", "\u{2500}".repeat(99));

    for pr in &summary.programs {
        let label = format!("{}/{}", pr.category, pr.name);
        let fmt_phase = |key: &str| -> String {
            pr.phases
                .get(key)
                .map_or("\u{2014}".to_string(), |v| format!("{v:.2}s"))
        };
        let load = fmt_phase("frontend");
        let cg = fmt_phase("cg_refine");
        let ander = fmt_phase("andersen");
        let mssa = fmt_phase("mssa_svfg");
        let fspta = fmt_phase("fs_pta");

        let status = if pr.error.is_some() { " ERR" } else { "" };

        eprintln!(
            "{:<20} {:>6} {:>8}  {:>8} {:>8} {:>8} {:>9} {:>8} {:>7.2}s{}",
            label,
            pr.ir_stats.functions,
            format_count(pr.ir_stats.instructions),
            load,
            cg,
            ander,
            mssa,
            fspta,
            pr.total_secs,
            status,
        );
    }

    eprintln!("{}", "\u{2500}".repeat(99));

    // Show errors if any
    let errors: Vec<_> = summary
        .programs
        .iter()
        .filter(|p| p.error.is_some())
        .collect();
    if !errors.is_empty() {
        eprintln!();
        eprintln!("Errors:");
        for pr in errors {
            eprintln!(
                "  {}/{}: {}",
                pr.category,
                pr.name,
                pr.error.as_deref().unwrap_or("unknown")
            );
        }
    }

    // SVF-style per-program stats
    for pr in &summary.programs {
        if pr.error.is_none() {
            print_program_stats(pr);
        }
    }
}

/// Print SVF-style statistics for a single program.
fn print_program_stats(pr: &ProgramResult) {
    let s = &pr.stats;
    let label = format!("{}/{}", pr.category, pr.name);

    eprintln!();
    eprintln!("*********General Stats***************");
    eprintln!("################ (program : {label})###############");
    eprintln!("{:<24}{}", "AddrConstraints", s.addr_constraints);
    eprintln!("{:<24}{}", "CopyConstraints", s.copy_constraints);
    eprintln!("{:<24}{}", "GepConstraints", s.gep_constraints);
    eprintln!("{:<24}{}", "LoadConstraints", s.load_constraints);
    eprintln!("{:<24}{}", "StoreConstraints", s.store_constraints);
    eprintln!(
        "{:<24}{}",
        "TotalConstraints",
        s.addr_constraints
            + s.copy_constraints
            + s.load_constraints
            + s.store_constraints
            + s.gep_constraints
    );
    eprintln!(
        "{:<24}{}",
        "PostHVNConstraints", s.post_hvn_total_constraints
    );
    eprintln!("{:<24}{}", "PtaPointers", s.pta_pointers);
    eprintln!("{:<24}{}", "TotalPointerValues", s.total_pointer_values);
    eprintln!("{:<24}{}", "ObjCount", s.obj_count);
    eprintln!("{:<24}{}", "FieldLocations", s.field_location_count);
    eprintln!("----------------Time and memory stats--------------------");
    for (key, phase_key) in [
        ("FrontendTime", "frontend"),
        ("CGRefineTime", "cg_refine"),
        ("AndersenTime", "andersen"),
        ("MSSASVFGTime", "mssa_svfg"),
        ("FSPTATime", "fs_pta"),
    ] {
        if let Some(v) = pr.phases.get(phase_key) {
            eprintln!("{key:<24}{v:.3}");
        }
    }
    eprintln!("#######################################################");

    eprintln!();
    eprintln!("*********Andersen Pointer Analysis Stats***************");
    eprintln!("################ (program : {label})###############");
    eprintln!("{:<24}{:.6}", "AvgPtsSetSize", s.avg_pts_size);
    eprintln!("{:<24}{:.6}", "AvgPtsSetSizeSVF", s.avg_pts_size_svf);
    eprintln!("{:<24}{}", "MaxPtsSetSize", s.max_pts_size);
    eprintln!("{:<24}{}", "SolveIterations", s.solve_iterations);
    eprintln!("{:<24}{}", "IndCallSites", s.ind_call_sites);
    eprintln!("{:<24}{}", "IndCallsResolved", s.indirect_calls_resolved);
    eprintln!("#######################################################");

    eprintln!();
    eprintln!("*********PTACallGraph Stats***************");
    eprintln!("################ (program : {label})###############");
    eprintln!("{:<24}{}", "TotalNode", s.cg_nodes);
    eprintln!("{:<24}{}", "UniquePairEdges", s.cg_edges);
    eprintln!("{:<24}{}", "TotalEdge", s.cg_callsite_edges);
    eprintln!("#######################################################");
}
