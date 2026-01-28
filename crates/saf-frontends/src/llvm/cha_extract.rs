//! CHA (Class Hierarchy Analysis) extraction from LLVM IR.
//!
//! Extracts type hierarchy information from C++ vtable (`_ZTV*`) and
//! typeinfo (`_ZTI*`) globals in LLVM IR.
//!
//! ## LLVM vtable layout (Itanium ABI)
//!
//! A typical vtable global `_ZTV<mangled_class>` is an `unnamed_addr` constant
//! with an aggregate initializer:
//! ```text
//! @_ZTV7Derived = constant { [N x ptr] } { [N x ptr] [
//!   ptr null,                          ; offset-to-top
//!   ptr @_ZTI7Derived,                 ; typeinfo pointer
//!   ptr @_ZN7Derived7processEPKc,      ; vtable slot 0
//!   ptr @_ZN7Derived9transformEPKc,    ; vtable slot 1
//!   ...
//! ] }
//! ```
//!
//! The first two entries are metadata (offset-to-top and RTTI pointer);
//! actual virtual method slots start at index 2.
//!
//! ## Typeinfo layout
//!
//! Single inheritance:
//! ```text
//! @_ZTI7Derived = constant { ptr, ptr, ptr } {
//!   ptr @_ZTVN10__cxxabiv120__si_class_type_infoE+16,
//!   ptr @_ZTS7Derived,
//!   ptr @_ZTI4Base   ; <-- base class typeinfo
//! }
//! ```

#![cfg(any(feature = "llvm-17", feature = "llvm-18"))]

use std::collections::BTreeMap;

use inkwell::module::Module;
use inkwell::values::{AnyValue, BasicValueEnum, GlobalValue};

use saf_core::air::TypeHierarchyEntry;
use saf_core::air::VirtualMethodSlot;

use super::mapping::MappingContext;

/// Create a `TypeHierarchyEntry` with empty base types and virtual methods.
fn empty_entry(type_name: String) -> TypeHierarchyEntry {
    TypeHierarchyEntry {
        type_name,
        base_types: Vec::new(),
        virtual_methods: Vec::new(),
    }
}

/// Extract type hierarchy entries from LLVM module globals.
///
/// Scans `_ZTV*` (vtable) and `_ZTI*` (typeinfo) globals to build
/// a type hierarchy suitable for CHA resolution.
pub fn extract_type_hierarchy(
    module: &Module<'_>,
    ctx: &MappingContext<'_>,
) -> Vec<TypeHierarchyEntry> {
    let mut entries_map: BTreeMap<String, TypeHierarchyEntry> = BTreeMap::new();

    // First pass: extract vtables (_ZTV* globals)
    for global in module.get_globals() {
        let name = match global.get_name().to_str() {
            Ok(n) => n.to_string(),
            Err(_) => continue,
        };

        if name.starts_with("_ZTV") && !name.starts_with("_ZTVN10__cxxabiv1") {
            if let Some(class_name) = demangle_vtable_name(&name) {
                let vtable_slots = extract_vtable_slots(global, ctx);
                let entry = entries_map
                    .entry(class_name.clone())
                    .or_insert_with(|| empty_entry(class_name));
                entry.virtual_methods = vtable_slots;
            }
        }
    }

    // Second pass: extract typeinfo for base class relationships (_ZTI* globals)
    for global in module.get_globals() {
        let name = match global.get_name().to_str() {
            Ok(n) => n.to_string(),
            Err(_) => continue,
        };

        if name.starts_with("_ZTI") && !name.starts_with("_ZTVN10__cxxabiv1") {
            if let Some(class_name) = demangle_typeinfo_name(&name) {
                let bases = extract_base_classes(global, &class_name);
                if !bases.is_empty() {
                    let entry = entries_map
                        .entry(class_name.clone())
                        .or_insert_with(|| empty_entry(class_name));
                    entry.base_types = bases;
                }
            }
        }
    }

    // Third pass: extract construction vtables (_ZTC* globals) for virtual inheritance.
    // Classes with virtual bases (e.g., B and C in a diamond A←{B,C}←D) may not have
    // their own _ZTV* globals. Their vtable entries are only available via construction
    // vtables (_ZTC<Derived><Offset>_<Base>), which represent the base subobject's
    // vtable as used during construction of the derived class.
    for global in module.get_globals() {
        let name = match global.get_name().to_str() {
            Ok(n) => n.to_string(),
            Err(_) => continue,
        };

        if !name.starts_with("_ZTC") {
            continue;
        }

        let Some(base_class) = demangle_construction_vtable_name(&name) else {
            continue;
        };

        // Only populate if this base class has no vtable entries yet.
        // Classes with their own _ZTV* already have correct entries from pass 1.
        let needs_vtable = entries_map
            .get(&base_class)
            .is_none_or(|e| e.virtual_methods.is_empty());
        if !needs_vtable {
            continue;
        }

        let vtable_slots = extract_vtable_slots(global, ctx);
        if vtable_slots.is_empty() {
            continue;
        }

        let entry = entries_map
            .entry(base_class.clone())
            .or_insert_with(|| empty_entry(base_class));
        entry.virtual_methods = vtable_slots;
    }

    entries_map.into_values().collect()
}

