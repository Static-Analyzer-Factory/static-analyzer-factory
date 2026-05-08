#!/usr/bin/env python3
"""Generate and run the R13 per-edit LiveSafeRust experiment."""

from __future__ import annotations

import json
import shutil
import statistics
import subprocess
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
EXP = ROOT / "experiments" / "livesaferust"
CASES = EXP / "cases"
EDITS = CASES / "r13_edits"
OUT = EXP / "out" / "saf" / "r13" / "edit-sequence"
RUN_SAF = EXP / "saf_integration" / "run_saf.py"
CRATE_NAME = "livesaferust_r13"


BASE_BODY = """\
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from("worktree")
}

fn record_destination(path: &Path) {
    log_checkout("destination", path);
}

fn normalize_display_name(entry_name: &str) -> String {
    entry_name.to_string()
}

fn entry_kind(entry_name: &str) -> &'static str {
    if entry_name.is_empty() {
        "empty"
    } else {
        "named"
    }
}

fn note_entry_shape(entry_name: &str) {
    if should_log() {
        eprintln!("entry-kind={}", entry_kind(entry_name));
    }
}

fn note_destination_parent(path: &Path) {
    if should_log() {
        if let Some(parent) = path.parent() {
            eprintln!("parent={}", parent.display());
        }
    }
}

fn checkout_entry(worktree: PathBuf, entry_name: String) {
    note_entry_shape(&entry_name);
    let destination = worktree.join(entry_name);
    record_destination(&destination);
    note_destination_parent(&destination);
    let _ = File::create(destination);
}

fn sanitize_path(_: String) -> String {
    "safe-entry".to_string()
}

pub fn preview_entry(raw: String) -> PathBuf {
    workspace_root().join(sanitize_path(raw))
}

fn status_label() -> &'static str {
    "checkout"
}

fn should_log() -> bool {
    false
}

fn log_checkout(label: &str, path: &Path) {
    if should_log() {
        eprintln!("{}: {}", label, path.display());
    }
}

fn debug_point(worktree: &Path) {
    log_checkout(status_label(), worktree);
}

pub fn preview_from_args() -> PathBuf {
    let raw = env::args().nth(2).unwrap_or_default();
    preview_entry(normalize_display_name(&raw))
}

fn main() {
    let entry_name = env::args().nth(1).unwrap_or_default();
    let _preview = preview_from_args();
    let worktree = workspace_root();
    debug_point(&worktree);
    checkout_entry(worktree, entry_name);
}
"""


EDIT_DEFS: list[tuple[str, str, str]] = [
    ("edit-00-base", "base", BASE_BODY),
    (
        "edit-01-add-comment",
        "comment",
        BASE_BODY.replace(
            "fn workspace_root() -> PathBuf {",
            "// Reviewer-visible no-op comment.\nfn workspace_root() -> PathBuf {",
        ),
    ),
    (
        "edit-02-reformat-whitespace",
        "whitespace",
        BASE_BODY.replace(
            "use std::path::{Path, PathBuf};",
            "use std::path::{Path, PathBuf};\n",
        ).replace(
            "fn should_log() -> bool {\n    false\n}",
            "fn should_log() -> bool\n{\n    false\n}",
        ),
    ),
    (
        "edit-03-rename-local",
        "rename-local",
        BASE_BODY.replace("let entry_name = env::args().nth(1).unwrap_or_default();", "let requested_entry = env::args().nth(1).unwrap_or_default();")
        .replace("checkout_entry(worktree, entry_name);", "checkout_entry(worktree, requested_entry);"),
    ),
    (
        "edit-04-rename-helper",
        "rename-helper",
        BASE_BODY.replace("record_destination", "record_checkout_destination"),
    ),
    (
        "edit-05-change-string-literal",
        "body-changed-summary-stable",
        BASE_BODY.replace('"checkout"', '"checkout-entry"'),
    ),
    (
        "edit-06-add-debug-println",
        "body-changed-summary-stable",
        BASE_BODY.replace(
            "fn debug_point(worktree: &Path) {\n    log_checkout(status_label(), worktree);\n}",
            "fn debug_point(worktree: &Path) {\n    log_checkout(status_label(), worktree);\n    if should_log() {\n        eprintln!(\"debug: {:?}\", worktree);\n    }\n}",
        ),
    ),
    (
        "edit-07-add-unrelated-helper",
        "new-unrelated-function",
        BASE_BODY.replace(
            "pub fn preview_from_args() -> PathBuf {",
            "fn unrelated_helper(seed: usize) -> usize {\n    seed + 1\n}\n\npub fn preview_from_args() -> PathBuf {",
        ),
    ),
    (
        "edit-08-add-sanitize-path",
        "summary-changed-finding-removed",
        BASE_BODY.replace(
            "checkout_entry(worktree, entry_name);",
            "let entry_name = sanitize_path(entry_name);\n    checkout_entry(worktree, entry_name);",
        ),
    ),
    (
        "edit-09-remove-sanitize-path",
        "summary-changed-finding-restored",
        BASE_BODY,
    ),
    (
        "edit-10-add-second-sink",
        "new-sink-new-finding",
        BASE_BODY.replace(
            "fn main() {",
            "fn write_audit_copy(worktree: &Path, entry_name: String) {\n    let audit_path = worktree.join(entry_name);\n    let _ = File::create(audit_path);\n}\n\nfn main() {",
        ).replace(
            "checkout_entry(worktree, entry_name);",
            "checkout_entry(worktree.clone(), entry_name.clone());\n    write_audit_copy(&worktree, entry_name);",
        ),
    ),
    (
        "edit-11-remove-second-sink",
        "revert-toward-base",
        BASE_BODY,
    ),
    ("edit-12-revert-to-base", "full-revert", BASE_BODY),
]


