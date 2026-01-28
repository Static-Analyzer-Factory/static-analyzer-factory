//! Struct layout computation for AIR types.
//!
//! Reimplements LLVM's `StructLayout` algorithm for computing field byte
//! offsets, sizes, and alignment. Uses System V AMD64 ABI defaults which
//! cover ~99% of x86-64 C/C++ code.

use std::collections::BTreeMap;

use crate::air::{AirType, StructField};
use crate::ids::TypeId;

/// Round `value` up to the next multiple of `align`.
///
/// `align` must be a power of two.
const fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

/// Returns the ABI alignment in bytes for a type under System V AMD64 rules.
///
/// Uses a default pointer width of 8 bytes (64-bit). For target-aware layout,
/// use [`abi_alignment_with_ptr`].
///
/// Returns `None` for `Opaque` types or types that reference unknown `TypeId`s.
///
/// # Alignment rules
///
/// - `Pointer` / `Reference` -> 8
/// - `Integer { bits }` -> `min(ceil(bits/8), 16).next_power_of_two()` (at least 1)
/// - `Float { bits: 32 }` -> 4
/// - `Float { bits: 64 }` -> 8
/// - `Float { other }` -> 8 (default for unusual float widths)
/// - `Vector { element, lanes }` -> `min(total_bytes.next_power_of_two(), 64)`
/// - `Array { element, .. }` -> alignment of element type
/// - `Struct { fields, .. }` -> max of all field alignments
/// - `Void` -> 1
/// - `Function { .. }` -> 1 (function types have no meaningful alignment)
/// - `Opaque` -> `None`
pub fn abi_alignment(ty: &AirType, types: &BTreeMap<TypeId, AirType>) -> Option<u64> {
    abi_alignment_with_ptr(ty, types, 8)
}

/// Returns the ABI alignment in bytes for a type, using the specified pointer width.
///
/// `ptr_width` is the pointer size in bytes (4 for 32-bit, 8 for 64-bit).
///
/// Returns `None` for `Opaque` types or types that reference unknown `TypeId`s.
pub fn abi_alignment_with_ptr(
    ty: &AirType,
    types: &BTreeMap<TypeId, AirType>,
    ptr_width: u32,
) -> Option<u64> {
    match ty {
        AirType::Pointer | AirType::Reference { .. } => Some(u64::from(ptr_width)),
        AirType::Integer { bits } => {
            // ceil(bits / 8), clamped to [1, 16], then rounded up to power of two
            let bytes = u64::from(*bits).div_ceil(8).clamp(1, 16);
            Some(bytes.next_power_of_two())
        }
        AirType::Float { bits } => match bits {
            32 => Some(4),
            // 64-bit and all unusual float widths default to 8-byte alignment
            _ => Some(8),
        },
        AirType::Vector { element, lanes } => {
            let elem_ty = types.get(element)?;
            let elem_size = alloc_size_with_ptr(elem_ty, types, ptr_width)?;
            let total = elem_size * u64::from(*lanes);
            Some(total.next_power_of_two().min(64)) // SIMD alignment capped at 64
        }
        AirType::Array { element, .. } => {
            let elem_ty = types.get(element)?;
            abi_alignment_with_ptr(elem_ty, types, ptr_width)
        }
        AirType::Struct { fields, .. } => {
            let mut max_align: u64 = 1;
            for field in fields {
                let field_ty = types.get(&field.field_type)?;
                let align = abi_alignment_with_ptr(field_ty, types, ptr_width)?;
                max_align = max_align.max(align);
            }
            Some(max_align)
        }
        AirType::Void | AirType::Function { .. } => Some(1),
        AirType::Opaque => None,
    }
}

/// Returns the allocation size in bytes for a type.
///
/// Uses a default pointer width of 8 bytes (64-bit). For target-aware layout,
/// use [`alloc_size_with_ptr`].
///
/// Returns `None` for `Opaque` types, variable-length arrays, or types that
/// reference unknown `TypeId`s.
///
/// # Size rules
///
/// - `Pointer` / `Reference` -> 8
/// - `Integer { bits }` -> `ceil(bits / 8)`
/// - `Float { bits }` -> `bits / 8` (minimum 4)
/// - `Vector { element, lanes }` -> `alloc_size(element) * lanes`
/// - `Array { element, count: Some(n) }` -> `n * alloc_size(element)`
/// - `Array { element, count: None }` -> `None` (VLA)
/// - `Struct { total_size, .. }` -> `total_size`
/// - `Void` -> 0
/// - `Function { .. }` -> 0 (function types have no allocation size)
/// - `Opaque` -> `None`
pub fn alloc_size(ty: &AirType, types: &BTreeMap<TypeId, AirType>) -> Option<u64> {
    alloc_size_with_ptr(ty, types, 8)
}

