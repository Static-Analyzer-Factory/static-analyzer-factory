"""Visualization utilities for SAF property graphs.

Convert SAF ``PropertyGraph`` dicts to various visualization formats
including Graphviz DOT, interactive HTML (Cytoscape.js), NetworkX graphs,
and Cytoscape JSON for Jupyter notebooks.

Dependency-free functions (pure Python):
    - :func:`to_dot` -- Graphviz DOT string
    - :func:`to_html` -- standalone interactive HTML with Cytoscape.js (CDN)
    - :func:`to_cytoscape_json` -- Cytoscape.js JSON for ipycytoscape
    - :func:`visualize` -- generate HTML and open in browser or save to file

Functions requiring optional dependencies:
    - :func:`to_networkx` -- requires ``networkx``
    - :func:`to_graphviz` -- requires ``graphviz`` Python package

Example::

    from saf.viz import to_dot, visualize

    pg = graphs.export("callgraph")
    print(to_dot(pg))
    visualize(pg, output="callgraph.html")
"""

from __future__ import annotations

import html
import json
import tempfile
import webbrowser
from collections import defaultdict
from pathlib import Path
from typing import Any

__all__ = [
    "to_dot",
    "to_networkx",
    "to_graphviz",
    "to_html",
    "visualize",
    "to_cytoscape_json",
    "to_neo4j",
]

# Node shape mapping by label for DOT/Graphviz output.
_LABEL_SHAPES: dict[str, str] = {
    "Function": "box",
    "External": "box",
    "Indirect": "box",
    "Block": "rectangle",
    "Entry": "rectangle",
    "Exit": "rectangle",
    "Value": "ellipse",
    "MemPhi": "diamond",
    "LiveOnEntry": "diamond",
    "Location": "hexagon",
    "Pointer": "ellipse",
    "Type": "component",
    "Thread": "parallelogram",
    "Lock": "octagon",
    "Instruction": "rectangle",
    "Fact": "ellipse",
    "MemAccess": "rectangle",
    "UnknownMem": "hexagon",
}

# Default layouts per graph type for Cytoscape.js.
_GRAPH_LAYOUTS: dict[str, str] = {
    "callgraph": "dagre",
    "cfg": "dagre",
    "icfg": "dagre",
    "cha": "dagre",
    "valueflow": "cola",
    "svfg": "cola",
    "defuse": "dagre",
    "pta": "cola",
    "cspta": "cola",
    "fspta": "cola",
    "mssa": "dagre",
    "ifds": "dagre",
    "ide": "dagre",
    "absint": "dagre",
    "mta": "cola",
    "dda": "cola",
}


def _dot_escape(s: str) -> str:
    """Escape a string for use in DOT labels."""
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def _node_label(node: dict[str, Any]) -> str:
    """Derive a human-readable label for a property graph node."""
    props = node.get("properties", {})
    name = props.get("name")
    if name:
        return str(name)
    labels = node.get("labels", [])
    if labels:
        return labels[0]
    node_id = node.get("id", "?")
    # Truncate long hex IDs for readability.
    if isinstance(node_id, str) and len(node_id) > 12:
        return node_id[:12] + "..."
    return str(node_id)


def _node_shape(node: dict[str, Any]) -> str:
    """Determine DOT shape for a node based on its labels."""
    for label in node.get("labels", []):
        if label in _LABEL_SHAPES:
            return _LABEL_SHAPES[label]
    return "ellipse"


