#!/usr/bin/env python3
"""Detect custom resource bugs using user-defined specifications.

This tutorial demonstrates how to define custom resource specifications
for domain-specific resources that SAF doesn't have built-in support for.

The vulnerable.c file contains a simulated database connection API with:
1. Connection leak: db_connect without db_disconnect on error path
2. Use-after-disconnect: db_query after db_disconnect
3. Double-disconnect: calling db_disconnect twice

We'll define a custom typestate spec for db_connect/db_disconnect and
also demonstrate the check_custom() API.

CWE-772: Missing Release of Resource after Effective Lifetime
CWE-416: Use After Free (applied to non-memory resources)

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

    # Step 3: Define a custom typestate specification for database connections
    print("\nStep 3: Defining custom typestate spec for database connections...")
    db_spec = saf.TypestateSpec(
        name="db_connection",
        states=["uninit", "connected", "disconnected", "error"],
        initial_state="uninit",
        error_states=["error"],
        accepting_states=["uninit", "disconnected"],
        transitions=[
            # Normal lifecycle
            ("uninit", "db_connect", "connected"),
            ("connected", "db_query", "connected"),  # Query keeps state
            ("connected", "db_disconnect", "disconnected"),

            # Error transitions
            ("disconnected", "db_query", "error"),       # Use-after-disconnect
            ("disconnected", "db_disconnect", "error"),  # Double-disconnect
        ],
        constructors=["db_connect"],
    )
    print(f"  Spec name: {db_spec.name}")
    print(f"  States: {db_spec.states}")
    print(f"  Error states: {db_spec.error_states}")
    print(f"  Accepting states: {db_spec.accepting_states}")
    print(f"  Transitions: {len(db_spec.transitions)}")

    # Step 4: Run typestate analysis with the custom spec
    print("\nStep 4: Running typestate analysis with custom spec...")
    result = proj.typestate_custom(db_spec)
    print(f"  Result: {result}")

    # Step 5: Inspect findings
    print("\nStep 5: Inspecting findings...")

    leaks = result.leak_findings()
    print(f"\n  Leak findings (connection not closed): {len(leaks)}")
    for f in leaks:
        print(f"    [{f.kind}] state={f.state}, resource={f.resource}")

    errors = result.error_findings()
    print(f"\n  Error findings (use-after-disconnect, double-disconnect): {len(errors)}")
    for f in errors:
        print(f"    [{f.kind}] state={f.state}, resource={f.resource}")
        print(f"      at instruction: {f.inst}")

    # Step 6: Also demonstrate check_custom for SVFG-based detection
    print("\nStep 6: Using check_custom() for SVFG-based leak detection...")

    # First, add our custom resource to the resource table
    table = proj.resource_table()
    print(f"  Built-in resource table size: {table.size}")

    # Add custom resource roles
    table.add("db_connect", saf.Allocator)
    table.add("db_disconnect", saf.Deallocator)
    print(f"  After adding db_connect/db_disconnect: {table.size}")

    # Run a custom check for connection leaks
    custom_findings = proj.check_custom(
        "db-connection-leak",
        mode=saf.MustNotReach,
        source_role=saf.Allocator,
        source_match_return=True,  # Track return value of db_connect
        sink_is_exit=True,         # Check if reaches function exit
        sanitizer_role=saf.Deallocator,  # db_disconnect sanitizes
        sanitizer_match_return=False,
        cwe=772,
        severity=saf.Warning,
    )
    print(f"\n  check_custom findings: {len(custom_findings)}")
    for f in custom_findings:
        print(f"    [{f.severity}] {f.message}")
        d = f.to_dict()
        print(f"      CWE: {d.get('cwe', 'N/A')}")

    # Step 7: Compare the two approaches
    print("\nStep 7: Comparing approaches...")
    print("  Typestate analysis (typestate_custom):")
    print(f"    - Detects leaks: {len(leaks)}")
    print(f"    - Detects use-after-disconnect/double-disconnect: {len(errors)}")
    print("  SVFG reachability (check_custom):")
    print(f"    - Detects leaks: {len(custom_findings)}")
    print("    - Cannot detect use-after-disconnect (no state tracking)")

    # Step 8: List available built-in typestate specs
    print("\nStep 8: Available built-in typestate specs...")
    for spec_name in saf.typestate_specs():
        print(f"  - {spec_name}")

    # Summary
    total = len(leaks) + len(errors)
    print(f"\n{'='*60}")
    print("SUMMARY: Custom Resource Analysis")
    print(f"  Typestate violations found: {total}")
    print(f"    Connection leaks: {len(leaks)}")
    print(f"    Use-after-disconnect / double-disconnect: {len(errors)}")
    print(f"  SVFG-based findings: {len(custom_findings)}")

    if total > 0 or custom_findings:
        print("\n  VULNERABILITIES DETECTED in vulnerable.c")
        print("  The code has the following bugs:")
        if leaks:
            print("    - Connection leak in process_data_leak()")
        if errors:
            print("    - Use-after-disconnect in process_data_use_after_close()")
            print("    - Double-disconnect in process_data_double_close()")
    else:
        print("  No custom resource issues detected.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
