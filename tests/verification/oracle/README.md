# Oracle Verification Suite

Hand-crafted C programs with known-correct analysis results.
Tests core library outputs (PTA, CG, CFG, MSSA, SVFG), NOT checker verdicts.

## Adding a new test

1. Write a small C program: `<layer>/<name>.c` (10-30 lines)
2. Write expected results: `<layer>/<name>.expected.yaml`
3. Run `make compile-oracle` to compile C → LLVM IR
4. Run `make verify-oracle` to check

## Verdict types

- **PASS**: Analysis results match expectations exactly
- **WARN**: Imprecision — extra items in points-to sets (acceptable)
- **FAIL**: Unsoundness — missing items in must_contain (BUG)

## Schema

See `schema.yaml` for the expected results format.
