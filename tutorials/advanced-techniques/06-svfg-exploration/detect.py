#!/usr/bin/env python3
"""Build and inspect a Sparse Value-Flow Graph (SVFG).

Demonstrates how SAF's SVFG unifies direct (register/SSA) and indirect
(memory) value-flow into one graph, enabling precise tracking of data
through stores and loads.

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "program.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g0",
         "-fno-discard-value-names",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load via LLVM frontend
    proj = Project.open(str(llvm_ir))

    # Step 3: Build the SVFG
    #   The SVFG combines:
    #     - Direct edges: SSA def-use, transforms, call arg/return
    #     - Indirect edges: store→load flow via Memory SSA clobber analysis
    svfg = proj.svfg()

    print("SVFG Construction Results:")
    print(f"  Nodes: {svfg.node_count}")
    print(f"  Edges: {svfg.edge_count}")

    # Step 4: Inspect construction diagnostics
    diag = svfg.diagnostics()
    print(f"\n  Direct edges:   {diag['direct_edge_count']}")
    print(f"  Indirect edges: {diag['indirect_edge_count']}")
    print(f"  MemPhi nodes:   {diag['mem_phi_count']}")

    # Step 5: Export the SVFG
    export = svfg.export()
    print(f"\n  Export schema version: {export['schema_version']}")

    # Classify nodes by kind
    value_nodes = [n for n in export["nodes"] if n["kind"] == "value"]
    phi_nodes = [n for n in export["nodes"] if n["kind"] == "mem_phi"]
    print(f"  Value nodes:    {len(value_nodes)}")
    print(f"  MemPhi nodes:   {len(phi_nodes)}")

    # Classify edges by kind
    edge_kinds: dict[str, int] = {}
    for edge in export["edges"]:
        kind = edge["kind"]
        edge_kinds[kind] = edge_kinds.get(kind, 0) + 1

    print(f"\n  Edge breakdown:")
    for kind in sorted(edge_kinds):
        print(f"    {kind}: {edge_kinds[kind]}")

    print(f"\n  {repr(svfg)}")


if __name__ == "__main__":
    main()
