# Juliet Benchmark Integration — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Compile Juliet C/C++ tests (15 CWEs) with LLVM 18 + mem2reg, run SAF property analyzers, compute precision/recall/F1 per CWE.

**Architecture:** Reuse existing SV-COMP YAML pipeline (`SvCompRunner`) with a Juliet-specific wrapper (`juliet.rs`) that extracts CWE categories from filenames, forces aggressive mode, and computes precision/recall/F1 from verdict outcomes.

**Tech Stack:** Rust (saf-bench crate), Bash (compile script), clang-18/opt-18 (compilation), Makefile

**Design doc:** `docs/plans/2026-02-21-juliet-benchmark-design.md`

---

### Task 1: Compilation Script

**Files:**
- Create: `scripts/compile-juliet.sh`

**Step 1: Write the compilation script**

Create `scripts/compile-juliet.sh` following the pattern from `scripts/compile-ptaben.sh`:

```bash
#!/bin/bash
# Compile Juliet C/C++ test suite (supported CWEs only) with LLVM 18 + mem2reg
# Usage: ./scripts/compile-juliet.sh [--cwe CWE476] [--jobs N] [--verbose] [--clean]
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

# Find YAML files for supported CWEs
find_yamls() {
    for cwe in $SUPPORTED_CWES; do
        find "$JULIET_DIR" -name "${cwe}*.yml" -type f 2>/dev/null
    done | sort
}

total=$(find_yamls | wc -l | tr -d ' ')
echo "Found $total task files for supported CWEs"
echo ""

compiled=0
failed=0
skipped=0

# Compile a single .i file to .ll with mem2reg
compile_one() {
    local yml="$1"
    local yml_dir
    yml_dir=$(dirname "$yml")

    # Extract input file from YAML
    local input_file
    input_file=$(grep -oE "input_files:\s*['\"]?[^'\"[:space:]]+" "$yml" 2>/dev/null | sed "s/input_files:\s*['\"]*//" | head -1)
    if [[ -z "$input_file" ]]; then
        return 2  # skip
    fi

    local src="$yml_dir/$input_file"
    if [[ ! -f "$src" ]]; then
        return 2  # skip
    fi

    # Skip non-C
    if ! grep -q "language: C" "$yml" 2>/dev/null; then
        return 2  # skip
    fi

    # Get data model flag
    local model_flag=""
    local model
    model=$(grep -oE "data_model:\s*\S+" "$yml" 2>/dev/null | sed "s/data_model:\s*//" | tr -d "'" || echo "")
    case "$model" in
        ILP32) model_flag="-m32" ;;
        LP64) model_flag="-m64" ;;
    esac

    # Extract CWE from filename for output directory structure
    local base_name
    base_name=$(basename "$yml" .yml)
    local cwe
    cwe=$(echo "$base_name" | grep -oE '^CWE[0-9]+')
    local out_file="$OUT_DIR/$cwe/$base_name.ll"
    local out_dir
    out_dir=$(dirname "$out_file")
    mkdir -p "$out_dir"

    # Compile: .i -> .ll (text IR) then mem2reg
    local err_file
    err_file=$(mktemp)
    if $CLANG -S -emit-llvm -O0 -Xclang -disable-O0-optnone \
        -Wno-everything $model_flag "$src" -o "$out_file" 2>"$err_file"; then
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
export OUT_DIR CLANG OPT VERBOSE SUPPORTED_CWES

echo "Compiling..."
# Process each YAML; count results
while IFS= read -r yml; do
    compile_one "$yml"
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
done < <(find_yamls)

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
```

Key differences from `compile-ptaben.sh`:
- Filters to 15 supported CWEs via `SUPPORTED_CWES` list
- Compiles `.i` (preprocessed) files, not raw `.c`
- Uses `--cwe` filter flag (not `--category`)
- Output organized by CWE: `.compiled-juliet/CWE476/...`
- Applies `-Xclang -disable-O0-optnone` then `opt -passes=mem2reg` (same as PTABen)
- Sequential with progress counter (not parallel — the inner loop approach matches compile-svcomp.sh; parallel can be added later if needed)

