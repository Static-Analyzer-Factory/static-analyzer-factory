//! E2E tests for Multi-Threaded Analysis (MTA).
//!
//! Tests MTA analysis on existing fixtures to verify thread discovery and MHP work.
#![cfg(feature = "analysis-mta")]

use saf_analysis::callgraph::CallGraph;
use saf_analysis::icfg::Icfg;
use saf_analysis::mta::{MtaAnalysis, MtaConfig, ThreadId};
use saf_test_utils::load_ll_fixture;

/// Test MTA on a simple C program (uses `checker_leak_simple` which has main).
/// Verifies that main thread discovery works correctly.
#[test]
fn mta_discovers_main_thread_from_simple_program() {
    let module = load_ll_fixture("checker_leak_simple");
    let callgraph = CallGraph::build(&module);
    let icfg = Icfg::build(&module, &callgraph);
    let config = MtaConfig::default();

    let mta = MtaAnalysis::new(&module, &callgraph, &icfg, config);
    let result = mta.analyze();

    // Main thread (thread 0) should always exist
    assert!(
        result.threads().contains_key(&ThreadId::MAIN),
        "main thread (thread 0) should be discovered"
    );

    let main_thread = result.threads().get(&ThreadId::MAIN).unwrap();
    assert_eq!(main_thread.entry_function_name, "main");
    assert!(main_thread.parent.is_none(), "main thread has no parent");
}

/// Test that MTA exports produce valid JSON.
#[test]
fn mta_export_produces_valid_json() {
    let module = load_ll_fixture("checker_leak_simple");
    let callgraph = CallGraph::build(&module);
    let icfg = Icfg::build(&module, &callgraph);
    let config = MtaConfig::default();

    let mta = MtaAnalysis::new(&module, &callgraph, &icfg, config);
    let result = mta.analyze();

    let export = result.export();
    let json = serde_json::to_string(&export).expect("export should serialize to JSON");

    assert!(!json.is_empty(), "exported JSON should not be empty");
    assert!(
        json.contains("threads"),
        "exported JSON should contain threads"
    );
}

/// Test determinism: same input produces same output.
#[test]
fn mta_is_deterministic() {
    let module = load_ll_fixture("checker_leak_simple");
    let callgraph = CallGraph::build(&module);
    let icfg = Icfg::build(&module, &callgraph);
    let config = MtaConfig::default();

    let mta1 = MtaAnalysis::new(&module, &callgraph, &icfg, config.clone());
    let result1 = mta1.analyze();

    let mta2 = MtaAnalysis::new(&module, &callgraph, &icfg, config);
    let result2 = mta2.analyze();

    // Thread counts should match
    assert_eq!(
        result1.threads().len(),
        result2.threads().len(),
        "thread counts should match"
    );

    // Export should be identical
    let export1 = serde_json::to_string(&result1.export()).unwrap();
    let export2 = serde_json::to_string(&result2.export()).unwrap();
    assert_eq!(export1, export2, "exports should be byte-identical");
}

/// Test MTA config defaults are sane.
#[test]
fn mta_config_defaults() {
    let config = MtaConfig::default();

    assert_eq!(config.max_context_depth, 10);
    assert!(!config.track_locks);
    assert!(
        config
            .thread_create_funcs
            .contains(&"pthread_create".to_string())
    );
    assert!(
        config
            .thread_join_funcs
            .contains(&"pthread_join".to_string())
    );
}
