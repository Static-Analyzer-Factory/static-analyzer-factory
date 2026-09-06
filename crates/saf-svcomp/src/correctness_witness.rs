//! SV-COMP **YAML witness format 2.0** *correctness*-witness (`invariant_set`)
//! emitter — the TRUE-side counterpart to `witness_yaml`'s violation emitter
//! (plan 207, roadmap item 1b).
//!
//! Given a flat list of [`SourceInvariant`]s (each a loop-head location + a
//! side-effect-free C expression) plus a [`WitnessMeta`],
//! [`InvariantSetWitness::assemble`] builds an `entry_type: invariant_set`
//! witness whose serialization ([`InvariantSetWitness::to_yaml_string`]) is what
//! the real CPAchecker correctness-witness validator confirms (`Verification
//! result: TRUE`). The metadata layer (uuid / hashes / fixed creation_time) is
//! shared with the violation emitter for byte-determinism (NFR-DET-001).
//!
//! This module is **property-generic**: it knows nothing about which property or
//! program the invariants came from. Reading SAF's converged interval fixpoint
//! and choosing which loops to emit lives in the driver (plan 207 Slice 2); this
//! module only *renders and assembles*.

use serde::Serialize;

use crate::witness_yaml::{LocationOut, Metadata, WitnessMeta, build_metadata_seeded};

/// Render an integer interval `[lo, hi]` on a `bits`-wide variable `name` as a
/// side-effect-free C expression, dropping type-trivial bounds (Goblint's
/// `of_interval_opt` idea) so we never emit a vacuous `INT_MIN <= x && x <= INT_MAX`.
///
/// - `[v, v]` (singleton) -> `"name == v"`.
/// - a bound at the signed type extreme (`lo == signed_min(bits)` /
///   `hi == signed_max(bits)`) is dropped as implied by the type.
/// - fully-trivial (top) or empty (`lo > hi`) -> `None`.
///
/// Bounds are rendered from the raw signed `i128` interpretation the interval
/// domain uses; a kept bound is always a *true* fact (soundness), even if — for
/// an unsigned variable — a `0 <= name` lower bound is redundant.
#[must_use]
pub fn interval_to_c_expr(name: &str, lo: i128, hi: i128, bits: u8) -> Option<String> {
    if lo > hi {
        return None; // empty interval — nothing to say (and never emit on ⊥).
    }
    if lo == hi {
        return Some(format!("{name} == {lo}"));
    }
    let mut parts: Vec<String> = Vec::new();
    if lo > signed_min(bits) {
        parts.push(format!("{lo} <= {name}"));
    }
    if hi < signed_max(bits) {
        parts.push(format!("{name} <= {hi}"));
    }
    if parts.is_empty() {
        None // fully trivial (⊤ for the type) — nothing worth emitting.
    } else {
        Some(parts.join(" && "))
    }
}

/// Minimum value of a `bits`-wide two's-complement signed integer (the interval
/// domain's TOP lower bound), as `i128`.
fn signed_min(bits: u8) -> i128 {
    match bits {
        0 => 0,
        b if b >= 128 => i128::MIN,
        b => -(1i128 << (b - 1)),
    }
}

/// Maximum value of a `bits`-wide two's-complement signed integer, as `i128`.
fn signed_max(bits: u8) -> i128 {
    match bits {
        0 => 0,
        b if b >= 128 => i128::MAX,
        b => (1i128 << (b - 1)) - 1,
    }
}

/// A single loop-head invariant to emit: a source location (column ALWAYS
/// present — CPAchecker 4.2.2 crashes if it is omitted) + a C expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInvariant {
    /// Source file name (basename; matched by the validator against the program).
    pub file_name: String,
    /// 1-based line of the loop keyword (`while`/`for`/`do`).
    pub line: u32,
    /// 1-based column of the loop keyword. Never omitted.
    pub column: u32,
    /// Enclosing function name (cosmetic; optional).
    pub function: Option<String>,
    /// The side-effect-free C expression (e.g. `0 <= i && i <= 1000000`).
    pub value: String,
}

/// An assembled YAML 2.0 correctness (`invariant_set`) witness, ready to serialize.
#[derive(Debug)]
pub struct InvariantSetWitness {
    entry: CorrectnessEntry,
}

