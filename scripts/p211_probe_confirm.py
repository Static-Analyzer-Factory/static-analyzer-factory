#!/usr/bin/env python3
"""For every UNSOLVED no-overflow TRUE cluster, find a task SAF's own sentinel PROVES,
then ask whether the correctness witness CONFIRMS.

This separates two very different failures that both show up as "cluster scores 0":
  (a) SAF cannot prove it            -> a PROVER problem (build analysis)
  (b) SAF proves it, validator won't confirm -> a VALIDATOR-CAPABILITY ceiling

(b) matters enormously for plan 211's ranking: unreach-call and no-overflow TRUE score
ONLY on a confirmed correctness witness, while termination / valid-memsafety /
valid-memcleanup / no-data-race TRUE score on the verdict alone. If SAF's proofs cannot
be confirmed, no amount of prover work converts them into points.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 /workspace/probe_confirm.py no-overflow 6 4'
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
PRP = {"unreach-call": "unreach-call.prp", "no-overflow": "no-overflow.prp"}


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
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    per = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    subcmd, prp = SUBCMD[prop], os.path.join(SVB, "properties", PRP[prop])

    rows = [json.loads(l) for l in open("lever1-pertask.jsonl")]
    fix = {}
    for l in open("lever1-true99-pertask.jsonl"):
        r = json.loads(l)
        fix[r["rel_yml"] + "|" + r["property"]] = r
    for r in rows:
        k = r["rel_yml"] + "|" + r["property"]
        if k in fix and r["confirmed"] == 0 and fix[k]["confirmed"] > 0:
            r["confirmed"] = fix[k]["confirmed"]

    score, trues = collections.defaultdict(int), collections.defaultdict(list)
    for r in rows:
        k = (r["group"], r["property"])
        score[k] += r["confirmed"]
        if r["expected"] and r["property"] == prop:
            trues[k].append(r)
    unsolved = sorted(k for k in trues if score[k] <= 0)
    cand = []
    for k in unsolved:
        cand += sorted(trues[k], key=lambda r: r["rel_yml"])[:per]
    print(f"{prop}: {len(unsolved)} unsolved TRUE clusters, {len(cand)} candidate tasks",
          flush=True)

    def prove(r):
        src = resolve_source(r["rel_yml"])
        if not src:
            return None
        try:
            p = subprocess.run([SAF, subcmd, "--data-model", r["data_model"], src],
                               capture_output=True, text=True, timeout=180)
        except subprocess.TimeoutExpired:
            return None
        if "PROVE" not in (p.stdout + p.stderr):
            return None
        return (r["group"], r["rel_yml"], src, r["data_model"])

    proven = [x for x in ThreadPoolExecutor(max_workers=jobs).map(prove, cand) if x]
    by_cluster = {}
    for g, y, s, dm in proven:
        by_cluster.setdefault(g, (y, s, dm))
    print(f"SAF PROVES at least one task in {len(by_cluster)} of {len(unsolved)} "
          f"unsolved clusters\n", flush=True)

    def confirm(item):
        g, (y, src, dm) = item
        w = f"/tmp/w_{abs(hash(g))}.yml"
        try:
            with open(w, "w") as fh:
                p = subprocess.run([SAF, "emit-correctness-witness", "--data-model", dm, src],
                                   stdout=fh, stderr=subprocess.PIPE, text=True, timeout=180)
            if os.path.getsize(w) == 0:
                return g, y, "NO_WITNESS", (p.stderr or "").strip()[:90]
            v = subprocess.run(["bash", "scripts/validate_correctness_witness.sh", w, src, prp, dm],
                               capture_output=True, text=True, timeout=900)
            tail = [x for x in v.stdout.strip().splitlines() if "CONFIRM" in x or "ABSENT" in x]
            return g, y, (tail[-1] if tail else "NO_VERDICT"), ""
        except subprocess.TimeoutExpired:
            return g, y, "VALIDATOR_TIMEOUT", ""

    res = list(ThreadPoolExecutor(max_workers=max(2, jobs // 2)).map(confirm, by_cluster.items()))
    print("cluster                          witness/validator outcome")
    for g, y, out, extra in sorted(res):
        print(f"  {g:30s} {out}  {extra}")
    c = collections.Counter(o.split(" ")[0] for _, _, o, _ in res)
    print(f"\n{dict(c)}")
    print(f"\nCONFIRMED clusters = bankable by removing the in-process CPAchecker gate: "
          f"{c['CONFIRMED']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
