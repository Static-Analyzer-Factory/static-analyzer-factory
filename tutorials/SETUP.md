# SAF Tutorial Setup

This guide covers the one-time setup required before working through any SAF tutorial.

## Prerequisites

- Docker installed and running
- Git
- A terminal / command line

## Step 1: Clone the Repository

```bash
git clone https://github.com/anthropics/static-analyzer-factory.git
cd static-analyzer-factory
```

## Step 2: Start the Docker Development Shell

All SAF tools run inside a Docker container with pre-installed dependencies
(Rust toolchain, Python 3.12, LLVM 18, clang-18, maturin).

```bash
make shell
```

This drops you into an interactive shell inside the container. The project
directory is mounted at `/workspace`.

On first run, the container automatically creates a Python virtual environment
and builds the SAF Python SDK (`maturin develop --release`). Subsequent runs
skip the build if `saf` is already importable.

The Docker image includes `clang-18` and `rustc`, which the tutorials use to
compile vulnerable source code to LLVM IR at runtime. No manual installation
is needed.

## Step 3: Verify the Installation

Run a quick check to confirm everything works:

```bash
python3 -c "import saf; print('SAF version:', saf.__version__)"
```

You should see the SAF version printed without errors.

You can also run the full test suite to confirm the build is healthy:

```bash
make test
```

## How the Tutorials Work

Each tutorial contains source file(s) and detection scripts (`detect.py` and
optionally `detect.rs`). When you run a tutorial, the detection script:

1. **Compiles** the source code to LLVM IR using `clang-18` (C/C++) or `rustc` (Rust)
2. **Loads** the LLVM IR through SAF's LLVM frontend
3. **Analyzes** the program using the appropriate analysis (taint flow, checkers, PTA, etc.)
4. **Reports** findings, graph structures, or analysis results

No pre-built fixtures or manual compilation steps are required — the full
pipeline runs automatically.

## Tutorial Categories

Tutorials are organized into eight categories by vulnerability type and technique:

| Category | Tutorials | Focus | CWEs |
|----------|-----------|-------|------|
| [Getting Started](getting-started/) | 4 | First contact with SAF | - |
| [Information Flow](information-flow/) | 6 | Taint analysis, command injection, format strings | CWE-78, CWE-134 |
| [Memory Safety](memory-safety/) | 5 | Leaks, UAF, double-free | CWE-401, CWE-415, CWE-416 |
| [Buffer Overflow](buffer-overflow/) | 3 | Heap/stack overflow detection | CWE-120, CWE-121, CWE-131 |
| [Integer Issues](integer-issues/) | 2 | Overflow/underflow detection | CWE-190, CWE-191 |
| [Resource Safety](resource-safety/) | 4 | File handles, locks, custom resources | CWE-775, CWE-667 |
| [Advanced Techniques](advanced-techniques/) | 7 | PTA, Memory SSA, SVFG internals | - |
| [Integration](integration/) | 4 | API, JSON export, SARIF, batch scanning | - |

## Full Tutorial Index

### Getting Started (First contact with SAF)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Hello Taint](getting-started/01-hello-taint/) | Beginner | argv → system (C) |
| 02 | [Call Graph & CFG](getting-started/02-call-graph-cfg/) | Beginner | Program structure graphs |
| 03 | [Def-Use & ValueFlow](getting-started/03-defuse-valueflow/) | Beginner | Data flow concepts |
| 04 | [Your First Checker](getting-started/04-your-first-checker/) | Beginner | Memory leak detection |

### Information Flow (CWE-78, CWE-134)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Command Injection](information-flow/01-command-injection/) | Beginner | argv → system (C) |
| 02 | [Cross-Language Rust](information-flow/02-cross-language-rust/) | Intermediate | Rust FFI taint detection |
| 03 | [Format String](information-flow/03-format-string/) | Beginner | User input → printf |
| 04 | [Cross-Module](information-flow/04-cross-module/) | Intermediate | Multi-file taint tracking |
| 05 | [IFDS Precision](information-flow/05-ifds-precision/) | Advanced | IFDS vs BFS comparison |
| 06 | [Z3 Refinement](information-flow/06-z3-refinement/) | Advanced | Z3 false positive filtering |

