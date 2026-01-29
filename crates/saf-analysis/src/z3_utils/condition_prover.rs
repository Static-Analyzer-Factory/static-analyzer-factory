//! Condition prover for `svf_assert(condition)` style assertions.
//!
//! Unlike `prove_assertions` which checks reachability to `__assert_fail` calls,
//! this module evaluates direct condition assertions where the condition argument
//! should be provably TRUE.
//!
//! # PTABen `svf_assert` Semantics
//!
//! ```c
//! svf_assert(y >= 7);  // Assertion: y >= 7 must be true at this point
//! ```
//!
//! The analyzer should prove that the condition is always satisfied by:
//! 1. Finding the comparison instruction that computes the condition
//! 2. Using abstract interpretation to get bounds on the operands
//! 3. Evaluating if the comparison is always true given those bounds

use std::collections::BTreeMap;

use saf_core::air::{AirFunction, AirModule, BinaryOp, CastKind, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};
use serde::{Deserialize, Serialize};

use crate::absint::{AbstractDomain, AbstractInterpResult, InterproceduralResult, Interval};
use crate::defuse::DefUseGraph;

// ---------------------------------------------------------------------------
// Interval Query Trait
// ---------------------------------------------------------------------------

/// Trait for querying interval information from analysis results.
///
/// This allows condition proving to work with both intraprocedural
/// (`AbstractInterpResult`) and interprocedural (`InterproceduralResult`) analyses.
pub trait IntervalQuery {
    /// Get the interval for a value at an instruction point.
    fn interval_at_inst(&self, inst: InstId, value: ValueId, bits: u8) -> Interval;

    /// Access the constant map.
    fn constant_map(&self) -> &BTreeMap<ValueId, Interval>;

    /// Check if a block was reached during abstract interpretation.
    ///
    /// Returns `false` if the block's entry state is bottom (unreachable).
    /// Used by the phi handler to skip incoming values from unreachable predecessors,
    /// e.g. short-circuit `&&` patterns where one branch is never taken.
    fn is_block_reachable(&self, _block_id: BlockId) -> bool {
        true // default: conservatively assume all blocks are reachable
    }
}

impl IntervalQuery for AbstractInterpResult {
    fn interval_at_inst(&self, inst: InstId, value: ValueId, bits: u8) -> Interval {
        AbstractInterpResult::interval_at_inst(self, inst, value, bits)
    }

    fn constant_map(&self) -> &BTreeMap<ValueId, Interval> {
        AbstractInterpResult::constant_map(self)
    }

    fn is_block_reachable(&self, block_id: BlockId) -> bool {
        self.block_states()
            .get(&block_id)
            .is_some_and(|s| !s.is_unreachable())
    }
}

impl IntervalQuery for InterproceduralResult {
    fn interval_at_inst(&self, inst: InstId, value: ValueId, bits: u8) -> Interval {
        InterproceduralResult::interval_at_inst(self, inst, value, bits)
    }

    fn constant_map(&self) -> &BTreeMap<ValueId, Interval> {
        InterproceduralResult::constant_map(self)
    }

    fn is_block_reachable(&self, block_id: BlockId) -> bool {
        self.intraprocedural()
            .block_states()
            .get(&block_id)
            .is_some_and(|s| !s.is_unreachable())
    }
}

/// Maximum number of hops to trace through operations to find a comparison.
const MAX_TRACE_DEPTH: usize = 10;

/// Maximum recursion depth for Phi node evaluation.
const MAX_PHI_DEPTH: usize = 5;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Status of a condition proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionStatus {
    /// Condition is provably always true.
    Proven,
    /// Condition may be false (counterexample possible).
    MayFail,
    /// Unable to determine (e.g., condition definition not found).
    Unknown,
}

/// A finding about an `svf_assert` condition.
#[derive(Debug, Clone)]
pub struct ConditionFinding {
    /// The function containing the assertion.
    pub function: String,
    /// The function ID.
    pub function_id: FunctionId,
    /// The instruction ID of the svf_assert call.
    pub inst: InstId,
    /// Human-readable description of the condition.
    pub condition_desc: String,
    /// Proof status.
    pub status: ConditionStatus,
    /// Interval bounds used in the proof (for debugging).
    pub interval_info: Option<String>,
}

/// Result of condition proving.
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// Conditions proven to always hold.
    pub proven: Vec<ConditionFinding>,
    /// Conditions that may fail.
    pub may_fail: Vec<ConditionFinding>,
    /// Conditions where analysis couldn't determine status.
    pub unknown: Vec<ConditionFinding>,
    /// Diagnostics.
    pub diagnostics: ConditionDiagnostics,
}

/// Diagnostics from condition proving.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConditionDiagnostics {
    /// Total svf_assert calls found.
    pub total_assertions: usize,
    /// Proven count.
    pub proven_count: usize,
    /// May-fail count.
    pub may_fail_count: usize,
    /// Unknown count.
    pub unknown_count: usize,
    /// Conditions found without comparison definition.
    pub missing_definitions: usize,
}

// ---------------------------------------------------------------------------
// Condition prover
// ---------------------------------------------------------------------------

/// Prove or disprove `svf_assert(condition)` assertions.
///
/// Scans for calls to the specified assertion function, extracts the condition
/// operand, finds the comparison instruction that defines it, and uses abstract
/// interpretation intervals to evaluate the comparison.
///
/// # Arguments
///
/// * `module` - The AIR module to analyze
/// * `absint_result` - Abstract interpretation result with interval bounds
/// * `assert_func_name` - The assertion function name (e.g., "svf_assert")
pub fn prove_conditions(
    module: &AirModule,
    absint_result: &AbstractInterpResult,
    assert_func_name: &str,
) -> ConditionResult {
    prove_conditions_generic(module, absint_result, assert_func_name)
}

