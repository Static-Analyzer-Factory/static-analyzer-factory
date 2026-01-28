//! Program database — graph-first analysis framework.
//!
//! `ProgramDatabase` owns all precomputed program analysis graphs and provides
//! query primitives for LLM agents and programmatic consumers.

pub mod catalog;
pub mod config;
pub mod handler;
pub mod protocol;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, InstId, LocId, ValueId};

use crate::AliasResult;
use crate::PtaResult;
use crate::build_valueflow;
use crate::callgraph::{CallGraph, CallGraphNode};
use crate::cfg::Cfg;
use crate::defuse::DefUseGraph;
use crate::export::PropertyGraph;
use crate::icfg::Icfg;
use crate::mssa::MemorySsa;
use crate::pipeline::{PipelineConfig, PipelineStats, run_pipeline};
use crate::svfg::{Svfg, SvfgBuilder};
use crate::valueflow::{ValueFlowConfig, ValueFlowGraph};

use crate::checkers::ResourceTable;
use catalog::{CatalogEntry, CheckCatalog};

/// Discovery schema returned by `ProgramDatabase::schema()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Available named checks.
    pub checks: Vec<CatalogEntry>,
    /// Available graph types.
    pub graphs: Vec<String>,
    /// Available query primitives.
    pub queries: Vec<String>,
}

/// A program database containing all precomputed analysis graphs.
///
/// Two-phase usage:
/// 1. **Build:** `ProgramDatabase::build(module, config)` constructs core graphs
/// 2. **Query:** Use graph accessors and query primitives to analyze the program
pub struct ProgramDatabase {
    module: Arc<AirModule>,
    call_graph: CallGraph,
    icfg: OnceLock<Icfg>,
    pta_result: Option<PtaResult>,
    defuse: DefUseGraph,
    valueflow: OnceLock<ValueFlowGraph>,
    valueflow_config: ValueFlowConfig,
    resolved_sites: BTreeMap<InstId, Vec<FunctionId>>,
    stats: PipelineStats,
    svfg: OnceLock<Svfg>,
    /// Custom resource table for checkers (overrides default when set).
    resource_table: Option<ResourceTable>,
}

impl ProgramDatabase {
    /// Build a `ProgramDatabase` from an `AirModule`.
    ///
    /// Runs the full analysis pipeline (CG refinement, PTA, value flow)
    /// and stores all resulting graphs for subsequent queries.
    #[must_use]
    pub fn build(module: AirModule, config: &PipelineConfig) -> Self {
        let resource_table = config.specs.as_ref().map(ResourceTable::from_specs);
        let pipeline = run_pipeline(&module, config);
        Self {
            module: Arc::new(module),
            call_graph: pipeline.call_graph,
            icfg: OnceLock::new(),
            pta_result: pipeline.pta_result,
            defuse: pipeline.defuse,
            valueflow: OnceLock::new(),
            valueflow_config: config.valueflow.clone(),
            resolved_sites: pipeline.resolved_sites,
            stats: pipeline.stats,
            svfg: OnceLock::new(),
            resource_table,
        }
    }

    /// Build a `ProgramDatabase` from pre-built components.
    ///
    /// Used by Python bindings where the pipeline has already run.
    /// Pre-built VFG and ICFG are stored directly (no lazy rebuild needed).
    #[must_use]
    pub fn from_parts(
        module: Arc<AirModule>,
        call_graph: CallGraph,
        icfg: Icfg,
        pta_result: Option<PtaResult>,
        defuse: DefUseGraph,
        valueflow: ValueFlowGraph,
        stats: PipelineStats,
    ) -> Self {
        let icfg_lock = OnceLock::new();
        let _ = icfg_lock.set(icfg);
        let vfg_lock = OnceLock::new();
        let _ = vfg_lock.set(valueflow);
        Self {
            module,
            call_graph,
            icfg: icfg_lock,
            pta_result,
            defuse,
            valueflow: vfg_lock,
            valueflow_config: ValueFlowConfig::default(),
            resolved_sites: BTreeMap::new(),
            stats,
            svfg: OnceLock::new(),
            resource_table: None,
        }
    }

