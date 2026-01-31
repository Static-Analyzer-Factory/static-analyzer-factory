//! SAF Benchmark Runner CLI
#![allow(clippy::doc_markdown)]
//!
//! Run benchmark suites against SAF analysis.
//!
//! # Usage
//!
//! ```bash
//! # PTABen benchmarks
//! saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled
//! saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --json
//! saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "basic_c_tests/*"
//!
//! # SV-COMP benchmarks
//! saf-bench svcomp --compiled-dir tests/benchmarks/sv-benchmarks/.compiled
//! saf-bench svcomp --compiled-dir tests/benchmarks/sv-benchmarks/.compiled --category array-examples
//! ```

mod incremental;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use saf_bench::cruxbc::{CruxBcConfig, CruxBcRunner};
use saf_bench::juliet::{JulietConfig, JulietRunner};
use saf_bench::svcomp::{
    PropertyAnalysisConfig, SvCompConfig, SvCompRunner, SvCompSummary, format_duration,
};
use saf_bench::{
    AliasKind, BenchmarkSuite, DoubleFreeKind, Expectation, MemLeakKind, SuiteSummary, TestResult,
    ptaben::PTABen, report,
};
use saf_core::config::Config;
use saf_core::config::PtaSolver;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "saf-bench")]
#[command(about = "SAF Benchmark Suite Runner")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run PTABen (SVF Test-Suite) benchmarks
    Ptaben {
        /// Directory containing compiled .ll/.bc files
        #[arg(long)]
        compiled_dir: PathBuf,

        /// Output results as JSON (to stdout, or to file with --output)
        #[arg(long)]
        json: bool,

        /// Write JSON results directly to a file (implies --json, avoids Docker stdout issues)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Filter tests by pattern (e.g., "basic_c_tests/*")
        #[arg(long)]
        filter: Option<String>,

        /// Number of parallel jobs (default: number of CPUs)
        #[arg(long, short = 'j')]
        jobs: Option<usize>,

        /// PTA solver to use: "worklist" (default) or "datalog"
        #[arg(long, default_value = "worklist")]
        solver: String,
    },

    /// Run SV-COMP benchmarks
    Svcomp {
        /// Directory containing compiled .bc files
        #[arg(long)]
        compiled_dir: PathBuf,

        /// Output results as JSON (to stdout, or to file with --output)
        #[arg(long)]
        json: bool,

        /// Write JSON results directly to a file (implies --json, avoids Docker stdout issues)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Filter by category name (substring match)
        #[arg(long)]
        category: Option<String>,

        /// Filter by property (unreach-call, valid-memsafety, etc.)
        #[arg(long)]
        property: Option<String>,

        /// Number of parallel jobs (default: number of CPUs)
        #[arg(long, short = 'j')]
        jobs: Option<usize>,

        /// Timeout per task in seconds
        #[arg(long, default_value = "900")]
        timeout: u64,

        /// Z3 timeout in milliseconds per query
        #[arg(long, default_value = "5000")]
        z3_timeout: u64,

        /// Enable aggressive analysis mode (higher coverage, small risk of incorrect verdicts)
        #[arg(long)]
        aggressive: bool,
    },

    /// Run Juliet C/C++ benchmark suite (precision/recall/F1 per CWE)
    Juliet {
        /// Directory containing compiled .ll files
        #[arg(
            long,
            default_value = "tests/benchmarks/sv-benchmarks/.compiled-juliet"
        )]
        compiled_dir: PathBuf,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Write JSON results to a file (implies --json)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Filter to single CWE (e.g., "CWE476")
        #[arg(long)]
        cwe: Option<String>,

        /// Number of parallel jobs (default: number of CPUs)
        #[arg(long, short = 'j')]
        jobs: Option<usize>,

        /// Timeout per task in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Z3 timeout in milliseconds per query
        #[arg(long, default_value = "5000")]
        z3_timeout: u64,
    },

    /// Run oracle verification suite (hand-crafted programs with expected results)
    Oracle {
        /// Directory containing oracle test programs
        #[arg(long, default_value = "tests/verification/oracle")]
        oracle_dir: PathBuf,

        /// Filter by layer (pta, callgraph, cfg, mssa, svfg)
        #[arg(long)]
        layer: Option<String>,
    },

    /// Run CruxBC scalability benchmarks
    Cruxbc {
        /// Directory containing compiled .ll/.bc files
        #[arg(long)]
        compiled_dir: PathBuf,

        /// Output results as JSON to stdout
        #[arg(long)]
        json: bool,

        /// Write JSON results to a file (implies --json)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,

        /// Filter programs by pattern (e.g., "small", "dc")
        #[arg(long)]
        filter: Option<String>,

        /// PTA solver to use: "worklist" (default) or "datalog"
        #[arg(long, default_value = "worklist")]
        solver: String,
    },

    /// Run a single CruxBC program (internal — used for process isolation)
    #[command(hide = true)]
    CruxbcSingle {
        /// Path to the .ll/.bc file
        #[arg(long)]
        path: PathBuf,

        /// Category (small/big/extra)
        #[arg(long)]
        category: String,

        /// PTA solver: "worklist" or "datalog"
        #[arg(long, default_value = "worklist")]
        solver: String,
    },

    /// Run incremental analysis benchmark on a real C project
    Incremental {
        /// Directory containing compiled .ll files
        #[arg(long)]
        compiled_dir: PathBuf,
        /// Directory containing patch files (.ll v2 variants)
        #[arg(long)]
        patches_dir: PathBuf,
        /// Output JSON results to file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Run fresh full analysis for result equivalence verification
        #[arg(long)]
        verify: bool,
    },
}

