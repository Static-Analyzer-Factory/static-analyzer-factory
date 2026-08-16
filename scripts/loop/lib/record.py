#!/usr/bin/env python3
"""Assemble ONE per-arm observability record from artifacts the supervisor already produced
(plan 205 §1b/§5a/§5b): before/after scorer JSON, the worker transcript telemetry + tool
fingerprint, the verdict, a git diffstat, and the before/after task-flip diff.

Pure + deterministic: same inputs -> byte-identical JSON. Observability only — this module NEVER
gates a decision. Every field is either copied verbatim from an artifact or a plain arithmetic diff.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import flips as _flips  # noqa: E402


def _load(p: str | None) -> dict:
    """Tolerant: a missing/corrupt artifact (e.g. an arm that broke the build so `after.json` was
    never written) -> {}. Views label a missing `after` as build-broken via `overall_confirmed_after`."""
    if not p:
        return {}
    try:
        return json.loads(Path(p).read_text())
    except (OSError, ValueError, TypeError):
        return {}


def per_property_confirmed(scorer: dict) -> dict[str, int]:
    return {k: (v or {}).get("confirmed", 0) for k, v in (scorer.get("per_property") or {}).items()}


def per_property_recall(scorer: dict) -> dict[str, str]:
    """confirmed_false / false_total per family — the recall the competition actually scores."""
    out: dict[str, str] = {}
    for k, v in (scorer.get("per_property") or {}).items():
        v = v or {}
        out[k] = f"{v.get('confirmed_false', 0)}/{v.get('false_total', 0)}"
    return out


def _iter_tool_uses(obj: object):
    """Yield every tool_use dict (has `name` + `input`) anywhere in a parsed transcript record."""
    if isinstance(obj, dict):
        if obj.get("type") == "tool_use":
            yield obj
        for v in obj.values():
            yield from _iter_tool_uses(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _iter_tool_uses(v)


def fingerprint(transcript_path: str) -> dict:
    """Cheap tally of what the worker actually DID (§5b): Edit/Write/Bash counts, distinct files
    touched, and whether it ran `cargo nextest`/`cargo test` (self-tested) or `clippy` (self-linted).
    A worker that edited-and-stopped without testing is a strong REVERT predictor. Advisory only."""
    out = {"edits": 0, "writes": 0, "bash": 0, "ran_tests": False, "ran_clippy": False, "files": []}
    files: set[str] = set()
    try:
        lines = open(transcript_path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return out
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except (ValueError, TypeError):
            continue
        for tu in _iter_tool_uses(rec):
            name = tu.get("name")
            inp = tu.get("input")
            if not isinstance(inp, dict):
                inp = {}
            if name == "Edit":
                out["edits"] += 1
            elif name == "Write":
                out["writes"] += 1
            elif name == "Bash":
                out["bash"] += 1
                cmd = str(inp.get("command", ""))
                if "nextest" in cmd or "cargo test" in cmd:
                    out["ran_tests"] = True
                if "clippy" in cmd:
                    out["ran_clippy"] = True
            fp = inp.get("file_path")
            if name in ("Edit", "Write") and isinstance(fp, str) and fp:
                files.add(fp)
    out["files"] = sorted(files)
    return out


def build(*, n, lever, mode, family, scope, decision, delta, progressed,
          before, after, verdict, result, diffstat, ts,
          flips=None, fingerprint=None) -> dict:  # noqa: A002 (shadows module by design for the field name)
    b, a = per_property_confirmed(before), per_property_confirmed(after)
    fams = sorted(set(b) | set(a))
    return {
        "arm": n, "ts": ts,
        "lever": lever, "mode": mode, "family": family, "scope": scope,
        "decision": decision, "confirmed_delta": delta, "progressed": bool(progressed),
        # score movement, per property (NOT just the total)
        "per_property_before": {f: b.get(f, 0) for f in sorted(b)},
        "per_property_after": {f: a.get(f, 0) for f in sorted(a)},
        "per_property_delta": {f: a.get(f, 0) - b.get(f, 0) for f in fams},
        "per_property_recall_after": per_property_recall(after),
        "overall_confirmed_before": before.get("confirmed_score"),
        "overall_confirmed_after": after.get("confirmed_score"),
        # deduped/weighted pool totals (present only under --group-weight, i.e. gen mode; None in legacy).
        # NOTE: in gen mode `confirmed_delta` above is the reward the loop maximizes — the VAL-set weighted
        # delta — while `per_property_*` is the raw POOL breakdown; these weighted totals make the pool
        # basis explicit so the two are not mis-reconciled by an operator.
        "overall_confirmed_weighted_before": before.get("confirmed_score_weighted"),
        "overall_confirmed_weighted_after": after.get("confirmed_score_weighted"),
        "false_alarms_after": after.get("false_alarms"),
        "wrong_true_after": after.get("wrong_true"),
        # cross-family safety + the proximate revert cause (populated by verify_arm's verdict)
        "regressed_families": verdict.get("regressed_families", []),
        "revert_reason": verdict.get("revert_reason"),
        # worker cost / effort
        "worker": {
            "subtype": result.get("subtype"),
            "num_turns": result.get("num_turns"),
            "hit_max_turns": result.get("hit_max_turns"),
            "total_cost_usd": round(result.get("total_cost_usd", 0.0) or 0.0, 4),
            "duration_ms": result.get("duration_ms"),
            "api_retries": result.get("api_retries"),
        },
        # which individual tasks flipped (§5a) — the memorization-vs-generalization instrument
        "flips": flips or {"gained_confirmed": [], "lost_confirmed": [], "new_false_alarms": []},
        # what the worker actually did (§5b) — advisory
        "fingerprint": fingerprint or {"edits": 0, "writes": 0, "bash": 0,
                                       "ran_tests": False, "ran_clippy": False, "files": []},
        # what changed on disk (git diff --numstat, computed in bash)
        "diff": diffstat,
        # the worker's own final message — ADVISORY, never trusted for scoring
        "worker_summary": result.get("summary", ""),
    }


if __name__ == "__main__":
    # argv: <scalars-json> <before> <after> <verdict> <result> <diffstat-json>
    #       [<transcript> <before.pertask> <after.pertask>]   (the last three drive fingerprint+flips)
    scalars = json.loads(sys.argv[1])
    before, after = _load(sys.argv[2]), _load(sys.argv[3])
    verdict, result = _load(sys.argv[4]), _load(sys.argv[5])
    diffstat = json.loads(sys.argv[6]) if len(sys.argv) > 6 and sys.argv[6] else {}
    fp = fingerprint(sys.argv[7]) if len(sys.argv) > 7 and sys.argv[7] else None
    fl = (_flips.flips(sys.argv[8], sys.argv[9])
          if len(sys.argv) > 9 and sys.argv[8] and sys.argv[9] else None)
    row = build(before=before, after=after, verdict=verdict, result=result,
                diffstat=diffstat, flips=fl, fingerprint=fp, **scalars)
    print(json.dumps(row, sort_keys=True))
