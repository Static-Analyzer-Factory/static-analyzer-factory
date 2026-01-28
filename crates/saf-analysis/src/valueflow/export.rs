//! Export types for value flow analysis results.
//!
//! Provides JSON and SARIF format exports for findings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use saf_core::air::{AirModule, Operation};

use saf_core::span::Span;

use crate::display::DisplayResolver;
use crate::export::{
    PgEdge, PgNode, PropertyGraph, build_value_type_lookup, enrich_node, insert_type_property,
    span_to_property,
};

use super::ValueFlowConfig;
use super::finding::{Finding, FindingId};
use super::query::QueryLimits;
use super::trace::{EnrichedTrace, SpanInfo};

/// Current export schema version.
pub const EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// Exported value flow analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFlowExport {
    /// Schema version.
    pub schema_version: String,
    /// Configuration used.
    pub config: ExportedConfig,
    /// Query limits used.
    pub limits: QueryLimits,
    /// Number of findings.
    pub finding_count: usize,
    /// Exported findings.
    pub findings: Vec<ExportedFinding>,
}

/// Exported configuration (subset of `ValueFlowConfig`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedConfig {
    /// Analysis mode.
    pub mode: String,
    /// Max locations per access.
    pub max_locations_per_access: usize,
}

impl From<&ValueFlowConfig> for ExportedConfig {
    fn from(config: &ValueFlowConfig) -> Self {
        Self {
            mode: format!("{:?}", config.mode).to_lowercase(),
            max_locations_per_access: config.max_locations_per_access,
        }
    }
}

/// Exported finding with enriched trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedFinding {
    /// Finding ID (deterministic).
    pub id: FindingId,
    /// Rule ID if provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    /// Source value info.
    pub source: String,
    /// Sink value info.
    pub sink: String,
    /// Enriched trace.
    pub trace: EnrichedTrace,
}

impl ExportedFinding {
    /// Create an exported finding from a finding and module.
    #[must_use]
    pub fn from_finding(finding: &Finding, module: &AirModule) -> Self {
        Self {
            id: finding.id,
            rule_id: finding.rule_id.clone(),
            source: finding.source.to_hex(),
            sink: finding.sink.to_hex(),
            trace: finding.trace.enrich(module),
        }
    }
}

impl ValueFlowExport {
    /// Create a new export from findings.
    #[must_use]
    pub fn new(
        config: &ValueFlowConfig,
        limits: &QueryLimits,
        findings: &[Finding],
        module: &AirModule,
    ) -> Self {
        let exported_findings: Vec<_> = findings
            .iter()
            .map(|f| ExportedFinding::from_finding(f, module))
            .collect();

        Self {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            config: ExportedConfig::from(config),
            limits: limits.clone(),
            finding_count: exported_findings.len(),
            findings: exported_findings,
        }
    }
}

// =============================================================================
// SARIF Export
// =============================================================================

/// SARIF format export.
///
/// See <https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifExport {
    /// SARIF version.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// SARIF version.
    pub version: String,
    /// Runs.
    pub runs: Vec<SarifRun>,
}

/// A SARIF run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    /// Tool information.
    pub tool: SarifTool,
    /// Results.
    pub results: Vec<SarifResult>,
}

/// SARIF tool information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifTool {
    /// Driver (the analysis tool).
    pub driver: SarifDriver,
}

/// SARIF driver (analysis tool) information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    /// Tool name.
    pub name: String,
    /// Tool version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Information URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    /// Rules used.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<SarifRule>,
}

/// SARIF rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRule {
    /// Rule ID.
    pub id: String,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<SarifMessage>,
    /// Full description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMessage>,
}

/// SARIF message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifMessage {
    /// Message text.
    pub text: String,
}

impl SarifMessage {
    /// Create a new message.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// SARIF result (finding).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    /// Rule ID.
    pub rule_id: String,
    /// Message.
    pub message: SarifMessage,
    /// Locations.
    pub locations: Vec<SarifLocation>,
    /// Code flows (traces).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub code_flows: Vec<SarifCodeFlow>,
    /// Fingerprints for deduplication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<SarifFingerprints>,
}

/// SARIF fingerprints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifFingerprints {
    /// Primary fingerprint.
    #[serde(rename = "primaryLocationLineHash")]
    pub primary: String,
}

/// SARIF location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    /// Physical location.
    pub physical_location: SarifPhysicalLocation,
}

/// SARIF physical location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    /// Artifact location.
    pub artifact_location: SarifArtifactLocation,
    /// Region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

/// SARIF artifact location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    /// URI.
    pub uri: String,
}

/// SARIF region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    /// Start line (1-based).
    pub start_line: u32,
    /// Start column (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    /// End line (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// End column (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
}

impl From<&SpanInfo> for SarifRegion {
    fn from(span: &SpanInfo) -> Self {
        Self {
            start_line: span.start_line,
            start_column: Some(span.start_col),
            end_line: span.end_line,
            end_column: span.end_col,
        }
    }
}

