# Plan 046: PTABen Benchmark Integration

## Overview

Integrate the [SVF Test-Suite (PTABen)](https://github.com/SVF-tools/Test-Suite) — a micro-benchmark suite of ~400 hand-written test programs — for comprehensive validation of SAF's pointer analysis and bug checkers.

## Goals

1. **Correctness validation** — Ensure SAF's PTA and checkers produce correct results against known-good test cases
2. **Performance benchmarking** — Measure analysis speed/memory
3. **Feature coverage gap analysis** — Identify which tests SAF can/cannot handle
4. **Extensible framework** — Support future benchmark suites (Juliet, SPEC, custom)

## Non-Goals (Deferred)

- CI integration (manual `make test-ptaben` only initially)
- Performance trend tracking / historical database
- Regression baseline files
- MTA (multithreading) tests (until SAF has MHP analysis)

## Design

### Repository Structure

```
static-analyzer-lib/
├── crates/
│   └── saf-bench/                    # New benchmark crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                # BenchmarkSuite trait + common infra
│           ├── main.rs               # CLI binary
│           ├── ptaben.rs             # PTABen implementation
│           └── report.rs             # Human + JSON reporting
├── scripts/
│   └── compile-ptaben.sh             # Parallel compilation script
├── tests/
│   └── benchmarks/
│       └── ptaben/                   # Git submodule → SVF-tools/Test-Suite
│           └── .compiled/            # Generated bitcode (git-ignored)
└── Makefile                          # New targets
```

### Makefile Targets

```makefile
compile-ptaben: ## Compile PTABen test suite with LLVM 18 (run once after clone/update)
	@echo "Compiling PTABen test suite (~400 files)..."
	$(DOCKER_RUN) ./scripts/compile-ptaben.sh

test-ptaben: ## Run PTABen benchmark suite against SAF
	@echo "Running PTABen benchmarks..."
	$(DOCKER_RUN) cargo run --release -p saf-bench -- ptaben \
		--compiled-dir tests/benchmarks/ptaben/.compiled

test-ptaben-json: ## Run PTABen benchmarks with JSON output
	$(DOCKER_RUN) cargo run --release -p saf-bench -- ptaben \
		--compiled-dir tests/benchmarks/ptaben/.compiled \
		--json

clean-ptaben: ## Remove compiled PTABen bitcode
	rm -rf tests/benchmarks/ptaben/.compiled
```

### Crate Design (`saf-bench`)

#### Core Trait

```rust
/// A benchmark suite with its own oracle format and validation logic
pub trait BenchmarkSuite {
    fn name(&self) -> &str;
    fn discover_tests(&self, dir: &Path) -> Result<Vec<TestCase>>;
    fn extract_expectations(&self, bc: &Path) -> Result<Vec<Expectation>>;
    fn validate(&self, expected: &Expectation, actual: &AnalysisResult) -> Outcome;
}

/// Validate an alias result against dual ground truth
fn validate_alias(expectation: &AliasExpectation, actual: AliasKind) -> Outcome {
    let floor = &expectation.sound_floor;
    let ceiling = &expectation.precise_ceiling;

    // Check if actual meets the sound floor
    let meets_floor = match (floor, &actual) {
        // NoAlias floor: only NoAlias is acceptable
        (AliasKind::NoAlias, AliasKind::NoAlias) => true,
        (AliasKind::NoAlias, _) => false,

        // MustAlias floor: only MustAlias is acceptable
        (AliasKind::MustAlias, AliasKind::MustAlias) => true,
        (AliasKind::MustAlias, _) => false,

        // MayAlias floor: MayAlias, MustAlias, or PartialAlias acceptable
        (AliasKind::MayAlias, AliasKind::NoAlias) => false,  // Too precise / unsound
        (AliasKind::MayAlias, _) => true,

        // PartialAlias floor: PartialAlias or MustAlias acceptable
        (AliasKind::PartialAlias, AliasKind::NoAlias) => false,
        (AliasKind::PartialAlias, AliasKind::MayAlias) => false,
        (AliasKind::PartialAlias, _) => true,
    };

    if !meets_floor {
        // Check if this is "beyond floor" (more precise) or truly unsound
        let is_more_precise = match (floor, &actual) {
            (AliasKind::MayAlias, AliasKind::NoAlias) => true,
            _ => false,
        };

        if is_more_precise && ceiling.is_none() {
            // More precise than floor, but we don't know the ceiling
            // This needs manual verification
            Outcome::BeyondFloor {
                floor: format!("{:?}", floor),
                actual: format!("{:?}", actual),
            }
        } else {
            Outcome::Unsound {
                expected: format!("{:?}", floor),
                actual: format!("{:?}", actual),
            }
        }
    } else if ceiling.as_ref() == Some(&actual) {
        Outcome::Exact
    } else {
        Outcome::Sound
    }
}
```

#### Expectation Types (Dual Ground Truth)

PTABen's original annotations may be conservative (e.g., `MAYALIAS` where a more
precise analysis could prove `NoAlias`). To handle this correctly, we use a
**dual ground truth** system with sound floor and precise ceiling:

