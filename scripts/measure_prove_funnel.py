#!/usr/bin/env python3
"""Measure SAF's TRUE-side proving funnel: how often the sound sentinel PROVEs, and
where it abstains when it does not.

This is the progress instrument for building SAF's own prover. The verdict-level
score moves far too slowly to steer by (a prover improvement shows up as a handful of
tasks), whereas the abstain-reason histogram tells you immediately WHICH wall you are
standing in front of.

    docker compose run --rm -T -e SKIP_MATURIN_BUILD=1 dev sh -c \\
        'python3 scripts/measure_prove_funnel.py --per-task lever1-pertask.jsonl -n 120'

Baseline recorded 2026-09-12 (unreach-call, n=120 stratified over 115 clusters):

    PROVE                      2      <- 1.7%
    ABSTAIN:error-reachable   80      <- the real wall: the absint cannot kill the block
    ABSTAIN:threads           17
    (no output)               13
    ABSTAIN:module-too-large   7
    ABSTAIN:not-converged      1

`error-reachable` dominating is the finding that matters: it is NOT a cost guard and
NOT convergence, so raising limits or swapping the abstract domain does not move it.
SAF analyses each function independently from a TOP entry, and 92.5% of these tasks
hide `reach_error` inside a `void __VERIFIER_assert(int cond)` wrapper, so `cond` is
unconstrained and the error block is trivially "reachable". Inlining that wrapper was
measured to flip only 1 of 9 applicable tasks (0 of 4 in the largest cluster), so the
gap is interprocedural reasoning, not domain precision.
"""
from __future__ import annotations

import argparse
import collections
import json
import os
import random
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")


def sentinel(src: str, data_model: str, timeout: int) -> str:
    """Run the sound sentinel; return `PROVE` or `ABSTAIN:<reason>`."""
    cmd = [SAF, "prove-unreachable", "--data-model", data_model, src]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return "PROBE_TIMEOUT"
    lines = (p.stdout + p.stderr).strip().splitlines()
    return next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(no output)")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--per-task", required=True,
                    help="a per-task JSONL from svcomp_split_eval.py (the task pool)")
    ap.add_argument("--property", default="unreach-call")
    ap.add_argument("-n", "--sample", type=int, default=120)
    ap.add_argument("--per-cluster", type=int, default=2,
                    help="cap per origin cluster BEFORE sampling, so one huge cluster "
                         "cannot dominate the histogram")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--seed", type=int, default=0)
    args = ap.parse_args()

    random.seed(args.seed)
    by_cluster: dict[str, list[dict]] = collections.defaultdict(list)
    for line in open(args.per_task):
        r = json.loads(line)
        # Ground-truth-TRUE tasks only: those are the ones a prover could ever win.
        if r["property"] == args.property and r["expected"]:
            by_cluster[r.get("group", "")].append(r)
    pool: list[dict] = []
    for _, rs in sorted(by_cluster.items()):
        pool += random.sample(rs, min(args.per_cluster, len(rs)))
    tasks = random.sample(pool, min(args.sample, len(pool)))
    print(f"{len(by_cluster)} clusters hold {args.property} TRUE tasks; probing {len(tasks)}",
          flush=True)

    def run(r: dict):
        base = r["rel_yml"][:-4]
        src = next((f"{SVB}/{base}{e}" for e in (".c", ".i")
                    if os.path.exists(f"{SVB}/{base}{e}")), None)
        if not src:
            return r.get("group", ""), "(no source)"
        return r.get("group", ""), sentinel(src, r["data_model"], args.timeout)

    res = list(ThreadPoolExecutor(max_workers=args.jobs).map(run, tasks))
    hist = collections.Counter(
        o.split(" ")[0] if o.startswith(("PROVE", "ABSTAIN")) else o for _, o in res)
    proved = sum(1 for _, o in res if o.startswith("PROVE"))
    print(f"\nPROVE: {proved} / {len(res)}  ({100 * proved / max(len(res), 1):.1f}%)\n")
    for reason, n in hist.most_common():
        print(f"  {n:>5}  {reason}")
    print("\nproved clusters:", sorted({g for g, o in res if o.startswith('PROVE')}) or "(none)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
