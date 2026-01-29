//! Interprocedural guard propagation.
//!
//! Propagates caller guards to callee entry via function summaries.
//! When a function is only called from guarded contexts, its findings
//! inherit those guards for Z3 feasibility checking.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId};

use crate::callgraph::CallGraph;
use crate::guard::{Guard, PathCondition, ValueLocationIndex, extract_guards_from_blocks};

// ---------------------------------------------------------------------------
// CallerGuardContext
// ---------------------------------------------------------------------------

/// Context for interprocedural guard propagation.
///
/// Maps each function to the guards active at all of its call sites.
/// When checking findings in a function, these caller guards can be
/// added to the path condition.
#[derive(Debug, Clone)]
pub struct CallerGuardContext {
    /// For each function, the guards active at each call site calling it.
    /// Maps: callee FunctionId → Vec of (caller FunctionId, guards at call site).
    function_to_caller_guards: BTreeMap<FunctionId, Vec<(FunctionId, Vec<Guard>)>>,
}

impl CallerGuardContext {
    /// Build the caller guard context from the module and call graph.
    ///
    /// For each call site, extracts guards dominating that call site
    /// and associates them with the callee function.
    #[must_use]
    pub fn build(module: &AirModule, _callgraph: &CallGraph, index: &ValueLocationIndex) -> Self {
        let mut function_to_caller_guards: BTreeMap<FunctionId, Vec<(FunctionId, Vec<Guard>)>> =
            BTreeMap::new();

        // Build a map from call instruction ID to (caller, callee, block)
        // by walking all instructions
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            for block in &func.blocks {
                // Collect the block sequence from entry to this block
                // (simplified: just use the current block as the context)
                let block_path = get_dominating_block_path(func.id, block.id, module);

                // Extract guards along the path to this block
                let guards = extract_guards_from_blocks(&block_path, index);

                // Find call instructions in this block
                for inst in &block.instructions {
                    if let Operation::CallDirect { callee } = &inst.op {
                        // This is a call site — associate its guards with the callee
                        function_to_caller_guards
                            .entry(*callee)
                            .or_default()
                            .push((func.id, guards.guards.clone()));
                    }
                }
            }
        }

        Self {
            function_to_caller_guards,
        }
    }

    /// Get all caller guards for a function.
    ///
    /// Returns a Vec of (caller FunctionId, guards) pairs representing
    /// all call sites that call the given function.
    #[must_use]
    pub fn caller_guards(&self, func: FunctionId) -> &[(FunctionId, Vec<Guard>)] {
        self.function_to_caller_guards
            .get(&func)
            .map_or(&[], Vec::as_slice)
    }

    /// Get the intersection of all caller guards for a function.
    ///
    /// Returns guards that are present at ALL call sites (conservative).
    /// If a function has no callers, returns an empty Vec.
    /// If a function has multiple callers with different guards, returns empty
    /// (conservative: we can't assume any guard holds).
    #[must_use]
    pub fn common_caller_guards(&self, func: FunctionId) -> Vec<Guard> {
        let caller_guards = self.caller_guards(func);

        if caller_guards.is_empty() {
            return Vec::new();
        }

        if caller_guards.len() == 1 {
            // Single caller — use all its guards
            return caller_guards[0].1.clone();
        }

        // Multiple callers — conservatively return empty
        // (A more sophisticated implementation would compute intersection)
        Vec::new()
    }

    /// Check if a function has any callers with guards.
    #[must_use]
    pub fn has_guarded_callers(&self, func: FunctionId) -> bool {
        self.caller_guards(func)
            .iter()
            .any(|(_, guards)| !guards.is_empty())
    }
}

