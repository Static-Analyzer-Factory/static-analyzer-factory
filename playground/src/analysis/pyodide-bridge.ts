/**
 * Pyodide loader and SAF Python bridge for the browser-based analyzer.
 *
 * Lazy-loads Pyodide from CDN, injects SAF analysis results as a Python
 * module, and executes user-written analyzer scripts.
 */

import type { AnalysisResults } from '@saf/web-shared/types';
import type { WasmAnalysisConfig } from '@saf/web-shared/analysis';
import { runAnalysis, runQuery, isDatabaseReady } from '@saf/web-shared/analysis';

// ---------------------------------------------------------------------------
// Pyodide lifecycle
// ---------------------------------------------------------------------------

let pyodideInstance: any = null;

/**
 * Lazy-load and initialize Pyodide. Returns the cached instance on
 * subsequent calls.
 */
export async function initPyodide(): Promise<any> {
  if (pyodideInstance) return pyodideInstance;

  // Dynamically load the Pyodide script from CDN
  const script = document.createElement('script');
  script.src = 'https://cdn.jsdelivr.net/pyodide/v0.27.5/full/pyodide.js';
  document.head.appendChild(script);

  await new Promise<void>((resolve, reject) => {
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load Pyodide'));
  });

  pyodideInstance = await (window as any).loadPyodide();
  return pyodideInstance;
}

/** Check if Pyodide has been initialized. */
export function isPyodideReady(): boolean {
  return pyodideInstance !== null;
}

/** Fire-and-forget pre-warm so the first "Run" is faster. */
export function preloadPyodide(): void {
  initPyodide().catch(() => {});
}

// ---------------------------------------------------------------------------
// SAF Python bridge
// ---------------------------------------------------------------------------

export interface Finding {
  nodeId: string;
  severity: string;
  message: string;
}

/**
 * Register a `_saf_bridge` JS module in Pyodide that exposes the current
 * analysis results, then execute the Python helper that wraps it into
 * a user-friendly `saf` module.
 *
 * When `airJson` and `baseConfig` are provided, the bridge also exposes a
 * `reanalyze(config_json)` function that lets Python scripts re-run the
 * WASM analysis with custom configuration (e.g. different PTA solver).
 */
export async function setupSafBridge(
  pyodide: any,
  results: AnalysisResults | null,
  onReport: (finding: Finding) => void,
  airJson?: string | null,
  baseConfig?: WasmAnalysisConfig,
): Promise<void> {
  // Mutable reference so reanalyze() can update the results in-place
  let currentResults = results;

  pyodide.registerJsModule('_saf_bridge', {
    get_cfg: () => (currentResults?.cfg ? JSON.stringify(currentResults.cfg) : 'null'),
    get_callgraph: () =>
      currentResults?.callgraph ? JSON.stringify(currentResults.callgraph) : 'null',
    get_defuse: () =>
      currentResults?.defuse ? JSON.stringify(currentResults.defuse) : 'null',
    get_valueflow: () =>
      currentResults?.valueflow ? JSON.stringify(currentResults.valueflow) : 'null',
    get_pta: () => (currentResults?.pta ? JSON.stringify(currentResults.pta) : 'null'),
    report: (nodeId: string, severity: string, message: string) => {
      onReport({ nodeId, severity, message });
    },
    // Re-run WASM analysis with custom config overrides (called from Python).
    // Returns "ok" on success or throws on failure.
    reanalyze: (configJson: string) => {
      if (!airJson) {
        throw new Error('No AIR JSON available for reanalysis');
      }
      const overrides = JSON.parse(configJson);
      const mergedConfig: WasmAnalysisConfig = { ...baseConfig, ...overrides };
      currentResults = runAnalysis(airJson, mergedConfig);
      return 'ok';
    },
    // JSON protocol query — delegates to the WASM ProgramDatabase.
    query: (requestJson: string) => {
      if (!isDatabaseReady()) {
        return JSON.stringify({
          status: 'error',
          error: { code: 'NOT_READY', message: 'ProgramDatabase not initialized. Call analyze() first.' },
        });
      }
      try {
        return JSON.stringify(runQuery(requestJson));
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : String(err);
        return JSON.stringify({
          status: 'error',
          error: { code: 'QUERY_FAILED', message },
        });
      }
    },
    is_database_ready: () => isDatabaseReady(),
  });

  await pyodide.runPythonAsync(SAF_PYTHON_MODULE);
}