**Step 2: Make it executable and test compilation of a single CWE**

```bash
chmod +x scripts/compile-juliet.sh
# Inside Docker:
./scripts/compile-juliet.sh --cwe CWE476 --verbose
```

Expected: ~468 `.ll` files in `.compiled-juliet/CWE476/`

**Step 3: Commit**

```bash
git add scripts/compile-juliet.sh
git commit -m "feat(bench): add Juliet compilation script for supported CWEs"
```

---

### Task 2: Juliet Runner Module

**Files:**
- Create: `crates/saf-bench/src/juliet.rs`
- Modify: `crates/saf-bench/src/lib.rs` (add `pub mod juliet;`)

**Step 1: Register the module**

In `crates/saf-bench/src/lib.rs`, add `pub mod juliet;` alongside existing modules.

**Step 2: Write `juliet.rs`**

The module wraps `SvCompRunner` with CWE-aware categorization and precision/recall scoring:

```rust
//! Juliet C/C++ Test Suite benchmark integration.
//!
//! Wraps the SV-COMP runner with Juliet-specific CWE extraction
//! and precision/recall/F1 scoring.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::svcomp::{
    PropertyAnalysisConfig, SvCompConfig, SvCompRunner, SvCompOutcome, TaskResult,
};

/// Supported CWEs and their descriptions.
const SUPPORTED_CWES: &[(&str, &str)] = &[
    ("CWE121", "Stack Buffer Overflow"),
    ("CWE122", "Heap Buffer Overflow"),
    ("CWE124", "Buffer Underwrite"),
    ("CWE126", "Buffer Over-read"),
    ("CWE127", "Buffer Under-read"),
    ("CWE190", "Integer Overflow"),
    ("CWE191", "Integer Underflow"),
    ("CWE401", "Memory Leak"),
    ("CWE415", "Double Free"),
    ("CWE416", "Use After Free"),
    ("CWE476", "NULL Pointer Dereference"),
    ("CWE590", "Free of Non-Heap Variable"),
    ("CWE690", "NULL Deref from Return"),
    ("CWE761", "Free Pointer Not at Start"),
    ("CWE789", "Uncontrolled Memory Allocation"),
];

/// Configuration for Juliet benchmarks.
#[derive(Debug, Clone)]
pub struct JulietConfig {
    /// Directory containing compiled .ll files organized by CWE.
    pub compiled_dir: PathBuf,
    /// Filter to a single CWE (e.g., "CWE476").
    pub cwe_filter: Option<String>,
    /// Number of parallel jobs.
    pub jobs: usize,
    /// Timeout per task in seconds.
    pub timeout_secs: u64,
    /// Z3 timeout in milliseconds.
    pub z3_timeout_ms: u64,
}

impl Default for JulietConfig {
    fn default() -> Self {
        Self {
            compiled_dir: PathBuf::from("tests/benchmarks/sv-benchmarks/.compiled-juliet"),
            cwe_filter: None,
            jobs: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4),
            timeout_secs: 300,
            z3_timeout_ms: 5000,
        }
    }
}

/// Juliet benchmark runner.
pub struct JulietRunner {
    config: JulietConfig,
}

impl JulietRunner {
    pub fn new(config: JulietConfig) -> Self {
        Self { config }
    }

    /// Run Juliet benchmarks and return summary with precision/recall/F1.
    pub fn run(&self) -> Result<JulietSummary> {
        let start = Instant::now();

        // The compiled directory has per-CWE subdirectories.
        // The YAML files live alongside the source .i files in sv-benchmarks/c/Juliet_Test/.
        // SvCompRunner discovers YAMLs by walking the sv-benchmarks/c/ tree,
        // then checks for matching .bc/.ll in compiled_dir.
        //
        // We configure SvCompRunner to:
        // 1. Point compiled_dir at .compiled-juliet/
        // 2. Use category_filter = "Juliet_Test" to restrict to Juliet YAMLs
        // 3. Use aggressive mode (conservative = false)

        let svcomp_config = SvCompConfig {
            compiled_dir: self.config.compiled_dir.clone(),
            category_filter: self.config.cwe_filter.as_ref().map_or_else(
                || Some("Juliet_Test".to_string()),
                |cwe| Some(cwe.clone()),
            ),
            property_filter: None,
            jobs: self.config.jobs,
            timeout_secs: self.config.timeout_secs,
            analysis_config: PropertyAnalysisConfig {
                z3_timeout_ms: self.config.z3_timeout_ms,
                conservative: false, // Always aggressive for precision/recall
                ..Default::default()
            },
        };

        let runner = SvCompRunner::new(svcomp_config);
        let svcomp_summary = runner.run()?;

        // Post-process: extract CWE from task filenames and compute metrics
        let mut by_cwe: BTreeMap<String, CweMetrics> = BTreeMap::new();

        for cat_summary in &svcomp_summary.categories {
            for result in &cat_summary.results {
                let cwe = extract_cwe(&result.task.path.to_string_lossy());
                let Some(cwe) = cwe else { continue };

                // Only count supported CWEs
                if !SUPPORTED_CWES.iter().any(|(c, _)| *c == cwe) {
                    continue;
                }

                let metrics = by_cwe.entry(cwe.clone()).or_insert_with(|| {
                    let desc = SUPPORTED_CWES
                        .iter()
                        .find(|(c, _)| *c == cwe)
                        .map_or("", |(_, d)| d);
                    CweMetrics {
                        cwe: cwe.clone(),
                        description: desc.to_string(),
                        ..Default::default()
                    }
                });

                metrics.total += 1;

                // Classify based on verdict vs expected
                match result.outcome {
                    Some(SvCompOutcome::FalseCorrect) => metrics.tp += 1,
                    Some(SvCompOutcome::FalseIncorrect) => metrics.fp += 1,
                    Some(SvCompOutcome::TrueIncorrect) => metrics.fn_count += 1,
                    Some(SvCompOutcome::TrueCorrect) => metrics.tn += 1,
                    Some(SvCompOutcome::Unknown) => {
                        // UNKNOWN on bad file = FN, UNKNOWN on good file = TN
                        if result.expected == Some(false) {
                            metrics.fn_count += 1;
                        } else {
                            metrics.tn += 1;
                        }
                    }
                    None => {
                        if result.expected == Some(false) {
                            metrics.fn_count += 1;
                        } else {
                            metrics.tn += 1;
                        }
                    }
                }
            }
        }

        // Compute per-CWE precision/recall/F1
        let cwe_results: Vec<CweResult> = by_cwe
            .values()
            .map(|m| {
                let precision = safe_div(m.tp as f64, (m.tp + m.fp) as f64);
                let recall = safe_div(m.tp as f64, (m.tp + m.fn_count) as f64);
                let f1 = safe_div(2.0 * precision * recall, precision + recall);
                CweResult {
                    cwe: m.cwe.clone(),
                    description: m.description.clone(),
                    total: m.total,
                    tp: m.tp,
                    fp: m.fp,
                    fn_count: m.fn_count,
                    tn: m.tn,
                    precision,
                    recall,
                    f1,
                }
            })
            .collect();

        // Compute aggregate
        let agg_tp: usize = cwe_results.iter().map(|c| c.tp).sum();
        let agg_fp: usize = cwe_results.iter().map(|c| c.fp).sum();
        let agg_fn: usize = cwe_results.iter().map(|c| c.fn_count).sum();
        let agg_tn: usize = cwe_results.iter().map(|c| c.tn).sum();
        let agg_precision = safe_div(agg_tp as f64, (agg_tp + agg_fp) as f64);
        let agg_recall = safe_div(agg_tp as f64, (agg_tp + agg_fn) as f64);
        let agg_f1 = safe_div(2.0 * agg_precision * agg_recall, agg_precision + agg_recall);

        let total_time = start.elapsed();

        Ok(JulietSummary {
            suite: "juliet".to_string(),
            total_tasks: svcomp_summary.total_tasks,
            aggregate: AggregateMetrics {
                tp: agg_tp,
                fp: agg_fp,
                fn_count: agg_fn,
                tn: agg_tn,
                precision: agg_precision,
                recall: agg_recall,
                f1: agg_f1,
            },
            by_cwe: cwe_results,
            svcomp_scoring: SvCompScoring {
                total_score: svcomp_summary.total_score,
                max_possible_score: svcomp_summary.max_possible_score,
                true_correct: svcomp_summary.true_correct,
                false_correct: svcomp_summary.false_correct,
                true_incorrect: svcomp_summary.true_incorrect,
                false_incorrect: svcomp_summary.false_incorrect,
                unknown: svcomp_summary.unknown,
            },
            timing_secs: total_time.as_secs_f64(),
        })
    }
}

/// Extract CWE number from a file path (e.g., "CWE476_NULL..." -> "CWE476").
fn extract_cwe(path: &str) -> Option<String> {
    // Find CWE pattern anywhere in the path
    let idx = path.find("CWE")?;
    let rest = &path[idx..];
    // Take CWE + digits
    let end = rest[3..]
        .find(|c: char| !c.is_ascii_digit())
        .map_or(rest.len(), |i| i + 3);
    let cwe = &rest[..end];
    if cwe.len() > 3 {
        Some(cwe.to_string())
    } else {
        None
    }
}

/// Safe division that returns 0.0 for 0/0.
fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 { 0.0 } else { numerator / denominator }
}

// --- Data types ---

#[derive(Default)]
struct CweMetrics {
    cwe: String,
    description: String,
    total: usize,
    tp: usize,
    fp: usize,
    fn_count: usize,
    tn: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JulietSummary {
    pub suite: String,
    pub total_tasks: usize,
    pub aggregate: AggregateMetrics,
    pub by_cwe: Vec<CweResult>,
    pub svcomp_scoring: SvCompScoring,
    pub timing_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetrics {
    pub tp: usize,
    pub fp: usize,
    pub fn_count: usize,
    pub tn: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CweResult {
    pub cwe: String,
    pub description: String,
    pub total: usize,
    pub tp: usize,
    pub fp: usize,
    pub fn_count: usize,
    pub tn: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvCompScoring {
    pub total_score: i32,
    pub max_possible_score: i32,
    pub true_correct: usize,
    pub false_correct: usize,
    pub true_incorrect: usize,
    pub false_incorrect: usize,
    pub unknown: usize,
}

/// Print human-readable Juliet results.
pub fn print_human(summary: &JulietSummary) {
    eprintln!();
    eprintln!("=== Juliet Benchmark Results ===");
    eprintln!("Tasks: {} | Time: {:.1}s", summary.total_tasks, summary.timing_secs);
    eprintln!();
    eprintln!(
        "{:<8} {:<32} {:>5} {:>5} {:>5} {:>5} {:>5} {:>7} {:>7} {:>7}",
        "CWE", "Description", "Total", "TP", "FP", "FN", "TN", "Prec", "Recall", "F1"
    );
    eprintln!("{}", "-".repeat(97));

    for cwe in &summary.by_cwe {
        eprintln!(
            "{:<8} {:<32} {:>5} {:>5} {:>5} {:>5} {:>5} {:>6.1}% {:>6.1}% {:>6.1}%",
            cwe.cwe,
            truncate(&cwe.description, 32),
            cwe.total,
            cwe.tp,
            cwe.fp,
            cwe.fn_count,
            cwe.tn,
            cwe.precision * 100.0,
            cwe.recall * 100.0,
            cwe.f1 * 100.0,
        );
    }

    let a = &summary.aggregate;
    eprintln!("{}", "-".repeat(97));
    eprintln!(
        "{:<8} {:<32} {:>5} {:>5} {:>5} {:>5} {:>5} {:>6.1}% {:>6.1}% {:>6.1}%",
        "ALL", "",
        summary.total_tasks, a.tp, a.fp, a.fn_count, a.tn,
        a.precision * 100.0, a.recall * 100.0, a.f1 * 100.0,
    );
    eprintln!();

    // SV-COMP scoring
    let s = &summary.svcomp_scoring;
    eprintln!(
        "SV-COMP Score: {} / {} ({:.0}%)",
        s.total_score,
        s.max_possible_score,
        if s.max_possible_score > 0 {
            f64::from(s.total_score) / f64::from(s.max_possible_score) * 100.0
        } else {
            0.0
        }
    );
    if s.true_incorrect > 0 || s.false_incorrect > 0 {
        eprintln!(
            "  Incorrect: {} TRUE incorrect (-32), {} FALSE incorrect (-16)",
            s.true_incorrect, s.false_incorrect
        );
    }
    eprintln!();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cwe() {
        assert_eq!(extract_cwe("CWE476_NULL_Pointer_Dereference__int_01_bad.yml"), Some("CWE476".to_string()));
        assert_eq!(extract_cwe("/path/to/CWE190_Integer_Overflow__int_add_01.ll"), Some("CWE190".to_string()));
        assert_eq!(extract_cwe("no_cwe_here.yml"), None);
        assert_eq!(extract_cwe("CWE"), None);  // Just "CWE" without digits
    }

    #[test]
    fn test_safe_div() {
        assert_eq!(safe_div(0.0, 0.0), 0.0);
        assert_eq!(safe_div(5.0, 10.0), 0.5);
        assert_eq!(safe_div(0.0, 10.0), 0.0);
    }

    #[test]
    fn test_supported_cwes_sorted() {
        // Verify the list is sorted for consistent output
        let cwes: Vec<&str> = SUPPORTED_CWES.iter().map(|(c, _)| *c).collect();
        let mut sorted = cwes.clone();
        sorted.sort();
        assert_eq!(cwes, sorted);
    }
}
```

