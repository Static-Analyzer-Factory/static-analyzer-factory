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
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")


# Each prove-* subcommand prints exactly one line: `PROVE` or `ABSTAIN:<reason>`.
SUBCMD = {"unreach-call": "prove-unreachable", "no-overflow": "prove-no-overflow"}


def sentinel(src: str, data_model: str, timeout: int, subcmd: str) -> str:
    """Run the sound sentinel; return `PROVE` or `ABSTAIN:<reason>`."""
    cmd = [SAF, subcmd, "--data-model", data_model, src]
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return "PROBE_TIMEOUT"
    lines = (p.stdout + p.stderr).strip().splitlines()
    return next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(no output)")


def resolve_source(rel_yml: str) -> str | None:
    """The source file the TASK DEFINITION names — never a guess.

    13,419 of 36,336 expected-TRUE tasks ship both a `.c` and a preprocessed `.i` and the
    .yml names the `.i`. Probing the `.c` analyses a different program than the harness
    does, and for benchmarks carrying their own `assert.h` it does not compile at all.
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
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--per-task", required=True,
                    help="a per-task JSONL from svcomp_split_eval.py (the task pool)")
    ap.add_argument("--property", default="unreach-call", choices=sorted(SUBCMD))
    ap.add_argument("--subcommand", default=None,
                    help="prove-* subcommand to drive (default: derived from --property)")
    ap.add_argument("-n", "--sample", type=int, default=120)
    ap.add_argument("--per-cluster", type=int, default=2,
                    help="cap per origin cluster BEFORE sampling, so one huge cluster "
                         "cannot dominate the histogram")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--timeout", type=int, default=120)
    ap.add_argument("--seed", type=int, default=0)
    args = ap.parse_args()

    subcmd = args.subcommand or SUBCMD[args.property]
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
    print(f"{len(by_cluster)} clusters hold {args.property} TRUE tasks; probing "
          f"{len(tasks)} with `saf {subcmd}`",
          flush=True)

    def run(r: dict):
        src = resolve_source(r["rel_yml"])
        if not src:
            return r.get("group", ""), "(no source)"
        return r.get("group", ""), sentinel(src, r["data_model"], args.timeout, subcmd)

    res = list(ThreadPoolExecutor(max_workers=args.jobs).map(run, tasks))
    hist = collections.Counter(
        o.split(" ")[0] if o.startswith(("PROVE", "ABSTAIN")) else o for _, o in res)
    proved = sum(1 for _, o in res if o.startswith("PROVE"))
    print(f"\nPROVE: {proved} / {len(res)}  ({100 * proved / max(len(res), 1):.1f}%)\n")
    for reason, n in hist.most_common():
        print(f"  {n:>5}  {reason}")
    print("\nproved clusters:", sorted({g for g, o in res if o.startswith('PROVE')}) or "(none)")
    per_cluster: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for g, o in res:
        per_cluster[g][o.split(" ")[0]] += 1
    print(f"\nper-cluster dominant outcome ({len(per_cluster)} clusters probed):")
    dom = collections.Counter(c.most_common(1)[0][0] for c in per_cluster.values())
    for reason, n in dom.most_common():
        print(f"  {n:>5} clusters  {reason}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
