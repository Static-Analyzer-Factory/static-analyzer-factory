#!/usr/bin/env rust-script
//! Dump AIR JSON from inkwell LLVM frontend.
//! This is a conceptual script — run via `cargo test` in saf-frontends instead.
//! See the actual test below.

// The actual comparison should be done via:
// 1. `cargo test dump_air_json -- --nocapture` inside Docker
// 2. Then compare the output files with tree-sitter converter output
