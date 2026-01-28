//! Export types for PTA results.
//!
//! Provides JSON-serializable export formats for points-to analysis results
//! and constraint graphs.

use serde::Serialize;

use super::config::PtaConfig;
use super::constraint::ConstraintSet;
use super::context::PtaDiagnostics;
use super::result::PtaResult;
use super::solver::PointsToMap;

/// Current export schema version.
pub const EXPORT_SCHEMA_VERSION: &str = "0.1.0";

/// Exported PTA result.
#[derive(Debug, Serialize)]
pub struct PtaExport {
    /// Schema version for compatibility checking.
    pub schema_version: String,
    /// Configuration used for analysis.
    pub config: PtaConfigExport,
    /// All abstract locations.
    pub locations: Vec<LocationExport>,
    /// Points-to sets for each value.
    pub points_to: Vec<PointsToExport>,
    /// Analysis diagnostics.
    pub diagnostics: DiagnosticsExport,
}

/// Exported PTA configuration.
#[derive(Debug, Serialize)]
pub struct PtaConfigExport {
    /// Whether analysis was enabled.
    pub enabled: bool,
    /// Field sensitivity setting.
    pub field_sensitivity: String,
    /// Maximum objects limit.
    pub max_objects: usize,
    /// Maximum iterations limit.
    pub max_iterations: usize,
}

/// Exported location info.
#[derive(Debug, Serialize)]
pub struct LocationExport {
    /// Location ID as hex string.
    pub id: String,
    /// Object ID as hex string.
    pub obj: String,
    /// Field path as list of step strings.
    pub path: Vec<String>,
}

/// Exported points-to entry.
#[derive(Debug, Serialize)]
pub struct PointsToExport {
    /// Value ID as hex string.
    pub value: String,
    /// Location IDs that the value may point to.
    pub locations: Vec<String>,
}

/// Exported diagnostics.
#[derive(Debug, Serialize)]
pub struct DiagnosticsExport {
    /// Number of solver iterations used.
    pub iterations: usize,
    /// Whether iteration limit was hit.
    pub iteration_limit_hit: bool,
    /// Number of collapse warnings.
    pub collapse_warning_count: usize,
    /// Total constraints extracted.
    pub constraint_count: usize,
    /// Total locations created.
    pub location_count: usize,
}

/// Exported constraint graph (for debugging).
#[derive(Debug, Serialize)]
pub struct ConstraintGraphExport {
    /// Schema version.
    pub schema_version: String,
    /// Addr constraints.
    pub addr: Vec<AddrConstraintExport>,
    /// Copy constraints.
    pub copy: Vec<CopyConstraintExport>,
    /// Load constraints.
    pub load: Vec<LoadConstraintExport>,
    /// Store constraints.
    pub store: Vec<StoreConstraintExport>,
    /// GEP constraints.
    pub gep: Vec<GepConstraintExport>,
}

/// Exported Addr constraint.
#[derive(Debug, Serialize)]
pub struct AddrConstraintExport {
    /// Pointer value ID.
    pub ptr: String,
    /// Location ID.
    pub loc: String,
}

/// Exported Copy constraint.
#[derive(Debug, Serialize)]
pub struct CopyConstraintExport {
    /// Destination value ID.
    pub dst: String,
    /// Source value ID.
    pub src: String,
}

/// Exported Load constraint.
#[derive(Debug, Serialize)]
pub struct LoadConstraintExport {
    /// Destination value ID.
    pub dst: String,
    /// Source pointer value ID.
    pub src_ptr: String,
}

/// Exported Store constraint.
#[derive(Debug, Serialize)]
pub struct StoreConstraintExport {
    /// Destination pointer value ID.
    pub dst_ptr: String,
    /// Source value ID.
    pub src: String,
}

/// Exported GEP constraint.
#[derive(Debug, Serialize)]
pub struct GepConstraintExport {
    /// Destination value ID.
    pub dst: String,
    /// Source pointer value ID.
    pub src_ptr: String,
    /// Field path.
    pub path: Vec<String>,
}

