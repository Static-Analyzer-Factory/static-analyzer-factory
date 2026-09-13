#!/bin/sh
# Movement 5, take 3. Isolate the CPAchecker TRUE gate for real.
#
# Take 1 failed: resolve_cpachecker() found /workspace/.svtools via its CWD fallback.
# Take 2 failed: the relocated binary could not find share/saf/stubs/sv-comp-stubs.h,
#                so SAF bailed to `unknown` before reaching the gate.
#
# This take builds the ARCHIVE layout that resolve_svcomp_stub() actually accepts
# (<root>/bin/saf + <root>/share/saf/...), runs from a CWD with no .svtools/
# anywhere above it, and uses a REAL task from the gated set.
set -eu

ROOT=/tmp/safarch
rm -rf "$ROOT"
mkdir -p "$ROOT/bin"
cp /workspace/target/release/saf "$ROOT/bin/saf"
cp -r /workspace/share "$ROOT/share"

echo "############ archive-shaped layout"
find "$ROOT" -maxdepth 2 | sort | head
echo "  (no .svtools anywhere above cwd:)"
ls "$ROOT/.svtools" 2>&1 || true
ls /tmp/.svtools 2>&1 || true

# Pick the first no-overflow task from the gated manifest.
TASK=$(python3 -c "
import json
for l in open('/workspace/splits/lever1_true99.jsonl'):
    r = json.loads(l)
    if r.get('property') == 'no-overflow':
        print(r['rel_yml']); break
")
echo
echo "############ task: $TASK"
SVB=/workspace/tests/benchmarks/sv-benchmarks/c
YML="$SVB/$TASK"
# No pyyaml in the container's system python3; parse the task-def by hand.
BASE=${YML%.yml}
SRC=""
for ext in .i .c; do [ -f "$BASE$ext" ] && SRC="$BASE$ext" && break; done
[ -n "$SRC" ] || { echo "no source next to $YML"; exit 1; }
DM=$(grep -o 'ILP32\|LP64' "$YML" | head -1)
[ -n "$DM" ] || DM=ILP32
echo "  source:     $SRC"
echo "  data model: $DM"
cp "$SRC" "$ROOT/task.c"
cp "$SVB/properties/no-overflow.prp" "$ROOT/"

cd "$ROOT"
echo
echo "############ ARM 1 - CPAchecker ABSENT (the archive condition)"
SAF_CBMC=/nonexistent SAF_CPACHECKER=/nonexistent \
  ./bin/saf verify --property no-overflow.prp --data-model "$DM" --witness w1.yml task.c > v1.txt 2> e1.txt || true
echo "  verdict: $(cat v1.txt)"
echo "  gate msg: $(grep -i 'cpachecker\|abstain' e1.txt | head -3 || echo '(none)')"

echo
echo "############ ARM 2 - CPAchecker PRESENT (control)"
SAF_CBMC=/nonexistent SAF_CPACHECKER=/workspace/.svtools/CPAchecker-4.2.2-unix \
  ./bin/saf verify --property no-overflow.prp --data-model "$DM" --witness w2.yml task.c > v2.txt 2> e2.txt || true
echo "  verdict: $(cat v2.txt)"
echo "  gate msg: $(grep -i 'cpachecker\|abstain' e2.txt | head -3 || echo '(none)')"

echo
echo "############ VERDICT OF THE EXPERIMENT"
V1=$(cat v1.txt); V2=$(cat v2.txt)
if [ "$V2" = "true" ] && [ "$V1" = "unknown" ]; then
  echo "  FAIL-CLOSED confirmed: gate present -> '$V2', gate absent -> '$V1'."
  echo "  Dropping CPAchecker costs the TRUE verdict but CANNOT produce a wrong TRUE."
elif [ "$V2" = "true" ] && [ "$V1" = "true" ]; then
  echo "  *** FAIL-OPEN *** gate absent still emits '$V1'. Soundness fix REQUIRED."
else
  echo "  inconclusive: arm1='$V1' arm2='$V2' (task may not be in SAF's provable class)"
fi

echo
echo "############ stderr byte counts (2 MB competition limit)"
wc -c e1.txt e2.txt
