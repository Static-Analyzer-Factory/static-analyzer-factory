//! Summary-based constraint generation for PTA.
//!
//! Generates PTA constraints from [`FunctionSummary`] at call sites, replacing
//! the spec-based extraction with a unified summary-driven approach. Summaries
//! can originate from YAML specs, analysis computation, or derived overlays.

use saf_core::id::make_id;
use saf_core::ids::{ObjId, ValueId};
use saf_core::summary::{AccessPath, FunctionSummary};

use super::constraint::{
    AddrConstraint, ConstraintSet, CopyConstraint, GepConstraint, LoadConstraint, StoreConstraint,
};
use super::location::{FieldPath, LocationFactory};

/// Generate PTA constraints from a [`FunctionSummary`] at a specific call site.
///
/// This is the summary-driven replacement for spec-based constraint extraction.
/// Given a summary (from YAML specs, analysis computation, or derived overlay)
/// and the concrete call-site arguments, it maps each summary effect to the
/// appropriate PTA constraint.
///
/// # Effect mapping
///
/// - [`ReturnEffect`](saf_core::summary::ReturnEffect) with `aliases` -> [`CopyConstraint`]
///   from the resolved access path to the return value
/// - [`ReturnEffect`](saf_core::summary::ReturnEffect) with `fresh_allocation` -> [`AddrConstraint`]
///   for a fresh location assigned to the return value
/// - [`AllocationEffect`](saf_core::summary::AllocationEffect) -> [`AddrConstraint`]
///   at the resolved target
/// - [`MemoryEffect`](saf_core::summary::MemoryEffect) with `writes` -> [`StoreConstraint`]
/// - [`MemoryEffect`](saf_core::summary::MemoryEffect) with `reads` -> [`LoadConstraint`]
/// - Field access in [`AccessPath`] -> [`GepConstraint`]
///
/// # Access path resolution
///
/// - `Param(i)` -> `call_args[i]`
/// - `Return` -> `return_value`
/// - `Global(vid)` -> `vid` directly
/// - `Deref(inner)` -> generates a [`LoadConstraint`] from the resolved inner
/// - `Field(inner, idx)` -> generates a [`GepConstraint`] from the resolved inner
pub fn instantiate_summary_at_callsite(
    summary: &FunctionSummary,
    call_args: &[ValueId],
    return_value: Option<ValueId>,
    factory: &mut LocationFactory,
    constraints: &mut ConstraintSet,
) {
    // A counter to generate deterministic synthetic ValueIds for intermediate
    // temporaries (e.g., load results from Deref paths).
    let mut synth_counter: u64 = 0;
    let fid_raw = summary.function_id.raw();

    let mut make_synthetic = |tag: &str| -> ValueId {
        let seed = [
            fid_raw.to_le_bytes().as_slice(),
            synth_counter.to_le_bytes().as_slice(),
        ]
        .concat();
        synth_counter += 1;
        ValueId::new(make_id(tag, &seed))
    };

    // --- Return effects ---
    for effect in &summary.return_effects {
        let Some(ret) = return_value else {
            continue;
        };

        // Fresh allocation: return value points to a new abstract object
        if effect.fresh_allocation {
            let obj = ObjId::new(make_id("summary_alloc", &fid_raw.to_le_bytes()));
            let loc = factory.get_or_create(obj, FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr: ret, loc });
        }

        // Aliasing: return value copies from the resolved access path
        if let Some(ref path) = effect.aliases {
            if let Some(src) = resolve_access_path(
                path,
                call_args,
                return_value,
                &mut make_synthetic,
                constraints,
            ) {
                constraints.copy.insert(CopyConstraint { dst: ret, src });
            }
        }
    }

    // --- Allocation effects ---
    for (idx, effect) in summary.allocation_effects.iter().enumerate() {
        if let Some(target) = resolve_access_path(
            &effect.target,
            call_args,
            return_value,
            &mut make_synthetic,
            constraints,
        ) {
            let seed = [
                fid_raw.to_le_bytes().as_slice(),
                (idx as u64).to_le_bytes().as_slice(),
            ]
            .concat();
            let obj = ObjId::new(make_id("summary_alloc_effect", &seed));
            let loc = factory.get_or_create(obj, FieldPath::empty());
            constraints.addr.insert(AddrConstraint { ptr: target, loc });
        }
    }

    // --- Memory effects ---
    for effect in &summary.memory_effects {
        if let Some(resolved) = resolve_access_path(
            &effect.path,
            call_args,
            return_value,
            &mut make_synthetic,
            constraints,
        ) {
            if effect.writes {
                // Write through the pointer: *resolved = <synthetic>
                // We create a synthetic value representing the unknown written data.
                let synth_val = make_synthetic("summary_write_val");
                let synth_obj = ObjId::new(synth_val.raw());
                let loc = factory.get_or_create(synth_obj, FieldPath::empty());
                constraints.addr.insert(AddrConstraint {
                    ptr: synth_val,
                    loc,
                });
                constraints.store.insert(StoreConstraint {
                    dst_ptr: resolved,
                    src: synth_val,
                });
            }
            if effect.reads {
                // Read through the pointer: <synthetic> = *resolved
                let synth_dst = make_synthetic("summary_read_val");
                constraints.load.insert(LoadConstraint {
                    dst: synth_dst,
                    src_ptr: resolved,
                });
            }
        }
    }
}

