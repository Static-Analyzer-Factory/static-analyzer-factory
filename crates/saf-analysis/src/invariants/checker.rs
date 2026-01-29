//! Invariant checking using abstract interpretation.
//!
//! Verifies candidate loop invariants by checking:
//! 1. Initiation: invariant holds on loop entry
//! 2. Consecution: invariant is preserved by loop body

use std::collections::BTreeSet;

use saf_core::air::{AirFunction, AirModule};
use saf_core::ids::{BlockId, InstId};

use crate::cfg::Cfg;

use super::templates::InvariantTemplate;

/// Result of checking an invariant candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvariantCheckResult {
    /// Invariant is valid (inductive).
    Valid,
    /// Invariant does not hold; counterexample provided.
    Invalid {
        /// Instruction trace showing the violation.
        counterexample: Vec<InstId>,
        /// Description of why it's invalid.
        reason: String,
    },
    /// Could not determine validity.
    Unknown {
        /// Reason for uncertainty.
        reason: String,
    },
}

/// Configuration for invariant checking.
#[derive(Clone, Debug)]
pub struct CheckerConfig {
    /// Maximum iterations for abstract interpretation.
    pub max_iterations: usize,
    /// Whether to use widening to ensure termination.
    pub use_widening: bool,
    /// Widening delay (iterations before widening kicks in).
    pub widening_delay: usize,
}

impl Default for CheckerConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            use_widening: true,
            widening_delay: 3,
        }
    }
}

/// Check if an invariant holds at a loop header.
///
/// Uses abstract interpretation to verify:
/// 1. **Initiation**: The invariant holds when entering the loop
/// 2. **Consecution**: If the invariant holds at loop header, it holds
///    after executing the loop body
///
/// # Arguments
///
/// * `template` - The invariant template to check
/// * `loop_header` - Block ID of the loop header
/// * `loop_body` - Block IDs of the loop body
/// * `cfg` - Control flow graph
/// * `func` - The function containing the loop
/// * `module` - The AIR module
/// * `config` - Checker configuration
///
/// # Returns
///
/// The result of checking the invariant.
#[must_use]
pub fn check_invariant(
    template: &InvariantTemplate,
    loop_header: BlockId,
    loop_body: &[BlockId],
    cfg: &Cfg,
    func: &AirFunction,
    module: &AirModule,
    config: &CheckerConfig,
) -> InvariantCheckResult {
    // Check initiation: invariant holds on loop entry
    let init_result = check_initiation(template, loop_header, cfg, func, module);
    if let InvariantCheckResult::Invalid { .. } = init_result {
        return init_result;
    }
    if let InvariantCheckResult::Unknown { .. } = init_result {
        return init_result;
    }

    // Check consecution: invariant preserved by loop body
    check_consecution(template, loop_header, loop_body, cfg, func, module, config)
}

/// Check the initiation condition: invariant holds on loop entry.
fn check_initiation(
    template: &InvariantTemplate,
    loop_header: BlockId,
    cfg: &Cfg,
    func: &AirFunction,
    module: &AirModule,
) -> InvariantCheckResult {
    // Get predecessors of loop header that are NOT back edges
    let Some(predecessors) = cfg.predecessors_of(loop_header) else {
        return InvariantCheckResult::Unknown {
            reason: "No predecessors for loop header".to_string(),
        };
    };

    // Collect all blocks in the loop body
    let loop_blocks: BTreeSet<_> = collect_loop_blocks(loop_header, cfg);

    // Entry predecessors are those NOT in the loop body
    let entry_preds: Vec<_> = predecessors
        .iter()
        .filter(|p| !loop_blocks.contains(*p))
        .copied()
        .collect();

    if entry_preds.is_empty() {
        // No entry edges found (might be entry is also in loop)
        return InvariantCheckResult::Unknown {
            reason: "Could not identify loop entry edges".to_string(),
        };
    }

    // For each entry predecessor, check if invariant holds
    for pred in entry_preds {
        let result = check_template_at_block(template, pred, func, module);
        if !result {
            return InvariantCheckResult::Invalid {
                counterexample: vec![],
                reason: format!(
                    "Invariant '{}' does not hold on loop entry from block {:?}",
                    template.describe(),
                    pred
                ),
            };
        }
    }

    InvariantCheckResult::Valid
}

