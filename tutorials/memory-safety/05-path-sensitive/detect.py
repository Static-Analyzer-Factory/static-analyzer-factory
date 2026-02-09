#!/usr/bin/env python3
"""Compare path-insensitive and path-sensitive memory safety analysis.

This tutorial demonstrates how Z3-based path-sensitive analysis reduces
false positives in memory safety checking. Many programs have null guards
or conditional cleanup that make certain bug patterns infeasible, but
path-insensitive analysis cannot distinguish these cases.

The vulnerable.c file contains:
- Null-guarded pointer dereferences (infeasible null-deref on guarded paths)
- A genuine use-after-free (accessing freed config entry)

Path-insensitive analysis may flag both; path-sensitive analysis filters
the guarded cases while preserving the real bug.

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

    # Step 2: Load the project
    print("\nStep 2: Loading via LLVM frontend...")
    proj = saf.Project.open(str(llvm_ir))
    print(f"  Project loaded: {proj}")

    # Step 3: Path-insensitive analysis (baseline)
    print("\n" + "=" * 60)
    print("ANALYSIS MODE: Path-Insensitive (baseline)")
    print("=" * 60)
    pi_findings = proj.check_all()
    print(f"\nTotal findings: {len(pi_findings)}")

    by_checker = {}
    for f in pi_findings:
        by_checker.setdefault(f.checker, []).append(f)
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print("\nBreakdown by checker:")
    for checker, findings in sorted(by_checker.items()):
        print(f"  {checker}: {len(findings)}")

    # Step 4: Path-sensitive analysis (Z3-filtered)
    print("\n" + "=" * 60)
    print("ANALYSIS MODE: Path-Sensitive (Z3-filtered)")
    print("=" * 60)
    ps_result = proj.check_all_path_sensitive()
    print(f"\n{ps_result}")

    print(f"\nFeasible findings (CONFIRMED BUGS): {len(ps_result.feasible)}")
    for f in ps_result.feasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nInfeasible findings (FALSE POSITIVES filtered): {len(ps_result.infeasible)}")
    for f in ps_result.infeasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nUnknown findings (kept conservatively): {len(ps_result.unknown)}")
    for f in ps_result.unknown:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    # Step 5: Side-by-side comparison
    print("\n" + "=" * 60)
    print("COMPARISON: Path-Insensitive vs Path-Sensitive")
    print("=" * 60)

    diag = ps_result.diagnostics
    pi_total = len(pi_findings)
    ps_confirmed = len(ps_result.feasible)
    ps_filtered = len(ps_result.infeasible)
    ps_unknown = len(ps_result.unknown)

    print(f"\n  {'Metric':<40} {'PI':>6} {'PS':>6}")
    print(f"  {'-'*40} {'-'*6} {'-'*6}")
    print(f"  {'Total findings reported':<40} {pi_total:>6} {ps_confirmed + ps_unknown:>6}")
    print(f"  {'Confirmed bugs (feasible)':<40} {'N/A':>6} {ps_confirmed:>6}")
    print(f"  {'False positives (infeasible)':<40} {'N/A':>6} {ps_filtered:>6}")
    print(f"  {'Uncertain (unknown)':<40} {'N/A':>6} {ps_unknown:>6}")

    if pi_total > 0:
        reduction_pct = ps_filtered / pi_total * 100
        print(f"\n  False positive reduction: {ps_filtered}/{pi_total} ({reduction_pct:.0f}%)")

    # Step 6: How path-sensitivity works
    print("\nStep 6: How path-sensitive analysis works...")
    print("  1. Run path-insensitive checkers (Stage 1)")
    print("  2. For each finding, extract branch guards along the SVFG trace")
    print("  3. Translate guards to Z3 formulas")
    print("  4. Check satisfiability:")
    print("     - SAT (satisfiable) -> feasible (real bug)")
    print("     - UNSAT (unsatisfiable) -> infeasible (false positive)")
    print("     - timeout/unknown -> kept conservatively")

    # Step 7: Z3 solver statistics
    print("\nStep 7: Z3 solver statistics...")
    print(f"  Guards extracted:          {diag['guards_extracted']}")
    print(f"  Z3 solver calls:           {diag['z3_calls']}")
    print(f"  Z3 timeouts:               {diag['z3_timeouts']}")
    print(f"  Skipped (too many guards): {diag['skipped_too_many_guards']}")

    # Step 8: Post-filter existing findings
    print("\nStep 8: Post-filter with filter_infeasible()...")
    filtered = proj.filter_infeasible(pi_findings)
    print(f"  Input findings:           {len(pi_findings)}")
    print(f"  Feasible output:          {len(filtered.feasible)}")
    print(f"  Infeasible output:        {len(filtered.infeasible)}")
    print(f"  False positives removed:  {filtered.false_positives_filtered}")

    # Summary
    print(f"\n{'='*60}")
    print("CONCLUSION")
    print(f"  Path-insensitive analysis reported {pi_total} finding(s).")
    if ps_filtered > 0:
        print(f"  Path-sensitive analysis filtered {ps_filtered} false positive(s),")
        print(f"  leaving {ps_confirmed} confirmed + {ps_unknown} uncertain finding(s).")
    else:
        print(f"  Path-sensitive analysis confirmed all {ps_confirmed} finding(s).")
    if ps_unknown > 0:
        print(f"  {ps_unknown} finding(s) could not be decided (kept conservatively).")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
