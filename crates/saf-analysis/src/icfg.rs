//! Interprocedural Control Flow Graph construction.
//!
//! See FR-GRAPH-002 for requirements.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::air::{AirModule, Operation};
use saf_core::ids::{BlockId, FunctionId, InstId};

use crate::callgraph::{CallGraph, CallGraphNode};
use crate::cfg::Cfg;

/// Edge type in the interprocedural CFG.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IcfgEdge {
    /// Intraprocedural edge (within same function).
    Intra,
    /// Call edge from caller to callee entry.
    Call {
        /// The call site instruction.
        site: InstId,
    },
    /// Return edge from callee exit to caller return point.
    Return {
        /// The call site instruction this returns to.
        site: InstId,
    },
}

/// Interprocedural control flow graph.
///
/// Combines per-function CFGs with interprocedural call/return edges.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Icfg {
    /// Per-function CFGs.
    pub cfgs: BTreeMap<FunctionId, Cfg>,
    /// Interprocedural edges: list of `(from_block, to_block, edge_type)`.
    /// Uses a `BTreeSet` for deterministic iteration and deduplication.
    pub inter_edges: BTreeSet<(BlockId, BlockId, IcfgEdge)>,
    /// Call site to `(caller_block, callee_entry)` mapping.
    ///
    /// For multi-target indirect calls, a single site maps to multiple
    /// `(caller_block, callee_entry)` pairs.
    pub call_site_map: BTreeMap<InstId, Vec<(BlockId, BlockId)>>,
    /// Index from source block to interprocedural successor blocks for O(log n) lookup.
    pub(crate) inter_successors: BTreeMap<BlockId, BTreeSet<BlockId>>,
}

impl Icfg {
    /// Build an ICFG from an AIR module and call graph.
    ///
    /// Creates CFGs for all defined functions and adds interprocedural edges
    /// for resolved direct calls.
    #[must_use]
    pub fn build(module: &AirModule, callgraph: &CallGraph) -> Self {
        let mut cfgs = BTreeMap::new();
        let mut inter_edges = BTreeSet::new();
        let mut call_site_map: BTreeMap<InstId, Vec<(BlockId, BlockId)>> = BTreeMap::new();

        // Build CFG for each defined function
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            let cfg = Cfg::build(func);
            cfgs.insert(func.id, cfg);
        }

