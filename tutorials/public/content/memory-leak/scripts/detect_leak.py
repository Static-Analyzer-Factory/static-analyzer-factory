#!/usr/bin/env python3
"""Detect memory leaks using the SAF SDK."""
import saf

proj = saf.Project.open("tests/fixtures/llvm/e2e/memory_leak.ll")
vf = proj.graphs().export("valueflow")
print("Value Flow Graph:")
print(f"  Nodes: {len(vf['nodes'])}")
print(f"  Edges: {len(vf['edges'])}")

# Find malloc allocations without matching free
for finding in proj.check("memory-leak"):
    print(f"  [{finding.severity}] {finding.message}")