impl PtaResult {
    /// Export as a [`PropertyGraph`](crate::export::PropertyGraph).
    ///
    /// Pointer values become nodes with labels `["Pointer"]`.
    /// Locations become nodes with labels `["Location"]` carrying `obj`
    /// and `path` properties.  Edges are `POINTS_TO`.
    #[must_use]
    pub fn to_pg(
        &self,
        resolver: Option<&crate::display::DisplayResolver<'_>>,
    ) -> crate::export::PropertyGraph {
        use crate::export::{PgEdge, PgNode, PropertyGraph, enrich_node};
        use std::collections::BTreeMap;

        let mut pg = PropertyGraph::new("pta");

        // Collect all locations that appear in any points-to set
        let mut seen_locs: std::collections::BTreeSet<saf_core::ids::LocId> =
            std::collections::BTreeSet::new();

        for (value_id, locs) in self.points_to_map() {
            if locs.is_empty() {
                continue;
            }

            // Pointer node
            let mut ptr_node = PgNode {
                id: value_id.to_hex(),
                labels: vec!["Pointer".to_string()],
                properties: BTreeMap::new(),
            };
            enrich_node(&mut ptr_node, resolver);
            pg.nodes.push(ptr_node);

            for loc_id in locs {
                seen_locs.insert(*loc_id);

                pg.edges.push(PgEdge {
                    src: value_id.to_hex(),
                    dst: loc_id.to_hex(),
                    edge_type: "POINTS_TO".to_string(),
                    properties: BTreeMap::new(),
                });
            }
        }

        // Location nodes
        for loc_id in &seen_locs {
            let mut properties = BTreeMap::new();
            if let Some(loc) = self.locations().get(loc_id) {
                properties.insert("obj".to_string(), serde_json::json!(loc.obj.to_hex()));
                let path: Vec<String> = export_field_path(&loc.path);
                if !path.is_empty() {
                    properties.insert("path".to_string(), serde_json::json!(path));
                }
            }

            let mut loc_node = PgNode {
                id: loc_id.to_hex(),
                labels: vec!["Location".to_string()],
                properties,
            };
            enrich_node(&mut loc_node, resolver);
            pg.nodes.push(loc_node);
        }

        pg
    }

    /// Export the result to a serializable format.
    #[must_use]
    pub fn export(&self, config: &PtaConfig) -> PtaExport {
        PtaExport {
            schema_version: EXPORT_SCHEMA_VERSION.to_string(),
            config: export_config(config),
            locations: export_locations(self.locations()),
            points_to: export_points_to(self.points_to_map()),
            diagnostics: export_diagnostics(self.diagnostics()),
        }
    }
}

/// Export constraints to a serializable format.
pub fn export_constraints(constraints: &ConstraintSet) -> ConstraintGraphExport {
    ConstraintGraphExport {
        schema_version: EXPORT_SCHEMA_VERSION.to_string(),
        addr: constraints
            .addr
            .iter()
            .map(|c| AddrConstraintExport {
                ptr: c.ptr.to_hex(),
                loc: c.loc.to_hex(),
            })
            .collect(),
        copy: constraints
            .copy
            .iter()
            .map(|c| CopyConstraintExport {
                dst: c.dst.to_hex(),
                src: c.src.to_hex(),
            })
            .collect(),
        load: constraints
            .load
            .iter()
            .map(|c| LoadConstraintExport {
                dst: c.dst.to_hex(),
                src_ptr: c.src_ptr.to_hex(),
            })
            .collect(),
        store: constraints
            .store
            .iter()
            .map(|c| StoreConstraintExport {
                dst_ptr: c.dst_ptr.to_hex(),
                src: c.src.to_hex(),
            })
            .collect(),
        gep: constraints
            .gep
            .iter()
            .map(|c| GepConstraintExport {
                dst: c.dst.to_hex(),
                src_ptr: c.src_ptr.to_hex(),
                path: export_field_path(&c.path),
            })
            .collect(),
    }
}

fn export_config(config: &PtaConfig) -> PtaConfigExport {
    let field_sensitivity = match &config.field_sensitivity {
        super::config::FieldSensitivity::None => "none".to_string(),
        super::config::FieldSensitivity::StructFields { max_depth } => {
            format!("struct_fields(max_depth={max_depth})")
        }
    };

    PtaConfigExport {
        enabled: config.enabled,
        field_sensitivity,
        max_objects: config.max_objects,
        max_iterations: config.max_iterations,
    }
}

fn export_locations(
    locations: &rustc_hash::FxHashMap<saf_core::ids::LocId, super::location::Location>,
) -> Vec<LocationExport> {
    let mut result: Vec<LocationExport> = locations
        .iter()
        .map(|(id, loc)| LocationExport {
            id: id.to_hex(),
            obj: loc.obj.to_hex(),
            path: export_field_path(&loc.path),
        })
        .collect();
    // Sort by ID for deterministic output (FxHashMap iteration is unordered)
    result.sort_by(|a, b| a.id.cmp(&b.id));
    result
}

fn export_points_to(pts: &PointsToMap) -> Vec<PointsToExport> {
    pts.iter()
        .map(|(value, locs)| PointsToExport {
            value: value.to_hex(),
            locations: locs.iter().map(|l| l.to_hex()).collect(),
        })
        .collect()
}

