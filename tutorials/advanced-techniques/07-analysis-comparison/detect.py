#!/usr/bin/env python3
"""Comprehensive Z3 analysis comparison across all analysis types.

This tutorial combines:
1. Path-insensitive vs path-sensitive analysis comparison
2. Z3 assertion proving for static verification
3. Multiple analysis categories (memory, typestate, numeric, taint)

The vulnerable.c exercises four analysis categories:
- Memory safety (correlated malloc/free vs. genuine leak)
- Typestate / file I/O (correlated open/close vs. genuine leak)
- Numeric (guarded array access vs. genuine overflow)
- Taint (sanitized vs. unsanitized command injection)

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

    # Step 1: Compile
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

    # ===== PART A: Path-Insensitive Analysis (Baseline) =====
    print("\n" + "=" * 70)
    print("PART A: Path-Insensitive Analysis (Baseline)")
    print("=" * 70)
    pi_findings = proj.check_all()
    pi_total = len(pi_findings)
    print(f"\nTotal findings: {pi_total}")

    by_checker_pi = {}
    for f in pi_findings:
        by_checker_pi.setdefault(f.checker, []).append(f)
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    # ===== PART B: Path-Sensitive Analysis (Z3-Refined) =====
    print("\n" + "=" * 70)
    print("PART B: Path-Sensitive Analysis (Z3-Refined)")
    print("=" * 70)
    ps_result = proj.check_all_path_sensitive(z3_timeout_ms=2000, max_guards=64)
    print(f"\n{ps_result}")

    ps_feasible = len(ps_result.feasible)
    ps_infeasible = len(ps_result.infeasible)
    ps_unknown = len(ps_result.unknown)

    print(f"\nConfirmed bugs: {ps_feasible}")
    for f in ps_result.feasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nFiltered (false positives): {ps_infeasible}")
    for f in ps_result.infeasible:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    print(f"\nUncertain: {ps_unknown}")
    for f in ps_result.unknown:
        print(f"  [{f.severity}] {f.checker}: {f.message}")

    # ===== PART C: Assertion Proving =====
    print("\n" + "=" * 70)
    print("PART C: Z3 Assertion Proving")
    print("=" * 70)
    assertion_result = proj.prove_assertions(z3_timeout_ms=2000, max_guards=64)
    a_diag = assertion_result.diagnostics
    print(f"\n  Total assertions: {a_diag['total_assertions']}")
    print(f"  Proven:           {a_diag['proven_count']}")
    print(f"  May fail:         {a_diag['may_fail_count']}")
    print(f"  Unknown:          {a_diag['unknown_count']}")

    if assertion_result.proven:
        print("\n  Proven assertions:")
        for a in assertion_result.proven[:5]:
            d = a.to_dict()
            print(f"    {d['function']}: always holds")
        if len(assertion_result.proven) > 5:
            print(f"    ... and {len(assertion_result.proven) - 5} more")

    if assertion_result.may_fail:
        print("\n  May-fail assertions:")
        for a in assertion_result.may_fail[:5]:
            d = a.to_dict()
            print(f"    {d['function']}: may fail")
            if d.get('counterexample'):
                print(f"      Counterexample: {d['counterexample']}")
        if len(assertion_result.may_fail) > 5:
            print(f"    ... and {len(assertion_result.may_fail) - 5} more")

    # ===== PART D: Side-by-Side Comparison Table =====
    print("\n" + "=" * 70)
    print("PART D: Comparison Table")
    print("=" * 70)

    diag = ps_result.diagnostics

    print(f"\n  {'Category':<30} {'PI':>6} {'PS':>6} {'Saved':>6}")
    print(f"  {'-'*30} {'-'*6} {'-'*6} {'-'*6}")

    for checker, pi_list in sorted(by_checker_pi.items()):
        ps_count = sum(
            1 for f in ps_result.feasible if f.checker == checker
        ) + sum(
            1 for f in ps_result.unknown if f.checker == checker
        )
        saved = len(pi_list) - ps_count
        print(f"  {checker:<30} {len(pi_list):>6} {ps_count:>6} {saved:>6}")

    print(f"  {'-'*30} {'-'*6} {'-'*6} {'-'*6}")
    print(f"  {'TOTAL':<30} {pi_total:>6} {ps_feasible + ps_unknown:>6} {ps_infeasible:>6}")

    # ===== PART E: Z3 Solver Statistics =====
    print("\n" + "=" * 70)
    print("PART E: Z3 Solver Statistics")
    print("=" * 70)
    print(f"  Guards extracted:          {diag['guards_extracted']}")
    print(f"  Z3 solver calls:           {diag['z3_calls']}")
    print(f"  Z3 timeouts:               {diag['z3_timeouts']}")
    print(f"  Skipped (too many guards): {diag['skipped_too_many_guards']}")

    # ===== Summary =====
    print("\n" + "=" * 70)
    print("CONCLUSION")
    print("=" * 70)

    print("\n  Analysis Techniques Demonstrated:")
    print("  1. Path-insensitive SVFG reachability (fast, may over-report)")
    print("  2. Path-sensitive Z3 refinement (precise, filters infeasible paths)")
    print("  3. Z3 assertion proving (static verification of invariants)")

    if ps_infeasible > 0:
        pct = ps_infeasible / pi_total * 100 if pi_total > 0 else 0
        print(f"\n  Z3 refinement filtered {ps_infeasible} false positive(s)")
        print(f"  out of {pi_total} total finding(s) ({pct:.0f}% reduction).")
        print(f"  {ps_feasible} confirmed bug(s) remain for developer review.")
    else:
        print(f"\n  All {pi_total} finding(s) confirmed as genuine bugs.")

    proven = a_diag['proven_count']
    total_assert = a_diag['total_assertions']
    if total_assert > 0:
        print(f"  {proven}/{total_assert} assertion(s) proven to always hold.")

    print("\n  Key Insight:")
    print("    Combining multiple analysis techniques provides both")
    print("    broad coverage (path-insensitive) and high precision")
    print("    (path-sensitive Z3 refinement).")


if __name__ == "__main__":
    main()
