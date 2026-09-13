#!/usr/bin/env python3
"""Realign smoketest.sh's --version assertion to the actual SV-COMP requirement.

I over-asserted. `make svcomp-archive` caught it, which is the gate working, but
the assertion is in the wrong place:

fm-tools ci/check_archive.py reads the version from the BenchExec tool-info
module's version(), not from the binary. benchexec/tools/saf.py strips clap's
leading "saf " token, so the module reports "0.1.0 (LLVM 18.1)" and every
check_archive rule passes. Asserting digit-first on the BINARY's raw output is
stricter than SV-COMP requires and would refuse a conforming archive.

The digit-first property is pinned where it is actually read: test_saf.py now
parametrises over the exact string the binary prints today and asserts
`not version.lower().startswith(tool.name().lower())`.

Kept here: non-empty, single line, and <= 100 characters -- all three are real
check_archive.py rules and all three are properties of the raw output.
"""
import sys
from pathlib import Path

SMOKE = Path("smoketest/smoketest.sh")

OLD = '''# fm-tools ci/check_archive.py rejects the archive when the version string is
# empty, spans lines, exceeds 100 characters, or starts with the name the
# BenchExec tool-info module reports. That last one currently passes only
# because the comparison is case-sensitive ("SAF" vs "saf"), so pin the
# stronger property here -- this is the cheapest place to catch a regression.
if [ "${#version}" -gt 100 ]; then
    fail "'$SAF --version' printed ${#version} characters; check_archive.py caps it at 100"
fi
case "$version" in
[0-9]*) ;;
*) fail "'$SAF --version' must start with a digit, got: $version" ;;
esac
'''

NEW = '''# fm-tools ci/check_archive.py rejects the archive when the version string is
# empty, spans lines or exceeds 100 characters. The first two are checked above;
# this is the third.
#
# It ALSO rejects a version that starts with the name the tool-info module
# reports -- but that check reads the MODULE's version(), not this binary's raw
# output. clap prints "saf 0.1.0 (LLVM 18.1)" and benchexec/tools/saf.py strips
# the leading tool-name token before handing it to fm-tools. So the digit-first
# property is pinned in benchexec/tools/test_saf.py, parametrised over the exact
# string this binary prints. Asserting it here would be stricter than SV-COMP
# requires and would refuse a conforming archive.
if [ "${#version}" -gt 100 ]; then
    fail "'$SAF --version' printed ${#version} characters; check_archive.py caps it at 100"
fi
'''


def main():
    s = SMOKE.read_text()
    if "would refuse a conforming archive" in s:
        sys.exit("already applied")
    n = s.count(OLD)
    if n != 1:
        sys.exit(f"pattern found {n} times (want 1) -- not patching")
    SMOKE.write_text(s.replace(OLD, NEW))
    remaining = SMOKE.read_text().count("start with a digit")
    print(f"patched; 'start with a digit' occurrences now: {remaining}")
    if remaining:
        sys.exit("assertion still present -- patch did not take")


if __name__ == "__main__":
    main()