/// Prove conditions using interprocedural analysis results.
///
/// This uses refined call site states from interprocedural analysis,
/// providing better precision for assertions involving function return values.
pub fn prove_conditions_interprocedural(
    module: &AirModule,
    interprocedural_result: &InterproceduralResult,
    assert_func_name: &str,
) -> ConditionResult {
    prove_conditions_generic(module, interprocedural_result, assert_func_name)
}

/// Generic condition prover that works with any interval query source.
fn prove_conditions_generic<Q: IntervalQuery>(
    module: &AirModule,
    interval_source: &Q,
    assert_func_name: &str,
) -> ConditionResult {
    // Build def-use graph to find condition definitions
    let defuse = DefUseGraph::build(module);

    // Build instruction lookup for finding comparison operations
    let inst_map = build_inst_map(module);

    // Find the assertion function ID
    let assert_func_id: Option<FunctionId> = module
        .functions
        .iter()
        .find(|f| f.name == assert_func_name)
        .map(|f| f.id);

    let Some(assert_func_id) = assert_func_id else {
        return ConditionResult {
            proven: Vec::new(),
            may_fail: Vec::new(),
            unknown: Vec::new(),
            diagnostics: ConditionDiagnostics::default(),
        };
    };

    let mut proven = Vec::new();
    let mut may_fail = Vec::new();
    let mut unknown = Vec::new();
    let mut diagnostics = ConditionDiagnostics::default();

    // Scan for svf_assert calls
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                // Check if this is a call to svf_assert
                let is_assert = match &inst.op {
                    Operation::CallDirect { callee } => *callee == assert_func_id,
                    _ => false,
                };

                if !is_assert {
                    continue;
                }

                diagnostics.total_assertions += 1;

                // Get the condition operand (first argument)
                let Some(&condition_value) = inst.operands.first() else {
                    unknown.push(ConditionFinding {
                        function: func.name.clone(),
                        function_id: func.id,
                        inst: inst.id,
                        condition_desc: "no condition operand".to_string(),
                        status: ConditionStatus::Unknown,
                        interval_info: None,
                    });
                    diagnostics.unknown_count += 1;
                    continue;
                };

                // Build block terminator map for this function
                let terminator_map = build_block_terminator_map(func);

                // Try to find the comparison that defines this condition
                let (status, desc, interval_info) = evaluate_condition_with_cfg(
                    condition_value,
                    inst.id,
                    block.id,
                    &defuse,
                    &inst_map,
                    interval_source,
                    &terminator_map,
                    &mut diagnostics,
                );

                let finding = ConditionFinding {
                    function: func.name.clone(),
                    function_id: func.id,
                    inst: inst.id,
                    condition_desc: desc,
                    status: status.clone(),
                    interval_info,
                };

                match status {
                    ConditionStatus::Proven => {
                        proven.push(finding);
                        diagnostics.proven_count += 1;
                    }
                    ConditionStatus::MayFail => {
                        may_fail.push(finding);
                        diagnostics.may_fail_count += 1;
                    }
                    ConditionStatus::Unknown => {
                        unknown.push(finding);
                        diagnostics.unknown_count += 1;
                    }
                }
            }
        }
    }

    ConditionResult {
        proven,
        may_fail,
        unknown,
        diagnostics,
    }
}

/// Information about a block's terminator instruction.
#[derive(Debug, Clone)]
struct BlockTerminator {
    /// The condition value for conditional branches (if any).
    condition: Option<ValueId>,
    /// The "then" target block for conditional branches.
    #[allow(dead_code)] // Retained for structural completeness with else_target
    then_target: Option<BlockId>,
    /// The "else" target block for conditional branches.
    else_target: Option<BlockId>,
    /// The CondBr instruction ID (for interval lookups at branch point).
    cond_inst_id: Option<InstId>,
}

/// Build a map from block ID to its terminator info for a function.
fn build_block_terminator_map(func: &AirFunction) -> BTreeMap<BlockId, BlockTerminator> {
    let mut map = BTreeMap::new();

    for block in &func.blocks {
        // Find the terminator (last instruction that branches)
        for inst in &block.instructions {
            match &inst.op {
                Operation::CondBr {
                    then_target,
                    else_target,
                } => {
                    let condition = inst.operands.first().copied();
                    map.insert(
                        block.id,
                        BlockTerminator {
                            condition,
                            then_target: Some(*then_target),
                            else_target: Some(*else_target),
                            cond_inst_id: Some(inst.id),
                        },
                    );
                }
                Operation::Br { target: _ }
                | Operation::Ret
                | Operation::Switch { .. }
                | Operation::Unreachable => {
                    map.insert(
                        block.id,
                        BlockTerminator {
                            condition: None,
                            then_target: None,
                            else_target: None,
                            cond_inst_id: None,
                        },
                    );
                }
                _ => {}
            }
        }
    }

    map
}

/// Build a map from instruction ID to instruction reference.
fn build_inst_map(module: &AirModule) -> BTreeMap<InstId, &Instruction> {
    let mut map = BTreeMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                map.insert(inst.id, inst);
            }
        }
    }
    map
}

/// Evaluate a condition value to determine if it's always true.
///
/// Traces through Cast, Copy, and Phi operations to find underlying comparisons.
#[allow(dead_code)]
fn evaluate_condition<Q: IntervalQuery>(
    condition_value: ValueId,
    assert_inst: InstId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    diagnostics: &mut ConditionDiagnostics,
) -> (ConditionStatus, String, Option<String>) {
    evaluate_condition_recursive(
        condition_value,
        assert_inst,
        defuse,
        inst_map,
        interval_source,
        diagnostics,
        0, // phi_depth
    )
}

