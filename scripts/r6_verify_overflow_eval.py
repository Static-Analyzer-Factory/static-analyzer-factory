#!/usr/bin/env python3
"""Slice 1e — blind `saf verify` no-overflow reservoir eval (plan 199, R6).

Clone of scripts/r5_verify_memsafety_eval.py for the no-overflow property. Runs the
REAL `saf verify` binary (overflow_strategy + UBSan confirmer) over a stratified sample
of no-overflow sv-benchmarks tasks and classifies the verdict vs the .yml
expected_verdict. no-overflow has NO sub-property — the verdict is just
false(no-overflow). The −16 audit (run at N>=100, per the plan-198 discipline — R5's
−16s only surfaced at eval scale): FP (false-alarm on a safe program) MUST be 0, and
SAF must NEVER print `true`. Also reports recall and how many false verdicts wrote a
witness file.

Run (release binary; container, workspace root):
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev python3 scripts/r6_verify_overflow_eval.py
Env: N (per class x data-model, default 25), TASK_TIMEOUT (default 60),
     DM_STRATIFY=0 for size-representative sampling (reflects ~68% Juliet).
"""
import os
import re
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
N = int(os.environ.get("N", "25"))
TASK_TIMEOUT = int(os.environ.get("TASK_TIMEOUT", "60"))
# The overflow confirmer abstains on reachable thread spawns (plan 199 D5), so exclude
# the dedicated concurrency dirs from the primary eval (they resolve to unknown anyway).
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/ldv-linux-3.14-races/",
             "/libvsync/", "/locks/", "/ddv-machzwd/")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
VERDICT_RE = re.compile(r"^(true|false\(no-overflow\)|unknown)$")


def find_prp():
    for c in (SVB / "properties" / "no-overflow.prp",
              Path("tests/programs/c/svcomp/no-overflow.prp")):
        if c.exists():
            return str(c)
    raise SystemExit("no no-overflow.prp found")


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "no-overflow" not in text:
            continue
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "no-overflow" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                break
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists() or any(d in str(src) for d in CONC_DIRS):
            continue
        md = DM_RE.search(text)
        tasks.append({"src": str(src), "exp": exp,
                      "dm": md.group(1) if md else "LP64", "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def verify(task, dm, witness):
    cmd = [SAF, "verify", "--property", find_prp(), "--data-model", dm,
           "--timeout", str(TASK_TIMEOUT), "--witness", witness, task]
    env = dict(os.environ, SAF_VERIFY_REPLAY_TIMEOUT="5")
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, errors="replace",
                           timeout=TASK_TIMEOUT + 30, env=env)
    except subprocess.TimeoutExpired:
        return "timeout"
    out = r.stdout.strip().splitlines()
    return out[-1] if out else "(no-output)"


def main():
    tasks = collect()
    safe = [t for t in tasks if t["exp"]]
    buggy = [t for t in tasks if not t["exp"]]
    if os.environ.get("DM_STRATIFY", "1") == "0":
        sample = stride(safe, N) + stride(buggy, N)
    else:
        sample = []
        for pool in (safe, buggy):
            for dm in ("ILP32", "LP64"):
                sample += stride([t for t in pool if t["dm"] == dm], N)
    print(f"no-overflow tasks (excl conc): {len(tasks)} (safe={len(safe)} "
          f"buggy={len(buggy)}); sampling {len(sample)} via `saf verify`\n")

    conf = Counter()
    fps, trues = [], []
    wrote_witness = 0
    for t in sample:
        name = Path(t["src"]).name
        with tempfile.TemporaryDirectory() as d:
            w = os.path.join(d, "w.yml")
            v = verify(t["src"], t["dm"], w)
            has_w = os.path.exists(w)
        m = VERDICT_RE.match(v)
        if v == "true":
            conf["TRUE_BUG"] += 1; trues.append(f"{name}[{t['dm']}]"); tag = "TRUE!!"
        elif m and m.group(1).startswith("false"):
            if has_w:
                wrote_witness += 1
            if t["exp"]:
                conf["FP"] += 1; fps.append(f"{name}[{t['dm']}]"); tag = "FP!!"
            else:
                conf["TP"] += 1; tag = "TP false(no-overflow)"
        elif v in ("timeout", "(no-output)"):
            conf[v] += 1; tag = v
        else:  # unknown
            if t["exp"]:
                conf["TN"] += 1; tag = "TN"
            else:
                conf["FN"] += 1; tag = "FN(miss)"
        print(f"  exp={'safe ' if t['exp'] else 'buggy'} {t['dm']:<5} {tag:<24} {name}")

    tp, fn, fp, tn = conf["TP"], conf["FN"], conf["FP"], conf["TN"]
    print("\n=== R6 blind `saf verify` no-overflow (real pipeline) ===")
    print(f"  TP={tp}  FN={fn}  FP={fp}  TN={tn}  TRUE_BUG={conf['TRUE_BUG']}  "
          f"timeout={conf['timeout']}  no_output={conf['(no-output)']}")
    print(f"  *** SOUNDNESS: false-alarm FP={fp} (MUST be 0)   TRUE emitted={conf['TRUE_BUG']} (MUST be 0) ***")
    if tp + fn:
        print(f"  recall = {tp}/{tp+fn} = {tp/(tp+fn):.0%}")
    print(f"  false verdicts that wrote a witness = {wrote_witness}/{tp+fp}")
    if fps:
        print("  !!! FALSE ALARMS:", ", ".join(fps))
    if trues:
        print("  !!! TRUE EMITTED (critical soundness bug):", ", ".join(trues))


if __name__ == "__main__":
    main()
