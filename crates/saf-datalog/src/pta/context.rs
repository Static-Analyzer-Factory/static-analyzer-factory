//! Ascent PTA integration with `saf-analysis` `PtaContext`.
//!
//! Provides [`analyze_with_ascent`] as a drop-in alternative to the imperative
//! solver in [`saf_analysis::PtaContext::analyze_with_specs`]. Uses the same
//! constraint extraction and multiplicity classification pipeline, but replaces
//! the worklist solver with the Ascent-based Datalog fixpoint solver and uses
//! demand-driven field location creation instead of eager precomputation.
//!
//! Also provides [`solve_from_constraints`] for re-solving PTA from a
//! pre-built `ConstraintSet` (e.g., after adding resolved indirect call
//! constraints from callgraph refinement).
//!
//! Since `saf-datalog` depends on `saf-analysis` (not the reverse), this
//! module lives here rather than in `saf-analysis::pta::context`. Callers
//! that want Ascent-based PTA should call [`analyze_with_ascent`] directly.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use saf_analysis::cg_refinement::{
    PtaSolveResult, RefinementPrepared, resolve_indirect_calls_from_pts,
};
use saf_analysis::{
    ConstraintSet, FieldSensitivity, IndexSensitivity, LocationFactory, PointsToMap,
    PtaAnalysisResult, PtaConfig, PtaDiagnostics, classify_multiplicity, extract_constraints,
    extract_spec_constraints, resolve_gep_path,
};
use saf_core::air::{AirModule, Constant};
use saf_core::ids::{FunctionId, InstId, ValueId};
use saf_core::spec::SpecRegistry;

use crate::pta::hvn::preprocess_hvn;
use crate::pta::registry::{LocIdRegistry, with_registry};
use crate::pta::scc::{detect_scc, rewrite_facts_with_scc};
use crate::pta::solver::ascent_solve;

