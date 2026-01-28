//! JSON export for flow-sensitive PTA results.

use serde::{Deserialize, Serialize};

use saf_core::id::id_to_hex;

use super::{FlowSensitivePtaResult, FsPtaDiagnostics};

/// Schema version for flow-sensitive PTA export.
pub const FS_PTA_EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// JSON-serializable flow-sensitive PTA export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsPtaExport {
    /// Schema version.
    pub schema_version: String,
    /// Analysis diagnostics.
    pub diagnostics: FsPtaDiagnostics,
    /// Number of top-level pointers with points-to sets.
    pub top_level_pointer_count: usize,
    /// Number of SVFG nodes with `dfIn` data.
    pub df_in_node_count: usize,
    /// Number of SVFG nodes with `dfOut` data.
    pub df_out_node_count: usize,
    /// Top-level pointer points-to summary (value_hex → list of loc_hex).
    pub points_to: Vec<PtsEntry>,
}

/// A single top-level pointer's points-to set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtsEntry {
    /// Value ID as hex string.
    pub value: String,
    /// Points-to location IDs as hex strings.
    pub locations: Vec<String>,
}

/// Export a `FlowSensitivePtaResult` to JSON-serializable format.
pub(crate) fn export_fs_pta(result: &FlowSensitivePtaResult) -> FsPtaExport {
    let mut points_to: Vec<PtsEntry> = result
        .points_to_map()
        .iter()
        .filter(|(_, locs)| !locs.is_empty())
        .map(|(vid, locs)| PtsEntry {
            value: id_to_hex(vid.raw()),
            locations: locs.iter().map(|l| id_to_hex(l.raw())).collect(),
        })
        .collect();

    // Sort for determinism (BTreeMap already ordered, but be explicit)
    points_to.sort_by(|a, b| a.value.cmp(&b.value));

    FsPtaExport {
        schema_version: FS_PTA_EXPORT_SCHEMA_VERSION.to_string(),
        diagnostics: result.diagnostics().clone(),
        top_level_pointer_count: result.points_to_map().len(),
        df_in_node_count: result.df_in().len(),
        df_out_node_count: result.df_out().len(),
        points_to,
    }
}

/// Export a `FlowSensitivePtaResult` as a [`PropertyGraph`](crate::export::PropertyGraph).
///
/// Values become `["Pointer"]` nodes, locations become `["Location"]` nodes,
/// and edges are `POINTS_TO`.
pub(crate) fn to_property_graph(
    result: &FlowSensitivePtaResult,
    resolver: Option<&crate::display::DisplayResolver<'_>>,
) -> crate::export::PropertyGraph {
    use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
    use std::collections::{BTreeMap, BTreeSet};

    let mut pg = PropertyGraph::new("fspta");

    let mut seen_locs: BTreeSet<saf_core::ids::LocId> = BTreeSet::new();

    for (vid, locs) in result.points_to_map() {
        if locs.is_empty() {
            continue;
        }

        let mut ptr_node = PgNode {
            id: id_to_hex(vid.raw()),
            labels: vec!["Pointer".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut ptr_node, resolver);
        pg.nodes.push(ptr_node);

        for loc_id in locs {
            seen_locs.insert(*loc_id);

            pg.edges.push(PgEdge {
                src: id_to_hex(vid.raw()),
                dst: id_to_hex(loc_id.raw()),
                edge_type: "POINTS_TO".to_string(),
                properties: BTreeMap::new(),
            });
        }
    }

    for loc_id in &seen_locs {
        let mut loc_node = PgNode {
            id: id_to_hex(loc_id.raw()),
            labels: vec!["Location".to_string()],
            properties: BTreeMap::new(),
        };
        enrich_node(&mut loc_node, resolver);
        pg.nodes.push(loc_node);
    }

    pg
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use saf_core::ids::{LocId, ValueId};

    use super::*;
    use crate::fspta::FsPtaDiagnostics;
    use crate::svfg::SvfgNodeId;

    #[test]
    fn export_empty_result() {
        let result = FlowSensitivePtaResult {
            pts: BTreeMap::new(),
            df_in: BTreeMap::new(),
            df_out: BTreeMap::new(),
            diagnostics: FsPtaDiagnostics::default(),
        };

        let export = result.export();
        assert_eq!(export.schema_version, "0.1.0");
        assert_eq!(export.top_level_pointer_count, 0);
        assert!(export.points_to.is_empty());
    }

    #[test]
    fn export_with_pts() {
        let mut pts = BTreeMap::new();
        let mut set = BTreeSet::new();
        set.insert(LocId::new(0));
        set.insert(LocId::new(1));
        pts.insert(ValueId::new(42), set);

        let result = FlowSensitivePtaResult {
            pts,
            df_in: BTreeMap::new(),
            df_out: BTreeMap::new(),
            diagnostics: FsPtaDiagnostics::default(),
        };

        let export = result.export();
        assert_eq!(export.top_level_pointer_count, 1);
        assert_eq!(export.points_to.len(), 1);
        assert_eq!(export.points_to[0].locations.len(), 2);
    }

    #[test]
    fn export_json_roundtrip() {
        let mut pts = BTreeMap::new();
        let mut set = BTreeSet::new();
        set.insert(LocId::new(0));
        pts.insert(ValueId::new(1), set);

        let mut df_in = BTreeMap::new();
        let mut in_map = BTreeMap::new();
        let mut loc_pts = BTreeSet::new();
        loc_pts.insert(LocId::new(0));
        in_map.insert(LocId::new(0), loc_pts);
        df_in.insert(SvfgNodeId::Value(ValueId::new(1)), in_map);

        let result = FlowSensitivePtaResult {
            pts,
            df_in,
            df_out: BTreeMap::new(),
            diagnostics: FsPtaDiagnostics {
                iterations: 10,
                strong_updates: 3,
                weak_updates: 7,
                ..FsPtaDiagnostics::default()
            },
        };

        let export = result.export();
        let json = serde_json::to_string(&export).expect("serialize");
        let parsed: FsPtaExport = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.schema_version, "0.1.0");
        assert_eq!(parsed.diagnostics.iterations, 10);
        assert_eq!(parsed.diagnostics.strong_updates, 3);
        assert_eq!(parsed.top_level_pointer_count, 1);
    }

    #[test]
    fn export_is_deterministic() {
        let mut pts = BTreeMap::new();
        let mut set1 = BTreeSet::new();
        set1.insert(LocId::new(0));
        set1.insert(LocId::new(1));
        pts.insert(ValueId::new(1), set1);

        let mut set2 = BTreeSet::new();
        set2.insert(LocId::new(2));
        pts.insert(ValueId::new(2), set2);

        let result = FlowSensitivePtaResult {
            pts: pts.clone(),
            df_in: BTreeMap::new(),
            df_out: BTreeMap::new(),
            diagnostics: FsPtaDiagnostics::default(),
        };

        let json1 = serde_json::to_string(&result.export()).unwrap();
        let json2 = serde_json::to_string(&result.export()).unwrap();
        assert_eq!(json1, json2);
    }
}