/// Evaluate a condition value with CFG information for guard-aware Phi handling.
///
/// This version can prove short-circuit AND/OR patterns by checking if the
/// guard condition for constant false paths is always true.
// INVARIANT: CFG-aware condition evaluation requires block context, def-use graph,
// instruction map, interval source, terminator map, and diagnostics accumulator.
#[allow(clippy::too_many_arguments)]
fn evaluate_condition_with_cfg<Q: IntervalQuery>(
    condition_value: ValueId,
    assert_inst: InstId,
    current_block: BlockId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    terminator_map: &BTreeMap<BlockId, BlockTerminator>,
    diagnostics: &mut ConditionDiagnostics,
) -> (ConditionStatus, String, Option<String>) {
    evaluate_condition_with_cfg_recursive(
        condition_value,
        assert_inst,
        current_block,
        defuse,
        inst_map,
        interval_source,
        terminator_map,
        diagnostics,
        0, // phi_depth
    )
}

/// Recursive helper for condition evaluation with Phi depth tracking.
// NOTE: This function implements the condition evaluation algorithm as a single
// cohesive unit, handling casts, comparisons, and Phi nodes. Splitting would
// obscure the recursive traversal logic.
#[allow(clippy::too_many_lines)]
fn evaluate_condition_recursive<Q: IntervalQuery>(
    condition_value: ValueId,
    assert_inst: InstId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    // Trace through casts/copies to find the underlying comparison
    let mut current_value = condition_value;
    let mut depth = 0;

    loop {
        if depth >= MAX_TRACE_DEPTH {
            return (
                ConditionStatus::Unknown,
                "exceeded max trace depth".to_string(),
                None,
            );
        }

        // Find the definition of the current value
        let Some(Some(def_inst_id)) = defuse.defs.get(&current_value) else {
            // Check if this is a constant (no definition because it's a literal)
            // For boolean conditions, constant 0 = false, nonzero = true
            if let Some(constant_interval) = interval_source.constant_map().get(&current_value) {
                // Check if constant is definitely true or false
                if constant_interval.lo() == constant_interval.hi() {
                    let value = constant_interval.lo();
                    if value == 0 {
                        // Constant false - this path definitely fails
                        return (
                            ConditionStatus::MayFail,
                            "constant false (value=0)".to_string(),
                            Some(format!("constant: {constant_interval:?}")),
                        );
                    }
                    // Constant nonzero/true - this path is proven
                    return (
                        ConditionStatus::Proven,
                        format!("constant true (value={value})"),
                        Some(format!("constant: {constant_interval:?}")),
                    );
                }
            }

            diagnostics.missing_definitions += 1;
            return (
                ConditionStatus::Unknown,
                "condition definition not found".to_string(),
                None,
            );
        };
        let def_inst_id = *def_inst_id;

        // Get the defining instruction
        let Some(def_inst) = inst_map.get(&def_inst_id) else {
            return (
                ConditionStatus::Unknown,
                "definition instruction not in map".to_string(),
                None,
            );
        };

        match &def_inst.op {
            // Found a comparison - evaluate it
            Operation::BinaryOp { kind } => {
                return evaluate_comparison(
                    *kind,
                    &def_inst.operands,
                    assert_inst,
                    def_inst_id,
                    interval_source,
                );
            }

            // Handle Phi nodes by evaluating all incoming values
            Operation::Phi { incoming } => {
                return evaluate_phi_condition(
                    incoming,
                    assert_inst,
                    defuse,
                    inst_map,
                    interval_source,
                    diagnostics,
                    phi_depth,
                );
            }

            // Handle Select (ternary) - both branches must be provable
            Operation::Select => {
                if def_inst.operands.len() >= 3 {
                    let true_val = def_inst.operands[1];
                    let false_val = def_inst.operands[2];
                    return evaluate_select_condition(
                        true_val,
                        false_val,
                        assert_inst,
                        defuse,
                        inst_map,
                        interval_source,
                        diagnostics,
                        phi_depth,
                    );
                }
            }

            // Trace through Cast operations (ZExt, SExt, etc.)
            Operation::Cast { kind, .. } => {
                // ZExt/SExt of a boolean preserves truth value
                if matches!(kind, CastKind::ZExt | CastKind::SExt) {
                    if let Some(&src_value) = def_inst.operands.first() {
                        current_value = src_value;
                        depth += 1;
                        continue;
                    }
                }
                // Fall through to interval check for other casts
            }

            // Trace through Copy operations
            Operation::Copy => {
                if let Some(&src_value) = def_inst.operands.first() {
                    current_value = src_value;
                    depth += 1;
                    continue;
                }
            }

            // For other operations, check the interval
            _ => {}
        }

        // Try multi-width interval lookup for the condition value
        let cond_interval = best_interval(interval_source, assert_inst, condition_value);

        if cond_interval.is_bottom() {
            return (
                ConditionStatus::Unknown,
                format!("condition from non-comparison op: {:?}", def_inst.op),
                None,
            );
        }

        // If the interval is [1,1] or more generally [n,n] where n > 0, condition is true
        if cond_interval.lo() == cond_interval.hi() && cond_interval.lo() > 0 {
            return (
                ConditionStatus::Proven,
                format!("condition from {:?} is always nonzero", def_inst.op),
                Some(format!("cond_interval: {cond_interval:?}")),
            );
        }
        if cond_interval.hi() == 0 {
            return (
                ConditionStatus::MayFail,
                format!("condition from {:?} is always 0", def_inst.op),
                Some(format!("cond_interval: {cond_interval:?}")),
            );
        }
        return (
            ConditionStatus::Unknown,
            format!("condition from non-comparison: {:?}", def_inst.op),
            Some(format!("cond_interval: {cond_interval:?}")),
        );
    }
}

