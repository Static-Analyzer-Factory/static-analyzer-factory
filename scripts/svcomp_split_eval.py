#!/usr/bin/env python3
"""Manifest-driven, property-generic blind `saf verify` eval with CONFIRMED scoring
(plan 203).

Runs the REAL `saf verify` binary blind over the tasks in a split manifest
(`train.jsonl` / `holdout.jsonl` from scripts/svcomp_split.py), then scores the
run TWO ways so the gap between them is explicit:

  * RAW score      — the naive verdict-vs-expected score (SvCompOutcome in
                     crates/saf-bench/src/svcomp/scoring.rs): +2 correct-TRUE,
                     +1 correct-FALSE, -16 false-alarm, -32 wrong-TRUE, 0 unknown.
  * CONFIRMED score — the OFFICIAL score under the **SV-COMP 2027** rules: a correct
                     verdict earns its points only if the witness its BASE CATEGORY
                     requires is CONFIRMED by a validator (else 0 —
                     "correct-unconfirmed"); where the base category requires no
                     witness ("not supported" or "(demo mode)") the verdict scores
                     alone. Negative outcomes (-16 / -32) apply regardless of
                     confirmation — a wrong verdict is always penalized.

The requirement is per BASE CATEGORY `C.<property>.<suffix>`, NOT per property —
see `scripts/svcomp_witness_rules.py`. This matters: 2027 moved `C.termination.*`
from "2.1 (demo mode)" (free) to "2.1 or higher" (required), and SAF emits no
correctness witness for termination, so 36 of its dedup-weighted points went to 0.
Reporting the pre-2027 rule overstated the score by 41 weighted points.

Reporting RAW alone overstates the competition score, because unconfirmed-correct
results score 0 in the scored categories. Run this on the HOLDOUT manifest for an
honest generalization estimate; compare to TRAIN to see the overfit gap.

Soundness audit (per property + overall): false alarms (FALSE on an expected-TRUE
task, -16) MUST be 0, and wrong TRUEs (TRUE on an expected-FALSE task, -32) MUST
be 0 — a single one outweighs ~16 correct answers.

Run inside the dev container from the workspace root (needs the release binary and,
for --confirm-witness, witnesslint + CPAchecker per validate_witness.sh):
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
        'cargo build --release -p saf-cli && \
         python3 scripts/svcomp_split_eval.py --manifest tests/benchmarks/svcomp-splits/holdout.jsonl \
             --confirm-witness --jobs 4 -o holdout-score.json'
"""
from __future__ import annotations

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import tempfile
import time
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from svcomp_witness_rules import (  # noqa: E402
    base_categories, false_witness_requirement, load_set_membership,
    sniff_witness_format, true_witness_requirement, version_satisfies)

SAF = os.environ.get("SAF_BIN", "target/release/saf")
VALIDATE_SH = os.environ.get("SAF_VALIDATE_WITNESS", "scripts/validate_witness.sh")
# The TRUE-side counterpart: CPAchecker `correctness-witness-validation.properties`
# over SAF's YAML-2.0 `invariant_set` witness. Until this was wired in, the harness
# validated VIOLATION witnesses only, so every TRUE row was `witness: None` and
# `confirmed_score` returned 0 for the two witness-required properties no matter how
# good the witness was — understating the score rather than measuring it.
CORRECTNESS_VALIDATE_SH = os.environ.get(
    "SAF_VALIDATE_CORRECTNESS_WITNESS", "scripts/validate_correctness_witness.sh")
VERDICT_RE = re.compile(r"^(true|false\(([a-z0-9-]+)\)|unknown)$")

# The witness requirement is per BASE CATEGORY `C.<property>.<suffix>`, not per
# property — see `scripts/svcomp_witness_rules.py`, which holds the SV-COMP 2027
# table and the base-category membership map. Until 2026 this file encoded the
# rule as two flat per-property sets; that was right for 2026 and is wrong for
# 2027, most expensively for termination (36 weighted points).
SCORE = {"TrueCorrect": 2, "FalseCorrect": 1, "TrueIncorrect": -32,
         "FalseIncorrect": -16, "Unknown": 0}


