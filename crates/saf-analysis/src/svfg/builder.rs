//! SVFG builder — 5-phase construction algorithm.
//!
//! Phase 1: Collect store→value mappings from MSSA `Def`s.
//! Phase 2: Build Memory `Phi` edges (`IndirectStore`, `PhiFlow`).
//! Phase 3: Build indirect store→load edges via MSSA clobber queries.
//! Phase 4: Build direct edges (`DirectDef`, `DirectTransform`, `CallArg`, `Return`).
//! Phase 5: Build interprocedural indirect edges for `CallArg` pointer arguments.

use std::collections::BTreeMap;

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, InstId, ValueId};

use crate::PtaResult;
use crate::callgraph::{CallGraph, CallGraphNode};
use crate::defuse::DefUseGraph;
use crate::mssa::{MemAccessId, MemoryAccess, MemorySsa};

use super::program_point::{ProgramPoint, ProgramPointMap};
use super::{Svfg, SvfgEdgeKind, SvfgNodeId};

/// Builder for constructing a Sparse Value-Flow Graph.
pub struct SvfgBuilder<'a> {
    module: &'a AirModule,
    #[allow(dead_code)]
    defuse: &'a DefUseGraph,
    callgraph: &'a CallGraph,
    pta: &'a PtaResult,
    mssa: &'a mut MemorySsa,
}

impl<'a> SvfgBuilder<'a> {
    /// Create a new SVFG builder.
    #[must_use]
    pub fn new(
        module: &'a AirModule,
        defuse: &'a DefUseGraph,
        callgraph: &'a CallGraph,
        pta: &'a PtaResult,
        mssa: &'a mut MemorySsa,
    ) -> Self {
        Self {
            module,
            defuse,
            callgraph,
            pta,
            mssa,
        }
    }

