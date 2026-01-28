//! Def-Use graph construction.
//!
//! See FR-GRAPH-003 for requirements.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use saf_core::air::{AirFunction, AirModule, Operation};
use saf_core::ids::{InstId, ValueId};

use saf_core::span::Span;

use crate::display::DisplayResolver;
use crate::export::{
    PgEdge, PgNode, PropertyGraph, build_value_type_lookup, enrich_node, insert_type_property,
    span_to_property,
};

/// Def-Use graph for SSA values.
///
/// Tracks where values are defined and where they are used.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DefUseGraph {
    /// Map from value to the set of instructions that use it.
    pub uses: BTreeMap<ValueId, BTreeSet<InstId>>,
    /// Map from value to the instruction that defines it.
    /// `None` means the value is a function parameter (no defining instruction).
    pub defs: BTreeMap<ValueId, Option<InstId>>,
}

impl DefUseGraph {
    /// Returns `true` if the graph has no definitions or uses.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty() && self.uses.is_empty()
    }

    /// Build a def-use graph from an AIR module.
    ///
    /// Scans all functions to build definition and use information.
    #[must_use]
    pub fn build(module: &AirModule) -> Self {
        let mut uses: BTreeMap<ValueId, BTreeSet<InstId>> = BTreeMap::new();
        let mut defs: BTreeMap<ValueId, Option<InstId>> = BTreeMap::new();

        for func in &module.functions {
            if func.is_declaration {
                continue; // No body to analyze
            }

            // Parameters are definitions without an instruction
            for param in &func.params {
                defs.insert(param.id, None);
                uses.entry(param.id).or_default();
            }

            // Scan instructions
            for block in &func.blocks {
                for inst in &block.instructions {
                    // Destination is a definition
                    if let Some(dst) = inst.dst {
                        defs.insert(dst, Some(inst.id));
                        uses.entry(dst).or_default();
                    }

                    // Operands are uses
                    for operand in &inst.operands {
                        uses.entry(*operand).or_default().insert(inst.id);
                    }

                    // Phi nodes: incoming values are also uses
                    if let Operation::Phi { incoming } = &inst.op {
                        for (_, value) in incoming {
                            uses.entry(*value).or_default().insert(inst.id);
                        }
                    }
                }
            }
        }

        // Also handle global variables
        for global in &module.globals {
            defs.insert(global.id, None);
            uses.entry(global.id).or_default();
        }

        Self { uses, defs }
    }

    /// Get all instructions that use a value.
    #[must_use]
    pub fn users_of(&self, value: ValueId) -> Option<&BTreeSet<InstId>> {
        self.uses.get(&value)
    }

    /// Get the instruction that defines a value.
    ///
    /// Returns `Some(Some(inst_id))` for instruction-defined values,
    /// `Some(None)` for parameters/globals,
    /// `None` if the value is unknown.
    #[must_use]
    pub fn def_of(&self, value: ValueId) -> Option<Option<InstId>> {
        self.defs.get(&value).copied()
    }

    /// Check if a value is a parameter (defined without an instruction).
    #[must_use]
    pub fn is_parameter(&self, value: ValueId) -> bool {
        self.defs.get(&value) == Some(&None)
    }

    /// Get all values in the graph.
    #[must_use]
    pub fn values(&self) -> BTreeSet<ValueId> {
        self.defs.keys().copied().collect()
    }

    /// Get the number of uses for a value.
    #[must_use]
    pub fn use_count(&self, value: ValueId) -> usize {
        self.uses.get(&value).map_or(0, BTreeSet::len)
    }

    /// Check if a value is dead (has no uses).
    #[must_use]
    pub fn is_dead(&self, value: ValueId) -> bool {
        self.use_count(value) == 0
    }
}

// =============================================================================
// Export types
// =============================================================================

/// Exportable def-use graph representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefUseExport {
    /// All definitions.
    pub definitions: Vec<DefExport>,
    /// All uses.
    pub uses: Vec<UseExport>,
}

/// Exportable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefExport {
    /// Value ID as hex string.
    pub value: String,
    /// Defining instruction ID, or null for parameters.
    pub defined_by: Option<String>,
}

/// Exportable use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseExport {
    /// Value ID as hex string.
    pub value: String,
    /// Using instruction ID as hex string.
    pub used_by: String,
}

impl DefUseGraph {
    /// Export def-use graph to serializable format.
    #[must_use]
    pub fn export(&self) -> DefUseExport {
        let definitions: Vec<_> = self
            .defs
            .iter()
            .map(|(value, def_inst)| DefExport {
                value: value.to_hex(),
                defined_by: def_inst.map(InstId::to_hex),
            })
            .collect();

        let mut uses = Vec::new();
        for (value, users) in &self.uses {
            for user in users {
                uses.push(UseExport {
                    value: value.to_hex(),
                    used_by: user.to_hex(),
                });
            }
        }

        DefUseExport { definitions, uses }
    }

