//! Pre-computed ICFG navigation index shared by IFDS and IDE solvers.
//!
//! Both solvers need to map instructions to blocks/functions, find successors,
//! locate entry/exit points, and resolve call sites. This module extracts that
//! shared map-building into a single struct built once and reused.

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirModule, Instruction, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId};

use crate::cfg::Cfg;
use crate::icfg::Icfg;

/// Pre-computed navigation index over an ICFG for use by IFDS/IDE solvers.
///
/// Contains helper maps that allow O(1) or O(log n) lookup of instruction
/// neighbourhoods, call/return relationships, and function entry/exit points.
#[derive(Debug)]
pub struct IcfgIndex<'a> {
    /// Instruction ID -> owning function.
    pub inst_to_func: BTreeMap<InstId, FunctionId>,
    /// Instruction ID -> owning block.
    pub inst_to_block: BTreeMap<InstId, BlockId>,
    /// Block ID -> owning function.
    pub block_to_func: BTreeMap<BlockId, FunctionId>,
    /// For each call site, the return-site instruction(s) — the next instruction
    /// after the call in the same block, or the first instructions of successor blocks.
    pub call_site_return_inst: BTreeMap<InstId, Vec<InstId>>,
    /// Call instruction -> callee `FunctionId`s (only defined, non-declaration callees).
    pub call_to_callees: BTreeMap<InstId, Vec<FunctionId>>,
    /// Block -> ordered list of instruction IDs within the block.
    pub block_instructions: BTreeMap<BlockId, Vec<InstId>>,
    /// Instruction -> next instruction in the same block (`None` if last).
    pub next_inst_in_block: BTreeMap<InstId, Option<InstId>>,
    /// Function -> first instruction of the entry block.
    pub func_entry_inst: BTreeMap<FunctionId, InstId>,
    /// Function -> terminator instructions of exit blocks (`Ret`/`Unreachable`).
    pub func_exit_insts: BTreeMap<FunctionId, BTreeSet<InstId>>,
    /// Callee function -> set of call-site instruction IDs that call it.
    pub callee_to_call_sites: BTreeMap<FunctionId, BTreeSet<InstId>>,
    /// Instruction ID -> `&Instruction` reference for quick lookup.
    pub inst_lookup: BTreeMap<InstId, &'a Instruction>,
}

impl<'a> IcfgIndex<'a> {
    /// Build the navigation index from a module and its ICFG.
    ///
    /// Iterates over all defined (non-declaration) functions once to populate
    /// every helper map. Borrows instructions from `module` with lifetime `'a`.
    // NOTE: This function builds all ICFG navigation maps in a single pass over
    // the module. Splitting would require multiple passes or scattered state.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn build(module: &'a AirModule, icfg: &Icfg) -> Self {
        let mut inst_to_func: BTreeMap<InstId, FunctionId> = BTreeMap::new();
        let mut inst_to_block: BTreeMap<InstId, BlockId> = BTreeMap::new();
        let mut block_to_func: BTreeMap<BlockId, FunctionId> = BTreeMap::new();
        let mut call_site_return_inst: BTreeMap<InstId, Vec<InstId>> = BTreeMap::new();
        let mut call_to_callees: BTreeMap<InstId, Vec<FunctionId>> = BTreeMap::new();
        let mut block_instructions: BTreeMap<BlockId, Vec<InstId>> = BTreeMap::new();
        let mut next_inst_in_block: BTreeMap<InstId, Option<InstId>> = BTreeMap::new();
        let mut func_entry_inst: BTreeMap<FunctionId, InstId> = BTreeMap::new();
        let mut func_exit_insts: BTreeMap<FunctionId, BTreeSet<InstId>> = BTreeMap::new();
        let mut callee_to_call_sites: BTreeMap<FunctionId, BTreeSet<InstId>> = BTreeMap::new();

        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            let Some(cfg) = icfg.cfg(func.id) else {
                continue;
            };

            // Determine entry instruction.
            let entry_block_id = func
                .entry_block
                .or_else(|| func.blocks.first().map(|b| b.id))
                .unwrap_or(BlockId::new(0));
            if let Some(entry_block) = func.blocks.iter().find(|b| b.id == entry_block_id) {
                if let Some(first_inst) = entry_block.instructions.first() {
                    func_entry_inst.insert(func.id, first_inst.id);
                }
            }

            // Populate exit instructions.
            let mut exits = BTreeSet::new();
            for block in &func.blocks {
                if cfg.exits.contains(&block.id) {
                    if let Some(term) = block.terminator() {
                        exits.insert(term.id);
                    }
                }
            }
            func_exit_insts.insert(func.id, exits);

