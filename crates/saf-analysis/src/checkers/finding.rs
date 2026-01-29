//! Checker finding types and export formats.

use serde::{Deserialize, Serialize};

use crate::svfg::SvfgNodeId;

use super::spec::Severity;

// ---------------------------------------------------------------------------
// NullSourceKind
// ---------------------------------------------------------------------------

/// Distinguishes the origin of a null-dereference source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullSourceKind {
    /// Source is an explicit `NULL` constant assignment (e.g., `ptr = NULL`).
    ExplicitNull,
    /// Source is a function that may return NULL (e.g., `malloc`).
    FunctionMayReturnNull,
    /// Origin unknown.
    Unknown,
}

impl Default for NullSourceKind {
    fn default() -> Self {
        Self::Unknown
    }
}

// ---------------------------------------------------------------------------
// CheckerFinding
// ---------------------------------------------------------------------------

/// A finding from a checker — a source→sink reachability violation.
#[derive(Debug, Clone)]
pub struct CheckerFinding {
    /// Name of the checker that produced this finding.
    pub checker_name: String,
    /// Severity of this finding.
    pub severity: Severity,
    /// The source SVFG node where the issue originates.
    pub source_node: SvfgNodeId,
    /// The sink SVFG node where the issue manifests.
    pub sink_node: SvfgNodeId,
    /// The SVFG path from source to sink.
    pub trace: Vec<SvfgNodeId>,
    /// CWE ID (if applicable).
    pub cwe: Option<u32>,
    /// Human-readable message describing the finding.
    pub message: String,
    /// Per-sink traces for `MultiReach` findings.
    /// Each entry is `(sink_node, trace_from_source_to_sink)`.
    /// Only populated for `MultiReach` mode (e.g., double-free).
    /// Empty for `MayReach`/`MustNotReach` findings.
    pub sink_traces: Vec<(SvfgNodeId, Vec<SvfgNodeId>)>,
    /// For null-deref findings: distinguishes explicit NULL from maybe-NULL sources.
    pub source_kind: NullSourceKind,
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// JSON-serializable finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingExport {
    /// Checker name.
    pub checker: String,
    /// Severity level.
    pub severity: Severity,
    /// CWE ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<u32>,
    /// Human-readable message.
    pub message: String,
    /// Source node hex ID.
    pub source: String,
    /// Sink node hex ID.
    pub sink: String,
    /// Trace (list of hex node IDs).
    pub trace: Vec<String>,
    /// Per-sink traces for `MultiReach` findings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sink_traces: Vec<SinkTraceExport>,
}

/// A single sink trace in a `MultiReach` finding export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkTraceExport {
    /// Sink node hex ID.
    pub sink: String,
    /// Trace from source to this sink (list of hex node IDs).
    pub trace: Vec<String>,
}

impl From<&CheckerFinding> for FindingExport {
    fn from(f: &CheckerFinding) -> Self {
        Self {
            checker: f.checker_name.clone(),
            severity: f.severity,
            cwe: f.cwe,
            message: f.message.clone(),
            source: f.source_node.to_hex(),
            sink: f.sink_node.to_hex(),
            trace: f.trace.iter().map(SvfgNodeId::to_hex).collect(),
            sink_traces: f
                .sink_traces
                .iter()
                .map(|(sink, trace)| SinkTraceExport {
                    sink: sink.to_hex(),
                    trace: trace.iter().map(SvfgNodeId::to_hex).collect(),
                })
                .collect(),
        }
    }
}

/// SARIF 2.1.0 export for checker findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifExport {
    /// SARIF schema URI.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// SARIF version.
    pub version: String,
    /// SARIF runs.
    pub runs: Vec<SarifRun>,
}

/// A SARIF run entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    /// The tool that produced the results.
    pub tool: SarifTool,
    /// The results (findings).
    pub results: Vec<SarifResult>,
}

/// SARIF tool descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    /// Driver info.
    pub driver: SarifDriver,
}

/// SARIF driver info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifDriver {
    /// Tool name.
    pub name: String,
    /// Tool version.
    pub version: String,
    /// Rules (checker specs).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<SarifRule>,
}

/// A SARIF rule (corresponds to a checker spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRule {
    /// Rule ID (checker name).
    pub id: String,
    /// Short description.
    #[serde(rename = "shortDescription")]
    pub short_description: SarifMessage,
    /// Help / full description.
    #[serde(rename = "fullDescription")]
    pub full_description: SarifMessage,
    /// Properties (CWE, severity).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<SarifRuleProperties>,
}

