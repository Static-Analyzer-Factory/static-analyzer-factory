//! Python bindings for Finding and Trace types.

use pyo3::prelude::*;
use saf_analysis::display::DisplayResolver;
use saf_analysis::{EnrichedTrace, Finding as RustFinding, NodeInfo, SpanInfo, Trace as RustTrace};
use saf_core::air::AirModule;

/// A step in a taint trace.
#[pyclass(name = "TraceStep")]
#[derive(Clone)]
pub struct PyTraceStep {
    /// Node ID of the source.
    #[pyo3(get)]
    pub from_id: String,
    /// Kind of the source node.
    #[pyo3(get)]
    pub from_kind: String,
    /// Symbol at the source (if available).
    #[pyo3(get)]
    pub from_symbol: Option<String>,
    /// Location of the source (if available).
    #[pyo3(get)]
    pub from_location: Option<String>,
    /// Edge kind (def_use, transform, store, load, etc).
    #[pyo3(get)]
    pub edge: String,
    /// Node ID of the target.
    #[pyo3(get)]
    pub to_id: String,
    /// Kind of the target node.
    #[pyo3(get)]
    pub to_kind: String,
    /// Symbol at the target (if available).
    #[pyo3(get)]
    pub to_symbol: Option<String>,
    /// Location of the target (if available).
    #[pyo3(get)]
    pub to_location: Option<String>,
}

impl PyTraceStep {
    fn from_enriched(from: &NodeInfo, edge: &str, to: &NodeInfo) -> Self {
        Self {
            from_id: from.id.clone(),
            from_kind: from.kind.clone(),
            from_symbol: from.symbol.clone(),
            from_location: format_span(from.span.as_ref()),
            edge: edge.to_string(),
            to_id: to.id.clone(),
            to_kind: to.kind.clone(),
            to_symbol: to.symbol.clone(),
            to_location: format_span(to.span.as_ref()),
        }
    }

    /// Create from enriched node info, using a `DisplayResolver` to fill
    /// missing `from_symbol`/`to_symbol` gaps.
    fn from_enriched_with_resolver(
        from: &NodeInfo,
        edge: &str,
        to: &NodeInfo,
        resolver: &DisplayResolver<'_>,
    ) -> Self {
        let from_symbol = from
            .symbol
            .clone()
            .or_else(|| parse_hex_id(&from.id).map(|id| resolver.resolve(id).short_name));
        let to_symbol = to
            .symbol
            .clone()
            .or_else(|| parse_hex_id(&to.id).map(|id| resolver.resolve(id).short_name));

        Self {
            from_id: from.id.clone(),
            from_kind: from.kind.clone(),
            from_symbol,
            from_location: format_span(from.span.as_ref()),
            edge: edge.to_string(),
            to_id: to.id.clone(),
            to_kind: to.kind.clone(),
            to_symbol,
            to_location: format_span(to.span.as_ref()),
        }
    }
}

#[pymethods]
impl PyTraceStep {
    fn __repr__(&self) -> String {
        let from_name = self.from_symbol.as_deref().unwrap_or(&self.from_id);
        let to_name = self.to_symbol.as_deref().unwrap_or(&self.to_id);
        format!("TraceStep({} --{}-> {})", from_name, self.edge, to_name)
    }
}

/// A trace from source to sink through the value flow graph.
#[pyclass(name = "Trace")]
#[derive(Clone)]
pub struct PyTrace {
    /// Steps in the trace (from source to sink).
    #[pyo3(get)]
    pub steps: Vec<PyTraceStep>,
}

impl PyTrace {
    /// Create from a Rust Trace and module.
    pub fn from_trace(trace: &RustTrace, module: &AirModule) -> Self {
        let enriched = trace.enrich(module);
        Self::from_enriched(&enriched)
    }

    /// Create from an enriched trace.
    pub fn from_enriched(enriched: &EnrichedTrace) -> Self {
        let steps = enriched
            .steps
            .iter()
            .map(|step| PyTraceStep::from_enriched(&step.from, &step.edge, &step.to))
            .collect();
        Self { steps }
    }

