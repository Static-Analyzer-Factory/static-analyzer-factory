#!/usr/bin/env python3
"""Verify-path check on the no-overflow TRUE population: does the re-enabled arm
actually emit `true`, and does it still emit `false` on the FALSE ones?"""
import json, os, re, subprocess, sys, collections
from concurrent.futures import ThreadPoolExecutor
os.chdir("/workspace" if os.path.isdir("/workspace/crates") else os.path.expanduser("~/static-analyzer-factory"))
SVB = "tests/benchmarks/sv-benchmarks/c"
PRP = "tests/programs/c/svcomp/no-overflow.prp"
N = int(sys.argv[1]) if len(sys.argv) > 1 else 40

rows = [json.loads(l) for l in open("m1-true99-native.jsonl")]
nov = [r for r in rows if r["property"] == "no-overflow"][:N]

def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", open(y, encoding="utf-8", errors="replace").read())
    src = os.path.join(os.path.dirname(y), m.group(1).strip())
    return src if os.path.exists(src) else None

def one(r):
    src = resolve(r["rel_yml"])
    if not src:
        return (r, "NO_SOURCE")
    try:
        p = subprocess.run(["./target/release/saf", "verify", "--property", PRP,
                            "--data-model", r["data_model"], src],
                           capture_output=True, text=True, timeout=300)
        return (r, (p.stdout or "").strip().splitlines()[0] if p.stdout.strip() else "(empty)")
    except subprocess.TimeoutExpired:
        return (r, "TIMEOUT")

res = list(ThreadPoolExecutor(max_workers=8).map(one, nov))
c = collections.Counter(v for _, v in res)
print(f"=== verify path on {len(res)} expected-TRUE no-overflow tasks ===")
for k, v in c.most_common():
    print(f"   {v:4d}  {k}")
print(f"\nTRUE emitted: {c['true']}/{len(res)}")
bad = [(r, v) for r, v in res if v.startswith("false")]
print(f"WRONG FALSE (would be a false alarm): {len(bad)}")
for r, v in bad:
    print(f"   {r['cluster']:30s} {r['rel_yml']}  -> {v}")
notrue = [(r, v) for r, v in res if v != "true"]
print(f"\nnot true ({len(notrue)}):")
for r, v in notrue[:15]:
    print(f"   {r['cluster']:30s} {r['rel_yml']:52s} {v}  (sentinel: {r['outcome']})")
