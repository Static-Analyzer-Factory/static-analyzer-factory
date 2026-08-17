//! SV-COMP verification engine for SAF.
//!
//! This crate is the *pure* SV-COMP analysis engine: given an
//! [`saf_core::air::AirModule`] plus a [`Property`] and a
//! [`PropertyAnalysisConfig`], it produces a [`PropertyResult`] (and, later, a
//! validator-consumable witness). It performs **no** subprocess execution, reads
//! **no** `expected_verdict`, and does **no** scoring — those concerns belong to
//! the benchmark/self-grading harness in `saf-bench`, and the competition entry
//! point (`saf verify`) lives in `saf-cli`.
//!
//! It sits below both `saf-cli` and `saf-bench` in the crate graph so that the
//! blind `saf verify` command and the self-grading harness share one engine
//! (see plan 192).
#![allow(clippy::doc_markdown)]

pub mod fast_paths;
pub mod fuzz;
pub mod memsafety;
pub mod overflow;
pub mod property;
pub mod property_kind;
pub mod race;
pub mod ranking;
pub mod summaries;
pub mod termination;
pub mod witness_lower;
pub mod witness_yaml;

pub use memsafety::{
    AsanHit, asan_class_to_subproperty, lower_memsafety_hit, memsafety_verdict, parse_asan_report,
};
pub use overflow::{OverflowHit, lower_overflow_hit, overflow_verdict, parse_ubsan_overflow};
pub use property::{
    AnalysisContext, FalseCandidate, NondetCall, PropertyAnalysisConfig, PropertyResult,
    analyze_property, analyze_property_with_context, enumerate_false_candidates,
    enumerate_false_candidates_interproc, must_reach_error, reach_error_call_sites,
};
pub use property_kind::{DataModel, Language, Property};
pub use race::{
    RaceCandidate, RaceHit, find_race_candidates, parse_tsan_report, race_graphml_witness,
};
pub use termination::{
    is_known_returning_external, program_structurally_terminates, termination_verdict,
};
pub use witness_lower::{lower_candidate, lower_must_reach, span_to_location};
pub use witness_yaml::{
    Action, Constraint, SourceWaypoint, ViolationWitness, WaypointKind, WitnessMeta,
    compute_file_hash,
};
