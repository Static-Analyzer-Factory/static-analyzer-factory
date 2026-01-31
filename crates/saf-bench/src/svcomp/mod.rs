//! SV-COMP benchmark suite integration.
//!
//! This module provides infrastructure for running SAF against the
//! [SV-COMP (Software Verification Competition)](https://sv-comp.sosy-lab.org/)
//! benchmarks, following the official scoring and property definitions.
//!
//! # Usage
//!
//! ```bash
//! # Compile benchmarks first
//! make compile-svcomp
//!
//! # Run all benchmarks
//! make test-svcomp
//!
//! # Run specific category
//! make test-svcomp-category CAT=array-examples
//! ```
//!
//! # Supported Properties
//!
//! | Property | Description | SAF Analysis |
//! |----------|-------------|--------------|
//! | `unreach-call` | `reach_error()` never called | Z3 path reachability |
//! | `valid-memsafety` | No memory safety violations | SVFG checkers |
//! | `valid-memcleanup` | No memory leaks | Memory leak checker |
//! | `no-overflow` | No signed integer overflows | Abstract interpretation |
//! | `no-data-race` | No data races | MTA MHP analysis |
//! | `termination` | Program terminates | Not supported (UNKNOWN) |
//!
//! # Scoring
//!
//! Per SV-COMP 2026 rules:
//! - TRUE correct: +2 points
//! - FALSE correct: +1 point
//! - TRUE incorrect: -32 points (unsound)
//! - FALSE incorrect: -16 points (false alarm)
//! - UNKNOWN: 0 points

pub mod fast_paths;
pub mod property;
pub mod scoring;
pub mod summaries;
pub mod task;
pub mod witness;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use rayon::prelude::*;
use saf_cli::bench_types::{AnalysisFlags, BenchConfig, BenchPtaConfig, BenchResult};
use walkdir::WalkDir;

use crate::runner;

pub use property::{PropertyAnalysisConfig, PropertyResult, analyze_property};
pub use scoring::{CategorySummary, SvCompOutcome, SvCompSummary, SvCompVerdict, TaskResult};
pub use task::{DataModel, Language, Property, PropertySpec, SvCompTask};
pub use witness::{Witness, WitnessEdge, WitnessNode, WitnessType};

/// Configuration for running SV-COMP benchmarks.
#[derive(Debug, Clone)]
pub struct SvCompConfig {
    /// Directory containing compiled bitcode files.
    pub compiled_dir: PathBuf,

    /// Filter by category name (substring match).
    pub category_filter: Option<String>,

    /// Filter by property type.
    pub property_filter: Option<Property>,

    /// Maximum parallel jobs.
    pub jobs: usize,

    /// Timeout per task in seconds.
    pub timeout_secs: u64,

    /// Property analysis configuration.
    pub analysis_config: PropertyAnalysisConfig,
}

impl Default for SvCompConfig {
    fn default() -> Self {
        Self {
            compiled_dir: PathBuf::from("tests/benchmarks/sv-benchmarks/.compiled"),
            category_filter: None,
            property_filter: None,
            jobs: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4),
            timeout_secs: 900,
            analysis_config: PropertyAnalysisConfig::default(),
        }
    }
}

/// SV-COMP benchmark runner.
pub struct SvCompRunner {
    config: SvCompConfig,
}

impl SvCompRunner {
    /// Create a new runner with the given configuration.
    pub fn new(config: SvCompConfig) -> Self {
        Self { config }
    }

    /// Discover all tasks from YAML files in the sv-benchmarks directory.
    ///
    /// # Errors
    ///
    /// This function currently doesn't return errors (invalid files are skipped).
    pub fn discover_tasks(&self) -> Result<Vec<SvCompTask>> {
        let yaml_dir = self
            .config
            .compiled_dir
            .parent()
            .unwrap_or(Path::new("."))
            .join("c");

        let mut tasks = Vec::new();

        for entry in WalkDir::new(&yaml_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "yml"))
        {
            let path = entry.path();

            // Apply category filter
            if let Some(filter) = &self.config.category_filter {
                if !path.to_string_lossy().contains(filter) {
                    continue;
                }
            }

            let Ok(mut task) = SvCompTask::from_yaml_file(path) else {
                // Skip invalid YAML files
                continue;
            };

            // Check if compiled bitcode exists
            let rel_path = path.strip_prefix(&yaml_dir).unwrap_or(path);
            let bc_path = self.config.compiled_dir.join(rel_path.with_extension("bc"));

            if bc_path.exists() {
                task.bitcode_path = bc_path;

                // Apply property filter
                if let Some(prop_filter) = &self.config.property_filter {
                    let has_property = task
                        .verification_properties()
                        .any(|p| &p.property == prop_filter);
                    if has_property {
                        tasks.push(task);
                    }
                } else {
                    tasks.push(task);
                }
            }
        }

