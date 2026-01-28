//! Demand-Driven Pointer Analysis (DDA).
//!
//! Computes points-to information only for explicitly queried pointers,
//! using CFL-reachability for context-sensitive backward traversal on SVFG.
//!
//! DDA is more scalable than whole-program analysis for large codebases
//! where only a subset of pointers need to be analyzed (e.g., for specific
//! source/sink pairs in taint analysis).
//!
//! ## Key Features
//!
//! - **On-demand**: Only analyzes pointers that are queried
//! - **Context-sensitive**: Uses CFL-reachability for call/return matching
//! - **Strong updates**: Precise handling of singleton stores
//! - **Budget-bounded**: Falls back to CI-PTA when budget exhausted
//! - **Cached**: Persistent cache amortizes cost across queries
//!
//! ## Usage
//!
//! ```ignore
//! // Create DDA solver
//! let mut dda = DdaPta::new(&svfg, &mssa, &ci_pta, DdaConfig::default());
//!
//! // Query a pointer
//! let pts = dda.points_to(ptr_value_id);
//!
//! // Check alias
//! let alias = dda.may_alias(p, q);
//!
//! // Get diagnostics
//! let stats = dda.diagnostics();
//! ```
//!
//! See Plan 043 for full design documentation.

mod solver;
mod types;

pub use solver::DdaPta;
pub use types::{
    Budget, CacheStats, CallString, DdaCache, DdaConfig, DdaConfigExport, DdaDiagnostics,
    DdaExport, Dpm, ExhaustionReason, ReachabilityResult,
};
