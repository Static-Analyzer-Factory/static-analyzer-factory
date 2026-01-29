//! Dominator tree computation and dominator-based guard extraction.
//!
//! For point-based analyses (typestate, numeric, assertions) where there is no
//! source→sink trace, we collect guards that **dominate** the finding's block.
//! A dominator block's condition must be true on ALL paths reaching the finding.
//!
//! Uses the Cooper-Harvey-Kennedy iterative dominator algorithm.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::ids::{BlockId, FunctionId};

use crate::cfg::Cfg;

use crate::guard::{Guard, PathCondition, TerminatorInfo, ValueLocationIndex};

// ---------------------------------------------------------------------------
// Dominator computation (Cooper-Harvey-Kennedy)
// ---------------------------------------------------------------------------

/// Compute the immediate dominator of each block in a CFG.
///
/// Uses the Cooper-Harvey-Kennedy iterative algorithm. The entry block has
/// no dominator (absent from the returned map).
///
/// Returns a map from `BlockId` → immediate dominator `BlockId`.
#[must_use]
pub fn compute_dominators(cfg: &Cfg) -> BTreeMap<BlockId, BlockId> {
    let entry = cfg.entry;
    let blocks: Vec<BlockId> = cfg.successors.keys().copied().collect();

    if blocks.is_empty() {
        return BTreeMap::new();
    }

    // Compute reverse postorder for iteration
    let rpo = reverse_postorder(entry, &cfg.successors);
    let rpo_index: BTreeMap<BlockId, usize> =
        rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();

    // Initialize: entry dominates itself, others undefined (represented by None)
    let mut idom: BTreeMap<BlockId, Option<BlockId>> = BTreeMap::new();
    for &b in &blocks {
        idom.insert(b, None);
    }
    idom.insert(entry, Some(entry));

    let mut changed = true;
    while changed {
        changed = false;
        for &b in &rpo {
            if b == entry {
                continue;
            }

            // Find the first processed predecessor
            let Some(preds) = cfg.predecessors.get(&b) else {
                continue;
            };

            let mut new_idom: Option<BlockId> = None;
            for &p in preds {
                if idom.get(&p).and_then(|d| d.as_ref()).is_some() {
                    match new_idom {
                        None => {
                            new_idom = Some(p);
                        }
                        Some(cur) => {
                            new_idom = Some(intersect(cur, p, &idom, &rpo_index));
                        }
                    }
                }
            }

            if new_idom != *idom.get(&b).unwrap_or(&None) {
                idom.insert(b, new_idom);
                changed = true;
            }
        }
    }

    // Convert to final map (exclude entry which dominates itself)
    let mut result = BTreeMap::new();
    for (&b, dom) in &idom {
        if b != entry {
            if let Some(d) = dom {
                result.insert(b, *d);
            }
        }
    }
    result
}

/// Intersect operation for the Cooper-Harvey-Kennedy algorithm.
fn intersect(
    mut b1: BlockId,
    mut b2: BlockId,
    idom: &BTreeMap<BlockId, Option<BlockId>>,
    rpo_index: &BTreeMap<BlockId, usize>,
) -> BlockId {
    while b1 != b2 {
        let idx1 = rpo_index.get(&b1).copied().unwrap_or(usize::MAX);
        let idx2 = rpo_index.get(&b2).copied().unwrap_or(usize::MAX);

        if idx1 > idx2 {
            match idom.get(&b1).and_then(|d| *d) {
                Some(d) => b1 = d,
                None => break,
            }
        } else {
            match idom.get(&b2).and_then(|d| *d) {
                Some(d) => b2 = d,
                None => break,
            }
        }
    }
    b1
}

/// Compute reverse postorder traversal of CFG blocks.
fn reverse_postorder(
    entry: BlockId,
    successors: &BTreeMap<BlockId, BTreeSet<BlockId>>,
) -> Vec<BlockId> {
    let mut visited = BTreeSet::new();
    let mut postorder = Vec::new();
    rpo_visit(entry, successors, &mut visited, &mut postorder);
    postorder.reverse();
    postorder
}

fn rpo_visit(
    block: BlockId,
    successors: &BTreeMap<BlockId, BTreeSet<BlockId>>,
    visited: &mut BTreeSet<BlockId>,
    postorder: &mut Vec<BlockId>,
) {
    if visited.contains(&block) {
        return;
    }
    visited.insert(block);
    if let Some(succs) = successors.get(&block) {
        for &s in succs {
            rpo_visit(s, successors, visited, postorder);
        }
    }
    postorder.push(block);
}

// ---------------------------------------------------------------------------
// Dominator-based guard extraction
// ---------------------------------------------------------------------------

