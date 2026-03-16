# Plan 087: Unified Property Graph Export + Visualization

## Context

SAF has 16+ graph types (CFG, CallGraph, DefUse, ValueFlow, SVFG, MSSA, PTA, CSPTA, FSPTA, DDA, IFDS, IDE, AbsInt, MTA, CHA, ICFG) but zero visualization support. All exports are JSON with graph-specific shapes that vary per type. Users cannot visualize graphs without manual conversion. Future goal: store graphs in Neo4j.

**Goal**: Replace the fragmented per-graph JSON export with a **unified property graph format** that maps directly to the property graph model (Neo4j-compatible). Add DOT export in Rust, interactive Cytoscape.js HTML export, and a Python `saf.viz` module for rendering (DOT, NetworkX, Jupyter inline via ipycytoscape). This replaces the current export entirely (no backward compat — no external consumers).

**Why property graph**: The property graph model (labeled nodes with properties, typed directed edges with properties) is the universal interchange format. Neo4j, TigerGraph, Memgraph, Cytoscape.js, and NetworkX all use it. One format serves all destinations.

## Phase 1: Define `PropertyGraph` format (Rust)

### New file: `crates/saf-analysis/src/export.rs`

```rust
/// Unified property graph export format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyGraph {
    pub schema_version: String,      // "0.1.0"
    pub graph_type: String,          // "callgraph", "cfg", "valueflow", etc.
    pub metadata: BTreeMap<String, serde_json::Value>,  // graph-level stats, diagnostics
    pub nodes: Vec<PgNode>,
    pub edges: Vec<PgEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgNode {
    pub id: String,                  // hex ID
    pub labels: Vec<String>,         // ["Function"], ["Block", "Entry"], etc.
    pub properties: BTreeMap<String, serde_json::Value>,  // name, kind, function, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgEdge {
    pub src: String,
    pub dst: String,
    pub edge_type: String,           // "CALLS", "FLOWS_TO", "POINTS_TO", etc.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, serde_json::Value>,
}
```

### Methods on `PropertyGraph`

```rust
impl PropertyGraph {
    /// Generate Graphviz DOT format string.
    pub fn to_dot(&self) -> String { ... }

    /// Generate standalone interactive HTML using Cytoscape.js.
    /// Embeds the graph data as JSON and Cytoscape.js from CDN.
    /// Includes: zoom/pan, dagre layout (hierarchical), search, tooltips.
    pub fn to_html(&self) -> String { ... }
}
```

### `to_dot()` implementation
- Generates `digraph { ... }` with node labels from `properties["name"]` or first label
- Edge labels from `edge_type`
- Node shapes based on labels (e.g., `Function` → box, `Block` → rectangle, `MemPhi` → diamond)
- Subgraph clustering by `properties["function"]` when present (groups CFG blocks by function)

### `to_html()` implementation
- Generates a self-contained HTML file with:
  - Cytoscape.js loaded from CDN (`https://unpkg.com/cytoscape`, `cytoscape-dagre`, `cytoscape-cola`)
  - Graph data embedded as inline JSON
  - Layout selector: dagre (default for CFG/CallGraph), cola (default for ValueFlow/SVFG), cose (force)
  - Node styling: shape and color by label type
  - Edge styling: color and arrow by edge_type
  - Toolbar: layout switch, zoom-to-fit, search by name, filter by edge type
  - Hover tooltips showing all node/edge properties
  - Click to highlight neighbors
- The HTML template is a Rust `const &str` or `include_str!()` from a `.html` template file

### Files to create/modify
- **Create**: `crates/saf-analysis/src/export.rs`
- **Create**: `crates/saf-analysis/src/export_html_template.html` (Cytoscape.js HTML template)
- **Modify**: `crates/saf-analysis/src/lib.rs` — add `pub mod export;`
- **Modify**: `crates/saf-analysis/Cargo.toml` — ensure `serde_json` is a dependency (likely already is)

## Phase 2: Convert each graph's `export()` to return `PropertyGraph`

Replace per-graph export structs (`CallGraphExport`, `CfgExport`, `DefUseExport`, `SvfgExport`, etc.) with implementations that return `PropertyGraph`.

### Graph-specific mappings

| Graph | Node Labels | Edge Type | Key Properties |
|-------|-------------|-----------|----------------|
| CallGraph | `Function`, `External`, `Indirect` | `CALLS` | `name`, `kind` |
| CFG | `Block` + optional `Entry`/`Exit` | `FLOWS_TO` | `function`, `label` |
| ICFG | `Block` | `FLOWS_TO`, `CALL`, `RETURN` | `function` |
| DefUse | `Value` | `DEFINED_BY`, `USED_BY` | `operation` |
| ValueFlow | `Value`, `Location`, `UnknownMem` | `DIRECT`, `INDIRECT`, `CALL`, `RETURN` | `kind` |
| SVFG | `Value`, `MemPhi` | `DIRECT_DEF`, `INDIRECT_STORE`, `INDIRECT_LOAD`, etc. | `kind` |
| PTA | `Pointer`, `Location` | `POINTS_TO` | location details |
| MSSA | `MemAccess`, `MemPhi`, `LiveOnEntry` | `DEF`, `USE` | `function`, `access_kind` |
| IFDS | `Instruction`, `Fact` | `FLOW`, `SUMMARY` | `function`, fact details |
| AbsInt | `Instruction` | `FLOW` | abstract state |
| MTA | `Thread`, `Lock` | `HAPPENS_BEFORE`, `MAY_PARALLEL` | thread details |
| CHA | `Type` | `INHERITS` | `name` |