/// CFG-aware recursive helper for condition evaluation.
///
/// This version passes terminator information through to Phi evaluation,
/// enabling guard-aware handling of short-circuit AND/OR patterns.
// NOTE: This function mirrors `evaluate_condition_recursive` with additional
// CFG context for guard-aware Phi handling. Splitting would fragment the
// recursive condition traversal logic.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn evaluate_condition_with_cfg_recursive<Q: IntervalQuery>(
    condition_value: ValueId,
    assert_inst: InstId,
    current_block: BlockId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    terminator_map: &BTreeMap<BlockId, BlockTerminator>,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    // Trace through casts/copies to find the underlying comparison
    let mut current_value = condition_value;
    let mut depth = 0;

    loop {
        if depth >= MAX_TRACE_DEPTH {
            return (
                ConditionStatus::Unknown,
                "exceeded max trace depth".to_string(),
                None,
            );
        }

        // Find the definition of the current value
        let Some(Some(def_inst_id)) = defuse.defs.get(&current_value) else {
            // Check if this is a constant (no definition because it's a literal)
            if let Some(constant_interval) = interval_source.constant_map().get(&current_value) {
                if constant_interval.lo() == constant_interval.hi() {
                    let value = constant_interval.lo();
                    if value == 0 {
                        return (
                            ConditionStatus::MayFail,
                            "constant false (value=0)".to_string(),
                            Some(format!("constant: {constant_interval:?}")),
                        );
                    }
                    return (
                        ConditionStatus::Proven,
                        format!("constant true (value={value})"),
                        Some(format!("constant: {constant_interval:?}")),
                    );
                }
            }

            diagnostics.missing_definitions += 1;
            return (
                ConditionStatus::Unknown,
                "condition definition not found".to_string(),
                None,
            );
        };
        let def_inst_id = *def_inst_id;

        // Get the defining instruction
        let Some(def_inst) = inst_map.get(&def_inst_id) else {
            return (
                ConditionStatus::Unknown,
                "definition instruction not in map".to_string(),
                None,
            );
        };

        match &def_inst.op {
            // Found a comparison - evaluate it
            Operation::BinaryOp { kind } => {
                return evaluate_comparison(
                    *kind,
                    &def_inst.operands,
                    assert_inst,
                    def_inst_id,
                    interval_source,
                );
            }

            // Handle Phi nodes with guard-aware evaluation
            Operation::Phi { incoming } => {
                return evaluate_phi_condition_with_cfg(
                    incoming,
                    assert_inst,
                    current_block,
                    defuse,
                    inst_map,
                    interval_source,
                    terminator_map,
                    diagnostics,
                    phi_depth,
                );
            }

            // Handle Select (ternary)
            Operation::Select => {
                if def_inst.operands.len() >= 3 {
                    let true_val = def_inst.operands[1];
                    let false_val = def_inst.operands[2];
                    return evaluate_select_condition_with_cfg(
                        true_val,
                        false_val,
                        assert_inst,
                        current_block,
                        defuse,
                        inst_map,
                        interval_source,
                        terminator_map,
                        diagnostics,
                        phi_depth,
                    );
                }
            }

            // Trace through Cast operations
            Operation::Cast { kind, .. } => {
                if matches!(kind, CastKind::ZExt | CastKind::SExt) {
                    if let Some(&src_value) = def_inst.operands.first() {
                        current_value = src_value;
                        depth += 1;
                        continue;
                    }
                }
            }

            // Trace through Copy operations
            Operation::Copy => {
                if let Some(&src_value) = def_inst.operands.first() {
                    current_value = src_value;
                    depth += 1;
                    continue;
                }
            }

            _ => {}
        }

        // Fallback: multi-width interval lookup
        let cond_interval = best_interval(interval_source, assert_inst, condition_value);

        if cond_interval.is_bottom() {
            return (
                ConditionStatus::Unknown,
                format!("condition from non-comparison op: {:?}", def_inst.op),
                None,
            );
        }

        if cond_interval.lo() == cond_interval.hi() && cond_interval.lo() > 0 {
            return (
                ConditionStatus::Proven,
                format!("condition from {:?} is always nonzero", def_inst.op),
                Some(format!("cond_interval: {cond_interval:?}")),
            );
        }
        if cond_interval.hi() == 0 {
            return (
                ConditionStatus::MayFail,
                format!("condition from {:?} is always 0", def_inst.op),
                Some(format!("cond_interval: {cond_interval:?}")),
            );
        }
        return (
            ConditionStatus::Unknown,
            format!("condition from non-comparison: {:?}", def_inst.op),
            Some(format!("cond_interval: {cond_interval:?}")),
        );
    }
}

