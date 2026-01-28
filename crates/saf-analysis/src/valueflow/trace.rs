//! Trace types for value flow paths.
//!
//! Represents execution traces through the value flow graph, with optional
//! enrichment for export.

use serde::{Deserialize, Serialize};

use saf_core::air::AirModule;
use saf_core::span::Span;

use super::edge::EdgeKind;
use super::node::NodeId;

/// A step in a trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    /// Source node.
    pub from: NodeId,
    /// Edge kind.
    pub edge: EdgeKind,
    /// Target node.
    pub to: NodeId,
}

impl TraceStep {
    /// Create a new trace step.
    #[must_use]
    pub const fn new(from: NodeId, edge: EdgeKind, to: NodeId) -> Self {
        Self { from, edge, to }
    }
}

/// A trace through the value flow graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trace {
    /// Steps in the trace (from source to sink).
    pub steps: Vec<TraceStep>,
}

impl Trace {
    /// Create an empty trace.
    #[must_use]
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Create a trace from steps.
    #[must_use]
    pub fn from_steps(steps: Vec<TraceStep>) -> Self {
        Self { steps }
    }

    /// Get the number of steps.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the trace is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Add a step to the trace.
    pub fn push(&mut self, step: TraceStep) {
        self.steps.push(step);
    }

    /// Create a new trace with an additional step.
    #[must_use]
    pub fn with_step(&self, step: TraceStep) -> Self {
        let mut new_trace = self.clone();
        new_trace.push(step);
        new_trace
    }

    /// Enrich the trace with module information.
    #[must_use]
    pub fn enrich(&self, module: &AirModule) -> EnrichedTrace {
        EnrichedTrace::from_trace(self, module)
    }
}

/// Information about a node for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID as string.
    pub id: String,
    /// Node kind (`"value"`, `"location"`, `"unknown_mem"`).
    pub kind: String,
    /// Symbol/name if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Source span if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SpanInfo>,
}

/// Source span information for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Source file path.
    pub file: String,
    /// Start line (1-based).
    pub start_line: u32,
    /// Start column (1-based).
    pub start_col: u32,
    /// End line (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    /// End column (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u32>,
}

impl SpanInfo {
    /// Create from a span and module.
    #[must_use]
    pub fn from_span(span: &Span, module: &AirModule) -> Self {
        let file = module
            .source_files
            .iter()
            .find(|sf| sf.id == span.file_id)
            .map_or_else(
                || format!("file_{:x}", span.file_id.raw()),
                |sf| sf.path.clone(),
            );

        Self {
            file,
            start_line: span.line_start,
            start_col: span.col_start,
            end_line: Some(span.line_end),
            end_col: Some(span.col_end),
        }
    }
}

impl NodeInfo {
    /// Create node info from a node ID and module.
    #[must_use]
    pub fn from_node(node: NodeId, module: &AirModule) -> Self {
        match node {
            NodeId::Value { id } => {
                // Try to find symbol and span from the module
                let (symbol, span) = find_value_info(id, module);
                Self {
                    id: id.to_hex(),
                    kind: "value".to_string(),
                    symbol,
                    span: span.map(|s| SpanInfo::from_span(&s, module)),
                }
            }
            NodeId::Location { id } => Self {
                id: id.to_hex(),
                kind: "location".to_string(),
                symbol: None,
                span: None,
            },
            NodeId::UnknownMem => Self {
                id: "unknown_mem".to_string(),
                kind: "unknown_mem".to_string(),
                symbol: None,
                span: None,
            },
        }
    }
}

/// Find symbol and span for a value ID.
fn find_value_info(
    value_id: saf_core::ids::ValueId,
    module: &AirModule,
) -> (Option<String>, Option<Span>) {
    // Check function parameters
    for func in &module.functions {
        for param in &func.params {
            if param.id == value_id {
                return (param.name.clone(), func.span.clone());
            }
        }

        // Check instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.dst == Some(value_id) {
                    return (
                        inst.symbol.as_ref().map(|s| s.display_name.clone()),
                        inst.span.clone(),
                    );
                }
            }
        }
    }

    // Check globals
    for global in &module.globals {
        if global.id == value_id {
            return (Some(global.name.clone()), global.span.clone());
        }
    }

    (None, None)
}

/// An enriched step with full node information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedStep {
    /// Source node information.
    pub from: NodeInfo,
    /// Edge kind as string.
    pub edge: String,
    /// Target node information.
    pub to: NodeInfo,
}

/// An enriched trace with full node information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTrace {
    /// Steps with full information.
    pub steps: Vec<EnrichedStep>,
}

impl EnrichedTrace {
    /// Create an enriched trace from a trace and module.
    #[must_use]
    pub fn from_trace(trace: &Trace, module: &AirModule) -> Self {
        let steps = trace
            .steps
            .iter()
            .map(|step| EnrichedStep {
                from: NodeInfo::from_node(step.from, module),
                edge: step.edge.name().to_string(),
                to: NodeInfo::from_node(step.to, module),
            })
            .collect();

        Self { steps }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    use saf_core::ids::{LocId, ValueId};

    #[test]
    fn trace_empty() {
        let trace = Trace::new();
        assert!(trace.is_empty());
        assert_eq!(trace.len(), 0);
    }

    #[test]
    fn trace_push() {
        let mut trace = Trace::new();
        let step = TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        );
        trace.push(step);

        assert!(!trace.is_empty());
        assert_eq!(trace.len(), 1);
    }

    #[test]
    fn trace_with_step() {
        let trace = Trace::new();
        let step = TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        );
        let new_trace = trace.with_step(step.clone());

        assert!(trace.is_empty());
        assert_eq!(new_trace.len(), 1);
        assert_eq!(new_trace.steps[0], step);
    }

    #[test]
    fn trace_from_steps() {
        let steps = vec![
            TraceStep::new(
                NodeId::value(ValueId::new(1)),
                EdgeKind::DefUse,
                NodeId::value(ValueId::new(2)),
            ),
            TraceStep::new(
                NodeId::value(ValueId::new(2)),
                EdgeKind::Store,
                NodeId::location(LocId::new(100)),
            ),
        ];

        let trace = Trace::from_steps(steps);
        assert_eq!(trace.len(), 2);
    }

    #[test]
    fn trace_step_equality() {
        let s1 = TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        );
        let s2 = TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::DefUse,
            NodeId::value(ValueId::new(2)),
        );
        let s3 = TraceStep::new(
            NodeId::value(ValueId::new(1)),
            EdgeKind::Transform,
            NodeId::value(ValueId::new(2)),
        );

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn node_info_value() {
        use saf_core::ids::ModuleId;

        let module = AirModule {
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
        };

        let info = NodeInfo::from_node(NodeId::value(ValueId::new(42)), &module);
        assert_eq!(info.kind, "value");
        assert!(info.symbol.is_none());
    }

    #[test]
    fn node_info_unknown_mem() {
        use saf_core::ids::ModuleId;

        let module = AirModule {
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
        };

        let info = NodeInfo::from_node(NodeId::unknown_mem(), &module);
        assert_eq!(info.kind, "unknown_mem");
        assert_eq!(info.id, "unknown_mem");
    }
}
