# Python SDK

The SAF Python SDK (`import saf`) provides full access to SAF's static analysis
capabilities from Python. It is built with PyO3 and installed via `maturin`.

## Installation

The SDK is built automatically when entering the Docker environment:

```bash
make shell
# SDK is available immediately:
python3 -c "import saf; print(saf.version())"
```

For manual installation inside the dev container:

```bash
maturin develop --release
```

## Core API

### Project

The `Project` class is the main entry point for all analysis operations.

```python
from saf import Project

# Open a project from LLVM IR
proj = Project.open("program.ll")

# Open from AIR-JSON
proj = Project.open("program.air.json")

# Open with analysis tuning parameters
proj = Project.open(
    "program.ll",
    vf_mode="precise",                # "fast" (default) or "precise"
    pta_solver="worklist",             # "worklist" (default) or "datalog"
    pta_max_iterations=20000,          # default: 10000
    field_sensitivity_depth=3,         # default: 2 (0 = disabled)
    max_refinement_iterations=5,       # default: 10
)
```

**`Project.open()` signature:**

```python
Project.open(
    path: str,
    *,
    vf_mode: str = "fast",
    pta_solver: str = "worklist",
    pta_max_iterations: int | None = None,
    field_sensitivity_depth: int | None = None,
    max_refinement_iterations: int | None = None,
) -> Project
```

| Parameter | Description |
|-----------|-------------|
| `path` | Path to input file (`.air.json`, `.ll`, or `.bc`). Frontend is selected automatically by extension. |
| `vf_mode` | `"fast"` routes all memory through a single unknown node for robust taint analysis. `"precise"` uses points-to analysis to resolve memory locations (may miss flows through unresolved pointers). |
| `pta_solver` | `"worklist"` uses the imperative worklist-based solver. `"datalog"` uses the Ascent Datalog fixpoint solver. |
| `pta_max_iterations` | Maximum PTA solver iterations. Default: 10000. |
| `field_sensitivity_depth` | Field sensitivity depth. 0 = disabled, default: 2. Higher values track deeper nested struct fields. |
| `max_refinement_iterations` | Maximum CG refinement iterations. Default: 10. |

**Raises:** `FrontendError` if the input file cannot be parsed or the required frontend is not available.

### Schema Discovery

```python
schema = proj.schema()
# Returns a dict with structured information about:
# - tool_version, schema_version
# - frontends (air-json, llvm with extensions and descriptions)
# - graphs (cfg, callgraph, defuse, valueflow)
# - queries (taint_flow, flows, points_to, may_alias with parameters)
# - selectors (sources, sinks, sanitizers)
```

### Query Context

```python
from saf import sources, sinks, sanitizers

q = proj.query()

# Taint flow analysis
findings = q.taint_flow(
    sources=sources.function_param("main", 1),
    sinks=sinks.call("system", arg_index=0),
)

# With sanitizers (accepts a Selector or SelectorSet, not strings)
findings = q.taint_flow(
    sources=sources.function_param("main", 1),
    sinks=sinks.call("system", arg_index=0),
    sanitizers=sanitizers.call("validate_input"),
    limit=500,  # default: 1000
)

# Data flows (without sanitizer filtering)
findings = q.flows(
    sources=sources.function_param("read_data"),
    sinks=sinks.arg_to("write_output", 0),
    limit=100,
)

# Points-to query (takes hex value ID string)
pts = q.points_to("0x00000000000000000000000000000001")

# Alias query (takes hex value ID strings)
alias = q.may_alias("0x00000001", "0x00000002")
```

**Query method signatures:**

| Method | Parameters | Returns |
|--------|-----------|---------|
| `taint_flow(sources, sinks, sanitizers=None, *, limit=1000)` | `Selector`/`SelectorSet` for each | `list[Finding]` |
| `flows(sources, sinks, *, limit=1000)` | `Selector`/`SelectorSet` for each | `list[Finding]` |
| `points_to(ptr)` | Hex string value ID | `list[str]` (location IDs) |
| `may_alias(p, q)` | Hex string value IDs | `bool` |