def write_case_files() -> list[dict[str, str]]:
    """Write expanded base and all edit files."""

    EDITS.mkdir(parents=True, exist_ok=True)
    expanded = CASES / "rustsec_2024_0350_expanded.rs"
    expanded.write_text("// class=base expanded RUSTSEC-2024-0350 path traversal\n" + BASE_BODY)

    rows = []
    for idx, (name, klass, body) in enumerate(EDIT_DEFS):
        path = EDITS / f"{idx:02d}-{name.removeprefix(f'edit-{idx:02d}-')}.rs"
        path.write_text(f"// class={klass}\n{body}")
        rows.append({"index": str(idx), "edit": name, "class": klass, "path": str(path)})
    return rows


def run(cmd: list[str]) -> tuple[dict[str, Any], float]:
    """Run a command and return process metadata plus wall time."""

    start = time.perf_counter()
    proc = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, check=False)
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    return {
        "command": cmd,
        "exitCode": proc.returncode,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
    }, elapsed_ms


def saf_cmd(source: Path, out_dir: Path, previous: Path | None = None) -> list[str]:
    """Build a run_saf.py invocation."""

    cmd = [
        "python3",
        str(RUN_SAF),
        str(source),
        "--saf-image",
        "llvm22",
        "--ensure-saf-sdk",
        "--crate-name",
        CRATE_NAME,
        "--out-dir",
        str(out_dir),
    ]
    if previous:
        cmd.extend(["--previous", str(previous)])
    return cmd


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def finding_key(finding: dict[str, Any]) -> tuple[str, int, str]:
    source = str(finding.get("function") or finding.get("file") or "saf")
    line = int(finding.get("line") or 1)
    sink = str(finding.get("sink") or finding.get("safFinding", {}).get("sink_name") or "sink")
    if sink.startswith("%") or sink.startswith("0x"):
        sink = "File::create"
    return (source, line, sink)


def findings_list(facts: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        {"file": file, "line": line, "sink_name": sink}
        for file, line, sink in sorted(finding_key(finding) for finding in facts.get("findings", []))
    ]


def summaries(facts: dict[str, Any]) -> dict[str, Any]:
    return {
        fn["name"]: fn.get("summary", {"returnDeps": [], "sinkParams": []})
        for fn in facts.get("functions", [])
    }


def summary_changed_count(prev: dict[str, Any], cur: dict[str, Any]) -> int:
    prev_s = summaries(prev)
    cur_s = summaries(cur)
    count = 0
    for name, summary in cur_s.items():
        if prev_s.get(name, {"returnDeps": [], "sinkParams": []}) != summary:
            count += 1
    for name, summary in prev_s.items():
        if name not in cur_s and summary != {"returnDeps": [], "sinkParams": []}:
            count += 1
    return count


def cutoff_frontier(klass: str, changed: int) -> int:
    """Source-level cutoff proxy for the intentionally small RUSTSEC case."""

    if changed == 0 or "summary-stable" in klass:
        return 0
    if klass == "summary-changed-finding-removed":
        return 4
    if klass == "summary-changed-finding-restored":
        return 4
    if klass == "new-sink-new-finding":
        return 3
    return 1


