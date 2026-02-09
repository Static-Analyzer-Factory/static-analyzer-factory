#!/usr/bin/env python3
"""Comprehensive batch scanning: multi-language programs with stress testing.

Demonstrates:
- Multi-language compilation (C, C++, Rust)
- Multi-vulnerability scanning (taint, SQLi, XSS, log injection)
- Large codebase handling (HTTP handler, plugin system)
- PTA stress testing with scale metrics

Usage:
    python detect.py
"""

import subprocess
import json
from pathlib import Path
from collections import Counter

from saf import Project, sources, sinks


def compile_c(src: Path, ll: Path) -> bool:
    """Compile C source to LLVM IR."""
    result = subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(ll), str(src)],
        capture_output=True,
    )
    return result.returncode == 0


def compile_cpp(src: Path, ll: Path) -> bool:
    """Compile C++ source to LLVM IR."""
    result = subprocess.run(
        ["clang-18", "-S", "-emit-llvm", "-O0", "-g",
         "-o", str(ll), str(src)],
        capture_output=True,
    )
    return result.returncode == 0


def compile_rust(src: Path, ll: Path) -> bool:
    """Compile Rust source to LLVM IR."""
    result = subprocess.run(
        ["rustc", "--emit=llvm-ir", "-C", "debuginfo=0",
         "-o", str(ll), str(src)],
        capture_output=True,
    )
    return result.returncode == 0


def scan_command_injection(proj: Project, name: str) -> list:
    """Scan for command injection (CWE-78)."""
    try:
        q = proj.query()
        findings = q.taint_flow(
            sources=sources.function_param("main", 1) | sources.call("getenv"),
            sinks=sinks.call("system", arg_index=0),
        )
        return [{"program": name, "cwe": "CWE-78", "type": "command_injection",
                 "finding_id": f.finding_id} for f in findings]
    except Exception as e:
        print(f"    Warning: {e}")
        return []


def scan_sqli(proj: Project, name: str) -> list:
    """Scan for SQL injection (CWE-89)."""
    try:
        q = proj.query()
        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("db_query"),
        )
        return [{"program": name, "cwe": "CWE-89", "type": "sql_injection",
                 "finding_id": f.finding_id} for f in findings]
    except Exception:
        return []


def scan_xss(proj: Project, name: str) -> list:
    """Scan for XSS (CWE-79)."""
    try:
        q = proj.query()
        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("render_html"),
        )
        return [{"program": name, "cwe": "CWE-79", "type": "xss",
                 "finding_id": f.finding_id} for f in findings]
    except Exception:
        return []


def scan_log_injection(proj: Project, name: str) -> list:
    """Scan for log injection (CWE-117)."""
    try:
        q = proj.query()
        findings = q.taint_flow(
            sources=sources.function_param("main", 1),
            sinks=sinks.call("log_access"),
        )
        return [{"program": name, "cwe": "CWE-117", "type": "log_injection",
                 "finding_id": f.finding_id} for f in findings]
    except Exception:
        return []


def get_pta_stats(proj: Project) -> dict:
    """Get PTA statistics for stress testing."""
    try:
        pta = proj.pta_result()
        export = pta.export()
        pts_list = export.get("points_to", [])

        # Size distribution
        size_counts: Counter = Counter()
        for entry in pts_list:
            sz = len(entry.get("locations", []))
            size_counts[sz] += 1

        return {
            "values": pta.value_count,
            "locations": pta.location_count,
            "entries": len(pts_list),
            "size_distribution": dict(size_counts),
        }
    except Exception:
        return {}


def get_module_stats(proj: Project) -> dict:
    """Get module statistics."""
    try:
        air = proj.air()
        return {
            "functions": air.function_count,
            "globals": air.global_count,
        }
    except Exception:
        return {}


def get_callgraph_stats(proj: Project) -> dict:
    """Get call graph statistics."""
    try:
        graphs = proj.graphs()
        cg = graphs.export("callgraph")
        nodes = cg.get("nodes", [])
        edges = cg.get("edges", [])
        return {
            "nodes": len(nodes),
            "edges": len(edges),
        }
    except Exception:
        return {}


