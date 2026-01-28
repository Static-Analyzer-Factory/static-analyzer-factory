//! Convert function specs to taint selectors.
//!
//! This module bridges the function specification system with the selector-based
//! taint analysis infrastructure.

use saf_core::spec::{Role, SpecRegistry};

use super::Selector;

/// Extract taint source selectors from a spec registry.
///
/// Returns selectors for all functions marked with `role: source`.
#[must_use]
pub fn sources_from_specs(specs: &SpecRegistry) -> Vec<Selector> {
    let mut selectors = Vec::new();

    // Check exact specs
    for spec in specs.iter() {
        if spec.role == Some(Role::Source) {
            selectors.push(Selector::call_to(&spec.name));
        }
    }

    // Check pattern specs
    for spec in specs.patterns() {
        if spec.role == Some(Role::Source) {
            selectors.push(Selector::call_to(&spec.name));
        }
    }

    selectors
}

/// Extract taint sink selectors from a spec registry.
///
/// Returns selectors for all functions marked with `role: sink`.
#[must_use]
pub fn sinks_from_specs(specs: &SpecRegistry) -> Vec<Selector> {
    let mut selectors = Vec::new();

    for spec in specs.iter() {
        if spec.role == Some(Role::Sink) {
            // Sink is typically the arguments to the function
            selectors.push(Selector::arg_to(&spec.name, None));
        }
    }

    for spec in specs.patterns() {
        if spec.role == Some(Role::Sink) {
            selectors.push(Selector::arg_to(&spec.name, None));
        }
    }

    selectors
}

/// Extract sanitizer selectors from a spec registry.
///
/// Returns selectors for all functions marked with `role: sanitizer`.
#[must_use]
pub fn sanitizers_from_specs(specs: &SpecRegistry) -> Vec<Selector> {
    let mut selectors = Vec::new();

    for spec in specs.iter() {
        if spec.role == Some(Role::Sanitizer) {
            selectors.push(Selector::call_to(&spec.name));
        }
    }

    for spec in specs.patterns() {
        if spec.role == Some(Role::Sanitizer) {
            selectors.push(Selector::call_to(&spec.name));
        }
    }

    selectors
}

/// Extract all taint-relevant selectors from a spec registry.
///
/// Returns (sources, sinks, sanitizers) tuple.
#[must_use]
pub fn taint_selectors_from_specs(
    specs: &SpecRegistry,
) -> (Vec<Selector>, Vec<Selector>, Vec<Selector>) {
    (
        sources_from_specs(specs),
        sinks_from_specs(specs),
        sanitizers_from_specs(specs),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::spec::FunctionSpec;

    #[test]
    fn test_sources_from_specs() {
        let mut registry = SpecRegistry::new();

        let mut getenv = FunctionSpec::new("getenv");
        getenv.role = Some(Role::Source);
        registry.add(getenv).unwrap();

        let mut recv = FunctionSpec::new("recv");
        recv.role = Some(Role::Source);
        registry.add(recv).unwrap();

        let sources = sources_from_specs(&registry);
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn test_sinks_from_specs() {
        let mut registry = SpecRegistry::new();

        let mut system = FunctionSpec::new("system");
        system.role = Some(Role::Sink);
        registry.add(system).unwrap();

        let sinks = sinks_from_specs(&registry);
        assert_eq!(sinks.len(), 1);
        assert!(matches!(&sinks[0], Selector::ArgTo { callee, .. } if callee == "system"));
    }

    #[test]
    fn test_sanitizers_from_specs() {
        let mut registry = SpecRegistry::new();

        let mut escape = FunctionSpec::new("html_escape");
        escape.role = Some(Role::Sanitizer);
        registry.add(escape).unwrap();

        let sanitizers = sanitizers_from_specs(&registry);
        assert_eq!(sanitizers.len(), 1);
    }

    #[test]
    fn test_mixed_roles() {
        let mut registry = SpecRegistry::new();

        let mut source = FunctionSpec::new("read_input");
        source.role = Some(Role::Source);
        registry.add(source).unwrap();

        let mut sink = FunctionSpec::new("write_output");
        sink.role = Some(Role::Sink);
        registry.add(sink).unwrap();

        let mut sanitizer = FunctionSpec::new("validate");
        sanitizer.role = Some(Role::Sanitizer);
        registry.add(sanitizer).unwrap();

        let mut allocator = FunctionSpec::new("malloc");
        allocator.role = Some(Role::Allocator);
        registry.add(allocator).unwrap();

        let (sources, sinks, sanitizers) = taint_selectors_from_specs(&registry);

        assert_eq!(sources.len(), 1);
        assert_eq!(sinks.len(), 1);
        assert_eq!(sanitizers.len(), 1);
    }
}
