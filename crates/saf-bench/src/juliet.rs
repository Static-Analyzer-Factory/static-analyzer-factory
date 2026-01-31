//! NIST Juliet C/C++ Test Suite benchmark runner.
//!
//! This module wraps the SV-COMP analysis infrastructure with CWE-aware
//! categorization and precision/recall/F1 scoring for the
//! [NIST Juliet Test Suite](https://samate.nist.gov/SARD/test-suites/112).
//!
//! # Usage
//!
//! ```bash
//! # Compile Juliet benchmarks first
//! make compile-juliet
//!
//! # Run all CWE categories
//! saf-bench juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet
//!
//! # Run specific CWE
//! saf-bench juliet --compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet --cwe CWE476
//! ```
//!
//! # Scoring Model
//!
//! The runner computes precision, recall, and F1 per CWE category using the
//! following classification:
//!
//! | SV-COMP Outcome | Expected | Classification |
//! |-----------------|----------|----------------|
//! | `FalseCorrect`  | bad      | True Positive  |
//! | `FalseIncorrect` | good    | False Positive |
//! | `TrueIncorrect` | bad      | False Negative |
//! | `TrueCorrect`   | good     | True Negative  |
//! | `Unknown`       | bad      | False Negative |
//! | `Unknown`       | good     | True Negative  |
#![allow(clippy::doc_markdown)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use saf_cli::bench_types::BenchResult;

use crate::runner;
use crate::svcomp::scoring::compute_outcome;
use crate::svcomp::{
    Property, PropertyAnalysisConfig, PropertySpec, SvCompOutcome, SvCompTask, SvCompVerdict,
    bench_result_to_verdict, property_to_bench_config,
};

/// CWE categories supported by the Juliet runner.
///
/// Each entry maps a CWE identifier to a human-readable description.
const SUPPORTED_CWES: &[(&str, &str)] = &[
    ("CWE121", "Stack Buffer Overflow"),
    ("CWE122", "Heap Buffer Overflow"),
    ("CWE124", "Buffer Underwrite"),
    ("CWE126", "Buffer Over-read"),
    ("CWE127", "Buffer Under-read"),
    ("CWE190", "Integer Overflow"),
    ("CWE191", "Integer Underflow"),
    ("CWE401", "Memory Leak"),
    ("CWE415", "Double Free"),
    ("CWE416", "Use After Free"),
    ("CWE476", "NULL Pointer Dereference"),
    ("CWE590", "Free of Non-Heap Variable"),
    ("CWE690", "NULL Deref from Return"),
    ("CWE761", "Free Pointer Not at Start"),
    ("CWE789", "Uncontrolled Memory Allocation"),
];

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the Juliet benchmark runner.
#[derive(Debug, Clone)]
pub struct JulietConfig {
    /// Directory containing compiled `.ll` files organized by CWE subdirectory.
    ///
    /// Expected layout: `<compiled_dir>/CWE476/CWE476_xxx.ll`
    pub compiled_dir: PathBuf,

    /// Optional CWE filter (e.g., "CWE476"). Only run tests for this CWE.
    pub cwe_filter: Option<String>,

    /// Maximum parallel jobs.
    pub jobs: usize,

    /// Timeout per task in seconds.
    pub timeout_secs: u64,

    /// Z3 timeout in milliseconds for property analysis.
    pub z3_timeout_ms: u64,
}

impl Default for JulietConfig {
    fn default() -> Self {
        Self {
            compiled_dir: PathBuf::from("tests/benchmarks/sv-benchmarks/.compiled-juliet"),
            cwe_filter: None,
            jobs: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4),
            timeout_secs: 900,
            z3_timeout_ms: 5000,
        }
    }
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Juliet benchmark runner.
///
/// Discovers compiled Juliet test programs, matches them to SV-COMP YAML
/// task definitions, runs property analysis, and aggregates results by CWE.
pub struct JulietRunner {
    config: JulietConfig,
}

