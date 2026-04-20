//! JSON protocol types for LLM-SAF communication.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::config::AnalysisConfig;

/// A JSON request from an LLM agent to SAF.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Wire-protocol enum — variants vary in size intentionally
pub enum Request {
    /// Get the API schema and available checks.
    Schema,
    /// Run a named check or template.
    Check {
        /// Check name.
        name: String,
        /// Optional parameters.
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<BTreeMap<String, serde_json::Value>>,
    },
    /// Run all named checks.
    CheckAll,
    /// Run a full analysis config.
    Analyze {
        /// The analysis configuration.
        config: AnalysisConfig,
    },
    /// Run a Cypher query against Neo4j.
    Cypher {
        /// The Cypher query string.
        query: String,
        /// Query parameters.
        #[serde(default)]
        params: BTreeMap<String, serde_json::Value>,
    },
    /// Run a graph primitive query.
    Query {
        /// Query type (e.g., "alias", "points_to").
        #[serde(rename = "type")]
        query_type: String,
        /// Query parameters.
        params: BTreeMap<String, serde_json::Value>,
    },
}

/// Language for query execution.
///
/// Determines which engine processes the query: built-in checkers,
/// Python-based graph traversal, or Datalog rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryLanguage {
    /// Built-in checker queries (check, check_all).
    Builtin,
    /// Python-based graph traversal queries.
    Python,
    /// Datalog rules (Phase 2 — deferred).
    Datalog,
}

/// A finding from an analysis check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Check name that produced this finding.
    pub check: String,
    /// Severity level.
    pub severity: String,
    /// CWE ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<u32>,
    /// Human-readable message.
    pub message: String,
    /// Path trace events.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub path: Vec<PathEvent>,
    /// Object name (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Human-readable display name for the finding's primary entity
    /// (e.g., `"variable 'p'"`), resolved via `DisplayResolver`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// An event in a finding's path trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEvent {
    /// Location description (e.g., `"main() at file.c:12"` or hex ID).
    pub location: String,
    /// Event description (e.g., `"memory allocated"`, `"pointer used after free"`).
    pub event: String,
    /// State (for typestate analysis).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Human-readable display name for the entity at this location
    /// (e.g., `"variable 'p'"`, `"call @malloc"`), resolved via `DisplayResolver`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Source location details (file, line, column) for this event,
    /// resolved via `DisplayResolver`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_loc: Option<crate::display::SourceLoc>,
}

/// Metadata about a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Elapsed time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    /// Analysis engines used.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub engines_used: Vec<String>,
}

/// Error details in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
    /// Suggestions for fixing the error.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// A JSON response from SAF to an LLM agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Status: "ok" or "error".
    pub status: String,
    /// Findings (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<Finding>>,
    /// Query results (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<BTreeMap<String, serde_json::Value>>>,
    /// Error details (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
    /// Response metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ResponseMetadata>,
    /// Engine that processed the query (e.g., "builtin", "datalog").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Extra data (e.g., schema information).
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Response {
    /// Create a success response with findings.
    pub fn ok_findings(findings: Vec<Finding>, metadata: ResponseMetadata) -> Self {
        Self {
            status: "ok".to_string(),
            findings: Some(findings),
            results: None,
            error: None,
            metadata: Some(metadata),
            engine: None,
            extra: BTreeMap::new(),
        }
    }

    /// Create a success response with query results (not findings).
    ///
    /// Use this for `points_to`, `alias`, and other query-type responses
    /// that return structured result rows rather than checker findings.
    pub fn ok_results(
        results: Vec<BTreeMap<String, serde_json::Value>>,
        metadata: ResponseMetadata,
    ) -> Self {
        Self {
            status: "ok".to_string(),
            findings: None,
            results: Some(results),
            error: None,
            metadata: Some(metadata),
            engine: None,
            extra: BTreeMap::new(),
        }
    }

    /// Create an error response.
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            findings: None,
            results: None,
            error: Some(ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                suggestions: Vec::new(),
            }),
            metadata: None,
            engine: None,
            extra: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_request_deserializes() {
        let json = r#"{"action": "schema"}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Schema));
    }

    #[test]
    fn check_request_deserializes() {
        let json = r#"{"action": "check", "name": "use_after_free"}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Check { .. }));
    }

    #[test]
    fn check_request_with_params() {
        let json =
            r#"{"action": "check", "name": "taint", "params": {"source": "read", "sink": "exec"}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        if let Request::Check { name, params } = req {
            assert_eq!(name, "taint");
            assert!(params.is_some());
        } else {
            panic!("expected Check variant");
        }
    }

    #[test]
    fn analyze_request_deserializes() {
        let json = r#"{"action": "analyze", "config": {"name": "test", "severity": "error", "sources": [{"call": {"name": "read"}}]}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        assert!(matches!(req, Request::Analyze { .. }));
    }

    #[test]
    fn error_response_serializes() {
        let resp = Response::error("UNKNOWN_CHECK", "No check named 'uaf'");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("UNKNOWN_CHECK"));
    }

    #[test]
    fn query_language_serializes_snake_case() {
        let lang = QueryLanguage::Builtin;
        let json = serde_json::to_value(&lang).unwrap();
        assert_eq!(json, "builtin");

        let lang = QueryLanguage::Datalog;
        let json = serde_json::to_value(&lang).unwrap();
        assert_eq!(json, "datalog");

        let lang = QueryLanguage::Python;
        let json = serde_json::to_value(&lang).unwrap();
        assert_eq!(json, "python");
    }

    #[test]
    fn query_language_deserializes() {
        let lang: QueryLanguage = serde_json::from_str(r#""builtin""#).unwrap();
        assert_eq!(lang, QueryLanguage::Builtin);

        let lang: QueryLanguage = serde_json::from_str(r#""datalog""#).unwrap();
        assert_eq!(lang, QueryLanguage::Datalog);
    }

    #[test]
    fn response_engine_field_skipped_when_none() {
        let resp = Response::ok_findings(
            vec![],
            ResponseMetadata {
                elapsed_ms: Some(10),
                engines_used: vec![],
            },
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("engine"));
    }

    #[test]
    fn response_engine_field_present_when_set() {
        let mut resp = Response::ok_findings(
            vec![],
            ResponseMetadata {
                elapsed_ms: Some(10),
                engines_used: vec![],
            },
        );
        resp.engine = Some("datalog".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""engine":"datalog""#));
    }

    #[test]
    fn query_request_deserializes() {
        let json = r#"{"action": "query", "type": "alias", "params": {"language": "builtin"}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        if let Request::Query { query_type, params } = req {
            assert_eq!(query_type, "alias");
            assert_eq!(
                params.get("language").and_then(|v| v.as_str()),
                Some("builtin")
            );
        } else {
            panic!("expected Query variant");
        }
    }
}
