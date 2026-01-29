/// Smoke test: saf-analysis crate compiles and can be used.
#[test]
fn crate_compiles() {
    // Verify we can reference types from the crate.
    let _err = saf_analysis::error::AnalysisError::NotImplemented("smoke".to_string());
}