/// Demangle a vtable name (`_ZTV<class>`) to a class name.
///
/// Uses simple Itanium ABI demangling: `_ZTV<len><name>` → `<name>`.
/// For nested names: `_ZTVN<len1><name1><len2><name2>E` → `<name1>::<name2>`.
fn demangle_vtable_name(name: &str) -> Option<String> {
    let mangled = name.strip_prefix("_ZTV")?;
    demangle_name(mangled)
}

/// Demangle a typeinfo name (`_ZTI<class>`) to a class name.
fn demangle_typeinfo_name(name: &str) -> Option<String> {
    let mangled = name.strip_prefix("_ZTI")?;
    demangle_name(mangled)
}

/// Demangle a construction vtable name (`_ZTC<derived><offset>_<base>`) to extract
/// the base class name.
///
/// Construction vtables are generated for virtual inheritance. The naming is:
/// `_ZTC` + `derived_class` (length-prefixed) + `offset_digits` + `_` + `base_class` (length-prefixed)
///
/// Examples:
/// - `_ZTC1D0_1B` → `Some("B")` (B-in-D at offset 0)
/// - `_ZTC1D8_1C` → `Some("C")` (C-in-D at offset 8)
fn demangle_construction_vtable_name(name: &str) -> Option<String> {
    let rest = name.strip_prefix("_ZTC")?;
    // Parse derived class name (length-prefixed)
    let (_, after_derived) = parse_length_prefixed_name(rest, 0)?;
    // Skip offset digits
    let bytes = rest.as_bytes();
    let mut pos = after_derived;
    while pos < bytes.len() && bytes[pos].is_ascii_digit() {
        pos += 1;
    }
    // Skip '_' separator
    if pos >= bytes.len() || bytes[pos] != b'_' {
        return None;
    }
    pos += 1;
    // Parse base class name
    let (base_name, _) = parse_length_prefixed_name(rest, pos)?;
    Some(base_name)
}

/// Demangle an Itanium ABI name.
///
/// Supports simple names (`<len><name>`) and nested names (`N<parts>E`).
fn demangle_name(mangled: &str) -> Option<String> {
    if mangled.starts_with('N') {
        // Nested name: N<len1><name1><len2><name2>...E
        let inner = mangled.strip_prefix('N')?.strip_suffix('E')?;
        let mut parts = Vec::new();
        let mut pos = 0;
        let bytes = inner.as_bytes();
        while pos < bytes.len() {
            let (name_part, new_pos) = parse_length_prefixed_name(inner, pos)?;
            parts.push(name_part);
            pos = new_pos;
        }
        if parts.is_empty() {
            return None;
        }
        Some(parts.join("::"))
    } else {
        // Simple name: <len><name>
        let (name, _) = parse_length_prefixed_name(mangled, 0)?;
        Some(name)
    }
}

