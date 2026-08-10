#!/usr/bin/env python3
"""Blind acceptance harness for `saf verify` (plan 192, slice 1).

Samples unreach-call tasks from sv-benchmarks, runs `saf verify` BLIND (verdict
only), then audits each task's `expected_verdict` AFTER the run. The critical
metric is ZERO false alarms on the expected==true subset (and zero TRUE emitted).

Run inside the dev container from the workspace root, e.g.:
    docker compose run --rm -e SKIP_MATURIN_BUILD=1 dev sh -c \
        'cargo build --release -p saf-cli && python3 scripts/svcomp_verify_eval.py'
"""
import os
import re
import subprocess
import sys
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
VERIFY_TIMEOUT = os.environ.get("EVAL_VERIFY_TIMEOUT", "25")  # --timeout per task
KILL_AFTER = int(os.environ.get("EVAL_KILL_AFTER", "45"))     # hard subprocess kill (s)
N_PER_CLASS = int(os.environ.get("EVAL_N_PER_CLASS", "20"))

DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
EXP_RE = re.compile(r"expected_verdict:\s*(true|false)")


def parse_task(yml: Path):
    """Return a task dict if `yml` is an unreach-call task, else None."""
    try:
        text = yml.read_text(errors="replace")
    except OSError:
        return None
    lines = text.splitlines()

    expected = None
    for i, line in enumerate(lines):
        if "property_file" in line and "unreach-call" in line:
            for j in range(i, min(i + 3, len(lines))):
                m = EXP_RE.search(lines[j])
                if m:
                    expected = m.group(1) == "true"
                    break
            break
    if expected is None:
        return None

    mi = INPUT_RE.search(text)
    if not mi:
        return None
    src = (yml.parent / mi.group(1).strip()).resolve()
    if not src.exists():
        return None

    prop = None
    for cand in yml.parent.rglob("unreach-call.prp"):
        prop = cand.resolve()
        break
    if prop is None:
        prop = (SVB / "properties" / "unreach-call.prp").resolve()
    if not prop.exists():
        return None

    md = DM_RE.search(text)
    data_model = md.group(1) if md else "LP64"
    return {
        "src": str(src),
        "prop": str(prop),
        "expected": expected,
        "data_model": data_model,
        "size": src.stat().st_size,
    }


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        t = parse_task(yml)
        if t:
            tasks.append(t)
    return tasks


def run_one(t):
    dm = "ILP32" if t["data_model"].upper() == "ILP32" else "LP64"
    cmd = [SAF, "verify", "--property", t["prop"], "--data-model", dm,
           "--timeout", VERIFY_TIMEOUT, t["src"]]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=KILL_AFTER)
        out = r.stdout.strip()
    except subprocess.TimeoutExpired:
        return "unknown"
    if out.startswith("false("):
        return "false"
    if out == "true":
        return "true"
    return "unknown"


def stride(lst, n):
    """Deterministically pick `n` items spread evenly across `lst` (sorted by
    path), so the sample spans many benchmark categories rather than clustering
    in one regression micro-suite."""
    lst = sorted(lst, key=lambda t: t["src"])
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def main():
    tasks = collect()
    trues = stride([t for t in tasks if t["expected"]], N_PER_CLASS)
    falses = stride([t for t in tasks if not t["expected"]], N_PER_CLASS)
    sample = trues + falses
    print(f"unreach-call tasks found: {len(tasks)} "
          f"({len(trues)} expected-true, {len(falses)} expected-false)")
    print(f"sampling {len(sample)} (smallest-first, deterministic); "
          f"--timeout {VERIFY_TIMEOUT}s, kill {KILL_AFTER}s\n")

    conf = {}
    false_alarms = []
    for t in sample:
        verdict = run_one(t)
        conf[(t["expected"], verdict)] = conf.get((t["expected"], verdict), 0) + 1
        tag = "true " if t["expected"] else "false"
        flag = ""
        if t["expected"] and verdict == "false":
            false_alarms.append(t["src"])
            flag = "  <-- FALSE ALARM"
        print(f"  expected={tag} verdict={verdict:<8} {Path(t['src']).name}{flag}")

    print("\n=== confusion (expected \\ verdict) ===")
    for exp in (True, False):
        cells = ", ".join(f"{v}={conf.get((exp, v), 0)}" for v in ("false", "true", "unknown"))
        print(f"  expected={'TRUE ' if exp else 'FALSE'}: {cells}")

    emitted_true = sum(conf.get((e, "true"), 0) for e in (True, False))
    caught = conf.get((False, "false"), 0)
    total_false = sum(conf.get((False, v), 0) for v in ("false", "true", "unknown"))
    print(f"\nfalse alarms (expected TRUE -> false):  {len(false_alarms)}   "
          f"[MUST be 0]")
    print(f"TRUE verdicts emitted:                  {emitted_true}   [MUST be 0]")
    print(f"violations detected (expected FALSE -> false): {caught}/{total_false}")
    if false_alarms:
        print("\nFALSE ALARM tasks:")
        for fa in false_alarms:
            print("  ", fa)
    ok = (len(false_alarms) == 0 and emitted_true == 0)
    print("\nRESULT:", "PASS (sound: no false alarms, no TRUE)" if ok else "FAIL")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
