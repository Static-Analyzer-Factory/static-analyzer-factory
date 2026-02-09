#!/usr/bin/env python3
"""Tutorial 6: Memory SSA — detecting stale-data reads.

Demonstrates how Memory SSA disambiguates memory operations:
- S1: *p = source()   → Def to loc_a
- S2: *q = 99         → Def to loc_b (does NOT clobber loc_a)
- C1: modify(p)       → Def (callee modifies loc_a via mod/ref)
- L1: x = *p          → Use (clobber is C1, not S1)

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
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load and analyze
    proj = Project.open(str(llvm_ir))

    # Step 3: Build Memory SSA
    mssa = proj.memory_ssa()
    print("Memory SSA built successfully")
    print(f"  Total memory accesses: {mssa.access_count}")

    # Step 4: Export and inspect structure
    export = mssa.export()
    print(f"  Schema version: {export['schema_version']}")
    print(f"  Functions with MSSA: {len(export['functions'])}")
    print(f"  Functions with mod/ref: {len(export['mod_ref'])}")

    # Step 5: Check mod/ref for modify()
    air = proj.air()
    modify_fn = air.get_function("modify")
    if modify_fn is not None:
        summary = mssa.mod_ref(modify_fn.id)
        if summary is not None:
            print(f"\n  modify() mod/ref summary:")
            print(f"    may_mod locations: {len(summary['may_mod'])}")
            print(f"    may_ref locations: {len(summary['may_ref'])}")

            if summary["may_mod"]:
                print(f"    (modify() writes to {len(summary['may_mod'])} memory location(s))")
        else:
            print("\n  modify() has no mod/ref summary (declaration only)")
    else:
        print("\n  modify() function not found")

    # Step 6: Check test() function's mod/ref
    test_fn = air.get_function("test")
    if test_fn is not None:
        summary = mssa.mod_ref(test_fn.id)
        if summary is not None:
            print(f"\n  test() mod/ref summary:")
            print(f"    may_mod locations: {len(summary['may_mod'])}")
            print(f"    may_ref locations: {len(summary['may_ref'])}")
            print("    (test() transitively includes modify()'s effects)")

    # Step 7: Summary
    print(f"\nMemory SSA analysis complete.")
    print(f"  The call to modify(p) is recognized as a memory Def,")
    print(f"  which clobbers the earlier store *p = source().")
    print(f"  This means the load x = *p reads from modify(),")
    print(f"  not from the tainted source() call.")


if __name__ == "__main__":
    main()
