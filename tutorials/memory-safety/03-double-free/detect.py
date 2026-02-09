#!/usr/bin/env python3
"""Detect double-free vulnerabilities using SAF's checker framework.

This tutorial demonstrates double-free detection (CWE-415) using the
SVFG-based checker framework. Double-free occurs when free() is called
twice on the same pointer, corrupting heap metadata.

The vulnerable.c file contains a Buffer type with a clear double-free:
buffer_free() is called twice on the same pointer.

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

    # Step 3: Run double-free checker
    print("\nStep 3: Running double-free checker (CWE-415)...")
    findings = proj.check("double-free")
    print(f"  Findings: {len(findings)}")

    for i, f in enumerate(findings):
        print(f"\n  [{i}] {f.message}")
        print(f"      Severity: {f.severity}")
        print(f"      Checker:  {f.checker}")
        print(f"      Source:   {f.source}")
        print(f"      Sink:     {f.sink}")

    # Step 4: Understand double-free detection
    print("\n\nStep 4: Understanding double-free detection...")
    print("  Double-free uses 'may_reach' reachability mode:")
    print("  - Source: first free() call (pointer becomes invalid)")
    print("  - Sink: second free() call (double-free bug)")
    print("  - Mode: may_reach - if freed pointer CAN reach another free()")
    print("         on any path, it's a potential double-free.")

    # Step 5: Compare with UAF checker
    print("\nStep 5: Compare with use-after-free checker...")
    uaf_findings = proj.check("use-after-free")
    print(f"  use-after-free findings: {len(uaf_findings)}")
    print("  Note: UAF tracks free->use; double-free tracks free->free")

    # Step 6: Run all memory checkers
    print("\nStep 6: Running all memory-related checkers...")
    all_findings = proj.check_all()

    memory_checkers = ["memory-leak", "use-after-free", "double-free", "null-dereference"]
    for checker in memory_checkers:
        checker_findings = [f for f in all_findings if f.checker == checker]
        print(f"  {checker}: {len(checker_findings)} finding(s)")

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY: Double-Free Detection (CWE-415)")
    print(f"  Findings: {len(findings)}")
    if findings:
        print("  DOUBLE-FREE DETECTED in vulnerable.c")
        print("  buffer_free() is called twice on the same pointer,")
        print("  corrupting heap allocator metadata.")
    else:
        print("  No double-free detected.")
        print("  Note: Detection depends on SVFG precision and")
        print("  the ability to track pointer values across calls.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
