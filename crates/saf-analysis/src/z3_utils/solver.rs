//! Z3-based path feasibility checking.
//!
//! Translates AIR guard conditions into Z3 formulas and checks
//! satisfiability. UNSAT means the path is infeasible (false positive),
//! SAT means feasible (keep the finding).

use std::collections::BTreeMap;

use saf_core::air::BinaryOp;
use saf_core::ids::ValueId;
use serde::{Deserialize, Serialize};

use crate::guard::{ConditionInfo, Guard, OperandInfo, PathCondition, ValueLocationIndex};

// ---------------------------------------------------------------------------
// Feasibility result
// ---------------------------------------------------------------------------

/// Result of Z3 feasibility checking for a single path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeasibilityResult {
    /// SAT — the path is feasible.
    Feasible,
    /// UNSAT — the path is provably infeasible (false positive).
    Infeasible,
    /// Timeout or undecidable — conservatively keep.
    Unknown,
}

// ---------------------------------------------------------------------------
// Z3 filter diagnostics (shared across all features)
// ---------------------------------------------------------------------------

/// Diagnostics from Z3-based filtering, shared across all analysis features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Z3FilterDiagnostics {
    /// Total items processed.
    pub total_items: usize,
    /// Items classified as feasible / confirmed.
    pub feasible_count: usize,
    /// Items classified as infeasible / refuted.
    pub infeasible_count: usize,
    /// Items classified as unknown (timeout).
    pub unknown_count: usize,
    /// Total guards extracted across all items.
    pub guards_extracted: usize,
    /// Total Z3 solver calls.
    pub z3_calls: usize,
    /// Total Z3 timeouts.
    pub z3_timeouts: usize,
    /// Items skipped due to `max_guards` limit.
    pub skipped_too_many_guards: usize,
}

// ---------------------------------------------------------------------------
// Path feasibility checker
// ---------------------------------------------------------------------------

/// Z3-based path feasibility checker.
///
/// Translates `PathCondition` guards into Z3 AST, conjoins them, and
/// checks satisfiability. Fresh Z3 variables are created for unknown operands.
pub struct PathFeasibilityChecker {
    /// Timeout per check in milliseconds.
    timeout_ms: u64,
}

impl PathFeasibilityChecker {
    /// Create a new checker with the given per-finding timeout.
    #[must_use]
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    /// Check whether a path condition is feasible.
    ///
    /// Returns:
    /// - `Feasible` if the conjoined guards are satisfiable
    /// - `Infeasible` if the guards are contradictory (UNSAT)
    /// - `Unknown` if the solver times out or cannot decide
    pub fn check_feasibility(
        &self,
        path_condition: &PathCondition,
        index: &ValueLocationIndex,
    ) -> FeasibilityResult {
        if path_condition.is_empty() {
            return FeasibilityResult::Feasible;
        }

        // Pre-Z3 quick check: detect self-contradictory guards.
        // If the same condition (`ValueId`) appears with opposite `branch_taken`
        // within a single path, the path is infeasible without needing Z3.
        for i in 0..path_condition.guards.len() {
            for j in (i + 1)..path_condition.guards.len() {
                if path_condition.guards[i].condition == path_condition.guards[j].condition
                    && path_condition.guards[i].branch_taken
                        != path_condition.guards[j].branch_taken
                {
                    return FeasibilityResult::Infeasible;
                }
            }
        }

        let solver = z3::Solver::new();
        let mut params = z3::Params::new();
        #[allow(clippy::cast_possible_truncation)]
        // Z3 API uses u32; values >> u32::MAX are impractical
        params.set_u32("timeout", self.timeout_ms as u32);
        solver.set_params(&params);

        let mut var_cache: BTreeMap<ValueId, z3::ast::Int> = BTreeMap::new();

        for guard in &path_condition.guards {
            let cond_expr = self.translate_guard(guard, index, &mut var_cache);
            if let Some(expr) = cond_expr {
                solver.assert(&expr);
            }
        }

        match solver.check() {
            z3::SatResult::Sat => FeasibilityResult::Feasible,
            z3::SatResult::Unsat => FeasibilityResult::Infeasible,
            z3::SatResult::Unknown => FeasibilityResult::Unknown,
        }
    }

