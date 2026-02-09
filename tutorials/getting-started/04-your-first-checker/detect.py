#!/usr/bin/env python3
"""Detect memory leaks using SAF's checker framework.

This tutorial demonstrates:
1. Using built-in checkers for common bug patterns
2. Understanding checker output (findings, traces, severity)
3. Exploring the checker framework API

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

import saf


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile to LLVM IR
    print("Step 1: Compiling C to LLVM IR...")
    subprocess.run(
        [
            "clang-18", "-S", "-emit-llvm", "-O0", "-g0",
            "-Xclang", "-disable-O0-optnone",
            "-fno-discard-value-names",
            "-o", str(llvm_ir), str(source),
        ],
        check=True,
    )
    print(f"  Compiled: {source.name}")

    # Step 2: Load project
    print("\nStep 2: Loading project...")
    proj = saf.Project.open(str(llvm_ir))
    print("  Project loaded successfully")

    # Step 3: Explore available checkers
    print("\nStep 3: Available checkers...")
    schema = proj.checker_schema()
    print(f"  SAF has {schema['count']} built-in checkers:")
    for checker in schema["checkers"]:
        cwe = f"CWE-{checker['cwe']}" if checker.get("cwe") else "N/A"
        print(f"    - {checker['name']}: {checker['description']} ({cwe})")

    # Step 4: Run the memory leak checker
    print("\n" + "=" * 50)
    print("Step 4: Running memory-leak checker")
    print("=" * 50)

    findings = proj.check("memory-leak")

    print(f"\nFindings: {len(findings)}")
    for i, finding in enumerate(findings):
        print(f"\n  Finding {i + 1}:")
        print(f"    Checker: {finding.checker}")
        print(f"    Severity: {finding.severity}")
        print(f"    Message: {finding.message}")
        print(f"    Source: {finding.source}")
        print(f"    Sink: {finding.sink}")
        print(f"    Trace length: {len(finding.trace)} step(s)")

    # Step 5: Run all checkers
    print("\n" + "=" * 50)
    print("Step 5: Running all checkers")
    print("=" * 50)

    all_findings = proj.check_all()
    print(f"\nTotal findings from all checkers: {len(all_findings)}")

    # Group by checker
    by_checker = {}
    for f in all_findings:
        by_checker.setdefault(f.checker, []).append(f)

    for checker_name, checker_findings in sorted(by_checker.items()):
        print(f"  {checker_name}: {len(checker_findings)} finding(s)")

    # Step 6: View diagnostics
    print("\n" + "=" * 50)
    print("Step 6: Checker diagnostics")
    print("=" * 50)

    diag = proj.checker_diagnostics()
    print(f"  Checkers run: {diag['checkers_run']}")
    print(f"  Source nodes identified: {diag['source_nodes']}")
    print(f"  Sink nodes identified: {diag['sink_nodes']}")
    print(f"  Sanitizer nodes: {diag['sanitizer_nodes']}")
    print(f"  Total findings: {diag['total_findings']}")

    # Step 7: Explore resource table
    print("\n" + "=" * 50)
    print("Step 7: Resource table (built-in knowledge)")
    print("=" * 50)

    table = proj.resource_table()
    print(f"  Built-in entries: {table.size}")
    print(f"  malloc is allocator: {table.has_role('malloc', 'allocator')}")
    print(f"  free is deallocator: {table.has_role('free', 'deallocator')}")
    print(f"  calloc is allocator: {table.has_role('calloc', 'allocator')}")
    print(f"  realloc is allocator: {table.has_role('realloc', 'allocator')}")

    # Summary
    print("\n" + "=" * 50)
    print("Summary")
    print("=" * 50)
    if findings:
        print(f"  MEMORY LEAK DETECTED!")
        print(f"  The checker found {len(findings)} leak(s) in vulnerable.c")
        print(f"  The malloc in create_greeting() returns memory that is never freed.")
    else:
        print("  No memory leaks detected.")
        print("  Note: Detection depends on SVFG precision and analysis scope.")


if __name__ == "__main__":
    main()
