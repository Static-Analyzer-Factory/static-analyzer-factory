#!/usr/bin/env python3
"""Slice 1f — blind `saf verify` valid-memsafety reservoir eval (plan 197, R5).

Runs the REAL `saf verify` binary (with the memsafety_strategy + ASan confirmer)
over a stratified sample of valid-memsafety sv-benchmarks tasks and classifies the
verdict vs the .yml expected_verdict + subproperty. Unlike the Slice-0c standalone
ASan probe, this exercises the whole pipeline: compile→ingest→memsafety_strategy→
ASan replay→witness. The −16 audit: FP (false-alarm on a safe program) MUST be 0,
and SAF must NEVER print `true`. Also reports recall, sub-property agreement, and
how many false verdicts wrote a witness file.

Run (release binary; container, workspace root):
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev python3 scripts/r5_verify_memsafety_eval.py
"""
import os
import re
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
N = int(os.environ.get("N", "20"))            # per (class x data-model)
TASK_TIMEOUT = int(os.environ.get("TASK_TIMEOUT", "60"))
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/libvsync/",
             "/locks/", "/ddv-machzwd/")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
VERDICT_RE = re.compile(r"^(true|false\((valid-[a-z]+)\)|unknown)$")


def find_prp():
    for c in (SVB / "properties" / "valid-memsafety.prp",
              Path("tests/programs/c/svcomp/valid-memsafety.prp")):
        if c.exists():
            return str(c)
    raise SystemExit("no valid-memsafety.prp found")


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        exp, sub = None, None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                    ms = re.search(r"subproperty:\s*(valid-[a-z]+)", lines[j])
                    if ms:
                        sub = ms.group(1)
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
        tasks.append({"src": str(src), "exp": exp, "sub": sub,
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
    sample = []
    for pool in (safe, buggy):
        for dm in ("ILP32", "LP64"):
            sample += stride([t for t in pool if t["dm"] == dm], N)
    print(f"sequential valid-memsafety tasks: {len(tasks)} (safe={len(safe)} "
          f"buggy={len(buggy)}); sampling {len(sample)} via `saf verify`\n")

    conf = Counter()
    subdis, fps, trues = [], [], []
    wrote_witness = 0
    for t in sample:
        name = Path(t["src"]).name
        with tempfile.TemporaryDirectory() as d:
            w = os.path.join(d, "w.yml")
            v = verify(t["src"], t["dm"], w)
            has_w = os.path.exists(w)
        m = VERDICT_RE.match(v)
        if v == "true" or (m and m.group(1) == "true"):
            conf["TRUE_BUG"] += 1; trues.append(f"{name}[{t['dm']}]"); tag = "TRUE!!"
        elif m and m.group(1).startswith("false"):
            sub = m.group(2)
            if has_w:
                wrote_witness += 1
            if t["exp"]:
                conf["FP"] += 1; fps.append(f"{name}[{t['dm']}]:{sub}"); tag = f"FP!! {sub}"
            else:
                conf["TP"] += 1
                if t["sub"] == sub:
                    conf["sub_agree"] += 1; tag = f"TP {sub}==exp"
                elif t["sub"]:
                    conf["sub_disagree"] += 1
                    subdis.append(f"{name}: saf->{sub} exp={t['sub']}")
                    tag = f"TP {sub}!=EXP({t['sub']})"
                else:
                    tag = f"TP {sub}"
        elif v in ("timeout", "(no-output)"):
            conf[v] += 1; tag = v
        else:  # unknown
            if t["exp"]:
                conf["TN"] += 1; tag = "TN"
            else:
                conf["FN"] += 1; tag = "FN(miss)"
        print(f"  exp={'safe ' if t['exp'] else 'buggy'} {t['dm']:<5} {tag:<26} {name}")

    tp, fn, fp, tn = conf["TP"], conf["FN"], conf["FP"], conf["TN"]
    print("\n=== R5 blind `saf verify` valid-memsafety (real pipeline) ===")
    print(f"  TP={tp}  FN={fn}  FP={fp}  TN={tn}  TRUE_BUG={conf['TRUE_BUG']}  "
          f"timeout={conf['timeout']}  no_output={conf['(no-output)']}")
    print(f"  *** SOUNDNESS: false-alarm FP={fp} (MUST be 0)   TRUE emitted={conf['TRUE_BUG']} (MUST be 0) ***")
    if tp + fn:
        print(f"  recall = {tp}/{tp+fn} = {tp/(tp+fn):.0%}")
    sa, sd = conf["sub_agree"], conf["sub_disagree"]
    if sa + sd:
        print(f"  sub-property match = {sa}/{sa+sd} agree ({sd} wrong = -16 risk)")
    print(f"  false verdicts that wrote a witness = {wrote_witness}/{tp+fp}")
    if fps:
        print("  !!! FALSE ALARMS:", ", ".join(fps))
    if trues:
        print("  !!! TRUE EMITTED (critical soundness bug):", ", ".join(trues))
    if subdis:
        print("  sub-property disagreements:", "; ".join(subdis))


if __name__ == "__main__":
    main()
