use crate::cg_refinement::RefinementResult;
use crate::defuse::DefUseGraph;
use crate::error::AnalysisError;
use crate::pass::{AnalysisContext, AnalysisPass, PassId};
use crate::{ValueFlowConfig, build_valueflow};
use saf_core::air::AirModule;

/// Pass that builds the [`ValueFlowGraph`] using def-use and PTA results.
///
/// Depends on the `"defuse"` and `"pta"` passes having already run.
pub struct ValueFlowPass {
    /// Value-flow analysis configuration.
    pub config: ValueFlowConfig,
}

impl AnalysisPass for ValueFlowPass {
    fn id(&self) -> PassId {
        "valueflow"
    }

    fn dependencies(&self) -> &[PassId] {
        &["defuse", "pta"]
    }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let defuse = ctx
            .get::<DefUseGraph>("defuse")
            .ok_or_else(|| AnalysisError::Config("defuse pass has not run".into()))?;
        let refinement = ctx
            .get::<RefinementResult>("pta")
            .ok_or_else(|| AnalysisError::Config("pta pass has not run".into()))?;

        let vfg = build_valueflow(
            &self.config,
            module,
            defuse,
            &refinement.call_graph,
            refinement.pta_result.as_ref(),
        );
        ctx.insert(self.id(), vfg);
        Ok(())
    }
}
