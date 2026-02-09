#!/usr/bin/env python3
"""Detect buffer overflow bugs using SAF's abstract interpretation framework.

This tutorial demonstrates the interval analysis (abstract interpretation)
approach to finding buffer overflows (CWE-120). Instead of tracking data flow
like the SVFG-based checkers in tutorials 01-02, abstract interpretation
computes numeric value ranges at every program point and checks whether
array indices may exceed buffer boundaries.

The vulnerable.c file contains an HTTP request path parser with:
1. A safe version (parse_path_safe) that correctly bounds its loop
2. A buggy version (parse_path_overflow) with an off-by-one error

Abstract interpretation tracks the loop counter's interval [lo, hi]
through each iteration, using widening to ensure convergence at loop
headers and narrowing to recover precision.

Usage:
    python detect.py
"""

import subprocess
import sys
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
    print(f"  Narrowing iterations: {diag['narrowing_iterations_performed']}")
    print(f"  Converged: {diag['converged']}")
    print(f"  Functions analyzed: {diag['functions_analyzed']}")

    # Step 4: Run buffer overflow checker
    print("\nStep 4: Running buffer overflow checker (CWE-120)...")
    findings = proj.check_numeric("buffer_overflow")
    print(f"  Findings: {len(findings)}")

    warnings = [f for f in findings if f.severity == "warning"]
    errors = [f for f in findings if f.severity == "error"]
    safe = [f for f in findings if f.severity == "safe"]

    print(f"  Safe GEP accesses: {len(safe)}")
    print(f"  Warnings (may overflow): {len(warnings)}")
    print(f"  Errors (definite overflow): {len(errors)}")

    print("\n  Details:")
    for f in findings:
        if f.severity != "safe":
            print(f"    [{f.severity.upper()}] in {f.function}: {f.description}")
            print(f"      Interval: {f.interval}")

    # Step 5: Run all numeric checkers
    print("\nStep 5: Running all numeric checkers (buffer overflow + integer overflow)...")
    all_findings = proj.check_all_numeric()
    print(f"  Total findings: {len(all_findings)}")
    by_checker: dict[str, list] = {}
    for f in all_findings:
        by_checker.setdefault(f.checker, []).append(f)
    for checker, checker_findings in sorted(by_checker.items()):
        print(f"    {checker}: {len(checker_findings)} finding(s)")

    # Step 6: Explore computed invariants
    print("\nStep 6: Exploring computed invariants...")
    export = result.export()
    n_blocks = export["block_count"]
    n_insts = export["inst_count"]
    n_invariants = sum(len(v) for v in export["block_invariants"].values())
    print(f"  Block states: {n_blocks}")
    print(f"  Instruction states: {n_insts}")
    print(f"  Total value-interval pairs: {n_invariants}")

    # Show a few block invariants
    print("\n  Sample block invariants:")
    count = 0
    for block_hex, values in export["block_invariants"].items():
        if count >= 3:
            break
        print(f"    Block {block_hex[:18]}...:")
        for val_hex, interval_str in list(values.items())[:3]:
            print(f"      {val_hex[:18]}... = {interval_str}")
        count += 1

    # Summary
    non_safe = [f for f in findings if f.severity != "safe"]
    print(f"\n{'='*60}")
    print(f"SUMMARY: Buffer Overflow Analysis (CWE-120)")
    print(f"  Total GEP operations checked: {len(findings)}")
    print(f"  Potential issues found: {len(non_safe)}")
    if non_safe:
        print("  POTENTIAL BUFFER OVERFLOW DETECTED in vulnerable.c")
        print("  The off-by-one error in parse_path_overflow allows")
        print("  the loop index to reach MAX_PATH_LEN (128),")
        print("  writing one byte past the buffer boundary.")
    else:
        print("  No buffer overflow issues detected.")
        print("  Note: The checker analyses GEP index intervals.")
        print("  Results depend on the precision of interval widening.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
