#!/usr/bin/env bash
# Validate an SV-COMP YAML 2.0 *violation* witness (plan 194 Slice F/G).
#
#   validate_witness.sh <witness.yml> <program.c> [property.prp] [data_model]
#
# Stage 1 — witnesslint (HARD syntactic gate): the witness must conform to the
#           2.0 violation-witness schema. Non-conformance => exit 2.
# Stage 2 — CPAchecker witness validation (semantic confirmation, BEST-EFFORT):
#           prints CONFIRMED / NOT_CONFIRMED / CPACHECKER_ABSENT. Never fails the
#           script (the score-predicting measurement lives in the eval harness).
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
    chmod +x "$CPA_HOME/bin/cpachecker" 2>/dev/null || true
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
case "$verdict" in
    FALSE) echo "CONFIRMED" ;;
    TRUE | UNKNOWN) echo "NOT_CONFIRMED ($verdict)" ;;
    *) echo "NOT_CONFIRMED (cpachecker gave no parseable verdict)" ;;
esac
exit 0