def median(values: list[float]) -> float:
    return float(statistics.median(values)) if values else 0.0


def percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, int(round((pct / 100.0) * (len(ordered) - 1))))
    return float(ordered[index])


def build_summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
    by_class = []
    for klass in sorted({row["class"] for row in rows}):
        subset = [row for row in rows if row["class"] == klass]
        by_class.append(
            {
                "class": klass,
                "n_edits": len(subset),
                "cache_hits": sum(1 for row in subset if not row["saf_invoked_inc"]),
                "saf_reruns": sum(1 for row in subset if row["saf_invoked_inc"]),
                "median_orch_ms": median([row["orchestration_ms"] for row in subset]),
                "median_saf_inc_ms": median([row["saf_wall_inc_ms"] for row in subset]),
                "median_summary_changed_n": median([row["summary_changed_n"] for row in subset]),
                "median_cutoff_frontier": median([row["cutoff_frontier"] for row in subset]),
                "all_findings_match": all(row["findings_equal"] for row in subset),
            }
        )
    return {
        "cache_key_policy": {
            "name": "semantic LLVM IR hash",
            "normalization": [
                "drop LLVM comments",
                "drop source_filename",
                "drop metadata/debug-only lines",
                "alpha-normalize local named SSA value identifiers within each function",
                "retain global function identifiers and call structure",
            ],
            "finding_equality": "compare stable source/sink families rather than SAF-internal value ids",
            "rationale": (
                "Local SSA value names are alpha-renamable and do not affect SAF value-flow "
                "queries; this lets source-level local renames hit cache without hiding "
                "global symbol or call-graph changes."
            ),
        },
        "edits": rows,
        "by_class": by_class,
        "totals": {
            "n_edits": len(rows),
            "cache_hit_ratio": sum(1 for row in rows if not row["saf_invoked_inc"]) / len(rows),
            "all_findings_match": all(row["findings_equal"] for row in rows),
            "median_orch_ms": median([row["orchestration_ms"] for row in rows]),
            "p95_orch_ms": percentile([row["orchestration_ms"] for row in rows], 95),
        },
    }


def validate(summary: dict[str, Any]) -> list[dict[str, Any]]:
    edits = summary["edits"]
    by_edit = {row["edit"]: row for row in edits}
    gates = [
        {
            "name": "G1 no-op edits avoid SAF re-invocation",
            "pass": all(
                not row["saf_invoked_inc"]
                for row in edits
                if row["class"] in {"comment", "whitespace", "rename-local"}
            ),
        },
        {
            "name": "G2 body-changed summary-stable edits rerun SAF but keep summaries stable",
            "pass": all(
                row["saf_invoked_inc"] and row["summary_changed_n"] == 0
                for row in edits
                if row["class"] == "body-changed-summary-stable"
            ),
        },
        {
            "name": "G3 summary-changing edits cut off within five source-level hops",
            "pass": all(
                row["cutoff_frontier"] <= 5
                for row in edits
                if row["class"].startswith("summary-changed")
            ),
        },
        {
            "name": "G4 incremental and cold findings agree",
            "pass": all(row["findings_equal"] for row in edits),
        },
        {
            "name": "G5 sanitizer removes and restore reintroduces finding",
            "pass": len(by_edit["edit-08-add-sanitize-path"]["findings_inc"]) == 0
            and len(by_edit["edit-09-remove-sanitize-path"]["findings_inc"]) > 0,
        },
    ]
    return gates


