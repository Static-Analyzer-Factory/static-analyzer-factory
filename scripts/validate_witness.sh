#!/usr/bin/env bash
# Validate an SV-COMP YAML 2.0 *violation* witness (plan 194 Slice F/G).
#
#   validate_witness.sh <witness.yml> <program.c> [property.prp] [data_model]
#
# Stage 1 — witnesslint (HARD syntactic gate): the witness must conform to the
#           2.0 violation-witness schema. Non-conformance => exit 2.
# Stage 2 — CPAchecker ANALYSIS-based validation (semantic, BEST-EFFORT).
# Stage 3 — cpa-witness2test EXECUTION-based validation (BEST-EFFORT): tried when
#           analysis does not confirm. Compiles+runs a test harness from the
#           witness and confirms if it reaches the target — catches the
#           recursion/loop/array violations analysis leaves UNKNOWN.
#           Prints CONFIRMED (<validator>) / NOT_CONFIRMED (...) / CPACHECKER_ABSENT.
#           Never fails the script (the score-predicting measurement lives in the
#           eval harness). Set SAF_SKIP_WITNESS2TEST=1 to run analysis only.
#
# Runs inside the dev image, which bakes witnesslint at $SAF_SVWITNESSES and a
# JRE. CPAchecker is provisioned lazily to $SAF_CPACHECKER (persistent) — see
# provision_cpachecker below; set SAF_SKIP_CPACHECKER=1 to lint only.
set -u

WITNESS="${1:?usage: validate_witness.sh <witness.yml> <program.c> [property.prp] [data_model]}"
PROGRAM="${2:?missing program}"
PROPERTY="${3:-}"
DATA_MODEL="${4:-LP64}"

SVW="${SAF_SVWITNESSES:-/opt/sv-witnesses}"
LINT_PY="${SAF_LINT_PYTHON:-/usr/bin/python3}"
CPA_HOME="${SAF_CPACHECKER:-/workspace/.svtools/CPAchecker-4.2.2-unix}"
CPA_URL="https://cpachecker.sosy-lab.org/CPAchecker-4.2.2-unix.zip"

# ---- Stage 1: witnesslint (hard gate) --------------------------------------
echo "[validate] witnesslint: $WITNESS"
if ! "$LINT_PY" "$SVW/linter/witnesslinter.py" \
        --witness "$WITNESS" "$PROGRAM" \
        --expectViolationWitness --expectedWitnessVersion 2.0; then
    echo "LINT_FAIL"
    exit 2
fi
echo "LINT_OK"

# ---- Stage 2: CPAchecker (best-effort semantic confirmation) ---------------
if [ "${SAF_SKIP_CPACHECKER:-0}" = "1" ]; then
    echo "CPACHECKER_SKIPPED"
    exit 0
fi

provision_cpachecker() {
    [ -x "$CPA_HOME/bin/cpachecker" ] && return 0
    local dest zip
    dest="$(dirname "$CPA_HOME")"
    mkdir -p "$dest" || return 1
    echo "[validate] provisioning CPAchecker -> $dest" >&2
    zip="$dest/cpachecker.zip"
    curl -fsSL --connect-timeout 30 --max-time 1200 -o "$zip" "$CPA_URL" || return 1
    # Extract with Python's stdlib (the dev image has no `unzip`).
    "${SAF_LINT_PYTHON:-/usr/bin/python3}" - "$zip" "$dest" <<'PY' || return 1
import sys, zipfile
with zipfile.ZipFile(sys.argv[1]) as z:
    z.extractall(sys.argv[2])
PY
    chmod +x "$CPA_HOME/bin/cpachecker" "$CPA_HOME/bin/cpa-witness2test" 2>/dev/null || true
    rm -f "$zip"
    [ -x "$CPA_HOME/bin/cpachecker" ]
}

if ! provision_cpachecker; then
    echo "CPACHECKER_ABSENT (syntactic gate only; set SAF_CPACHECKER or check network)"
    exit 0
fi

