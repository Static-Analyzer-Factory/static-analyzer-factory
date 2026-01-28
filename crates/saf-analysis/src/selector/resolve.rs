//! Selector resolution logic.

use std::collections::BTreeSet;

use saf_core::air::{AirModule, Operation};
use saf_core::ids::ValueId;

use super::Selector;

/// Error during selector resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Invalid hex ID format.
    InvalidHexId(String),
    /// Invalid glob pattern.
    InvalidPattern(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHexId(id) => write!(f, "invalid hex ID: {id}"),
            Self::InvalidPattern(pat) => write!(f, "invalid pattern: {pat}"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// Resolver for selectors against an AIR module.
pub struct SelectorResolver<'a> {
    module: &'a AirModule,
}

impl<'a> SelectorResolver<'a> {
    /// Create a new resolver for the given module.
    #[must_use]
    pub const fn new(module: &'a AirModule) -> Self {
        Self { module }
    }

    /// Resolve a selector to a set of value IDs.
    ///
    /// # Errors
    /// Returns an error if the selector cannot be resolved.
    pub fn resolve(&self, selector: &Selector) -> Result<BTreeSet<ValueId>, ResolveError> {
        match selector {
            Selector::ValueId { id } => Self::resolve_value_id(id),
            Selector::FunctionParam { function, index } => {
                self.resolve_function_param(function, *index)
            }
            Selector::FunctionReturn { function } => self.resolve_function_return(function),
            Selector::FunctionAll { function } => self.resolve_function_all(function),
            Selector::Global { name } => self.resolve_global(name),
            Selector::CallTo { callee } => self.resolve_call_to(callee),
            Selector::ArgTo { callee, index } => self.resolve_arg_to(callee, *index),
        }
    }

    fn resolve_value_id(id: &str) -> Result<BTreeSet<ValueId>, ResolveError> {
        let hex_str = id.strip_prefix("0x").unwrap_or(id);
        let raw = u128::from_str_radix(hex_str, 16)
            .map_err(|_| ResolveError::InvalidHexId(id.to_string()))?;
        Ok([ValueId::new(raw)].into_iter().collect())
    }

    // NOTE: The remaining resolve_* methods return `Result` for API consistency — all
    // methods share the same signature allowing uniform error handling at call sites,
    // even though these particular methods cannot fail. This matches the delegation
    // pattern in `resolve()`.

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_function_param(
        &self,
        pattern: &str,
        index: Option<u32>,
    ) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        for func in &self.module.functions {
            if matches_pattern(&func.name, pattern) {
                match index {
                    Some(i) => {
                        if let Some(param) = func.params.get(i as usize) {
                            result.insert(param.id);
                        }
                    }
                    None => {
                        for param in &func.params {
                            result.insert(param.id);
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_function_return(&self, pattern: &str) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        for func in &self.module.functions {
            if matches_pattern(&func.name, pattern) {
                for block in &func.blocks {
                    for inst in &block.instructions {
                        if let Operation::Ret = &inst.op {
                            // Return operand is the returned value
                            if let Some(ret_val) = inst.operands.first() {
                                result.insert(*ret_val);
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_function_all(&self, pattern: &str) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        for func in &self.module.functions {
            if matches_pattern(&func.name, pattern) {
                // Add parameters
                for param in &func.params {
                    result.insert(param.id);
                }
                // Add instruction results
                for block in &func.blocks {
                    for inst in &block.instructions {
                        if let Some(dst) = inst.dst {
                            result.insert(dst);
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_global(&self, pattern: &str) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        for global in &self.module.globals {
            if matches_pattern(&global.name, pattern) {
                result.insert(global.id);
            }
        }
        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_call_to(&self, pattern: &str) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        // Find function IDs matching the pattern
        let matching_funcs: BTreeSet<_> = self
            .module
            .functions
            .iter()
            .filter(|f| matches_pattern(&f.name, pattern))
            .map(|f| f.id)
            .collect();

        // Find call instructions to those functions, and HeapAlloc operations
        // whose kind matches the pattern (for malloc, calloc, etc.)
        for func in &self.module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    match &inst.op {
                        Operation::CallDirect { callee } => {
                            if matching_funcs.contains(callee) {
                                if let Some(dst) = inst.dst {
                                    result.insert(dst);
                                }
                            }
                        }
                        Operation::HeapAlloc { kind } => {
                            // HeapAlloc operations are recognized heap allocations
                            // (malloc, calloc, etc.) - match against the kind string
                            if matches_pattern(kind.as_str(), pattern) {
                                if let Some(dst) = inst.dst {
                                    result.insert(dst);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)] // API consistency with other resolve_* methods
    fn resolve_arg_to(
        &self,
        pattern: &str,
        index: Option<u32>,
    ) -> Result<BTreeSet<ValueId>, ResolveError> {
        let mut result = BTreeSet::new();
        // Find function IDs matching the pattern
        let matching_funcs: BTreeSet<_> = self
            .module
            .functions
            .iter()
            .filter(|f| matches_pattern(&f.name, pattern))
            .map(|f| f.id)
            .collect();

        // Find call instructions to those functions and extract arguments,
        // also check HeapAlloc operations
        for func in &self.module.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    let matches = match &inst.op {
                        Operation::CallDirect { callee } => matching_funcs.contains(callee),
                        Operation::HeapAlloc { kind } => matches_pattern(kind.as_str(), pattern),
                        _ => false,
                    };

                    if matches {
                        match index {
                            Some(i) => {
                                if let Some(arg) = inst.operands.get(i as usize) {
                                    result.insert(*arg);
                                }
                            }
                            None => {
                                for arg in &inst.operands {
                                    result.insert(*arg);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}

/// Simple glob-style pattern matching.
///
/// Supports:
/// - `*` matches any sequence of characters
/// - `?` matches any single character
/// - Literal characters match themselves
fn matches_pattern(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let mut text_chars = text.chars().peekable();
    let mut pattern_chars = pattern.chars().peekable();

    matches_pattern_impl(&mut text_chars, &mut pattern_chars)
}

fn matches_pattern_impl(
    text: &mut std::iter::Peekable<std::str::Chars>,
    pattern: &mut std::iter::Peekable<std::str::Chars>,
) -> bool {
    loop {
        match (pattern.peek(), text.peek()) {
            (None, None) => return true,
            (Some('*'), _) => {
                pattern.next();
                // Try matching zero or more characters
                if pattern.peek().is_none() {
                    return true; // * at end matches everything
                }
                // Try matching at current position and all subsequent positions
                let mut text_clone = text.clone();
                loop {
                    let mut pattern_clone = pattern.clone();
                    if matches_pattern_impl(&mut text_clone.clone(), &mut pattern_clone) {
                        return true;
                    }
                    if text_clone.next().is_none() {
                        return false;
                    }
                }
            }
            (Some('?'), Some(_)) => {
                pattern.next();
                text.next();
            }
            (Some(p), Some(t)) if *p == *t => {
                pattern.next();
                text.next();
            }
            // All other cases: mismatch or pattern char with no text
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use saf_core::air::{AirBlock, AirFunction, AirGlobal, AirParam, Instruction};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ObjId};

    fn make_test_module() -> AirModule {
        let func1 = AirFunction {
            id: FunctionId::new(1),
            name: "main".to_string(),
            params: vec![
                AirParam::new(ValueId::new(10), 0),
                AirParam::new(ValueId::new(11), 1),
            ],
            blocks: vec![AirBlock {
                id: BlockId::new(1),
                label: None,
                instructions: vec![
                    Instruction::new(InstId::new(100), Operation::Alloca { size_bytes: None })
                        .with_dst(ValueId::new(20)),
                    Instruction::new(
                        InstId::new(101),
                        Operation::CallDirect {
                            callee: FunctionId::new(2),
                        },
                    )
                    .with_operands(vec![ValueId::new(10)])
                    .with_dst(ValueId::new(21)),
                    Instruction::new(InstId::new(102), Operation::Ret)
                        .with_operands(vec![ValueId::new(21)]),
                ],
            }],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let func2 = AirFunction {
            id: FunctionId::new(2),
            name: "helper".to_string(),
            params: vec![AirParam::new(ValueId::new(30), 0)],
            blocks: vec![AirBlock {
                id: BlockId::new(2),
                label: None,
                instructions: vec![
                    Instruction::new(InstId::new(200), Operation::Ret)
                        .with_operands(vec![ValueId::new(30)]),
                ],
            }],
            entry_block: None,
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let global = AirGlobal::new(ValueId::new(50), ObjId::new(50), "g_config");

        AirModule {
            id: ModuleId::derive(b"test"),
            name: Some("test".to_string()),
            functions: vec![func1, func2],
            globals: vec![global],
            source_files: Vec::new(),
            type_hierarchy: Vec::new(),
            constants: std::collections::BTreeMap::new(),
            types: std::collections::BTreeMap::new(),
            target_pointer_width: 8,
            function_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        }
    }

    #[test]
    fn pattern_exact_match() {
        assert!(matches_pattern("main", "main"));
        assert!(!matches_pattern("main", "Main"));
        assert!(!matches_pattern("main", "main2"));
    }

    #[test]
    fn pattern_star_any() {
        assert!(matches_pattern("main", "*"));
        assert!(matches_pattern("", "*"));
        assert!(matches_pattern("anything", "*"));
    }

    #[test]
    fn pattern_star_prefix() {
        assert!(matches_pattern("get_value", "get_*"));
        assert!(matches_pattern("get_", "get_*"));
        assert!(!matches_pattern("set_value", "get_*"));
    }

    #[test]
    fn pattern_star_suffix() {
        assert!(matches_pattern("my_func", "*_func"));
        assert!(matches_pattern("_func", "*_func"));
        assert!(!matches_pattern("my_function", "*_func"));
    }

    #[test]
    fn pattern_star_middle() {
        assert!(matches_pattern("get_user_info", "get_*_info"));
        assert!(matches_pattern("get__info", "get_*_info"));
        assert!(!matches_pattern("get_user_data", "get_*_info"));
    }

    #[test]
    fn pattern_question_mark() {
        assert!(matches_pattern("a", "?"));
        assert!(matches_pattern("ab", "??"));
        assert!(!matches_pattern("", "?"));
        assert!(!matches_pattern("abc", "??"));
    }

    #[test]
    fn resolve_value_id() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::value_id("0x0000000000000000000000000000000a"))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(10)));
    }

    #[test]
    fn resolve_value_id_no_prefix() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::value_id("0000000000000000000000000000000a"))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(10)));
    }

    #[test]
    fn resolve_value_id_invalid() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver.resolve(&Selector::value_id("not_hex"));
        assert!(matches!(result, Err(ResolveError::InvalidHexId(_))));
    }

    #[test]
    fn resolve_function_param_specific() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::function_param("main", Some(0)))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(10)));
    }

    #[test]
    fn resolve_function_param_all() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::function_param("main", None))
            .unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&ValueId::new(10)));
        assert!(result.contains(&ValueId::new(11)));
    }

    #[test]
    fn resolve_function_param_wildcard() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::function_param("*", Some(0)))
            .unwrap();
        assert_eq!(result.len(), 2); // main param 0 and helper param 0
        assert!(result.contains(&ValueId::new(10)));
        assert!(result.contains(&ValueId::new(30)));
    }

    #[test]
    fn resolve_function_return() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::function_return("main"))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(21)));
    }

    #[test]
    fn resolve_function_all() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver.resolve(&Selector::function_all("helper")).unwrap();
        // helper has 1 param (30) and no instruction results (ret has no dst)
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(30)));
    }

    #[test]
    fn resolve_global() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver.resolve(&Selector::global("g_*")).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(50)));
    }

    #[test]
    fn resolve_call_to() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver.resolve(&Selector::call_to("helper")).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(21)));
    }

    #[test]
    fn resolve_arg_to() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver
            .resolve(&Selector::arg_to("helper", Some(0)))
            .unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(10)));
    }

    #[test]
    fn resolve_arg_to_all() {
        let module = make_test_module();
        let resolver = SelectorResolver::new(&module);

        let result = resolver.resolve(&Selector::arg_to("helper", None)).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains(&ValueId::new(10)));
    }
}
