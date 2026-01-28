//! JSON export for abstract interpretation results.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::checker::NumericFinding;
use super::result::{AbstractInterpDiagnostics, AbstractInterpResult};

/// Exportable abstract interpretation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractInterpExport {
    /// Schema version.
    pub schema: String,
    /// Number of blocks with computed states.
    pub block_count: usize,
    /// Number of instruction states.
    pub inst_count: usize,
    /// Analysis diagnostics.
    pub diagnostics: AbstractInterpDiagnostics,
    /// Block invariants (block hex ID → value hex ID → interval string).
    pub block_invariants: BTreeMap<String, BTreeMap<String, String>>,
}

impl AbstractInterpResult {
    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Each block becomes a `["Block"]` node with invariant values stored as
    /// properties (one property per tracked value, keyed by the value hex ID,
    /// with the interval string as the value).  No edges are emitted since
    /// abstract interpretation results are block-level invariants.
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgNode, PropertyGraph, enrich_node};
        use std::collections::BTreeMap;

        let mut pg = PropertyGraph::new("absint");
        pg.metadata.insert(
            "block_count".to_string(),
            serde_json::json!(self.block_count()),
        );

        for (block_id, state) in self.block_states() {
            let mut properties = BTreeMap::new();

            for (val_id, interval) in state.entries() {
                properties.insert(val_id.to_hex(), serde_json::json!(format!("{interval}")));
            }

            let mut pg_node = PgNode {
                id: block_id.to_hex(),
                labels: vec!["Block".to_string()],
                properties,
            };
            enrich_node(&mut pg_node, resolver);
            pg.nodes.push(pg_node);
        }

        pg
    }

    /// Export the result to a serializable format.
    #[must_use]
    pub fn export(&self) -> AbstractInterpExport {
        let mut block_invariants = BTreeMap::new();

        for (block_id, state) in self.block_states() {
            let mut value_map = BTreeMap::new();
            for (val_id, interval) in state.entries() {
                value_map.insert(val_id.to_hex(), format!("{interval}"));
            }
            if !value_map.is_empty() {
                block_invariants.insert(block_id.to_hex(), value_map);
            }
        }

        AbstractInterpExport {
            schema: "absint-v0.1.0".to_string(),
            block_count: self.block_count(),
            inst_count: self.inst_count(),
            diagnostics: self.diagnostics(),
            block_invariants,
        }
    }
}

/// Exportable numeric findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericFindingsExport {
    /// Schema version.
    pub schema: String,
    /// Total findings.
    pub count: usize,
    /// The findings.
    pub findings: Vec<NumericFinding>,
}

/// Export numeric findings to a serializable format.
#[must_use]
#[allow(dead_code)]
pub fn export_findings(findings: &[NumericFinding]) -> NumericFindingsExport {
    NumericFindingsExport {
        schema: "numeric-findings-v0.1.0".to_string(),
        count: findings.len(),
        findings: findings.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_empty() {
        let export = export_findings(&[]);
        assert_eq!(export.count, 0);
        assert!(export.findings.is_empty());
        assert_eq!(export.schema, "numeric-findings-v0.1.0");
    }

    #[test]
    fn export_serializes() {
        let export = export_findings(&[]);
        let json = serde_json::to_string(&export).expect("serialize");
        assert!(json.contains("numeric-findings-v0.1.0"));
    }
}
