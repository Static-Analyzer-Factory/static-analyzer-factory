//! Control Flow Graph construction (per function).
//!
//! See FR-GRAPH-001 for requirements.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::air::{AirFunction, Operation};
use saf_core::ids::{BlockId, FunctionId};

use saf_core::span::{SourceFile, Span};

use crate::display::DisplayResolver;
use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node, span_to_property};
use crate::graph_algo::Successors;

/// Control flow graph for a single function.
///
/// Block-level CFG with deterministic iteration order (BTreeMap/BTreeSet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cfg {
    /// The function this CFG belongs to.
    pub function: FunctionId,
    /// Entry block ID.
    pub entry: BlockId,
    /// Exit blocks (blocks ending with Ret or Unreachable).
    pub exits: BTreeSet<BlockId>,
    /// Successor map: block -> set of successor blocks.
    pub successors: BTreeMap<BlockId, BTreeSet<BlockId>>,
    /// Predecessor map: block -> set of predecessor blocks.
    pub predecessors: BTreeMap<BlockId, BTreeSet<BlockId>>,
}

impl Cfg {
    /// Build a CFG from an AIR function.
    ///
    /// Extracts control flow edges from block terminators.
    ///
    /// # Panics
    ///
    /// Panics if a terminator references a block that doesn't exist in the function.
    /// This indicates malformed AIR input.
    #[must_use]
    pub fn build(func: &AirFunction) -> Self {
        let mut successors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        let mut predecessors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        let mut exits = BTreeSet::new();

        // Initialize all blocks with empty successor/predecessor sets
        for block in &func.blocks {
            successors.insert(block.id, BTreeSet::new());
            predecessors.insert(block.id, BTreeSet::new());
        }

        // Extract edges from terminators
        for block in &func.blocks {
            let block_succs = extract_successors(block.terminator());
            if block_succs.is_empty() {
                exits.insert(block.id);
            }
            for succ in &block_succs {
                successors
                    .get_mut(&block.id)
                    .expect("block was inserted in initialization loop")
                    .insert(*succ);
                predecessors
                    .get_mut(succ)
                    .expect("successor block should exist in function")
                    .insert(block.id);
            }
        }

        // Determine entry block
        let entry = func.entry_block.unwrap_or_else(|| {
            func.blocks
                .first()
                .map_or_else(|| BlockId::new(0), |b| b.id)
        });

        Self {
            function: func.id,
            entry,
            exits,
            successors,
            predecessors,
        }
    }

    /// Get all block IDs in the CFG.
    #[must_use]
    pub fn blocks(&self) -> BTreeSet<BlockId> {
        self.successors.keys().copied().collect()
    }

    /// Check if a block is an exit block.
    #[must_use]
    pub fn is_exit(&self, block: BlockId) -> bool {
        self.exits.contains(&block)
    }

    /// Get successors of a block.
    #[must_use]
    pub fn successors_of(&self, block: BlockId) -> Option<&BTreeSet<BlockId>> {
        self.successors.get(&block)
    }

    /// Get predecessors of a block.
    #[must_use]
    pub fn predecessors_of(&self, block: BlockId) -> Option<&BTreeSet<BlockId>> {
        self.predecessors.get(&block)
    }

    /// Check if `target` block is reachable from `source` block.
    ///
    /// Uses BFS traversal to check if there exists a path from source to target.
    /// This is used for temporal ordering: if target is reachable from source,
    /// then instructions in target can execute after instructions in source.
    #[must_use]
    pub fn is_reachable(&self, source: BlockId, target: BlockId) -> bool {
        use std::collections::VecDeque;

        // Same block is trivially reachable
        if source == target {
            return true;
        }

        // BFS traversal
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(source);

        while let Some(block) = queue.pop_front() {
            if block == target {
                return true;
            }

            if !visited.insert(block) {
                continue;
            }

            if let Some(succs) = self.successors.get(&block) {
                for succ in succs {
                    if !visited.contains(succ) {
                        queue.push_back(*succ);
                    }
                }
            }
        }

        false
    }
}

impl Successors<BlockId> for Cfg {
    fn successors(&self, node: &BlockId) -> Option<&BTreeSet<BlockId>> {
        self.successors.get(node)
    }
}

