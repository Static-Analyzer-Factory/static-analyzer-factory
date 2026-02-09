#!/usr/bin/env python3
"""Detect file descriptor leaks using SAF's checker framework.

This tutorial demonstrates detecting file handle leaks (CWE-775) using
SAF's SVFG-based reachability analysis. The checker finds paths where
fopen() returns a file handle that reaches program exit without fclose().

The vulnerable.c file contains a config file processor with:
- File handle leak on malloc failure (fopen without fclose)
- Memory leak on error path (second allocation fails)

CWE-775: Missing Release of File Descriptor or Handle

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

    # Step 1: Compile C source to LLVM IR
    print("Step 1: Compiling C source to LLVM IR...")
    subprocess.run(
        [
            "clang-18", "-S", "-emit-llvm", "-O0", "-g0",
            "-Xclang", "-disable-O0-optnone",
            "-fno-discard-value-names",
            "-o", str(llvm_ir), str(source),
        ],
        check=True,
    )
    print(f"  Compiled: {source} -> {llvm_ir}")

    # Step 2: Load the compiled IR
    print("\nStep 2: Loading via LLVM frontend...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  Project loaded: {proj}")

    # Step 3: Run file descriptor leak checker specifically
    print("\nStep 3: Running file descriptor leak checker (CWE-775)...")
    fd_findings = proj.check("file-descriptor-leak")
    print(f"  Findings: {len(fd_findings)}")

    if fd_findings:
        print("\n  File descriptor leak details:")
        for i, f in enumerate(fd_findings):
            print(f"    [{i+1}] Severity: {f.severity}")
            print(f"        Message: {f.message}")
            if f.cwe:
                print(f"        CWE-{f.cwe}")

    # Step 4: Explain the checker's approach
    print("\nStep 4: Understanding the checker...")
    print("  The 'file-descriptor-leak' checker uses SVFG reachability:")
    print("    - Source: Return value of fopen()")
    print("    - Sink: Function exit points")
    print("    - Sanitizer: Arguments to fclose()")
    print("    - Mode: must_not_reach (leak if source reaches sink)")

    # Step 5: View the checker's configuration
    print("\nStep 5: Checker configuration...")
    schema = proj.checker_schema()
    for c in schema["checkers"]:
        if c["name"] == "file-descriptor-leak":
            print(f"  Name: {c['name']}")
            print(f"  Mode: {c['mode']}")
            print(f"  CWE: {c.get('cwe', 'N/A')}")
            print(f"  Description: File handle reaches exit without fclose()")

    # Step 6: Compare with running all checkers
    print("\nStep 6: Running all built-in checkers for comparison...")
    all_findings = proj.check_all()
    print(f"  Total findings from all checkers: {len(all_findings)}")

    by_checker = {}
    for f in all_findings:
        by_checker.setdefault(f.checker, []).append(f)

    print("  Findings by checker:")
    for checker, findings in sorted(by_checker.items()):
        print(f"    {checker}: {len(findings)} finding(s)")

    # Step 7: Explore the resource table
    print("\nStep 7: Built-in resource table...")
    table = proj.resource_table()
    print(f"  Built-in entries: {table.size}")
    print(f"  fopen is file_opener: {table.has_role('fopen', 'file_opener')}")
    print(f"  fclose is file_closer: {table.has_role('fclose', 'file_closer')}")

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY: File Descriptor Leak Detection (CWE-775)")
    print(f"  File descriptor leaks found: {len(fd_findings)}")

    if fd_findings:
        print("\n  VULNERABILITY DETECTED in vulnerable.c")
        print("  The read_config_value() function has a path where:")
        print("    1. fopen() opens a file successfully")
        print("    2. malloc() fails, triggering early return")
        print("    3. fclose() is never called")
        print("  This leaks the file descriptor.")
    else:
        print("  No file descriptor leaks detected.")
        print("  Note: Detection depends on SVFG precision.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