def to_dot(pg: dict[str, Any]) -> str:
    """Convert a PropertyGraph dict to a Graphviz DOT format string.

    This is a dependency-free function that generates DOT source as a plain
    string. Nodes are shaped according to their labels and grouped into
    subgraph clusters when a ``function`` property is present.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.

    Returns:
        A DOT language string representing the graph.

    Example::

        dot_str = to_dot(graphs.export("callgraph"))
        with open("callgraph.dot", "w") as f:
            f.write(dot_str)
    """
    lines: list[str] = ["digraph {"]
    lines.append('  rankdir=TB;')
    lines.append('  node [fontname="Helvetica", fontsize=10];')
    lines.append('  edge [fontname="Helvetica", fontsize=9];')
    lines.append("")

    nodes = pg.get("nodes", [])
    edges = pg.get("edges", [])

    # Group nodes by function property for subgraph clustering.
    function_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    ungrouped: list[dict[str, Any]] = []

    for node in nodes:
        func = node.get("properties", {}).get("function")
        if func:
            function_groups[str(func)].append(node)
        else:
            ungrouped.append(node)

    def _emit_node(node: dict[str, Any], indent: str = "  ") -> str:
        nid = node.get("id", "")
        label = _dot_escape(_node_label(node))
        shape = _node_shape(node)
        return f'{indent}"{_dot_escape(nid)}" [label="{label}", shape={shape}];'

    # Emit ungrouped nodes.
    for node in ungrouped:
        lines.append(_emit_node(node))

    # Emit clustered subgraphs.
    for idx, (func_name, func_nodes) in enumerate(sorted(function_groups.items())):
        lines.append("")
        lines.append(f"  subgraph cluster_{idx} {{")
        lines.append(f'    label="{_dot_escape(func_name)}";')
        lines.append("    style=dashed;")
        lines.append('    color="#666666";')
        for node in func_nodes:
            lines.append(_emit_node(node, indent="    "))
        lines.append("  }")

    # Emit edges.
    lines.append("")
    for edge in edges:
        src = _dot_escape(edge.get("src", ""))
        dst = _dot_escape(edge.get("dst", ""))
        edge_type = _dot_escape(edge.get("edge_type", ""))
        lines.append(f'  "{src}" -> "{dst}" [label="{edge_type}"];')

    lines.append("}")
    return "\n".join(lines) + "\n"


def to_networkx(pg: dict[str, Any]) -> Any:
    """Convert a PropertyGraph dict to a NetworkX ``DiGraph``.

    Each node gets its labels and properties as node attributes. Each edge
    gets ``edge_type`` and its properties as edge attributes.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.

    Returns:
        A ``networkx.DiGraph`` instance.

    Raises:
        ImportError: If the ``networkx`` package is not installed.

    Example::

        G = to_networkx(graphs.export("callgraph"))
        print(G.number_of_nodes(), G.number_of_edges())
    """
    try:
        import networkx as nx
    except ImportError as exc:
        raise ImportError(
            "The 'networkx' package is required for to_networkx(). "
            "Install it with: pip install networkx>=3.0  "
            "or: pip install saf[viz]"
        ) from exc

    g = nx.DiGraph()
    g.graph["graph_type"] = pg.get("graph_type", "")
    g.graph["schema_version"] = pg.get("schema_version", "")
    for key, val in pg.get("metadata", {}).items():
        g.graph[key] = val

    for node in pg.get("nodes", []):
        nid = node.get("id", "")
        attrs: dict[str, Any] = {"labels": node.get("labels", [])}
        attrs.update(node.get("properties", {}))
        g.add_node(nid, **attrs)

    for edge in pg.get("edges", []):
        src = edge.get("src", "")
        dst = edge.get("dst", "")
        attrs = {"edge_type": edge.get("edge_type", "")}
        attrs.update(edge.get("properties", {}))
        g.add_edge(src, dst, **attrs)

    return g