impl InvariantSetWitness {
    /// Assemble an `invariant_set` witness from a non-empty list of loop-head
    /// invariants.
    ///
    /// # Errors
    /// Returns `Err` if `invariants` is empty (a correctness witness with no
    /// invariant carries no information and should not be emitted).
    pub fn assemble(meta: &WitnessMeta, invariants: &[SourceInvariant]) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !invariants.is_empty(),
            "an invariant_set witness needs at least one invariant"
        );

        // Fold the ordered invariant content into the uuid seed so that distinct
        // invariant sets (and a violation witness for the same program) get
        // distinct, still-deterministic uuids.
        let mut seed = Vec::new();
        for inv in invariants {
            seed.extend_from_slice(inv.file_name.as_bytes());
            seed.extend_from_slice(&inv.line.to_le_bytes());
            seed.extend_from_slice(&inv.column.to_le_bytes());
            if let Some(f) = &inv.function {
                seed.extend_from_slice(f.as_bytes());
            }
            seed.extend_from_slice(inv.value.as_bytes());
            seed.push(0);
        }

        let content = invariants
            .iter()
            .map(|inv| InvariantWrap {
                invariant: InvariantOut {
                    kind: "loop_invariant",
                    location: LocationOut::new(
                        inv.file_name.clone(),
                        inv.line,
                        Some(inv.column),
                        inv.function.clone(),
                    ),
                    value: inv.value.clone(),
                    format: "c_expression",
                },
            })
            .collect();

        Ok(Self {
            entry: CorrectnessEntry {
                entry_type: "invariant_set",
                metadata: build_metadata_seeded(meta, &seed),
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

// ---------------------------------------------------------------------------
// Internal serde model (controls the exact serialized shape).
// ---------------------------------------------------------------------------

// `entry_type` is a mandatory YAML-2.0 key, not renameable.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Serialize)]
struct CorrectnessEntry {
    entry_type: &'static str,
    metadata: Metadata,
    content: Vec<InvariantWrap>,
}

#[derive(Debug, Serialize)]
struct InvariantWrap {
    invariant: InvariantOut,
}

#[derive(Debug, Serialize)]
struct InvariantOut {
    #[serde(rename = "type")]
    kind: &'static str,
    location: LocationOut,
    value: String,
    format: &'static str,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::property_kind::{DataModel, Language};

    // ---- interval_to_c_expr --------------------------------------------------

    #[test]
    fn singleton_renders_equality() {
        assert_eq!(interval_to_c_expr("s", 0, 0, 32).as_deref(), Some("s == 0"));
    }

    #[test]
    fn bounded_range_renders_conjunction() {
        assert_eq!(
            interval_to_c_expr("i", 0, 1_000_000, 32).as_deref(),
            Some("0 <= i && i <= 1000000")
        );
    }

    #[test]
    fn signed_top_is_dropped() {
        assert_eq!(
            interval_to_c_expr("x", i128::from(i32::MIN), i128::from(i32::MAX), 32),
            None
        );
    }

    #[test]
    fn trivial_lower_dropped_keeps_upper() {
        assert_eq!(
            interval_to_c_expr("i", i128::from(i32::MIN), 100, 32).as_deref(),
            Some("i <= 100")
        );
    }

    #[test]
    fn trivial_upper_dropped_keeps_lower() {
        assert_eq!(
            interval_to_c_expr("s", 5, i128::from(i32::MAX), 32).as_deref(),
            Some("5 <= s")
        );
    }

    #[test]
    fn empty_interval_is_none() {
        assert_eq!(interval_to_c_expr("x", 10, 5, 32), None);
    }

    // ---- assemble / serialize ------------------------------------------------

    fn meta() -> WitnessMeta {
        WitnessMeta {
            producer_version: "0.1.0".to_string(),
            specification: "SPEC".to_string(),
            data_model: DataModel::ILP32,
            language: Language::C,
            input_file: PathBuf::from("/nonexistent/prog.c"),
        }
    }

    fn inv(value: &str) -> SourceInvariant {
        SourceInvariant {
            file_name: "prog.c".to_string(),
            line: 7,
            column: 3,
            function: Some("main".to_string()),
            value: value.to_string(),
        }
    }

    #[test]
    fn assemble_rejects_empty() {
        assert!(InvariantSetWitness::assemble(&meta(), &[]).is_err());
    }

    #[test]
    fn assemble_is_byte_stable() {
        let a = InvariantSetWitness::assemble(&meta(), &[inv("s == 0")])
            .unwrap()
            .to_yaml_string()
            .unwrap();
        let b = InvariantSetWitness::assemble(&meta(), &[inv("s == 0")])
            .unwrap()
            .to_yaml_string()
            .unwrap();
        assert_eq!(a, b, "witness bytes must be deterministic");
    }

    #[test]
    fn golden_invariant_set_layout() {
        let w = InvariantSetWitness::assemble(&meta(), &[inv("0 <= i && i <= 1000000")]).unwrap();
        let yaml = w.to_yaml_string().unwrap();
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
- entry_type: invariant_set
  metadata:
    format_version: '2.0'
    uuid: <UUID>
    creation_time: 2024-01-01T00:00:00Z
    producer:
      name: SAF
      version: 0.1.0
    task:
      input_files:
      - prog.c
      input_file_hashes:
        prog.c: unknown
      specification: SPEC
      data_model: ILP32
      language: C
  content:
  - invariant:
      type: loop_invariant
      location:
        file_name: prog.c
        line: 7
        column: 3
        function: main
      value: 0 <= i && i <= 1000000
      format: c_expression";
        assert_eq!(masked.trim_end(), expected);
    }
}
