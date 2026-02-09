#!/usr/bin/env python3
"""Explore def-use chains and the ValueFlow graph with SAF.

This tutorial demonstrates:
1. Def-use chains: tracking where values are defined and used
2. ValueFlow graph: interprocedural data flow with memory modeling

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path
from collections import Counter

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "program.c"
    llvm_ir = tutorial_dir / "program.ll"

    # Step 1: Compile to LLVM IR
    print("Step 1: Compiling C to LLVM IR...")
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load project
    print("\nStep 2: Loading project...")
    proj = Project.open(str(llvm_ir))
    graphs = proj.graphs()

    # ========================================
    # PART A: Def-Use Chains
    # ========================================
    print("\n" + "=" * 50)
    print("PART A: Def-Use Chains")
    print("=" * 50)

    # PropertyGraph format:
    # {"schema_version": "0.1.0", "graph_type": "defuse",
    #  "nodes": [{"id": ..., "labels": ["Value"] or ["Instruction"], "properties": {...}}, ...],
    #  "edges": [{"src": ..., "dst": ..., "edge_type": "DEFINES" or "USED_BY", "properties": {}}, ...]}
    defuse = graphs.export("defuse")

    defuse_nodes = defuse.get("nodes", [])
    defuse_edges = defuse.get("edges", [])

    # Separate edges by type
    definitions = [e for e in defuse_edges if e.get("edge_type") == "DEFINES"]
    uses = [e for e in defuse_edges if e.get("edge_type") == "USED_BY"]

    print(f"\nDef-Use Summary:")
    print(f"  Definitions: {len(definitions)}")
    print(f"  Uses: {len(uses)}")

    # Count how many times each value is used
    # In USED_BY edges, src is the value, dst is the instruction using it
    use_count = Counter()
    for use in uses:
        value = use.get("src", "")
        use_count[value] += 1

    # Find values with multiple uses (high fan-out)
    multi_use = [(vid, cnt) for vid, cnt in use_count.items() if cnt > 1]
    print(f"\nValues with multiple uses: {len(multi_use)}")
    print("  (These are values that flow to multiple places)")
    for vid, cnt in sorted(multi_use, key=lambda x: -x[1])[:5]:
        short_id = vid[:24] + "..." if len(vid) > 24 else vid
        print(f"    {short_id}: {cnt} uses")

    # Find values defined but never used (dead definitions)
    # In DEFINES edges, dst is the value being defined
    defined_values = {d["dst"] for d in definitions}
    used_values = set(use_count.keys())
    unused = defined_values - used_values
    print(f"\nDefined but unused: {len(unused)} values")
    print("  (These may be dead code or return values)")

    # ========================================
    # PART B: ValueFlow Graph
    # ========================================
    print("\n" + "=" * 50)
    print("PART B: ValueFlow Graph")
    print("=" * 50)

    # PropertyGraph format:
    # {"schema_version": "0.1.0", "graph_type": "valueflow",
    #  "metadata": {"node_count": N, "edge_count": N},
    #  "nodes": [{"id": ..., "labels": [...], "properties": {"kind": ...}}, ...],
    #  "edges": [{"src": ..., "dst": ..., "edge_type": "Direct"|"Store"|..., "properties": {}}, ...]}
    vf = graphs.export("valueflow")

    metadata = vf.get("metadata", {})
    node_count = metadata.get("node_count", len(vf.get("nodes", [])))
    edge_count = metadata.get("edge_count", len(vf.get("edges", [])))
    nodes = vf.get("nodes", [])
    edges = vf.get("edges", [])

    print(f"\nValueFlow Summary:")
    print(f"  Nodes: {node_count}")
    print(f"  Edges: {edge_count}")

    # Count edges by kind (edge_type in PropertyGraph)
    edge_kinds = Counter()
    for edge in edges:
        kind = edge.get("edge_type", "unknown")
        edge_kinds[kind] += 1

    print(f"\nEdge types (data flow mechanisms):")
    for kind, count in sorted(edge_kinds.items()):
        explanation = ""
        if kind == "Direct":
            explanation = " - value assigned directly"
        elif kind == "Store":
            explanation = " - value written to memory"
        elif kind == "Load":
            explanation = " - value read from memory"
        elif kind == "CallArg":
            explanation = " - value passed as function argument"
        elif kind == "Return":
            explanation = " - value returned from function"
        print(f"    {kind}: {count}{explanation}")

    # Count node types (from properties.kind in PropertyGraph)
    node_kinds = Counter()
    for node in nodes:
        kind = node.get("properties", {}).get("kind", "unknown")
        node_kinds[kind] += 1

    print(f"\nNode types:")
    for kind, count in sorted(node_kinds.items()):
        explanation = ""
        if kind == "Value":
            explanation = " - SSA values (registers)"
        elif kind == "Location":
            explanation = " - memory locations"
        elif kind == "UnknownMem":
            explanation = " - unknown/external memory"
        print(f"    {kind}: {count}{explanation}")

    # Find high-connectivity nodes
    out_degree = Counter()
    in_degree = Counter()
    for edge in edges:
        src = edge.get("src", "")
        dst = edge.get("dst", "")
        out_degree[src] += 1
        in_degree[dst] += 1

    print(f"\nHigh-connectivity nodes (data flow hubs):")
    print(f"  Top by outgoing edges:")
    for node, count in out_degree.most_common(3):
        short = str(node)[:30] + "..." if len(str(node)) > 30 else node
        print(f"    {short}: {count} outgoing")

    print(f"  Top by incoming edges:")
    for node, count in in_degree.most_common(3):
        short = str(node)[:30] + "..." if len(str(node)) > 30 else node
        print(f"    {short}: {count} incoming")

    # ========================================
    # Summary
    # ========================================
    print("\n" + "=" * 50)
    print("Summary")
    print("=" * 50)
    print("  Def-Use chains track value definitions and their uses within functions.")
    print("  The ValueFlow graph extends this to track data across functions and memory.")
    print("  Together, they enable precise taint analysis and data flow queries.")


if __name__ == "__main__":
    main()
