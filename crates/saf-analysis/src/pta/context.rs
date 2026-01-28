//! PTA context and analysis entry point.
//!
//! Bundles configuration, location factory, and diagnostics for a PTA run.

use saf_core::air::{AirModule, Constant};
use saf_core::ids::ObjId;
use saf_core::spec::SpecRegistry;

use super::config::{IndexSensitivity, PtaConfig};
use super::constraint::ConstraintSet;
use super::extract::extract_constraints;
use super::location::{FieldPath, LocationFactory, PathStep, resolve_gep_path};
use super::solver::{PointsToMap, solve_with_index_config};
use super::spec_constraints::extract_spec_constraints;

/// Diagnostics collected during PTA.
#[derive(Debug, Clone, Default)]
pub struct PtaDiagnostics {
    /// Number of solver iterations used.
    pub iterations: usize,
    /// Whether the solver hit the iteration limit.
    pub iteration_limit_hit: bool,
    /// Number of collapse warnings generated.
    pub collapse_warning_count: usize,
    /// Total constraints extracted.
    pub constraint_count: usize,
    /// Total locations created.
    pub location_count: usize,
}

/// Context for running points-to analysis.
///
/// Bundles configuration, location factory, and diagnostics.
pub struct PtaContext {
    /// Analysis configuration.
    config: PtaConfig,
    /// Location factory for managing abstract locations.
    factory: LocationFactory,
    /// Diagnostics collected during analysis.
    diagnostics: PtaDiagnostics,
}

impl PtaContext {
    /// Create a new PTA context with the given configuration.
    #[must_use]
    pub fn new(config: PtaConfig) -> Self {
        let factory = LocationFactory::new(config.field_sensitivity.clone());
        Self {
            config,
            factory,
            diagnostics: PtaDiagnostics::default(),
        }
    }

    /// Run points-to analysis on an AIR module.
    ///
    /// Returns the analysis result including points-to sets and diagnostics.
    pub fn analyze(&mut self, module: &AirModule) -> PtaAnalysisResult {
        self.analyze_with_specs(module, None)
    }

    /// Run points-to analysis with optional function specifications.
    ///
    /// When specs are provided, additional constraints are extracted for
    /// library functions (e.g., malloc returns fresh heap, memcpy copies).
    pub fn analyze_with_specs(
        &mut self,
        module: &AirModule,
        specs: Option<&SpecRegistry>,
    ) -> PtaAnalysisResult {
        if !self.config.enabled {
            return PtaAnalysisResult {
                pts: PointsToMap::new(),
                constraints: ConstraintSet::default(),
                factory: std::mem::replace(
                    &mut self.factory,
                    LocationFactory::new(self.config.field_sensitivity.clone()),
                ),
                diagnostics: self.diagnostics.clone(),
            };
        }

        // Extract constraints from module
        // (this includes global initializer constraints via extract_base_constraints)
        let mut constraints = extract_constraints(module, &mut self.factory);

        // Extract additional constraints from function specs
        if let Some(specs) = specs {
            extract_spec_constraints(module, specs, &mut self.factory, &mut constraints);
        }

        self.diagnostics.constraint_count = constraints.total_count();

        // Pre-create field locations for all GEP constraints
        // This ensures the solver can find exact field matches instead of
        // falling back to base object locations
        precompute_indexed_locations(
            &constraints,
            &mut self.factory,
            &module.constants,
            self.config.index_sensitivity,
        );

        // Classify allocation multiplicity for must-alias soundness.
        // Must run after constraint extraction (which registers regions)
        // and before building PtaResult (which reads multiplicities).
        super::multiplicity::classify_multiplicity(module, &mut self.factory);

        // Collect collapse warnings
        let warnings = self.factory.drain_warnings();
        self.diagnostics.collapse_warning_count = warnings.len();
        self.diagnostics.location_count = self.factory.len();

        // Solve with index sensitivity configuration
        let (pts, iteration_limit_hit) = solve_with_index_config(
            &constraints,
            &self.factory,
            self.config.max_iterations,
            &self.config.pts_config,
            Some(module),
            self.config.index_sensitivity,
        );
        self.diagnostics.iteration_limit_hit = iteration_limit_hit;

        // Take factory out for the result
        let factory = std::mem::replace(
            &mut self.factory,
            LocationFactory::new(self.config.field_sensitivity.clone()),
        );

        PtaAnalysisResult {
            pts,
            constraints,
            factory,
            diagnostics: self.diagnostics.clone(),
        }
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &PtaConfig {
        &self.config
    }

    /// Get the current diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &PtaDiagnostics {
        &self.diagnostics
    }
}

