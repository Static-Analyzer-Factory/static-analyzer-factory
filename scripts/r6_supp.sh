#!/usr/bin/env sh
# Slice 0 supplementary — R6 recall-ceiling + design characterization (measurement only).
#   (A) left-shift recall gap: how many buggy tasks trap ONLY when -fsanitize=shift is
#       added (the research's recall gap; decides whether R6 should add shift later).
#   (B) compile_fail cause: why ~24% of the measurement sample fails to link (const-driver
#       symbol collision? missing lib? -m32?) — a recall-ceiling artifact to characterize.
# Run: docker compose run --rm -T --entrypoint sh dev -c 'sh scripts/r6_supp.sh'
set -u
CLANG=clang-18
STUB="share/saf/stubs/sv-comp-stubs.h"
DRIVER="scripts/r5_asan_constdriver.c"
SIO="-fsanitize=signed-integer-overflow -fno-sanitize-recover=signed-integer-overflow"
SHIFT="-fsanitize=signed-integer-overflow,shift -fno-sanitize-recover=signed-integer-overflow,shift"
UBOPTS="halt_on_error=1:abort_on_error=0:print_stacktrace=0"
CONSTS="0 1 2 42 255 256 1024 65535 2147483647 -1 -2147483648 2147483648"

trap_under() { # <flags> <src> <bits> -> prints the first trapping report line or nothing
  flags="$1"; src="$2"; bits="$3"; sd="$(dirname "$src")"
  $CLANG -O0 -g $flags "-m$bits" -include "$STUB" -I "$sd" -Wno-everything "$src" "$DRIVER" -o /tmp/r6s 2>/tmp/r6s_cc || { echo "CCFAIL"; return; }
  for k in $CONSTS; do
    e="$(UBSAN_OPTIONS="$UBOPTS" SAF_NONDET_CONST="$k" /tmp/r6s 2>&1 1>/dev/null)"
    line="$(printf '%s\n' "$e" | grep -m1 'runtime error:')"
    [ -n "$line" ] && { printf '%s (const=%s)\n' "$(printf '%s' "$line" | cut -c1-105)" "$k"; return; }
  done
  echo "no-trap"
}

echo "########## (A) LEFT-SHIFT recall gap (signed-integer-overflow vs +shift) ##########"
echo "For buggy tasks in shift-prone families, does adding ,shift catch a task SIO misses?"
for src in $(grep -rl "no-overflow" tests/benchmarks/sv-benchmarks/c/bitvector \
                tests/benchmarks/sv-benchmarks/c/uthash-2.0.2 \
                tests/benchmarks/sv-benchmarks/c/recursive --include="*.yml" 2>/dev/null \
             | while read y; do
                 grep -q "expected_verdict: false" "$y" && \
                 f="$(grep -oE "input_files:[^\n]*" "$y" | head -1 | sed "s/input_files:[ '\"]*//; s/['\"].*//")" && \
                 echo "$(dirname "$y")/$f"; done | head -14); do
  [ -f "$src" ] || continue
  n="$(basename "$src")"
  sio="$(trap_under "$SIO" "$src" 64)"
  shf="$(trap_under "$SHIFT" "$src" 64)"
  # highlight the interesting case: SIO no-trap but +shift traps on a "left shift" msg
  flag=""
  case "$sio" in no-trap|CCFAIL) case "$shf" in *"shift"*|*"runtime error"*) flag="  <== SHIFT-ONLY GAP" ;; esac ;; esac
  printf '%-42s SIO=%-16.16s SHIFT=%-28.28s%s\n' "$n" "$sio" "$shf" "$flag"
done

echo ""
echo "########## (B) compile_fail cause (first 6 measurement compile-fails) ##########"
for src in $(grep -rl "no-overflow" tests/benchmarks/sv-benchmarks/c/nla-digbench-scaling \
                tests/benchmarks/sv-benchmarks/c/loop-zilu --include="*.yml" 2>/dev/null \
             | while read y; do
                 f="$(grep -oE "input_files:[^\n]*" "$y" | head -1 | sed "s/input_files:[ '\"]*//; s/['\"].*//")"; \
                 echo "$(dirname "$y")/$f"; done | head -6); do
  [ -f "$src" ] || continue
  n="$(basename "$src")"; sd="$(dirname "$src")"
  if $CLANG -O0 -g $SIO -m64 -include "$STUB" -I "$sd" -Wno-everything "$src" "$DRIVER" -o /tmp/r6s 2>/tmp/r6s_cc; then
    echo "$n: OK"
  else
    echo "$n: FAIL -> $(grep -m1 -iE 'error|multiple definition|undefined|conflicting' /tmp/r6s_cc | cut -c1-90)"
  fi
done
