pub mod air;
pub mod air_visitor;
pub mod cache;
pub mod config;
pub mod error;
pub mod id;
pub mod ids;
pub mod layout;
pub mod logging;
pub mod manifest;
pub mod program;
pub mod span;
pub mod spec;
pub mod summary;
pub mod summary_registry;

pub use error::CoreError;

// SAF log module/phase registry — must be at crate root so that
// $crate::__saf_log_registry resolves correctly in cross-crate saf_log! usage.
saf_log_module! {
    pta { constraint, solve, hvn, scc, lcd },
    callgraph { build, refine },
    cfg { build },
    svfg { build, optimize },
    valueflow { build, query },
    defuse { build },
    mssa { build },
    checker { memleak, uaf, nullptr, pathsens },
    absint { interproc, transfer, escape, nullness },
    frontend { ingest },
    pipeline { constraint, incremental, analysis },
}
