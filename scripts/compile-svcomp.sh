#!/bin/bash
# Compile SV-COMP benchmarks with LLVM 18
# Usage: ./scripts/compile-svcomp.sh [--category CAT] [--jobs N] [--verbose] [--clean]
#
# This script compiles SV-COMP benchmark C files into LLVM bitcode for SAF analysis.
# It parses YAML task definitions to determine compilation settings (data model, etc.).

set -euo pipefail

SVCOMP_DIR="tests/benchmarks/sv-benchmarks"
C_DIR="$SVCOMP_DIR/c"
OUT_DIR="$SVCOMP_DIR/.compiled"
STUBS_H="scripts/sv-comp-stubs.h"
JOBS="$(nproc)"
VERBOSE="${VERBOSE:-0}"
CLEAN=0
CATEGORY=""

# Parse optional arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --category)
            CATEGORY="${2:-}"
            shift 2
            ;;
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

echo "=== SV-COMP Compilation ==="
echo "Source: $C_DIR"
echo "Output: $OUT_DIR"
echo "Jobs:   $JOBS"
if [[ -n "$CATEGORY" ]]; then
    echo "Filter: $CATEGORY"
fi
echo ""

# Ensure source exists
if [[ ! -d "$C_DIR" ]]; then
    echo "ERROR: SV-COMP benchmarks not found at $C_DIR"
    echo "Did you run: git submodule update --init tests/benchmarks/sv-benchmarks"
    exit 1
fi

# Create output directory
mkdir -p "$OUT_DIR"

# Get clang version
get_clang() {
    if command -v clang-18 &>/dev/null; then
        echo "clang-18"
    elif command -v clang-17 &>/dev/null; then
        echo "clang-17"
    else
        echo "clang"
    fi
}

CLANG="$(get_clang)"
echo "Compiler: $CLANG"
echo ""

# Common flags for analysis-friendly bitcode
COMMON_FLAGS="-g -emit-llvm -c -Wno-everything"

# Function to parse YAML and get data model
get_data_model() {
    local yml_file="$1"
    if [[ -f "$yml_file" ]]; then
        # Simple grep for data_model - avoid complex YAML parsing in bash
        local model
        model=$(grep -oP 'data_model:\s*\K\S+' "$yml_file" 2>/dev/null | tr -d "'" || echo "")
        case "$model" in
            ILP32) echo "-m32" ;;
            LP64) echo "-m64" ;;
            *) echo "" ;;  # Default: let clang choose
        esac
    else
        echo ""
    fi
}

# Function to get input files from YAML
get_input_files() {
    local yml_file="$1"
    local yml_dir
    yml_dir=$(dirname "$yml_file")

    if [[ -f "$yml_file" ]]; then
        # Extract input_files value (handles both string and list formats)
        grep -oP "input_files:\s*['\"]?\K[^'\"\n]+" "$yml_file" 2>/dev/null | while read -r f; do
            echo "$yml_dir/$f"
        done
    fi
}

# Compile a single source file
compile_source() {
    local src="$1"
    local yml="$2"
    local out_base="$3"

    # Get data model from YAML
    local model_flag
    model_flag=$(get_data_model "$yml")

    # Determine output path
    local out_file="$OUT_DIR/${out_base}.bc"
    local out_dir
    out_dir=$(dirname "$out_file")
    mkdir -p "$out_dir"

    # Include stubs and benchmark directory
    local include_flags="-include $STUBS_H -I$(dirname "$src")"

    # Compile
    if $CLANG $COMMON_FLAGS $model_flag $include_flags "$src" -o "$out_file" 2>/dev/null; then
        if [[ "$VERBOSE" == "1" ]]; then
            echo "  [OK] $out_base"
        fi
        return 0
    else
        if [[ "$VERBOSE" == "1" ]]; then
            echo "  [FAIL] $out_base"
        fi
        return 1
    fi
}

# Find all YAML task definitions
find_tasks() {
    if [[ -n "$CATEGORY" ]]; then
        # Filter by category/subcategory name
        find "$C_DIR" -path "*$CATEGORY*" -name "*.yml" -type f 2>/dev/null | sort
    else
        find "$C_DIR" -name "*.yml" -type f 2>/dev/null | sort
    fi
}

# Process tasks
compiled=0
failed=0
skipped=0
total=0

echo "Scanning for task definitions..."
task_count=$(find_tasks | wc -l | tr -d ' ')
echo "Found $task_count task files"
echo ""

if [[ "$task_count" -eq 0 ]]; then
    echo "No tasks found. Check --category filter."
    exit 0
fi

echo "Compiling..."

# Process each YAML task definition
while IFS= read -r yml; do
    ((total++)) || true

    # Skip non-C tasks
    if ! grep -q "language: C" "$yml" 2>/dev/null; then
        ((skipped++)) || true
        continue
    fi

    # Get the source file path
    yml_dir=$(dirname "$yml")
    rel_dir="${yml_dir#$C_DIR/}"

    # Extract input file name
    input_file=$(grep -oP "input_files:\s*['\"]?\K[^'\"\n]+" "$yml" 2>/dev/null | head -1 || echo "")

    if [[ -z "$input_file" ]]; then
        ((skipped++)) || true
        continue
    fi

    # Full path to source
    src="$yml_dir/$input_file"

    if [[ ! -f "$src" ]]; then
        # Try preprocessed (.i) file
        if [[ ! -f "$src" ]]; then
            ((skipped++)) || true
            continue
        fi
    fi

    # Output base name (preserving directory structure)
    base_name=$(basename "$yml" .yml)
    out_base="$rel_dir/$base_name"

    if compile_source "$src" "$yml" "$out_base"; then
        ((compiled++)) || true
    else
        ((failed++)) || true
    fi

    # Progress indicator every 100 files
    if [[ $((total % 100)) -eq 0 ]]; then
        echo "  Progress: $total tasks processed..."
    fi
done < <(find_tasks)

echo ""
echo "=== Compilation Complete ==="
echo "Tasks:    $total"
echo "Compiled: $compiled"
echo "Failed:   $failed"
echo "Skipped:  $skipped (non-C or missing source)"
echo "Output:   $OUT_DIR/"
