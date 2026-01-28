//! SFS worklist solver for flow-sensitive pointer analysis.
//!
//! Implements the core propagation loop:
//! 1. Seed top-level `points_to` from Andersen pre-analysis
//! 2. Process nodes from a deterministic worklist (`BTreeSet`)
//! 3. Store nodes: update `dfOut` with strong or weak update
//! 4. Load nodes: read from `dfIn` into top-level `points_to`
//! 5. Propagate: direct edges update `points_to`, indirect edges update `dfIn`/`dfOut`
//! 6. Converge when no new facts are discovered
//!
//! The solver is generic over `PtsSet` representations, allowing different
//! backing data structures (`BTreePtsSet`, `RoaringPtsSet`, `BddPtsSet`)
//! without changing algorithm logic. Results are converted to `BTreeSet<LocId>`
//! at the solver boundary for API stability.

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;

use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, LocId, ValueId};

use crate::PtaResult;
use crate::callgraph::CallGraph;
use crate::pta::ptsset::{BTreePtsSet, PtsSet};
use crate::svfg::SvfgNodeId;

use super::StoreInfo;
use super::strong_update::StrongUpdateInfo;
use super::version_table::{NodeVersionMap, VersionTable};
use super::{FlowSensitivePtaResult, FsPtaConfig, FsPtaDiagnostics, FsSvfg};

/// Run sparse flow-sensitive pointer analysis.
///
/// Builds on Andersen CI pre-analysis, propagating points-to information
/// along the object-labeled `FsSvfg` with strong updates at singleton stores.
///
/// Dispatches to the generic solver with `BTreePtsSet`. The generic
/// infrastructure supports alternative representations for future extension.
pub fn solve_flow_sensitive(
    module: &AirModule,
    fs_svfg: &FsSvfg,
    pta_result: &PtaResult,
    callgraph: &CallGraph,
    config: &FsPtaConfig,
) -> FlowSensitivePtaResult {
    solve_flow_sensitive_generic::<BTreePtsSet>(module, fs_svfg, pta_result, callgraph, config)
}

/// Generic flow-sensitive solver parameterized over points-to set representation.
///
/// All internal data structures use `P` for points-to sets and `IndexMap` for
/// hot-path maps. Results are normalized to `BTreeMap<_, BTreeSet<LocId>>` at
/// the boundary for API stability.
// NOTE: This function implements the SFS worklist algorithm with VFS version
// management, periodic compaction, and diagnostics logging as a single cohesive
// unit. Splitting would fragment the solver state flow.
#[allow(clippy::too_many_lines)]
fn solve_flow_sensitive_generic<P: PtsSet>(
    module: &AirModule,
    fs_svfg: &FsSvfg,
    pta_result: &PtaResult,
    callgraph: &CallGraph,
    config: &FsPtaConfig,
) -> FlowSensitivePtaResult {
    let su_info = StrongUpdateInfo::new(callgraph);
    let inst_to_func = build_inst_to_func_map(module);

    // Step 1: Initialize points_to from Andersen results
    let mut points_to: IndexMap<ValueId, P> = IndexMap::new();
    for (vid, locs) in pta_result.points_to_map() {
        if !locs.is_empty() {
            points_to.insert(*vid, P::from_btreeset(locs));
        }
    }

    // VFS state: version table + thin per-node version maps
    let mut ver_table: VersionTable<P> = VersionTable::new();
    let mut ver_in: IndexMap<SvfgNodeId, NodeVersionMap> = IndexMap::new();
    let mut ver_out: IndexMap<SvfgNodeId, NodeVersionMap> = IndexMap::new();

    let mut diagnostics = FsPtaDiagnostics {
        fs_svfg_nodes: fs_svfg.node_count(),
        fs_svfg_edges: fs_svfg.edge_count(),
        store_nodes: fs_svfg.store_count(),
        load_nodes: fs_svfg.load_count(),
        ..FsPtaDiagnostics::default()
    };

    // Seed worklist with all nodes that have non-empty points_to (value nodes only)
    let mut worklist: BTreeSet<SvfgNodeId> = BTreeSet::new();
    for node in fs_svfg.nodes() {
        if let SvfgNodeId::Value(vid) = node {
            if points_to.get(vid).is_some_and(|s| !s.is_empty()) {
                worklist.insert(*node);
            }
        }
    }

    // Step 2: Process worklist
    let mut iterations = 0usize;
    while let Some(node) = worklist.pop_first() {
        if iterations >= config.max_iterations {
            diagnostics.iteration_limit_hit = true;
            break;
        }
        iterations += 1;

        // Periodic compaction
        if config.compact_interval > 0 && iterations % config.compact_interval == 0 {
            ver_table.compact(&mut ver_in, &mut ver_out);
            diagnostics.compactions += 1;
        }

        // Process the node's instruction semantics
        process_node(
            node,
            fs_svfg,
            pta_result,
            &su_info,
            &inst_to_func,
            &mut points_to,
            &mut ver_in,
            &mut ver_out,
            &mut ver_table,
            &mut diagnostics,
        );

        // Propagate to successors
        for edge in fs_svfg.successors_of(node) {
            let changed = if edge.kind.is_direct() {
                propagate_direct(node, edge.target, &points_to).is_some_and(|ref new_pt_set| {
                    union_points_to(&mut points_to, edge.target, new_pt_set)
                })
            } else {
                propagate_indirect(
                    node,
                    edge.target,
                    &edge.objects,
                    &ver_out,
                    &mut ver_in,
                    &mut ver_table,
                )
            };

            if changed {
                worklist.insert(edge.target);
            }
        }
    }

    diagnostics.iterations = iterations;
    diagnostics.version_count = ver_table.len();

    // Convert VFS state to public BTreeMap/BTreeSet representation
    let pts_btree: BTreeMap<ValueId, BTreeSet<LocId>> = points_to
        .into_iter()
        .map(|(k, v)| (k, v.to_btreeset()))
        .collect();

    let (df_in_btree, df_out_btree) = if config.skip_df_materialization {
        (BTreeMap::new(), BTreeMap::new())
    } else {
        (
            convert_ver_map(&ver_in, &ver_table),
            convert_ver_map(&ver_out, &ver_table),
        )
    };

    FlowSensitivePtaResult {
        pts: pts_btree,
        df_in: df_in_btree,
        df_out: df_out_btree,
        diagnostics,
    }
}