/// Result of points-to analysis.
pub struct PtaAnalysisResult {
    /// Points-to map from values to locations.
    pub pts: PointsToMap,
    /// Constraints extracted from the module.
    pub constraints: ConstraintSet,
    /// Location factory with all created locations.
    pub factory: LocationFactory,
    /// Diagnostics from the analysis.
    pub diagnostics: PtaDiagnostics,
}

/// Pre-create field locations for all GEP constraints.
///
/// For each GEP constraint, creates corresponding field locations in the factory.
/// This ensures the solver can find exact matches instead of falling back to
/// collapsed base locations.
///
/// Uses a two-phase approach:
/// 1. First, add every GEP's direct path (handles GEPs whose src is a Copy target)
/// 2. Then, trace GEP chains to compute cumulative paths for chained GEPs
///
/// Also resolves constant index operands when index sensitivity is enabled.
///
/// This function should be called after constraint extraction and before solving.
/// The CI, CS, and FS pointer analyses all need to call this to get field-sensitive
/// locations.
pub fn precompute_indexed_locations(
    constraints: &ConstraintSet,
    factory: &mut LocationFactory,
    constants: &std::collections::BTreeMap<saf_core::ids::ValueId, Constant>,
    index_sensitivity: IndexSensitivity,
) {
    use saf_core::ids::ValueId;
    use std::collections::BTreeMap;

    // Collect only objects that have address-taken constraints (allocas, globals,
    // heap allocations). Function-address objects and other objects that never
    // need field locations are excluded, avoiding a Cartesian product blowup.
    let objects: Vec<ObjId> = constraints
        .addr
        .iter()
        .filter_map(|a| factory.get(a.loc).map(|l| l.obj))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // Build a map from GEP dst → resolved path
    // We'll use this to trace GEP chains
    let mut gep_dst_to_path: BTreeMap<ValueId, FieldPath> = BTreeMap::new();
    let mut gep_dst_to_src: BTreeMap<ValueId, ValueId> = BTreeMap::new();

    for gep in &constraints.gep {
        let resolved_path = resolve_gep_path(
            &gep.path,
            &gep.index_operands,
            Some(constants),
            index_sensitivity,
        );
        gep_dst_to_path.insert(gep.dst, resolved_path);
        gep_dst_to_src.insert(gep.dst, gep.src_ptr);
    }

    // PHASE 1: Add every GEP's direct path
    // This handles GEPs where src_ptr is a Copy target (not direct alloca or another GEP)
    // Without this, paths like `%gep = gep %copy_of_alloca, 0, 0` would be missed
    for gep in &constraints.gep {
        let resolved_path = gep_dst_to_path.get(&gep.dst).cloned().unwrap_or_default();
        if !resolved_path.steps.is_empty() {
            for &obj in &objects {
                factory.get_or_create(obj, resolved_path.clone());
            }
        }
    }

    // Find allocation sites (values with Addr constraints)
    let alloc_sites: std::collections::BTreeSet<ValueId> =
        constraints.addr.iter().map(|addr| addr.ptr).collect();

    // PHASE 2: Trace GEP chains to compute cumulative paths
    // This handles chained GEPs like `%gep2 = gep %gep1, 0, 0`
    for gep in &constraints.gep {
        // Collect all paths in the chain from this GEP back to an alloca
        let mut cumulative_paths: Vec<FieldPath> = Vec::new();
        compute_cumulative_paths(
            gep.dst,
            &gep_dst_to_path,
            &gep_dst_to_src,
            &alloc_sites,
            &FieldPath::empty(),
            &mut cumulative_paths,
        );

        // Create locations for all cumulative paths
        for path in cumulative_paths {
            if path.steps.is_empty() {
                continue;
            }
            for &obj in &objects {
                factory.get_or_create(obj, path.clone());
            }
        }
    }
}

