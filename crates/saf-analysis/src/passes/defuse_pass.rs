use crate::defuse::DefUseGraph;
use crate::error::AnalysisError;
use crate::pass::{AnalysisContext, AnalysisPass, PassId};
use saf_core::air::AirModule;

/// Pass that builds a [`DefUseGraph`] from the module.
pub struct DefUsePass;

impl AnalysisPass for DefUsePass {
    fn id(&self) -> PassId {
        "defuse"
    }

    fn run(&self, module: &AirModule, ctx: &mut AnalysisContext) -> Result<(), AnalysisError> {
        let graph = DefUseGraph::build(module);
        ctx.insert(self.id(), graph);
        Ok(())
    }
}
