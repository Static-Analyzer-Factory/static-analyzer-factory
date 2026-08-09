//! Function summaries for interprocedural error reachability analysis.
//!
//! This module provides summary-based interprocedural analysis for the
//! `unreach-call` property. Instead of analyzing paths across function
//! boundaries directly, we compute per-function summaries that capture
//! under what conditions the function may reach an error.

use std::collections::BTreeMap;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::z3_utils::reachability::{PathReachability, check_path_reachable};
use saf_core::air::{AirFunction, AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId};

use super::property::PropertyAnalysisConfig;

/// Summary of error reachability for a function.
#[derive(Debug, Clone)]
pub enum ErrorSummary {
    /// Function never reaches error (all paths return normally).
    NeverErrors,

    /// Function always reaches error (unconditional).
    AlwaysErrors,

    /// Function may reach error under certain conditions.
    /// Contains the number of paths that were checked.
    MayError { paths_checked: usize },

    /// Could not compute summary (too complex, timeout, etc.).
    Unknown { reason: String },
}

/// Compute error summaries for all functions in the module.
///
/// Only functions containing direct error calls get non-trivial summaries.
/// Other functions get `NeverErrors` by default.
pub fn compute_error_summaries(
    module: &AirModule,
    _callgraph: &CallGraph,
    config: &PropertyAnalysisConfig,
) -> BTreeMap<FunctionId, ErrorSummary> {
    let mut summaries = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let error_blocks = find_error_blocks_in_function(func, module);

        if error_blocks.is_empty() {
            // No error calls in this function
            summaries.insert(func.id, ErrorSummary::NeverErrors);
        } else {
            // Compute summary for this function
            let summary = compute_function_summary(func, module, &error_blocks, config);
            summaries.insert(func.id, summary);
        }
    }

    summaries
}

/// Find all blocks in a function that contain error calls.
fn find_error_blocks_in_function(func: &AirFunction, module: &AirModule) -> Vec<BlockId> {
    const ERROR_NAMES: &[&str] = &["reach_error", "__VERIFIER_error"];

    let mut error_blocks = Vec::new();

    for block in &func.blocks {
        for inst in &block.instructions {
            if let Operation::CallDirect { callee, .. } = &inst.op {
                if let Some(target) = module.function(*callee) {
                    if ERROR_NAMES.contains(&target.name.as_str()) {
                        error_blocks.push(block.id);
                        break; // Only need to record block once
                    }
                }
            }
        }
    }

    error_blocks
}

/// Compute error summary for a single function.
fn compute_function_summary(
    func: &AirFunction,
    module: &AirModule,
    error_blocks: &[BlockId],
    config: &PropertyAnalysisConfig,
) -> ErrorSummary {
    let Some(entry_block) = func
        .entry_block
        .or_else(|| func.blocks.first().map(|b| b.id))
    else {
        return ErrorSummary::Unknown {
            reason: "Function has no entry block".into(),
        };
    };

    let mut all_unreachable = true;
    let mut total_paths_checked = 0;

    for error_block in error_blocks {
        let result = check_path_reachable(
            entry_block,
            *error_block,
            func.id,
            module,
            config.z3_timeout_ms,
            config.max_guards,
            config.max_paths,
        );

        total_paths_checked += result.paths_checked;

        match result.result {
            PathReachability::Reachable(_) => {
                all_unreachable = false;
            }
            PathReachability::Unreachable => {
                // This error site is unreachable within the function
            }
            PathReachability::Unknown => {
                return ErrorSummary::Unknown {
                    reason: format!(
                        "Z3 could not determine path feasibility (checked {} paths)",
                        result.paths_checked
                    ),
                };
            }
        }
    }

    if all_unreachable {
        ErrorSummary::NeverErrors
    } else {
        ErrorSummary::MayError {
            paths_checked: total_paths_checked,
        }
    }
}

/// Check if a call site's error is reachable given caller context and callee summary.
///
/// Returns TRUE (error unreachable), FALSE (error reachable), or UNKNOWN.
pub fn check_call_site_error_reachable(
    call_site_block: BlockId,
    callee_summary: &ErrorSummary,
    caller_func: &AirFunction,
    module: &AirModule,
    config: &PropertyAnalysisConfig,
) -> ErrorSummary {
    // First check: is the callee summary conclusive?
    match callee_summary {
        ErrorSummary::NeverErrors => {
            // Callee never errors, so this call site is safe
            return ErrorSummary::NeverErrors;
        }
        ErrorSummary::Unknown { reason } => {
            // Can't determine callee behavior
            return ErrorSummary::Unknown {
                reason: format!("Callee summary unknown: {reason}"),
            };
        }
        ErrorSummary::AlwaysErrors | ErrorSummary::MayError { .. } => {
            // Need to check if call site is reachable
        }
    }

    // Second check: is the call site reachable from function entry?
    let Some(entry_block) = caller_func
        .entry_block
        .or_else(|| caller_func.blocks.first().map(|b| b.id))
    else {
        return ErrorSummary::Unknown {
            reason: "Caller has no entry block".into(),
        };
    };

    let result = check_path_reachable(
        entry_block,
        call_site_block,
        caller_func.id,
        module,
        config.z3_timeout_ms,
        config.max_guards,
        config.max_paths,
    );

    match result.result {
        PathReachability::Unreachable => {
            // Call site is unreachable, so callee's error is unreachable
            ErrorSummary::NeverErrors
        }
        PathReachability::Reachable(_) => {
            // Call site is reachable, and callee may error
            match callee_summary {
                ErrorSummary::AlwaysErrors => ErrorSummary::AlwaysErrors,
                ErrorSummary::MayError { paths_checked } => ErrorSummary::MayError {
                    paths_checked: result.paths_checked + paths_checked,
                },
                _ => unreachable!(),
            }
        }
        PathReachability::Unknown => ErrorSummary::Unknown {
            reason: format!(
                "Z3 could not determine call site reachability (checked {} paths)",
                result.paths_checked
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_summary_variants() {
        let never = ErrorSummary::NeverErrors;
        assert!(matches!(never, ErrorSummary::NeverErrors));

        let always = ErrorSummary::AlwaysErrors;
        assert!(matches!(always, ErrorSummary::AlwaysErrors));

        let may = ErrorSummary::MayError { paths_checked: 5 };
        assert!(matches!(may, ErrorSummary::MayError { paths_checked: 5 }));

        let unknown = ErrorSummary::Unknown {
            reason: "test".into(),
        };
        assert!(matches!(unknown, ErrorSummary::Unknown { .. }));
    }
}
