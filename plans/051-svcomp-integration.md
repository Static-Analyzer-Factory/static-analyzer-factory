# Plan 051: SV-COMP Benchmark Integration

## Overview

Integrate the SV-COMP (Software Verification Competition) benchmarks into `saf-bench`, following the PTABen pattern but with SV-COMP's external YAML task definitions and per-category scoring system.

**Repository:** https://gitlab.com/sosy-lab/benchmarking/sv-benchmarks

## Motivation

SV-COMP is the premier benchmark for software verification tools, with:
- 8 major C categories (ReachSafety, MemSafety, NoOverflows, Concurrency, Termination, SoftwareSystems)
- Real-world programs (AWS, coreutils, BusyBox, Linux drivers)
- Standardized scoring system used by academic community
- Well-defined properties and ground truth

Integrating SV-COMP allows SAF to:
1. Compare against state-of-the-art verifiers (CPAchecker, CBMC, Ultimate, etc.)
2. Identify precision/soundness gaps on realistic programs
3. Track progress over time with reproducible benchmarks

## SV-COMP Scoring Model

Per SV-COMP 2026 rules:

| Result | Points | Description |
|--------|--------|-------------|
| TRUE correct | +2 | Proved property holds |
| FALSE correct | +1 | Found real violation |
| TRUE incorrect | -32 | Missed bug (unsound) |
| FALSE incorrect | -16 | False alarm |
| UNKNOWN | 0 | Timeout, crash, limitation |

Each category is scored independently, then normalized for meta-categories.

## Categories and Properties

| Category | Property | SAF Support |
|----------|----------|-------------|
| ReachSafety | `unreach-call` | Z3 path reachability |
| MemSafety | `valid-memsafety` | SVFG checkers + nullness |
| MemCleanup | `valid-memcleanup` | Memory leak checker |
| NoOverflows | `no-overflow` | Abstract interpretation |
| ConcurrencySafety | `no-data-race` | MTA MHP (partial) |
| Termination | `termination` | Not supported (UNKNOWN) |

## Implementation Phases

### Phase 1: Git & Build Infrastructure

**1.1 Add git submodule**
```bash
git submodule add --depth 1 \
  https://gitlab.com/sosy-lab/benchmarking/sv-benchmarks.git \
  tests/benchmarks/sv-benchmarks
```

**1.2 Update `.gitmodules`**
```gitmodules
[submodule "tests/benchmarks/sv-benchmarks"]
	path = tests/benchmarks/sv-benchmarks
	url = https://gitlab.com/sosy-lab/benchmarking/sv-benchmarks.git
	shallow = true
```

**1.3 Update `.gitignore`**
```gitignore
# SV-COMP compiled bitcode
tests/benchmarks/sv-benchmarks/.compiled/

# SV-COMP task metadata cache
tests/benchmarks/sv-benchmarks/.task-cache/
```

**1.4 Create `scripts/sv-comp-stubs.h`**
```c
// SV-COMP special functions - declarations only
extern void __VERIFIER_error(void) __attribute__((noreturn));
extern void __VERIFIER_assume(int condition);
extern int __VERIFIER_nondet_int(void);
extern unsigned int __VERIFIER_nondet_uint(void);
extern long __VERIFIER_nondet_long(void);
extern char __VERIFIER_nondet_char(void);
extern _Bool __VERIFIER_nondet_bool(void);
extern void* __VERIFIER_nondet_pointer(void);
extern float __VERIFIER_nondet_float(void);
extern double __VERIFIER_nondet_double(void);
extern void __VERIFIER_atomic_begin(void);
extern void __VERIFIER_atomic_end(void);
```

**1.5 Create `scripts/compile-svcomp.sh`**

Compilation script that:
- Parses YAML task definitions
- Handles ILP32/LP64 data models (-m32/-m64)
- Links multi-file tasks with llvm-link
- Includes sv-comp-stubs.h
- Compiles with clang-18 inside Docker

**1.6 Add Makefile targets**
```makefile
compile-svcomp: ## Compile SV-COMP test suite with LLVM 18
	docker compose run --rm dev ./scripts/compile-svcomp.sh

compile-svcomp-category: ## Compile single category (CAT=ReachSafety)
	docker compose run --rm dev ./scripts/compile-svcomp.sh --category $(CAT)

test-svcomp: ## Run SV-COMP benchmarks
	docker compose run --rm dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled

test-svcomp-category: ## Run single category (CAT=MemSafety)
	docker compose run --rm dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled \
		--category $(CAT)

test-svcomp-json: ## JSON output
	docker compose run --rm dev cargo run --release -p saf-bench -- svcomp \
		--compiled-dir tests/benchmarks/sv-benchmarks/.compiled --json

clean-svcomp: ## Remove compiled bitcode
	rm -rf tests/benchmarks/sv-benchmarks/.compiled
```

