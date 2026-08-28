//! Property-generic SV-COMP **YAML witness format 2.0** *violation*-witness emitter.
//!
//! Given a flat, ordered list of source-level [`SourceWaypoint`]s plus a
//! [`WitnessMeta`], [`ViolationWitness::assemble`] builds a witness whose
//! serialization ([`ViolationWitness::to_yaml_string`]) is a
//! `entry_type: violation_sequence` entry — a sequence of *segments*, each a run
//! of `action: avoid` waypoints terminated by exactly one `action: follow`, the
//! final segment ending in a `type: target` waypoint (SPIN 2024 "Software
//! Verification Witnesses 2.0", Fig. 2 + Table 3).
//!
//! This module is **property-generic**: it knows nothing about `unreach-call`,
//! AIR, or any specific analysis — the AIR→source lowering that produces the
//! waypoints lives in `witness_lower`, and the per-property strategy lives in
//! `saf-cli`. Determinism (NFR-DET-001): the output is byte-identical for
//! identical inputs — a **fixed** `creation_time`, a content-derived RFC-4122
//! `uuid`, and `BTreeMap`-ordered file hashes, with no `SystemTime::now()`/RNG.
//!
//! NOTE: the exact byte layout (segment/waypoint wrapper keys, metadata
//! spellings, `file_name` form, branching-value quoting) is pinned to what the
//! provisioned `witnesslint` + CPAchecker accept (plan 194 Slice F), not frozen
//! from the paper.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::property_kind::{DataModel, Language};

const FORMAT_VERSION: &str = "2.0";
const PRODUCER_NAME: &str = "SAF";
/// Fixed creation timestamp: witness bytes must be byte-identical for identical
/// inputs (NFR-DET-001), so the emitter never reads the wall clock.
const FIXED_CREATION_TIME: &str = "2024-01-01T00:00:00Z";

/// Caller-supplied witness metadata; the emitter derives `uuid`, hashes, and the
/// fixed `creation_time` from these.
#[derive(Debug, Clone)]
pub struct WitnessMeta {
    /// Producer version string (e.g. `saf --version`).
    pub producer_version: String,
    /// The SV-COMP specification text (raw, trimmed `.prp` contents).
    pub specification: String,
    /// Data model the program was compiled under.
    pub data_model: DataModel,
    /// Source language.
    pub language: Language,
    /// The un-preprocessed input file handed to `verify` (hashed for the task).
    pub input_file: PathBuf,
}

/// Whether an execution must pass through (`follow`) or avoid a waypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Executions represented by the witness must pass through this waypoint.
    Follow,
    /// Executions represented by the witness must not pass through this waypoint.
    Avoid,
}

impl Action {
    fn as_str(self) -> &'static str {
        match self {
            Action::Follow => "follow",
            Action::Avoid => "avoid",
        }
    }
}

/// The five waypoint requirement types defined by witness format 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointKind {
    /// Constraint on variable values (C expression) before a statement.
    Assumption,
    /// A branch evaluated a specific way (value `true`/`false`/int/`default`).
    Branching,
    /// A called function is entered (constraint omitted).
    FunctionEnter,
    /// A function-call return value satisfies an ACSL constraint.
    FunctionReturn,
    /// The program location where the property is violated (constraint omitted).
    Target,
}

impl WaypointKind {
    fn as_str(self) -> &'static str {
        match self {
            WaypointKind::Assumption => "assumption",
            WaypointKind::Branching => "branching",
            WaypointKind::FunctionEnter => "function_enter",
            WaypointKind::FunctionReturn => "function_return",
            WaypointKind::Target => "target",
        }
    }

    /// Per the format, `target` and `function_enter` must OMIT `constraint`;
    /// the other three REQUIRE it.
    fn requires_constraint(self) -> bool {
        !matches!(self, WaypointKind::Target | WaypointKind::FunctionEnter)
    }
}

/// A waypoint constraint: `format` (e.g. `c_expression`, `acsl_expression`) is
/// omitted for `branching` (which carries only `value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    /// The constraint language (`c_expression` / `acsl_expression`), or `None`.
    pub format: Option<String>,
    /// The constraint value (a C/ACSL expression, or `true`/`false`/int).
    pub value: String,
}

