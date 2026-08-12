#!/usr/bin/env python3
"""Slice 2 de-risk (plan 197) — measure the scalar-nondet-steerable miss reservoir.

Before building a Z3 steering engine (the R4 lesson: a common shape can still catch
~0 if the required reasoning is out of scope), measure whether steering scalar
nondet inputs would catch a MATERIAL fraction of the buggy valid-memsafety tasks the
Slice-1 unsteered probe misses.

Method: compile each buggy task ONCE with the const-driver (-fsanitize=address), then
run it under a spread of constants (a mini-fuzz — one binary, many runs via
$SAF_NONDET_CONST). Classify:
  caught@0            -> already confirmed by Slice 1 (unsteered).
  minifuzz@nonzero    -> SCALAR-STEERABLE (a constant != 0 traps) = Slice-2 addressable
                         (a LOWER bound; Z3 could solve for specific values a fuzz misses).
  never-caught        -> sub-classify: leak (valid-memtrack, out of scope), no-scalar-nondet
                         (unsteerable by scalar), or has-scalar-nondet (maybe Z3-steerable /
                         maybe needs non-scalar input or loops/arrays).

GO (build Z3 steering) iff minifuzz-steerable + has-scalar-nondet-and-loopfree is a
material fraction; else NO-GO (report, like R4).

Run (throwaway root container w/ ASan pkgs; workspace root):
    docker compose run --rm -T --user root --entrypoint sh dev /workspace/scripts/r5_slice2_derisk_run.sh
"""
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
STUB = "share/saf/stubs/sv-comp-stubs.h"
DRIVER = "scripts/r5_asan_constdriver.c"
CLANG = os.environ.get("SAF_CLANG", "clang-18")
N = int(os.environ.get("N", "60"))
RUN_TIMEOUT = float(os.environ.get("RUN_TIMEOUT", "4"))
ASAN_OPTIONS = "exitcode=1:abort_on_error=0:detect_leaks=0:check_printf=0"
# check_printf=0 here is fine: the de-risk only asks "does SOME constant trap a
# program memory fault"; the printf artifact would only inflate, and we classify
# by whether ANY constant beats 0, so a uniform printf suppression is neutral.
CONSTS = [0, 1, 2, 8, 42, 255, 256, 1024, 65535, 2147483647, -1]
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/libvsync/",
             "/locks/", "/ddv-machzwd/")
SCALAR_NONDET_RE = re.compile(
    r"__VERIFIER_nondet_(int|uint|long|ulong|longlong|ulonglong|short|ushort|"
    r"char|uchar|bool|size_t|unsigned|U8|U16|U32|S8|S16|S32|loff_t)\b")
LOOP_RE = re.compile(r"\b(for|while|goto)\b")
INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")
BANNER = "ERROR: AddressSanitizer"


