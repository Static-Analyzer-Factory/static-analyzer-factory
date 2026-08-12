#!/bin/sh
# Slice 0c runner — install the 0a-confirmed ASan package set (ephemeral, throwaway
# root container; NO image/Dockerfile change) then run the ASan-gate measurement.
set -u
cd /workspace || exit 1
dpkg --add-architecture i386
apt-get update -qq 2>&1 | tail -1
apt-get install -y --no-install-recommends \
    libclang-rt-18-dev libc6-dev:i386 libstdc++6:i386 libgcc-s1:i386 2>&1 | tail -2
echo "--- asan ready; running measurement (N=${N:-40}) ---"
python3 /workspace/scripts/r5_asan_measure.py
