//! Schema builder for the Python SDK.
//!
//! Provides the `schema()` function that returns a dictionary describing
//! all available analysis capabilities for AI agent discoverability.

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Build the schema dictionary describing all SAF capabilities.
///
/// This provides AI agents with structured information about:
/// - Available frontends and their capabilities
/// - Graph types that can be exported
/// - Query methods and their parameters
/// - Selector types for sources, sinks, and sanitizers
/// - Configuration options
// NOTE: Schema builder populates a flat dictionary with all analysis
// metadata in a single pass — splitting would fragment the schema definition.
#[allow(clippy::too_many_lines)]
pub fn build_schema(py: Python<'_>) -> PyResult<Py<PyDict>> {
    let schema = PyDict::new(py);

    // Tool metadata
    schema.set_item("tool_version", env!("CARGO_PKG_VERSION"))?;
    schema.set_item("schema_version", 1)?;

    // Frontends
    let frontends = PyDict::new(py);

    let air_json = PyDict::new(py);
    air_json.set_item("description", "AIR JSON intermediate representation")?;
    air_json.set_item("extensions", vec![".air.json"])?;
    air_json.set_item("example", "Project.open('program.air.json')")?;
    frontends.set_item("air-json", air_json)?;

    let llvm = PyDict::new(py);
    llvm.set_item("description", "LLVM bitcode (requires llvm feature)")?;
    llvm.set_item("extensions", vec![".bc", ".ll"])?;
    llvm.set_item("example", "Project.open('program.bc', frontend='llvm')")?;
    frontends.set_item("llvm", llvm)?;

    schema.set_item("frontends", frontends)?;

    // Graphs
    let graphs = PyDict::new(py);

    let cfg = PyDict::new(py);
    cfg.set_item("description", "Control flow graph (per-function)")?;
    cfg.set_item("nodes", "basic blocks")?;
    cfg.set_item("edges", "control flow transitions")?;
    graphs.set_item("cfg", cfg)?;

    let callgraph = PyDict::new(py);
    callgraph.set_item("description", "Inter-procedural call graph")?;
    callgraph.set_item("nodes", "functions")?;
    callgraph.set_item("edges", "call relationships")?;
    graphs.set_item("callgraph", callgraph)?;

    let defuse = PyDict::new(py);
    defuse.set_item("description", "Definition-use chains")?;
    defuse.set_item("nodes", "values")?;
    defuse.set_item("edges", "def-use relationships")?;
    graphs.set_item("defuse", defuse)?;

    let valueflow = PyDict::new(py);
    valueflow.set_item("description", "Value flow graph for taint analysis")?;
    valueflow.set_item("nodes", "values and memory locations")?;
    valueflow.set_item(
        "edges",
        "data flow (DefUse, Transform, Store, Load, CallArg, Return)",
    )?;
    graphs.set_item("valueflow", valueflow)?;

    schema.set_item("graphs", graphs)?;

    // Queries
    let queries = PyDict::new(py);

    let taint_flow = PyDict::new(py);
    taint_flow.set_item("description", "Find taint flows from sources to sinks")?;
    let taint_params = PyDict::new(py);
    taint_params.set_item("sources", "Selector or SelectorSet for taint sources")?;
    taint_params.set_item("sinks", "Selector or SelectorSet for taint sinks")?;
    taint_params.set_item(
        "sanitizers",
        "Optional Selector or SelectorSet for sanitizers",
    )?;
    taint_params.set_item("limit", "Maximum findings to return (default 1000)")?;
    taint_flow.set_item("parameters", taint_params)?;
    taint_flow.set_item("returns", "List[Finding]")?;
    taint_flow.set_item(
        "example",
        "q.taint_flow(sources.function_param('main', 0), sinks.call('dangerous', arg_index=0))",
    )?;
    queries.set_item("taint_flow", taint_flow)?;

    let flows = PyDict::new(py);
    flows.set_item(
        "description",
        "Find data flows from sources to sinks (without sanitizers)",
    )?;
    let flows_params = PyDict::new(py);
    flows_params.set_item("sources", "Selector or SelectorSet for sources")?;
    flows_params.set_item("sinks", "Selector or SelectorSet for sinks")?;
    flows_params.set_item("limit", "Maximum findings to return (default 1000)")?;
    flows.set_item("parameters", flows_params)?;
    flows.set_item("returns", "List[Finding]")?;
    queries.set_item("flows", flows)?;

    let points_to = PyDict::new(py);
    points_to.set_item("description", "Get points-to set for a pointer value")?;
    let pts_params = PyDict::new(py);
    pts_params.set_item("ptr", "Value ID (hex string)")?;
    points_to.set_item("parameters", pts_params)?;
    points_to.set_item("returns", "List[str] - location IDs")?;
    points_to.set_item("example", "q.points_to('0x0000000000000001')")?;
    queries.set_item("points_to", points_to)?;

    let may_alias = PyDict::new(py);
    may_alias.set_item("description", "Check if two pointers may alias")?;
    let alias_params = PyDict::new(py);
    alias_params.set_item("p", "First pointer value ID")?;
    alias_params.set_item("q", "Second pointer value ID")?;
    may_alias.set_item("parameters", alias_params)?;
    may_alias.set_item("returns", "bool")?;
    queries.set_item("may_alias", may_alias)?;

    schema.set_item("queries", queries)?;

    // Selectors
    let selectors = PyDict::new(py);

    // Source selectors
    let sources = PyDict::new(py);

    let func_param = PyDict::new(py);
    func_param.set_item("description", "Select function parameters by name pattern")?;
    func_param.set_item("example", "sources.function_param('main', 0)")?;
    sources.set_item("function_param", func_param)?;

    let func_return = PyDict::new(py);
    func_return.set_item("description", "Select function return values")?;
    func_return.set_item("example", "sources.function_return('get_*')")?;
    sources.set_item("function_return", func_return)?;

    let call_src = PyDict::new(py);
    call_src.set_item("description", "Select return values of calls to a function")?;
    call_src.set_item("example", "sources.call('getenv')")?;
    sources.set_item("call", call_src)?;

    let argv = PyDict::new(py);
    argv.set_item("description", "Select command-line arguments")?;
    argv.set_item("example", "sources.argv()")?;
    sources.set_item("argv", argv)?;

    let getenv = PyDict::new(py);
    getenv.set_item("description", "Select environment variable reads")?;
    getenv.set_item("example", "sources.getenv()")?;
    sources.set_item("getenv", getenv)?;

    selectors.set_item("sources", sources)?;

    // Sink selectors
    let sinks = PyDict::new(py);

    let call_sink = PyDict::new(py);
    call_sink.set_item(
        "description",
        "Select calls to a function (optionally specific argument)",
    )?;
    call_sink.set_item("example", "sinks.call('printf', arg_index=0)")?;
    sinks.set_item("call", call_sink)?;

    let arg_to = PyDict::new(py);
    arg_to.set_item("description", "Select arguments passed to a function")?;
    arg_to.set_item("example", "sinks.arg_to('free', 0)")?;
    sinks.set_item("arg_to", arg_to)?;

    selectors.set_item("sinks", sinks)?;

    // Sanitizer selectors
    let sanitizers = PyDict::new(py);

    let san_call = PyDict::new(py);
    san_call.set_item("description", "Select calls to sanitizing functions")?;
    san_call.set_item("example", "sanitizers.call('escape_html', arg_index=0)")?;
    sanitizers.set_item("call", san_call)?;

    let san_arg_to = PyDict::new(py);
    san_arg_to.set_item("description", "Select arguments to sanitizing functions")?;
    san_arg_to.set_item("example", "sanitizers.arg_to('sanitize', 0)")?;
    sanitizers.set_item("arg_to", san_arg_to)?;

    selectors.set_item("sanitizers", sanitizers)?;

    schema.set_item("selectors", selectors)?;

    // Configuration
    let config = PyDict::new(py);

    let frontend_cfg = PyDict::new(py);
    frontend_cfg.set_item("type", "str")?;
    frontend_cfg.set_item("default", "llvm")?;
    frontend_cfg.set_item("choices", vec!["llvm", "air-json"])?;
    frontend_cfg.set_item("description", "Frontend to use for parsing input files")?;
    config.set_item("frontend", frontend_cfg)?;

    let cache_dir_cfg = PyDict::new(py);
    cache_dir_cfg.set_item("type", "str | None")?;
    cache_dir_cfg.set_item("default", py.None())?;
    cache_dir_cfg.set_item("description", "Directory for caching analysis results")?;
    config.set_item("cache_dir", cache_dir_cfg)?;

    schema.set_item("config", config)?;

    Ok(schema.into())
}
