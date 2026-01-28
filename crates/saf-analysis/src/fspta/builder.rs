//! `FsSvfg` builder — annotates SVFG indirect edges with `LocId` object labels
//! and collects store/load metadata for the SFS solver.
//!
//! Replays the SVFG builder's clobber logic (Phases 1-3) to recover the
//! `LocId` that motivated each indirect edge. Direct edges are copied with
//! an empty object set.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{InstId, LocId, ValueId};

use crate::PtaResult;
use crate::callgraph::CallGraph;
use crate::mssa::{MemAccessId, MemoryAccess, MemorySsa};
use crate::svfg::{Svfg, SvfgEdgeKind, SvfgNodeId};

use super::{FsSvfg, FsSvfgEdge, LoadInfo, StoreInfo};

/// Builder for the object-labeled `FsSvfg`.
pub struct FsSvfgBuilder<'a> {
    module: &'a AirModule,
    svfg: &'a Svfg,
    pta: &'a PtaResult,
    mssa: &'a mut MemorySsa,
    #[allow(dead_code)]
    callgraph: &'a CallGraph,
}

impl<'a> FsSvfgBuilder<'a> {
    /// Create a new builder.
    #[must_use]
    pub fn new(
        module: &'a AirModule,
        svfg: &'a Svfg,
        pta: &'a PtaResult,
        mssa: &'a mut MemorySsa,
        callgraph: &'a CallGraph,
    ) -> Self {
        Self {
            module,
            svfg,
            pta,
            mssa,
            callgraph,
        }
    }

    /// Build the `FsSvfg`.
    ///
    /// Applies SVFG optimization (redundant `MemPhi` elimination) before
    /// constructing the object-labeled graph. Rewritten edges from optimized-away
    /// phis get empty object sets; the solver propagates all objects on such edges.
    pub fn build(mut self) -> FsSvfg {
        // Optimize SVFG — remove redundant MemPhi nodes before building FsSvfg.
        // This reduces the number of nodes the FSPTA solver must process.
        let optimized_svfg = crate::svfg::optimize::optimize(self.svfg);
        let svfg = &optimized_svfg;

        let mut fs = FsSvfg::new();

        // Step 1: Collect store and load metadata from the module
        let store_map = self.collect_store_map();
        let load_map = self.collect_load_map();

        // Step 2: Register store/load info on value nodes
        for (inst_id, (ptr, val)) in &store_map {
            // The stored value is the SVFG node for stores
            let node = SvfgNodeId::Value(*val);
            if svfg.contains_node(node) {
                fs.add_store_info(
                    node,
                    StoreInfo {
                        ptr: *ptr,
                        val: *val,
                    },
                );
            }
            // Also try the instruction's MSSA access → the store value node
            let _ = inst_id; // store_map is keyed by InstId for MSSA lookup
        }

        for (ptr, dst) in load_map.values() {
            let node = SvfgNodeId::Value(*dst);
            if svfg.contains_node(node) {
                fs.set_load_info(
                    node,
                    LoadInfo {
                        ptr: *ptr,
                        dst: *dst,
                    },
                );
            }
        }

        // Step 3: Replay indirect edges with LocId annotations
        // Build the MSSA store_map (MemAccessId → stored ValueId) for lookup
        let mssa_store_map = Self::collect_mssa_store_map(self.module, self.mssa);

        // Pre-compute store instruction → PTA locations to avoid O(n) scans
        let store_locs = self.precompute_store_locs();

        // Build per-indirect-edge object labels by replaying clobber logic
        let indirect_edge_objects =
            self.compute_indirect_edge_objects(&mssa_store_map, &load_map, &store_locs);

        // Step 4: Copy all edges from optimized SVFG into FsSvfg with annotations
        for node in svfg.nodes() {
            fs.add_node(*node);
            if let Some(succs) = svfg.successors_of(*node) {
                for (kind, target) in succs {
                    let objects = if kind.is_indirect() {
                        indirect_edge_objects
                            .get(&(*node, *kind, *target))
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        BTreeSet::new()
                    };
                    fs.add_edge(
                        *node,
                        FsSvfgEdge {
                            kind: *kind,
                            target: *target,
                            objects,
                        },
                    );
                }
            }
        }

        fs
    }

