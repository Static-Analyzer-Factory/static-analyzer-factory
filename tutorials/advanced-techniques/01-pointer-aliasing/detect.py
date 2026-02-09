#!/usr/bin/env python3
"""Explore pointer aliasing and indirect call resolution using SAF's PTA.

This tutorial combines:
- Basic pointer aliasing concepts (points-to sets, may-alias, no-alias)
- Indirect call resolution via function pointers

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "program.c"
    llvm_ir = tutorial_dir / "program.ll"

    # Step 1: Compile C source to LLVM IR
    print("Step 1: Compiling C source to LLVM IR...")
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )
    print(f"  Compiled: {source.name} -> {llvm_ir.name}")

    # Step 2: Load via LLVM frontend and build analysis
    print("\nStep 2: Loading project and running PTA...")
    proj = Project.open(str(llvm_ir))

    # ===== PART A: Basic Pointer Aliasing =====
    print("\n" + "=" * 60)
    print("PART A: Basic Pointer Aliasing")
    print("=" * 60)

    # Step 3: Get PTA results
    pta = proj.pta_result()

    print("\nPTA Statistics:")
    print(f"  Values tracked: {pta.value_count}")
    print(f"  Abstract locations: {pta.location_count}")

    # Step 4: Export the full PTA result for inspection
    export = pta.export()
    pts_list = export.get("points_to", [])
    print(f"  Points-to entries: {len(pts_list)}")

    # Step 5: Show points-to sets
    print("\nPoints-to sets (first 10):")
    for entry in pts_list[:10]:
        val_id = entry["value"]
        locs = entry.get("locations", [])
        print(f"  {val_id[:20]}... -> {len(locs)} location(s)")

    # Step 6: Try alias queries using exported value IDs
    val_ids = [entry["value"] for entry in pts_list]
    if len(val_ids) >= 2:
        v1, v2 = val_ids[0], val_ids[1]
        try:
            alias = pta.may_alias(v1, v2)
            print(f"\nAlias queries:")
            print(f"  may_alias({v1[:16]}..., {v2[:16]}...) = {alias}")
        except Exception as e:
            print(f"\n  Alias query error: {e}")

    if len(val_ids) >= 3:
        v1, v3 = val_ids[0], val_ids[2]
        try:
            no = pta.no_alias(v1, v3)
            print(f"  no_alias({v1[:16]}..., {v3[:16]}...) = {no}")
        except Exception as e:
            print(f"  No-alias query error: {e}")

    # ===== PART B: Indirect Call Resolution =====
    print("\n" + "=" * 60)
    print("PART B: Indirect Call Resolution")
    print("=" * 60)

    # Step 7: Export the call graph (includes PTA-resolved indirect call edges)
    graphs = proj.graphs()
    callgraph = graphs.export("callgraph")

    # PropertyGraph format:
    # {"schema_version": "0.1.0", "graph_type": "callgraph",
    #  "nodes": [{"id": ..., "labels": ["Function"],
    #             "properties": {"name": ..., "kind": ...}}, ...],
    #  "edges": [{"src": ..., "dst": ..., "edge_type": "CALLS", "properties": {}}, ...]}
    node_list = callgraph.get("nodes", [])
    edges = callgraph.get("edges", [])
    nodes = {n["id"]: n for n in node_list}
    print(f"\nCall Graph: {len(nodes)} functions, {len(edges)} call edges")

    # Step 8: List all functions found in the call graph
    print("\nFunctions:")
    for nid, info in nodes.items():
        name = info.get("properties", {}).get("name", nid)
        print(f"  {name}")

    # Step 9: Show all call edges
    print("\nCall edges:")
    for edge in edges:
        src = edge.get("src", "")
        dst = edge.get("dst", "")
        src_name = nodes.get(src, {}).get("properties", {}).get("name", str(src)[:20])
        dst_name = nodes.get(dst, {}).get("properties", {}).get("name", str(dst)[:20])
        print(f"  {src_name} -> {dst_name}")

    # Step 10: Inspect the AIR module
    air = proj.air()
    print(f"\nAIR Module:")
    print(f"  Functions: {air.function_names()}")

    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)
    print("  - PTA computes points-to sets for all pointer values")
    print("  - may_alias() checks if two pointers may point to overlapping locations")
    print("  - no_alias() checks if two pointers definitely point to disjoint locations")
    print("  - PTA resolves indirect calls by looking up function pointer targets")
    print("  - The call graph includes both direct and PTA-resolved indirect edges")


if __name__ == "__main__":
    main()
