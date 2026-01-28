//! JSON export for context-sensitive PTA results.

use serde::Serialize;

use super::solver::{CsPtaConfig, CsPtaResult};

/// Export schema version.
pub const CS_EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// Exported CS-PTA result.
#[derive(Debug, Serialize)]
pub struct CsPtaExport {
    /// Schema version.
    pub schema_version: String,
    /// Configuration.
    pub config: CsPtaConfigExport,
    /// Diagnostics.
    pub diagnostics: CsPtaDiagnosticsExport,
    /// Context-sensitive points-to entries.
    pub contexts: Vec<CsPtsEntryExport>,
    /// CI summary.
    pub ci_summary: CiSummaryExport,
}

/// Exported configuration.
#[derive(Debug, Serialize)]
pub struct CsPtaConfigExport {
    /// Context depth k.
    pub k: u32,
    /// Field sensitivity.
    pub field_sensitivity: String,
    /// Max iterations.
    pub max_iterations: usize,
    /// Max objects.
    pub max_objects: usize,
}

/// Exported diagnostics.
#[derive(Debug, Serialize)]
pub struct CsPtaDiagnosticsExport {
    /// Iterations used.
    pub iterations: usize,
    /// Whether limit was hit.
    pub iteration_limit_hit: bool,
    /// Unique context count.
    pub context_count: usize,
    /// Max points-to set size.
    pub max_pts_size: usize,
    /// SCC function count.
    pub scc_function_count: usize,
    /// Total constraints.
    pub constraint_count: usize,
    /// Total locations.
    pub location_count: usize,
    /// Heap-cloned objects.
    pub heap_clone_count: usize,
}

/// A single context-sensitive points-to entry.
#[derive(Debug, Serialize)]
pub struct CsPtsEntryExport {
    /// Value ID as hex.
    pub value: String,
    /// Context (list of call-site hex IDs).
    pub context: Vec<String>,
    /// Points-to locations (hex).
    pub points_to: Vec<String>,
}

/// CI summary export.
#[derive(Debug, Serialize)]
pub struct CiSummaryExport {
    /// Points-to entries (union across contexts).
    pub points_to: Vec<CiPtsEntryExport>,
}

/// CI summary entry.
#[derive(Debug, Serialize)]
pub struct CiPtsEntryExport {
    /// Value ID as hex.
    pub value: String,
    /// Location IDs (hex).
    pub locations: Vec<String>,
}