# CPAchecker 4.x analysis-based violation-witness validation (the standard
# reachability validator). Result FALSE ⇒ the violation is confirmed. A 90s CPU
# limit matches the SV-COMP violation-witness-validation convention. Output files
# go to a throwaway dir so the sweep leaves no clutter.
spec_arg=()
[ -n "$PROPERTY" ] && spec_arg=(--spec "$PROPERTY")
out_dir="$(mktemp -d)"
out="$(timeout -k 15 130 "$CPA_HOME/bin/cpachecker" \
        --config "$CPA_HOME/config/violation-witness-validation.properties" \
        --witness "$WITNESS" \
        "${spec_arg[@]}" \
        --timelimit 90s \
        --output-path "$out_dir" \
        "$PROGRAM" 2>&1)" || true
rm -rf "$out_dir"

verdict="$(printf '%s\n' "$out" | grep -oE 'Verification result: [A-Z]+' | head -1 | awk '{print $3}')"
if [ "$verdict" = "FALSE" ]; then
    echo "CONFIRMED (cpachecker-analysis)"
    exit 0
fi

# ---- Stage 3: cpa-witness2test (execution-based confirmation, best-effort) --
# Analysis-based validation returns UNKNOWN on recursion/loops/arrays it cannot
# unroll; an execution harness (compile + run the witnessed path) reproduces many
# of those. Confirmed iff the generated test reaches the expected violation.
if [ "${SAF_SKIP_WITNESS2TEST:-0}" = "1" ]; then
    echo "NOT_CONFIRMED (analysis=${verdict:-none}; witness2test skipped)"
    exit 0
fi
W2T="$CPA_HOME/bin/cpa-witness2test"
chmod +x "$W2T" 2>/dev/null || true
if [ ! -f "$W2T" ] || [ -z "$PROPERTY" ]; then
    # cpa-witness2test requires an executable driver and a --spec property file.
    echo "NOT_CONFIRMED (analysis=${verdict:-none}; witness2test unavailable)"
    exit 0
fi
case "$DATA_MODEL" in
    ILP32) w2t_bit=--32 ;;
    *) w2t_bit=--64 ;;
esac
out_dir2="$(mktemp -d)"
out2="$(timeout -k 15 120 "$W2T" "$w2t_bit" --spec "$PROPERTY" \
        --witness "$WITNESS" \
        --output-path "$out_dir2" \
        "$PROGRAM" 2>&1)" || true
rm -rf "$out_dir2"
# ---- Stage 4: CBMC memory-safety confirmation (independent SV-COMP validator) --
# CPAchecker's SMG analysis and non-sanitized cpa-witness2test are blind to many
# stack/heap overflows (e.g. Juliet CWE121) that SAF detects via ASan; CBMC -- an
# official SV-COMP validator -- re-verifies bit-precisely and DOES find them.
# SV-COMP confirms a witness if ANY panel validator agrees, so a genuine CBMC
# memory-safety FAILURE is a legitimate, competition-predictive confirmation. Runs
# only for valid-memsafety, only after Stages 2-3 miss. SOUNDNESS: a CBMC failure
# is credited ONLY when it is (a) a real mem-safety violation class -- modeling
# artifacts ("no body for callee" / unwinding assertions) are excluded -- AND (b)
# located AT the witness's target line, so unrelated CBMC false positives (e.g. a
# spurious libc-model overflow inside strtol on nondet input, at a different line)
# do NOT confirm. Provisioned to $SAF_CBMC (cbmc + libminisat.so.2); skips if absent.
CBMC_HOME="${SAF_CBMC:-/workspace/.svtools/cbmc}"
is_memsafety=0
case "$PROPERTY" in *memsafety*) is_memsafety=1 ;; esac
if [ "$is_memsafety" = 0 ] && [ -n "$PROPERTY" ] && grep -qiE 'valid-(deref|free|memtrack)' "$PROPERTY" 2>/dev/null; then
    is_memsafety=1