/// SARIF message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    /// Message text.
    pub text: String,
}

/// SARIF rule properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRuleProperties {
    /// CWE IDs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cwe: Vec<String>,
}

/// A SARIF result (corresponds to a finding).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    /// Rule ID this result belongs to.
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    /// Severity level.
    pub level: String,
    /// Finding message.
    pub message: SarifMessage,
    /// CWE IDs on the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<SarifRuleProperties>,
}

/// Convert severity to SARIF level string.
fn severity_to_sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "note",
        Severity::Warning => "warning",
        Severity::Error | Severity::Critical => "error",
    }
}

/// Export findings as JSON-serializable list.
pub fn export_findings_json(findings: &[CheckerFinding]) -> Vec<FindingExport> {
    findings.iter().map(FindingExport::from).collect()
}

/// Export findings as SARIF 2.1.0.
pub fn export_findings_sarif(
    findings: &[CheckerFinding],
    checker_specs: &[super::spec::CheckerSpec],
) -> SarifExport {
    // Build rules from specs
    let rules: Vec<SarifRule> = checker_specs
        .iter()
        .map(|spec| SarifRule {
            id: spec.name.clone(),
            short_description: SarifMessage {
                text: spec.description.clone(),
            },
            full_description: SarifMessage {
                text: spec.description.clone(),
            },
            properties: spec.cwe.map(|cwe| SarifRuleProperties {
                cwe: vec![format!("CWE-{cwe}")],
            }),
        })
        .collect();

    // Build results from findings
    let results: Vec<SarifResult> = findings
        .iter()
        .map(|f| SarifResult {
            rule_id: f.checker_name.clone(),
            level: severity_to_sarif_level(f.severity).to_string(),
            message: SarifMessage {
                text: f.message.clone(),
            },
            properties: f.cwe.map(|cwe| SarifRuleProperties {
                cwe: vec![format!("CWE-{cwe}")],
            }),
        })
        .collect();

    SarifExport {
        schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "SAF Checker Framework".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    rules,
                },
            },
            results,
        }],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ValueId;

    fn sample_finding() -> CheckerFinding {
        CheckerFinding {
            checker_name: "memory-leak".to_string(),
            severity: Severity::Warning,
            source_node: SvfgNodeId::value(ValueId::new(1)),
            sink_node: SvfgNodeId::value(ValueId::new(2)),
            trace: vec![
                SvfgNodeId::value(ValueId::new(1)),
                SvfgNodeId::value(ValueId::new(2)),
            ],
            cwe: Some(401),
            message: "memory-leak: heap allocation not freed".to_string(),
            sink_traces: vec![],
            source_kind: NullSourceKind::Unknown,
        }
    }

    #[test]
    fn finding_export_json() {
        let findings = vec![sample_finding()];
        let exported = export_findings_json(&findings);
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].checker, "memory-leak");
        assert_eq!(exported[0].cwe, Some(401));
        assert_eq!(exported[0].trace.len(), 2);

        // Verify JSON serialization
        let json = serde_json::to_string_pretty(&exported).unwrap();
        assert!(json.contains("memory-leak"));
        assert!(json.contains("401"));
    }

    #[test]
    fn finding_export_sarif() {
        let findings = vec![sample_finding()];
        let specs = vec![super::super::spec::memory_leak()];

        let sarif = export_findings_sarif(&findings, &specs);
        assert_eq!(sarif.version, "2.1.0");
        assert_eq!(sarif.runs.len(), 1);
        assert_eq!(sarif.runs[0].results.len(), 1);
        assert_eq!(sarif.runs[0].tool.driver.rules.len(), 1);
        assert_eq!(sarif.runs[0].results[0].rule_id, "memory-leak");
        assert_eq!(sarif.runs[0].results[0].level, "warning");

        // Verify CWE in rules
        let rule = &sarif.runs[0].tool.driver.rules[0];
        assert!(rule.properties.is_some());
        assert_eq!(rule.properties.as_ref().unwrap().cwe, vec!["CWE-401"]);

        // Verify JSON serialization
        let json = serde_json::to_string_pretty(&sarif).unwrap();
        assert!(json.contains("sarif-2.1.0"));
        assert!(json.contains("CWE-401"));
    }

    #[test]
    fn empty_findings_export() {
        let findings: Vec<CheckerFinding> = vec![];
        let exported = export_findings_json(&findings);
        assert!(exported.is_empty());

        let sarif = export_findings_sarif(&findings, &[]);
        assert!(sarif.runs[0].results.is_empty());
    }
}