/// Run Ascent-based points-to analysis on an AIR module.
///
/// This is the Ascent/Datalog equivalent of
/// [`PtaContext::analyze_with_specs`](saf_analysis::PtaContext::analyze_with_specs).
/// It shares the same constraint extraction and pre-processing pipeline but
/// replaces the imperative worklist solver with the Ascent fixpoint solver.
///
/// # Pipeline
///
/// 1. Extract constraints from module (+ specs if provided)
/// 2. Classify allocation multiplicity
/// 3. Convert constraints to Datalog facts (with index resolution)
/// 4. Apply HVN preprocessing (constraint reduction)
/// 5. Apply SCC detection and rewriting
/// 6. Iterative GEP resolution (demand-driven field locations)
/// 7. Run Ascent fixpoint solver
/// 8. Return `PtaAnalysisResult`
///
/// Unlike the imperative solver, this pipeline **skips**
/// `precompute_indexed_locations()` which eagerly creates a Cartesian product
/// of all objects x all GEP paths. Instead, field locations are created
/// demand-driven during GEP resolution using actual points-to results,
/// avoiding the O(objects * GEPs) blowup that dominates Ascent analysis time
/// on large programs.
///
/// # Arguments
///
/// * `module` — The AIR module to analyze.
/// * `config` — PTA configuration (field sensitivity, etc.).
/// * `specs` — Optional function specifications for library modeling.
// NOTE: `PtaDiagnostics` fields are set incrementally as analysis phases
// complete, so struct-literal initialization is not feasible here.
#[allow(clippy::field_reassign_with_default)]
pub fn analyze_with_ascent(
    module: &AirModule,
    config: &PtaConfig,
    specs: Option<&SpecRegistry>,
) -> PtaAnalysisResult {
    let mut diagnostics = PtaDiagnostics::default();
    let mut factory = LocationFactory::new(config.field_sensitivity.clone());

    if !config.enabled {
        return PtaAnalysisResult {
            pts: PointsToMap::new(),
            constraints: ConstraintSet::default(),
            factory,
            diagnostics,
        };
    }

    // Step 1: Extract constraints from module
    let mut constraints = extract_constraints(module, &mut factory);

    // Step 2: Extract additional constraints from function specs
    if let Some(specs) = specs {
        extract_spec_constraints(module, specs, &mut factory, &mut constraints);
    }

    diagnostics.constraint_count = constraints.total_count();

    // Step 3: Classify allocation multiplicity
    classify_multiplicity(module, &mut factory);

    // Step 4: Collect collapse warnings
    let warnings = factory.drain_warnings();
    diagnostics.collapse_warning_count = warnings.len();
    diagnostics.location_count = factory.len();

    // Step 5: Convert constraints to Datalog facts.
    // Resolve GEP index operands (dynamic array indices → constants) here
    // instead of calling `precompute_indexed_locations()`, which creates an
    // expensive O(objects × GEPs) Cartesian product of field locations.
    // Field locations are created demand-driven during two-phase GEP resolution.
    let mut dl_facts =
        constraint_set_to_facts_ref(&constraints, &module.constants, config.index_sensitivity);

    // Step 6: Apply HVN preprocessing
    let hvn_result = preprocess_hvn(&mut dl_facts);

    // Step 7: Apply SCC detection and rewriting
    let scc_result = detect_scc(&dl_facts.copy);
    rewrite_facts_with_scc(&mut dl_facts, &scc_result.representatives);

    // Step 8: Iterative demand-driven GEP resolution
    //
    // Phase 1: Solve without GEPs to get preliminary points-to.
    // This captures locations reachable via store/load chains that
    // single-pass addr_of-only resolution would miss.
    let original_geps = std::mem::take(&mut dl_facts.gep);

    let current_pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };

    // Phase 2+: Iteratively resolve GEP facts until fixpoint.
    // Each iteration resolves GEPs whose src_ptr has known points-to
    // targets, adds addr_of facts for field locations, and re-solves.
    // GEPs whose src_ptr has no pts yet (e.g., from virtual dispatch
    // resolved after callgraph refinement) are retried in subsequent
    // iterations as the points-to map grows.
    let mut pts = iterative_gep_resolve(
        &mut dl_facts,
        &mut factory,
        original_geps,
        current_pts,
        false,
    );

    // Expand HVN-merged values back to their original ValueIds.
    // HVN merges equivalent values into representative nodes; the solver
    // only produces PTS entries for representatives. Without expansion,
    // original values that were merged away have empty PTS.
    hvn_result.expand_results(&mut pts);

    // Also expand SCC-collapsed values. SCC rewriting maps all values in
    // a strongly connected component to the minimum ValueId representative.
    // Like HVN, original (non-representative) values need expansion.
    for (original, rep) in &scc_result.representatives {
        if let Some(rep_pts) = pts.get(rep).cloned() {
            pts.insert(*original, rep_pts);
        }
    }

    // Ascent always converges (no iteration limit)
    diagnostics.iteration_limit_hit = false;

    PtaAnalysisResult {
        pts,
        constraints,
        factory,
        diagnostics,
    }
}

/// Maximum number of GEP resolution iterations before falling back to copy.
///
/// Each iteration resolves GEPs whose `src_ptr` has known points-to targets,
/// adds field-qualified `addr_of` facts, and re-solves. However, re-solving
/// grows the points-to sets, causing subsequent GEP resolutions to produce
/// O(|`pending_geps`| * `avg_pts_size`) new `addr_of` entries — an exponential
/// feedback loop. Profiling on bash (`CruxBC`) shows:
///   iter 0: 82K `addr_of` (1.2s)
///   iter 1: 310K `addr_of` (6.5s)
///   iter 2: 1.25M `addr_of` (hundreds of seconds)
///
/// 2 iterations is the sweet spot: resolves ~56% of GEPs precisely while
/// keeping solve time manageable. Remaining GEPs use conservative copy edges
/// (overapproximation — sound but less precise).
const MAX_GEP_ITERATIONS: usize = 2;

