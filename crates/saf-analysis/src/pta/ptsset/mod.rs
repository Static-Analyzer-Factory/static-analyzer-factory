//! Points-to set representations for scalable pointer analysis.
//!
//! This module provides multiple implementations of points-to sets, allowing
//! the PTA solver to select the most efficient representation based on program
//! characteristics:
//!
//! - [`FxHashPtsSet`]: Default for small programs (<10K alloc sites). O(1) hash operations.
//! - [`RoaringPtsSet`]: Default for medium/large programs (>=10K alloc sites). Compressed
//!   bitmaps with SIMD-optimized operations and frozen (lock-free) indexer during solving.
//! - [`BddPtsSet`]: Experimental, explicit opt-in for >100K sites. Structural sharing.
//! - [`BTreePtsSet`]: Baseline for debugging and deterministic inspection.
//! - [`IdBitSet`]: Generic bitvec set for non-PTS uses (worklists, etc.).
//!
//! Selection is controlled via [`PtsConfig`] with auto-detection based on
//! allocation site count. The solver pre-builds a [`FrozenIndexer`] from all
//! allocation sites before solving, enabling lock-free `RoaringPtsSet` operations.
//!
//! # Example
//!
//! ```ignore
//! use saf_analysis::pta::ptsset::{PtsSet, BTreePtsSet, RoaringPtsSet, BddPtsSet};
//! use saf_core::ids::LocId;
//! use std::sync::{Arc, RwLock};
//!
//! // BTreePtsSet for small programs
//! let mut pts = BTreePtsSet::empty();
//! pts.insert(LocId::new(1));
//! pts.insert(LocId::new(2));
//! assert_eq!(pts.len(), 2);
//!
//! // RoaringPtsSet for medium programs (shared indexer for efficiency)
//! use saf_analysis::pta::ptsset::LocIdIndexer;
//!
//! let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
//! let mut pts = RoaringPtsSet::with_indexer(Arc::clone(&indexer));
//! pts.insert(LocId::new(1));
//! pts.insert(LocId::new(2));
//! assert_eq!(pts.len(), 2);
//!
//! // BddPtsSet for large programs (shared context and indexer)
//! use saf_analysis::pta::ptsset::BddContext;
//!
//! let context = Arc::new(RwLock::new(BddContext::new(16)));
//! let indexer = Arc::new(RwLock::new(LocIdIndexer::new()));
//! let mut pts = BddPtsSet::with_context_and_indexer(Arc::clone(&context), Arc::clone(&indexer));
//! pts.insert(LocId::new(1));
//! pts.insert(LocId::new(2));
//! assert_eq!(pts.len(), 2);
//! ```

mod bdd;
mod btree;
mod config;
mod fxhash;
mod id_bitset;
mod indexer;
mod roaring_pts;
mod trait_def;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod cross_impl_tests;
#[cfg(test)]
mod edge_case_tests;

// Public API re-exports - some may not be used internally but are part of the public API
// for external users who want fine-grained control over PtsSet implementations.
#[allow(unused_imports)]
pub use bdd::{BddContext, BddPtsSet};
pub use btree::BTreePtsSet;
#[allow(unused_imports)]
pub use config::{ClusteringMode, PtsConfig, PtsRepresentation, count_allocation_sites};
pub use fxhash::FxHashPtsSet;
#[allow(unused_imports)]
pub use id_bitset::IdBitSet;
#[allow(unused_imports)]
pub use indexer::{FrozenIndexer, FrozenLocIdIndexer, Indexer, LocIdIndexer};
#[allow(unused_imports)]
pub use roaring_pts::RoaringPtsSet;
pub use trait_def::PtsSet;