// NOTE: main dispatches CLI subcommands to their respective runners.
// Splitting would not improve readability.
#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    // Initialize tracing (write to stderr so --json stdout is clean)
    use tracing_subscriber::prelude::*;
    let saf_layer = saf_core::logging::subscriber::init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(std::io::stderr)
                .with_filter(
                    EnvFilter::from_default_env().add_directive("saf_bench=info".parse()?),
                ),
        )
        .with(saf_layer)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ptaben {
            compiled_dir,
            json,
            output,
            filter,
            jobs,
            solver,
        } => {
            let pta_solver = match solver.as_str() {
                "worklist" | "legacy" => PtaSolver::Worklist,
                _ => PtaSolver::Datalog,
            };
            run_suite(
                PTABen::new()
                    .with_solver(pta_solver)
                    .with_compiled_dir(&compiled_dir),
                &compiled_dir,
                json || output.is_some(),
                output,
                filter,
                jobs,
            )?;
        }
        Commands::Svcomp {
            compiled_dir,
            json,
            output,
            category,
            property,
            jobs,
            timeout,
            z3_timeout,
            aggressive,
        } => {
            run_svcomp(
                compiled_dir,
                json || output.is_some(),
                output,
                category,
                property,
                jobs,
                timeout,
                z3_timeout,
                aggressive,
            )?;
        }
        Commands::Juliet {
            compiled_dir,
            json,
            output,
            cwe,
            jobs,
            timeout,
            z3_timeout,
        } => {
            run_juliet(
                compiled_dir,
                json || output.is_some(),
                output,
                cwe,
                jobs,
                timeout,
                z3_timeout,
            )?;
        }
        Commands::Oracle { oracle_dir, layer } => {
            let tests = saf_bench::oracle::discover_tests(&oracle_dir, layer.as_deref())
                .context("Failed to discover oracle tests")?;

            eprintln!("Found {} oracle test cases", tests.len());

            let mut summary = saf_bench::oracle::OracleSummary::default();

            for test in &tests {
                let verdict = test.verify();
                match &verdict {
                    saf_bench::oracle::OracleVerdict::Pass => summary.pass += 1,
                    saf_bench::oracle::OracleVerdict::Warn(_) => summary.warn += 1,
                    saf_bench::oracle::OracleVerdict::Fail(_) => summary.fail += 1,
                    saf_bench::oracle::OracleVerdict::Error(_) => summary.error += 1,
                }
                summary.results.push((test.name.clone(), verdict));
            }

            saf_bench::oracle::print_report(&summary);

            // Exit with failure if any unsoundness found
            if summary.fail > 0 {
                std::process::exit(1);
            }
        }
        Commands::Cruxbc {
            compiled_dir,
            json,
            output,
            filter,
            solver,
        } => {
            let pta_solver = match solver.as_str() {
                "worklist" | "legacy" => PtaSolver::Worklist,
                _ => PtaSolver::Datalog,
            };
            run_cruxbc(
                compiled_dir,
                json || output.is_some(),
                output,
                filter,
                pta_solver,
            )?;
        }
        Commands::CruxbcSingle {
            path,
            category,
            solver,
        } => {
            let pta_solver = match solver.as_str() {
                "worklist" | "legacy" => PtaSolver::Worklist,
                _ => PtaSolver::Datalog,
            };
            let name = path.file_stem().map_or_else(
                || "unknown".to_string(),
                |s| s.to_string_lossy().to_string(),
            );
            let prog = saf_bench::cruxbc::CruxBcProgram {
                path,
                category,
                name,
            };
            let result = saf_bench::cruxbc::run_single_program(&prog, pta_solver);
            println!("{}", serde_json::to_string(&result)?);
        }
        Commands::Incremental {
            compiled_dir,
            patches_dir,
            output,
            verify,
        } => {
            incremental::run(&compiled_dir, &patches_dir, output.as_deref(), verify)?;
        }
    }

    Ok(())
}

