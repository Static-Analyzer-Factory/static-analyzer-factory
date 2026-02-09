#!/usr/bin/env python3
"""Detect CWE-416 use-after-free using the SAF Python SDK.

This tutorial demonstrates memory safety analysis: tracking
heap allocation through free() to identify dangling pointer use.

The script compiles the vulnerable C source to LLVM IR, loads it
through the LLVM frontend, and runs taint analysis end-to-end.

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
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load via LLVM frontend
    proj = Project.open(str(llvm_ir))
    q = proj.query()

    # Track malloc return value flowing to free's argument
    # (the same pointer is also used after the free)
    findings = q.taint_flow(
        sources=sources.call("malloc"),
        sinks=sinks.call("free", arg_index=0),
    )

    print(f"Found {len(findings)} alloc-to-free flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")

    # Also inspect the value-flow graph structure
    graphs = proj.graphs()
    vf = graphs.export("valueflow")
    print(f"\nValueFlow graph: {len(vf['nodes'])} nodes, {len(vf['edges'])} edges")


if __name__ == "__main__":
    main()
