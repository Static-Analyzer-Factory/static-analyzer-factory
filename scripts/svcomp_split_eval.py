#!/usr/bin/env python3
"""Manifest-driven, property-generic blind `saf verify` eval with CONFIRMED scoring
(plan 203).

Runs the REAL `saf verify` binary blind over the tasks in a split manifest
(`train.jsonl` / `holdout.jsonl` from scripts/svcomp_split.py), then scores the
run TWO ways so the gap between them is explicit:

  * RAW score      — the naive verdict-vs-expected score (SvCompOutcome in
                     crates/saf-bench/src/svcomp/scoring.rs): +2 correct-TRUE,
                     +1 correct-FALSE, -16 false-alarm, -32 wrong-TRUE, 0 unknown.
  * CONFIRMED score — the OFFICIAL score: a correct FALSE earns +1 only if its
                     violation witness is CONFIRMED by a validator (else 0 —
                     "correct-unconfirmed"); a correct TRUE earns +2 on the verdict
                     alone for the witness-not-required properties (termination /
                     valid-memsafety / valid-memcleanup / no-data-race) and only
                     with a confirmed correctness witness for unreach-call /
                     no-overflow. Negative outcomes (-16 / -32) apply regardless of
                     confirmation — a wrong verdict is always penalized.

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

SAF = os.environ.get("SAF_BIN", "target/release/saf")
VALIDATE_SH = os.environ.get("SAF_VALIDATE_WITNESS", "scripts/validate_witness.sh")
VERDICT_RE = re.compile(r"^(true|false\(([a-z0-9-]+)\)|unknown)$")

# TRUE is scored on the verdict alone (no correctness witness required) for these
# properties in SV-COMP 2025/2026; unreach-call and no-overflow TRUE need a confirmed
# correctness witness (SAF never emits TRUE for those, so this is defensive).
TRUE_WITNESS_NOT_REQUIRED = {
    "termination", "valid-memsafety", "valid-memcleanup", "no-data-race",
}
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
        time.sleep(0.5)

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


def confirm_witness(task: dict, src: str, prp: str, witness: str | None,
                    timeout: int) -> str:
    """Validate an emitted violation witness. Returns CONFIRMED / NOT_CONFIRMED /
    LINT_FAIL / LINT_ONLY (validator unavailable) / TIMEOUT / NO_WITNESS / ERROR."""
    if not witness or not os.path.exists(witness):
        return "NO_WITNESS"
    cmd = ["bash", VALIDATE_SH, witness, src, prp, task["data_model"]]
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


def confirmed_score(outcome: str, prop: str, witness_status: str | None) -> int:
    """Official-style points: gate positive credit on confirmation; penalties stand."""
    if outcome == "FalseCorrect":
        return 1 if witness_status == "CONFIRMED" else 0
    if outcome == "TrueCorrect":
        if prop in TRUE_WITNESS_NOT_REQUIRED:
            return 2
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
             confirm_timeout: int, max_rss_mb: int = 0) -> dict:
    prp = resolve_prp(svb, task["property"])
    # Resolve a PORTABLE source path against --svb (relative → works both on the host
    # and inside the Docker container where the repo is at /workspace). Falls back to
    # the stored absolute path for older manifests without rel_src.
    src = str(svb / task["rel_src"]) if task.get("rel_src") else task["src"]
    with tempfile.TemporaryDirectory() as d:
        w = os.path.join(d, "w.yml") if confirm else None
        line, dur, err_tail = run_verify(task, src, prp, timeout, w, max_rss_mb)
        kind, sub = classify(line)
        wstatus = None
        if confirm and kind == "false":
            wstatus = confirm_witness(task, src, prp, w, confirm_timeout)
    outcome = raw_outcome(kind, task["expected"])
    # Keep SAF's stderr tail only where it's diagnostically useful (an abstain, a
    # miss, a crash) — not on clean scoring verdicts — to bound the dump size.
    keep_err = kind in ("unknown", "timeout", "error") or outcome == "FalseIncorrect"
    return {
        "property": task["property"], "expected": task["expected"],
        "kind": kind, "sub": sub, "rel_yml": task["rel_yml"],
        "data_model": task["data_model"], "group": task.get("group", ""),
        "outcome": outcome, "raw": SCORE[outcome],
        "confirmed": confirmed_score(outcome, task["property"], wstatus),
        "witness": wstatus, "duration_s": round(dur, 2),
        "stderr_tail": err_tail if keep_err else "",
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
                    help="validate each emitted FALSE witness (needed for CONFIRMED score)")
    ap.add_argument("--confirm-timeout", type=int, default=150)
    ap.add_argument("--jobs", type=int, default=1, help="parallel verify workers")
    ap.add_argument("-o", "--out", default=None, help="write JSON summary to file")
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
    print(f"== {label}: {len(rows)} tasks; confirm={args.confirm_witness}; "
          f"jobs={args.jobs}; timeout={args.timeout}s ==")

    def work(t):
        return eval_one(t, svb, args.timeout, args.confirm_witness,
                        args.confirm_timeout, args.max_rss_mb)

    if args.jobs > 1:
        with ThreadPoolExecutor(max_workers=args.jobs) as ex:
            results = list(ex.map(work, rows))
    else:
        results = [work(t) for t in rows]

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
        Path(args.out).write_text(json.dumps({
            "manifest": args.manifest, "n": tot["n"],
            "raw_score": tot["raw"], "confirmed_score": tot["confirmed"],
            "max_score": tot["max"], "false_alarms": tot["FP"],
            "wrong_true": tot["wrong_true"], "lint_only": lint_only,
            "per_property": per,
        }, indent=2, default=int))
        print(f"\n  wrote {args.out}")
    print("\nRESULT:", "PASS (sound)" if ok else "FAIL (soundness violation)")
    return 0 if ok else 1


def pct(x: int, m: int) -> str:
    return f"{(100.0 * x / m):.1f}%" if m else "n/a"


if __name__ == "__main__":
    sys.exit(main())