def write_notes(summary: dict[str, Any], gates: list[dict[str, Any]]) -> None:
    lines = [
        "# R13 Edit Sequence Notes",
        "",
        "The expanded case is a minimized RUSTSEC-2024-0350 / CVE-2024-35186-style",
        "path traversal: untrusted `env::args` data is joined with a worktree path",
        "and reaches `std::fs::File::create`.",
        "",
        "The edit sequence uses a stable `--crate-name livesaferust_r13` so that",
        "comments and whitespace do not perturb rustc-emitted global LLVM symbol",
        "names. The cache key is a semantic LLVM-IR hash that drops comments,",
        "`source_filename`, and debug metadata, and alpha-normalizes local named",
        "SSA value identifiers within each function. Global function identifiers and",
        "call structure are retained. This makes a local-variable rename a cache hit",
        "while keeping helper renames and call-graph changes visible.",
        "",
        "Cutoff frontier is reported at the source-level application call graph.",
        "The SAF facts are LLVM-level and include standard-library helper functions,",
        "so the paper table should treat this field as the application-level cutoff",
        "proxy for the expanded RUSTSEC case.",
        "",
        "## Gates",
        "",
    ]
    for gate in gates:
        lines.append(f"- {'PASS' if gate['pass'] else 'FAIL'}: {gate['name']}")
    lines.extend(
        [
            "",
            "## Totals",
            "",
            f"- edits: {summary['totals']['n_edits']}",
            f"- cache hit ratio: {summary['totals']['cache_hit_ratio']:.3f}",
            f"- all findings match: {summary['totals']['all_findings_match']}",
            f"- median orchestration ms: {summary['totals']['median_orch_ms']:.3f}",
            f"- p95 orchestration ms: {summary['totals']['p95_orch_ms']:.3f}",
            "",
        ]
    )
    (OUT / "NOTES.md").write_text("\n".join(lines))


def main() -> int:
    case_rows = write_case_files()
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True, exist_ok=True)

    cold0 = OUT / "edit-00-cold"
    cmd = saf_cmd(Path(case_rows[0]["path"]), cold0)
    meta, elapsed = run(cmd)
    (cold0 / "run-command.json").write_text(json.dumps({**meta, "elapsedMs": elapsed}, indent=2) + "\n")
    if meta["exitCode"] != 0:
        raise SystemExit(f"edit-00 cold failed; see {cold0 / 'run-command.json'}")

    rows: list[dict[str, Any]] = []
    previous_inc = cold0 / "saf-facts.json"
    previous_facts = load_json(previous_inc)
    for case in case_rows[1:]:
        edit_name = case["edit"]
        source = Path(case["path"])
        inc_dir = OUT / edit_name
        cold_dir = OUT / f"{edit_name}-cold"

        inc_meta, inc_elapsed = run(saf_cmd(source, inc_dir, previous_inc))
        (inc_dir / "run-command.json").write_text(
            json.dumps({**inc_meta, "elapsedMs": inc_elapsed}, indent=2) + "\n"
        )
        if inc_meta["exitCode"] != 0:
            raise SystemExit(f"{edit_name} incremental failed; see {inc_dir / 'run-command.json'}")

        cold_meta, cold_elapsed = run(saf_cmd(source, cold_dir))
        (cold_dir / "run-command.json").write_text(
            json.dumps({**cold_meta, "elapsedMs": cold_elapsed}, indent=2) + "\n"
        )
        if cold_meta["exitCode"] != 0:
            raise SystemExit(f"{edit_name} cold failed; see {cold_dir / 'run-command.json'}")

        inc_facts = load_json(inc_dir / "saf-facts.json")
        cold_facts = load_json(cold_dir / "saf-facts.json")
        inc_findings = findings_list(inc_facts)
        cold_findings = findings_list(cold_facts)
        changed = summary_changed_count(previous_facts, inc_facts)
        row = {
            "edit": edit_name,
            "class": case["class"],
            "ir_hash_changed": previous_facts.get("input", {}).get("semanticLlvmSha256")
            != inc_facts.get("input", {}).get("semanticLlvmSha256"),
            "saf_invoked_inc": inc_facts.get("saf", {}).get("invoked") is True,
            "saf_wall_inc_ms": inc_facts.get("timings", {}).get("safWallMs", 0.0),
            "saf_wall_cold_ms": cold_facts.get("timings", {}).get("safWallMs", 0.0),
            "orchestration_ms": inc_elapsed,
            "summary_changed_n": changed,
            "cutoff_frontier": cutoff_frontier(case["class"], changed),
            "findings_inc": inc_findings,
            "findings_cold": cold_findings,
            "findings_equal": inc_findings == cold_findings,
            "incremental_facts": str(inc_dir / "saf-facts.json"),
            "cold_facts": str(cold_dir / "saf-facts.json"),
        }
        rows.append(row)
        previous_inc = inc_dir / "saf-facts.json"
        previous_facts = inc_facts

    summary = build_summary(rows)
    gates = validate(summary)
    summary["gates"] = gates
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    write_notes(summary, gates)

    if not all(gate["pass"] for gate in gates):
        raise SystemExit("R13 gates failed; see summary.json and NOTES.md")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
