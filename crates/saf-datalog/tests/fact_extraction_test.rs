use saf_analysis::{FieldSensitivity, LocationFactory};
use saf_datalog::facts::{AnalysisScope, extract_facts, extract_facts_default};
use saf_test_utils::load_ll_fixture;

fn make_factory() -> LocationFactory {
    LocationFactory::new(FieldSensitivity::default())
}

#[test]
fn extract_facts_produces_addr_constraints() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory = make_factory();
    let facts = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);

    assert!(
        !facts.addr_of.is_empty(),
        "expected addr_of facts from constraint extraction fixture"
    );
}

#[test]
fn extract_facts_produces_copy_constraints() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory = make_factory();
    let facts = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);

    assert!(
        !facts.copy.is_empty(),
        "expected copy facts from constraint extraction fixture"
    );
}

#[test]
fn extract_facts_total_is_consistent() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory = make_factory();
    let facts = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);

    let expected_total = facts.addr_of.len()
        + facts.copy.len()
        + facts.load.len()
        + facts.store.len()
        + facts.gep.len();
    assert_eq!(facts.total(), expected_total);
    assert!(!facts.is_empty());
}

#[test]
fn extract_facts_intraprocedural_has_fewer_or_equal_copies() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory1 = make_factory();
    let mut factory2 = make_factory();

    let whole = extract_facts(&module, &mut factory1, AnalysisScope::WholeProgram);
    let intra = extract_facts(&module, &mut factory2, AnalysisScope::Intraprocedural);

    // Whole program includes interprocedural arg->param and return->caller copies,
    // so it should have >= intraprocedural copy facts.
    assert!(
        whole.copy.len() >= intra.copy.len(),
        "whole-program copies ({}) should >= intraprocedural copies ({})",
        whole.copy.len(),
        intra.copy.len(),
    );

    // Both should produce the same addr_of facts (base constraints are always extracted).
    assert_eq!(
        whole.addr_of.len(),
        intra.addr_of.len(),
        "addr_of should be identical for both scopes"
    );
}

#[test]
fn extract_facts_default_convenience_works() {
    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let (facts, _factory) = extract_facts_default(&module);

    assert!(
        !facts.is_empty(),
        "extract_facts_default should produce non-empty facts"
    );
}

#[test]
fn extract_facts_reachable_delegates_to_whole_program() {
    use std::collections::BTreeSet;

    let module = load_ll_fixture("pta_verification_constraint_extraction");
    let mut factory1 = make_factory();
    let mut factory2 = make_factory();

    let whole = extract_facts(&module, &mut factory1, AnalysisScope::WholeProgram);
    let reachable = extract_facts(
        &module,
        &mut factory2,
        AnalysisScope::Reachable(BTreeSet::new()),
    );

    // Until extract_constraints_reachable is exposed, Reachable delegates
    // to WholeProgram, so results should be identical.
    assert_eq!(whole.total(), reachable.total());
}

#[test]
fn extract_facts_empty_module_produces_empty_facts() {
    use saf_core::air::AirModule;
    use saf_core::ids::ModuleId;

    let module = AirModule::new(ModuleId::derive(b"empty_test"));
    let mut factory = make_factory();
    let facts = extract_facts(&module, &mut factory, AnalysisScope::WholeProgram);

    assert!(facts.is_empty(), "empty module should produce no facts");
    assert_eq!(facts.total(), 0);
}
