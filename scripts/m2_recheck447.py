#!/usr/bin/env python3
"""Re-verify the merge gate's TRUE set against the FINAL binary.

The gate ran before two late changes: the empty-thread-discovery guard (which can
only REMOVE proofs) and the precomputed global-size map (a pure refactor). Neither
can add a proof, so the gate's +10 is only valid if every task it scored
`TrueCorrect` still PROVEs. This checks exactly those tasks.
"""
import collections, json, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")


def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    txt = open(y, encoding="utf-8", errors="replace").read()
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    dm = re.search(r"data_model:\s*(\w+)", txt)
    src = os.path.join(os.path.dirname(y), m.group(1).strip()) if m else None
    return (src if src and os.path.exists(src) else None), (dm.group(1) if dm else "ILP32")


def main():
    pertask = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 12
    rows = [json.loads(l) for l in open(pertask) if l.strip()]
    trues = [r for r in rows if r["outcome"] == "TrueCorrect"]
    print(f"gate scored {len(trues)} tasks TrueCorrect; re-proving with the final binary", flush=True)

    def run(r):
        src, dm = resolve(r["rel_yml"])
        if not src:
            return (r, "(no-source)")
        try:
            p = subprocess.run([SAF, "memsafe-prove", "--data-model", dm, src],
                               capture_output=True, text=True, timeout=180)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))), "(no-output)")
        except subprocess.TimeoutExpired:
            out = "TIMEOUT"
        return (r, out.split(" ")[0])

    res = list(ThreadPoolExecutor(max_workers=jobs).map(run, trues))
    proved = [r for r, o in res if o == "PROVE"]
    lost = [(r, o) for r, o in res if o != "PROVE"]
    print(f"  still PROVE : {len(proved)} / {len(trues)}")
    if lost:
        print(f"  LOST        : {len(lost)}")
        for o, c in collections.Counter(o for _, o in lost).most_common():
            print(f"      {o:48s} {c}")
        byg = collections.Counter(r.get("group", "?") for r, _ in lost)
        print("    by cluster:", dict(byg.most_common(10)))
        clusters_before = {r.get("group") for r in trues}
        clusters_after = {r.get("group") for r in proved}
        print(f"    clusters {len(clusters_before)} -> {len(clusters_after)}; "
              f"lost entirely: {sorted(clusters_before - clusters_after)}")
    else:
        print("  NO LOSS — the gate's +10 stands for the final binary.")


if __name__ == "__main__":
    main()
