//! Context-sensitive pointer analysis (CS-PTA) using k-CFA.
//!
//! Implements a k-call-site-sensitive whole-program Andersen pointer analysis.
//! Each pointer value is qualified by a bounded [`CallSiteContext`] — a sequence
//! of up to `k` call-site instruction IDs representing the call chain that led
//! to the current function. This enables the analysis to distinguish calls to
//! the same function from different call sites, improving precision over the
//! context-insensitive (CI) PTA in [`crate::pta`].
//!
//! # Context-Sensitive vs Context-Insensitive
//!
//! CI-PTA merges all calling contexts: if function `f` is called from sites A
//! and B with different pointer arguments, CI-PTA unions both into a single
//! points-to set for `f`'s parameter. CS-PTA maintains separate sets per
//! context, so the return value at site A only reflects data from A's arguments.
//!
//! # Key Features
//!
//! - **Configurable context depth** (`k=1, 2, 3`): Higher `k` gives more
//!   precision but increases memory/time. `k=1` is the default and usually
//!   sufficient.
//! - **SCC collapse**: Functions in recursive strongly-connected components are
//!   collapsed to the empty context, preventing infinite context chains while
//!   maintaining soundness.
//! - **Heap cloning**: Callee-local allocations (e.g., `-O0` retval allocas)
//!   receive per-context cloned `LocId`s, so different calling contexts get
//!   separate abstract objects. Non-local locations (caller/global) use a
//!   global mirror for cross-context visibility.
//! - **CI summary**: A context-insensitive summary (union across all contexts)
//!   is automatically computed for backward compatibility with analyses that
//!   consume [`crate::pta::PtaResult`]-style queries.
//! - **Pluggable points-to set representations**: Like CI-PTA, the solver is
//!   generic over the [`crate::pta::ptsset::PtsSet`] trait (`BTreeSet`,
//!   `BitVector`, or `BDD`).
//!
//! # Solver Architecture
//!
//! ```text
//! 1. Extract intraprocedural constraints (reuses crate::pta::extract)
//! 2. Compute SCC functions on callgraph (Tarjan's algorithm)
//! 3. Build call-site maps (arg→param, ret→caller bindings)
//! 4. Seed Addr constraints in empty context
//! 5. Top-down context creation for all call sites
//! 6. Fixed-point worklist iteration:
//!    - Value worklist: Copy/Load/Store/Gep + interprocedural propagation
//!    - Location worklist (CS): per-context store content
//!    - Location worklist (global): cross-context visibility mirror
//! 7. Normalize to CsPtaResult with CI summary
//! ```
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::cspta::{solve_context_sensitive, CsPtaConfig};
//! let config = CsPtaConfig { k: 1, ..Default::default() };
//! let result = solve_context_sensitive(&module, &callgraph, &config);
//! let pts = result.points_to_any(ptr_value);
//! ```

pub mod context;
mod export;
mod solver;

pub use context::CallSiteContext;
pub use export::CsPtaExport;
pub use solver::{
    CsPtaConfig, CsPtaDiagnostics, CsPtaResult, CtxValue, solve_context_sensitive,
    solve_context_sensitive_with_resolved, solve_cs_generic, solve_cs_generic_with_resolved,
};
