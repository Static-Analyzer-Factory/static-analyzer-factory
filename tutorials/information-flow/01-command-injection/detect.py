#!/usr/bin/env python3
"""Detect CWE-78 command injection using the SAF Python SDK.

This tutorial shows how to use the SAF taint flow API to find
vulnerabilities where user-controlled data reaches dangerous sinks.

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

    # Step 3: Create a query context
    q = proj.query()

    # Step 4: Define the taint flow query:
    #   SOURCE: main's argv parameter (parameter index 1)
    #   SINK:   first argument to system() calls
    findings = q.taint_flow(
        sources=sources.function_param("main", 1),
        sinks=sinks.call("system", arg_index=0),
    )

    # Report results
    print(f"Found {len(findings)} taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace steps: {len(f.trace.steps)}")


if __name__ == "__main__":
    main()