    /// Check whether two path conditions can hold simultaneously.
    ///
    /// Used for `MultiReach` filtering: if an allocation reaches two sinks
    /// through mutually exclusive branches, the conjunction of their
    /// path conditions is UNSAT, proving the finding is a false positive.
    ///
    /// Uses a shared variable namespace so that the same `ValueId` maps to
    /// the same Z3 variable in both paths (e.g., `cond != 0` from path A
    /// and `cond == 0` from path B correctly yields UNSAT).
    ///
    /// Returns:
    /// - `Feasible` if the conjoined guards from both paths are satisfiable
    /// - `Infeasible` if the conjunction is UNSAT (mutually exclusive paths)
    /// - `Unknown` on timeout
    pub fn check_joint_feasibility(
        &self,
        pc_a: &PathCondition,
        pc_b: &PathCondition,
        index: &ValueLocationIndex,
    ) -> FeasibilityResult {
        if pc_a.is_empty() || pc_b.is_empty() {
            return FeasibilityResult::Feasible;
        }

        // Pre-Z3 quick check: detect complementary guards across paths.
        // If the same condition (`ValueId`) appears with opposite `branch_taken`
        // in the two paths, they are mutually exclusive without needing Z3.
        for guard_a in &pc_a.guards {
            for guard_b in &pc_b.guards {
                if guard_a.condition == guard_b.condition
                    && guard_a.branch_taken != guard_b.branch_taken
                {
                    return FeasibilityResult::Infeasible;
                }
            }
        }

        let solver = z3::Solver::new();
        let mut params = z3::Params::new();
        #[allow(clippy::cast_possible_truncation)]
        params.set_u32("timeout", self.timeout_ms as u32);
        solver.set_params(&params);

        let mut var_cache: BTreeMap<ValueId, z3::ast::Int> = BTreeMap::new();

        // Assert all guards from path A
        for guard in &pc_a.guards {
            if let Some(expr) = self.translate_guard(guard, index, &mut var_cache) {
                solver.assert(&expr);
            }
        }

        // Assert all guards from path B (shared variable namespace)
        for guard in &pc_b.guards {
            if let Some(expr) = self.translate_guard(guard, index, &mut var_cache) {
                solver.assert(&expr);
            }
        }

        match solver.check() {
            z3::SatResult::Sat => FeasibilityResult::Feasible,
            z3::SatResult::Unsat => FeasibilityResult::Infeasible,
            z3::SatResult::Unknown => FeasibilityResult::Unknown,
        }
    }

    /// Translate a single guard into a Z3 boolean expression.
    #[allow(clippy::unused_self)] // Method signature for future extensibility
    fn translate_guard(
        &self,
        guard: &Guard,
        index: &ValueLocationIndex,
        var_cache: &mut BTreeMap<ValueId, z3::ast::Int>,
    ) -> Option<z3::ast::Bool> {
        let cond_info = index.condition_info(guard.condition)?;
        let cmp_expr = Self::translate_icmp(cond_info, var_cache)?;

        Some(if guard.branch_taken {
            cmp_expr
        } else {
            cmp_expr.not()
        })
    }

    /// Translate an ICmp comparison into a Z3 boolean expression.
    #[allow(deprecated)] // z3 crate _eq → eq migration pending
    fn translate_icmp(
        cond: &ConditionInfo,
        var_cache: &mut BTreeMap<ValueId, z3::ast::Int>,
    ) -> Option<z3::ast::Bool> {
        let lhs = Self::translate_operand(&cond.lhs, var_cache);
        let rhs = Self::translate_operand(&cond.rhs, var_cache);

        let result = match cond.cmp_kind {
            BinaryOp::ICmpEq => lhs._eq(&rhs),
            BinaryOp::ICmpNe => lhs._eq(&rhs).not(),
            BinaryOp::ICmpSlt | BinaryOp::ICmpUlt => lhs.lt(&rhs),
            BinaryOp::ICmpSle | BinaryOp::ICmpUle => lhs.le(&rhs),
            BinaryOp::ICmpSgt | BinaryOp::ICmpUgt => lhs.gt(&rhs),
            BinaryOp::ICmpSge | BinaryOp::ICmpUge => lhs.ge(&rhs),
            _ => return None,
        };

        Some(result)
    }

