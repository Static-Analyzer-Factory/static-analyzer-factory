# Test Fixtures

This directory contains test fixtures for SAF integration and determinism tests.

## Structure

- `bc/` — LLVM bitcode (`.bc`) fixtures compiled from C/Rust sources
- `air_json/` — AIR JSON (`.air.json`) fixtures representing equivalent programs
- `expected/` — Expected outputs for determinism regression tests

## Conventions

Each fixture set should include both an LLVM bitcode file and its AIR JSON
equivalent so that cross-frontend tests (AT-EXT-01) can verify identical
analysis results.
