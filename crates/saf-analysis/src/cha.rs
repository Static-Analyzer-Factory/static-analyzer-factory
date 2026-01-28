//! Class Hierarchy Analysis (CHA) for virtual dispatch resolution.
//!
//! Builds a class hierarchy from AIR `TypeHierarchyEntry` records and resolves
//! virtual method calls by considering all possible receiver types (the declared
//! receiver plus every transitive subclass).
//!
//! See Plan 024 for design context and FR-CALL-003 for requirements.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use saf_core::air::TypeHierarchyEntry;
use saf_core::ids::FunctionId;

/// Class hierarchy built from AIR type-hierarchy entries.
///
/// Stores direct base-class relationships, pre-computed transitive subclass
/// sets, and per-class vtable mappings.  All collections use `BTreeMap` /
/// `BTreeSet` to guarantee deterministic iteration order (NFR-DET-001).
#[derive(Clone)]
pub struct ClassHierarchy {
    /// Direct base classes for each class (`class -> [direct bases]`).
    bases: BTreeMap<String, Vec<String>>,
    /// Transitive subclasses for each class (`class -> {transitive subclasses}`).
    subclasses: BTreeMap<String, BTreeSet<String>>,
    /// Per-class vtable: slot index maps to an optional `FunctionId`.
    /// `None` means the slot is pure-virtual (abstract).
    vtables: BTreeMap<String, Vec<Option<FunctionId>>>,
}

impl ClassHierarchy {
    /// Build a `ClassHierarchy` from a slice of [`TypeHierarchyEntry`] records.
    ///
    /// 1. Populates `bases` and `vtables` directly from the entries.
    /// 2. Computes the inverse mapping (class -> direct *children*).
    /// 3. For every known class, performs a BFS over the inverse mapping to
    ///    collect all transitive subclasses.
    #[must_use]
    pub fn build(entries: &[TypeHierarchyEntry]) -> Self {
        // --- Step 1: populate bases and vtables ---
        let mut bases: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut vtables: BTreeMap<String, Vec<Option<FunctionId>>> = BTreeMap::new();

        for entry in entries {
            bases.insert(entry.type_name.clone(), entry.base_types.clone());

            // Build vtable: expand to cover every slot index mentioned.
            let mut vtable: Vec<Option<FunctionId>> = Vec::new();
            for slot in &entry.virtual_methods {
                // Ensure the vtable is large enough
                if slot.index >= vtable.len() {
                    vtable.resize(slot.index + 1, None);
                }
                vtable[slot.index] = slot.function;
            }
            vtables.insert(entry.type_name.clone(), vtable);
        }

        // Make sure every referenced base type has an entry in `bases` even if
        // it was not itself listed as an entry (it may be an external type).
        let all_base_refs: Vec<String> = bases.values().flat_map(|bs| bs.iter().cloned()).collect();
        for base in all_base_refs {
            bases.entry(base).or_default();
        }

        // --- Step 2: compute inverse mapping (parent -> direct children) ---
        let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (class, class_bases) in &bases {
            for base in class_bases {
                children
                    .entry(base.clone())
                    .or_default()
                    .insert(class.clone());
            }
        }

        // --- Step 3: BFS to compute transitive subclasses for every class ---
        let mut subclasses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for class in bases.keys() {
            let mut visited = BTreeSet::new();
            let mut queue = VecDeque::new();

            // Seed with direct children
            if let Some(direct) = children.get(class) {
                for child in direct {
                    if visited.insert(child.clone()) {
                        queue.push_back(child.clone());
                    }
                }
            }

            // BFS over children-of-children
            while let Some(current) = queue.pop_front() {
                if let Some(grandchildren) = children.get(&current) {
                    for gc in grandchildren {
                        if visited.insert(gc.clone()) {
                            queue.push_back(gc.clone());
                        }
                    }
                }
            }

            subclasses.insert(class.clone(), visited);
        }

        Self {
            bases,
            subclasses,
            vtables,
        }
    }