/// Parse a length-prefixed name starting at `pos`.
/// Returns the name and the position after it.
fn parse_length_prefixed_name(s: &str, pos: usize) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    if pos >= bytes.len() || !bytes[pos].is_ascii_digit() {
        return None;
    }

    // Parse the length digits
    let mut end = pos;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let len: usize = s[pos..end].parse().ok()?;
    let name_start = end;
    let name_end = name_start + len;
    if name_end > s.len() {
        return None;
    }
    Some((s[name_start..name_end].to_string(), name_end))
}

/// Extract vtable slots from a `_ZTV*` global's initializer.
///
/// Handles three vtable layouts:
/// - **Single inheritance:** `{ [N x ptr] }` — one sub-array with 2 metadata entries.
/// - **Multiple inheritance:** `{ [N x ptr], [M x ptr], ... }` — multiple sub-arrays,
///   each with its own metadata. Only the **primary vtable** (first sub-array) is used.
/// - **Virtual inheritance:** primary sub-array may have 3+ metadata entries
///   (virtual-base-offset, offset-to-top, RTTI, ...). RTTI is detected by `_ZTI*` name.
fn extract_vtable_slots(
    global: GlobalValue<'_>,
    ctx: &MappingContext<'_>,
) -> Vec<VirtualMethodSlot> {
    let Some(init) = global.get_initializer() else {
        return Vec::new();
    };

    // Extract the primary vtable's function pointers.
    // For multi-inheritance vtables (StructValue with multiple ArrayValue fields),
    // use only the first sub-array (primary vtable at offset 0).
    let func_ptrs = extract_primary_vtable_pointers(init);

    // Find the RTTI entry to determine where metadata ends.
    // In Itanium ABI vtables, entries before (and including) the _ZTI* RTTI pointer
    // are metadata. The first entry after RTTI is virtual method slot 0.
    let metadata_count = find_rtti_index(&func_ptrs).map_or(2, |rtti_idx| rtti_idx + 1); // Fallback: skip 2 (offset-to-top + RTTI)

    let mut slots = Vec::new();
    for (i, func_name) in func_ptrs.iter().enumerate().skip(metadata_count) {
        let function = func_name
            .as_ref()
            .and_then(|name| ctx.get_function_id_by_name(name));

        slots.push(VirtualMethodSlot {
            index: i - metadata_count,
            function,
        });
    }

    slots
}

/// Extract function pointers from only the primary vtable sub-array.
///
/// For a struct-of-arrays vtable (multi-inheritance), returns only the first
/// sub-array's entries. For a single struct wrapping one array, returns that
/// array's entries.
fn extract_primary_vtable_pointers(value: BasicValueEnum<'_>) -> Vec<Option<String>> {
    match value {
        BasicValueEnum::StructValue(sv) => {
            // Check if this struct contains multiple array fields (multi-inheritance vtable)
            let num = sv.count_fields();
            if num == 0 {
                return Vec::new();
            }

            // Use only the first field — the primary vtable.
            // Secondary vtables (for base subobjects) contain thunks/adjustors
            // that duplicate the primary vtable's slots.
            if let Some(first_field) = sv.get_field_at_index(0) {
                match first_field {
                    BasicValueEnum::ArrayValue(av) => {
                        let s = av.print_to_string().to_string();
                        parse_function_pointers_from_ir_string(&s)
                    }
                    // Single-element struct wrapping a non-array (rare)
                    _ => extract_function_pointers_from_constant(first_field),
                }
            } else {
                Vec::new()
            }
        }
        BasicValueEnum::ArrayValue(av) => {
            // Bare array (no struct wrapper)
            let s = av.print_to_string().to_string();
            parse_function_pointers_from_ir_string(&s)
        }
        _ => Vec::new(),
    }
}

/// Find the index of the RTTI entry (`_ZTI*`) in a list of function pointer names.
///
/// In the Itanium ABI, vtable metadata entries are:
/// - `offset-to-top` (null or integer)
/// - `_ZTI<class>` (RTTI pointer)
///
/// For virtual inheritance, there may be extra offset entries before RTTI:
/// - `virtual-base-offset` (null or integer)
/// - `offset-to-top` (null or integer)
/// - `_ZTI<class>` (RTTI pointer)
///
/// Returns the index of the `_ZTI*` entry, or `None` if not found.
fn find_rtti_index(ptrs: &[Option<String>]) -> Option<usize> {
    // Only search in the first few entries (metadata is always at the beginning)
    let search_limit = ptrs.len().min(6);
    ptrs[..search_limit]
        .iter()
        .position(|entry| entry.as_ref().is_some_and(|name| name.starts_with("_ZTI")))
}