impl JulietRunner {
    /// Create a new runner with the given configuration.
    #[must_use]
    pub fn new(config: JulietConfig) -> Self {
        Self { config }
    }

    /// Run the Juliet benchmark suite and return a summary.
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal or thread pool creation fails.
    // NOTE: This function implements the full Juliet benchmark pipeline
    // (discover, load, analyze, aggregate) as a cohesive unit.
    #[allow(clippy::too_many_lines)]
    pub fn run(&self) -> Result<JulietSummary> {
        let start = Instant::now();

        // Derive the YAML directory from compiled_dir.
        // compiled_dir is e.g. "tests/benchmarks/sv-benchmarks/.compiled-juliet"
        // YAML files are at "tests/benchmarks/sv-benchmarks/c/Juliet_Test/*.yml"
        let yaml_dir = self
            .config
            .compiled_dir
            .parent()
            .unwrap_or(Path::new("."))
            .join("c")
            .join("Juliet_Test");

        // Discover .ll files in compiled_dir/<CWE>/ subdirectories
        let tasks = self.discover_tasks(&yaml_dir);

        if tasks.is_empty() {
            return Ok(JulietSummary {
                suite: "juliet".to_string(),
                total_tasks: 0,
                aggregate: AggregateMetrics::default(),
                by_cwe: Vec::new(),
                svcomp_scoring: SvCompScoring::default(),
                task_details: Vec::new(),
                timing_secs: start.elapsed().as_secs_f64(),
            });
        }

        eprintln!("Discovered {} Juliet tasks", tasks.len());

        // Build property analysis config
        let analysis_config = PropertyAnalysisConfig {
            z3_timeout_ms: self.config.z3_timeout_ms,
            conservative: false,
            ..PropertyAnalysisConfig::default()
        };

        // Run tasks in parallel, collecting (cwe_id, outcome, expected_verdict, task_name)
        let results: Vec<(String, SvCompOutcome, bool, String)> = tasks
            .par_iter()
            .filter_map(|(cwe_id, task)| {
                let task_name = task
                    .bitcode_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                self.run_task(task, cwe_id, &analysis_config)
                    .map(|(outcome, expected)| (cwe_id.clone(), outcome, expected, task_name))
            })
            .collect();

        // Aggregate by CWE
        let mut cwe_metrics: BTreeMap<String, CweMetrics> = BTreeMap::new();

        for (cwe_id, outcome, expected, _) in &results {
            let m = cwe_metrics.entry(cwe_id.clone()).or_default();
            m.total += 1;
            classify_outcome(*outcome, *expected, m);
        }

        // Build CWE results
        let cwe_lookup: BTreeMap<&str, &str> = SUPPORTED_CWES
            .iter()
            .map(|(id, desc)| (*id, *desc))
            .collect();

        let by_cwe: Vec<CweResult> = cwe_metrics
            .iter()
            .map(|(cwe_id, m)| {
                let description =
                    (*cwe_lookup.get(cwe_id.as_str()).unwrap_or(&"Unknown CWE")).to_string();
                #[allow(clippy::cast_precision_loss)]
                let precision = safe_div(m.tp as f64, (m.tp + m.fp) as f64);
                #[allow(clippy::cast_precision_loss)]
                let recall = safe_div(m.tp as f64, (m.tp + m.fn_count) as f64);
                let f1 = safe_div(2.0 * precision * recall, precision + recall);

                CweResult {
                    cwe: cwe_id.clone(),
                    description,
                    total: m.total,
                    tp: m.tp,
                    fp: m.fp,
                    fn_count: m.fn_count,
                    tn: m.tn,
                    precision,
                    recall,
                    f1,
                }
            })
            .collect();

        // Compute aggregate metrics
        let mut agg = CweMetrics::default();
        for m in cwe_metrics.values() {
            agg.total += m.total;
            agg.tp += m.tp;
            agg.fp += m.fp;
            agg.fn_count += m.fn_count;
            agg.tn += m.tn;
        }

        #[allow(clippy::cast_precision_loss)]
        let precision = safe_div(agg.tp as f64, (agg.tp + agg.fp) as f64);
        #[allow(clippy::cast_precision_loss)]
        let recall = safe_div(agg.tp as f64, (agg.tp + agg.fn_count) as f64);
        let f1 = safe_div(2.0 * precision * recall, precision + recall);

        let aggregate = AggregateMetrics {
            tp: agg.tp,
            fp: agg.fp,
            fn_count: agg.fn_count,
            tn: agg.tn,
            precision,
            recall,
            f1,
        };

        // Collect per-task details (FP and FN only, to keep output manageable)
        let mut task_details: Vec<TaskDetail> = Vec::new();
        for (cwe_id, outcome, expected, task_name) in &results {
            let classification = match outcome {
                SvCompOutcome::FalseIncorrect => "FP",
                SvCompOutcome::TrueIncorrect => "FN",
                SvCompOutcome::Unknown if !expected => "FN",
                _ => continue,
            };
            task_details.push(TaskDetail {
                task_name: task_name.clone(),
                cwe: cwe_id.clone(),
                classification: classification.to_string(),
            });
        }
        task_details.sort_by(|a, b| a.task_name.cmp(&b.task_name));

        // Compute SV-COMP scoring
        let mut scoring = SvCompScoring::default();
        for (_, outcome, _, _) in &results {
            match outcome {
                SvCompOutcome::TrueCorrect => {
                    scoring.true_correct += 1;
                    scoring.total_score += 2;
                    scoring.max_possible_score += 2;
                }
                SvCompOutcome::FalseCorrect => {
                    scoring.false_correct += 1;
                    scoring.total_score += 1;
                    scoring.max_possible_score += 1;
                }
                SvCompOutcome::TrueIncorrect => {
                    scoring.true_incorrect += 1;
                    scoring.total_score -= 32;
                    scoring.max_possible_score += 2;
                }
                SvCompOutcome::FalseIncorrect => {
                    scoring.false_incorrect += 1;
                    scoring.total_score -= 16;
                    scoring.max_possible_score += 1;
                }
                SvCompOutcome::Unknown => {
                    scoring.unknown += 1;
                    // Unknown contributes 0 to score, but we still track max possible
                    // based on expected verdict (unknown from YAML is skipped earlier)
                }
            }
        }

        Ok(JulietSummary {
            suite: "juliet".to_string(),
            total_tasks: results.len(),
            aggregate,
            by_cwe,
            svcomp_scoring: scoring,
            task_details,
            timing_secs: start.elapsed().as_secs_f64(),
        })
    }

