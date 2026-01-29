//! Python bindings for Memory SSA.
//!
//! Provides access to Memory SSA def-use chains, clobber queries,
//! and mod/ref summaries.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use saf_analysis::PtaResult;
use saf_analysis::callgraph::CallGraph;
use saf_analysis::mssa::MemorySsa;
use saf_core::air::AirModule;

use crate::id_parse::{parse_function_id, parse_inst_id, parse_loc_id, parse_mem_access_id};

/// Python wrapper for Memory SSA.
///
/// Provides access to memory def-use chains for address-taken variables.
/// Uses a hybrid approach: location-partitioned skeleton with demand-driven
/// clobber disambiguation.
///
/// Obtain via `Project.memory_ssa()`.
#[pyclass(name = "MemorySsa")]
#[allow(clippy::module_name_repetitions)]
pub struct PyMemorySsa {
    inner: MemorySsa,
}

impl PyMemorySsa {
    /// Build Memory SSA from project internals.
    #[allow(clippy::must_use_candidate)]
    pub fn build(module: &AirModule, callgraph: &CallGraph, pta: PtaResult) -> Self {
        let cfgs = crate::helpers::build_cfgs(module);

        let mssa = MemorySsa::build(module, &cfgs, pta, callgraph);
        Self { inner: mssa }
    }
}

#[pymethods]
impl PyMemorySsa {
    /// Get the total number of memory accesses.
    #[getter]
    fn access_count(&self) -> usize {
        self.inner.access_count()
    }

    /// Get the memory access kind for an instruction.
    ///
    /// Args:
    ///     inst_id: Instruction ID as hex string.
    ///
    /// Returns:
    ///     The access kind ("def", "use", "phi", "live_on_entry"), or None
    ///     if the instruction has no memory access.
    fn access_kind(&self, inst_hex: &str) -> PyResult<Option<String>> {
        let id = parse_inst_id(inst_hex)?;
        Ok(self.inner.access_for(id).map(|a| {
            if a.is_def() {
                "def".to_string()
            } else if a.is_use() {
                "use".to_string()
            } else if a.is_phi() {
                "phi".to_string()
            } else {
                "live_on_entry".to_string()
            }
        }))
    }

    /// Get the memory access ID for an instruction.
    ///
    /// Args:
    ///     inst_id: Instruction ID as hex string.
    ///
    /// Returns:
    ///     The access ID as hex string, or None if no access.
    fn access_id_for(&self, inst_hex: &str) -> PyResult<Option<String>> {
        let id = parse_inst_id(inst_hex)?;
        Ok(self
            .inner
            .access_id_for(id)
            .map(saf_analysis::mssa::MemAccessId::to_hex))
    }

    /// Get the LiveOnEntry access ID for a function.
    ///
    /// Args:
    ///     func_id: Function ID as hex string.
    ///
    /// Returns:
    ///     The LiveOnEntry access ID as hex string, or None.
    fn live_on_entry(&self, func_hex: &str) -> PyResult<Option<String>> {
        let id = parse_function_id(func_hex)?;
        Ok(self
            .inner
            .live_on_entry(id)
            .map(saf_analysis::mssa::MemAccessId::to_hex))
    }

    /// Get the mod/ref summary for a function.
    ///
    /// Args:
    ///     func_id: Function ID as hex string.
    ///
    /// Returns:
    ///     A dict with "may_mod" and "may_ref" lists of location IDs, or None.
    fn mod_ref(&self, py: Python<'_>, func_hex: &str) -> PyResult<Option<Py<PyDict>>> {
        let id = parse_function_id(func_hex)?;
        match self.inner.mod_ref(id) {
            Some(summary) => {
                let dict = PyDict::new(py);
                let may_mod: Vec<String> = summary
                    .may_mod
                    .iter()
                    .map(|l| format!("0x{:032x}", l.raw()))
                    .collect();
                let may_ref: Vec<String> = summary
                    .may_ref
                    .iter()
                    .map(|l| format!("0x{:032x}", l.raw()))
                    .collect();
                dict.set_item("may_mod", may_mod)?;
                dict.set_item("may_ref", may_ref)?;
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Find the clobbering def for a use at a specific location.
    ///
    /// Walks the def chain backward from the Use's reaching def, consulting
    /// PTA to determine if each Def actually clobbers the queried location.
    ///
    /// Args:
    ///     use_id: Use access ID as hex string.
    ///     loc_id: Location ID as hex string.
    ///
    /// Returns:
    ///     The clobbering def access ID as hex string.
    fn clobber_for(&mut self, use_hex: &str, loc_hex: &str) -> PyResult<String> {
        let use_access = parse_mem_access_id(use_hex)?;
        let loc = parse_loc_id(loc_hex)?;
        let result = self.inner.clobber_for(use_access, loc);
        Ok(result.to_hex())
    }

    /// Export the Memory SSA to a dictionary.
    ///
    /// Returns:
    ///     dict: Full Memory SSA export including accesses, phis, and mod/ref.
    fn export(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let export = self.inner.export();
        crate::helpers::serde_to_py_dict(py, &export)
    }

    fn __repr__(&self) -> String {
        format!("MemorySsa(accesses={})", self.inner.access_count())
    }
}

/// Register MSSA types with the module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMemorySsa>()?;
    Ok(())
}
