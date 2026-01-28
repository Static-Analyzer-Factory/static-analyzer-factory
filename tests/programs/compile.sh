#!/usr/bin/env bash
# Compiles test program sources to LLVM IR (.ll) text format.
# Run inside Docker: make compile-fixtures
#
# Flags:
#   -S -emit-llvm  : text LLVM IR (readable, diffable)
#   -O0            : no optimizations (preserves source structure)
#   -g             : debug info (provides source spans)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIXTURE_DIR="$SCRIPT_DIR/../fixtures/llvm/e2e"

mkdir -p "$FIXTURE_DIR"

echo "=== Compiling C programs ==="
for src in "$SCRIPT_DIR"/c/*.c; do
    [ -f "$src" ] || continue
    base="$(basename "$src" .c)"
    echo "  $base.c -> $base.ll"
    clang-18 -S -emit-llvm -O0 -g -o "$FIXTURE_DIR/$base.ll" "$src"
done

echo "=== Compiling C++ programs ==="
for src in "$SCRIPT_DIR"/cpp/*.cpp; do
    [ -f "$src" ] || continue
    base="$(basename "$src" .cpp)"
    echo "  $base.cpp -> $base.ll"
    clang++-18 -S -emit-llvm -O0 -g -o "$FIXTURE_DIR/$base.ll" "$src"
done

echo "=== Compiling Rust programs ==="
for src in "$SCRIPT_DIR"/rust/*.rs; do
    [ -f "$src" ] || continue
    base="$(basename "$src" .rs)"
    echo "  $base.rs -> $base.ll"
    rustc --emit=llvm-ir -C opt-level=0 -o "$FIXTURE_DIR/$base.ll" "$src"
done

echo "=== Done ==="
ls -la "$FIXTURE_DIR"/*.ll 2>/dev/null || echo "(no .ll files generated)"
