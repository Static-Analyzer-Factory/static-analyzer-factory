//! Type interning for LLVM-to-AIR type conversion.
//!
//! Deduplicates `AirType` values into a centralized type table using
//! content-addressed `TypeId` derivation for deterministic IDs.

#![cfg(any(feature = "llvm-17", feature = "llvm-18"))]

use std::collections::BTreeMap;

use rustc_hash::FxHashMap;

use saf_core::air::{AirType, StructField};
use saf_core::ids::TypeId;
use saf_core::layout;

/// Interns `AirType` values into a deduplicated type table.
///
/// Uses content-addressed `TypeId` derivation for determinism.
/// Maintains a reverse map for O(1) dedup lookups.
pub(crate) struct TypeInterner {
    /// Forward map: `TypeId` -> `AirType` (becomes `AirModule.types`).
    forward: BTreeMap<TypeId, AirType>,
    /// Reverse map: `AirType` -> `TypeId` (dedup lookup).
    reverse: FxHashMap<AirType, TypeId>,
}

impl TypeInterner {
    /// Create a new, empty `TypeInterner`.
    pub(crate) fn new() -> Self {
        Self {
            forward: BTreeMap::new(),
            reverse: FxHashMap::default(),
        }
    }

    /// Intern a type. If already interned (via reverse map lookup), returns the
    /// existing `TypeId`. Otherwise, derives a `TypeId` from a canonical string
    /// representation and inserts into both maps.
    pub(crate) fn intern(&mut self, ty: AirType) -> TypeId {
        if let Some(&id) = self.reverse.get(&ty) {
            return id;
        }

        let canonical = canonical_string(&ty);
        let id = TypeId::derive(canonical.as_bytes());

        self.forward.insert(id, ty.clone());
        self.reverse.insert(ty, id);
        id
    }

    /// Shorthand for interning `AirType::Pointer`.
    pub(crate) fn intern_pointer(&mut self) -> TypeId {
        self.intern(AirType::Pointer)
    }

    /// Shorthand for interning `AirType::Integer` with given bit width.
    pub(crate) fn intern_integer(&mut self, bits: u16) -> TypeId {
        self.intern(AirType::Integer { bits })
    }

    /// Shorthand for interning `AirType::Float` with given bit width.
    pub(crate) fn intern_float(&mut self, bits: u16) -> TypeId {
        self.intern(AirType::Float { bits })
    }

    /// Shorthand for interning `AirType::Void`.
    pub(crate) fn intern_void(&mut self) -> TypeId {
        self.intern(AirType::Void)
    }

    /// Shorthand for interning `AirType::Opaque`.
    pub(crate) fn intern_opaque(&mut self) -> TypeId {
        self.intern(AirType::Opaque)
    }

    /// Consume the interner and return the forward type table.
    ///
    /// The returned `BTreeMap` is suitable for assigning to `AirModule.types`.
    pub(crate) fn into_table(self) -> BTreeMap<TypeId, AirType> {
        self.forward
    }

    /// Parse an LLVM IR type string and intern the resulting `AirType`.
    ///
    /// Parsing rules:
    /// - `"ptr"` -> `AirType::Pointer`
    /// - `"iN"` (e.g. `"i32"`) -> `AirType::Integer { bits: N }`
    /// - `"float"` -> `AirType::Float { bits: 32 }`
    /// - `"double"` -> `AirType::Float { bits: 64 }`
    /// - `"void"` -> `AirType::Void`
    /// - `"[N x TYPE]"` -> `AirType::Array { element, count }`
    /// - `"{ TYPE, TYPE, ... }"` -> `AirType::Struct` (with layout computation)
    /// - `"<{ TYPE, TYPE, ... }>"` -> packed struct (alignment=1 for all fields)
    /// - Anything else -> `AirType::Opaque`
    pub(crate) fn parse_llvm_type_string(&mut self, type_str: &str) -> TypeId {
        let trimmed = type_str.trim();

        // Named type: %struct.Foo = type { i32, ptr }
        // or forward reference: %struct.Foo
        if trimmed.starts_with('%') {
            if let Some(body_start) = trimmed.find("= type ") {
                let body = trimmed[body_start + 7..].trim();
                return self.parse_llvm_type_string(body);
            }
            // Forward reference without definition — treat as Opaque
            return self.intern_opaque();
        }

        // Pointer
        if trimmed == "ptr" {
            return self.intern_pointer();
        }

        // Integer: iN
        if let Some(bits_str) = trimmed.strip_prefix('i') {
            if let Ok(bits) = bits_str.parse::<u16>() {
                return self.intern_integer(bits);
            }
        }

        // Float
        if trimmed == "float" {
            return self.intern_float(32);
        }

        // Double
        if trimmed == "double" {
            return self.intern_float(64);
        }

        // Void
        if trimmed == "void" {
            return self.intern_void();
        }

        // Array: [N x TYPE]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return self.parse_array(trimmed);
        }