    /// Translate an operand to a Z3 integer expression.
    pub fn translate_operand(
        operand: &OperandInfo,
        var_cache: &mut BTreeMap<ValueId, z3::ast::Int>,
    ) -> z3::ast::Int {
        match operand {
            OperandInfo::IntConst(v) => z3::ast::Int::from_i64(*v),
            OperandInfo::Null => z3::ast::Int::from_i64(0),
            OperandInfo::Value(vid) => {
                if let Some(cached) = var_cache.get(vid) {
                    return cached.clone();
                }
                let name = format!("v_{}", vid.raw());
                let var = z3::ast::Int::new_const(name.as_str());
                var_cache.insert(*vid, var.clone());
                var
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::{Guard, OperandInfo, PathCondition};

    fn make_checker() -> PathFeasibilityChecker {
        PathFeasibilityChecker::new(5000)
    }

    fn make_index_with_conditions(
        conditions: Vec<(ValueId, BinaryOp, OperandInfo, OperandInfo)>,
    ) -> ValueLocationIndex {
        ValueLocationIndex::from_conditions(conditions)
    }

    #[test]
    fn z3_smoke_test() {
        let solver = z3::Solver::new();
        let x = z3::ast::Int::new_const("x");
        let zero = z3::ast::Int::from_i64(0);

        solver.assert(&x.gt(&zero));
        solver.assert(&x.lt(&zero));

        assert_eq!(solver.check(), z3::SatResult::Unsat);
    }

    #[test]
    fn z3_sat_test() {
        let solver = z3::Solver::new();
        let x = z3::ast::Int::new_const("x");
        let zero = z3::ast::Int::from_i64(0);
        let ten = z3::ast::Int::from_i64(10);

        solver.assert(&x.gt(&zero));
        solver.assert(&x.lt(&ten));

        assert_eq!(solver.check(), z3::SatResult::Sat);
    }

    #[test]
    fn contradictory_guards_infeasible() {
        let checker = make_checker();

        let x = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpEq,
                OperandInfo::Value(x),
                OperandInfo::IntConst(0),
            ),
            (
                cond2,
                BinaryOp::ICmpNe,
                OperandInfo::Value(x),
                OperandInfo::IntConst(0),
            ),
        ]);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond1,
                    branch_taken: true,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond2,
                    branch_taken: true,
                },
            ],
        };

        let result = checker.check_feasibility(&pc, &index);
        assert_eq!(result, FeasibilityResult::Infeasible);
    }

    #[test]
    fn compatible_guards_feasible() {
        let checker = make_checker();

        let x = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpSgt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(0),
            ),
            (
                cond2,
                BinaryOp::ICmpSlt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(10),
            ),
        ]);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond1,
                    branch_taken: true,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond2,
                    branch_taken: true,
                },
            ],
        };

        let result = checker.check_feasibility(&pc, &index);
        assert_eq!(result, FeasibilityResult::Feasible);
    }

    #[test]
    fn empty_guards_feasible() {
        let checker = make_checker();
        let index = make_index_with_conditions(vec![]);
        let pc = PathCondition::empty();

        let result = checker.check_feasibility(&pc, &index);
        assert_eq!(result, FeasibilityResult::Feasible);
    }

    #[test]
    fn null_check_guard() {
        let checker = make_checker();

        let ptr = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpEq,
                OperandInfo::Value(ptr),
                OperandInfo::Null,
            ),
            (
                cond2,
                BinaryOp::ICmpNe,
                OperandInfo::Value(ptr),
                OperandInfo::Null,
            ),
        ]);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond1,
                    branch_taken: true,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond2,
                    branch_taken: true,
                },
            ],
        };

        assert_eq!(
            checker.check_feasibility(&pc, &index),
            FeasibilityResult::Infeasible
        );
    }

    #[test]
    fn negated_branch_works() {
        let checker = make_checker();

        let x = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpSgt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(5),
            ),
            (
                cond2,
                BinaryOp::ICmpEq,
                OperandInfo::Value(x),
                OperandInfo::IntConst(3),
            ),
        ]);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond1,
                    branch_taken: false,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond2,
                    branch_taken: true,
                },
            ],
        };

        assert_eq!(
            checker.check_feasibility(&pc, &index),
            FeasibilityResult::Feasible
        );
    }

    #[test]
    fn negated_branch_infeasible() {
        let checker = make_checker();

        let x = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpSgt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(5),
            ),
            (
                cond2,
                BinaryOp::ICmpEq,
                OperandInfo::Value(x),
                OperandInfo::IntConst(10),
            ),
        ]);

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond1,
                    branch_taken: false,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond2,
                    branch_taken: true,
                },
            ],
        };

        assert_eq!(
            checker.check_feasibility(&pc, &index),
            FeasibilityResult::Infeasible
        );
    }

    #[test]
    fn z3_filter_diagnostics_default() {
        let diag = Z3FilterDiagnostics::default();
        assert_eq!(diag.total_items, 0);
        assert_eq!(diag.feasible_count, 0);
        assert_eq!(diag.infeasible_count, 0);
        assert_eq!(diag.unknown_count, 0);
    }

    // ---- joint feasibility tests ----

    #[test]
    fn joint_feasibility_mutually_exclusive() {
        // Path A takes then-branch (x == 0), Path B takes else-branch (x != 0)
        // Joint: x == 0 AND x != 0 → UNSAT
        let checker = make_checker();
        let x = ValueId::new(1);
        let cond = ValueId::new(100);
        let index = make_index_with_conditions(vec![(
            cond,
            BinaryOp::ICmpEq,
            OperandInfo::Value(x),
            OperandInfo::IntConst(0),
        )]);

        let pc_a = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: true, // x == 0
            }],
        };
        let pc_b = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: false, // x != 0
            }],
        };

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Infeasible
        );
    }

    #[test]
    fn joint_feasibility_compatible_paths() {
        // Path A: x > 5, Path B: x > 10
        // Joint: x > 5 AND x > 10 → SAT (e.g., x = 11)
        let checker = make_checker();
        let x = ValueId::new(1);
        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);
        let index = make_index_with_conditions(vec![
            (
                cond1,
                BinaryOp::ICmpSgt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(5),
            ),
            (
                cond2,
                BinaryOp::ICmpSgt,
                OperandInfo::Value(x),
                OperandInfo::IntConst(10),
            ),
        ]);

        let pc_a = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond1,
                branch_taken: true,
            }],
        };
        let pc_b = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(2),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond2,
                branch_taken: true,
            }],
        };

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Feasible
        );
    }

    #[test]
    fn joint_feasibility_empty_path_conservative() {
        // If either path has no guards, can't prove mutual exclusivity
        let checker = make_checker();
        let index = make_index_with_conditions(vec![]);
        let pc_a = PathCondition::empty();
        let pc_b = PathCondition::empty();

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Feasible
        );
    }

    // ---- pre-Z3 complementary guard tests ----

    #[test]
    fn self_contradictory_guards_detected_pre_z3() {
        // A single path with the SAME condition but opposite branch_taken
        // should be detected as infeasible by the pre-Z3 check, even with
        // an empty index (no condition info for Z3 to translate).
        let checker = make_checker();
        let cond = ValueId::new(200);
        let index = make_index_with_conditions(vec![]); // empty — no Z3 info

        let pc = PathCondition {
            guards: vec![
                Guard {
                    block: saf_core::ids::BlockId::new(1),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond,
                    branch_taken: true,
                },
                Guard {
                    block: saf_core::ids::BlockId::new(2),
                    function: saf_core::ids::FunctionId::new(1),
                    condition: cond,
                    branch_taken: false,
                },
            ],
        };

        assert_eq!(
            checker.check_feasibility(&pc, &index),
            FeasibilityResult::Infeasible,
        );
    }

    #[test]
    fn joint_complementary_guards_detected_pre_z3() {
        // Two paths with the SAME condition but opposite branch_taken should
        // be detected as infeasible by the pre-Z3 check, even with an empty
        // index (no condition info entries for Z3 to translate).
        let checker = make_checker();
        let cond = ValueId::new(200);
        let index = make_index_with_conditions(vec![]); // empty — no Z3 info

        let pc_a = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: true,
            }],
        };
        let pc_b = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(2),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: false,
            }],
        };

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Infeasible,
        );
    }

    #[test]
    fn compatible_same_condition_same_branch() {
        // Two guards with the SAME condition AND SAME branch_taken should NOT
        // trigger the pre-Z3 contradiction check — they are compatible.
        let checker = make_checker();
        let cond = ValueId::new(200);
        let x = ValueId::new(1);
        let index = make_index_with_conditions(vec![(
            cond,
            BinaryOp::ICmpEq,
            OperandInfo::Value(x),
            OperandInfo::IntConst(0),
        )]);

        let pc_a = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: true,
            }],
        };
        let pc_b = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(2),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: true, // same branch — compatible
            }],
        };

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Feasible,
        );
    }

    #[test]
    fn joint_feasibility_one_empty_conservative() {
        // One path has guards, the other is empty — can't prove exclusive
        let checker = make_checker();
        let x = ValueId::new(1);
        let cond = ValueId::new(100);
        let index = make_index_with_conditions(vec![(
            cond,
            BinaryOp::ICmpEq,
            OperandInfo::Value(x),
            OperandInfo::IntConst(0),
        )]);

        let pc_a = PathCondition {
            guards: vec![Guard {
                block: saf_core::ids::BlockId::new(1),
                function: saf_core::ids::FunctionId::new(1),
                condition: cond,
                branch_taken: true,
            }],
        };
        let pc_b = PathCondition::empty();

        assert_eq!(
            checker.check_joint_feasibility(&pc_a, &pc_b, &index),
            FeasibilityResult::Feasible
        );
    }
}
