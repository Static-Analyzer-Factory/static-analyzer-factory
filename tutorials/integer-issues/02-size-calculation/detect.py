#!/usr/bin/env python3
"""Detect allocation size overflow bugs using SAF's abstract interpretation.

This tutorial demonstrates how integer overflow in size calculations leads
to undersized allocations. The checker uses interval analysis to track
the range of values flowing into allocation functions.

The vulnerable.c file contains several allocation patterns:
1. Simple width * height overflow
2. Multi-factor multiplication overflow (width * height * channels * bpp)
3. Array count * sizeof overflow
4. Reallocation growth factor overflow
5. Stride calculation overflow

CWE-190: Integer Overflow or Wraparound
CWE-122: Heap-based Buffer Overflow (consequence)

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

    # Step 4: Run all numeric checkers to find both overflow and allocation issues
    print("\nStep 4: Running all numeric checkers...")
    all_findings = proj.check_all_numeric()

    # Categorize findings
    by_checker: dict[str, list] = {}
    for f in all_findings:
        by_checker.setdefault(f.checker, []).append(f)

    print(f"  Total findings: {len(all_findings)}")
    for checker, checker_findings in sorted(by_checker.items()):
        non_safe = [cf for cf in checker_findings if cf.severity != "safe"]
        print(f"    {checker}: {len(non_safe)} non-safe finding(s)")

    # Step 5: Focus on integer overflow findings
    print("\nStep 5: Integer overflow analysis (CWE-190)...")
    overflow_findings = proj.check_numeric("integer_overflow")
    non_safe_overflow = [f for f in overflow_findings if f.severity != "safe"]

    print(f"  Arithmetic operations checked: {len(overflow_findings)}")
    print(f"  Potential overflow issues: {len(non_safe_overflow)}")

    # Group by function to see which functions are problematic
    by_function: dict[str, list] = {}
    for f in non_safe_overflow:
        by_function.setdefault(f.function, []).append(f)

    print("\n  Findings by function:")
    for func, findings in sorted(by_function.items()):
        warnings = [f for f in findings if f.severity == "warning"]
        errors = [f for f in findings if f.severity == "error"]
        print(f"    {func}: {len(warnings)} warning(s), {len(errors)} error(s)")

    # Step 6: Show detailed findings for allocation functions
    print("\nStep 6: Detailed findings in allocation functions...")
    allocation_funcs = [
        "allocate_pixel_buffer",
        "allocate_rgba_buffer",
        "allocate_pixel_array",
        "grow_buffer",
        "allocate_image_with_stride",
    ]

    for func in allocation_funcs:
        if func in by_function:
            print(f"\n  {func}:")
            for f in by_function[func]:
                print(f"    [{f.severity.upper()}] {f.description}")
                print(f"      Interval: {f.interval}")

    # Step 7: Check the safe version
    print("\nStep 7: Checking the safe implementation...")
    safe_func = "allocate_pixel_buffer_safe"
    if safe_func in by_function:
        print(f"  {safe_func} has {len(by_function[safe_func])} finding(s)")
        for f in by_function[safe_func]:
            print(f"    [{f.severity.upper()}] {f.description}")
    else:
        print(f"  {safe_func}: No overflow findings (properly checked)")

    # Step 8: Export abstract interpretation state
    print("\nStep 8: Abstract interpretation state summary...")
    export = result.export()
    print(f"  Block states: {export['block_count']}")
    print(f"  Instruction states: {export['inst_count']}")
    print(f"  Schema: {export['schema']}")

    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY: Allocation Size Overflow Analysis")
    print(f"  Functions analyzed: {diag['functions_analyzed']}")
    print(f"  Arithmetic operations: {len(overflow_findings)}")
    print(f"  Potential overflows: {len(non_safe_overflow)}")

    if non_safe_overflow:
        print("\n  POTENTIAL ALLOCATION SIZE OVERFLOW DETECTED")
        print("  The following functions have overflow risks:")
        for func in sorted(by_function.keys()):
            if func != safe_func:
                count = len(by_function[func])
                print(f"    - {func}: {count} issue(s)")
        print("\n  Impact: Integer overflow in size calculations can")
        print("  cause undersized allocations, leading to heap buffer")
        print("  overflows when the full expected data is written.")
    else:
        print("  No allocation size overflow issues detected.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