/// Extract guards that dominate a given block.
///
/// Walks from the target block up to the function entry via immediate
/// dominators. At each dominator block with a `CondBr` terminator,
/// determines which branch leads toward the target block and records
/// the guard.
///
/// This is the key technique for point-based analyses (typestate, numeric,
/// assertions) where there is no source→sink trace.
pub fn extract_dominating_guards(
    block: BlockId,
    function_id: FunctionId,
    cfg: &Cfg,
    dominators: &BTreeMap<BlockId, BlockId>,
    index: &ValueLocationIndex,
) -> PathCondition {
    let mut guards = Vec::new();
    let mut current = block;

    // Walk up the dominator tree to entry
    while let Some(&idom) = dominators.get(&current) {
        // Check if idom has a CondBr terminator
        if let Some(TerminatorInfo::CondBr {
            condition,
            then_target,
            else_target,
        }) = index.terminator_of(idom)
        {
            // Determine which branch leads toward `current`.
            // Since idom dominates current, the branch reaching current
            // must go through one of the two successors.
            let branch_taken = if dominates_or_is(*then_target, current, dominators, cfg.entry) {
                true
            } else if dominates_or_is(*else_target, current, dominators, cfg.entry) {
                false
            } else {
                // Cannot determine which branch — skip this guard
                current = idom;
                continue;
            };

            guards.push(Guard {
                block: idom,
                function: function_id,
                condition: *condition,
                branch_taken,
            });
        }

        current = idom;
    }

    // Reverse so guards are in entry→block order (outer to inner)
    guards.reverse();
    PathCondition { guards }
}

