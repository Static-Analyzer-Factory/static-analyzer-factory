# Integer Issues

This category covers integer overflow and underflow vulnerabilities. Tutorials cover basic arithmetic overflow and allocation size calculation issues.

## CWE Coverage

| CWE | Description | Tutorials |
|-----|-------------|-----------|
| CWE-190 | Integer Overflow | `01-basic-overflow`, `02-size-calculation` |
| CWE-191 | Integer Underflow | `01-basic-overflow` |

## Learning Objectives

After completing these tutorials, you will be able to:
- Detect arithmetic overflows using interval analysis
- Find allocation size calculation bugs
- Understand wrapped vs unwrapped integer semantics

## Tutorials

| # | Tutorial | Difficulty | Technique |
|---|----------|------------|-----------|
| 01 | [Basic Overflow](01-basic-overflow/) | Beginner | Interval analysis |
| 02 | [Size Calculation](02-size-calculation/) | Intermediate | Interval + allocation tracking |

## Difficulty Progression

- **Beginner (01)**: Direct arithmetic overflow (`a + b` exceeds bit width)
- **Intermediate (02)**: Overflow in allocation size calculation (`width * height * bpp`)

## Prerequisites

- Complete [Getting Started](../getting-started/) tutorials first
- Basic understanding of fixed-width integer arithmetic

## Key Concepts

### Wrapped Semantics
SAF's interval analysis uses wrapped semantics matching LLVM IR's modular arithmetic. An operation like `255 + 1` on an 8-bit unsigned integer wraps to `0`, not `256`.

### Three-Valued Results
Integer overflow checkers return:
- **Safe**: Provably no overflow
- **Warning**: Possible overflow (inconclusive)
- **Error**: Definite overflow

## Related Categories

- [Buffer Overflow](../buffer-overflow/) for allocation size issues leading to overflows