### Files to modify (one per graph type)
- `crates/saf-analysis/src/callgraph.rs` — replace `CallGraphExport` with `PropertyGraph`
- `crates/saf-analysis/src/cfg.rs` — replace `CfgExport` with `PropertyGraph`
- `crates/saf-analysis/src/icfg.rs` — replace `IcfgExport`
- `crates/saf-analysis/src/defuse.rs` — replace `DefUseExport`
- `crates/saf-analysis/src/valueflow/export.rs` — replace with `PropertyGraph`
- `crates/saf-analysis/src/svfg/export.rs` — replace `SvfgExport`
- `crates/saf-analysis/src/pta/export.rs` — replace `PtaExport`
- `crates/saf-analysis/src/cspta/export.rs` — replace `CsptaExport`
- `crates/saf-analysis/src/fspta/export.rs` — replace `FsptaExport`
- `crates/saf-analysis/src/mssa/export.rs` — replace `MssaExport`
- `crates/saf-analysis/src/ifds/export.rs` — replace `IfdsExport`
- `crates/saf-analysis/src/absint/export.rs` — replace `AbsintExport`
- `crates/saf-analysis/src/dda/solver.rs` — replace `DdaExport`
- `crates/saf-analysis/src/mta/export.rs` — replace `MtaExport`
- `crates/saf-analysis/src/cha.rs` — replace `ChaExport`

**Priority**: Start with the 4 graphs currently exposed in Python (callgraph, cfg, defuse, valueflow), then do the rest.

## Phase 3: Update Python bindings

### Modify `crates/saf-python/src/graphs.rs`

- `PyGraphStore::export(name)` returns a dict with the unified PG shape for all graph types
- `PyGraphStore::available()` expands to include all graph types (not just 4)
- Add `PyGraphStore::to_dot(name)` → returns DOT string directly (calls Rust `PropertyGraph::to_dot()`)
- Add `PyGraphStore::to_html(name)` → returns HTML string (calls Rust `PropertyGraph::to_html()`)

### Expose more graphs
Currently `PyGraphStore` only holds: `module`, `callgraph`, `defuse`, `valueflow`.

**Scoping decision**: For this plan, convert the 4 existing graphs to PG format. Exposing additional graphs (SVFG, MSSA, etc.) that require extra analysis steps is a follow-up.

## Phase 4: Python `saf.viz` module

### New file: `python/saf/viz.py` (pure Python, no Rust dependency for viz logic)

```python
def to_dot(pg: dict) -> str:
    """Convert PropertyGraph dict to Graphviz DOT string. No deps needed."""

def to_networkx(pg: dict) -> "nx.DiGraph":
    """Convert PropertyGraph dict to NetworkX DiGraph with node/edge attributes.
    Requires: pip install networkx"""

def to_graphviz(pg: dict) -> "graphviz.Digraph":
    """Convert PropertyGraph dict to graphviz.Digraph object for rendering.
    Requires: pip install graphviz"""

def to_html(pg: dict, layout: str = "auto") -> str:
    """Generate standalone interactive HTML with Cytoscape.js. No deps needed.
    Layout auto-selects: dagre for CFG/CallGraph, cola for ValueFlow/SVFG."""

def visualize(pg: dict, output: str = None, layout: str = "auto") -> None:
    """Quick visualization — generates HTML and opens in browser or saves to file."""

def to_cytoscape_json(pg: dict) -> dict:
    """Convert to Cytoscape.js JSON for ipycytoscape Jupyter widget.
    Usage in notebook:
        import ipycytoscape
        w = ipycytoscape.CytoscapeWidget()
        w.graph.add_graph_from_json(to_cytoscape_json(pg))
        display(w)
    """

def to_neo4j(pg: dict, driver) -> None:
    """Load PropertyGraph into Neo4j. Requires: pip install neo4j
    Future — not implemented in this plan."""
```

### Optional dependencies (in pyproject.toml extras)
```toml
[project.optional-dependencies]
viz = ["graphviz>=0.20", "networkx>=3.0"]
notebook = ["ipycytoscape>=1.3"]
```

### Dependency-free functions
- `to_dot()` — pure string generation
- `to_html()` — pure string generation (Cytoscape.js loaded from CDN in the HTML)
- `to_cytoscape_json()` — pure dict transformation
- `visualize()` — calls `to_html()` + opens browser (uses `webbrowser` stdlib)

### Functions requiring optional deps
- `to_networkx()` — requires `networkx`
- `to_graphviz()` — requires `graphviz` Python package + system Graphviz binary

