//! Parameter effect summaries for cross-function reasoning.
//!
//! Computes how each function affects its parameters (free, dereference)
//! and whether it returns newly allocated memory. Used by temporal safety
//! checkers (UAF, double-free, memory-leak) to filter false positives
//! arising from cross-function paths.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, ValueId};

use crate::callgraph::CallGraph;
use crate::graph_algo;

use super::resource_table::{ResourceRole, ResourceTable};

/// Summary of how a function affects its parameters and return value.
///
/// Used by temporal safety checkers (UAF, double-free, memory-leak) to determine
/// whether a cross-function finding is genuine. If a called function doesn't
/// actually free its parameter, a UAF finding through that call is a false positive.
#[derive(Debug, Clone)]
pub struct ParameterEffectSummary {
    /// The function this summary describes.
    pub func_id: FunctionId,
    /// Whether the callee frees `param[i]` (directly or transitively).
    pub param_freed: BTreeMap<usize, bool>,
    /// Whether the callee dereferences `param[i]` (load/store through it).
    pub param_dereferenced: BTreeMap<usize, bool>,
    /// Whether the function returns newly allocated memory.
    pub return_is_allocated: bool,
}

/// Compute parameter effect summaries for all functions in the module.
///
/// Uses bottom-up analysis on the call graph:
/// 1. External/declaration functions: populated from `ResourceTable` roles
/// 2. Leaf functions: scanned for free/deref/alloc instructions
/// 3. Internal functions: compose callee summaries transitively
///
/// Returns a map from `FunctionId` to its effect summary.
// NOTE: This function processes declarations and defined functions in a single
// bottom-up pass over the call graph. Splitting would fragment the analysis logic.
#[allow(clippy::too_many_lines)]
pub fn compute_parameter_effect_summaries(
    module: &AirModule,
    table: &ResourceTable,
) -> BTreeMap<FunctionId, ParameterEffectSummary> {
    let callgraph = CallGraph::build(module);

    // Attempt topological sort (dependencies-first).
    // If cycles exist, fall back to processing all functions without
    // transitive composition.
    let has_topo = graph_algo::toposort(&callgraph.nodes, &callgraph).is_some();

    let mut summaries: BTreeMap<FunctionId, ParameterEffectSummary> = BTreeMap::new();

    // Process declarations first (they only depend on the resource table).
    for func in &module.functions {
        if !func.is_declaration {
            continue;
        }
        let mut param_freed = BTreeMap::new();
        let mut param_dereferenced = BTreeMap::new();
        let mut return_is_allocated = false;

        if table.has_role(&func.name, ResourceRole::Deallocator)
            || table.has_role(&func.name, ResourceRole::Release)
            || table.has_role(&func.name, ResourceRole::Unlock)
        {
            param_freed.insert(0, true);
        }
        if table.has_role(&func.name, ResourceRole::Dereference) {
            param_dereferenced.insert(0, true);
        }
        if table.has_role(&func.name, ResourceRole::Allocator) {
            return_is_allocated = true;
        }

        summaries.insert(
            func.id,
            ParameterEffectSummary {
                func_id: func.id,
                param_freed,
                param_dereferenced,
                return_is_allocated,
            },
        );
    }

    // Process defined functions. If we have a valid topo ordering we can
    // compose callee summaries transitively; otherwise we only do direct
    // instruction scanning.
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        // Map each parameter ValueId to its index for fast lookup.
        let param_value_ids: BTreeMap<ValueId, usize> = func
            .params
            .iter()
            .map(|p| (p.id, p.index as usize))
            .collect();

        let mut param_freed: BTreeMap<usize, bool> = BTreeMap::new();
        let mut param_dereferenced: BTreeMap<usize, bool> = BTreeMap::new();
        let mut return_is_allocated = false;

        // Collect returned ValueIds so we can check if a callee's
        // allocated return flows out of this function.
        let mut returned_values: Vec<ValueId> = Vec::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if matches!(inst.op, Operation::Ret) {
                    if let Some(&ret_val) = inst.operands.first() {
                        returned_values.push(ret_val);
                    }
                }
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::CallDirect { callee } => {
                        // Only compose transitively when topo order is available.
                        if !has_topo {
                            continue;
                        }
                        let Some(callee_summary) = summaries.get(callee) else {
                            continue;
                        };

                        // Map call operands to callee parameter indices.
                        for (arg_idx, &arg_val) in inst.operands.iter().enumerate() {
                            if let Some(&our_param_idx) = param_value_ids.get(&arg_val) {
                                if callee_summary
                                    .param_freed
                                    .get(&arg_idx)
                                    .copied()
                                    .unwrap_or(false)
                                {
                                    param_freed.insert(our_param_idx, true);
                                }
                                if callee_summary
                                    .param_dereferenced
                                    .get(&arg_idx)
                                    .copied()
                                    .unwrap_or(false)
                                {
                                    param_dereferenced.insert(our_param_idx, true);
                                }
                            }
                        }

                        // If the callee returns allocated memory and we return it,
                        // mark our function as returning allocated memory.
                        if callee_summary.return_is_allocated {
                            if let Some(dst) = inst.dst {
                                if returned_values.contains(&dst) {
                                    return_is_allocated = true;
                                }
                            }
                        }
                    }
                    Operation::Load => {
                        // operands[0] is the pointer being loaded from.
                        if let Some(&ptr) = inst.operands.first() {
                            if let Some(&param_idx) = param_value_ids.get(&ptr) {
                                param_dereferenced.insert(param_idx, true);
                            }
                        }
                    }
                    Operation::Store => {
                        // operands[1] is the pointer being stored to.
                        if let Some(&ptr) = inst.operands.get(1) {
                            if let Some(&param_idx) = param_value_ids.get(&ptr) {
                                param_dereferenced.insert(param_idx, true);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        summaries.insert(
            func.id,
            ParameterEffectSummary {
                func_id: func.id,
                param_freed,
                param_dereferenced,
                return_is_allocated,
            },
        );
    }

    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        let mut module = AirModule::new(ModuleId::derive(b"test"));
        module.name = Some("test".to_string());
        for func in functions {
            module.add_function(func);
        }
        module
    }

    fn make_declaration(id: u128, name: &str) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    #[test]
    fn external_deallocator_sets_param_freed() {
        let free_fn = make_declaration(1, "free");
        let module = make_module(vec![free_fn]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("free summary");
        assert_eq!(summary.param_freed.get(&0), Some(&true));
        assert!(!summary.return_is_allocated);
    }

    #[test]
    fn external_allocator_sets_return_allocated() {
        let malloc_fn = make_declaration(1, "malloc");
        let module = make_module(vec![malloc_fn]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("malloc summary");
        assert!(summary.return_is_allocated);
    }

    #[test]
    fn external_release_sets_param_freed() {
        let fclose_fn = make_declaration(1, "fclose");
        let module = make_module(vec![fclose_fn]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("fclose summary");
        assert_eq!(summary.param_freed.get(&0), Some(&true));
    }

    #[test]
    fn external_dereference_sets_param_dereferenced() {
        let strlen_fn = make_declaration(1, "strlen");
        let module = make_module(vec![strlen_fn]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("strlen summary");
        assert_eq!(summary.param_dereferenced.get(&0), Some(&true));
    }

    #[test]
    fn defined_function_load_dereferences_param() {
        // fn foo(ptr):
        //   load ptr
        //   ret
        let param_val = ValueId::new(100);
        let mut func = AirFunction::new(FunctionId::new(1), "foo");
        func.params = vec![AirParam::new(param_val, 0)];

        let mut block = AirBlock::new(BlockId::new(1));
        block.instructions.push(
            Instruction::new(InstId::new(10), Operation::Load)
                .with_operands(vec![param_val])
                .with_dst(ValueId::new(101)),
        );
        block
            .instructions
            .push(Instruction::new(InstId::new(11), Operation::Ret));
        func.add_block(block);

        let module = make_module(vec![func]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("foo summary");
        assert_eq!(summary.param_dereferenced.get(&0), Some(&true));
    }

    #[test]
    fn defined_function_store_dereferences_param() {
        // fn foo(ptr):
        //   store val, ptr
        //   ret
        let param_val = ValueId::new(100);
        let store_val = ValueId::new(101);
        let mut func = AirFunction::new(FunctionId::new(1), "foo");
        func.params = vec![AirParam::new(param_val, 0)];

        let mut block = AirBlock::new(BlockId::new(1));
        block.instructions.push(
            Instruction::new(InstId::new(10), Operation::Store)
                .with_operands(vec![store_val, param_val]),
        );
        block
            .instructions
            .push(Instruction::new(InstId::new(11), Operation::Ret));
        func.add_block(block);

        let module = make_module(vec![func]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("foo summary");
        assert_eq!(summary.param_dereferenced.get(&0), Some(&true));
    }

    #[test]
    fn transitive_free_through_callee() {
        // declare free(ptr)
        // fn wrapper(p):
        //   call free(p)
        //   ret
        let free_fn = make_declaration(1, "free");

        let param_val = ValueId::new(200);
        let mut wrapper = AirFunction::new(FunctionId::new(2), "wrapper");
        wrapper.params = vec![AirParam::new(param_val, 0)];

        let mut block = AirBlock::new(BlockId::new(1));
        let mut call_inst = Instruction::new(
            InstId::new(20),
            Operation::CallDirect {
                callee: FunctionId::new(1),
            },
        );
        call_inst.operands = vec![param_val];
        block.instructions.push(call_inst);
        block
            .instructions
            .push(Instruction::new(InstId::new(21), Operation::Ret));
        wrapper.add_block(block);

        let module = make_module(vec![free_fn, wrapper]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(2)).expect("wrapper summary");
        assert_eq!(summary.param_freed.get(&0), Some(&true));
    }

    #[test]
    fn no_effect_on_unrelated_param() {
        // fn foo(a, b):
        //   load a  ; only dereferences param 0, not param 1
        //   ret
        let param_a = ValueId::new(100);
        let param_b = ValueId::new(101);
        let mut func = AirFunction::new(FunctionId::new(1), "foo");
        func.params = vec![AirParam::new(param_a, 0), AirParam::new(param_b, 1)];

        let mut block = AirBlock::new(BlockId::new(1));
        block.instructions.push(
            Instruction::new(InstId::new(10), Operation::Load)
                .with_operands(vec![param_a])
                .with_dst(ValueId::new(102)),
        );
        block
            .instructions
            .push(Instruction::new(InstId::new(11), Operation::Ret));
        func.add_block(block);

        let module = make_module(vec![func]);
        let table = ResourceTable::new();

        let summaries = compute_parameter_effect_summaries(&module, &table);

        let summary = summaries.get(&FunctionId::new(1)).expect("foo summary");
        assert_eq!(summary.param_dereferenced.get(&0), Some(&true));
        assert!(summary.param_dereferenced.get(&1).is_none());
    }
}