        // Build block-to-function map for finding which block a call site is in
        let mut block_to_func: BTreeMap<BlockId, FunctionId> = BTreeMap::new();
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }
            for block in &func.blocks {
                block_to_func.insert(block.id, func.id);
            }
        }

        // Scan for call sites and add interprocedural edges
        for func in &module.functions {
            if func.is_declaration {
                continue;
            }

            for block in &func.blocks {
                for inst in &block.instructions {
                    if matches!(&inst.op, Operation::CallDirect { .. }) {
                        // Get target node from call graph - only add edges for defined functions
                        if let Some(CallGraphNode::Function(callee_id)) =
                            callgraph.call_site_target(inst.id)
                        {
                            if let Some(callee_cfg) = cfgs.get(callee_id) {
                                // Call edge: caller block -> callee entry
                                inter_edges.insert((
                                    block.id,
                                    callee_cfg.entry,
                                    IcfgEdge::Call { site: inst.id },
                                ));
                                call_site_map
                                    .entry(inst.id)
                                    .or_default()
                                    .push((block.id, callee_cfg.entry));

                                // Return edges: callee exits -> caller block
                                for exit in &callee_cfg.exits {
                                    inter_edges.insert((
                                        *exit,
                                        block.id,
                                        IcfgEdge::Return { site: inst.id },
                                    ));
                                }
                            }
                        }
                    }
                    // Indirect calls: edges added later via resolve_indirect when PTA runs
                }
            }
        }

        // Build the inter_successors index from inter_edges
        let mut inter_successors: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        for (from, to, _) in &inter_edges {
            inter_successors.entry(*from).or_default().insert(*to);
        }

        Self {
            cfgs,
            inter_edges,
            call_site_map,
            inter_successors,
        }
    }

    /// Get the CFG for a function.
    #[must_use]
    pub fn cfg(&self, func: FunctionId) -> Option<&Cfg> {
        self.cfgs.get(&func)
    }

    /// Get all call edges.
    #[must_use]
    pub fn call_edges(&self) -> Vec<((BlockId, BlockId), InstId)> {
        self.inter_edges
            .iter()
            .filter_map(|(from, to, edge)| match edge {
                IcfgEdge::Call { site } => Some(((*from, *to), *site)),
                _ => None,
            })
            .collect()
    }

    /// Get all return edges.
    #[must_use]
    pub fn return_edges(&self) -> Vec<((BlockId, BlockId), InstId)> {
        self.inter_edges
            .iter()
            .filter_map(|(from, to, edge)| match edge {
                IcfgEdge::Return { site } => Some(((*from, *to), *site)),
                _ => None,
            })
            .collect()
    }

    /// Get interprocedural successors of a block.
    ///
    /// Returns both intraprocedural successors (from CFG) and
    /// interprocedural successors (call/return edges).
    #[must_use]
    pub fn successors(&self, block: BlockId, func: FunctionId) -> BTreeSet<BlockId> {
        let mut result = BTreeSet::new();

        // Intraprocedural successors
        if let Some(cfg) = self.cfgs.get(&func) {
            if let Some(succs) = cfg.successors_of(block) {
                result.extend(succs.iter().copied());
            }
        }

        // Interprocedural successors (O(log n) lookup via index)
        if let Some(succs) = self.inter_successors.get(&block) {
            result.extend(succs);
        }

        result
    }

    /// Resolve indirect call site to specific targets.
    ///
    /// Called by PTA to add edges for resolved indirect calls.
    pub fn resolve_indirect(
        &mut self,
        site: InstId,
        caller_block: BlockId,
        targets: &[FunctionId],
    ) {
        for target_id in targets {
            if let Some(callee_cfg) = self.cfgs.get(target_id) {
                let callee_entry = callee_cfg.entry;
                let callee_exits: Vec<BlockId> = callee_cfg.exits.iter().copied().collect();

                // Add call edge
                self.inter_edges
                    .insert((caller_block, callee_entry, IcfgEdge::Call { site }));
                self.inter_successors
                    .entry(caller_block)
                    .or_default()
                    .insert(callee_entry);

                // Update `call_site_map` so return flow analysis can find
                // the correct return site for this resolved callee.
                self.call_site_map
                    .entry(site)
                    .or_default()
                    .push((caller_block, callee_entry));

                // Add return edges
                for exit in &callee_exits {
                    self.inter_edges
                        .insert((*exit, caller_block, IcfgEdge::Return { site }));
                    self.inter_successors
                        .entry(*exit)
                        .or_default()
                        .insert(caller_block);
                }
            }
        }
    }
}

// =============================================================================
// Export types
// =============================================================================

/// Exportable ICFG representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcfgExport {
    /// Per-function CFG exports.
    pub functions: Vec<crate::cfg::CfgExport>,
    /// Interprocedural edges.
    pub inter_edges: Vec<IcfgEdgeExport>,
}

/// Exportable interprocedural edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcfgEdgeExport {
    /// Source block ID as hex string.
    pub src: String,
    /// Destination block ID as hex string.
    pub dst: String,
    /// Edge kind: "call" or "return".
    pub kind: String,
    /// Call site instruction ID (for call/return edges).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
}

impl Icfg {
    /// Export the ICFG as a [`PropertyGraph`].
    ///
    /// Each basic block becomes a node with labels `["Block"]` and optionally
    /// `"Entry"` or `"Exit"`.  Intraprocedural edges use `FLOWS_TO`,
    /// interprocedural edges use `CALL` or `RETURN` with a `site` property.
    #[must_use]
    pub fn to_pg(
        &self,
        module: &AirModule,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};

        let mut pg = PropertyGraph::new("icfg");

