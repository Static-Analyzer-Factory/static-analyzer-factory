#!/usr/bin/env python3
"""Is SAF's own sentinel sound ENOUGH to be the final TRUE authority?

Today `try_unreach_true` / `try_overflow_true` in crates/saf-cli/src/commands.rs emit
`true` only when an in-process CPAchecker confirms — which makes those results
"computable by CPAchecker alone" and puts SAF on the wrong side of the SV-COMP
meta-verifier line. Removing that gate means SAF's OWN sentinel becomes the final
authority, so its false-positive rate against GROUND TRUTH must be exactly 0.

This measures precisely that: run the sentinel over ground-truth-FALSE tasks (where the
property provably does NOT hold, so any PROVE is a would-be wrong TRUE worth -32 and
uncapped by the dedup rule). Also runs the ground-truth-TRUE side for the reference
prove rate, so the two numbers come from the same sample and the same binary.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 /workspace/probe_soundness.py no-overflow 400 8'
"""
from __future__ import annotations

import collections
import json
import os
import random
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")
SUBCMD = {"unreach-call": "prove-unreachable", "no-overflow": "prove-no-overflow"}


def main() -> int:
    prop = sys.argv[1] if len(sys.argv) > 1 else "no-overflow"
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 400
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 8
    subcmd = SUBCMD[prop]
    random.seed(0)

    # Stratify by cluster on BOTH sides so no single family dominates.
    sides: dict[bool, dict[str, list[dict]]] = {True: collections.defaultdict(list),
                                                False: collections.defaultdict(list)}
    for line in open("lever1-pertask.jsonl"):
        r = json.loads(line)
        if r["property"] == prop:
            sides[bool(r["expected"])][r.get("group", "")].append(r)

    tasks = []
    for exp in (False, True):
        pool = []
        for _, rs in sorted(sides[exp].items()):
            pool += random.sample(rs, min(4, len(rs)))
        pick = random.sample(pool, min(want // 2, len(pool)))
        for r in pick:
            r["_exp"] = exp
        tasks += pick
        print(f"expected={exp}: {len(sides[exp])} clusters -> probing {len(pick)}", flush=True)

    def run(r):
        base = r["rel_yml"][:-4]
        src = next((f"{SVB}/{base}{e}" for e in (".c", ".i")
                   if os.path.exists(f"{SVB}/{base}{e}")), None)
        if not src:
            return None
        try:
            p = subprocess.run([SAF, subcmd, "--data-model", r["data_model"], src],
                               capture_output=True, text=True, timeout=150)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))),
                       "(no output)")
        except subprocess.TimeoutExpired:
            out = "PROBE_TIMEOUT"
        return {"group": r["group"], "rel_yml": r["rel_yml"], "expected": r["_exp"],
                "outcome": out.split(" ")[0]}

    res = [x for x in ThreadPoolExecutor(max_workers=jobs).map(run, tasks) if x]
    out_path = f"probe_soundness_{prop}.jsonl"
    with open(out_path, "w") as fh:
        for r in res:
            fh.write(json.dumps(r) + "\n")

    print(f"\nwrote {out_path} ({len(res)} rows)")
    for exp in (False, True):
        rs = [r for r in res if r["expected"] == exp]
        c = collections.Counter(r["outcome"] for r in rs)
        label = "GROUND-TRUTH FALSE (any PROVE here is a WRONG TRUE)" if not exp \
            else "GROUND-TRUTH TRUE (PROVE here is the prize)"
        print(f"\n{label}  n={len(rs)}")
        print(f"  PROVE = {c['PROVE']}"
              + ("   <== MUST BE 0" if not exp else
                 f"  ({100*c['PROVE']/max(len(rs),1):.1f}%)"))
        for k, v in c.most_common():
            if k != "PROVE":
                print(f"  {v:>5}  {k}")
        if not exp and c["PROVE"]:
            print("  OFFENDING TASKS (each one is -32 and bypasses the dedup cap):")
            for r in rs:
                if r["outcome"] == "PROVE":
                    print(f"    {r['rel_yml']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
