#!/usr/bin/env python3
"""Per-arm task-flip capture (plan 205 observability §5a) — the single most important
generalization instrument.

The scorer's `--per-task` JSONL dump lists every task's outcome (`property, expected, kind,
outcome, witness, rel_yml, group, data_model, duration_s, stderr_tail`). Diffing the before/after
dumps by `rel_yml` (the stable task id) tells the operator EXACTLY which individual tasks an arm
flipped — the difference between "added real recall" (solved genuinely new tasks) and "memorized
three task shapes" (re-confirmed tasks a prior arm already covered).

Pure + deterministic: same inputs -> byte-identical output (sorted lists). Tolerant of a
missing/empty/corrupt dump (a broken-build arm never writes `after.pertask.jsonl`) -> empty index,
never raises. Observability only — never gates a decision.
"""
from __future__ import annotations

import json


def _index(path: str) -> dict[str, dict]:
    """Map rel_yml -> row for one per-task JSONL dump. Missing/unreadable file or malformed lines
    are skipped silently (returns whatever parsed), so a broken arm degrades to empty, not a crash."""
    out: dict[str, dict] = {}
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
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
        rel = rec.get("rel_yml")
        if rel:
            out[rel] = rec
    return out


def _is_confirmed(row: dict) -> bool:
    """A task scores a point ONLY as a confirmed FALSE: outcome FalseCorrect AND witness CONFIRMED.
    A correct-but-unconfirmed FALSE earns 0, so it is NOT a solved task for flip purposes."""
    return row.get("outcome") == "FalseCorrect" and row.get("witness") == "CONFIRMED"


def flips(before_jsonl: str, after_jsonl: str) -> dict[str, list[str]]:
    """Which individual tasks changed outcome between two per-task dumps, keyed by `rel_yml`:

      * gained_confirmed  — became a confirmed FALSE and was not one before (a NEW solve)
      * lost_confirmed    — was a confirmed FALSE before and is no longer (a regression)
      * new_false_alarms  — became a FalseIncorrect (correct-TRUE task called FALSE) and was not before
      * new_wrong_true    — became a TrueIncorrect (expected-FALSE task called TRUE) and was not before

    `new_false_alarms` (-16) and `new_wrong_true` (-32) are the two soundness blockers that trip the hard
    KEEP gate. Surfacing their exact task ids (lever_outcome_digest) is what lets the next arm ABSTAIN on
    them instead of re-deriving the same too-permissive gate blind. All lists are sorted for determinism.
    """
    b, a = _index(before_jsonl), _index(after_jsonl)
    if not a:
        # No post-arm evidence at all (a broken-build arm never writes an `after` dump, and the scorer
        # only writes one when --per-task is passed). Report NOTHING rather than flagging every
        # previously-confirmed task as "lost" — that would fabricate a full mass regression signal.
        return {"gained_confirmed": [], "lost_confirmed": [], "new_false_alarms": [], "new_wrong_true": []}
    gained = [k for k, r in a.items() if _is_confirmed(r) and not _is_confirmed(b.get(k, {}))]
    lost = [k for k, r in b.items() if _is_confirmed(r) and not _is_confirmed(a.get(k, {}))]
    new_fp = [k for k, r in a.items()
              if r.get("outcome") == "FalseIncorrect" and b.get(k, {}).get("outcome") != "FalseIncorrect"]
    new_wt = [k for k, r in a.items()
              if r.get("outcome") == "TrueIncorrect" and b.get(k, {}).get("outcome") != "TrueIncorrect"]
    return {"gained_confirmed": sorted(gained),
            "lost_confirmed": sorted(lost),
            "new_false_alarms": sorted(new_fp),
            "new_wrong_true": sorted(new_wt)}


if __name__ == "__main__":
    import sys
    print(json.dumps(flips(sys.argv[1], sys.argv[2])))
