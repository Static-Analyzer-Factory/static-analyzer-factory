//! JSON export for Memory SSA.
//!
//! Provides a serializable representation of the Memory SSA for debugging,
//! tutorials, and interoperability.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use saf_core::ids::LocId;

use super::MemorySsa;
use super::access::MemoryAccess;

/// Export schema version.
pub const EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// Serializable Memory SSA export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySsaExport {
    /// Schema version.
    pub schema_version: String,
    /// Total number of memory accesses.
    pub access_count: usize,
    /// Memory accesses grouped by function.
    pub functions: BTreeMap<String, FunctionMssaExport>,
    /// Mod/ref summaries per function.
    pub mod_ref: BTreeMap<String, ModRefExport>,
}

/// Per-function Memory SSA export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMssaExport {
    /// LiveOnEntry sentinel ID.
    pub live_on_entry: String,
    /// Memory accesses in this function.
    pub accesses: Vec<AccessExport>,
    /// Phi nodes at block entries.
    pub phis: BTreeMap<String, Vec<String>>,
}

/// Serializable memory access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessExport {
    /// Access ID (hex).
    pub id: String,
    /// Access kind: "live_on_entry", "def", "use", "phi".
    pub kind: String,
    /// Instruction ID (hex), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inst: Option<String>,
    /// Block ID (hex), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    /// Defining access ID (hex), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defining: Option<String>,
    /// Phi operands: predecessor block → reaching def.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operands: Option<BTreeMap<String, String>>,
}

/// Serializable mod/ref summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRefExport {
    /// Locations this function may modify.
    pub may_mod: Vec<String>,
    /// Locations this function may read.
    pub may_ref: Vec<String>,
}

/// Export the Memory SSA to a serializable format.
pub fn export(mssa: &MemorySsa) -> MemorySsaExport {
    let mut functions: BTreeMap<String, FunctionMssaExport> = BTreeMap::new();

    // Group accesses by function via live_on_entry
    // For single-function case, export all accesses under that function
    if let Some((&func_id, &loe_id)) = mssa.live_on_entry.iter().next() {
        let func_hex = func_id.to_hex();
        let mut func_accesses = Vec::new();
        let mut func_phis: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // Export LiveOnEntry
        func_accesses.push(export_access(&mssa.accesses[&loe_id]));

        // Export all accesses (we include all — the function grouping is best-effort)
        for access in mssa.accesses.values() {
            if access.id() == loe_id {
                continue; // already added
            }
            // We include all non-LOE accesses. In practice, they belong to
            // the function that contains their block. For export simplicity,
            // we group them all under the function they relate to.
            func_accesses.push(export_access(access));
        }

        // Export block phis
        for (&block_id, phi_ids) in &mssa.block_phis {
            if !phi_ids.is_empty() {
                func_phis.insert(
                    block_id.to_hex(),
                    phi_ids.iter().map(|id| id.to_hex()).collect(),
                );
            }
        }

        functions.insert(
            func_hex,
            FunctionMssaExport {
                live_on_entry: loe_id.to_hex(),
                accesses: func_accesses,
                phis: func_phis,
            },
        );
    }

    // If there are multiple functions, export properly per-function
    if mssa.live_on_entry.len() > 1 {
        functions.clear();
        // Group accesses by function using block ownership
        let mut block_to_func: BTreeMap<super::access::MemAccessId, saf_core::ids::FunctionId> =
            BTreeMap::new();
        for (&func_id, &loe_id) in &mssa.live_on_entry {
            block_to_func.insert(loe_id, func_id);
        }

        // Build per-function export
        for (&func_id, &loe_id) in &mssa.live_on_entry {
            let func_hex = func_id.to_hex();
            let mut func_accesses = vec![export_access(&mssa.accesses[&loe_id])];
            let mut func_phis: BTreeMap<String, Vec<String>> = BTreeMap::new();

            // Add all Defs and Uses (all functions for now — multi-function
            // grouping uses the block_phis mapping)
            for access in mssa.accesses.values() {
                if access.id() == loe_id {
                    continue;
                }
                if access.is_live_on_entry() {
                    continue; // belongs to another function
                }
                func_accesses.push(export_access(access));
            }

            for (&block_id, phi_ids) in &mssa.block_phis {
                if !phi_ids.is_empty() {
                    func_phis.insert(
                        block_id.to_hex(),
                        phi_ids.iter().map(|id| id.to_hex()).collect(),
                    );
                }
            }

            functions.insert(
                func_hex,
                FunctionMssaExport {
                    live_on_entry: loe_id.to_hex(),
                    accesses: func_accesses,
                    phis: func_phis,
                },
            );
        }
    }

    // Export mod/ref summaries
    let mut mod_ref = BTreeMap::new();
    for (&func_id, summary) in &mssa.mod_ref_summaries {
        mod_ref.insert(
            func_id.to_hex(),
            ModRefExport {
                may_mod: summary.may_mod.iter().map(LocId::to_hex).collect(),
                may_ref: summary.may_ref.iter().map(LocId::to_hex).collect(),
            },
        );
    }

    MemorySsaExport {
        schema_version: EXPORT_SCHEMA_VERSION.to_string(),
        access_count: mssa.accesses.len(),
        functions,
        mod_ref,
    }
}