/// Resolve an [`AccessPath`] to a concrete [`ValueId`] at a call site.
///
/// For compound paths (`Deref`, `Field`), this generates intermediate
/// constraints (Load/GEP) and returns the synthetic `ValueId` representing
/// the resolved value.
///
/// Returns `None` if the path cannot be resolved (e.g., `Param(i)` where
/// `i >= call_args.len()`, or `Return` with no return value).
fn resolve_access_path(
    path: &AccessPath,
    call_args: &[ValueId],
    return_value: Option<ValueId>,
    make_synthetic: &mut impl FnMut(&str) -> ValueId,
    constraints: &mut ConstraintSet,
) -> Option<ValueId> {
    match path {
        AccessPath::Param(i) => call_args.get(*i as usize).copied(),
        AccessPath::Return => return_value,
        AccessPath::Global(vid) => Some(*vid),
        AccessPath::Deref(inner) => {
            // Resolve the inner path, then load through the resulting pointer
            let inner_val =
                resolve_access_path(inner, call_args, return_value, make_synthetic, constraints)?;
            let synth_dst = make_synthetic("summary_deref");
            constraints.load.insert(LoadConstraint {
                dst: synth_dst,
                src_ptr: inner_val,
            });
            Some(synth_dst)
        }
        AccessPath::Field(inner, field_idx) => {
            // Resolve the inner path, then apply GEP for the field offset
            let inner_val =
                resolve_access_path(inner, call_args, return_value, make_synthetic, constraints)?;
            let synth_dst = make_synthetic("summary_field");
            constraints.gep.insert(GepConstraint {
                dst: synth_dst,
                src_ptr: inner_val,
                path: FieldPath::field(*field_idx),
                index_operands: vec![],
            });
            Some(synth_dst)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::{FunctionId, ValueId};
    use saf_core::summary::{
        AccessPath, AllocationEffect, FunctionSummary, MemoryEffect, ReturnEffect,
    };

    use crate::pta::config::FieldSensitivity;
    use crate::pta::location::LocationFactory;

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    #[test]
    fn returns_param_generates_copy_constraint() {
        // Summary: function returns param.0 (like strcpy)
        let fid = FunctionId::derive(b"returns_param0");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Param(0)),
            fresh_allocation: false,
        });

        let arg0 = ValueId::derive(b"arg0");
        let ret = ValueId::derive(b"retval");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0],
            Some(ret),
            &mut factory,
            &mut constraints,
        );

        // Should generate: ret = arg0 (Copy)
        let has_copy = constraints
            .copy
            .iter()
            .any(|c| c.dst == ret && c.src == arg0);
        assert!(
            has_copy,
            "Expected Copy(ret <- arg0), got: {:?}",
            constraints.copy
        );
    }

    #[test]
    fn fresh_allocation_generates_addr_constraint() {
        // Summary: function returns a fresh heap allocation (like malloc)
        let fid = FunctionId::derive(b"allocator");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: None,
            fresh_allocation: true,
        });

        let ret = ValueId::derive(b"malloc_ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(&summary, &[], Some(ret), &mut factory, &mut constraints);

        // Should generate: ret -> fresh_loc (Addr)
        assert_eq!(
            constraints.addr.len(),
            1,
            "Expected 1 Addr constraint for allocation"
        );
        let addr = constraints.addr.iter().next().expect("addr constraint");
        assert_eq!(addr.ptr, ret);
    }

    #[test]
    fn allocation_effect_generates_addr_at_target() {
        // Summary: function allocates to its return value
        let fid = FunctionId::derive(b"alloc_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.allocation_effects.push(AllocationEffect {
            target: AccessPath::Return,
            heap: true,
        });

        let ret = ValueId::derive(b"alloc_ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(&summary, &[], Some(ret), &mut factory, &mut constraints);

        // Should generate: ret -> fresh_loc (Addr)
        assert_eq!(
            constraints.addr.len(),
            1,
            "Expected 1 Addr for AllocationEffect"
        );
        let addr = constraints.addr.iter().next().expect("addr constraint");
        assert_eq!(addr.ptr, ret);
    }

    #[test]
    fn field_access_generates_gep_constraint() {
        // Summary: function returns param.0->field(4)
        let fid = FunctionId::derive(b"field_accessor");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Field(Box::new(AccessPath::Param(0)), 4)),
            fresh_allocation: false,
        });

        let arg0 = ValueId::derive(b"struct_ptr");
        let ret = ValueId::derive(b"field_ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0],
            Some(ret),
            &mut factory,
            &mut constraints,
        );

        // Should generate a GEP constraint for the field access
        assert_eq!(constraints.gep.len(), 1, "Expected 1 GEP for field access");
        let gep = constraints.gep.iter().next().expect("gep constraint");
        assert_eq!(gep.src_ptr, arg0);
        assert_eq!(gep.path, FieldPath::field(4));

        // And a Copy from the GEP result to ret
        assert_eq!(
            constraints.copy.len(),
            1,
            "Expected 1 Copy for return alias"
        );
        let copy = constraints.copy.iter().next().expect("copy constraint");
        assert_eq!(copy.dst, ret);
        assert_eq!(copy.src, gep.dst);
    }

    #[test]
    fn deref_generates_load_constraint() {
        // Summary: function returns *param.0 (deref of first param)
        let fid = FunctionId::derive(b"deref_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Deref(Box::new(AccessPath::Param(0)))),
            fresh_allocation: false,
        });

        let arg0 = ValueId::derive(b"ptr_arg");
        let ret = ValueId::derive(b"deref_ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0],
            Some(ret),
            &mut factory,
            &mut constraints,
        );

        // Should generate a Load for the deref
        assert_eq!(constraints.load.len(), 1, "Expected 1 Load for deref");
        let load = constraints.load.iter().next().expect("load constraint");
        assert_eq!(load.src_ptr, arg0);

        // And a Copy from load result to ret
        assert_eq!(constraints.copy.len(), 1);
        let copy = constraints.copy.iter().next().expect("copy constraint");
        assert_eq!(copy.dst, ret);
        assert_eq!(copy.src, load.dst);
    }

    #[test]
    fn memory_write_generates_store_constraint() {
        // Summary: function writes to *param.0
        let fid = FunctionId::derive(b"write_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.memory_effects.push(MemoryEffect {
            path: AccessPath::Param(0),
            reads: false,
            writes: true,
        });

        let arg0 = ValueId::derive(b"write_target");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(&summary, &[arg0], None, &mut factory, &mut constraints);

        // Should generate: *arg0 = synthetic (Store + Addr for synthetic)
        assert_eq!(
            constraints.store.len(),
            1,
            "Expected 1 Store for memory write"
        );
        let store = constraints.store.iter().next().expect("store");
        assert_eq!(store.dst_ptr, arg0);

        // The synthetic value should have an Addr constraint
        assert_eq!(
            constraints.addr.len(),
            1,
            "Expected 1 Addr for synthetic write value"
        );
    }

    #[test]
    fn memory_read_generates_load_constraint() {
        // Summary: function reads from param.0
        let fid = FunctionId::derive(b"read_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.memory_effects.push(MemoryEffect {
            path: AccessPath::Param(0),
            reads: true,
            writes: false,
        });

        let arg0 = ValueId::derive(b"read_source");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(&summary, &[arg0], None, &mut factory, &mut constraints);

        // Should generate: synthetic = *arg0 (Load)
        assert_eq!(constraints.load.len(), 1, "Expected 1 Load for memory read");
        let load = constraints.load.iter().next().expect("load");
        assert_eq!(load.src_ptr, arg0);
    }

    #[test]
    fn no_return_value_skips_return_effects() {
        // Summary has return effect but call has no return value
        let fid = FunctionId::derive(b"void_call");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Param(0)),
            fresh_allocation: true,
        });

        let arg0 = ValueId::derive(b"arg");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0],
            None, // no return value
            &mut factory,
            &mut constraints,
        );

        // No constraints should be generated
        assert!(
            constraints.is_empty(),
            "No constraints when return value is None"
        );
    }

    #[test]
    fn out_of_bounds_param_is_skipped() {
        // Summary references param.5 but only 2 args passed
        let fid = FunctionId::derive(b"oob_param");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Param(5)),
            fresh_allocation: false,
        });

        let arg0 = ValueId::derive(b"a0");
        let arg1 = ValueId::derive(b"a1");
        let ret = ValueId::derive(b"ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0, arg1],
            Some(ret),
            &mut factory,
            &mut constraints,
        );

        // No copy generated (param index out of bounds)
        assert!(
            constraints.copy.is_empty(),
            "Should skip unresolvable param index"
        );
    }

    #[test]
    fn global_access_path_resolves_directly() {
        // Summary: returns a global value
        let global_vid = ValueId::derive(b"my_global");
        let fid = FunctionId::derive(b"global_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: Some(AccessPath::Global(global_vid)),
            fresh_allocation: false,
        });

        let ret = ValueId::derive(b"ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(&summary, &[], Some(ret), &mut factory, &mut constraints);

        // Should generate: ret = global_vid (Copy)
        let has_copy = constraints
            .copy
            .iter()
            .any(|c| c.dst == ret && c.src == global_vid);
        assert!(has_copy, "Expected Copy(ret <- global)");
    }

    #[test]
    fn combined_return_and_allocation_effects() {
        // Summary: malloc-like function (fresh allocation + alloc effect)
        let fid = FunctionId::derive(b"combined_fn");
        let mut summary = FunctionSummary::default_for(fid);
        summary.return_effects.push(ReturnEffect {
            aliases: None,
            fresh_allocation: true,
        });
        summary.allocation_effects.push(AllocationEffect {
            target: AccessPath::Param(0),
            heap: true,
        });

        let arg0 = ValueId::derive(b"out_ptr");
        let ret = ValueId::derive(b"combined_ret");

        let mut factory = make_factory();
        let mut constraints = ConstraintSet::default();

        instantiate_summary_at_callsite(
            &summary,
            &[arg0],
            Some(ret),
            &mut factory,
            &mut constraints,
        );

        // 1 Addr for return fresh alloc + 1 Addr for allocation effect at param.0
        assert_eq!(
            constraints.addr.len(),
            2,
            "Expected 2 Addr constraints (return + alloc effect)"
        );
    }
}