        // Packed struct: <{ TYPE, TYPE, ... }>
        if trimmed.starts_with("<{") && trimmed.ends_with("}>") {
            return self.parse_packed_struct(trimmed);
        }

        // Struct: { TYPE, TYPE, ... }
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return self.parse_struct(trimmed);
        }

        // Anything else -> Opaque
        self.intern_opaque()
    }

    /// Parse an LLVM array type string like `[10 x i32]`.
    fn parse_array(&mut self, s: &str) -> TypeId {
        // Strip brackets: "10 x i32"
        let inner = &s[1..s.len() - 1].trim();

        // Find " x " separator
        if let Some(x_pos) = inner.find(" x ") {
            let count_str = inner[..x_pos].trim();
            let elem_str = inner[x_pos + 3..].trim();

            if let Ok(count) = count_str.parse::<u64>() {
                let element = self.parse_llvm_type_string(elem_str);
                let ty = AirType::Array {
                    element,
                    count: Some(count),
                };
                return self.intern(ty);
            }
        }

        // Could not parse -> Opaque
        self.intern_opaque()
    }

    /// Parse an LLVM struct type string like `{ i32, ptr }`.
    fn parse_struct(&mut self, s: &str) -> TypeId {
        // Strip braces: " i32, ptr "
        let inner = &s[1..s.len() - 1];
        self.parse_struct_fields(inner, false)
    }

    /// Parse an LLVM packed struct type string like `<{ i32, ptr }>`.
    fn parse_packed_struct(&mut self, s: &str) -> TypeId {
        // Strip "<{" and "}>"
        let inner = &s[2..s.len() - 2];
        self.parse_struct_fields(inner, true)
    }

    /// Parse a comma-separated list of field types and build a struct `AirType`.
    ///
    /// Uses bracket-depth counting to correctly split fields that contain
    /// nested brackets (e.g., `[10 x i32]` as a struct field).
    fn parse_struct_fields(&mut self, inner: &str, packed: bool) -> TypeId {
        let field_strs = split_type_list(inner);

        // Parse each field type
        let mut field_type_ids = Vec::with_capacity(field_strs.len());
        for field_str in &field_strs {
            let trimmed = field_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            let fid = self.parse_llvm_type_string(trimmed);
            field_type_ids.push(fid);
        }

        if field_type_ids.is_empty() {
            // Empty struct
            let ty = AirType::Struct {
                fields: Vec::new(),
                total_size: 0,
            };
            return self.intern(ty);
        }

        // Build StructField entries (without layout info initially)
        let fields_no_layout: Vec<StructField> = field_type_ids
            .iter()
            .map(|&fid| StructField {
                field_type: fid,
                byte_offset: None,
                byte_size: None,
                name: None,
            })
            .collect();

        if packed {
            // Packed struct: all alignment = 1, offsets are cumulative sizes
            self.build_packed_struct(&fields_no_layout)
        } else {
            // Normal struct: use layout::compute_struct_layout
            self.build_normal_struct(fields_no_layout)
        }
    }

    /// Build a normal (non-packed) struct with layout computation.
    fn build_normal_struct(&mut self, fields_no_layout: Vec<StructField>) -> TypeId {
        if let Some((offsets, total_size)) =
            layout::compute_struct_layout(&fields_no_layout, &self.forward)
        {
            // Populate byte_offset and byte_size on each field
            let mut fields = Vec::with_capacity(fields_no_layout.len());
            for (i, field) in fields_no_layout.into_iter().enumerate() {
                let byte_offset = offsets[i];
                let byte_size = self
                    .forward
                    .get(&field.field_type)
                    .and_then(|ty| layout::alloc_size(ty, &self.forward));
                fields.push(StructField {
                    field_type: field.field_type,
                    byte_offset: Some(byte_offset),
                    byte_size,
                    name: None,
                });
            }
            let ty = AirType::Struct { fields, total_size };
            self.intern(ty)
        } else {
            // Layout computation failed (e.g., Opaque sub-field) -> set offsets to None
            let ty = AirType::Struct {
                fields: fields_no_layout,
                total_size: 0,
            };
            self.intern(ty)
        }
    }

    /// Build a packed struct where all fields have alignment 1.
    fn build_packed_struct(&mut self, fields_no_layout: &[StructField]) -> TypeId {
        let mut offset: u64 = 0;
        let mut fields = Vec::with_capacity(fields_no_layout.len());

        for field in fields_no_layout {
            let size = self
                .forward
                .get(&field.field_type)
                .and_then(|ty| layout::alloc_size(ty, &self.forward));

            if let Some(sz) = size {
                fields.push(StructField {
                    field_type: field.field_type,
                    byte_offset: Some(offset),
                    byte_size: Some(sz),
                    name: None,
                });
                offset += sz;
            } else {
                // Unknown size — can't compute layout
                fields.push(StructField {
                    field_type: field.field_type,
                    byte_offset: None,
                    byte_size: None,
                    name: None,
                });
            }
        }

        let ty = AirType::Struct {
            fields,
            total_size: offset,
        };
        self.intern(ty)
    }
}

