#!/bin/bash
# Compile PTABen test suite with LLVM 18
# Usage: ./scripts/compile-ptaben.sh [--jobs N] [--verbose] [--clean]
#
# This script compiles all C/C++ source files from the PTABen (SVF Test-Suite)
# benchmark into LLVM text IR (.ll) files for analysis with SAF.
#
# Compilation flags match PTABen's upstream generate_bc.sh:
# - Textual IR output (-S -emit-llvm) with mem2reg promotion
# - Per-directory flags (e.g. -DINCLUDEMAIN for CWE-based tests)
# - -fno-discard-value-names for readable IR
#
# Note: The upstream generate_bc.sh misleadingly names output files .bc despite
# producing text IR (-S flag). This script uses the correct .ll extension.

set -euo pipefail

PTABEN_DIR="tests/benchmarks/ptaben"
SRC_DIR="$PTABEN_DIR/src"
OUT_DIR="$PTABEN_DIR/.compiled"
JOBS="$(nproc)"
VERBOSE="${VERBOSE:-0}"
CLEAN=0

# Parse optional arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --jobs)
            JOBS="${2:-$JOBS}"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=1
            shift
            ;;
        --clean)
            CLEAN=1
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Clean if requested
if [[ "$CLEAN" == "1" ]] && [[ -d "$OUT_DIR" ]]; then
    echo "Cleaning previous compilation..."
    rm -rf "$OUT_DIR"
fi

echo "=== PTABen Compilation ==="
echo "Source: $SRC_DIR"
echo "Output: $OUT_DIR"
echo "Jobs:   $JOBS"
echo ""

# Ensure source exists
if [[ ! -d "$SRC_DIR" ]]; then
    echo "ERROR: PTABen source not found at $SRC_DIR"
    echo "Did you run: git submodule update --init tests/benchmarks/ptaben"
    exit 1
fi

# Create output directory
mkdir -p "$OUT_DIR"

# Apply SAF oracle annotations patch if not already applied
PATCH_FILE="$(cd "$(dirname "$PTABEN_DIR")" && pwd)/ptaben.patch"
if [[ -f "$PATCH_FILE" ]]; then
    if git -C "$PTABEN_DIR" apply --check "$PATCH_FILE" 2>/dev/null; then
        echo "Applying SAF oracle annotations patch..."
        git -C "$PTABEN_DIR" apply "$PATCH_FILE"
    else
        echo "(Patch already applied or not needed, skipping)"
    fi
    echo ""
fi

# Include path for PTABen headers (aliascheck.h, std_testcase.h, etc.)
INCLUDE_FLAGS="-I$PTABEN_DIR"

# Detect LLVM tools (prefer versioned names)
OPT="opt"
if command -v opt-18 &>/dev/null; then
    OPT="opt-18"
elif command -v opt-17 &>/dev/null; then
    OPT="opt-17"
fi

# Directories that need -DINCLUDEMAIN (CWE-based tests with guarded main())
INCLUDEMAIN_DIRS="ae_assert_tests ae_overflow_tests ae_recursion_tests ae_wto_assert ae_assert_tests_fail ae_overflow_tests_fail ae_nullptr_deref_tests ae_nullptr_deref_tests_failed ae_recursion_tests_fail"

# Find all C and C++ source files
find_sources() {
    find "$SRC_DIR" -type f \( -name "*.c" -o -name "*.cpp" -o -name "*.cc" \) 2>/dev/null | sort
}