/// Convert VFS version maps to the public `BTreeMap` representation.
fn convert_ver_map<P: PtsSet>(
    ver_map: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &VersionTable<P>,
) -> BTreeMap<SvfgNodeId, BTreeMap<LocId, BTreeSet<LocId>>> {
    ver_map
        .iter()
        .map(|(node, versions)| {
            let converted: BTreeMap<LocId, BTreeSet<LocId>> = versions
                .iter()
                .filter_map(|(loc, vid)| {
                    let pts = ver_table.get(*vid).to_btreeset();
                    if pts.is_empty() {
                        None
                    } else {
                        Some((*loc, pts))
                    }
                })
                .collect();
            (*node, converted)
        })
        .collect()
}

/// Process a single SVFG node: apply store/load/pass-through semantics.
// NOTE: This function requires many parameters because it operates on the shared
// solver state (graphs, points-to sets, dataflow facts, diagnostics) that cannot
// be bundled without introducing borrow checker conflicts in the main loop.
#[allow(clippy::too_many_arguments)]
fn process_node<P: PtsSet>(
    node: SvfgNodeId,
    fs_svfg: &FsSvfg,
    pta_result: &PtaResult,
    su_info: &StrongUpdateInfo,
    inst_to_func: &IndexMap<ValueId, FunctionId>,
    points_to: &mut IndexMap<ValueId, P>,
    ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
    diagnostics: &mut FsPtaDiagnostics,
) {
    // Store node: create new versions for stored-to objects
    let store_infos = fs_svfg.store_infos(node);
    if !store_infos.is_empty() {
        process_stores(
            node,
            store_infos,
            pta_result,
            su_info,
            inst_to_func,
            points_to,
            ver_in,
            ver_out,
            ver_table,
            diagnostics,
        );
        return;
    }

    // Load node: read from version table into points_to
    if let Some(load_info) = fs_svfg.load_info(node) {
        process_load(
            node,
            load_info.ptr,
            load_info.dst,
            points_to,
            ver_in,
            ver_table,
        );
        return;
    }

    // Pass-through: ver_out[node] = ver_in[node]
    if let Some(in_versions) = ver_in.get(&node).cloned() {
        ver_out.insert(node, in_versions);
    }
}