**Important**: `SvCompRunner::discover_tasks()` walks `compiled_dir.parent()/c/` for YAML files, then checks if `.bc` exists in `compiled_dir`. Since our compiled dir is `.compiled-juliet/` (sibling of `c/`), the YAML discovery will find Juliet YAMLs in `c/Juliet_Test/`. However, the runner looks for `.bc` extension while we compile to `.ll`. We need to verify this works — the `SvCompRunner` checks `bc_path.exists()` where `bc_path = compiled_dir.join(rel_path.with_extension("bc"))`. Our files have `.ll` extension, so this won't match.

**Fix**: We need to modify how `JulietRunner` discovers tasks. Instead of using `SvCompRunner` directly, we should:
1. Walk `.compiled-juliet/` to find `.ll` files
2. Find matching YAML files in `sv-benchmarks/c/Juliet_Test/`
3. Parse YAML for expected verdict
4. Run property analysis directly

This means `JulietRunner.run()` should NOT delegate to `SvCompRunner.run()` but instead use `SvCompRunner`'s individual components: `SvCompTask::from_yaml_file()`, `analyze_property()`, and `compute_outcome()`.

The revised `run()` method should:
1. Walk `compiled_dir` for `.ll` files
2. For each `.ll`, find the corresponding `.yml` in `sv-benchmarks/c/Juliet_Test/`
3. Parse the YAML to get expected verdict and property
4. Load the `.ll`, run `analyze_property()`, compute outcome
5. Aggregate into CWE-level precision/recall/F1

