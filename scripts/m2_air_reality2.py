#!/usr/bin/env python3
"""What can an AIR-based Anchored-Object prover discharge, with no frontend change?

Established empirically (probes g1/g3/safe/unsafe) about AIR at 5f274d13:
  * AirGlobal.value_type is never set by the LLVM frontend -> a global has NO size.
  * Aggregate types are never interned -> no size is recoverable from the type table.
  * Operation::Gep carries only FieldPath{Index | Field{i}} -- no element type, no
    byte offset -- so a constant index cannot be turned into a byte offset.
  * A constant-expression GEP over a non-aggregate-initialised global is FLATTENED to
    the bare base ValueId, and even when decomposed the pointer-level first index is
    DROPPED (mapping.rs:345-359, 424-437).
Therefore: obligation 3 (offset + width <= size_of(base)) is discharge-able only on
programs that contain NO pointer arithmetic at all. This measures that population,
directly from the LLVM IR the production recipe produces -- no `saf index` needed
(its --output path trips a serde bug on Constant::GlobalRef).
"""
import argparse, collections, concurrent.futures as cf, json, os, re, subprocess, tempfile

BENCH = "tests/benchmarks/sv-benchmarks/c"
STUB = "share/saf/stubs/sv-comp-stubs.h"
CLANG = os.environ.get("SAF_CLANG", "clang-18")
OPT = os.environ.get("SAF_OPT", "opt-18")

GLOBAL_DECL_RE = re.compile(r"^@([\w.$]+)\s*=\s*(.*)$", re.M)


def _type_prefix(s):
    """Longest balanced LLVM type prefix of `s` (stops at a depth-0 space/comma)."""
    depth = 0
    for i, ch in enumerate(s):
        if ch in "[{<":
            depth += 1
        elif ch in "]}>":
            depth -= 1
        elif depth == 0 and ch in " ,":
            return s[:i]
    return s


def global_types(txt):
    """name -> declared LLVM type, parsed past the linkage/attribute words."""
    out = {}
    for m in GLOBAL_DECL_RE.finditer(txt):
        rest = m.group(2)
        kw = re.search(r"\b(?:global|constant)\s+", rest)
        if not kw:
            continue
        out[m.group(1)] = _type_prefix(rest[kw.end():].strip())
    return out
STORE_RE = re.compile(r"^\s*store\s+(?:atomic\s+|volatile\s+)*([^,]+?)\s+[^,]+,\s*ptr\s+@([\w.$]+)", re.M)
LOAD_RE = re.compile(r"^\s*%\S+\s*=\s*load\s+(?:atomic\s+|volatile\s+)*([^,]+),\s*ptr\s+@([\w.$]+)", re.M)


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
        ll, ll2 = os.path.join(d, "a.ll"), os.path.join(d, "b.ll")
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
            txt = open(ll2, errors="ignore").read()
        except subprocess.TimeoutExpired:
            out["status"] = "timeout"; return out
        except Exception as e:                                  # noqa: BLE001
            out["status"] = "error"; out["err"] = str(e)[:160]; return out

    out["status"] = "ok"
    out["n_gep"] = txt.count("getelementptr")
    out["n_alloca"] = txt.count(" alloca ")
    out["n_inttoptr"] = txt.count("inttoptr") + txt.count("ptrtoint")
    out["n_memintrin"] = len(re.findall(r"llvm\.mem(cpy|set|move)", txt))

    gtypes = global_types(txt)
    out["n_globals"] = len(gtypes)
    # Direct (non-GEP) accesses to a global: is the accessed type the global's own
    # declared type? If yes the access is a whole-object access at the natural width
    # and needs no size arithmetic; if no, a size is genuinely required.
    mismatch = whole = 0
    samples = []
    for rx in (STORE_RE, LOAD_RE):
        for m in rx.finditer(txt):
            ty, name = m.group(1).strip(), m.group(2)
            decl = gtypes.get(name)
            if decl is None:
                continue
            if decl == ty:
                whole += 1
            else:
                mismatch += 1
                if len(samples) < 4:
                    samples.append(f"{name}: decl={decl!r} access={ty!r}")
    out["direct_whole_obj_access"] = whole
    out["direct_width_mismatch"] = mismatch
    out["mismatch_samples"] = samples
    # TIER 1: no pointer arithmetic anywhere -> AIR loses no offset information.
    out["tier1_no_ptr_arith"] = (out["n_gep"] == 0 and out["n_inttoptr"] == 0
                                 and out["n_memintrin"] == 0)
    # TIER 2: additionally every direct global access is at the global's own width,
    # so obligation 3 holds WITHOUT needing size_of(global) at all.
    out["tier2_no_size_needed"] = out["tier1_no_ptr_arith"] and mismatch == 0
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--yield-file", default="m2-FINAL-yield.jsonl")
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=12)
    ap.add_argument("--only-ok", action="store_true")
    ap.add_argument("--field", default="ok")
    a = ap.parse_args()

    tasks = []
    for line in open(a.yield_file):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        if a.only_ok and not d.get(a.field):
            continue
        tasks.append((d["rel_yml"], d.get("group", "?")))
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
            if done % 50 == 0:
                print(f"  {done}/{len(tasks)}", flush=True)

    rows = [json.loads(l) for l in open(a.out) if l.strip()]
    print("\n=== STATUS ===")
    for k, v in collections.Counter(r["status"] for r in rows).most_common():
        print(f"  {k:15s} {v}")
    ok = [r for r in rows if r["status"] == "ok"]
    t1 = [r for r in ok if r["tier1_no_ptr_arith"]]
    t2 = [r for r in ok if r["tier2_no_size_needed"]]
    print(f"\n=== OVER {len(ok)} COMPILED ===")
    print(f"  have >=1 getelementptr                 : {sum(1 for r in ok if r['n_gep'])}")
    print(f"  have >=1 inttoptr/ptrtoint             : {sum(1 for r in ok if r['n_inttoptr'])}")
    print(f"  have >=1 llvm.mem* intrinsic           : {sum(1 for r in ok if r['n_memintrin'])}")
    print(f"  have a direct access at a NON-natural width : {sum(1 for r in ok if r['direct_width_mismatch'])}")
    for tier, rs in (("TIER 1 (no pointer arithmetic)", t1), ("TIER 2 (also no size needed)", t2)):
        byg = collections.Counter(r["group"] for r in rs)
        print(f"\n  {tier}: {len(rs)} tasks, {len(byg)} clusters")
        for g, c in byg.most_common():
            print(f"      {g:28s} {c}")


if __name__ == "__main__":
    main()