/// Run a benchmark suite and output results.
#[allow(clippy::needless_pass_by_value)]
fn run_suite<S: BenchmarkSuite + Sync>(
    suite: S,
    compiled_dir: &PathBuf,
    json_output: bool,
    output_path: Option<PathBuf>,
    filter: Option<String>,
    jobs: Option<usize>,
) -> Result<()> {
    let start = Instant::now();

    // Configure thread pool
    if let Some(j) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j)
            .build_global()
            .ok();
    }

    info!("Discovering tests in {:?}...", compiled_dir);
    let mut tests = suite
        .discover_tests(compiled_dir)
        .context("Failed to discover tests")?;

    // Apply filter if specified
    if let Some(ref pattern) = filter {
        tests.retain(|t| matches_filter(&t.path, pattern));
        info!("Filtered to {} tests matching '{}'", tests.len(), pattern);
    }

    info!("Running {} tests...", tests.len());

    // Run tests in parallel
    let results: Vec<TestResult> = tests
        .par_iter()
        .map(|test| {
            let bc_path = compiled_dir.join(&test.path);

            // Load bitcode
            let bundle = match load_bitcode(&bc_path) {
                Ok(b) => b,
                Err(e) => {
                    return TestResult {
                        test: test.clone(),
                        outcomes: vec![],
                        duration: Duration::ZERO,
                        error: Some(format!("Failed to load bitcode: {e}")),
                        bench_result: None,
                    };
                }
            };

            // Validate
            match suite.validate(test, &bundle) {
                Ok(result) => result,
                Err(e) => TestResult {
                    test: test.clone(),
                    outcomes: vec![],
                    duration: Duration::ZERO,
                    error: Some(format!("Validation error: {e}")),
                    bench_result: None,
                },
            }
        })
        .collect();

    let elapsed = start.elapsed();

    // Sum per-test durations by category
    let mut category_cpu_secs: BTreeMap<String, f64> = BTreeMap::new();
    for result in &results {
        *category_cpu_secs
            .entry(result.test.category.clone())
            .or_default() += result.duration.as_secs_f64();
    }

    // Build summary
    let summary = build_summary(suite.name(), &results, elapsed, &category_cpu_secs);

    // Output results
    if let Some(ref path) = output_path {
        let json = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, &json)
            .with_context(|| format!("Failed to write JSON to {}", path.display()))?;
        info!("Results written to {}", path.display());
        // Also print human summary to stderr for visibility
        report::print_human_to(&summary, &mut std::io::stderr().lock());
    } else if json_output {
        report::print_json(&summary)?;
    } else {
        report::print_human(&summary);
    }

    // Exit with non-zero if there were unsound results (definite failures)
    if summary.unsound > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Load a bitcode file into an `AirBundle`.