    /// Discover tasks by walking compiled `.ll` files and matching to YAML definitions.
    fn discover_tasks(&self, yaml_dir: &Path) -> Vec<(String, SvCompTask)> {
        let mut tasks = Vec::new();

        for entry in WalkDir::new(&self.config.compiled_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "ll" || ext == "bc")
            })
        {
            let ll_path = entry.path();

            // Extract CWE from directory structure
            let rel_path = ll_path
                .strip_prefix(&self.config.compiled_dir)
                .unwrap_or(ll_path);
            let Some(cwe_id) = extract_cwe(&rel_path.to_string_lossy()) else {
                continue;
            };

            // Apply CWE filter
            if let Some(filter) = &self.config.cwe_filter {
                if !cwe_id.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            // Find matching YAML file
            let ll_stem = ll_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let yaml_path = yaml_dir.join(format!("{ll_stem}.yml"));

            if !yaml_path.exists() {
                // Try without the CWE subdirectory prefix
                continue;
            }

            // Parse YAML and override bitcode path
            let Ok(mut task) = SvCompTask::from_yaml_file(&yaml_path) else {
                continue;
            };

            // Override bitcode_path to point to our compiled .ll file
            task.bitcode_path = ll_path.to_path_buf();

            tasks.push((cwe_id, task));
        }

        // Sort for deterministic order
        tasks.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.bitcode_path.cmp(&b.1.bitcode_path))
        });

        tasks
    }

    /// Run a single task via subprocess (`saf run --bench-config`).
    ///
    /// Returns `(outcome, expected_verdict)` or `None` if the task has no
    /// verification property or cannot be processed.
    #[allow(clippy::unused_self)]
    fn run_task(
        &self,
        task: &SvCompTask,
        cwe_id: &str,
        _analysis_config: &PropertyAnalysisConfig,
    ) -> Option<(SvCompOutcome, bool)> {
        // Get the best-matching verification property for this CWE.
        // For CWE401 (memory leak), prefer `valid-memcleanup` over `valid-memsafety`
        // because some _bad files are memsafety-correct but have leaks.
        let prop_spec = cwe_preferred_property(task, cwe_id)?;
        let expected = prop_spec.expected_verdict?;

        // Skip unsupported properties
        if !prop_spec.property.is_supported() {
            return Some((SvCompOutcome::Unknown, expected));
        }

        // Build bench config from property type
        let bench_config = property_to_bench_config(&prop_spec.property);

        // Create temp files for config and result JSON
        let tmp_dir = tempfile::tempdir().ok()?;
        let config_path = tmp_dir.path().join("config.json");
        let result_path = tmp_dir.path().join("result.json");

        // Run analysis via subprocess
        let bench_result = runner::run_saf_bench(
            &task.bitcode_path,
            &bench_config,
            &config_path,
            &result_path,
        );

        let verdict = match bench_result {
            Ok(result) => cwe_verdict(&result, prop_spec.property, cwe_id),
            Err(_e) => SvCompVerdict::Unknown {
                reason: "Subprocess failed".to_string(),
            },
        };

        let outcome = compute_outcome(&verdict, expected);
        Some((outcome, expected))
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Summary of a Juliet benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JulietSummary {
    /// Suite identifier.
    pub suite: String,

    /// Total number of tasks analyzed.
    pub total_tasks: usize,

    /// Aggregate precision/recall/F1 across all CWEs.
    pub aggregate: AggregateMetrics,

    /// Per-CWE results.
    pub by_cwe: Vec<CweResult>,

    /// SV-COMP scoring breakdown.
    pub svcomp_scoring: SvCompScoring,

    /// Per-task details for FP and FN cases.
    pub task_details: Vec<TaskDetail>,

    /// Total wall-clock time in seconds.
    pub timing_secs: f64,
}

/// Detail for a single misclassified task (FP or FN).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    /// Test file stem.
    pub task_name: String,
    /// CWE category.
    pub cwe: String,
    /// "FP" or "FN".
    pub classification: String,
}

