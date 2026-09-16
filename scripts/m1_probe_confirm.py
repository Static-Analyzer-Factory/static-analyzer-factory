#!/usr/bin/env python3
"""Probe 2-B: do SAF's own correctness witnesses still CONFIRM after the fixes?

The +12 weighted this movement recovers is ENTIRELY `no-overflow` TRUE, and every
`C.no-overflow.*` base category needs a CONFIRMED 2.0+ correctness witness. The
witness body is SAF's converged interval fixpoint -- which both fixes change. So a
fix that keeps the PROVE but perturbs the invariants could still cost the points.

Emits SAF's witness with the PRE-fix and POST-fix binaries over a sample of the
no-overflow tasks SAF proves, validates both with the offline CPAchecker oracle
(the same instrument scripts/svcomp_split_eval.py uses to decide CONFIRMED), and
compares.
"""
import json, os, random, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

PRE = "/workspace/.m1-prefix-wt/target/release/saf"
POST = "./target/release/saf"
SVB = "tests/benchmarks/sv-benchmarks/c"
PRP = "tests/programs/c/svcomp/no-overflow.prp"
N = int(sys.argv[1]) if len(sys.argv) > 1 else 20
JOBS = int(sys.argv[2]) if len(sys.argv) > 2 else 6

rows = [json.loads(l) for l in open("m1-true99-native.jsonl")]
proved = [r for r in rows if r["property"] == "no-overflow" and r["outcome"] == "PROVE"]
rng = random.Random(20260914)
rng.shuffle(proved)
sample = proved[:N]
print(f"no-overflow tasks SAF proves: {len(proved)}; sampling {len(sample)}", flush=True)


def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    txt = open(y, encoding="utf-8", errors="replace").read()
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    src = os.path.join(os.path.dirname(y), m.group(1).strip())
    return src if os.path.exists(src) else None


def one(args):
    tag, binpath, r = args
    src = resolve(r["rel_yml"])
    if not src:
        return (tag, r["rel_yml"], r["cluster"], "NO_SOURCE")
    wit = f"/tmp/m1wit_{tag}_{abs(hash(r['rel_yml']))}.yml"
    try:
        p = subprocess.run([binpath, "emit-correctness-witness", "--data-model",
                            r["data_model"], "-o", wit, src],
                           capture_output=True, text=True, timeout=300)
        if p.returncode != 0 or not os.path.exists(wit):
            return (tag, r["rel_yml"], r["cluster"], "EMIT_FAILED")
        v = subprocess.run(["bash", "scripts/validate_correctness_witness.sh",
                            wit, src, PRP, r["data_model"]],
                           capture_output=True, text=True, timeout=600)
        out = v.stdout + v.stderr
        for k in ("CONFIRMED", "NOT_CONFIRMED", "CPACHECKER_ABSENT", "CPACHECKER_SKIPPED"):
            if k in out:
                # NOT_CONFIRMED contains CONFIRMED as a substring -- check it first
                return (tag, r["rel_yml"], r["cluster"],
                        "NOT_CONFIRMED" if "NOT_CONFIRMED" in out else k)
        return (tag, r["rel_yml"], r["cluster"], "NO_VERDICT")
    except subprocess.TimeoutExpired:
        return (tag, r["rel_yml"], r["cluster"], "TIMEOUT")


jobs = [("pre", PRE, r) for r in sample] + [("post", POST, r) for r in sample]
res = list(ThreadPoolExecutor(max_workers=JOBS).map(one, jobs))
with open("m1-confirm-probe.jsonl", "w") as fh:
    for tag, rel, clu, verdict in res:
        fh.write(json.dumps({"tag": tag, "rel_yml": rel, "cluster": clu, "verdict": verdict}) + "\n")

import collections
for tag in ("pre", "post"):
    c = collections.Counter(v for t, _, _, v in res if t == tag)
    n = sum(c.values())
    print(f"\n=== {tag}-fix: {n} tasks ===")
    for k, v in c.most_common():
        print(f"   {v:4d}  {k}")
    print(f"   CONFIRMED RATE: {c['CONFIRMED']}/{n}")

pre_m = {rel: v for t, rel, _, v in res if t == "pre"}
post_m = {rel: v for t, rel, _, v in res if t == "post"}
changed = [(k, pre_m[k], post_m[k]) for k in pre_m if pre_m[k] != post_m.get(k)]
print(f"\n=== tasks whose confirmation CHANGED: {len(changed)} ===")
for k, a, b in changed:
    print(f"   {k:55s} {a} -> {b}")