```rust
/// Alias expectation with dual ground truth
pub struct AliasExpectation {
    pub ptr_a: String,
    pub ptr_b: String,
    /// Sound floor: minimum precision any correct analysis must achieve
    /// (e.g., MayAlias means "at least report MayAlias")
    pub sound_floor: AliasKind,
    /// Precise ceiling: most precise correct answer (if known)
    /// None means we don't know the precise answer
    pub precise_ceiling: Option<AliasKind>,
}

/// What a test expects (extensible for future suites)
pub enum Expectation {
    Alias(AliasExpectation),
    MemLeak { location: String },
    DoubleFree { location: String },
    Custom(String, serde_json::Value),  // Catch-all for new suites
}

pub enum AliasKind {
    MayAlias,
    NoAlias,
    MustAlias,
    PartialAlias,
}
```

**Interpretation of PTABen intrinsics:**

| PTABen Intrinsic | Sound Floor | Precise Ceiling | Meaning |
|------------------|-------------|-----------------|---------|
| `MAYALIAS(p,q)` | MayAlias | None (unknown) | Must report at least MayAlias |
| `NOALIAS(p,q)` | NoAlias | NoAlias | Must report exactly NoAlias |
| `MUSTALIAS(p,q)` | MustAlias | MustAlias | Must report exactly MustAlias |

#### Test Outcomes

```rust
pub enum Outcome {
    /// Analysis result matches the precise ceiling (optimal)
    Exact,
    /// Analysis is sound (meets floor) but less precise than ceiling
    Sound,
    /// Analysis is more precise than floor — either correct or unsound
    /// Requires manual verification to determine which
    BeyondFloor { floor: String, actual: String },
    /// Analysis is unsound (missed a real alias)
    Unsound { expected: String, actual: String },
    /// Test skipped
    Skip { reason: String },
}
```

**Outcome decision logic:**

```
Given: sound_floor, precise_ceiling (optional), actual_result

if actual == precise_ceiling:
    Exact                          # Perfect match to known precise answer
elif actual is_at_least sound_floor:
    if precise_ceiling is None:
        if actual is_more_precise_than sound_floor:
            BeyondFloor            # More precise, needs verification
        else:
            Sound                  # Matches floor exactly
    else:
        Sound                      # Between floor and ceiling
else:
    Unsound                        # Failed to meet floor
```

**Alias precision ordering:** `MustAlias > PartialAlias > MayAlias > NoAlias`

- `MustAlias` is the most precise (definite aliasing)
- `NoAlias` is the most restrictive (definite non-aliasing)
- `MayAlias` is the least precise (conservative)

#### CLI Interface