### Note on Rust vs Python `to_dot()`/`to_html()`
Both Rust (`PropertyGraph::to_dot()`) and Python (`saf.viz.to_dot()`) provide the same functionality. The Rust version is faster and available via `graphs.to_dot("callgraph")` in Python bindings. The Python version works on any PG dict (e.g., loaded from a JSON file) without needing the SAF Rust library. Users can use either.

## Phase 5: Update tests

### Rust tests
- Update all integration tests in `crates/saf-analysis/tests/` to use the new `PropertyGraph` shape
- Regenerate all `insta` snapshots: `cargo insta review`
- Update determinism tests (serialize `PropertyGraph` instead of per-graph structs)
- Add unit tests for `to_dot()` and `to_html()` output

### Python tests
- Update `python/tests/test_acceptance.py` — assert on `"nodes"`, `"edges"`, `"graph_type"` in PG format
- Update `python/tests/e2e/test_*.py` files
- Add tests for `saf.viz` module functions

### Key test files
- `crates/saf-analysis/tests/graph_integration.rs`
- `crates/saf-analysis/tests/*_e2e.rs` (14+ files)
- `python/tests/test_acceptance.py`
- `python/tests/e2e/test_oop_e2e.py`
- `python/tests/e2e/test_memory_e2e.py`

## Phase 6: Update tutorials + docs

### Tutorial scripts (16 files)
All `detect.py` files that call `graphs.export()` need to parse the new PG shape. The conversion is mechanical:
- Old: `cg["nodes"][i]["name"]` → New: `cg["nodes"][i]["properties"]["name"]`
- Old: `cg["edges"][i]["src"]` → New: `cg["edges"][i]["src"]` (same)
- Old: `cfg["functions"][name]["blocks"]` → New: iterate `cfg["nodes"]` with label filter

### Tutorial enhancement
Add visualization examples to tutorials using the new capabilities:
```python
# Quick interactive visualization
from saf.viz import visualize
visualize(graphs.export("callgraph"), output="callgraph.html")

# DOT export
dot_str = graphs.to_dot("callgraph")
with open("callgraph.dot", "w") as f:
    f.write(dot_str)
```

### Key tutorial files
- `tutorials/integration/02-json-export/detect.py`
- `tutorials/getting-started/02-call-graph-cfg/detect.py`
- `tutorials/getting-started/03-defuse-valueflow/detect.py`
- `tutorials/advanced-techniques/01-pointer-aliasing/detect.py`
- `tutorials/buffer-overflow/01-taint-detection/detect.py`
- + 11 more

### Documentation
- Update `CLAUDE.md` — replace per-graph JSON shape documentation with PG format
- Update tutorial READMEs that reference export shapes

## Phase 7: Jupyter support via `make notebook`

### Modify `Makefile`
Add a `notebook` target:
```makefile
notebook:  ## Start Jupyter Lab with port forwarding (no rebuild)
	docker compose run --rm -p 8888:8888 dev sh -c \
		'pip install jupyterlab ipycytoscape && jupyter lab --ip=0.0.0.0 --allow-root --no-browser'
```

### Modify `docker-compose.yml`
No changes needed — `docker compose run --rm -p 8888:8888` handles port forwarding at runtime.

### Output convention
- Tutorials and scripts write visualization files to `output/` directory (gitignored)
- Add `output/` to `.gitignore` if not already present
- Files written in Docker at `/workspace/output/` appear on host at `<project>/output/` via bind mount

## Implementation Order

1. **Phase 1**: Define `PropertyGraph` struct + `to_dot()` + `to_html()` in `crates/saf-analysis/src/export.rs`
2. **Phase 2a**: Convert callgraph, cfg, defuse, valueflow exports (the 4 Python-exposed graphs)
3. **Phase 3**: Update Python bindings + add `to_dot()`/`to_html()` methods
4. **Phase 5a**: Update Rust and Python tests for the 4 converted graphs
5. **Phase 6**: Update tutorials
6. **Phase 2b**: Convert remaining graph exports (SVFG, PTA, MSSA, IFDS, etc.)
7. **Phase 5b**: Update remaining tests
8. **Phase 4**: Add `saf.viz` Python module
9. **Phase 7**: Add `make notebook` target + `output/` gitignore

## Verification

1. `make fmt && make lint` — all clippy/fmt checks pass
2. `make test` — all Rust + Python tests pass
3. `cargo insta review` (inside Docker) — snapshots updated
4. Run tutorial scripts in Docker — all produce correct output
5. Manual DOT test: export a callgraph as DOT, render with `dot -Tsvg`, verify it looks correct
6. Manual HTML test: export a CFG as HTML, open in browser, verify interactive features work (zoom, pan, dagre layout, search)
7. `make notebook` — starts Jupyter Lab, accessible at `localhost:8888` from host browser
8. Jupyter test: load PG dict in notebook, convert with `to_cytoscape_json()`, display with `ipycytoscape`
9. PTABen benchmarks still pass (export changes shouldn't affect analysis results, but verify)
