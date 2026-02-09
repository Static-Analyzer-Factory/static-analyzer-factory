#!/usr/bin/env python3
"""Explore call graphs and control flow graphs (CFGs) with SAF.

This tutorial demonstrates:
1. Exporting and analyzing the call graph (which functions call which)
2. Exporting and analyzing CFGs (control flow within functions)

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
    print(f"  Available graphs: {graphs.available()}")

    # ========================================
    # PART A: Call Graph
    # ========================================
    print("\n" + "=" * 50)
    print("PART A: Call Graph")
    print("=" * 50)

    callgraph = graphs.export("callgraph")

    # PropertyGraph format:
    # {"schema_version": "0.1.0", "graph_type": "callgraph",
    #  "nodes": [{"id": ..., "labels": ["Function"],
    #             "properties": {"name": ..., "kind": ...}}, ...],
    #  "edges": [{"src": ..., "dst": ..., "edge_type": "CALLS", "properties": {}}, ...]}
    node_list = callgraph.get("nodes", [])
    edges = callgraph.get("edges", [])

    # Build lookup by id
    nodes = {n["id"]: n for n in node_list}

    print(f"\nCall Graph Summary:")
    print(f"  Functions: {len(nodes)}")
    print(f"  Call edges: {len(edges)}")

    # Build caller/callee maps
    callees_map = {nid: set() for nid in nodes}
    callers_map = {nid: set() for nid in nodes}

    for edge in edges:
        src = edge.get("src", "")
        dst = edge.get("dst", "")
        if src in callees_map:
            callees_map[src].add(dst)
        if dst in callers_map:
            callers_map[dst].add(src)

    print("\nFunction relationships:")
    for node_id, info in nodes.items():
        name = info.get("properties", {}).get("name", node_id)
        n_callees = len(callees_map.get(node_id, set()))
        n_callers = len(callers_map.get(node_id, set()))
        role = ""
        if n_callers == 0:
            role = " [ENTRY POINT]"
        if n_callees == 0:
            role += " [LEAF]"
        print(f"  {name}: {n_callers} caller(s), {n_callees} callee(s){role}")

    # ========================================
    # PART B: Control Flow Graph (CFG)
    # ========================================
    print("\n" + "=" * 50)
    print("PART B: Control Flow Graph (CFG)")
    print("=" * 50)

    cfg = graphs.export("cfg")

    # PropertyGraph format:
    # {"schema_version": "0.1.0", "graph_type": "cfg",
    #  "nodes": [{"id": ..., "labels": ["Block", "Entry"], "properties": {"name": ..., "function": ...}}, ...],
    #  "edges": [{"src": ..., "dst": ..., "edge_type": "FLOWS_TO", "properties": {}}, ...]}
    cfg_nodes = cfg.get("nodes", [])
    cfg_edges = cfg.get("edges", [])

    # Group blocks by function
    functions: dict[str, list[dict]] = {}
    for node in cfg_nodes:
        if "Block" in node.get("labels", []):
            func_name = node.get("properties", {}).get("function", "<unknown>")
            functions.setdefault(func_name, []).append(node)

    print(f"\nCFG for {len(functions)} function(s)")

    # Build edge lookup: src -> list of dst
    edge_map: dict[str, list[str]] = {}
    for edge in cfg_edges:
        src = edge.get("src", "")
        dst = edge.get("dst", "")
        edge_map.setdefault(src, []).append(dst)

    for fname, blocks in functions.items():
        # Find entry and exit blocks from labels
        entry = ""
        exits = []
        for block in blocks:
            block_labels = block.get("labels", [])
            if "Entry" in block_labels:
                entry = block["id"]
            if "Exit" in block_labels:
                exits.append(block["id"])

        # Calculate in/out degree for each block
        out_degree = Counter()
        in_degree = Counter()
        block_ids = {b["id"] for b in blocks}
        for block in blocks:
            bid = block["id"]
            succs = edge_map.get(bid, [])
            out_degree[bid] = len(succs)
            for s in succs:
                if s in block_ids:
                    in_degree[s] += 1

        # Classify blocks
        branch_blocks = [b["id"] for b in blocks if out_degree.get(b["id"], 0) > 1]
        merge_blocks = [b["id"] for b in blocks if in_degree.get(b["id"], 0) > 1]

        print(f"\n  Function: {fname}")
        print(f"    Basic blocks: {len(blocks)}")
        print(f"    Entry block: {entry[:16]}..." if entry else "    Entry block: N/A")
        print(f"    Exit blocks: {len(exits)}")
        print(f"    Branch points: {len(branch_blocks)} (if/while/switch)")
        print(f"    Merge points: {len(merge_blocks)} (join points)")

        # Show first few blocks with their classification
        if blocks:
            print(f"    Block details (first 5):")
            for block in blocks[:5]:
                bid = block["id"]
                short_id = bid[:16] + "..."
                out = out_degree.get(bid, 0)
                inc = in_degree.get(bid, 0)
                labels = []
                if bid == entry:
                    labels.append("ENTRY")
                if out > 1:
                    labels.append("BRANCH")
                if out == 0:
                    labels.append("EXIT")
                if inc > 1:
                    labels.append("MERGE")
                label_str = f" [{', '.join(labels)}]" if labels else ""
                print(f"      {short_id}: {inc} in, {out} out{label_str}")

    # ========================================
    # Summary
    # ========================================
    print("\n" + "=" * 50)
    print("Summary")
    print("=" * 50)
    print(f"  The call graph shows {len(nodes)} functions and how they call each other.")
    print(f"  The CFG shows the internal structure of each function.")
    print(f"  Together, they enable interprocedural analysis across the whole program.")


if __name__ == "__main__":
    main()
