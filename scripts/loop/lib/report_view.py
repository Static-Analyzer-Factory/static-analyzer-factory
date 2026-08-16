#!/usr/bin/env python3
"""Derived read-only views over .loop-state/arms.jsonl (+ heldout-*.json) — plan 205 §2c/§4b/§5c.

Pure; no side effects. Every number is RECOMPUTED from the append-only spine, so a view can never
disagree with recorded history. `report.sh` is glue that formats what these functions return.
"""
from __future__ import annotations

import glob
import json
import os
import sys
from collections import defaultdict

# Map every decision word the gates can emit (legacy `decide` + gen-mode `decide_v2` + the
# supervisor's WORKER_FAIL) onto one of four ROI buckets. KEEP/KEEP_POOL both bank points and
# advance the branch; ACCUMULATE/ACCUMULATE_PLUS preserve score-neutral work; the rest discard.
_BUCKET = {
    "KEEP": "KEEP", "KEEP_POOL": "KEEP",
    "ACCUMULATE": "ACCUMULATE", "ACCUMULATE_PLUS": "ACCUMULATE",
    "REVERT": "REVERT", "WORKER_FAIL": "REVERT",
    "REJECT_TAMPER": "REJECT", "REJECT_HOLDOUT": "REJECT",
}


def load_arms(path: str) -> list[dict]:
    rows: list[dict] = []
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return rows
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except (ValueError, TypeError):
            continue
    return rows


def _int(x) -> int:
    return x if isinstance(x, int) else 0


def _num(x) -> float:
    return x if isinstance(x, (int, float)) else 0.0


def lever_roi(rows: list[dict]) -> dict[str, dict]:
    """Per-lever ledger: decision-bucket counts, net confirmed points on KEEP-bucket arms, total
    cost/turns/max_turns, and $/confirmed-point. Sorted by (-netDelta, cost) so payers lead."""
    g: dict[str, dict] = defaultdict(lambda: {"arms": 0, "KEEP": 0, "ACCUMULATE": 0, "REVERT": 0,
                                              "REJECT": 0, "netDelta": 0, "cost": 0.0, "turns": 0,
                                              "maxturns": 0})
    for r in rows:
        s = g[r.get("lever", "?")]
        s["arms"] += 1
        bucket = _BUCKET.get(r.get("decision", "REVERT"), "REJECT")
        s[bucket] += 1
        if bucket == "KEEP":
            s["netDelta"] += _int(r.get("confirmed_delta"))
        w = r.get("worker") or {}
        s["cost"] += _num(w.get("total_cost_usd"))
        s["turns"] += _int(w.get("num_turns"))
        s["maxturns"] += 1 if w.get("hit_max_turns") else 0
    for s in g.values():
        s["cost"] = round(s["cost"], 2)
        s["dollars_per_point"] = round(s["cost"] / s["netDelta"], 3) if s["netDelta"] > 0 else None
    return dict(sorted(g.items(), key=lambda kv: (-kv[1]["netDelta"], kv[1]["cost"])))


def lift_since(rows: list[dict], since_arm: int) -> dict[str, int]:
    """Per-lever sum of KEEP-bucket `confirmed_delta` for arms numbered strictly > `since_arm` — the
    reasoning-set (`val`) lift a lever banked since the last holdout checkpoint (plan 205 §2.4). Only
    KEEP/KEEP_POOL arms contribute (they advanced the integration branch); REVERT/ACCUMULATE/REJECT
    contribute 0. In gen mode `confirmed_delta` IS the val weighted delta, so this is exactly the
    per-lever generalization lift the holdout brake attributes movement to. Deterministic."""
    out: dict[str, int] = defaultdict(int)
    for r in rows:
        if _int(r.get("arm")) <= since_arm:
            continue
        if _BUCKET.get(r.get("decision", ""), "") == "KEEP":
            out[r.get("lever", "?")] += _int(r.get("confirmed_delta"))
    return dict(out)