**Step 3: Run tests**

```bash
# Inside Docker
cargo test -p saf-bench -- juliet --nocapture
```

Expected: unit tests for `extract_cwe`, `safe_div`, `supported_cwes_sorted` pass.

**Step 4: Commit**

```bash
git add crates/saf-bench/src/juliet.rs crates/saf-bench/src/lib.rs
git commit -m "feat(bench): add Juliet runner module with precision/recall/F1 scoring"
```

---

### Task 3: CLI Subcommand

**Files:**
- Modify: `crates/saf-bench/src/main.rs`

**Step 1: Add `Juliet` variant to the `Commands` enum**

Add alongside existing `Ptaben`, `Svcomp`, `Cruxbc` variants:

```rust
/// Run Juliet C/C++ benchmark suite (precision/recall/F1 per CWE)
Juliet {
    /// Directory containing compiled .ll files
    #[arg(long, default_value = "tests/benchmarks/sv-benchmarks/.compiled-juliet")]
    compiled_dir: PathBuf,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Write JSON results to a file (implies --json)
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Filter to single CWE (e.g., "CWE476")
    #[arg(long)]
    cwe: Option<String>,

    /// Number of parallel jobs (default: number of CPUs)
    #[arg(long, short = 'j')]
    jobs: Option<usize>,

    /// Timeout per task in seconds
    #[arg(long, default_value = "300")]
    timeout: u64,

    /// Z3 timeout in milliseconds per query
    #[arg(long, default_value = "5000")]
    z3_timeout: u64,
},
```

