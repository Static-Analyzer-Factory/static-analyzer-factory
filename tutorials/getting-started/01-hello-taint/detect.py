#!/usr/bin/env python3
"""Detect command injection using SAF - Your First Tutorial!

This is your first SAF tutorial! It demonstrates:
1. Compiling C code to LLVM IR
2. Loading the IR with SAF
3. Running taint flow analysis to find vulnerabilities

The vulnerability: Command-line arguments (argv) flow to system() without
any sanitization, allowing arbitrary command execution.

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

    # Step 1: Compile C source to LLVM IR using clang-18
    print("Step 1: Compiling C to LLVM IR...")
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )
    print(f"  Created: {llvm_ir.name}")

    # Step 2: Load the LLVM IR into SAF
    print("\nStep 2: Loading project...")
    proj = Project.open(str(llvm_ir))
    print(f"  Project loaded successfully")

    # Step 3: Create a query context for analysis
    print("\nStep 3: Running taint analysis...")
    q = proj.query()

    # Step 4: Define source and sink, then find taint flows
    #   SOURCE: main's argv parameter (parameter index 1 = argv)
    #   SINK: first argument to system() - command execution
    findings = q.taint_flow(
        sources=sources.function_param("main", 1),
        sinks=sinks.call("system", arg_index=0),
    )

    # Step 5: Report results
    print(f"\nResults:")
    print(f"  Found {len(findings)} taint flow(s)")

    for i, finding in enumerate(findings):
        print(f"\n  Finding {i + 1}:")
        print(f"    ID: {finding.finding_id}")
        if finding.trace:
            print(f"    Trace: {len(finding.trace.steps)} step(s)")
            print(f"    Path: argv -> ... -> system()")

    if findings:
        print("\n  VULNERABILITY DETECTED: Command-line input flows to system()!")
        print("  This is CWE-78: Improper Neutralization of Special Elements")
        print("  used in an OS Command (Command Injection)")
    else:
        print("\n  No vulnerabilities found.")


if __name__ == "__main__":
    main()
