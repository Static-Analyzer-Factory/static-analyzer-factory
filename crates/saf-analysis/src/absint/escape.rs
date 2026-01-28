//! Escape analysis for pointer values.
//!
//! Determines whether pointers escape their creating scope:
//! - **NoEscape**: Pointer stays local to the function
//! - **ReturnEscape**: Pointer may be returned to caller
//! - **GlobalEscape**: Pointer may be stored to global memory or passed to escaping functions
//!
//! This information is used for:
//! - Stack allocation promotion (allocas that don't escape can be optimized)
//! - Precise mod/ref analysis (non-escaping pointers have bounded effects)
//! - Thread-safety analysis (non-escaping pointers can't be accessed by other threads)

use std::collections::{BTreeMap, BTreeSet};

use saf_core::air::{AirFunction, AirModule, Operation};
use saf_core::ids::{FunctionId, ValueId};
use saf_core::spec::SpecRegistry;

/// Escape state for a pointer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EscapeState {
    /// Pointer does not escape the function.
    NoEscape,
    /// Pointer may be returned to caller.
    ReturnEscape,
    /// Pointer may escape to global memory or other threads.
    GlobalEscape,
}

impl EscapeState {
    /// Join two escape states (lattice meet for must-analysis, join for may-analysis).
    ///
    /// Uses the order: `NoEscape` < `ReturnEscape` < `GlobalEscape`
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::GlobalEscape, _) | (_, Self::GlobalEscape) => Self::GlobalEscape,
            (Self::ReturnEscape, _) | (_, Self::ReturnEscape) => Self::ReturnEscape,
            (Self::NoEscape, Self::NoEscape) => Self::NoEscape,
        }
    }

    /// Check if the pointer escapes (returns or global).
    #[must_use]
    pub fn escapes(self) -> bool {
        !matches!(self, Self::NoEscape)
    }

    /// Check if the pointer escapes globally.
    #[must_use]
    pub fn escapes_globally(self) -> bool {
        matches!(self, Self::GlobalEscape)
    }
}

impl Default for EscapeState {
    fn default() -> Self {
        Self::NoEscape
    }
}

/// Result of escape analysis for a function.
#[derive(Debug, Clone)]
pub struct EscapeResult {
    /// Escape state for each pointer value.
    states: BTreeMap<ValueId, EscapeState>,
    /// Values that escape via return.
    return_escapes: BTreeSet<ValueId>,
    /// Values that escape globally.
    global_escapes: BTreeSet<ValueId>,
    /// Diagnostics.
    pub diagnostics: EscapeDiagnostics,
}

impl EscapeResult {
    /// Get the escape state for a value.
    #[must_use]
    pub fn state(&self, value: ValueId) -> EscapeState {
        self.states
            .get(&value)
            .copied()
            .unwrap_or(EscapeState::NoEscape)
    }

    /// Check if a value escapes.
    #[must_use]
    pub fn escapes(&self, value: ValueId) -> bool {
        self.state(value).escapes()
    }

    /// Check if a value escapes globally.
    #[must_use]
    pub fn escapes_globally(&self, value: ValueId) -> bool {
        self.state(value).escapes_globally()
    }

    /// Get all values that escape via return.
    #[must_use]
    pub fn return_escapes(&self) -> &BTreeSet<ValueId> {
        &self.return_escapes
    }

    /// Get all values that escape globally.
    #[must_use]
    pub fn global_escapes(&self) -> &BTreeSet<ValueId> {
        &self.global_escapes
    }

    /// Get all escape states.
    #[must_use]
    pub fn states(&self) -> &BTreeMap<ValueId, EscapeState> {
        &self.states
    }
}

/// Diagnostics from escape analysis.
#[derive(Debug, Clone, Default)]
pub struct EscapeDiagnostics {
    /// Number of values analyzed.
    pub values_analyzed: u64,
    /// Number of values that escape via return.
    pub return_escape_count: u64,
    /// Number of values that escape globally.
    pub global_escape_count: u64,
    /// Number of iterations to reach fixpoint.
    pub iterations: u32,
}

/// Configuration for escape analysis.
#[derive(Debug, Clone)]
pub struct EscapeConfig {
    /// Maximum iterations before giving up.
    pub max_iterations: u32,
}

impl Default for EscapeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
        }
    }
}

/// Analyze escape for a single function.
///
/// Uses a forward dataflow analysis to track which pointers escape:
/// - Alloca/HeapAlloc: initially NoEscape
/// - Store to global: GlobalEscape
/// - Return: ReturnEscape
/// - Call with escaping param: depends on spec
/// - Copy/GEP: inherits from source
#[must_use]
pub fn analyze_escape(
    func: &AirFunction,
    module: &AirModule,
    config: &EscapeConfig,
) -> EscapeResult {
    analyze_escape_with_specs(func, module, config, None)
}