/// Iteratively resolve GEP facts and solve until fixpoint.
///
/// Returns the final points-to map after all resolvable GEPs have been
/// processed. Unresolvable GEPs (where `src_ptr` never appears in any
/// points-to set) fall back to conservative copy edges.
///
/// When `defer_final_solve` is true, unresolved GEPs are converted to copy
/// fallbacks in `dl_facts` but the final re-solve is skipped. The caller is
/// responsible for doing the solve later (e.g., after adding CG refinement
/// copies). This avoids a redundant full Ascent fixpoint when the caller
/// will re-solve anyway.
fn iterative_gep_resolve(
    dl_facts: &mut crate::facts::PtaFacts,
    factory: &mut LocationFactory,
    original_geps: Vec<(ValueId, ValueId, saf_analysis::FieldPath)>,
    mut current_pts: crate::pta::solver::PointsToMap,
    defer_final_solve: bool,
) -> PointsToMap {
    let mut pending_geps = original_geps;
    // Track whether the loop's last action was a re-solve, to avoid
    // a redundant final solve when no copy fallbacks were added after it.
    let mut needs_final_solve = false;

    for _iteration in 0..MAX_GEP_ITERATIONS {
        if pending_geps.is_empty() {
            break;
        }

        // Move pending GEPs into facts (take leaves pending_geps valid but empty)
        dl_facts.gep = std::mem::take(&mut pending_geps);

        // Try to resolve — returns unresolved GEPs without copy fallback
        let result = crate::pta::gep::try_resolve_gep_facts(dl_facts, factory, &current_pts);

        if result.resolved_count == 0 {
            // No progress — add copy fallbacks for remaining unresolved GEPs
            for (dst, src, _) in result.unresolved {
                dl_facts.copy.push((dst, src));
            }
            needs_final_solve = true;
            break;
        }

        pending_geps = result.unresolved;

        // Re-solve with newly resolved GEP addr_of facts.
        // Rebuild registry since GEP resolution created new LocIds.
        let registry = Arc::new(LocIdRegistry::from_facts(dl_facts));
        current_pts = with_registry(registry, || ascent_solve(dl_facts));
        needs_final_solve = false;
    }

    // Add copy fallbacks for any GEPs remaining after loop exhaustion
    if !pending_geps.is_empty() {
        for (dst, src, _) in pending_geps {
            dl_facts.copy.push((dst, src));
        }
        needs_final_solve = true;
    }

    if needs_final_solve && !defer_final_solve {
        // Re-solve to incorporate copy fallbacks added after the last solve
        let registry = Arc::new(LocIdRegistry::from_facts(dl_facts));
        with_registry(registry, || ascent_solve(dl_facts))
    } else {
        current_pts
    }
}

/// Solve PTA from a pre-built `ConstraintSet` using the Ascent pipeline.
///
/// Performs fact conversion, HVN preprocessing, SCC detection, iterative
/// GEP resolution, and Ascent fixpoint solving. Use this when you have an
/// augmented `ConstraintSet` (e.g., after adding resolved indirect call
/// constraints from callgraph refinement) and need to re-solve.
///
/// This is the "Steps 5-9" portion of [`analyze_with_ascent`], extracted
/// for reuse by callers that manage constraint extraction themselves.
pub fn solve_from_constraints(
    constraints: &ConstraintSet,
    constants: &std::collections::BTreeMap<ValueId, Constant>,
    index_sensitivity: IndexSensitivity,
    factory: &mut LocationFactory,
) -> PointsToMap {
    // Convert constraints to Datalog facts (with GEP index resolution)
    let mut dl_facts = constraint_set_to_facts_ref(constraints, constants, index_sensitivity);

    // Apply HVN preprocessing
    let hvn_result = preprocess_hvn(&mut dl_facts);

    // Apply SCC detection and rewriting
    let scc_result = detect_scc(&dl_facts.copy);
    rewrite_facts_with_scc(&mut dl_facts, &scc_result.representatives);

    // Iterative GEP resolution + solve
    let original_geps = std::mem::take(&mut dl_facts.gep);

    let current_pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };

    let mut pts = iterative_gep_resolve(&mut dl_facts, factory, original_geps, current_pts, false);

    // Expand HVN-merged values back to original ValueIds
    hvn_result.expand_results(&mut pts);

    // Expand SCC-collapsed values back to original ValueIds
    for (original, rep) in &scc_result.representatives {
        if let Some(rep_pts) = pts.get(rep).cloned() {
            pts.insert(*original, rep_pts);
        }
    }

    pts
}