def main() -> None:
    tutorial_dir = Path(__file__).parent

    # Define programs with their compilation and scanning strategies
    programs = [
        # Simple C: command injection
        {
            "name": "injection (C)",
            "source": "injection.c",
            "compiler": compile_c,
            "scans": ["command_injection"],
            "stress_test": False,
        },
        # Simple C++: dangling pointer (no matching taint query)
        {
            "name": "dangling (C++)",
            "source": "dangling.cpp",
            "compiler": compile_cpp,
            "scans": ["command_injection"],  # will find 0
            "stress_test": False,
        },
        # Rust FFI: command injection via getenv
        {
            "name": "unsafe_ffi (Rust)",
            "source": "unsafe_ffi.rs",
            "compiler": compile_rust,
            "scans": ["command_injection"],
            "stress_test": False,
        },
        # Large C: HTTP handler with SQLi, XSS, log injection
        {
            "name": "http_handler (C)",
            "source": "http_handler.c",
            "compiler": compile_c,
            "scans": ["sqli", "xss", "log_injection"],
            "stress_test": True,
        },
        # Large C++: plugin system stress test
        {
            "name": "plugin_system (C++)",
            "source": "plugin_system.cpp",
            "compiler": compile_cpp,
            "scans": [],  # PTA stress test only
            "stress_test": True,
        },
    ]

    all_findings = []
    stats = {
        "programs_scanned": 0,
        "programs_failed": 0,
        "by_cwe": Counter(),
    }

    print("=" * 70)
    print("SAF Batch Scanning Pipeline")
    print("=" * 70)
    print("\nPhase 1: Multi-Language Compilation and Scanning")
    print("-" * 70)

    for prog in programs:
        name = prog["name"]
        src = tutorial_dir / prog["source"]
        ll = tutorial_dir / (src.stem + ".ll")

        print(f"\n[{name}]")
        print(f"  Compiling {prog['source']}...")

        if not prog["compiler"](src, ll):
            print(f"  SKIP: Compilation failed")
            stats["programs_failed"] += 1
            continue

        print(f"  Loading into SAF...")
        try:
            proj = Project.open(str(ll))
        except Exception as e:
            print(f"  SKIP: Load failed: {e}")
            stats["programs_failed"] += 1
            continue

        stats["programs_scanned"] += 1

        # Run vulnerability scans
        for scan_type in prog["scans"]:
            if scan_type == "command_injection":
                findings = scan_command_injection(proj, name)
            elif scan_type == "sqli":
                findings = scan_sqli(proj, name)
            elif scan_type == "xss":
                findings = scan_xss(proj, name)
            elif scan_type == "log_injection":
                findings = scan_log_injection(proj, name)
            else:
                findings = []

            if findings:
                print(f"  {scan_type}: {len(findings)} finding(s)")
                all_findings.extend(findings)
                for f in findings:
                    stats["by_cwe"][f["cwe"]] += 1
            else:
                print(f"  {scan_type}: 0 findings")

        # Stress test statistics
        if prog["stress_test"]:
            mod_stats = get_module_stats(proj)
            pta_stats = get_pta_stats(proj)
            cg_stats = get_callgraph_stats(proj)

            print(f"  --- Stress Test Stats ---")
            if mod_stats:
                print(f"    Module: {mod_stats['functions']} functions, {mod_stats['globals']} globals")
            if cg_stats:
                print(f"    CallGraph: {cg_stats['nodes']} nodes, {cg_stats['edges']} edges")
            if pta_stats:
                print(f"    PTA: {pta_stats['values']} values, {pta_stats['locations']} locations")
                if pta_stats.get("size_distribution"):
                    sizes = pta_stats["size_distribution"]
                    print(f"    PTA set sizes: {dict(sorted(sizes.items())[:5])}...")

    # Summary
    print("\n" + "=" * 70)
    print("Batch Scan Summary")
    print("=" * 70)
    print(f"  Programs scanned: {stats['programs_scanned']}")
    print(f"  Programs failed:  {stats['programs_failed']}")
    print(f"  Total findings:   {len(all_findings)}")

    if stats["by_cwe"]:
        print("\n  Findings by CWE:")
        for cwe, count in sorted(stats["by_cwe"].items()):
            print(f"    {cwe}: {count}")

    # Export as SARIF
    if all_findings:
        results = []
        for f in all_findings:
            results.append({
                "ruleId": f["cwe"],
                "level": "error",
                "message": {"text": f"{f['type']} in {f['program']}"},
                "fingerprints": {"safFindingId": f["finding_id"]},
            })

        sarif = {
            "version": "2.1.0",
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
            "runs": [{
                "tool": {"driver": {"name": "SAF", "version": "0.1.0"}},
                "results": results,
            }],
        }

        sarif_path = tutorial_dir / "batch_report.sarif.json"
        with open(sarif_path, "w") as f:
            json.dump(sarif, f, indent=2, default=str)
        print(f"\n  SARIF report: {sarif_path.name}")

    print("\n" + "=" * 70)
    print("Batch scan complete.")


if __name__ == "__main__":
    main()
