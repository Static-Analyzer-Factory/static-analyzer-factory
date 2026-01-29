//! Export functionality for MTA analysis results.

use serde::{Deserialize, Serialize};

use super::MtaResult;

/// Schema version for MTA export format.
pub const EXPORT_SCHEMA_VERSION: &str = "1.0.0";

/// Exported MTA analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtaExport {
    /// Schema version.
    pub schema_version: String,

    /// All discovered threads.
    pub threads: Vec<ExportedThread>,

    /// Thread concurrency pairs.
    pub concurrency: Vec<ConcurrencyPair>,

    /// MHP information per program point.
    pub mhp_points: Vec<MhpPoint>,
}

/// Exported thread information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedThread {
    /// Thread ID (numeric).
    pub id: u32,

    /// Context string (e.g., "cs1.Call,cs2.foo").
    pub context: String,

    /// Entry function name.
    pub entry_function: String,

    /// Parent thread ID (None for main).
    pub parent: Option<u32>,

    /// Creation site instruction ID (hex).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_site: Option<String>,
}

/// A pair of threads that may run concurrently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyPair {
    /// First thread ID.
    pub thread_a: u32,

    /// Second thread ID.
    pub thread_b: u32,

    /// Whether they may run concurrently.
    pub may_run_concurrently: bool,
}

/// MHP information at a program point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MhpPoint {
    /// Thread executing at this point.
    pub thread_id: u32,

    /// Instruction ID (hex).
    pub inst_id: String,

    /// Thread IDs that may be concurrent here.
    pub concurrent_threads: Vec<u32>,
}

impl MtaExport {
    /// Create export from MTA result.
    pub fn from_result(result: &MtaResult) -> Self {
        let threads: Vec<ExportedThread> = result
            .thread_graph
            .threads
            .iter()
            .map(|(id, ctx)| ExportedThread {
                id: id.0,
                context: ctx.to_context_string(),
                entry_function: ctx.entry_function_name.clone(),
                parent: ctx.parent.map(|p| p.0),
                creation_site: ctx.creation_site.map(|s| format!("{s:?}")),
            })
            .collect();

        // Build concurrency pairs
        let mut concurrency = Vec::new();
        let thread_ids: Vec<_> = result.thread_graph.threads.keys().collect();
        for (i, &t1) in thread_ids.iter().enumerate() {
            for &t2 in thread_ids.iter().skip(i + 1) {
                concurrency.push(ConcurrencyPair {
                    thread_a: t1.0,
                    thread_b: t2.0,
                    may_run_concurrently: result.thread_graph.may_run_concurrently(t1, t2),
                });
            }
        }

        // Build MHP points
        let mhp_points: Vec<MhpPoint> = result
            .mhp_result
            .program_points()
            .map(|(thread_id, inst_id)| {
                let concurrent = result.mhp_result.concurrent_at(thread_id, inst_id);
                MhpPoint {
                    thread_id: thread_id.0,
                    inst_id: format!("{inst_id:?}"),
                    concurrent_threads: concurrent.iter().map(|t| t.0).collect(),
                }
            })
            .collect();

        Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            threads,
            concurrency,
            mhp_points,
        }
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Each thread becomes a node with label `"Thread"` and properties for
    /// entry function and context. Concurrency pairs with
    /// `may_run_concurrently = true` become "MAY_PARALLEL" edges.
    /// Parent-child relationships become "SPAWNS" edges.
    #[must_use]
    pub fn to_pg(
        &self,
        _resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph};
        use std::collections::BTreeMap;

        let mut pg = PropertyGraph::new("mta");

        for thread in &self.threads {
            let id = format!("thread_{}", thread.id);
            let mut properties = BTreeMap::new();
            properties.insert(
                "name".to_string(),
                serde_json::Value::String(thread.entry_function.clone()),
            );
            properties.insert(
                "thread_id".to_string(),
                serde_json::Value::Number(thread.id.into()),
            );
            if !thread.context.is_empty() {
                properties.insert(
                    "context".to_string(),
                    serde_json::Value::String(thread.context.clone()),
                );
            }

            pg.nodes.push(PgNode {
                id,
                labels: vec!["Thread".to_string()],
                properties,
            });
        }

        // Add SPAWNS edges (parent -> child)
        for thread in &self.threads {
            if let Some(parent_id) = thread.parent {
                pg.edges.push(PgEdge {
                    src: format!("thread_{parent_id}"),
                    dst: format!("thread_{}", thread.id),
                    edge_type: "SPAWNS".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        // Add MAY_PARALLEL edges
        for pair in &self.concurrency {
            if pair.may_run_concurrently {
                pg.edges.push(PgEdge {
                    src: format!("thread_{}", pair.thread_a),
                    dst: format!("thread_{}", pair.thread_b),
                    edge_type: "MAY_PARALLEL".to_string(),
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

    #[test]
    fn test_export_serialization() {
        let export = MtaExport {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            threads: vec![
                ExportedThread {
                    id: 0,
                    context: "".to_string(),
                    entry_function: "main".to_string(),
                    parent: None,
                    creation_site: None,
                },
                ExportedThread {
                    id: 1,
                    context: "cs1.Call,cs2.foo".to_string(),
                    entry_function: "foo".to_string(),
                    parent: Some(0),
                    creation_site: Some("0x123".to_string()),
                },
            ],
            concurrency: vec![ConcurrencyPair {
                thread_a: 0,
                thread_b: 1,
                may_run_concurrently: true,
            }],
            mhp_points: vec![],
        };

        let json = export.to_json().unwrap();
        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"cs1.Call,cs2.foo\""));

        let parsed: MtaExport = MtaExport::from_json(&json).unwrap();
        assert_eq!(parsed.threads.len(), 2);
    }
}
