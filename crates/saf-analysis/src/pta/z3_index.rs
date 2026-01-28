//! Z3-based array index alias refinement.
//!
//! When two array accesses use symbolic indices (e.g., `a[i]` and `a[j]`),
//! Z3 can determine if they may or must alias by checking whether `i == j`
//! is satisfiable.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, BinaryOp, Constant, Instruction, Operation};
use saf_core::ids::ValueId;
use serde::{Deserialize, Serialize};

use super::location::IndexExpr;

/// Result of Z3 index comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexComparisonResult {
    /// Indices must be equal (i == j is always true).
    MustEqual,
    /// Indices may be equal (i == j is satisfiable but not always true).
    MayEqual,
    /// Indices can never be equal (i == j is unsatisfiable).
    NeverEqual,
    /// Cannot determine (timeout or unsupported operation).
    Unknown,
}

/// Diagnostics for Z3 index checking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Z3IndexDiagnostics {
    /// Number of Z3 calls made.
    pub z3_calls: u32,
    /// Number of timeouts.
    pub timeouts: u32,
    /// Number of must-equal results.
    pub must_equal: u32,
    /// Number of may-equal results.
    pub may_equal: u32,
    /// Number of never-equal results.
    pub never_equal: u32,
    /// Number of unknown results.
    pub unknown: u32,
}

/// Z3-based index checker with timeout support.
pub struct Z3IndexChecker {
    timeout_ms: u64,
}

impl Z3IndexChecker {
    /// Create a new index checker with the specified timeout.
    #[must_use]
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    /// Check if two index expressions may be equal.
    ///
    /// For constant indices, this is a simple comparison.
    /// For symbolic indices, this uses Z3 to check satisfiability.
    #[must_use]
    pub fn indices_may_equal(
        &self,
        idx1: &IndexExpr,
        idx2: &IndexExpr,
        module: Option<&AirModule>,
    ) -> IndexComparisonResult {
        match (idx1, idx2) {
            // Unknown indices are treated conservatively as may-equal
            (IndexExpr::Unknown, _) | (_, IndexExpr::Unknown) => IndexComparisonResult::MayEqual,

            // Two constants: direct comparison
            (IndexExpr::Constant(a), IndexExpr::Constant(b)) => {
                if a == b {
                    IndexComparisonResult::MustEqual
                } else {
                    IndexComparisonResult::NeverEqual
                }
            }

            // Constant and symbolic: check if symbolic can equal constant
            (IndexExpr::Constant(c), IndexExpr::Symbolic(v))
            | (IndexExpr::Symbolic(v), IndexExpr::Constant(c)) => {
                self.check_symbolic_equals_constant(*v, *c, module)
            }

            // Two symbolic: check if they can be equal
            (IndexExpr::Symbolic(v1), IndexExpr::Symbolic(v2)) => {
                if v1 == v2 {
                    // Same SSA value means same runtime value
                    IndexComparisonResult::MustEqual
                } else {
                    self.check_symbolics_may_equal(*v1, *v2, module)
                }
            }
        }
    }

    /// Check if a symbolic index can equal a constant.
    fn check_symbolic_equals_constant(
        &self,
        sym: ValueId,
        constant: i64,
        module: Option<&AirModule>,
    ) -> IndexComparisonResult {
        let Some(module) = module else {
            return IndexComparisonResult::MayEqual;
        };

        // Check if the symbolic value is itself a constant
        if let Some(Constant::Int { value, .. }) = module.constants.get(&sym) {
            return if *value == constant {
                IndexComparisonResult::MustEqual
            } else {
                IndexComparisonResult::NeverEqual
            };
        }

        // Try to build Z3 expression for the symbolic value
        let solver = z3::Solver::new();
        let mut params = z3::Params::new();
        #[allow(clippy::cast_possible_truncation)]
        params.set_u32("timeout", self.timeout_ms as u32);
        solver.set_params(&params);

        let mut var_cache = BTreeMap::new();
        let Some(sym_expr) = self.build_z3_expr(sym, module, &mut var_cache) else {
            return IndexComparisonResult::MayEqual;
        };

        let const_expr = z3::ast::Int::from_i64(constant);

        // Check if sym == constant is satisfiable
        #[allow(deprecated, clippy::needless_borrows_for_generic_args)]
        solver.assert(&sym_expr._eq(&const_expr));
        match solver.check() {
            z3::SatResult::Sat => {
                // Also check if sym != constant is satisfiable
                let solver2 = z3::Solver::new();
                solver2.set_params(&params);
                #[allow(deprecated)]
                let not_eq = sym_expr._eq(&const_expr).not();
                solver2.assert(&not_eq);
                match solver2.check() {
                    z3::SatResult::Sat => IndexComparisonResult::MayEqual,
                    z3::SatResult::Unsat => IndexComparisonResult::MustEqual,
                    z3::SatResult::Unknown => IndexComparisonResult::Unknown,
                }
            }
            z3::SatResult::Unsat => IndexComparisonResult::NeverEqual,
            z3::SatResult::Unknown => IndexComparisonResult::Unknown,
        }
    }