        // Build function-name lookup
        let func_names: BTreeMap<FunctionId, &str> = module
            .functions
            .iter()
            .map(|f| (f.id, f.name.as_str()))
            .collect();

        // Emit nodes for every block in every CFG
        for (func_id, cfg) in &self.cfgs {
            let func_name = func_names.get(func_id).copied().unwrap_or("unknown");

            for block_id in cfg.successors.keys() {
                let mut labels = vec!["Block".to_string()];
                if *block_id == cfg.entry {
                    labels.push("Entry".to_string());
                }
                if cfg.exits.contains(block_id) {
                    labels.push("Exit".to_string());
                }

                let mut properties = BTreeMap::new();
                properties.insert("function".to_string(), serde_json::json!(func_name));

                let mut pg_node = PgNode {
                    id: block_id.to_hex(),
                    labels,
                    properties,
                };
                enrich_node(&mut pg_node, resolver);
                pg.nodes.push(pg_node);

                // Intraprocedural edges
                if let Some(succs) = cfg.successors_of(*block_id) {
                    for succ in succs {
                        pg.edges.push(PgEdge {
                            src: block_id.to_hex(),
                            dst: succ.to_hex(),
                            edge_type: "FLOWS_TO".to_string(),
                            properties: BTreeMap::new(),
                        });
                    }
                }
            }
        }

        // Interprocedural edges
        for (from, to, edge) in &self.inter_edges {
            let (edge_type, site_hex) = match edge {
                IcfgEdge::Intra => ("FLOWS_TO", None),
                IcfgEdge::Call { site } => ("CALL", Some(site.to_hex())),
                IcfgEdge::Return { site } => ("RETURN", Some(site.to_hex())),
            };

            let mut properties = BTreeMap::new();
            if let Some(hex) = site_hex {
                properties.insert("site".to_string(), serde_json::json!(hex));
            }

            pg.edges.push(PgEdge {
                src: from.to_hex(),
                dst: to.to_hex(),
                edge_type: edge_type.to_string(),
                properties,
            });
        }

