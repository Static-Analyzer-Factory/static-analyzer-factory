//! Z3-based refinement for numeric checker findings.
//!
//! Uses dominator-based guard extraction to verify whether buffer overflow
//! or integer overflow warnings are on feasible execution paths.

use std::collections::BTreeMap;

use saf_core::air::AirModule;
use saf_core::ids::BlockId;

use crate::cfg::Cfg;
use crate::z3_utils::{
    FeasibilityResult, PathFeasibilityChecker, ValueLocationIndex, Z3FilterDiagnostics,
    compute_dominators, extract_dominating_guards,
};

use super::checker::NumericFinding;

/// Result of Z3-based numeric checker refinement.
#[derive(Debug, Clone)]
pub struct NumericZ3Result {
    /// Findings confirmed by Z3 (SAT — overflow is feasible).
    pub confirmed: Vec<NumericFinding>,
    /// Findings refuted by Z3 (UNSAT — false positive from widening).
    pub refuted: Vec<NumericFinding>,
    /// Findings where Z3 timed out.
    pub uncertain: Vec<NumericFinding>,
    /// Z3 filtering diagnostics.
    pub diagnostics: Z3FilterDiagnostics,
}

/// Filter numeric findings using Z3 with dominator-based guards.
pub fn filter_numeric_z3(
    findings: &[NumericFinding],
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> NumericZ3Result {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    let mut cfgs: BTreeMap<saf_core::ids::FunctionId, Cfg> = BTreeMap::new();
    let mut dom_maps: BTreeMap<saf_core::ids::FunctionId, BTreeMap<BlockId, BlockId>> =
        BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        let doms = compute_dominators(&cfg);
        cfgs.insert(func.id, cfg);
        dom_maps.insert(func.id, doms);
    }

    let mut confirmed = Vec::new();
    let mut refuted = Vec::new();
    let mut uncertain = Vec::new();
    let mut diagnostics = Z3FilterDiagnostics {
        total_items: findings.len(),
        ..Default::default()
    };

    for finding in findings {
        let (func_id, block_id) = finding.location;

        let (Some(cfg), Some(doms)) = (cfgs.get(&func_id), dom_maps.get(&func_id)) else {
            confirmed.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        };

        let pc = extract_dominating_guards(block_id, func_id, cfg, doms, &index);
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            confirmed.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        }

        if pc.guards.len() > max_guards {
            uncertain.push(finding.clone());
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        match checker.check_feasibility(&pc, &index) {
            FeasibilityResult::Feasible => {
                confirmed.push(finding.clone());
                diagnostics.feasible_count += 1;
            }
            FeasibilityResult::Infeasible => {
                refuted.push(finding.clone());
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                uncertain.push(finding.clone());
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    NumericZ3Result {
        confirmed,
        refuted,
        uncertain,
        diagnostics,
    }
}