/// SARIF code flow (trace).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifCodeFlow {
    /// Thread flows.
    pub thread_flows: Vec<SarifThreadFlow>,
}

/// SARIF thread flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifThreadFlow {
    /// Locations in the flow.
    pub locations: Vec<SarifThreadFlowLocation>,
}

/// SARIF thread flow location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifThreadFlowLocation {
    /// Location.
    pub location: SarifLocation,
}

impl SarifExport {
    /// SARIF schema URL.
    pub const SCHEMA: &'static str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json";
    /// SARIF version.
    pub const VERSION: &'static str = "2.1.0";

    /// Create a SARIF export from findings.
    #[must_use]
    pub fn from_findings(
        findings: &[Finding],
        module: &AirModule,
        tool_name: &str,
        tool_version: Option<&str>,
    ) -> Self {
        let results: Vec<_> = findings
            .iter()
            .map(|f| sarif_result_from_finding(f, module))
            .collect();

        // Collect unique rules
        let rules: Vec<_> = findings
            .iter()
            .filter_map(|f| f.rule_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|id| SarifRule {
                id,
                short_description: None,
                full_description: None,
            })
            .collect();

        Self {
            schema: Self::SCHEMA.to_string(),
            version: Self::VERSION.to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: tool_name.to_string(),
                        version: tool_version.map(String::from),
                        information_uri: None,
                        rules,
                    },
                },
                results,
            }],
        }
    }
}

/// Convert a finding to a SARIF result.
fn sarif_result_from_finding(finding: &Finding, module: &AirModule) -> SarifResult {
    let enriched = finding.trace.enrich(module);

    // Build locations from trace
    let locations = if let Some(first_step) = enriched.steps.first() {
        if let Some(span) = &first_step.from.span {
            vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: span.file.clone(),
                    },
                    region: Some(SarifRegion::from(span)),
                },
            }]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Build code flow from trace
    let thread_flow_locations: Vec<_> = enriched
        .steps
        .iter()
        .filter_map(|step| {
            step.from.span.as_ref().map(|span| SarifThreadFlowLocation {
                location: SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: span.file.clone(),
                        },
                        region: Some(SarifRegion::from(span)),
                    },
                },
            })
        })
        .collect();

    let code_flows = if thread_flow_locations.is_empty() {
        vec![]
    } else {
        vec![SarifCodeFlow {
            thread_flows: vec![SarifThreadFlow {
                locations: thread_flow_locations,
            }],
        }]
    };

    SarifResult {
        rule_id: finding
            .rule_id
            .clone()
            .unwrap_or_else(|| "taint-flow".to_string()),
        message: SarifMessage::new(format!(
            "Data flows from {} to {}",
            finding.source.to_hex(),
            finding.sink.to_hex()
        )),
        locations,
        code_flows,
        fingerprints: Some(SarifFingerprints {
            primary: finding.id.to_hex(),
        }),
    }
}