            for block in &func.blocks {
                block_to_func.insert(block.id, func.id);
                let inst_ids: Vec<InstId> = block.instructions.iter().map(|i| i.id).collect();
                block_instructions.insert(block.id, inst_ids.clone());

                for (i, inst) in block.instructions.iter().enumerate() {
                    inst_to_func.insert(inst.id, func.id);
                    inst_to_block.insert(inst.id, block.id);

                    if i + 1 < block.instructions.len() {
                        next_inst_in_block.insert(inst.id, Some(block.instructions[i + 1].id));
                    } else {
                        next_inst_in_block.insert(inst.id, None);
                    }

                    // Identify call sites and their callees.
                    match &inst.op {
                        Operation::CallDirect { callee } => {
                            // Look up if callee has a defined body.
                            if let Some(callee_func) = module.function(*callee) {
                                if !callee_func.is_declaration {
                                    call_to_callees
                                        .entry(inst.id)
                                        .or_default()
                                        .push(callee_func.id);
                                    callee_to_call_sites
                                        .entry(callee_func.id)
                                        .or_default()
                                        .insert(inst.id);
                                }
                            }
                            // Return site = next instruction in same block (if any).
                            if let Some(Some(next)) = next_inst_in_block.get(&inst.id) {
                                call_site_return_inst
                                    .entry(inst.id)
                                    .or_default()
                                    .push(*next);
                            } else {
                                // If call is the last non-terminator, successors' first
                                // instructions are return sites.
                                collect_successor_first_insts(
                                    block.id,
                                    cfg,
                                    &block_instructions,
                                    &mut call_site_return_inst,
                                    inst.id,
                                );
                            }
                        }
                        Operation::CallIndirect { .. } => {
                            // Indirect calls: currently no resolution, treat as
                            // call-to-return only.
                            if let Some(Some(next)) = next_inst_in_block.get(&inst.id) {
                                call_site_return_inst
                                    .entry(inst.id)
                                    .or_default()
                                    .push(*next);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Build instruction lookup table.
        let mut inst_lookup: BTreeMap<InstId, &'a Instruction> = BTreeMap::new();
        for func in &module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    inst_lookup.insert(inst.id, inst);
                }
            }
        }

        Self {
            inst_to_func,
            inst_to_block,
            block_to_func,
            call_site_return_inst,
            call_to_callees,
            block_instructions,
            next_inst_in_block,
            func_entry_inst,
            func_exit_insts,
            callee_to_call_sites,
            inst_lookup,
        }
    }

    /// Get successor instructions for a given instruction.
    ///
    /// - If there is a next instruction in the same block, return it.
    /// - If this is a terminator, return first instructions of successor blocks.
    pub fn successor_instructions(
        &self,
        inst_id: InstId,
        func: FunctionId,
        instruction: &Instruction,
        icfg: &Icfg,
    ) -> Vec<InstId> {
        // If there is a next instruction in the same block, use that.
        if let Some(Some(next)) = self.next_inst_in_block.get(&inst_id) {
            return vec![*next];
        }

        // This is the last instruction in its block; find successor blocks.
        if !instruction.is_terminator() {
            // Non-terminator at end of block -- should not happen in well-formed IR,
            // but handle gracefully by returning empty.
            return Vec::new();
        }

        let Some(&block_id) = self.inst_to_block.get(&inst_id) else {
            return Vec::new();
        };

        // Get intraprocedural successors from the CFG.
        let Some(cfg) = icfg.cfg(func) else {
            return Vec::new();
        };

        let mut result = Vec::new();
        if let Some(succs) = cfg.successors_of(block_id) {
            for succ_block in succs {
                if let Some(insts) = self.block_instructions.get(succ_block) {
                    if let Some(first) = insts.first() {
                        result.push(*first);
                    }
                }
            }
        }

        result
    }
}

/// Collect first instructions of successor blocks as return sites.
fn collect_successor_first_insts(
    block_id: BlockId,
    cfg: &Cfg,
    block_instructions: &BTreeMap<BlockId, Vec<InstId>>,
    call_site_return_inst: &mut BTreeMap<InstId, Vec<InstId>>,
    call_inst_id: InstId,
) {
    if let Some(succs) = cfg.successors_of(block_id) {
        for succ_block in succs {
            if let Some(insts) = block_instructions.get(succ_block) {
                if let Some(first) = insts.first() {
                    call_site_return_inst
                        .entry(call_inst_id)
                        .or_default()
                        .push(*first);
                }
            }
        }
    }
}