fn load_bitcode(path: &Path) -> Result<saf_core::air::AirBundle> {
    let frontend = LlvmFrontend::default();
    let config = Config::default();
    frontend
        .ingest(&[path], &config)
        .with_context(|| format!("Failed to load {}", path.display()))
}

/// Check if a test path matches a filter pattern.
fn matches_filter(path: &str, pattern: &str) -> bool {
    // Simple glob matching: only support * as wildcard
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let (prefix, suffix) = (parts[0], parts[1]);
            return path.starts_with(prefix) && path.ends_with(suffix);
        }
    }
    path.contains(pattern)
}

/// Category counts for dual ground truth outcomes
#[derive(Default)]
struct CategoryCounts {
    exact: usize,
    sound: usize,
    to_verify: usize,
    unsound: usize,
    skip: usize,
    skip_reason: Option<String>,
}

/// Build a summary from test results using dual ground truth outcomes.
// NOTE: This function aggregates test outcomes into category summaries and
// detail lists. Splitting would fragment the single-pass accumulation logic.
#[allow(clippy::too_many_lines)]
fn build_summary(
    suite_name: &str,
    results: &[TestResult],
    elapsed: Duration,
    category_cpu_secs: &BTreeMap<String, f64>,
) -> SuiteSummary {
    let mut by_category: BTreeMap<String, CategoryCounts> = BTreeMap::new();
    let mut unsound_details = Vec::new();
    let mut to_verify_details = Vec::new();
    let mut sound_details = Vec::new();
    let mut expectedfail_total: usize = 0;
    let mut expectedfail_exact: usize = 0;
    let mut expectedfail_details = Vec::new();

    for result in results {
        let cat = &result.test.category;
        let entry = by_category.entry(cat.clone()).or_default();

        // Handle errors as unsound
        if let Some(ref err) = result.error {
            entry.unsound += 1;
            unsound_details.push(saf_bench::FailureDetail {
                file: result.test.path.clone(),
                expectation: "load/analysis".to_string(),
                expected: "success".to_string(),
                actual: err.clone(),
            });
            continue;
        }

        // Count outcomes
        for (exp, outcome) in &result.outcomes {
            match outcome {
                saf_bench::Outcome::Exact => {
                    entry.exact += 1;
                }
                saf_bench::Outcome::Sound => {
                    entry.sound += 1;
                    // Extract alias debug info from bench_result if available
                    let (
                        ctx_insensitive_str,
                        ctx_sensitive_str,
                        flow_sensitive_str,
                        ci_pts_a,
                        ci_pts_b,
                        ci_unique,
                        fs_pts_a,
                        fs_pts_b,
                        ps_str,
                        ps_perpath_str,
                        ps_callsite_str,
                        ps_guard_str,
                        ps_dead,
                    ) = if let (Some(br), Expectation::Alias { ptr_a, ptr_b, .. }) =
                        (&result.bench_result, exp)
                    {
                        let pa = format!("0x{:032x}", ptr_a.raw());
                        let pb = format!("0x{:032x}", ptr_b.raw());
                        if let Some(ae) = br
                            .alias_results
                            .iter()
                            .find(|r| r.ptr_a == pa && r.ptr_b == pb)
                        {
                            (
                                ae.ci.clone(),
                                ae.cs.clone(),
                                ae.fs.clone(),
                                ae.ci_pts_a_size,
                                ae.ci_pts_b_size,
                                ae.ci_unique,
                                ae.fs_pts_a_size,
                                ae.fs_pts_b_size,
                                ae.ps.clone(),
                                ae.ps_perpath.clone(),
                                ae.ps_callsite.clone(),
                                ae.ps_guard.clone(),
                                ae.ps_dead_code,
                            )
                        } else {
                            (
                                String::new(),
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                false,
                            )
                        }
                    } else {
                        (
                            String::new(),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            false,
                        )
                    };
                    sound_details.push(saf_bench::SoundDetail {
                        file: result.test.path.clone(),
                        expectation: format!("{exp:?}"),
                        ci: ctx_insensitive_str,
                        cs: ctx_sensitive_str,
                        fs: flow_sensitive_str,
                        ci_pts_a,
                        ci_pts_b,
                        ci_unique,
                        fs_pts_a,
                        fs_pts_b,
                        ps: ps_str,
                        ps_perpath: ps_perpath_str,
                        ps_callsite: ps_callsite_str,
                        ps_guard: ps_guard_str,
                        ps_dead_code: ps_dead,
                    });
                }
                saf_bench::Outcome::ToVerify {
                    floor,
                    actual,
                    note,
                } => {
                    entry.to_verify += 1;
                    to_verify_details.push(saf_bench::ToVerifyDetail {
                        file: result.test.path.clone(),
                        expectation: format!("{exp:?}"),
                        floor: floor.clone(),
                        actual: actual.clone(),
                        note: note.clone(),
                    });
                }
                saf_bench::Outcome::Unsound { expected, actual } => {
                    entry.unsound += 1;
                    unsound_details.push(saf_bench::FailureDetail {
                        file: result.test.path.clone(),
                        expectation: format!("{exp:?}"),
                        expected: expected.clone(),
                        actual: actual.clone(),
                    });
                }
                saf_bench::Outcome::Skip { reason } => {
                    entry.skip += 1;
                    if entry.skip_reason.is_none() {
                        entry.skip_reason = Some(reason.clone());
                    }
                }
            }

            // Track expected-imprecision oracles for SAF-vs-SVF reporting.
            // These are cases where SVF has known false positives or false negatives.
            let is_expectedfail = match exp {
                Expectation::Alias { kind, .. } => matches!(
                    kind,
                    AliasKind::ExpectedFailMayAlias | AliasKind::ExpectedFailNoAlias
                ),
                Expectation::MemLeak { kind, .. } => matches!(
                    kind,
                    MemLeakKind::NeverFreeFP
                        | MemLeakKind::PartialLeakFP
                        | MemLeakKind::FalseNegative
                ),
                Expectation::DoubleFree { kind, .. } => matches!(
                    kind,
                    DoubleFreeKind::FalsePositive | DoubleFreeKind::FalseNegative
                ),
                _ => false,
            };
            if is_expectedfail {
                expectedfail_total += 1;
                if outcome.is_exact() {
                    expectedfail_exact += 1;
                    expectedfail_details.push(saf_bench::ExpectedFailDetail {
                        file: result.test.path.clone(),
                        expectation: format!("{exp:?}"),
                        result: describe_expectedfail_win(exp),
                    });
                }
            }
        }
    }

    // Convert to category summaries
    let category_summaries: Vec<saf_bench::CategorySummary> = by_category
        .into_iter()
        .map(|(cat, counts)| {
            // Summed CPU time across all tests in this category
            let timing_secs = category_cpu_secs.get(&cat).copied().unwrap_or(0.0);
            saf_bench::CategorySummary {
                category: cat,
                exact: counts.exact,
                sound: counts.sound,
                to_verify: counts.to_verify,
                unsound: counts.unsound,
                skip: counts.skip,
                skip_reason: counts.skip_reason,
                timing_secs,
                // Legacy fields
                pass: counts.exact + counts.sound,
                fail: counts.unsound,
            }
        })
        .collect();

    // Build skipped details
    let skipped: Vec<saf_bench::SkipDetail> = category_summaries
        .iter()
        .filter(|c| c.skip > 0 && c.skip_reason.is_some())
        .map(|c| saf_bench::SkipDetail {
            category: c.category.clone(),
            reason: c.skip_reason.clone().unwrap_or_default(),
            count: c.skip,
        })
        .collect();

    let exact: usize = category_summaries.iter().map(|c| c.exact).sum();
    let sound: usize = category_summaries.iter().map(|c| c.sound).sum();
    let to_verify: usize = category_summaries.iter().map(|c| c.to_verify).sum();
    let unsound: usize = category_summaries.iter().map(|c| c.unsound).sum();
    let skip: usize = category_summaries.iter().map(|c| c.skip).sum();

    SuiteSummary {
        suite: suite_name.to_string(),
        total: results.len(),
        exact,
        sound,
        to_verify,
        unsound,
        skip,
        by_category: category_summaries,
        unsound_details,
        to_verify_details,
        sound_details,
        skipped,
        timing_secs: elapsed.as_secs_f64(),
        expectedfail_total,
        expectedfail_exact,
        expectedfail_details,
        // Legacy fields
        pass: exact + sound,
        fail: unsound,
        failures: Vec::new(), // Deprecated, use unsound_details
        precision_improvements: to_verify,
    }
}

