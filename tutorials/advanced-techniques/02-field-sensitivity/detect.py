#!/usr/bin/env python3
"""Explore field-sensitive pointer analysis with structs and linked lists.

Demonstrates how SAF's PTA tracks struct fields separately (field sensitivity)
and what happens when linked list depth exceeds max_depth.

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "program.c"
    llvm_ir = tutorial_dir / "program.ll"

    # Step 1: Compile C source to LLVM IR
    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load and analyze
    proj = Project.open(str(llvm_ir))

    # Step 3: PTA results
    pta = proj.pta_result()
    print("PTA Statistics:")
    print(f"  Values tracked: {pta.value_count}")
    print(f"  Abstract locations: {pta.location_count}")

    # Step 4: Export full PTA
    # Format: {"points_to": [{"value": id, "locations": [id, ...]}, ...], ...}
    export = pta.export()
    pts_list = export.get("points_to", [])
    print(f"  Points-to entries: {len(pts_list)}")

    # Step 5: Show all points-to sets
    print("\nPoints-to sets:")
    for entry in pts_list:
        val_id = entry["value"]
        locs = entry.get("locations", [])
        print(f"  {val_id[:24]}... -> {len(locs)} loc(s)")

    # Step 6: Look for entries with multiple locations
    multi_target = [e for e in pts_list if len(e.get("locations", [])) > 1]
    if multi_target:
        print(f"\nValues pointing to multiple locations ({len(multi_target)}):")
        for entry in multi_target:
            val_id = entry["value"]
            locs = entry.get("locations", [])
            print(f"  {val_id[:24]}... -> {len(locs)} loc(s)")
    else:
        print("\nNo values with multiple targets (fully field-sensitive)")

    # Step 7: AIR inspection
    air = proj.air()
    print(f"\nFunctions: {air.function_names()}")


if __name__ == "__main__":
    main()