/// Process a store node with potentially multiple store infos.
///
/// A single SVFG value node may correspond to multiple store instructions when
/// the same value is stored through different pointers (e.g., `store &a, %c`
/// and `store &a, %d`). We process each store's pointer target independently
/// and merge the results into a single `ver_out`.
// NOTE: This function requires many parameters because it implements the store
// transfer function which needs access to multiple analysis components (PTA results,
// strong update info, dataflow maps) that cannot be bundled without lifetime conflicts.
#[allow(clippy::too_many_arguments)]
fn process_stores<P: PtsSet>(
    node: SvfgNodeId,
    store_infos: &[StoreInfo],
    pta_result: &PtaResult,
    su_info: &StrongUpdateInfo,
    inst_to_func: &IndexMap<ValueId, FunctionId>,
    points_to: &IndexMap<ValueId, P>,
    ver_in: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_out: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
    diagnostics: &mut FsPtaDiagnostics,
) {
    let empty_ver: NodeVersionMap = BTreeMap::new();
    let in_versions = ver_in.get(&node).unwrap_or(&empty_ver);
    // Clone version map (~20 bytes/entry) instead of full PTS map (~1600 bytes/entry)
    let mut out_versions = in_versions.clone();

    for store_info in store_infos {
        let empty_pts = P::empty();
        let pointer_pt_set = points_to.get(&store_info.ptr).unwrap_or(&empty_pts);
        let val_pt_set = points_to.get(&store_info.val).unwrap_or(&empty_pts);

        let func_id = inst_to_func
            .get(&store_info.ptr)
            .or_else(|| inst_to_func.get(&store_info.val))
            .copied()
            .unwrap_or(FunctionId::new(0));

        // Convert to BTreeSet for strong update check
        let pointer_btree = pointer_pt_set.to_btreeset();
        let can_strong =
            su_info.can_strong_update(store_info.ptr, func_id, &pointer_btree, pta_result);

        for loc in pointer_pt_set.iter() {
            if can_strong {
                diagnostics.strong_updates += 1;
                // Strong update: new version = just the stored value's PTS
                let vid = ver_table.new_version(val_pt_set.clone());
                out_versions.insert(loc, vid);
            } else {
                diagnostics.weak_updates += 1;
                // Weak update: merge old version PTS with stored value's PTS
                let mut merged = in_versions
                    .get(&loc)
                    .map_or_else(P::empty, |vid| ver_table.get(*vid).clone());
                if merged.union(val_pt_set) {
                    // PTS actually grew — create new version
                    let vid = ver_table.new_version(merged);
                    out_versions.insert(loc, vid);
                } else if let Some(&vid) = in_versions.get(&loc) {
                    // PTS unchanged — reuse existing version
                    out_versions.insert(loc, vid);
                }
            }
        }
    }

    ver_out.insert(node, out_versions);
}

/// Process a load node: read `ver_in[node][loc]` into `points_to[dst]`.
fn process_load<P: PtsSet>(
    node: SvfgNodeId,
    pointer: ValueId,
    dst: ValueId,
    points_to: &mut IndexMap<ValueId, P>,
    ver_in: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &VersionTable<P>,
) {
    let pointer_pt_set = points_to.get(&pointer).cloned().unwrap_or_else(P::empty);
    let empty_ver: NodeVersionMap = BTreeMap::new();
    let in_versions = ver_in.get(&node).unwrap_or(&empty_ver);

    let mut new_pt_set = P::empty();
    for loc in pointer_pt_set.iter() {
        if let Some(vid) = in_versions.get(&loc) {
            new_pt_set.union(ver_table.get(*vid));
        }
    }

    if !new_pt_set.is_empty() {
        let dst_pt_set = points_to.entry(dst).or_insert_with(P::empty);
        dst_pt_set.union(&new_pt_set);
    }
}

/// Propagate top-level `points_to` along a direct edge.
///
/// Returns the set to union into the target's `points_to`, or `None` if no propagation.
fn propagate_direct<P: PtsSet>(
    src: SvfgNodeId,
    _target: SvfgNodeId,
    points_to: &IndexMap<ValueId, P>,
) -> Option<P> {
    let SvfgNodeId::Value(src_vid) = src else {
        return None;
    };
    points_to.get(&src_vid).cloned()
}

/// Union a set into a target value's `points_to`. Returns true if anything changed.
fn union_points_to<P: PtsSet>(
    points_to: &mut IndexMap<ValueId, P>,
    target: SvfgNodeId,
    new_pt_set: &P,
) -> bool {
    let SvfgNodeId::Value(target_vid) = target else {
        return false;
    };
    if new_pt_set.is_empty() {
        return false;
    }
    let entry = points_to.entry(target_vid).or_insert_with(P::empty);
    entry.union(new_pt_set)
}

