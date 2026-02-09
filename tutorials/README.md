# SAF Tutorials

This directory contains tutorials for the Static Analyzer Factory (SAF), organized by vulnerability type with progressive difficulty levels within each category.

## Tutorial Structure

Tutorials are organized into 8 categories targeting different learning goals:

| Category | Focus | Difficulty Range | CWE Coverage |
|----------|-------|------------------|--------------|
| [Getting Started](getting-started/) | First contact with SAF | Beginner | - |
| [Memory Safety](memory-safety/) | UAF, double-free, leaks | Beginner → Advanced | CWE-416, 415, 401 |
| [Buffer Overflow](buffer-overflow/) | Heap/stack overflow | Beginner → Advanced | CWE-120, 121, 131 |
| [Integer Issues](integer-issues/) | Integer overflow/underflow | Beginner → Intermediate | CWE-190, 191 |
| [Resource Safety](resource-safety/) | File handles, locks | Beginner → Advanced | CWE-775, 667 |
| [Information Flow](information-flow/) | Taint, command injection | Beginner → Advanced | CWE-78, 134 |
| [Advanced Techniques](advanced-techniques/) | Analysis internals | Intermediate → Advanced | - |
| [Integration](integration/) | Tooling, CI/CD | Beginner → Advanced | - |

## Difficulty Levels

Each tutorial is tagged with a difficulty level based on concept complexity (not code length):

- **Beginner**: Single-function, direct data flows (e.g., `getenv` → `system`)
- **Intermediate**: Multi-function, indirect flows, basic aliasing
- **Advanced**: Whole-program analysis, path-sensitivity, multiple techniques combined

## Recommended Learning Paths

### Security Researcher
Focus on finding vulnerabilities in real-world code:
1. `getting-started/01-hello-taint` — Your first taint flow
2. `information-flow/01-command-injection` — Classic CWE-78
3. `memory-safety/02-use-after-free` — Heap UAF detection
4. `buffer-overflow/01-taint-detection` — Heap overflow via taint
5. `information-flow/05-ifds-precision` — IFDS for precise tracking

### Analysis Student
Understand program analysis fundamentals:
1. `getting-started/02-call-graph-cfg` — Program structure graphs
2. `getting-started/03-defuse-valueflow` — Data flow concepts
3. `advanced-techniques/01-pointer-aliasing` — PTA basics
4. `advanced-techniques/04-context-sensitive` — Context sensitivity
5. `advanced-techniques/05-memory-ssa` — Memory SSA representation

### Tool Integrator
Build SAF into your workflow:
1. `integration/01-schema-discovery` — Explore the API
2. `integration/02-json-export` — Export graphs for visualization
3. `integration/03-sarif-reporting` — CI/CD integration
4. `integration/04-batch-scanning` — Multi-file analysis

### Bug Hunter
Use checkers to find real bugs:
1. `getting-started/04-your-first-checker` — Checker basics
2. `memory-safety/01-leak-detection` — Memory leaks
3. `resource-safety/02-typestate-file-io` — File handle leaks
4. `memory-safety/05-path-sensitive` — Z3-refined detection

## CWE Cross-Reference Index

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-78 | OS Command Injection | `information-flow/01-command-injection` |
| CWE-120 | Buffer Overflow (Heap) | `buffer-overflow/01-taint-detection`, `buffer-overflow/02-interval-analysis` |
| CWE-121 | Stack-based Buffer Overflow | `buffer-overflow/03-complex-patterns` |
| CWE-131 | Incorrect Calculation of Buffer Size | `buffer-overflow/02-interval-analysis` |
| CWE-134 | Format String Vulnerability | `information-flow/03-format-string` |
| CWE-190 | Integer Overflow | `integer-issues/01-basic-overflow`, `integer-issues/02-size-calculation` |
| CWE-191 | Integer Underflow | `integer-issues/01-basic-overflow` |
| CWE-401 | Memory Leak | `memory-safety/01-leak-detection`, `memory-safety/04-typestate-memory` |
| CWE-415 | Double Free | `memory-safety/03-double-free` |
| CWE-416 | Use After Free | `memory-safety/02-use-after-free` |
| CWE-667 | Improper Locking | `resource-safety/03-lock-safety` |
| CWE-775 | Missing Release of File Descriptor | `resource-safety/01-file-leak`, `resource-safety/02-typestate-file-io` |

## Quick Start

1. Complete the [Setup Guide](SETUP.md) to configure your environment
2. Start with `getting-started/01-hello-taint` for your first tutorial
3. Follow the learning path that matches your goal

Each tutorial directory contains:
- `README.md` — Tutorial guide with explanation and walkthrough
- `detect.py` — Python detection script
- `detect.rs` — Rust detection script (where applicable)
- Source files (`.c`, `.cpp`, or `.rs`) — Vulnerable code to analyze

Run a tutorial with:
```bash
make shell                      # Enter Docker environment
cd tutorials/<category>/<name>
python3 detect.py               # Run Python detector
```
