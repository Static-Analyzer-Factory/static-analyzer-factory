//! SVFG JSON export.

use serde::{Deserialize, Serialize};

use super::{Svfg, SvfgDiagnostics, SvfgEdgeKind, SvfgNodeId};

/// Schema version for SVFG export format.
pub const SVFG_EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// JSON-serializable SVFG export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvfgExport {
    /// Schema version.
    pub schema_version: String,
    /// Total number of nodes.
    pub node_count: usize,
    /// Total number of edges.
    pub edge_count: usize,
    /// Construction diagnostics.
    pub diagnostics: SvfgDiagnostics,
    /// All nodes.
    pub nodes: Vec<SvfgNodeExport>,
    /// All edges.
    pub edges: Vec<SvfgEdgeExport>,
}

/// A single exported node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvfgNodeExport {
    /// Node kind: `"value"` or `"mem_phi"`.
    pub kind: String,
    /// Node ID as hex string.
    pub id: String,
}

/// A single exported edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvfgEdgeExport {
    /// Source node ID as hex string.
    pub src: String,
    /// Destination node ID as hex string.
    pub dst: String,
    /// Edge kind.
    pub kind: SvfgEdgeKind,
}

/// Export an SVFG to JSON-serializable format.
pub(crate) fn export_svfg(svfg: &Svfg) -> SvfgExport {
    // Nodes are already sorted by BTreeSet iteration order
    let nodes: Vec<SvfgNodeExport> = svfg
        .nodes()
        .iter()
        .map(|node| SvfgNodeExport {
            kind: match node {
                SvfgNodeId::Value(_) => "value".to_string(),
                SvfgNodeId::MemPhi(_) => "mem_phi".to_string(),
            },
            id: node.to_hex(),
        })
        .collect();

    // Edges are already deterministically ordered by BTreeMap/BTreeSet iteration
    let mut edges: Vec<SvfgEdgeExport> = Vec::new();
    for node in svfg.nodes() {
        if let Some(succs) = svfg.successors_of(*node) {
            for (kind, to) in succs {
                edges.push(SvfgEdgeExport {
                    src: node.to_hex(),
                    dst: to.to_hex(),
                    kind: *kind,
                });
            }
        }
    }

    SvfgExport {
        schema_version: SVFG_EXPORT_SCHEMA_VERSION.to_string(),
        node_count: svfg.node_count(),
        edge_count: svfg.edge_count(),
        diagnostics: svfg.diagnostics().clone(),
        nodes,
        edges,
    }
}

/// Export an SVFG as a [`PropertyGraph`](crate::export::PropertyGraph).
///
/// Nodes carry labels `["Value"]` or `["MemPhi"]` and a `kind` property.
/// Edges use the uppercase `SvfgEdgeKind` debug name as `edge_type`
/// (e.g. `DIRECT_DEF`, `INDIRECT_STORE`).
pub(crate) fn to_property_graph(
    svfg: &Svfg,
    resolver: Option<&crate::display::DisplayResolver<'_>>,
) -> crate::export::PropertyGraph {
    use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
    use std::collections::BTreeMap;

    let mut pg = PropertyGraph::new("svfg");
    pg.metadata.insert(
        "node_count".to_string(),
        serde_json::json!(svfg.node_count()),
    );
    pg.metadata.insert(
        "edge_count".to_string(),
        serde_json::json!(svfg.edge_count()),
    );

    for node in svfg.nodes() {
        let (labels, kind) = match node {
            SvfgNodeId::Value(_) => (vec!["Value".to_string()], "value"),
            SvfgNodeId::MemPhi(_) => (vec!["MemPhi".to_string()], "mem_phi"),
        };

        let mut properties = BTreeMap::new();
        properties.insert("kind".to_string(), serde_json::json!(kind));

        let mut pg_node = PgNode {
            id: node.to_hex(),
            labels,
            properties,
        };
        enrich_node(&mut pg_node, resolver);
        pg.nodes.push(pg_node);
    }

    for node in svfg.nodes() {
        if let Some(succs) = svfg.successors_of(*node) {
            for (kind, to) in succs {
                let edge_type = kind.name().to_uppercase();

                pg.edges.push(PgEdge {
                    src: node.to_hex(),
                    dst: to.to_hex(),
                    edge_type,
                    properties: BTreeMap::new(),
                });
            }
        }
    }

    pg
}

#[cfg(test)]
mod tests {
    use saf_core::ids::ValueId;

    use super::*;
    use crate::mssa::MemAccessId;

    #[test]
    fn export_empty_graph() {
        let graph = Svfg::new();
        let export = graph.export();

        assert_eq!(export.schema_version, "0.1.0");
        assert_eq!(export.node_count, 0);
        assert_eq!(export.edge_count, 0);
        assert!(export.nodes.is_empty());
        assert!(export.edges.is_empty());
    }

    #[test]
    fn export_with_nodes_and_edges() {
        let mut graph = Svfg::new();
        let v1 = SvfgNodeId::value(ValueId::new(1));
        let phi = SvfgNodeId::mem_phi(MemAccessId::new(100));
        let v2 = SvfgNodeId::value(ValueId::new(2));

        graph.add_edge(v1, SvfgEdgeKind::IndirectStore, phi);
        graph.add_edge(phi, SvfgEdgeKind::IndirectLoad, v2);

        let export = graph.export();
        assert_eq!(export.node_count, 3);
        assert_eq!(export.edge_count, 2);
        assert_eq!(export.nodes.len(), 3);
        assert_eq!(export.edges.len(), 2);
    }

    #[test]
    fn export_json_roundtrip() {
        let mut graph = Svfg::new();
        graph.add_edge(
            SvfgNodeId::value(ValueId::new(1)),
            SvfgEdgeKind::DirectDef,
            SvfgNodeId::value(ValueId::new(2)),
        );

        let export = graph.export();
        let json = serde_json::to_string(&export).unwrap();
        let parsed: SvfgExport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.schema_version, export.schema_version);
        assert_eq!(parsed.node_count, export.node_count);
        assert_eq!(parsed.edge_count, export.edge_count);
    }

    #[test]
    fn export_deterministic() {
        let mut g1 = Svfg::new();
        let mut g2 = Svfg::new();

        let v1 = SvfgNodeId::value(ValueId::new(1));
        let v2 = SvfgNodeId::value(ValueId::new(2));
        let v3 = SvfgNodeId::value(ValueId::new(3));

        // Add in different order
        g1.add_edge(v1, SvfgEdgeKind::DirectDef, v2);
        g1.add_edge(v2, SvfgEdgeKind::DirectTransform, v3);

        g2.add_edge(v2, SvfgEdgeKind::DirectTransform, v3);
        g2.add_edge(v1, SvfgEdgeKind::DirectDef, v2);

        let json1 = serde_json::to_string(&g1.export()).unwrap();
        let json2 = serde_json::to_string(&g2.export()).unwrap();
        assert_eq!(json1, json2);
    }
}