    /// Set a custom `ResourceTable` for SVFG checkers.
    ///
    /// When set, checkers use this table instead of the default
    /// `ResourceTable::new()`. This allows bench mode to register
    /// test-specific allocator/deallocator wrapper functions.
    pub fn set_resource_table(&mut self, table: ResourceTable) {
        self.resource_table = Some(table);
    }

    /// The analyzed `AirModule`.
    pub fn module(&self) -> &AirModule {
        &self.module
    }

    /// The refined call graph.
    pub fn call_graph(&self) -> &CallGraph {
        &self.call_graph
    }

    /// The ICFG (interprocedural control flow graph).
    ///
    /// Built lazily on first access from the module and call graph.
    pub fn icfg(&self) -> &Icfg {
        self.icfg
            .get_or_init(|| Icfg::build(&self.module, &self.call_graph))
    }

    /// The PTA result (if PTA ran).
    pub fn pta_result(&self) -> Option<&PtaResult> {
        self.pta_result.as_ref()
    }

    /// The def-use graph.
    pub fn defuse(&self) -> &DefUseGraph {
        &self.defuse
    }

    /// The value-flow graph.
    ///
    /// Built lazily on first access using the stored `ValueFlowConfig`.
    /// When defuse was skipped during pipeline (empty), builds it on-the-fly.
    pub fn valueflow(&self) -> &ValueFlowGraph {
        self.valueflow.get_or_init(|| {
            // When the pipeline skipped defuse build (build_valueflow=false),
            // self.defuse is empty. Build it on demand for the VFG.
            let temp_defuse;
            let defuse = if self.defuse.is_empty() {
                temp_defuse = DefUseGraph::build(&self.module);
                &temp_defuse
            } else {
                &self.defuse
            };
            build_valueflow(
                &self.valueflow_config,
                &self.module,
                defuse,
                &self.call_graph,
                self.pta_result.as_ref(),
            )
        })
    }

    /// Resolved indirect call sites from CG refinement.
    pub fn resolved_sites(&self) -> &BTreeMap<InstId, Vec<FunctionId>> {
        &self.resolved_sites
    }

