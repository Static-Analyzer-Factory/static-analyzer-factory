#!/usr/bin/env python3
"""Slice 0c — R5 ASan-gate reservoir measurement (measurement only; NO production code).

For a stratified sample of real valid-memsafety sv-benchmarks tasks (safe + buggy,
ILP32 + LP64), compile the ORIGINAL program natively with
    clang-18 -O0 -g -fsanitize=address -fno-sanitize-recover=address {-m32|-m64}
             -include <stub> -I<srcdir> <src> <zerodriver.c>
run it with zeroed nondet inputs, and record whether AddressSanitizer reports a
violation (+ the class). Classify vs the .yml expected_verdict:
    exp=safe (True)  & ASan report -> FP  (the -16 false alarm; MUST be ~0)
    exp=safe         & no report   -> TN
    exp=buggy (False)& ASan report -> TP  (confirmed FALSE = default-nondet recall)
    exp=buggy        & no report   -> FN  (miss: guarded/needs-steering or out-of-scope)

This is the confirmer-first go/no-go evidence: near-0 FP proves ASan is a sound gate;
the TP count is the recall floor of an UNSTEERED probe; FN is the reservoir that would
justify Slice 2 (Z3-steered nondets). Class distribution validates class->sub-property.

Run (throwaway root container, ASan pkgs installed first — see the wrapper):
    docker compose run --rm -T --user root --entrypoint sh dev \\
        -c 'apt-get ... && python3 scripts/r5_asan_measure.py'
"""
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

SVB = Path("tests/benchmarks/sv-benchmarks/c")
STUB = "share/saf/stubs/sv-comp-stubs.h"
DRIVER = "scripts/r5_asan_zerodriver.c"
CLANG = os.environ.get("SAF_CLANG", "clang-18")
N = int(os.environ.get("N", "40"))          # per (class); total ~= 4*N (safe/buggy x dm)
RUN_TIMEOUT = float(os.environ.get("RUN_TIMEOUT", "10"))
# check_printf=0: SV-COMP valid-memsafety does NOT count libc printf("%s") string
# reads (they fire inside ASan's printf_common interceptor from Juliet's printLine
# output helper on non-terminated buffers) -> disabling it removes that artifact
# while keeping every heap/stack/global/UAF/double-free/memcpy check. Sound: abstain, not guess.
ASAN_OPTIONS = "exitcode=1:abort_on_error=0:detect_leaks=0:check_printf=0"
BANNER_RE = re.compile(r"ERROR: AddressSanitizer: ([A-Za-z0-9_-]+)")
# R5 is sequential (valid-deref/valid-free); concurrency memsafety is out of scope
# (plan 193 §4.5). Exclude by benchmark CATEGORY DIR — content grep is unreliable
# because preprocessed .i files inline the pthread.h prototype even when never called.
CONC_DIRS = ("/pthread", "/weaver/", "/goblint", "/ldv-races/", "/libvsync/",
             "/locks/", "/ddv-machzwd/")


def is_concurrency(src):
    return any(d in src for d in CONC_DIRS)


def class_to_subprop(klass):
    """ASan error class -> SV-COMP sub-property (or None = abstain/unknown)."""
    k = klass.lower()
    if k == "attempting":                       # 'attempting double-free' / bad free
        return "valid-free"
    if "use-after-free" in k:                   # deref of freed memory
        return "valid-deref"
    if ("buffer-overflow" in k or "buffer-underflow" in k or "buffer-overread" in k
            or "buffer-underread" in k or "use-after-scope" in k
            or "use-after-return" in k or "use-after-poison" in k or k == "segv"):
        return "valid-deref"
    return None                                 # unmapped -> confirmer would abstain

INPUT_RE = re.compile(r"input_files:\s*['\"]?([^'\"\n]+)")
DM_RE = re.compile(r"data_model:\s*'?(ILP32|LP64)")


def collect():
    tasks = []
    for yml in SVB.rglob("*.yml"):
        try:
            text = yml.read_text(errors="replace")
        except OSError:
            continue
        if "valid-memsafety" not in text:
            continue
        exp = None
        sub = None
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
        if exp is None:
            continue
        mi = INPUT_RE.search(text)
        if not mi:
            continue
        src = (yml.parent / mi.group(1).strip()).resolve()
        if not src.exists():
            continue
        md = DM_RE.search(text)
        dm = md.group(1) if md else "LP64"
        tasks.append({"src": str(src), "exp": exp, "sub": sub, "dm": dm,
                      "size": src.stat().st_size})
    return tasks


def stride(lst, n):
    lst = sorted(lst, key=lambda t: (t["size"], t["src"]))
    if len(lst) <= n:
        return lst
    step = len(lst) / n
    return [lst[int(i * step)] for i in range(n)]