/// Returns the allocation size in bytes for a type, using the specified pointer width.
///
/// `ptr_width` is the pointer size in bytes (4 for 32-bit, 8 for 64-bit).
///
/// Returns `None` for `Opaque` types, variable-length arrays, or types that
/// reference unknown `TypeId`s.
pub fn alloc_size_with_ptr(
    ty: &AirType,
    types: &BTreeMap<TypeId, AirType>,
    ptr_width: u32,
) -> Option<u64> {
    match ty {
        AirType::Pointer | AirType::Reference { .. } => Some(u64::from(ptr_width)),
        AirType::Integer { bits } => Some(u64::from(*bits).div_ceil(8)),
        AirType::Float { bits } => {
            let bytes = u64::from(*bits) / 8;
            Some(bytes.max(4))
        }
        AirType::Vector { element, lanes } => {
            let elem_ty = types.get(element)?;
            let elem_size = alloc_size_with_ptr(elem_ty, types, ptr_width)?;
            Some(elem_size * u64::from(*lanes))
        }
        AirType::Array { element, count } => {
            let n = (*count)?;
            let elem_ty = types.get(element)?;
            let elem_size = alloc_size_with_ptr(elem_ty, types, ptr_width)?;
            Some(n * elem_size)
        }
        AirType::Struct { total_size, .. } => Some(*total_size),
        AirType::Void | AirType::Function { .. } => Some(0),
        AirType::Opaque => None,
    }
}

/// Computes struct layout: field byte offsets and total size (including tail padding).
///
/// Uses a default pointer width of 8 bytes (64-bit). For target-aware layout,
/// use [`compute_struct_layout_with_ptr`].
///
/// Returns `(field_offsets, total_size)` or `None` if any field has an unknown type.
pub fn compute_struct_layout(
    fields: &[StructField],
    types: &BTreeMap<TypeId, AirType>,
) -> Option<(Vec<u64>, u64)> {
    compute_struct_layout_with_ptr(fields, types, 8)
}