**Step 2: Add match arm in `main()`**

```rust
Commands::Juliet {
    compiled_dir,
    json,
    output,
    cwe,
    jobs,
    timeout,
    z3_timeout,
} => {
    run_juliet(compiled_dir, json || output.is_some(), output, cwe, jobs, timeout, z3_timeout)?;
}
```

**Step 3: Add `run_juliet` function**

```rust
use saf_bench::juliet::{JulietConfig, JulietRunner};

fn run_juliet(
    compiled_dir: PathBuf,
    json_output: bool,
    output_path: Option<PathBuf>,
    cwe: Option<String>,
    jobs: Option<usize>,
    timeout: u64,
    z3_timeout: u64,
) -> Result<()> {
    if let Some(j) = jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j)
            .build_global()
            .ok();
    }

    let config = JulietConfig {
        compiled_dir,
        cwe_filter: cwe,
        jobs: jobs.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4)
        }),
        timeout_secs: timeout,
        z3_timeout_ms: z3_timeout,
    };

    info!("Running Juliet benchmarks...");
    let runner = JulietRunner::new(config);
    let summary = runner.run()?;

    if let Some(ref path) = output_path {
        let json = serde_json::to_string_pretty(&summary)?;
        std::fs::write(path, &json)
            .with_context(|| format!("Failed to write JSON to {}", path.display()))?;
        info!("Results written to {}", path.display());
        saf_bench::juliet::print_human(&summary);
    } else if json_output {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        saf_bench::juliet::print_human(&summary);
    }

    Ok(())
}
```

