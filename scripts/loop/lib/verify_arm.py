#!/usr/bin/env python3
"""The bash<->python boundary the supervisor shells out to after each arm.

Collects the already-produced inputs (baseline + post-arm scorer JSON, the immutable manifest, the
worker transcript), runs the pure gate functions in `gates.py`, and prints ONE decision word on the
last stdout line:

    REJECT_TAMPER | REJECT_HOLDOUT | REVERT | KEEP

The decision is exactly: anti-cheat (no tamper, no holdout read) + did the CONFIRMED score improve.
Soundness is intrinsic to the score (reported, not gated); build/tests/clippy are the agent's job.
Exit code: 0 iff KEEP, else 1.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gates  # noqa: E402


def _load(p: str | None) -> dict:
    """Tolerant load: a missing/corrupt scorer JSON (e.g. an arm that broke the build so `after.json`
    was never written) returns {} -> soundness_ok({}) is False -> the arm reverts, never crashes."""
    if not p:
        return {}
    fp = Path(p)
    if not fp.exists():
        return {}
    try:
        return json.loads(fp.read_text())
    except (ValueError, OSError):
        return {}


def revert_reason(*, decision: str, after_present: bool, family_regressed: bool,
                  delta: int) -> str | None:
    """Name the PROXIMATE cause a non-kept arm was discarded (plan 205 §5d) — observability only,
    never a gate. Returns one of tamper | holdout | build_broken | family_trade | regression |
    no_gain, or None for any keep-family decision. Lets the dashboard separate "lever produces bad
    changes" (regression/family_trade) from "lever produces no-ops" (no_gain) — different fixes.
    `delta` is the driving score delta (confirmed in legacy mode; min(gen,pool) in gen mode)."""
    if decision == "REJECT_TAMPER":
        return "tamper"
    if decision == "REJECT_HOLDOUT":
        return "holdout"
    if decision not in ("REVERT", "WORKER_FAIL"):
        return None  # KEEP / KEEP_POOL / ACCUMULATE / ACCUMULATE_PLUS
    if not after_present:
        return "build_broken"
    if family_regressed:
        return "family_trade"
    if delta < 0:
        return "regression"
    return "no_gain"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--before", required=True, help="baseline scorer JSON (svcomp_split_eval -o)")
    ap.add_argument("--after", required=True, help="post-arm scorer JSON")
    ap.add_argument("--immutable-manifest", required=True, help="JSON map relpath -> sha256")
    ap.add_argument("--repo-root", required=True)
    ap.add_argument("--transcript", required=True, help="worker stream-json/JSONL transcript")
    ap.add_argument("--forbidden", action="append", default=[],
                    help="forbidden path substring (repeatable) — the holdout manifest(s)")
    ap.add_argument("--progressed", choices=["0", "1"], default="0",
                    help="1 iff (score-neutral) work is USEFUL: compiles + tests pass + non-empty diff "
                         "(supervisor-computed; only consulted when delta == 0 -> ACCUMULATE vs REVERT)")
    ap.add_argument("--check-all-families", action="store_true",
                    help="CROSS-CUTTING arm: before/after are ALL-property evals; REVERT if any family's "
                         "confirmed score dropped (never trade one property's recall for another).")
    # --- generalization mode (LOOP_GEN_MODE=on): --before/--after are the VAL (reasoning) evals with
    # --group-weight; --pool-before/--pool-after are the deduped TRAIN-pool guard evals. decide_v2 replaces
    # decide: a KEEP needs a val gain with no pool regression; a pool-only gain is KEEP_POOL. ---
    ap.add_argument("--gen-mode", action="store_true", help="use the generalization gate (decide_v2)")
    ap.add_argument("--pool-before", default=None, help="gen-mode: deduped TRAIN-pool scorer JSON (before)")
    ap.add_argument("--pool-after", default=None, help="gen-mode: deduped TRAIN-pool scorer JSON (after)")
    ap.add_argument("--novel-solved", choices=["0", "1"], default="0",
                    help="gen-mode: 1 iff a capability arm made genuine novel-solving progress (-> ACCUMULATE_PLUS)")
    ap.add_argument("--verdict-out", default=None, help="optional: write a JSON verdict record here")
    args = ap.parse_args(argv)

    before, after = _load(args.before), _load(args.after)
    manifest = _load(args.immutable_manifest)
    transcript_lines = Path(args.transcript).read_text().splitlines() if Path(args.transcript).exists() else []

    immutable_violations = gates.verify_immutables(manifest, Path(args.repo_root))
    forbidden_reads = gates.audit_forbidden_reads(transcript_lines, args.forbidden)
    progressed = args.progressed == "1"
    delta = gates.confirmed_delta(before, after)

    verdict = {
        "immutable_violations": immutable_violations, "forbidden_reads": forbidden_reads,
        "progressed": progressed,
        "sound": gates.soundness_ok(after),  # informational only
        "false_alarms": after.get("false_alarms"), "wrong_true": after.get("wrong_true"),
    }

    if args.gen_mode:
        # --before/--after are the VAL (reasoning) evals; --pool-* are the deduped-pool guard.
        pool_before, pool_after = _load(args.pool_before), _load(args.pool_after)
        gen_delta_w = gates.generalization_delta(before, after)
        pool_delta_w = gates.pool_guard_delta_w(pool_before, pool_after)
        # a cross-cutting arm must not regress any family's DEDUPED pool score
        regressed_families = gates.family_regression(pool_before, pool_after) if args.check_all_families else []
        decision = gates.decide_v2(
            immutable_violations=immutable_violations, forbidden_reads=forbidden_reads,
            gen_delta_w=gen_delta_w, pool_delta_w=pool_delta_w,
            novel_solved=args.novel_solved == "1", progressed=progressed,
            family_regressed=bool(regressed_families),
        )
        verdict.update({"decision": decision, "gen_mode": True, "gen_delta_w": gen_delta_w,
                        "pool_delta_w": pool_delta_w, "novel_solved": args.novel_solved == "1",
                        "regressed_families": regressed_families})
        after_present = gates.weighted_confirmed(after) is not None and gates.weighted_confirmed(pool_after) is not None
        verdict["revert_reason"] = revert_reason(
            decision=decision, after_present=after_present,
            family_regressed=bool(regressed_families), delta=min(gen_delta_w, pool_delta_w))
    else:
        regressed_families = gates.family_regression(before, after) if args.check_all_families else []
        decision = gates.decide(
            immutable_violations=immutable_violations, forbidden_reads=forbidden_reads,
            delta=delta, progressed=progressed, family_regressed=bool(regressed_families),
        )
        verdict.update({"decision": decision, "gen_mode": False, "confirmed_delta": delta,
                        "check_all_families": args.check_all_families, "regressed_families": regressed_families})
        verdict["revert_reason"] = revert_reason(
            decision=decision, after_present=after.get("confirmed_score") is not None,
            family_regressed=bool(regressed_families), delta=delta)

    if args.verdict_out:
        Path(args.verdict_out).write_text(json.dumps(verdict, indent=2))

    print(decision)
    return 0 if decision in ("KEEP", "KEEP_POOL", "ACCUMULATE", "ACCUMULATE_PLUS") else 1


if __name__ == "__main__":
    sys.exit(main())
