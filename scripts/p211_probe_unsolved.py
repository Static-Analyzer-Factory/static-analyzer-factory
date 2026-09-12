#!/usr/bin/env python3
"""Funnel restricted to the clusters where points are ACTUALLY available.

The plain funnel samples every cluster holding TRUE tasks — including the ones SAF
already scores in, which are worth ZERO more under the per-(group,property) dedup cap.
This restricts the probe to the UNSOLVED TRUE clusters (score 0 today, >=1 ground-truth
TRUE task), which is the only dedup-weighted headroom that exists, and reports the
abstain distribution PER CLUSTER — because one cluster is worth exactly one point
whether it holds 3 tasks or 6,789.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 /workspace/probe_unsolved.py no-overflow 4 7'
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
CONC = re.compile(rb"pthread_create|__VERIFIER_atomic|thrd_create")


def resolve_source(rel_yml: str) -> str | None:
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
    prop = sys.argv[1] if len(sys.argv) > 1 else "no-overflow"
    per = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    jobs = int(sys.argv[3]) if len(sys.argv) > 3 else 7
    subcmd = SUBCMD[prop]

    # Post-scoreboard-fix cluster scores (patch TRUE rows from the re-score dump).
    rows = [json.loads(l) for l in open("lever1-pertask.jsonl")]
    fix = {}
    for l in open("lever1-true99-pertask.jsonl"):
        r = json.loads(l)
        fix[r["rel_yml"] + "|" + r["property"]] = r
    for r in rows:
        k = r["rel_yml"] + "|" + r["property"]
        if k in fix and r["confirmed"] == 0 and fix[k]["confirmed"] > 0:
            r["confirmed"] = fix[k]["confirmed"]

    score = collections.defaultdict(int)
    trues = collections.defaultdict(list)
    for r in rows:
        k = (r["group"], r["property"])
        score[k] += r["confirmed"]
        if r["expected"] and r["property"] == prop:
            trues[k].append(r)

    unsolved = sorted(k for k in trues if score[k] <= 0)
    tasks = []
    for k in unsolved:
        tasks += sorted(trues[k], key=lambda r: r["rel_yml"])[:per]
    print(f"{prop}: {len(unsolved)} UNSOLVED TRUE clusters, probing {len(tasks)} tasks "
          f"(<= {per}/cluster) with `saf {subcmd}`", flush=True)

    def run(r):
        src = resolve_source(r["rel_yml"])
        if not src:
            return r["group"], "(no source)", False
        conc = bool(CONC.search(open(src, "rb").read()))
        try:
            p = subprocess.run([SAF, subcmd, "--data-model", r["data_model"], src],
                               capture_output=True, text=True, timeout=180)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))),
                       "(no output)")
        except subprocess.TimeoutExpired:
            out = "PROBE_TIMEOUT"
        return r["group"], out.split(" ")[0], conc

    res = list(ThreadPoolExecutor(max_workers=jobs).map(run, tasks))
    with open(f"probe_unsolved_{prop}.jsonl", "w") as fh:
        for g, o, c in res:
            fh.write(json.dumps({"group": g, "outcome": o, "conc": c}) + "\n")

    per_cluster = collections.defaultdict(collections.Counter)
    conc_of = {}
    for g, o, c in res:
        per_cluster[g][o] += 1
        conc_of[g] = conc_of.get(g, False) or c

    print(f"\n=== per-CLUSTER best outcome over {len(per_cluster)} unsolved TRUE "
          f"clusters (a cluster is worth exactly 1 point) ===")
    best = {}
    for g, c in per_cluster.items():
        best[g] = "PROVE" if c["PROVE"] else c.most_common(1)[0][0]
    hist = collections.Counter(best.values())
    for k, v in hist.most_common():
        seq = sum(1 for g, b in best.items() if b == k and not conc_of[g])
        print(f"  {v:>4} clusters  {k:34s} (sequential: {seq}, concurrent: {v - seq})")
    provable = [g for g, b in best.items() if b == "PROVE"]
    print(f"\nclusters ALREADY provable but unscored (a wiring/witness gap, not an "
          f"analysis gap): {len(provable)}")
    for g in sorted(provable):
        print(f"  {g}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