### Graph Export

```python
graphs = proj.graphs()

# List available graph types
print(graphs.available())  # ["cfg", "callgraph", "defuse", "valueflow"]

# Export to PropertyGraph dict
cfg = graphs.export("cfg")
cfg_main = graphs.export("cfg", function="main")  # single function
cg = graphs.export("callgraph")
du = graphs.export("defuse")
vf = graphs.export("valueflow")

# Export to Graphviz DOT string
dot_str = graphs.to_dot("callgraph")

# Export to interactive HTML (Cytoscape.js)
html_str = graphs.to_html("cfg", function="main")
```

All `export()` calls return a unified PropertyGraph dict:

```python
{
    "schema_version": "0.1.0",
    "graph_type": "callgraph",
    "metadata": {},
    "nodes": [{"id": "0x...", "labels": [...], "properties": {...}}, ...],
    "edges": [{"src": "0x...", "dst": "0x...", "edge_type": "...", "properties": {}}, ...],
}
```

## Source and Sink Selectors

Selectors identify values in the program for taint analysis. They can be
combined using the `|` operator to form a `SelectorSet`.

### Sources (`saf.sources`)

| Function | Description |
|----------|-------------|
| `function_param(function, index=None)` | Select function parameters by name pattern (glob-style). `index` is 0-based; `None` selects all parameters. |
| `function_return(function)` | Select function return values by name pattern. |
| `call(callee)` | Select return values of calls to a function. |
| `argv()` | Select command-line arguments (shortcut for `function_param("main", None)`). |
| `getenv(name=None)` | Select environment variable reads (shortcut for `call("getenv")`). |

```python
from saf import sources

src = sources.function_param("main", 1)
src = sources.function_param("read_*")      # glob pattern
src = sources.function_return("get_input")
src = sources.call("getenv")
src = sources.argv()

# Combine with |
combined = sources.argv() | sources.getenv()
```

### Sinks (`saf.sinks`)

| Function | Description |
|----------|-------------|
| `call(callee, *, arg_index=None)` | Select calls to a function. If `arg_index` is given, selects that argument; otherwise selects the call result. |
| `arg_to(callee, index)` | Select arguments passed to a function (0-based index). |

```python
from saf import sinks

sink = sinks.call("system", arg_index=0)
sink = sinks.call("printf", arg_index=0)
sink = sinks.arg_to("free", 0)

# Without arg_index, selects the call result
sink = sinks.call("dangerous_function")
```

### Sanitizers (`saf.sanitizers`)

| Function | Description |
|----------|-------------|
| `call(callee, *, arg_index=None)` | Select calls to a sanitizing function. If `arg_index` is given, selects that argument; otherwise the return value is considered sanitized. |
| `arg_to(callee, index)` | Select arguments passed to a sanitizing function. |

```python
from saf import sanitizers

san = sanitizers.call("escape_html", arg_index=0)
san = sanitizers.call("sanitize_input")

# Combine sanitizers
combined = sanitizers.call("sanitize") | sanitizers.call("escape")
```

### Module-Level Selector Factories

The following factory functions are also available directly from `saf._saf`
(used internally by `sources`, `sinks`, and `sanitizers` modules):

- `function_param(function, index=None)` -- Select function parameters.
- `function_return(function)` -- Select function return values.
- `call(callee)` -- Select call results.
- `arg_to(callee, index=None)` -- Select arguments to a callee.

## Checker Framework

### Running Built-In Checkers

```python
# Run a specific checker
findings = proj.check("memory-leak")

# Run multiple checkers at once (pass a list)
findings = proj.check(["memory-leak", "use-after-free", "double-free"])

# Run all 9 built-in checkers
findings = proj.check_all()

# List available checkers and their metadata
schema = proj.checker_schema()
for checker in schema["checkers"]:
    print(f"{checker['name']}: {checker['description']} (CWE-{checker['cwe']})")
```