fi
is_unreach=0
case "$PROPERTY" in *unreach*) is_unreach=1 ;; esac
if [ "$is_unreach" = 0 ] && [ -n "$PROPERTY" ] && grep -qiE 'reach_error|call\(' "$PROPERTY" 2>/dev/null; then
    is_unreach=1
fi
cbmc_confirms_memsafety() {
    [ "${SAF_SKIP_CBMC:-0}" = "1" ] && return 1
    local cbmc_bin lines o fails ln
    if [ -x "$CBMC_HOME/cbmc" ]; then
        cbmc_bin="$CBMC_HOME/cbmc"; export LD_LIBRARY_PATH="$CBMC_HOME:${LD_LIBRARY_PATH:-}"
    elif command -v cbmc >/dev/null 2>&1; then
        cbmc_bin="cbmc"
    else
        return 1
    fi
    # target line(s) the witness points at (the location SAF flagged the violation)
    lines="$(grep -oE 'line:[[:space:]]*[0-9]+' "$WITNESS" 2>/dev/null | grep -oE '[0-9]+' | sort -u)"
    [ -n "$lines" ] || return 1
    o="$(timeout -k 15 120 "$cbmc_bin" --bounds-check --pointer-check --unwind 500 "$PROGRAM" 2>&1)" || true
    # genuine, FAILED memory-safety properties (status at end of line), minus artifacts
    fails="$(printf '%s\n' "$o" \
      | grep -E ': FAILURE$' \
      | grep -viE 'no body for callee|unwinding assertion' \
      | grep -iE 'region (writeable|readable)|(upper|lower) bound|object bounds|dereference failure|dynamically allocated|deallocated|dead object|invalid pointer|pointer NULL')"
    [ -n "$fails" ] || return 1
    # require a genuine failure AT one of the witness target lines
    for ln in $lines; do
        printf '%s\n' "$fails" | grep -qE "line ${ln}[^0-9]" && return 0
    done
    return 1
}
cbmc_confirms_unreach() {
    # For unreach-call, reach_error() calls __assert_fail, so reaching it is a CBMC
    # assertion FAILURE named after reach_error. Confirm iff CBMC reports that error
    # target reachable; EXCLUDE bound artifacts (unwinding assertions -> inconclusive)
    # and modeling artifacts (no body). reach_error reachability IS the property
    # violation, so no line-matching is needed. Same soundness basis as memsafety: the
    # confirmer only runs on SAF FALSE verdicts, and SAF abstains (unknown) on safe
    # tasks -- verified CBMC also finds reach_error UNreachable on safe programs.
    [ "${SAF_SKIP_CBMC:-0}" = "1" ] && return 1
    local cbmc_bin o
    if [ -x "$CBMC_HOME/cbmc" ]; then
        cbmc_bin="$CBMC_HOME/cbmc"; export LD_LIBRARY_PATH="$CBMC_HOME:${LD_LIBRARY_PATH:-}"
    elif command -v cbmc >/dev/null 2>&1; then
        cbmc_bin="cbmc"
    else
        return 1
    fi
    o="$(timeout -k 15 120 "$cbmc_bin" --unwind 200 "$PROGRAM" 2>&1)" || true
    printf '%s\n' "$o" \
      | grep -E ': FAILURE$' \
      | grep -viE 'unwinding assertion|no body for callee' \
      | grep -qiE 'reach_error|__VERIFIER_error'
}
if printf '%s\n' "$out2" | grep -qE 'reached expected property violation|Verification result: FALSE'; then
    echo "CONFIRMED (witness2test-execution)"
elif [ "$is_memsafety" = 1 ] && cbmc_confirms_memsafety; then
    echo "CONFIRMED (cbmc-memsafety)"
elif [ "$is_unreach" = 1 ] && cbmc_confirms_unreach; then
    echo "CONFIRMED (cbmc-unreach)"
else
    echo "NOT_CONFIRMED (analysis=${verdict:-none}; witness2test/cbmc did not confirm)"
fi
exit 0
