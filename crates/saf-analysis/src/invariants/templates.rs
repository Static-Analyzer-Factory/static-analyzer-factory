//! Loop invariant templates for synthesis.
//!
//! Provides templates for common loop invariant patterns that can be
//! checked and strengthened during invariant synthesis.

use std::collections::BTreeSet;

use saf_core::air::{AirBlock, AirFunction, AirModule, Operation};
use saf_core::ids::{BlockId, ValueId};

/// Template for a loop invariant.
///
/// Templates are patterns for invariants that can be instantiated
/// with specific values from the loop.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvariantTemplate {
    /// Value is in a constant interval: `lo <= x <= hi`
    Interval {
        /// Variable being bounded.
        var: ValueId,
        /// Lower bound (None = unbounded below).
        lo: Option<i128>,
        /// Upper bound (None = unbounded above).
        hi: Option<i128>,
    },

    /// Linear inequality: `x <= y + c`
    LinearLeq {
        /// Left-hand side variable.
        left: ValueId,
        /// Right-hand side variable.
        right: ValueId,
        /// Constant offset.
        constant: i128,
    },

    /// Linear equality: `x == y + c`
    LinearEq {
        /// Left-hand side variable.
        left: ValueId,
        /// Right-hand side variable.
        right: ValueId,
        /// Constant offset.
        constant: i128,
    },

    /// Modular constraint: `x mod m == r`
    Modular {
        /// Variable.
        var: ValueId,
        /// Modulus (must be positive).
        modulus: i128,
        /// Remainder.
        remainder: i128,
    },

    /// Non-null pointer: `x != NULL`
    NonNull {
        /// Pointer variable.
        ptr: ValueId,
    },

    /// Loop counter bound: `i < n` (common pattern for bounded loops)
    CounterBound {
        /// Loop counter.
        counter: ValueId,
        /// Bound variable.
        bound: ValueId,
    },
}

impl InvariantTemplate {
    /// Get all variables referenced by this template.
    #[must_use]
    pub fn variables(&self) -> BTreeSet<ValueId> {
        match self {
            Self::Interval { var, .. } | Self::Modular { var, .. } => [*var].into_iter().collect(),
            Self::LinearLeq { left, right, .. } | Self::LinearEq { left, right, .. } => {
                [*left, *right].into_iter().collect()
            }
            Self::NonNull { ptr } => [*ptr].into_iter().collect(),
            Self::CounterBound { counter, bound } => [*counter, *bound].into_iter().collect(),
        }
    }

    /// Check if this is a relational template (involves multiple variables).
    #[must_use]
    pub fn is_relational(&self) -> bool {
        matches!(
            self,
            Self::LinearLeq { .. } | Self::LinearEq { .. } | Self::CounterBound { .. }
        )
    }

    /// Human-readable description of the template.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Interval { var, lo, hi } => {
                let lo_str = lo.map_or("unbounded".to_string(), |v| v.to_string());
                let hi_str = hi.map_or("unbounded".to_string(), |v| v.to_string());
                format!("{lo_str} <= v{} <= {hi_str}", var.0)
            }
            Self::LinearLeq {
                left,
                right,
                constant,
            } => {
                format!("v{} <= v{} + {constant}", left.0, right.0)
            }
            Self::LinearEq {
                left,
                right,
                constant,
            } => {
                format!("v{} == v{} + {constant}", left.0, right.0)
            }
            Self::Modular {
                var,
                modulus,
                remainder,
            } => {
                format!("v{} mod {modulus} == {remainder}", var.0)
            }
            Self::NonNull { ptr } => format!("v{} != NULL", ptr.0),
            Self::CounterBound { counter, bound } => {
                format!("v{} < v{}", counter.0, bound.0)
            }
        }
    }
}

/// Generate candidate invariant templates from loop structure.
///
/// Analyzes the loop header and body to propose candidate invariants based on:
/// - Loop guards (conditions that control loop exit)
/// - Induction variables (counters incremented each iteration)
/// - Pointer operations (for non-null invariants)
///
/// # Arguments
///
/// * `loop_header` - Block ID of the loop header
/// * `loop_body` - Block IDs comprising the loop body
/// * `func` - The function containing the loop
/// * `module` - The AIR module
///
/// # Returns
///
/// A vector of candidate invariant templates to check.
#[must_use]
pub fn generate_templates(
    loop_header: BlockId,
    loop_body: &[BlockId],
    func: &AirFunction,
    _module: &AirModule,
) -> Vec<InvariantTemplate> {
    let mut templates = Vec::new();

    // Find the loop header block
    let Some(header_block) = func.blocks.iter().find(|b| b.id == loop_header) else {
        return templates;
    };

    // Analyze the header block for guards
    analyze_guards(header_block, &mut templates);

    // Analyze loop body for induction variables
    let body_blocks: Vec<_> = func
        .blocks
        .iter()
        .filter(|b| loop_body.contains(&b.id))
        .collect();
    analyze_induction_vars(&body_blocks, &mut templates);

    // Analyze for pointer non-nullness
    analyze_pointers(&body_blocks, &mut templates);

    templates
}