### Phase 2: YAML Task Parser

**2.1 Create `crates/saf-bench/src/svcomp/task.rs`**

Parse SV-COMP task definition format (version 2.0+):
```yaml
format_version: "2.0"
input_files:
  - example.c
properties:
  - property_file: ../properties/unreach-call.prp
    expected_verdict: true
options:
  language: C
  data_model: ILP32
```

Key types:
```rust
pub struct SvCompTask {
    pub path: PathBuf,
    pub input_files: Vec<PathBuf>,
    pub properties: Vec<PropertySpec>,
    pub language: Language,
    pub data_model: DataModel,
}

pub struct PropertySpec {
    pub property_file: PathBuf,
    pub property: Property,
    pub expected_verdict: bool,
    pub subproperty: Option<String>,
}

pub enum Property {
    UnreachCall,
    ValidMemsafety,
    ValidMemcleanup,
    NoOverflow,
    NoDataRace,
    Termination,
}

pub enum DataModel {
    ILP32,  // 32-bit
    LP64,   // 64-bit
}
```

### Phase 3: Property Analyzers

**3.1 Create `crates/saf-bench/src/svcomp/property.rs`**

Map each property to optimal SAF analysis:

```rust
pub fn analyze_property(
    property: &Property,
    module: &AirModule,
    config: &AnalysisConfig,
) -> PropertyResult {
    match property {
        Property::UnreachCall => analyze_unreachability(module, config),
        Property::ValidMemsafety => analyze_memsafety(module, config),
        Property::ValidMemcleanup => analyze_memcleanup(module, config),
        Property::NoOverflow => analyze_no_overflow(module, config),
        Property::NoDataRace => analyze_no_data_race(module, config),
        Property::Termination => PropertyResult::Unknown {
            reason: "Termination analysis not implemented".into()
        },
    }
}
```

**3.2 `unreach-call` analyzer**
```rust
fn analyze_unreachability(module: &AirModule, config: &AnalysisConfig) -> PropertyResult {
    // 1. Find calls to __VERIFIER_error
    let error_calls = find_calls_to(module, "__VERIFIER_error");
    if error_calls.is_empty() {
        return PropertyResult::True;
    }

    // 2. Build infrastructure
    let callgraph = CallGraph::build(module);
    let cfgs = build_cfgs(module);

    // 3. For each error call, check path feasibility with Z3
    for call in &error_calls {
        let result = check_path_reachable(
            entry_block, call.block,
            call.function, module,
            config.z3_timeout_ms,
            config.max_guards,
            config.max_paths,
        );
        match result.result {
            PathReachability::Reachable(witness) => {
                return PropertyResult::False { witness };
            }
            PathReachability::Unknown => {
                return PropertyResult::Unknown { reason: "Path explosion".into() };
            }
            PathReachability::Unreachable => continue,
        }
    }
    PropertyResult::True
}
```

**3.3 `valid-memsafety` analyzer**
```rust
fn analyze_memsafety(module: &AirModule, config: &AnalysisConfig) -> PropertyResult {
    // Build full analysis infrastructure
    let pta = solve_combined_pta(module);  // CS + FS
    let svfg = build_svfg(module, &pta);
    let nullness = analyze_nullness(module);

    // Run memory safety checkers with path-sensitivity
    let specs = vec![
        spec::null_deref(),
        spec::use_after_free(),
        spec::double_free(),
    ];

    let ps_config = PathSensitiveConfig {
        z3_timeout_ms: config.z3_timeout_ms,
        max_guards: config.max_guards,
        ..Default::default()
    };

    let result = run_checkers_path_sensitive(&specs, module, &svfg, &table, &ps_config);

    if result.feasible.is_empty() {
        PropertyResult::True
    } else {
        PropertyResult::False {
            witness: result.feasible[0].trace.clone(),
        }
    }
}
```

**3.4 `no-overflow` analyzer**
```rust
fn analyze_no_overflow(module: &AirModule, config: &AnalysisConfig) -> PropertyResult {
    let absint_config = AbstractInterpConfig {
        use_widening: true,
        use_narrowing: true,
        ..Default::default()
    };

    // Use interprocedural AI for precision
    let result = solve_interprocedural(module, &absint_config);
    let overflow_findings = check_integer_overflow(module, &absint_config);

    if overflow_findings.findings.is_empty() {
        PropertyResult::True
    } else {
        // Verify with Z3 to reduce false positives
        let verified = verify_overflows_z3(&overflow_findings, module);
        if verified.is_empty() {
            PropertyResult::True
        } else {
            PropertyResult::False { witness: verified[0].clone() }
        }
    }
}
```

### Phase 4: Scoring & Reporting

**4.1 Create `crates/saf-bench/src/svcomp/scoring.rs`**

