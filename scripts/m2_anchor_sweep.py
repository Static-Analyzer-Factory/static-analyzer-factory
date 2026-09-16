#!/usr/bin/env python3
"""Movement 2 spike sweeps: yield over the unsolved clusters, soundness over ALL
expected-FALSE `valid-memsafety` tasks.

Task list comes from the authoritative post-Movement-1 per-task dump (rel_yml,
data_model, group, sets) so clusters are classified by `.set` membership, never by
grepping sources for `pthread_create` (plans/214 sec 5).

    python3 m2_anchor_sweep.py yield  <pertask.jsonl> <out.jsonl> [jobs]
    python3 m2_anchor_sweep.py false  <pertask.jsonl> <out.jsonl> [jobs]
"""
from __future__ import annotations

import collections
import json
import os
import sys
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m2_anchor_prototype import parse_yml, prove_source   # noqa: E402

SVB = os.environ.get("M2_SVB", "/workspace/tests/benchmarks/sv-benchmarks/c")


def resolve(rel_yml: str):
    y = os.path.join(SVB, rel_yml)
    try:
        inp, dm, _ = parse_yml(y)
    except OSError:
        return None, None
    if not inp:
        return None, None
    src = os.path.join(os.path.dirname(y), inp)
    return (src if os.path.exists(src) else None), dm


def work(item):
    rel, group, sets, dm_run = item
    src, dm = resolve(rel)
    if src is None:
        return {"rel_yml": rel, "group": group, "sets": sets, "ok": False,
                "why": "no-source"}
    ok, why = prove_source(src, dm or dm_run)
    return {"rel_yml": rel, "group": group, "sets": sets, "ok": ok, "why": why}


def main() -> int:
    mode, pertask, out = sys.argv[1], sys.argv[2], sys.argv[3]
    jobs = int(sys.argv[4]) if len(sys.argv) > 4 else 14

    rows = []
    score = collections.Counter()
    trues = collections.defaultdict(list)
    falses = []
    for line in open(pertask):
        r = json.loads(line)
        if r["property"] != "valid-memsafety":
            continue
        rows.append(r)
        score[r["group"]] += r["confirmed"]
        if r["expected"]:
            trues[r["group"]].append(r)
        else:
            falses.append(r)

    if mode == "yield":
        unsolved = sorted(g for g in trues if score[g] <= 0)
        items = [(r["rel_yml"], r["group"], r.get("sets") or [], r["data_model"])
                 for g in unsolved for r in sorted(trues[g], key=lambda x: x["rel_yml"])]
        print(f"yield: {len(unsolved)} unsolved clusters, {len(items)} expected-TRUE "
              f"tasks", flush=True)
    else:
        items = [(r["rel_yml"], r["group"], r.get("sets") or [], r["data_model"])
                 for r in sorted(falses, key=lambda x: x["rel_yml"])]
        print(f"soundness: {len(items)} expected-FALSE valid-memsafety tasks", flush=True)

    done = 0
    with open(out, "w") as fh, ProcessPoolExecutor(max_workers=jobs) as ex:
        for res in ex.map(work, items, chunksize=4):
            fh.write(json.dumps(res) + "\n")
            done += 1
            if done % 500 == 0:
                fh.flush()
                print(f"  ... {done}/{len(items)}", flush=True)
    print(f"DONE {done}/{len(items)} -> {out}", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
