//! Function-to-location mapping for indirect call resolution.
//!
//! When PTA says "value V points to location L with `ObjId` O", this map
//! provides the reverse lookup from `ObjId` to `FunctionId`, enabling
//! resolution of indirect calls via function pointer analysis.

use std::collections::BTreeMap;

use saf_core::air::AirModule;
use saf_core::ids::{FunctionId, ObjId};

/// Maps `ObjId` to `FunctionId` for function-as-location resolution.
///
/// Each function implicitly has an `ObjId` equal to `ObjId::new(func.id.raw())`.
/// When PTA determines that a value points to a location with that `ObjId`,
/// this map resolves which function the value is a pointer to.
pub struct FunctionLocationMap {
    /// Reverse map from `ObjId` to `FunctionId`.
    obj_to_func: BTreeMap<ObjId, FunctionId>,
}

impl FunctionLocationMap {
    /// Build a `FunctionLocationMap` from an AIR module.
    ///
    /// Each function (including declarations) has an implicit `ObjId`
    /// derived from its `FunctionId`: `ObjId::new(func.id.raw())`.
    #[must_use]
    pub fn build(module: &AirModule) -> Self {
        let mut obj_to_func = BTreeMap::new();
        for func in &module.functions {
            let obj = ObjId::new(func.id.raw());
            obj_to_func.insert(obj, func.id);
        }
        Self { obj_to_func }
    }

    /// Look up the function for a given `ObjId`.
    ///
    /// Returns `Some(FunctionId)` if the `ObjId` corresponds to a known
    /// function's implicit object, or `None` otherwise.
    #[must_use]
    pub fn get(&self, obj: ObjId) -> Option<FunctionId> {
        self.obj_to_func.get(&obj).copied()
    }

    /// Number of entries in the map.
    #[must_use]
    #[allow(dead_code)] // Standard API when pta module is exposed directly
    pub fn len(&self) -> usize {
        self.obj_to_func.len()
    }

    /// Check if the map is empty.
    #[must_use]
    #[allow(dead_code)] // Standard API when pta module is exposed directly
    pub fn is_empty(&self) -> bool {
        self.obj_to_func.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saf_core::air::{AirFunction, AirModule};
    use saf_core::ids::{FunctionId, ModuleId};

    fn make_module() -> AirModule {
        AirModule::new(ModuleId::derive(b"test"))
    }

    #[test]
    fn empty_module_produces_empty_map() {
        let module = make_module();
        let map = FunctionLocationMap::build(&module);
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn build_from_module_with_three_functions() {
        let mut module = make_module();

        let f1_id = FunctionId::derive(b"func_a");
        let f2_id = FunctionId::derive(b"func_b");
        let f3_id = FunctionId::derive(b"func_c");

        let mut f1 = AirFunction::new(f1_id, "func_a");
        // Add a minimal block so f1 is not a declaration
        let block = saf_core::air::AirBlock::new(saf_core::ids::BlockId::derive(b"entry_a"));
        f1.blocks.push(block);

        let mut f2 = AirFunction::new(f2_id, "func_b");
        let block2 = saf_core::air::AirBlock::new(saf_core::ids::BlockId::derive(b"entry_b"));
        f2.blocks.push(block2);

        let mut f3 = AirFunction::new(f3_id, "func_c");
        let block3 = saf_core::air::AirBlock::new(saf_core::ids::BlockId::derive(b"entry_c"));
        f3.blocks.push(block3);

        module.functions.push(f1);
        module.functions.push(f2);
        module.functions.push(f3);

        let map = FunctionLocationMap::build(&module);
        assert_eq!(map.len(), 3);

        // Each function's ObjId is derived from its FunctionId
        assert_eq!(map.get(ObjId::new(f1_id.raw())), Some(f1_id));
        assert_eq!(map.get(ObjId::new(f2_id.raw())), Some(f2_id));
        assert_eq!(map.get(ObjId::new(f3_id.raw())), Some(f3_id));

        // Unknown ObjId returns None
        assert_eq!(map.get(ObjId::new(999)), None);
    }

    #[test]
    fn declarations_are_also_mapped() {
        let mut module = make_module();

        let decl_id = FunctionId::derive(b"external_func");
        let mut decl = AirFunction::new(decl_id, "external_func");
        decl.is_declaration = true;

        let def_id = FunctionId::derive(b"defined_func");
        let mut def = AirFunction::new(def_id, "defined_func");
        let block = saf_core::air::AirBlock::new(saf_core::ids::BlockId::derive(b"entry"));
        def.blocks.push(block);

        module.functions.push(decl);
        module.functions.push(def);

        let map = FunctionLocationMap::build(&module);
        assert_eq!(map.len(), 2);

        // Both declarations and definitions are mapped
        assert_eq!(map.get(ObjId::new(decl_id.raw())), Some(decl_id));
        assert_eq!(map.get(ObjId::new(def_id.raw())), Some(def_id));
    }
}
