#!/usr/bin/env python3
"""Detect CWE-78 cross-module command injection using SAF.

Demonstrates taint flow across translation unit boundaries: the source
(getenv) is in module_a.c and the sink (system) is in module_b.c.

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project, sources, sinks


def main() -> None:
    tutorial_dir = Path(__file__).parent
    module_a_src = tutorial_dir / "module_a.c"
    module_b_src = tutorial_dir / "module_b.c"
    module_a_ll = tutorial_dir / "module_a.ll"
    module_b_ll = tutorial_dir / "module_b.ll"
    combined_ll = tutorial_dir / "combined.ll"

    # Step 1: Compile both modules to LLVM IR
    for src, ll in [(module_a_src, module_a_ll), (module_b_src, module_b_ll)]:
        subprocess.run(
            ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
             "-o", str(ll), str(src)],
            check=True,
        )

    # Step 2: Link the two LLVM IR modules into one
    subprocess.run(
        ["llvm-link-18", "-S", "-o", str(combined_ll),
         str(module_a_ll), str(module_b_ll)],
        check=True,
    )

    # Step 3: Load the combined module via the LLVM frontend
    proj = Project.open(str(combined_ll))

    # Step 4: Query for cross-module taint flow
    q = proj.query()
    findings = q.taint_flow(
        sources=sources.call("getenv"),
        sinks=sinks.call("system", arg_index=0),
    )

    # Step 5: Report results
    print(f"Cross-module taint flows found: {len(findings)}")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace steps: {len(f.trace.steps)}")
            for step in f.trace.steps:
                sym = step.to_symbol or "?"
                print(f"         {step.from_kind} -> [{step.edge}] -> {step.to_kind} ({sym})")


if __name__ == "__main__":
    main()