    /// Check if two symbolic indices may be equal.
    fn check_symbolics_may_equal(
        &self,
        sym1: ValueId,
        sym2: ValueId,
        module: Option<&AirModule>,
    ) -> IndexComparisonResult {
        let Some(module) = module else {
            return IndexComparisonResult::MayEqual;
        };

        // First check if both are constants
        if let (Some(Constant::Int { value: v1, .. }), Some(Constant::Int { value: v2, .. })) =
            (module.constants.get(&sym1), module.constants.get(&sym2))
        {
            return if v1 == v2 {
                IndexComparisonResult::MustEqual
            } else {
                IndexComparisonResult::NeverEqual
            };
        }

        // Try to build Z3 expressions
        let solver = z3::Solver::new();
        let mut params = z3::Params::new();
        #[allow(clippy::cast_possible_truncation)]
        params.set_u32("timeout", self.timeout_ms as u32);
        solver.set_params(&params);

        let mut var_cache = BTreeMap::new();
        let Some(expr1) = self.build_z3_expr(sym1, module, &mut var_cache) else {
            return IndexComparisonResult::MayEqual;
        };
        let Some(expr2) = self.build_z3_expr(sym2, module, &mut var_cache) else {
            return IndexComparisonResult::MayEqual;
        };

        // Check if sym1 == sym2 is satisfiable
        #[allow(deprecated, clippy::needless_borrows_for_generic_args)]
        solver.assert(&expr1._eq(&expr2));
        match solver.check() {
            z3::SatResult::Sat => {
                // Also check if sym1 != sym2 is satisfiable
                let solver2 = z3::Solver::new();
                solver2.set_params(&params);
                #[allow(deprecated)]
                let not_eq = expr1._eq(&expr2).not();
                solver2.assert(&not_eq);
                match solver2.check() {
                    z3::SatResult::Sat => IndexComparisonResult::MayEqual,
                    z3::SatResult::Unsat => IndexComparisonResult::MustEqual,
                    z3::SatResult::Unknown => IndexComparisonResult::Unknown,
                }
            }
            z3::SatResult::Unsat => IndexComparisonResult::NeverEqual,
            z3::SatResult::Unknown => IndexComparisonResult::Unknown,
        }
    }

    /// Build a Z3 integer expression for a ValueId.
    ///
    /// Returns None if the value cannot be represented as a Z3 expression.
    fn build_z3_expr(
        &self,
        value: ValueId,
        module: &AirModule,
        var_cache: &mut BTreeMap<ValueId, z3::ast::Int>,
    ) -> Option<z3::ast::Int> {
        // Check cache first
        if let Some(cached) = var_cache.get(&value) {
            return Some(cached.clone());
        }

        // Check if it's a constant
        if let Some(Constant::Int { value: v, .. }) = module.constants.get(&value) {
            let expr = z3::ast::Int::from_i64(*v);
            var_cache.insert(value, expr.clone());
            return Some(expr);
        }

        // Try to find the instruction that defines this value
        let inst = self.find_defining_instruction(value, module)?;
        let expr = self.instruction_to_z3(inst, module, var_cache)?;
        var_cache.insert(value, expr.clone());
        Some(expr)
    }