### Built-In Checker Table

| Checker Name | CWE | Description |
|-------------|-----|-------------|
| `memory-leak` | 401 | Allocated memory never freed |
| `use-after-free` | 416 | Memory accessed after being freed |
| `double-free` | 415 | Memory freed more than once |
| `null-deref` | 476 | Null pointer dereference |
| `file-descriptor-leak` | 403 | Opened file never closed |
| `uninit-use` | 457 | Use of uninitialized memory |
| `stack-escape` | 562 | Returning stack address |
| `lock-not-released` | 764 | Mutex not unlocked |
| `generic-resource-leak` | N/A | Custom resource tracking |

### Custom Checkers

```python
# Define a custom checker with source/sink/sanitizer roles
findings = proj.check_custom(
    "my-custom-leak",
    mode="must_not_reach",        # "may_reach", "must_not_reach", or "never_reach_sink"
    source_role="allocator",       # resource role for sources
    source_match_return=True,      # match return value (True) or first arg (False)
    sink_is_exit=True,             # sinks are function exits
    sink_role=None,                # or a resource role string
    sanitizer_role="deallocator",  # or None
    sanitizer_match_return=False,
    cwe=401,                       # optional CWE ID
    severity="warning",            # "info", "warning", "error", "critical"
)
```

### Path-Sensitive Checking (Z3)

```python
# Run checkers with Z3-based path feasibility filtering
result = proj.check_path_sensitive("null-deref", z3_timeout_ms=2000, max_guards=64)

# Or run all checkers with path sensitivity
result = proj.check_all_path_sensitive(z3_timeout_ms=1000, max_guards=64)

# Result has feasible, infeasible, and unknown findings
print(f"Real bugs: {len(result.feasible)}")
print(f"False positives filtered: {len(result.infeasible)}")
print(f"Unknown: {len(result.unknown)}")
print(result.diagnostics)  # dict with Z3 statistics

# Post-filter existing findings
raw_findings = proj.check_all()
result = proj.filter_infeasible(raw_findings, z3_timeout_ms=1000, max_guards=64)
```

### `CheckerFinding` Attributes

Each item returned by `check()`, `check_all()`, or `check_custom()` is a
`CheckerFinding`:

| Attribute | Type | Description |
|-----------|------|-------------|
| `checker` | `str` | Checker name that produced this finding |
| `severity` | `str` | `"info"`, `"warning"`, `"error"`, or `"critical"` |
| `cwe` | `int \| None` | CWE ID if applicable |
| `message` | `str` | Human-readable description |
| `source` | `str` | Source SVFG node hex ID |
| `sink` | `str` | Sink SVFG node hex ID |
| `trace` | `list[str]` | Path from source to sink as hex node IDs |
| `sink_traces` | `list[dict]` | Per-sink traces for multi-reach findings (e.g., double-free). Each dict has `"sink"` and `"trace"` keys. |

```python
for f in proj.check("use-after-free"):
    print(f.checker, f.severity, f.message)
    print(f"  CWE-{f.cwe}: {f.source} -> {f.sink}")
    print(f"  Trace length: {len(f.trace)}")
    # Convert to dict
    d = f.to_dict()
```

## Finding Objects

The `taint_flow()` and `flows()` query methods return `Finding` objects, which
are distinct from `CheckerFinding` objects.

### `Finding` Attributes

| Attribute | Type | Description |
|-----------|------|-------------|
| `finding_id` | `str` | Deterministic hex identifier |
| `source_location` | `str` | Source location (file:line:col or value ID) |
| `sink_location` | `str` | Sink location (file:line:col or value ID) |
| `source_id` | `str` | Source value ID (hex) |
| `sink_id` | `str` | Sink value ID (hex) |
| `rule_id` | `str \| None` | Optional rule identifier |
| `trace` | `Trace` | Step-by-step data flow path |