/// Aggregate precision, recall, and F1 metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregateMetrics {
    /// True positives (SAF found bug in bad file).
    pub tp: usize,
    /// False positives (SAF found bug in good file).
    pub fp: usize,
    /// False negatives (SAF missed bug in bad file).
    pub fn_count: usize,
    /// True negatives (SAF said safe for good file).
    pub tn: usize,
    /// Precision: TP / (TP + FP).
    pub precision: f64,
    /// Recall: TP / (TP + FN).
    pub recall: f64,
    /// F1 score: 2 * P * R / (P + R).
    pub f1: f64,
}

/// Per-CWE result with precision/recall/F1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CweResult {
    /// CWE identifier (e.g., "CWE476").
    pub cwe: String,
    /// Human-readable description.
    pub description: String,
    /// Total tests in this category.
    pub total: usize,
    /// True positives.
    pub tp: usize,
    /// False positives.
    pub fp: usize,
    /// False negatives.
    pub fn_count: usize,
    /// True negatives.
    pub tn: usize,
    /// Precision.
    pub precision: f64,
    /// Recall.
    pub recall: f64,
    /// F1 score.
    pub f1: f64,
}

/// SV-COMP scoring breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SvCompScoring {
    /// Total SV-COMP score.
    pub total_score: i32,
    /// Maximum possible score.
    pub max_possible_score: i32,
    /// TRUE correct count (+2 each).
    pub true_correct: usize,
    /// FALSE correct count (+1 each).
    pub false_correct: usize,
    /// TRUE incorrect count (-32 each, unsound).
    pub true_incorrect: usize,
    /// FALSE incorrect count (-16 each, false alarm).
    pub false_incorrect: usize,
    /// UNKNOWN count (0 each).
    pub unknown: usize,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// Accumulator for per-CWE metrics during aggregation.
