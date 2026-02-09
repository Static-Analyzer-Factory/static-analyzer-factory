# Buffer Overflow

This category covers buffer overflow vulnerabilities including heap overflows, stack overflows, and size calculation errors. Tutorials progress from simple taint detection to complex multi-technique analysis.

## CWE Coverage

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-120 | Buffer Copy without Size Check (Heap) | `01-taint-detection`, `02-interval-analysis` |
| CWE-121 | Stack-based Buffer Overflow | `03-complex-patterns` |
| CWE-131 | Incorrect Calculation of Buffer Size | `02-interval-analysis` |

## Learning Objectives

After completing these tutorials, you will be able to:
- Detect heap overflows via taint flow from user input
- Use interval analysis for off-by-one detection
- Combine CS-PTA and Z3 for complex overflow patterns

## Tutorials

| # | Tutorial | Difficulty | Technique |
|---|----------|------------|-----------|
| 01 | [Taint Detection](01-taint-detection/) | Beginner | Taint flow |
| 02 | [Interval Analysis](02-interval-analysis/) | Intermediate | Abstract interpretation |
| 03 | [Complex Patterns](03-complex-patterns/) | Advanced | CS-PTA + Z3 |

## Difficulty Progression

- **Beginner (01)**: Direct user input → buffer write overflow
- **Intermediate (02)**: Off-by-one errors detected via interval analysis
- **Advanced (03)**: Indirect flows through pointers requiring CS-PTA + Z3

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first
- For tutorial 03, familiarity with pointer analysis and Z3 concepts helps

## Related Categories

- [Memory Safety](../memory-safety/) for UAF/double-free (related memory issues)
- [Integer Issues](../integer-issues/) for size calculation overflows
