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
import tempfile
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
VERIFY_TIMEOUT = os.environ.get("EVAL_VERIFY_TIMEOUT", "25")  # --timeout per task
KILL_AFTER = int(os.environ.get("EVAL_KILL_AFTER", "45"))     # hard subprocess kill (s)
N_PER_CLASS = int(os.environ.get("EVAL_N_PER_CLASS", "20"))

# Witness-confirmation mode (plan 194 R2/Slice G): when EVAL_CONFIRM_WITNESS=1,
# `saf verify` writes a YAML witness per task and each emitted FALSE is run
# through scripts/validate_witness.sh (witnesslint + CPAchecker) to measure the
# confirmed-witness % — the C.FalseOverall score predictor.
CONFIRM = os.environ.get("EVAL_CONFIRM_WITNESS", "0") == "1"
CONFIRM_TIMEOUT = int(os.environ.get("EVAL_CONFIRM_TIMEOUT", "150"))
VALIDATE_SH = os.environ.get("SAF_VALIDATE_WITNESS", "scripts/validate_witness.sh")
WITNESS_DIR = tempfile.mkdtemp(prefix="saf_witness_eval_") if CONFIRM else None

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


def run_one(t, idx):
    """Run `saf verify` on task `t`; return (verdict, witness_path_or_None)."""
    dm = "ILP32" if t["data_model"].upper() == "ILP32" else "LP64"
    cmd = [SAF, "verify", "--property", t["prop"], "--data-model", dm,
           "--timeout", VERIFY_TIMEOUT]
    witness = None
    if CONFIRM:
        assert WITNESS_DIR is not None  # set whenever CONFIRM is true
        witness = os.path.join(WITNESS_DIR, f"w_{idx}.yml")
        cmd += ["--witness", witness]
    cmd.append(t["src"])
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=KILL_AFTER)
        out = r.stdout.strip()
    except subprocess.TimeoutExpired:
        return "unknown", None
    if out.startswith("false("):
        return "false", (witness if witness and os.path.exists(witness) else None)
    if out == "true":
        return "true", None
    return "unknown", None


def confirm_witness(t, witness):
    """Validate an emitted witness via validate_witness.sh. Returns one of
    CONFIRMED / NOT_CONFIRMED / LINT_FAIL / LINT_ONLY (CPAchecker unavailable) /
    TIMEOUT / NO_WITNESS / ERROR."""
    if not witness:
        return "NO_WITNESS"
    cmd = ["bash", VALIDATE_SH, witness, t["src"], t["prop"], t["data_model"]]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=CONFIRM_TIMEOUT)
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
    confirmations = {}
    for idx, t in enumerate(sample):
        verdict, witness = run_one(t, idx)
        conf[(t["expected"], verdict)] = conf.get((t["expected"], verdict), 0) + 1
        tag = "true " if t["expected"] else "false"
        flag = ""
        if t["expected"] and verdict == "false":
            false_alarms.append(t["src"])
            flag = "  <-- FALSE ALARM"
        cstat = ""
        if CONFIRM and verdict == "false":
            status = confirm_witness(t, witness)
            confirmations[status] = confirmations.get(status, 0) + 1
            cstat = f"  witness={status}"
        print(f"  expected={tag} verdict={verdict:<8} {Path(t['src']).name}{flag}{cstat}")

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

    if CONFIRM:
        print("\n=== witness confirmation (witnesslint + CPAchecker) ===")
        for k in sorted(confirmations):
            print(f"  {k:<14} {confirmations[k]}")
        confirmed = confirmations.get("CONFIRMED", 0)
        total_w = sum(confirmations.values())
        pct = (100.0 * confirmed / total_w) if total_w else 0.0
        print(f"confirmed-witness: {confirmed}/{total_w} ({pct:.0f}%) of emitted FALSE")

    ok = (len(false_alarms) == 0 and emitted_true == 0)
    print("\nRESULT:", "PASS (sound: no false alarms, no TRUE)" if ok else "FAIL")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