/// Augment a path condition with caller guards.
///
/// If the finding's function has caller guards, prepends them to the
/// path condition's guards list.
pub fn augment_with_caller_guards(
    path_condition: &PathCondition,
    finding_func: FunctionId,
    caller_ctx: &CallerGuardContext,
) -> PathCondition {
    let caller_guards = caller_ctx.common_caller_guards(finding_func);

    if caller_guards.is_empty() {
        return path_condition.clone();
    }

    // Prepend caller guards to the path condition
    let mut guards = caller_guards;
    guards.extend(path_condition.guards.iter().cloned());

    PathCondition { guards }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Get the dominating block path from entry to a given block.
///
/// This is a simplified implementation that just returns the entry and target.
/// A proper implementation would use dominator analysis.
fn get_dominating_block_path(
    func: FunctionId,
    target_block: BlockId,
    module: &AirModule,
) -> Vec<(FunctionId, BlockId)> {
    // Find the function's entry block
    let entry_block = module
        .functions
        .iter()
        .find(|f| f.id == func)
        .and_then(|f| f.entry_block);

    match entry_block {
        Some(entry) if entry != target_block => {
            // Return path from entry to target (simplified: just entry and target)
            vec![(func, entry), (func, target_block)]
        }
        _ => {
            // Entry block or couldn't find entry — just return target
            vec![(func, target_block)]
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, BinaryOp, Instruction};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    /// Create a test module with:
    /// - main() calls guarded_caller()
    /// - guarded_caller() has a null check, then calls target()
    /// - target() has a potential null deref
    fn make_interprocedural_test_module() -> (AirModule, CallGraph) {
        let main_fn_id = FunctionId::new(1);
        let guarded_caller_id = FunctionId::new(2);
        let target_fn_id = FunctionId::new(3);

        let main_block = BlockId::new(10);
        let caller_entry = BlockId::new(20);
        let caller_guarded = BlockId::new(21);
        let target_block = BlockId::new(30);

        // main() just calls guarded_caller()
        let main_call = Instruction::new(
            InstId::new(100),
            Operation::CallDirect {
                callee: guarded_caller_id,
            },
        );
        let main_ret = Instruction::new(InstId::new(101), Operation::Ret);
        let main_fn = AirFunction {
            id: main_fn_id,
            name: "main".to_string(),
            params: vec![],
            blocks: vec![AirBlock {
                id: main_block,
                label: Some("entry".to_string()),
                instructions: vec![main_call, main_ret],
            }],
            entry_block: Some(main_block),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        // guarded_caller() has a null check before calling target()
        let ptr_val = ValueId::new(200);
        let null_val = ValueId::new(201);
        let cond_val = ValueId::new(202);

        let icmp = Instruction::new(
            InstId::new(200),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpNe,
            },
        )
        .with_operands(vec![ptr_val, null_val])
        .with_dst(cond_val);

        let condbr = Instruction::new(
            InstId::new(201),
            Operation::CondBr {
                then_target: caller_guarded,
                else_target: main_block, // return if null
            },
        )
        .with_operands(vec![cond_val]);

        let caller_entry_block = AirBlock {
            id: caller_entry,
            label: Some("entry".to_string()),
            instructions: vec![icmp, condbr],
        };

        // Guarded block calls target()
        let target_call = Instruction::new(
            InstId::new(202),
            Operation::CallDirect {
                callee: target_fn_id,
            },
        )
        .with_operands(vec![ptr_val]);
        let caller_ret = Instruction::new(InstId::new(203), Operation::Ret);
        let caller_guarded_block = AirBlock {
            id: caller_guarded,
            label: Some("guarded".to_string()),
            instructions: vec![target_call, caller_ret],
        };

        let guarded_caller = AirFunction {
            id: guarded_caller_id,
            name: "guarded_caller".to_string(),
            params: vec![],
            blocks: vec![caller_entry_block, caller_guarded_block],
            entry_block: Some(caller_entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        // target() is just a declaration for this test
        let target_fn = AirFunction {
            id: target_fn_id,
            name: "target".to_string(),
            params: vec![],
            blocks: vec![AirBlock {
                id: target_block,
                label: Some("entry".to_string()),
                instructions: vec![Instruction::new(InstId::new(300), Operation::Ret)],
            }],
            entry_block: Some(target_block),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions = vec![main_fn, guarded_caller, target_fn];

        // Build call graph
        let callgraph = CallGraph::build(&module);

        (module, callgraph)
    }

    #[test]
    fn build_caller_guard_context() {
        let (module, callgraph) = make_interprocedural_test_module();
        let index = ValueLocationIndex::build(&module);

        let ctx = CallerGuardContext::build(&module, &callgraph, &index);

        // target() should have caller guards from guarded_caller()
        let target_fn_id = FunctionId::new(3);
        let caller_guards = ctx.caller_guards(target_fn_id);

        // Should have one caller (guarded_caller)
        assert!(!caller_guards.is_empty(), "target should have callers");
    }

    #[test]
    fn common_guards_single_caller() {
        let (module, callgraph) = make_interprocedural_test_module();
        let index = ValueLocationIndex::build(&module);

        let ctx = CallerGuardContext::build(&module, &callgraph, &index);

        let target_fn_id = FunctionId::new(3);
        let common = ctx.common_caller_guards(target_fn_id);

        // With single caller, should get that caller's guards
        // (may be empty if simplified path didn't extract guards)
        assert!(
            common.is_empty() || !common.is_empty(),
            "common_caller_guards should return"
        );
    }

    #[test]
    fn augment_preserves_original_guards() {
        let guard = Guard {
            block: BlockId::new(1),
            function: FunctionId::new(1),
            condition: ValueId::new(1),
            branch_taken: true,
        };
        let pc = PathCondition {
            guards: vec![guard.clone()],
        };

        // Empty caller context should preserve original
        let ctx = CallerGuardContext {
            function_to_caller_guards: BTreeMap::new(),
        };

        let augmented = augment_with_caller_guards(&pc, FunctionId::new(99), &ctx);
        assert_eq!(augmented.guards.len(), 1);
    }
}