# Compile a single source file to bitcode
# Matches generate_bc.sh flags: textual IR, mem2reg, per-directory flags
compile_one() {
    local src="$1"
    local rel_path="${src#$SRC_DIR/}"

    # Extract extension and compute output path
    local ext="${src##*.}"
    local base="${rel_path%.$ext}"
    local out_path="$OUT_DIR/${base}.ll"
    local out_dir
    out_dir=$(dirname "$out_path")

    mkdir -p "$out_dir"

    # Detect which test directory this file belongs to
    local test_dir="${rel_path%%/*}"

    # Select compiler
    local clang_c="clang"
    local clang_cxx="clang++"
    if command -v clang-18 &>/dev/null; then
        clang_c="clang-18"
        clang_cxx="clang++-18"
    elif command -v clang-17 &>/dev/null; then
        clang_c="clang-17"
        clang_cxx="clang++-17"
    fi

    local clang_cmd="$clang_c"
    case "$src" in
        *.cpp|*.cc) clang_cmd="$clang_cxx" ;;
    esac

    # Build per-directory compiler flags (matching generate_bc.sh)
    # -disable-O0-optnone: strip optnone attribute so mem2reg can promote
    # allocas to SSA registers globally. This produces cleaner SSA IR that
    # mature analyzers operate on. Address-taken allocas survive mem2reg.
    local cflags="-Wno-everything -S -emit-llvm -fno-discard-value-names -Xclang -disable-O0-optnone"

    # Check if this directory needs -DINCLUDEMAIN
    local needs_includemain=0
    for d in $INCLUDEMAIN_DIRS; do
        if [[ "$test_dir" == "$d" ]]; then
            needs_includemain=1
            break
        fi
    done

    if [[ "$needs_includemain" == "1" ]]; then
        # ae_* tests: add -DINCLUDEMAIN, -g, -Wno-implicit-function-declaration
        cflags="$cflags -DINCLUDEMAIN -Wno-implicit-function-declaration -g"
    elif [[ "$test_dir" == "mem_leak" ]] || [[ "$test_dir" == "double_free" ]]; then
        # mem_leak/double_free: add -g for debug info
        cflags="$cflags -g"
    fi
    # All other dirs: no -g, no -DINCLUDEMAIN (matching generate_bc.sh default branch)

    # Compile, capturing stderr for error reporting
    local err_file
    err_file=$(mktemp)
    if $clang_cmd $cflags $INCLUDE_FLAGS "$src" -o "$out_path" 2>"$err_file"; then
        # Run mem2reg promotion pass (matching generate_bc.sh line 119)
        if ! $OPT -S -p=mem2reg "$out_path" -o "$out_path" 2>>"$err_file"; then
            echo "  [FAIL-OPT] $rel_path"
            if [[ "${VERBOSE:-}" == "1" ]]; then
                head -1 "$err_file" | sed 's/^/         /'
            fi
            rm -f "$err_file"
            return 1
        fi
        echo "  [OK] $rel_path"
        rm -f "$err_file"
        return 0
    else
        echo "  [FAIL] $rel_path"
        if [[ "${VERBOSE:-}" == "1" ]]; then
            head -1 "$err_file" | sed 's/^/         /'
        fi
        rm -f "$err_file"
        return 1
    fi
}

export -f compile_one
export SRC_DIR OUT_DIR PTABEN_DIR INCLUDE_FLAGS VERBOSE OPT INCLUDEMAIN_DIRS

# Count source files
total=$(find_sources | wc -l)
echo "Found $total source files"
echo ""

# Compile in parallel
echo "Compiling..."
compiled=0
failed=0

# Use parallel if available, otherwise fallback to sequential
if command -v parallel &> /dev/null; then
    find_sources | parallel -j "$JOBS" compile_one {}
else
    # Fallback: process sequentially with some parallelism via background jobs
    while IFS= read -r src; do
        compile_one "$src" &

        # Limit concurrent jobs
        while [[ $(jobs -r -p | wc -l) -ge $JOBS ]]; do
            wait -n 2>/dev/null || true
        done
    done < <(find_sources)
    wait
fi

# Count results
compiled=$(find "$OUT_DIR" -name "*.ll" 2>/dev/null | wc -l)
echo ""
echo "=== Compilation Complete ==="
echo "Compiled: $compiled / $total"
echo "Output:   $OUT_DIR/"