/// Extract function pointer names from a constant value (recursive).
///
/// Returns a list of `Option<String>` — `Some(name)` for function pointers,
/// `None` for null pointers or non-function values.
fn extract_function_pointers_from_constant(value: BasicValueEnum<'_>) -> Vec<Option<String>> {
    let mut result = Vec::new();

    match value {
        BasicValueEnum::StructValue(sv) => {
            // Recurse into struct fields
            let num = sv.count_fields();
            for i in 0..num {
                if let Some(field) = sv.get_field_at_index(i) {
                    result.extend(extract_function_pointers_from_constant(field));
                }
            }
        }
        BasicValueEnum::ArrayValue(av) => {
            // Process array elements by printing and parsing
            // inkwell doesn't expose direct element access for const arrays,
            // so we extract from the string representation
            let s = av.print_to_string().to_string();
            let names = parse_function_pointers_from_ir_string(&s);
            result.extend(names);
        }
        BasicValueEnum::PointerValue(pv) => {
            if pv.is_null() {
                result.push(None);
            } else {
                let name = pv.get_name().to_str().unwrap_or("").to_string();
                if name.is_empty() {
                    result.push(None);
                } else {
                    result.push(Some(name));
                }
            }
        }
        _ => {
            result.push(None);
        }
    }

    result
}

/// Parse function pointer names from an LLVM IR array constant string.
///
/// The string looks like:
/// ```text
/// [4 x ptr] [ptr null, ptr @_ZTI7Derived, ptr @_ZN7Derived7processEPKc, ptr @_ZN7Derived9otherEv]
/// ```
pub(crate) fn parse_function_pointers_from_ir_string(s: &str) -> Vec<Option<String>> {
    let mut result = Vec::new();

    // Find the array content between [ ... ]
    // Look for the second '[' which starts the element list
    let inner = if let Some(pos) = s.find("] [") {
        &s[pos + 3..]
    } else if let Some(pos) = s.find('[') {
        &s[pos + 1..]
    } else {
        return result;
    };

    // Split by comma and parse each element
    for element in inner.split(',') {
        let elem = element.trim().trim_end_matches(']');
        if elem.contains("null") {
            result.push(None);
        } else if let Some(at_pos) = elem.find('@') {
            let name_start = at_pos + 1;
            let name = elem[name_start..]
                .trim()
                .trim_end_matches(')')
                .trim_end_matches(']');
            if name.is_empty() {
                result.push(None);
            } else {
                result.push(Some(name.to_string()));
            }
        } else {
            result.push(None);
        }
    }

    result
}