/// A source-level waypoint: property-agnostic input to [`ViolationWitness::assemble`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceWaypoint {
    /// The requirement type.
    pub kind: WaypointKind,
    /// `follow` (closes a segment) or `avoid`.
    pub action: Action,
    /// Source file name (matched by the validator against the program it is given).
    pub file_name: String,
    /// 1-based line.
    pub line: u32,
    /// 1-based column, if known.
    pub column: Option<u32>,
    /// Enclosing function name (cosmetic; optional).
    pub function: Option<String>,
    /// Constraint (required for assumption/branching/function_return; else `None`).
    pub constraint: Option<Constraint>,
}

/// An assembled YAML 2.0 violation witness, ready to serialize.
#[derive(Debug)]
pub struct ViolationWitness {
    entry: Entry,
}

impl ViolationWitness {
    /// Assemble a violation witness from a flat, ordered waypoint list.
    ///
    /// Groups waypoints into segments: each `Follow` closes the current segment.
    /// The list MUST end with a `Target` waypoint (which closes the final
    /// segment).
    ///
    /// # Errors
    /// Returns `Err` if the list is empty, does not end in a `Target`, leaves a
    /// trailing unterminated segment (final waypoint not `follow`), or a
    /// waypoint's `constraint` presence does not match its `kind` (assumption /
    /// branching / function_return require one; target / function_enter omit it).
    pub fn assemble(meta: &WitnessMeta, waypoints: &[SourceWaypoint]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !waypoints.is_empty(),
            "a violation witness needs at least a target waypoint"
        );
        anyhow::ensure!(
            matches!(waypoints.last().map(|w| w.kind), Some(WaypointKind::Target)),
            "a violation witness must end in a `target` waypoint"
        );

        let mut content: Vec<SegmentWrap> = Vec::new();
        let mut current: Vec<WaypointWrap> = Vec::new();
        for wp in waypoints {
            if wp.kind.requires_constraint() {
                anyhow::ensure!(
                    wp.constraint.is_some(),
                    "waypoint type `{}` requires a constraint",
                    wp.kind.as_str()
                );
            } else {
                anyhow::ensure!(
                    wp.constraint.is_none(),
                    "waypoint type `{}` must omit its constraint",
                    wp.kind.as_str()
                );
            }
            current.push(WaypointWrap {
                waypoint: waypoint_out(wp),
            });
            if matches!(wp.action, Action::Follow) {
                content.push(SegmentWrap {
                    segment: std::mem::take(&mut current),
                });
            }
        }
        anyhow::ensure!(
            current.is_empty(),
            "the final waypoint must be a `follow` (unterminated segment)"
        );

        Ok(Self {
            entry: Entry {
                entry_type: "violation_sequence",
                metadata: build_metadata(meta),
                content,
            },
        })
    }

    /// Serialize to a YAML 2.0 witness document (a one-entry list). Deterministic.
    ///
    /// # Errors
    /// Returns `Err` if `serde_yaml` fails to serialize the witness.
    pub fn to_yaml_string(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(&vec![&self.entry])?)
    }
}

/// SHA-256 hex digest of a file's contents; `"unknown"` if unreadable.
#[must_use]
pub fn compute_file_hash(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(contents) => {
            let mut hasher = Sha256::new();
            hasher.update(&contents);
            let result = hasher.finalize();
            format!("{result:064x}")
        }
        Err(_) => "unknown".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Internal serde model (controls the exact serialized shape).
// ---------------------------------------------------------------------------

// `entry_type` is a mandatory YAML-2.0 key, not renameable — silence the
// field-starts-with-struct-name lint rather than break the schema.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize)]
struct Entry {
    entry_type: &'static str,
    metadata: Metadata,
    content: Vec<SegmentWrap>,
}

#[derive(Debug, Serialize)]
struct Metadata {
    format_version: &'static str,
    uuid: String,
    creation_time: &'static str,
    producer: Producer,
    task: Task,
}

#[derive(Debug, Serialize)]
struct Producer {
    name: &'static str,
    version: String,
}

#[derive(Debug, Serialize)]
struct Task {
    input_files: Vec<String>,
    input_file_hashes: BTreeMap<String, String>,
    specification: String,
    data_model: String,
    language: &'static str,
}

#[derive(Debug, Serialize)]
struct SegmentWrap {
    segment: Vec<WaypointWrap>,
}

#[derive(Debug, Serialize)]
struct WaypointWrap {
    waypoint: WaypointOut,
}

#[derive(Debug, Serialize)]
struct WaypointOut {
    action: &'static str,
    #[serde(rename = "type")]
    kind: &'static str,
    location: LocationOut,
    #[serde(skip_serializing_if = "Option::is_none")]
    constraint: Option<ConstraintOut>,
}

#[derive(Debug, Serialize)]
struct LocationOut {
    file_name: String,
    line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
}

#[derive(Debug, Serialize)]
struct ConstraintOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    value: String,
}

