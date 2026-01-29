//! Python bindings for the Static Analyzer Factory.
//!
//! This crate provides PyO3 bindings exposing SAF's static analysis capabilities
//! through a layered Python API designed for both AI agents and human developers.

// PyO3-specific clippy allows — these are inherent to how PyO3 macros work
// and cannot be avoided without significant API changes.
// - unnecessary_wraps: PyO3 #[pyfunction] often requires PyResult return type
// - needless_pass_by_value: PyO3 requires owned types for Python conversion
// - unused_self: PyO3 #[pymethods] requires &self parameter even for static methods
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
// Documentation lints — kept at crate level, tracked as future work in FUTURE.md
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// Structural/style lints — narrowed to function/struct/module level where needed:
// - too_many_arguments: added to specific functions in checkers.rs, z3_refine.rs, ide.rs
// - too_many_lines: added to specific functions where needed
// - module_name_repetitions: PyO3 Py* naming convention, added to submodules
// - struct_excessive_bools: added to specific structs (absint::PyInterval)
// - similar_names: added to specific functions with p/q, mssa/mssa2 patterns
// - must_use_candidate: added to submodules with public helper functions

mod absint;
mod air;
mod cg_refinement;
mod checkers;
mod combined;
mod cspta;
mod dda;
mod display;
mod exceptions;
mod finding;
mod fspta;
mod graphs;
mod helpers;
mod id_parse;
mod ide;
mod ifds;
mod mssa;
mod project;
mod pta;
mod query;
mod schema;
mod selector;
mod spec;
mod svfg;
mod z3_refine;

use pyo3::prelude::*;

/// Returns the SAF version string.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Create a function_param selector.
///
/// Selects function parameters by function name pattern.
///
/// Args:
///     function: Function name pattern (glob-style, e.g., "main", "process_*").
///     index: Parameter index (0-based), or None for all parameters.
///
/// Returns:
///     A Selector object.
#[pyfunction]
#[pyo3(signature = (function, index=None))]
fn function_param(function: &str, index: Option<u32>) -> selector::PySelector {
    selector::PySelector::new(saf_analysis::selector::Selector::function_param(
        function, index,
    ))
}

/// Create a function_return selector.
///
/// Selects function return values by function name pattern.
///
/// Args:
///     function: Function name pattern (glob-style).
///
/// Returns:
///     A Selector object.
#[pyfunction]
fn function_return(function: &str) -> selector::PySelector {
    selector::PySelector::new(saf_analysis::selector::Selector::function_return(function))
}

/// Create a call selector.
///
/// Selects call results to specific functions.
///
/// Args:
///     callee: Callee function name pattern (glob-style or regex).
///
/// Returns:
///     A Selector object.
#[pyfunction]
fn call(callee: &str) -> selector::PySelector {
    selector::PySelector::new(saf_analysis::selector::Selector::call_to(callee))
}

/// Create an arg_to selector.
///
/// Selects arguments passed to specific functions.
///
/// Args:
///     callee: Callee function name pattern.
///     index: Argument index (0-based), or None for all arguments.
///
/// Returns:
///     A Selector object.
#[pyfunction]
#[pyo3(signature = (callee, index=None))]
fn arg_to(callee: &str, index: Option<u32>) -> selector::PySelector {
    selector::PySelector::new(saf_analysis::selector::Selector::arg_to(callee, index))
}

/// Python module for the Static Analyzer Factory.
#[pymodule]
fn _saf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core functions
    m.add_function(wrap_pyfunction!(version, m)?)?;

    // Selector factory functions (for direct import)
    m.add_function(wrap_pyfunction!(function_param, m)?)?;
    m.add_function(wrap_pyfunction!(function_return, m)?)?;
    m.add_function(wrap_pyfunction!(call, m)?)?;
    m.add_function(wrap_pyfunction!(arg_to, m)?)?;

    // Register classes from submodules
    project::register(m)?;
    query::register(m)?;
    finding::register(m)?;
    display::register(m)?;
    selector::register(m)?;
    exceptions::register(m)?;
    air::register(m)?;
    graphs::register(m)?;
    pta::register(m)?;
    ide::register(m)?;
    ifds::register(m)?;
    cg_refinement::register(m)?;
    mssa::register(m)?;
    svfg::register(m)?;
    fspta::register(m)?;
    cspta::register(m)?;
    dda::register(m)?;
    checkers::register(m)?;
    absint::register(m)?;
    z3_refine::register(m)?;
    combined::register(m)?;
    spec::register(m)?;

    Ok(())
}
