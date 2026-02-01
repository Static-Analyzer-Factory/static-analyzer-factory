//! Datalog-based analysis engine for SAF.
//!
//! Uses Ascent (compile-time Datalog) for:
//! - Pointer analysis (Andersen's with lattice-based fixpoint)
//! - SVFG checker rules (reachability queries)
//!
//! Phase 2 (deferred): runtime Datalog interpreter for user-authored rules.

// Ascent proc macros generate underscore-prefixed variables internally.
#![allow(clippy::used_underscore_binding)]

pub mod checkers;
pub mod facts;
pub mod pta;
