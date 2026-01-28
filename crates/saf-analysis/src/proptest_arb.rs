//! Proptest arbitrary generators for AIR types.
//!
//! Used for property-based testing of graph builders and PTA.

#![allow(dead_code)] // Utility functions for future tests

use std::collections::BTreeMap;

use proptest::prelude::*;

use saf_core::air::{AirBlock, AirFunction, AirModule, CastKind, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

/// Strategy for generating block IDs.
pub fn arb_block_id() -> impl Strategy<Value = BlockId> {
    (1u128..=100).prop_map(BlockId::new)
}

/// Strategy for generating instruction IDs.
pub fn arb_inst_id() -> impl Strategy<Value = InstId> {
    (1u128..=10000).prop_map(InstId::new)
}

/// Strategy for generating value IDs.
pub fn arb_value_id() -> impl Strategy<Value = ValueId> {
    (1u128..=10000).prop_map(ValueId::new)
}

/// Strategy for generating terminator operations.
fn arb_terminator(block_ids: Vec<BlockId>) -> impl Strategy<Value = Operation> {
    if block_ids.is_empty() {
        // No targets available, must return
        Just(Operation::Ret).boxed()
    } else {
        prop_oneof![
            // Ret - no successors
            Just(Operation::Ret),
            // Unreachable - no successors
            Just(Operation::Unreachable),
            // Unconditional branch
            proptest::sample::select(block_ids.clone()).prop_map(|target| Operation::Br { target }),
            // Conditional branch (if we have at least 2 blocks)
            if block_ids.len() >= 2 {
                (
                    proptest::sample::select(block_ids.clone()),
                    proptest::sample::select(block_ids.clone()),
                )
                    .prop_map(|(then_target, else_target)| Operation::CondBr {
                        then_target,
                        else_target,
                    })
                    .boxed()
            } else {
                proptest::sample::select(block_ids.clone())
                    .prop_map(|target| Operation::Br { target })
                    .boxed()
            },
        ]
        .boxed()
    }
}

/// Strategy for generating a basic block with a terminator.
fn arb_block(id: BlockId, possible_targets: Vec<BlockId>) -> impl Strategy<Value = AirBlock> {
    arb_terminator(possible_targets).prop_map(move |term| {
        let inst_id = InstId::new(id.raw() * 100);
        AirBlock {
            id,
            label: None,
            instructions: vec![Instruction::new(inst_id, term)],
        }
    })
}

/// Strategy for generating a function with multiple blocks.
pub fn arb_air_function() -> impl Strategy<Value = AirFunction> {
    // Generate 1-5 blocks
    (1usize..=5).prop_flat_map(|num_blocks| {
        let block_ids: Vec<BlockId> = (1..=num_blocks as u128).map(BlockId::new).collect();
        let block_ids_clone = block_ids.clone();

        // Generate blocks with proper terminators
        let blocks_strategy = block_ids
            .into_iter()
            .map(move |id| {
                let targets = block_ids_clone.clone();
                arb_block(id, targets)
            })
            .collect::<Vec<_>>();

        blocks_strategy
            .prop_map(|blocks_vec| blocks_vec.into_iter().collect::<Vec<_>>())
            .prop_map(|blocks| AirFunction {
                id: FunctionId::derive(b"test_func"),
                name: "test_func".to_string(),
                params: Vec::new(),
                blocks,
                entry_block: None,
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            })
    })
}

/// Strategy for generating a simple module with one function.
pub fn arb_air_module() -> impl Strategy<Value = AirModule> {
    arb_air_function().prop_map(|func| AirModule {
        id: ModuleId::derive(b"test_module"),
        name: Some("test_module".to_string()),
        functions: vec![func],
        globals: Vec::new(),
        source_files: Vec::new(),
        type_hierarchy: Vec::new(),
        constants: std::collections::BTreeMap::new(),
        types: std::collections::BTreeMap::new(),
        target_pointer_width: 8,
        function_index: BTreeMap::new(),
        name_index: BTreeMap::new(),
    })
}

/// Strategy for generating a module with pointer operations for PTA testing.
///
/// Generates a function with allocas, loads, stores, and copies.
pub fn arb_pta_module() -> impl Strategy<Value = AirModule> {
    arb_pta_function().prop_map(|func| AirModule {
        id: ModuleId::derive(b"pta_test_module"),
        name: Some("pta_test_module".to_string()),
        functions: vec![func],
        globals: Vec::new(),
        source_files: Vec::new(),
        type_hierarchy: Vec::new(),
        constants: std::collections::BTreeMap::new(),
        types: std::collections::BTreeMap::new(),
        target_pointer_width: 8,
        function_index: BTreeMap::new(),
        name_index: BTreeMap::new(),
    })
}

/// Strategy for generating a function with pointer operations.
pub fn arb_pta_function() -> impl Strategy<Value = AirFunction> {
    // Generate 1-5 allocas, then 0-10 pointer operations, then Ret
    (1usize..=5, 0usize..=10).prop_flat_map(|(num_allocas, num_ops)| {
        // Generate alloca value IDs
        let alloca_ids: Vec<ValueId> = (1..=num_allocas as u128)
            .map(|i| ValueId::new(0x100 + i))
            .collect();

        // Generate operation value IDs (for loads/copies)
        let op_ids: Vec<ValueId> = (1..=num_ops as u128)
            .map(|i| ValueId::new(0x200 + i))
            .collect();

        let alloca_ids_clone = alloca_ids.clone();
        let op_ids_clone = op_ids.clone();

        // Generate pointer operations
        proptest::collection::vec(
            arb_ptr_operation(alloca_ids_clone.clone(), op_ids_clone.clone()),
            num_ops,
        )
        .prop_map(move |ops| {
            let mut instructions = Vec::new();
            let mut inst_counter = 1u128;

            // First, add all allocas
            for alloca_id in &alloca_ids {
                instructions.push(
                    Instruction::new(
                        InstId::new(inst_counter),
                        Operation::Alloca { size_bytes: None },
                    )
                    .with_dst(*alloca_id),
                );
                inst_counter += 1;
            }

            // Add pointer operations
            for (i, ptr_op) in ops.into_iter().enumerate() {
                let dst_id = op_ids
                    .get(i)
                    .copied()
                    .unwrap_or(ValueId::new(0x300 + i as u128));
                let inst = Instruction::new(InstId::new(inst_counter), ptr_op.op)
                    .with_operands(ptr_op.operands);
                if ptr_op.has_dst {
                    instructions.push(inst.with_dst(dst_id));
                } else {
                    instructions.push(inst);
                }
                inst_counter += 1;
            }

            // Add terminator
            instructions.push(Instruction::new(InstId::new(inst_counter), Operation::Ret));

            let block = AirBlock {
                id: BlockId::new(1),
                label: Some("entry".to_string()),
                instructions,
            };

            AirFunction {
                id: FunctionId::derive(b"pta_test_func"),
                name: "pta_test_func".to_string(),
                params: Vec::new(),
                blocks: vec![block],
                entry_block: None,
                is_declaration: false,
                span: None,
                symbol: None,
                block_index: BTreeMap::new(),
            }
        })
    })
}

/// Helper struct for pointer operation generation.
#[derive(Debug, Clone)]
struct PtrOp {
    op: Operation,
    operands: Vec<ValueId>,
    has_dst: bool,
}

/// Generate a PTA module that tracks how many of each instruction type it contains.
/// Returns `(AirModule, expected_alloca_count, expected_store_count, expected_load_count)`.
pub fn arb_tracked_pta_module() -> impl Strategy<Value = (AirModule, usize, usize, usize)> {
    arb_pta_function().prop_map(|func| {
        let mut alloca_count = 0usize;
        let mut store_count = 0usize;
        let mut load_count = 0usize;

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    Operation::Alloca { .. } => alloca_count += 1,
                    Operation::Store => store_count += 1,
                    Operation::Load => load_count += 1,
                    _ => {}
                }
            }
        }

        let module = AirModule {
            id: ModuleId::derive(b"tracked_test"),
            name: Some("tracked_test".to_string()),
            functions: vec![func],
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };

        (module, alloca_count, store_count, load_count)
    })
}

