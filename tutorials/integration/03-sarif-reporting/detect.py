#!/usr/bin/env python3
"""Generate a SARIF 2.1.0 report from SAF taint analysis findings.

Demonstrates how to build standards-compliant SARIF output for
GitHub Code Scanning, VS Code SARIF Viewer, DefectDojo, etc.

Usage:
    python detect.py
"""

import subprocess
import json
from pathlib import Path

from saf import Project, sources, sinks


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
    q = proj.query()

    # Find command injection vulnerabilities
    findings = q.taint_flow(
        sources=sources.function_param("main", 1),
        sinks=sinks.call("system", arg_index=0),
    )

    print(f"Found {len(findings)} finding(s)")

    # Build SARIF 2.1.0 envelope
    results = []
    for f in findings:
        d = f.to_dict()
        result = {
            "ruleId": "CWE-78",
            "level": "error",
            "message": {
                "text": f"Taint flow from {d.get('source_location', 'unknown')} to {d.get('sink_location', 'unknown')}"
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": str(source)
                    }
                }
            }],
            "fingerprints": {
                "safFindingId": d.get("finding_id", "")
            }
        }
        # Add code flow from trace
        if d.get("trace"):
            thread_flows = []
            for step in d["trace"]:
                loc = step.get("to_location") or step.get("from_location", "")
                thread_flows.append({
                    "location": {
                        "message": {"text": f"{step.get('edge', '')}"},
                        "physicalLocation": {
                            "artifactLocation": {"uri": str(source)},
                        }
                    }
                })
            if thread_flows:
                result["codeFlows"] = [{
                    "threadFlows": [{"locations": thread_flows}]
                }]
        results.append(result)

    sarif = {
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "SAF",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/Static-Analyzer-Factory/static-analyzer-factory",
                    "rules": [{
                        "id": "CWE-78",
                        "name": "OSCommandInjection",
                        "shortDescription": {"text": "OS Command Injection"},
                        "helpUri": "https://cwe.mitre.org/data/definitions/78.html"
                    }]
                }
            },
            "results": results
        }]
    }

    # Write SARIF report
    sarif_path = tutorial_dir / "report.sarif.json"
    with open(sarif_path, "w") as f:
        json.dump(sarif, f, indent=2, default=str)

    print(f"\nSARIF report written to: {sarif_path}")
    print(f"Results: {len(results)}")
    print(f"\nReport preview:")
    print(json.dumps(sarif, indent=2, default=str)[:500])


if __name__ == "__main__":
    main()
