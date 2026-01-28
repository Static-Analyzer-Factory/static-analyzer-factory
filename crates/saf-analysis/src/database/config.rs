//! Declarative analysis configuration types.
//!
//! `AnalysisConfig` is the JSON schema that LLM agents use to describe
//! what properties to check. It decomposes into graph primitive operations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Severity level for analysis findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational — no action needed.
    Info,
    /// Warning — potential issue.
    Warning,
    /// Error — likely bug.
    Error,
    /// Critical — definite security vulnerability.
    Critical,
}

/// A declarative specification of a program site (source, sink, or barrier).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteSpec {
    /// Match a function call by name.
    Call {
        /// Function name to match.
        name: String,
        /// Which argument to track (0-indexed).
        #[serde(skip_serializing_if = "Option::is_none")]
        arg: Option<u32>,
        /// Whether to match the return value instead of argument.
        #[serde(default)]
        match_return: bool,
    },
    /// Match a dereference of a bound variable.
    Deref {
        /// Variable name to match against bindings.
        bind_var: String,
    },
    /// Match any function exit.
    FunctionExit {},
}

/// Where to bind a variable (at source or sink site).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindSite {
    /// Bind at the source site.
    Source,
    /// Bind at the sink site.
    Sink,
}

/// What to bind (argument, return value, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindTarget {
    /// Bind a function argument by index.
    Arg(u32),
    /// Bind the return value.
    ReturnValue,
}

/// Variable binding for alias correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindSpec {
    /// Where to bind (source or sink).
    pub at: BindSite,
    /// What to bind (arg or return value).
    pub what: BindTarget,
}

/// A state transition in a typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Site that triggers the transition.
    pub at: SiteSpec,
    /// Required state before transition (None = any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// State after transition.
    pub to: String,
}

/// Error condition for typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCondition {
    /// State that triggers an error.
    pub state: String,
    /// Whether the error occurs at a sink.
    pub at_sink: bool,
}

/// State machine configuration for typestate analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStateConfig {
    /// List of valid states.
    pub states: Vec<String>,
    /// Initial state.
    pub initial: String,
    /// State transitions.
    pub transitions: Vec<Transition>,
    /// Error condition.
    pub error: ErrorCondition,
}

/// The unified analysis configuration.
///
/// LLM agents generate this JSON to describe what properties to check.
/// The query decomposition engine translates it into graph primitive operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Name of the analysis.
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// CWE ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<u32>,
    /// Severity level.
    pub severity: Severity,
    /// Source site specifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SiteSpec>>,
    /// Sink site specifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sinks: Option<Vec<SiteSpec>>,
    /// Barrier/sanitizer site specifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barriers: Option<Vec<SiteSpec>>,
    /// Variable bindings for alias correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_variables: Option<BTreeMap<String, BindSpec>>,
    /// Flow state configuration for typestate analysis.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_states: Option<FlowStateConfig>,
}

impl AnalysisConfig {
    /// Check if this config uses bind variables (alias correlation).
    pub fn has_bind_variables(&self) -> bool {
        self.bind_variables
            .as_ref()
            .is_some_and(|bv| !bv.is_empty())
    }

    /// Check if this config uses flow state tracking (typestate).
    pub fn has_flow_states(&self) -> bool {
        self.flow_states.is_some()
    }

    /// Check if all sinks are function exits (must-reach pattern).
    pub fn sinks_are_function_exit(&self) -> bool {
        self.sinks.as_ref().is_some_and(|sinks| {
            sinks
                .iter()
                .all(|s| matches!(s, SiteSpec::FunctionExit { .. }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_config_deserializes_from_json() {
        let json = r#"{
            "name": "test-taint",
            "severity": "error",
            "sources": [{"call": {"name": "read"}}],
            "sinks": [{"call": {"name": "exec"}}]
        }"#;
        let config: AnalysisConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "test-taint");
        assert_eq!(config.sources.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn site_spec_call_variant() {
        let json = r#"{"call": {"name": "free", "arg": 0}}"#;
        let spec: SiteSpec = serde_json::from_str(json).unwrap();
        assert!(matches!(spec, SiteSpec::Call { .. }));
    }

    #[test]
    fn flow_state_config_roundtrip() {
        let json = r#"{
            "states": ["Alive", "Freed"],
            "initial": "Alive",
            "transitions": [{"at": {"call": {"name": "free"}}, "to": "Freed"}],
            "error": {"state": "Freed", "at_sink": true}
        }"#;
        let fsc: FlowStateConfig = serde_json::from_str(json).unwrap();
        assert_eq!(fsc.states.len(), 2);
        assert_eq!(fsc.initial, "Alive");
    }
}
