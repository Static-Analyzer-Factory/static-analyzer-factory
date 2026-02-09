#!/usr/bin/env python3
"""Discover SAF's API capabilities at runtime using schema().

Shows how an AI agent or tool integrator can programmatically
discover all available queries, selectors, and graph types.

Usage:
    python detect.py
"""

import subprocess
import json
from pathlib import Path

from saf import Project


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "program.c"
    llvm_ir = tutorial_dir / "program.ll"

    subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(llvm_ir), str(source)],
        check=True,
    )

    proj = Project.open(str(llvm_ir))

    # Discover all capabilities
    schema = proj.schema()

    print("SAF Schema Discovery")
    print("=" * 50)
    print(f"\nSchema keys: {list(schema.keys())}")

    for key, value in schema.items():
        print(f"\n--- {key} ---")
        if isinstance(value, dict):
            for k, v in value.items():
                print(f"  {k}: {v}")
        elif isinstance(value, list):
            for item in value:
                print(f"  - {item}")
        else:
            print(f"  {value}")

    # Pretty-print as JSON for external tools
    print("\n\nFull schema (JSON):")
    print(json.dumps(schema, indent=2, default=str))


if __name__ == "__main__":
    main()
