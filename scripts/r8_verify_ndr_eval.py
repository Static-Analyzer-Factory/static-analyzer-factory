#!/usr/bin/env python3
"""R8 Slice-0 de-risk — blind `saf verify --property no-data-race.prp` eval.

Runs the REAL `saf verify` binary (with the THROWAWAY `no_data_race_strategy`
arm wired by r8_derisk_patch.py, which reuses the plan-198 `reachable_spawns_threads`
gate) over the SV-COMP no-data-race reservoir and classifies each verdict vs the
`.yml` expected_verdict.

R8 is TRUE-only (never emits `false`). The DOMINANT risk is a wrong `true` on a
RACY task (expected-FALSE) = -32. So:
  * WRONG_TRUE (a `true` on an expected-FALSE / racy task) MUST be 0 (the -32 audit).
  * FALSE_EMITTED (any `false(...)`) MUST be 0 (R8 has no `false` path).
Recall = correct `true` / expected-TRUE (the provably-sequential subset; the go/no-go).

Run (release binary; container, workspace root):
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev \
        sh -c "cargo build --release -p saf-cli && python3 scripts/r8_verify_ndr_eval.py"
Env: N_FALSE (racy sample, default 1000 = all 236), N_TRUE (race-free sample,
     default 400), TASK_TIMEOUT (default 60), DETERMINISM (default 15).
"""
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
N_FALSE = int(os.environ.get("N_FALSE", "1000"))
N_TRUE = int(os.environ.get("N_TRUE", "400"))
TASK_TIMEOUT = int(os.environ.get("TASK_TIMEOUT", "60"))
DETERMINISM = int(os.environ.get("DETERMINISM", "15"))
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
VERDICT_RE = re.compile(r"^(true|unknown|false\([^)]*\)|false)$")
PRP = None


def find_prp():
    for c in (SVB / "properties" / "no-data-race.prp",
              Path("tests/programs/c/svcomp/no-data-race.prp")):
        if c.exists():
            return str(c)
    raise SystemExit("no no-data-race.prp found")


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "no-data-race.prp" not in text:
            continue
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "no-data-race.prp" in ln and "property_file" in ln:
                for j in range(i + 1, len(lines)):
                    if "property_file" in lines[j]:
                        break
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                        break
                break
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
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


def verify(task, dm):
    global PRP
    if PRP is None:
        PRP = find_prp()
    cmd = [SAF, "verify", "--property", PRP, "--data-model", dm,
           "--timeout", str(TASK_TIMEOUT), task]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, errors="replace",
                           timeout=TASK_TIMEOUT + 30)
    except subprocess.TimeoutExpired:
        return "timeout"
    out = r.stdout.strip().splitlines()
    return out[-1] if out else "(no-output)"


def main():
    tasks = collect()
    racefree = [t for t in tasks if t["exp"]]     # expected-true (no race)
    racy = [t for t in tasks if not t["exp"]]     # expected-false (has race)
    sample_true = stride(racefree, N_TRUE)
    sample_false = stride(racy, N_FALSE)
    print(f"no-data-race tasks: {len(tasks)} (race-free={len(racefree)} "
          f"racy={len(racy)}); sampling "
          f"{len(sample_true)} true + {len(sample_false)} false via `saf verify`\n")

    conf = Counter()
    wrong_trues, false_emitted, correct_trues = [], [], []
    cache = {}
    for t in sample_true + sample_false:
        name = Path(t["src"]).name
        v = verify(t["src"], t["dm"])
        cache[(t["src"], t["dm"])] = v
        m = VERDICT_RE.match(v)
        if v == "true":
            if t["exp"]:
                conf["TP"] += 1
                correct_trues.append(f"{name}[{t['dm']}]")
                tag = "TP true (+2)"
            else:
                conf["WRONG_TRUE"] += 1
                wrong_trues.append(f"{name}[{t['dm']}]")
                tag = "WRONG_TRUE!! (-32)"
        elif m and m.group(1).startswith("false"):
            conf["FALSE_EMITTED"] += 1
            false_emitted.append(f"{name}[{t['dm']}]:{v}")
            tag = "FALSE!! (R8 bug)"
        elif v in ("timeout", "(no-output)"):
            conf[v] += 1
            tag = v
        else:  # unknown
            if t["exp"]:
                conf["FN"] += 1
                tag = "FN(abstain)"
            else:
                conf["TN"] += 1
                tag = "TN(abstain)"
        print(f"  exp={'norace' if t['exp'] else 'racy  '} {t['dm']:<5} {tag:<20} {name}")

    det_pool = stride(sample_true + sample_false, DETERMINISM)
    det_mismatch = []
    for t in det_pool:
        v2 = verify(t["src"], t["dm"])
        v1 = cache.get((t["src"], t["dm"]))
        if v2 != v1:
            det_mismatch.append(f"{Path(t['src']).name}[{t['dm']}]: {v1!r}!={v2!r}")

    tp, fn, tn = conf["TP"], conf["FN"], conf["TN"]
    print("\n=== R8 blind `saf verify` no-data-race (real pipeline, THROWAWAY arm) ===")
    print(f"  race-free: TP(true)={tp}  FN(abstain)={fn}")
    print(f"  racy:      TN(abstain)={tn}  WRONG_TRUE={conf['WRONG_TRUE']}")
    print(f"  FALSE_EMITTED={conf['FALSE_EMITTED']}  timeout={conf['timeout']}  "
          f"no_output={conf['(no-output)']}")
    print(f"  *** SOUNDNESS (the -32 audit): WRONG_TRUE={conf['WRONG_TRUE']} (MUST be 0)   "
          f"FALSE_EMITTED={conf['FALSE_EMITTED']} (MUST be 0) ***")
    if tp + fn:
        print(f"  recall = {tp}/{tp + fn} = {tp / (tp + fn):.1%} (provably-sequential subset)")
    print(f"  determinism: {len(det_pool) - len(det_mismatch)}/{len(det_pool)} re-runs identical")
    if correct_trues:
        print("  TP tasks:", ", ".join(correct_trues[:40]))
    if wrong_trues:
        print("  !!! WRONG_TRUE (-32 CRITICAL):", ", ".join(wrong_trues))
    if false_emitted:
        print("  !!! FALSE EMITTED (R8 bug):", ", ".join(false_emitted))
    if det_mismatch:
        print("  !!! NONDETERMINISM:", ", ".join(det_mismatch))


if __name__ == "__main__":
    main()
