#!/usr/bin/env python3
"""Detect heap buffer overflow using taint flow analysis.

This tutorial demonstrates detecting buffer overflows (CWE-120) by tracking
heap allocations through pointer arithmetic to memory access functions.
Taint analysis follows the data flow from allocation to use, identifying
when pointer arithmetic may produce out-of-bounds addresses.

The vulnerable.c file allocates a small buffer and then computes a pointer
past its boundary using pointer arithmetic before passing it to puts().

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project, sources, sinks


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C source to LLVM IR
    print("Step 1: Compiling C source to LLVM IR...")
    subprocess.run(
        [
            "clang-18", "-S", "-emit-llvm", "-O0", "-g",
            "-o", str(llvm_ir), str(source),
        ],
        check=True,
    )
    print(f"  Compiled: {source.name} -> {llvm_ir.name}")

    # Step 2: Load via LLVM frontend
    print("\nStep 2: Loading via LLVM frontend...")
    proj = Project.open(str(llvm_ir))
    q = proj.query()
    print(f"  Project loaded: {proj}")

    # Step 3: Track malloc result flowing through arithmetic to puts
    print("\nStep 3: Running taint flow analysis...")
    print("  Source: malloc() return value (heap base pointer)")
    print("  Sink: puts() argument (string pointer)")

    findings = q.taint_flow(
        sources=sources.call("malloc"),
        sinks=sinks.call("puts", arg_index=0),
    )

    print(f"\n  Found {len(findings)} buffer overflow flow(s):")
    for i, f in enumerate(findings):
        print(f"\n  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"      trace steps: {len(f.trace.steps)}")
            for j, step in enumerate(f.trace.steps):
                print(f"        step {j}: {step}")

    # Step 4: Explain the detection
    print("\n\nStep 4: Understanding the detection...")
    print("  The buffer overflow is detected through pointer arithmetic tracking:")
    print()
    print("  1. malloc(16) returns a pointer to 16-byte buffer")
    print("     -> Source: the return value is tainted (allocation base)")
    print()
    print("  2. buf + 256 computes an out-of-bounds address")
    print("     -> Transform edge: arithmetic preserves taint")
    print()
    print("  3. puts(oob_ptr) receives the out-of-bounds pointer")
    print("     -> Sink: tainted data reaches a memory access function")

    # Step 5: Explore the ValueFlow graph
    print("\nStep 5: Exploring the ValueFlow graph...")
    graphs = proj.graphs()
    vf = graphs.export("valueflow")
    # PropertyGraph format: nodes and edges are dicts with id/labels/properties
    print(f"  Nodes: {len(vf['nodes'])}")
    print(f"  Edges: {len(vf['edges'])}")

    # Count edge types (edge_type field in PropertyGraph)
    edge_types = {}
    for edge in vf["edges"]:
        kind = edge.get("edge_type", "unknown")
        edge_types[kind] = edge_types.get(kind, 0) + 1

    print("\n  Edge types:")
    for kind, count in sorted(edge_types.items()):
        print(f"    {kind}: {count}")

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY: Buffer Overflow Detection via Taint Flow (CWE-120)")
    print(f"  Findings: {len(findings)}")
    if findings:
        print("  HEAP BUFFER OVERFLOW DETECTED in vulnerable.c")
        print("  Pointer arithmetic (buf + 256) exceeds the 16-byte allocation,")
        print("  and the out-of-bounds pointer is passed to puts().")
    else:
        print("  No buffer overflow flows detected.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
