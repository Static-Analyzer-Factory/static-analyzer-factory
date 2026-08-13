#!/usr/bin/env bash
# Slice 0e — R6 no-overflow witness confirmation de-risk (measurement only; NO production code).
# The PRIMARY go/no-go gate (research red flag): does a TARGET-ONLY YAML-2.0 violation
# witness at the UBSan-reported overflow line get CONFIRMED by CPAchecker for no-overflow,
# and does the UBSan column align with the spec's "first character of the containing
# statement" rule (or must we drop the column)?
#
# For each (task, data_model): compile the ORIGINAL with UBSan signed-overflow + the
# const-driver, mini-fuzz to find a trapping constant, capture file:line:col, then
# generate TWO witnesses (with UBSan column / line-only) and run validate_witness.sh
# (witnesslint + CPAchecker) against the ORIGINAL program for the no-overflow property.
#
# Usage (inside the dev container):  sh scripts/r6_witness_derisk.sh <task.i> <LP64|ILP32>
set -u
SRC="${1:?usage: r6_witness_derisk.sh <task.(i|c)> <LP64|ILP32>}"
DM="${2:-LP64}"
PRP="tests/benchmarks/sv-benchmarks/c/properties/no-overflow.prp"
DRIVER="scripts/r5_asan_constdriver.c"
STUB="share/saf/stubs/sv-comp-stubs.h"
CLANG="${SAF_CLANG:-clang-18}"
BITS="-m64"; [ "$DM" = "ILP32" ] && BITS="-m32"
CONSTS="0 1 2 42 255 256 1024 65535 2147483647 -1 -2147483648 2147483648"
UBOPTS="halt_on_error=1:abort_on_error=0:print_stacktrace=1"
OVF='runtime error: (signed integer overflow|negation of|division of)'

srcdir="$(dirname "$SRC")"
bin="/tmp/r6_wit_h"
echo "== compiling $SRC ($DM) with -fsanitize=signed-integer-overflow =="
if ! "$CLANG" -O0 -g -fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow \
        "$BITS" -include "$STUB" -I "$srcdir" -Wno-everything "$SRC" "$DRIVER" -o "$bin" 2>/tmp/r6_cc.log; then
  echo "COMPILE_FAIL:"; sed 's/^/    /' /tmp/r6_cc.log | head -6; exit 1
fi

report=""
for k in $CONSTS; do
  err="$(UBSAN_OPTIONS="$UBOPTS" SAF_NONDET_CONST="$k" "$bin" 2>&1 1>/dev/null)"
  if printf '%s' "$err" | grep -qE "$OVF"; then
    echo "== TRAP at const=$k =="
    printf '%s\n' "$err" | grep -E "$OVF|#0 " | head -3
    report="$err"; break
  fi
done
[ -z "$report" ] && { echo "NO_TRAP (no constant reproduced an overflow)"; exit 1; }

# The located line: "<file>:<line>:<col>: runtime error: ..."
locline="$(printf '%s\n' "$report" | grep -E "$OVF" | head -1)"
loc="$(printf '%s' "$locline" | grep -oE '^[^ ]+:[0-9]+:[0-9]+' | head -1)"
line="$(printf '%s' "$loc" | awk -F: '{print $(NF-1)}')"
col="$(printf '%s' "$loc" | awk -F: '{print $NF}')"
echo "located: line=$line col=$col  (spec wants FIRST CHAR of the containing statement)"

echo ""; echo "########## WITNESS A: with UBSan column ($col) ##########"
python3 scripts/r6_make_witness.py "$SRC" "$line" "$col" "$DM" "$PRP" > /tmp/r6_wit_col.yml
SAF_SKIP_WITNESS2TEST=0 bash scripts/validate_witness.sh /tmp/r6_wit_col.yml "$SRC" "$PRP" "$DM" 2>&1 | grep -vE '^\[validate\] provisioning' | tail -8

echo ""; echo "########## WITNESS B: line-only (no column) ##########"
python3 scripts/r6_make_witness.py "$SRC" "$line" "" "$DM" "$PRP" > /tmp/r6_wit_line.yml
SAF_SKIP_WITNESS2TEST=0 bash scripts/validate_witness.sh /tmp/r6_wit_line.yml "$SRC" "$PRP" "$DM" 2>&1 | grep -vE '^\[validate\] provisioning' | tail -8
