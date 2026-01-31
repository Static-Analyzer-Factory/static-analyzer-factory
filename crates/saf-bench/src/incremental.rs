//! Incremental analysis benchmark harness.
//!
//! Runs full -> edit -> incremental on a real C project (e.g., CPython) and
//! reports wall-clock times, PTA iterations, and constraint diff metrics.

use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use saf_analysis::pipeline::{PipelineConfig, run_pipeline_incremental};
use saf_analysis::session::AnalysisSession;
use saf_core::config::Config;
use saf_core::program::AirProgram;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

#[derive(Serialize)]
struct BenchmarkReport {
    project: String,
    total_modules: usize,
    scenarios: Vec<ScenarioResult>,
}

#[derive(Serialize)]
struct ScenarioResult {
    name: String,
    changed_files: usize,
    full_run_secs: f64,
    incremental_run_secs: f64,
    speedup: String,
    pta_iterations_full: usize,
    pta_iterations_incr: usize,
    constraint_diff: ConstraintDiffReport,
    results_match: bool,
}

#[derive(Serialize)]
struct ConstraintDiffReport {
    added: usize,
    removed: usize,
    changed_modules: usize,
}

/// Load all `.ll` files from a directory into an `AirProgram`,
/// optionally swapping one module with a replacement file.
fn load_program(compiled_dir: &Path, swap: Option<(&str, &Path)>) -> Result<AirProgram> {
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let mut bundles = Vec::new();

    for entry in std::fs::read_dir(compiled_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "ll") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            if let Some((orig, replacement)) = swap {
                if stem == orig {
                    let bundle = frontend
                        .ingest(&[replacement], &config)
                        .map_err(|e| anyhow::anyhow!("ingest {}: {e}", replacement.display()))?;
                    bundles.push(bundle);
                    continue;
                }
            }
            let bundle = frontend
                .ingest(&[path.as_path()], &config)
                .map_err(|e| anyhow::anyhow!("ingest {}: {e}", path.display()))?;
            bundles.push(bundle);
        }
    }

    Ok(AirProgram::link(bundles))
}

/// Run a single benchmark scenario.
fn run_scenario(
    compiled_dir: &Path,
    scenario_name: &str,
    swap_original: &str,
    swap_replacement: &Path,
    config: &PipelineConfig,
    verify: bool,
) -> Result<ScenarioResult> {
    eprintln!("  Running scenario: {scenario_name}");

    // Full first run
    let tmp = tempfile::tempdir()?;
    let mut session = AnalysisSession::new(tmp.path().to_owned());
    let program_v1 = load_program(compiled_dir, None)?;
    eprintln!(
        "    Full analysis ({} modules)...",
        program_v1.modules.len()
    );
    let result_full = run_pipeline_incremental(&program_v1, config, &mut session);

    // Incremental second run
    let program_v2 = load_program(compiled_dir, Some((swap_original, swap_replacement)))?;
    eprintln!("    Incremental analysis...");
    let result_incr = run_pipeline_incremental(&program_v2, config, &mut session);

    let speedup = if result_incr.stats.total_secs > 0.0 {
        result_full.stats.total_secs / result_incr.stats.total_secs
    } else {
        0.0
    };

    // Optional result equivalence check
    let results_match = if verify {
        eprintln!("    Verifying against fresh full analysis...");
        let tmp2 = tempfile::tempdir()?;
        let mut session2 = AnalysisSession::new(tmp2.path().to_owned());
        let prog = load_program(compiled_dir, Some((swap_original, swap_replacement)))?;
        let fresh = run_pipeline_incremental(&prog, config, &mut session2);
        match (result_incr.pta_result.as_ref(), fresh.pta_result.as_ref()) {
            (Some(a), Some(b)) => a.points_to_map() == b.points_to_map(),
            (None, None) => true,
            _ => false,
        }
    } else {
        true
    };

    Ok(ScenarioResult {
        name: scenario_name.to_owned(),
        changed_files: 1,
        full_run_secs: result_full.stats.total_secs,
        incremental_run_secs: result_incr.stats.total_secs,
        speedup: format!("{speedup:.1}x"),
        pta_iterations_full: result_full.stats.pta_iterations,
        pta_iterations_incr: result_incr.stats.pta_iterations,
        constraint_diff: ConstraintDiffReport {
            added: result_incr.stats.constraint_diff_added,
            removed: result_incr.stats.constraint_diff_removed,
            changed_modules: result_incr.stats.changed_module_count,
        },
        results_match,
    })
}

/// Entry point for the incremental benchmark.
pub fn run(
    compiled_dir: &Path,
    patches_dir: &Path,
    output: Option<&Path>,
    verify: bool,
) -> Result<()> {
    eprintln!("=== Incremental Analysis Benchmark ===");
    let config = PipelineConfig::default();

    // Count modules
    let module_count = std::fs::read_dir(compiled_dir)?
        .filter(|e| {
            e.as_ref()
                .map(|e| e.path().extension().is_some_and(|ext| ext == "ll"))
                .unwrap_or(false)
        })
        .count();

    // Discover scenarios from patches directory
    // Convention: each .ll file is a v2 variant named <original>_v2.ll
    let mut scenarios = Vec::new();
    if patches_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(patches_dir)?
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                let p = e.path();
                p.extension().is_some_and(|ext| ext == "ll")
                    && p.file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.ends_with("_v2"))
            })
            .collect();
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in &entries {
            let path = entry.path();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            // Derive original module stem: "foo_v2" -> "foo"
            let original_stem = stem.trim_end_matches("_v2");
            match run_scenario(compiled_dir, stem, original_stem, &path, &config, verify) {
                Ok(result) => scenarios.push(result),
                Err(e) => eprintln!("  ERROR in {stem}: {e:#}"),
            }
        }
    }

    let report = BenchmarkReport {
        project: compiled_dir
            .parent()
            .and_then(|p| p.file_name())
            .map_or_else(
                || "unknown".to_string(),
                |n| n.to_string_lossy().to_string(),
            ),
        total_modules: module_count,
        scenarios,
    };

    // Print summary to stderr
    eprintln!("\n=== Results ===");
    for s in &report.scenarios {
        eprintln!(
            "  {}: full={:.1}s, incr={:.1}s, speedup={}, match={}",
            s.name, s.full_run_secs, s.incremental_run_secs, s.speedup, s.results_match
        );
    }

    // Write JSON if requested
    if let Some(path) = output {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)?;
        eprintln!("\nJSON written to {}", path.display());
    }

    Ok(())
}
