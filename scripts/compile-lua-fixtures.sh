#!/usr/bin/env bash
# Compile Lua 5.4.7 sources to LLVM IR for incremental analysis fixtures.
# Must be run inside Docker (requires clang/LLVM 18).
set -euo pipefail

LUA_VERSION="5.4.7"
LUA_URL="https://www.lua.org/ftp/lua-${LUA_VERSION}.tar.gz"
WORK_DIR="/tmp/lua-fixtures-build"
OUTPUT_DIR="/workspace/tests/fixtures/incremental/lua"
PATCHES_DIR="/workspace/tests/fixtures/incremental/lua/patches"

echo "=== Compiling Lua ${LUA_VERSION} to LLVM IR ==="

# Download and extract
rm -rf "${WORK_DIR}"
mkdir -p "${WORK_DIR}"
cd "${WORK_DIR}"
curl -sL "${LUA_URL}" | tar xz
cd "lua-${LUA_VERSION}/src"

# Compile all .c files to .ll (no debug info to save space)
mkdir -p "${OUTPUT_DIR}"
for f in *.c; do
    echo "  Compiling ${f}..."
    clang-18 -S -emit-llvm -O0 -I. "${f}" -o "${OUTPUT_DIR}/${f%.c}.ll"
done

# Generate v2 variants by applying patches and recompiling
echo "=== Generating v2 variants ==="

# Leaf edit: lmathlib
if [ -f "${PATCHES_DIR}/lmathlib-leaf.patch" ]; then
    cp lmathlib.c lmathlib_v2.c
    patch lmathlib_v2.c "${PATCHES_DIR}/lmathlib-leaf.patch" || {
        echo "ERROR: lmathlib-leaf.patch failed to apply cleanly"
        exit 1
    }
    clang-18 -S -emit-llvm -O0 -I. lmathlib_v2.c -o "${OUTPUT_DIR}/lmathlib_v2.ll"
    # Fix module identity to match original (critical for incremental diff)
    sed -i "s/lmathlib_v2\.c/lmathlib.c/g" "${OUTPUT_DIR}/lmathlib_v2.ll"
    echo "  lmathlib_v2.ll generated"
else
    echo "WARNING: ${PATCHES_DIR}/lmathlib-leaf.patch not found, skipping"
fi

# Core edit: lobject
if [ -f "${PATCHES_DIR}/lobject-core.patch" ]; then
    cp lobject.c lobject_v2.c
    patch lobject_v2.c "${PATCHES_DIR}/lobject-core.patch" || {
        echo "ERROR: lobject-core.patch failed to apply cleanly"
        exit 1
    }
    clang-18 -S -emit-llvm -O0 -I. lobject_v2.c -o "${OUTPUT_DIR}/lobject_v2.ll"
    # Fix module identity to match original (critical for incremental diff)
    sed -i "s/lobject_v2\.c/lobject.c/g" "${OUTPUT_DIR}/lobject_v2.ll"
    echo "  lobject_v2.ll generated"
else
    echo "WARNING: ${PATCHES_DIR}/lobject-core.patch not found, skipping"
fi

# Report summary
echo ""
echo "=== Fixture summary ==="
echo "Files: $(ls "${OUTPUT_DIR}"/*.ll | wc -l)"
echo "Total size: $(du -sh "${OUTPUT_DIR}" | cut -f1)"
echo "Output: ${OUTPUT_DIR}"