fn waypoint_out(wp: &SourceWaypoint) -> WaypointOut {
    WaypointOut {
        action: wp.action.as_str(),
        kind: wp.kind.as_str(),
        location: LocationOut {
            file_name: wp.file_name.clone(),
            line: wp.line,
            column: wp.column,
            function: wp.function.clone(),
        },
        constraint: wp.constraint.as_ref().map(|c| ConstraintOut {
            format: c.format.clone(),
            value: c.value.clone(),
        }),
    }
}

fn build_metadata(meta: &WitnessMeta) -> Metadata {
    let hash = compute_file_hash(&meta.input_file);
    let basename = meta
        .input_file
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("input.c")
        .to_string();
    let mut input_file_hashes = BTreeMap::new();
    input_file_hashes.insert(basename.clone(), hash.clone());

    let data_model = match meta.data_model {
        DataModel::ILP32 => "ILP32",
        DataModel::LP64 => "LP64",
    }
    .to_string();
    let language = match meta.language {
        Language::C => "C",
        Language::Cpp => "C++",
    };

    // Deterministic uuid from content: identical inputs -> identical witness.
    let mut seed = Vec::new();
    seed.extend_from_slice(hash.as_bytes());
    seed.extend_from_slice(meta.specification.as_bytes());
    seed.extend_from_slice(meta.producer_version.as_bytes());

    Metadata {
        format_version: FORMAT_VERSION,
        uuid: deterministic_uuid(&seed),
        creation_time: FIXED_CREATION_TIME,
        producer: Producer {
            name: PRODUCER_NAME,
            version: meta.producer_version.clone(),
        },
        task: Task {
            input_files: vec![basename],
            input_file_hashes,
            specification: meta.specification.clone(),
            data_model,
            language,
        },
    }
}