```python
for f in q.taint_flow(sources.argv(), sinks.call("system", arg_index=0)):
    print(f"{f.source_location} -> {f.sink_location}")
    print(f.trace.pretty())       # human-readable trace
    print(f"Steps: {len(f.trace)}")
    d = f.to_dict()               # convert to dict
```

### Trace and TraceStep

A `Trace` contains a list of `TraceStep` objects. Each step represents one
hop in the value-flow graph:

| TraceStep Attribute | Type | Description |
|---------------------|------|-------------|
| `from_id` | `str` | Source node ID |
| `from_kind` | `str` | Source node kind |
| `from_symbol` | `str \| None` | Symbol name at source |
| `from_location` | `str \| None` | Source file:line:col |
| `edge` | `str` | Edge kind (def_use, transform, store, load, etc.) |
| `to_id` | `str` | Target node ID |
| `to_kind` | `str` | Target node kind |
| `to_symbol` | `str \| None` | Symbol name at target |
| `to_location` | `str \| None` | Target file:line:col |

```python
for step in finding.trace.steps:
    print(f"  {step.from_symbol or step.from_id} --{step.edge}-> "
          f"{step.to_symbol or step.to_id}")
```

## Resource Table

The resource table maps function names to resource management roles. It ships
with built-in entries for C stdlib, C++ operators, POSIX I/O, and pthreads.

```python
table = proj.resource_table()

table.has_role("malloc", "allocator")    # True
table.has_role("free", "deallocator")    # True
table.has_role("fopen", "acquire")       # True
table.has_role("fclose", "release")      # True

# Add custom entries
table.add("my_alloc", "allocator")
table.add("my_free", "deallocator")

# Inspect
print(table.size)                 # number of entries
print(table.function_names())     # sorted list of function names
entries = table.export()          # list of {"name": ..., "roles": [...]}
```

**Available roles:** `allocator`, `deallocator`, `reallocator`, `acquire`,
`release`, `lock`, `unlock`, `null_source`, `dereference`.

## Advanced Analysis

### IFDS Taint Analysis

Precise interprocedural taint tracking using the IFDS framework
(Reps/Horwitz/Sagiv tabulation algorithm):

```python
result = proj.ifds_taint(
    sources=sources.function_param("main", 0),
    sinks=sinks.call("system", arg_index=0),
    sanitizers=sanitizers.call("validate"),  # optional
)
```

### Typestate Analysis

Track per-resource state machines using the IDE framework:

```python
# Built-in specs: "file_io", "mutex_lock", "memory_alloc"
result = proj.typestate("file_io")

# Custom typestate spec
from saf import TypestateSpec
result = proj.typestate_custom(spec)
```

### Flow-Sensitive Pointer Analysis

More precise than Andersen's flow-insensitive analysis for programs with
pointer reassignment:

```python
fs_result = proj.flow_sensitive_pta(pts_repr="auto")
# pts_repr options: "auto", "btreeset", "bitvector", "bdd"
```

### Context-Sensitive Pointer Analysis (k-CFA)

Distinguishes calls to the same function from different call sites:

```python
cs_result = proj.context_sensitive_pta(k=1, pts_repr="auto")
```

### Demand-Driven Pointer Analysis

Computes points-to information only for explicitly queried pointers:

```python
dda = proj.demand_pta(
    max_steps=100_000,
    max_context_depth=10,
    timeout_ms=5000,
    enable_strong_updates=True,
    pts_repr="auto",
)
```

### Memory SSA and SVFG

```python
mssa = proj.memory_ssa()      # Memory SSA representation
svfg = proj.svfg()            # Sparse Value-Flow Graph
```

### Call Graph Refinement

Iterative CHA + PTA-based indirect call resolution:

```python
result = proj.refine_call_graph(entry_points="all", max_iterations=10)
```

### Abstract Interpretation

Numeric interval analysis with widening/narrowing:

```python
result = proj.abstract_interp(
    max_widening=100,
    narrowing_iterations=3,
    use_thresholds=True,
)
```

### Numeric Checkers

