#!/usr/bin/env python3
"""Tutorial 7: Flow-Sensitive PTA — comparing Andersen vs flow-sensitive.

Demonstrates how flow-sensitive pointer analysis achieves higher precision
than Andersen's flow-insensitive analysis through strong updates.

A "connection" pointer is first assigned to a secret connection, then
reassigned to a public connection.  Andersen's analysis merges both
assignments and reports the pointer may target either connection.
Flow-sensitive PTA recognises the strong update and narrows the
points-to set at the load site to only the public connection.

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load and analyze
    proj = Project.open(str(llvm_ir))

    # Step 3: Andersen (flow-insensitive) PTA
    pta = proj.pta_result()
    pta_export = pta.export()
    print("=== Andersen (flow-insensitive) PTA ===")
    print(f"  Values with points-to info: {len(pta_export['points_to'])}")
    print(f"  Abstract locations: {len(pta_export['locations'])}")

    # Step 4: Flow-sensitive PTA
    fs = proj.flow_sensitive_pta()
    fs_diag = fs.diagnostics()
    print("\n=== Flow-Sensitive PTA ===")
    print(f"  Solver iterations: {fs_diag['iterations']}")
    print(f"  Converged: {not fs_diag['iteration_limit_hit']}")
    print(f"  Strong updates: {fs_diag['strong_updates']}")
    print(f"  Weak updates: {fs_diag['weak_updates']}")
    print(f"  FsSvfg nodes: {fs_diag['fs_svfg_nodes']}")
    print(f"  FsSvfg edges: {fs_diag['fs_svfg_edges']}")
    print(f"  Store nodes: {fs_diag['store_nodes']}")
    print(f"  Load nodes: {fs_diag['load_nodes']}")

    # Step 5: Export and compare
    fs_export = fs.export()
    print(f"\n=== Export ===")
    print(f"  Schema version: {fs_export['schema_version']}")
    print(f"  Points-to entries in export: {len(fs_export['points_to'])}")

    # Step 6: Summary
    print("\n=== Summary ===")
    print("  Andersen (flow-insensitive):")
    print("    Merges all program points — `conn` may point to")
    print("    {secret_conn, pub_conn} everywhere.")
    print("  Flow-sensitive:")
    print("    Tracks per-program-point. After `conn = &pub_conn`,")
    print("    the strong update kills secret_conn from the set.")
    print("    At the load site, conn -> {pub_conn} only.")
    if fs_diag["strong_updates"] > 0:
        print(f"    (Performed {fs_diag['strong_updates']} strong update(s))")


if __name__ == "__main__":
    main()