        Ok(tasks)
    }

    /// Run all discovered tasks and return the summary.
    ///
    /// # Errors
    ///
    /// Returns an error if task discovery fails.
    pub fn run(&self) -> Result<SvCompSummary> {
        let tasks = self.discover_tasks()?;

        if tasks.is_empty() {
            return Ok(SvCompSummary::default());
        }

        let start_time = Instant::now();

        // Group tasks by category
        let mut by_category: BTreeMap<String, Vec<&SvCompTask>> = BTreeMap::new();
        for task in &tasks {
            by_category
                .entry(task.category.clone())
                .or_default()
                .push(task);
        }

        // Process each category
        let category_summaries: Vec<CategorySummary> = by_category
            .into_par_iter()
            .map(|(category, cat_tasks)| {
                let mut summary = CategorySummary::new(
                    category.clone(),
                    cat_tasks
                        .first()
                        .and_then(|t| t.primary_property())
                        .map(|p| p.property.name().to_string())
                        .unwrap_or_default(),
                );

                for task in cat_tasks {
                    if let Some(result) = self.run_task(task) {
                        summary.add_result(result);
                    }
                }

                summary
            })
            .collect();

        let total_time = start_time.elapsed();

        Ok(SvCompSummary::from_categories(
            category_summaries,
            total_time,
        ))
    }

    /// Run a single task via subprocess (`saf run --bench-config`).
    #[allow(clippy::unused_self)]
    fn run_task(&self, task: &SvCompTask) -> Option<TaskResult> {
        let start = Instant::now();

        // Get primary verification property
        let prop_spec = task.primary_property()?;

        // Skip unsupported properties
        if !prop_spec.property.is_supported() {
            return Some(TaskResult::new(
                task.clone(),
                prop_spec.property,
                SvCompVerdict::Unknown {
                    reason: format!("Property {} not supported", prop_spec.property.name()),
                },
                prop_spec.expected_verdict,
                start.elapsed(),
            ));
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
            Ok(result) => bench_result_to_verdict(&result, &prop_spec.property),
            Err(e) => SvCompVerdict::Unknown {
                reason: format!("Subprocess failed: {e}"),
            },
        };

        Some(TaskResult::new(
            task.clone(),
            prop_spec.property,
            verdict,
            prop_spec.expected_verdict,
            start.elapsed(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Subprocess helpers
// ---------------------------------------------------------------------------

/// Map an SV-COMP property to the `BenchConfig` analysis flags needed to
/// evaluate it via `saf run --bench-config`.
pub fn property_to_bench_config(property: &Property) -> BenchConfig {
    let analyses = match property {
        Property::UnreachCall | Property::NoOverflow => AnalysisFlags {
            z3_prove: true,
            ..Default::default()
        },
        Property::ValidMemsafety => AnalysisFlags {
            checkers: true,
            buffer_overflow: true,
            nullness: true,
            ..Default::default()
        },
        Property::ValidMemcleanup => AnalysisFlags {
            checkers: true,
            ..Default::default()
        },
        Property::NoDataRace => AnalysisFlags {
            mta: true,
            ..Default::default()
        },
        // Unsupported properties get no analyses (will produce UNKNOWN)
        Property::Termination | Property::Coverage | Property::Unknown => AnalysisFlags::default(),
    };

    BenchConfig {
        analyses,
        pta_config: BenchPtaConfig::default(),
        ..default_bench_config()
    }
}

/// Create a default `BenchConfig` with no queries.
fn default_bench_config() -> BenchConfig {
    BenchConfig {
        alias_queries: Vec::new(),
        nullness_queries: Vec::new(),
        interval_queries: Vec::new(),
        interleaving_queries: Vec::new(),
        tct_queries: Vec::new(),
        analyses: AnalysisFlags::default(),
        pta_config: BenchPtaConfig::default(),
    }
}

/// Interpret a `BenchResult` as an `SvCompVerdict` for a given property.
///
/// The verdict mapping depends on the property type:
/// - **`unreach-call`**: Unproved assertions indicate reachable error paths.
/// - **`valid-memsafety`**: Checker or buffer findings indicate violations.
/// - **`valid-memcleanup`**: Memory leak checker findings indicate violations.
/// - **`no-overflow`**: Unproved assertions indicate potential overflows.
/// - **`no-data-race`**: MTA results with multiple threads indicate potential races.
pub fn bench_result_to_verdict(result: &BenchResult, property: &Property) -> SvCompVerdict {
    // Subprocess error -> UNKNOWN
    if !result.success {
        return SvCompVerdict::Unknown {
            reason: result
                .error
                .clone()
                .unwrap_or_else(|| "subprocess reported failure".to_string()),
        };
    }

    match property {
        Property::UnreachCall => {
            // If any assertion was NOT proved, reach_error may be reachable
            let any_unproved = result.assertion_results.iter().any(|a| !a.proved);
            if any_unproved {
                SvCompVerdict::False { witness: None }
            } else if result.assertion_results.is_empty() {
                // No assertions found — can't determine reachability
                SvCompVerdict::Unknown {
                    reason: "No assertion oracles found in subprocess output".to_string(),
                }
            } else {
                SvCompVerdict::True
            }
        }

        Property::ValidMemsafety => {
            // valid-memsafety covers valid-free, valid-deref, valid-memtrack.
            // Only count findings from relevant checkers; exclude checkers like
            // stack-escape and lock-not-released that are unrelated to SV-COMP
            // memsafety properties.
            let relevant_findings: Vec<_> = result
                .checker_findings
                .iter()
                .filter(|f| {
                    let c = f.check.as_str();
                    c.contains("leak")
                        || c.contains("memtrack")
                        || c.contains("null")
                        || c.contains("use-after")
                        || c.contains("double-free")
                        || c.contains("free")
                })
                .collect();
            // Filter buffer findings: exclude "Unconstrained" warnings where
            // interval analysis returned TOP (no information). These represent
            // analysis imprecision, not proven out-of-bounds accesses.
            let definite_buffer: Vec<_> = result
                .buffer_findings
                .iter()
                .filter(|f| !f.description.contains("Unconstrained"))
                .collect();
            if !relevant_findings.is_empty() || !definite_buffer.is_empty() {
                let mut witness = Vec::new();
                for f in relevant_findings.iter().take(3) {
                    witness.push(format!("{}: {} ({})", f.check, f.severity, f.alloc_site));
                }
                for f in definite_buffer.iter().take(3) {
                    witness.push(format!("{}: {}", f.kind, f.description));
                }
                SvCompVerdict::False {
                    witness: Some(witness),
                }
            } else {
                SvCompVerdict::True
            }
        }

        Property::ValidMemcleanup => {
            // Memory leak findings from checkers
            let has_leak = result
                .checker_findings
                .iter()
                .any(|f| f.check.contains("leak") || f.check.contains("memtrack"));
            if has_leak {
                SvCompVerdict::False { witness: None }
            } else {
                SvCompVerdict::True
            }
        }

        Property::NoOverflow => {
            // Unproved assertions indicate overflow
            let any_unproved = result.assertion_results.iter().any(|a| !a.proved);
            if any_unproved {
                SvCompVerdict::False { witness: None }
            } else if result.assertion_results.is_empty() {
                SvCompVerdict::Unknown {
                    reason: "No overflow oracles found in subprocess output".to_string(),
                }
            } else {
                SvCompVerdict::True
            }
        }

        Property::NoDataRace => {
            // MTA results with multiple threads may indicate races
            if let Some(mta) = &result.mta_results {
                if mta.thread_contexts.len() > 1 {
                    SvCompVerdict::Unknown {
                        reason: format!(
                            "{} threads detected; race analysis limited in bench mode",
                            mta.thread_contexts.len()
                        ),
                    }
                } else {
                    SvCompVerdict::True
                }
            } else {
                SvCompVerdict::Unknown {
                    reason: "No MTA results from subprocess".to_string(),
                }
            }
        }

        Property::Termination | Property::Coverage | Property::Unknown => SvCompVerdict::Unknown {
            reason: format!("Property {} not supported", property.name()),
        },
    }
}

/// Format duration for human-readable output.
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 3600 {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{:.1}s", d.as_secs_f64())
    }
}
