#![cfg(feature = "llvm-22")]

use std::collections::BTreeSet;

use saf_analysis::callgraph::CallGraph;
use saf_analysis::defuse::DefUseGraph;
use saf_analysis::selector::Selector;
use saf_analysis::{QueryLimits, ValueFlowBuilder, ValueFlowConfig};
use saf_core::air::AirModule;
use saf_core::ids::ValueId;
use saf_test_utils::load_ll_fixture;

fn build_fast_vf(module: &AirModule) -> saf_analysis::ValueFlowGraph {
    let defuse = DefUseGraph::build(module);
    let callgraph = CallGraph::build(module);
    let config = ValueFlowConfig::fast();
    let builder = ValueFlowBuilder::new(&config, module, &defuse, &callgraph, None);
    builder.build()
}

fn resolve(selector: &Selector, module: &AirModule) -> BTreeSet<ValueId> {
    selector
        .resolve(module)
        .expect("selector resolution failed")
}

#[test]
fn rust_env_args_to_std_command_new_finds_flow() {
    let module = load_ll_fixture("rust_std_command");
    let graph = build_fast_vf(&module);

    let sources = resolve(&Selector::call_to("*env*args*"), &module);
    let sinks = resolve(&Selector::arg_to("*Command*new*", Some(1)), &module);
    let limits = QueryLimits::default();

    assert!(!sources.is_empty(), "env::args source call should resolve");
    assert!(!sinks.is_empty(), "Command::new program argument should resolve");

    let flows = graph.taint_flow(&sources, &sinks, &BTreeSet::new(), &limits);
    assert!(
        !flows.is_empty(),
        "should find taint flow from env::args() to std::process::Command::new"
    );
}
