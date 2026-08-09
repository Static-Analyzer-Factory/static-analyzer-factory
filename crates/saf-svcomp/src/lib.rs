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
pub mod property;
pub mod property_kind;
pub mod summaries;
pub mod witness;

pub use property::{
    AnalysisContext, PropertyAnalysisConfig, PropertyResult, analyze_property,
    analyze_property_with_context,
};
pub use property_kind::{DataModel, Language, Property};
pub use witness::{Witness, WitnessEdge, WitnessNode, WitnessType};
