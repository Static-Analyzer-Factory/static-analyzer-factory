#!/bin/bash
# Compile Juliet C/C++ test suite (supported CWEs only) with LLVM 18 + mem2reg
# Usage: ./scripts/compile-juliet.sh [--cwe CWE476] [--jobs N] [--verbose] [--clean]
#
# Compiles preprocessed .i files from sv-benchmarks/c/Juliet_Test/ into LLVM
# text IR (.ll) files, organized by CWE category. Only the 15 CWE categories
# that SAF can analyze are compiled.
#
# Output: tests/benchmarks/sv-benchmarks/.compiled-juliet/<CWE>/<testname>.ll

set -euo pipefail

SVCOMP_DIR="tests/benchmarks/sv-benchmarks"
JULIET_DIR="$SVCOMP_DIR/c/Juliet_Test"
OUT_DIR="$SVCOMP_DIR/.compiled-juliet"
JOBS="$(nproc)"
VERBOSE="${VERBOSE:-0}"
CLEAN=0
CWE_FILTER=""

# Supported CWEs (15 categories SAF can analyze)
SUPPORTED_CWES="CWE121 CWE122 CWE124 CWE126 CWE127 CWE190 CWE191 CWE401 CWE415 CWE416 CWE476 CWE590 CWE690 CWE761 CWE789"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --cwe) CWE_FILTER="${2:-}"; shift 2 ;;
        --jobs) JOBS="${2:-$JOBS}"; shift 2 ;;
        --verbose|-v) VERBOSE=1; shift ;;
        --clean) CLEAN=1; shift ;;
        *) shift ;;
    esac
done

# Validate CWE filter
if [[ -n "$CWE_FILTER" ]]; then
    if ! echo "$SUPPORTED_CWES" | grep -qw "$CWE_FILTER"; then
        echo "ERROR: Unsupported CWE: $CWE_FILTER"
        echo "Supported: $SUPPORTED_CWES"
        exit 1
    fi
    SUPPORTED_CWES="$CWE_FILTER"
fi

# Clean if requested
if [[ "$CLEAN" == "1" ]] && [[ -d "$OUT_DIR" ]]; then
    echo "Cleaning previous compilation..."
    rm -rf "$OUT_DIR"
fi

echo "=== Juliet Compilation ==="
echo "Source: $JULIET_DIR"
echo "Output: $OUT_DIR"
echo "Jobs:   $JOBS"
echo "CWEs:   $(echo $SUPPORTED_CWES | wc -w | tr -d ' ')"
echo ""

# Ensure source exists
if [[ ! -d "$JULIET_DIR" ]]; then
    echo "ERROR: Juliet tests not found at $JULIET_DIR"
    echo "Did you run: git submodule update --init tests/benchmarks/sv-benchmarks"
    exit 1
fi

mkdir -p "$OUT_DIR"

# Detect LLVM tools
CLANG="clang"
OPT="opt"
if command -v clang-18 &>/dev/null; then CLANG="clang-18"; fi
if command -v opt-18 &>/dev/null; then OPT="opt-18"; fi
echo "Compiler: $CLANG"
echo "Optimizer: $OPT"
echo ""

# Find .i files for supported CWEs
find_sources() {
    for cwe in $SUPPORTED_CWES; do
        find "$JULIET_DIR" -name "${cwe}*.i" -type f 2>/dev/null
    done | sort
}

total=$(find_sources | wc -l | tr -d ' ')
echo "Found $total source files for supported CWEs"
echo ""

compiled=0
failed=0
skipped=0

# Compile a single .i file to .ll with mem2reg
compile_one() {
    local src="$1"
    local base_name
    base_name=$(basename "$src" .i)

    # Extract CWE from filename for output directory structure
    local cwe
    cwe=$(echo "$base_name" | grep -oE '^CWE[0-9]+')
    if [[ -z "$cwe" ]]; then
        return 2  # skip: no CWE in filename
    fi

    local out_file="$OUT_DIR/$cwe/$base_name.ll"
    mkdir -p "$OUT_DIR/$cwe"

    # Compile: .i -> .ll (text IR) then mem2reg
    local err_file
    err_file=$(mktemp)
    if $CLANG -S -emit-llvm -O0 -Xclang -disable-O0-optnone \
        -Wno-everything "$src" -o "$out_file" 2>"$err_file"; then
        # Apply mem2reg
        if $OPT -S -passes=mem2reg "$out_file" -o "$out_file" 2>>"$err_file"; then
            if [[ "$VERBOSE" == "1" ]]; then echo "  [OK] $base_name"; fi
            rm -f "$err_file"
            return 0
        fi
    fi

    if [[ "$VERBOSE" == "1" ]]; then
        echo "  [FAIL] $base_name"
        head -1 "$err_file" 2>/dev/null | sed 's/^/         /'
    fi
    rm -f "$err_file" "$out_file"
    return 1
}

export -f compile_one
export OUT_DIR CLANG OPT VERBOSE

echo "Compiling..."

# Process each source file; count results
while IFS= read -r src; do
    compile_one "$src"
    status=$?
    if [[ $status -eq 0 ]]; then
        ((compiled++)) || true
    elif [[ $status -eq 2 ]]; then
        ((skipped++)) || true
    else
        ((failed++)) || true
    fi

    # Progress every 500
    count=$((compiled + failed + skipped))
    if [[ $((count % 500)) -eq 0 ]] && [[ $count -gt 0 ]]; then
        echo "  Progress: $count / $total processed ($compiled compiled)..."
    fi
done < <(find_sources)

echo ""
echo "=== Juliet Compilation Complete ==="
echo "Total:    $total"
echo "Compiled: $compiled"
echo "Failed:   $failed"
echo "Skipped:  $skipped"
echo "Output:   $OUT_DIR/"

# Print per-CWE counts
echo ""
echo "Per-CWE:"
for cwe in $SUPPORTED_CWES; do
    count=$(find "$OUT_DIR/$cwe" -name "*.ll" 2>/dev/null | wc -l | tr -d ' ')
    echo "  $cwe: $count files"
done
