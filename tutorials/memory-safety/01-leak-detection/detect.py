#!/usr/bin/env python3
"""Detect memory leaks using SAF's checker framework.

This tutorial focuses specifically on memory leak detection (CWE-401) using
the SVFG-based checker framework. Memory leaks occur when allocated memory
is never freed, eventually exhausting available memory.

The vulnerable.c file contains a simplified HTTP header parser where:
- parse_header() allocates h->name and h->value, but on allocation failure
  only frees h, leaking the partially allocated fields.

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
    print(f"  Compiled: {source.name} -> {llvm_ir.name}")

    # Step 2: Load the compiled IR
    print("\nStep 2: Loading via LLVM frontend...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  Project loaded: {proj}")

    # Step 3: Run memory leak checker
    print("\nStep 3: Running memory leak checker (CWE-401)...")
    findings = proj.check("memory-leak")
    print(f"  Findings: {len(findings)}")

    for i, f in enumerate(findings):
        print(f"\n  [{i}] {f.message}")
        print(f"      Severity: {f.severity}")
        print(f"      Checker:  {f.checker}")
        print(f"      Source:   {f.source}")
        print(f"      Sink:     {f.sink}")
        print(f"      Trace length: {len(f.trace)}")

    # Step 4: Understand the detection technique
    print("\n\nStep 4: Understanding the detection technique...")
    print("  Memory leak detection uses 'must_not_reach' reachability mode:")
    print("  - Source: malloc() return value (allocated pointer)")
    print("  - Sink (sanitizer): free() argument (deallocated pointer)")
    print("  - A leak occurs when the pointer reaches function exit")
    print("    WITHOUT passing through free() on ALL paths.")

    # Step 5: Inspect checker schema
    print("\nStep 5: Checker schema for memory-leak...")
    schema = proj.checker_schema()
    for c in schema["checkers"]:
        if c["name"] == "memory-leak":
            print(f"  Name: {c['name']}")
            print(f"  CWE:  CWE-{c['cwe']}")
            print(f"  Mode: {c['mode']}")
            print(f"  Severity: {c['severity']}")
            break

    # Step 6: Resource table entries
    print("\nStep 6: Resource table entries for allocators/deallocators...")
    table = proj.resource_table()
    print(f"  malloc is allocator: {table.has_role('malloc', 'allocator')}")
    print(f"  free is deallocator: {table.has_role('free', 'deallocator')}")

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY: Memory Leak Detection (CWE-401)")
    print(f"  Findings: {len(findings)}")
    if findings:
        print("  MEMORY LEAK DETECTED in vulnerable.c")
        print("  The partial cleanup in parse_header() leaks h->name")
        print("  when h->value allocation fails.")
    else:
        print("  No memory leaks detected.")
        print("  Note: Detection depends on SVFG precision.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