/// Extract base class names from a `_ZTI*` typeinfo global.
///
/// For single-inheritance classes (using `__si_class_type_info`),
/// the third element is the base class typeinfo pointer.
/// For multiple inheritance (`__vmi_class_type_info`),
/// base classes start at element 3, every other entry.
///
/// `self_class_name` is the demangled name of the class whose typeinfo
/// is being parsed, used to filter out self-references.
fn extract_base_classes(global: GlobalValue<'_>, self_class_name: &str) -> Vec<String> {
    let mut bases = Vec::new();

    let Some(init) = global.get_initializer() else {
        return bases;
    };

    // Print the initializer and parse base class references
    let s = init.print_to_string().to_string();

    // Look for references to other _ZTI* globals
    for part in s.split('@') {
        let name = part
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next()
            .unwrap_or("");

        if name.starts_with("_ZTI") && !name.starts_with("_ZTVN10__cxxabiv1") {
            if let Some(class_name) = demangle_typeinfo_name(name) {
                // Skip self-references: the typeinfo for a class may reference
                // its own _ZTI* global, which should not be treated as a base class.
                if class_name != self_class_name {
                    bases.push(class_name);
                }
            }
        }
    }

    bases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demangle_simple_name() {
        assert_eq!(demangle_name("4Base"), Some("Base".to_string()));
        assert_eq!(demangle_name("7Derived"), Some("Derived".to_string()));
        assert_eq!(demangle_name("9Processor"), Some("Processor".to_string()));
    }

    #[test]
    fn demangle_nested_name() {
        // N2ns5ClassE -> ns::Class
        assert_eq!(demangle_name("N2ns5ClassE"), Some("ns::Class".to_string()));
    }

    #[test]
    fn demangle_vtable_name_works() {
        assert_eq!(demangle_vtable_name("_ZTV4Base"), Some("Base".to_string()));
        assert_eq!(
            demangle_vtable_name("_ZTV7Derived"),
            Some("Derived".to_string())
        );
    }

    #[test]
    fn demangle_typeinfo_name_works() {
        assert_eq!(
            demangle_typeinfo_name("_ZTI4Base"),
            Some("Base".to_string())
        );
    }

    #[test]
    fn parse_function_pointers_from_ir() {
        let ir = "[3 x ptr] [ptr null, ptr @_ZTI4Base, ptr @_ZN4Base7processEv]";
        let ptrs = parse_function_pointers_from_ir_string(ir);
        assert_eq!(ptrs.len(), 3);
        assert_eq!(ptrs[0], None); // null
        assert_eq!(ptrs[1], Some("_ZTI4Base".to_string()));
        assert_eq!(ptrs[2], Some("_ZN4Base7processEv".to_string()));
    }

    #[test]
    fn demangle_returns_none_for_invalid() {
        assert_eq!(demangle_name(""), None);
        assert_eq!(demangle_name("abc"), None);
        assert_eq!(demangle_name("N"), None);
    }

    #[test]
    fn parse_empty_ir_string() {
        let ptrs = parse_function_pointers_from_ir_string("");
        assert!(ptrs.is_empty());
    }

    #[test]
    fn find_rtti_index_simple_vtable() {
        // Standard vtable: [null, _ZTI4Base, method0, method1]
        let ptrs = vec![
            None,
            Some("_ZTI4Base".to_string()),
            Some("_ZN4Base1fEv".to_string()),
            Some("_ZN4Base1gEv".to_string()),
        ];
        assert_eq!(find_rtti_index(&ptrs), Some(1));
    }

    #[test]
    fn find_rtti_index_virtual_inheritance() {
        // Virtual inheritance: [null, null, _ZTI7Derived, method0]
        // Extra metadata before RTTI (virtual-base-offset)
        let ptrs = vec![
            None,
            None,
            Some("_ZTI7Derived".to_string()),
            Some("_ZN7Derived1fEv".to_string()),
        ];
        assert_eq!(find_rtti_index(&ptrs), Some(2));
    }

    #[test]
    fn find_rtti_index_missing() {
        // No RTTI entry
        let ptrs = vec![None, None, Some("_ZN4Base1fEv".to_string())];
        assert_eq!(find_rtti_index(&ptrs), None);
    }

    #[test]
    fn demangle_construction_vtable() {
        // _ZTC1D0_1B → B (B-in-D at offset 0)
        assert_eq!(
            demangle_construction_vtable_name("_ZTC1D0_1B"),
            Some("B".to_string())
        );
        // _ZTC1D8_1C → C (C-in-D at offset 8)
        assert_eq!(
            demangle_construction_vtable_name("_ZTC1D8_1C"),
            Some("C".to_string())
        );
        // _ZTC1E0_1B → B (B-in-E at offset 0)
        assert_eq!(
            demangle_construction_vtable_name("_ZTC1E0_1B"),
            Some("B".to_string())
        );
        // _ZTC1E8_1C → C (C-in-E at offset 8)
        assert_eq!(
            demangle_construction_vtable_name("_ZTC1E8_1C"),
            Some("C".to_string())
        );
        // Invalid inputs
        assert_eq!(demangle_construction_vtable_name("_ZTV1D"), None);
        assert_eq!(demangle_construction_vtable_name("_ZTC"), None);
    }
}
