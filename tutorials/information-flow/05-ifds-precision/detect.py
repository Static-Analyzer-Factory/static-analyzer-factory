#!/usr/bin/env python3
"""Detect interprocedural command injection using IFDS taint analysis.

This tutorial demonstrates the IFDS (Interprocedural Finite Distributive
Subset) taint analyzer, which uses the Reps/Horwitz/Sagiv tabulation
algorithm for precise interprocedural taint tracking.

Compared to the BFS-based taint_flow query, IFDS:
- Tracks taint precisely through function calls and returns
- Builds summary edges so repeated callee analysis is avoided
- Provides fact-level results (which values are tainted at each point)

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project, sources, sinks


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.c"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g0", "-fno-discard-value-names",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load via LLVM frontend
    proj = Project.open(str(llvm_ir))

    # Step 3: Run IFDS taint analysis
    #   SOURCE: getenv() return value
    #   SINK:   first argument to system()
    result = proj.ifds_taint(
        sources=sources.call("getenv"),
        sinks=sinks.call("system", arg_index=0),
    )

    # Step 4: Check if taint reaches the sink
    sink_sel = sinks.call("system", arg_index=0)
    taint_found = result.has_taint_at_sink(sink_sel)

    print(f"IFDS taint analysis complete.")
    print(f"  Taint reaches sink: {taint_found}")

    # Step 5: Inspect diagnostics
    diag = result.diagnostics()
    print(f"  Iterations: {diag['iterations']}")
    print(f"  Path edges explored: {diag['path_edges_explored']}")
    print(f"  Summary edges created: {diag['summary_edges_created']}")

    # Step 6: List tainted values
    tainted = result.tainted_values()
    print(f"  Tainted values: {len(tainted)}")
    for v in tainted:
        print(f"    {v}")

    # Step 7: Export full result
    export = result.export()
    print(f"  Exported {len(export['facts'])} instruction facts")

    if taint_found:
        print("\nVULNERABILITY DETECTED: Tainted data from getenv() reaches system()")
    else:
        print("\nNo taint flow detected.")


if __name__ == "__main__":
    main()
