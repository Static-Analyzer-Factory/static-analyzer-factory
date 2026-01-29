//! Z3-based refinement for typestate analysis.
//!
//! Uses dominator-based guard extraction to verify typestate findings
//! are on feasible execution paths.

use std::collections::BTreeMap;

use saf_core::air::AirModule;
use saf_core::ids::BlockId;

use crate::cfg::Cfg;
use crate::z3_utils::{
    FeasibilityResult, PathFeasibilityChecker, ValueLocationIndex, Z3FilterDiagnostics,
    compute_dominators, extract_dominating_guards,
};

use super::typestate::TypestateFinding;

/// Result of Z3-based typestate refinement.
#[derive(Debug, Clone)]
pub struct TypestateZ3Result {
    pub feasible: Vec<TypestateFinding>,
    pub infeasible: Vec<TypestateFinding>,
    pub unknown: Vec<TypestateFinding>,
    pub diagnostics: Z3FilterDiagnostics,
}

/// Filter typestate findings using Z3 with dominator-based guards.
///
/// For each finding, collects branch guards that dominate the finding's
/// block and checks if they are satisfiable.
pub fn filter_typestate_z3(
    findings: &[TypestateFinding],
    module: &AirModule,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> TypestateZ3Result {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    // Build CFGs and dominators per function
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

    let mut feasible = Vec::new();
    let mut infeasible = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = Z3FilterDiagnostics {
        total_items: findings.len(),
        ..Default::default()
    };

    for finding in findings {
        let Some((func_id, block_id)) = finding.location else {
            // Unknown location — conservatively treat as feasible.
            feasible.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        };

        let (Some(cfg), Some(doms)) = (cfgs.get(&func_id), dom_maps.get(&func_id)) else {
            feasible.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        };

        let pc = extract_dominating_guards(block_id, func_id, cfg, doms, &index);
        diagnostics.guards_extracted += pc.guards.len();

        if pc.is_empty() {
            feasible.push(finding.clone());
            diagnostics.feasible_count += 1;
            continue;
        }

        if pc.guards.len() > max_guards {
            unknown.push(finding.clone());
            diagnostics.unknown_count += 1;
            diagnostics.skipped_too_many_guards += 1;
            continue;
        }

        diagnostics.z3_calls += 1;
        match checker.check_feasibility(&pc, &index) {
            FeasibilityResult::Feasible => {
                feasible.push(finding.clone());
                diagnostics.feasible_count += 1;
            }
            FeasibilityResult::Infeasible => {
                infeasible.push(finding.clone());
                diagnostics.infeasible_count += 1;
            }
            FeasibilityResult::Unknown => {
                unknown.push(finding.clone());
                diagnostics.unknown_count += 1;
                diagnostics.z3_timeouts += 1;
            }
        }
    }

    TypestateZ3Result {
        feasible,
        infeasible,
        unknown,
        diagnostics,
    }
}
