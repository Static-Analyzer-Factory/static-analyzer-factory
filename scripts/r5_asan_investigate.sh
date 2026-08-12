#!/bin/sh
# Slice 0c investigation — dump the FULL AddressSanitizer report (faulting line +
# allocation site) for specific tasks, to classify a false alarm as harness-artifact
# vs real. Args: task basenames (glob). Run as root in the dev image.
set -u
cd /workspace || exit 1
dpkg --add-architecture i386 >/dev/null 2>&1
apt-get update -qq >/dev/null 2>&1
apt-get install -y --no-install-recommends libclang-rt-18-dev libc6-dev:i386 libstdc++6:i386 libgcc-s1:i386 >/dev/null 2>&1
DRIVER=scripts/r5_asan_zerodriver.c
STUB=share/saf/stubs/sv-comp-stubs.h
for pat in "$@"; do
  f=$(find tests/benchmarks/sv-benchmarks/c -name "$pat" 2>/dev/null | head -1)
  echo "################################ $pat"
  echo "path: $f"
  [ -z "$f" ] && { echo "NOT FOUND"; continue; }
  if ! clang-18 -O0 -g -fsanitize=address -fno-sanitize-recover=address -m64 \
       -include "$STUB" -I "$(dirname "$f")" -Wno-everything "$f" "$DRIVER" -o /tmp/inv 2>/tmp/cerr; then
    echo COMPILE_FAIL; head -5 /tmp/cerr; continue
  fi
  ASAN_OPTIONS=exitcode=1:abort_on_error=0:detect_leaks=0 /tmp/inv >/tmp/out 2>/tmp/err
  echo "exit=$?"
  sed -n '1,22p' /tmp/err
  echo
done