/// Evaluate a Phi node condition by checking all incoming values.
///
/// For a condition to be proven via Phi:
/// - ALL incoming paths must prove the condition → Proven
/// - ANY incoming path definitely fails → MayFail
/// - Otherwise → Unknown
// INVARIANT: Phi evaluation requires incoming edges, assertion point, def-use graph,
// instruction map, interval source, diagnostics, and recursion depth tracker.
#[allow(clippy::too_many_arguments)]
fn evaluate_phi_condition<Q: IntervalQuery>(
    incoming: &[(saf_core::ids::BlockId, ValueId)],
    assert_inst: InstId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    // Guard against infinite recursion through nested Phi nodes
    if phi_depth >= MAX_PHI_DEPTH {
        return (
            ConditionStatus::Unknown,
            "exceeded max Phi recursion depth".to_string(),
            None,
        );
    }

    // Empty incoming list - can't evaluate
    if incoming.is_empty() {
        return (
            ConditionStatus::Unknown,
            "Phi has no incoming values".to_string(),
            None,
        );
    }

    let mut all_proven = true;
    let mut any_may_fail = false;
    let mut descriptions = Vec::new();
    let mut reachable_count = 0;

    for (block_id, value_id) in incoming {
        // Skip phi incoming from unreachable predecessors.
        // This handles short-circuit patterns like `&&` where one branch
        // is never taken — the `false` constant from the unreachable
        // predecessor should not prevent proving the phi condition.
        if !interval_source.is_block_reachable(*block_id) {
            descriptions.push("unreachable predecessor (skipped)".to_string());
            continue;
        }
        reachable_count += 1;

        let (status, desc, _interval_info) = evaluate_condition_recursive(
            *value_id,
            assert_inst,
            defuse,
            inst_map,
            interval_source,
            diagnostics,
            phi_depth + 1,
        );

        descriptions.push(desc.clone());

        match status {
            ConditionStatus::Proven => {
                // This path is proven, continue checking others
            }
            ConditionStatus::MayFail => {
                any_may_fail = true;
                all_proven = false;
            }
            ConditionStatus::Unknown => {
                all_proven = false;
            }
        }
    }

    // If all incoming paths were unreachable, the phi itself is unreachable
    if reachable_count == 0 {
        return (
            ConditionStatus::Unknown,
            "all Phi predecessors unreachable".to_string(),
            None,
        );
    }

    let combined_desc = format!(
        "Phi with {} incoming ({} reachable): [{}]",
        incoming.len(),
        reachable_count,
        descriptions.join(", ")
    );

    if all_proven {
        (
            ConditionStatus::Proven,
            format!("all Phi paths proven: {combined_desc}"),
            None,
        )
    } else if any_may_fail {
        (
            ConditionStatus::MayFail,
            format!("Phi path may fail: {combined_desc}"),
            None,
        )
    } else {
        (
            ConditionStatus::Unknown,
            format!("Phi has unknown paths: {combined_desc}"),
            None,
        )
    }
}

/// Evaluate a Select (ternary) condition by checking both branches.
///
/// For a condition to be proven via Select:
/// - BOTH true and false branches must prove → Proven
/// - EITHER branch definitely fails → MayFail
/// - Otherwise → Unknown
// INVARIANT: Select evaluation requires both branch values, assertion point,
// def-use graph, instruction map, interval source, diagnostics, and depth tracker.
#[allow(clippy::too_many_arguments)]
fn evaluate_select_condition<Q: IntervalQuery>(
    true_val: ValueId,
    false_val: ValueId,
    assert_inst: InstId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    // Guard against infinite recursion
    if phi_depth >= MAX_PHI_DEPTH {
        return (
            ConditionStatus::Unknown,
            "exceeded max recursion depth in Select".to_string(),
            None,
        );
    }

    let (true_status, true_desc, _) = evaluate_condition_recursive(
        true_val,
        assert_inst,
        defuse,
        inst_map,
        interval_source,
        diagnostics,
        phi_depth + 1,
    );

    let (false_status, false_desc, _) = evaluate_condition_recursive(
        false_val,
        assert_inst,
        defuse,
        inst_map,
        interval_source,
        diagnostics,
        phi_depth + 1,
    );

    let combined_desc = format!("Select(true: {true_desc}, false: {false_desc})");

    match (true_status, false_status) {
        (ConditionStatus::Proven, ConditionStatus::Proven) => (
            ConditionStatus::Proven,
            format!("both Select branches proven: {combined_desc}"),
            None,
        ),
        (ConditionStatus::MayFail, _) | (_, ConditionStatus::MayFail) => (
            ConditionStatus::MayFail,
            format!("Select branch may fail: {combined_desc}"),
            None,
        ),
        _ => (
            ConditionStatus::Unknown,
            format!("Select has unknown branch: {combined_desc}"),
            None,
        ),
    }
}