**Step 4: Verify it compiles**

```bash
cargo build -p saf-bench
```

**Step 5: Commit**

```bash
git add crates/saf-bench/src/main.rs
git commit -m "feat(bench): add juliet CLI subcommand"
```

---

### Task 4: Makefile Targets

**Files:**
- Modify: `Makefile`

**Step 1: Add Juliet targets after the SV-COMP section**

Add after `clean-svcomp:` (around line 143), before the CruxBC section:

```makefile
# --- Juliet Benchmark Suite (NIST CWE Test Suite) ---
# Precision/Recall/F1 scoring per CWE category

compile-juliet: ## Compile Juliet tests (15 supported CWEs) with LLVM 18 + mem2reg
	@echo "Compiling Juliet test suite..."
	docker compose run --rm dev ./scripts/compile-juliet.sh --verbose

test-juliet: ## Run Juliet benchmarks with precision/recall/F1 (CWE=CWE476 to filter)
	@echo "Running Juliet benchmarks..."
	docker compose run --rm dev cargo run --release -p saf-bench -- juliet \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet \
		$(if $(CWE),--cwe $(CWE),)

test-juliet-json: ## Run Juliet benchmarks with JSON output to file
	docker compose run --rm dev cargo run --release -p saf-bench -- juliet \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled-juliet \
		$(if $(CWE),--cwe $(CWE),) \
		-o /workspace/tests/benchmarks/sv-benchmarks/juliet-results.json

juliet-categories: ## List supported Juliet CWE categories
	@echo "Supported CWEs (15 categories, ~24,815 tests):"
	@echo ""
	@echo "  Memory Safety (valid-memsafety):"
	@echo "    CWE121  Stack Buffer Overflow"
	@echo "    CWE122  Heap Buffer Overflow"
	@echo "    CWE124  Buffer Underwrite"
	@echo "    CWE126  Buffer Over-read"
	@echo "    CWE127  Buffer Under-read"
	@echo "    CWE401  Memory Leak"
	@echo "    CWE415  Double Free"
	@echo "    CWE416  Use After Free"
	@echo "    CWE476  NULL Pointer Dereference"
	@echo "    CWE590  Free of Non-Heap Variable"
	@echo "    CWE690  NULL Deref from Return"
	@echo "    CWE761  Free Pointer Not at Start"
	@echo "    CWE789  Uncontrolled Memory Allocation"
	@echo ""
	@echo "  Integer Overflow (no-overflow):"
	@echo "    CWE190  Integer Overflow"
	@echo "    CWE191  Integer Underflow"
	@echo ""
	@echo "Usage: make test-juliet CWE=CWE476"

clean-juliet: ## Remove compiled Juliet bitcode
	rm -rf tests/benchmarks/sv-benchmarks/.compiled-juliet
```

