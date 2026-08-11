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
out="$("$CPA_HOME/bin/cpachecker" \
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
out2="$(timeout 120 "$W2T" "$w2t_bit" --spec "$PROPERTY" \
        --witness "$WITNESS" \
        --output-path "$out_dir2" \
        "$PROGRAM" 2>&1)" || true
rm -rf "$out_dir2"
if printf '%s\n' "$out2" | grep -qE 'reached expected property violation|Verification result: FALSE'; then
    echo "CONFIRMED (witness2test-execution)"
else
    echo "NOT_CONFIRMED (analysis=${verdict:-none}; witness2test did not reproduce)"
fi
exit 0