#[derive(Debug, Default)]
struct CweMetrics {
    total: usize,
    tp: usize,
    fp: usize,
    fn_count: usize,
    tn: usize,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract a CWE identifier from a file path string.
///
/// Looks for patterns like "CWE476" or "CWE121" in the path.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(extract_cwe("CWE476/CWE476_test.ll"), Some("CWE476".to_string()));
/// assert_eq!(extract_cwe("foo/bar.ll"), None);
/// ```
fn extract_cwe(path: &str) -> Option<String> {
    // Look for CWE followed by digits
    let mut start = None;
    for (i, _) in path.match_indices("CWE") {
        // Check that the characters after "CWE" are digits
        let rest = &path[i + 3..];
        let digit_count = rest.chars().take_while(char::is_ascii_digit).count();
        if digit_count > 0 {
            start = Some(i);
            break;
        }
    }

    let i = start?;
    let rest = &path[i + 3..];
    let digit_count = rest.chars().take_while(char::is_ascii_digit).count();
    Some(path[i..i + 3 + digit_count].to_string())
}

/// Select the best-matching verification property for a CWE.
///
/// For CWE401 (memory leak), prefer `valid-memcleanup` over `valid-memsafety`
/// because some test files are memsafety-correct but still leak memory.
/// Falls back to `primary_property()` for all other CWEs.
fn cwe_preferred_property<'a>(task: &'a SvCompTask, cwe_id: &str) -> Option<&'a PropertySpec> {
    let preferred = match cwe_id {
        "CWE401" => Some(Property::ValidMemcleanup),
        _ => None,
    };

    if let Some(pref) = preferred {
        if let Some(spec) = task.verification_properties().find(|p| p.property == pref) {
            return Some(spec);
        }
    }

    task.primary_property()
}

/// Maps a CWE to the specific checker name(s) relevant for verdict filtering.
///
/// When a CWE maps to specific checkers, only findings from those checkers
/// count toward the verdict. This prevents cross-contamination (e.g., a
/// null-deref finding inflating FPs on a memory-leak benchmark).
fn cwe_checker_names(cwe_id: &str) -> Option<&'static [&'static str]> {
    match cwe_id {
        "CWE401" => Some(&["memory-leak"]),
        "CWE415" => Some(&["double-free"]),
        "CWE416" => Some(&["use-after-free"]),
        "CWE476" | "CWE690" => Some(&["null-deref"]),
        _ => None,
    }
}

/// Returns `true` if the CWE should be evaluated using only buffer findings.
fn cwe_uses_buffer_findings(cwe_id: &str) -> bool {
    matches!(cwe_id, "CWE121" | "CWE122" | "CWE124" | "CWE126" | "CWE127")
}

/// Produce a CWE-specific verdict by filtering findings to only the checker
/// relevant for the CWE. Falls back to `bench_result_to_verdict` for CWEs
/// without specific filtering (e.g., `CWE590`, `CWE761`, `CWE789`).
fn cwe_verdict(result: &BenchResult, property: Property, cwe_id: &str) -> SvCompVerdict {
    if !result.success {
        return bench_result_to_verdict(result, &property);
    }

    if let Some(checker_names) = cwe_checker_names(cwe_id) {
        let has_finding = result
            .checker_findings
            .iter()
            .any(|f| checker_names.iter().any(|name| f.check == *name));
        if has_finding {
            SvCompVerdict::False { witness: None }
        } else {
            SvCompVerdict::True
        }
    } else if cwe_uses_buffer_findings(cwe_id) {
        let has_buffer = result
            .buffer_findings
            .iter()
            .any(|f| !f.description.contains("Unconstrained"));
        if has_buffer {
            SvCompVerdict::False { witness: None }
        } else {
            SvCompVerdict::True
        }
    } else {
        bench_result_to_verdict(result, &property)
    }
}