```python
# Individual numeric checker
findings = proj.check_numeric("buffer_overflow")     # CWE-120
findings = proj.check_numeric("integer_overflow")    # CWE-190
findings = proj.check_numeric("division_by_zero")    # CWE-369
findings = proj.check_numeric("shift_count")         # CWE-682

# All numeric checkers at once
findings = proj.check_all_numeric()
```

### Combined PTA + Abstract Interpretation

Alias-aware numeric analysis with bidirectional refinement:

```python
result = proj.analyze_combined(
    enable_refinement=True,
    max_refinement_iterations=3,
)
interval = result.interval_at("0x1234...")
alias = result.may_alias("0x5678...", "0x9abc...")
```

### Z3 Path Refinement

```python
# Prove/disprove assertions
result = proj.prove_assertions(z3_timeout_ms=1000, max_guards=64)

# Refine alias query with path constraints
result = proj.refine_alias("0xP", "0xQ", at_block="0xB", func_id="0xF")

# Check if a feasible path exists between two blocks
result = proj.check_path_reachable(
    from_block="0xB1", to_block="0xB2", func_id="0xF",
    z3_timeout_ms=1000, max_guards=64, max_paths=100,
)
```

### JSON Protocol (LLM Agent Interface)

```python
import json
resp = proj.request('{"action": "schema"}')
data = json.loads(resp)
```

## AIR Module Access

The `AirModule` provides mid-level access to the intermediate representation:

```python
air = proj.air()

print(air.name)              # module name
print(air.id)                # module ID (hex)
print(air.function_count)    # number of functions
print(air.global_count)      # number of globals
print(air.function_names())  # list of function names
print(air.global_names())    # list of global names
```

## Visualization

The `saf.viz` module provides dependency-free graph visualization:

```python
from saf import viz

pg = proj.graphs().export("callgraph")

# Graphviz DOT string (no dependencies)
dot_str = viz.to_dot(pg)

# Interactive HTML with Cytoscape.js (no dependencies)
html_str = viz.to_html(pg)

# Open in browser or save to file
viz.visualize(pg)                              # opens in browser
viz.visualize(pg, output="callgraph.html")     # saves to file

# Cytoscape.js JSON (for ipycytoscape in Jupyter)
cy_json = viz.to_cytoscape_json(pg)

# NetworkX DiGraph (requires networkx)
G = viz.to_networkx(pg)

# graphviz.Digraph object (requires graphviz package)
gv = viz.to_graphviz(pg)
gv.render("graph", format="svg")
```

## Exceptions

All SAF exceptions inherit from `SafError` and carry `.code` and `.details`
attributes for structured error handling:

| Exception | Description |
|-----------|-------------|
| `SafError` | Base exception for all SAF errors |
| `FrontendError` | Frontend ingestion errors (parsing, I/O, unsupported features) |
| `AnalysisError` | Analysis errors (PTA timeout, ValueFlow build error) |
| `QueryError` | Query execution errors (invalid selector, no match) |
| `ConfigError` | Configuration errors (invalid field, incompatible options) |

```python
from saf import Project, SafError, FrontendError

try:
    proj = Project.open("nonexistent.ll")
except FrontendError as e:
    print(f"Error: {e}")
```

## Module Exports

The `saf` package exports the following names:

```python
from saf import (
    # Core classes
    Project, Query, Finding, Trace, TraceStep,
    # Selectors
    Selector, SelectorSet,
    # Selector modules
    sources, sinks, sanitizers, viz,
    # Checker types
    CheckerFinding, PathSensitiveResult, ResourceTable,
    # Typestate types
    TypestateResult, TypestateFinding, TypestateSpec, typestate_specs,
    # Exceptions
    SafError, FrontendError, AnalysisError, QueryError, ConfigError,
    # Resource role constants
    Allocator, Deallocator, Reallocator, Acquire, Release,
    NullSource, Dereference,
    # Reachability mode constants
    MayReach, MustNotReach,
    # Severity constants
    Info, Warning, Error, Critical,
    # Functions
    version,
)
```
