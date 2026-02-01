//! Ascent-based pointer analysis solver.

pub mod context;
pub mod gep;
pub mod hvn;
pub mod pts_lattice;
pub mod registry;
pub mod scc;
pub mod solver;

pub use context::{analyze_with_ascent, refine_ascent, refine_with_ascent, solve_from_constraints};
pub use gep::{
    GepResolutionResult, resolve_gep_facts, resolve_gep_facts_with_pts, try_resolve_gep_facts,
};
pub use hvn::{DatalogHvnResult, preprocess_hvn};
pub use pts_lattice::AscentPtsSet;
pub use registry::{LocIdRegistry, current_registry, with_registry};
pub use scc::{SccResult, detect_scc, rewrite_facts_with_scc};
pub use solver::{PointsToMap, ascent_solve};
