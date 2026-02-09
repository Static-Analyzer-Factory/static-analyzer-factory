#!/usr/bin/env python3
"""Detect CWE-134 format string vulnerability using the SAF Python SDK.

This tutorial demonstrates using call-return sources:
the return value of gets() is tainted, and it flows to
printf()'s format argument (arg_index=0).

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

    # SOURCE: return value of gets() — tainted user input
    # SINK:   argument 0 of printf() — the format string position
    findings = q.taint_flow(
        sources=sources.call("gets"),
        sinks=sinks.call("printf", arg_index=0),
    )

    print(f"Found {len(findings)} format string taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")


if __name__ == "__main__":
    main()
