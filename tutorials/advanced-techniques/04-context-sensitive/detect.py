#!/usr/bin/env python3
"""Tutorial: Context-Sensitive PTA with Virtual Dispatch and CG Refinement.

This tutorial combines three advanced pointer analysis concepts:
1. C++ virtual dispatch resolution through PTA
2. Context-sensitive PTA (k-CFA) for factory functions
3. Call graph refinement via CHA + PTA iteration

The program demonstrates how context sensitivity distinguishes allocations
from different call sites, and how CG refinement resolves virtual calls.

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
        ["clang-18", "-S", "-emit-llvm", "-O0",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )
    print(f"  Compiled: {source.name} -> {llvm_ir.name}")

    # Step 2: Load and analyze
    print("\nStep 2: Loading project...")
    proj = Project.open(str(llvm_ir))

    # ===== PART A: Context-Insensitive (Andersen) PTA =====
    print("\n" + "=" * 70)
    print("PART A: Context-Insensitive PTA (Andersen)")
    print("=" * 70)
    pta = proj.pta_result()
    pta_export = pta.export()
    print(f"  Values tracked: {pta.value_count}")
    print(f"  Abstract locations: {pta.location_count}")
    print(f"  Points-to entries: {len(pta_export['points_to'])}")

    print("\n  Andersen merges all call sites to make_pair().")
    print("  Both `a` and `b` may point to the same abstract object.")

    # ===== PART B: Context-Sensitive PTA (k=1) =====
    print("\n" + "=" * 70)
    print("PART B: Context-Sensitive PTA (k=1)")
    print("=" * 70)
    cs1 = proj.context_sensitive_pta(k=1)
    cs1_diag = cs1.diagnostics()
    print(f"  Iterations: {cs1_diag['iterations']}")
    print(f"  Converged: {not cs1_diag['iteration_limit_hit']}")
    print(f"  Unique contexts: {cs1_diag['context_count']}")
    print(f"  Constraints: {cs1_diag['constraint_count']}")
    print(f"  Locations: {cs1_diag['location_count']}")
    print(f"  SCC functions: {cs1_diag['scc_function_count']}")
    print(f"  Max PTS size: {cs1_diag['max_pts_size']}")

    print("\n  k=1 sensitivity gives each call site its own context.")
    print("  `a` and `b` point to distinct context-qualified objects.")

    # ===== PART C: Context-Sensitive PTA (k=2) =====
    print("\n" + "=" * 70)
    print("PART C: Context-Sensitive PTA (k=2)")
    print("=" * 70)
    cs2 = proj.context_sensitive_pta(k=2)
    cs2_diag = cs2.diagnostics()
    print(f"  Iterations: {cs2_diag['iterations']}")
    print(f"  Unique contexts: {cs2_diag['context_count']}")
    print(f"  Constraints: {cs2_diag['constraint_count']}")
    print(f"  (k=2 >= k=1 contexts: "
          f"{cs2_diag['context_count'] >= cs1_diag['context_count']})")

    # ===== PART D: Call Graph Refinement =====
    print("\n" + "=" * 70)
    print("PART D: Call Graph Refinement (CHA + PTA)")
    print("=" * 70)
    result = proj.refine_call_graph(entry_points="main", max_iterations=10)

    print(f"  Refinement iterations: {result.iterations}")

    # Inspect the refined call graph
    cg = result.call_graph_export()
    nodes = cg["nodes"]
    edges = cg["edges"]

    print(f"  Call graph nodes: {len(nodes)}")
    print(f"  Call graph edges: {len(edges)}")

    print(f"\n  Functions in call graph:")
    for node in nodes[:10]:  # Limit output
        name = node.get("name", "<unknown>")
        print(f"    {name}")
    if len(nodes) > 10:
        print(f"    ... and {len(nodes) - 10} more")

    # Build name lookup
    id_to_name = {n["id"]: n.get("name", "<unknown>") for n in nodes}

    print(f"\n  Call edges (first 15):")
    for edge in edges[:15]:
        src_name = id_to_name.get(edge["src"], edge["src"])
        dst_name = id_to_name.get(edge["dst"], edge["dst"])
        print(f"    {src_name} -> {dst_name}")
    if len(edges) > 15:
        print(f"    ... and {len(edges) - 15} more")

    # Step 5: Inspect the class hierarchy (if present)
    cha = result.class_hierarchy()
    if cha is not None:
        hierarchy = cha.export()
        classes = hierarchy["classes"]
        vtables = hierarchy.get("vtables", {})

        print(f"\n  Class Hierarchy:")
        print(f"    Classes found: {len(classes)}")
        for cls in classes[:5]:
            print(f"    - {cls}")
        if len(classes) > 5:
            print(f"    ... and {len(classes) - 5} more")

        print(f"    Vtable entries: {len(vtables)}")
    else:
        print("\n  No class hierarchy (pure C program)")

    # Resolved indirect call sites
    sites = result.resolved_sites()
    print(f"\n  Resolved indirect call sites: {len(sites)}")

    # ===== PART E: CS-PTA Export =====
    print("\n" + "=" * 70)
    print("PART E: CS-PTA Export")
    print("=" * 70)
    export = cs1.export()
    print(f"  Schema version: {export['schema_version']}")
    print(f"  Config k: {export['config']['k']}")
    print(f"  CS entries: {len(export['contexts'])}")
    print(f"  CI summary entries: {len(export['ci_summary']['points_to'])}")

    # Show a few context-sensitive points-to entries
    print("\n  Context-sensitive points-to entries (first 5):")
    for entry in export["contexts"][:5]:
        val_short = entry["value"][:18] + "..."
        ctx_len = len(entry["context"])
        pts_count = len(entry["points_to"])
        print(f"    {val_short} [ctx depth={ctx_len}] -> {pts_count} loc(s)")

    # ===== Summary =====
    print("\n" + "=" * 70)
    print("Summary")
    print("=" * 70)
    print("  Andersen (context-insensitive):")
    print("    - Merges all call sites to factory functions")
    print("    - All returned pointers may alias each other")
    print(f"    - Single context, {len(pta_export['points_to'])} PTS entries")
    print()
    print("  k=1 Context-Sensitive:")
    print("    - Each call site gets its own context")
    print("    - Pointers from different sites don't alias")
    print(f"    - {cs1_diag['context_count']} contexts created")
    print()
    print("  k=2 Context-Sensitive:")
    print("    - Two levels of call site history")
    print(f"    - {cs2_diag['context_count']} contexts created")
    print()
    print("  Call Graph Refinement:")
    print("    - CHA bootstraps virtual call resolution")
    print("    - PTA refines indirect call targets")
    print(f"    - {result.iterations} iteration(s) to fixed point")
    print(f"    - {len(sites)} indirect call site(s) resolved")


if __name__ == "__main__":
    main()