        pg
    }

    /// Export ICFG to serializable format.
    #[must_use]
    pub fn export(&self, module: &AirModule) -> IcfgExport {
        let functions: Vec<_> = self
            .cfgs
            .iter()
            .filter_map(|(func_id, cfg)| {
                module
                    .functions
                    .iter()
                    .find(|f| f.id == *func_id)
                    .map(|func| cfg.export(func))
            })
            .collect();

        let inter_edges: Vec<_> = self
            .inter_edges
            .iter()
            .map(|(from, to, edge)| {
                let (kind, site) = match edge {
                    IcfgEdge::Intra => ("intra".to_string(), None),
                    IcfgEdge::Call { site } => ("call".to_string(), Some(site.to_hex())),
                    IcfgEdge::Return { site } => ("return".to_string(), Some(site.to_hex())),
                };
                IcfgEdgeExport {
                    src: from.to_hex(),
                    dst: to.to_hex(),
                    kind,
                    site,
                }
            })
            .collect();

        IcfgExport {
            functions,
            inter_edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction};
    use saf_core::ids::ModuleId;

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

    fn make_function(id: u128, name: &str, blocks: Vec<AirBlock>) -> AirFunction {
        AirFunction {
            id: FunctionId::new(id),
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

    fn make_block(id: u128, instructions: Vec<Instruction>) -> AirBlock {
        AirBlock {
            id: BlockId::new(id),
            label: None,
            instructions,
        }
    }

    #[test]
    fn icfg_no_calls() {
        // Single function, no interprocedural edges
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![Instruction::new(InstId::new(100), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        assert_eq!(icfg.cfgs.len(), 1);
        assert!(icfg.inter_edges.is_empty());
    }

    #[test]
    fn icfg_direct_call() {
        // main calls helper
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        // Should have call edge and return edge
        let call_edges = icfg.call_edges();
        assert_eq!(call_edges.len(), 1);
        assert_eq!(call_edges[0].0, (BlockId::new(1), BlockId::new(2)));
        assert_eq!(call_edges[0].1, InstId::new(100));

        let return_edges = icfg.return_edges();
        assert_eq!(return_edges.len(), 1);
        assert_eq!(return_edges[0].0, (BlockId::new(2), BlockId::new(1)));
        assert_eq!(return_edges[0].1, InstId::new(100));
    }

    #[test]
    fn icfg_multiple_exits() {
        // helper has two exit blocks
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![
                make_block(
                    2,
                    vec![Instruction::new(
                        InstId::new(200),
                        Operation::CondBr {
                            then_target: BlockId::new(3),
                            else_target: BlockId::new(4),
                        },
                    )],
                ),
                make_block(3, vec![Instruction::new(InstId::new(300), Operation::Ret)]),
                make_block(4, vec![Instruction::new(InstId::new(400), Operation::Ret)]),
            ],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        // Should have one call edge and two return edges
        let call_edges = icfg.call_edges();
        assert_eq!(call_edges.len(), 1);

        let return_edges = icfg.return_edges();
        assert_eq!(return_edges.len(), 2);
    }

    #[test]
    fn icfg_recursive_call() {
        // fib calls itself
        let fib = make_function(
            1,
            "fib",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(1),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![fib]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        // Self-call edge: block 1 -> block 1
        let call_edges = icfg.call_edges();
        assert_eq!(call_edges.len(), 1);
        assert_eq!(call_edges[0].0, (BlockId::new(1), BlockId::new(1)));
    }

    #[test]
    fn icfg_call_site_map() {
        // Verify call site mapping
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        let entries = icfg.call_site_map.get(&InstId::new(100)).unwrap();
        assert_eq!(entries.len(), 1);
        let (caller_block, callee_entry) = &entries[0];
        assert_eq!(*caller_block, BlockId::new(1));
        assert_eq!(*callee_entry, BlockId::new(2));
    }

    #[test]
    fn icfg_successors() {
        // Test combined intra and inter successors
        let main = make_function(
            1,
            "main",
            vec![
                make_block(
                    1,
                    vec![
                        Instruction::new(
                            InstId::new(100),
                            Operation::CallDirect {
                                callee: FunctionId::new(2),
                            },
                        ),
                        Instruction::new(
                            InstId::new(101),
                            Operation::Br {
                                target: BlockId::new(10),
                            },
                        ),
                    ],
                ),
                make_block(10, vec![Instruction::new(InstId::new(102), Operation::Ret)]),
            ],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        // Block 1 has intra successor (block 10) and inter successor (block 2)
        let succs = icfg.successors(BlockId::new(1), FunctionId::new(1));
        assert!(succs.contains(&BlockId::new(10))); // intra edge
        assert!(succs.contains(&BlockId::new(2))); // call edge
    }

    #[test]
    fn icfg_external_call_no_edge() {
        // Calls to external functions don't create interprocedural edges
        let printf = AirFunction {
            id: FunctionId::new(2),
            name: "printf".to_string(),
            params: Vec::new(),
            blocks: Vec::new(),
            entry_block: None,
            is_declaration: true,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let module = make_module(vec![main, printf]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        // No interprocedural edges for external call
        assert!(icfg.inter_edges.is_empty());
    }

    #[test]
    fn icfg_export_is_deterministic() {
        let main = make_function(
            1,
            "main",
            vec![make_block(
                1,
                vec![
                    Instruction::new(
                        InstId::new(100),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    ),
                    Instruction::new(InstId::new(101), Operation::Ret),
                ],
            )],
        );
        let helper = make_function(
            2,
            "helper",
            vec![make_block(
                2,
                vec![Instruction::new(InstId::new(200), Operation::Ret)],
            )],
        );
        let module = make_module(vec![main, helper]);
        let cg = CallGraph::build(&module);
        let icfg = Icfg::build(&module, &cg);

        let export1 = serde_json::to_string(&icfg.export(&module)).unwrap();
        let export2 = serde_json::to_string(&icfg.export(&module)).unwrap();
        assert_eq!(export1, export2);
    }
}
