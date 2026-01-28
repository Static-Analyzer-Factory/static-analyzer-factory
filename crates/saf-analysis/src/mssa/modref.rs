//! Interprocedural mod/ref summaries for Memory SSA.
//!
//! Computes per-function may-modify and may-reference location sets,
//! bottom-up on the call graph. For recursive function groups (SCCs),
//! iterates to a fixed point.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{FunctionId, LocId};

use crate::PtaResult;
use crate::callgraph::{CallGraph, CallGraphNode};
use crate::graph_algo;
use crate::pta::ptsset::{IdBitSet, Indexer};

/// Mod/ref summary for a function.
///
/// Records which abstract memory locations the function may modify (write)
/// and may reference (read), including transitive effects through callees.
#[derive(Debug, Clone)]
pub struct ModRefSummary {
    /// Locations this function may modify (directly or transitively).
    pub may_mod: IdBitSet<LocId>,
    /// Locations this function may read (directly or transitively).
    pub may_ref: IdBitSet<LocId>,
}

impl PartialEq for ModRefSummary {
    fn eq(&self, other: &Self) -> bool {
        self.may_mod == other.may_mod && self.may_ref == other.may_ref
    }
}
impl Eq for ModRefSummary {}

impl ModRefSummary {
    /// Create an empty summary.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            may_mod: IdBitSet::empty(),
            may_ref: IdBitSet::empty(),
        }
    }

    /// Create an empty summary sharing the given indexer for fast unions.
    #[must_use]
    pub fn with_indexer(indexer: &Arc<RwLock<Indexer<LocId>>>) -> Self {
        Self {
            may_mod: IdBitSet::with_indexer(Arc::clone(indexer)),
            may_ref: IdBitSet::with_indexer(Arc::clone(indexer)),
        }
    }
}

/// Compute mod/ref summaries for all functions in a module.
///
/// Algorithm:
/// 1. Process functions in reverse topological order of the call graph.
/// 2. For each function, scan instructions:
///    - Store: `may_mod.extend(pta.points_to(ptr))`
///    - Load: `may_ref.extend(pta.points_to(ptr))`
///    - Memcpy/Memset: `may_mod.extend(pta.points_to(dst))`
///    - CallDirect: union callee's mod/ref into this function's
/// 3. For SCCs (recursive functions): iterate until fixed point.
pub fn compute_mod_ref(
    module: &AirModule,
    pta: &PtaResult,
    callgraph: &CallGraph,
) -> BTreeMap<FunctionId, ModRefSummary> {
    let indexer = Arc::new(RwLock::new(Indexer::<LocId>::new()));
    let mut summaries: BTreeMap<FunctionId, ModRefSummary> = BTreeMap::new();

    // Build function ID set for the call graph
    let func_nodes: BTreeSet<CallGraphNode> = callgraph.nodes.clone();

    // Compute SCCs in the call graph (reverse topological order = leaf SCCs first)
    let sccs = graph_algo::tarjan_scc(&func_nodes, &callgraph.edges);

    // Process SCCs in order (leaves first → callee summaries available for callers)
    for scc in &sccs {
        // Collect function IDs in this SCC
        let scc_funcs: Vec<FunctionId> =
            scc.iter().filter_map(CallGraphNode::function_id).collect();

        if scc_funcs.len() <= 1 {
            // Non-recursive function (or single self-recursive)
            for &func_id in &scc_funcs {
                let summary = compute_function_summary(module, func_id, pta, &summaries, &indexer);
                summaries.insert(func_id, summary);
            }

            // Handle self-recursion: re-compute once with own summary available
            for &func_id in &scc_funcs {
                let is_self_recursive = callgraph
                    .edges
                    .get(&CallGraphNode::Function(func_id))
                    .is_some_and(|callees| callees.contains(&CallGraphNode::Function(func_id)));
                if is_self_recursive {
                    let summary =
                        compute_function_summary(module, func_id, pta, &summaries, &indexer);
                    summaries.insert(func_id, summary);
                }
            }
        } else {
            // Mutually recursive SCC: iterate to fixed point
            // Initialize all summaries in the SCC
            for &func_id in &scc_funcs {
                summaries.insert(func_id, ModRefSummary::with_indexer(&indexer));
            }

            let mut changed = true;
            let mut iterations = 0;
            #[allow(clippy::items_after_statements)]
            const MAX_ITERATIONS: usize = 100;

            while changed && iterations < MAX_ITERATIONS {
                changed = false;
                iterations += 1;

                for &func_id in &scc_funcs {
                    let new_summary =
                        compute_function_summary(module, func_id, pta, &summaries, &indexer);
                    let old_summary = summaries.get(&func_id);
                    if old_summary != Some(&new_summary) {
                        summaries.insert(func_id, new_summary);
                        changed = true;
                    }
                }
            }
        }
    }

    summaries
}