/// Guard-aware Phi condition evaluation.
///
/// For short-circuit AND patterns like `phi [false, pred_A], [result, pred_B]`:
/// - If `result` is Proven AND the guard leading to `false` (pred_A) is provably
///   always true, then the false path is unreachable and the whole Phi is Proven.
// NOTE: This function implements guard-aware Phi evaluation with CFG analysis.
// Splitting would fragment the short-circuit pattern detection logic.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn evaluate_phi_condition_with_cfg<Q: IntervalQuery>(
    incoming: &[(BlockId, ValueId)],
    assert_inst: InstId,
    phi_block: BlockId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    terminator_map: &BTreeMap<BlockId, BlockTerminator>,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    // Guard against infinite recursion
    if phi_depth >= MAX_PHI_DEPTH {
        return (
            ConditionStatus::Unknown,
            "exceeded max Phi recursion depth".to_string(),
            None,
        );
    }

    if incoming.is_empty() {
        return (
            ConditionStatus::Unknown,
            "Phi has no incoming values".to_string(),
            None,
        );
    }

    // First pass: evaluate all incoming values
    let mut path_results: Vec<(BlockId, ValueId, ConditionStatus, String)> = Vec::new();

    for (block_id, value_id) in incoming {
        let (status, desc, _) = evaluate_condition_with_cfg_recursive(
            *value_id,
            assert_inst,
            phi_block,
            defuse,
            inst_map,
            interval_source,
            terminator_map,
            diagnostics,
            phi_depth + 1,
        );
        path_results.push((*block_id, *value_id, status, desc));
    }

    // Check for short-circuit AND pattern:
    // - Some paths produce constant false (from short-circuit)
    // - Other paths produce the actual condition result
    let false_paths: Vec<_> = path_results
        .iter()
        .filter(|(_, value_id, status, _)| {
            if *status != ConditionStatus::MayFail {
                return false;
            }
            // Semantic check: is this value a constant zero?
            interval_source
                .constant_map()
                .get(value_id)
                .is_some_and(|c| c.is_singleton() && c.lo() == 0)
        })
        .collect();

    let other_paths: Vec<_> = path_results
        .iter()
        .filter(|(_, value_id, status, _)| {
            if *status != ConditionStatus::MayFail {
                return true;
            }
            // Keep paths that are NOT constant zero
            !interval_source
                .constant_map()
                .get(value_id)
                .is_some_and(|c| c.is_singleton() && c.lo() == 0)
        })
        .collect();

    // If we have false paths and other paths, check if it's a short-circuit AND pattern
    if !false_paths.is_empty() && !other_paths.is_empty() {
        // Check if all non-false paths are proven
        let all_other_proven = other_paths
            .iter()
            .all(|(_, _, status, _)| *status == ConditionStatus::Proven);

        if all_other_proven {
            // For each false path, check if the guard condition is provably always true
            // (which would make the false path unreachable)
            let mut all_guards_proven = true;

            for (false_pred_block, _, _, _) in &false_paths {
                // Check if the predecessor has a CondBr and the false path is the else branch
                if let Some(terminator) = terminator_map.get(false_pred_block) {
                    if let (Some(cond_value), Some(else_target)) =
                        (terminator.condition, terminator.else_target)
                    {
                        // The false path comes from this predecessor when the condition is FALSE
                        // (i.e., we took the else branch to the phi_block)
                        if else_target == phi_block {
                            // The false path is taken when cond_value is FALSE
                            // So to prove the false path is unreachable, we need cond_value always TRUE

                            // Fast path: check the fixpoint's i1 interval for the guard condition
                            if let Some(cond_inst) = terminator.cond_inst_id {
                                let guard_iv =
                                    best_interval(interval_source, cond_inst, cond_value);
                                if guard_iv.lo() >= 1 {
                                    // Guard always true → else branch (false path) unreachable
                                    continue;
                                }
                            }

                            let (guard_status, _, _) = evaluate_condition_with_cfg_recursive(
                                cond_value,
                                assert_inst,
                                phi_block,
                                defuse,
                                inst_map,
                                interval_source,
                                terminator_map,
                                diagnostics,
                                phi_depth + 1,
                            );

                            if guard_status != ConditionStatus::Proven {
                                all_guards_proven = false;
                            }
                        } else {
                            // The false path might be from the then branch (for OR pattern)
                            // or unconditional - can't prove guard
                            all_guards_proven = false;
                        }
                    } else {
                        // No conditional branch - can't prove guard
                        all_guards_proven = false;
                    }
                } else {
                    all_guards_proven = false;
                }
            }

            if all_guards_proven {
                let descriptions: Vec<_> = path_results
                    .iter()
                    .map(|(_, _, _, desc)| desc.clone())
                    .collect();
                return (
                    ConditionStatus::Proven,
                    format!(
                        "short-circuit AND proven: all non-false paths proven and guards proven: [{}]",
                        descriptions.join(", ")
                    ),
                    None,
                );
            }
        }
    }

    // Check for short-circuit OR pattern:
    // phi [true, pred_A], [result, pred_B]
    // If all non-true paths are Proven, the phi is Proven.
    {
        let true_paths: Vec<_> = path_results
            .iter()
            .filter(|(_, value_id, _, _)| {
                interval_source
                    .constant_map()
                    .get(value_id)
                    .is_some_and(|c| c.is_singleton() && c.lo() != 0)
            })
            .collect();

        if !true_paths.is_empty() {
            let non_true_paths: Vec<_> = path_results
                .iter()
                .filter(|(_, value_id, _, _)| {
                    !interval_source
                        .constant_map()
                        .get(value_id)
                        .is_some_and(|c| c.is_singleton() && c.lo() != 0)
                })
                .collect();

            let all_non_true_proven = non_true_paths
                .iter()
                .all(|(_, _, status, _)| *status == ConditionStatus::Proven);

            if all_non_true_proven {
                let descriptions: Vec<_> = path_results
                    .iter()
                    .map(|(_, _, _, desc)| desc.clone())
                    .collect();
                return (
                    ConditionStatus::Proven,
                    format!(
                        "short-circuit OR proven: all non-true paths proven: [{}]",
                        descriptions.join(", ")
                    ),
                    None,
                );
            }
        }
    }

    // Fall back to standard evaluation
    let mut all_proven = true;
    let mut any_may_fail = false;
    let mut descriptions = Vec::new();

    for (_, _, status, desc) in &path_results {
        descriptions.push(desc.clone());
        match status {
            ConditionStatus::Proven => {}
            ConditionStatus::MayFail => {
                any_may_fail = true;
                all_proven = false;
            }
            ConditionStatus::Unknown => {
                all_proven = false;
            }
        }
    }

    let combined_desc = format!(
        "Phi with {} incoming: [{}]",
        incoming.len(),
        descriptions.join(", ")
    );

    if all_proven {
        (
            ConditionStatus::Proven,
            format!("all Phi paths proven: {combined_desc}"),
            None,
        )
    } else if any_may_fail {
        (
            ConditionStatus::MayFail,
            format!("Phi path may fail: {combined_desc}"),
            None,
        )
    } else {
        (
            ConditionStatus::Unknown,
            format!("Phi has unknown paths: {combined_desc}"),
            None,
        )
    }
}