/// Strategy for generating pointer operations (Load, Store, Copy).
fn arb_ptr_operation(
    alloca_ids: Vec<ValueId>,
    op_ids: Vec<ValueId>,
) -> impl Strategy<Value = PtrOp> {
    // All possible source values (allocas + operation results)
    let all_values: Vec<ValueId> = alloca_ids.iter().chain(op_ids.iter()).copied().collect();

    if alloca_ids.is_empty() {
        // No allocas, return a no-op
        return Just(PtrOp {
            op: Operation::Ret,
            operands: vec![],
            has_dst: false,
        })
        .boxed();
    }

    prop_oneof![
        // Load from an alloca (ptr = alloca result, so we load *ptr)
        proptest::sample::select(alloca_ids.clone()).prop_map(|ptr| PtrOp {
            op: Operation::Load,
            operands: vec![ptr],
            has_dst: true,
        }),
        // Store value to alloca (Store has no dst)
        (
            proptest::sample::select(all_values.clone()),
            proptest::sample::select(alloca_ids.clone()),
        )
            .prop_map(|(val, ptr)| PtrOp {
                op: Operation::Store,
                operands: vec![val, ptr],
                has_dst: false,
            }),
        // Copy (Cast)
        proptest::sample::select(all_values.clone()).prop_map(|src| PtrOp {
            op: Operation::Cast {
                kind: CastKind::Bitcast,
                target_bits: None,
            },
            operands: vec![src],
            has_dst: true,
        }),
    ]
    .boxed()
}
