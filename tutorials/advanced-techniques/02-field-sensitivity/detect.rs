// Explore field-sensitive pointer analysis using the SAF Rust API.
//
// This tutorial shows how field sensitivity affects PTA precision
// and how to configure the max_depth parameter.
//
// Build and run:
//   cargo run --features llvm-18 --example detect_field_sensitive

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use saf_analysis::{FieldSensitivity, PtaConfig, PtaContext, PtaResult};
use saf_core::config::Config;
use saf_frontends::api::Frontend;
use saf_frontends::llvm::LlvmFrontend;

fn main() {
    let tutorial_dir = PathBuf::from("tutorials/pta/04-field-sensitive-structs");
    let source = tutorial_dir.join("program.c");
    let llvm_ir = tutorial_dir.join("program.ll");

    // Step 1: Compile C source to LLVM IR
    let status = Command::new("clang-18")
        .args(["-S", "-emit-llvm", "-O0", "-g",
               "-o", llvm_ir.to_str().unwrap(), source.to_str().unwrap()])
        .status()
        .expect("failed to run clang-18");
    assert!(status.success(), "clang-18 compilation failed");

    // Step 2: Load via LLVM frontend
    let frontend = LlvmFrontend::new();
    let config = Config::default();
    let bundle = frontend.ingest(&[llvm_ir.as_path()], &config)
        .expect("failed to load LLVM IR");
    let module = bundle.module;

    // Step 3: Run field-sensitive PTA with max_depth=3
    // Higher depth tracks more levels of nested struct fields
    let pta_config = PtaConfig {
        enabled: true,
        field_sensitivity: FieldSensitivity::StructFields { max_depth: 3 },
        max_objects: 100_000,
        max_iterations: 1_000_000,
    };
    let mut pta_ctx = PtaContext::new(pta_config);
    let pta_analysis = pta_ctx.analyze(&module);
    let pta_result = PtaResult::new(
        pta_analysis.pts,
        Arc::new(pta_analysis.factory),
        pta_analysis.diagnostics,
    );

    let export = pta_result.export();
    println!("Field-sensitive PTA (max_depth=3):");
    println!("  Points-to entries: {}", export.points_to.len());
    println!("  Diagnostics: {}", pta_result.diagnostics().len());

    // Step 4: Show all points-to sets
    println!("\nPoints-to sets:");
    for (val_id, locs) in export.points_to.iter().take(20) {
        println!("  {:?} -> {} location(s)", val_id, locs.len());
    }

    // Step 5: Count entries with multiple targets
    let multi_count = export.points_to.values().filter(|v| v.len() > 1).count();
    println!("\nValues with multiple targets: {}", multi_count);
}
