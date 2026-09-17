#!/usr/bin/env python3
"""How much does interning a global's type perturb `build_obj_type_map`?

`convert_global` now sets `AirGlobal.value_type`, which interns that type into
`AirModule.types`. `absint::build_obj_type_map` builds a size -> struct-type index
from that table and maps an alloca to a type ONLY when exactly one struct shares
its size -- so a NEW struct entry can also DROP an existing mapping. That is a
precision change, not an additive one, and it needs measuring rather than assuming.

A named struct prints as `%struct.S` and `parse_llvm_type_string` maps it to
`Opaque`, contributing nothing. Only an ANONYMOUS struct global (`{ i32, ptr }`,
common in CIL output) can add a `Struct` entry. This counts those.
"""
import collections, json, os, random, re, subprocess, sys, tempfile
from concurrent.futures import ProcessPoolExecutor

BENCH = "tests/benchmarks/sv-benchmarks"
STUB = "share/saf/stubs/sv-comp-stubs.h"
CLANG, OPT = "clang-18", "opt-18"
# `@name = <linkage words> global <TYPE> ...`; we only care whether TYPE starts
# with `{` or `<{` (an anonymous, possibly packed, struct literal).
GLOBAL_RE = re.compile(r"^@[\w.$]+\s*=\s*(.*)$", re.M)


def anon_struct_globals(txt):
    n = 0
    for m in GLOBAL_RE.finditer(txt):
        rest = m.group(1)
        kw = re.search(r"\b(?:global|constant)\s+", rest)
        if not kw:
            continue
        ty = rest[kw.end():].lstrip()
        if ty.startswith("{") or ty.startswith("<{"):
            n += 1
    return n


def analyze(row):
    src = row["src"].replace("/workspace/", "")
    dm = row.get("data_model", "ILP32")
    with tempfile.TemporaryDirectory() as d:
        ll, ll2 = os.path.join(d, "a.ll"), os.path.join(d, "b.ll")
        cc = [CLANG, "-g", "-S", "-emit-llvm", "-O0", "-Xclang", "-disable-O0-optnone",
              "-Wno-everything", "-m32" if dm.upper() == "ILP32" else "-m64",
              "-include", STUB, "-I", os.path.dirname(src), src, "-o", ll]
        try:
            if subprocess.run(cc, capture_output=True, timeout=180).returncode:
                return None
            if subprocess.run([OPT, "-S", "-passes=mem2reg", ll, "-o", ll2],
                              capture_output=True, timeout=180).returncode:
                return None
            txt = open(ll2, errors="ignore").read()
        except Exception:                                       # noqa: BLE001
            return None
    return {"group": row["group"], "property": row["property"],
            "anon": anon_struct_globals(txt)}


def main():
    prop = sys.argv[1]
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 300
    workers = int(sys.argv[3]) if len(sys.argv) > 3 else 4
    rows = [json.loads(l) for l in open("splits/train.jsonl")]
    rows = [r for r in rows if r["property"] == prop]
    random.Random(0).shuffle(rows)
    rows = rows[:n]
    print(f"{prop}: sampling {len(rows)} of the population, {workers} workers", flush=True)

    res = [r for r in ProcessPoolExecutor(workers).map(analyze, rows) if r]
    with_anon = [r for r in res if r["anon"]]
    print(f"  compiled           : {len(res)}")
    print(f"  with >=1 ANONYMOUS struct global (can perturb build_obj_type_map): "
          f"{len(with_anon)}  ({100.0*len(with_anon)/max(len(res),1):.2f}%)")
    if with_anon:
        for g, c in collections.Counter(r["group"] for r in with_anon).most_common(10):
            print(f"      {g:30s} {c}")


if __name__ == "__main__":
    main()
