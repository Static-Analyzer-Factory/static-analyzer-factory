use saf_core::air::{AirBundle, AirModule};
use saf_core::id::{id_to_hex, make_id};
use saf_core::ids::{BlockId, FunctionId, ModuleId, ValueId};

#[test]
fn id_generation_is_deterministic() {
    let a = make_id("smoke", b"test-data");
    let b = make_id("smoke", b"test-data");
    assert_eq!(a, b);

    let hex_a = id_to_hex(a);
    let hex_b = id_to_hex(b);
    assert_eq!(hex_a, hex_b);
    assert!(hex_a.starts_with("0x"));
    assert_eq!(hex_a.len(), 34);
}

#[test]
fn air_module_round_trip() {
    let mut module = AirModule::new(ModuleId::derive(b"smoke-test"));
    module.name = Some("test".to_string());

    let bundle = AirBundle::new("smoke", module);
    let json = serde_json::to_string_pretty(&bundle).expect("serialize");
    let parsed: AirBundle = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.module.name, Some("test".to_string()));
    assert_eq!(parsed.frontend_id, "smoke");
    assert_eq!(parsed.schema_version, AirBundle::SCHEMA_VERSION);
}

#[test]
fn id_domain_separation() {
    let func_id = FunctionId::derive(b"same-input");
    let block_id = BlockId::derive(b"same-input");

    // BLAKE3 domain separation: same input bytes, different ID types => different raw values
    assert_ne!(func_id.raw(), block_id.raw());
}

#[test]
fn id_deterministic_across_calls() {
    let a = ValueId::derive(b"deterministic-check");
    let b = ValueId::derive(b"deterministic-check");

    assert_eq!(a, b);
    assert_eq!(a.raw(), b.raw());
    assert_eq!(a.to_hex(), b.to_hex());
}

#[test]
fn derived_spec_alloc_size_bound() {
    use saf_core::spec::{BoundMode, ComputedBound, DerivedSpec};
    use std::collections::BTreeMap;

    let bound = ComputedBound {
        param_index: 0,
        mode: BoundMode::AllocSizeMinusOne,
    };
    let spec = DerivedSpec {
        computed_return_bound: Some(bound),
        param_freed: BTreeMap::new(),
        param_dereferenced: BTreeMap::new(),
        return_is_allocated: false,
    };
    assert_eq!(spec.computed_return_bound.as_ref().unwrap().param_index, 0);
    assert!(matches!(
        spec.computed_return_bound.as_ref().unwrap().mode,
        BoundMode::AllocSizeMinusOne
    ));
}

#[test]
fn analyzed_registry_yaml_lookup() {
    use saf_core::spec::{AnalyzedSpecRegistry, FunctionSpec, ReturnSpec, SpecRegistry};

    let mut yaml_reg = SpecRegistry::default();
    let mut spec = FunctionSpec::new("rand");
    spec.returns = Some(ReturnSpec {
        interval: Some((0, 2_147_483_647)),
        ..ReturnSpec::default()
    });
    yaml_reg.add(spec).unwrap();

    let analyzed = AnalyzedSpecRegistry::new(yaml_reg);
    let result = analyzed.lookup("rand");
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(matches!(
        result,
        saf_core::spec::LookupResult::Yaml(_, None)
    ));
    let func_spec = result.yaml().unwrap();
    assert_eq!(
        func_spec.returns.as_ref().unwrap().interval,
        Some((0, 2_147_483_647))
    );
    assert!(result.derived().is_none()); // No analysis overlay for rand
}

#[test]
fn analyzed_registry_derived_overlay() {
    use saf_core::spec::{
        AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec, FunctionSpec, ReturnSpec,
        SpecRegistry,
    };

    let mut yaml_reg = SpecRegistry::default();
    let mut spec = FunctionSpec::new("strlen");
    spec.returns = Some(ReturnSpec {
        interval: Some((0, 9_223_372_036_854_775_807)),
        ..ReturnSpec::default()
    });
    yaml_reg.add(spec).unwrap();

    let mut analyzed = AnalyzedSpecRegistry::new(yaml_reg);

    let derived = DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSizeMinusOne,
        }),
        ..DerivedSpec::empty()
    };
    analyzed.add_derived("strlen", derived);

    let result = analyzed.lookup("strlen");
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(matches!(
        result,
        saf_core::spec::LookupResult::Yaml(_, Some(_))
    ));
    // YAML spec still accessible
    let func_spec = result.yaml().unwrap();
    assert!(func_spec.returns.is_some());
    // Derived overlay also accessible
    let derived = result.derived().unwrap();
    assert!(derived.computed_return_bound.is_some());
    assert_eq!(
        derived.computed_return_bound.as_ref().unwrap().param_index,
        0
    );
}

