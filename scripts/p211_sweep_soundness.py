#!/usr/bin/env python3
"""DEFINITIVE soundness sweep: run a prove-* sentinel over EVERY ground-truth-FALSE task.

Why this is the gating measurement for plan 211: today `try_unreach_true` /
`try_overflow_true` emit `true` only when an in-process CPAchecker confirms, which makes
those results computable by CPAchecker alone (the SV-COMP meta-verifier line). Removing
that gate makes SAF's OWN sentinel the final TRUE authority — so its PROVE rate on
ground-truth-FALSE tasks must be exactly 0, not "0 in a sample".

A 200-task stratified sample already found 2 wrong PROVEs for unreach-call
(bitvector-regression/implicitunsignedconversion-1, loop-simple/deep-nested), both traced
to crates/saf-analysis/src/absint/transfer.rs:1510-1520 applying SIGNED interval
refinement to UNSIGNED comparisons (the evaluation side guards on `lo >= 0`; the
refinement side does not). This sweep finds the whole class.

Resumable and incremental: re-running skips tasks already in the output file.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 /workspace/sweep_soundness.py unreach-call 6'
"""
from __future__ import annotations

import collections
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")
SUBCMD = {"unreach-call": "prove-unreachable", "no-overflow": "prove-no-overflow"}


def resolve_source(rel_yml: str) -> str | None:
    """The source the TASK DEFINITION names. 13,419 of 36,336 TRUE tasks ship both a
    `.c` and a `.i` and the .yml names the `.i` — guessing analyses a different program.
    """
    y = os.path.join(SVB, rel_yml)
    try:
        txt = open(y, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    if not m:
        return None
    src = os.path.join(os.path.dirname(y), m.group(1).strip())
    return src if os.path.exists(src) else None


def main() -> int:
    prop = sys.argv[1] if len(sys.argv) > 1 else "unreach-call"
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    subcmd = SUBCMD[prop]
    out_path = f"sweep_soundness_{prop}.jsonl"

    done = set()
    if os.path.exists(out_path):
        for line in open(out_path):
            try:
                done.add(json.loads(line)["rel_yml"])
            except Exception:  # noqa: BLE001
                pass

    tasks = []
    for line in open("lever1-pertask.jsonl"):
        r = json.loads(line)
        if r["property"] == prop and not r["expected"] and r["rel_yml"] not in done:
            tasks.append(r)
    print(f"{prop}: {len(tasks)} ground-truth-FALSE tasks to probe "
          f"({len(done)} already done)", flush=True)

    fh = open(out_path, "a", buffering=1)
    n = 0
    wrong = []

    def run(r):
        src = resolve_source(r["rel_yml"])
        if not src:
            return {"rel_yml": r["rel_yml"], "group": r["group"], "outcome": "(no source)"}
        try:
            p = subprocess.run([SAF, subcmd, "--data-model", r["data_model"], src],
                               capture_output=True, text=True, timeout=180)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))),
                       "(no output)")
        except subprocess.TimeoutExpired:
            out = "PROBE_TIMEOUT"
        return {"rel_yml": r["rel_yml"], "group": r["group"], "outcome": out.split(" ")[0]}

    for res in ThreadPoolExecutor(max_workers=jobs).map(run, tasks):
        fh.write(json.dumps(res) + "\n")
        n += 1
        if res["outcome"] == "PROVE":
            wrong.append(res["rel_yml"])
            print(f"  !! WRONG PROVE ({len(wrong)}): {res['rel_yml']}", flush=True)
        if n % 250 == 0:
            print(f"  {n}/{len(tasks)}  wrong_true_candidates={len(wrong)}", flush=True)
    fh.close()

    allrows = [json.loads(l) for l in open(out_path)]
    c = collections.Counter(r["outcome"] for r in allrows)
    print(f"\n=== {prop}: {len(allrows)} ground-truth-FALSE tasks ===")
    print(f"PROVE (would-be WRONG TRUE, -32 each, uncapped by dedup) = {c['PROVE']}")
    for k, v in c.most_common():
        if k != "PROVE":
            print(f"  {v:>6}  {k}")
    if c["PROVE"]:
        byg = collections.Counter(r["group"] for r in allrows if r["outcome"] == "PROVE")
        print("\nwrong PROVEs by cluster:")
        for g, v in byg.most_common():
            print(f"  {v:>4}  {g}")
        print("\nall offending tasks:")
        for r in allrows:
            if r["outcome"] == "PROVE":
                print(f"  {r['rel_yml']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
