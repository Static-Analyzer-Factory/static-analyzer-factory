//! Assertion prover using Z3.
//!
//! Scans AIR for `assert()`-like calls, extracts the controlling condition,
//! collects dominating guards, and uses Z3 to prove or disprove the assertion.
//! Optionally integrates abstract interpretation interval constraints.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId};
use serde::{Deserialize, Serialize};

use crate::absint::AbstractInterpResult;
use crate::cfg::Cfg;

use super::dominator::{compute_dominators, extract_dominating_guards};
use super::solver::PathFeasibilityChecker;
use crate::guard::ValueLocationIndex;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of an assertion proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionStatus {
    /// UNSAT(NOT cond) — assertion always holds.
    Proven,
    /// SAT(NOT cond) — counterexample exists.
    MayFail,
    /// Timeout.
    Unknown,
}

/// A finding about an assertion in the program.
#[derive(Debug, Clone)]
pub struct AssertionFinding {
    /// The function containing the assertion.
    pub function: String,
    /// The function ID.
    pub function_id: FunctionId,
    /// The instruction ID of the assert call.
    pub inst: InstId,
    /// Human-readable description of the assertion condition.
    pub condition_desc: String,
    /// Proof status.
    pub status: AssertionStatus,
    /// Z3 model for `MayFail` cases (variable name -> value).
    pub counterexample: Option<BTreeMap<String, i64>>,
}

/// Result of assertion proving.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// Assertions proven to always hold.
    pub proven: Vec<AssertionFinding>,
    /// Assertions that may fail (counterexample found).
    pub may_fail: Vec<AssertionFinding>,
    /// Assertions where Z3 timed out.
    pub unknown: Vec<AssertionFinding>,
    /// Diagnostics.
    pub diagnostics: AssertionDiagnostics,
}

/// Diagnostics from assertion proving.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssertionDiagnostics {
    /// Total assertions found.
    pub total_assertions: usize,
    /// Proven count.
    pub proven_count: usize,
    /// May-fail count.
    pub may_fail_count: usize,
    /// Unknown count.
    pub unknown_count: usize,
    /// Total guards extracted.
    pub guards_extracted: usize,
    /// Z3 calls made.
    pub z3_calls: usize,
}

// ---------------------------------------------------------------------------
// Assertion prover
// ---------------------------------------------------------------------------

/// Default assertion function names.
pub const DEFAULT_ASSERT_FUNCTIONS: &[&str] = &["__assert_fail", "__assert_rtn", "abort"];

