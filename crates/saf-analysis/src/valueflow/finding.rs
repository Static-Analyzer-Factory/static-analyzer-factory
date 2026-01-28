//! Finding types for value flow analysis results.
//!
//! Represents individual findings from taint analysis with deterministic IDs.

use serde::{Deserialize, Serialize};

use saf_core::id::make_id;
use saf_core::ids::ValueId;

use super::trace::Trace;

/// Unique identifier for a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FindingId(u128);

impl FindingId {
    /// Create a finding ID from a raw value.
    #[must_use]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Derive a finding ID from source, sink, and trace.
    #[must_use]
    pub fn derive(source: ValueId, sink: ValueId, trace: &Trace, rule_id: Option<&str>) -> Self {
        let mut data = Vec::new();

        // Include source and sink
        data.extend_from_slice(&source.raw().to_le_bytes());
        data.extend_from_slice(&sink.raw().to_le_bytes());

        // Include trace length and edges
        data.extend_from_slice(&(trace.len() as u64).to_le_bytes());
        for step in &trace.steps {
            // Include edge kind
            data.push(step.edge as u8);
        }

        // Include rule ID if present
        if let Some(rule) = rule_id {
            data.extend_from_slice(rule.as_bytes());
        }

        Self(make_id("finding", &data))
    }

    /// Get the raw u128 value.
    #[must_use]
    pub const fn raw(self) -> u128 {
        self.0
    }

    /// Format as hex string.
    #[must_use]
    pub fn to_hex(self) -> String {
        saf_core::id::id_to_hex(self.0)
    }
}

impl Serialize for FindingId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for FindingId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let hex_str = s.strip_prefix("0x").unwrap_or(&s);
        let id = u128::from_str_radix(hex_str, 16).map_err(serde::de::Error::custom)?;
        Ok(Self(id))
    }
}

/// A finding from taint analysis.
#[derive(Debug, Clone)]
pub struct Finding {
    /// Deterministic ID.
    pub id: FindingId,
    /// Source value.
    pub source: ValueId,
    /// Sink value.
    pub sink: ValueId,
    /// Trace from source to sink.
    pub trace: Trace,
    /// Optional rule ID (user-provided or auto-generated).
    pub rule_id: Option<String>,
}

impl Finding {
    /// Create a new finding.
    #[must_use]
    pub fn new(source: ValueId, sink: ValueId, trace: Trace, rule_id: Option<String>) -> Self {
        let id = FindingId::derive(source, sink, &trace, rule_id.as_deref());
        Self {
            id,
            source,
            sink,
            trace,
            rule_id,
        }
    }

    /// Create a finding from a flow.
    #[must_use]
    pub fn from_flow(flow: super::query::Flow, rule_id: Option<String>) -> Self {
        Self::new(flow.source, flow.sink, flow.trace, rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valueflow::trace::TraceStep;
    use crate::valueflow::{EdgeKind, NodeId};

    fn make_trace() -> Trace {
        Trace::from_steps(vec![
            TraceStep::new(
                NodeId::value(ValueId::new(1)),
                EdgeKind::DefUse,
                NodeId::value(ValueId::new(2)),
            ),
            TraceStep::new(
                NodeId::value(ValueId::new(2)),
                EdgeKind::Transform,
                NodeId::value(ValueId::new(3)),
            ),
        ])
    }

    #[test]
    fn finding_id_derive_deterministic() {
        let trace = make_trace();
        let id1 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, None);
        let id2 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, None);
        assert_eq!(id1, id2);
    }

    #[test]
    fn finding_id_differs_by_source() {
        let trace = make_trace();
        let id1 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, None);
        let id2 = FindingId::derive(ValueId::new(2), ValueId::new(3), &trace, None);
        assert_ne!(id1, id2);
    }

    #[test]
    fn finding_id_differs_by_sink() {
        let trace = make_trace();
        let id1 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, None);
        let id2 = FindingId::derive(ValueId::new(1), ValueId::new(4), &trace, None);
        assert_ne!(id1, id2);
    }

    #[test]
    fn finding_id_differs_by_rule() {
        let trace = make_trace();
        let id1 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, None);
        let id2 = FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, Some("rule1"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn finding_id_hex() {
        let id = FindingId::new(0x123);
        let hex = id.to_hex();
        assert!(hex.starts_with("0x"));
        assert_eq!(hex.len(), 34); // 0x + 32 hex chars
    }

    #[test]
    fn finding_id_serialization_roundtrip() {
        let id = FindingId::derive(ValueId::new(1), ValueId::new(3), &make_trace(), None);
        let json = serde_json::to_string(&id).unwrap();
        let parsed: FindingId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn finding_creation() {
        let trace = make_trace();
        let finding = Finding::new(ValueId::new(1), ValueId::new(3), trace.clone(), None);

        assert_eq!(finding.source, ValueId::new(1));
        assert_eq!(finding.sink, ValueId::new(3));
        assert_eq!(finding.trace.len(), 2);
        assert!(finding.rule_id.is_none());
    }

    #[test]
    fn finding_with_rule_id() {
        let trace = make_trace();
        let finding = Finding::new(
            ValueId::new(1),
            ValueId::new(3),
            trace,
            Some("SQL_INJECTION".to_string()),
        );

        assert_eq!(finding.rule_id, Some("SQL_INJECTION".to_string()));
    }

    #[test]
    fn finding_id_from_finding_matches_derive() {
        let trace = make_trace();
        let rule_id = Some("test_rule".to_string());
        let finding = Finding::new(
            ValueId::new(1),
            ValueId::new(3),
            trace.clone(),
            rule_id.clone(),
        );

        let expected_id =
            FindingId::derive(ValueId::new(1), ValueId::new(3), &trace, rule_id.as_deref());
        assert_eq!(finding.id, expected_id);
    }
}
