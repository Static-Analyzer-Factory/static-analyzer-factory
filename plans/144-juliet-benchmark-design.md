# Juliet Benchmark Integration for SAF

## Overview

Integrate the NIST Juliet C/C++ Test Suite (v1.3) from SV-COMP into SAF's benchmark infrastructure. Compile supported CWE categories with LLVM 18 + mem2reg, run SAF's property analyzers, and compute precision/recall/F1 per CWE and aggregate.

## Target CWEs (15 categories, ~24,815 tests)

### `valid-memsafety` property (~18,583 tests)

| CWE | Description | SAF Detection |
|-----|-------------|---------------|
| CWE121 | Stack Buffer Overflow | AbsInt GEP checker |
| CWE122 | Heap Buffer Overflow | AbsInt GEP checker |
| CWE124 | Buffer Underwrite | AbsInt negative GEP |
| CWE126 | Buffer Over-read | AbsInt GEP checker |
| CWE127 | Buffer Under-read | AbsInt negative GEP |
| CWE401 | Memory Leak | SVFG MustNotReach checker |
| CWE415 | Double Free | SVFG MultiReach checker |
| CWE416 | Use After Free | SVFG MayReach checker |
| CWE476 | NULL Pointer Dereference | Nullness domain + SVFG checker |
| CWE590 | Free of Non-Heap Variable | SVFG checker |
| CWE690 | NULL from Return | Nullness domain |
| CWE761 | Free Pointer Not at Start | SVFG checker |
| CWE789 | Uncontrolled Memory Alloc | AbsInt interval |

### `no-overflow` property (~6,232 tests)

| CWE | Description | SAF Detection |
|-----|-------------|---------------|
| CWE190 | Integer Overflow | AbsInt interval domain |
| CWE191 | Integer Underflow | AbsInt interval domain |

## Architecture

### Approach: SV-COMP YAML reuse

Juliet tests already have SV-COMP 2.0 YAML task definitions with `expected_verdict: true/false` and property mappings. We reuse the existing `SvCompRunner` pipeline and add a Juliet-specific wrapper for CWE extraction and precision/recall scoring.

### New files

```
scripts/compile-juliet.sh              # Compile .i -> .ll with mem2reg
crates/saf-bench/src/juliet.rs         # Juliet runner + precision/recall scoring
crates/saf-bench/src/main.rs           # Add `juliet` CLI subcommand
Makefile                               # Add juliet make targets
```

### Compilation pipeline (`scripts/compile-juliet.sh`)

- Input: `.i` files from `tests/benchmarks/sv-benchmarks/c/Juliet_Test/`
- Filter: only supported CWEs (15 listed above)
- Compile: `clang-18 -S -emit-llvm -O0 -Xclang -disable-O0-optnone <file>.i` then `opt-18 -passes=mem2reg -S`
- Output: `tests/benchmarks/sv-benchmarks/.compiled-juliet/<CWE>/.../*.ll`
- Parallel: `xargs -P$(nproc)`

### Juliet runner (`juliet.rs`)

Wraps `SvCompRunner` with:

1. **CWE extraction**: parse CWE number from filename (e.g., `CWE476_NULL_Pointer...` -> `CWE476`)
2. **Aggressive mode**: always enabled (no conservative toggle) — we want definitive verdicts for precision/recall measurement
3. **Precision/recall/F1**: computed from SV-COMP verdict outcomes

### Scoring model (per-function verdict)

| SAF Verdict | Expected (YAML) | Classification |
|-------------|-----------------|----------------|
| FALSE (bug) | false (`_bad`) | True Positive |
| FALSE (bug) | true (`_good`) | False Positive |
| TRUE / UNKNOWN | false (`_bad`) | False Negative |
| TRUE / UNKNOWN | true (`_good`) | True Negative |

- **Precision** = TP / (TP + FP)
- **Recall** = TP / (TP + FN)
- **F1** = 2 * P * R / (P + R)

### Output format

```json
{
  "suite": "juliet",
  "total_tasks": 24815,
  "aggregate": {
    "precision": 0.85, "recall": 0.42, "f1": 0.56,
    "tp": 2100, "fp": 370, "fn": 2900, "tn": 19445
  },
  "by_cwe": [
    {
      "cwe": "CWE476", "description": "NULL Pointer Dereference",
      "total": 468, "precision": 0.92, "recall": 0.55, "f1": 0.69,
      "tp": 120, "fp": 10, "fn": 98, "tn": 240
    }
  ],
  "svcomp_scoring": { "total_score": 1234, "max_possible_score": 5000 },
  "timing_secs": 3600.0
}
```

### Make commands

```bash
make compile-juliet                  # Compile supported CWEs with LLVM 18 + mem2reg
make test-juliet                     # Run all, show precision/recall/F1
make test-juliet CWE=CWE476         # Filter to single CWE
make test-juliet-json                # JSON output to file
make juliet-categories               # List CWEs and counts
make clean-juliet                    # Clean compiled files
```

## Key design decisions

1. **Reuse SV-COMP pipeline**: no new analysis code — Juliet maps to existing `analyze_memsafety` and `analyze_no_overflow` property analyzers
2. **Always aggressive mode**: precision/recall requires definitive verdicts, not UNKNOWN
3. **Focus on 15 supported CWEs**: compile only what SAF can analyze, not all 118
4. **Compile .i (preprocessed) files**: already available, no need for raw .c compilation with headers
5. **mem2reg pass**: promotes stack allocas to SSA registers, improving analysis precision
