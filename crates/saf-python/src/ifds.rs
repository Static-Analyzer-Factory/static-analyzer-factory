//! Python bindings for the IFDS framework.

use std::collections::BTreeSet;
use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::ifds::{IfdsConfig, IfdsResult, TaintFact, TaintIfdsProblem, solve_ifds};
use saf_analysis::selector::Selector;
use saf_core::air::AirModule;

use crate::exceptions::{codes, query_error};
use crate::selector::SelectorOrSet;

/// Result of an IFDS taint analysis.
///
/// Contains the computed taint facts at each program point and
/// solver diagnostics.
#[pyclass(name = "IfdsResult")]
pub struct PyIfdsResult {
    result: IfdsResult<TaintFact>,
    module: Arc<AirModule>,
}

#[pymethods]
impl PyIfdsResult {
    /// Check if any taint reaches any of the given sink values.
    ///
    /// This checks whether any `Tainted(v)` fact exists at any instruction
    /// that uses one of the sink values as an operand.
    ///
    /// Args:
    ///     sinks: Selector or SelectorSet identifying sink values.
    ///
    /// Returns:
    ///     bool: True if taint reaches any sink.
    fn has_taint_at_sink(&self, py: Python<'_>, sinks: SelectorOrSet) -> PyResult<bool> {
        let sink_set = sinks.into_selector_set();
        let mut sink_values = BTreeSet::new();
        for sel in sink_set.inner() {
            match sel.resolve(&self.module) {
                Ok(vals) => sink_values.extend(vals),
                Err(e) => {
                    return Err(query_error(
                        py,
                        codes::INVALID_SELECTOR,
                        &format!("Failed to resolve sink selector: {e}"),
                    ));
                }
            }
        }

        // Check if any tainted fact mentions a sink value.
        for facts in self.result.facts.values() {
            for fact in facts {
                if let TaintFact::Tainted(v) = fact {
                    if sink_values.contains(v) {
                        return Ok(true);
                    }
                }
            }
        }

        // Also check if taint reaches any instruction that uses a sink value.
        for func in &self.module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    // Check if this instruction uses any sink value as operand.
                    let uses_sink = inst.operands.iter().any(|op| sink_values.contains(op));
                    if uses_sink {
                        if let Some(facts) = self.result.facts_at(inst.id) {
                            for fact in facts {
                                if let TaintFact::Tainted(v) = fact {
                                    if inst.operands.contains(v) {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get solver diagnostics as a dictionary.
    ///
    /// Returns:
    ///     dict: Diagnostics with keys: iterations, path_edges_explored,
    ///         summary_edges_created, facts_at_peak, reached_limit.
    fn diagnostics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("iterations", self.result.diagnostics.iterations)?;
        d.set_item(
            "path_edges_explored",
            self.result.diagnostics.path_edges_explored,
        )?;
        d.set_item(
            "summary_edges_created",
            self.result.diagnostics.summary_edges_created,
        )?;
        d.set_item("facts_at_peak", self.result.diagnostics.facts_at_peak)?;
        d.set_item("reached_limit", self.result.diagnostics.reached_limit)?;
        Ok(d)
    }

    /// Export the IFDS result as a dictionary.
    ///
    /// Returns:
    ///     dict: Exportable representation with facts, summaries, diagnostics.
    fn export(&self, py: Python<'_>) -> PyResult<PyObject> {
        let export = self.result.export();
        let dict = crate::helpers::serde_to_py_dict(py, &export)?;
        Ok(dict.into())
    }

    /// Get all tainted values found at any program point.
    ///
    /// Returns:
    ///     list[str]: List of tainted value IDs as hex strings.
    fn tainted_values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let mut tainted = BTreeSet::new();
        for facts in self.result.facts.values() {
            for fact in facts {
                if let TaintFact::Tainted(v) = fact {
                    tainted.insert(v.to_hex());
                }
            }
        }
        let list: Vec<String> = tainted.into_iter().collect();
        PyList::new(py, list)
    }

    fn __repr__(&self) -> String {
        let n_facts: usize = self.result.facts.values().map(BTreeSet::len).sum();
        format!(
            "IfdsResult(facts={}, iterations={})",
            n_facts, self.result.diagnostics.iterations
        )
    }
}

/// Run IFDS taint analysis on a module.
///
/// This is called from Project.ifds_taint().
pub fn run_ifds_taint(
    _py: Python<'_>,
    module: &Arc<AirModule>,
    callgraph: &CallGraph,
    source_selectors: Vec<Selector>,
    sink_selectors: Vec<Selector>,
    sanitizer_selectors: Vec<Selector>,
) -> PyResult<PyIfdsResult> {
    let _ = &sink_selectors; // Sinks checked on result, not in problem.

    let icfg = Icfg::build(module, callgraph);
    let problem = TaintIfdsProblem::new(module, &source_selectors, &sanitizer_selectors);
    let config = IfdsConfig::default();

    let result = solve_ifds(&problem, &icfg, callgraph, &config);

    Ok(PyIfdsResult {
        result,
        module: Arc::clone(module),
    })
}

/// Register IFDS types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyIfdsResult>()?;
    Ok(())
}
