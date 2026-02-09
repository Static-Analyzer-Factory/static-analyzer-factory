#!/usr/bin/env python3
"""Detect integer overflow bugs using SAF's abstract interpretation framework.

This tutorial demonstrates how interval analysis finds integer overflow
(CWE-190) — arithmetic results that exceed the range representable in a
given bit-width. The checker computes the "unwrapped" result of each
arithmetic operation and checks whether it fits within the operand's
signed range [-(2^(n-1)), 2^(n-1) - 1].

The vulnerable.c file contains an image dimension calculator with:
1. A safe addition function (safe_add) that constrains inputs to [0, 100]
2. A buggy compute_image_size() that multiplies width * height * bpp
   without overflow checks
3. A sum_pixels() loop that accumulates without overflow checks

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

    # Step 3: Run abstract interpretation
    print("\nStep 3: Running abstract interpretation...")
    result = proj.abstract_interp()
    diag = result.diagnostics()
    print(f"  Blocks analyzed: {diag['blocks_analyzed']}")
    print(f"  Widening applications: {diag['widening_applications']}")
    print(f"  Converged: {diag['converged']}")
    print(f"  Functions analyzed: {diag['functions_analyzed']}")

    # Step 4: Run integer overflow checker
    print("\nStep 4: Running integer overflow checker (CWE-190)...")
    findings = proj.check_numeric("integer_overflow")
    print(f"  Findings: {len(findings)}")

    warnings = [f for f in findings if f.severity == "warning"]
    errors = [f for f in findings if f.severity == "error"]

    print(f"  Warnings (may overflow): {len(warnings)}")
    print(f"  Errors (definite overflow): {len(errors)}")

    print("\n  Details:")
    for f in findings:
        if f.severity != "safe":
            print(f"    [{f.severity.upper()}] in {f.function}: {f.description}")
            print(f"      Interval: {f.interval}")

    # Step 5: Also run buffer overflow checker for comparison
    print("\nStep 5: Running buffer overflow checker for comparison...")
    bo_findings = proj.check_numeric("buffer_overflow")
    print(f"  Buffer overflow findings: {len(bo_findings)}")

    # Step 6: Run all numeric checkers at once
    print("\nStep 6: Running all numeric checkers...")
    all_findings = proj.check_all_numeric()
    print(f"  Total findings: {len(all_findings)}")
    by_checker: dict[str, list] = {}
    for f in all_findings:
        by_checker.setdefault(f.checker, []).append(f)
    for checker, checker_findings in sorted(by_checker.items()):
        non_safe = [cf for cf in checker_findings if cf.severity != "safe"]
        print(f"    {checker}: {len(non_safe)} non-safe finding(s)")

    # Step 7: Explore the abstract state
    print("\nStep 7: Exploring abstract interpretation results...")
    export = result.export()
    print(f"  Block states: {export['block_count']}")
    print(f"  Instruction states: {export['inst_count']}")
    print(f"  Schema: {export['schema']}")

    # Summary
    non_safe = [f for f in findings if f.severity != "safe"]
    print(f"\n{'='*60}")
    print("SUMMARY: Integer Overflow Analysis (CWE-190)")
    print(f"  Arithmetic operations checked: {len(findings)}")
    print(f"  Potential issues found: {len(non_safe)}")
    if non_safe:
        print("  POTENTIAL INTEGER OVERFLOW DETECTED in vulnerable.c")
        print("  The multiply in compute_image_size() can overflow when")
        print("  width and height are large. The sum in sum_pixels()")
        print("  can also overflow for large pixel counts.")
    else:
        print("  No integer overflow issues detected.")
        print("  Note: The checker analyses arithmetic intervals.")
        print("  Functions with unknown-range parameters (e.g., function")
        print("  args mapped to top) produce top-range results, which may")
        print("  generate warnings for any arithmetic operation.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