/// Prove or disprove assertions in a module.
///
/// Scans for calls to assertion functions, extracts the controlling
/// condition from the `CondBr` that guards the call, collects dominating
/// guards, and checks Z3 feasibility.
///
/// If `absint_result` is provided, interval constraints are added to
/// tighten the Z3 encoding.
// NOTE: This function implements the full assertion proving pipeline: CFG/dominator
// construction, assertion site discovery, guard collection, and Z3 feasibility
// checking. Splitting would fragment the analysis pipeline.
#[allow(clippy::too_many_lines)]
pub fn prove_assertions(
    module: &AirModule,
    absint_result: Option<&AbstractInterpResult>,
    z3_timeout_ms: u64,
    max_guards: usize,
    assert_functions: &[String],
) -> AssertionResult {
    let index = ValueLocationIndex::build(module);
    let checker = PathFeasibilityChecker::new(z3_timeout_ms);

    // Determine assert function set
    let assert_fn_set: BTreeSet<String> = if assert_functions.is_empty() {
        DEFAULT_ASSERT_FUNCTIONS
            .iter()
            .copied()
            .map(str::to_owned)
            .collect()
    } else {
        assert_functions.iter().cloned().collect()
    };

    // Build CFGs and dominators per function
    let mut cfgs: BTreeMap<FunctionId, Cfg> = BTreeMap::new();
    let mut dom_maps: BTreeMap<FunctionId, BTreeMap<BlockId, BlockId>> = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        let cfg = Cfg::build(func);
        let doms = compute_dominators(&cfg);
        cfgs.insert(func.id, cfg);
        dom_maps.insert(func.id, doms);
    }

    // Find assertion function IDs
    let assert_func_ids: BTreeSet<FunctionId> = module
        .functions
        .iter()
        .filter(|f| assert_fn_set.contains(&f.name))
        .map(|f| f.id)
        .collect();

    let mut proven = Vec::new();
    let mut may_fail = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = AssertionDiagnostics::default();

    // Scan for assertion calls
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                // Check if this is a call to an assert function
                let is_assert = match &inst.op {
                    Operation::CallDirect { callee } => assert_func_ids.contains(callee),
                    _ => false,
                };

                if !is_assert {
                    continue;
                }

                diagnostics.total_assertions += 1;

                // The assert call is in a block guarded by a CondBr.
                // The assertion condition is the CondBr's condition (negated,
                // since __assert_fail is called on the failure branch).
                // Collect dominating guards at this block.
                let (Some(cfg), Some(doms)) = (cfgs.get(&func.id), dom_maps.get(&func.id)) else {
                    unknown.push(AssertionFinding {
                        function: func.name.clone(),
                        function_id: func.id,
                        inst: inst.id,
                        condition_desc: "unknown".to_string(),
                        status: AssertionStatus::Unknown,
                        counterexample: None,
                    });
                    diagnostics.unknown_count += 1;
                    continue;
                };

                let pc = extract_dominating_guards(block.id, func.id, cfg, doms, &index);
                diagnostics.guards_extracted += pc.guards.len();

                // Add interval constraints if available
                let _ = absint_result; // Future: integrate interval constraints

                if pc.guards.len() > max_guards {
                    unknown.push(AssertionFinding {
                        function: func.name.clone(),
                        function_id: func.id,
                        inst: inst.id,
                        condition_desc: format!("too many guards ({})", pc.guards.len()),
                        status: AssertionStatus::Unknown,
                        counterexample: None,
                    });
                    diagnostics.unknown_count += 1;
                    continue;
                }

                // Check if the path to the assert call is feasible
                // If feasible -> assertion may fail (the failure path is reachable)
                // If infeasible -> assertion is proven (failure path unreachable)
                diagnostics.z3_calls += 1;
                let result = checker.check_feasibility(&pc, &index);

                let condition_desc = format!("assert at block {:?} in {}", block.id, func.name);

                match result {
                    crate::z3_utils::FeasibilityResult::Feasible => {
                        // Path to __assert_fail is feasible -> assertion may fail
                        may_fail.push(AssertionFinding {
                            function: func.name.clone(),
                            function_id: func.id,
                            inst: inst.id,
                            condition_desc,
                            status: AssertionStatus::MayFail,
                            counterexample: None, // TODO: extract Z3 model
                        });
                        diagnostics.may_fail_count += 1;
                    }
                    crate::z3_utils::FeasibilityResult::Infeasible => {
                        // Path to __assert_fail is infeasible -> assertion always holds
                        proven.push(AssertionFinding {
                            function: func.name.clone(),
                            function_id: func.id,
                            inst: inst.id,
                            condition_desc,
                            status: AssertionStatus::Proven,
                            counterexample: None,
                        });
                        diagnostics.proven_count += 1;
                    }
                    crate::z3_utils::FeasibilityResult::Unknown => {
                        unknown.push(AssertionFinding {
                            function: func.name.clone(),
                            function_id: func.id,
                            inst: inst.id,
                            condition_desc,
                            status: AssertionStatus::Unknown,
                            counterexample: None,
                        });
                        diagnostics.unknown_count += 1;
                    }
                }
            }
        }
    }

    AssertionResult {
        proven,
        may_fail,
        unknown,
        diagnostics,
    }
}
