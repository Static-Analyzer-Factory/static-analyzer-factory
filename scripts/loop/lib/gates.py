"""Deterministic, testable gate decisions for the SAF autonomous improvement loop (plan 204).

Pure functions with no Claude, no network, no Docker: they take already-collected inputs
(the immutable scorer's JSON, a manifest of sha256s, a worker transcript) and return the
supervisor's keep/revert/reject decision.

GATE MODEL (user decision 2026-08-16): only TWO things gate an arm — the anti-reward-hacking
checks (no scorer/label/harness tamper, held-out set not read) and whether the CONFIRMED score
IMPROVED. Soundness is NOT a separate gate: it is baked into the score (a false alarm is -16, a
wrong TRUE -32), so unsound changes lower the score, while a large real gain can outweigh minor
unsoundness (aim for soundness, but big gains are acceptable). Build/tests/clippy are NOT gated —
they are the loop AGENT's responsibility (a non-building arm just scores nothing -> not improved).
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Iterable


def _iter_tool_inputs(obj: object) -> Iterable[dict]:
    """Yield every tool_use `input` dict found anywhere in a parsed transcript record."""
    if isinstance(obj, dict):
        if obj.get("type") == "tool_use" and isinstance(obj.get("input"), dict):
            yield obj["input"]
        for v in obj.values():
            yield from _iter_tool_inputs(v)
    elif isinstance(obj, list):
        for v in obj:
            yield from _iter_tool_inputs(v)


def audit_forbidden_reads(transcript_lines: Iterable[str], forbidden: list[str]) -> list[str]:
    """Post-hoc backstop for the holdout-not-read invariant. Scan a worker's stream-json/JSONL
    transcript and return the sorted subset of `forbidden` path substrings that appear in ANY
    tool_use input — a Read `file_path`, a Bash `command` (`cat holdout`), a Grep path, etc.
    Empty = clean. Non-JSON lines are ignored. The preventive control is a PreToolUse denylist
    hook; this audit is the fail-closed second layer (Addendum A6: isolation = BOTH).
    """
    touched: set[str] = set()
    for line in transcript_lines:
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except (ValueError, TypeError):
            continue
        for inp in _iter_tool_inputs(rec):
            blob = json.dumps(inp)
            for f in forbidden:
                if f in blob:
                    touched.add(f)
    return sorted(touched)


def soundness_ok(scorer: dict) -> bool:
    """INFORMATIONAL soundness read (no longer a gate): does the immutable scorer report ZERO false
    alarms and ZERO wrong TRUEs? Reported in the verdict so kept arms' soundness is visible; not used
    to veto (the score's -16/-32 already penalizes unsoundness). Missing keys -> not sound (fail-closed
    for the reported flag). `scorer` is svcomp_split_eval.py's -o JSON.
    """
    return scorer.get("false_alarms", 1) == 0 and scorer.get("wrong_true", 1) == 0


def confirmed_delta(before: dict, after: dict) -> int:
    """Change in the CONFIRMED (competition-predictive) score. A kept arm requires this > 0. If the
    after-eval is missing its score (e.g. the arm broke the build so no after.json was written),
    return a large negative so the arm counts as a regression and reverts — build health is the
    agent's job, enforced implicitly by "no score -> not improved", not by a separate build gate.
    """
    a = after.get("confirmed_score")
    if a is None:
        return -(10 ** 9)
    return a - before.get("confirmed_score", 0)


def family_regression(before: dict, after: dict) -> list[str]:
    """Return the sorted property families whose CONFIRMED score DROPPED from `before` to `after`,
    read from the scorer's per-property breakdown (svcomp_split_eval.py's `per_property` map).

    Used ONLY for CROSS-CUTTING arms — changes to shared infrastructure (frontend / PTA / AIR /
    program slicing) that can move EVERY property, not just the lever's nominal family. Such an arm is
    eval'd on ALL properties and reverted if ANY family regressed, even when the OVERALL score rose:
    SAF must never trade one property's recall away for another's when touching shared code. Local
    (single-confirmer) arms don't use this — they are eval'd on their own family only, and the periodic
    all-property checkpoint backstops any leakage. Missing/empty `per_property` -> [] (no detectable
    regression; the total `confirmed_delta` still gates and the checkpoint is the real backstop).
    """
    pb = before.get("per_property") or {}
    pa = after.get("per_property") or {}
    regressed: list[str] = []
    for fam, b in pb.items():
        before_conf = (b or {}).get("confirmed", 0)
        after_conf = (pa.get(fam) or {}).get("confirmed", 0)
        if after_conf < before_conf:
            regressed.append(fam)
    return sorted(regressed)


def decide(*, immutable_violations: list[str], forbidden_reads: list[str],
           delta: int, progressed: bool, family_regressed: bool = False) -> str:
    """The supervisor's decision for one arm. Returns exactly one of
    REJECT_TAMPER / REJECT_HOLDOUT / KEEP / ACCUMULATE / REVERT.

    Anti-reward-hacking violations come FIRST (they ALERT, not merely revert). Then:
      * `family_regressed` -> REVERT  (a CROSS-CUTTING arm dropped some OTHER property's confirmed
                        score — never trade one property's recall for another when touching shared
                        frontend/PTA/AIR code; checked only for crosscut levers, see `family_regression`)
      * `delta > 0`  -> KEEP        (a real score gain — a win; the integration branch advances)
      * `delta == 0 and progressed` -> ACCUMULATE  (useful, score-NEUTRAL work — it compiles, its
                        tests pass, and it changed something; we PRESERVE it on the integration branch
                        so future arms build ON it instead of re-deriving it — user ask 2026-08-16)
      * otherwise    -> REVERT      (a regression `delta < 0`, or nothing useful — discarded)
    Soundness stays intrinsic to the score (a false alarm is -16); ACCUMULATE never lowers the score
    (delta == 0), so the integration branch is monotonic non-decreasing.
    """
    if immutable_violations:
        return "REJECT_TAMPER"
    if forbidden_reads:
        return "REJECT_HOLDOUT"
    if family_regressed:
        return "REVERT"
    if delta > 0:
        return "KEEP"
    if delta == 0 and progressed:
        return "ACCUMULATE"
    return "REVERT"


def weighted_confirmed(scorer: dict) -> int | None:
    """The per-cluster-DEDUPED confirmed score (svcomp_split_eval.py --group-weight → `confirmed_score_weighted`).
    Capping each generator cluster at 1 point means confirming 300 near-duplicate Juliet tasks moves this by at
    most 1 — so it measures DISTINCT solving power, not pool volume. Returns None if the (weighted) eval is
    missing (broken build / no --group-weight), so callers treat it as a regression."""
    return scorer.get("confirmed_score_weighted")


def generalization_delta(val_before: dict, val_after: dict) -> int:
    """Change in the DEDUPED confirmed score on the reasoning VALIDATION set (`val.jsonl` ⊂ train, Juliet/
    generator clusters excluded). This is the quantity the loop now maximizes: a gain here is real solving power
    on novel-shaped tasks, not memorization of a repeated cluster. Missing after-eval → large negative (regression)."""
    a = weighted_confirmed(val_after)
    if a is None:
        return -(10 ** 9)
    return a - (weighted_confirmed(val_before) or 0)


def pool_guard_delta_w(pool_before: dict, pool_after: dict) -> int:
    """Change in the DEDUPED confirmed score on the (Juliet-inclusive) TRAIN pool — the no-regression GUARD.
    A KEEP must not lower it (never trade pool recall away); a pool-only gain with val flat is a KEEP_POOL.
    Deduping means 'confirm 3 of 300 near-dups' ≈ 0 movement, so memorization can't even tie-break. Missing
    after-eval → large negative."""
    a = weighted_confirmed(pool_after)
    if a is None:
        return -(10 ** 9)
    return a - (weighted_confirmed(pool_before) or 0)


def decide_v2(*, immutable_violations: list[str], forbidden_reads: list[str],
              gen_delta_w: int, pool_delta_w: int, novel_solved: bool = False,
              progressed: bool = False, family_regressed: bool = False) -> str:
    """Generalization-aware decision (LOOP_GEN_MODE=on). Returns one of
    REJECT_TAMPER / REJECT_HOLDOUT / KEEP / KEEP_POOL / ACCUMULATE_PLUS / ACCUMULATE / REVERT.

    The loop now maximizes DEDUPED reasoning-set recall, not confirmed FALSEs on a Juliet-dominated sample:
      * `gen_delta_w > 0 AND pool_delta_w >= 0` -> KEEP         (real generalization gain; pool not regressed)
      * `pool_delta_w > 0 AND gen_delta_w >= 0` -> KEEP_POOL    (honest Juliet points, val flat — BANKED, but the
                          supervisor does NOT reset the lever's revert-stall, so pure-memorization levers still park)
      * `novel_solved AND both deltas >= 0`     -> ACCUMULATE_PLUS  (a capability arm confirmed its FIRST novel task
                          — preserve its source AND give it priority/budget instead of the park countdown)
      * `gen==0 AND pool==0 AND progressed`     -> ACCUMULATE   (useful score-neutral work — compiles+tests+diff)
      * otherwise (either set regressed, or a no-op)            -> REVERT
    Anti-cheat rejects come first; a crosscut arm that regressed another family is a hard REVERT. Neither KEEP
    variant may lower the deduped pool (`pool_delta_w >= 0` is required), so the pool score is monotone."""
    if immutable_violations:
        return "REJECT_TAMPER"
    if forbidden_reads:
        return "REJECT_HOLDOUT"
    if family_regressed:
        return "REVERT"
    if gen_delta_w > 0 and pool_delta_w >= 0:
        return "KEEP"
    if pool_delta_w > 0 and gen_delta_w >= 0:
        return "KEEP_POOL"
    if novel_solved and gen_delta_w >= 0 and pool_delta_w >= 0:
        return "ACCUMULATE_PLUS"
    if gen_delta_w == 0 and pool_delta_w == 0 and progressed:
        return "ACCUMULATE"
    return "REVERT"


def sha256_file(p: Path) -> str:
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def verify_immutables(manifest: dict[str, str], root: Path) -> list[str]:
    """Return the sorted list of manifest-listed paths whose current sha256 differs from the
    frozen value (tamper) or that are now missing. Empty list = every immutable is intact.

    `manifest` maps repo-relative path -> expected hex sha256; `root` is the repo root. This is
    the primary reward-hacking defense: the agent may ADD confirmers but never edit the scorer,
    audit, labels, or holdout — any drift here rejects the arm before its score is trusted.
    """
    violations: list[str] = []
    root = Path(root)
    for rel, expected in manifest.items():
        p = root / rel
        if not p.exists() or sha256_file(p) != expected:
            violations.append(rel)
    return sorted(violations)