/// A deterministic RFC-4122 (v5-style) UUID string derived from `seed`.
fn deterministic_uuid(seed: &[u8]) -> String {
    let digest = Sha256::digest(seed);
    let mut b = [0u8; 16];
    b.copy_from_slice(&digest[..16]);
    b[6] = (b[6] & 0x0f) | 0x50; // version 5
    b[8] = (b[8] & 0x3f) | 0x80; // variant RFC-4122
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_meta() -> WitnessMeta {
        WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "CHECK( init(main()), LTL(G ! call(reach_error())) )".to_string(),
            data_model: DataModel::LP64,
            language: Language::C,
            // Nonexistent path -> hash "unknown"; fine and deterministic for the test.
            input_file: PathBuf::from("/nonexistent/a.c"),
        }
    }

    fn branching(line: u32, action: Action, value: &str) -> SourceWaypoint {
        SourceWaypoint {
            kind: WaypointKind::Branching,
            action,
            file_name: "a.c".to_string(),
            line,
            column: None,
            function: None,
            constraint: Some(Constraint {
                format: None,
                value: value.to_string(),
            }),
        }
    }

    fn target(line: u32) -> SourceWaypoint {
        SourceWaypoint {
            kind: WaypointKind::Target,
            action: Action::Follow,
            file_name: "a.c".to_string(),
            line,
            column: None,
            function: None,
            constraint: None,
        }
    }

    #[test]
    fn assemble_groups_follow_into_segments_and_requires_target() {
        let meta = tiny_meta();
        // branching(follow) then target(follow) => two segments.
        let w =
            ViolationWitness::assemble(&meta, &[branching(11, Action::Follow, "true"), target(17)])
                .expect("assemble ok");
        let yaml = w.to_yaml_string().expect("serialize ok");
        assert!(yaml.contains("entry_type: violation_sequence"), "{yaml}");
        assert!(yaml.contains("type: target"), "{yaml}");
        assert_eq!(yaml.matches("segment:").count(), 2, "two segments\n{yaml}");

        // An `avoid` waypoint does NOT close a segment: [assumption(avoid), target] => 1 segment.
        let assumption_avoid = SourceWaypoint {
            kind: WaypointKind::Assumption,
            action: Action::Avoid,
            file_name: "a.c".to_string(),
            line: 5,
            column: None,
            function: None,
            constraint: Some(Constraint {
                format: Some("c_expression".to_string()),
                value: "x == 0".to_string(),
            }),
        };
        let w2 =
            ViolationWitness::assemble(&meta, &[assumption_avoid, target(9)]).expect("assemble ok");
        let yaml2 = w2.to_yaml_string().expect("serialize ok");
        assert_eq!(yaml2.matches("segment:").count(), 1, "one segment\n{yaml2}");
    }

    #[test]
    fn assemble_rejects_missing_target_and_constraint_mismatches() {
        let meta = tiny_meta();
        // Final waypoint is not a target => Err.
        assert!(
            ViolationWitness::assemble(&meta, &[branching(1, Action::Follow, "true")]).is_err()
        );
        // Empty => Err.
        assert!(ViolationWitness::assemble(&meta, &[]).is_err());
        // Target with a constraint => Err (must be omitted).
        let bad_target = SourceWaypoint {
            constraint: Some(Constraint {
                format: None,
                value: "x".to_string(),
            }),
            ..target(3)
        };
        assert!(ViolationWitness::assemble(&meta, &[bad_target]).is_err());
        // Branching without a constraint => Err (required).
        let bad_branch = SourceWaypoint {
            constraint: None,
            ..branching(2, Action::Follow, "true")
        };
        assert!(ViolationWitness::assemble(&meta, &[bad_branch, target(3)]).is_err());
    }

    #[test]
    fn to_yaml_is_byte_stable_and_has_fixed_creation_time() {
        let meta = tiny_meta();
        let a = ViolationWitness::assemble(&meta, &[target(17)])
            .unwrap()
            .to_yaml_string()
            .unwrap();
        let b = ViolationWitness::assemble(&meta, &[target(17)])
            .unwrap()
            .to_yaml_string()
            .unwrap();
        assert_eq!(a, b, "witness bytes must be deterministic");
        assert!(a.contains("creation_time: 2024-01-01T00:00:00Z"), "{a}");
        assert!(a.contains("format_version:"), "{a}");
        assert!(a.contains("data_model: LP64"), "{a}");
        assert!(a.contains("specification:"), "{a}");
    }

    #[test]
    fn deterministic_uuid_is_stable_and_well_formed() {
        let u1 = deterministic_uuid(b"seed-abc");
        let u2 = deterministic_uuid(b"seed-abc");
        assert_eq!(u1, u2);
        assert_ne!(
            deterministic_uuid(b"seed-abc"),
            deterministic_uuid(b"seed-xyz")
        );
        // 8-4-4-4-12 with version 5 and RFC-4122 variant.
        let lens: Vec<usize> = u1.split('-').map(str::len).collect();
        assert_eq!(lens, [8, 4, 4, 4, 12]);
        assert_eq!(&u1[14..15], "5");
        assert!(matches!(&u1[19..20], "8" | "9" | "a" | "b"));
    }

    /// Golden layout: locks the exact serialized shape (a witnesslint-accepted
    /// 2.0 violation witness — see plan 194 Slice F) against accidental serde
    /// drift. The content-derived `uuid` line is masked (it varies with inputs).
    #[test]
    fn golden_target_only_layout() {
        let meta = WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "SPEC".to_string(),
            data_model: DataModel::LP64,
            language: Language::C,
            input_file: PathBuf::from("/nonexistent/x.c"),
        };
        let wp = SourceWaypoint {
            kind: WaypointKind::Target,
            action: Action::Follow,
            file_name: "x.c".to_string(),
            line: 42,
            column: Some(7),
            function: Some("main".to_string()),
            constraint: None,
        };
        let yaml = ViolationWitness::assemble(&meta, &[wp])
            .unwrap()
            .to_yaml_string()
            .unwrap();
        let masked: String = yaml
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("uuid:") {
                    "    uuid: <UUID>"
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let expected = "\
- entry_type: violation_sequence
  metadata:
    format_version: '2.0'
    uuid: <UUID>
    creation_time: 2024-01-01T00:00:00Z
    producer:
      name: SAF
      version: 0.1.0
    task:
      input_files:
      - x.c
      input_file_hashes:
        x.c: unknown
      specification: SPEC
      data_model: LP64
      language: C
  content:
  - segment:
    - waypoint:
        action: follow
        type: target
        location:
          file_name: x.c
          line: 42
          column: 7
          function: main";
        assert_eq!(masked.trim_end(), expected);
    }
}