    /// Build the SVFG using 4-phase construction.
    ///
    /// Returns the SVFG and a `ProgramPointMap` tracking where each value is defined.
    /// The program point map enables temporal ordering queries for UAF filtering.
    pub fn build(self) -> (Svfg, ProgramPointMap) {
        let mut graph = Svfg::new();
        let mut program_points = ProgramPointMap::new();

        // Build program point map during instruction iteration
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for (inst_idx, inst) in block.instructions.iter().enumerate() {
                    let pp = ProgramPoint::new(
                        func.id,
                        block.id,
                        // INVARIANT: Basic blocks in real programs have < 2^32 instructions
                        #[allow(clippy::cast_possible_truncation)]
                        (inst_idx as u32),
                    );

                    // Track the defining location for instruction results
                    if let Some(dst) = inst.dst {
                        program_points.insert(dst, pp);
                    }

                    // Track operands for parameters/globals (first use location)
                    for &operand in &inst.operands {
                        program_points.insert(operand, pp);
                    }
                }
            }
        }

        // Track function parameters at their function's entry
        // This includes BOTH definitions and declarations - declarations have params
        // that can appear in SVFG CallArg edges, so we need program points for them too.
        for func in &self.module.functions {
            let entry_block = func.entry_block.unwrap_or_else(|| {
                // For definitions, use the first block; for declarations, use a synthetic block ID
                func.blocks
                    .first()
                    .map_or_else(|| saf_core::ids::BlockId::new(0), |b| b.id)
            });
            let entry_pp = ProgramPoint::new(func.id, entry_block, 0);
            for param in &func.params {
                program_points.insert(param.id, entry_pp);
            }
        }

        // Track global variable addresses
        // These can appear in SVFG edges and need program points for temporal ordering.
        // Use a synthetic "module-level" program point (FunctionId 0, BlockId 0, inst 0).
        let global_pp = ProgramPoint::new(FunctionId::new(0), saf_core::ids::BlockId::new(0), 0);
        for global in &self.module.globals {
            program_points.insert(global.id, global_pp);
        }

        // Track constants
        // These can appear as SVFG operands and need program points.
        for &const_vid in self.module.constants.keys() {
            program_points.insert(const_vid, global_pp);
        }

        // Phase 1: Collect store→value mappings
        let store_map = Self::collect_store_map(self.module, self.mssa);

        // Phase 2: Build Memory Phi edges
        Self::build_phi_edges_static(self.module, self.mssa, &mut graph, &store_map);

        // Phase 3: Build indirect store→load edges via clobber queries
        Self::build_clobber_edges_static(self.module, self.mssa, self.pta, &mut graph, &store_map);

        // Phase 4: Build direct edges
        self.build_direct_edges(&mut graph);

        // Phase 5: Build interprocedural indirect edges for pointer arguments.
        // When a pointer is passed via CallArg, stores through that pointer in
        // the caller should flow to loads through the formal parameter in the
        // callee.  MSSA is intraprocedural and misses this cross-function link.
        Self::build_callarg_indirect_edges(self.module, self.pta, &mut graph);

        (graph, program_points)
    }

    // -----------------------------------------------------------------------
    // Phase 1: Collect store→value mappings
    // -----------------------------------------------------------------------

    /// Scan all store instructions, record `MemAccessId` → `ValueId` mapping
    /// (each MSSA `Def` from a store → the stored value).
    fn collect_store_map(module: &AirModule, mssa: &MemorySsa) -> BTreeMap<MemAccessId, ValueId> {
        let mut store_map = BTreeMap::new();

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Store = &inst.op {
                        // operands: [value, pointer]
                        if inst.operands.len() >= 2 {
                            let stored_value = inst.operands[0];
                            if let Some(access_id) = mssa.access_id_for(inst.id) {
                                store_map.insert(access_id, stored_value);
                            }
                        }
                    }
                }
            }
        }

        store_map
    }

    // -----------------------------------------------------------------------
    // Phase 2: Build Memory Phi edges
    // -----------------------------------------------------------------------

    /// For each MSSA `Phi` node, walk operands and create `IndirectStore`/`PhiFlow` edges.
    ///
    /// When a predecessor block ends with a `CondBr` targeting the Phi's block,
    /// the extracted guard is attached to the SVFG edge.
    fn build_phi_edges_static(
        module: &AirModule,
        mssa: &MemorySsa,
        graph: &mut Svfg,
        store_map: &BTreeMap<MemAccessId, ValueId>,
    ) {
        // Build lookup maps: BlockId → &AirBlock and BlockId → FunctionId
        let mut block_map: BTreeMap<saf_core::ids::BlockId, &saf_core::air::AirBlock> =
            BTreeMap::new();
        let mut block_func_map: BTreeMap<saf_core::ids::BlockId, FunctionId> = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                block_map.insert(block.id, block);
                block_func_map.insert(block.id, func.id);
            }
        }

        for access in mssa.accesses().values() {
            if let MemoryAccess::Phi {
                id,
                block: phi_block,
                operands,
            } = access
            {
                let phi_node = SvfgNodeId::MemPhi(*id);
                graph.add_node(phi_node);
                graph.diagnostics_mut().mem_phi_count += 1;

                for (&pred_block_id, operand_id) in operands {
                    if let Some(operand_access) = mssa.access(*operand_id) {
                        // Extract guard from predecessor block's CondBr terminator
                        let guard = block_map.get(&pred_block_id).and_then(|pred_block| {
                            let func_id = block_func_map.get(&pred_block_id)?;
                            extract_cond_br_guard(pred_block, *func_id, *phi_block)
                        });

                        match operand_access {
                            MemoryAccess::Def { .. } => {
                                // Operand is a store Def → IndirectStore edge
                                if let Some(&stored_val) = store_map.get(operand_id) {
                                    let from = SvfgNodeId::Value(stored_val);
                                    graph.add_edge(from, SvfgEdgeKind::IndirectStore, phi_node);
                                    if let Some(g) = guard {
                                        graph.set_edge_guard(from, phi_node, vec![g]);
                                    }
                                    graph.diagnostics_mut().indirect_edge_count += 1;
                                }
                                // If Def is a call (not in store_map), skip per E12 scope
                            }
                            MemoryAccess::Phi {
                                id: inner_phi_id, ..
                            } => {
                                // Operand is another Phi → PhiFlow edge
                                let inner_phi_node = SvfgNodeId::MemPhi(*inner_phi_id);
                                graph.add_edge(inner_phi_node, SvfgEdgeKind::PhiFlow, phi_node);
                                if let Some(g) = guard {
                                    graph.set_edge_guard(inner_phi_node, phi_node, vec![g]);
                                }
                                graph.diagnostics_mut().indirect_edge_count += 1;
                            }
                            MemoryAccess::LiveOnEntry { .. } => {
                                // Skip LiveOnEntry operands
                                graph.diagnostics_mut().skipped_live_on_entry += 1;
                            }
                            MemoryAccess::Use { .. } => {
                                // Uses shouldn't appear as Phi operands normally
                            }
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Phase 3: Build indirect store→load edges via clobber queries
    // -----------------------------------------------------------------------

    /// For each load instruction, query MSSA clobber and create indirect edges.
    fn build_clobber_edges_static(
        module: &AirModule,
        mssa: &mut MemorySsa,
        pta: &PtaResult,
        graph: &mut Svfg,
        store_map: &BTreeMap<MemAccessId, ValueId>,
    ) {
        // Collect load info first to avoid borrow conflicts
        let mut loads: Vec<(InstId, ValueId, ValueId)> = Vec::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Load = &inst.op {
                        if let (Some(result), Some(ptr)) = (inst.dst, inst.operands.first()) {
                            loads.push((inst.id, result, *ptr));
                        }
                    }
                }
            }
        }

        for (load_inst, load_result, load_ptr) in loads {
            Self::process_load_static(
                mssa,
                pta,
                graph,
                store_map,
                load_inst,
                load_result,
                load_ptr,
            );
        }
    }

    /// Process a single load instruction: query MSSA clobber for each PTA location.
    fn process_load_static(
        mssa: &mut MemorySsa,
        pta: &PtaResult,
        graph: &mut Svfg,
        store_map: &BTreeMap<MemAccessId, ValueId>,
        load_inst: InstId,
        load_result: ValueId,
        load_ptr: ValueId,
    ) {
        // Get the load's MSSA Use access
        let Some(use_id) = mssa.access_id_for(load_inst) else {
            return;
        };

        // Get locations this pointer may point to
        let locations = pta.points_to(load_ptr);
        if locations.is_empty() {
            return;
        }

        for loc in locations {
            // Query clobber: who last wrote to this location before this load?
            let clobber_id = mssa.clobber_for(use_id, loc);

            // Clone the access to avoid borrow conflict with mssa
            let clobber_access = mssa.access(clobber_id).cloned();
            if let Some(clobber_access) = clobber_access {
                match clobber_access {
                    MemoryAccess::Def { .. } => {
                        // Clobber is a store → IndirectDef
                        if let Some(&stored_val) = store_map.get(&clobber_id) {
                            graph.add_edge(
                                SvfgNodeId::Value(stored_val),
                                SvfgEdgeKind::IndirectDef,
                                SvfgNodeId::Value(load_result),
                            );
                            graph.diagnostics_mut().indirect_edge_count += 1;
                        } else {
                            // Clobber is a call Def → skip per E12 scope
                            graph.diagnostics_mut().skipped_call_clobbers += 1;
                        }
                    }
                    MemoryAccess::Phi { id, .. } => {
                        // Clobber is a Phi → IndirectLoad edge
                        graph.add_edge(
                            SvfgNodeId::MemPhi(id),
                            SvfgEdgeKind::IndirectLoad,
                            SvfgNodeId::Value(load_result),
                        );
                        graph.diagnostics_mut().indirect_edge_count += 1;
                    }
                    MemoryAccess::LiveOnEntry { .. } => {
                        graph.diagnostics_mut().skipped_live_on_entry += 1;
                    }
                    MemoryAccess::Use { .. } => {
                        // Shouldn't happen — clobber should be Def/Phi/LiveOnEntry
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Phase 5: Build interprocedural indirect edges via CallArg pointers
    // -----------------------------------------------------------------------

    /// For each `CallArg` edge where the actual argument is a pointer,
    /// connect values stored through that pointer in the caller to values
    /// loaded through the corresponding formal parameter in the callee.
    ///
    /// This handles patterns like:
    /// - Variant 63: `sink(&data)` where sink does `free(*dataPtr)`
    /// - Variant 64: `sink((void*)&data)` where sink casts back and frees
    /// - Variant 66: `sink(dataArray)` where sink reads `dataArray[2]`
    fn build_callarg_indirect_edges(module: &AirModule, pta: &PtaResult, graph: &mut Svfg) {
        use std::collections::BTreeSet;

        // Build per-function store/load indexes and lookup maps.
        let (stores_by_func, loads_by_func, inst_to_func, param_to_func) =
            Self::collect_callarg_indexes(module);

        // Collect all CallArg edges from the SVFG.
        let callarg_edges: Vec<(ValueId, ValueId, InstId)> = graph
            .nodes()
            .iter()
            .filter_map(SvfgNodeId::as_value)
            .flat_map(|vid| {
                let succs = graph.successors_of_value(vid);
                succs
                    .into_iter()
                    .flat_map(|s| s.iter().copied().collect::<Vec<_>>())
                    .filter_map(move |(kind, target)| {
                        if let SvfgEdgeKind::CallArg { call_site } = kind {
                            target.as_value().map(|tv| (vid, tv, call_site))
                        } else {
                            None
                        }
                    })
            })
            .collect();

        // For each CallArg, connect stores in caller to loads in callee.
        let mut added = 0usize;
        for (actual, formal, call_site) in &callarg_edges {
            let pts_actual: BTreeSet<_> = pta.points_to(*actual).into_iter().collect();
            if pts_actual.is_empty() {
                continue;
            }
            let Some(&src_func) = inst_to_func.get(call_site) else {
                continue;
            };
            let Some(&dst_func) = param_to_func.get(formal) else {
                continue;
            };
            let pts_formal: BTreeSet<_> = pta.points_to(*formal).into_iter().collect();
            if pts_formal.is_empty() {
                continue;
            }

            let stores = stores_by_func.get(&src_func);
            let loads = loads_by_func.get(&dst_func);
            if let (Some(stores), Some(loads)) = (stores, loads) {
                added += Self::connect_stores_to_loads(
                    pta,
                    graph,
                    stores,
                    loads,
                    &pts_actual,
                    &pts_formal,
                );
            }
        }

        graph.diagnostics_mut().indirect_edge_count += added;
    }

    /// Build per-function store/load indexes and inst→func / param→func maps.
    #[allow(clippy::type_complexity)]
    fn collect_callarg_indexes(
        module: &AirModule,
    ) -> (
        BTreeMap<FunctionId, Vec<(ValueId, ValueId)>>,
        BTreeMap<FunctionId, Vec<(ValueId, ValueId)>>,
        BTreeMap<InstId, FunctionId>,
        BTreeMap<ValueId, FunctionId>,
    ) {
        let mut stores_by_func: BTreeMap<FunctionId, Vec<(ValueId, ValueId)>> = BTreeMap::new();
        let mut loads_by_func: BTreeMap<FunctionId, Vec<(ValueId, ValueId)>> = BTreeMap::new();
        let mut inst_to_func: BTreeMap<InstId, FunctionId> = BTreeMap::new();
        let mut param_to_func: BTreeMap<ValueId, FunctionId> = BTreeMap::new();

        for func in &module.functions {
            for param in &func.params {
                param_to_func.insert(param.id, func.id);
            }
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    inst_to_func.insert(inst.id, func.id);
                    match &inst.op {
                        Operation::Store if inst.operands.len() >= 2 => {
                            stores_by_func
                                .entry(func.id)
                                .or_default()
                                .push((inst.operands[0], inst.operands[1]));
                        }
                        Operation::Load => {
                            if let (Some(result), Some(&ptr)) = (inst.dst, inst.operands.first()) {
                                loads_by_func
                                    .entry(func.id)
                                    .or_default()
                                    .push((result, ptr));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        (stores_by_func, loads_by_func, inst_to_func, param_to_func)
    }

    /// Connect matching stores in a caller to loads in a callee via PTA alias.
    fn connect_stores_to_loads(
        pta: &PtaResult,
        graph: &mut Svfg,
        stores: &[(ValueId, ValueId)],
        loads: &[(ValueId, ValueId)],
        pts_actual: &std::collections::BTreeSet<saf_core::ids::LocId>,
        pts_formal: &std::collections::BTreeSet<saf_core::ids::LocId>,
    ) -> usize {
        use std::collections::BTreeSet;
        let mut added = 0;

        let store_values: Vec<ValueId> = stores
            .iter()
            .filter(|(_, store_ptr)| {
                let locs: BTreeSet<_> = pta.points_to(*store_ptr).into_iter().collect();
                !locs.is_disjoint(pts_actual)
            })
            .map(|(val, _)| *val)
            .collect();

        let load_results: Vec<ValueId> = loads
            .iter()
            .filter(|(_, load_ptr)| {
                let locs: BTreeSet<_> = pta.points_to(*load_ptr).into_iter().collect();
                !locs.is_disjoint(pts_formal)
            })
            .map(|(result, _)| *result)
            .collect();

        for &sv in &store_values {
            for &lr in &load_results {
                graph.add_edge(
                    SvfgNodeId::Value(sv),
                    SvfgEdgeKind::IndirectDef,
                    SvfgNodeId::Value(lr),
                );
                added += 1;
            }
        }

        added
    }

    // -----------------------------------------------------------------------
    // Phase 4: Build direct edges
    // -----------------------------------------------------------------------

    /// Scan all instructions and create direct edges (same logic as `ValueFlow`
    /// builder, but producing `SvfgEdgeKind` variants).
    fn build_direct_edges(&self, graph: &mut Svfg) {
        // Build block lookup map for guard extraction in Phi instructions
        let mut block_map: BTreeMap<saf_core::ids::BlockId, &saf_core::air::AirBlock> =
            BTreeMap::new();
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                block_map.insert(block.id, block);
            }
        }

        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    self.process_direct_instruction(
                        graph,
                        func.id,
                        block.id,
                        inst.id,
                        &inst.op,
                        &inst.operands,
                        inst.dst,
                        &block_map,
                    );
                }
            }
        }
    }

    /// Process a single instruction for direct edges.
    ///
    /// For `Phi` instructions, extracts `CondBr` guards from predecessor blocks
    /// and attaches them to the corresponding `DirectDef` edges.
    // NOTE: The extra block_id + block_map parameters are needed to extract
    // guards from predecessor CondBr terminators during Phi edge construction.
    #[allow(clippy::too_many_arguments)]
    fn process_direct_instruction(
        &self,
        graph: &mut Svfg,
        func_id: FunctionId,
        block_id: saf_core::ids::BlockId,
        inst_id: InstId,
        op: &Operation,
        operands: &[ValueId],
        dst: Option<ValueId>,
        block_map: &BTreeMap<saf_core::ids::BlockId, &saf_core::air::AirBlock>,
    ) {
        match op {
            // SSA Phi
            Operation::Phi { incoming } => {
                if let Some(result) = dst {
                    for (pred_block_id, value) in incoming {
                        let from = SvfgNodeId::Value(*value);
                        let to = SvfgNodeId::Value(result);
                        graph.add_edge(from, SvfgEdgeKind::DirectDef, to);

                        // Extract guard from predecessor block's CondBr terminator
                        if let Some(pred_block) = block_map.get(pred_block_id) {
                            if let Some(guard) =
                                extract_cond_br_guard(pred_block, func_id, block_id)
                            {
                                graph.set_edge_guard(from, to, vec![guard]);
                            }
                        }

                        graph.diagnostics_mut().direct_edge_count += 1;
                    }
                }
            }

            // Select: both branches flow to result
            Operation::Select => {
                if let Some(result) = dst {
                    if operands.len() >= 3 {
                        graph.add_edge(
                            SvfgNodeId::Value(operands[1]),
                            SvfgEdgeKind::DirectDef,
                            SvfgNodeId::Value(result),
                        );
                        graph.add_edge(
                            SvfgNodeId::Value(operands[2]),
                            SvfgEdgeKind::DirectDef,
                            SvfgNodeId::Value(result),
                        );
                        graph.diagnostics_mut().direct_edge_count += 2;
                    }
                }
            }

            // Transform operations
            Operation::BinaryOp { .. } => {
                if let Some(result) = dst {
                    for operand in operands {
                        graph.add_edge(
                            SvfgNodeId::Value(*operand),
                            SvfgEdgeKind::DirectTransform,
                            SvfgNodeId::Value(result),
                        );
                        graph.diagnostics_mut().direct_edge_count += 1;
                    }
                }
            }

            Operation::Cast { .. } | Operation::Gep { .. } => {
                if let Some(result) = dst {
                    if let Some(src) = operands.first() {
                        graph.add_edge(
                            SvfgNodeId::Value(*src),
                            SvfgEdgeKind::DirectTransform,
                            SvfgNodeId::Value(result),
                        );
                        graph.diagnostics_mut().direct_edge_count += 1;
                    }
                }
            }

            // Copy / Freeze
            Operation::Copy | Operation::Freeze => {
                if let Some(result) = dst {
                    if let Some(src) = operands.first() {
                        graph.add_edge(
                            SvfgNodeId::Value(*src),
                            SvfgEdgeKind::DirectDef,
                            SvfgNodeId::Value(result),
                        );
                        graph.diagnostics_mut().direct_edge_count += 1;
                    }
                }
            }

            // Call — CallArg + Return edges
            Operation::CallDirect { callee } => {
                self.add_call_edges(graph, func_id, inst_id, Some(*callee), operands, dst);
            }

            Operation::CallIndirect { .. } => {
                // Last operand is the function pointer (callee-LAST convention);
                // all preceding operands are arguments.
                if !operands.is_empty() {
                    let args = &operands[..operands.len() - 1];
                    self.add_call_edges(graph, func_id, inst_id, None, args, dst);
                }
            }

            // HeapAlloc: the return value is a fresh allocation pointer.
            // Ensure it exists in the SVFG so checkers (e.g., memory-leak)
            // can reason about it even when the pointer has no further flow.
            Operation::HeapAlloc { .. } => {
                if let Some(result) = dst {
                    graph.add_node(SvfgNodeId::Value(result));
                }
            }

            // Store, Load — handled in phases 1-3 (indirect edges)
            // Control flow and memory intrinsics — no direct value flow
            Operation::Store
            | Operation::Load
            | Operation::Alloca { .. }
            | Operation::Global { .. }
            | Operation::Memcpy
            | Operation::Memset
            | Operation::Br { .. }
            | Operation::CondBr { .. }
            | Operation::Switch { .. }
            | Operation::Ret
            | Operation::Unreachable => {}
        }
    }

    /// Add call argument and return edges.
    fn add_call_edges(
        &self,
        graph: &mut Svfg,
        caller_func: FunctionId,
        call_site: InstId,
        callee_id: Option<FunctionId>,
        args: &[ValueId],
        call_result: Option<ValueId>,
    ) {
        let target_funcs: Vec<FunctionId> = if let Some(callee) = callee_id {
            vec![callee]
        } else {
            // Indirect call — look up the call-site target first.
            // If PTA has resolved targets, they appear as `Function`/`External`
            // edges from the caller node in the callgraph (added by
            // `resolve_indirect`). When `call_site_target` returns an
            // `IndirectPlaceholder`, collect resolved `Function`/`External`
            // callees of the caller instead.
            let mut resolved = Vec::new();
            if let Some(target) = self.callgraph.call_site_target(call_site) {
                match target {
                    CallGraphNode::Function(id) | CallGraphNode::External { func: id, .. } => {
                        resolved.push(*id);
                    }
                    CallGraphNode::IndirectPlaceholder { .. } => {
                        // Placeholder not resolved to a single target --
                        // gather all resolved `Function`/`External` callees
                        // of the caller via the edge map.
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

        for target_func_id in target_funcs {
            if let Some(target_func) = self.module.function(target_func_id) {
                // CallArg: actual → formal
                for (i, actual) in args.iter().enumerate() {
                    if let Some(param) = target_func.params.get(i) {
                        graph.add_edge(
                            SvfgNodeId::Value(*actual),
                            SvfgEdgeKind::CallArg { call_site },
                            SvfgNodeId::Value(param.id),
                        );
                        graph.diagnostics_mut().direct_edge_count += 1;
                    }
                }

                // Return: callee ret → caller result
                if let Some(result) = call_result {
                    let mut found_return = false;
                    for block in &target_func.blocks {
                        for inst in &block.instructions {
                            if let Operation::Ret = &inst.op {
                                if let Some(ret_val) = inst.operands.first() {
                                    graph.add_edge(
                                        SvfgNodeId::Value(*ret_val),
                                        SvfgEdgeKind::Return { call_site },
                                        SvfgNodeId::Value(result),
                                    );
                                    graph.diagnostics_mut().direct_edge_count += 1;
                                    found_return = true;
                                }
                            }
                        }
                    }
                    // For declarations (no blocks) or functions whose returns
                    // carry no value: ensure the call result exists in the
                    // SVFG so checkers can reason about it.
                    if !found_return {
                        graph.add_node(SvfgNodeId::Value(result));
                    }
                }
            }
        }
    }
}

/// Extract a guard from a block's `CondBr` terminator for a specific successor target.
///
/// If the block's last instruction is a `CondBr`, and the given `target` matches
/// either the `then_target` or `else_target`, returns a `Guard` with the appropriate
/// `branch_taken` flag. Returns `None` if the block doesn't end with `CondBr` or
/// if `target` doesn't match either successor.
fn extract_cond_br_guard(
    block: &saf_core::air::AirBlock,
    func_id: FunctionId,
    target: saf_core::ids::BlockId,
) -> Option<crate::guard::Guard> {
    let term = block.instructions.last()?;
    if let Operation::CondBr {
        then_target,
        else_target,
    } = &term.op
    {
        let cond_id = *term.operands.first()?;
        if target == *then_target {
            Some(crate::guard::Guard {
                block: block.id,
                function: func_id,
                condition: cond_id,
                branch_taken: true,
            })
        } else if target == *else_target {
            Some(crate::guard::Guard {
                block: block.id,
                function: func_id,
                condition: cond_id,
                branch_taken: false,
            })
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, BinaryOp, Instruction};
    use saf_core::ids::{BlockId, ModuleId};

    use crate::callgraph::CallGraph;
    use crate::cfg::Cfg;
    use crate::defuse::DefUseGraph;
    use crate::pta::{PtaConfig, PtaContext};

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

    fn build_analysis(module: &AirModule) -> (DefUseGraph, CallGraph, PtaResult, MemorySsa) {
        let defuse = DefUseGraph::build(module);
        let callgraph = CallGraph::build(module);

        // Run PTA twice: once for SVFG queries, once for MSSA
        let pta_config = PtaConfig::default();
        let mut pta_ctx = PtaContext::new(pta_config.clone());
        let pta_raw = pta_ctx.analyze(module);
        let pta_result =
            PtaResult::new(pta_raw.pts, Arc::new(pta_raw.factory), pta_raw.diagnostics);

        let mut pta_ctx2 = PtaContext::new(pta_config);
        let pta_raw2 = pta_ctx2.analyze(module);
        let mssa_pta = PtaResult::new(
            pta_raw2.pts,
            Arc::new(pta_raw2.factory),
            pta_raw2.diagnostics,
        );

        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();
        let mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);

        (defuse, callgraph, pta_result, mssa)
    }

    #[test]
    fn direct_def_from_phi() {
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
        let (defuse, callgraph, pta, mut mssa) = build_analysis(&module);

        let svfg = {
            let (svfg, _pp) =
                SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();
            svfg
        };

        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v3 = SvfgNodeId::value(ValueId::new(3));

        let succs = svfg.successors_of(v1).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::DirectDef, v3)));
    }

    #[test]
    fn direct_transform_from_binop() {
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
        let (defuse, callgraph, pta, mut mssa) = build_analysis(&module);

        let svfg = {
            let (svfg, _pp) =
                SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();
            svfg
        };

        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v3 = SvfgNodeId::value(ValueId::new(3));

        let succs = svfg.successors_of(v1).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::DirectTransform, v3)));
        assert!(svfg.diagnostics().direct_edge_count > 0);
    }

    #[test]
    fn call_arg_and_return_edges() {
        let callee = make_function(
            2,
            "callee",
            vec![AirParam::new(ValueId::new(10), 0)],
            vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![
                    Instruction::new(InstId::new(200), Operation::Ret)
                        .with_operands(vec![ValueId::new(10)]),
                ],
            }],
        );

        let call = Instruction::new(
            InstId::new(100),
            Operation::CallDirect {
                callee: FunctionId::new(2),
            },
        )
        .with_operands(vec![ValueId::new(1)])
        .with_dst(ValueId::new(3));

        let caller = make_function(
            1,
            "caller",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![call, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );

        let module = make_module(vec![caller, callee]);
        let (defuse, callgraph, pta, mut mssa) = build_analysis(&module);

        let svfg = {
            let (svfg, _pp) =
                SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();
            svfg
        };

        // CallArg: v1 → v10
        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v10 = SvfgNodeId::value(ValueId::new(10));
        let v3 = SvfgNodeId::value(ValueId::new(3));

        let succs_v1 = svfg.successors_of(v1).unwrap();
        assert!(
            succs_v1.iter().any(
                |(kind, target)| matches!(kind, SvfgEdgeKind::CallArg { .. }) && *target == v10
            )
        );

        // Return: v10 → v3
        let succs_v10 = svfg.successors_of(v10).unwrap();
        assert!(
            succs_v10
                .iter()
                .any(|(kind, target)| matches!(kind, SvfgEdgeKind::Return { .. }) && *target == v3)
        );
    }

    #[test]
    fn copy_produces_direct_def() {
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
        let (defuse, callgraph, pta, mut mssa) = build_analysis(&module);

        let svfg = {
            let (svfg, _pp) =
                SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();
            svfg
        };

        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v2 = SvfgNodeId::value(ValueId::new(2));
        let succs = svfg.successors_of(v1).unwrap();
        assert!(succs.contains(&(SvfgEdgeKind::DirectDef, v2)));
    }

    #[test]
    fn store_load_creates_indirect_edge() {
        // alloca %ptr; store %val to %ptr; %result = load %ptr
        let alloca = Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(10)); // %ptr

        let store = Instruction::new(InstId::new(101), Operation::Store)
            .with_operands(vec![ValueId::new(1), ValueId::new(10)]); // store %val, %ptr

        let load = Instruction::new(InstId::new(102), Operation::Load)
            .with_operands(vec![ValueId::new(10)]) // load %ptr
            .with_dst(ValueId::new(20));

        let func = make_function(
            1,
            "test",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    alloca,
                    store,
                    load,
                    Instruction::new(InstId::new(103), Operation::Ret),
                ],
            }],
        );

        let module = make_module(vec![func]);
        let (defuse, callgraph, pta, mut mssa) = build_analysis(&module);

        let svfg = {
            let (svfg, _pp) =
                SvfgBuilder::new(&module, &defuse, &callgraph, &pta, &mut mssa).build();
            svfg
        };

        // Should have some indirect edges (exact count depends on PTA results)
        // At minimum, the graph should build without panic and have nodes
        assert!(svfg.node_count() > 0);
        assert!(svfg.edge_count() > 0);
    }
}