    /// Collect store instruction metadata: `InstId` → `(ptr, val)`.
    fn collect_store_map(&self) -> BTreeMap<InstId, (ValueId, ValueId)> {
        let mut map = BTreeMap::new();
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Store = &inst.op {
                        if inst.operands.len() >= 2 {
                            let val = inst.operands[0];
                            let ptr = inst.operands[1];
                            map.insert(inst.id, (ptr, val));
                        }
                    }
                }
            }
        }
        map
    }

    /// Collect load instruction metadata: `InstId` → `(ptr, dst)`.
    fn collect_load_map(&self) -> BTreeMap<InstId, (ValueId, ValueId)> {
        let mut map = BTreeMap::new();
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Load = &inst.op {
                        if let (Some(dst), Some(ptr)) = (inst.dst, inst.operands.first()) {
                            map.insert(inst.id, (*ptr, dst));
                        }
                    }
                }
            }
        }
        map
    }

    /// Collect MSSA store map: `MemAccessId` → stored `ValueId`.
    fn collect_mssa_store_map(
        module: &AirModule,
        mssa: &MemorySsa,
    ) -> BTreeMap<MemAccessId, ValueId> {
        let mut store_map = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Store = &inst.op {
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

    /// Compute object labels for each indirect SVFG edge.
    ///
    /// Replays the SVFG builder's Phase 2 (Phi edges) and Phase 3 (clobber
    /// edges) logic, recording which `LocId` motivated each edge.
    fn compute_indirect_edge_objects(
        &mut self,
        mssa_store_map: &BTreeMap<MemAccessId, ValueId>,
        load_map: &BTreeMap<InstId, (ValueId, ValueId)>,
        store_locs: &BTreeMap<InstId, Vec<LocId>>,
    ) -> BTreeMap<(SvfgNodeId, SvfgEdgeKind, SvfgNodeId), BTreeSet<LocId>> {
        let mut edge_objects: BTreeMap<(SvfgNodeId, SvfgEdgeKind, SvfgNodeId), BTreeSet<LocId>> =
            BTreeMap::new();

        // Phase 2 replay: Phi edges
        // For each MSSA Phi, operands that are store Defs produce IndirectStore edges.
        // We label these with ALL locations that the store pointer may point to.
        for access in self.mssa.accesses().values() {
            if let MemoryAccess::Phi { id, operands, .. } = access {
                let phi_node = SvfgNodeId::MemPhi(*id);
                for operand_id in operands.values() {
                    if let Some(operand_access) = self.mssa.access(*operand_id) {
                        match operand_access {
                            MemoryAccess::Def { inst, .. } => {
                                if let Some(&stored_val) = mssa_store_map.get(operand_id) {
                                    let src = SvfgNodeId::Value(stored_val);
                                    // O(1) lookup from pre-computed map
                                    if let Some(locs) = store_locs.get(inst) {
                                        edge_objects
                                            .entry((src, SvfgEdgeKind::IndirectStore, phi_node))
                                            .or_default()
                                            .extend(locs);
                                    }
                                }
                            }
                            MemoryAccess::Phi {
                                id: inner_phi_id, ..
                            } => {
                                let inner_phi = SvfgNodeId::MemPhi(*inner_phi_id);
                                // PhiFlow edges carry the union of all locations
                                // flowing through the phi chain. We label with all
                                // locations from the predecessor phi's mod-set.
                                // For simplicity, we use the empty set (PhiFlow
                                // passes through all objects in dfIn/dfOut).
                                edge_objects
                                    .entry((inner_phi, SvfgEdgeKind::PhiFlow, phi_node))
                                    .or_default();
                            }
                            MemoryAccess::LiveOnEntry { .. } | MemoryAccess::Use { .. } => {}
                        }
                    }
                }
            }
        }

        // Phase 3 replay: Clobber edges (store→load via MSSA clobber)
        // For each load, the SVFG builder queried clobber_for(use_id, loc)
        // for each loc in pts(load_ptr). The loc that produced the clobber
        // is the object label for that edge.
        let loads: Vec<(InstId, ValueId, ValueId)> = load_map
            .iter()
            .map(|(inst_id, (ptr, dst))| (*inst_id, *ptr, *dst))
            .collect();

        for (load_inst, load_ptr, load_result) in loads {
            let Some(use_id) = self.mssa.access_id_for(load_inst) else {
                continue;
            };
            let locations = self.pta.points_to(load_ptr);
            if locations.is_empty() {
                continue;
            }

            for loc in locations {
                let clobber_id = self.mssa.clobber_for(use_id, loc);
                let clobber_access = self.mssa.access(clobber_id).cloned();
                if let Some(clobber_access) = clobber_access {
                    match clobber_access {
                        MemoryAccess::Def { .. } => {
                            if let Some(&stored_val) = mssa_store_map.get(&clobber_id) {
                                let src = SvfgNodeId::Value(stored_val);
                                let dst = SvfgNodeId::Value(load_result);
                                edge_objects
                                    .entry((src, SvfgEdgeKind::IndirectDef, dst))
                                    .or_default()
                                    .insert(loc);
                            }
                        }
                        MemoryAccess::Phi { id, .. } => {
                            let phi = SvfgNodeId::MemPhi(id);
                            let dst = SvfgNodeId::Value(load_result);
                            edge_objects
                                .entry((phi, SvfgEdgeKind::IndirectLoad, dst))
                                .or_default()
                                .insert(loc);
                        }
                        MemoryAccess::LiveOnEntry { .. } | MemoryAccess::Use { .. } => {}
                    }
                }
            }
        }

        edge_objects
    }

    /// Pre-compute PTA locations for every store instruction's pointer operand.
    ///
    /// Returns a map from `InstId` to the `LocId` set that the store's pointer
    /// may point to. This replaces per-call O(n) linear scans with a single
    /// O(n) pass plus O(1) lookups.
    fn precompute_store_locs(&self) -> BTreeMap<InstId, Vec<LocId>> {
        let mut map = BTreeMap::new();
        for func in &self.module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Operation::Store = &inst.op {
                        if inst.operands.len() >= 2 {
                            let ptr = inst.operands[1];
                            let locs = self.pta.points_to(ptr);
                            map.insert(inst.id, locs);
                        }
                    }
                }
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction};
    use saf_core::ids::{BlockId, FunctionId, ModuleId};

    use crate::callgraph::CallGraph;
    use crate::cfg::Cfg;
    use crate::defuse::DefUseGraph;
    use crate::pta::{PtaConfig, PtaContext};
    use crate::svfg::SvfgBuilder;

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

    fn build_pipeline(module: &AirModule) -> (Svfg, PtaResult, MemorySsa, CallGraph) {
        let defuse = DefUseGraph::build(module);
        let callgraph = CallGraph::build(module);
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
        let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);
        let (svfg, _program_points) =
            SvfgBuilder::new(module, &defuse, &callgraph, &pta_result, &mut mssa).build();

        // Build a third PtaResult for the FsSvfg builder (MSSA consumed one)
        let mut pta_ctx3 = PtaContext::new(PtaConfig::default());
        let pta_raw3 = pta_ctx3.analyze(module);
        let pta_for_fs = PtaResult::new(
            pta_raw3.pts,
            Arc::new(pta_raw3.factory),
            pta_raw3.diagnostics,
        );

        let mut pta_ctx4 = PtaContext::new(PtaConfig::default());
        let pta_raw4 = pta_ctx4.analyze(module);
        let mssa_pta2 = PtaResult::new(
            pta_raw4.pts,
            Arc::new(pta_raw4.factory),
            pta_raw4.diagnostics,
        );
        let mssa2 = MemorySsa::build(module, &cfgs, mssa_pta2, &callgraph);

        (svfg, pta_for_fs, mssa2, callgraph)
    }

    #[test]
    fn fs_svfg_builds_from_store_load() {
        let alloca = Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(10));
        let store = Instruction::new(InstId::new(101), Operation::Store)
            .with_operands(vec![ValueId::new(1), ValueId::new(10)]);
        let load = Instruction::new(InstId::new(102), Operation::Load)
            .with_operands(vec![ValueId::new(10)])
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
        let (svfg, pta, mut mssa, callgraph) = build_pipeline(&module);

        let fs = FsSvfgBuilder::new(&module, &svfg, &pta, &mut mssa, &callgraph).build();

        // Should have at least as many nodes as the SVFG
        assert!(fs.node_count() > 0);
        // Should have edges
        assert!(fs.edge_count() > 0);
    }

    #[test]
    fn fs_svfg_copies_all_svfg_nodes() {
        let alloca = Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(10));
        let store = Instruction::new(InstId::new(101), Operation::Store)
            .with_operands(vec![ValueId::new(1), ValueId::new(10)]);
        let load = Instruction::new(InstId::new(102), Operation::Load)
            .with_operands(vec![ValueId::new(10)])
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
        let (svfg, pta, mut mssa, callgraph) = build_pipeline(&module);

        let fs = FsSvfgBuilder::new(&module, &svfg, &pta, &mut mssa, &callgraph).build();

        // FsSvfg should contain all nodes from the base SVFG
        for node in svfg.nodes() {
            assert!(
                fs.nodes().contains(node),
                "FsSvfg missing SVFG node {:?}",
                node
            );
        }
    }
}