fn export_diagnostics(diag: &PtaDiagnostics) -> DiagnosticsExport {
    DiagnosticsExport {
        iterations: diag.iterations,
        iteration_limit_hit: diag.iteration_limit_hit,
        collapse_warning_count: diag.collapse_warning_count,
        constraint_count: diag.constraint_count,
        location_count: diag.location_count,
    }
}

fn export_field_path(path: &super::location::FieldPath) -> Vec<String> {
    use super::location::IndexExpr;
    path.steps
        .iter()
        .map(|step| match step {
            super::location::PathStep::Index(expr) => match expr {
                IndexExpr::Unknown => "[]".to_string(),
                IndexExpr::Constant(v) => format!("[{v}]"),
                IndexExpr::Symbolic(vid) => format!("[v{:x}]", vid.raw()),
            },
            super::location::PathStep::Field { index } => format!(".{index}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use saf_core::ids::{LocId, ObjId, ValueId};

    use crate::pta::config::FieldSensitivity;
    use crate::pta::constraint::{AddrConstraint, CopyConstraint};
    use crate::pta::location::{FieldPath, LocationFactory};

    fn make_factory() -> LocationFactory {
        LocationFactory::new(FieldSensitivity::StructFields { max_depth: 2 })
    }

    fn make_config() -> PtaConfig {
        PtaConfig::default()
    }

    #[test]
    fn export_empty_result() {
        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(make_factory()),
            PtaDiagnostics::default(),
        );
        let config = make_config();
        let export = result.export(&config);

        assert_eq!(export.schema_version, EXPORT_SCHEMA_VERSION);
        assert!(export.locations.is_empty());
        assert!(export.points_to.is_empty());
    }

    #[test]
    fn export_with_locations() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        let _loc = factory.get_or_create(obj, FieldPath::empty());

        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );
        let config = make_config();
        let export = result.export(&config);

        assert_eq!(export.locations.len(), 1);
        assert!(export.locations[0].path.is_empty());
    }

    #[test]
    fn export_with_field_path() {
        let mut factory = make_factory();
        let obj = ObjId::new(100);
        let _loc = factory.get_or_create(obj, FieldPath::field(0));

        let result = PtaResult::new(
            PointsToMap::new(),
            Arc::new(factory),
            PtaDiagnostics::default(),
        );
        let config = make_config();
        let export = result.export(&config);

        assert_eq!(export.locations.len(), 1);
        assert_eq!(export.locations[0].path, vec![".0"]);
    }

    #[test]
    fn export_with_points_to() {
        let mut factory = make_factory();
        let loc = factory.get_or_create(ObjId::new(100), FieldPath::empty());

        let p = ValueId::new(1);
        let mut pts_map = PointsToMap::new();
        let mut p_set = BTreeSet::new();
        p_set.insert(loc);
        pts_map.insert(p, p_set);

        let result = PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default());
        let config = make_config();
        let export = result.export(&config);

        assert_eq!(export.points_to.len(), 1);
        assert_eq!(export.points_to[0].locations.len(), 1);
    }

    #[test]
    fn export_constraints() {
        let mut constraints = ConstraintSet::default();
        constraints.addr.insert(AddrConstraint {
            ptr: ValueId::new(1),
            loc: LocId::new(100),
        });
        constraints.copy.insert(CopyConstraint {
            dst: ValueId::new(2),
            src: ValueId::new(1),
        });

        let export = super::export_constraints(&constraints);

        assert_eq!(export.schema_version, EXPORT_SCHEMA_VERSION);
        assert_eq!(export.addr.len(), 1);
        assert_eq!(export.copy.len(), 1);
    }

    #[test]
    fn export_deterministic() {
        // Create same result twice and verify export is identical
        let create_result_fn = || {
            let mut factory = make_factory();
            let _loc1 = factory.get_or_create(ObjId::new(100), FieldPath::empty());
            let _loc2 = factory.get_or_create(ObjId::new(200), FieldPath::field(0));

            let p = ValueId::new(1);
            let q = ValueId::new(2);
            let mut pts_map = PointsToMap::new();
            pts_map.insert(p, BTreeSet::new());
            pts_map.insert(q, BTreeSet::new());

            PtaResult::new(pts_map, Arc::new(factory), PtaDiagnostics::default())
        };

        let config = make_config();
        let export1 = create_result_fn().export(&config);
        let export2 = create_result_fn().export(&config);

        let json1 = serde_json::to_string(&export1).unwrap();
        let json2 = serde_json::to_string(&export2).unwrap();

        assert_eq!(json1, json2);
    }
}