/// Safe division that returns 0.0 when the denominator is zero.
fn safe_div(num: f64, den: f64) -> f64 {
    if den == 0.0 { 0.0 } else { num / den }
}

/// Classify an `SvCompOutcome` into TP/FP/FN/TN based on expected verdict.
///
/// The scoring model:
/// - `FalseCorrect` (found bug, expected bad) = True Positive
/// - `FalseIncorrect` (found bug, expected good) = False Positive
/// - `TrueIncorrect` (said safe, expected bad) = False Negative
/// - `TrueCorrect` (said safe, expected good) = True Negative
/// - `Unknown` on bad file (expected=false) = False Negative
/// - `Unknown` on good file (expected=true) = True Negative
fn classify_outcome(outcome: SvCompOutcome, expected: bool, m: &mut CweMetrics) {
    match outcome {
        SvCompOutcome::FalseCorrect => m.tp += 1,
        SvCompOutcome::FalseIncorrect => m.fp += 1,
        SvCompOutcome::TrueIncorrect => m.fn_count += 1,
        SvCompOutcome::TrueCorrect => m.tn += 1,
        SvCompOutcome::Unknown => {
            if expected {
                // Good file, unknown = conservative safe = TN
                m.tn += 1;
            } else {
                // Bad file, unknown = missed bug = FN
                m.fn_count += 1;
            }
        }
    }
}

/// Truncate a string to `max` characters, appending "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let end = max.saturating_sub(3);
        format!("{}...", &s[..end])
    }
}

// ---------------------------------------------------------------------------
// Human-readable output
// ---------------------------------------------------------------------------

