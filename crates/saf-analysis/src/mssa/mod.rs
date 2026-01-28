//! Memory SSA — def-use chains for address-taken variables.
//!
//! Provides precise store→load def-use chains for heap/stack variables accessed
//! through pointers. Uses a hybrid approach:
//!
//! 1. **Skeleton** — Single def chain per function (LLVM-style compact).
//!    Every store/call gets a Def, every load gets a Use, join points get Phi.
//!
//! 2. **Demand-driven clobber walker** — When queried for a specific location,
//!    walks the def chain backward consulting PTA to disambiguate.
//!
//! 3. **Interprocedural mod/ref summaries** — Per-function may-mod/may-ref sets
//!    computed bottom-up on the call graph.
//!
//! See Plan 025 for full design documentation.

mod access;
mod builder;
mod export;
mod modref;
mod walker;

pub use access::{MemAccessId, MemoryAccess};
pub use export::MemorySsaExport;
pub use modref::ModRefSummary;

use std::collections::BTreeMap;

use saf_core::ids::{BlockId, FunctionId, InstId, LocId, ValueId};

use crate::PtaResult;
use crate::callgraph::CallGraph;
use crate::cfg::Cfg;

/// Metadata about a Def instruction, used by the clobber walker.
///
/// Records what kind of memory effect the instruction has, so the walker
/// can determine if it clobbers a specific location.
#[derive(Debug, Clone)]
pub(crate) enum InstInfo {
    /// Store instruction: operands[1] is the pointer.
    Store { ptr: ValueId },
    /// Call instruction: callee function (None for indirect calls).
    Call { callee: Option<FunctionId> },
    /// Memcpy: operands[0] is the destination pointer.
    Memcpy { dst_ptr: ValueId },
    /// Memset: operands[0] is the destination pointer.
    Memset { dst_ptr: ValueId },
}

/// Complete Memory SSA for a module.
///
/// Contains the skeleton (Def/Use/Phi/`LiveOnEntry` chain), mod/ref summaries,
/// and a demand-driven clobber cache.
pub struct MemorySsa {
    /// All memory accesses indexed by ID.
    accesses: BTreeMap<MemAccessId, MemoryAccess>,
    /// Instruction → memory access mapping.
    inst_to_access: BTreeMap<InstId, MemAccessId>,
    /// Block → phi accesses at block entry.
    block_phis: BTreeMap<BlockId, Vec<MemAccessId>>,
    /// Function → `LiveOnEntry` sentinel.
    live_on_entry: BTreeMap<FunctionId, MemAccessId>,
    /// Function → mod/ref summary.
    mod_ref_summaries: BTreeMap<FunctionId, ModRefSummary>,
    /// Clobber cache: (use_access, location) → clobbering def.
    clobber_cache: BTreeMap<(MemAccessId, LocId), MemAccessId>,
    /// Instruction metadata for clobber walker.
    pub(crate) inst_info: BTreeMap<InstId, InstInfo>,
    /// Reference to PTA result (needed for clobber queries).
    pta: PtaResult,
}

impl MemorySsa {
    /// Build Memory SSA for a module.
    ///
    /// Constructs the skeleton (Def/Use/LiveOnEntry), places Phi nodes at
    /// dominance frontiers, and computes interprocedural mod/ref summaries.
    ///
    /// The clobber walker is demand-driven: it only disambiguates store→load
    /// chains when `clobber_for()` is called.
    #[must_use]
    pub fn build(
        module: &saf_core::air::AirModule,
        cfgs: &BTreeMap<FunctionId, Cfg>,
        pta: PtaResult,
        callgraph: &CallGraph,
    ) -> Self {
        // Phase 5: Compute mod/ref summaries bottom-up on call graph
        let mod_ref_summaries = modref::compute_mod_ref(module, &pta, callgraph);

        // Phases 2-3: Build skeleton and place phis
        let (accesses, inst_to_access, block_phis, live_on_entry, inst_info) =
            builder::build_skeleton(module, cfgs);

        Self {
            accesses,
            inst_to_access,
            block_phis,
            live_on_entry,
            mod_ref_summaries,
            clobber_cache: BTreeMap::new(),
            inst_info,
            pta,
        }
    }

    /// Find the clobbering def for a Use at a specific location (demand-driven).
    ///
    /// Walks the def chain backward from the Use's reaching def, consulting PTA
    /// to determine if each Def actually clobbers the queried location. Results
    /// are cached for subsequent queries.
    ///
    /// Returns the `MemAccessId` of the clobbering def (which may be a Def,
    /// Phi, or LiveOnEntry).
    pub fn clobber_for(&mut self, use_id: MemAccessId, loc: LocId) -> MemAccessId {
        walker::clobber_for(self, use_id, loc)
    }

    /// Get the memory access for an instruction.
    #[must_use]
    pub fn access_for(&self, inst: InstId) -> Option<&MemoryAccess> {
        self.inst_to_access
            .get(&inst)
            .and_then(|id| self.accesses.get(id))
    }

    /// Get the memory access ID for an instruction.
    #[must_use]
    pub fn access_id_for(&self, inst: InstId) -> Option<MemAccessId> {
        self.inst_to_access.get(&inst).copied()
    }

    /// Get a memory access by ID.
    #[must_use]
    pub fn access(&self, id: MemAccessId) -> Option<&MemoryAccess> {
        self.accesses.get(&id)
    }

    /// Get all Phi accesses at a block's entry.
    #[must_use]
    pub fn phis_at(&self, block: BlockId) -> &[MemAccessId] {
        self.block_phis.get(&block).map_or(&[], |v| v.as_slice())
    }

    /// Get the LiveOnEntry sentinel for a function.
    #[must_use]
    pub fn live_on_entry(&self, func: FunctionId) -> Option<MemAccessId> {
        self.live_on_entry.get(&func).copied()
    }

    /// Get mod/ref summary for a function.
    #[must_use]
    pub fn mod_ref(&self, func: FunctionId) -> Option<&ModRefSummary> {
        self.mod_ref_summaries.get(&func)
    }

    /// Get all memory accesses.
    #[must_use]
    pub fn accesses(&self) -> &BTreeMap<MemAccessId, MemoryAccess> {
        &self.accesses
    }

    /// Get all mod/ref summaries.
    #[must_use]
    pub fn mod_ref_summaries(&self) -> &BTreeMap<FunctionId, ModRefSummary> {
        &self.mod_ref_summaries
    }

    /// Get the PTA result used for disambiguation.
    #[must_use]
    pub fn pta(&self) -> &PtaResult {
        &self.pta
    }

    /// Total number of memory accesses.
    #[must_use]
    pub fn access_count(&self) -> usize {
        self.accesses.len()
    }

    /// Export to JSON-serializable format.
    #[must_use]
    pub fn export(&self) -> MemorySsaExport {
        export::export(self)
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        export::to_property_graph(self, resolver)
    }
}