/// Compute the mod/ref summary for a single function.
fn compute_function_summary(
    module: &AirModule,
    func_id: FunctionId,
    pta: &PtaResult,
    existing: &BTreeMap<FunctionId, ModRefSummary>,
    indexer: &Arc<RwLock<Indexer<LocId>>>,
) -> ModRefSummary {
    let mut summary = ModRefSummary::with_indexer(indexer);

    let Some(func) = module.functions.iter().find(|f| f.id == func_id) else {
        return summary;
    };

    if func.is_declaration {
        return summary;
    }

    for block in &func.blocks {
        for inst in &block.instructions {
            match &inst.op {
                Operation::Store => {
                    // operands[1] is the pointer
                    if let Some(&ptr) = inst.operands.get(1) {
                        let points_to_set = pta.points_to(ptr);
                        summary.may_mod.extend(points_to_set);
                    }
                }
                Operation::Load => {
                    // operands[0] is the pointer
                    if let Some(&ptr) = inst.operands.first() {
                        let points_to_set = pta.points_to(ptr);
                        summary.may_ref.extend(points_to_set);
                    }
                }
                Operation::Memcpy => {
                    // operands[0] is dest, operands[1] is src
                    if let Some(&dst) = inst.operands.first() {
                        summary.may_mod.extend(pta.points_to(dst));
                    }
                    if let Some(&src) = inst.operands.get(1) {
                        summary.may_ref.extend(pta.points_to(src));
                    }
                }
                Operation::Memset => {
                    // operands[0] is dest
                    if let Some(&dst) = inst.operands.first() {
                        summary.may_mod.extend(pta.points_to(dst));
                    }
                }
                Operation::CallDirect { callee } => {
                    // Union callee's summary into this function's
                    if let Some(callee_summary) = existing.get(callee) {
                        summary.may_mod.union(&callee_summary.may_mod);
                        summary.may_ref.union(&callee_summary.may_ref);
                    }
                }
                Operation::CallIndirect { .. } => {
                    // Conservative: indirect calls could modify/read anything.
                    // Without PTA-resolved targets at this stage, we assume
                    // worst-case side effects: may modify and may reference
                    // every known abstract memory location. This is sound but
                    // imprecise — a future refinement could use CG-resolved
                    // targets to narrow this to the actual callees' summaries.
                    for &loc_id in pta.locations().keys() {
                        summary.may_mod.insert(loc_id);
                        summary.may_ref.insert(loc_id);
                    }
                }
                _ => {}
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId, ObjId, ValueId};
    use std::collections::BTreeSet;

    use crate::callgraph::CallGraph;
    use crate::{FieldPath, FieldSensitivity, LocationFactory, PointsToMap, PtaDiagnostics};

    fn make_simple_pta(mappings: &[(u128, &[u128])]) -> PtaResult {
        let mut factory = LocationFactory::new(FieldSensitivity::None);
        let mut pts_map = PointsToMap::new();

        for &(val_id, loc_ids) in mappings {
            let mut locs = BTreeSet::new();
            for &loc_raw in loc_ids {
                let loc = factory.get_or_create(ObjId::new(loc_raw), FieldPath::empty());
                locs.insert(loc);
            }
            pts_map.insert(ValueId::new(val_id), locs);
        }

        PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default())
    }

    fn make_store_inst(id: u128, val: u128, ptr: u128) -> Instruction {
        Instruction {
            id: InstId::new(id),
            op: Operation::Store,
            operands: vec![ValueId::new(val), ValueId::new(ptr)],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_load_inst(id: u128, ptr: u128, dst: u128) -> Instruction {
        Instruction {
            id: InstId::new(id),
            op: Operation::Load,
            operands: vec![ValueId::new(ptr)],
            dst: Some(ValueId::new(dst)),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_ret_inst(id: u128) -> Instruction {
        Instruction {
            id: InstId::new(id),
            op: Operation::Ret,
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_function(name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::derive(name.as_bytes()),
            name: name.to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    #[test]
    fn modref_store_records_may_mod() {
        // Function with a store to ptr→{loc1}
        let func = make_function(
            "writer",
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    make_store_inst(10, 100, 200), // store val to ptr
                    make_ret_inst(11),
                ],
            }],
        );

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            name: None,
            functions: vec![func.clone()],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };

        // PTA: ptr (200) → {loc at ObjId(50)}
        let pta = make_simple_pta(&[(200, &[50])]);
        let cg = CallGraph::build(&module);
        let summaries = compute_mod_ref(&module, &pta, &cg);

        let s = summaries.get(&func.id).expect("summary for writer");
        assert!(!s.may_mod.is_empty(), "writer should have may_mod entries");
    }

    #[test]
    fn modref_load_records_may_ref() {
        let func = make_function(
            "reader",
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    make_load_inst(10, 200, 101), // load from ptr
                    make_ret_inst(11),
                ],
            }],
        );

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            name: None,
            functions: vec![func.clone()],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };

        let pta = make_simple_pta(&[(200, &[50])]);
        let cg = CallGraph::build(&module);
        let summaries = compute_mod_ref(&module, &pta, &cg);

        let s = summaries.get(&func.id).expect("summary for reader");
        assert!(!s.may_ref.is_empty(), "reader should have may_ref entries");
    }

    #[test]
    fn modref_transitive_through_call() {
        // callee: stores to ptr→{loc1}
        // caller: calls callee
        let callee_id = FunctionId::derive(b"callee");
        let callee = AirFunction {
            id: callee_id,
            name: "callee".to_string(),
            params: Vec::new(),
            blocks: vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![make_store_inst(10, 100, 200), make_ret_inst(11)],
            }],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let caller = AirFunction {
            id: FunctionId::derive(b"caller"),
            name: "caller".to_string(),
            params: Vec::new(),
            blocks: vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![
                    Instruction {
                        id: InstId::new(20),
                        op: Operation::CallDirect { callee: callee_id },
                        operands: vec![],
                        dst: None,
                        span: None,
                        symbol: None,
                        result_type: None,
                        extensions: BTreeMap::new(),
                    },
                    make_ret_inst(21),
                ],
            }],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let module = AirModule {
            id: ModuleId::derive(b"test"),
            name: None,
            functions: vec![callee, caller.clone()],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };

        let pta = make_simple_pta(&[(200, &[50])]);
        let cg = CallGraph::build(&module);
        let summaries = compute_mod_ref(&module, &pta, &cg);

        // Caller's may_mod should include callee's may_mod (transitive)
        let caller_s = summaries.get(&caller.id).expect("summary for caller");
        let callee_s = summaries.get(&callee_id).expect("summary for callee");
        assert!(
            callee_s
                .may_mod
                .iter()
                .all(|loc| caller_s.may_mod.contains(loc)),
            "caller should transitively include callee's may_mod"
        );
    }
}