/// Build the canonical string representation of an `AirType` for `TypeId` derivation.
fn canonical_string(ty: &AirType) -> String {
    match ty {
        AirType::Pointer => "pointer".to_string(),
        AirType::Reference { nullable } => format!("reference:{nullable}"),
        AirType::Vector { element, lanes } => format!("vector:{}:{lanes}", element.to_hex()),
        AirType::Integer { bits } => format!("integer:{bits}"),
        AirType::Float { bits } => format!("float:{bits}"),
        AirType::Void => "void".to_string(),
        AirType::Opaque => "opaque".to_string(),
        AirType::Array { element, count } => {
            let count_str = count.map_or_else(|| "vla".to_string(), |c| c.to_string());
            format!("array:{}:{count_str}", element.to_hex())
        }
        AirType::Struct { fields, total_size } => {
            let mut s = String::from("struct:");
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    s.push(':');
                }
                s.push_str(&field.field_type.to_hex());
                s.push('@');
                if let Some(off) = field.byte_offset {
                    s.push_str(&off.to_string());
                } else {
                    s.push('?');
                }
            }
            s.push(':');
            s.push_str(&total_size.to_string());
            s
        }
        AirType::Function {
            params,
            return_type,
        } => {
            let mut s = String::from("function:");
            for (i, p) in params.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&p.to_hex());
            }
            s.push(':');
            s.push_str(&return_type.to_hex());
            s
        }
    }
}