/// Recursively compute cumulative paths for a GEP chain.
///
/// Starting from `current`, traces back through GEP chains until reaching
/// an allocation site, accumulating the full path along the way.
///
/// Also handles pointer-arithmetic GEP chains: when a single-step GEP operates
/// on the result of another GEP (e.g., `GEP ptr, %arr_elem0, 1` to advance from
/// element 0 to element 1), creates an "index-merged" path where the child's
/// index replaces the parent's last step. This connects stores through pointer
/// arithmetic to loads through direct array indexing.
fn compute_cumulative_paths(
    current: saf_core::ids::ValueId,
    gep_dst_to_path: &std::collections::BTreeMap<saf_core::ids::ValueId, FieldPath>,
    gep_dst_to_src: &std::collections::BTreeMap<saf_core::ids::ValueId, saf_core::ids::ValueId>,
    alloc_sites: &std::collections::BTreeSet<saf_core::ids::ValueId>,
    accumulated_path: &FieldPath,
    results: &mut Vec<FieldPath>,
) {
    // Get this GEP's path
    let Some(gep_path) = gep_dst_to_path.get(&current) else {
        // Not a GEP destination, nothing to do
        return;
    };

    // Extend the accumulated path with this GEP's path
    let new_accumulated = gep_path.extend(accumulated_path);

    // Always add the current cumulative path
    results.push(new_accumulated.clone());

    // Get the source of this GEP
    let Some(&src) = gep_dst_to_src.get(&current) else {
        return;
    };

    // If the source is an allocation site, we've reached the end
    if alloc_sites.contains(&src) {
        return;
    }

    // If the source is another GEP, recurse
    if gep_dst_to_path.contains_key(&src) {
        // Standard path composition (append)
        compute_cumulative_paths(
            src,
            gep_dst_to_path,
            gep_dst_to_src,
            alloc_sites,
            &new_accumulated,
            results,
        );

        // Pointer-arithmetic merging: when a single-step GEP (e.g., `GEP ptr, %elem, N`)
        // chains off another GEP whose last step is a Field index, the child GEP is doing
        // pointer arithmetic within the parent's array. Merge by replacing the parent's
        // last index step with the sum of both indices.
        //
        // Example: `%6 = GEP [2 x ptr], %4, 0, 0` (path [F0, F0]) then
        //          `%7 = GEP ptr, %6, 1` (path [F1])
        //   → merged path: [F0, F(0+1)] = [F0, F1]
        //   This matches `%8 = GEP [2 x ptr], %4, 0, 1` (path [F0, F1])
        if let Some(parent_path) = gep_dst_to_path.get(&src) {
            if gep_path.steps.len() == 1 {
                if let (
                    Some(PathStep::Field { index: child_idx }),
                    Some(PathStep::Field {
                        index: parent_last_idx,
                    }),
                ) = (gep_path.steps.first(), parent_path.steps.last())
                {
                    // Create merged path: parent's path with last step replaced by sum
                    let merged_idx = parent_last_idx.saturating_add(*child_idx);
                    let mut merged_steps = parent_path.steps.clone();
                    if let Some(last) = merged_steps.last_mut() {
                        *last = PathStep::Field { index: merged_idx };
                    }
                    let merged_path = FieldPath {
                        steps: merged_steps,
                    };
                    if !merged_path.steps.is_empty() {
                        // Also extend with accumulated path from downstream GEPs
                        let full_merged = merged_path.extend(accumulated_path);
                        results.push(full_merged);
                    }
                }
            }
        }
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
    fn context_new_creates_with_config() {
        let config = make_config();
        let ctx = PtaContext::new(config.clone());
        assert_eq!(ctx.config().enabled, config.enabled);
    }

    #[test]
    fn analyze_empty_module() {
        let mut ctx = PtaContext::new(make_config());
        let module = make_empty_module();
        let result = ctx.analyze(&module);

        assert!(result.pts.is_empty());
        assert!(result.constraints.is_empty());
        assert_eq!(result.diagnostics.constraint_count, 0);
    }

    #[test]
    fn analyze_disabled_returns_empty() {
        let mut config = make_config();
        config.enabled = false;
        let mut ctx = PtaContext::new(config);

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

        let result = ctx.analyze(&module);

        // Even with instructions, disabled analysis returns empty
        assert!(result.pts.is_empty());
    }

    #[test]
    fn analyze_single_alloca() {
        let mut ctx = PtaContext::new(make_config());

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

        let result = ctx.analyze(&module);

        // Should have Addr constraints: 1 for alloca + 1 for function address
        assert_eq!(result.constraints.addr.len(), 2);
        assert!(result.pts.contains_key(&dst_id));
        assert_eq!(result.diagnostics.constraint_count, 2);
        assert_eq!(result.diagnostics.location_count, 2);
    }

    #[test]
    fn analyze_preserves_diagnostics() {
        let mut ctx = PtaContext::new(make_config());

        let mut module = make_empty_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test"), "test");

        // Add multiple allocas
        let mut block = AirBlock::new(BlockId::derive(b"entry"));
        for i in 0..5 {
            let inst_id = InstId::derive(format!("alloca_{i}").as_bytes());
            let dst_id = ValueId::derive(format!("ptr_{i}").as_bytes());
            block.instructions.push(
                Instruction::new(inst_id, Operation::Alloca { size_bytes: None }).with_dst(dst_id),
            );
        }
        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let result = ctx.analyze(&module);

        // 5 alloca Addr constraints + 1 function address Addr constraint
        assert_eq!(result.diagnostics.constraint_count, 6);
        assert_eq!(result.diagnostics.location_count, 6);
    }

    #[test]
    fn analyze_gep_creates_field_locations() {
        use saf_core::air::FieldPath as AirFieldPath;
        use saf_core::air::FieldStep as AirFieldStep;

        let mut ctx = PtaContext::new(make_config());

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

        let result = ctx.analyze(&module);

        // Should have 6 locations:
        //   2 base (alloca_obj + func_obj) +
        //   2 field locations for alloca_obj ([0,0] and [0,1]) +
        //   2 field locations for func_obj ([0,0] and [0,1])
        // precompute_indexed_locations creates field locs for ALL Addr-constraint
        // objects, which includes both the alloca and the function address.
        assert_eq!(
            result.diagnostics.location_count, 6,
            "Expected 6 locations (2 base + 2 alloca fields + 2 func fields)"
        );

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

    /// Test that field locations are created even when GEP src is a Copy target.
    ///
    /// This mimics the structure in PTABen tests like struct-twoflds.c:
    /// ```c
    /// struct IntChar s;      // alloca
    /// pint1 = &s;            // Copy
    /// &pint1->f1             // GEP from Copy target
    /// &pint1->f2             // GEP from Copy target
    /// ```
    #[test]
    fn analyze_gep_with_copy_between_alloca_and_gep() {
        use saf_core::air::FieldPath as AirFieldPath;
        use saf_core::air::FieldStep as AirFieldStep;

        let mut ctx = PtaContext::new(make_config());

        let mut module = make_empty_module();
        let mut func = AirFunction::new(FunctionId::derive(b"test"), "test");
        let alloca_ptr = ValueId::derive(b"alloca");
        let copy_ptr = ValueId::derive(b"copy");
        let field_ptr1 = ValueId::derive(b"field1");
        let field_ptr2 = ValueId::derive(b"field2");

        let mut block = AirBlock::new(BlockId::derive(b"entry"));

        // Alloca for base object
        block.instructions.push(
            Instruction::new(
                InstId::derive(b"alloca"),
                Operation::Alloca { size_bytes: None },
            )
            .with_dst(alloca_ptr),
        );

        // Copy from alloca (like pint1 = &s)
        block.instructions.push(
            Instruction::new(InstId::derive(b"copy"), Operation::Copy)
                .with_dst(copy_ptr)
                .with_operands(vec![alloca_ptr]),
        );

        // GEP to field 0 from the Copy target (like &pint1->f1)
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
            .with_operands(vec![copy_ptr]),
        );

        // GEP to field 1 from the Copy target (like &pint1->f2)
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
            .with_operands(vec![copy_ptr]),
        );

        block
            .instructions
            .push(Instruction::new(InstId::derive(b"ret"), Operation::Ret));
        func.blocks.push(block);
        module.functions.push(func);

        let result = ctx.analyze(&module);

        // Debug: print all locations
        eprintln!("Locations: {}", result.diagnostics.location_count);
        for (loc_id, loc) in result.factory.all_locations() {
            eprintln!("  loc={:?} obj={:?} path={:?}", loc_id, loc.obj, loc.path);
        }

        // Should have 6 locations:
        //   2 base (alloca_obj + func_obj) +
        //   2 field locations for alloca_obj ([0,0] and [0,1]) +
        //   2 field locations for func_obj ([0,0] and [0,1])
        // precompute_indexed_locations creates field locs for ALL Addr-constraint
        // objects, which includes both the alloca and the function address.
        assert_eq!(
            result.diagnostics.location_count, 6,
            "Expected 6 locations (2 base + 2 alloca fields + 2 func fields)"
        );

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

        eprintln!("pts1: {:?}", pts1);
        eprintln!("pts2: {:?}", pts2);

        assert_ne!(
            pts1, pts2,
            "Different fields should point to different locations"
        );
    }
}