def waste(rows: list[dict]) -> dict:
    """Signals of wasted/re-derived work: reverting levers with net<=0, arms that hit max_turns,
    large reverts (>=8 files discarded), and near-identical diffs (same touched paths on >1 arm of
    the same lever = re-derivation)."""
    roi = lever_roi(rows)
    stuck = [k for k, s in roi.items() if s["REVERT"] >= 3 and s["netDelta"] <= 0]
    maxturns = [r.get("arm") for r in rows if (r.get("worker") or {}).get("hit_max_turns")]
    bigreverts = [(r.get("arm"), (r.get("diff") or {}).get("files", 0), (r.get("diff") or {}).get("insertions", 0))
                  for r in rows if r.get("decision") in ("REVERT", "WORKER_FAIL")
                  and (r.get("diff") or {}).get("files", 0) >= 8]
    seen: dict[tuple, list] = defaultdict(list)
    for r in rows:
        paths = tuple(sorted((r.get("diff") or {}).get("paths", []) or []))
        if paths:
            seen[(r.get("lever"), paths)].append(r.get("arm"))
    rederived = {k[0]: sorted(v) for k, v in seen.items() if len(v) > 1}
    return {"stuck_levers": sorted(stuck), "max_turns_arms": maxturns,
            "big_reverts": bigreverts, "rederived": rederived}


def capability_milestones(rows: list[dict]) -> dict[str, dict]:
    """Progress of capability-mode levers (build a feature over MANY arms, correctly score 0 for a
    while — invisible to a score-only view). Per capability lever: total arms, ACCUMULATE(_PLUS)
    count, cumulative insertions (LOC added), and the most recent worker summary."""
    g: dict[str, dict] = defaultdict(lambda: {"arms": 0, "accumulate_arms": 0, "keeps": 0,
                                              "loc_added": 0, "last_summary": ""})
    for r in rows:
        if r.get("mode") != "capability":
            continue
        s = g[r.get("lever", "?")]
        s["arms"] += 1
        bucket = _BUCKET.get(r.get("decision") or "")
        if bucket == "ACCUMULATE":
            s["accumulate_arms"] += 1
        elif bucket == "KEEP":
            s["keeps"] += 1
        s["loc_added"] += _int((r.get("diff") or {}).get("insertions"))
        if r.get("worker_summary"):
            s["last_summary"] = r["worker_summary"]
    return dict(g)


def _arm_num(path: str) -> int:
    """heldout-<n>.json -> n, for numeric (not lexicographic) ordering."""
    base = os.path.basename(path)
    try:
        return int(base.split("-")[-1].split(".")[0])
    except (ValueError, IndexError):
        return 0


def holdout_trend(state_dir: str) -> list[tuple]:
    """(name, confirmed_score, false_alarms, wrong_true) per heldout-*.json, ordered by arm number
    — the REAL anti-overfit signal. A flat trend under a rising train score = memorization."""
    pts: list[tuple] = []
    for f in sorted(glob.glob(os.path.join(state_dir, "heldout-*.json")), key=_arm_num):
        try:
            d = json.loads(open(f).read())
        except (OSError, ValueError):
            continue
        pts.append((os.path.basename(f), d.get("confirmed_score"),
                    d.get("false_alarms"), d.get("wrong_true")))
    return pts


def recent(rows: list[dict], n: int = 12) -> list[dict]:
    return rows[-n:]


def roi_oneline(rows: list[dict], top: int = 3) -> str:
    """Compact one-line lever-ROI summary for a journal note (§2c): the top-N levers by net
    confirmed points, with cost and $/point. Recomputed from the spine, so always consistent."""
    roi = lever_roi(rows)
    items = list(roi.items())[:top]
    if not items:
        return "lever ROI: n/a"
    parts = []
    for k, s in items:
        dpp = f" ${s['dollars_per_point']:.2f}/pt" if s["dollars_per_point"] else ""
        parts.append(f"{k} netΔ{s['netDelta']:+d} ${s['cost']:.2f}{dpp}")
    return "lever ROI: " + " | ".join(parts)


if __name__ == "__main__":
    cmd, state = sys.argv[1], sys.argv[2]
    rows = load_arms(os.path.join(state, "arms.jsonl"))
    if cmd == "roi":
        print(json.dumps(lever_roi(rows), indent=2))
    elif cmd == "waste":
        print(json.dumps(waste(rows), indent=2))
    elif cmd == "holdout":
        print(json.dumps(holdout_trend(state), indent=2))
    elif cmd == "caps":
        print(json.dumps(capability_milestones(rows), indent=2))
    elif cmd == "recent":
        print(json.dumps(recent(rows, int(sys.argv[3]) if len(sys.argv) > 3 else 12), indent=2))
    else:
        print("usage: report_view.py roi|waste|holdout|caps|recent <state_dir> [n]", file=sys.stderr)
        sys.exit(2)
