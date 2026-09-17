#!/usr/bin/env python3
"""Measure what the AIR *actually* exposes on the tasks the Python prototype proved.

The Python prototype ran on raw LLVM IR. The Rust prover must run on AIR. This
script quantifies the gap by compiling each task with the PRODUCTION recipe
(clang -O0 -disable-O0-optnone + opt -passes=mem2reg, per compile_to_ir) and
comparing the LLVM IR against the AIR that `saf index` produces from it.
"""
import argparse, collections, concurrent.futures as cf, json, os, re, subprocess, sys, tempfile

BENCH = "tests/benchmarks/sv-benchmarks/c"
STUB = "share/saf/stubs/sv-comp-stubs.h"
SAF = "target/release/saf"
CLANG = os.environ.get("SAF_CLANG", "clang-18")
OPT = os.environ.get("SAF_OPT", "opt-18")
AGG = {"array", "struct", "vector"}


def parse_yml(path):
    txt = open(path, errors="ignore").read()
    inp = (re.search(r"input_files:\s*'([^']+)'", txt)
           or re.search(r'input_files:\s*"([^"]+)"', txt)
           or re.search(r"input_files:[ \t]*([^\s'\"\[\n][^\n]*)", txt))
    dm = re.search(r"data_model:\s*(\w+)", txt)
    return (inp.group(1).strip() if inp else None, dm.group(1) if dm else "ILP32")