    /// Create from a Rust Trace and module, using a `DisplayResolver` to fill
    /// missing symbol gaps in trace steps.
    pub fn from_trace_with_resolver(
        trace: &RustTrace,
        module: &AirModule,
        resolver: &DisplayResolver<'_>,
    ) -> Self {
        let enriched = trace.enrich(module);
        Self::from_enriched_with_resolver(&enriched, resolver)
    }

    /// Create from an enriched trace, using a `DisplayResolver` to fill
    /// missing symbol gaps.
    pub fn from_enriched_with_resolver(
        enriched: &EnrichedTrace,
        resolver: &DisplayResolver<'_>,
    ) -> Self {
        let steps = enriched
            .steps
            .iter()
            .map(|step| {
                PyTraceStep::from_enriched_with_resolver(&step.from, &step.edge, &step.to, resolver)
            })
            .collect();
        Self { steps }
    }
}

#[pymethods]
impl PyTrace {
    /// Return a human-readable representation of the trace.
    fn pretty(&self) -> String {
        let mut lines = Vec::new();
        for (i, step) in self.steps.iter().enumerate() {
            let from_name = step.from_symbol.as_deref().unwrap_or(&step.from_id);
            let from_loc = step.from_location.as_deref().unwrap_or("?");
            let to_name = step.to_symbol.as_deref().unwrap_or(&step.to_id);
            let to_loc = step.to_location.as_deref().unwrap_or("?");

            if i == 0 {
                lines.push(format!("[source] {from_name} @ {from_loc}"));
            }
            lines.push(format!("  --{}-> {to_name} @ {to_loc}", step.edge));
        }
        if !self.steps.is_empty() {
            let last = self.steps.last().unwrap();
            let sink_name = last.to_symbol.as_deref().unwrap_or(&last.to_id);
            let sink_loc = last.to_location.as_deref().unwrap_or("?");
            lines.push(format!("[sink] {sink_name} @ {sink_loc}"));
        }
        lines.join("\n")
    }

    fn __repr__(&self) -> String {
        format!("Trace({} steps)", self.steps.len())
    }

    fn __len__(&self) -> usize {
        self.steps.len()
    }
}

/// A finding from taint analysis.
#[pyclass(name = "Finding")]
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PyFinding {
    /// Unique, deterministic finding ID.
    #[pyo3(get)]
    pub finding_id: String,
    /// Source location (file:line:col or value ID).
    #[pyo3(get)]
    pub source_location: String,
    /// Sink location (file:line:col or value ID).
    #[pyo3(get)]
    pub sink_location: String,
    /// Source value ID.
    #[pyo3(get)]
    pub source_id: String,
    /// Sink value ID.
    #[pyo3(get)]
    pub sink_id: String,
    /// Human-readable source name (resolved via `DisplayResolver`).
    #[pyo3(get)]
    pub source_name: Option<String>,
    /// Human-readable sink name (resolved via `DisplayResolver`).
    #[pyo3(get)]
    pub sink_name: Option<String>,
    /// Optional rule ID.
    #[pyo3(get)]
    pub rule_id: Option<String>,
    /// Trace from source to sink.
    #[pyo3(get)]
    pub trace: PyTrace,
}

impl PyFinding {
    /// Create from a Rust Finding and module (without display resolution).
    pub fn from_finding(finding: &RustFinding, module: &AirModule) -> Self {
        let trace = PyTrace::from_trace(&finding.trace, module);

        // Try to get source/sink locations from the trace
        let source_location = trace
            .steps
            .first()
            .and_then(|s| s.from_location.clone())
            .unwrap_or_else(|| finding.source.to_hex());

        let sink_location = trace
            .steps
            .last()
            .and_then(|s| s.to_location.clone())
            .unwrap_or_else(|| finding.sink.to_hex());

        Self {
            finding_id: finding.id.to_hex(),
            source_location,
            sink_location,
            source_id: finding.source.to_hex(),
            sink_id: finding.sink.to_hex(),
            source_name: None,
            sink_name: None,
            rule_id: finding.rule_id.clone(),
            trace,
        }
    }

