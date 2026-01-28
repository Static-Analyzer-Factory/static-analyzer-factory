//! Combined PTA + Abstract Interpretation analysis.
//!
//! Orchestrates pointer analysis and abstract interpretation with:
//! - Alias-aware memory tracking
//! - Indirect call resolution
//! - Bidirectional refinement (optional)

use std::collections::BTreeMap;
use std::sync::Arc;

use saf_core::air::AirModule;
use saf_core::ids::FunctionId;

use crate::absint::{
    AbstractInterpConfig, AbstractInterpResult, FunctionSummary, solve_interprocedural_with_pta,
};
use crate::pta::{PtaConfig, PtaContext, PtaResult};

/// Configuration for combined PTA + abstract interpretation.
#[derive(Debug, Clone)]
pub struct CombinedAnalysisConfig {
    /// PTA configuration.
    pub pta: PtaConfig,
    /// Abstract interpretation configuration.
    pub absint: AbstractInterpConfig,
    /// Enable bidirectional refinement loop.
    pub enable_refinement: bool,
    /// Maximum refinement iterations (0 = single pass).
    pub max_refinement_iterations: usize,
    /// Use context-sensitive PTA for indirect calls.
    pub context_sensitive_indirect: bool,
}

impl Default for CombinedAnalysisConfig {
    fn default() -> Self {
        Self {
            pta: PtaConfig::default(),
            absint: AbstractInterpConfig::default(),
            enable_refinement: true,
            max_refinement_iterations: 3,
            context_sensitive_indirect: false,
        }
    }
}

/// Combined analysis result.
pub struct CombinedAnalysisResult {
    /// Points-to analysis results.
    pub pta: PtaResult,
    /// Abstract interpretation results (with PTA-aware memory model).
    pub absint: AbstractInterpResult,
    /// Function summaries (return intervals + mod/ref).
    pub summaries: BTreeMap<FunctionId, FunctionSummary>,
    /// Number of refinement iterations performed.
    pub refinement_iterations: usize,
}

impl std::fmt::Debug for CombinedAnalysisResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombinedAnalysisResult")
            .field("pta_value_count", &self.pta.value_count())
            .field("absint_block_count", &self.absint.block_count())
            .field("summaries_count", &self.summaries.len())
            .field("refinement_iterations", &self.refinement_iterations)
            .finish()
    }
}

impl CombinedAnalysisResult {
    /// Get the PTA result.
    #[must_use]
    pub fn pta(&self) -> &PtaResult {
        &self.pta
    }

    /// Get the abstract interpretation result.
    #[must_use]
    pub fn absint(&self) -> &AbstractInterpResult {
        &self.absint
    }

    /// Get function summary by ID.
    #[must_use]
    pub fn function_summary(&self, func_id: &FunctionId) -> Option<&FunctionSummary> {
        self.summaries.get(func_id)
    }
}

/// Run combined PTA + abstract interpretation analysis.
///
/// Phases:
/// 1. Initial PTA
/// 2. Interprocedural abstract interpretation with PTA
/// 3. (Optional) Bidirectional refinement loop
#[must_use]
pub fn analyze_combined(
    module: &AirModule,
    config: &CombinedAnalysisConfig,
) -> CombinedAnalysisResult {
    // Phase 1: Initial PTA
    let mut pta_ctx = PtaContext::new(config.pta.clone());
    let pta_raw = pta_ctx.analyze(module);
    let pta = PtaResult::new(pta_raw.pts, Arc::new(pta_raw.factory), pta_raw.diagnostics);

    // Phase 2: Interprocedural absint with PTA
    let interprocedural = solve_interprocedural_with_pta(module, &config.absint, &pta);

    // Extract summaries from interprocedural result
    let mut summaries = BTreeMap::new();
    for func in &module.functions {
        if let Some(summary) = interprocedural.function_summary(&func.id) {
            summaries.insert(func.id, summary.clone());
        }
    }

    // Phase 3: Refinement loop (placeholder for future index refinement)
    let refinement_iterations = 0;

    CombinedAnalysisResult {
        pta,
        absint: interprocedural.intraprocedural().clone(),
        summaries,
        refinement_iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_config_default() {
        let config = CombinedAnalysisConfig::default();
        assert!(config.enable_refinement);
        assert_eq!(config.max_refinement_iterations, 3);
    }

    #[test]
    fn combined_result_accessors() {
        use saf_core::ids::ModuleId;

        let module = AirModule::new(ModuleId::derive(b"test"));

        let config = CombinedAnalysisConfig::default();
        let result = analyze_combined(&module, &config);

        assert_eq!(result.refinement_iterations, 0);
        assert!(result.summaries.is_empty());
    }
}
