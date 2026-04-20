#!/usr/bin/env bash
# Compiles the LLVM 19-22 syntax fixture sources with clang-22 into
# tests/fixtures/llvm/llvm22_syntax/. Run inside the dev-llvm22 Docker image:
#   make compile-llvm22-syntax-fixtures
#
# The fixtures exercise IR forms introduced between LLVM 18 and LLVM 22; each
# source is chosen to force clang to emit the specific construct we test for.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURE_DIR="$SCRIPT_DIR/../fixtures/llvm/llvm22_syntax"

CLANG="clang-22"
CLANGXX="clang++-22"

if ! command -v "$CLANG" &>/dev/null || ! command -v "$CLANGXX" &>/dev/null; then
    echo "ERROR: clang-22 / clang++-22 not found on PATH. Run inside dev-llvm22." >&2
    exit 1
fi

mkdir -p "$FIXTURE_DIR"

# -----------------------------------------------------------------------------
# inrange_attr: C++ vtable install -> `inrange(low, high)` on GEP (LLVM 20+).
# -----------------------------------------------------------------------------
echo "  inrange_attr.cpp -> inrange_attr.ll"
"$CLANGXX" -S -emit-llvm -O0 -Xclang -disable-O0-optnone \
    "$SCRIPT_DIR/cpp/llvm22_syntax/inrange_attr.cpp" \
    -o "$FIXTURE_DIR/inrange_attr.ll"

# -----------------------------------------------------------------------------
# captures_attr: C with no-capture-inferable params -> `captures(none)` (LLVM 21+).
# -----------------------------------------------------------------------------
echo "  captures_attr.c -> captures_attr.ll"
"$CLANG" -S -emit-llvm -O2 \
    "$SCRIPT_DIR/c/llvm22_syntax/captures_attr.c" \
    -o "$FIXTURE_DIR/captures_attr.ll"

# -----------------------------------------------------------------------------
# masked_intrinsic: loop-vectorizer emits `@llvm.masked.store` with LLVM 22's
# alignment-attribute-only signature.
# -----------------------------------------------------------------------------
echo "  masked_intrinsic.c -> masked_intrinsic.ll"
"$CLANG" -S -emit-llvm -O2 -mavx2 \
    "$SCRIPT_DIR/c/llvm22_syntax/masked_intrinsic.c" \
    -o "$FIXTURE_DIR/masked_intrinsic.ll"

echo "Done. Fixtures in $FIXTURE_DIR/"