impl CsPtaResult {
    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Values become `["Pointer"]` nodes with a `context` property (the
    /// call-site chain as a hex list).  Locations become `["Location"]`
    /// nodes.  Edges are `POINTS_TO`.
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
        use std::collections::{BTreeMap, BTreeSet};

        let mut pg = PropertyGraph::new("cspta");

        let mut seen_locs: BTreeSet<saf_core::ids::LocId> = BTreeSet::new();

        for (cv, locs) in self.cs_points_to_map() {
            if locs.is_empty() {
                continue;
            }

            // Build a unique node id that combines value + context
            let ctx_suffix: String = cv.ctx.to_hex_vec().join(",");
            let node_id = if ctx_suffix.is_empty() {
                cv.value.to_hex()
            } else {
                format!("{}@{}", cv.value.to_hex(), ctx_suffix)
            };

            let mut properties = BTreeMap::new();
            let ctx_vec = cv.ctx.to_hex_vec();
            if !ctx_vec.is_empty() {
                properties.insert("context".to_string(), serde_json::json!(ctx_vec));
            }
            properties.insert("value".to_string(), serde_json::json!(cv.value.to_hex()));

            let mut pg_node = PgNode {
                id: node_id.clone(),
                labels: vec!["Pointer".to_string()],
                properties,
            };
            enrich_node(&mut pg_node, resolver);
            pg.nodes.push(pg_node);

            for loc_id in locs {
                seen_locs.insert(*loc_id);

                pg.edges.push(PgEdge {
                    src: node_id.clone(),
                    dst: loc_id.to_hex(),
                    edge_type: "POINTS_TO".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        // Location nodes
        for loc_id in &seen_locs {
            let mut loc_node = PgNode {
                id: loc_id.to_hex(),
                labels: vec!["Location".to_string()],
                properties: BTreeMap::new(),
            };
            enrich_node(&mut loc_node, resolver);
            pg.nodes.push(loc_node);
        }

        pg
    }

    /// Export the result to a serializable format.
    #[must_use]
    pub fn export(&self, config: &CsPtaConfig) -> CsPtaExport {
        let field_sensitivity = match &config.field_sensitivity {
            crate::pta::FieldSensitivity::None => "none".to_string(),
            crate::pta::FieldSensitivity::StructFields { max_depth } => {
                format!("struct_fields(max_depth={max_depth})")
            }
        };

        let config_export = CsPtaConfigExport {
            k: config.k,
            field_sensitivity,
            max_iterations: config.max_iterations,
            max_objects: config.max_objects,
        };

        let diag = self.diagnostics();
        let diagnostics_export = CsPtaDiagnosticsExport {
            iterations: diag.iterations,
            iteration_limit_hit: diag.iteration_limit_hit,
            context_count: diag.context_count,
            max_pts_size: diag.max_pts_size,
            scc_function_count: diag.scc_function_count,
            constraint_count: diag.constraint_count,
            location_count: diag.location_count,
            heap_clone_count: diag.heap_clone_count,
        };

        let contexts: Vec<CsPtsEntryExport> = self
            .cs_points_to_map()
            .iter()
            .filter(|(_, locs)| !locs.is_empty())
            .map(|(cv, locs)| CsPtsEntryExport {
                value: cv.value.to_hex(),
                context: cv.ctx.to_hex_vec(),
                points_to: locs.iter().map(|l| l.to_hex()).collect(),
            })
            .collect();

        let ci_pts: Vec<CiPtsEntryExport> = self
            .ci_summary_map()
            .iter()
            .filter(|(_, locs)| !locs.is_empty())
            .map(|(v, locs)| CiPtsEntryExport {
                value: v.to_hex(),
                locations: locs.iter().map(|l| l.to_hex()).collect(),
            })
            .collect();

        CsPtaExport {
            schema_version: CS_EXPORT_SCHEMA_VERSION.to_string(),
            config: config_export,
            diagnostics: diagnostics_export,
            contexts,
            ci_summary: CiSummaryExport { points_to: ci_pts },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::callgraph::CallGraph;
    use crate::cspta::solve_context_sensitive;
    use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    #[test]
    fn export_empty_result() {
        let module = saf_core::air::AirModule::new(ModuleId::derive(b"test"));
        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);
        let export = result.export(&config);

        assert_eq!(export.schema_version, CS_EXPORT_SCHEMA_VERSION);
        assert!(export.contexts.is_empty());
        assert!(export.ci_summary.points_to.is_empty());
    }

    #[test]
    fn export_with_data() {
        let mut module = saf_core::air::AirModule::new(ModuleId::derive(b"test"));
        let mut func = AirFunction::new(FunctionId::derive(b"f"), "f");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(InstId::derive(b"a"), Operation::Alloca { size_bytes: None })
                .with_dst(ValueId::derive(b"p")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();
        let result = solve_context_sensitive(&module, &callgraph, &config);
        let export = result.export(&config);

        assert!(!export.contexts.is_empty(), "should have CS entries");
        assert!(
            !export.ci_summary.points_to.is_empty(),
            "should have CI summary"
        );
    }

    #[test]
    fn export_deterministic() {
        let mut module = saf_core::air::AirModule::new(ModuleId::derive(b"test"));
        let mut func = AirFunction::new(FunctionId::derive(b"f"), "f");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"a1"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ValueId::derive(b"p1")),
        );
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"a2"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ValueId::derive(b"p2")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let callgraph = CallGraph::build(&module);
        let config = CsPtaConfig::default();

        let r1 = solve_context_sensitive(&module, &callgraph, &config);
        let r2 = solve_context_sensitive(&module, &callgraph, &config);

        let json1 = serde_json::to_string(&r1.export(&config)).unwrap();
        let json2 = serde_json::to_string(&r2.export(&config)).unwrap();
        assert_eq!(json1, json2, "exports should be deterministic");
    }
}
