//! E2E for the correctness-witness driver (plan 207 Slice 2): SAF's converged
//! interval fixpoint over a real mem2reg'd loop program -> an `invariant_set`
//! witness naming the loop induction variable.
//!
//! Uses the committed fixture `dbgvalue_promoted_phi` (`for (i=0;i<100;i++) s+=i`,
//! a bounded loop with a NON-function-call guard, so the driver does not abstain).

use std::path::PathBuf;

use saf_svcomp::{DataModel, Language, WitnessMeta, build_interval_invariant_witness};
use saf_test_utils::load_ll_fixture;

#[test]
fn driver_emits_invariant_for_promoted_loop_counter() {
    let module = load_ll_fixture("dbgvalue_promoted_phi");
    let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .join("tests/programs/c/dbgvalue_promoted_phi.c");
    let source = std::fs::read_to_string(&src_path).expect("read fixture source");

    let meta = WitnessMeta {
        producer_version: "test".to_string(),
        specification: "G ! call(reach_error())".to_string(),
        data_model: DataModel::ILP32,
        language: Language::C,
        input_file: src_path,
    };

    let witness = build_interval_invariant_witness(&module, &source, &meta)
        .expect("driver should emit a witness for a bounded loop with named induction vars");
    let yaml = witness.to_yaml_string().expect("serialize");

    assert!(yaml.contains("entry_type: invariant_set"), "{yaml}");
    assert!(yaml.contains("type: loop_invariant"), "{yaml}");
    assert!(yaml.contains("format: c_expression"), "{yaml}");
    assert!(yaml.contains("function: main"), "{yaml}");
    // The loop counter `i` must appear with an interval bound.
    assert!(
        yaml.contains("<= i") || yaml.contains("i <=") || yaml.contains("i =="),
        "expected an interval bound on `i`\n{yaml}"
    );
}
