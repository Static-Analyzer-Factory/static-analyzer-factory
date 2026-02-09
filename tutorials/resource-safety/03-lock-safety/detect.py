#!/usr/bin/env python3
"""Detect mutex lock typestate bugs using SAF's IDE typestate analysis.

This tutorial demonstrates SAF's typestate analysis for pthread mutex
operations. The IDE framework tracks per-resource state machines to
detect:
- Lock acquired but never released (held lock at exit)
- Double-lock (lock while already locked)
- Unlock without prior lock

The vulnerable.cpp file contains:
1. acquire_no_release()       — lock without unlock (held lock)
2. acquire_release_correct()  — proper lock/unlock (no bug)

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

import saf


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.cpp"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C++ source to LLVM IR
    print("Step 1: Compiling C++ source to LLVM IR...")
    subprocess.run(
        [
            "clang++-18", "-S", "-emit-llvm", "-O0", "-g0",
            "-o", str(llvm_ir), str(source),
        ],
        check=True,
    )
    print(f"  Compiled: {source} -> {llvm_ir}")

    # Step 2: Load the compiled IR
    print("\nStep 2: Loading via LLVM frontend...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  Project loaded: {proj}")

    # Step 3: Run mutex_lock typestate analysis
    print("\nStep 3: Running mutex_lock typestate analysis...")
    result = proj.typestate("mutex_lock")
    print(f"  Result: {result}")

    # Step 4: Inspect findings
    print("\nStep 4: Inspecting findings...")

    leaks = result.leak_findings()
    print(f"\n  Held-lock findings (non-accepting at exit): {len(leaks)}")
    for f in leaks:
        print(f"    [{f.kind}] state={f.state}, resource={f.resource}")
        print(f"      spec: {f.spec_name}")

    errors = result.error_findings()
    print(f"\n  Error-state findings: {len(errors)}")
    for f in errors:
        print(f"    [{f.kind}] state={f.state}")

    # Step 5: Solver diagnostics
    print("\nStep 5: IDE solver diagnostics...")
    diag = result.diagnostics()
    print(f"  Jump function updates: {diag['jump_fn_updates']}")
    print(f"  Value propagations: {diag['value_propagations']}")

    # Step 6: Create a custom typestate spec
    print("\nStep 6: Custom typestate spec example...")
    custom_spec = saf.TypestateSpec(
        name="custom_mutex",
        states=["uninit", "unlocked", "locked", "error"],
        initial_state="unlocked",
        error_states=["error"],
        accepting_states=["uninit", "unlocked"],
        transitions=[
            ("unlocked", "pthread_mutex_lock", "locked"),
            ("locked", "pthread_mutex_unlock", "unlocked"),
            ("locked", "pthread_mutex_lock", "error"),
            ("unlocked", "pthread_mutex_unlock", "error"),
        ],
        constructors=["pthread_mutex_init"],
    )
    custom_result = proj.typestate_custom(custom_spec)
    print(f"  Custom spec result: {custom_result}")

    # Summary
    total = len(result)
    print(f"\n{'='*60}")
    print(f"SUMMARY: Found {total} mutex typestate violations")
    if leaks:
        print(f"  Held locks at exit: {len(leaks)}")
        print("  -> Lock leak DETECTED")
    if errors:
        print(f"  Error states: {len(errors)}")
        print("  -> Mutex misuse DETECTED")
    if not leaks and not errors:
        print("  No violations found.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