def asan_run(src, dm):
    """-> ('report', class) | ('clean', None) | ('nofault', None) | ('compile_fail', None) | ('timeout', None)"""
    flag = "-m32" if dm.upper() == "ILP32" else "-m64"
    binp = "/tmp/r5_asan_h"
    srcdir = str(Path(src).parent)
    try:
        c = subprocess.run(
            [CLANG, "-O0", "-g", "-fsanitize=address", "-fno-sanitize-recover=address",
             flag, "-include", STUB, "-I", srcdir, "-Wno-everything", src, DRIVER, "-o", binp],
            capture_output=True, text=True, errors="replace", timeout=45)
    except subprocess.TimeoutExpired:
        return ("compile_fail", None)
    if c.returncode != 0:
        return ("compile_fail", None)
    env = dict(os.environ, ASAN_OPTIONS=ASAN_OPTIONS)
    try:
        r = subprocess.run([binp], capture_output=True, text=True, errors="replace",
                           timeout=RUN_TIMEOUT, env=env)
    except subprocess.TimeoutExpired:
        return ("timeout", None)
    m = BANNER_RE.search(r.stderr or "")
    if m:
        return ("report", m.group(1))
    return ("clean", None)


def main():
    tasks = collect()
    safe = [t for t in tasks if t["exp"]]
    buggy = [t for t in tasks if not t["exp"]]
    # stratify by data model so ILP32 (the majority) is represented
    sample = []
    for pool in (safe, buggy):
        for dm in ("ILP32", "LP64"):
            sample += stride([t for t in pool if t["dm"] == dm], N)
    print(f"valid-memsafety tasks total: {len(tasks)} "
          f"(safe={len(safe)} buggy={len(buggy)}); sampling {len(sample)} "
          f"(N={N} per class x data-model)\n")

    conf = Counter()
    classes = Counter()
    tp_by_dm = Counter()
    fps, misses, subdis = [], [], []
    for t in sample:
        name = Path(t["src"]).name
        if is_concurrency(t["src"]):
            conf["mt_excluded"] += 1
            print(f"  exp={'safe ' if t['exp'] else 'buggy'} {t['dm']:<5} "
                  f"{'CONC(excluded)':<24} {name}")
            continue
        outcome, klass = asan_run(t["src"], t["dm"])
        if outcome in ("compile_fail", "timeout"):
            conf[outcome] += 1
            tag = outcome
        else:
            reported = outcome == "report"
            if reported:
                classes[klass] += 1
            if t["exp"] and reported:
                conf["FP"] += 1; tag = f"FP!! {klass}"; fps.append(f"{name}[{t['dm']}]")
            elif t["exp"] and not reported:
                conf["TN"] += 1; tag = "TN"
            elif (not t["exp"]) and reported:
                mapped = class_to_subprop(klass)
                if mapped is None:
                    conf["TP_unmapped_abstain"] += 1
                    tag = f"TP? {klass}(unmapped->unknown)"
                else:
                    conf["TP"] += 1; tp_by_dm[t["dm"]] += 1
                    if t["sub"] == mapped:
                        conf["sub_agree"] += 1; tag = f"TP {mapped}==exp"
                    elif t["sub"]:
                        conf["sub_disagree"] += 1
                        subdis.append(f"{name}: asan->{mapped} exp={t['sub']}")
                        tag = f"TP {mapped}!=EXP({t['sub']})"
                    else:
                        tag = f"TP {mapped}(no-exp-sub)"
            else:
                conf["FN"] += 1; tag = "FN(miss)"; misses.append(f"{name}[{t['dm']}]")
        print(f"  exp={'safe ' if t['exp'] else 'buggy'} {t['dm']:<5} {tag:<24} {name}")

    tp, fn, fp, tn = conf["TP"], conf["FN"], conf["FP"], conf["TN"]
    print("\n=== R5 ASan-gate (unsteered / zeroed-nondet) ===")
    print(f"  TP(confirmed buggy)={tp}  FN(missed buggy)={fn}  "
          f"FP(false alarm on safe)={fp}  TN(safe clean)={tn}  "
          f"compile_fail={conf['compile_fail']}  timeout={conf['timeout']}  "
          f"mt_excluded={conf['mt_excluded']}")
    if tp + fn:
        print(f"  default-nondet RECALL floor = {tp}/{tp+fn} = {tp/(tp+fn):.0%}")
    if fp + tn:
        print(f"  FALSE-ALARM rate on safe    = {fp}/{fp+tn}  <-- MUST be ~0 for a sound gate")
    print(f"  TP by data-model: ILP32={tp_by_dm['ILP32']}  LP64={tp_by_dm['LP64']}")
    sa, sd = conf["sub_agree"], conf["sub_disagree"]
    if sa + sd:
        print(f"  SUB-PROPERTY match (asan-class vs .yml expected) = {sa}/{sa+sd} "
              f"agree  ({sd} disagree = -16 risk)   TP_unmapped_abstain={conf['TP_unmapped_abstain']}")
    print(f"  ASan class distribution (confirmed): {dict(classes)}")
    if subdis:
        print("  !!! SUB-PROPERTY DISAGREEMENTS (would emit wrong sub-property = -16):")
        for s in subdis:
            print("      " + s)
    if fps:
        print("  !!! FALSE ALARMS (investigate — harness artifact or mislabeled):", ", ".join(fps))
    if misses:
        print(f"  sample misses (Slice-2 steering candidates), first 15: {', '.join(misses[:15])}")


if __name__ == "__main__":
    main()
