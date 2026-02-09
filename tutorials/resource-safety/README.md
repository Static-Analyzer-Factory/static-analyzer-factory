# Resource Safety

This category covers resource management vulnerabilities including file handle leaks, lock safety, and custom resource lifecycle tracking.

## CWE Coverage

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-775 | Missing Release of File Descriptor | `01-file-leak`, `02-typestate-file-io` |
| CWE-667 | Improper Locking | `03-lock-safety` |

## Learning Objectives

After completing these tutorials, you will be able to:
- Detect file handle leaks using SVFG reachability
- Apply typestate analysis for file I/O lifecycle
- Track mutex lock/unlock patterns
- Define custom resource specifications

## Tutorials

| # | Tutorial | Difficulty | Technique |
|---|----------|------------|-----------|
| 01 | [File Leak](01-file-leak/) | Beginner | SVFG `must_not_reach` |
| 02 | [Typestate File I/O](02-typestate-file-io/) | Intermediate | IDE solver + typestate |
| 03 | [Lock Safety](03-lock-safety/) | Intermediate | Typestate |
| 04 | [Custom Resources](04-custom-resources/) | Advanced | User-defined specs |

## Difficulty Progression

- **Beginner (01)**: Simple `fopen` without `fclose`
- **Intermediate (02-03)**: Typestate tracking for file/lock lifecycle
- **Advanced (04)**: Custom resource specifications for domain-specific resources

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first
- For tutorials 02-04, the [Memory Safety](../memory-safety/) category provides helpful context

## Key Concepts

### Typestate Analysis
Resources have states (e.g., file: `Uninit` → `Open` → `Closed`). Typestate analysis tracks these states to detect:
- Use in wrong state (e.g., read from closed file)
- Missing transitions (e.g., `fopen` without `fclose`)
- Invalid transitions (e.g., double `fclose`)

### Built-in Resource Specs
SAF includes specifications for:
- **file_io**: `fopen`/`fclose`/`fread`/`fwrite`
- **mutex_lock**: `pthread_mutex_lock`/`pthread_mutex_unlock`
- **memory_alloc**: `malloc`/`free`

## Related Categories

- [Memory Safety](../memory-safety/) for similar patterns with memory allocation