/// Guard-aware Select condition evaluation.
// INVARIANT: CFG-aware select evaluation requires both branch values, assertion
// point, block context, def-use graph, instruction map, interval source,
// terminator map, diagnostics, and depth tracker.
#[allow(clippy::too_many_arguments)]
fn evaluate_select_condition_with_cfg<Q: IntervalQuery>(
    true_val: ValueId,
    false_val: ValueId,
    assert_inst: InstId,
    current_block: BlockId,
    defuse: &DefUseGraph,
    inst_map: &BTreeMap<InstId, &Instruction>,
    interval_source: &Q,
    terminator_map: &BTreeMap<BlockId, BlockTerminator>,
    diagnostics: &mut ConditionDiagnostics,
    phi_depth: usize,
) -> (ConditionStatus, String, Option<String>) {
    if phi_depth >= MAX_PHI_DEPTH {
        return (
            ConditionStatus::Unknown,
            "exceeded max recursion depth in Select".to_string(),
            None,
        );
    }

    let (true_status, true_desc, _) = evaluate_condition_with_cfg_recursive(
        true_val,
        assert_inst,
        current_block,
        defuse,
        inst_map,
        interval_source,
        terminator_map,
        diagnostics,
        phi_depth + 1,
    );

    let (false_status, false_desc, _) = evaluate_condition_with_cfg_recursive(
        false_val,
        assert_inst,
        current_block,
        defuse,
        inst_map,
        interval_source,
        terminator_map,
        diagnostics,
        phi_depth + 1,
    );

    let combined_desc = format!("Select(true: {true_desc}, false: {false_desc})");

    match (true_status, false_status) {
        (ConditionStatus::Proven, ConditionStatus::Proven) => (
            ConditionStatus::Proven,
            format!("both Select branches proven: {combined_desc}"),
            None,
        ),
        (ConditionStatus::MayFail, _) | (_, ConditionStatus::MayFail) => (
            ConditionStatus::MayFail,
            format!("Select branch may fail: {combined_desc}"),
            None,
        ),
        _ => (
            ConditionStatus::Unknown,
            format!("Select has unknown branch: {combined_desc}"),
            None,
        ),
    }
}

/// Find the best (most precise) interval for a value across common bit widths.
///
/// The abstract interpreter may store intervals at various bit widths (64, 32, 8, 1)
/// depending on the LLVM types involved. This function tries common widths from
/// widest to narrowest and returns the **widest** precise (non-top, non-bottom)
/// interval found. This prevents narrow-width lookups (e.g., 1-bit boolean)
/// from shadowing the correct wider computation result (e.g., 32-bit `[256,256]`
/// from `zext i8 255 + 1`).
fn best_interval<Q: IntervalQuery>(source: &Q, inst: InstId, value: ValueId) -> Interval {
    // Try common bit widths from widest to narrowest.
    // Return the WIDEST precise interval to avoid 1-bit boolean [1,1]
    // shadowing the correct 32-bit computation result.
    let mut best: Option<Interval> = None;
    for bits in [64, 32, 8, 1] {
        let iv = source.interval_at_inst(inst, value, bits);
        if !iv.is_bottom() && !iv.is_top() && best.is_none() {
            // First (widest) precise interval found — use it
            best = Some(iv);
        }
    }
    if let Some(iv) = best {
        return iv;
    }
    // Fall back: prefer non-bottom (returns TOP at the found width)
    for bits in [64, 32] {
        let iv = source.interval_at_inst(inst, value, bits);
        if !iv.is_bottom() {
            return iv;
        }
    }
    // All lookups returned bottom (unreachable instruction) — preserve bottom
    Interval::make_bottom(64)
}

