//! Source-level `__VERIFIER_nondet_*` read recognition.
//!
//! This module is **pure** (no I/O, no subprocess): it maps 1-based source lines to
//! the scalar-integer nondet read on that line ([`nondet_line_map`] / [`NondetSite`]),
//! which the replay driver uses to pin concrete inputs to the right call sites.
//!
//! It was once `cbmc.rs` and also held a pre-filter and trace parser for the CBMC
//! oracle. SAF no longer ships or invokes CBMC -- CBMC is itself an SV-COMP
//! participant, and SAF bundles no competitor (user decision 2026-09-13) -- so those
//! items are gone and only the source-scanning half, which was never CBMC-specific,
//! remains.
use crate::fuzz::SCALAR_NONDET;
use std::collections::BTreeMap;

/// One scalar-integer `__VERIFIER_nondet_*` read recognized at a source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NondetSite {
    /// The nondet function called, e.g. `__VERIFIER_nondet_uint`.
    pub func_name: String,
    /// `true` when the call is part of a larger expression (`= f() & mask`), so the
    /// RAW value lives in a `return_value` temp; `false` for a bare
    /// `lhs = f();` where the raw value is assigned directly to `lhs`.
    pub compound: bool,
    /// The assignment's left-hand-side variable (used to pick the raw-value trace
    /// step for a simple assignment).
    pub lhs: String,
}

/// Build a map from 1-based source line number to the scalar-integer nondet read on
/// that line, for every line of `source` that assigns a `__VERIFIER_nondet_*` call.
///
/// A line qualifies when it contains `<lhs> = __VERIFIER_nondet_<type>( )` with
/// `<type>` in the scalar-integer family. `compound` is set when text other than a
/// terminating `;` follows the call (so the raw value is masked / combined into a
/// temp). Lines without a scalar nondet read are absent from the map.
#[must_use]
pub fn nondet_line_map(source: &str) -> BTreeMap<u32, NondetSite> {
    let mut map = BTreeMap::new();
    for (idx, line) in source.lines().enumerate() {
        let Some(site) = parse_nondet_line(line) else {
            continue;
        };
        // 1-based line numbers (matches CBMC's `line N`).
        #[allow(clippy::cast_possible_truncation)]
        let lineno = (idx + 1) as u32;
        map.insert(lineno, site);
    }
    map
}

/// Recognize a single `<lhs> = __VERIFIER_nondet_<scalar>()` assignment on one
/// source line, returning its [`NondetSite`]. Returns `None` for any other line.
fn parse_nondet_line(line: &str) -> Option<NondetSite> {
    const CALL: &str = "__VERIFIER_nondet_";
    let call_at = line.find(CALL)?;
    // The type token follows the prefix, up to '('.
    let after = &line[call_at + CALL.len()..];
    let paren = after.find('(')?;
    let type_token = &after[..paren];
    if type_token.is_empty()
        || !type_token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }
    let func_name = format!("{CALL}{type_token}");
    if !SCALAR_NONDET.iter().any(|(name, _)| *name == func_name) {
        return None;
    }
    // There must be an assignment `=` before the call; the lhs is the token
    // immediately before it. Guard against `==` / `!=` / `<=` / `>=`.
    let before = &line[..call_at];
    let eq = before.rfind('=')?;
    let eq_prev = before.as_bytes().get(eq.wrapping_sub(1)).copied();
    let eq_next = before.as_bytes().get(eq + 1).copied();
    if matches!(eq_prev, Some(b'=' | b'!' | b'<' | b'>')) || eq_next == Some(b'=') {
        return None;
    }
    let lhs = before[..eq].split_whitespace().last()?.to_string();
    if lhs.is_empty() {
        return None;
    }
    // Compound iff anything but a terminating `;` follows the call's `()`.
    let close = after[paren..].find(')')?;
    let tail = after[paren + close + 1..].trim();
    let compound = !(tail.is_empty() || tail.starts_with(';'));
    Some(NondetSite {
        func_name,
        compound,
        lhs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- nondet_line_map / parse_nondet_line ---

    #[test]
    fn recognizes_simple_scalar_assignment() {
        let s = parse_nondet_line("  input_44 = __VERIFIER_nondet_ushort();").unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_ushort");
        assert!(!s.compound);
        assert_eq!(s.lhs, "input_44");
    }

    #[test]
    fn recognizes_compound_masked_assignment() {
        let s = parse_nondet_line("  SORT_3 state_6 = __VERIFIER_nondet_ushort() & mask_SORT_3;")
            .unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_ushort");
        assert!(s.compound);
        assert_eq!(s.lhs, "state_6");
    }

    #[test]
    fn recognizes_declaration_with_init() {
        let s = parse_nondet_line("  int x = __VERIFIER_nondet_int();").unwrap();
        assert_eq!(s.func_name, "__VERIFIER_nondet_int");
        assert!(!s.compound);
        assert_eq!(s.lhs, "x");
    }

    #[test]
    fn ignores_non_scalar_and_equality_and_bare_call() {
        assert!(parse_nondet_line("  double d = __VERIFIER_nondet_double();").is_none());
        assert!(parse_nondet_line("  if (x == __VERIFIER_nondet_int()) {").is_none());
        // A bare call with no assignment target.
        assert!(parse_nondet_line("  __VERIFIER_nondet_int();").is_none());
        assert!(parse_nondet_line("  int y = 3;").is_none());
    }

    #[test]
    fn line_map_is_one_based_and_selective() {
        let src = "int main(){\n  int a = __VERIFIER_nondet_int();\n  int b = 5;\n}\n";
        let m = nondet_line_map(src);
        assert_eq!(m.len(), 1);
        assert!(m.contains_key(&2));
        assert_eq!(m[&2].lhs, "a");
    }
}
