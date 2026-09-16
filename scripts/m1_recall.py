#!/usr/bin/env python3
"""Movement 1: measure the RECALL delta of the two absint fixes, pre vs post.

Runs the unreach-call sentinel over a stratified sample of expected-TRUE tasks with
BOTH binaries and reports the PROVE delta, split by whether the task's 2027 base
category needs a confirmed correctness witness:

  * VERDICT-ONLY (C.unreach-call.{Arrays,Heap,Floats}) -- a PROVE is bankable
    immediately, no witness, no validator.
  * WITNESS-REQUIRED -- a PROVE is worth 0 unless a validator confirms SAF's
    correctness witness, and that validator went 0-for-5 on SAF's best existing
    proofs (saf-validator-ceiling-true-side).
"""
import collections, json, os, random, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, "scripts")
import svcomp_witness_rules as R  # noqa: E402

PRE = "/workspace/.m1-prefix-wt/target/release/saf"
POST = "./target/release/saf"
SVB = "tests/benchmarks/sv-benchmarks/c"
N = int(sys.argv[1]) if len(sys.argv) > 1 else 700

member = R.load_set_membership("tests/benchmarks/sv-benchmarks")

tasks = []
for line in open("lever1-pertask.jsonl"):
    r = json.loads(line)
    if r["property"] != "unreach-call" or not r["expected"]:
        continue
    sets = member.get(r["rel_yml"])
    if not sets:
        continue  # out of competition for this property
    cats = R.base_categories("unreach-call", sets)
    if not cats:
        continue
    need, _ver = R.true_witness_requirement("unreach-call", cats)
    r["cluster"] = r.get("cluster") or r.get("group") or "?"
    r["cats"] = sorted(cats)
    r["verdict_only"] = not need
    tasks.append(r)

print(f"in-competition expected-TRUE unreach-call tasks: {len(tasks)}")
vo = [t for t in tasks if t["verdict_only"]]
print(f"  verdict-only (Arrays/Heap/Floats): {len(vo)}")
print(f"  witness-required:                  {len(tasks)-len(vo)}")

# stratify by cluster so no single huge family dominates the sample
byclu = collections.defaultdict(list)
for t in tasks:
    byclu[t["cluster"]].append(t)
rng = random.Random(20260914)
sample, per = [], max(1, N // max(1, len(byclu)))
for clu, ts in sorted(byclu.items()):
    rng.shuffle(ts)
    sample.extend(ts[:per])
rng.shuffle(sample)
sample = sample[:N]
print(f"sampled {len(sample)} tasks across {len(byclu)} clusters\n")


def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    try:
        txt = open(y, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    import re
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    if not m:
        return None
    src = os.path.join(os.path.dirname(y), m.group(1).strip())
    return src if os.path.exists(src) else None


def run_one(args):
    binpath, t = args
    src = resolve(t["rel_yml"])
    if not src:
        return "(no source)"
    try:
        p = subprocess.run([binpath, "prove-unreachable", "--data-model", t["data_model"], src],
                           capture_output=True, text=True, timeout=120)
        lines = (p.stdout + p.stderr).strip().splitlines()
        out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(no output)")
    except subprocess.TimeoutExpired:
        out = "TIMEOUT"
    return out.split(" ")[0]


rows = []
for label, binpath in (("pre", PRE), ("post", POST)):
    if not os.path.exists(binpath):
        print(f"!! missing binary {binpath}")
        sys.exit(1)
    res = list(ThreadPoolExecutor(max_workers=10).map(run_one, [(binpath, t) for t in sample]))
    for t, o in zip(sample, res):
        t[f"out_{label}"] = o
    c = collections.Counter(res)
    print(f"=== {label}: {c['PROVE']}/{len(sample)} PROVE ===")
    for k, v in c.most_common(6):
        print(f"   {v:5d}  {k}")
    print()

with open("m1-recall-delta.jsonl", "w") as fh:
    for t in sample:
        fh.write(json.dumps({k: t[k] for k in
                 ("rel_yml", "cluster", "cats", "verdict_only", "out_pre", "out_post")}) + "\n")

pre_p = sum(1 for t in sample if t["out_pre"] == "PROVE")
post_p = sum(1 for t in sample if t["out_post"] == "PROVE")
gained = [t for t in sample if t["out_pre"] != "PROVE" and t["out_post"] == "PROVE"]
lost = [t for t in sample if t["out_pre"] == "PROVE" and t["out_post"] != "PROVE"]

print("=" * 64)
print(f"RECALL: pre {pre_p}  ->  post {post_p}   (delta {post_p-pre_p:+d})")
print(f"  newly PROVEd : {len(gained)}")
print(f"  lost         : {len(lost)}")
print()
gv = [t for t in gained if t["verdict_only"]]
print(f"  newly PROVEd in VERDICT-ONLY categories (bankable, no witness): {len(gv)}")
for t in gv:
    print(f"     {t['cluster']:32s} {','.join(t['cats']):18s} {t['rel_yml']}")
gw = [t for t in gained if not t["verdict_only"]]
print(f"  newly PROVEd in WITNESS-REQUIRED categories: {len(gw)}")
for g, v in collections.Counter(t["cluster"] for t in gw).most_common(20):
    print(f"     {v:4d}  {g}")
if lost:
    print("\n  LOST (precision regression -- investigate):")
    for t in lost:
        print(f"     {t['cluster']:32s} {t['rel_yml']}  {t['out_pre']} -> {t['out_post']}")
print()
print("NEW CLUSTERS with >=1 PROVE post that had 0 pre (in this sample):")
pre_clu = {t["cluster"] for t in sample if t["out_pre"] == "PROVE"}
post_clu = {t["cluster"] for t in sample if t["out_post"] == "PROVE"}
newc = sorted(post_clu - pre_clu)
print(f"  {len(newc)} clusters: {', '.join(newc) if newc else '(none)'}")