/// Describe what SAF got right for an expected-imprecision oracle.
fn describe_expectedfail_win(exp: &Expectation) -> String {
    match exp {
        Expectation::Alias { kind, .. } => match kind {
            AliasKind::ExpectedFailMayAlias => "Proved NoAlias (SVF reports MayAlias)".to_string(),
            AliasKind::ExpectedFailNoAlias => "Found aliasing (SVF reports NoAlias)".to_string(),
            _ => String::new(),
        },
        Expectation::MemLeak { kind, .. } => match kind {
            MemLeakKind::NeverFreeFP | MemLeakKind::PartialLeakFP => {
                "No false positive (SVF incorrectly reports leak)".to_string()
            }
            MemLeakKind::FalseNegative => "Found leak (SVF misses it)".to_string(),
            _ => String::new(),
        },
        Expectation::DoubleFree { kind, .. } => match kind {
            DoubleFreeKind::FalsePositive => {
                "No false positive (SVF incorrectly reports double-free)".to_string()
            }
            DoubleFreeKind::FalseNegative => "Found double-free (SVF misses it)".to_string(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

/// Run CruxBC scalability benchmarks.
#[allow(clippy::needless_pass_by_value)]
fn run_cruxbc(
    compiled_dir: PathBuf,
    json_output: bool,
    output_path: Option<PathBuf>,
    filter: Option<String>,
    solver: PtaSolver,
) -> Result<()> {
    let config = CruxBcConfig {
        compiled_dir,
        filter,
        solver,
    };

    info!("Running cruxbc scalability benchmarks...");
    let runner = CruxBcRunner::new(config);
    let summary = runner.run()?;

    if let Some(ref path) = output_path {
        let json_str = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, &json_str)
            .with_context(|| format!("Failed to write JSON to {}", path.display()))?;
        info!("Results written to {}", path.display());
        saf_bench::cruxbc::print_human(&summary);
    } else if json_output {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        saf_bench::cruxbc::print_human(&summary);
    }

    Ok(())
}

/// Run Juliet C/C++ benchmarks with precision/recall/F1 scoring per CWE.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn run_juliet(
    compiled_dir: PathBuf,
    json_output: bool,
    output_path: Option<PathBuf>,
    cwe: Option<String>,
    jobs: Option<usize>,
    timeout: u64,
    z3_timeout: u64,
) -> Result<()> {
    // Configure thread pool
    if let Some(j) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j)
            .build_global()
            .ok();
    }

    let config = JulietConfig {
        compiled_dir,
        cwe_filter: cwe,
        jobs: jobs.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4)
        }),
        timeout_secs: timeout,
        z3_timeout_ms: z3_timeout,
    };

    info!("Running Juliet benchmarks...");
    let runner = JulietRunner::new(config);
    let summary = runner.run()?;

    if let Some(ref path) = output_path {
        let json = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, &json)
            .with_context(|| format!("Failed to write JSON to {}", path.display()))?;
        info!("Results written to {}", path.display());
        saf_bench::juliet::print_human(&summary);
    } else if json_output {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        saf_bench::juliet::print_human(&summary);
    }

    Ok(())
}

