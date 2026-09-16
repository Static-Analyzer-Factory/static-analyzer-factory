import json, os, re, subprocess
os.chdir("/workspace") if os.path.isdir("/workspace/crates") else os.chdir(os.path.expanduser("~/static-analyzer-factory"))
rows = [json.loads(l) for l in open("m1-confirm-probe.jsonl")]
pre = {r["rel_yml"]: r["verdict"] for r in rows if r["tag"] == "pre"}
post = {r["rel_yml"]: r["verdict"] for r in rows if r["tag"] == "post"}
fp = {k for k, v in pre.items() if v == "EMIT_FAILED"}
fq = {k for k, v in post.items() if v == "EMIT_FAILED"}
print(f"EMIT_FAILED pre={len(fp)} post={len(fq)}  identical_set={fp == fq}")
print("failing tasks:")
for k in sorted(fp):
    print("  ", k)
print("\nCONFIRMED tasks (both runs):")
for k in sorted(k for k, v in pre.items() if v == "CONFIRMED"):
    print(f"   {k:52s} pre={pre[k]:10s} post={post[k]}")

# why does emit fail? resolve the source the .yml names
SVB = "tests/benchmarks/sv-benchmarks/c"
k = sorted(fp)[0]
y = os.path.join(SVB, k)
txt = open(y, encoding="utf-8", errors="replace").read()
m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
src = os.path.join(os.path.dirname(y), m.group(1).strip())
print(f"\nprobe task: {k}\n  input_files -> {src}\n  exists: {os.path.exists(src)}")
p = subprocess.run(["./target/release/saf", "emit-correctness-witness",
                    "--data-model", "ILP32", "-o", "/tmp/w.yml", src],
                   capture_output=True, text=True, timeout=300)
print(f"  exit={p.returncode}")
print("  stderr:", (p.stderr or "").strip()[:600])
