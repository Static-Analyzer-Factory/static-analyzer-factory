//! Value flow graph builder.
//!
//! Constructs the value flow graph from AIR using def-use information,
//! call graph, and optionally PTA results.

use saf_core::air::{AirModule, Operation};
use saf_core::id::make_id;
use saf_core::ids::{FunctionId, InstId, ValueId};

use crate::PtaResult;
use crate::callgraph::{CallGraph, CallGraphNode};
use crate::defuse::DefUseGraph;

use super::ValueFlowGraph;
use super::config::{OpKind, TransformPropagation, ValueFlowConfig, ValueFlowMode};
use super::edge::EdgeKind;
use super::node::NodeId;

/// Builder for constructing a value flow graph.
pub struct ValueFlowBuilder<'a> {
    config: &'a ValueFlowConfig,
    module: &'a AirModule,
    #[allow(dead_code)] // Reserved for future use (e.g., dead value elimination)
    defuse: &'a DefUseGraph,
    callgraph: &'a CallGraph,
    pta: Option<&'a PtaResult>,
}

impl<'a> ValueFlowBuilder<'a> {
    /// Create a new value flow builder.
    ///
    /// # Arguments
    /// - `config`: Configuration for the analysis
    /// - `module`: The AIR module to analyze
    /// - `defuse`: Pre-computed def-use graph
    /// - `callgraph`: Pre-computed call graph
    /// - `pta`: Optional PTA result (required for precise mode)
    ///
    /// # Panics
    /// Panics if precise mode is requested but no PTA result is provided.
    #[must_use]
    pub fn new(
        config: &'a ValueFlowConfig,
        module: &'a AirModule,
        defuse: &'a DefUseGraph,
        callgraph: &'a CallGraph,
        pta: Option<&'a PtaResult>,
    ) -> Self {
        if config.requires_pta() {
            assert!(
                pta.is_some(),
                "PTA result required for precise mode but not provided"
            );
        }
        Self {
            config,
            module,
            defuse,
            callgraph,
            pta,
        }
    }

    /// Build the value flow graph.
    #[must_use]
    pub fn build(self) -> ValueFlowGraph {
        let mut graph = ValueFlowGraph::new();

        // Process all functions
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }

