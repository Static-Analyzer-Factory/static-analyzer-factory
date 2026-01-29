//! Constraint-based alias refinement using Z3.
//!
//! When PTA says two pointers may-alias, this module encodes path
//! constraints to check if aliasing is feasible on any concrete path.

use saf_core::air::AirModule;
use saf_core::ids::{BlockId, FunctionId, ValueId};
use serde::{Deserialize, Serialize};

use crate::cfg::Cfg;
use crate::pta::PtaResult;

use super::dominator::{compute_dominators, extract_dominating_guards};
use super::solver::{FeasibilityResult, PathFeasibilityChecker, Z3FilterDiagnostics};
use crate::guard::ValueLocationIndex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of alias refinement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AliasRefinement {
    /// SAT — aliasing is feasible on some concrete path.
    ConfirmedAlias,
    /// UNSAT — aliasing is infeasible on any feasible path.
    NoAlias,
    /// Timeout or undecidable.
    Unknown,
}

/// Result of Z3-based alias refinement.
#[derive(Debug, Clone)]
pub struct AliasRefinementResult {
    /// The refinement verdict.
    pub result: AliasRefinement,
    /// Z3 filtering diagnostics.
    pub diagnostics: Z3FilterDiagnostics,
}

// ---------------------------------------------------------------------------
// Alias refinement
// ---------------------------------------------------------------------------

/// Refine a may-alias query using Z3.
///
/// Given two pointers `p` and `q` and a program point `at_block` in
/// function `func_id`, checks if they can actually alias by encoding
/// dominating path constraints.
///
/// Algorithm:
/// 1. Get points-to sets from PTA.
/// 2. If pts(p) ∩ pts(q) = ∅: return `NoAlias` (no Z3 needed).
/// 3. Collect dominating guards at `at_block`.
/// 4. If guards are satisfiable: `ConfirmedAlias`.
/// 5. If guards are unsatisfiable: `NoAlias`.
#[allow(clippy::too_many_arguments)] // Z3 alias refinement requires all context for precision
pub fn refine_alias(
    p: ValueId,
    q: ValueId,
    at_block: BlockId,
    func_id: FunctionId,
    module: &AirModule,
    pta: &PtaResult,
    z3_timeout_ms: u64,
    max_guards: usize,
) -> AliasRefinementResult {
    let mut diagnostics = Z3FilterDiagnostics {
        total_items: 1,
        ..Default::default()
    };

    // Step 1: Check PTA overlap
    let pts_p = pta.points_to(p);
    let pts_q = pta.points_to(q);

    let overlap: Vec<_> = pts_p.iter().filter(|loc| pts_q.contains(loc)).collect();

    if overlap.is_empty() {
        diagnostics.infeasible_count = 1;
        return AliasRefinementResult {
            result: AliasRefinement::NoAlias,
            diagnostics,
        };
    }

    // Step 2: Collect dominating guards at the use point
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    let func = module.function(func_id);
    let (cfg, doms) = match func {
        Some(f) if !f.is_declaration => {
            let cfg = Cfg::build(f);
            let doms = compute_dominators(&cfg);
            (cfg, doms)
        }
        _ => {
            diagnostics.feasible_count = 1;
            return AliasRefinementResult {
                result: AliasRefinement::ConfirmedAlias,
                diagnostics,
            };
        }
    };

    let pc = extract_dominating_guards(at_block, func_id, &cfg, &doms, &index);
    diagnostics.guards_extracted = pc.guards.len();

    if pc.is_empty() {
        diagnostics.feasible_count = 1;
        return AliasRefinementResult {
            result: AliasRefinement::ConfirmedAlias,
            diagnostics,
        };
    }

    if pc.guards.len() > max_guards {
        diagnostics.unknown_count = 1;
        diagnostics.skipped_too_many_guards = 1;
        return AliasRefinementResult {
            result: AliasRefinement::Unknown,
            diagnostics,
        };
    }

    // Step 3: Check Z3 feasibility
    diagnostics.z3_calls = 1;
    match checker.check_feasibility(&pc, &index) {
        FeasibilityResult::Feasible => {
            diagnostics.feasible_count = 1;
            AliasRefinementResult {
                result: AliasRefinement::ConfirmedAlias,
                diagnostics,
            }
        }
        FeasibilityResult::Infeasible => {
            diagnostics.infeasible_count = 1;
            AliasRefinementResult {
                result: AliasRefinement::NoAlias,
                diagnostics,
            }
        }
        FeasibilityResult::Unknown => {
            diagnostics.unknown_count = 1;
            diagnostics.z3_timeouts = 1;
            AliasRefinementResult {
                result: AliasRefinement::Unknown,
                diagnostics,
            }
        }
    }
}