    /// Export def-use graph as a unified [`PropertyGraph`].
    ///
    /// The `module` parameter provides span information for instruction
    /// and value nodes, and type annotations for value nodes.
    // NOTE: This method builds nodes for both def and use sides of the graph
    // with span lookups, making it naturally long. Splitting would obscure
    // the export logic.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn to_pg(
        &self,
        module: &AirModule,
        resolver: Option<&DisplayResolver<'_>>,
    ) -> PropertyGraph {
        // Build ID → span lookup from all instructions
        let mut id_spans: BTreeMap<u128, &Span> = BTreeMap::new();
        // Build param ID → parent function name for parameter nodes
        let mut param_func_name: BTreeMap<u128, &str> = BTreeMap::new();
        let func_by_id: BTreeMap<u128, &AirFunction> =
            module.functions.iter().map(|f| (f.id.raw(), f)).collect();

        for func in &module.functions {
            for param in &func.params {
                param_func_name.insert(param.id.raw(), &func.name);
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(span) = &inst.span {
                        id_spans.insert(inst.id.raw(), span);
                        if let Some(dst) = inst.dst {
                            id_spans.insert(dst.raw(), span);
                        }
                        // Propagate call-site spans to callee parameter nodes
                        if let Operation::CallDirect { callee } = &inst.op {
                            if let Some(callee_fn) = func_by_id.get(&callee.raw()) {
                                for param in &callee_fn.params {
                                    id_spans.entry(param.id.raw()).or_insert(span);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Build ValueId → TypeId lookup for type annotations on nodes
        let value_types = build_value_type_lookup(module);

        let mut pg = PropertyGraph::new("defuse");
        let mut seen_values = BTreeSet::new();
        let mut seen_insts = BTreeSet::new();

        // Add value nodes and definition edges
        for (value, def_inst) in &self.defs {
            let vid = value.to_hex();
            if seen_values.insert(vid.clone()) {
                let mut properties = BTreeMap::new();
                if let Some(span) = id_spans.get(&value.raw()) {
                    properties.insert(
                        "span".to_string(),
                        span_to_property(span, &module.source_files),
                    );
                }
                if let Some(fname) = param_func_name.get(&value.raw()) {
                    properties.insert("parent_function".to_string(), serde_json::json!(fname));
                }
                insert_type_property(&mut properties, value.raw(), &value_types, module);
                let mut pg_node = PgNode {
                    id: vid.clone(),
                    labels: vec!["Value".to_string()],
                    properties,
                };
                enrich_node(&mut pg_node, resolver);
                pg.nodes.push(pg_node);
            }
            if let Some(inst) = def_inst {
                let iid = inst.to_hex();
                if seen_insts.insert(iid.clone()) {
                    let mut properties = BTreeMap::new();
                    if let Some(span) = id_spans.get(&inst.raw()) {
                        properties.insert(
                            "span".to_string(),
                            span_to_property(span, &module.source_files),
                        );
                    }
                    let mut pg_node = PgNode {
                        id: iid.clone(),
                        labels: vec!["Instruction".to_string()],
                        properties,
                    };
                    enrich_node(&mut pg_node, resolver);
                    pg.nodes.push(pg_node);
                }
                pg.edges.push(PgEdge {
                    src: iid,
                    dst: vid,
                    edge_type: "DEFINES".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        // Add use edges
        for (value, users) in &self.uses {
            let vid = value.to_hex();
            if seen_values.insert(vid.clone()) {
                let mut properties = BTreeMap::new();
                if let Some(span) = id_spans.get(&value.raw()) {
                    properties.insert(
                        "span".to_string(),
                        span_to_property(span, &module.source_files),
                    );
                }
                if let Some(fname) = param_func_name.get(&value.raw()) {
                    properties.insert("parent_function".to_string(), serde_json::json!(fname));
                }
                insert_type_property(&mut properties, value.raw(), &value_types, module);
                let mut pg_node = PgNode {
                    id: vid.clone(),
                    labels: vec!["Value".to_string()],
                    properties,
                };
                enrich_node(&mut pg_node, resolver);
                pg.nodes.push(pg_node);
            }
            for user in users {
                let iid = user.to_hex();
                if seen_insts.insert(iid.clone()) {
                    let mut properties = BTreeMap::new();
                    if let Some(span) = id_spans.get(&user.raw()) {
                        properties.insert(
                            "span".to_string(),
                            span_to_property(span, &module.source_files),
                        );
                    }
                    let mut pg_node = PgNode {
                        id: iid.clone(),
                        labels: vec!["Instruction".to_string()],
                        properties,
                    };
                    enrich_node(&mut pg_node, resolver);
                    pg.nodes.push(pg_node);
                }
                pg.edges.push(PgEdge {
                    src: vid.clone(),
                    dst: iid,
                    edge_type: "USED_BY".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        pg.metadata.insert(
            "definition_count".to_string(),
            serde_json::json!(self.defs.len()),
        );
        pg.metadata
            .insert("use_count".to_string(), serde_json::json!(self.uses.len()));
        pg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirParam, Instruction};
    use saf_core::ids::{BlockId, FunctionId, ModuleId};

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

    fn make_function_with_params(
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
    fn defuse_parameters() {
        // Function with two parameters
        let params = vec![
            AirParam::new(ValueId::new(1), 0),
            AirParam::new(ValueId::new(2), 1),
        ];
        let func = make_function_with_params(
            1,
            "add",
            params,
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![Instruction::new(InstId::new(100), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        // Parameters are defined but have no defining instruction
        assert!(du.is_parameter(ValueId::new(1)));
        assert!(du.is_parameter(ValueId::new(2)));
        assert_eq!(du.def_of(ValueId::new(1)), Some(None));
        assert_eq!(du.def_of(ValueId::new(2)), Some(None));
    }

    #[test]
    fn defuse_instruction_result() {
        // Instruction with a destination
        let inst = Instruction::new(InstId::new(100), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));

        let func = make_function_with_params(
            1,
            "test",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![inst, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        // Value 2 is defined by instruction 100
        assert_eq!(du.def_of(ValueId::new(2)), Some(Some(InstId::new(100))));
        assert!(!du.is_parameter(ValueId::new(2)));

        // Value 1 (param) is used by instruction 100
        let users = du.users_of(ValueId::new(1)).unwrap();
        assert!(users.contains(&InstId::new(100)));
    }

    #[test]
    fn defuse_operand_uses() {
        // Binary op using two operands
        let add = Instruction::new(
            InstId::new(100),
            Operation::BinaryOp {
                kind: saf_core::air::BinaryOp::Add,
            },
        )
        .with_operands(vec![ValueId::new(1), ValueId::new(2)])
        .with_dst(ValueId::new(3));

        let func = make_function_with_params(
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
        let du = DefUseGraph::build(&module);

        // Both operands are used by the add instruction
        assert!(
            du.users_of(ValueId::new(1))
                .unwrap()
                .contains(&InstId::new(100))
        );
        assert!(
            du.users_of(ValueId::new(2))
                .unwrap()
                .contains(&InstId::new(100))
        );
    }

    #[test]
    fn defuse_phi_incoming() {
        // Phi node with incoming values
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

        let func = make_function_with_params(
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
        let du = DefUseGraph::build(&module);

        // Phi incoming values count as uses
        assert!(
            du.users_of(ValueId::new(1))
                .unwrap()
                .contains(&InstId::new(100))
        );
        assert!(
            du.users_of(ValueId::new(2))
                .unwrap()
                .contains(&InstId::new(100))
        );
        // Phi result is defined
        assert_eq!(du.def_of(ValueId::new(3)), Some(Some(InstId::new(100))));
    }

    #[test]
    fn defuse_multiple_uses() {
        // Value used by multiple instructions
        let load1 = Instruction::new(InstId::new(100), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));
        let load2 = Instruction::new(InstId::new(101), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(3));

        let func = make_function_with_params(
            1,
            "test",
            vec![AirParam::new(ValueId::new(1), 0)],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    load1,
                    load2,
                    Instruction::new(InstId::new(102), Operation::Ret),
                ],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        // Value 1 is used by both load instructions
        let users = du.users_of(ValueId::new(1)).unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.contains(&InstId::new(100)));
        assert!(users.contains(&InstId::new(101)));
    }

    #[test]
    fn defuse_dead_value() {
        // Value defined but never used
        let alloca = Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
            .with_dst(ValueId::new(1));

        let func = make_function_with_params(
            1,
            "test",
            vec![],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![alloca, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        assert!(du.is_dead(ValueId::new(1)));
        assert_eq!(du.use_count(ValueId::new(1)), 0);
    }

    #[test]
    fn defuse_use_count() {
        // Test use counting
        let param = AirParam::new(ValueId::new(1), 0);
        let load1 = Instruction::new(InstId::new(100), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));
        let load2 = Instruction::new(InstId::new(101), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(3));
        let load3 = Instruction::new(InstId::new(102), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(4));

        let func = make_function_with_params(
            1,
            "test",
            vec![param],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    load1,
                    load2,
                    load3,
                    Instruction::new(InstId::new(103), Operation::Ret),
                ],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        assert_eq!(du.use_count(ValueId::new(1)), 3);
    }

    #[test]
    fn defuse_export_is_deterministic() {
        let param = AirParam::new(ValueId::new(1), 0);
        let load = Instruction::new(InstId::new(100), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));

        let func = make_function_with_params(
            1,
            "test",
            vec![param],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![load, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        let export1 = serde_json::to_string(&du.export()).unwrap();
        let export2 = serde_json::to_string(&du.export()).unwrap();
        assert_eq!(export1, export2);
    }

    #[test]
    fn defuse_values() {
        let param = AirParam::new(ValueId::new(1), 0);
        let load = Instruction::new(InstId::new(100), Operation::Load)
            .with_operands(vec![ValueId::new(1)])
            .with_dst(ValueId::new(2));

        let func = make_function_with_params(
            1,
            "test",
            vec![param],
            vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![load, Instruction::new(InstId::new(101), Operation::Ret)],
            }],
        );
        let module = make_module(vec![func]);
        let du = DefUseGraph::build(&module);

        let values = du.values();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&ValueId::new(1)));
        assert!(values.contains(&ValueId::new(2)));
    }
}
