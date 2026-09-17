#!/usr/bin/env python3
"""How much no-overflow recall does wiring up `fixpoint_verified` cost?

plans/213 measured the post-fixpoint check as "costs nothing (93/99 unchanged)",
but that was taken while the flag was disconnected, so it measured nothing. This
re-measures the sentinel over a stride sample of expected-TRUE no-overflow tasks.
"""
import collections, json, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "target/release/saf")
SVB = "tests/benchmarks/sv-benchmarks/c"


def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    txt = open(y, encoding="utf-8", errors="replace").read()
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    dm = re.search(r"data_model:\s*(\w+)", txt)
    src = os.path.join(os.path.dirname(y), m.group(1).strip()) if m else None
    return (src if src and os.path.exists(src) else None), (dm.group(1) if dm else "ILP32")


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    rows = [json.loads(l) for l in open("splits/train.jsonl") if l.strip()]
    trues = [r for r in rows if r["property"] == "no-overflow" and r["expected"]]
    step = max(1, len(trues) // n)
    sample = trues[::step][:n]
    print(f"{len(sample)} expected-TRUE no-overflow tasks (stride of {len(trues)})", flush=True)

    def run(r):
        src, dm = resolve(r["rel_yml"])
        if not src:
            return "(no-source)"
        try:
            p = subprocess.run([SAF, "prove-no-overflow", "--data-model", dm, src],
                               capture_output=True, text=True, timeout=180)
            lines = (p.stdout + p.stderr).strip().splitlines()
            return next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(none)").split(" ")[0]
        except subprocess.TimeoutExpired:
            return "TIMEOUT"

    out = list(ThreadPoolExecutor(max_workers=12).map(run, sample))
    c = collections.Counter(out)
    print(f"  PROVE = {c.get('PROVE', 0)} / {len(sample)}")
    for k, v in c.most_common(10):
        print(f"    {k:40s} {v}")


if __name__ == "__main__":
    main()