```rust
#[derive(Debug, Clone)]
pub enum SvCompVerdict {
    True,
    False { witness: Option<Trace> },
    Unknown { reason: String },
}

#[derive(Debug, Clone)]
pub enum SvCompOutcome {
    TrueCorrect,      // +2
    FalseCorrect,     // +1
    TrueIncorrect,    // -32
    FalseIncorrect,   // -16
    Unknown,          // 0
}

impl SvCompOutcome {
    pub fn score(&self) -> i32 {
        match self {
            Self::TrueCorrect => 2,
            Self::FalseCorrect => 1,
            Self::TrueIncorrect => -32,
            Self::FalseIncorrect => -16,
            Self::Unknown => 0,
        }
    }
}

pub fn compute_outcome(verdict: &SvCompVerdict, expected: bool) -> SvCompOutcome {
    match (verdict, expected) {
        (SvCompVerdict::True, true) => SvCompOutcome::TrueCorrect,
        (SvCompVerdict::True, false) => SvCompOutcome::TrueIncorrect,
        (SvCompVerdict::False { .. }, false) => SvCompOutcome::FalseCorrect,
        (SvCompVerdict::False { .. }, true) => SvCompOutcome::FalseIncorrect,
        (SvCompVerdict::Unknown { .. }, _) => SvCompOutcome::Unknown,
    }
}
```

**4.2 Category summary**
```rust
pub struct SvCompCategorySummary {
    pub category: String,
    pub subcategory: Option<String>,
    pub property: String,
    pub total_tasks: usize,
    pub true_correct: usize,
    pub false_correct: usize,
    pub true_incorrect: usize,
    pub false_incorrect: usize,
    pub unknown: usize,
    pub score: i32,
    pub max_possible_score: i32,
}

pub struct SvCompSummary {
    pub categories: Vec<SvCompCategorySummary>,
    pub total_score: i32,
    pub max_possible_score: i32,
    pub timing_secs: f64,
}
```

### Phase 5: CLI Integration

**5.1 Update `crates/saf-bench/src/main.rs`**

Add `svcomp` subcommand:
```rust
Svcomp {
    #[arg(long)]
    compiled_dir: PathBuf,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    category: Option<String>,

    #[arg(long)]
    subcategory: Option<String>,

    #[arg(long)]
    property: Option<String>,

    #[arg(long)]
    filter: Option<String>,

    #[arg(long, short = 'j')]
    jobs: Option<usize>,

    #[arg(long, default_value = "900")]
    timeout: u64,

    #[arg(long, default_value = "15000")]
    memory_mb: u64,
}
```

### Phase 6: Human Report Format

```
=== SV-COMP Benchmark Results ===
Categories: 6 | Tasks: 1,234 | Time: 45m 23s

Category            TRUE  FALSE  Score   Max      %
─────────────────────────────────────────────────────
ReachSafety           89     12   +166   +200   83%
MemSafety             45      8    +82   +100   82%
MemCleanup            23      5    +41    +50   82%
NoOverflows           67     15   +119   +150   79%
ConcurrencySafety     12      3    +21    +30   70%
Termination            0      0     +0   +100    0%
─────────────────────────────────────────────────────
TOTAL                236     43   +429   +630   68%

Incorrect Verdicts:
  TRUE incorrect (-32): 2 tasks
  FALSE incorrect (-16): 5 tasks

Unknown: 148 tasks (Termination: 100, Timeout: 32, Error: 16)
```

## File Structure

```
tests/benchmarks/
├── ptaben/                      # Existing
└── sv-benchmarks/               # New submodule
    ├── c/                       # Source
    ├── .compiled/               # Bitcode (gitignored)
    └── .task-cache/             # YAML cache (gitignored)

scripts/
├── compile-ptaben.sh            # Existing
├── compile-svcomp.sh            # New
└── sv-comp-stubs.h              # New

crates/saf-bench/
├── Cargo.toml                   # Add serde_yaml dependency
└── src/
    ├── lib.rs                   # Add svcomp module
    ├── main.rs                  # Add svcomp subcommand
    ├── ptaben.rs                # Existing
    ├── report.rs                # Extend for SV-COMP
    └── svcomp/                  # New
        ├── mod.rs
        ├── task.rs
        ├── property.rs
        └── scoring.rs
```

## Dependencies

Add to `crates/saf-bench/Cargo.toml`:
```toml
serde_yaml = "0.9"
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Completed (scores computed) |
| 1 | Analysis errors |
| 2 | Compilation errors |

## Success Criteria

1. `make compile-svcomp` successfully compiles all supported categories
2. `make test-svcomp` runs and produces per-category scores
3. Scores are comparable to published SV-COMP results for similar tools
4. JSON output matches expected schema for tooling integration

## Future Enhancements

- Witness generation (GraphML/YAML format)
- Witness validation integration
- SoftwareSystems category (large programs)
- Java benchmarks
- Termination analysis (ranking functions)
