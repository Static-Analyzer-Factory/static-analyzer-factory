#!/usr/bin/env python3
"""Detect memory lifecycle violations using SAF's typestate analysis.

This tutorial demonstrates the IDE-based typestate analysis for tracking
memory allocation lifecycle. Unlike the SVFG-based checkers, typestate
analysis uses a formal state machine to track each resource through its
lifecycle: Allocated -> Freed -> Error

The vulnerable.c file contains four functions:
1. alloc_then_leak() - malloc without free (leak)
2. alloc_double_free() - free called twice (double-free)
3. alloc_use_after_free() - dereference after free (UAF)
4. alloc_correct() - proper lifecycle (no bug)

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

    # Step 3: Run memory_alloc typestate analysis
    print("\nStep 3: Running memory_alloc typestate analysis...")
    result = proj.typestate("memory_alloc")
    print(f"  Result: {result}")

    # Step 4: Inspect findings by type
    print("\nStep 4: Inspecting findings by type...")

    # Error-state findings (double-free, use-after-free)
    errors = result.error_findings()
    print(f"\n  Error-state findings (double-free, UAF): {len(errors)}")
    for f in errors:
        print(f"    [{f.kind}] state={f.state}")
        print(f"      Resource: {f.resource}")
        print(f"      Function: {f.function}")
        print(f"      Spec: {f.spec_name}")

    # Non-accepting-at-exit findings (memory leaks)
    leaks = result.leak_findings()
    print(f"\n  Leak findings (non-accepting at exit): {len(leaks)}")
    for f in leaks:
        print(f"    [{f.kind}] state={f.state}")
        print(f"      Resource: {f.resource}")
        print(f"      Function: {f.function}")

    # Step 5: Understand the typestate spec
    print("\nStep 5: Understanding the memory_alloc typestate spec...")
    print("  States:")
    print("    - Allocated: memory has been allocated")
    print("    - Freed: memory has been freed")
    print("    - Error: invalid operation (double-free, UAF)")
    print()
    print("  Transitions:")
    print("    - malloc/calloc/realloc -> Allocated")
    print("    - free (from Allocated) -> Freed (accepting)")
    print("    - free (from Freed) -> Error (double-free)")
    print("    - use (from Freed) -> Error (use-after-free)")
    print()
    print("  Accepting states: [Freed]")
    print("  Non-accepting at exit: leak (resource still in Allocated state)")

    # Step 6: List available typestate specs
    print("\nStep 6: Available built-in typestate specs...")
    for spec_name in saf.typestate_specs():
        print(f"  - {spec_name}")

    # Step 7: IDE solver diagnostics
    print("\nStep 7: IDE solver diagnostics...")
    diag = result.diagnostics()
    print(f"  Jump function updates: {diag['jump_fn_updates']}")
    print(f"  Value propagations: {diag['value_propagations']}")

    # Step 8: Export findings as dictionaries
    print("\nStep 8: Finding details...")
    for f in result.findings()[:3]:
        d = f.to_dict()
        print(f"  Kind: {d['kind']}, State: {d['state']}")

    # Summary
    total = len(result)
    print(f"\n{'='*60}")
    print(f"SUMMARY: Memory Typestate Analysis")
    print(f"  Total findings: {total}")
    print(f"  - Error states (double-free, UAF): {len(errors)}")
    print(f"  - Memory leaks: {len(leaks)}")
    if errors:
        print("  -> Double-free or use-after-free DETECTED")
    if leaks:
        print("  -> Memory leak DETECTED")
    if not errors and not leaks:
        print("  No typestate violations detected.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