def to_graphviz(pg: dict[str, Any]) -> Any:
    """Convert a PropertyGraph dict to a ``graphviz.Digraph`` object.

    Uses the same node shapes and labels as :func:`to_dot`. The returned
    object can be rendered to PDF, SVG, PNG, etc.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.

    Returns:
        A ``graphviz.Digraph`` instance.

    Raises:
        ImportError: If the ``graphviz`` package is not installed.

    Example::

        gv = to_graphviz(graphs.export("cfg"))
        gv.render("cfg", format="svg", cleanup=True)
    """
    try:
        import graphviz
    except ImportError as exc:
        raise ImportError(
            "The 'graphviz' package is required for to_graphviz(). "
            "Install it with: pip install graphviz>=0.20  "
            "or: pip install saf[viz]  "
            "Note: you also need the Graphviz system binary (apt install graphviz)."
        ) from exc

    dot = graphviz.Digraph(
        graph_attr={"rankdir": "TB"},
        node_attr={"fontname": "Helvetica", "fontsize": "10"},
        edge_attr={"fontname": "Helvetica", "fontsize": "9"},
    )

    nodes = pg.get("nodes", [])
    edges = pg.get("edges", [])

    # Group nodes by function for subgraph clustering.
    function_groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    ungrouped: list[dict[str, Any]] = []

    for node in nodes:
        func = node.get("properties", {}).get("function")
        if func:
            function_groups[str(func)].append(node)
        else:
            ungrouped.append(node)

    for node in ungrouped:
        nid = node.get("id", "")
        dot.node(nid, label=_node_label(node), shape=_node_shape(node))

    for idx, (func_name, func_nodes) in enumerate(sorted(function_groups.items())):
        with dot.subgraph(name=f"cluster_{idx}") as sub:
            sub.attr(label=func_name, style="dashed", color="#666666")
            for node in func_nodes:
                nid = node.get("id", "")
                sub.node(nid, label=_node_label(node), shape=_node_shape(node))

    for edge in edges:
        dot.edge(
            edge.get("src", ""),
            edge.get("dst", ""),
            label=edge.get("edge_type", ""),
        )

    return dot


def to_cytoscape_json(pg: dict[str, Any]) -> dict[str, Any]:
    """Convert a PropertyGraph dict to Cytoscape.js JSON format.

    The returned dict can be used with ``ipycytoscape`` in Jupyter notebooks
    or directly with the Cytoscape.js JavaScript library.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.

    Returns:
        A dict in Cytoscape.js JSON format with ``elements.nodes`` and
        ``elements.edges``.

    Example::

        import ipycytoscape
        w = ipycytoscape.CytoscapeWidget()
        w.graph.add_graph_from_json(to_cytoscape_json(pg))
        display(w)
    """
    cy_nodes: list[dict[str, Any]] = []
    for node in pg.get("nodes", []):
        data: dict[str, Any] = {"id": node.get("id", "")}
        data["label"] = _node_label(node)
        labels = node.get("labels", [])
        if labels:
            data["node_type"] = labels[0]
        for key, val in node.get("properties", {}).items():
            data[key] = val
        cy_nodes.append({"data": data})

    cy_edges: list[dict[str, Any]] = []
    for edge in pg.get("edges", []):
        data = {
            "source": edge.get("src", ""),
            "target": edge.get("dst", ""),
            "label": edge.get("edge_type", ""),
            "edge_type": edge.get("edge_type", ""),
        }
        for key, val in edge.get("properties", {}).items():
            data[key] = val
        cy_edges.append({"data": data})

    return {"elements": {"nodes": cy_nodes, "edges": cy_edges}}


