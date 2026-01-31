#!/usr/bin/env bash
# Compile CPython to LLVM IR for incremental analysis benchmarking.
# Must be run inside Docker (requires clang-18/LLVM 18).
set -euo pipefail

VERSION="${1:-v3.13.0}"
# Use workspace-mounted dir so source persists across Docker runs
SOURCE_DIR="/workspace/tests/benchmarks/incremental/cpython/source"
OUTPUT_DIR="/workspace/tests/benchmarks/incremental/cpython/.compiled"

echo "=== Compiling CPython ${VERSION} to LLVM IR ==="

# Clone if not already present
if [ ! -d "${SOURCE_DIR}" ]; then
    echo "Cloning CPython ${VERSION}..."
    git clone --depth 1 --branch "${VERSION}" \
        "https://github.com/python/cpython.git" "${SOURCE_DIR}"
fi

# Run ./configure to generate pyconfig.h (required for most files)
if [ ! -f "${SOURCE_DIR}/pyconfig.h" ]; then
    echo "Running ./configure to generate pyconfig.h..."
    cd "${SOURCE_DIR}"
    CC=clang-18 ./configure --without-ensurepip --disable-shared 2>&1 | tail -5
    cd /workspace
fi

rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}"

CLANG_FLAGS="-S -emit-llvm -O0 -DPy_BUILD_CORE \
    -I${SOURCE_DIR} \
    -I${SOURCE_DIR}/Include \
    -I${SOURCE_DIR}/Include/internal \
    -I${SOURCE_DIR}/Include/cpython \
    -I${SOURCE_DIR}/Python \
    -I${SOURCE_DIR}/Objects"

# Compile Objects/ and Python/ .c files (the core of CPython)
compiled=0
skipped=0
for subdir in Objects Python; do
    echo "Compiling ${subdir}/..."
    find "${SOURCE_DIR}/${subdir}" -maxdepth 1 -name '*.c' | sort | while read -r f; do
        relpath="${f#${SOURCE_DIR}/}"
        outname="$(echo "${relpath}" | tr '/' '_' | sed 's/\.c$/.ll/')"
        if clang-18 ${CLANG_FLAGS} "${f}" -o "${OUTPUT_DIR}/${outname}" 2>/dev/null; then
            echo "  ${relpath}"
        else
            echo "  SKIP ${relpath}"
        fi
    done
done

echo ""
echo "=== Generating v2 variants ==="

# Leaf edit: add allocation in builtin_len (Python/bltinmodule.c)
BLTINMODULE="${SOURCE_DIR}/Python/bltinmodule.c"
if [ -f "${BLTINMODULE}" ] && [ -f "${OUTPUT_DIR}/Python_bltinmodule.ll" ]; then
    cp "${BLTINMODULE}" /tmp/bltinmodule_v2.c
    # Insert allocation after "builtin_len(PyObject *module, PyObject *obj)" opening brace
    sed -i '/^builtin_len(PyObject/,/^{/{
        /^{/a\    void *incr_test = PyMem_Malloc(64); PyMem_Free(incr_test);
    }' /tmp/bltinmodule_v2.c
    if clang-18 ${CLANG_FLAGS} /tmp/bltinmodule_v2.c -o "${OUTPUT_DIR}/Python_bltinmodule_v2.ll" 2>/dev/null; then
        # Fix only the top-level module identity (not string constants)
        sed -i '1s/bltinmodule_v2\.c/bltinmodule.c/' "${OUTPUT_DIR}/Python_bltinmodule_v2.ll"
        sed -i '2s/bltinmodule_v2\.c/bltinmodule.c/' "${OUTPUT_DIR}/Python_bltinmodule_v2.ll"
        echo "  Python_bltinmodule_v2.ll generated"
    else
        echo "  WARNING: bltinmodule_v2 compile failed"
    fi
else
    echo "  SKIP: bltinmodule.c not compiled or not found"
fi

# Core edit: add indirection in _Py_Dealloc (Objects/object.c)
OBJECT_C="${SOURCE_DIR}/Objects/object.c"
if [ -f "${OBJECT_C}" ] && [ -f "${OUTPUT_DIR}/Objects_object.ll" ]; then
    cp "${OBJECT_C}" /tmp/object_v2.c
    # Insert function pointer indirection after _Py_Dealloc opening brace
    sed -i '/_Py_Dealloc(PyObject \*op)/{
        n
        /^{/a\    /* Incremental test: pointer indirection */\n    typedef void (*dealloc_hook_t)(PyObject *);\n    static dealloc_hook_t hook = NULL;\n    if (hook) { hook(op); return; }
    }' /tmp/object_v2.c
    if clang-18 ${CLANG_FLAGS} /tmp/object_v2.c -o "${OUTPUT_DIR}/Objects_object_v2.ll" 2>/dev/null; then
        # Fix only the top-level module identity (not string constants)
        sed -i '1s/object_v2\.c/object.c/' "${OUTPUT_DIR}/Objects_object_v2.ll"
        sed -i '2s/object_v2\.c/object.c/' "${OUTPUT_DIR}/Objects_object_v2.ll"
        echo "  Objects_object_v2.ll generated"
    else
        echo "  WARNING: object_v2 compile failed"
    fi
else
    echo "  SKIP: object.c not compiled or not found"
fi

# Report
echo ""
echo "=== Summary ==="
TOTAL=$(ls "${OUTPUT_DIR}"/*.ll 2>/dev/null | wc -l)
V2=$(ls "${OUTPUT_DIR}"/*_v2.ll 2>/dev/null | wc -l)
echo "Files: ${TOTAL} (${V2} v2 variants)"
echo "Total size: $(du -sh "${OUTPUT_DIR}" | cut -f1)"