/// Run Ascent-based PTA with CG refinement.
///
/// Replaces the legacy worklist solver inside the CG refinement loop.
/// First iteration runs the full Ascent pipeline (SCC + iterative GEP).
/// Subsequent iterations only add interprocedural copy facts and re-solve
/// (skipping HVN/SCC/GEP -- only copy constraints change).
/// Maximum CG refinement iterations for Ascent.
///
/// Ascent is a batch solver — each re-solve recomputes the full fixpoint
/// (~18s on bash). Unlike the legacy worklist solver which incrementally
/// drains only new constraints, Ascent must replay all facts. Capping at
/// 1 CG refinement pass (2 total solves) balances soundness with performance:
/// the first solve resolves most indirect calls, and one refinement pass
/// picks up interprocedural edges from newly discovered targets.
const MAX_ASCENT_CG_ITERATIONS: usize = 1;

// NOTE: This function implements the full Ascent CG refinement pipeline
// as a single cohesive unit including profiling instrumentation.
#[allow(clippy::too_many_lines)]
pub fn refine_with_ascent(
    module: &AirModule,
    prepared: &mut RefinementPrepared,
    max_iterations: usize,
) -> PtaSolveResult {
    #[cfg(not(target_arch = "wasm32"))]
    let pta_start = std::time::Instant::now();
    let index_sensitivity = IndexSensitivity::default();
    // Cap iterations: Ascent re-solves are expensive (full fixpoint each time)
    let effective_max = max_iterations.min(MAX_ASCENT_CG_ITERATIONS);

    // --- First iteration: full Ascent pipeline (SCC + GEP) ---
    let mut dl_facts =
        constraint_set_to_facts_ref(&prepared.reduced, &module.constants, index_sensitivity);

    let scc_result = detect_scc(&dl_facts.copy);
    rewrite_facts_with_scc(&mut dl_facts, &scc_result.representatives);

    let original_geps = std::mem::take(&mut dl_facts.gep);

    let current_pts = {
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        with_registry(registry, || ascent_solve(&dl_facts))
    };

    // Defer the final copy-fallback solve so we can merge it with CG refinement.
    // This avoids one full Ascent fixpoint (~15s on bash).
    let mut pts = iterative_gep_resolve(
        &mut dl_facts,
        &mut prepared.factory,
        original_geps,
        current_pts,
        true, // defer_final_solve: merge with CG refinement
    );

    // Expand HVN mapping (prepared.hvn_result is saf_analysis `HvnResult`, access mapping directly)
    for (original, rep) in &prepared.hvn_result.mapping {
        if let Some(p) = pts.get(rep).cloned() {
            pts.insert(*original, p);
        }
    }

    // Expand SCC-collapsed values
    for (original, rep) in &scc_result.representatives {
        if let Some(rep_pts) = pts.get(rep).cloned() {
            pts.insert(*original, rep_pts);
        }
    }

    let mut resolved_calls: BTreeMap<InstId, BTreeSet<FunctionId>> = BTreeMap::new();
    let mut iterations = 1;

    // --- CG refinement: resolve indirect calls using pre-copy-fallback PTS,
    //     then merge copy fallbacks + CG copies into ONE combined solve. ---
    //
    // The GEP resolution deferred its final copy-fallback solve so we can
    // batch it with CG refinement copies. This avoids one full Ascent
    // fixpoint (~15s on bash).
    {
        // Resolve indirect calls using the pre-copy-fallback PTS.
        // This may miss a few targets that only become visible after copy
        // fallbacks expand PTS, but the tradeoff saves a full re-solve.
        let new_copies = if effective_max > 0 {
            resolve_indirect_calls_from_pts(
                &pts,
                &prepared.factory,
                &prepared.indirect_sites,
                &prepared.func_loc_map,
                module,
                &prepared.return_values,
                &mut resolved_calls,
                &mut prepared.cg,
            )
        } else {
            Vec::new()
        };

        if !new_copies.is_empty() {
            iterations += 1;
            let extra: Vec<(ValueId, ValueId)> =
                new_copies.iter().map(|c| (c.dst, c.src)).collect();
            dl_facts.copy.extend(extra);
        }

        // Combined solve: copy fallbacks + CG copies in one pass
        let registry = Arc::new(LocIdRegistry::from_facts(&dl_facts));
        let raw_pts = with_registry(registry, || ascent_solve(&dl_facts));

        // Expand HVN mapping
        pts = raw_pts;
        for (original, rep) in &prepared.hvn_result.mapping {
            if let Some(p) = pts.get(rep).cloned() {
                pts.insert(*original, p);
            }
        }

        // Expand SCC-collapsed values
        for (original, rep) in &scc_result.representatives {
            if let Some(rep_pts) = pts.get(rep).cloned() {
                pts.insert(*original, rep_pts);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    let total_secs = pta_start.elapsed().as_secs_f64();
    #[cfg(target_arch = "wasm32")]
    let total_secs = 0.0;

    PtaSolveResult {
        pts,
        factory: std::mem::replace(
            &mut prepared.factory,
            LocationFactory::new(FieldSensitivity::default()),
        ),
        resolved_calls,
        iterations,
        pta_solve_secs: total_secs,
        iteration_limit_hit: false, // Ascent always converges
    }
}

/// Convenience wrapper: run full Ascent-based CG refinement pipeline.
///
/// Calls [`refine_prepare()`](saf_analysis::cg_refinement::refine_prepare),
/// [`refine_with_ascent()`], and
/// [`refine_finalize()`](saf_analysis::cg_refinement::refine_finalize)
/// in sequence.
pub fn refine_ascent(
    module: &AirModule,
    config: &saf_analysis::cg_refinement::RefinementConfig,
    specs: Option<&SpecRegistry>,
) -> saf_analysis::cg_refinement::RefinementResult {
    let mut prepared = saf_analysis::cg_refinement::refine_prepare(module, config, specs);
    let solve_result = refine_with_ascent(module, &mut prepared, config.max_iterations);
    saf_analysis::cg_refinement::refine_finalize(module, prepared, solve_result)
}

/// Convert a `ConstraintSet` reference into `PtaFacts` without consuming it.
///
/// Similar to `constraint_set_to_facts` in `facts.rs` but works on a reference
/// so the original `ConstraintSet` can be returned in the `PtaAnalysisResult`.
///
/// GEP paths are resolved with index operand information (dynamic array indices
/// are replaced with constant values when the operand is a known constant).
/// This preserves the precision that `precompute_indexed_locations` would have
/// provided, without the expensive Cartesian product of objects x paths.
fn constraint_set_to_facts_ref(
    cs: &ConstraintSet,
    constants: &std::collections::BTreeMap<saf_core::ids::ValueId, saf_core::air::Constant>,
    index_sensitivity: saf_analysis::IndexSensitivity,
) -> crate::facts::PtaFacts {
    crate::facts::PtaFacts {
        addr_of: cs.addr.iter().map(|a| (a.ptr, a.loc)).collect(),
        copy: cs.copy.iter().map(|c| (c.dst, c.src)).collect(),
        load: cs.load.iter().map(|l| (l.dst, l.src_ptr)).collect(),
        store: cs.store.iter().map(|s| (s.dst_ptr, s.src)).collect(),
        gep: cs
            .gep
            .iter()
            .map(|g| {
                let resolved_path = resolve_gep_path(
                    &g.path,
                    &g.index_operands,
                    Some(constants),
                    index_sensitivity,
                );
                (g.dst, g.src_ptr, resolved_path)
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirBlock, AirFunction, Instruction, Operation};
    use saf_core::ids::{BlockId, FunctionId, InstId, ModuleId, ValueId};

    fn make_config() -> PtaConfig {
        PtaConfig::default()
    }

    fn make_empty_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn ascent_analyze_empty_module() {
        let config = make_config();
        let module = make_empty_module();
        let result = analyze_with_ascent(&module, &config, None);

        assert!(result.pts.is_empty());
        assert!(result.constraints.is_empty());
        assert_eq!(result.diagnostics.constraint_count, 0);
    }

    #[test]
    fn ascent_analyze_disabled_returns_empty() {
        let mut config = make_config();
        config.enabled = false;

        let mut module = make_empty_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test"), "test");
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(ValueId::derive(b"ptr")),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let result = analyze_with_ascent(&module, &config, None);
        assert!(result.pts.is_empty());
    }

    #[test]
    fn ascent_analyze_single_alloca() {
        let config = make_config();
        let mut module = make_empty_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test"), "test");
        let dst_id = ValueId::derive(b"ptr");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(dst_id),
        );
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let result = analyze_with_ascent(&module, &config, None);

        // Should have Addr constraints: 1 for alloca + 1 for function address
        assert_eq!(result.constraints.addr.len(), 2);
        assert!(result.pts.contains_key(&dst_id));
        assert_eq!(result.diagnostics.constraint_count, 2);
    }

    #[test]
    fn ascent_analyze_gep_creates_field_locations() {
        use saf_core::air::FieldPath as AirFieldPath;
        use saf_core::air::FieldStep as AirFieldStep;

        let config = make_config();
        let mut module = make_empty_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test"), "test");
        let base_ptr = ValueId::derive(b"base");
        let field_ptr1 = ValueId::derive(b"field1");
        let field_ptr2 = ValueId::derive(b"field2");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        // Alloca for base object
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(base_ptr),
        );

        // GEP to field 0 (path [0, 0])
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"gep1"),
                Operation::Gep {
                    field_path: AirFieldPath {
                        steps: vec![
                            AirFieldStep::Field { index: 0 },
                            AirFieldStep::Field { index: 0 },
                        ],
                    },
                },
            )
            .with_dst(field_ptr1)
            .with_operands(vec![base_ptr]),
        );

        // GEP to field 1 (path [0, 1])
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"gep2"),
                Operation::Gep {
                    field_path: AirFieldPath {
                        steps: vec![
                            AirFieldStep::Field { index: 0 },
                            AirFieldStep::Field { index: 1 },
                        ],
                    },
                },
            )
            .with_dst(field_ptr2)
            .with_operands(vec![base_ptr]),
        );

        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let result = analyze_with_ascent(&module, &config, None);

        // Both GEPs should have points-to entries
        assert!(
            result.pts.contains_key(&field_ptr1),
            "field_ptr1 should have points-to set"
        );
        assert!(
            result.pts.contains_key(&field_ptr2),
            "field_ptr2 should have points-to set"
        );

        // The two field pointers should point to DIFFERENT locations
        let pts1 = result.pts.get(&field_ptr1).unwrap();
        let pts2 = result.pts.get(&field_ptr2).unwrap();
        assert_ne!(
            pts1, pts2,
            "Different fields should point to different locations"
        );
    }
}
