//! IDE analysis result types.
//!
//! Contains the result of an IDE analysis including both IFDS-level facts and
//! IDE-level values at each program point.

use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use saf_core::ids::InstId;

use super::lattice::Lattice;
use super::result::IfdsResult;

/// Result of an IDE analysis.
#[derive(Debug, Clone)]
pub struct IdeResult<F: Ord + Clone + Debug, V: Lattice> {
    /// Values at each instruction for each fact.
    pub values: BTreeMap<InstId, BTreeMap<F, V>>,
    /// The underlying IFDS result (facts, summaries, diagnostics).
    pub ifds_result: IfdsResult<F>,
    /// IDE-specific diagnostics.
    pub diagnostics: IdeDiagnostics,
}

/// IDE solver diagnostics.
#[derive(Debug, Clone, Default)]
pub struct IdeDiagnostics {
    /// Number of jump function updates in Phase 1.
    pub jump_fn_updates: usize,
    /// Number of value propagation steps in Phase 2.
    pub value_propagations: usize,
    /// Total distinct (d1, inst, d2) entries in the jump function table.
    pub jump_fn_entries: usize,
}

impl<F: Ord + Clone + Debug, V: Lattice> IdeResult<F, V> {
    /// Get all values at a specific instruction.
    #[must_use]
    pub fn values_at(&self, inst: InstId) -> Option<&BTreeMap<F, V>> {
        self.values.get(&inst)
    }

    /// Get the value for a specific fact at a specific instruction.
    #[must_use]
    pub fn value_at(&self, inst: InstId, fact: &F) -> Option<&V> {
        self.values.get(&inst).and_then(|m| m.get(fact))
    }

    /// Check if a specific fact holds at a specific instruction (delegates to IFDS).
    #[must_use]
    pub fn holds_at(&self, inst: InstId, fact: &F) -> bool {
        self.ifds_result.holds_at(inst, fact)
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Like [`IfdsResult::to_pg`](super::result::IfdsResult::to_pg) but fact
    /// nodes also carry a `value` property with the IDE-level computed value.
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
        use std::collections::BTreeMap;

        let mut pg = PropertyGraph::new("ide");

        let mut fact_counter: u64 = 0;

        for (inst_id, fact_map) in &self.values {
            // Instruction node
            let mut inst_node = PgNode {
                id: inst_id.to_hex(),
                labels: vec!["Instruction".to_string()],
                properties: BTreeMap::new(),
            };
            enrich_node(&mut inst_node, resolver);
            pg.nodes.push(inst_node);

            for (fact, val) in fact_map {
                let fact_debug = format!("{fact:?}");
                let val_debug = format!("{val:?}");
                let fact_id = format!("fact_{fact_counter}");
                fact_counter += 1;

                let mut properties = BTreeMap::new();
                properties.insert("fact".to_string(), serde_json::json!(fact_debug));
                properties.insert("value".to_string(), serde_json::json!(val_debug));

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

    /// Export the IDE result to a serializable format.
    #[must_use]
    pub fn export(&self) -> IdeExport {
        let values: BTreeMap<String, BTreeMap<String, String>> = self
            .values
            .iter()
            .map(|(inst_id, fact_map)| {
                let key = inst_id.to_hex();
                let facts: BTreeMap<String, String> = fact_map
                    .iter()
                    .map(|(f, v)| (format!("{f:?}"), format!("{v:?}")))
                    .collect();
                (key, facts)
            })
            .collect();

        let ifds_export = self.ifds_result.export();

        IdeExport {
            values,
            ifds: serde_json::to_value(&ifds_export).unwrap_or_default(),
            diagnostics: IdeDiagnosticsExport::from(&self.diagnostics),
        }
    }
}

/// Exportable IDE result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeExport {
    /// Values per instruction per fact.
    pub values: BTreeMap<String, BTreeMap<String, String>>,
    /// The underlying IFDS export.
    pub ifds: serde_json::Value,
    /// IDE diagnostics.
    pub diagnostics: IdeDiagnosticsExport,
}

/// Exportable IDE diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeDiagnosticsExport {
    /// Jump function updates in Phase 1.
    pub jump_fn_updates: usize,
    /// Value propagation steps in Phase 2.
    pub value_propagations: usize,
    /// Entries in the jump function table.
    pub jump_fn_entries: usize,
}

impl From<&IdeDiagnostics> for IdeDiagnosticsExport {
    fn from(d: &IdeDiagnostics) -> Self {
        Self {
            jump_fn_updates: d.jump_fn_updates,
            value_propagations: d.value_propagations,
            jump_fn_entries: d.jump_fn_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::result::IfdsDiagnostics;
    use super::*;
    use std::collections::BTreeSet;

    use saf_core::ids::FunctionId;

    /// Simple test lattice.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum TestVal {
        Bot,
        Top,
    }

    impl Lattice for TestVal {
        fn top() -> Self {
            TestVal::Top
        }
        fn bottom() -> Self {
            TestVal::Bot
        }
        fn join(&self, other: &Self) -> Self {
            if *self == TestVal::Top || *other == TestVal::Top {
                TestVal::Top
            } else {
                TestVal::Bot
            }
        }
        fn meet(&self, other: &Self) -> Self {
            if *self == TestVal::Bot || *other == TestVal::Bot {
                TestVal::Bot
            } else {
                TestVal::Top
            }
        }
        fn leq(&self, other: &Self) -> bool {
            self.join(other) == *other
        }
    }

    #[test]
    fn value_at_returns_none_for_unknown() {
        let result: IdeResult<u64, TestVal> = IdeResult {
            values: BTreeMap::new(),
            ifds_result: IfdsResult {
                facts: BTreeMap::new(),
                summaries: BTreeMap::<FunctionId, BTreeSet<(u64, u64)>>::new(),
                diagnostics: IfdsDiagnostics::default(),
            },
            diagnostics: IdeDiagnostics::default(),
        };
        assert!(result.value_at(InstId::new(999), &42).is_none());
    }

    #[test]
    fn export_produces_valid_json() {
        let mut values = BTreeMap::new();
        let mut fact_map = BTreeMap::new();
        fact_map.insert(1_u64, TestVal::Top);
        values.insert(InstId::new(1), fact_map);

        let result: IdeResult<u64, TestVal> = IdeResult {
            values,
            ifds_result: IfdsResult {
                facts: BTreeMap::new(),
                summaries: BTreeMap::<FunctionId, BTreeSet<(u64, u64)>>::new(),
                diagnostics: IfdsDiagnostics::default(),
            },
            diagnostics: IdeDiagnostics::default(),
        };

        let export = result.export();
        let json = serde_json::to_string(&export).expect("export should serialize");
        assert!(json.contains("values"));
        assert!(json.contains("diagnostics"));
    }
}
