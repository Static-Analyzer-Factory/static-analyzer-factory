#!/usr/bin/env python3
"""Detect file I/O typestate bugs using SAF's IDE typestate analysis.

This tutorial demonstrates SAF's typestate analysis, which uses the IDE
framework (Sagiv/Reps/Horwitz TCS'96) to track per-resource state machines
across program paths.

The vulnerable.c file contains four functions:
1. read_config_leak()        — fopen without fclose (file leak)
2. read_config_double_close() — fclose called twice (double-close)
3. read_config_use_after_close() — fread after fclose (use-after-close)
4. read_config_correct()     — proper open/read/close (no bug)

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
            "-o", str(llvm_ir), str(source),
        ],
        check=True,
    )
    print(f"  Compiled: {source} -> {llvm_ir}")

    # Step 2: Load the compiled IR
    print("\nStep 2: Loading via LLVM frontend...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  Project loaded: {proj}")

    # Step 3: Run file I/O typestate analysis
    print("\nStep 3: Running file_io typestate analysis...")
    result = proj.typestate("file_io")
    print(f"  Result: {result}")

    # Step 4: Inspect findings
    print("\nStep 4: Inspecting findings...")

    # Error-state findings (double-close, use-after-close)
    errors = result.error_findings()
    print(f"\n  Error-state findings: {len(errors)}")
    for f in errors:
        print(f"    [{f.kind}] state={f.state}, resource={f.resource}")
        print(f"      spec: {f.spec_name}, at instruction: {f.inst}")

    # Non-accepting-at-exit findings (file leaks)
    leaks = result.leak_findings()
    print(f"\n  Leak findings (non-accepting at exit): {len(leaks)}")
    for f in leaks:
        print(f"    [{f.kind}] state={f.state}, resource={f.resource}")

    # Step 5: Export finding details
    print("\nStep 5: Finding details as dictionaries...")
    for f in result.findings()[:3]:
        d = f.to_dict()
        print(f"  {d}")

    # Step 6: Solver diagnostics
    print("\nStep 6: IDE solver diagnostics...")
    diag = result.diagnostics()
    print(f"  Jump function updates: {diag['jump_fn_updates']}")
    print(f"  Value propagations: {diag['value_propagations']}")

    # Step 7: List available built-in specs
    print("\nStep 7: Available typestate specs...")
    for spec_name in saf.typestate_specs():
        print(f"  - {spec_name}")

    # Summary
    total = len(result)
    print(f"\n{'='*60}")
    print(f"SUMMARY: Found {total} typestate violations")
    print(f"  Error states (double-close, use-after-close): {len(errors)}")
    print(f"  File leaks (non-accepting at exit): {len(leaks)}")
    if errors:
        print("  -> Double-close or use-after-close DETECTED")
    if leaks:
        print("  -> File leak DETECTED")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
