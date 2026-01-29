//! IFDS result export types.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use saf_core::ids::{FunctionId, InstId};

use super::result::{IfdsDiagnostics, IfdsResult};

/// Exportable IFDS result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfdsExport {
    /// Facts per instruction: `inst_hex` maps to a list of fact debug strings.
    pub facts: BTreeMap<String, Vec<String>>,
    /// Summary edges per function.
    pub summaries: Vec<IfdsSummaryExport>,
    /// Solver diagnostics.
    pub diagnostics: IfdsDiagnosticsExport,
}

/// Exportable summary edges for a single function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfdsSummaryExport {
    /// Function ID as hex string.
    pub function: String,
    /// `(entry_fact, exit_fact)` pairs as debug strings.
    pub edges: Vec<(String, String)>,
}

/// Exportable solver diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfdsDiagnosticsExport {
    /// Total worklist iterations performed.
    pub iterations: usize,
    /// Total path edges explored.
    pub path_edges_explored: usize,
    /// Total summary edges created.
    pub summary_edges_created: usize,
    /// Peak number of facts at any single program point.
    pub facts_at_peak: usize,
    /// Whether the solver hit a configured limit.
    pub reached_limit: bool,
}

impl From<&IfdsDiagnostics> for IfdsDiagnosticsExport {
    fn from(d: &IfdsDiagnostics) -> Self {
        Self {
            iterations: d.iterations,
            path_edges_explored: d.path_edges_explored,
            summary_edges_created: d.summary_edges_created,
            facts_at_peak: d.facts_at_peak,
            reached_limit: d.reached_limit,
        }
    }
}

impl<F: Ord + Clone + std::fmt::Debug> IfdsResult<F> {
    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Instructions become nodes with labels `["Instruction"]`.  Each fact
    /// that holds at an instruction becomes a `["Fact"]` node connected by
    /// a `HOLDS_AT` edge.
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
        use std::collections::BTreeMap;

        let mut pg = PropertyGraph::new("ifds");

        let mut fact_counter: u64 = 0;

        for (inst_id, fact_set) in &self.facts {
            // Instruction node
            let mut inst_node = PgNode {
                id: inst_id.to_hex(),
                labels: vec!["Instruction".to_string()],
                properties: BTreeMap::new(),
            };
            enrich_node(&mut inst_node, resolver);
            pg.nodes.push(inst_node);

            for fact in fact_set {
                let fact_debug = format!("{fact:?}");
                let fact_id = format!("fact_{fact_counter}");
                fact_counter += 1;

                let mut properties = BTreeMap::new();
                properties.insert("value".to_string(), serde_json::json!(fact_debug));

                pg.nodes.push(PgNode {
                    id: fact_id.clone(),
                    labels: vec!["Fact".to_string()],
                    properties,
                });

                pg.edges.push(PgEdge {
                    src: inst_id.to_hex(),
                    dst: fact_id,
                    edge_type: "HOLDS_AT".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        pg
    }

    /// Export the IFDS result to a serializable format.
    #[must_use]
    pub fn export(&self) -> IfdsExport {
        let facts: BTreeMap<String, Vec<String>> = self
            .facts
            .iter()
            .map(|(inst_id, fact_set)| {
                let key = inst_id.to_hex();
                let values: Vec<String> = fact_set.iter().map(|f| format!("{f:?}")).collect();
                (key, values)
            })
            .collect();

        let summaries: Vec<IfdsSummaryExport> = self
            .summaries
            .iter()
            .map(|(func_id, edges)| {
                let function = func_id.to_hex();
                let edge_strs: Vec<(String, String)> = edges
                    .iter()
                    .map(|(entry, exit)| (format!("{entry:?}"), format!("{exit:?}")))
                    .collect();
                IfdsSummaryExport {
                    function,
                    edges: edge_strs,
                }
            })
            .collect();

        IfdsExport {
            facts,
            summaries,
            diagnostics: IfdsExport::diagnostics_from(&self.diagnostics),
        }
    }
}

impl IfdsExport {
    fn diagnostics_from(d: &IfdsDiagnostics) -> IfdsDiagnosticsExport {
        IfdsDiagnosticsExport::from(d)
    }
}

/// Helper to convert an `InstId` to its hex representation.
/// Used internally — callers should use `IfdsResult::export()`.
#[must_use]
pub fn inst_id_hex(id: InstId) -> String {
    id.to_hex()
}

/// Helper to convert a `FunctionId` to its hex representation.
#[must_use]
pub fn func_id_hex(id: FunctionId) -> String {
    id.to_hex()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn export_produces_valid_json() {
        let mut facts = BTreeMap::new();
        let mut set = BTreeSet::new();
        set.insert(42_u64);
        facts.insert(InstId::new(1), set);

        let result: IfdsResult<u64> = IfdsResult {
            facts,
            summaries: BTreeMap::new(),
            diagnostics: IfdsDiagnostics::default(),
        };

        let export = result.export();
        let json = serde_json::to_string(&export).expect("export should serialize to JSON");
        assert!(json.contains("facts"));
        assert!(json.contains("summaries"));
        assert!(json.contains("diagnostics"));
    }

    #[test]
    fn export_is_deterministic() {
        let mut facts = BTreeMap::new();
        let mut set = BTreeSet::new();
        set.insert(1_u64);
        set.insert(2);
        set.insert(3);
        facts.insert(InstId::new(1), set);

        let result: IfdsResult<u64> = IfdsResult {
            facts,
            summaries: BTreeMap::new(),
            diagnostics: IfdsDiagnostics::default(),
        };

        let json1 = serde_json::to_string(&result.export()).unwrap();
        let json2 = serde_json::to_string(&result.export()).unwrap();
        assert_eq!(json1, json2);
    }
}