/// Run SV-COMP benchmarks.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn run_svcomp(
    compiled_dir: PathBuf,
    json_output: bool,
    output_path: Option<PathBuf>,
    category: Option<String>,
    property: Option<String>,
    jobs: Option<usize>,
    timeout: u64,
    z3_timeout: u64,
    aggressive: bool,
) -> Result<()> {
    // Configure thread pool
    if let Some(j) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j)
            .build_global()
            .ok();
    }

    // Parse property filter
    let property_filter = property.as_ref().map(|p| {
        use saf_bench::svcomp::Property;
        match p.as_str() {
            "unreach-call" => Property::UnreachCall,
            "valid-memsafety" => Property::ValidMemsafety,
            "valid-memcleanup" => Property::ValidMemcleanup,
            "no-overflow" => Property::NoOverflow,
            "no-data-race" => Property::NoDataRace,
            "termination" => Property::Termination,
            _ => Property::Unknown,
        }
    });

    let config = SvCompConfig {
        compiled_dir,
        category_filter: category,
        property_filter,
        jobs: jobs.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4)
        }),
        timeout_secs: timeout,
        analysis_config: PropertyAnalysisConfig {
            z3_timeout_ms: z3_timeout,
            conservative: !aggressive,
            ..Default::default()
        },
    };

    let mode_str = if aggressive {
        "aggressive"
    } else {
        "conservative"
    };
    info!("Running SV-COMP benchmarks in {} mode...", mode_str);
    let runner = SvCompRunner::new(config);
    let summary = runner.run()?;

    if let Some(ref path) = output_path {
        let json = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, &json)
            .with_context(|| format!("Failed to write JSON to {}", path.display()))?;
        info!("Results written to {}", path.display());
        print_svcomp_human(&summary);
    } else if json_output {
        print_svcomp_json(&summary)?;
    } else {
        print_svcomp_human(&summary);
    }

    // Exit with non-zero if there were incorrect results
    if summary.true_incorrect > 0 || summary.false_incorrect > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Print SV-COMP results as JSON.