/// Evaluate a comparison operation given abstract intervals for operands.
///
/// `assert_inst` is the instruction where the condition is checked (typically the
/// `svf_assert` call). `cmp_inst` is the comparison instruction itself, used as a
/// fallback when operands aren't available at the assertion point (e.g., when the
/// comparison is in a predecessor block reached through a phi).
// NOTE: This function handles all comparison kinds (eq, ne, slt, sge, ult, uge)
// with multi-width interval lookup and fallback logic. Splitting by comparison
// type would duplicate the interval resolution and fallback code.
#[allow(clippy::too_many_lines)]
fn evaluate_comparison<Q: IntervalQuery>(
    kind: BinaryOp,
    operands: &[ValueId],
    assert_inst: InstId,
    cmp_inst: InstId,
    interval_source: &Q,
) -> (ConditionStatus, String, Option<String>) {
    if operands.len() < 2 {
        return (
            ConditionStatus::Unknown,
            "comparison has fewer than 2 operands".to_string(),
            None,
        );
    }

    let lhs_value = operands[0];
    let rhs_value = operands[1];

    // Get intervals for both operands using multi-width lookup.
    // Try the assertion point first (may have refined intervals from branch conditions),
    // then fall back to the comparison point (where operands are always defined).
    let mut lhs = best_interval(interval_source, assert_inst, lhs_value);
    let mut rhs = best_interval(interval_source, assert_inst, rhs_value);

    // Fallback: when operands are bottom/top at the assertion point (e.g., because
    // the comparison is in a phi predecessor block and operands were dropped during
    // join), try the comparison instruction's own point where operands are defined.
    if cmp_inst != assert_inst {
        if lhs.is_bottom() || lhs.is_top() {
            let fallback = best_interval(interval_source, cmp_inst, lhs_value);
            if !fallback.is_bottom() && (!lhs.is_bottom() || !fallback.is_top()) {
                lhs = fallback;
            }
        }
        if rhs.is_bottom() || rhs.is_top() {
            let fallback = best_interval(interval_source, cmp_inst, rhs_value);
            if !fallback.is_bottom() && (!rhs.is_bottom() || !fallback.is_top()) {
                rhs = fallback;
            }
        }
    }

    let interval_info = Some(format!(
        "lhs: [{}, {}], rhs: [{}, {}]",
        lhs.lo(),
        lhs.hi(),
        rhs.lo(),
        rhs.hi()
    ));

    // If either operand is bottom, we can't evaluate
    if lhs.is_bottom() || rhs.is_bottom() {
        return (
            ConditionStatus::Unknown,
            format!("{kind:?}: operand interval is bottom"),
            interval_info,
        );
    }

    // Evaluate the comparison based on intervals
    let (always_true, always_false) = match kind {
        // Signed comparisons
        BinaryOp::ICmpSge => {
            // lhs >= rhs is always true if lhs.lo >= rhs.hi (signed)
            let always_true = signed_ge(lhs.lo(), rhs.hi());
            // lhs >= rhs is always false if lhs.hi < rhs.lo (signed)
            let always_false = signed_lt(lhs.hi(), rhs.lo());
            (always_true, always_false)
        }
        BinaryOp::ICmpSgt => {
            // lhs > rhs is always true if lhs.lo > rhs.hi (signed)
            let always_true = signed_gt(lhs.lo(), rhs.hi());
            // lhs > rhs is always false if lhs.hi <= rhs.lo (signed)
            let always_false = signed_le(lhs.hi(), rhs.lo());
            (always_true, always_false)
        }
        BinaryOp::ICmpSle => {
            // lhs <= rhs is always true if lhs.hi <= rhs.lo (signed)
            let always_true = signed_le(lhs.hi(), rhs.lo());
            // lhs <= rhs is always false if lhs.lo > rhs.hi (signed)
            let always_false = signed_gt(lhs.lo(), rhs.hi());
            (always_true, always_false)
        }
        BinaryOp::ICmpSlt => {
            // lhs < rhs is always true if lhs.hi < rhs.lo (signed)
            let always_true = signed_lt(lhs.hi(), rhs.lo());
            // lhs < rhs is always false if lhs.lo >= rhs.hi (signed)
            let always_false = signed_ge(lhs.lo(), rhs.hi());
            (always_true, always_false)
        }
        // Unsigned comparisons
        BinaryOp::ICmpUge => {
            let always_true = lhs.lo() >= rhs.hi();
            let always_false = lhs.hi() < rhs.lo();
            (always_true, always_false)
        }
        BinaryOp::ICmpUgt => {
            let always_true = lhs.lo() > rhs.hi();
            let always_false = lhs.hi() <= rhs.lo();
            (always_true, always_false)
        }
        BinaryOp::ICmpUle => {
            let always_true = lhs.hi() <= rhs.lo();
            let always_false = lhs.lo() > rhs.hi();
            (always_true, always_false)
        }
        BinaryOp::ICmpUlt => {
            let always_true = lhs.hi() < rhs.lo();
            let always_false = lhs.lo() >= rhs.hi();
            (always_true, always_false)
        }
        // Equality
        BinaryOp::ICmpEq => {
            // Equal is always true if both intervals are singleton and equal
            let always_true = lhs.lo() == lhs.hi() && rhs.lo() == rhs.hi() && lhs.lo() == rhs.lo();
            // Equal is always false if intervals don't overlap
            let always_false = lhs.hi() < rhs.lo() || rhs.hi() < lhs.lo();
            (always_true, always_false)
        }
        BinaryOp::ICmpNe => {
            // Not equal is always true if intervals don't overlap
            let always_true = lhs.hi() < rhs.lo() || rhs.hi() < lhs.lo();
            // Not equal is always false if both are singleton and equal
            let always_false = lhs.lo() == lhs.hi() && rhs.lo() == rhs.hi() && lhs.lo() == rhs.lo();
            (always_true, always_false)
        }
        // Other operations (not comparisons)
        _ => {
            return (
                ConditionStatus::Unknown,
                format!("not a comparison operation: {kind:?}"),
                interval_info,
            );
        }
    };

    // Include interval info in the description for debugging
    let interval_desc = format!(
        " (lhs=[{},{}], rhs=[{},{}])",
        lhs.lo(),
        lhs.hi(),
        rhs.lo(),
        rhs.hi()
    );

    if always_true {
        (
            ConditionStatus::Proven,
            format!("{kind:?} always true{interval_desc}"),
            interval_info,
        )
    } else if always_false {
        (
            ConditionStatus::MayFail,
            format!("{kind:?} always false{interval_desc}"),
            interval_info,
        )
    } else {
        (
            ConditionStatus::MayFail,
            format!("{kind:?} may be false{interval_desc}"),
            interval_info,
        )
    }
}

// ---------------------------------------------------------------------------
// Signed comparison helpers for i128 interval bounds
// ---------------------------------------------------------------------------

/// Signed greater-than-or-equal comparison.
fn signed_ge(a: i128, b: i128) -> bool {
    a >= b
}

/// Signed greater-than comparison.
fn signed_gt(a: i128, b: i128) -> bool {
    a > b
}

/// Signed less-than-or-equal comparison.
fn signed_le(a: i128, b: i128) -> bool {
    a <= b
}

/// Signed less-than comparison.
fn signed_lt(a: i128, b: i128) -> bool {
    a < b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_comparisons() {
        // Positive numbers
        assert!(signed_ge(10, 5));
        assert!(signed_gt(10, 5));
        assert!(!signed_le(10, 5));
        assert!(!signed_lt(10, 5));

        // Negative numbers
        assert!(signed_ge(-1, -5)); // -1 >= -5
        assert!(signed_gt(-1, -5)); // -1 > -5
        assert!(!signed_le(-1, -5));
        assert!(!signed_lt(-1, -5));

        // Cross zero
        assert!(signed_ge(5, -1)); // 5 >= -1
        assert!(signed_gt(5, -1)); // 5 > -1
        assert!(signed_le(-1, 5)); // -1 <= 5
        assert!(signed_lt(-1, 5)); // -1 < 5
    }
}
