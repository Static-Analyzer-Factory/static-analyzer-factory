#!/usr/bin/env python3
"""Detect CWE-78 in Rust unsafe code using the SAF Python SDK.

This tutorial demonstrates cross-language taint detection:
Rust's env::args() return value flows through unsafe FFI to
libc's system() function.

The script compiles the vulnerable Rust source to LLVM IR, loads it
through the LLVM frontend, and runs taint analysis end-to-end.

SAF's language-agnostic AIR representation enables detection
regardless of the source language (C, C++, or Rust).

Usage:
    python detect.py
"""

import subprocess
from pathlib import Path

from saf import Project, sources, sinks


def main() -> None:
    tutorial_dir = Path(__file__).parent
    source = tutorial_dir / "vulnerable.rs"
    llvm_ir = tutorial_dir / "vulnerable.ll"

    # Step 1: Compile Rust source to LLVM IR
    subprocess.run(
        ["rustc", "--emit=llvm-ir", "-o", str(llvm_ir), str(source)],
        check=True,
    )

    # Step 2: Load via LLVM frontend
    proj = Project.open(str(llvm_ir))
    q = proj.query()

    # SOURCE: return value of getenv() — user-controlled environment variable
    # SINK:   argument 0 of system() — libc command execution
    findings = q.taint_flow(
        sources=sources.call("getenv"),
        sinks=sinks.call("system", arg_index=0),
    )

    print(f"Found {len(findings)} cross-language taint flow(s):")
    for i, f in enumerate(findings):
        print(f"  [{i}] finding_id={f.finding_id}")
        if f.trace:
            print(f"       trace: {f.trace.pretty()}")


if __name__ == "__main__":
    main()
