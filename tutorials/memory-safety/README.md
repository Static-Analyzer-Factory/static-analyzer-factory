# Memory Safety

This category covers memory safety vulnerabilities including use-after-free (UAF), double-free, and memory leaks. Tutorials progress from simple leak detection to path-sensitive analysis.

## CWE Coverage

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-401 | Memory Leak | `01-leak-detection`, `04-typestate-memory` |
| CWE-415 | Double Free | `03-double-free` |
| CWE-416 | Use After Free | `02-use-after-free` |

## Learning Objectives

After completing these tutorials, you will be able to:
- Detect memory leaks using SVFG reachability
- Find use-after-free vulnerabilities via taint flow
- Track resource lifecycle with double-free detection
- Apply typestate analysis to memory allocation
- Use path-sensitive analysis to eliminate false positives

## Tutorials

| # | Tutorial | Difficulty | Technique |
|---|----------|------------|-----------|
| 01 | [Leak Detection](01-leak-detection/) | Beginner | SVFG `must_not_reach` |
| 02 | [Use-After-Free](02-use-after-free/) | Intermediate | Taint flow |
| 03 | [Double-Free](03-double-free/) | Intermediate | Lifecycle tracking |
| 04 | [Typestate Memory](04-typestate-memory/) | Advanced | IDE solver + typestate |
| 05 | [Path-Sensitive](05-path-sensitive/) | Advanced | Z3-refined detection |

## Difficulty Progression

- **Beginner (01)**: Single-function, direct malloc→return without free
- **Intermediate (02-03)**: Multi-function, free→use or free→free patterns
- **Advanced (04-05)**: Whole-program typestate tracking, path-sensitive refinement

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first
- For advanced tutorials (04-05), familiarity with typestate and Z3 concepts helps

## Next Steps

After completing this category:
- [Buffer Overflow](../buffer-overflow/) for related memory issues
- [Resource Safety](../resource-safety/) for similar patterns with files/locks