/// Analyze escape with function specifications.
///
/// When specs are available, uses the `escapes` field on parameters
/// to determine if passing a pointer to a function causes escape.
// NOTE: This function implements the escape analysis fixpoint loop as a
// single cohesive unit. Splitting would obscure the analysis structure.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn analyze_escape_with_specs(
    func: &AirFunction,
    module: &AirModule,
    config: &EscapeConfig,
    specs: Option<&SpecRegistry>,
) -> EscapeResult {
    let mut states: BTreeMap<ValueId, EscapeState> = BTreeMap::new();
    let mut diagnostics = EscapeDiagnostics::default();

    // Build function name map for spec lookup
    let func_names: BTreeMap<FunctionId, &str> = module
        .functions
        .iter()
        .map(|f| (f.id, f.name.as_str()))
        .collect();

    // Initialize: all allocations start as NoEscape
    for block in &func.blocks {
        for inst in &block.instructions {
            if let Some(dst) = inst.dst {
                match &inst.op {
                    Operation::Alloca { .. } | Operation::HeapAlloc { .. } => {
                        states.insert(dst, EscapeState::NoEscape);
                        diagnostics.values_analyzed += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // Fixpoint iteration
    let mut changed = true;
    let mut iteration = 0;

    while changed && iteration < config.max_iterations {
        changed = false;
        iteration += 1;

        for block in &func.blocks {
            for inst in &block.instructions {
                match &inst.op {
                    // Return: value escapes to caller
                    Operation::Ret => {
                        if let Some(&ret_val) = inst.operands.first() {
                            if let Some(&current) = states.get(&ret_val) {
                                let new_state = current.join(EscapeState::ReturnEscape);
                                if new_state != current {
                                    states.insert(ret_val, new_state);
                                    changed = true;
                                }
                            }
                        }
                    }

                    // Store to unknown/global: value escapes globally
                    Operation::Store => {
                        if inst.operands.len() >= 2 {
                            let value = inst.operands[0];
                            let ptr = inst.operands[1];

                            // If storing to a global or unknown location, value escapes
                            // For now, conservatively assume any store to non-alloca escapes
                            if !states.contains_key(&ptr) {
                                if let Some(&current) = states.get(&value) {
                                    let new_state = current.join(EscapeState::GlobalEscape);
                                    if new_state != current {
                                        states.insert(value, new_state);
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }

                    // Call: check spec for escaping params
                    Operation::CallDirect { callee } => {
                        let escaping_params = get_escaping_params(*callee, &func_names, specs);

                        for (i, &arg) in inst.operands.iter().enumerate() {
                            // Check if this param is marked as escaping
                            // INVARIANT: LLVM limits function parameters to < 2^32
                            #[allow(clippy::cast_possible_truncation)]
                            let param_idx = i as u32;
                            let arg_escapes = escaping_params
                                .as_ref()
                                .is_none_or(|params| params.contains(&param_idx));

                            if arg_escapes {
                                if let Some(&current) = states.get(&arg) {
                                    let new_state = current.join(EscapeState::GlobalEscape);
                                    if new_state != current {
                                        states.insert(arg, new_state);
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }

                    // Indirect call: conservative, all args may escape
                    Operation::CallIndirect { .. } => {
                        for &arg in &inst.operands {
                            if let Some(&current) = states.get(&arg) {
                                let new_state = current.join(EscapeState::GlobalEscape);
                                if new_state != current {
                                    states.insert(arg, new_state);
                                    changed = true;
                                }
                            }
                        }
                    }

                    // Copy/GEP: result inherits escape state from source
                    Operation::Copy | Operation::Gep { .. } => {
                        if let Some(dst) = inst.dst {
                            if let Some(&src) = inst.operands.first() {
                                if let Some(&src_state) = states.get(&src) {
                                    let current =
                                        states.get(&dst).copied().unwrap_or(EscapeState::NoEscape);
                                    let new_state = current.join(src_state);
                                    if new_state != current {
                                        states.insert(dst, new_state);
                                        changed = true;
                                    }
                                }
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    diagnostics.iterations = iteration;

    // Collect return and global escapes
    let mut return_escapes = BTreeSet::new();
    let mut global_escapes = BTreeSet::new();

    for (&value, &state) in &states {
        match state {
            EscapeState::ReturnEscape => {
                return_escapes.insert(value);
                diagnostics.return_escape_count += 1;
            }
            EscapeState::GlobalEscape => {
                global_escapes.insert(value);
                diagnostics.global_escape_count += 1;
            }
            EscapeState::NoEscape => {}
        }
    }

    EscapeResult {
        states,
        return_escapes,
        global_escapes,
        diagnostics,
    }
}

/// Get escaping parameter indices from spec.
///
/// Returns `None` if no spec exists (conservative: all params may escape).
/// Returns `Some(set)` with indices of params that have `escapes: true`.
fn get_escaping_params(
    callee: FunctionId,
    func_names: &BTreeMap<FunctionId, &str>,
    specs: Option<&SpecRegistry>,
) -> Option<BTreeSet<u32>> {
    let name = func_names.get(&callee)?;
    let registry = specs?;
    let spec = registry.lookup(name)?;

    let mut escaping = BTreeSet::new();
    for param in &spec.params {
        if param.escapes == Some(true) {
            escaping.insert(param.index);
        }
    }

    Some(escaping)
}

/// Analyze escape for all functions in a module.
#[must_use]
pub fn analyze_module_escape(
    module: &AirModule,
    config: &EscapeConfig,
    specs: Option<&SpecRegistry>,
) -> BTreeMap<FunctionId, EscapeResult> {
    let mut results = BTreeMap::new();

    for func in &module.functions {
        if func.is_declaration || func.blocks.is_empty() {
            continue;
        }

        let result = analyze_escape_with_specs(func, module, config, specs);
        results.insert(func.id, result);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, Instruction};
    use saf_core::ids::{BlockId, InstId, ModuleId};

    fn vid(n: u128) -> ValueId {
        ValueId::new(n)
    }

    fn iid(n: u128) -> InstId {
        InstId::new(n)
    }

    fn bid(n: u128) -> BlockId {
        BlockId::new(n)
    }

    fn fid(n: u128) -> FunctionId {
        FunctionId::new(n)
    }

    #[test]
    fn escape_state_join() {
        use EscapeState::*;

        assert_eq!(NoEscape.join(NoEscape), NoEscape);
        assert_eq!(NoEscape.join(ReturnEscape), ReturnEscape);
        assert_eq!(NoEscape.join(GlobalEscape), GlobalEscape);
        assert_eq!(ReturnEscape.join(ReturnEscape), ReturnEscape);
        assert_eq!(ReturnEscape.join(GlobalEscape), GlobalEscape);
        assert_eq!(GlobalEscape.join(GlobalEscape), GlobalEscape);
    }

    #[test]
    fn alloca_no_escape() {
        // Simple alloca that doesn't escape
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(
            Instruction::new(iid(1), Operation::Alloca { size_bytes: None }).with_dst(vid(1)),
        );
        block
            .instructions
            .push(Instruction::new(iid(2), Operation::Ret));

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: Some(bid(1)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let module = AirModule::new(ModuleId::derive(b"test"));
        let config = EscapeConfig::default();

        let result = analyze_escape(&func, &module, &config);

        assert_eq!(result.state(vid(1)), EscapeState::NoEscape);
        assert!(!result.escapes(vid(1)));
    }

    #[test]
    fn alloca_return_escape() {
        // Alloca that is returned
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(
            Instruction::new(iid(1), Operation::Alloca { size_bytes: None }).with_dst(vid(1)),
        );
        block
            .instructions
            .push(Instruction::new(iid(2), Operation::Ret).with_operands(vec![vid(1)]));

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: Some(bid(1)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let module = AirModule::new(ModuleId::derive(b"test"));
        let config = EscapeConfig::default();

        let result = analyze_escape(&func, &module, &config);

        assert_eq!(result.state(vid(1)), EscapeState::ReturnEscape);
        assert!(result.escapes(vid(1)));
        assert!(!result.escapes_globally(vid(1)));
    }

    #[test]
    fn escape_diagnostics() {
        let mut block = AirBlock::new(bid(1));
        block.instructions.push(
            Instruction::new(iid(1), Operation::Alloca { size_bytes: None }).with_dst(vid(1)),
        );
        block
            .instructions
            .push(Instruction::new(iid(2), Operation::Ret).with_operands(vec![vid(1)]));

        let func = AirFunction {
            id: fid(1),
            name: "test".to_string(),
            params: Vec::new(),
            blocks: vec![block],
            entry_block: Some(bid(1)),
            is_declaration: false,
            span: None,
            symbol: None,
            block_index: BTreeMap::new(),
        };

        let module = AirModule::new(ModuleId::derive(b"test"));
        let config = EscapeConfig::default();

        let result = analyze_escape(&func, &module, &config);

        assert_eq!(result.diagnostics.values_analyzed, 1);
        assert_eq!(result.diagnostics.return_escape_count, 1);
        assert_eq!(result.diagnostics.global_escape_count, 0);
    }
}