def collect_buggy():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        exp, sub = None, None
        lines = text.splitlines()
        for i, ln in enumerate(lines):
            if "valid-memsafety" in ln and "property_file" in ln:
                for j in range(i, min(i + 5, len(lines))):
                    m = re.search(r"expected_verdict:\s*(true|false)", lines[j])
                    if m:
                        exp = m.group(1) == "true"
                    ms = re.search(r"subproperty:\s*(valid-[a-z]+)", lines[j])
                    if ms:
                        sub = ms.group(1)
                break
        if exp is not False:  # buggy only
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists() or any(d in str(src) for d in CONC_DIRS):
            continue
        if os.environ.get("SEQ_ONLY") == "1":
            try:
                b = src.read_text(errors="replace")
            except OSError:
                continue
            if b.count("pthread_create") > 1 or b.count("thrd_create") > 1:
                continue
        md = DM_RE.search(text)
        tasks.append({"src": str(src), "sub": sub,
                      "dm": md.group(1) if md else "LP64", "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def minifuzz(src, dm):
    """-> ('caught', const, klass) | ('nofault', None, None) | ('compile_fail', ...)."""
    flag = "-m32" if dm.upper() == "ILP32" else "-m64"
    binp = "/tmp/r5_cf"
    srcdir = str(Path(src).parent)
    try:
        c = subprocess.run(
            [CLANG, "-O0", "-g", "-fsanitize=address", "-fno-sanitize-recover=address",
             flag, "-include", STUB, "-I", srcdir, "-Wno-everything", src, DRIVER, "-o", binp],
            capture_output=True, text=True, errors="replace", timeout=45)
    except subprocess.TimeoutExpired:
        return ("compile_fail", None, None)
    if c.returncode != 0:
        return ("compile_fail", None, None)
    for k in CONSTS:
        env = dict(os.environ, ASAN_OPTIONS=ASAN_OPTIONS, SAF_NONDET_CONST=str(k))
        try:
            r = subprocess.run([binp], capture_output=True, text=True, errors="replace",
                               timeout=RUN_TIMEOUT, env=env)
        except subprocess.TimeoutExpired:
            continue
        if BANNER in (r.stderr or ""):
            m = re.search(r"AddressSanitizer: ([A-Za-z0-9_-]+)", r.stderr)
            return ("caught", k, m.group(1) if m else "?")
    return ("nofault", None, None)


def main():
    tasks = stride(collect_buggy(), N)
    print(f"sampling {len(tasks)} buggy sequential valid-memsafety tasks "
          f"(consts={CONSTS})\n")
    conf = Counter()
    steerable, unsteer_scalar_loopfree = [], []
    for t in tasks:
        name = Path(t["src"]).name
        outcome, k, klass = minifuzz(t["src"], t["dm"])
        try:
            body = Path(t["src"]).read_text(errors="replace")
        except OSError:
            body = ""
        has_scalar = bool(SCALAR_NONDET_RE.search(body))
        has_loop = bool(LOOP_RE.search(body))
        if outcome == "compile_fail":
            conf["compile_fail"] += 1; tag = "compile_fail"
        elif outcome == "caught" and k == 0:
            conf["caught0"] += 1; tag = f"caught@0 {klass}"
        elif outcome == "caught":
            conf["minifuzz_steerable"] += 1
            steerable.append(f"{name}[{t['dm']}]@{k}:{klass}")
            tag = f"STEERABLE@{k} {klass}"
        else:  # nofault under all consts
            if t["sub"] == "valid-memtrack":
                conf["never_leak"] += 1; tag = "never(leak)"
            elif not has_scalar:
                conf["never_no_scalar"] += 1; tag = "never(no-scalar-nondet)"
            elif not has_loop:
                conf["never_scalar_loopfree"] += 1
                unsteer_scalar_loopfree.append(f"{name}[{t['dm']}]")
                tag = "never(scalar,loop-free = Z3-maybe)"
            else:
                conf["never_scalar_loops"] += 1; tag = "never(scalar+loops/arrays)"
        print(f"  {t['dm']:<5} {tag:<34} {name}")

    print("\n=== Slice 2 de-risk: scalar-nondet steerability of buggy tasks ===")
    for klabel in ("caught0", "minifuzz_steerable", "never_leak", "never_no_scalar",
                   "never_scalar_loopfree", "never_scalar_loops", "compile_fail"):
        print(f"  {klabel:<24} = {conf[klabel]}")
    total = sum(conf.values())
    ms = conf["minifuzz_steerable"]
    z3maybe = conf["never_scalar_loopfree"]
    print(f"\n  MINI-FUZZ STEERABLE (lower bound of Slice-2 recall) = {ms}/{total} = {ms/total:.0%}")
    print(f"  + never-caught-but-scalar-&-loop-free (Z3-maybe upper margin) = {z3maybe}")
    print(f"  => Slice-2 addressable window ~ {ms}-{ms+z3maybe} of {total}")
    if steerable:
        print("  steerable examples:", ", ".join(steerable[:12]))
    if unsteer_scalar_loopfree:
        print("  Z3-maybe examples:", ", ".join(unsteer_scalar_loopfree[:12]))


if __name__ == "__main__":
    main()