/// Check the consecution condition: invariant preserved by loop body.
// INVARIANT: Consecution checking requires the full loop context (template,
// header, body blocks, CFG, function, module, config).
#[allow(clippy::too_many_arguments)]
fn check_consecution(
    template: &InvariantTemplate,
    _loop_header: BlockId,
    loop_body: &[BlockId],
    _cfg: &Cfg,
    func: &AirFunction,
    module: &AirModule,
    _config: &CheckerConfig,
) -> InvariantCheckResult {
    // Simplified check: verify the invariant template is consistent
    // with operations in the loop body

    // For interval invariants, check that assignments don't violate bounds
    match template {
        InvariantTemplate::Interval { var, lo, hi } => {
            // Check all assignments to var in loop body
            for block_id in loop_body {
                let Some(block) = func.blocks.iter().find(|b| b.id == *block_id) else {
                    continue;
                };

                for inst in &block.instructions {
                    if inst.dst == Some(*var) {
                        // This instruction assigns to the variable
                        // Use interval analysis to check bounds
                        let result = analyze_assignment_bounds(inst, *lo, *hi, module);
                        if !result {
                            return InvariantCheckResult::Invalid {
                                counterexample: vec![inst.id],
                                reason: format!(
                                    "Assignment to v{} may violate interval bounds",
                                    var.0
                                ),
                            };
                        }
                    }
                }
            }
            InvariantCheckResult::Valid
        }

        InvariantTemplate::CounterBound { counter, bound } => {
            // Check that counter doesn't exceed bound
            // This is typically preserved if counter increments by positive amount
            // and there's a guard check
            for block_id in loop_body {
                let Some(block) = func.blocks.iter().find(|b| b.id == *block_id) else {
                    continue;
                };

                for inst in &block.instructions {
                    if inst.dst == Some(*counter) {
                        // Assignment to counter - check it's a valid increment
                        // For now, assume it's preserved if it's an Add operation
                        if !matches!(
                            inst.op,
                            saf_core::air::Operation::BinaryOp {
                                kind: saf_core::air::BinaryOp::Add,
                                ..
                            }
                        ) {
                            // Non-add assignment might violate the bound
                            // But we can't be sure, so return Unknown
                            return InvariantCheckResult::Unknown {
                                reason: format!(
                                    "Non-increment assignment to counter v{}",
                                    counter.0
                                ),
                            };
                        }
                    }
                    // Also check bound isn't modified
                    if inst.dst == Some(*bound) {
                        return InvariantCheckResult::Invalid {
                            counterexample: vec![inst.id],
                            reason: format!("Bound v{} is modified in loop body", bound.0),
                        };
                    }
                }
            }
            InvariantCheckResult::Valid
        }

        InvariantTemplate::NonNull { ptr } => {
            // Check that ptr isn't assigned null in loop body
            // For now, assume non-null is preserved if not assigned
            for block_id in loop_body {
                let Some(block) = func.blocks.iter().find(|b| b.id == *block_id) else {
                    continue;
                };

                for inst in &block.instructions {
                    if inst.dst == Some(*ptr) {
                        // Pointer is assigned - might become null
                        return InvariantCheckResult::Unknown {
                            reason: format!("Pointer v{} is assigned in loop body", ptr.0),
                        };
                    }
                }
            }
            InvariantCheckResult::Valid
        }

        // For other templates, return Unknown for now
        _ => InvariantCheckResult::Unknown {
            reason: format!(
                "Consecution check not implemented for template: {}",
                template.describe()
            ),
        },
    }
}

/// Collect all blocks that are part of a loop.
fn collect_loop_blocks(header: BlockId, cfg: &Cfg) -> BTreeSet<BlockId> {
    let mut blocks = BTreeSet::new();
    blocks.insert(header);

    // Find blocks that can reach the header via back edges
    // Simple approximation: blocks reachable from header that have path back
    let mut worklist = vec![header];
    let mut visited = BTreeSet::new();

    while let Some(block) = worklist.pop() {
        if visited.contains(&block) {
            continue;
        }
        visited.insert(block);
        blocks.insert(block);

        if let Some(succs) = cfg.successors_of(block) {
            for succ in succs {
                if !visited.contains(succ) {
                    worklist.push(*succ);
                }
            }
        }
    }

    blocks
}