    /// Resolve a virtual call on `receiver` at vtable `slot`.
    ///
    /// Returns every non-`None` function ID found in that slot for the
    /// receiver class and all of its transitive subclasses.  The result is
    /// deduplicated and sorted (via `BTreeSet`) for determinism.
    #[must_use]
    pub fn resolve_virtual(&self, receiver: &str, slot: usize) -> Vec<FunctionId> {
        let mut result = BTreeSet::new();

        // Helper: look up slot in a class's vtable
        let lookup = |class: &str| -> Option<FunctionId> {
            self.vtables
                .get(class)
                .and_then(|vt| vt.get(slot).copied())
                .flatten()
        };

        // Check the receiver itself
        if let Some(fid) = lookup(receiver) {
            result.insert(fid);
        }

        // Check every transitive subclass
        if let Some(subs) = self.subclasses.get(receiver) {
            for sub in subs {
                if let Some(fid) = lookup(sub) {
                    result.insert(fid);
                }
            }
        }

        result.into_iter().collect()
    }

    /// Return the set of transitive subclasses of `class`.
    ///
    /// Returns an empty set if the class is unknown.
    #[must_use]
    pub fn subclasses_of(&self, class: &str) -> BTreeSet<String> {
        self.subclasses.get(class).cloned().unwrap_or_default()
    }

    /// Export the hierarchy as a JSON value suitable for serialization.
    ///
    /// The returned object has three keys:
    /// - `"classes"`: sorted list of all known class names.
    /// - `"inheritance"`: map from class name to its direct base classes.
    /// - `"vtables"`: map from class name to list of slot entries
    ///   (`{"slot": <index>, "function": <hex_id> | null}`).
    #[must_use]
    pub fn export(&self) -> serde_json::Value {
        let classes: Vec<&String> = self.bases.keys().collect();

        let inheritance: BTreeMap<&String, &Vec<String>> =
            self.bases.iter().filter(|(_, v)| !v.is_empty()).collect();

        let vtables_export: BTreeMap<&String, Vec<serde_json::Value>> = self
            .vtables
            .iter()
            .map(|(class, vt)| {
                let slots: Vec<serde_json::Value> = vt
                    .iter()
                    .enumerate()
                    .map(|(idx, func)| {
                        serde_json::json!({
                            "slot": idx,
                            "function": func.map(saf_core::ids::FunctionId::to_hex),
                        })
                    })
                    .collect();
                (class, slots)
            })
            .collect();

        serde_json::json!({
            "classes": classes,
            "inheritance": inheritance,
            "vtables": vtables_export,
        })
    }

