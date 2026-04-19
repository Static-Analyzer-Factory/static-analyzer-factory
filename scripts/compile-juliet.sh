#!/bin/bash
# Compile Juliet C/C++ test suite (supported CWEs only) + mem2reg.
# Usage: ./scripts/compile-juliet.sh [--cwe CWE476] [--jobs N] [--verbose] [--clean]
#
# Env:
#   SAF_JULIET_CLANG_VERSION  Pin clang/opt major version (e.g. 22). If
#                             unset, picks newest installed (22 > 18 > …).
#   SAF_JULIET_OUT_DIR        Override the output directory. Defaults to
#                             `.compiled-juliet/` for clang-18 or
#                             `.compiled-juliet-llvm<N>/` otherwise.
#
# Compiles preprocessed .i files from sv-benchmarks/c/Juliet_Test/ into LLVM
# text IR (.ll) files, organized by CWE category. Only the 15 CWE categories
# that SAF can analyze are compiled.
#
# Output: tests/benchmarks/sv-benchmarks/<OUT_DIR>/<CWE>/<testname>.ll

set -euo pipefail

SVCOMP_DIR="tests/benchmarks/sv-benchmarks"
JULIET_DIR="$SVCOMP_DIR/c/Juliet_Test"
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

# Ensure source exists
if [[ ! -d "$JULIET_DIR" ]]; then
    echo "ERROR: Juliet tests not found at $JULIET_DIR"
    echo "Did you run: git submodule update --init tests/benchmarks/sv-benchmarks"
    exit 1
fi

# Resolve LLVM toolchain — explicit pin, else newest available.
resolve_llvm_version() {
    if [[ -n "${SAF_JULIET_CLANG_VERSION:-}" ]]; then
        if command -v "clang-${SAF_JULIET_CLANG_VERSION}" &>/dev/null; then
            echo "$SAF_JULIET_CLANG_VERSION"
            return
        fi
        echo "ERROR: SAF_JULIET_CLANG_VERSION=${SAF_JULIET_CLANG_VERSION} requested but clang-${SAF_JULIET_CLANG_VERSION} not found" >&2
        exit 1
    fi
    for v in 22 21 20 19 18 17; do
        if command -v "clang-${v}" &>/dev/null; then
            echo "$v"
            return
        fi
    done
    echo "ERROR: no versioned clang found (tried 22, 21, 20, 19, 18, 17)" >&2
    exit 1
}

LLVM_VERSION="$(resolve_llvm_version)"
CLANG="clang-${LLVM_VERSION}"
OPT="opt-${LLVM_VERSION}"
if ! command -v "$OPT" &>/dev/null; then
    echo "ERROR: ${OPT} not found; cannot run mem2reg pass" >&2
    exit 1
fi

if [[ -n "${SAF_JULIET_OUT_DIR:-}" ]]; then
    OUT_DIR="$SAF_JULIET_OUT_DIR"
elif [[ "$LLVM_VERSION" == "18" ]]; then
    OUT_DIR="$SVCOMP_DIR/.compiled-juliet"
else
    OUT_DIR="$SVCOMP_DIR/.compiled-juliet-llvm${LLVM_VERSION}"
fi

# Clean if requested (after OUT_DIR is resolved).
if [[ "$CLEAN" == "1" ]] && [[ -d "$OUT_DIR" ]]; then
    echo "Cleaning previous compilation..."
    rm -rf "$OUT_DIR"
fi

mkdir -p "$OUT_DIR"

echo "=== Juliet Compilation ==="
echo "Source:    $JULIET_DIR"
echo "Output:    $OUT_DIR"
echo "Compiler:  $CLANG"
echo "Optimizer: $OPT"
echo "Jobs:      $JOBS"
echo "CWEs:      $(echo $SUPPORTED_CWES | wc -w | tr -d ' ')"
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