/// Extract successor block IDs from a terminator instruction.
fn extract_successors(terminator: Option<&saf_core::air::Instruction>) -> Vec<BlockId> {
    let Some(inst) = terminator else {
        return Vec::new();
    };

    match &inst.op {
        Operation::Br { target } => vec![*target],
        Operation::CondBr {
            then_target,
            else_target,
        } => vec![*then_target, *else_target],
        Operation::Switch { default, cases } => {
            let mut targets = vec![*default];
            targets.extend(cases.iter().map(|(_, target)| *target));
            targets
        }
        // Terminators with no successors and non-terminator operations
        _ => Vec::new(),
    }
}

// =============================================================================
// Export types
// =============================================================================

/// Exportable CFG representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgExport {
    /// Function ID as hex string.
    pub function: String,
    /// Entry block ID as hex string.
    pub entry: String,
    /// Exit block IDs as hex strings.
    pub exits: Vec<String>,
    /// Block information.
    pub blocks: Vec<CfgBlockExport>,
}

/// Exportable block representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfgBlockExport {
    /// Block ID as hex string.
    pub id: String,
    /// Optional block label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Successor block IDs as hex strings.
    pub successors: Vec<String>,
}

impl Cfg {
    /// Export CFG to serializable format.
    #[must_use]
    pub fn export(&self, func: &AirFunction) -> CfgExport {
        let block_labels: BTreeMap<BlockId, Option<String>> = func
            .blocks
            .iter()
            .map(|b| (b.id, b.label.clone()))
            .collect();

        let blocks = self
            .successors
            .iter()
            .map(|(id, succs)| CfgBlockExport {
                id: id.to_hex(),
                label: block_labels.get(id).and_then(Clone::clone),
                successors: succs.iter().map(|s| s.to_hex()).collect(),
            })
            .collect();

        CfgExport {
            function: self.function.to_hex(),
            entry: self.entry.to_hex(),
            exits: self.exits.iter().map(|e| e.to_hex()).collect(),
            blocks,
        }
    }

