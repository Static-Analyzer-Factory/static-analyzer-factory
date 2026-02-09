#!/usr/bin/env python3
"""Export all SAF graphs to JSON for external tool integration.

Demonstrates the graph export pipeline: export each available graph
type and compute basic metrics (node count, edge count, max fan-out).

Usage:
    python detect.py
"""

import subprocess
import json
from collections import Counter
from pathlib import Path

from saf import Project


def _count_graph(graph_name: str, data: dict) -> tuple[int, int, Counter]:
    """Extract node count, edge count, and out-degree map for any graph type.

    All graph types now use the unified PropertyGraph format:
    {"schema_version": "0.1.0", "graph_type": "...",
     "nodes": [{"id": ..., "labels": [...], "properties": {...}}, ...],
     "edges": [{"src": ..., "dst": ..., "edge_type": "...", "properties": {...}}, ...]}
    """
    out_degree: Counter = Counter()

    nodes = data.get("nodes", [])
    edges = data.get("edges", [])

    for edge in edges:
        out_degree[edge.get("src", "")] += 1

    return len(nodes), len(edges), out_degree


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "program.c"
    llvm_ir = tutorial_dir / "program.ll"

    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    proj = Project.open(str(llvm_ir))
    graphs = proj.graphs()

    available = graphs.available()
    print(f"Available graph types: {available}")

    for graph_name in available:
        print(f"\n{'='*50}")
        print(f"Graph: {graph_name}")
        print(f"{'='*50}")

        data = graphs.export(graph_name)

        n_nodes, n_edges, out_degree = _count_graph(graph_name, data)

        print(f"  Nodes: {n_nodes}")
        print(f"  Edges: {n_edges}")

        if out_degree:
            max_node, max_deg = out_degree.most_common(1)[0]
            short = str(max_node)[:20] + "..." if len(str(max_node)) > 20 else str(max_node)
            print(f"  Max fan-out: {max_deg} (node {short})")

        # Save to file
        out_file = tutorial_dir / f"{graph_name}.json"
        with open(out_file, "w") as f:
            json.dump(data, f, indent=2, default=str)
        print(f"  Saved to: {out_file.name}")


if __name__ == "__main__":
    main()
