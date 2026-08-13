#!/usr/bin/env python3
"""Slice 0e (sweep) — R6 no-overflow witness CONFIRMED-% over the buggy reservoir
(measurement only; NO production code).

For a stratified sample of BUGGY no-overflow tasks (the ones SAF would emit
false(no-overflow) on): compile the ORIGINAL with UBSan signed-overflow + const-driver,
mini-fuzz to the first trapping constant, capture file:line:col + the overflowing TYPE,
generate a target-only (line+col) YAML-2.0 witness (r6_make_witness.py), and run
validate_witness.sh (witnesslint + CPAchecker analysis + cpa-witness2test). Reports the
CONFIRMED-% — the metric that actually SCORES (a sound false(no-overflow) with an
unconfirmed witness scores 0, like the unreach/memsafety confirmed-% story). Stratifies
CONFIRMED vs NOT_CONFIRMED by overflowing int TYPE ('int' vs 'int64_t/long') to test the
hypothesis that 64-bit overflows confirm worse.

Run (dev container; CPAchecker already provisioned at .svtools/):
    docker compose run --rm -T --entrypoint sh dev -c 'python3 scripts/r6_witness_sweep.py'
Env: N (buggy tasks per data-model, default 12), RUN_TIMEOUT (per const, default 8).
"""
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
STUB = "share/saf/stubs/sv-comp-stubs.h"
DRIVER = "scripts/r5_asan_constdriver.c"
PRP = "tests/benchmarks/sv-benchmarks/c/properties/no-overflow.prp"
CLANG = os.environ.get("SAF_CLANG", "clang-18")
N = int(os.environ.get("N", "12"))
RUN_TIMEOUT = float(os.environ.get("RUN_TIMEOUT", "8"))
UBSAN_OPTIONS = "halt_on_error=1:abort_on_error=0:print_stacktrace=1"
CONSTS = [0, 1, 2, 42, 255, 256, 1024, 65535, 2147483647, -1, -2147483648, 2147483648]
OVF_RE = re.compile(r"runtime error: (signed integer overflow|negation of|division of)")
LOC_RE = re.compile(r"^([^ :]+):(\d+):(\d+): runtime error:")
TYPE_RE = re.compile(r"cannot be represented in type '([^']+)'")
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/ldv-linux-3.14-races/",
             "/libvsync/", "/locks/", "/ddv-machzwd/")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")


def collect_buggy():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "no-overflow" not in text:
            continue
        exp = None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "no-overflow" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                break
        if exp is not False:            # keep only buggy (expected false)
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists() or any(d in str(src) for d in CONC_DIRS):
            continue
        md = DM_RE.search(text)
        tasks.append({"src": str(src), "dm": md.group(1) if md else "LP64",
                      "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def ubsan_locate(src, dm):
    flag = "-m32" if dm == "ILP32" else "-m64"
    binp = "/tmp/r6_sweep_h"
    srcdir = str(Path(src).parent)
    try:
        c = subprocess.run(
            [CLANG, "-O0", "-g", "-fsanitize=signed-integer-overflow",
             "-fno-sanitize-recover=signed-integer-overflow", flag, "-include", STUB,
             "-I", srcdir, "-Wno-everything", src, DRIVER, "-o", binp],
            capture_output=True, text=True, errors="replace", timeout=60)
    except subprocess.TimeoutExpired:
        return None
    if c.returncode != 0:
        return None
    for k in CONSTS:
        env = dict(os.environ, UBSAN_OPTIONS=UBSAN_OPTIONS, SAF_NONDET_CONST=str(k))
        try:
            r = subprocess.run([binp], capture_output=True, text=True, errors="replace",
                               timeout=RUN_TIMEOUT, env=env)
        except subprocess.TimeoutExpired:
            continue
        err = r.stderr or ""
        if not OVF_RE.search(err):
            continue
        for line in err.splitlines():
            m = LOC_RE.match(line.strip())
            if m:
                ty = TYPE_RE.search(line)
                return {"line": int(m.group(2)), "col": int(m.group(3)),
                        "type": ty.group(1) if ty else "?", "const": k}
    return None


def validate(src, line, col, dm):
    wit = "/tmp/r6_sweep_w.yml"
    with open(wit, "w") as f:
        subprocess.run(["python3", "scripts/r6_make_witness.py", src, str(line), str(col), dm, PRP],
                       stdout=f, check=True)
    try:
        out = subprocess.run(["bash", "scripts/validate_witness.sh", wit, src, PRP, dm],
                             capture_output=True, text=True, errors="replace", timeout=300).stdout
    except subprocess.TimeoutExpired:
        return "TIMEOUT"
    if "LINT_FAIL" in out:
        return "LINT_FAIL"
    if "CONFIRMED (cpachecker-analysis)" in out:
        return "CONFIRMED_analysis"
    if "CONFIRMED (witness2test-execution)" in out:
        return "CONFIRMED_w2t"
    if "CPACHECKER_ABSENT" in out:
        return "CPA_ABSENT"
    return "NOT_CONFIRMED"


def main():
    tasks = collect_buggy()
    sample = []
    for dm in ("ILP32", "LP64"):
        sample += stride([t for t in tasks if t["dm"] == dm], N)
    print(f"buggy no-overflow tasks (excl conc): {len(tasks)}; sampling {len(sample)} for witness sweep\n")

    conf = Counter()
    by_type = Counter()      # (result_bucket, int-width)
    for t in sample:
        name = Path(t["src"]).name
        loc = ubsan_locate(t["src"], t["dm"])
        if not loc:
            conf["no_trap_or_compile_fail"] += 1
            print(f"  {t['dm']:<5} ---(no UBSan trap / compile_fail)     {name}")
            continue
        res = validate(t["src"], loc["line"], loc["col"], t["dm"])
        conf[res] += 1
        width = "64bit" if ("64" in loc["type"] or loc["type"] in ("long", "long long")) else "32bit"
        bucket = "CONFIRMED" if res.startswith("CONFIRMED") else res
        by_type[(bucket, width)] += 1
        print(f"  {t['dm']:<5} {res:<20} type={loc['type']:<16} @const={loc['const']:<11} {name}")

    confirmed = conf["CONFIRMED_analysis"] + conf["CONFIRMED_w2t"]
    validated = confirmed + conf["NOT_CONFIRMED"] + conf["TIMEOUT"] + conf["LINT_FAIL"]
    print(f"\n=== R6 no-overflow witness CONFIRMED-% ===")
    print(f"  CONFIRMED={confirmed} (analysis={conf['CONFIRMED_analysis']} "
          f"w2t={conf['CONFIRMED_w2t']})  NOT_CONFIRMED={conf['NOT_CONFIRMED']}  "
          f"TIMEOUT={conf['TIMEOUT']}  LINT_FAIL={conf['LINT_FAIL']}")
    print(f"  no_trap/compile_fail (excluded from %): {conf['no_trap_or_compile_fail']}  "
          f"CPA_absent={conf['CPA_ABSENT']}")
    if validated:
        print(f"  CONFIRMED-% (of tasks with an emitted witness) = {confirmed}/{validated} = "
              f"{confirmed/validated:.0%}")
    print(f"  by int-width: {dict(by_type)}")


if __name__ == "__main__":
    main()