            // Process all instructions
            for block in &func.blocks {
                for inst in &block.instructions {
                    self.process_instruction(
                        &mut graph,
                        func.id,
                        inst.id,
                        &inst.op,
                        &inst.operands,
                        inst.dst,
                    );
                }
            }
        }

        graph
    }

    /// Process a single instruction and add appropriate edges.
    // NOTE: Unified instruction handler — splitting by operation type would
    // duplicate the common prologue/epilogue and reduce locality.
    #[allow(clippy::too_many_lines)]
    fn process_instruction(
        &self,
        graph: &mut ValueFlowGraph,
        func_id: FunctionId,
        inst_id: InstId,
        op: &Operation,
        operands: &[ValueId],
        dst: Option<ValueId>,
    ) {
        match op {
            // SSA operations that merge values
            Operation::Phi { incoming } => {
                if let Some(result) = dst {
                    for (_, value) in incoming {
                        graph.add_edge(
                            NodeId::value(*value),
                            EdgeKind::DefUse,
                            NodeId::value(result),
                        );
                    }
                }
            }

            Operation::Select => {
                // Select: operands are [condition, true_val, false_val]
                if let Some(result) = dst {
                    if operands.len() >= 3 {
                        // Edges from both true and false values
                        graph.add_edge(
                            NodeId::value(operands[1]),
                            EdgeKind::DefUse,
                            NodeId::value(result),
                        );
                        graph.add_edge(
                            NodeId::value(operands[2]),
                            EdgeKind::DefUse,
                            NodeId::value(result),
                        );
                    }
                }
            }

            // Transform operations
            Operation::BinaryOp { kind } => {
                if let Some(result) = dst {
                    if self.should_propagate_transform(OpKind::from(*kind)) {
                        for operand in operands {
                            graph.add_edge(
                                NodeId::value(*operand),
                                EdgeKind::Transform,
                                NodeId::value(result),
                            );
                        }
                    }
                }
            }

            Operation::Cast { .. } => {
                if let Some(result) = dst {
                    if self.should_propagate_transform(OpKind::Cast) {
                        if let Some(src) = operands.first() {
                            graph.add_edge(
                                NodeId::value(*src),
                                EdgeKind::Transform,
                                NodeId::value(result),
                            );
                        }
                    }
                }
            }

            Operation::Gep { .. } => {
                if let Some(result) = dst {
                    if self.should_propagate_transform(OpKind::Gep) {
                        // GEP: first operand is base pointer
                        if let Some(base) = operands.first() {
                            graph.add_edge(
                                NodeId::value(*base),
                                EdgeKind::Transform,
                                NodeId::value(result),
                            );
                        }
                    }
                }
            }

            // Memory operations
            Operation::Store => {
                // operands: [value, pointer]
                if operands.len() >= 2 {
                    let value = operands[0];
                    let ptr = operands[1];
                    self.add_store_edges(graph, value, ptr);
                }
            }

            Operation::Load => {
                // operands: [pointer]
                if let Some(result) = dst {
                    if let Some(ptr) = operands.first() {
                        self.add_load_edges(graph, *ptr, result);
                    }
                }
            }

            // Call operations
            Operation::CallDirect { callee } => {
                self.add_call_edges(graph, func_id, inst_id, Some(*callee), operands, dst);
            }

            Operation::CallIndirect { .. } => {
                // Last operand is the function pointer (callee-LAST convention);
                // all preceding operands are arguments.
                graph.diagnostics_mut().indirect_calls += 1;
                if !operands.is_empty() {
                    let args = &operands[..operands.len() - 1];
                    self.add_call_edges(graph, func_id, inst_id, None, args, dst);
                }
            }

            // Copy/identity - propagate through
            Operation::Copy | Operation::Freeze => {
                if let Some(result) = dst {
                    if let Some(src) = operands.first() {
                        graph.add_edge(
                            NodeId::value(*src),
                            EdgeKind::DefUse,
                            NodeId::value(result),
                        );
                    }
                }
            }

            // Memcpy: model as Load from src + Store to dst.
            // `operands[0]` = dst pointer, `operands[1]` = src pointer.
            // This mirrors the PTA constraint extraction pattern (Load+Store)
            // using a synthetic intermediate value for the data flowing through.
            Operation::Memcpy => {
                if operands.len() >= 2 {
                    let dst_ptr = operands[0];
                    let src_ptr = operands[1];
                    // Rust and C++ lower many by-value moves to memcpy over
                    // aggregate stack slots. The existing fast memory model
                    // routes memory through unknown_mem, which is safe but can
                    // lose the explicit slot-to-slot relation needed by
                    // selector-level taint queries. Preserve that relation
                    // conservatively.
                    graph.add_edge(
                        NodeId::value(src_ptr),
                        EdgeKind::DefUse,
                        NodeId::value(dst_ptr),
                    );
                    // Create a synthetic intermediate value to bridge the
                    // Load and Store, deterministically derived from the
                    // instruction ID.
                    let tmp = ValueId::new(make_id("vf_memcpy_tmp", &inst_id.raw().to_le_bytes()));
                    self.add_load_edges(graph, src_ptr, tmp);
                    self.add_store_edges(graph, tmp, dst_ptr);
                }
            }

            // Memset: model as Store of the constant value to dst.
            // `operands[0]` = dst pointer, `operands[1]` = value being set.
            Operation::Memset => {
                if operands.len() >= 2 {
                    let dst_ptr = operands[0];
                    let value = operands[1];
                    self.add_store_edges(graph, value, dst_ptr);
                }
            }

            // Operations that don't propagate values
            Operation::Alloca { .. }
            | Operation::Global { .. }
            | Operation::HeapAlloc { .. }
            | Operation::Br { .. }
            | Operation::CondBr { .. }
            | Operation::Switch { .. }
            | Operation::Ret
            | Operation::Unreachable => {}
        }
    }

    /// Check if a transform operation should create edges based on config.
    fn should_propagate_transform(&self, kind: OpKind) -> bool {
        match &self.config.transform_propagation {
            TransformPropagation::All => true,
            TransformPropagation::Whitelist { ops } => ops.contains(&kind),
        }
    }

    /// Add store edges from value to memory location(s).
    fn add_store_edges(&self, graph: &mut ValueFlowGraph, value: ValueId, ptr: ValueId) {
        match self.config.mode {
            ValueFlowMode::Fast => {
                // Fast mode: all stores go to unknown_mem
                graph.add_edge(NodeId::value(value), EdgeKind::Store, NodeId::unknown_mem());
            }
            ValueFlowMode::Precise => {
                if let Some(pta) = self.pta {
                    let locations = pta.points_to(ptr);
                    if locations.is_empty() {
                        // Unknown pointer - use unknown_mem
                        graph.add_edge(
                            NodeId::value(value),
                            EdgeKind::Store,
                            NodeId::unknown_mem(),
                        );
                    } else if locations.len() > self.config.max_locations_per_access {
                        // Too many locations - collapse to unknown_mem
                        graph.diagnostics_mut().locations_collapsed += 1;
                        graph.add_edge(
                            NodeId::value(value),
                            EdgeKind::Store,
                            NodeId::unknown_mem(),
                        );
                    } else {
                        // Add edge to each location
                        for loc_id in locations {
                            graph.add_edge(
                                NodeId::value(value),
                                EdgeKind::Store,
                                NodeId::location(loc_id),
                            );
                        }
                    }
                } else {
                    // No PTA - fallback to unknown_mem
                    graph.add_edge(NodeId::value(value), EdgeKind::Store, NodeId::unknown_mem());
                }
            }
        }
    }

    /// Add load edges from memory location(s) to result value.
    fn add_load_edges(&self, graph: &mut ValueFlowGraph, ptr: ValueId, result: ValueId) {
        match self.config.mode {
            ValueFlowMode::Fast => {
                // Fast mode: all loads come from unknown_mem
                graph.add_edge(NodeId::unknown_mem(), EdgeKind::Load, NodeId::value(result));
            }
            ValueFlowMode::Precise => {
                if let Some(pta) = self.pta {
                    let locations = pta.points_to(ptr);
                    if locations.is_empty() {
                        // Unknown pointer - use unknown_mem
                        graph.add_edge(
                            NodeId::unknown_mem(),
                            EdgeKind::Load,
                            NodeId::value(result),
                        );
                    } else if locations.len() > self.config.max_locations_per_access {
                        // Too many locations - collapse to unknown_mem
                        graph.diagnostics_mut().locations_collapsed += 1;
                        graph.add_edge(
                            NodeId::unknown_mem(),
                            EdgeKind::Load,
                            NodeId::value(result),
                        );
                    } else {
                        // Add edge from each location
                        for loc_id in locations {
                            graph.add_edge(
                                NodeId::location(loc_id),
                                EdgeKind::Load,
                                NodeId::value(result),
                            );
                        }
                    }
                } else {
                    // No PTA - fallback to unknown_mem
                    graph.add_edge(NodeId::unknown_mem(), EdgeKind::Load, NodeId::value(result));
                }
            }
        }
    }

    /// Add call edges for arguments and return value.
    ///
    /// For indirect calls (`callee_id` is `None`), resolved targets are
    /// obtained from the callgraph edges of the caller function. After PTA
    /// runs `resolve_indirect`, the caller node has direct edges to the
    /// resolved `Function`/`External` nodes.
    fn add_call_edges(
        &self,
        graph: &mut ValueFlowGraph,
        caller_func: FunctionId,
        call_site: InstId,
        callee_id: Option<FunctionId>,
        args: &[ValueId],
        call_result: Option<ValueId>,
    ) {
        // Find target function(s)
        let target_funcs: Vec<FunctionId> = if let Some(callee) = callee_id {
            vec![callee]
        } else {
            // Indirect call — look up the call-site target first.
            // If PTA has resolved targets, they appear as Function/External
            // edges from the caller node in the callgraph (added by
            // `resolve_indirect`). When `call_site_target` returns an
            // `IndirectPlaceholder`, collect resolved Function/External
            // callees of the caller instead.
            let mut resolved = Vec::new();
            if let Some(target) = self.callgraph.call_site_target(call_site) {
                match target {
                    CallGraphNode::Function(id) | CallGraphNode::External { func: id, .. } => {
                        resolved.push(*id);
                    }
                    CallGraphNode::IndirectPlaceholder { .. } => {
                        // Placeholder not resolved to a single target —
                        // gather all resolved Function/External callees
                        // of the caller.
                        if let Some(caller_node) = self.callgraph.node_for_function(caller_func) {
                            if let Some(callees) = self.callgraph.callees_of(caller_node) {
                                for callee_node in callees {
                                    if let Some(fid) = callee_node.function_id() {
                                        resolved.push(fid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            resolved
        };

        // Add edges for each resolved target
        for target_func_id in target_funcs {
            if let Some(target_func) = self.module.function(target_func_id) {
                // CallArg edges: actual → formal
                for (i, actual) in args.iter().enumerate() {
                    if let Some(param) = target_func.params.get(i) {
                        graph.add_edge(
                            NodeId::value(*actual),
                            EdgeKind::CallArg,
                            NodeId::value(param.id),
                        );
                    }
                }

                // Return edges: find return values in callee
                // For now, we add edges from any return statement's operand to the call result
                if let Some(result) = call_result {
                    // Rust frequently lowers returns of aggregate values via
                    // the LLVM `sret` ABI: operand 0 is an out pointer where
                    // the callee writes its logical return value. AIR does not
                    // currently record the `sret` attribute, so model the
                    // common ABI shape conservatively: the synthetic call
                    // result and all non-out arguments may flow to operand 0.
                    // This makes std::env/std::process flows visible without
                    // requiring a Rust-specific frontend.
                    if let Some(sret_slot) = args.first() {
                        graph.add_edge(
                            NodeId::value(result),
                            EdgeKind::Return,
                            NodeId::value(*sret_slot),
                        );
                        for actual in args.iter().skip(1) {
                            graph.add_edge(
                                NodeId::value(*actual),
                                EdgeKind::Return,
                                NodeId::value(*sret_slot),
                            );
                        }
                    }

                    for block in &target_func.blocks {
                        for inst in &block.instructions {
                            if let Operation::Ret = &inst.op {
                                if let Some(ret_val) = inst.operands.first() {
                                    graph.add_edge(
                                        NodeId::value(*ret_val),
                                        EdgeKind::Return,
                                        NodeId::value(result),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Convenience function to build a value flow graph.
pub fn build_valueflow(
    config: &ValueFlowConfig,
    module: &AirModule,
    defuse: &DefUseGraph,
    callgraph: &CallGraph,
    pta: Option<&PtaResult>,
) -> ValueFlowGraph {
    ValueFlowBuilder::new(config, module, defuse, callgraph, pta).build()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, BinaryOp, Instruction};
    use saf_core::ids::{BlockId, ModuleId};

    fn make_module(functions: Vec<AirFunction>) -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions,
            globals: Vec::new(),
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    fn make_function(
        id: u128,
        name: &str,
        params: Vec<AirParam>,
        blocks: Vec<AirBlock>,
    ) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
            name: name.to_string(),
            params,
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    #[test]
    fn defuse_from_phi() {
        // phi %3 = [%1 from bb1, %2 from bb2]
        let phi = Instruction::new(
            InstId::new(100),
            Operation::Phi {
                incoming: vec![
                    (BlockId::new(1), ValueId::new(1)),
                    (BlockId::new(2), ValueId::new(2)),
                ],
            },
        )
        .with_dst(ValueId::new(3));

        let func = make_function(
            1,
            "test",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
            ],
            vec![AirBlock {
                id: BlockId::new(3),
                label: None,
                instructions: vec![phi, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Should have edges: v1 -> v3 and v2 -> v3
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));

        assert!(graph.contains_node(n1));
        assert!(graph.contains_node(n2));
        assert!(graph.contains_node(n3));

        let succs1 = graph.successors_of(n1).unwrap();
        assert!(succs1.contains(&(EdgeKind::DefUse, n3)));

        let succs2 = graph.successors_of(n2).unwrap();
        assert!(succs2.contains(&(EdgeKind::DefUse, n3)));
    }

    #[test]
    fn defuse_from_select() {
        // select %4 = %1 ? %2 : %3
        let select = Instruction::new(InstId::new(100), Operation::Select)
            .with_operands(vec![ValueId::new(1), ValueId::new(2), ValueId::new(3)])
            .with_dst(ValueId::new(4));

        let func = make_function(
            1,
            "test",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
                AirParam::new(ValueId::new(3), 2),
            ],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![select, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Should have edges: v2 -> v4 and v3 -> v4 (condition is not propagated)
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));
        let n4 = NodeId::value(ValueId::new(4));

        let succs2 = graph.successors_of(n2).unwrap();
        assert!(succs2.contains(&(EdgeKind::DefUse, n4)));

        let succs3 = graph.successors_of(n3).unwrap();
        assert!(succs3.contains(&(EdgeKind::DefUse, n4)));
    }

    #[test]
    fn transform_from_binary_op() {
        // %3 = add %1, %2
        let add = Instruction::new(
            InstId::new(100),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(1), ValueId::new(2)])
        .with_dst(ValueId::new(3));

        let func = make_function(
            1,
            "test",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
            ],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![add, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Should have Transform edges: v1 -> v3 and v2 -> v3
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));
        let n3 = NodeId::value(ValueId::new(3));

        let succs1 = graph.successors_of(n1).unwrap();
        assert!(succs1.contains(&(EdgeKind::Transform, n3)));

        let succs2 = graph.successors_of(n2).unwrap();
        assert!(succs2.contains(&(EdgeKind::Transform, n3)));
    }

    #[test]
    fn transform_whitelist_filters() {
        // %3 = add %1, %2 (add should be filtered)
        let add = Instruction::new(
            InstId::new(100),
            Operation::BinaryOp {
                kind: BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(1), ValueId::new(2)])
        .with_dst(ValueId::new(3));

        let func = make_function(
            1,
            "test",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
            ],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![add, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);

        // Only allow Sub, not Add
        let mut ops = BTreeSet::new();
        ops.insert(OpKind::Sub);
        let config = ValueFlowConfig {
            transform_propagation: TransformPropagation::Whitelist { ops },
            ..ValueFlowConfig::fast()
        };

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Should NOT have any Transform edges (Add is filtered)
        let n1 = NodeId::value(ValueId::new(1));
        let succs = graph.successors_of(n1);

        if let Some(s) = succs {
            // Should not contain Transform edge to v3
            assert!(!s.iter().any(|(k, _)| *k == EdgeKind::Transform));
        }
    }

    #[test]
    fn memory_fast_mode() {
        // store %1 to %2, load from %3 to %4
        let store = Instruction::new(InstId::new(100), Operation::Store)
            .with_operands(vec![ValueId::new(1), ValueId::new(2)]);
        let load = Instruction::new(InstId::new(101), Operation::Load)
            .with_operands(vec![ValueId::new(3)])
            .with_dst(ValueId::new(4));

        let func = make_function(
            1,
            "test",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
                AirParam::new(ValueId::new(3), 2),
            ],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    store,
                    load,
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // In fast mode, all memory goes through unknown_mem
        let v1 = NodeId::value(ValueId::new(1));
        let v4 = NodeId::value(ValueId::new(4));
        let um = NodeId::unknown_mem();

        // Store: v1 -> unknown_mem
        let succs_v1 = graph.successors_of(v1).unwrap();
        assert!(succs_v1.contains(&(EdgeKind::Store, um)));

        // Load: unknown_mem -> v4
        let succs_um = graph.successors_of(um).unwrap();
        assert!(succs_um.contains(&(EdgeKind::Load, v4)));
    }

    #[test]
    fn copy_propagates() {
        // %2 = copy %1
        let copy = Instruction::new(InstId::new(100), Operation::Copy)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));

        let func = make_function(
            1,
            "test",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![copy, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Should have DefUse edge: v1 -> v2
        let n1 = NodeId::value(ValueId::new(1));
        let n2 = NodeId::value(ValueId::new(2));

        let succs = graph.successors_of(n1).unwrap();
        assert!(succs.contains(&(EdgeKind::DefUse, n2)));
    }

    #[test]
    fn call_arg_edges() {
        // callee(p0, p1) with body
        let callee = make_function(
            2,
            "callee",
            vec![
                AirParam::new(ValueId::new(10), 0),
                AirParam::new(ValueId::new(11), 1),
            ],
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![Instruction::new(InstId::new(200), Operation::Ret)],
            }],
        );

        // caller: call callee(%1, %2)
        let call = Instruction::new(
            InstId::new(100),
            Operation::CallDirect {
                callee: FunctionId::new(2),
            },
        )
        .with_operands(vec![ValueId::new(1), ValueId::new(2)]);

        let caller = make_function(
            1,
            "caller",
            vec![
                AirParam::new(ValueId::new(1), 0),
                AirParam::new(ValueId::new(2), 1),
            ],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![call, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![caller, callee]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // CallArg edges: v1 -> v10, v2 -> v11
        let v1 = NodeId::value(ValueId::new(1));
        let v2 = NodeId::value(ValueId::new(2));
        let v10 = NodeId::value(ValueId::new(10));
        let v11 = NodeId::value(ValueId::new(11));

        let succs_v1 = graph.successors_of(v1).unwrap();
        assert!(succs_v1.contains(&(EdgeKind::CallArg, v10)));

        let succs_v2 = graph.successors_of(v2).unwrap();
        assert!(succs_v2.contains(&(EdgeKind::CallArg, v11)));
    }

    #[test]
    fn return_edges() {
        // callee returns %10
        let callee = make_function(
            2,
            "callee",
            vec![],
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![
                    Instruction::new(InstId::new(200), Operation::Ret)
                        .with_operands(vec![ValueId::new(10)]),
                ],
            }],
        );

        // caller: %3 = call callee()
        let call = Instruction::new(
            InstId::new(100),
            Operation::CallDirect {
                callee: FunctionId::new(2),
            },
        )
        .with_dst(ValueId::new(3));

        let caller = make_function(
            1,
            "caller",
            vec![],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![call, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![caller, callee]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        // Return edge: v10 -> v3
        let v10 = NodeId::value(ValueId::new(10));
        let v3 = NodeId::value(ValueId::new(3));

        let succs = graph.successors_of(v10).unwrap();
        assert!(succs.contains(&(EdgeKind::Return, v3)));
    }

    #[test]
    fn indirect_call_counted() {
        let call_indirect = Instruction::new(
            InstId::new(100),
            Operation::CallIndirect {
                expected_signature: None,
            },
        )
        .with_operands(vec![ValueId::new(1)]); // function pointer

        let func = make_function(
            1,
            "test",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    call_indirect,
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            }],
        );

        let module = make_module(vec![func]);
        let defuse = DefUseGraph::build(&module);
        let callgraph = CallGraph::build(&module);
        let config = ValueFlowConfig::fast();

        let graph = build_valueflow(&config, &module, &defuse, &callgraph, None);

        assert_eq!(graph.diagnostics().indirect_calls, 1);
    }
}