fn export_access(access: &MemoryAccess) -> AccessExport {
    match access {
        MemoryAccess::LiveOnEntry { id, .. } => AccessExport {
            id: id.to_hex(),
            kind: "live_on_entry".to_string(),
            inst: None,
            block: None,
            defining: None,
            operands: None,
        },
        MemoryAccess::Def {
            id,
            inst,
            block,
            defining,
        } => AccessExport {
            id: id.to_hex(),
            kind: "def".to_string(),
            inst: Some(inst.to_hex()),
            block: Some(block.to_hex()),
            defining: Some(defining.to_hex()),
            operands: None,
        },
        MemoryAccess::Use {
            id,
            inst,
            block,
            defining,
        } => AccessExport {
            id: id.to_hex(),
            kind: "use".to_string(),
            inst: Some(inst.to_hex()),
            block: Some(block.to_hex()),
            defining: Some(defining.to_hex()),
            operands: None,
        },
        MemoryAccess::Phi {
            id,
            block,
            operands,
        } => AccessExport {
            id: id.to_hex(),
            kind: "phi".to_string(),
            inst: None,
            block: Some(block.to_hex()),
            defining: None,
            operands: Some(
                operands
                    .iter()
                    .map(|(b, a)| (b.to_hex(), a.to_hex()))
                    .collect(),
            ),
        },
    }
}

/// Export a `MemorySsa` as a [`PropertyGraph`](crate::export::PropertyGraph).
///
/// Memory accesses become nodes with labels reflecting their kind:
/// - `["MemAccess", "Def"]` for stores/calls that modify memory.
/// - `["MemAccess", "Use"]` for loads.
/// - `["MemPhi"]` for phi merge points.
/// - `["LiveOnEntry"]` for the sentinel.
///
/// Edges:
/// - `DEF` from the defining access to each `Def`/`Use` that references it.
/// - `PHI_OP` from each phi operand access to its `Phi` node.
pub fn to_property_graph(
    mssa: &MemorySsa,
    resolver: Option<&crate::display::DisplayResolver<'_>>,
) -> crate::export::PropertyGraph {
    use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
    use std::collections::BTreeMap;

    let mut pg = PropertyGraph::new("mssa");
    pg.metadata.insert(
        "access_count".to_string(),
        serde_json::json!(mssa.accesses.len()),
    );

    for access in mssa.accesses.values() {
        let (labels, properties) = match access {
            MemoryAccess::LiveOnEntry { id: _, function } => {
                let mut props = BTreeMap::new();
                props.insert("function".to_string(), serde_json::json!(function.to_hex()));
                (vec!["LiveOnEntry".to_string()], props)
            }
            MemoryAccess::Def {
                id: _,
                inst,
                block,
                defining: _,
            } => {
                let mut props = BTreeMap::new();
                props.insert("inst".to_string(), serde_json::json!(inst.to_hex()));
                props.insert("block".to_string(), serde_json::json!(block.to_hex()));
                (vec!["MemAccess".to_string(), "Def".to_string()], props)
            }
            MemoryAccess::Use {
                id: _,
                inst,
                block,
                defining: _,
            } => {
                let mut props = BTreeMap::new();
                props.insert("inst".to_string(), serde_json::json!(inst.to_hex()));
                props.insert("block".to_string(), serde_json::json!(block.to_hex()));
                (vec!["MemAccess".to_string(), "Use".to_string()], props)
            }
            MemoryAccess::Phi {
                id: _,
                block,
                operands: _,
            } => {
                let mut props = BTreeMap::new();
                props.insert("block".to_string(), serde_json::json!(block.to_hex()));
                (vec!["MemPhi".to_string()], props)
            }
        };

        let mut pg_node = PgNode {
            id: access.id().to_hex(),
            labels,
            properties,
        };
        enrich_node(&mut pg_node, resolver);
        pg.nodes.push(pg_node);

        // DEF edges: defining access → this access
        match access {
            MemoryAccess::Def { defining, .. } | MemoryAccess::Use { defining, .. } => {
                pg.edges.push(PgEdge {
                    src: defining.to_hex(),
                    dst: access.id().to_hex(),
                    edge_type: "DEF".to_string(),
                    properties: BTreeMap::new(),
                });
            }
            MemoryAccess::Phi { operands, .. } => {
                for (pred_block, operand_id) in operands {
                    let mut props = BTreeMap::new();
                    props.insert(
                        "predecessor".to_string(),
                        serde_json::json!(pred_block.to_hex()),
                    );
                    pg.edges.push(PgEdge {
                        src: operand_id.to_hex(),
                        dst: access.id().to_hex(),
                        edge_type: "PHI_OP".to_string(),
                        properties: props,
                    });
                }
            }
            MemoryAccess::LiveOnEntry { .. } => {}
        }
    }

    pg
}
