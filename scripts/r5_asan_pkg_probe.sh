#!/bin/sh
# Slice 0a — find the minimal apt package set for `clang-18 -m32/-m64 -fsanitize=address`.
# Run as root inside the dev image. Ephemeral (throwaway container) — the real fix goes in the Dockerfile.
set -u
dpkg --add-architecture i386
apt-get update -qq 2>&1 | tail -1

echo "=== install amd64 compiler-rt (libclang-rt-18-dev) ==="
apt-get install -y --no-install-recommends libclang-rt-18-dev 2>&1 | tail -2

echo "=== i386 + x86_64 asan static archives shipped by the amd64 package? ==="
dpkg -L libclang-rt-18-dev | grep asan | grep -E 'i386|x86_64' | grep '[.]a$'

echo "=== install i386 libc dev + i386 c++/gcc runtime (skip the broken :i386 rt pkg) ==="
apt-get install -y --no-install-recommends libc6-dev:i386 libstdc++6:i386 libgcc-s1:i386 2>&1 | tail -6

echo "=== toolchain probe (both -m64 and -m32) ==="
sh /workspace/scripts/r5_asan_toolchain_probe.sh
