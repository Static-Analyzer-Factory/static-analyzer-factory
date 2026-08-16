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


def run(state_dir: str, arm_n: int, holdout_json: str, *, min_lift: int, stall_to_park: int = 2) -> str:
    """Apply the holdout brake for one checkpoint. Returns a one-line human summary for the journal/log.

    Reads/writes only files under `state_dir`:
      heldout_weighted_prev, last_heldout_arm, arms.jsonl, and per lever:
      lever.<id>.holdout_stall / .gen_credit / .stall / .parked, plus ALERT_OVERFIT_<id> on a park.
    """
    sd = Path(state_dir)
    try:
        curr = gates.weighted_confirmed(json.loads(Path(holdout_json).read_text()))
    except (OSError, ValueError, TypeError):
        curr = None
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

    increased = float(curr) > float(prev)
    holdout_stall = {lever: _read_int(sd / f"lever.{lever}.holdout_stall", 0) for lever in lift}
    upd = gates.holdout_lever_updates(holdout_increased=increased, per_lever_lift=lift,
                                      holdout_stall=holdout_stall, min_lift=min_lift,
                                      stall_to_park=stall_to_park)

    for lever, n in upd["new_stall"].items():
        _write(sd / f"lever.{lever}.holdout_stall", n)
    for lever in upd["boost"]:
        gc = _read_int(sd / f"lever.{lever}.gen_credit", 0) + 1
        _write(sd / f"lever.{lever}.gen_credit", gc)      # priority nudge in pick_lever (within-family)
        _write(sd / f"lever.{lever}.stall", 0)            # a reproduced gain resets the revert-stall
    for lever in upd["park"]:
        (sd / f"lever.{lever}.parked").touch()
        (sd / f"ALERT_OVERFIT_{lever}").touch()

    trend = "UP" if increased else "flat/down"
    parts = [f"holdout weighted {prev}->{curr} ({trend})"]
    if upd["boost"]:
        parts.append("BOOST " + ",".join(upd["boost"]))
    if upd["park"]:
        parts.append("PARK-OVERFIT " + ",".join(upd["park"]))
    if not upd["boost"] and not upd["park"]:
        stalled = ",".join(f"{k}:{v}" for k, v in sorted(upd["new_stall"].items()))
        parts.append(f"stalled {stalled}" if stalled else "no lever crossed a threshold")
    return "; ".join(parts)


if __name__ == "__main__":
    # argv: <state_dir> <arm_n> <holdout_json> <min_lift> [stall_to_park]
    _sd, _n, _hj, _ml = sys.argv[1], int(sys.argv[2]), sys.argv[3], int(sys.argv[4])
    _stp = int(sys.argv[5]) if len(sys.argv) > 5 else 2
    print(run(_sd, _n, _hj, min_lift=_ml, stall_to_park=_stp))