fn print_svcomp_json(summary: &SvCompSummary) -> Result<()> {
    let json = serde_json::to_string_pretty(summary)?;
    println!("{json}");
    Ok(())
}

/// Print SV-COMP results in human-readable format.
fn print_svcomp_human(summary: &SvCompSummary) {
    println!();
    println!("=== SV-COMP Benchmark Results ===");
    println!(
        "Categories: {} | Tasks: {} | Time: {}",
        summary.categories.len(),
        summary.total_tasks,
        format_duration(Duration::from_secs_f64(summary.timing_secs))
    );
    println!();

    // Header
    println!(
        "{:<25} {:>6} {:>6} {:>8} {:>8} {:>6}",
        "Category", "TRUE", "FALSE", "Score", "Max", "%"
    );
    println!("{}", "-".repeat(65));

    // Per-category results
    for cat in &summary.categories {
        let pct = cat.score_percentage();
        println!(
            "{:<25} {:>6} {:>6} {:>+8} {:>8} {:>5.0}%",
            truncate_str(&cat.category, 25),
            cat.true_correct,
            cat.false_correct,
            cat.score,
            cat.max_possible_score,
            pct
        );
    }

    // Total
    println!("{}", "-".repeat(65));
    let total_pct = summary.score_percentage();
    println!(
        "{:<25} {:>6} {:>6} {:>+8} {:>8} {:>5.0}%",
        "TOTAL",
        summary.true_correct,
        summary.false_correct,
        summary.total_score,
        summary.max_possible_score,
        total_pct
    );
    println!();

    // Incorrect verdicts
    if summary.true_incorrect > 0 || summary.false_incorrect > 0 {
        println!("Incorrect Verdicts:");
        if summary.true_incorrect > 0 {
            println!("  TRUE incorrect (-32): {} tasks", summary.true_incorrect);
        }
        if summary.false_incorrect > 0 {
            println!("  FALSE incorrect (-16): {} tasks", summary.false_incorrect);
        }
        println!();
    }

    // Unknown summary
    if summary.unknown > 0 {
        println!("Unknown: {} tasks", summary.unknown);
        println!();
    }
}

/// Truncate a string to a maximum length.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