def resolve_prp(svb: Path, prop: str) -> str:
    for c in (svb / "c" / "properties" / f"{prop}.prp",
              Path("tests/programs/c/svcomp") / f"{prop}.prp"):
        if c.exists():
            return str(c.resolve())
    raise SystemExit(f"no {prop}.prp found under {svb}/c/properties or fallback")


def _rss_kb(pid: int) -> int:
    try:
        with open(f"/proc/{pid}/status") as f:
            for line in f:
                if line.startswith("VmRSS:"):
                    return int(line.split()[1])
    except (OSError, ValueError, IndexError):
        return 0
    return 0


def _subtree_pids(root_pid: int) -> list[int]:
    """root_pid and ALL descendants (via /proc PPID links), root-first."""
    try:
        pids = [int(p) for p in os.listdir("/proc") if p.isdigit()]
    except OSError:
        return [root_pid]
    children: dict[int, list[int]] = {}
    for pid in pids:
        try:
            with open(f"/proc/{pid}/stat") as f:
                ppid = int(f.read().rsplit(")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            continue
        children.setdefault(ppid, []).append(pid)
    out, stack, seen = [], [root_pid], set()
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        out.append(pid)
        stack.extend(children.get(pid, []))
    return out


def _subtree_rss_mb(root_pid: int) -> int:
    """Sum resident memory (MB) of root_pid and ALL descendants — catches SAF's own PTA
    growth (unreach) AND a runaway ASan/replay harness child (memsafety/overflow)."""
    return sum(_rss_kb(p) for p in _subtree_pids(root_pid)) // 1024


def _kill_subtree(root_pid: int) -> None:
    """SIGKILL the whole saf subtree. Enumerate FIRST (before anything dies), then kill
    the process group AND every descendant pid explicitly — a forked grandchild that
    changed its group (so `killpg` misses it) is still caught, leaving no orphan hog."""
    pids = _subtree_pids(root_pid)
    try:
        os.killpg(os.getpgid(root_pid), signal.SIGKILL)
    except (ProcessLookupError, PermissionError, OSError):
        pass
    for pid in pids:
        try:
            os.kill(pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass


def run_verify(task: dict, src: str, prp: str, timeout: int,
               witness: str | None, max_rss_mb: int = 0) -> tuple[str, float, str]:
    """Run `saf verify`; return (verdict_line, wallclock_seconds, stderr_tail).
    The stderr tail carries SAF's own diagnostics (why it abstained / a compile or
    ingest failure) — the signal for improving recall on the misses. When max_rss_mb>0,
    an RSS watchdog kills the whole process group if the saf+harness subtree exceeds the
    cap (→ treated as `unknown`) so no single task can OOM a swap-less host."""
    dm = "ILP32" if task["data_model"].upper() == "ILP32" else "LP64"
    cmd = [SAF, "verify", "--property", prp, "--data-model", dm,
           "--timeout", str(timeout)]
    if witness:
        cmd += ["--witness", witness]
    cmd.append(src)
    t0 = time.monotonic()
    outf = tempfile.TemporaryFile(mode="w+", errors="replace")
    errf = tempfile.TemporaryFile(mode="w+", errors="replace")
    try:
        # start_new_session so the saf process is its own group leader -> one killpg
        # takes down saf AND its clang/harness children (no orphaned memory hog).
        proc = subprocess.Popen(cmd, stdout=outf, stderr=errf, start_new_session=True)
    except OSError as e:
        outf.close()
        errf.close()
        return "error", time.monotonic() - t0, f"(spawn failed: {e})"

    reason = None
    hard = timeout + 30
    while proc.poll() is None:
        el = time.monotonic() - t0
        if el > hard:
            reason = "timeout"
        elif max_rss_mb and _subtree_rss_mb(proc.pid) > max_rss_mb:
            reason = "rss"
        if reason:
            _kill_subtree(proc.pid)
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                pass
            break
        # 0.2s poll: the ASan/UBSan harness bombs are bounded at the source by ASan's own
        # hard_rss_limit_mb; this watchdog is the backstop for SAF's own PTA growth
        # (unreach, not sanitized) and UBSan, where growth is gradual enough for 0.2s.
        time.sleep(0.2)

    dur = time.monotonic() - t0
    outf.seek(0)
    out = outf.read()
    outf.close()
    errf.seek(0)
    err = errf.read()
    errf.close()
    if reason == "timeout":
        return "timeout", dur, "(killed: wall-clock timeout)"
    if reason == "rss":
        return "timeout", dur, f"(killed: subtree RSS > {max_rss_mb} MB)"
    lines = [ln.strip() for ln in out.splitlines() if ln.strip()]
    line = lines[-1] if lines else "(no-output)"
    err_tail = (err or "")[-600:].strip()
    return line, dur, err_tail


def witness_validator_for(kind: str, prop: str, suffixes=()) -> str | None:
    """Which validator script should confirm this verdict's witness, or `None` when the
    verdict is scored without one (so running a validator would be wasted wall-clock).

    Mirrors `confirmed_score`: a FALSE needs a confirmed VIOLATION witness, a TRUE needs
    a confirmed CORRECTNESS witness, and either is skipped when the task's 2027 base
    categories require no witness. FALSE keeps its historical behaviour of always being
    validated — even where the score does not require it — so dumps stay comparable
    across runs; only the TRUE side gates on the requirement, because a correctness
    validation re-proves the program and is far more expensive.

    `suffixes` is the task's 2027 base-category suffix set (see `svcomp_witness_rules`).
    It defaults to empty, which falls back to the property's generic cell.
    """
    if kind == "false":
        return VALIDATE_SH
    if kind == "true" and true_witness_requirement(prop, suffixes)[0]:
        return CORRECTNESS_VALIDATE_SH
    return None


def confirm_witness(task: dict, src: str, prp: str, witness: str | None,
                    timeout: int, validator: str = VALIDATE_SH) -> str:
    """Validate an emitted witness with `validator` (violation or correctness). Returns
    CONFIRMED / NOT_CONFIRMED / LINT_FAIL / LINT_ONLY (validator unavailable) /
    TIMEOUT / NO_WITNESS / ERROR."""
    if not witness or not os.path.exists(witness):
        return "NO_WITNESS"
    cmd = ["bash", validator, witness, src, prp, task["data_model"]]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return "TIMEOUT"
    o = r.stdout + r.stderr
    if "LINT_FAIL" in o:
        return "LINT_FAIL"
    if "CPACHECKER_ABSENT" in o or "CPACHECKER_SKIPPED" in o:
        return "LINT_ONLY"
    if "NOT_CONFIRMED" in o:
        return "NOT_CONFIRMED"
    if "CONFIRMED" in o:
        return "CONFIRMED"
    return "ERROR"


def classify(verdict_line: str):
    """(kind, sub) where kind in {true, false, unknown, timeout, error}."""
    if verdict_line in ("timeout", "(no-output)"):
        return ("timeout" if verdict_line == "timeout" else "error"), None
    m = VERDICT_RE.match(verdict_line)
    if not m:
        return "error", None
    if m.group(1) == "true":
        return "true", None
    if m.group(1) == "unknown":
        return "unknown", None
    return "false", m.group(2)


def raw_outcome(kind: str, expected: bool) -> str:
    if kind == "true":
        return "TrueCorrect" if expected else "TrueIncorrect"
    if kind == "false":
        return "FalseCorrect" if not expected else "FalseIncorrect"
    return "Unknown"  # unknown / timeout / error all score 0


def confirmed_score(outcome: str, prop: str, witness_status: str | None,
                    suffixes=(), sub: str | None = None,
                    witness_format: str | None = None, version_aware: bool = False) -> int:
    """Official-style points under the SV-COMP 2027 rules: gate positive credit on
    confirmation where the task's base category requires a witness; penalties stand.

    `suffixes` — the task's 2027 base-category suffixes; `sub` — the violated
    subproperty of a FALSE (drives the valid-memtrack footnote).

    `version_aware` additionally enforces the cell's format-version FLOOR against the
    witness SAF actually emitted. Off by default so that (a) every dump written before
    the `witness_format` field re-scores identically and (b) the headline stays directly
    comparable with prior A/B runs. Report both: the floor is real (SAF emits 2.0 into
    cells demanding 2.2) and it is the only signal that tells 0B whether a new witness
    failed on its CONTENT or merely on its declared version.
    """
    if outcome == "FalseCorrect":
        required, minimum = false_witness_requirement(prop, suffixes, sub)
        if not required:
            return 1
        if version_aware and not version_satisfies(witness_format, minimum):
            return 0
        return 1 if witness_status == "CONFIRMED" else 0
    if outcome == "TrueCorrect":
        required, minimum = true_witness_requirement(prop, suffixes)
        if not required:
            return 2
        if version_aware and not version_satisfies(witness_format, minimum):
            return 0
        return 2 if witness_status == "CONFIRMED" else 0
    return SCORE[outcome]  # -16 / -32 / 0 apply regardless of confirmation


def stride(rows: list[dict], n: int) -> list[dict]:
    rows = sorted(rows, key=lambda t: t["rel_yml"])
    if n <= 0 or len(rows) <= n:
        return rows
    step = len(rows) / n
    return [rows[int(i * step)] for i in range(n)]


def load_manifest(path: Path, only_prop: str | None, sample: int) -> list[dict]:
    rows = [json.loads(ln) for ln in path.read_text().splitlines() if ln.strip()]
    if only_prop:
        rows = [r for r in rows if r["property"] == only_prop]
    if sample > 0:
        by_prop: dict[str, list[dict]] = defaultdict(list)
        for r in rows:
            by_prop[r["property"]].append(r)
        picked: list[dict] = []
        for p in sorted(by_prop):
            trues = [r for r in by_prop[p] if r["expected"]]
            falses = [r for r in by_prop[p] if not r["expected"]]
            picked += stride(trues, sample // 2) + stride(falses, sample - sample // 2)
        rows = picked
    return rows


def eval_one(task: dict, svb: Path, timeout: int, confirm: bool,
             confirm_timeout: int, max_rss_mb: int = 0,
             correctness_confirm_timeout: int | None = None,
             membership: dict | None = None) -> dict:
    prp = resolve_prp(svb, task["property"])
    # The 2027 base categories this task belongs to decide whether a witness is
    # required at all. An empty set means SV-COMP runs no category containing it
    # (the `Unused_*` and `/todo` populations) — `confirmed_score` then falls back
    # to the property's generic cell rather than granting a free pass.
    sets = (membership or {}).get(task["rel_yml"], frozenset())
    suffixes = base_categories(task["property"], sets)
    # Resolve a PORTABLE source path against --svb (relative → works both on the host
    # and inside the Docker container where the repo is at /workspace). Falls back to
    # the stored absolute path for older manifests without rel_src.
    src = str(svb / task["rel_src"]) if task.get("rel_src") else task["src"]
    with tempfile.TemporaryDirectory() as d:
        w = os.path.join(d, "w.yml") if confirm else None
        line, dur, err_tail = run_verify(task, src, prp, timeout, w, max_rss_mb)
        kind, sub = classify(line)
        wstatus = None
        wformat = sniff_witness_format(w)
        validator = witness_validator_for(kind, task["property"], suffixes) if confirm else None
        if validator:
            # A correctness witness is re-verified from scratch by CPAchecker
            # (k-induction), which is slower than violation validation and needs its
            # own, larger budget — sharing the FALSE-side one truncated it.
            budget = (confirm_timeout if validator == VALIDATE_SH
                      else (correctness_confirm_timeout or confirm_timeout))
            wstatus = confirm_witness(task, src, prp, w, budget, validator)
    outcome = raw_outcome(kind, task["expected"])
    # Keep SAF's stderr tail only where it's diagnostically useful (an abstain, a
    # miss, a crash) — not on clean scoring verdicts — to bound the dump size.
    keep_err = kind in ("unknown", "timeout", "error") or outcome == "FalseIncorrect"
    return {
        "property": task["property"], "expected": task["expected"],
        "kind": kind, "sub": sub, "rel_yml": task["rel_yml"],
        "data_model": task["data_model"], "group": task.get("group", ""),
        "outcome": outcome, "raw": SCORE[outcome],
        "confirmed": confirmed_score(outcome, task["property"], wstatus, suffixes, sub,
                                     wformat, version_aware=False),
        "confirmed_va": confirmed_score(outcome, task["property"], wstatus, suffixes, sub,
                                        wformat, version_aware=True),
        "witness": wstatus, "witness_format": wformat,
        "base_categories": sorted(suffixes), "sets": sorted(sets),
        "duration_s": round(dur, 2),
        "stderr_tail": err_tail if keep_err else "",
    }


def weighted_confirmed_summary(results: list[dict], cap: int = 1, field: str = "confirmed") -> dict:
    """Per-CLUSTER-deduped confirmed score. Each origin cluster (the task's `group`, e.g. a single Juliet
    CWE family) contributes at most `cap` POSITIVE points — so confirming 300 near-duplicate tasks moves the
    number by at most `cap` — while penalties (−16/−32) pass through in FULL (a cluster can never hide a
    false alarm). This is the quantity the loop's generalization gate maximizes on the reasoning `val` set:
    distinct solving power, not pool volume. Returns `confirmed_score_weighted` + per-property
    `{confirmed_weighted, confirmed_clusters}` + `confirmed_by_cluster`.

    Attribution is keyed by (origin `group`, `property`): one origin cluster (e.g. a single Juliet CWE
    dir) frequently contains tasks of DIFFERENT properties (a memory bug AND a signed overflow), and each
    property SAF actually solves in that cluster is DISTINCT solving power. Keying by `group` alone
    attributed the whole cluster to a single property (last-writer-wins), which silently zeroed e.g.
    no-overflow's real dedup credit when its clusters overlapped memsafety. Per-(group,property) keeps the
    anti-Juliet-memorization cap fully intact (still at most `cap` positive points per (cluster,property))
    while crediting every property a cluster genuinely solves. `confirmed_by_cluster` (display) stays keyed
    by group total for backward compatibility."""
    by_cluster: dict[str, int] = defaultdict(int)               # display: per origin-group total (unchanged shape)
    by_cluster_prop: dict[tuple[str, str], int] = defaultdict(int)  # scoring: per (group, property)
    for r in results:
        g = r.get("group") or r["rel_yml"]
        pts = r.get(field, r["confirmed"])   # `field` lets the caller score the
        by_cluster[g] += pts                 # version-aware column off the same rows
        by_cluster_prop[(g, r["property"])] += pts

    def capped(c: int) -> int:
        return min(c, cap) if c > 0 else c   # cap positives per (cluster,property); keep penalties whole

    per: dict[str, dict] = defaultdict(lambda: {"confirmed_weighted": 0, "confirmed_clusters": 0})
    total = 0
    for (g, prop), c in by_cluster_prop.items():
        w = capped(c)
        total += w
        per[prop]["confirmed_weighted"] += w
        if c > 0:
            per[prop]["confirmed_clusters"] += 1
    return {
        "confirmed_score_weighted": total,
        "per_property_weighted": {p: dict(v) for p, v in per.items()},
        "confirmed_by_cluster": dict(by_cluster),
    }


def summarize(results: list[dict]) -> dict:
    per: dict[str, dict] = defaultdict(lambda: {
        "n": 0, "TP": 0, "FP": 0, "TN": 0, "FN": 0, "true_emitted": 0,
        "wrong_true": 0, "unknown": 0, "timeout": 0, "error": 0,
        "raw": 0, "confirmed": 0, "max": 0,
        "false_total": 0, "confirmed_false": 0, "emitted_false": 0,
    })
    for r in results:
        s = per[r["property"]]
        s["n"] += 1
        s["raw"] += r["raw"]
        s["confirmed"] += r["confirmed"]
        s["max"] += 2 if r["expected"] else 1
        if not r["expected"]:
            s["false_total"] += 1
        o = r["outcome"]
        if o == "FalseCorrect":
            s["TP"] += 1
            s["emitted_false"] += 1
            if r["witness"] == "CONFIRMED":
                s["confirmed_false"] += 1
        elif o == "FalseIncorrect":
            s["FP"] += 1
            s["emitted_false"] += 1  # a false alarm still emits a (bogus) witness
        elif o == "TrueCorrect":
            s["TN"] += 1
        elif o == "TrueIncorrect":
            s["wrong_true"] += 1
        if r["kind"] == "true":
            s["true_emitted"] += 1
        if r["kind"] == "unknown" and not r["expected"]:
            s["FN"] += 1
        if r["kind"] in ("unknown", "timeout", "error"):
            s[r["kind"] if r["kind"] != "unknown" else "unknown"] += 1
    return per


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--manifest", required=True, help="a <split>.jsonl from svcomp_split.py")
    ap.add_argument("--svb", default="tests/benchmarks/sv-benchmarks")
    ap.add_argument("--property", default=None, help="restrict to one property")
    ap.add_argument("--sample", type=int, default=0,
                    help="stride-sample N per property (0 = all tasks)")
    ap.add_argument("--timeout", type=int, default=60, help="--timeout per task (s)")
    ap.add_argument("--confirm-witness", action="store_true",
                    help="validate each emitted witness — a VIOLATION witness for a FALSE, and a "
                         "CORRECTNESS witness for a TRUE on unreach-call / no-overflow (needed for "
                         "CONFIRMED score; TRUE on the verdict-only properties needs no witness)")
    ap.add_argument("--confirm-timeout", type=int, default=150)
    ap.add_argument("--correctness-confirm-timeout", type=int, default=240,
                    help="separate budget for CORRECTNESS-witness validation, which re-proves the "
                         "program from scratch (CPAchecker k-induction, itself --timelimit 150s) and "
                         "so does not fit in the violation-side budget")
    ap.add_argument("--jobs", type=int, default=1, help="parallel verify workers")
    ap.add_argument("-o", "--out", default=None, help="write JSON summary to file")
    ap.add_argument("--group-weight", action="store_true",
                    help="also emit confirmed_score_weighted (per-cluster-deduped): each origin cluster "
                         "contributes at most --weight-cap POSITIVE points (penalties pass through whole). "
                         "The generalization gate's metric — rewards distinct solving, not pool volume.")
    ap.add_argument("--weight-cap", type=int, default=1)
    ap.add_argument("--out-of-competition", choices=("keep", "drop"), default="keep",
                    help="what to do with tasks in NO SV-COMP 2027 base category (the "
                         "Unused_* and /todo populations, ~19.9k of 55,690 rows — SV-COMP "
                         "never runs them). 'keep' (default) scores them under the "
                         "property's generic cell and preserves n for cross-run diffing; "
                         "'drop' models the competition population exactly.")
    ap.add_argument("--per-task", default=None,
                    help="write a per-task diagnostic JSONL (verdict, outcome, "
                         "duration, stderr tail on misses) for later inspection")
    ap.add_argument("--max-rss-mb", type=int, default=0,
                    help="RSS watchdog: kill a saf task (+ its harness children) whose "
                         "process-subtree resident memory exceeds this many MB, scoring "
                         "it unknown. 0 = disabled. Prevents a runaway PTA/alloc task "
                         "from OOM-ing a swap-less host.")
    args = ap.parse_args()

    svb = Path(args.svb)
    rows = load_manifest(Path(args.manifest), args.property, args.sample)
    label = os.path.basename(args.manifest)

    # ~50k globs over 37 `.set` files — build once, share across every worker.
    t_mem = time.time()
    membership = load_set_membership(svb)
    print(f"== .set membership: {len(membership)} tasks reachable "
          f"({time.time() - t_mem:.1f}s) ==")
    if args.out_of_competition == "drop":
        before = len(rows)
        rows = [t for t in rows
                if base_categories(t["property"], membership.get(t["rel_yml"], frozenset()))]
        print(f"== dropped {before - len(rows)} tasks in no 2027 base category "
              f"({len(rows)} remain) ==")

    print(f"== {label}: {len(rows)} tasks; confirm={args.confirm_witness}; "
          f"jobs={args.jobs}; timeout={args.timeout}s ==")

    def work(t):
        return eval_one(t, svb, args.timeout, args.confirm_witness,
                        args.confirm_timeout, args.max_rss_mb,
                        args.correctness_confirm_timeout, membership)

    # Crash-resilient: append each result as it completes (flushed) to a `.partial`
    # sidecar, so a mid-run failure (e.g. ENOSPC) never loses the whole run — the
    # partial can be resumed/analyzed. Also prints coarse progress for live monitoring.
    results = []
    partial_f = open(args.per_task + ".partial", "w") if args.per_task else None
    total = len(rows)
    if args.jobs > 1:
        from concurrent.futures import as_completed
        with ThreadPoolExecutor(max_workers=args.jobs) as ex:
            futs = [ex.submit(work, t) for t in rows]
            for fut in as_completed(futs):
                r = fut.result()
                results.append(r)
                if partial_f:
                    partial_f.write(json.dumps(r) + "\n"); partial_f.flush()
                if len(results) % 500 == 0:
                    print(f"  progress: {len(results)}/{total} tasks", flush=True)
    else:
        for t in rows:
            r = work(t)
            results.append(r)
            if partial_f:
                partial_f.write(json.dumps(r) + "\n"); partial_f.flush()
    if partial_f:
        partial_f.close()

    if args.per_task:
        with open(args.per_task, "w") as f:
            for r in sorted(results, key=lambda r: (r["property"], r["rel_yml"])):
                f.write(json.dumps(r) + "\n")
        misses = sum(1 for r in results
                     if not r["expected"] and r["kind"] in ("unknown", "timeout", "error"))
        print(f"  per-task dump: {args.per_task} ({len(results)} tasks, "
              f"{misses} missed FALSEs with stderr tails)")

    per = summarize(results)
    tot = {"raw": 0, "confirmed": 0, "max": 0, "FP": 0, "wrong_true": 0,
           "true_emitted": 0, "TP": 0, "false_total": 0, "confirmed_false": 0,
           "emitted_false": 0, "n": 0}
    lint_only = sum(1 for r in results if r["witness"] == "LINT_ONLY")

    print(f"\n{'property':<17} {'n':>5} {'TP':>4} {'FP':>4} {'wT':>3} "
          f"{'rawScore':>9} {'confScore':>10} {'maxS':>7} "
          f"{'recallRaw':>10} {'recallConf':>11} {'wConf%':>7}")
    for p in sorted(per):
        s = per[p]
        for k in tot:
            tot[k] += s.get(k, 0)
        rr = f"{s['TP']}/{s['false_total']}"
        rc = f"{s['confirmed_false']}/{s['false_total']}"
        wc = (100.0 * s["confirmed_false"] / s["emitted_false"]) if s["emitted_false"] else 0.0
        print(f"{p:<17} {s['n']:>5} {s['TP']:>4} {s['FP']:>4} {s['wrong_true']:>3} "
              f"{s['raw']:>9} {s['confirmed']:>10} {s['max']:>7} "
              f"{rr:>10} {rc:>11} {wc:>6.0f}%")

    print(f"\n{'TOTAL':<17} {tot['n']:>5} {tot['TP']:>4} {tot['FP']:>4} "
          f"{tot['wrong_true']:>3} {tot['raw']:>9} {tot['confirmed']:>10} {tot['max']:>7}")
    print(f"\n  *** SOUNDNESS: false alarms (FP) = {tot['FP']} [MUST be 0];  "
          f"wrong TRUE = {tot['wrong_true']} [MUST be 0] ***")
    print(f"  RAW score      = {tot['raw']}  ({pct(tot['raw'], tot['max'])} of max {tot['max']})")
    print(f"  CONFIRMED score= {tot['confirmed']}  "
          f"({pct(tot['confirmed'], tot['max'])} of max)  <-- the competition-predictive number")
    gap = tot["raw"] - tot["confirmed"]
    print(f"  confirmed-vs-raw gap = {gap} points lost to unconfirmed-but-correct FALSEs")
    if args.confirm_witness and lint_only:
        print(f"  WARNING: {lint_only} witnesses could not be validated (validator "
              f"absent → LINT_ONLY). CONFIRMED score is a LOWER BOUND; install "
              f"CPAchecker (validate_witness.sh) for a real number.")

    ok = tot["FP"] == 0 and tot["wrong_true"] == 0
    if args.out:
        out_obj = {
            "manifest": args.manifest, "n": tot["n"],
            "raw_score": tot["raw"], "confirmed_score": tot["confirmed"],
            "max_score": tot["max"], "false_alarms": tot["FP"],
            "wrong_true": tot["wrong_true"], "lint_only": lint_only,
            "per_property": per,
        }
        if args.group_weight:
            out_obj.update(weighted_confirmed_summary(results, args.weight_cap))
            # The version-AWARE column, scored off the same rows: identical except
            # that a witness below its base category's format floor scores 0. SAF
            # emits 2.0 into cells demanding 2.1/2.2, so this is the stricter and
            # more honest total; the blind one stays the headline because it is
            # what every prior A/B is comparable against.
            va = weighted_confirmed_summary(results, args.weight_cap, field="confirmed_va")
            out_obj["confirmed_score_weighted_version_aware"] = va["confirmed_score_weighted"]
            out_obj["per_property_weighted_version_aware"] = va["per_property_weighted"]
            out_obj["confirmed_score_version_aware"] = sum(
                r.get("confirmed_va", r["confirmed"]) for r in results)
        out_obj["rules_edition"] = "SV-COMP 2027"
        out_obj["out_of_competition"] = args.out_of_competition
        out_obj["svbench_commit"] = _svbench_commit(svb)
        Path(args.out).write_text(json.dumps(out_obj, indent=2, default=int))
        print(f"\n  wrote {args.out}")
    print("\nRESULT:", "PASS (sound)" if ok else "FAIL (soundness violation)")
    return 0 if ok else 1


def pct(x: int, m: int) -> str:
    return f"{(100.0 * x / m):.1f}%" if m else "n/a"


def _svbench_commit(svb: Path) -> str | None:
    """The sv-benchmarks pin. `.gitmodules` sets `ignore = dirty`, so a local edit to
    a `.set` would silently change every score and never show in `git status` — record
    the revision next to any published number."""
    try:
        r = subprocess.run(["git", "-C", str(svb), "rev-parse", "HEAD"],
                           capture_output=True, text=True, timeout=15)
        return r.stdout.strip() or None
    except (OSError, subprocess.SubprocessError):
        return None


if __name__ == "__main__":
    sys.exit(main())