/// Check if `ancestor` dominates `descendant` (or they are the same block).
fn dominates_or_is(
    ancestor: BlockId,
    descendant: BlockId,
    dominators: &BTreeMap<BlockId, BlockId>,
    entry: BlockId,
) -> bool {
    if ancestor == descendant {
        return true;
    }

    let mut current = descendant;
    loop {
        match dominators.get(&current) {
            Some(&idom) => {
                if idom == ancestor {
                    return true;
                }
                if idom == current {
                    // Self-loop in dominator tree — shouldn't happen, but prevent infinite loop
                    return false;
                }
                current = idom;
            }
            None => {
                // Reached entry (not in map) — ancestor is entry?
                return ancestor == entry;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirModule, BinaryOp, Instruction, Operation};
    use saf_core::ids::{FunctionId, InstId, ModuleId, ValueId};

    /// Build a CFG from edges for testing.
    fn make_cfg(entry: BlockId, edges: &[(BlockId, BlockId)]) -> Cfg {
        let mut successors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        let mut predecessors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        let mut exits = BTreeSet::new();

        // Collect all blocks
        let mut all_blocks = BTreeSet::new();
        all_blocks.insert(entry);
        for &(a, b) in edges {
            all_blocks.insert(a);
            all_blocks.insert(b);
        }

        for &b in &all_blocks {
            successors.insert(b, BTreeSet::new());
            predecessors.insert(b, BTreeSet::new());
        }

        for &(from, to) in edges {
            successors.get_mut(&from).unwrap().insert(to);
            predecessors.get_mut(&to).unwrap().insert(from);
        }

        // Exits = blocks with no successors
        for (&b, succs) in &successors {
            if succs.is_empty() {
                exits.insert(b);
            }
        }

        Cfg {
            function: FunctionId::new(1),
            entry,
            exits,
            successors,
            predecessors,
        }
    }

    #[test]
    fn dominators_linear_cfg() {
        // entry → b1 → b2 → b3
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);
        let b2 = BlockId::new(2);
        let b3 = BlockId::new(3);

        let cfg = make_cfg(entry, &[(entry, b1), (b1, b2), (b2, b3)]);

        let doms = compute_dominators(&cfg);
        assert_eq!(doms.get(&b1), Some(&entry));
        assert_eq!(doms.get(&b2), Some(&b1));
        assert_eq!(doms.get(&b3), Some(&b2));
    }

    #[test]
    fn dominators_diamond_cfg() {
        // entry → b1, entry → b2, b1 → b3, b2 → b3
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);
        let b2 = BlockId::new(2);
        let b3 = BlockId::new(3);

        let cfg = make_cfg(entry, &[(entry, b1), (entry, b2), (b1, b3), (b2, b3)]);

        let doms = compute_dominators(&cfg);
        assert_eq!(doms.get(&b1), Some(&entry));
        assert_eq!(doms.get(&b2), Some(&entry));
        // b3 is dominated by entry (not b1 or b2, since both paths reach it)
        assert_eq!(doms.get(&b3), Some(&entry));
    }

    #[test]
    fn dominators_loop_cfg() {
        // entry → header → body → header (loop), header → exit
        let entry = BlockId::new(0);
        let header = BlockId::new(1);
        let body = BlockId::new(2);
        let exit = BlockId::new(3);

        let cfg = make_cfg(
            entry,
            &[
                (entry, header),
                (header, body),
                (body, header),
                (header, exit),
            ],
        );

        let doms = compute_dominators(&cfg);
        assert_eq!(doms.get(&header), Some(&entry));
        assert_eq!(doms.get(&body), Some(&header));
        assert_eq!(doms.get(&exit), Some(&header));
    }

    #[test]
    fn dominators_nested_loops() {
        // entry → outer_hdr → inner_hdr → inner_body → inner_hdr,
        // inner_hdr → outer_body → outer_hdr, outer_hdr → exit
        let entry = BlockId::new(0);
        let outer_hdr = BlockId::new(1);
        let inner_hdr = BlockId::new(2);
        let inner_body = BlockId::new(3);
        let outer_body = BlockId::new(4);
        let exit = BlockId::new(5);

        let cfg = make_cfg(
            entry,
            &[
                (entry, outer_hdr),
                (outer_hdr, inner_hdr),
                (inner_hdr, inner_body),
                (inner_body, inner_hdr),
                (inner_hdr, outer_body),
                (outer_body, outer_hdr),
                (outer_hdr, exit),
            ],
        );

        let doms = compute_dominators(&cfg);
        assert_eq!(doms.get(&outer_hdr), Some(&entry));
        assert_eq!(doms.get(&inner_hdr), Some(&outer_hdr));
        assert_eq!(doms.get(&inner_body), Some(&inner_hdr));
        assert_eq!(doms.get(&outer_body), Some(&inner_hdr));
        assert_eq!(doms.get(&exit), Some(&outer_hdr));
    }

    #[test]
    fn dominators_single_block() {
        let entry = BlockId::new(0);
        let cfg = make_cfg(entry, &[]);

        let doms = compute_dominators(&cfg);
        assert!(doms.is_empty()); // entry has no dominator
    }

    #[test]
    fn dominators_empty_cfg() {
        let cfg = Cfg {
            function: FunctionId::new(1),
            entry: BlockId::new(0),
            exits: BTreeSet::new(),
            successors: BTreeMap::new(),
            predecessors: BTreeMap::new(),
        };

        let doms = compute_dominators(&cfg);
        assert!(doms.is_empty());
    }

    #[test]
    fn dominating_guards_single_guard() {
        // Build a module with CondBr at entry
        let func_id = FunctionId::new(1);
        let entry = BlockId::new(0);
        let then_blk = BlockId::new(1);
        let else_blk = BlockId::new(2);
        let cond_val = ValueId::new(100);

        let icmp = Instruction::new(
            InstId::new(1000),
            Operation::BinaryOp {
                kind: BinaryOp::ICmpEq,
            },
        )
        .with_operands(vec![ValueId::new(50), ValueId::new(51)])
        .with_dst(cond_val);

        let condbr = Instruction::new(
            InstId::new(1001),
            Operation::CondBr {
                then_target: then_blk,
                else_target: else_blk,
            },
        )
        .with_operands(vec![cond_val]);

        let then_inst = Instruction::new(InstId::new(1002), Operation::Ret);
        let else_inst = Instruction::new(InstId::new(1003), Operation::Ret);

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(AirFunction {
            id: func_id,
            name: "test".to_string(),
            params: vec![],
            blocks: vec![
                AirBlock {
                    id: entry,
                    label: Some("entry".to_string()),
                    instructions: vec![icmp, condbr],
                },
                AirBlock {
                    id: then_blk,
                    label: Some("then".to_string()),
                    instructions: vec![then_inst],
                },
                AirBlock {
                    id: else_blk,
                    label: Some("else".to_string()),
                    instructions: vec![else_inst],
                },
            ],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });

        let index = ValueLocationIndex::build(&module);
        let cfg = Cfg::build(&module.functions[0]);
        let doms = compute_dominators(&cfg);

        // Guards at then_blk: entry dominates it, entry has CondBr → then_blk
        let pc = extract_dominating_guards(then_blk, func_id, &cfg, &doms, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(pc.guards[0].branch_taken);
        assert_eq!(pc.guards[0].condition, cond_val);

        // Guards at else_blk: entry dominates it, entry has CondBr → else_blk
        let pc = extract_dominating_guards(else_blk, func_id, &cfg, &doms, &index);
        assert_eq!(pc.guards.len(), 1);
        assert!(!pc.guards[0].branch_taken);
    }

    #[test]
    fn dominating_guards_no_guards_at_entry() {
        let func_id = FunctionId::new(1);
        let entry = BlockId::new(0);

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(AirFunction {
            id: func_id,
            name: "test".to_string(),
            params: vec![],
            blocks: vec![AirBlock {
                id: entry,
                label: Some("entry".to_string()),
                instructions: vec![Instruction::new(InstId::new(1000), Operation::Ret)],
            }],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });

        let index = ValueLocationIndex::build(&module);
        let cfg = Cfg::build(&module.functions[0]);
        let doms = compute_dominators(&cfg);

        // No guards at entry block (it has no dominator)
        let pc = extract_dominating_guards(entry, func_id, &cfg, &doms, &index);
        assert!(pc.is_empty());
    }

    #[test]
    fn dominating_guards_multiple_guards() {
        // entry → b1 (CondBr) → b2 (CondBr) → b3
        let func_id = FunctionId::new(1);
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);
        let b2 = BlockId::new(2);
        let b3 = BlockId::new(3);
        let b4 = BlockId::new(4);
        let b5 = BlockId::new(5);

        let cond1 = ValueId::new(100);
        let cond2 = ValueId::new(101);

        let mut module = AirModule::new(ModuleId::new(1));
        module.functions.push(AirFunction {
            id: func_id,
            name: "test".to_string(),
            params: vec![],
            blocks: vec![
                AirBlock {
                    id: entry,
                    label: None,
                    instructions: vec![Instruction::new(
                        InstId::new(2000),
                        Operation::Br { target: b1 },
                    )],
                },
                AirBlock {
                    id: b1,
                    label: None,
                    instructions: vec![
                        Instruction::new(
                            InstId::new(2001),
                            Operation::BinaryOp {
                                kind: BinaryOp::ICmpEq,
                            },
                        )
                        .with_operands(vec![ValueId::new(50), ValueId::new(51)])
                        .with_dst(cond1),
                        Instruction::new(
                            InstId::new(2002),
                            Operation::CondBr {
                                then_target: b2,
                                else_target: b4,
                            },
                        )
                        .with_operands(vec![cond1]),
                    ],
                },
                AirBlock {
                    id: b2,
                    label: None,
                    instructions: vec![
                        Instruction::new(
                            InstId::new(2003),
                            Operation::BinaryOp {
                                kind: BinaryOp::ICmpSgt,
                            },
                        )
                        .with_operands(vec![ValueId::new(52), ValueId::new(53)])
                        .with_dst(cond2),
                        Instruction::new(
                            InstId::new(2004),
                            Operation::CondBr {
                                then_target: b3,
                                else_target: b5,
                            },
                        )
                        .with_operands(vec![cond2]),
                    ],
                },
                AirBlock {
                    id: b3,
                    label: None,
                    instructions: vec![Instruction::new(InstId::new(2005), Operation::Ret)],
                },
                AirBlock {
                    id: b4,
                    label: None,
                    instructions: vec![Instruction::new(InstId::new(2006), Operation::Ret)],
                },
                AirBlock {
                    id: b5,
                    label: None,
                    instructions: vec![Instruction::new(InstId::new(2007), Operation::Ret)],
                },
            ],
            entry_block: Some(entry),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        });

        let index = ValueLocationIndex::build(&module);
        let cfg = Cfg::build(&module.functions[0]);
        let doms = compute_dominators(&cfg);

        // b3 is dominated by b2, which is dominated by b1, which is dominated by entry
        // b1 has CondBr → b2 (then), b2 has CondBr → b3 (then)
        let pc = extract_dominating_guards(b3, func_id, &cfg, &doms, &index);
        assert_eq!(pc.guards.len(), 2);
        // First guard: b1's CondBr (outer), second: b2's CondBr (inner)
        assert_eq!(pc.guards[0].condition, cond1);
        assert!(pc.guards[0].branch_taken);
        assert_eq!(pc.guards[1].condition, cond2);
        assert!(pc.guards[1].branch_taken);
    }

    #[test]
    fn dominator_computation_deterministic() {
        let entry = BlockId::new(0);
        let b1 = BlockId::new(1);
        let b2 = BlockId::new(2);
        let b3 = BlockId::new(3);

        let cfg = make_cfg(entry, &[(entry, b1), (entry, b2), (b1, b3), (b2, b3)]);

        let doms1 = compute_dominators(&cfg);
        let doms2 = compute_dominators(&cfg);
        assert_eq!(doms1, doms2);
    }
}