    /// Returns `true` if the hierarchy contains no classes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bases.is_empty()
    }

    /// Returns the set of root classes (classes with no base types).
    ///
    /// Root classes are the top of independent class hierarchies. Resolving
    /// virtual calls at root classes covers all subclasses without cross-hierarchy
    /// contamination.
    #[must_use]
    pub fn root_classes(&self) -> Vec<&str> {
        self.bases
            .iter()
            .filter(|(_, base_list)| base_list.is_empty())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Returns all known class names that have vtable entries.
    #[must_use]
    pub fn classes_with_vtables(&self) -> Vec<&str> {
        self.vtables.keys().map(String::as_str).collect()
    }

    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Each class becomes a `["Type"]` node with `name` and `vtable_slots`
    /// properties.  Inheritance edges use `INHERITS` (child -> base).
    #[must_use]
    pub fn to_pg(
        &self,
        _resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph};

        let mut pg = PropertyGraph::new("cha");

        for class_name in self.bases.keys() {
            let mut properties = BTreeMap::new();
            properties.insert(
                "name".to_string(),
                serde_json::Value::String(class_name.clone()),
            );

            // Add vtable slot count if this class has a vtable
            if let Some(vt) = self.vtables.get(class_name) {
                if !vt.is_empty() {
                    properties.insert("vtable_slots".to_string(), serde_json::json!(vt.len()));
                }
            }

            pg.nodes.push(PgNode {
                id: class_name.clone(),
                labels: vec!["Type".to_string()],
                properties,
            });
        }

        for (class_name, base_list) in &self.bases {
            for base in base_list {
                pg.edges.push(PgEdge {
                    src: class_name.clone(),
                    dst: base.clone(),
                    edge_type: "INHERITS".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        pg
    }

    /// Check if `class_name` is known in the hierarchy.
    #[must_use]
    pub fn contains(&self, class_name: &str) -> bool {
        self.bases.contains_key(class_name)
    }

    /// Returns direct base classes of `class_name`.
    #[must_use]
    pub fn bases_of(&self, class_name: &str) -> &[String] {
        self.bases
            .get(class_name)
            .map_or(&[] as &[String], Vec::as_slice)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::VirtualMethodSlot;

    /// Helper: build a `TypeHierarchyEntry` with no virtual methods.
    fn entry(name: &str, bases: &[&str]) -> TypeHierarchyEntry {
        TypeHierarchyEntry {
            type_name: name.to_string(),
            base_types: bases.iter().map(|b| (*b).to_string()).collect(),
            virtual_methods: Vec::new(),
        }
    }

    /// Helper: build a `TypeHierarchyEntry` with virtual-method slots.
    fn entry_with_vtable(
        name: &str,
        bases: &[&str],
        slots: &[(usize, Option<FunctionId>)],
    ) -> TypeHierarchyEntry {
        TypeHierarchyEntry {
            type_name: name.to_string(),
            base_types: bases.iter().map(|b| (*b).to_string()).collect(),
            virtual_methods: slots
                .iter()
                .map(|(idx, func)| VirtualMethodSlot {
                    index: *idx,
                    function: *func,
                })
                .collect(),
        }
    }

    // -------------------------------------------------------------------------
    // 1. Empty hierarchy
    // -------------------------------------------------------------------------
    #[test]
    fn empty_hierarchy() {
        let cha = ClassHierarchy::build(&[]);
        assert!(cha.is_empty());
        assert!(cha.subclasses_of("Anything").is_empty());
        assert!(cha.resolve_virtual("Anything", 0).is_empty());

        let export = cha.export();
        let classes = export["classes"]
            .as_array()
            .expect("classes should be array");
        assert!(classes.is_empty());
    }

    // -------------------------------------------------------------------------
    // 2. Single inheritance chain: A -> B -> C
    //    (C derives from B, B derives from A)
    // -------------------------------------------------------------------------
    #[test]
    fn single_inheritance_chain() {
        let entries = vec![entry("A", &[]), entry("B", &["A"]), entry("C", &["B"])];
        let cha = ClassHierarchy::build(&entries);

        // A's transitive subclasses: {B, C}
        let a_subs = cha.subclasses_of("A");
        assert_eq!(a_subs, BTreeSet::from(["B".to_string(), "C".to_string()]));

        // B's transitive subclasses: {C}
        let b_subs = cha.subclasses_of("B");
        assert_eq!(b_subs, BTreeSet::from(["C".to_string()]));

        // C has no subclasses
        assert!(cha.subclasses_of("C").is_empty());

        assert!(!cha.is_empty());
    }

    // -------------------------------------------------------------------------
    // 3. Diamond inheritance: A -> B, A -> C, B -> D, C -> D
    //    (D derives from both B and C; B and C both derive from A)
    // -------------------------------------------------------------------------
    #[test]
    fn diamond_inheritance() {
        let entries = vec![
            entry("A", &[]),
            entry("B", &["A"]),
            entry("C", &["A"]),
            entry("D", &["B", "C"]),
        ];
        let cha = ClassHierarchy::build(&entries);

        // A's transitive subclasses: {B, C, D}
        let a_subs = cha.subclasses_of("A");
        assert_eq!(
            a_subs,
            BTreeSet::from(["B".to_string(), "C".to_string(), "D".to_string()])
        );

        // B's transitive subclasses: {D}
        assert_eq!(cha.subclasses_of("B"), BTreeSet::from(["D".to_string()]));

        // C's transitive subclasses: {D}
        assert_eq!(cha.subclasses_of("C"), BTreeSet::from(["D".to_string()]));

        // D has no subclasses
        assert!(cha.subclasses_of("D").is_empty());
    }

    // -------------------------------------------------------------------------
    // 4. Pure virtual: Base has None in slot 0, Derived fills it
    // -------------------------------------------------------------------------
    #[test]
    fn pure_virtual_slot() {
        let derived_func = FunctionId::derive(b"Derived::process");

        let entries = vec![
            entry_with_vtable("Base", &[], &[(0, None)]),
            entry_with_vtable("Derived", &["Base"], &[(0, Some(derived_func))]),
        ];
        let cha = ClassHierarchy::build(&entries);

        // Resolving on Base at slot 0: Base has None, but Derived has Some
        let resolved = cha.resolve_virtual("Base", 0);
        assert_eq!(resolved, vec![derived_func]);

        // Resolving on Derived at slot 0: only Derived's implementation
        let resolved_d = cha.resolve_virtual("Derived", 0);
        assert_eq!(resolved_d, vec![derived_func]);
    }

    // -------------------------------------------------------------------------
    // 5. resolve_virtual returns correct functions including subclass overrides
    // -------------------------------------------------------------------------
    #[test]
    fn resolve_virtual_with_overrides() {
        let base_func = FunctionId::derive(b"Base::do_thing");
        let mid_func = FunctionId::derive(b"Mid::do_thing");
        let leaf_func = FunctionId::derive(b"Leaf::do_thing");

        let entries = vec![
            entry_with_vtable("Base", &[], &[(0, Some(base_func))]),
            entry_with_vtable("Mid", &["Base"], &[(0, Some(mid_func))]),
            entry_with_vtable("Leaf", &["Mid"], &[(0, Some(leaf_func))]),
        ];
        let cha = ClassHierarchy::build(&entries);

        // Resolving on Base: Base + Mid + Leaf all have slot 0
        let resolved = cha.resolve_virtual("Base", 0);
        assert_eq!(resolved.len(), 3);
        assert!(resolved.contains(&base_func));
        assert!(resolved.contains(&mid_func));
        assert!(resolved.contains(&leaf_func));

        // Resolving on Mid: Mid + Leaf
        let resolved_mid = cha.resolve_virtual("Mid", 0);
        assert_eq!(resolved_mid.len(), 2);
        assert!(resolved_mid.contains(&mid_func));
        assert!(resolved_mid.contains(&leaf_func));

        // Resolving on Leaf: only Leaf
        let resolved_leaf = cha.resolve_virtual("Leaf", 0);
        assert_eq!(resolved_leaf, vec![leaf_func]);
    }

    // -------------------------------------------------------------------------
    // 6. Deterministic export
    // -------------------------------------------------------------------------
    #[test]
    fn deterministic_export() {
        let func_a = FunctionId::derive(b"A::foo");
        let func_b = FunctionId::derive(b"B::foo");

        let entries = vec![
            entry_with_vtable("A", &[], &[(0, Some(func_a))]),
            entry_with_vtable("B", &["A"], &[(0, Some(func_b))]),
        ];

        let cha = ClassHierarchy::build(&entries);

        let export1 = serde_json::to_string(&cha.export()).expect("export 1");
        let export2 = serde_json::to_string(&cha.export()).expect("export 2");
        assert_eq!(export1, export2, "exports must be byte-identical");

        // Verify structure
        let val = cha.export();
        assert!(val["classes"].is_array());
        assert!(val["inheritance"].is_object());
        assert!(val["vtables"].is_object());

        // classes should be sorted
        let classes: Vec<&str> = val["classes"]
            .as_array()
            .expect("array")
            .iter()
            .map(|v| v.as_str().expect("string"))
            .collect();
        assert_eq!(classes, vec!["A", "B"]);

        // inheritance: only B has bases
        assert!(val["inheritance"].get("A").is_none());
        let b_bases = val["inheritance"]["B"].as_array().expect("B bases array");
        assert_eq!(b_bases.len(), 1);
        assert_eq!(b_bases[0].as_str().expect("str"), "A");
    }

    // -------------------------------------------------------------------------
    // Additional: unknown class and out-of-range slot
    // -------------------------------------------------------------------------
    #[test]
    fn unknown_class_and_slot() {
        let entries = vec![entry_with_vtable(
            "X",
            &[],
            &[(0, Some(FunctionId::derive(b"X::m")))],
        )];
        let cha = ClassHierarchy::build(&entries);

        // Unknown receiver
        assert!(cha.resolve_virtual("NoSuchClass", 0).is_empty());

        // Known receiver, out-of-range slot
        assert!(cha.resolve_virtual("X", 999).is_empty());
    }
}