    /// Export CFG as a unified `PropertyGraph`.
    ///
    /// Each block becomes a node with label `["Block"]` plus `"Entry"` or
    /// `"Exit"` as appropriate. Blocks are tagged with a `function` property
    /// for subgraph clustering. Edges have `edge_type` = "FLOWS_TO".
    #[must_use]
    pub fn to_pg(
        &self,
        func: &AirFunction,
        source_files: &[SourceFile],
        resolver: Option<&DisplayResolver<'_>>,
    ) -> PropertyGraph {
        let block_labels: BTreeMap<BlockId, Option<String>> = func
            .blocks
            .iter()
            .map(|b| (b.id, b.label.clone()))
            .collect();

        // Build block_id → first instruction-with-span lookup
        let block_spans: BTreeMap<BlockId, &Span> = func
            .blocks
            .iter()
            .filter_map(|b| {
                let span = b.instructions.iter().find_map(|i| i.span.as_ref())?;
                Some((b.id, span))
            })
            .collect();

        let func_name = &func.name;
        let mut pg = PropertyGraph::new("cfg");

        for (block_id, succs) in &self.successors {
            let mut labels = vec!["Block".to_string()];
            if *block_id == self.entry {
                labels.push("Entry".to_string());
            }
            if self.exits.contains(block_id) {
                labels.push("Exit".to_string());
            }

            let mut properties = BTreeMap::new();
            properties.insert(
                "function".to_string(),
                serde_json::Value::String(func_name.clone()),
            );
            if let Some(Some(label)) = block_labels.get(block_id) {
                properties.insert("name".to_string(), serde_json::Value::String(label.clone()));
            }
            if let Some(span) = block_spans.get(block_id) {
                properties.insert("span".to_string(), span_to_property(span, source_files));
            }

            let id = block_id.to_hex();
            let mut pg_node = PgNode {
                id: id.clone(),
                labels,
                properties,
            };
            enrich_node(&mut pg_node, resolver);
            pg.nodes.push(pg_node);

            for succ in succs {
                pg.edges.push(PgEdge {
                    src: id.clone(),
                    dst: succ.to_hex(),
                    edge_type: "FLOWS_TO".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        pg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, Instruction};
    use saf_core::ids::InstId;

    fn make_function(blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::derive(b"test"),
            name: "test".to_string(),
            params: Vec::new(),
            blocks,
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        }
    }

    fn make_block(id: u128, terminator: Operation) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions: vec![Instruction::new(InstId::new(id * 100), terminator)],
        }
    }

    #[test]
    fn cfg_linear() {
        // entry -> b1 -> exit
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(
                2,
                Operation::Br {
                    target: BlockId::new(3),
                },
            ),
            make_block(3, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        assert_eq!(cfg.entry, BlockId::new(1));
        assert_eq!(cfg.exits, [BlockId::new(3)].into_iter().collect());

        // Check successors
        assert_eq!(
            cfg.successors_of(BlockId::new(1)),
            Some(&[BlockId::new(2)].into_iter().collect())
        );
        assert_eq!(
            cfg.successors_of(BlockId::new(2)),
            Some(&[BlockId::new(3)].into_iter().collect())
        );
        assert_eq!(cfg.successors_of(BlockId::new(3)), Some(&BTreeSet::new()));

        // Check predecessors
        assert_eq!(cfg.predecessors_of(BlockId::new(1)), Some(&BTreeSet::new()));
        assert_eq!(
            cfg.predecessors_of(BlockId::new(2)),
            Some(&[BlockId::new(1)].into_iter().collect())
        );
        assert_eq!(
            cfg.predecessors_of(BlockId::new(3)),
            Some(&[BlockId::new(2)].into_iter().collect())
        );
    }

    #[test]
    fn cfg_diamond() {
        // Diamond pattern:
        //     entry (1)
        //    /      \
        //  then(2)  else(3)
        //    \      /
        //     merge (4) -> exit
        let blocks = vec![
            make_block(
                1,
                Operation::CondBr {
                    then_target: BlockId::new(2),
                    else_target: BlockId::new(3),
                },
            ),
            make_block(
                2,
                Operation::Br {
                    target: BlockId::new(4),
                },
            ),
            make_block(
                3,
                Operation::Br {
                    target: BlockId::new(4),
                },
            ),
            make_block(4, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        assert_eq!(cfg.entry, BlockId::new(1));
        assert_eq!(cfg.exits, [BlockId::new(4)].into_iter().collect());

        // Entry has two successors
        let entry_succs = cfg.successors_of(BlockId::new(1)).unwrap();
        assert!(entry_succs.contains(&BlockId::new(2)));
        assert!(entry_succs.contains(&BlockId::new(3)));

        // Merge has two predecessors
        let merge_preds = cfg.predecessors_of(BlockId::new(4)).unwrap();
        assert!(merge_preds.contains(&BlockId::new(2)));
        assert!(merge_preds.contains(&BlockId::new(3)));
    }

    #[test]
    fn cfg_loop() {
        // Loop pattern:
        // entry (1) -> header (2) -> body (3) -> header (back edge)
        //                 \-> exit (4)
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(
                2,
                Operation::CondBr {
                    then_target: BlockId::new(3),
                    else_target: BlockId::new(4),
                },
            ),
            make_block(
                3,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(4, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        // Header has two predecessors: entry and body (back edge)
        let header_preds = cfg.predecessors_of(BlockId::new(2)).unwrap();
        assert!(header_preds.contains(&BlockId::new(1)));
        assert!(header_preds.contains(&BlockId::new(3)));

        // Body has one successor (back edge to header)
        assert_eq!(
            cfg.successors_of(BlockId::new(3)),
            Some(&[BlockId::new(2)].into_iter().collect())
        );
    }

    #[test]
    fn cfg_switch() {
        // Switch with 3 cases + default
        let blocks = vec![
            make_block(
                1,
                Operation::Switch {
                    default: BlockId::new(5),
                    cases: vec![
                        (0, BlockId::new(2)),
                        (1, BlockId::new(3)),
                        (2, BlockId::new(4)),
                    ],
                },
            ),
            make_block(2, Operation::Ret),
            make_block(3, Operation::Ret),
            make_block(4, Operation::Ret),
            make_block(5, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        // Entry has 4 successors (default + 3 cases)
        let entry_succs = cfg.successors_of(BlockId::new(1)).unwrap();
        assert_eq!(entry_succs.len(), 4);
        assert!(entry_succs.contains(&BlockId::new(2)));
        assert!(entry_succs.contains(&BlockId::new(3)));
        assert!(entry_succs.contains(&BlockId::new(4)));
        assert!(entry_succs.contains(&BlockId::new(5)));

        // All case blocks are exits
        assert_eq!(cfg.exits.len(), 4);
    }

    #[test]
    fn cfg_unreachable() {
        // Block ending with unreachable is also an exit
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(2, Operation::Unreachable),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        assert!(cfg.exits.contains(&BlockId::new(2)));
    }

    #[test]
    fn cfg_multiple_exits() {
        // Function with multiple exit points
        let blocks = vec![
            make_block(
                1,
                Operation::CondBr {
                    then_target: BlockId::new(2),
                    else_target: BlockId::new(3),
                },
            ),
            make_block(2, Operation::Ret),
            make_block(3, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        assert_eq!(cfg.exits.len(), 2);
        assert!(cfg.exits.contains(&BlockId::new(2)));
        assert!(cfg.exits.contains(&BlockId::new(3)));
    }

    #[test]
    fn cfg_predecessors_is_reverse_of_successors() {
        // Property: for every edge A -> B, A is in B's predecessors
        let blocks = vec![
            make_block(
                1,
                Operation::CondBr {
                    then_target: BlockId::new(2),
                    else_target: BlockId::new(3),
                },
            ),
            make_block(
                2,
                Operation::Br {
                    target: BlockId::new(4),
                },
            ),
            make_block(
                3,
                Operation::Br {
                    target: BlockId::new(4),
                },
            ),
            make_block(4, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        for (from, succs) in &cfg.successors {
            for to in succs {
                let preds = cfg.predecessors_of(*to).unwrap();
                assert!(
                    preds.contains(from),
                    "Edge {from:?} -> {to:?} not reflected in predecessors"
                );
            }
        }
    }

    #[test]
    fn cfg_export_is_deterministic() {
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(2, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        let export1 = serde_json::to_string(&cfg.export(&func)).unwrap();
        let export2 = serde_json::to_string(&cfg.export(&func)).unwrap();
        assert_eq!(export1, export2);
    }

    #[test]
    fn cfg_to_pg_basic() {
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(2, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);
        let pg = cfg.to_pg(&func, &[], None);

        assert_eq!(pg.graph_type, "cfg");
        assert_eq!(pg.nodes.len(), 2);
        assert_eq!(pg.edges.len(), 1);

        // Edge should be FLOWS_TO
        assert_eq!(pg.edges[0].edge_type, "FLOWS_TO");

        // All nodes should have function property
        assert!(
            pg.nodes
                .iter()
                .all(|n| n.properties.get("function").and_then(|v| v.as_str()) == Some("test"))
        );
    }

    #[test]
    fn cfg_to_pg_entry_exit_labels() {
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(2, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);
        let pg = cfg.to_pg(&func, &[], None);

        // Entry node should have ["Block", "Entry"]
        let entry_id = BlockId::new(1).to_hex();
        let entry_node = pg.nodes.iter().find(|n| n.id == entry_id).unwrap();
        assert!(entry_node.labels.contains(&"Block".to_string()));
        assert!(entry_node.labels.contains(&"Entry".to_string()));

        // Exit node should have ["Block", "Exit"]
        let exit_id = BlockId::new(2).to_hex();
        let exit_node = pg.nodes.iter().find(|n| n.id == exit_id).unwrap();
        assert!(exit_node.labels.contains(&"Block".to_string()));
        assert!(exit_node.labels.contains(&"Exit".to_string()));
    }

    #[test]
    fn cfg_to_pg_is_deterministic() {
        let blocks = vec![
            make_block(
                1,
                Operation::Br {
                    target: BlockId::new(2),
                },
            ),
            make_block(2, Operation::Ret),
        ];
        let func = make_function(blocks);
        let cfg = Cfg::build(&func);

        let pg1 = serde_json::to_string(&cfg.to_pg(&func, &[], None)).unwrap();
        let pg2 = serde_json::to_string(&cfg.to_pg(&func, &[], None)).unwrap();
        assert_eq!(pg1, pg2);
    }
}