/// Computes struct layout: field byte offsets and total size (including tail padding),
/// using the specified pointer width.
///
/// `ptr_width` is the pointer size in bytes (4 for 32-bit, 8 for 64-bit).
///
/// Returns `(field_offsets, total_size)` or `None` if any field has an unknown type.
///
/// Algorithm adapted from LLVM's `StructLayout` constructor in `DataLayout.cpp`:
/// 1. For each field, align the current offset to the field's ABI alignment.
/// 2. Record the aligned offset, then advance by the field's allocation size.
/// 3. After all fields, apply tail padding by aligning to the struct's overall alignment.
pub fn compute_struct_layout_with_ptr(
    fields: &[StructField],
    types: &BTreeMap<TypeId, AirType>,
    ptr_width: u32,
) -> Option<(Vec<u64>, u64)> {
    let mut offset: u64 = 0;
    let mut max_align: u64 = 1;
    let mut offsets = Vec::with_capacity(fields.len());

    for field in fields {
        let field_ty = types.get(&field.field_type)?;
        let align = abi_alignment_with_ptr(field_ty, types, ptr_width)?;
        let size = alloc_size_with_ptr(field_ty, types, ptr_width)?;

        offset = align_up(offset, align);
        offsets.push(offset);
        offset += size;
        max_align = max_align.max(align);
    }

    // Tail padding: align total size to the struct's overall alignment
    offset = align_up(offset, max_align);

    Some((offsets, offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::TypeId;

    /// Helper to build a type table with common primitive types for testing.
    fn make_types() -> BTreeMap<TypeId, AirType> {
        let mut types = BTreeMap::new();
        types.insert(TypeId::derive(b"ptr"), AirType::Pointer);
        types.insert(TypeId::derive(b"i8"), AirType::Integer { bits: 8 });
        types.insert(TypeId::derive(b"i16"), AirType::Integer { bits: 16 });
        types.insert(TypeId::derive(b"i32"), AirType::Integer { bits: 32 });
        types.insert(TypeId::derive(b"i64"), AirType::Integer { bits: 64 });
        types.insert(TypeId::derive(b"i128"), AirType::Integer { bits: 128 });
        types.insert(TypeId::derive(b"f32"), AirType::Float { bits: 32 });
        types.insert(TypeId::derive(b"f64"), AirType::Float { bits: 64 });
        types.insert(TypeId::derive(b"void"), AirType::Void);
        types
    }

    fn field(name: &[u8]) -> StructField {
        StructField {
            field_type: TypeId::derive(name),
            byte_offset: None,
            byte_size: None,
            name: None,
        }
    }

    // --- abi_alignment tests ---

    #[test]
    fn alignment_pointer() {
        let types = make_types();
        assert_eq!(abi_alignment(&AirType::Pointer, &types), Some(8));
    }

    #[test]
    fn alignment_integers() {
        let types = make_types();
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 1 }, &types),
            Some(1)
        );
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 8 }, &types),
            Some(1)
        );
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 16 }, &types),
            Some(2)
        );
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 32 }, &types),
            Some(4)
        );
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 64 }, &types),
            Some(8)
        );
        assert_eq!(
            abi_alignment(&AirType::Integer { bits: 128 }, &types),
            Some(16)
        );
    }

    #[test]
    fn alignment_floats() {
        let types = make_types();
        assert_eq!(abi_alignment(&AirType::Float { bits: 32 }, &types), Some(4));
        assert_eq!(abi_alignment(&AirType::Float { bits: 64 }, &types), Some(8));
    }

    #[test]
    fn alignment_void() {
        let types = make_types();
        assert_eq!(abi_alignment(&AirType::Void, &types), Some(1));
    }

    #[test]
    fn alignment_opaque_returns_none() {
        let types = make_types();
        assert_eq!(abi_alignment(&AirType::Opaque, &types), None);
    }

    // --- alloc_size tests ---

    #[test]
    fn size_pointer() {
        let types = make_types();
        assert_eq!(alloc_size(&AirType::Pointer, &types), Some(8));
    }

    #[test]
    fn size_integers() {
        let types = make_types();
        assert_eq!(alloc_size(&AirType::Integer { bits: 1 }, &types), Some(1));
        assert_eq!(alloc_size(&AirType::Integer { bits: 8 }, &types), Some(1));
        assert_eq!(alloc_size(&AirType::Integer { bits: 16 }, &types), Some(2));
        assert_eq!(alloc_size(&AirType::Integer { bits: 32 }, &types), Some(4));
        assert_eq!(alloc_size(&AirType::Integer { bits: 64 }, &types), Some(8));
        assert_eq!(
            alloc_size(&AirType::Integer { bits: 128 }, &types),
            Some(16)
        );
    }

    #[test]
    fn size_void() {
        let types = make_types();
        assert_eq!(alloc_size(&AirType::Void, &types), Some(0));
    }

    // --- compute_struct_layout tests ---

    #[test]
    fn layout_simple_i32_ptr() {
        // struct { i32, ptr } -> offsets [0, 8], total 16
        // i32 at offset 0 (4 bytes), 4 bytes padding, ptr at offset 8
        let types = make_types();
        let fields = vec![field(b"i32"), field(b"ptr")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 8]);
        assert_eq!(total, 16);
    }

    #[test]
    fn layout_three_i8() {
        // struct { i8, i8, i8 } -> offsets [0, 1, 2], total 3
        let types = make_types();
        let fields = vec![field(b"i8"), field(b"i8"), field(b"i8")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 1, 2]);
        assert_eq!(total, 3);
    }

    #[test]
    fn layout_i8_i32_padding() {
        // struct { i8, i32 } -> offsets [0, 4], total 8
        // i8 at 0, 3 bytes padding, i32 at 4, 0 bytes tail padding (align=4, 8 is aligned)
        let types = make_types();
        let fields = vec![field(b"i8"), field(b"i32")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 4]);
        assert_eq!(total, 8);
    }

    #[test]
    fn layout_i64_i8_tail_padding() {
        // struct { i64, i8 } -> offsets [0, 8], total 16
        // i64 at 0, i8 at 8, 7 bytes tail padding (align to 8)
        let types = make_types();
        let fields = vec![field(b"i64"), field(b"i8")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 8]);
        assert_eq!(total, 16);
    }

    #[test]
    fn layout_single_ptr() {
        // struct { ptr } -> offsets [0], total 8
        let types = make_types();
        let fields = vec![field(b"ptr")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0]);
        assert_eq!(total, 8);
    }

    #[test]
    fn layout_empty_struct() {
        // struct {} -> offsets [], total 0
        let types = make_types();
        let fields = vec![];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert!(offsets.is_empty());
        assert_eq!(total, 0);
    }

    #[test]
    fn layout_with_opaque_field_returns_none() {
        // If a field has unknown type, layout can't be computed
        let types = make_types();
        let opaque_field = StructField {
            field_type: TypeId::derive(b"unknown"),
            byte_offset: None,
            byte_size: None,
            name: None,
        };
        let fields = vec![field(b"i32"), opaque_field];
        assert!(compute_struct_layout(&fields, &types).is_none());
    }

    #[test]
    fn layout_nested_array() {
        // struct { [10 x i32], ptr } -> offsets [0, 40], total 48
        let mut types = make_types();
        let arr_id = TypeId::derive(b"[10 x i32]");
        types.insert(
            arr_id,
            AirType::Array {
                element: TypeId::derive(b"i32"),
                count: Some(10),
            },
        );
        let arr_field = StructField {
            field_type: arr_id,
            byte_offset: None,
            byte_size: None,
            name: None,
        };
        let fields = vec![arr_field, field(b"ptr")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 40]);
        assert_eq!(total, 48);
    }

    #[test]
    fn layout_i16_i64_i8() {
        // struct { i16, i64, i8 } -> offsets [0, 8, 16], total 24
        // i16 at 0 (2 bytes), 6 bytes padding, i64 at 8 (8 bytes), i8 at 16, 7 bytes tail padding
        let types = make_types();
        let fields = vec![field(b"i16"), field(b"i64"), field(b"i8")];
        let (offsets, total) = compute_struct_layout(&fields, &types).unwrap();
        assert_eq!(offsets, vec![0, 8, 16]);
        assert_eq!(total, 24);
    }

    #[test]
    fn alignment_reference() {
        let types = make_types();
        assert_eq!(
            abi_alignment(&AirType::Reference { nullable: false }, &types),
            Some(8)
        );
        assert_eq!(
            abi_alignment(&AirType::Reference { nullable: true }, &types),
            Some(8)
        );
    }

    #[test]
    fn size_reference() {
        let types = make_types();
        assert_eq!(
            alloc_size(&AirType::Reference { nullable: false }, &types),
            Some(8)
        );
        assert_eq!(
            alloc_size(&AirType::Reference { nullable: true }, &types),
            Some(8)
        );
    }

    #[test]
    fn alignment_and_size_vector() {
        let mut types = make_types();
        let f32_id = TypeId::derive(b"f32");
        types.insert(f32_id, AirType::Float { bits: 32 });
        // 4 x f32 = 16 bytes, alignment = 16
        let vec_ty = AirType::Vector {
            element: f32_id,
            lanes: 4,
        };
        assert_eq!(alloc_size(&vec_ty, &types), Some(16));
        assert_eq!(abi_alignment(&vec_ty, &types), Some(16));
    }

    // --- _with_ptr tests for 32-bit pointer width ---

    #[test]
    fn layout_32bit_pointer() {
        let types = make_types();
        assert_eq!(alloc_size_with_ptr(&AirType::Pointer, &types, 4), Some(4));
        assert_eq!(
            abi_alignment_with_ptr(&AirType::Pointer, &types, 4),
            Some(4)
        );
    }

    #[test]
    fn layout_32bit_reference() {
        let types = make_types();
        assert_eq!(
            alloc_size_with_ptr(&AirType::Reference { nullable: false }, &types, 4),
            Some(4)
        );
        assert_eq!(
            abi_alignment_with_ptr(&AirType::Reference { nullable: true }, &types, 4),
            Some(4)
        );
    }

    #[test]
    fn layout_32bit_struct_with_pointer() {
        // struct { i32, ptr } on 32-bit -> offsets [0, 4], total 8
        let types = make_types();
        let fields = vec![field(b"i32"), field(b"ptr")];
        let (offsets, total) = compute_struct_layout_with_ptr(&fields, &types, 4).unwrap();
        assert_eq!(offsets, vec![0, 4]);
        assert_eq!(total, 8);
    }

    #[test]
    fn layout_64bit_unchanged() {
        // Explicit ptr_width=8 should match the default wrapper
        let types = make_types();
        assert_eq!(
            alloc_size_with_ptr(&AirType::Pointer, &types, 8),
            alloc_size(&AirType::Pointer, &types)
        );
        assert_eq!(
            abi_alignment_with_ptr(&AirType::Pointer, &types, 8),
            abi_alignment(&AirType::Pointer, &types)
        );
    }
}
