#!/bin/sh
#
# SAF -- smoke test for the SV-COMP submission archive.
#
# fm-weck executes this as `subprocess.run(["./smoketest.sh"], cwd=<tool dir>)`
# (fm_weck/smoke_test_mode.py, functions `locate_and_check_smoke_test_file` and
# `run_smoke_test_gitlab_ci`), so the file must
#   (a) exist at the archive root under exactly this name,
#   (b) be non-empty,
#   (c) have the *other*-execute bit set -- fm-weck tests `st_mode & os.X_OK`,
#       and os.X_OK is 0o001, so mode 0700 would be rejected and 0755 is used,
#   (d) exit 0 on success and non-zero on any failure.
#
# It is deliberately POSIX sh, not bash: the run container is only guaranteed to
# provide /bin/sh. Consequently there is no `set -o pipefail`, and no assertion
# below depends on the exit status of the left-hand side of a pipeline.
#
# What it proves, in order:
#   1. bin/saf exists, is executable, and its dynamic loader dependencies
#      resolve (`--version` is the cheapest call that touches every .so).
#   2. share/saf/stubs/sv-comp-stubs.h is where SAF's `resolve_svcomp_stub()`
#      looks for it -- it walks the ancestors of `current_exe()` joined with
#      "share/saf/stubs/sv-comp-stubs.h", so <root>/bin/saf resolves
#      <root>/share/saf/stubs/sv-comp-stubs.h. Flattening the archive would
#      make SAF answer `unknown` on every task, silently, so check 3 below is
#      the load-bearing end-to-end assertion for the archive layout.
#   3. A nondet-guarded reachable error yields exactly `false(unreach-call)`
#      and a non-empty YAML-2.0 violation witness. Reaching that verdict needs
#      clang-18/opt-18 on PATH (or $SAF_CLANG/$SAF_OPT) plus the linked-in Z3;
#      with any of them missing SAF degrades to `unknown` and this fails.
#   4. An undecidable task yields exactly `unknown` and writes NO witness.
#
# SPDX-License-Identifier: MIT

set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

SAF="./bin/saf"
STUB="./share/saf/stubs/sv-comp-stubs.h"
PRP="./smoketest/unreach-call.prp"

# SAF's own graceful budget; on expiry it prints `unknown` and exits 0.
SAF_TIMEOUT=10
# Outer hard bound per invocation, so a wedged process cannot hang CI.
HARD_TIMEOUT=12

# Measured on the reference machine: 0.4 s for check 3 and 1.2 s for check 4,
# so the bounds above leave >25x headroom and the whole script stays well
# under the ~30 s a smoke test is allowed.
if command -v timeout >/dev/null 2>&1; then
    TIMEOUT="timeout $HARD_TIMEOUT"
else
    TIMEOUT=""
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/saf-smoketest.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT
trap 'rm -rf "$WORK"; exit 130' INT
trap 'rm -rf "$WORK"; exit 143' TERM

note() {
    printf 'smoketest: %s\n' "$*"
}

fail() {
    printf 'smoketest: FAIL: %s\n' "$*" >&2
    exit 1
}

# Echo a captured stderr file, indented, so a failure is diagnosable from the
# CI log alone.
dump_err() {
    if [ -s "$1" ]; then
        printf 'smoketest: --- stderr of the failing run ---\n' >&2
        sed 's/^/smoketest:   /' "$1" >&2
        printf 'smoketest: ------------------------------------\n' >&2
    fi
}

# ---------------------------------------------------------------------------
# 1. Dependency check: the binary runs at all.
# ---------------------------------------------------------------------------

[ -f "$SAF" ] || fail "$SAF not found -- the archive must keep the <root>/bin/saf layout"
[ -x "$SAF" ] || fail "$SAF is not executable -- the zip lost its Unix mode bits"

if ! version="$("$SAF" --version 2>"$WORK/version.err")"; then
    dump_err "$WORK/version.err"
    fail "'$SAF --version' exited non-zero (unresolved shared library? wrong architecture?)"
fi
[ -n "$version" ] || fail "'$SAF --version' printed nothing"
case "$version" in
*"
"*)
    fail "'$SAF --version' printed more than one line: $version"
    ;;
esac
# fm-tools ci/check_archive.py rejects the archive when the version string is
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
note "version: $version"

# ---------------------------------------------------------------------------
# 2. Data files shipped alongside the binary.
# ---------------------------------------------------------------------------

[ -f "$STUB" ] || fail "$STUB not found -- SAF resolves its SV-COMP stub header relative to the binary and would answer 'unknown' on every task"
[ -d ./share/saf/specs ] || fail "./share/saf/specs not found -- the whole share/ tree must ship"
[ -f "$PRP" ] || fail "$PRP not found"

# ---------------------------------------------------------------------------
# 3. A reachable error must be refuted: exactly `false(unreach-call)` + witness.
# ---------------------------------------------------------------------------

witness="$WORK/witness.yml"
note "running: $SAF verify --property $PRP ./smoketest/smoke_false_unreach.c"
# shellcheck disable=SC2086  # $TIMEOUT is an intentional word-split prefix
if ! verdict="$($TIMEOUT "$SAF" verify \
    --property "$PRP" \
    --data-model LP64 \
    --timeout "$SAF_TIMEOUT" \
    --witness "$witness" \
    ./smoketest/smoke_false_unreach.c 2>"$WORK/false.err")"; then
    dump_err "$WORK/false.err"
    fail "'$SAF verify' exited non-zero on smoke_false_unreach.c (it must always exit 0)"
fi

if [ "$verdict" != "false(unreach-call)" ]; then
    dump_err "$WORK/false.err"
    fail "smoke_false_unreach.c: expected 'false(unreach-call)', got '$verdict'"
fi
note "verdict: $verdict"

[ -s "$witness" ] || fail "smoke_false_unreach.c: no violation witness written to $witness (a FALSE without a confirmed witness scores 0)"
grep -q 'entry_type: violation_sequence' "$witness" ||
    fail "smoke_false_unreach.c: $witness is not a YAML violation-sequence witness"
note "witness: $(wc -c <"$witness") bytes, entry_type: violation_sequence"

# ---------------------------------------------------------------------------
# 4. An undecidable task must abstain: exactly `unknown`, and no witness.
# ---------------------------------------------------------------------------

witness_unknown="$WORK/witness-unknown.yml"
note "running: $SAF verify --property $PRP ./smoketest/smoke_unknown_unreach.c"
# shellcheck disable=SC2086  # $TIMEOUT is an intentional word-split prefix
if ! verdict="$($TIMEOUT "$SAF" verify \
    --property "$PRP" \
    --data-model LP64 \
    --timeout "$SAF_TIMEOUT" \
    --witness "$witness_unknown" \
    ./smoketest/smoke_unknown_unreach.c 2>"$WORK/unknown.err")"; then
    dump_err "$WORK/unknown.err"
    fail "'$SAF verify' exited non-zero on smoke_unknown_unreach.c (it must always exit 0)"
fi

if [ "$verdict" != "unknown" ]; then
    dump_err "$WORK/unknown.err"
    fail "smoke_unknown_unreach.c: expected 'unknown', got '$verdict'"
fi
note "verdict: $verdict"

[ ! -e "$witness_unknown" ] ||
    fail "smoke_unknown_unreach.c: SAF wrote a witness for an 'unknown' verdict"

note "PASS (4 checks)"
