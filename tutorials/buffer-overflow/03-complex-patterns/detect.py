#!/usr/bin/env python3
"""Detect complex buffer overflow patterns using CS-PTA and Z3.

This tutorial demonstrates combining context-sensitive pointer analysis
with Z3-based path sensitivity to detect buffer overflows that involve:
1. Indirect pointer flows through wrapper functions
2. Different allocation sizes at different call sites
3. Path-dependent overflow conditions

The vulnerable.c file has:
- create_buffer() wrapper called with different capacities
- buffer_write() with an off-by-one bounds check
- conditional_overflow() with path-dependent stack overflow

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

    # Step 3: Context-insensitive analysis (baseline)
    print("\n" + "=" * 60)
    print("TECHNIQUE 1: Context-Insensitive PTA")
    print("=" * 60)

    # Run standard buffer overflow checker
    print("\nRunning standard numeric checkers...")
    findings = proj.check_all_numeric()
    print(f"  Findings: {len(findings)}")
    for f in findings:
        if f.severity != "safe":
            print(f"  [{f.severity}] {f.checker} in {f.function}: {f.description}")

    # Step 4: Context-sensitive PTA
    print("\n" + "=" * 60)
    print("TECHNIQUE 2: Context-Sensitive PTA (k=2)")
    print("=" * 60)

    cs_result = proj.context_sensitive_pta(k=2)
    cs_diag = cs_result.diagnostics()
    print(f"\n  Contexts created: {cs_diag['contexts_created']}")
    print(f"  Iterations: {cs_diag['iterations']}")
    print(f"  Converged: {cs_diag['converged']}")

    # Show how CS-PTA distinguishes different call sites
    print("\n  Context-sensitive analysis distinguishes:")
    print("    - create_buffer(32)  -> meta with capacity 32")
    print("    - create_buffer(1024) -> content with capacity 1024")
    print()
    print("  Context-insensitive analysis conflates them, losing precision.")

    # Step 5: Path-sensitive analysis (Z3)
    print("\n" + "=" * 60)
    print("TECHNIQUE 3: Z3-Based Path Sensitivity")
    print("=" * 60)

    # Run all checkers with path sensitivity
    print("\nRunning path-insensitive checkers first...")
    pi_findings = proj.check_all()
    print(f"  Path-insensitive findings: {len(pi_findings)}")

    print("\nApplying Z3 path-sensitive filtering...")
    ps_result = proj.check_all_path_sensitive()
    print(f"  Feasible (real bugs):    {len(ps_result.feasible)}")
    print(f"  Infeasible (filtered):   {len(ps_result.infeasible)}")
    print(f"  Unknown (conservative):  {len(ps_result.unknown)}")

    ps_diag = ps_result.diagnostics
    print(f"\n  Z3 statistics:")
    print(f"    Guards extracted: {ps_diag['guards_extracted']}")
    print(f"    Solver calls:     {ps_diag['z3_calls']}")
    print(f"    Timeouts:         {ps_diag['z3_timeouts']}")

    # Step 6: Explain the combination
    print("\n" + "=" * 60)
    print("COMBINING TECHNIQUES")
    print("=" * 60)
    print()
    print("  For complex buffer overflow patterns, combine:")
    print()
    print("  1. CS-PTA: Distinguishes allocations at different call sites")
    print("     - Needed when wrapper functions create buffers of different sizes")
    print("     - k=2 tracks 2 levels of call context")
    print()
    print("  2. Z3 Path Sensitivity: Filters path-dependent false positives")
    print("     - conditional_overflow(0, 100) is safe (uses heap)")
    print("     - conditional_overflow(1, 100) overflows (uses stack)")
    print("     - Z3 can distinguish these paths by the flag condition")
    print()
    print("  3. Interval Analysis: Precise numeric bounds")
    print("     - Tracks loop counters and size variables")
    print("     - Detects off-by-one errors like buf->size + len < capacity")

    # Step 7: Summary
    total_issues = [f for f in findings if f.severity != "safe"]
    print(f"\n{'='*60}")
    print("SUMMARY: Complex Buffer Overflow Detection")
    print(f"  Numeric checker findings: {len(total_issues)}")
    print(f"  Path-insensitive SVFG findings: {len(pi_findings)}")
    print(f"  Path-sensitive confirmed: {len(ps_result.feasible)}")
    print(f"  False positives filtered: {len(ps_result.infeasible)}")
    print()
    if total_issues or ps_result.feasible:
        print("  POTENTIAL BUFFER OVERFLOW DETECTED")
        print("  The code has:")
        print("    - Off-by-one in buffer_write() bounds check")
        print("    - Path-dependent stack overflow in conditional_overflow()")
        print("    - Memory leaks on error paths in process_input()")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