/// Export a [`ValueFlowGraph`](super::ValueFlowGraph) as a unified [`PropertyGraph`].
///
/// The `module` parameter provides span information and type annotations
/// for value and location nodes.
pub fn to_property_graph(
    vfg: &super::ValueFlowGraph,
    module: &AirModule,
    resolver: Option<&DisplayResolver<'_>>,
) -> PropertyGraph {
    // Build ID → span lookup from all instructions
    let mut id_spans: BTreeMap<u128, &Span> = BTreeMap::new();
    // Build param ID → parent function name for parameter nodes
    let mut param_func_name: BTreeMap<u128, &str> = BTreeMap::new();
    // Build function ID → function lookup for call-site parameter mapping
    let func_by_id: BTreeMap<u128, &saf_core::air::AirFunction> =
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
                    // Propagate call-site spans to callee parameter nodes so
                    // that `source_line(param_id)` resolves to the call site.
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

    let mut pg = PropertyGraph::new("valueflow");
    pg.metadata.insert(
        "node_count".to_string(),
        serde_json::json!(vfg.node_count()),
    );
    pg.metadata.insert(
        "edge_count".to_string(),
        serde_json::json!(vfg.edge_count()),
    );

    for node in vfg.nodes() {
        let (id, raw_id, label, kind_str) = match node {
            super::NodeId::Value { id } => (id.to_hex(), Some(id.raw()), "Value", "value"),
            super::NodeId::Location { id } => (id.to_hex(), Some(id.raw()), "Location", "location"),
            super::NodeId::UnknownMem => {
                ("unknown_mem".to_string(), None, "UnknownMem", "unknown_mem")
            }
        };

        let mut properties = BTreeMap::new();
        properties.insert("kind".to_string(), serde_json::json!(kind_str));
        if let Some(raw) = raw_id {
            if let Some(span) = id_spans.get(&raw) {
                properties.insert(
                    "span".to_string(),
                    span_to_property(span, &module.source_files),
                );
            }
            if let Some(fname) = param_func_name.get(&raw) {
                properties.insert("parent_function".to_string(), serde_json::json!(fname));
            }
            insert_type_property(&mut properties, raw, &value_types, module);
        }

        let mut pg_node = PgNode {
            id,
            labels: vec![label.to_string()],
            properties,
        };
        enrich_node(&mut pg_node, resolver);
        pg.nodes.push(pg_node);
    }

    for node in vfg.nodes() {
        if let Some(succs) = vfg.successors_of(*node) {
            let src_id = match node {
                super::NodeId::Value { id } => id.to_hex(),
                super::NodeId::Location { id } => id.to_hex(),
                super::NodeId::UnknownMem => "unknown_mem".to_string(),
            };
            for (kind, succ) in succs {
                let dst_id = match succ {
                    super::NodeId::Value { id } => id.to_hex(),
                    super::NodeId::Location { id } => id.to_hex(),
                    super::NodeId::UnknownMem => "unknown_mem".to_string(),
                };
                pg.edges.push(PgEdge {
                    src: src_id.clone(),
                    dst: dst_id,
                    edge_type: format!("{kind:?}").to_uppercase(),
                    properties: BTreeMap::new(),
                });
            }
        }
    }

    pg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valueflow::trace::{Trace, TraceStep};
    use crate::valueflow::{EdgeKind, NodeId};
    use saf_core::ids::{ModuleId, ValueId};

    fn make_module() -> AirModule {
        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: Vec::new(),
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

    fn make_finding() -> Finding {
        let trace = Trace::from_steps(vec![TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        )]);
        Finding::new(
            ValueId::new(1),
            ValueId::new(2),
            trace,
            Some("SQL_INJECTION".to_string()),
        )
    }

    #[test]
    fn export_schema_version() {
        let config = ValueFlowConfig::fast();
        let limits = QueryLimits::default();
        let findings = vec![make_finding()];
        let module = make_module();

        let export = ValueFlowExport::new(&config, &limits, &findings, &module);
        assert_eq!(export.schema_version, EXPORT_SCHEMA_VERSION);
    }

    #[test]
    fn export_finding_count() {
        let config = ValueFlowConfig::fast();
        let limits = QueryLimits::default();
        let findings = vec![make_finding(), make_finding()];
        let module = make_module();

        let export = ValueFlowExport::new(&config, &limits, &findings, &module);
        assert_eq!(export.finding_count, 2);
        assert_eq!(export.findings.len(), 2);
    }

    #[test]
    fn export_serialization_roundtrip() {
        let config = ValueFlowConfig::fast();
        let limits = QueryLimits::default();
        let findings = vec![make_finding()];
        let module = make_module();

        let export = ValueFlowExport::new(&config, &limits, &findings, &module);
        let json = serde_json::to_string(&export).unwrap();
        let parsed: ValueFlowExport = serde_json::from_str(&json).unwrap();

        assert_eq!(export.schema_version, parsed.schema_version);
        assert_eq!(export.finding_count, parsed.finding_count);
    }

    #[test]
    fn sarif_schema_and_version() {
        let findings = vec![make_finding()];
        let module = make_module();

        let sarif = SarifExport::from_findings(&findings, &module, "SAF", Some("0.1.0"));
        assert_eq!(sarif.version, SarifExport::VERSION);
        assert!(sarif.schema.contains("sarif-schema"));
    }

    #[test]
    fn sarif_results_match_findings() {
        let findings = vec![make_finding()];
        let module = make_module();

        let sarif = SarifExport::from_findings(&findings, &module, "SAF", Some("0.1.0"));
        assert_eq!(sarif.runs.len(), 1);
        assert_eq!(sarif.runs[0].results.len(), 1);
        assert_eq!(sarif.runs[0].results[0].rule_id, "SQL_INJECTION");
    }

    #[test]
    fn sarif_fingerprint_matches_finding_id() {
        let finding = make_finding();
        let findings = vec![finding.clone()];
        let module = make_module();

        let sarif = SarifExport::from_findings(&findings, &module, "SAF", None);
        let fingerprints = sarif.runs[0].results[0].fingerprints.as_ref().unwrap();
        assert_eq!(fingerprints.primary, finding.id.to_hex());
    }

    #[test]
    fn sarif_serialization_valid() {
        let findings = vec![make_finding()];
        let module = make_module();

        let sarif = SarifExport::from_findings(&findings, &module, "SAF", Some("0.1.0"));
        let json = serde_json::to_string_pretty(&sarif).unwrap();

        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Should contain expected fields
        assert!(json.contains("$schema"));
        assert!(json.contains("version"));
        assert!(json.contains("runs"));
        assert!(json.contains("results"));
    }

    #[test]
    fn sarif_default_rule_id() {
        let trace = Trace::from_steps(vec![TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        )]);
        let finding = Finding::new(ValueId::new(1), ValueId::new(2), trace, None);
        let findings = vec![finding];
        let module = make_module();

        let sarif = SarifExport::from_findings(&findings, &module, "SAF", None);
        assert_eq!(sarif.runs[0].results[0].rule_id, "taint-flow");
    }
}
