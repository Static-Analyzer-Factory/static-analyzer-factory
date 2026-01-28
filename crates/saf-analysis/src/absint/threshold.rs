//! Widening threshold extraction from program constants.
//!
//! Scans AIR modules for integer constants in branch conditions,
//! allocation sizes, and loop bounds. These serve as widening thresholds
//! to improve precision (Astrée/IKOS technique).

use std::collections::BTreeSet;

use saf_core::air::{AirModule, BinaryOp, Constant, Operation};

/// Extract widening thresholds from program constants.
///
/// Collects:
/// - Constants from `ICmp` comparison operands (branch conditions)
/// - Constants from allocation function arguments
/// - Standard type boundary values
/// - Constants ± 1 (for off-by-one patterns)
#[must_use]
pub fn extract_thresholds(module: &AirModule) -> BTreeSet<i128> {
    let mut thresholds = BTreeSet::new();

    // Standard boundary constants
    add_standard_thresholds(&mut thresholds);

    // Build a quick ValueId → constant value lookup from the module
    let int_constants: std::collections::BTreeMap<_, _> = module
        .constants
        .iter()
        .filter_map(|(vid, c)| match c {
            Constant::Int { value, .. } => Some((*vid, i128::from(*value))),
            Constant::BigInt { value, bits: _ } => value.parse::<i128>().ok().map(|v| (*vid, v)),
            _ => None,
        })
        .collect();

    // Scan module for program-specific constants
    for func in &module.functions {
        if func.is_declaration {
            continue;
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                // Extract thresholds from comparison operands
                if let Operation::BinaryOp { kind } = &inst.op {
                    if is_comparison(*kind) {
                        for operand in &inst.operands {
                            if let Some(&v) = int_constants.get(operand) {
                                add_threshold_with_neighbors(&mut thresholds, v);
                            }
                        }
                    }
                }

                // Extract thresholds from allocation sizes so that
                // buffer bounds are captured as widening thresholds.
                if let Operation::Alloca {
                    size_bytes: Some(size),
                } = &inst.op
                {
                    add_threshold_with_neighbors(&mut thresholds, i128::from(*size));
                }

                // Extract thresholds from phi incoming constants.
                // After mem2reg, phi nodes directly reference constant values
                // (e.g., `phi [5, %entry], [%x.inc, %body]`). The constant `5`
                // should become a widening threshold so the ascending phase
                // stops at the branch boundary instead of jumping to ±∞.
                if let Operation::Phi { incoming } = &inst.op {
                    for (_block_id, val_id) in incoming {
                        if let Some(&v) = int_constants.get(val_id) {
                            add_threshold_with_neighbors(&mut thresholds, v);
                        }
                    }
                }
            }
        }
    }

    // Scan globals for constant initializers
    for global in &module.globals {
        if let Some(Constant::Int { value, .. }) = &global.init {
            let v = i128::from(*value);
            add_threshold_with_neighbors(&mut thresholds, v);
        }
    }

    thresholds
}

/// Add standard type boundary thresholds.
fn add_standard_thresholds(thresholds: &mut BTreeSet<i128>) {
    // Zero and neighbors
    thresholds.insert(-1);
    thresholds.insert(0);
    thresholds.insert(1);

    // i8 boundaries
    thresholds.insert(i128::from(i8::MIN));
    thresholds.insert(i128::from(i8::MAX));
    thresholds.insert(i128::from(u8::MAX));

    // i16 boundaries
    thresholds.insert(i128::from(i16::MIN));
    thresholds.insert(i128::from(i16::MAX));
    thresholds.insert(i128::from(u16::MAX));

    // i32 boundaries
    thresholds.insert(i128::from(i32::MIN));
    thresholds.insert(i128::from(i32::MAX));
    thresholds.insert(i128::from(u32::MAX));

    // i64 boundaries
    thresholds.insert(i128::from(i64::MIN));
    thresholds.insert(i128::from(i64::MAX));

    // Common buffer/array sizes
    for &size in &[
        2, 4, 8, 10, 16, 32, 64, 100, 128, 255, 256, 512, 1000, 1024, 2048, 4096, 8192, 16384,
        32768, 65536,
    ] {
        add_threshold_with_neighbors(thresholds, size);
    }
}

/// Add a threshold and its ±1 neighbors.
fn add_threshold_with_neighbors(thresholds: &mut BTreeSet<i128>, value: i128) {
    thresholds.insert(value.saturating_sub(1));
    thresholds.insert(value);
    thresholds.insert(value.saturating_add(1));
}

/// Check if a `BinaryOp` is a comparison operation.
fn is_comparison(kind: BinaryOp) -> bool {
    matches!(
        kind,
        BinaryOp::ICmpSlt
            | BinaryOp::ICmpSle
            | BinaryOp::ICmpSgt
            | BinaryOp::ICmpSge
            | BinaryOp::ICmpEq
            | BinaryOp::ICmpNe
            | BinaryOp::ICmpUlt
            | BinaryOp::ICmpUle
            | BinaryOp::ICmpUgt
            | BinaryOp::ICmpUge
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::ids::ModuleId;

    #[test]
    fn standard_thresholds_present() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        let thresholds = extract_thresholds(&module);

        assert!(thresholds.contains(&0));
        assert!(thresholds.contains(&1));
        assert!(thresholds.contains(&-1));
        assert!(thresholds.contains(&i128::from(i32::MAX)));
        assert!(thresholds.contains(&i128::from(i32::MIN)));
        assert!(thresholds.contains(&256));
        assert!(thresholds.contains(&1024));
    }

    #[test]
    fn neighbors_included() {
        let module = AirModule::new(ModuleId::derive(b"test"));
        let thresholds = extract_thresholds(&module);

        // 256 and its neighbors
        assert!(thresholds.contains(&255));
        assert!(thresholds.contains(&256));
        assert!(thresholds.contains(&257));
    }
}