    /// Create from a Rust Finding and module, enriching with a `DisplayResolver`.
    ///
    /// Resolves `source_name`/`sink_name` from the display resolver and fills
    /// missing `from_symbol`/`to_symbol` gaps in trace steps.
    pub fn from_finding_with_resolver(
        finding: &RustFinding,
        module: &AirModule,
        resolver: &DisplayResolver<'_>,
    ) -> Self {
        let trace = PyTrace::from_trace_with_resolver(&finding.trace, module, resolver);

        // Resolve source/sink names via DisplayResolver
        let source_label = resolver.resolve(finding.source.raw());
        let sink_label = resolver.resolve(finding.sink.raw());

        // Try to get source/sink locations from the trace
        let source_location = trace
            .steps
            .first()
            .and_then(|s| s.from_location.clone())
            .unwrap_or_else(|| finding.source.to_hex());

        let sink_location = trace
            .steps
            .last()
            .and_then(|s| s.to_location.clone())
            .unwrap_or_else(|| finding.sink.to_hex());

        Self {
            finding_id: finding.id.to_hex(),
            source_location,
            sink_location,
            source_id: finding.source.to_hex(),
            sink_id: finding.sink.to_hex(),
            source_name: Some(source_label.short_name),
            sink_name: Some(sink_label.short_name),
            rule_id: finding.rule_id.clone(),
            trace,
        }
    }
}

#[pymethods]
impl PyFinding {
    /// Convert the finding to a dictionary.
    ///
    /// Returns:
    ///     dict: Finding data as a dictionary.
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("finding_id", &self.finding_id)?;
        dict.set_item("source_location", &self.source_location)?;
        dict.set_item("sink_location", &self.sink_location)?;
        dict.set_item("source_id", &self.source_id)?;
        dict.set_item("sink_id", &self.sink_id)?;
        dict.set_item("source_name", &self.source_name)?;
        dict.set_item("sink_name", &self.sink_name)?;
        dict.set_item("rule_id", &self.rule_id)?;

        // Convert trace steps to list of dicts
        let steps_list = pyo3::types::PyList::empty(py);
        for step in &self.trace.steps {
            let step_dict = pyo3::types::PyDict::new(py);
            step_dict.set_item("from_id", &step.from_id)?;
            step_dict.set_item("from_kind", &step.from_kind)?;
            step_dict.set_item("from_symbol", &step.from_symbol)?;
            step_dict.set_item("from_location", &step.from_location)?;
            step_dict.set_item("edge", &step.edge)?;
            step_dict.set_item("to_id", &step.to_id)?;
            step_dict.set_item("to_kind", &step.to_kind)?;
            step_dict.set_item("to_symbol", &step.to_symbol)?;
            step_dict.set_item("to_location", &step.to_location)?;
            steps_list.append(step_dict)?;
        }
        dict.set_item("trace", steps_list)?;

        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "Finding(id={}, {} -> {})",
            &self.finding_id[..10],
            self.source_location,
            self.sink_location
        )
    }
}

/// Format a span as a location string.
fn format_span(span: Option<&SpanInfo>) -> Option<String> {
    span.map(|s| format!("{}:{}:{}", s.file, s.start_line, s.start_col))
}

/// Parse a hex string (with "0x" prefix) into a `u128`.
///
/// Returns `None` on parse failure (instead of panicking).
fn parse_hex_id(hex_str: &str) -> Option<u128> {
    let hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u128::from_str_radix(hex, 16).ok()
}

/// Register finding types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTraceStep>()?;
    m.add_class::<PyTrace>()?;
    m.add_class::<PyFinding>()?;
    Ok(())
}