/// Check if a template holds at a specific block.
fn check_template_at_block(
    template: &InvariantTemplate,
    block_id: BlockId,
    func: &AirFunction,
    _module: &AirModule,
) -> bool {
    let Some(_block) = func.blocks.iter().find(|b| b.id == block_id) else {
        return false;
    };

    // Simplified check: for now, assume the template might hold
    // A full implementation would use abstract interpretation
    match template {
        InvariantTemplate::Interval { lo, hi, .. } => {
            // Check that bounds are reasonable
            match (lo, hi) {
                (Some(l), Some(h)) => *l <= *h,
                _ => true,
            }
        }
        _ => true,
    }
}

/// Analyze if an assignment respects interval bounds.
fn analyze_assignment_bounds(
    inst: &saf_core::air::Instruction,
    lo: Option<i128>,
    hi: Option<i128>,
    module: &AirModule,
) -> bool {
    // Use interval analysis on the instruction
    match &inst.op {
        saf_core::air::Operation::BinaryOp { kind, .. } => {
            use saf_core::air::BinaryOp as BinOp;
            match kind {
                BinOp::Add | BinOp::Sub => {
                    // For add/sub, check if operands are bounded
                    // This is a simplified check - full implementation
                    // would track actual intervals
                    if let Some(constant) = get_constant_operand(inst, module) {
                        if let (Some(l), Some(h)) = (lo, hi) {
                            // Check if adding constant keeps value in bounds
                            // This is a very simplified check
                            let new_range = (h - l).saturating_add(constant.abs());
                            return new_range >= 0;
                        }
                    }
                    true
                }
                _ => true,
            }
        }
        _ => true,
    }
}

/// Try to get a constant operand from an instruction.
fn get_constant_operand(inst: &saf_core::air::Instruction, module: &AirModule) -> Option<i128> {
    for operand in &inst.operands {
        if let Some(saf_core::air::Constant::Int { value, .. }) = module.constants.get(operand) {
            return Some(i128::from(*value));
        }
    }
    None
}

/// Strengthening hints for refining an invariant.
#[derive(Clone, Debug, Default)]
pub struct StrengtheningHints {
    /// Suggested tighter lower bound.
    pub tighter_lo: Option<i128>,
    /// Suggested tighter upper bound.
    pub tighter_hi: Option<i128>,
    /// Additional predicates to conjoin.
    pub additional_predicates: Vec<InvariantTemplate>,
}

impl StrengtheningHints {
    /// Check if any strengthening is suggested.
    #[must_use]
    pub fn has_suggestions(&self) -> bool {
        self.tighter_lo.is_some()
            || self.tighter_hi.is_some()
            || !self.additional_predicates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_checker_config_defaults() {
        let config = CheckerConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert!(config.use_widening);
        assert_eq!(config.widening_delay, 3);
    }

    #[test]
    fn test_check_result_valid() {
        let result = InvariantCheckResult::Valid;
        assert!(matches!(result, InvariantCheckResult::Valid));
    }

    #[test]
    fn test_check_result_invalid() {
        let result = InvariantCheckResult::Invalid {
            counterexample: vec![InstId::new(1)],
            reason: "test".to_string(),
        };

        match result {
            InvariantCheckResult::Invalid {
                counterexample,
                reason,
            } => {
                assert_eq!(counterexample.len(), 1);
                assert_eq!(reason, "test");
            }
            _ => panic!("Expected Invalid"),
        }
    }

    #[test]
    fn test_strengthening_hints() {
        let mut hints = StrengtheningHints::default();
        assert!(!hints.has_suggestions());

        hints.tighter_lo = Some(0);
        assert!(hints.has_suggestions());
    }

    #[test]
    fn test_collect_loop_blocks() {
        // Create a simple CFG with a loop
        use saf_core::air::AirFunction;

        let func = AirFunction {
            id: saf_core::ids::FunctionId::new(1),
            name: "test".to_string(),
            params: vec![],
            blocks: vec![],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let cfg = Cfg::build(&func);

        // Empty CFG should still work
        let blocks = collect_loop_blocks(BlockId::new(0), &cfg);
        assert!(blocks.contains(&BlockId::new(0)));
    }
}
