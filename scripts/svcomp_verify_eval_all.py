#!/usr/bin/env python3
"""All-property blind sweep for `saf verify` (audit session, plan 191/192).

Single pass over sv-benchmarks: for EACH SV-COMP verification property, sample
tasks (stride across categories), run `saf verify` BLIND, and build a per-property
confusion matrix. Empirically establishes (a) which properties `verify` actually
scores vs returns `unknown` for, and (b) unreach-call soundness (0 false alarms,
0 TRUE) + real recall by benchmark family.

Run inside the dev container from the workspace root:
    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \
        'cargo build --release -p saf-cli && python3 scripts/svcomp_verify_eval_all.py'
"""
import os
import re
import subprocess
import sys
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
SAF = os.environ.get("SAF_BIN", "target/release/saf")
TIMEOUT = os.environ.get("EVAL_VERIFY_TIMEOUT", "20")   # --timeout per task
KILL = int(os.environ.get("EVAL_KILL_AFTER", "40"))     # hard subprocess kill (s)

# property -> (.prp basename, sample size)
PROPS = {
    "unreach-call":     ("unreach-call.prp",     int(os.environ.get("N_UNREACH", "40"))),
    "valid-memsafety":  ("valid-memsafety.prp",  int(os.environ.get("N_MEM", "12"))),
    "valid-memcleanup": ("valid-memcleanup.prp", int(os.environ.get("N_CLEAN", "8"))),
    "no-overflow":      ("no-overflow.prp",      int(os.environ.get("N_OVF", "12"))),
    "termination":      ("termination.prp",      int(os.environ.get("N_TERM", "10"))),
    "no-data-race":     ("no-data-race.prp",     int(os.environ.get("N_RACE", "10"))),
}

INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
PROP_RE = re.compile(r"([\w-]+)\.prp")
EXP_RE = re.compile(r"expected_verdict:\s*(true|false)")


def parse_props(lines):
    """Yield (prp_basename, expected_bool|None) for each property block."""
    out = []
    for i, line in enumerate(lines):
        if "property_file:" in line:
            m = PROP_RE.search(line)
            if not m:
                continue
            name = m.group(1)
            exp = None
            for j in range(i, min(i + 4, len(lines))):
                me = EXP_RE.search(lines[j])
                if me:
                    exp = me.group(1) == "true"
                    break
            out.append((name, exp))
    return out


def collect():
    """Single pass: bucket every task by the properties it declares."""
    buckets = {p: [] for p in PROPS}
    names = {v[0].replace(".prp", ""): p for p, v in PROPS.items()}  # basename->prop key
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        lines = text.splitlines()
        props = parse_props(lines)
        if not props:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
            continue
        md = DM_RE.search(text)
        dm = md.group(1) if md else "LP64"
        try:
            cat = src.relative_to(SVB.resolve()).parts[0]
        except ValueError:
            cat = "?"
        for name, exp in props:
            key = names.get(name)
            if key is None or exp is None:
                continue
            prp = (SVB / "properties" / PROPS[key][0]).resolve()
            if not prp.exists():
                continue
            buckets[key].append({
                "src": str(src), "prp": str(prp), "expected": exp,
                "dm": dm, "cat": cat, "size": src.stat().st_size,
            })
    return buckets


def stride(lst, n):
    lst = sorted(lst, key=lambda t: t["src"])
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def run_one(t):
    dm = "ILP32" if t["dm"].upper() == "ILP32" else "LP64"
    cmd = [SAF, "verify", "--property", t["prp"], "--data-model", dm,
           "--timeout", TIMEOUT, t["src"]]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=KILL)
        out = r.stdout.strip()
    except subprocess.TimeoutExpired:
        return "unknown", "(killed)"
    v = "false" if out.startswith("false(") else ("true" if out == "true" else "unknown")
    return v, out


def main():
    buckets = collect()
    print(f"collected tasks per property: "
          + ", ".join(f"{k}={len(v)}" for k, v in buckets.items()) + "\n")
    grand = {}
    for target, (prp, n) in PROPS.items():
        tasks = buckets[target]
        half = max(1, n // 2)
        trues = stride([t for t in tasks if t["expected"]], half)
        falses = stride([t for t in tasks if not t["expected"]], n - half)
        sample = trues + falses
        conf, fa, caught = {}, [], []
        for t in sample:
            v, _out = run_one(t)
            conf[(t["expected"], v)] = conf.get((t["expected"], v), 0) + 1
            if t["expected"] and v == "false":
                fa.append(t["src"])
            if (not t["expected"]) and v == "false":
                caught.append(f"{t['cat']}/{Path(t['src']).name}")
        et = sum(conf.get((e, "true"), 0) for e in (True, False))
        tf = sum(conf.get((False, v), 0) for v in ("false", "true", "unknown"))
        cf = conf.get((False, "false"), 0)
        print(f"### {target}: found {len(tasks)} tasks; sampled {len(sample)} "
              f"({len(trues)}T/{len(falses)}F)")
        for exp in (True, False):
            cells = ", ".join(f"{v}={conf.get((exp, v), 0)}" for v in ("false", "true", "unknown"))
            print(f"    expected={'TRUE ' if exp else 'FALSE'}: {cells}")
        print(f"    false_alarms={len(fa)} [MUST 0]; TRUE_emitted={et} [MUST 0]; "
              f"violations_caught={cf}/{tf}")
        if caught:
            print("    caught:", "; ".join(caught))
        if fa:
            print("    !!! FALSE ALARMS:", fa)
        grand[target] = {"found": len(tasks), "sampled": len(sample), "fa": len(fa),
                         "true": et, "caught": cf, "total_false": tf}
        print()
    print("=== GRAND SUMMARY ===")
    for k, v in grand.items():
        print(f"  {k:<17} recall {v['caught']}/{v['total_false']:<3} "
              f"false_alarms {v['fa']}  TRUE {v['true']}   (of {v['found']} tasks)")
    bad = any(v["fa"] > 0 or v["true"] > 0 for v in grand.values())
    print("\nSOUNDNESS:",
          "VIOLATED" if bad else "OK (0 false alarms, 0 TRUE across all sampled properties)")


if __name__ == "__main__":
    main()
