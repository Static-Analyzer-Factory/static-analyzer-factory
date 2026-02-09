#!/usr/bin/env python3
"""Custom UAF detector using the SAF SDK."""
import saf
import json

# Load and analyze the program
proj = saf.Project.open("tests/fixtures/llvm/e2e/use_after_free.ll")

# Export graphs
cfg = proj.graphs().export("cfg")
vf = proj.graphs().export("valueflow")
pta = proj.pta_result().export()

print(f"CFG: {len(cfg['nodes'])} nodes, {len(cfg['edges'])} edges")
print(f"Value Flow: {len(vf['nodes'])} nodes, {len(vf['edges'])} edges")

# Check for findings
for finding in proj.check_all():
    print(f"[{finding.severity}] {finding.message}")
    if finding.source:
        print(f"  Source: {finding.source}")