// ---------------------------------------------------------------------------
// Python `saf` helper module (injected as a string)
// ---------------------------------------------------------------------------

const SAF_PYTHON_MODULE = `
import sys, types, json as _json

# Force re-import so registerJsModule updates take effect
if '_saf_bridge' in sys.modules:
    del sys.modules['_saf_bridge']
import _saf_bridge

# Create a proper 'saf' module so user scripts can 'import saf'
_saf_mod = types.ModuleType('saf')
_saf_mod.__doc__ = "SAF browser-side analysis bridge."
sys.modules['saf'] = _saf_mod

# ---------------------------------------------------------------------------
# Existing PropertyGraph wrappers (backward compatible)
# ---------------------------------------------------------------------------

class PgNode:
    """A node in a PropertyGraph."""
    def __init__(self, data: dict):
        self.id: str = data.get("id", "")
        self.labels: list[str] = data.get("labels", [])
        self.properties: dict = data.get("properties", {})

    def __repr__(self):
        return f"PgNode({self.id}, labels={self.labels})"

class PgEdge:
    """An edge in a PropertyGraph."""
    def __init__(self, data: dict):
        self.src: str = data.get("src", "")
        self.dst: str = data.get("dst", "")
        self.edge_type: str = data.get("edge_type", "")
        self.properties: dict = data.get("properties", {})

    def __repr__(self):
        return f"PgEdge({self.src} -> {self.dst}, type={self.edge_type})"

class PropertyGraph:
    """Wrapper around SAF's PropertyGraph JSON export."""
    def __init__(self, raw: str | None):
        if raw is None or raw == "null":
            self.nodes: list[PgNode] = []
            self.edges: list[PgEdge] = []
            self.metadata: dict = {}
            self.graph_type: str = "unknown"
            return
        data = _json.loads(raw)
        self.graph_type = data.get("graph_type", "unknown")
        self.metadata = data.get("metadata", {})
        self.nodes = [PgNode(n) for n in data.get("nodes", [])]
        self.edges = [PgEdge(e) for e in data.get("edges", [])]

    def find_nodes(self, **kwargs) -> list[PgNode]:
        """Find nodes matching all given property filters."""
        result = []
        for node in self.nodes:
            match = True
            for key, val in kwargs.items():
                if key == "label":
                    if val not in node.labels:
                        match = False
                elif node.properties.get(key) != val:
                    match = False
            if match:
                result.append(node)
        return result

    def successors(self, node_id: str) -> list[PgNode]:
        """Return nodes reachable via one outgoing edge."""
        dst_ids = [e.dst for e in self.edges if e.src == node_id]
        return [n for n in self.nodes if n.id in dst_ids]

    def predecessors(self, node_id: str) -> list[PgNode]:
        """Return nodes with an edge leading to node_id."""
        src_ids = [e.src for e in self.edges if e.dst == node_id]
        return [n for n in self.nodes if n.id in src_ids]

class AnalysisResult:
    """Container holding all available analysis graphs."""
    def __init__(self):
        self.cfg = PropertyGraph(_saf_bridge.get_cfg())
        self.callgraph = PropertyGraph(_saf_bridge.get_callgraph())
        self.defuse = PropertyGraph(_saf_bridge.get_defuse())
        self.valueflow = PropertyGraph(_saf_bridge.get_valueflow())
        # PTA has a different format
        _pta_raw = _saf_bridge.get_pta()
        self.pta = _json.loads(_pta_raw) if _pta_raw and _pta_raw != "null" else None

# Module-level singleton
_result: AnalysisResult | None = None
_findings: list[dict] = []

def analyze(*, pta_solver: str | None = None, **kwargs) -> AnalysisResult:
    """Load analysis results, optionally re-running with custom config.

    Args:
        pta_solver: PTA solver -- "worklist" (default) or "datalog".
        **kwargs: Additional config overrides forwarded to the WASM analyzer.

    When called with no arguments, loads the pre-computed results.
    When called with overrides (e.g. pta_solver="datalog"), re-runs the
    WASM analysis with the specified configuration.
    """
    global _result
    overrides = {}
    if pta_solver is not None:
        overrides["pta_solver"] = pta_solver
    overrides.update(kwargs)

    if overrides:
        _saf_bridge.reanalyze(_json.dumps(overrides))

    _result = AnalysisResult()
    return _result

def report(node_id: str, severity: str, message: str):
    """Report a finding. Severity should be 'info', 'low', 'medium', 'high', or 'critical'."""
    _saf_bridge.report(node_id, severity, message)
    _findings.append({"node_id": node_id, "severity": severity, "message": message})

def source_line(node_id: str) -> int | None:
    """Look up the C source line number for a node ID from its span property.

    Searches all graphs (cfg, callgraph, defuse, valueflow) for a node
    matching the given ID and returns the line_start from its span, or
    None if not available.
    """
    if _result is None:
        return None
    for graph in [_result.cfg, _result.callgraph, _result.defuse, _result.valueflow]:
        for node in graph.nodes:
            if node.id == node_id:
                span = node.properties.get("span")
                if span and isinstance(span, dict):
                    line = span.get("line_start")
                    if line is not None:
                        return int(line)
    return None

# ---------------------------------------------------------------------------
# New SDK-compatible classes
# ---------------------------------------------------------------------------

class Selector:
    """Stores a query selector kind and its parameters."""
    def __init__(self, kind: str, **kwargs):
        self.kind = kind
        self.params = kwargs
    def to_dict(self):
        d = {"kind": self.kind}
        d.update(self.params)
        return d
    def __repr__(self):
        return f"Selector(kind={self.kind}, {self.params})"

class TraceStep:
    """A single step in a finding trace."""
    def __init__(self, location: str = "", event: str = "", state: str | None = None, **_extra):
        self.location = location
        self.event = event
        self.state = state
        self.display_name = _extra.get("display_name")
        self.source_loc = _extra.get("source_loc")
    def __repr__(self):
        return f"{self.location}: {self.event}"

class Trace:
    """A sequence of trace steps describing a finding path."""
    def __init__(self, steps: list):
        self.steps = [TraceStep(**s) if isinstance(s, dict) else s for s in steps]
    def pretty(self) -> str:
        lines = []
        for i, step in enumerate(self.steps):
            prefix = "-> " if i > 0 else "  "
            lines.append(f"{prefix}{step.location}: {step.event}")
        return "\\n".join(lines)
    def __repr__(self):
        return f"Trace({len(self.steps)} steps)"

class Finding:
    """A finding from a query result."""
    def __init__(self, data: dict):
        self.check = data.get("check", "")
        self.severity = data.get("severity", "info")
        self.cwe = data.get("cwe")
        self.message = data.get("message", "")
        self.object = data.get("object")
        self.display_name = data.get("display_name")
        path = data.get("path", [])
        self.trace = Trace(path)
        # Convenience accessors
        self.source_location = path[0].get("location", "") if path else ""
        self.sink_location = path[-1].get("location", "") if path else ""
        self.source_id = data.get("source_id", "")
        self.sink_id = data.get("sink_id", "")
    def __repr__(self):
        return f"Finding({self.check}: {self.message})"

class PtaResult:
    """Wraps pointer-analysis data for convenient querying."""
    def __init__(self, pta_data):
        self._data = pta_data
    def points_to(self, pointer_id: str) -> list[str]:
        if not self._data:
            return []
        for entry in self._data.get("points_to", []):
            if entry.get("value") == pointer_id:
                return entry.get("locations", [])
        return []
    def all_entries(self) -> list[dict]:
        if not self._data:
            return []
        return self._data.get("points_to", [])
    def may_alias(self, p: str, q: str) -> bool:
        p_locs = set(self.points_to(p))
        q_locs = set(self.points_to(q))
        return bool(p_locs & q_locs)
    def __repr__(self):
        count = len(self._data.get("points_to", [])) if self._data else 0
        return f"PtaResult({count} entries)"

class GraphStore:
    """Unified graph access from an AnalysisResult."""
    def __init__(self, analysis_result):
        self._result = analysis_result
    def available(self) -> list[str]:
        graphs = []
        if self._result.cfg.nodes: graphs.append("cfg")
        if self._result.callgraph.nodes: graphs.append("callgraph")
        if self._result.defuse.nodes: graphs.append("defuse")
        if self._result.valueflow.nodes: graphs.append("valueflow")
        return graphs
    def get(self, name: str) -> PropertyGraph | None:
        return getattr(self._result, name, None)
    def export(self, name: str, function: str | None = None) -> PropertyGraph | None:
        graph = self.get(name)
        if graph is None:
            return None
        if function and name == "cfg":
            # Filter to nodes for this function
            nodes = [n for n in graph.nodes if n.properties.get("function") == function]
            node_ids = {n.id for n in nodes}
            edges = [e for e in graph.edges if e.src in node_ids or e.dst in node_ids]
            filtered = PropertyGraph.__new__(PropertyGraph)
            filtered.nodes = nodes
            filtered.edges = edges
            filtered.metadata = graph.metadata
            filtered.graph_type = graph.graph_type
            return filtered
        return graph

class Query:
    """Query methods via JSON protocol to the WASM ProgramDatabase."""
    def __init__(self, project):
        self._project = project

    def check(self, checker_name: str) -> list:
        """Run a built-in checker by name (e.g., 'null-deref', 'use-after-free')."""
        request = {"action": "check", "name": checker_name}
        result_json = _saf_bridge.query(_json.dumps(request))
        resp = _json.loads(result_json)
        return self._parse_findings(resp)

    def check_all(self) -> list:
        """Run all built-in checkers."""
        request = {"action": "check_all"}
        result_json = _saf_bridge.query(_json.dumps(request))
        resp = _json.loads(result_json)
        return self._parse_findings(resp)

    def taint_flow(self, sources, sinks, sanitizers=None) -> list:
        src_list = sources if isinstance(sources, list) else [sources]
        snk_list = sinks if isinstance(sinks, list) else [sinks]
        san_list = sanitizers if sanitizers else []
        if isinstance(san_list, Selector):
            san_list = [san_list]

        params = {
            "sources": [s.to_dict() for s in src_list],
            "sinks": [s.to_dict() for s in snk_list],
        }
        if san_list:
            params["sanitizers"] = [s.to_dict() for s in san_list]

        resp = self._query("taint_flow", params)
        return self._parse_findings(resp)

    def flows(self, sources, sinks) -> list:
        src_list = sources if isinstance(sources, list) else [sources]
        snk_list = sinks if isinstance(sinks, list) else [sinks]
        params = {
            "sources": [s.to_dict() for s in src_list],
            "sinks": [s.to_dict() for s in snk_list],
        }
        resp = self._query("flows", params)
        return self._parse_findings(resp)

    def points_to(self, pointer_id: str) -> list[str]:
        resp = self._query("points_to", {"pointer": pointer_id})
        if resp.get("status") == "ok" and resp.get("results"):
            locs = []
            for r in resp["results"]:
                locs.extend(r.get("locations", []))
            return locs
        return []

    def may_alias(self, p: str, q: str) -> bool:
        resp = self._query("alias", {"p": p, "q": q})
        if resp.get("status") == "ok" and resp.get("results"):
            return resp["results"][0].get("may_alias", False)
        return False

    def _query(self, query_type: str, params: dict) -> dict:
        request = {"action": "query", "type": query_type, "params": params}
        result_json = _saf_bridge.query(_json.dumps(request))
        return _json.loads(result_json)

    def _parse_findings(self, resp: dict) -> list:
        if resp.get("status") != "ok":
            err = resp.get("error", {})
            raise RuntimeError(f"Query failed: {err.get('message', 'unknown error')}")
        findings_data = resp.get("findings", [])
        return [Finding(f) for f in findings_data]

class Project:
    """SDK-compatible entry point for SAF analysis in the browser."""
    def __init__(self):
        self._result = analyze()

    def query(self) -> Query:
        return Query(self)

    def graphs(self) -> GraphStore:
        return GraphStore(self._result)

    def pta_result(self) -> PtaResult:
        return PtaResult(self._result.pta)

    def call_graph(self):
        return self._result.callgraph

    def value_flow(self):
        return self._result.valueflow

    # Unsupported SDK features
    @staticmethod
    def open(path=None, **kwargs):
        raise NotImplementedError("Project.open() requires native SAF SDK -- use Project() in browser")

    def ifds_taint(self, *a, **kw):
        raise NotImplementedError("IFDS requires native SAF SDK")

    def typestate(self, *a, **kw):
        raise NotImplementedError("Typestate requires native SAF SDK")

# ---------------------------------------------------------------------------
# Selector factory functions (mirroring Python SDK)
# ---------------------------------------------------------------------------

def function_param(function: str, index: int = None) -> Selector:
    kwargs = {"function": function}
    if index is not None:
        kwargs["index"] = index
    return Selector("function_param", **kwargs)

def function_return(function: str) -> Selector:
    return Selector("function_return", function=function)

def call(callee: str) -> Selector:
    return Selector("call_to", callee=callee)

def arg_to(callee: str, index: int = None) -> Selector:
    kwargs = {"callee": callee}
    if index is not None:
        kwargs["index"] = index
    return Selector("arg_to", **kwargs)

def global_var(name: str) -> Selector:
    return Selector("global", name=name)

# ---------------------------------------------------------------------------
# Expose public API on the saf module
# ---------------------------------------------------------------------------

# Existing (backward compat)
_saf_mod.PgNode = PgNode
_saf_mod.PgEdge = PgEdge
_saf_mod.PropertyGraph = PropertyGraph
_saf_mod.AnalysisResult = AnalysisResult
_saf_mod.analyze = analyze
_saf_mod.report = report
_saf_mod.source_line = source_line

# New SDK-compatible API
_saf_mod.Project = Project
_saf_mod.Query = Query
_saf_mod.Selector = Selector
_saf_mod.Finding = Finding
_saf_mod.Trace = Trace
_saf_mod.TraceStep = TraceStep
_saf_mod.GraphStore = GraphStore
_saf_mod.PtaResult = PtaResult

# Factory functions
_saf_mod.function_param = function_param
_saf_mod.function_return = function_return
_saf_mod.call = call
_saf_mod.arg_to = arg_to
_saf_mod.global_var = global_var
`;

// ---------------------------------------------------------------------------
// User script execution
// ---------------------------------------------------------------------------

export interface ScriptResult {
  stdout: string;
  error: string | null;
}

/**
 * Execute user-written Python code in Pyodide, capturing stdout and errors.
 */
export async function runUserScript(
  pyodide: any,
  code: string,
): Promise<ScriptResult> {
  const outputLines: string[] = [];

  pyodide.setStdout({
    batched: (line: string) => {
      outputLines.push(line);
    },
  });

  try {
    await pyodide.runPythonAsync(code);
    return { stdout: outputLines.join('\n'), error: null };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      stdout: outputLines.join('\n'),
      error: message,
    };
  }
}
