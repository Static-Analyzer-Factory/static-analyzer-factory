#!/usr/bin/env python3
"""Z3-refined taint analysis: filter infeasible cross-branch taint flows.

This tutorial demonstrates how Z3 SMT solver refinement eliminates
false-positive taint flows caused by branch-dependent sanitization.

The vulnerable.c contains an HTTP request handler where:
- GET requests sanitize input before use (no bug)
- POST requests pass raw input to system() (genuine bug)

Path-insensitive analysis merges both paths and reports both as tainted.
Z3 refinement checks branch feasibility and filters the sanitized path.

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

    # Step 2: Load project
    print("\nStep 2: Loading project...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  {proj}")

    # Step 3: Path-insensitive taint analysis (baseline)
    print("\n" + "=" * 60)
    print("ANALYSIS: Path-Insensitive Taint (baseline)")
    print("=" * 60)
    pi_findings = proj.check_all()
    print(f"\nTotal findings: {len(pi_findings)}")
    for f in pi_findings:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    # Step 4: Path-sensitive analysis with Z3 refinement
    print("\n" + "=" * 60)
    print("ANALYSIS: Path-Sensitive Taint (Z3-refined)")
    print("=" * 60)
    ps_result = proj.check_all_path_sensitive(z3_timeout_ms=2000, max_guards=64)
    print(f"\n{ps_result}")

    print(f"\nConfirmed bugs (feasible): {len(ps_result.feasible)}")
    for f in ps_result.feasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nFalse positives (infeasible): {len(ps_result.infeasible)}")
    for f in ps_result.infeasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nUncertain (unknown): {len(ps_result.unknown)}")
    for f in ps_result.unknown:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    # Step 5: Comparison
    print("\n" + "=" * 60)
    print("COMPARISON")
    print("=" * 60)
    diag = ps_result.diagnostics
    pi_total = len(pi_findings)
    ps_confirmed = len(ps_result.feasible)
    ps_filtered = len(ps_result.infeasible)

    print(f"\n  Path-insensitive findings: {pi_total}")
    print(f"  Path-sensitive confirmed:  {ps_confirmed}")
    print(f"  False positives filtered:  {ps_filtered}")

    if pi_total > 0 and ps_filtered > 0:
        pct = ps_filtered / pi_total * 100
        print(f"\n  False positive reduction: {ps_filtered}/{pi_total} ({pct:.0f}%)")

    print(f"\n  Z3 solver calls: {diag['z3_calls']}")
    print(f"  Z3 timeouts:     {diag['z3_timeouts']}")


if __name__ == "__main__":
    main()
