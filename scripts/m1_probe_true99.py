#!/usr/bin/env python3
"""Probe 2-A: does SAF's OWN sentinel prove the 99 tasks the CPAchecker gate carried?

These 99 are the population whose removal cost -12 weighted (READINESS.md 5).
The gate never proved anything -- SAF's interval sentinel PROPOSED and CPAchecker
CONFIRMED. So the sentinel should already prove them. Measure it, don't assume it.
"""
import json, os, subprocess, sys, collections
from concurrent.futures import ThreadPoolExecutor

SAF = "./target/release/saf"
SUBCMD = {"unreach-call": "prove-unreachable", "no-overflow": "prove-no-overflow"}
tasks = [json.loads(l) for l in open("splits/lever1_true99.jsonl")]

def run(r):
    src = r["src"]
    if not os.path.exists(src):
        return {**r, "outcome": "(no source)"}
    try:
        p = subprocess.run([SAF, SUBCMD[r["property"]], "--data-model", r["data_model"], src],
                           capture_output=True, text=True, timeout=300)
        lines = (p.stdout + p.stderr).strip().splitlines()
        out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(no output)")
    except subprocess.TimeoutExpired:
        out = "PROBE_TIMEOUT"
    return {**r, "outcome": out.split(" ")[0]}

res = list(ThreadPoolExecutor(max_workers=8).map(run, tasks))
with open("m1-true99-native.jsonl", "w") as fh:
    for r in res:
        fh.write(json.dumps({k: r[k] for k in ("rel_yml","property","cluster","data_model","outcome")}) + "\n")

for prop in ("no-overflow", "unreach-call"):
    sub = [r for r in res if r["property"] == prop]
    c = collections.Counter(r["outcome"] for r in sub)
    print(f"=== {prop}: {len(sub)} tasks ===")
    for k, v in c.most_common():
        print(f"  {v:4d}  {k}")
    print(f"  NATIVE PROVE RATE: {c['PROVE']}/{len(sub)}")
    miss = [r for r in sub if r["outcome"] != "PROVE"]
    if miss:
        print("  NOT proved natively:")
        for r in miss:
            print(f"    {r['cluster']:35s} {r['rel_yml']:55s} {r['outcome']}")
    print()

allc = collections.Counter(r["outcome"] for r in res)
print(f"OVERALL: {allc['PROVE']}/{len(res)} PROVE natively")
byclu = collections.Counter(r["cluster"] for r in res if r["outcome"] == "PROVE")
print(f"clusters with >=1 native PROVE: {len(byclu)}")
for g, v in sorted(byclu.items()):
    print(f"  {v:4d}  {g}")