def to_html(pg: dict[str, Any], layout: str = "auto") -> str:
    """Generate a self-contained interactive HTML page with Cytoscape.js.

    The HTML includes all dependencies loaded from CDN and the graph data
    embedded as inline JSON. Features include search, zoom-to-fit, tooltips,
    click-to-highlight neighbors, and edge type filtering.

    No dependencies are required -- this is pure string generation.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.
        layout: Layout algorithm. ``"auto"`` selects based on graph type
            (dagre for cfg/callgraph, cola for valueflow/svfg, cose otherwise).
            Other options: ``"dagre"``, ``"cola"``, ``"cose"``,
            ``"breadthfirst"``, ``"circle"``, ``"grid"``.

    Returns:
        A complete HTML document string.

    Example::

        html_str = to_html(graphs.export("cfg"))
        with open("cfg.html", "w") as f:
            f.write(html_str)
    """
    graph_type = pg.get("graph_type", "")
    if layout == "auto":
        layout = _GRAPH_LAYOUTS.get(graph_type, "cose")

    cy_data = to_cytoscape_json(pg)
    elements_json = json.dumps(cy_data["elements"], indent=2)

    # Collect unique edge types for the filter UI.
    edge_types: list[str] = sorted(
        {e.get("edge_type", "") for e in pg.get("edges", []) if e.get("edge_type")}
    )
    edge_type_checkboxes = "\n".join(
        f'        <label><input type="checkbox" class="edge-filter" '
        f'value="{html.escape(et)}" checked> {html.escape(et)}</label>'
        for et in edge_types
    )

    title = html.escape(f"SAF {graph_type}" if graph_type else "SAF Graph")
    node_count = len(pg.get("nodes", []))
    edge_count = len(pg.get("edges", []))

    # Cytoscape node shape mapping.
    node_shape_styles = "\n".join(
        f"      {{ selector: 'node[node_type=\"{label}\"]', style: {{"
        f" shape: '{_cytoscape_shape(shape)}' }} }},"
        for label, shape in _LABEL_SHAPES.items()
    )

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<script src="https://unpkg.com/cytoscape@3/dist/cytoscape.min.js"></script>
<script src="https://unpkg.com/dagre@0.8/dist/dagre.min.js"></script>
<script src="https://unpkg.com/cytoscape-dagre@2/cytoscape-dagre.js"></script>
<script src="https://unpkg.com/webcola@3/WebCola/cola.min.js"></script>
<script src="https://unpkg.com/cytoscape-cola@2/cytoscape-cola.js"></script>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }}
  #toolbar {{
    position: fixed; top: 0; left: 0; right: 0; z-index: 100;
    background: #1a1a2e; color: #e0e0e0; padding: 8px 16px;
    display: flex; align-items: center; gap: 12px; flex-wrap: wrap;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3); font-size: 13px;
  }}
  #toolbar .title {{ font-weight: 600; color: #8be9fd; margin-right: 8px; }}
  #toolbar .stats {{ color: #999; font-size: 12px; }}
  #toolbar input[type="text"] {{
    padding: 4px 8px; border: 1px solid #444; border-radius: 4px;
    background: #2a2a3e; color: #e0e0e0; font-size: 13px; width: 180px;
  }}
  #toolbar button {{
    padding: 4px 10px; border: 1px solid #444; border-radius: 4px;
    background: #2a2a3e; color: #e0e0e0; cursor: pointer; font-size: 12px;
  }}
  #toolbar button:hover {{ background: #3a3a5e; }}
  #toolbar select {{
    padding: 4px 6px; border: 1px solid #444; border-radius: 4px;
    background: #2a2a3e; color: #e0e0e0; font-size: 12px;
  }}
  .filter-group {{
    display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
  }}
  .filter-group label {{ cursor: pointer; font-size: 12px; }}
  #cy {{ position: absolute; top: 52px; left: 0; right: 0; bottom: 0; background: #0d1117; }}
  #tooltip {{
    display: none; position: fixed; z-index: 200;
    background: #1e1e30; color: #e0e0e0; padding: 10px 14px;
    border-radius: 6px; font-size: 12px; max-width: 400px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.5); pointer-events: none;
    border: 1px solid #333;
  }}
  #tooltip .tt-title {{ font-weight: 600; color: #8be9fd; margin-bottom: 4px; }}
  #tooltip .tt-row {{ margin: 2px 0; }}
  #tooltip .tt-key {{ color: #999; }}
</style>
</head>
<body>
<div id="toolbar">
  <span class="title">{title}</span>
  <span class="stats">{node_count} nodes, {edge_count} edges</span>
  <input type="text" id="search" placeholder="Search nodes...">
  <button id="btn-fit">Fit</button>
  <select id="layout-select">
    <option value="dagre" {"selected" if layout == "dagre" else ""}>Dagre</option>
    <option value="cola" {"selected" if layout == "cola" else ""}>Cola</option>
    <option value="cose" {"selected" if layout == "cose" else ""}>CoSE</option>
    <option value="breadthfirst" {"selected" if layout == "breadthfirst" else ""}>Breadthfirst</option>
    <option value="circle" {"selected" if layout == "circle" else ""}>Circle</option>
    <option value="grid" {"selected" if layout == "grid" else ""}>Grid</option>
  </select>
  <div class="filter-group">
{edge_type_checkboxes}
  </div>