/// Propagate address-taken objects along an indirect edge using VFS version IDs.
///
/// Copies `ver_out[src][obj]` into `ver_in[target][obj]` for each obj in `edge.objects`.
/// Returns true if anything changed.
fn propagate_indirect<P: PtsSet>(
    src: SvfgNodeId,
    target: SvfgNodeId,
    objects: &BTreeSet<LocId>,
    ver_out: &IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_in: &mut IndexMap<SvfgNodeId, NodeVersionMap>,
    ver_table: &mut VersionTable<P>,
) -> bool {
    let Some(src_versions) = ver_out.get(&src) else {
        return false;
    };

    let mut changed = false;

    // Determine which objects to propagate
    let propagate_all = objects.is_empty();

    let obj_iter: Box<dyn Iterator<Item = &LocId>> = if propagate_all {
        // PhiFlow or unlabeled indirect edge: propagate all objects
        Box::new(src_versions.keys())
    } else {
        // Labeled indirect edge: propagate only specified objects
        Box::new(objects.iter())
    };

    for obj in obj_iter {
        let Some(&src_vid) = src_versions.get(obj) else {
            continue;
        };

        let target_in = ver_in.entry(target).or_default();
        match target_in.get(obj).copied() {
            None => {
                // First arrival: just copy the VersionId (4 bytes, no PTS clone!)
                target_in.insert(*obj, src_vid);
                changed = true;
            }
            Some(target_vid) if target_vid == src_vid => {
                // Same version — no work needed (common case in converged state)
            }
            Some(target_vid) => {
                // Different versions — merge their PTS
                let mut merged = ver_table.get(target_vid).clone();
                if merged.union(ver_table.get(src_vid)) {
                    // PTS grew — create new merged version
                    let new_vid = ver_table.new_version(merged);
                    target_in.insert(*obj, new_vid);
                    changed = true;
                }
                // If union didn't grow, target already subsumes src — no change
            }
        }
    }

    changed
}

/// Build a mapping from `ValueId` -> `FunctionId` for store/load ptr/val
/// values to determine which function a store is in.
fn build_inst_to_func_map(module: &AirModule) -> IndexMap<ValueId, FunctionId> {
    let mut map = IndexMap::new();
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }
        for param in &func.params {
            map.insert(param.id, func.id);
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(dst) = inst.dst {
                    map.insert(dst, func.id);
                }
                for op in &inst.operands {
                    map.entry(*op).or_insert(func.id);
                }
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId};

    use crate::callgraph::CallGraph;
    use crate::cfg::Cfg;
    use crate::defuse::DefUseGraph;
    use crate::fspta::builder::FsSvfgBuilder;
    use crate::mssa::MemorySsa;
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

    fn run_full_pipeline(module: &AirModule) -> FlowSensitivePtaResult {
        let defuse = DefUseGraph::build(module);
        let callgraph = CallGraph::build(module);
        let pta_config = PtaConfig::default();

        // PTA 1: for SVFG
        let mut ctx1 = PtaContext::new(pta_config.clone());
        let raw1 = ctx1.analyze(module);
        let pta1 = PtaResult::new(raw1.pts, Arc::new(raw1.factory), raw1.diagnostics);

        // PTA 2: consumed by MSSA
        let mut ctx2 = PtaContext::new(pta_config.clone());
        let raw2 = ctx2.analyze(module);
        let mssa_pta = PtaResult::new(raw2.pts, Arc::new(raw2.factory), raw2.diagnostics);

        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();
        let mut mssa = MemorySsa::build(module, &cfgs, mssa_pta, &callgraph);
        let (svfg, _program_points) =
            SvfgBuilder::new(module, &defuse, &callgraph, &pta1, &mut mssa).build();

        // PTA 3: for FsSvfg builder
        let mut ctx3 = PtaContext::new(pta_config.clone());
        let raw3 = ctx3.analyze(module);
        let pta3 = PtaResult::new(raw3.pts, Arc::new(raw3.factory), raw3.diagnostics);

        // PTA 4: consumed by MSSA for FsSvfg
        let mut ctx4 = PtaContext::new(pta_config);
        let raw4 = ctx4.analyze(module);
        let mssa_pta2 = PtaResult::new(raw4.pts, Arc::new(raw4.factory), raw4.diagnostics);
        let mut mssa2 = MemorySsa::build(module, &cfgs, mssa_pta2, &callgraph);

        let fs_svfg = FsSvfgBuilder::new(module, &svfg, &pta3, &mut mssa2, &callgraph).build();

        let config = super::super::FsPtaConfig::default();
        solve_flow_sensitive(module, &fs_svfg, &pta3, &callgraph, &config)
    }

    #[test]
    fn solver_converges_on_simple_store_load() {
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
        let result = run_full_pipeline(&module);

        // Solver should converge without hitting the limit
        assert!(!result.diagnostics().iteration_limit_hit);
    }

    #[test]
    fn solver_diagnostics_populated() {
        let alloca = Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(10));
        let store = Instruction::new(InstId::new(101), Operation::Store)
            .with_operands(vec![ValueId::new(1), ValueId::new(10)]);

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
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            }],
        );

        let module = make_module(vec![func]);
        let result = run_full_pipeline(&module);

        // Solver should produce a valid result (even if empty for simple AIR)
        assert!(!result.diagnostics().iteration_limit_hit);
    }
}