### Memory Safety (CWE-401, CWE-415, CWE-416)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Leak Detection](memory-safety/01-leak-detection/) | Beginner | Simple malloc/free miss |
| 02 | [Use-After-Free](memory-safety/02-use-after-free/) | Intermediate | Heap UAF via taint |
| 03 | [Double-Free](memory-safety/03-double-free/) | Intermediate | Lifecycle tracking |
| 04 | [Typestate Memory](memory-safety/04-typestate-memory/) | Advanced | IDE solver + typestate |
| 05 | [Path-Sensitive](memory-safety/05-path-sensitive/) | Advanced | Z3-refined leak detection |

### Buffer Overflow (CWE-120, CWE-121, CWE-131)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Taint Detection](buffer-overflow/01-taint-detection/) | Beginner | Simple heap overflow via taint |
| 02 | [Interval Analysis](buffer-overflow/02-interval-analysis/) | Intermediate | Off-by-one via abstract interp |
| 03 | [Complex Patterns](buffer-overflow/03-complex-patterns/) | Advanced | Indirect + CS-PTA + Z3 |

### Integer Issues (CWE-190, CWE-191)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Basic Overflow](integer-issues/01-basic-overflow/) | Beginner | Simple arithmetic overflow |
| 02 | [Size Calculation](integer-issues/02-size-calculation/) | Intermediate | Allocation size overflow |

### Resource Safety (CWE-775, CWE-667)

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [File Leak](resource-safety/01-file-leak/) | Beginner | Simple fopen without fclose |
| 02 | [Typestate File I/O](resource-safety/02-typestate-file-io/) | Intermediate | File lifecycle via IDE |
| 03 | [Lock Safety](resource-safety/03-lock-safety/) | Intermediate | Mutex lock/unlock |
| 04 | [Custom Resources](resource-safety/04-custom-resources/) | Advanced | User-defined resource specs |

### Advanced Techniques

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Pointer Aliasing](advanced-techniques/01-pointer-aliasing/) | Intermediate | Andersen PTA basics + indirect calls |
| 02 | [Field Sensitivity](advanced-techniques/02-field-sensitivity/) | Intermediate | Field-sensitive struct analysis |
| 03 | [Flow-Sensitive PTA](advanced-techniques/03-flow-sensitive-pta/) | Intermediate | Per-program-point precision |
| 04 | [Context-Sensitive](advanced-techniques/04-context-sensitive/) | Advanced | k-CFA + virtual dispatch + CG refinement |
| 05 | [Memory SSA](advanced-techniques/05-memory-ssa/) | Advanced | MSSA internals |
| 06 | [SVFG Exploration](advanced-techniques/06-svfg-exploration/) | Advanced | Sparse value-flow graph |
| 07 | [Analysis Comparison](advanced-techniques/07-analysis-comparison/) | Advanced | All techniques on one program |

### Integration

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Schema Discovery](integration/01-schema-discovery/) | Beginner | Exploring the API |
| 02 | [JSON Export](integration/02-json-export/) | Beginner | Exporting graphs |
| 03 | [SARIF Reporting](integration/03-sarif-reporting/) | Intermediate | CI/CD integration |
| 04 | [Batch Scanning](integration/04-batch-scanning/) | Advanced | Multi-file analysis |

## Recommended Learning Paths

### Security Researcher
1. Getting Started → Information Flow → Memory Safety → Buffer Overflow

### Analysis Student
1. Getting Started → Advanced Techniques → Integration

### Tool Integrator
1. Getting Started → Integration

### Bug Hunter
1. Getting Started → Memory Safety → Buffer Overflow → Integer Issues → Resource Safety

## Start Here

Begin with **getting-started/01-hello-taint** if this is your first time using SAF.

```bash
make shell
cd tutorials-new/getting-started/01-hello-taint
python detect.py
```