Also add the targets to the `.PHONY` list at the top of the Makefile.

**Step 2: Verify make targets**

```bash
make juliet-categories
```

Expected: prints the CWE list.

**Step 3: Commit**

```bash
git add Makefile
git commit -m "feat(bench): add Juliet make targets"
```

---

### Task 5: Integration Test (compile + run on single CWE)

**Step 1: Compile CWE476 inside Docker**

```bash
make compile-juliet CWE=CWE476
# Or: docker compose run --rm dev ./scripts/compile-juliet.sh --cwe CWE476 --verbose
```

Wait for compilation (~468 files, ~1-2 min).

**Step 2: Run Juliet benchmarks on CWE476**

```bash
make test-juliet CWE=CWE476
```

Expected output: table showing TP/FP/FN/TN and precision/recall/F1 for CWE476.

**Step 3: Run with JSON output**

```bash
make test-juliet-json CWE=CWE476
```

Verify JSON file at `tests/benchmarks/sv-benchmarks/juliet-results.json`.

**Step 4: Fix any issues found during integration testing**

Address compilation errors, path mismatches, or scoring bugs.

**Step 5: Commit fixes**

```bash
git add -A
git commit -m "fix(bench): integration test fixes for Juliet benchmarks"
```

---

### Task 6: Full Compilation + Baseline Run

**Step 1: Compile all 15 CWEs**

```bash
make compile-juliet
```

Expected: ~24,815 `.ll` files, 10-30 min.

**Step 2: Run full suite with JSON output**

```bash
make test-juliet-json
```

This will take a long time (potentially hours). Run in background.

**Step 3: Analyze results**

Read the JSON output and verify:
- All 15 CWEs have results
- Precision/recall/F1 numbers are reasonable (not all 0 or all 1)
- SV-COMP scoring produces positive scores

**Step 4: Commit baseline results**

```bash
git add tests/benchmarks/sv-benchmarks/juliet-results.json
git commit -m "bench: add Juliet baseline results for 15 CWEs"
```

---

### Task 7: Update CLAUDE.md and PROGRESS.md

**Files:**
- Modify: `CLAUDE.md` (add Juliet benchmark section)
- Modify: `plans/PROGRESS.md` (add session log)

**Step 1: Add Juliet section to CLAUDE.md**

Under the "PTABen Benchmark Testing" section, add a "Juliet Benchmark Testing" section:

```markdown
**Juliet Benchmark Testing:**
- NIST Juliet C/C++ v1.3 integrated via SV-COMP YAML task definitions
- 15 supported CWEs (~24,815 tests) across `valid-memsafety` and `no-overflow` properties
- Compiled with `make compile-juliet` (LLVM 18 + mem2reg)
- Output in `.compiled-juliet/<CWE>/` organized by CWE number
- **Always uses aggressive mode** for precision/recall measurement
- Commands:
  ```bash
  make compile-juliet                # One-time compilation
  make test-juliet                   # Run all with precision/recall/F1
  make test-juliet CWE=CWE476       # Single CWE
  make test-juliet-json              # JSON output to file
  make juliet-categories             # List supported CWEs
  ```
```

**Step 2: Update PROGRESS.md with session log**

**Step 3: Commit**

```bash
git add CLAUDE.md plans/PROGRESS.md
git commit -m "docs: add Juliet benchmark documentation"
```
