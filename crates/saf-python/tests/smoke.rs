/// Smoke test: saf-python crate compiles.
///
/// Actual Python integration tests live in python/tests/ and run via pytest.
/// This test only verifies the Rust cdylib compiles.
#[test]
fn crate_compiles() {
    // cdylib crate — no Rust-level public API to test here.
    // Python-level tests are in python/tests/test_smoke.py.
    assert!(true);
}