#[test]
fn analyzed_registry_derived_only() {
    use saf_core::spec::{AnalyzedSpecRegistry, DerivedSpec, LookupResult, SpecRegistry};
    use std::collections::BTreeMap;

    let yaml_reg = SpecRegistry::default(); // empty
    let mut analyzed = AnalyzedSpecRegistry::new(yaml_reg);

    let derived = DerivedSpec::from_effects(BTreeMap::from([(0, true)]), BTreeMap::new(), false);
    analyzed.add_derived("my_free_wrapper", derived);

    // Verify lookup() now returns DerivedOnly variant (was previously None)
    let result = analyzed.lookup("my_free_wrapper");
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(matches!(result, LookupResult::DerivedOnly(_)));
    assert!(result.yaml().is_none());
    assert!(result.derived().unwrap().param_freed[&0]);
}

#[test]
fn analyzed_registry_default() {
    use saf_core::spec::AnalyzedSpecRegistry;

    let reg = AnalyzedSpecRegistry::default();
    assert!(reg.lookup("nonexistent").is_none());
    assert_eq!(reg.derived_count(), 0);
}

#[test]
fn analyzed_registry_add_derived_overwrite() {
    use saf_core::spec::{AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec};

    let mut reg = AnalyzedSpecRegistry::default();

    // First add
    reg.add_derived(
        "foo",
        DerivedSpec {
            computed_return_bound: Some(ComputedBound {
                param_index: 0,
                mode: BoundMode::AllocSizeMinusOne,
            }),
            ..DerivedSpec::empty()
        },
    );
    assert_eq!(reg.derived_count(), 1);

    // Overwrite with different spec
    reg.add_derived(
        "foo",
        DerivedSpec {
            computed_return_bound: Some(ComputedBound {
                param_index: 1,
                mode: BoundMode::AllocSize,
            }),
            ..DerivedSpec::empty()
        },
    );
    assert_eq!(reg.derived_count(), 1); // Still 1, not 2
    let derived = reg.lookup_derived("foo").unwrap();
    assert_eq!(
        derived.computed_return_bound.as_ref().unwrap().param_index,
        1
    );
    assert!(matches!(
        derived.computed_return_bound.as_ref().unwrap().mode,
        BoundMode::AllocSize
    ));
}

#[test]
fn analyzed_registry_iter_derived() {
    use saf_core::spec::{AnalyzedSpecRegistry, BoundMode, ComputedBound, DerivedSpec};

    let mut reg = AnalyzedSpecRegistry::default();
    reg.add_derived(
        "alpha",
        DerivedSpec {
            computed_return_bound: Some(ComputedBound {
                param_index: 0,
                mode: BoundMode::AllocSizeMinusOne,
            }),
            ..DerivedSpec::empty()
        },
    );
    reg.add_derived(
        "beta",
        DerivedSpec {
            computed_return_bound: Some(ComputedBound {
                param_index: 0,
                mode: BoundMode::AllocSize,
            }),
            ..DerivedSpec::empty()
        },
    );

    assert_eq!(reg.derived_count(), 2);
    let names: Vec<&str> = reg.iter_derived().map(|(name, _)| name).collect();
    assert_eq!(names, vec!["alpha", "beta"]); // BTreeMap = sorted
}

#[test]
fn derived_spec_bound_mode_variants() {
    use saf_core::spec::{BoundMode, ComputedBound, DerivedSpec};

    // AllocSize variant
    let alloc_size_spec = DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 0,
            mode: BoundMode::AllocSize,
        }),
        ..DerivedSpec::empty()
    };
    assert!(matches!(
        alloc_size_spec.computed_return_bound.as_ref().unwrap().mode,
        BoundMode::AllocSize
    ));

    // ParamValueMinusOne variant
    let param_val_spec = DerivedSpec {
        computed_return_bound: Some(ComputedBound {
            param_index: 2,
            mode: BoundMode::ParamValueMinusOne,
        }),
        ..DerivedSpec::empty()
    };
    assert_eq!(
        param_val_spec
            .computed_return_bound
            .as_ref()
            .unwrap()
            .param_index,
        2
    );
    assert!(matches!(
        param_val_spec.computed_return_bound.as_ref().unwrap().mode,
        BoundMode::ParamValueMinusOne
    ));
}

#[test]
fn derived_spec_partial_eq() {
    use saf_core::spec::DerivedSpec;

    let a = DerivedSpec::empty();
    let b = DerivedSpec::empty();
    assert_eq!(a, b);

    let c = DerivedSpec::from_effects(
        std::collections::BTreeMap::from([(0, true)]),
        std::collections::BTreeMap::new(),
        false,
    );
    assert_ne!(a, c);
}
