#!/usr/bin/env python3
"""Run the RUST Anchored-Object prover over the same expected-TRUE population the
Python prototype was measured on (`m2-FINAL-yield.jsonl`), so the two are directly
comparable and the porting attrition is a measured number rather than an estimate.
"""
import collections, json, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

SAF = os.environ.get("SAF_BIN", "./target/release/saf")
SVB = os.environ.get("SAF_SVB", "tests/benchmarks/sv-benchmarks/c")


def resolve(rel_yml):
    y = os.path.join(SVB, rel_yml)
    try:
        txt = open(y, encoding="utf-8", errors="replace").read()
    except OSError:
        return None, "ILP32"
    m = re.search(r"input_files:\s*['\"]?([^'\"\n]+)", txt)
    dm = re.search(r"data_model:\s*(\w+)", txt)
    if not m:
        return None, "ILP32"
    src = os.path.join(os.path.dirname(y), m.group(1).strip())
    return (src if os.path.exists(src) else None), (dm.group(1) if dm else "ILP32")


def main():
    yield_file = sys.argv[1] if len(sys.argv) > 1 else "m2-FINAL-yield.jsonl"
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 12
    out_path = "m2-rust-yield.jsonl"

    rows = [json.loads(l) for l in open(yield_file) if l.strip()]
    done = set()
    if os.path.exists(out_path):
        for l in open(out_path):
            try:
                done.add(json.loads(l)["rel_yml"])
            except Exception:  # noqa: BLE001
                pass
    tasks = [r for r in rows if r["rel_yml"] not in done]
    print(f"{len(tasks)} expected-TRUE tasks to probe ({len(done)} done)", flush=True)

    fh = open(out_path, "a", buffering=1)

    def run(r):
        src, dm = resolve(r["rel_yml"])
        if not src:
            return {**r, "rust": "(no-source)"}
        try:
            p = subprocess.run([SAF, "memsafe-prove", "--data-model", dm, src],
                               capture_output=True, text=True, timeout=180)
            lines = (p.stdout + p.stderr).strip().splitlines()
            out = next((x for x in reversed(lines) if x.startswith(("PROVE", "ABSTAIN"))),
                       "(no-output)")
        except subprocess.TimeoutExpired:
            out = "TIMEOUT"
        return {**r, "rust": out.split(" ")[0]}

    n = 0
    for res in ThreadPoolExecutor(max_workers=jobs).map(run, tasks):
        fh.write(json.dumps(res) + "\n")
        n += 1
        if n % 100 == 0:
            print(f"  {n}/{len(tasks)}", flush=True)
    fh.close()

    rows = [json.loads(l) for l in open(out_path) if l.strip()]
    proto_ok = [r for r in rows if r.get("ok")]
    rust_ok = [r for r in rows if r["rust"] == "PROVE"]
    print(f"\npopulation            : {len(rows)}")
    print(f"prototype PROVE (LLVM): {len(proto_ok)}  in {len({r['group'] for r in proto_ok})} clusters")
    print(f"RUST      PROVE (AIR) : {len(rust_ok)}  in {len({r['group'] for r in rust_ok})} clusters")
    print("\nRUST PROVE by cluster:")
    for g, c in collections.Counter(r["group"] for r in rust_ok).most_common():
        print(f"   {g:28s} {c}")
    lost = [r for r in proto_ok if r["rust"] != "PROVE"]
    print(f"\nATTRITION: {len(lost)} tasks the prototype proved that the Rust port does not")
    for why, c in collections.Counter(r["rust"] for r in lost).most_common(12):
        print(f"   {why:44s} {c}")
    gained = [r for r in rust_ok if not r.get("ok")]
    if gained:
        print(f"\nRust proves {len(gained)} the prototype did not:")
        for why, c in collections.Counter(r.get("why", "?") for r in gained).most_common(8):
            print(f"   (prototype said) {why:34s} {c}")


if __name__ == "__main__":
    main()