```
saf-bench <suite> --compiled-dir <path> [--json] [--filter <pattern>]

Examples:
  saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled
  saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --json
  saf-bench ptaben --compiled-dir tests/benchmarks/ptaben/.compiled --filter "fs_tests/*"
```

### PTABen Oracle Detection

PTABen encodes expectations as function calls. SAF detects these intrinsics in the bitcode:

| Source Macro | Bitcode Call Target |
|--------------|---------------------|
| `MAYALIAS(p, q)` | `@MAYALIAS(i8*, i8*)` |
| `NOALIAS(p, q)` | `@NOALIAS(i8*, i8*)` |
| `MUSTALIAS(p, q)` | `@MUSTALIAS(i8*, i8*)` |
| `PARTIALALIAS(p, q)` | `@PARTIALALIAS(i8*, i8*)` |
| `EXPECTEDLEAK(p)` | `@EXPECTEDLEAK(i8*)` |
| `UNEXPECTEDLEAK(p)` | `@UNEXPECTEDLEAK(i8*)` |
| `DOUBLEFREE(p)` | `@DOUBLEFREE(i8*)` |

Implementation:
1. Load bitcode via SAF's LLVM frontend → `AirBundle`
2. Scan all `Call` instructions for known oracle function names
3. Extract argument `ValueId`s (the pointers being tested)
4. Build `Vec<Expectation>` from discovered calls
5. Run SAF's PTA/checkers, compare against expectations

### Output Formats

#### Human-Readable (default)

```
PTABen Benchmark Results
========================

Legend:
  Exact       — Matches known precise answer
  Sound       — Meets soundness requirement (may be conservative)
  ToVerify    — More precise than baseline, requires manual verification
  Unsound     — Failed soundness check (missed alias)
  Skip        — Test skipped (unsupported feature)

Category          Exact  Sound  ToVerify  Unsound  Skip   Total
────────────────────────────────────────────────────────────────
basic_c_tests       40      3        2        2      0      47
basic_cpp_tests     30      4        2        2      0      38
fs_tests            25      2        1        1      0      29
cs_tests            20      6        5        0      0      31
path_tests          15      2        1        3      0      21
mem_leak            22      0        0        0      0      22
double_free         14      1        0        1      0      16
mta                  0      0        0        0     45      45
────────────────────────────────────────────────────────────────
TOTAL              166     18       11        9     45     249

Unsound (requires investigation):
  basic_c_tests/test12.c.bc
    MUSTALIAS(p, q): floor=MustAlias, got=MayAlias
  path_tests/path5.c.bc
    NOALIAS(x, y): floor=NoAlias, got=MayAlias

To Verify (SAF reported more precise than baseline):
  cs_tests/context1.c.bc
    MAYALIAS(p, q): floor=MayAlias, got=NoAlias
    → If correct: SAF is more precise than baseline
    → If incorrect: SAF has a soundness bug

Time: 12.4s
```

**Note on "To Verify" results:**
When SAF reports a more precise result than the benchmark's baseline (e.g.,
`NoAlias` where baseline expects `MayAlias`), this could mean:
1. SAF correctly proved a tighter bound (e.g., via context-sensitivity)
2. SAF has a soundness bug

These cases require manual inspection or additional validation.

#### JSON (`--json`)