</div>
<div id="cy"></div>
<div id="tooltip"></div>
<script>
(function() {{
  var elements = {elements_json};

  var cy = cytoscape({{
    container: document.getElementById('cy'),
    elements: elements,
    style: [
      {{
        selector: 'node',
        style: {{
          'label': 'data(label)',
          'font-size': '10px',
          'text-valign': 'center',
          'text-halign': 'center',
          'background-color': '#4a90d9',
          'color': '#e0e0e0',
          'text-outline-color': '#0d1117',
          'text-outline-width': 1,
          'width': 30,
          'height': 30,
          'border-width': 1,
          'border-color': '#666'
        }}
      }},
      {{
        selector: 'edge',
        style: {{
          'label': 'data(label)',
          'font-size': '8px',
          'color': '#999',
          'text-rotation': 'autorotate',
          'curve-style': 'bezier',
          'target-arrow-shape': 'triangle',
          'target-arrow-color': '#555',
          'line-color': '#555',
          'width': 1.5,
          'arrow-scale': 0.8,
          'text-outline-color': '#0d1117',
          'text-outline-width': 1
        }}
      }},
      {{
        selector: 'node.highlighted',
        style: {{
          'background-color': '#ff6b6b',
          'border-color': '#ff4444',
          'border-width': 2
        }}
      }},
      {{
        selector: 'node.neighbor',
        style: {{
          'background-color': '#ffa500',
          'border-color': '#ff8c00',
          'border-width': 2
        }}
      }},
      {{
        selector: 'edge.highlighted',
        style: {{
          'line-color': '#ff6b6b',
          'target-arrow-color': '#ff6b6b',
          'width': 3
        }}
      }},
      {{
        selector: 'node.search-match',
        style: {{
          'background-color': '#50fa7b',
          'border-color': '#30d95b',
          'border-width': 3
        }}
      }},
      {{
        selector: 'node.dimmed',
        style: {{ 'opacity': 0.2 }}
      }},
      {{
        selector: 'edge.dimmed',
        style: {{ 'opacity': 0.1 }}
      }},
      {{
        selector: 'edge.filtered-out',
        style: {{ 'display': 'none' }}
      }},
{node_shape_styles}
    ],
    layout: {{ name: '{layout}' }},
    wheelSensitivity: 0.3
  }});

  // Zoom to fit.
  document.getElementById('btn-fit').addEventListener('click', function() {{
    cy.fit(undefined, 30);
  }});

  // Layout switch.
  document.getElementById('layout-select').addEventListener('change', function() {{
    cy.layout({{ name: this.value, animate: true, animationDuration: 500 }}).run();
  }});

  // Search.
  var searchInput = document.getElementById('search');
  searchInput.addEventListener('input', function() {{
    var q = this.value.toLowerCase().trim();
    cy.nodes().removeClass('search-match');
    if (q) {{
      cy.nodes().forEach(function(n) {{
        var lbl = (n.data('label') || '').toLowerCase();
        var name = (n.data('name') || '').toLowerCase();
        if (lbl.indexOf(q) >= 0 || name.indexOf(q) >= 0) {{
          n.addClass('search-match');
        }}
      }});
      var matches = cy.nodes('.search-match');
      if (matches.length > 0) {{
        cy.fit(matches, 60);
      }}
    }}
  }});

  // Click to highlight neighbors.
  cy.on('tap', 'node', function(evt) {{
    var node = evt.target;
    cy.elements().removeClass('highlighted neighbor dimmed');
    cy.edges().removeClass('highlighted');

    node.addClass('highlighted');
    var neighborhood = node.neighborhood();
    neighborhood.nodes().addClass('neighbor');
    neighborhood.edges().addClass('highlighted');

    cy.elements().not(node).not(neighborhood).addClass('dimmed');
  }});

  // Click on background to clear.
  cy.on('tap', function(evt) {{
    if (evt.target === cy) {{
      cy.elements().removeClass('highlighted neighbor dimmed');
      cy.edges().removeClass('highlighted');
    }}
  }});

  // Tooltip.
  var tooltip = document.getElementById('tooltip');
  cy.on('mouseover', 'node', function(evt) {{
    var node = evt.target;
    var d = node.data();
    var rows = '<div class="tt-title">' + escapeHtml(d.label || d.id) + '</div>';
    if (d.node_type) rows += '<div class="tt-row"><span class="tt-key">type:</span> ' + escapeHtml(d.node_type) + '</div>';
    Object.keys(d).forEach(function(k) {{
      if (k !== 'id' && k !== 'label' && k !== 'node_type') {{
        rows += '<div class="tt-row"><span class="tt-key">' + escapeHtml(k) + ':</span> ' + escapeHtml(String(d[k])) + '</div>';
      }}
    }});
    tooltip.innerHTML = rows;
    tooltip.style.display = 'block';
  }});
  cy.on('mouseover', 'edge', function(evt) {{
    var edge = evt.target;
    var d = edge.data();
    var rows = '<div class="tt-title">' + escapeHtml(d.edge_type || '') + '</div>';
    rows += '<div class="tt-row"><span class="tt-key">source:</span> ' + escapeHtml(d.source) + '</div>';
    rows += '<div class="tt-row"><span class="tt-key">target:</span> ' + escapeHtml(d.target) + '</div>';
    Object.keys(d).forEach(function(k) {{
      if (k !== 'source' && k !== 'target' && k !== 'label' && k !== 'edge_type') {{
        rows += '<div class="tt-row"><span class="tt-key">' + escapeHtml(k) + ':</span> ' + escapeHtml(String(d[k])) + '</div>';
      }}
    }});
    tooltip.innerHTML = rows;
    tooltip.style.display = 'block';
  }});
  cy.on('mouseout', function() {{
    tooltip.style.display = 'none';
  }});
  document.addEventListener('mousemove', function(e) {{
    if (tooltip.style.display === 'block') {{
      tooltip.style.left = (e.pageX + 14) + 'px';
      tooltip.style.top = (e.pageY + 14) + 'px';
    }}
  }});

  // Edge type filter.
  document.querySelectorAll('.edge-filter').forEach(function(cb) {{
    cb.addEventListener('change', function() {{
      var checked = [];
      document.querySelectorAll('.edge-filter:checked').forEach(function(c) {{
        checked.push(c.value);
      }});
      cy.edges().forEach(function(edge) {{
        var et = edge.data('edge_type') || '';
        if (checked.indexOf(et) >= 0 || !et) {{
          edge.removeClass('filtered-out');
        }} else {{
          edge.addClass('filtered-out');
        }}
      }});
    }});
  }});

  function escapeHtml(s) {{
    var div = document.createElement('div');
    div.appendChild(document.createTextNode(s));
    return div.innerHTML;
  }}
}})();
</script>
</body>
</html>
"""


def _cytoscape_shape(dot_shape: str) -> str:
    """Map a DOT shape name to its Cytoscape.js equivalent."""
    mapping = {
        "box": "round-rectangle",
        "rectangle": "rectangle",
        "ellipse": "ellipse",
        "diamond": "diamond",
        "hexagon": "hexagon",
        "component": "barrel",
        "parallelogram": "rhomboid",
        "octagon": "octagon",
    }
    return mapping.get(dot_shape, "ellipse")


def visualize(
    pg: dict[str, Any],
    output: str | None = None,
    layout: str = "auto",
) -> None:
    """Generate an interactive HTML visualization and display or save it.

    If ``output`` is provided, the HTML is written to that file path.
    Otherwise, a temporary file is created and opened in the default
    web browser.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.
        output: File path to write HTML to. If ``None``, opens in browser.
        layout: Layout algorithm (see :func:`to_html` for options).

    Example::

        # Open in browser
        visualize(graphs.export("callgraph"))

        # Save to file
        visualize(graphs.export("cfg"), output="cfg.html")
    """
    html_content = to_html(pg, layout=layout)

    if output is not None:
        path = Path(output)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(html_content, encoding="utf-8")
    else:
        with tempfile.NamedTemporaryFile(
            suffix=".html", prefix="saf_viz_", delete=False, mode="w", encoding="utf-8"
        ) as f:
            f.write(html_content)
            tmp_path = f.name
        webbrowser.open(f"file://{tmp_path}")


def to_neo4j(pg: dict[str, Any], driver: Any) -> None:
    """Load a PropertyGraph into Neo4j.

    Args:
        pg: A PropertyGraph dict with ``nodes`` and ``edges`` keys.
        driver: A ``neo4j.Driver`` instance.

    Raises:
        NotImplementedError: Always. Neo4j export is planned for a future release.
    """
    raise NotImplementedError("Neo4j export is planned for a future release")