    /// Find the instruction that defines a value.
    #[allow(clippy::unused_self)]
    fn find_defining_instruction<'a>(
        &self,
        value: ValueId,
        module: &'a AirModule,
    ) -> Option<&'a Instruction> {
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if inst.dst == Some(value) {
                        return Some(inst);
                    }
                }
            }
        }
        None
    }

    /// Convert an instruction to a Z3 expression.
    fn instruction_to_z3(
        &self,
        inst: &Instruction,
        module: &AirModule,
        var_cache: &mut BTreeMap<ValueId, z3::ast::Int>,
    ) -> Option<z3::ast::Int> {
        match &inst.op {
            Operation::BinaryOp { kind } => {
                if inst.operands.len() < 2 {
                    return None;
                }
                let lhs = self.build_z3_expr(inst.operands[0], module, var_cache)?;
                let rhs = self.build_z3_expr(inst.operands[1], module, var_cache)?;

                Some(match kind {
                    BinaryOp::Add => z3::ast::Int::add(&[&lhs, &rhs]),
                    BinaryOp::Sub => z3::ast::Int::sub(&[&lhs, &rhs]),
                    BinaryOp::Mul => z3::ast::Int::mul(&[&lhs, &rhs]),
                    // Division and modulo need special handling for division by zero
                    _ => return None,
                })
            }

            Operation::Phi { .. } => {
                // For phi nodes, create a fresh symbolic variable
                // (conservative but sound)
                let name = format!("phi_{:x}", inst.dst?.raw());
                Some(z3::ast::Int::new_const(name.as_str()))
            }

            Operation::Copy => {
                if inst.operands.is_empty() {
                    return None;
                }
                self.build_z3_expr(inst.operands[0], module, var_cache)
            }

            Operation::Load => {
                // Loaded values are symbolic
                let name = format!("load_{:x}", inst.dst?.raw());
                Some(z3::ast::Int::new_const(name.as_str()))
            }

            Operation::CallDirect { .. } | Operation::CallIndirect { .. } => {
                // Call results are symbolic
                let name = format!("call_{:x}", inst.dst?.raw());
                Some(z3::ast::Int::new_const(name.as_str()))
            }

            Operation::Cast { .. } => {
                // For simple casts, propagate the operand
                if inst.operands.is_empty() {
                    return None;
                }
                self.build_z3_expr(inst.operands[0], module, var_cache)
            }

            _ => {
                // For unknown operations, create a fresh symbolic variable
                let name = format!("op_{:x}", inst.dst?.raw());
                Some(z3::ast::Int::new_const(name.as_str()))
            }
        }
    }
}

/// Convenience function for checking if two index expressions may be equal.
///
/// Uses default timeout of 100ms.
#[must_use]
pub fn indices_may_equal(
    idx1: &IndexExpr,
    idx2: &IndexExpr,
    module: Option<&AirModule>,
) -> IndexComparisonResult {
    let checker = Z3IndexChecker::new(100);
    checker.indices_may_equal(idx1, idx2, module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_indices_equal() {
        let result = indices_may_equal(&IndexExpr::Constant(5), &IndexExpr::Constant(5), None);
        assert_eq!(result, IndexComparisonResult::MustEqual);
    }

    #[test]
    fn constant_indices_not_equal() {
        let result = indices_may_equal(&IndexExpr::Constant(5), &IndexExpr::Constant(10), None);
        assert_eq!(result, IndexComparisonResult::NeverEqual);
    }

    #[test]
    fn unknown_index_may_equal() {
        let result = indices_may_equal(&IndexExpr::Unknown, &IndexExpr::Constant(5), None);
        assert_eq!(result, IndexComparisonResult::MayEqual);
    }

    #[test]
    fn same_symbolic_must_equal() {
        let v = ValueId::new(42);
        let result = indices_may_equal(&IndexExpr::Symbolic(v), &IndexExpr::Symbolic(v), None);
        assert_eq!(result, IndexComparisonResult::MustEqual);
    }

    #[test]
    fn different_symbolic_without_module_may_equal() {
        let v1 = ValueId::new(1);
        let v2 = ValueId::new(2);
        let result = indices_may_equal(&IndexExpr::Symbolic(v1), &IndexExpr::Symbolic(v2), None);
        // Without module context, we conservatively assume may-equal
        assert_eq!(result, IndexComparisonResult::MayEqual);
    }

    #[test]
    fn symbolic_vs_constant_without_module() {
        let v = ValueId::new(1);
        let result = indices_may_equal(&IndexExpr::Symbolic(v), &IndexExpr::Constant(5), None);
        // Without module context, we conservatively assume may-equal
        assert_eq!(result, IndexComparisonResult::MayEqual);
    }
}
