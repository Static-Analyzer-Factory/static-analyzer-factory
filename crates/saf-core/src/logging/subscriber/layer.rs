//! `SafLogLayer` — a `tracing` Layer that formats and emits SAF debug log lines.
//!
//! Listens for tracing events with target `"saf_debug"`, extracts the structured
//! fields (`saf_module`, `saf_phase`, `saf_tag`, `saf_narrative`, `saf_kv`),
//! checks them against the `SafLogFilter`, and writes formatted DSL lines.

use std::io::Write;
use std::sync::Mutex;

use tracing::Event;
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use super::filter::SafLogFilter;
use crate::logging::formatter::format_saf_log_line;

/// A tracing `Layer` that emits SAF structured debug log lines.
pub struct SafLogLayer {
    filter: SafLogFilter,
    writer: Mutex<Box<dyn Write + Send>>,
}

impl SafLogLayer {
    /// Create a new `SafLogLayer` with the given filter and output writer.
    pub fn new(filter: SafLogFilter, writer: Box<dyn Write + Send>) -> Self {
        Self {
            filter,
            writer: Mutex::new(writer),
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for SafLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Only process events with our target
        if event.metadata().target() != "saf_debug" {
            return;
        }

        // Skip if filter is not active
        if !self.filter.is_active() {
            return;
        }

        // Extract fields
        let mut visitor = SafLogVisitor::default();
        event.record(&mut visitor);

        let module = visitor.module.as_deref().unwrap_or("");
        let phase = visitor.phase.as_deref().unwrap_or("");
        let tag = visitor.tag.as_deref().unwrap_or("");

        // Check filter
        if !self.filter.matches(module, phase, tag) {
            return;
        }

        let narrative = visitor.narrative.as_deref().unwrap_or("");
        let kv = visitor.kv.as_deref().unwrap_or("");

        let line = format_saf_log_line(module, phase, tag, narrative, kv);

        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{line}");
        }
    }
}

/// Visitor that extracts SAF-specific fields from tracing events.
#[derive(Default)]
struct SafLogVisitor {
    module: Option<String>,
    phase: Option<String>,
    tag: Option<String>,
    narrative: Option<String>,
    kv: Option<String>,
}

impl Visit for SafLogVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "saf_module" => self.module = Some(value.to_string()),
            "saf_phase" => self.phase = Some(value.to_string()),
            "saf_tag" => self.tag = Some(value.to_string()),
            "saf_narrative" => self.narrative = Some(value.to_string()),
            "saf_kv" => self.kv = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        // Fallback for fields recorded as Debug
        let s = format!("{value:?}");
        self.record_str(field, &s);
    }
}