/// Print a human-readable summary of the Juliet benchmark results.
pub fn print_human(summary: &JulietSummary) {
    eprintln!();
    eprintln!("=== NIST Juliet Test Suite ===");
    eprintln!(
        "Tasks: {} | Time: {:.1}s",
        summary.total_tasks, summary.timing_secs
    );
    eprintln!();

    // Per-CWE table header
    eprintln!(
        "{:<8} {:<30} {:>5} {:>4} {:>4} {:>4} {:>4} {:>7} {:>7} {:>6}",
        "CWE", "Description", "Total", "TP", "FP", "FN", "TN", "Prec", "Recall", "F1"
    );
    eprintln!("{}", "\u{2500}".repeat(90));

    for cwe in &summary.by_cwe {
        eprintln!(
            "{:<8} {:<30} {:>5} {:>4} {:>4} {:>4} {:>4} {:>6.1}% {:>6.1}% {:>5.3}",
            cwe.cwe,
            truncate(&cwe.description, 30),
            cwe.total,
            cwe.tp,
            cwe.fp,
            cwe.fn_count,
            cwe.tn,
            cwe.precision * 100.0,
            cwe.recall * 100.0,
            cwe.f1,
        );
    }

    eprintln!("{}", "\u{2500}".repeat(90));

    // Aggregate row
    let agg = &summary.aggregate;
    eprintln!(
        "{:<8} {:<30} {:>5} {:>4} {:>4} {:>4} {:>4} {:>6.1}% {:>6.1}% {:>5.3}",
        "ALL",
        "Aggregate",
        summary.total_tasks,
        agg.tp,
        agg.fp,
        agg.fn_count,
        agg.tn,
        agg.precision * 100.0,
        agg.recall * 100.0,
        agg.f1,
    );

    // SV-COMP scoring
    let s = &summary.svcomp_scoring;
    eprintln!();
    eprintln!("SV-COMP Scoring:");
    eprintln!(
        "  Score: {} / {} ({:.1}%)",
        s.total_score,
        s.max_possible_score,
        if s.max_possible_score > 0 {
            f64::from(s.total_score) / f64::from(s.max_possible_score) * 100.0
        } else {
            0.0
        }
    );
    eprintln!(
        "  TRUE correct: {} | FALSE correct: {} | TRUE incorrect: {} | FALSE incorrect: {} | UNKNOWN: {}",
        s.true_correct, s.false_correct, s.true_incorrect, s.false_incorrect, s.unknown
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cwe() {
        assert_eq!(
            extract_cwe("CWE476/CWE476_NULL_Pointer_Deref.ll"),
            Some("CWE476".to_string())
        );
        assert_eq!(
            extract_cwe("CWE121/CWE121_Stack_Based_Buffer_Overflow__test.ll"),
            Some("CWE121".to_string())
        );
        assert_eq!(
            extract_cwe("CWE190/CWE190_Integer_Overflow.ll"),
            Some("CWE190".to_string())
        );
        assert_eq!(extract_cwe("foo/bar.ll"), None);
        assert_eq!(extract_cwe("CWE/no_digits.ll"), None);
        assert_eq!(extract_cwe(""), None);
    }

    #[test]
    fn test_safe_div() {
        assert!((safe_div(1.0, 2.0) - 0.5).abs() < f64::EPSILON);
        assert!((safe_div(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((safe_div(1.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((safe_div(0.0, 1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_supported_cwes_sorted() {
        let ids: Vec<&str> = SUPPORTED_CWES.iter().map(|(id, _)| *id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "SUPPORTED_CWES should be sorted by CWE ID");
    }

    #[test]
    fn test_classify_outcome_false_correct() {
        let mut m = CweMetrics::default();
        classify_outcome(SvCompOutcome::FalseCorrect, false, &mut m);
        assert_eq!(m.tp, 1);
        assert_eq!(m.fp, 0);
        assert_eq!(m.fn_count, 0);
        assert_eq!(m.tn, 0);
    }

    #[test]
    fn test_classify_outcome_false_incorrect() {
        let mut m = CweMetrics::default();
        classify_outcome(SvCompOutcome::FalseIncorrect, true, &mut m);
        assert_eq!(m.fp, 1);
    }

    #[test]
    fn test_classify_outcome_true_incorrect() {
        let mut m = CweMetrics::default();
        classify_outcome(SvCompOutcome::TrueIncorrect, false, &mut m);
        assert_eq!(m.fn_count, 1);
    }

    #[test]
    fn test_classify_outcome_true_correct() {
        let mut m = CweMetrics::default();
        classify_outcome(SvCompOutcome::TrueCorrect, true, &mut m);
        assert_eq!(m.tn, 1);
    }

    #[test]
    fn test_classify_outcome_unknown_bad() {
        let mut m = CweMetrics::default();
        // Unknown on bad file (expected=false) = FN
        classify_outcome(SvCompOutcome::Unknown, false, &mut m);
        assert_eq!(m.fn_count, 1);
        assert_eq!(m.tn, 0);
    }

    #[test]
    fn test_classify_outcome_unknown_good() {
        let mut m = CweMetrics::default();
        // Unknown on good file (expected=true) = TN
        classify_outcome(SvCompOutcome::Unknown, true, &mut m);
        assert_eq!(m.tn, 1);
        assert_eq!(m.fn_count, 0);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("", 5), "");
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn test_aggregate_metrics_default() {
        let m = AggregateMetrics::default();
        assert_eq!(m.tp, 0);
        assert_eq!(m.fp, 0);
        assert_eq!(m.fn_count, 0);
        assert_eq!(m.tn, 0);
        assert!((m.precision - 0.0).abs() < f64::EPSILON);
        assert!((m.recall - 0.0).abs() < f64::EPSILON);
        assert!((m.f1 - 0.0).abs() < f64::EPSILON);
    }
}