def analyze(rel_yml):
    yml = os.path.join(BENCH, rel_yml)
    out = {"rel_yml": rel_yml}
    inp, dm = parse_yml(yml)
    if not inp:
        out["status"] = "no-input"; return out
    src = os.path.join(os.path.dirname(yml), inp)
    out["data_model"] = dm
    with tempfile.TemporaryDirectory() as d:
        ll, ll2, aj = (os.path.join(d, n) for n in ("a.ll", "b.ll", "a.json"))
        mflag = "-m32" if dm.upper() == "ILP32" else "-m64"
        cc = [CLANG, "-g", "-S", "-emit-llvm", "-O0", "-Xclang",
              "-disable-O0-optnone", "-Wno-everything", mflag,
              "-include", STUB, "-I", os.path.dirname(src), src, "-o", ll]
        try:
            if subprocess.run(cc, capture_output=True, timeout=180).returncode:
                out["status"] = "compile-fail"; return out
            if subprocess.run([OPT, "-S", "-passes=mem2reg", ll, "-o", ll2],
                              capture_output=True, timeout=180).returncode:
                out["status"] = "opt-fail"; return out
            lltxt = open(ll2, errors="ignore").read()
            r = subprocess.run([SAF, "index", ll2, "--output", aj],
                               capture_output=True, timeout=600)
            if r.returncode:
                out["status"] = "index-fail"
                out["err"] = r.stderr.decode(errors="ignore")[-160:]
                return out
            m = json.load(open(aj))["module"]
        except subprocess.TimeoutExpired:
            out["status"] = "timeout"; return out
        except Exception as e:                                  # noqa: BLE001
            out["status"] = "error"; out["err"] = str(e)[:160]; return out

    out["status"] = "ok"
    out["ll_gep"] = lltxt.count("getelementptr")
    gids = {g["id"] for g in m.get("globals", [])}
    out["n_globals"] = len(gids)
    out["n_globals_with_value_type"] = sum(1 for g in m.get("globals", []) if g.get("value_type"))
    out["air_ptr_width"] = m.get("target_pointer_width")
    types = m.get("types", {})
    out["n_types"] = len(types)
    out["n_agg_types"] = sum(1 for v in types.values() if v.get("kind") in AGG)

    gep = ldst = ldst_global = memop = inttoptr = 0
    for f in m.get("functions", []):
        if f.get("is_declaration"):
            continue
        for b in f.get("blocks", []):
            for i in b.get("instructions", []):
                op = i.get("op")
                if op == "gep":
                    gep += 1
                elif op in ("memcpy", "memset"):
                    memop += 1
                elif op == "cast" and i.get("kind") in ("int_to_ptr", "ptr_to_int"):
                    inttoptr += 1
                elif op in ("load", "store"):
                    ldst += 1
                    ops = i.get("operands", [])
                    addr = ops[0] if op == "load" else (ops[1] if len(ops) > 1 else None)
                    if addr in gids:
                        ldst_global += 1
    out.update(air_gep=gep, air_ldst=ldst, air_ldst_global_addr=ldst_global,
               air_ldst_other=ldst - ldst_global, air_memop=memop, air_inttoptr=inttoptr)
    # What a NO-FRONTEND-CHANGE AIR prover could soundly prove: every memory access
    # is a whole-object access to a global, and no pointer arithmetic exists anywhere
    # in the LLVM IR either (so no constexpr GEP was silently folded into a base id).
    out["air_provable_candidate"] = (out["ll_gep"] == 0 and gep == 0 and memop == 0
                                     and inttoptr == 0 and out["air_ldst_other"] == 0)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--yield-file", default="m2-FINAL-yield.jsonl")
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=8)
    ap.add_argument("--only-ok", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    a = ap.parse_args()

    tasks = []
    for line in open(a.yield_file):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        if a.only_ok and not d.get("ok"):
            continue
        tasks.append((d["rel_yml"], d.get("group", "?")))
    if a.limit:
        tasks = tasks[:a.limit]
    print(f"analyzing {len(tasks)} tasks with {a.workers} workers", flush=True)

    done = 0
    with open(a.out, "w") as fh, cf.ProcessPoolExecutor(a.workers) as ex:
        futs = {ex.submit(analyze, ry): (ry, g) for ry, g in tasks}
        for fut in cf.as_completed(futs):
            ry, g = futs[fut]
            try:
                r = fut.result()
            except Exception as e:                              # noqa: BLE001
                r = {"rel_yml": ry, "status": "crash", "err": str(e)[:160]}
            r["group"] = g
            fh.write(json.dumps(r) + "\n"); fh.flush()
            done += 1
            if done % 25 == 0:
                print(f"  {done}/{len(tasks)}", flush=True)

    rows = [json.loads(l) for l in open(a.out) if l.strip()]
    st = collections.Counter(r["status"] for r in rows)
    ok = [r for r in rows if r["status"] == "ok"]
    cand = [r for r in ok if r["air_provable_candidate"]]
    print("\n=== STATUS ===")
    for k, v in st.most_common():
        print(f"  {k:15s} {v}")
    print(f"\n=== OVER {len(ok)} SUCCESSFULLY INGESTED ===")
    print(f"  tasks with >=1 getelementptr in LLVM IR : {sum(1 for r in ok if r['ll_gep'])}")
    print(f"  tasks with >=1 gep op in AIR            : {sum(1 for r in ok if r['air_gep'])}")
    print(f"  tasks with a non-global ld/st address   : {sum(1 for r in ok if r['air_ldst_other'])}")
    print(f"  tasks with memcpy/memset in AIR         : {sum(1 for r in ok if r['air_memop'])}")
    print(f"  tasks with any global carrying a type   : {sum(1 for r in ok if r['n_globals_with_value_type'])}")
    print(f"  tasks with any aggregate type interned  : {sum(1 for r in ok if r['n_agg_types'])}")
    print(f"  ILP32 tasks whose AIR ptr width is 8    : "
          f"{sum(1 for r in ok if r.get('data_model','').upper()=='ILP32' and r.get('air_ptr_width')==8)}"
          f" (of {sum(1 for r in ok if r.get('data_model','').upper()=='ILP32')} ILP32)")
    print(f"\n  AIR-PROVABLE CANDIDATES                 : {len(cand)} / {len(ok)}")
    byg = collections.Counter(r["group"] for r in cand)
    print(f"  distinct clusters with >=1 candidate    : {len(byg)}")
    for g, c in byg.most_common():
        print(f"      {g:28s} {c}")


if __name__ == "__main__":
    main()
