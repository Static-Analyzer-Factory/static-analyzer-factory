#!/bin/bash
# Compile oracle C programs to LLVM IR for verification testing.
# Must run inside Docker (needs clang/LLVM 18).
#
# Usage: ./scripts/compile-oracle.sh [--layer pta|callgraph|cfg|mssa|svfg]

set -euo pipefail

ORACLE_DIR="tests/verification/oracle"
COMPILED_DIR="tests/verification/oracle/.compiled"

LAYER="${1:-}"
if [[ "$LAYER" == "--layer" ]]; then
    LAYER="${2:-}"
fi

compile_file() {
    local src="$1"
    local layer
    layer=$(dirname "$src" | xargs basename)
    local name
    name=$(basename "$src" .c)
    local out_dir="$COMPILED_DIR/$layer"
    local out="$out_dir/${name}.ll"

    mkdir -p "$out_dir"

    local CC="${CLANG:-clang-18}"
    local OPT="${LLVM_OPT:-opt-18}"

    echo "  Compiling $layer/$name.c -> $name.ll"
    "$CC" -S -emit-llvm -g -O0 -Xclang -disable-O0-optnone "$src" -o "$out" 2>/dev/null || {
        echo "  WARNING: Failed to compile $src (skipping)"
        return 0
    }

    # Apply mem2reg for cleaner SSA
    "$OPT" -passes=mem2reg -S "$out" -o "$out"
}

echo "=== Compiling Oracle Verification Programs ==="
echo ""

count=0
for layer_dir in "$ORACLE_DIR"/*/; do
    layer=$(basename "$layer_dir")
    # Skip non-layer directories
    [[ "$layer" == ".compiled" ]] && continue
    [[ "$layer" == "harness" ]] && continue

    # Filter by layer if specified
    if [[ -n "$LAYER" ]] && [[ "$layer" != "$LAYER" ]]; then
        continue
    fi

    echo "Layer: $layer"
    for src in "$layer_dir"*.c; do
        [[ -f "$src" ]] || continue
        compile_file "$src"
        count=$((count + 1))
    done
    echo ""
done

echo "=== Compiled $count oracle programs ==="
