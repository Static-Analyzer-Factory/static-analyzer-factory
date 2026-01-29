use crate::cg_refinement::RefinementConfig;
use crate::error::AnalysisError;
use crate::pass::{AnalysisContext, AnalysisPass, PassId};
use saf_core::air::AirModule;
use saf_core::spec::SpecRegistry;

/// Pass that runs CG refinement (CHA bootstrap + iterative PTA).
///
/// Stores the full [`RefinementResult`] which includes the call graph,
/// ICFG, PTA result, class hierarchy, and statistics.
pub struct PtaPass {
    /// CG refinement configuration.
    pub config: RefinementConfig,
    /// Optional function specifications for PTA constraint generation.
    pub specs: Option<SpecRegistry>,
}

impl AnalysisPass for PtaPass {
    fn id(&self) -> PassId {
        "pta"
    }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let result = crate::cg_refinement::refine(module, &self.config, self.specs.as_ref());
        ctx.insert(self.id(), result);
        Ok(())
    }
}