/// Split a comma-separated list of LLVM type strings, respecting bracket nesting.
///
/// For example, `" [10 x i32], ptr "` splits into `["[10 x i32]", "ptr"]`.
/// Handles `[]`, `{}`, and `<>` bracket types.
fn split_type_list(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '[' | '{' | '<' => depth += 1,
            ']' | '}' | '>' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            ',' if depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    // Push the last segment
    let last = &s[start..];
    if !last.trim().is_empty() {
        result.push(last);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_deduplicates() {
        let mut interner = TypeInterner::new();
        let a = interner.intern_pointer();
        let b = interner.intern_pointer();
        assert_eq!(a, b);
        assert_eq!(interner.into_table().len(), 1);
    }

    #[test]
    fn intern_different_types_get_different_ids() {
        let mut interner = TypeInterner::new();
        let ptr = interner.intern_pointer();
        let i32_id = interner.intern_integer(32);
        assert_ne!(ptr, i32_id);
    }

    #[test]
    fn parse_ptr() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("ptr");
        let table = interner.into_table();
        assert!(matches!(table.get(&id), Some(AirType::Pointer)));
    }

    #[test]
    fn parse_integer_types() {
        let mut interner = TypeInterner::new();
        for (s, bits) in [
            ("i1", 1),
            ("i8", 8),
            ("i16", 16),
            ("i32", 32),
            ("i64", 64),
            ("i128", 128),
        ] {
            let id = interner.parse_llvm_type_string(s);
            assert!(
                matches!(interner.forward.get(&id), Some(AirType::Integer { bits: b }) if *b == bits),
                "expected Integer {{ bits: {bits} }} for \"{s}\""
            );
        }
    }

    #[test]
    fn parse_float_types() {
        let mut interner = TypeInterner::new();
        let f32_id = interner.parse_llvm_type_string("float");
        let f64_id = interner.parse_llvm_type_string("double");
        assert!(matches!(
            interner.forward.get(&f32_id),
            Some(AirType::Float { bits: 32 })
        ));
        assert!(matches!(
            interner.forward.get(&f64_id),
            Some(AirType::Float { bits: 64 })
        ));
    }

    #[test]
    fn parse_void() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("void");
        assert!(matches!(interner.forward.get(&id), Some(AirType::Void)));
    }

    #[test]
    fn parse_array() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("[10 x i32]");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Array {
                count: Some(10), ..
            }) => {}
            other => panic!("expected Array with count 10, got {other:?}"),
        }
    }

    #[test]
    fn parse_struct_with_layout() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("{ i32, ptr }");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Struct { fields, total_size }) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(*total_size, 16); // i32(4) + pad(4) + ptr(8)
                assert_eq!(fields[0].byte_offset, Some(0));
                assert_eq!(fields[1].byte_offset, Some(8));
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn parse_forward_ref_returns_opaque() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("%struct.Foo");
        assert!(matches!(interner.forward.get(&id), Some(AirType::Opaque)));
    }

    #[test]
    fn parse_named_struct_definition() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("%struct.Foo = type { i32, ptr }");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Struct { fields, total_size }) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(*total_size, 16); // i32(4) + pad(4) + ptr(8)
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn parse_named_packed_struct_definition() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("%struct.Bar = type <{ i32, i8 }>");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Struct { fields, total_size }) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(*total_size, 5);
            }
            other => panic!("expected packed Struct, got {other:?}"),
        }
    }

    #[test]
    fn parse_nested_array_in_struct() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("{ [10 x i32], ptr }");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Struct { fields, total_size }) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(*total_size, 48); // [10 x i32](40) + ptr(8)
                assert_eq!(fields[0].byte_offset, Some(0));
                assert_eq!(fields[1].byte_offset, Some(40));
            }
            other => panic!("expected Struct, got {other:?}"),
        }
    }

    #[test]
    fn deterministic_ids_across_interners() {
        let mut a = TypeInterner::new();
        let mut b = TypeInterner::new();
        assert_eq!(a.intern_pointer(), b.intern_pointer());
        assert_eq!(a.intern_integer(32), b.intern_integer(32));
    }

    #[test]
    fn split_type_list_simple() {
        let parts = split_type_list(" i32, ptr ");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].trim(), "i32");
        assert_eq!(parts[1].trim(), "ptr");
    }

    #[test]
    fn split_type_list_nested_brackets() {
        let parts = split_type_list(" [10 x i32], ptr ");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].trim(), "[10 x i32]");
        assert_eq!(parts[1].trim(), "ptr");
    }

    #[test]
    fn split_type_list_empty() {
        let parts = split_type_list("  ");
        assert!(parts.is_empty());
    }

    #[test]
    fn parse_packed_struct() {
        let mut interner = TypeInterner::new();
        let id = interner.parse_llvm_type_string("<{ i32, i8 }>");
        let table = interner.into_table();
        match table.get(&id) {
            Some(AirType::Struct { fields, total_size }) => {
                assert_eq!(fields.len(), 2);
                // Packed: i32(4) + i8(1) = 5, no padding
                assert_eq!(*total_size, 5);
                assert_eq!(fields[0].byte_offset, Some(0));
                assert_eq!(fields[1].byte_offset, Some(4));
            }
            other => panic!("expected packed Struct, got {other:?}"),
        }
    }

    #[test]
    fn canonical_string_primitives() {
        assert_eq!(canonical_string(&AirType::Pointer), "pointer");
        assert_eq!(
            canonical_string(&AirType::Integer { bits: 32 }),
            "integer:32"
        );
        assert_eq!(canonical_string(&AirType::Float { bits: 64 }), "float:64");
        assert_eq!(canonical_string(&AirType::Void), "void");
        assert_eq!(canonical_string(&AirType::Opaque), "opaque");
    }

    #[test]
    fn intern_void_and_opaque() {
        let mut interner = TypeInterner::new();
        let v = interner.intern_void();
        let o = interner.intern_opaque();
        assert_ne!(v, o);
        assert_eq!(interner.forward.len(), 2);
    }

    #[test]
    fn intern_float_variants() {
        let mut interner = TypeInterner::new();
        let f32_id = interner.intern_float(32);
        let f64_id = interner.intern_float(64);
        assert_ne!(f32_id, f64_id);
        // Re-intern should dedup
        assert_eq!(f32_id, interner.intern_float(32));
    }
}
