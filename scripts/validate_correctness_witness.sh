#!/usr/bin/env bash
# Validate an SV-COMP YAML *correctness* (invariant_set) witness (plan 207 / 1b).
# Handles format 2.0 and 2.1 — 2.1 is what SV-COMP 2027 requires for every
# C.termination.* base category, and the version is read from the witness, never
# assumed (witnesslint fails a 2.1 document told to expect 2.0, and CPAchecker's
# 2.0 parser rejects a 2.1 document outright rather than tolerating it).
#
#   validate_correctness_witness.sh <witness.yml> <program.c> [property.prp] [data_model]
#
# The TRUE-side counterpart to validate_witness.sh (which is violation-only):
#   Stage 1 — witnesslint (best-effort syntactic check; --expectCorrectnessWitness).
#             Non-conformance / unsupported flag => LINT_WARN, but we still run
#             CPAchecker (the authoritative gate).
#   Stage 2 — CPAchecker correctness-witness validation
#             (--config correctness-witness-validation.properties). CONFIRMED iff
#             "Verification result: TRUE". Prints CONFIRMED / NOT_CONFIRMED /
#             CPACHECKER_ABSENT. There is NO cpa-witness2test stage (violation-only).
#
# Never fails the script; the confirmation string is the measurement.
# Env: SAF_CPACHECKER (CPAchecker home), SAF_SVWITNESSES (witnesslint), SAF_SKIP_CPACHECKER=1.
set -u

WITNESS="${1:?usage: validate_correctness_witness.sh <witness.yml> <program.c> [property.prp] [data_model]}"
PROGRAM="${2:?missing program}"
PROPERTY="${3:-}"
DATA_MODEL="${4:-ILP32}"

SVW="${SAF_SVWITNESSES:-/opt/sv-witnesses}"
LINT_PY="${SAF_LINT_PYTHON:-/usr/bin/python3}"
CPA_HOME="${SAF_CPACHECKER:-/workspace/.svtools/CPAchecker-4.2.2-unix}"

case "$DATA_MODEL" in
    ILP32 | 32 | 32bit) bit=--32 ;;
    *) bit=--64 ;;
esac

# ---- Stage 0: native-binary exec bits -------------------------------------
# The shipped archive leaves `lib/native/x86_64-linux/{z3,ltl3ba}` non-executable.
# `bin/cpachecker --version` still passes, so the skew is invisible until an
# analysis needs the SMT solver (z3) or the LTL-to-Buechi translator (ltl3ba —
# which is exactly what termination validation uses). Idempotent; cheap; must run
# before any measurement is trusted.
for nb in "$CPA_HOME"/lib/native/*/z3 "$CPA_HOME"/lib/native/*/ltl3ba; do
    [ -f "$nb" ] && [ ! -x "$nb" ] && chmod +x "$nb" 2>/dev/null && \
        echo "[validate-correctness] fixed missing exec bit on $(basename "$nb")" >&2
done

# ---- Stage 1: witnesslint (best-effort syntactic check) --------------------
# Take the version from the witness; an assumed version turns a well-formed 2.1
# witness into a lint failure that looks like a content bug.
WVERSION="$(grep -m1 -oE "format_version:[[:space:]]*['\"]?[0-9]+\.[0-9]+" "$WITNESS" 2>/dev/null \
            | grep -oE '[0-9]+\.[0-9]+' || true)"
WVERSION="${WVERSION:-2.0}"
echo "[validate-correctness] witnesslint: $WITNESS (format $WVERSION)"
if "$LINT_PY" "$SVW/linter/witnesslinter.py" \
        --witness "$WITNESS" "$PROGRAM" \
        --expectCorrectnessWitness --expectedWitnessVersion "$WVERSION" >/dev/null 2>&1; then
    echo "LINT_OK"
else
    echo "LINT_WARN (non-conformant or flag unsupported; continuing to CPAchecker)"
fi

# ---- Stage 2: CPAchecker correctness-witness validation --------------------
if [ "${SAF_SKIP_CPACHECKER:-0}" = "1" ]; then
    echo "CPACHECKER_SKIPPED"
    exit 0
fi
if [ ! -x "$CPA_HOME/bin/cpachecker" ]; then
    echo "CPACHECKER_ABSENT (set SAF_CPACHECKER to a CPAchecker-4.2.2 home)"
    exit 0
fi

spec_arg=()
[ -n "$PROPERTY" ] && spec_arg=(--spec "$PROPERTY")
out_dir="$(mktemp -d)"
# NB: explicit --32/--64 must match the witness data_model; witness.checkProgramHash
# is disabled (as SV-COMP's own validation runs do) so a benign hash mismatch does
# not mask the semantic verdict.
out="$(timeout -k 15 200 "$CPA_HOME/bin/cpachecker" \
        --config "$CPA_HOME/config/correctness-witness-validation.properties" \
        --witness "$WITNESS" \
        "${spec_arg[@]}" \
        "$bit" \
        --option witness.checkProgramHash=false \
        --timelimit 150s \
        --output-path "$out_dir" \
        "$PROGRAM" 2>&1)" || true
rm -rf "$out_dir"

# A dropped invariant (location did not map to a CFA node) is a common silent
# non-confirmation cause — surface it.
if printf '%s\n' "$out" | grep -q "Could not find node"; then
    echo "NOTE: 'Could not find node' — an invariant location did not map to a CFA node" >&2
fi

verdict="$(printf '%s\n' "$out" | grep -oE 'Verification result: [A-Z]+' | head -1 | awk '{print $3}')"
if [ "$verdict" = "TRUE" ]; then
    echo "CONFIRMED (cpachecker-correctness)"
    exit 0
fi
echo "NOT_CONFIRMED (cpachecker=${verdict:-none})"
exit 0
