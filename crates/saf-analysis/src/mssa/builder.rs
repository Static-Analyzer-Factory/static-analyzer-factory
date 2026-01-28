//! Memory SSA skeleton construction and phi placement.
//!
//! Phase 2: Walk functions in dominance order, create Def/Use/LiveOnEntry.
//! Phase 3: Place Phi nodes at dominance frontiers, rename reaching defs.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId, ValueId};

use crate::cfg::Cfg;
use crate::graph_algo;

use super::InstInfo;
use super::access::{MemAccessId, MemoryAccess};

/// Build the Memory SSA skeleton for all functions in a module.
///
/// Returns:
/// - `accesses`: all memory accesses by ID
/// - `inst_to_access`: instruction → access ID mapping
/// - `block_phis`: block → list of phi access IDs at entry
/// - `live_on_entry`: function → `LiveOnEntry` sentinel access ID
/// - `inst_info`: instruction metadata for clobber walker
// NOTE: This return type is a 5-tuple of related maps that are computed together
// during skeleton construction. Bundling into a struct would add ceremony without
// improving API clarity since callers destructure immediately.
#[allow(clippy::type_complexity)]
// NOTE: This function implements the complete Memory SSA skeleton construction
// (Phases 2-3 from the MSSA algorithm). The logic is sequential and interdependent:
// LiveOnEntry creation → block walking → phi placement → rename pass. Splitting would
// fragment the algorithm's structure and require threading state between sub-functions.
#[allow(clippy::too_many_lines)]
pub fn build_skeleton(
    module: &AirModule,
    cfgs: &BTreeMap<FunctionId, Cfg>,
) -> (
    BTreeMap<MemAccessId, MemoryAccess>,
    BTreeMap<InstId, MemAccessId>,
    BTreeMap<BlockId, Vec<MemAccessId>>,
    BTreeMap<FunctionId, MemAccessId>,
    BTreeMap<InstId, InstInfo>,
) {
    let mut accesses = BTreeMap::new();
    let mut inst_to_access = BTreeMap::new();
    let mut block_phis: BTreeMap<BlockId, Vec<MemAccessId>> = BTreeMap::new();
    let mut live_on_entry_map = BTreeMap::new();
    let mut inst_info_map: BTreeMap<InstId, InstInfo> = BTreeMap::new();

    // Counter for generating deterministic access IDs
    let mut id_counter: u128 = 0;

    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        let Some(cfg) = cfgs.get(&func.id) else {
            continue;
        };

        // Phase 2: Build skeleton
        let mut next_id = || -> MemAccessId {
            id_counter += 1;
            // Derive deterministic ID from function + counter
            let data = format!("{}:{}", func.id.raw(), id_counter);
            MemAccessId::derive(data.as_bytes())
        };

        // 1. Create LiveOnEntry sentinel
        let loe_id = next_id();
        let loe = MemoryAccess::LiveOnEntry {
            id: loe_id,
            function: func.id,
        };
        accesses.insert(loe_id, loe);
        live_on_entry_map.insert(func.id, loe_id);

        // 2. Walk blocks in reverse post-order (dominance approximation)
        let rpo = graph_algo::reverse_post_order(&cfg.entry, cfg);

        // Track current reaching def per block exit
        let mut block_exit_def: BTreeMap<BlockId, MemAccessId> = BTreeMap::new();
        // Track which blocks define memory (for phi placement)
        let mut def_blocks: BTreeSet<BlockId> = BTreeSet::new();

        // Collect block instructions index for quick lookup
        let block_map: BTreeMap<BlockId, &saf_core::air::AirBlock> =
            func.blocks.iter().map(|b| (b.id, b)).collect();

        // First pass: create Def/Use accesses in each block
        // Use LiveOnEntry as initial reaching def for the entry block
        let mut block_entry_def: BTreeMap<BlockId, MemAccessId> = BTreeMap::new();
        block_entry_def.insert(cfg.entry, loe_id);

        for &block_id in &rpo {
            let Some(block) = block_map.get(&block_id) else {
                continue;
            };

            // Get entering def for this block
            let mut current_def = block_entry_def.get(&block_id).copied().unwrap_or(loe_id);

            // Walk instructions in order
            for inst in &block.instructions {
                match &inst.op {
                    // Store -> Def
                    Operation::Store => {
                        let def_id = next_id();
                        let def = MemoryAccess::Def {
                            id: def_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(def_id, def);
                        inst_to_access.insert(inst.id, def_id);
                        // Store: operands[1] is the pointer
                        let ptr = inst.operands.get(1).copied().unwrap_or(ValueId::new(0));
                        inst_info_map.insert(inst.id, InstInfo::Store { ptr });
                        current_def = def_id;
                        def_blocks.insert(block_id);
                    }
                    // CallDirect -> Def (may modify memory)
                    Operation::CallDirect { callee } => {
                        let def_id = next_id();
                        let def = MemoryAccess::Def {
                            id: def_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(def_id, def);
                        inst_to_access.insert(inst.id, def_id);
                        inst_info_map.insert(
                            inst.id,
                            InstInfo::Call {
                                callee: Some(*callee),
                            },
                        );
                        current_def = def_id;
                        def_blocks.insert(block_id);
                    }
                    // CallIndirect -> Def (may modify memory)
                    Operation::CallIndirect { .. } => {
                        let def_id = next_id();
                        let def = MemoryAccess::Def {
                            id: def_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(def_id, def);
                        inst_to_access.insert(inst.id, def_id);
                        inst_info_map.insert(inst.id, InstInfo::Call { callee: None });
                        current_def = def_id;
                        def_blocks.insert(block_id);
                    }
                    // Memcpy -> Def (modify destination memory)
                    Operation::Memcpy => {
                        let def_id = next_id();
                        let def = MemoryAccess::Def {
                            id: def_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(def_id, def);
                        inst_to_access.insert(inst.id, def_id);
                        let dst_ptr = inst.operands.first().copied().unwrap_or(ValueId::new(0));
                        inst_info_map.insert(inst.id, InstInfo::Memcpy { dst_ptr });
                        current_def = def_id;
                        def_blocks.insert(block_id);
                    }
                    // Memset -> Def (modify destination memory)
                    Operation::Memset => {
                        let def_id = next_id();
                        let def = MemoryAccess::Def {
                            id: def_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(def_id, def);
                        inst_to_access.insert(inst.id, def_id);
                        let dst_ptr = inst.operands.first().copied().unwrap_or(ValueId::new(0));
                        inst_info_map.insert(inst.id, InstInfo::Memset { dst_ptr });
                        current_def = def_id;
                        def_blocks.insert(block_id);
                    }
                    // Load -> Use
                    Operation::Load => {
                        let use_id = next_id();
                        let use_acc = MemoryAccess::Use {
                            id: use_id,
                            inst: inst.id,
                            block: block_id,
                            defining: current_def,
                        };
                        accesses.insert(use_id, use_acc);
                        inst_to_access.insert(inst.id, use_id);
                    }
                    // Other operations: no memory effect
                    _ => {}
                }
            }

            block_exit_def.insert(block_id, current_def);

            // Propagate exit def to successors as their entry def
            // (will be overwritten by phi placement later)
            if let Some(succs) = cfg.successors_of(block_id) {
                for &succ in succs {
                    block_entry_def.entry(succ).or_insert(current_def);
                }
            }
        }

        // Phase 3: Phi placement via iterated dominance frontier
        //
        // Compute dominance frontiers and place phis at join points where
        // different memory definitions may reach.
        let dom_frontiers = compute_dominance_frontiers(cfg);

        // Collect blocks that need phis (iterated dominance frontier of def blocks)
        let phi_blocks = iterated_dominance_frontier(&def_blocks, &dom_frontiers);

        // Create phi accesses
        for &phi_block in &phi_blocks {
            let phi_id = next_id();
            // Gather operands from predecessor blocks
            let operands: BTreeMap<BlockId, MemAccessId> = cfg
                .predecessors_of(phi_block)
                .map(|preds| {
                    preds
                        .iter()
                        .map(|&pred| {
                            let def = block_exit_def.get(&pred).copied().unwrap_or(loe_id);
                            (pred, def)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let phi = MemoryAccess::Phi {
                id: phi_id,
                block: phi_block,
                operands,
            };
            accesses.insert(phi_id, phi);
            block_phis.entry(phi_block).or_default().push(phi_id);
        }

        // Phase 3b: Rename — walk dominator tree, update defining pointers
        // to use phi results where applicable.
        //
        // After phi placement, update the entry def for blocks that have phis,
        // then re-link accesses in those blocks.
        rename_pass(
            &rpo,
            cfg,
            &block_map,
            &mut accesses,
            &inst_to_access,
            &block_phis,
            &mut block_exit_def,
            loe_id,
        );
    }

    (
        accesses,
        inst_to_access,
        block_phis,
        live_on_entry_map,
        inst_info_map,
    )
}

/// Compute dominance frontiers for all blocks in a CFG.
///
/// Uses the algorithm from Cooper, Harvey, Kennedy (2001):
/// "A Simple, Fast Dominance Algorithm"
fn compute_dominance_frontiers(cfg: &Cfg) -> BTreeMap<BlockId, BTreeSet<BlockId>> {
    // First compute immediate dominators
    let idom = compute_idom(cfg);

    // Compute dominance frontiers
    let mut df: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
    for &block in cfg.successors.keys() {
        df.insert(block, BTreeSet::new());
    }

    for &block in cfg.successors.keys() {
        let preds = cfg.predecessors_of(block);
        let pred_count = preds.map_or(0, BTreeSet::len);
        if pred_count >= 2 {
            // This is a join point
            if let Some(preds) = cfg.predecessors_of(block) {
                for &pred in preds {
                    let mut runner = pred;
                    // Walk up the dominator tree from pred until we reach
                    // the immediate dominator of block
                    while Some(&runner) != idom.get(&block) {
                        df.entry(runner).or_default().insert(block);
                        match idom.get(&runner) {
                            Some(&d) => runner = d,
                            None => break, // reached entry
                        }
                    }
                }
            }
        }
    }

    df
}

/// Compute immediate dominators using Cooper-Harvey-Kennedy algorithm.
fn compute_idom(cfg: &Cfg) -> BTreeMap<BlockId, BlockId> {
    let rpo = graph_algo::reverse_post_order(&cfg.entry, cfg);
    let rpo_index: BTreeMap<BlockId, usize> =
        rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();

    let mut idom: BTreeMap<BlockId, BlockId> = BTreeMap::new();
    idom.insert(cfg.entry, cfg.entry); // entry dominates itself

    let mut changed = true;
    while changed {
        changed = false;
        for &block in &rpo {
            if block == cfg.entry {
                continue;
            }
            let Some(preds) = cfg.predecessors_of(block) else {
                continue;
            };

            // Find first processed predecessor
            let mut new_idom = None;
            for &pred in preds {
                if idom.contains_key(&pred) {
                    new_idom = Some(pred);
                    break;
                }
            }
            let Some(mut new_idom) = new_idom else {
                continue;
            };

            // Intersect with remaining processed predecessors
            for &pred in preds {
                if pred == new_idom {
                    continue;
                }
                if idom.contains_key(&pred) {
                    new_idom = intersect(pred, new_idom, &idom, &rpo_index);
                }
            }

            if idom.get(&block) != Some(&new_idom) {
                idom.insert(block, new_idom);
                changed = true;
            }
        }
    }

    idom
}

/// Intersect two dominators by walking up to their common ancestor.
fn intersect(
    mut a: BlockId,
    mut b: BlockId,
    idom: &BTreeMap<BlockId, BlockId>,
    rpo_index: &BTreeMap<BlockId, usize>,
) -> BlockId {
    while a != b {
        let idx_a = rpo_index.get(&a).copied().unwrap_or(usize::MAX);
        let idx_b = rpo_index.get(&b).copied().unwrap_or(usize::MAX);
        if idx_a > idx_b {
            a = *idom.get(&a).unwrap_or(&a);
        } else {
            b = *idom.get(&b).unwrap_or(&b);
        }
    }
    a
}

/// Compute the iterated dominance frontier of a set of blocks.
fn iterated_dominance_frontier(
    def_blocks: &BTreeSet<BlockId>,
    dom_frontiers: &BTreeMap<BlockId, BTreeSet<BlockId>>,
) -> BTreeSet<BlockId> {
    let mut phi_blocks = BTreeSet::new();
    let mut worklist: Vec<BlockId> = def_blocks.iter().copied().collect();
    let mut in_worklist: BTreeSet<BlockId> = def_blocks.clone();

    while let Some(block) = worklist.pop() {
        in_worklist.remove(&block);
        if let Some(df) = dom_frontiers.get(&block) {
            for &frontier in df {
                if phi_blocks.insert(frontier) {
                    // Newly added — also process its dominance frontier
                    if !in_worklist.contains(&frontier) {
                        worklist.push(frontier);
                        in_worklist.insert(frontier);
                    }
                }
            }
        }
    }

    phi_blocks
}

/// Rename pass: walk blocks in RPO, update reaching defs through phis.
///
/// After phi placement, we need to ensure that:
/// 1. Blocks with phis use the phi as their entry def
/// 2. All Def/Use accesses in a block chain correctly through the block's entry def
// NOTE: This function requires many parameters because it operates on the shared
// state of the Memory SSA skeleton (CFG, block map, accesses, phis, etc.) that
// cannot be bundled without introducing lifetime complexity in the caller.
#[allow(clippy::too_many_arguments)]
fn rename_pass(
    rpo: &[BlockId],
    cfg: &Cfg,
    block_map: &BTreeMap<BlockId, &saf_core::air::AirBlock>,
    accesses: &mut BTreeMap<MemAccessId, MemoryAccess>,
    inst_to_access: &BTreeMap<InstId, MemAccessId>,
    block_phis: &BTreeMap<BlockId, Vec<MemAccessId>>,
    block_exit_def: &mut BTreeMap<BlockId, MemAccessId>,
    loe_id: MemAccessId,
) {
    // For blocks with phis, the phi becomes the entry def
    let mut block_entry_def: BTreeMap<BlockId, MemAccessId> = BTreeMap::new();

    for &block_id in rpo {
        // Determine entry def: phi (if present) or inherited from dominator
        let entry_def = if let Some(phis) = block_phis.get(&block_id) {
            // Last phi (there's typically just one for memory SSA)
            phis.last().copied().unwrap_or(loe_id)
        } else {
            block_entry_def.get(&block_id).copied().unwrap_or(loe_id)
        };

        let Some(block) = block_map.get(&block_id) else {
            continue;
        };

        // Re-link accesses in this block
        let mut current_def = entry_def;
        for inst in &block.instructions {
            if let Some(&acc_id) = inst_to_access.get(&inst.id) {
                if let Some(acc) = accesses.get_mut(&acc_id) {
                    match acc {
                        MemoryAccess::Def { defining, .. } => {
                            *defining = current_def;
                            current_def = acc_id;
                        }
                        MemoryAccess::Use { defining, .. } => {
                            *defining = current_def;
                        }
                        _ => {}
                    }
                }
            }
        }

        block_exit_def.insert(block_id, current_def);

        // Propagate exit def to successors
        if let Some(succs) = cfg.successors_of(block_id) {
            for &succ in succs {
                // Only set if no phi will override
                if !block_phis.contains_key(&succ) {
                    block_entry_def.insert(succ, current_def);
                }
            }
        }
    }

    // Update phi operands with final exit defs
    for (&block_id, phi_ids) in block_phis {
        if let Some(preds) = cfg.predecessors_of(block_id) {
            for &phi_id in phi_ids {
                if let Some(MemoryAccess::Phi { operands, .. }) = accesses.get_mut(&phi_id) {
                    for &pred in preds {
                        let pred_def = block_exit_def.get(&pred).copied().unwrap_or(loe_id);
                        operands.insert(pred, pred_def);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::ValueId;

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

    fn make_store(inst_id: u128, val: u128, ptr: u128) -> Instruction {
        Instruction {
            id: InstId::new(inst_id),
            op: Operation::Store,
            operands: vec![ValueId::new(val), ValueId::new(ptr)],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_load(inst_id: u128, ptr: u128, dst: u128) -> Instruction {
        Instruction {
            id: InstId::new(inst_id),
            op: Operation::Load,
            operands: vec![ValueId::new(ptr)],
            dst: Some(ValueId::new(dst)),
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_br(inst_id: u128, target: u128) -> Instruction {
        Instruction {
            id: InstId::new(inst_id),
            op: Operation::Br {
                target: BlockId::new(target),
            },
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_condbr(inst_id: u128, cond: u128, then_blk: u128, else_blk: u128) -> Instruction {
        Instruction {
            id: InstId::new(inst_id),
            op: Operation::CondBr {
                then_target: BlockId::new(then_blk),
                else_target: BlockId::new(else_blk),
            },
            operands: vec![ValueId::new(cond)],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    fn make_ret(inst_id: u128) -> Instruction {
        Instruction {
            id: InstId::new(inst_id),
            op: Operation::Ret,
            operands: vec![],
            dst: None,
            span: None,
            symbol: None,
            result_type: None,
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn skeleton_simple_store_load() {
        // entry: store; load; ret
        let blocks = vec![AirBlock {
            id: BlockId::new(1),
            label: Some("entry".to_string()),
            instructions: vec![
                make_store(10, 100, 200),
                make_load(11, 200, 101),
                make_ret(12),
            ],
        }];
        let func = make_function("test", blocks);
        let _cfg = Cfg::build(&func);
        let module = AirModule {
            id: saf_core::ids::ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: vec![func],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();

        let (accesses, inst_to_access, block_phis, live_on_entry, _inst_info) =
            build_skeleton(&module, &cfgs);

        // Should have: LiveOnEntry + 1 Def (store) + 1 Use (load)
        assert_eq!(live_on_entry.len(), 1);
        assert!(inst_to_access.contains_key(&InstId::new(10))); // store
        assert!(inst_to_access.contains_key(&InstId::new(11))); // load

        // Store should be a Def
        let store_acc_id = inst_to_access[&InstId::new(10)];
        let store_acc = &accesses[&store_acc_id];
        assert!(store_acc.is_def());

        // Load should be a Use
        let load_acc_id = inst_to_access[&InstId::new(11)];
        let load_acc = &accesses[&load_acc_id];
        assert!(load_acc.is_use());

        // Load's defining should be the store
        assert_eq!(load_acc.defining(), Some(store_acc_id));

        // Store's defining should be LiveOnEntry
        let loe_id = live_on_entry.values().next().unwrap();
        assert_eq!(store_acc.defining(), Some(*loe_id));

        // No phis needed in a single block
        assert!(block_phis.is_empty() || block_phis.values().all(|v| v.is_empty()));
    }

    #[test]
    fn skeleton_diamond_places_phi() {
        // entry(1): condbr -> then(2), else(3)
        // then(2): store; br -> merge(4)
        // else(3): store; br -> merge(4)
        // merge(4): load; ret
        let blocks = vec![
            AirBlock {
                id: BlockId::new(1),
                label: Some("entry".to_string()),
                instructions: vec![make_condbr(10, 999, 2, 3)],
            },
            AirBlock {
                id: BlockId::new(2),
                label: Some("then".to_string()),
                instructions: vec![make_store(20, 100, 200), make_br(21, 4)],
            },
            AirBlock {
                id: BlockId::new(3),
                label: Some("else".to_string()),
                instructions: vec![make_store(30, 101, 200), make_br(31, 4)],
            },
            AirBlock {
                id: BlockId::new(4),
                label: Some("merge".to_string()),
                instructions: vec![make_load(40, 200, 102), make_ret(41)],
            },
        ];

        let func = make_function("diamond", blocks);
        let module = AirModule {
            id: saf_core::ids::ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: vec![func],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();

        let (accesses, inst_to_access, block_phis, _live_on_entry, _inst_info) =
            build_skeleton(&module, &cfgs);

        // Merge block (4) should have a phi
        let merge_phis = block_phis.get(&BlockId::new(4));
        assert!(
            merge_phis.is_some() && !merge_phis.unwrap().is_empty(),
            "merge block should have a phi node"
        );

        // Load in merge block should reference the phi
        let load_acc_id = inst_to_access[&InstId::new(40)];
        let load_acc = &accesses[&load_acc_id];
        let phi_id = merge_phis.unwrap()[0];
        assert_eq!(
            load_acc.defining(),
            Some(phi_id),
            "load's reaching def should be the phi"
        );

        // Phi should have operands from both predecessors
        if let MemoryAccess::Phi { operands, .. } = &accesses[&phi_id] {
            assert!(
                operands.contains_key(&BlockId::new(2)),
                "phi should have then-branch operand"
            );
            assert!(
                operands.contains_key(&BlockId::new(3)),
                "phi should have else-branch operand"
            );
        } else {
            panic!("expected Phi access");
        }
    }

    #[test]
    fn skeleton_loop_places_phi_at_header() {
        // entry(1): store; br -> header(2)
        // header(2): load; condbr -> body(3), exit(4)
        // body(3): store; br -> header(2)
        // exit(4): ret
        let blocks = vec![
            AirBlock {
                id: BlockId::new(1),
                label: Some("entry".to_string()),
                instructions: vec![make_store(10, 100, 200), make_br(11, 2)],
            },
            AirBlock {
                id: BlockId::new(2),
                label: Some("header".to_string()),
                instructions: vec![make_load(20, 200, 101), make_condbr(21, 999, 3, 4)],
            },
            AirBlock {
                id: BlockId::new(3),
                label: Some("body".to_string()),
                instructions: vec![make_store(30, 102, 200), make_br(31, 2)],
            },
            AirBlock {
                id: BlockId::new(4),
                label: Some("exit".to_string()),
                instructions: vec![make_ret(40)],
            },
        ];

        let func = make_function("loop_test", blocks);
        let module = AirModule {
            id: saf_core::ids::ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: vec![func],
            globals: vec![],
            source_files: vec![],
            type_hierarchy: vec![],
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        };
        let cfgs: BTreeMap<FunctionId, Cfg> = module
            .functions
            .iter()
            .filter(|f| !f.is_declaration)
            .map(|f| (f.id, Cfg::build(f)))
            .collect();

        let (_accesses, _inst_to_access, block_phis, _live_on_entry, _inst_info) =
            build_skeleton(&module, &cfgs);

        // Header block (2) should have a phi (join from entry + back edge)
        let header_phis = block_phis.get(&BlockId::new(2));
        assert!(
            header_phis.is_some() && !header_phis.unwrap().is_empty(),
            "loop header should have a phi node"
        );
    }

    #[test]
    fn dominance_frontier_diamond() {
        // Diamond: entry(1) -> then(2), else(3) -> merge(4)
        let blocks = vec![
            AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![make_condbr(10, 999, 2, 3)],
            },
            AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![make_br(20, 4)],
            },
            AirBlock {
                id: BlockId::new(3),
                label: None,
                instructions: vec![make_br(30, 4)],
            },
            AirBlock {
                id: BlockId::new(4),
                label: None,
                instructions: vec![make_ret(40)],
            },
        ];
        let func = make_function("df_test", blocks);
        let cfg = Cfg::build(&func);
        let df = compute_dominance_frontiers(&cfg);

        // then(2) and else(3) have merge(4) in their DF
        assert!(df[&BlockId::new(2)].contains(&BlockId::new(4)));
        assert!(df[&BlockId::new(3)].contains(&BlockId::new(4)));
        // entry(1) and merge(4) have empty DF
        assert!(df[&BlockId::new(1)].is_empty());
        assert!(df[&BlockId::new(4)].is_empty());
    }
}
