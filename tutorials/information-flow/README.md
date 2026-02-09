# Information Flow

This category covers information flow vulnerabilities including command injection, format string bugs, and taint tracking across modules and languages.

## CWE Coverage

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-78 | OS Command Injection | `01-command-injection`, `02-cross-language-rust`, `04-cross-module` |
| CWE-134 | Format String Vulnerability | `03-format-string` |

## Learning Objectives

After completing these tutorials, you will be able to:
- Detect command injection from environment variables and argv
- Track taint flow across Rust FFI boundaries
- Find format string vulnerabilities
- Trace cross-module taint propagation
- Use IFDS for precise context-sensitive taint tracking
- Apply Z3 refinement to eliminate false positives

## Tutorials

| # | Tutorial | Difficulty | Focus |
|---|----------|------------|-------|
| 01 | [Command Injection](01-command-injection/) | Beginner | `argv` to `system` (C) |
| 02 | [Cross-Language Rust](02-cross-language-rust/) | Intermediate | Rust FFI taint detection |
| 03 | [Format String](03-format-string/) | Beginner | User input to `printf` |
| 04 | [Cross-Module](04-cross-module/) | Intermediate | Multi-file taint tracking |
| 05 | [IFDS Precision](05-ifds-precision/) | Advanced | IFDS vs BFS comparison |
| 06 | [Z3 Refinement](06-z3-refinement/) | Advanced | Z3 false positive filtering |

## Difficulty Progression

- **Beginner (01, 03)**: Single-function, direct source-to-sink flow
- **Intermediate (02, 04)**: Cross-language or multi-file flows
- **Advanced (05, 06)**: IFDS precision and Z3-based refinement

## Prerequisites

- Complete the [Getting Started](../getting-started/) tutorials first (if available)
- Complete [SETUP.md](../../SETUP.md) for Docker environment setup

## Key Concepts

### Taint Sources and Sinks
- **Sources**: Functions that introduce untrusted data (`getenv`, `gets`, `read`, `argv`)
- **Sinks**: Functions that execute/output data (`system`, `printf`, `exec*`)
- **Sanitizers**: Functions that validate data (`validate_input`, bounds checking)

### IFDS vs BFS Taint Analysis
- **BFS** (`q.taint_flow()`): Fast, flow-insensitive, may have false positives
- **IFDS** (`proj.ifds_taint()`): Precise, context-sensitive, better for complex flows

### Cross-Language Analysis
SAF analyzes LLVM IR, enabling taint tracking across languages:
- Rust unsafe code calling C libraries
- C calling Rust functions
- Any language that compiles to LLVM IR

### Z3 Path-Sensitive Refinement
- **Path-insensitive**: Reports all possible flows (may include false positives)
- **Z3-refined**: Checks if paths are feasible, filters infeasible flows

## Quick Start

1. Start in the Docker environment:
   ```bash
   make shell
   ```

2. Run the first tutorial:
   ```bash
   python3 tutorials-new/information-flow/01-command-injection/detect.py
   ```

3. Follow the progression through all six tutorials.

## Related Categories

- [Memory Safety](../memory-safety/) for use-after-free and buffer overflow detection
- [Graph Exploration](../graphs/) for understanding SAF's internal representations