```json
{
  "suite": "ptaben",
  "summary": {
    "exact": 166,
    "sound": 18,
    "to_verify": 11,
    "unsound": 9,
    "skip": 45,
    "total": 249
  },
  "by_category": {
    "basic_c_tests": { "exact": 40, "sound": 3, "to_verify": 2, "unsound": 2, "skip": 0 },
    "cs_tests": { "exact": 20, "sound": 6, "to_verify": 5, "unsound": 0, "skip": 0 },
    "mta": { "exact": 0, "sound": 0, "to_verify": 0, "unsound": 0, "skip": 45, "skip_reason": "MTA unsupported" }
  },
  "unsound": [
    {
      "file": "basic_c_tests/test12.c.bc",
      "assertion": "MUSTALIAS(p, q)",
      "sound_floor": "MustAlias",
      "precise_ceiling": "MustAlias",
      "actual": "MayAlias",
      "verdict": "unsound"
    }
  ],
  "to_verify": [
    {
      "file": "cs_tests/context1.c.bc",
      "assertion": "MAYALIAS(p, q)",
      "sound_floor": "MayAlias",
      "precise_ceiling": null,
      "actual": "NoAlias",
      "verdict": "to_verify",
      "note": "SAF reported more precise than baseline; manual verification needed"
    }
  ],
  "skipped": [
    { "category": "mta", "reason": "MTA unsupported", "count": 45 }
  ],
  "timing_secs": 12.4
}
```

### Extensibility

The `BenchmarkSuite` trait allows future suites with different oracle formats:

```rust
// PTABen: Intrinsic-based oracle (function calls in bitcode)
impl BenchmarkSuite for PTABen {
    fn extract_expectations(&self, bc: &Path) -> Result<Vec<Expectation>> {
        let bundle = load_bitcode(bc)?;
        find_oracle_calls(&bundle, &["MAYALIAS", "NOALIAS", ...])
    }
}

// Future: Juliet - Comment-based oracle (CWE markers in source)
impl BenchmarkSuite for Juliet {
    fn extract_expectations(&self, bc: &Path) -> Result<Vec<Expectation>> {
        let src = find_source_for_bc(bc)?;
        parse_cwe_comments(&src)
    }
}

// Future: Custom - Sidecar file oracle (expectations in .json)
impl BenchmarkSuite for CustomSuite {
    fn extract_expectations(&self, bc: &Path) -> Result<Vec<Expectation>> {
        let json = bc.with_extension("expected.json");
        serde_json::from_reader(File::open(json)?)
    }
}
```

## Implementation Phases

### Phase 1: Infrastructure Setup
- [ ] Add git submodule: `git submodule add https://github.com/SVF-tools/Test-Suite tests/benchmarks/ptaben`
- [ ] Create `scripts/compile-ptaben.sh` with parallel compilation
- [ ] Add `.gitignore` entry for `tests/benchmarks/ptaben/.compiled/`
- [ ] Add Makefile targets with help descriptions

### Phase 2: saf-bench Crate Scaffolding
- [ ] Create `crates/saf-bench/Cargo.toml` with dependencies
- [ ] Implement `BenchmarkSuite` trait in `lib.rs`
- [ ] Implement `Expectation`, `Outcome`, `AnalysisResult` types
- [ ] Implement CLI argument parsing in `main.rs`

### Phase 3: PTABen Implementation
- [ ] Implement test discovery (find `.bc` files by category)
- [ ] Implement oracle extraction (parse intrinsic calls from AIR)
- [ ] Implement PTA validation (run SAF PTA, compare alias results)
- [ ] Implement checker validation (run SAF checkers, compare warnings)
- [ ] Handle skip logic for unsupported categories (MTA)

### Phase 4: Reporting
- [ ] Implement human-readable summary table
- [ ] Implement JSON output format
- [ ] Add `--filter` option for running subsets

### Phase 5: Documentation & Testing
- [ ] Add usage instructions to README or docs/
- [ ] Test full pipeline: compile → run → report
- [ ] Verify output accuracy against manual inspection

## Dependencies

- `saf-core`, `saf-frontends`, `saf-analysis` — SAF analysis infrastructure
- `clap` — CLI argument parsing
- `serde`, `serde_json` — JSON output
- `rayon` — Parallel test execution (optional)

## Future Enhancements

- CI workflow with nightly/weekly runs
- `--baseline failures.json` for regression detection
- Performance comparison against previous runs
- Integration with Juliet, SPEC, or custom suites
- MTA test support (when SAF gains multithreading analysis)
