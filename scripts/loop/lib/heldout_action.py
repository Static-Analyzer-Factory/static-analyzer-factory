#!/usr/bin/env python3
"""Actionable svcomp26-holdout brake (plan 205 §1a/§2.4) — finally wires the "a train gain that does not
reproduce on held-out is a HARD reject" rule that `maybe_heldout_check` used to only journal.

The holdout eval is GLOBAL (scores the whole integration branch) and READ-FORBIDDEN to the worker. This
runs SUPERVISOR-SIDE ONLY, AFTER the per-arm KEEP is already banked, so it can only *lower* a lever's
standing (park it as overfit) or *raise* it (a gen_credit priority boost + stall reset) — it can never
keep a bad change and it never influences the per-arm KEEP. Movement on the holdout is attributed to the
levers that banked reasoning-set (`val`) lift since the previous checkpoint (from arms.jsonl), because the
holdout number itself is not per-lever.

All state lives in files under STATE_DIR, so the step is deterministic given them (no network/Docker/Claude,
no wall-clock, no hash). The pure decision is `gates.holdout_lever_updates`; the attribution is
`report_view.lift_since`; this module is the thin IO orchestrator, unit-tested against a temp STATE_DIR.
"""
from __future__ import annotations

import json
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gates  # noqa: E402
import report_view as rv  # noqa: E402


def _read_int(p: Path, default: int = 0) -> int:
    try:
        return int(float(p.read_text().strip()))
    except (OSError, ValueError):
        return default


def _read_float(p: Path):
    try:
        return float(p.read_text().strip())
    except (OSError, ValueError):
        return None


def _write(p: Path, val) -> None:
    try:
        p.write_text(str(val))
    except OSError:
        pass


def run(state_dir: str, arm_n: int, holdout_json: str, *, min_lift: int, stall_to_park: int = 2,
        min_family_tasks: int = 10) -> str:
    """Apply the holdout brake for one checkpoint. Returns a one-line human summary for the journal/log.

    Reads/writes only files under `state_dir`:
      heldout_weighted_prev, last_heldout_arm, arms.jsonl, and per lever:
      lever.<id>.holdout_stall / .gen_credit / .stall / .parked, plus ALERT_OVERFIT_<id> on a park.

    2026-08-20 fix: the dedup-weighted holdout is tiny (order 1 distinct point), so a FLAT holdout is
    the default outcome and is NOT evidence of overfit. A lever is now only stalled/parked on an actual
    weighted REGRESSION, and only for a family the holdout has >= `min_family_tasks` tasks to resolve
    (so e.g. a 1-task termination holdout can never park a lever).
    """
    sd = Path(state_dir)
    try:
        doc = json.loads(Path(holdout_json).read_text())
    except (OSError, ValueError, TypeError):
        doc = None
    curr = gates.weighted_confirmed(doc) if isinstance(doc, dict) else None
    if curr is None:
        # scorer ran without --group-weight (legacy) or the eval failed — cannot compare; leave the
        # checkpoint pointers untouched so the NEXT weighted check still has a valid baseline.
        return "holdout: no weighted score (scorer without --group-weight or eval failed); no lever action"

    prev = _read_float(sd / "heldout_weighted_prev")
    old_last_arm = _read_int(sd / "last_heldout_arm", 0)

    rows = rv.load_arms(str(sd / "arms.jsonl"))
    lift = rv.lift_since(rows, old_last_arm)

    # advance the checkpoint pointers now that we have a valid weighted number
    _write(sd / "heldout_weighted_prev", curr)
    _write(sd / "last_heldout_arm", int(arm_n))

    if prev is None:
        return f"holdout: first weighted checkpoint = {curr} (baseline; no lever action yet)"

    # 3-state direction of the DEDUP-WEIGHTED holdout: only 'down' (a real regression) counts against a
    # lever; 'flat' is inconclusive (the holdout is too small to treat non-movement as a verdict).
    if float(curr) > float(prev):
        direction = "up"
    elif float(curr) < float(prev):
        direction = "down"
    else:
        direction = "flat"

    # Power guard: a lever can only be parked if its family has >= min_family_tasks held-out tasks, i.e.
    # the holdout can actually MEASURE that family's generalization (kills n~=1 false-parks structurally).
    per_prop = doc.get("per_property") if isinstance(doc, dict) else None
    if not isinstance(per_prop, dict):
        per_prop = {}
    resolvable_families = {fam for fam, st in per_prop.items()
                           if isinstance(st, dict) and int(st.get("n", 0) or 0) >= min_family_tasks}
    lever_family = {r.get("lever"): r.get("family") for r in rows if r.get("lever") and r.get("family")}
    resolvable_levers = {lv for lv in lift if lever_family.get(lv) in resolvable_families}

    holdout_stall = {lever: _read_int(sd / f"lever.{lever}.holdout_stall", 0) for lever in lift}
    upd = gates.holdout_lever_updates(holdout_direction=direction, per_lever_lift=lift,
                                      holdout_stall=holdout_stall, min_lift=min_lift,
                                      resolvable_levers=resolvable_levers, stall_to_park=stall_to_park)

    for lever, n in upd["new_stall"].items():
        _write(sd / f"lever.{lever}.holdout_stall", n)
    for lever in upd["boost"]:
        gc = _read_int(sd / f"lever.{lever}.gen_credit", 0) + 1
        _write(sd / f"lever.{lever}.gen_credit", gc)      # priority nudge in pick_lever (within-family)
        _write(sd / f"lever.{lever}.stall", 0)            # a reproduced gain resets the revert-stall
    for lever in upd["park"]:
        (sd / f"lever.{lever}.parked").touch()
        (sd / f"ALERT_OVERFIT_{lever}").touch()

    parts = [f"holdout weighted {prev}->{curr} ({direction})"]
    if upd["boost"]:
        parts.append("BOOST " + ",".join(upd["boost"]))
    if upd["park"]:
        parts.append("PARK-OVERFIT " + ",".join(upd["park"]))
    if not upd["boost"] and not upd["park"]:
        stalled = ",".join(f"{k}:{v}" for k, v in sorted(upd["new_stall"].items()))
        if stalled:
            parts.append(f"stalled {stalled}")
        elif direction == "flat":
            parts.append("flat: inconclusive (holdout too small to resolve), no lever action")
        else:
            parts.append("no lever crossed a threshold")
    return "; ".join(parts)


if __name__ == "__main__":
    # argv: <state_dir> <arm_n> <holdout_json> <min_lift> [stall_to_park]
    # min_family_tasks comes from $HELDOUT_MIN_FAMILY_TASKS (default 10) so no supervisor.sh arg change.
    _sd, _n, _hj, _ml = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4])
    _stp = int(sys.argv[5]) if len(sys.argv) > 5 else 2
    try:
        _mft = int(os.environ.get("HELDOUT_MIN_FAMILY_TASKS", "10") or "10")
    except ValueError:
        _mft = 10
    print(run(_sd, _n, _hj, min_lift=_ml, stall_to_park=_stp, min_family_tasks=_mft))