    /// Pipeline execution statistics.
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }

    /// Export all graphs as `PropertyGraph` structs.
    ///
    /// Returns one `PropertyGraph` per graph type: callgraph, defuse,
    /// valueflow, and (if PTA ran) pta.
    pub fn export_graphs(&self) -> Vec<PropertyGraph> {
        let mut graphs = Vec::new();

        // Call graph
        graphs.push(self.call_graph.to_pg(&self.module, None));

        // Def-use
        graphs.push(self.defuse.to_pg(&self.module, None));

        // Value flow
        graphs.push(crate::to_property_graph(
            self.valueflow(),
            &self.module,
            None,
        ));

        // PTA (if available)
        if let Some(pta) = &self.pta_result {
            graphs.push(pta.to_pg(None));
        }

        graphs
    }

    /// Query whether two pointers may alias.
    ///
    /// Returns `AliasResult::Unknown` if PTA is not available.
    pub fn may_alias(&self, p: ValueId, q: ValueId) -> AliasResult {
        match &self.pta_result {
            Some(pta) => pta.may_alias(p, q),
            None => AliasResult::Unknown,
        }
    }

    /// Query what locations a pointer may point to.
    ///
    /// Returns an empty vec if PTA is not available.
    pub fn points_to(&self, pointer: ValueId) -> Vec<LocId> {
        match &self.pta_result {
            Some(pta_res) => pta_res.points_to(pointer),
            None => Vec::new(),
        }
    }

    /// Build a CFG for a specific function.
    ///
    /// Panics if the function ID is not found in the module.
    pub fn cfg(&self, func_id: FunctionId) -> Cfg {
        let func = self
            .module
            .functions
            .iter()
            .find(|f| f.id == func_id)
            .expect("function not found in module");
        Cfg::build(func)
    }

    /// Find all functions reachable from the given starting functions
    /// in the call graph (forward BFS).
    pub fn cg_reachable_from(&self, starts: &[FunctionId]) -> BTreeSet<FunctionId> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();

        for &fid in starts {
            let node = CallGraphNode::Function(fid);
            if self.call_graph.nodes.contains(&node) {
                visited.insert(fid);
                queue.push_back(node);
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(callees) = self.call_graph.edges.get(&current) {
                for callee in callees {
                    if let CallGraphNode::Function(callee_id) = callee {
                        if visited.insert(*callee_id) {
                            queue.push_back(callee.clone());
                        }
                    }
                }
            }
        }

        visited
    }

    /// Return the discovery schema for LLM agents.
    pub fn schema(&self) -> Schema {
        let catalog = CheckCatalog::new();
        Schema {
            checks: catalog.entries().values().cloned().collect(),
            graphs: vec![
                "cfg".to_string(),
                "callgraph".to_string(),
                "defuse".to_string(),
                "valueflow".to_string(),
                "pta".to_string(),
                "svfg".to_string(),
            ],
            queries: vec![
                "alias".to_string(),
                "points_to".to_string(),
                "reachable".to_string(),
                "taint_flow".to_string(),
                "flows".to_string(),
            ],
        }
    }

    /// Create a [`DisplayResolver`] wired to this database's module and
    /// analysis results.
    ///
    /// The resolver provides both Tier 1 (AIR structure) and Tier 2
    /// (analysis-derived) resolution:
    /// - Functions, blocks, instructions, values, globals, source files
    /// - PTA locations (`LocId`) with region and variable context
    /// - SVFG nodes with value or `MemPhi` descriptions
    ///
    /// [`DisplayResolver`]: crate::display::DisplayResolver
    #[must_use]
    pub fn display_resolver(&self) -> crate::display::DisplayResolver<'_> {
        crate::display::DisplayResolver::with_analysis(
            &self.module,
            self.pta_result.as_ref(),
            self.svfg.get(),
        )
    }

    /// Store a pre-built SVFG in the database.
    ///
    /// If the SVFG was already built (lazily or by a previous call), this is a
    /// no-op.  Callers that build an SVFG for their own purposes (e.g. the
    /// FS-PTA bench path) can donate it here so that subsequent consumers
    /// (checkers, queries) reuse it instead of rebuilding from scratch.
    pub fn set_svfg(&self, svfg: Svfg) {
        let _ = self.svfg.set(svfg);
    }

    /// Get or lazily build the SVFG.
    pub(crate) fn get_or_build_svfg(&self) -> &Svfg {
        self.svfg.get_or_init(|| {
            let cfgs: BTreeMap<FunctionId, Cfg> = self
                .module
                .functions
                .iter()
                .filter(|f| !f.is_declaration)
                .map(|f| (f.id, Cfg::build(f)))
                .collect();

            // MSSA needs an owned PtaResult; clone the one we have.
            // If PTA didn't run, build an empty SVFG.
            let Some(pta) = &self.pta_result else {
                return Svfg::new();
            };

            // When pipeline skipped defuse build (build_valueflow=false),
            // self.defuse is empty. Build it on demand for SVFG construction.
            let temp_defuse;
            let defuse = if self.defuse.is_empty() {
                temp_defuse = DefUseGraph::build(&self.module);
                &temp_defuse
            } else {
                &self.defuse
            };

            let mssa_pta = pta.clone();
            let mut mssa = MemorySsa::build(&self.module, &cfgs, mssa_pta, &self.call_graph);
            let (mut svfg, _program_points) =
                SvfgBuilder::new(&self.module, defuse, &self.call_graph, pta, &mut mssa).build();

            // Run SCCP and prune SVFG edges from dead PHI incoming blocks
            // to eliminate false positives from infeasible data-flow through PHIs.
            let sccp_result = crate::absint::sccp::run_sccp_module(&self.module);
            if !sccp_result.dead_blocks.is_empty() {
                crate::svfg::prune_dead_phi_edges(
                    &mut svfg,
                    &self.module,
                    &sccp_result.dead_blocks,
                );
            }

            svfg
        })
    }
}