/// Analyze a block for guard conditions that could be invariants.
fn analyze_guards(block: &AirBlock, templates: &mut Vec<InvariantTemplate>) {
    for inst in &block.instructions {
        // Look for comparison operations
        if let Operation::BinaryOp { kind, .. } = &inst.op {
            // Check for comparisons that might be loop guards
            use saf_core::air::BinaryOp as BinOp;
            match kind {
                BinOp::ICmpSlt | BinOp::ICmpUlt => {
                    // x < y pattern: counter < bound
                    if inst.operands.len() >= 2 {
                        let counter = inst.operands[0];
                        let bound = inst.operands[1];
                        templates.push(InvariantTemplate::CounterBound { counter, bound });
                        // Also add as linear inequality
                        templates.push(InvariantTemplate::LinearLeq {
                            left: counter,
                            right: bound,
                            constant: -1,
                        });
                    }
                }
                BinOp::ICmpSle | BinOp::ICmpUle => {
                    // x <= y pattern
                    if inst.operands.len() >= 2 {
                        templates.push(InvariantTemplate::LinearLeq {
                            left: inst.operands[0],
                            right: inst.operands[1],
                            constant: 0,
                        });
                    }
                }
                BinOp::ICmpSge | BinOp::ICmpUge => {
                    // x >= y is equivalent to y <= x
                    if inst.operands.len() >= 2 {
                        templates.push(InvariantTemplate::LinearLeq {
                            left: inst.operands[1],
                            right: inst.operands[0],
                            constant: 0,
                        });
                    }
                }
                BinOp::ICmpEq => {
                    // x == y pattern (for equality invariants)
                    if inst.operands.len() >= 2 {
                        templates.push(InvariantTemplate::LinearEq {
                            left: inst.operands[0],
                            right: inst.operands[1],
                            constant: 0,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

/// Analyze loop body for induction variables.
fn analyze_induction_vars(body: &[&AirBlock], templates: &mut Vec<InvariantTemplate>) {
    for block in body {
        for inst in &block.instructions {
            // Look for increment patterns: x = x + c
            if let Operation::BinaryOp { kind, .. } = &inst.op {
                use saf_core::air::BinaryOp as BinOp;
                if matches!(kind, BinOp::Add | BinOp::Sub) {
                    if let Some(dst) = inst.dst {
                        // If destination appears in operands, this might be an induction var
                        if inst.operands.contains(&dst) {
                            // Suggest interval templates for induction variables
                            templates.push(InvariantTemplate::Interval {
                                var: dst,
                                lo: Some(0),
                                hi: None,
                            });
                            // Also suggest modular invariants (stride)
                            templates.push(InvariantTemplate::Modular {
                                var: dst,
                                modulus: 1,
                                remainder: 0,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Analyze loop body for pointer operations.
fn analyze_pointers(body: &[&AirBlock], templates: &mut Vec<InvariantTemplate>) {
    for block in body {
        for inst in &block.instructions {
            // Look for load/store operations that use pointers
            match &inst.op {
                Operation::Load | Operation::Store => {
                    // The first operand is typically the pointer
                    if let Some(&ptr) = inst.operands.first() {
                        templates.push(InvariantTemplate::NonNull { ptr });
                    }
                }
                Operation::Gep { .. } => {
                    // Base pointer for GEP
                    if let Some(&ptr) = inst.operands.first() {
                        templates.push(InvariantTemplate::NonNull { ptr });
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_template() {
        let template = InvariantTemplate::Interval {
            var: ValueId::new(1),
            lo: Some(0),
            hi: Some(100),
        };

        assert!(!template.is_relational());
        assert_eq!(template.variables().len(), 1);
        assert!(template.describe().contains("0"));
        assert!(template.describe().contains("100"));
    }

    #[test]
    fn test_linear_leq_template() {
        let template = InvariantTemplate::LinearLeq {
            left: ValueId::new(1),
            right: ValueId::new(2),
            constant: -1,
        };

        assert!(template.is_relational());
        assert_eq!(template.variables().len(), 2);
        assert!(template.describe().contains("<="));
    }

    #[test]
    fn test_counter_bound_template() {
        let template = InvariantTemplate::CounterBound {
            counter: ValueId::new(5),
            bound: ValueId::new(10),
        };

        assert!(template.is_relational());
        let vars = template.variables();
        assert!(vars.contains(&ValueId::new(5)));
        assert!(vars.contains(&ValueId::new(10)));
    }

    #[test]
    fn test_non_null_template() {
        let template = InvariantTemplate::NonNull {
            ptr: ValueId::new(42),
        };

        assert!(!template.is_relational());
        assert!(template.describe().contains("NULL"));
    }
}
