//! SV-COMP property kinds and compilation options.
//!
//! These enums are the shared vocabulary of the SV-COMP engine: the property
//! being verified, the source language, and the data model that selects the
//! clang target. They are used by the analysis engine (`property.rs`), the blind
//! `saf verify` CLI, and the self-grading harness in `saf-bench`.
//!
//! Property *recognition* has two entry points: [`Property::from_prp`] parses the
//! real `.prp` `CHECK(... LTL ...)` form (used by the competition CLI), while
//! [`Property::from_property_file`] matches the filename substring (retained for
//! the harness's YAML task loader). The competition path must use `from_prp`.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// SV-COMP property types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Property {
    /// `unreach-call`: `reach_error()` is never called.
    UnreachCall,

    /// `valid-memsafety`: No memory safety violations (valid-free, valid-deref, valid-memtrack).
    ValidMemsafety,

    /// `valid-memcleanup`: All memory is freed (no leaks).
    ValidMemcleanup,

    /// `no-overflow`: No signed integer overflows.
    NoOverflow,

    /// `no-data-race`: No data races in concurrent programs.
    NoDataRace,

    /// `termination`: Program terminates.
    Termination,

    /// Coverage property (not a verification property).
    Coverage,

    /// Unknown/unsupported property.
    Unknown,
}

impl Property {
    /// Parse a property from its file path or content.
    pub fn from_property_file(path: &Path) -> Self {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        if filename.contains("unreach-call") {
            Self::UnreachCall
        } else if filename.contains("valid-memsafety") {
            Self::ValidMemsafety
        } else if filename.contains("valid-memcleanup") {
            Self::ValidMemcleanup
        } else if filename.contains("no-overflow") {
            Self::NoOverflow
        } else if filename.contains("no-data-race") {
            Self::NoDataRace
        } else if filename.contains("termination") {
            Self::Termination
        } else if filename.contains("coverage") {
            Self::Coverage
        } else {
            Self::Unknown
        }
    }

    /// Human-readable name of the property.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnreachCall => "unreach-call",
            Self::ValidMemsafety => "valid-memsafety",
            Self::ValidMemcleanup => "valid-memcleanup",
            Self::NoOverflow => "no-overflow",
            Self::NoDataRace => "no-data-race",
            Self::Termination => "termination",
            Self::Coverage => "coverage",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true if SAF supports this property.
    pub fn is_supported(&self) -> bool {
        matches!(
            self,
            Self::UnreachCall
                | Self::ValidMemsafety
                | Self::ValidMemcleanup
                | Self::NoOverflow
                | Self::NoDataRace
        )
    }

    /// Parse the SV-COMP property from the **contents** of a `.prp` file.
    ///
    /// Keys on the actual `CHECK( init(main()), LTL(...) )` form, never the
    /// filename: SV-COMP passes the property as a first-class parameter and
    /// organizers may rename files, so filename inference is both non-compliant
    /// and unreliable. A memory-safety `.prp` carries three `CHECK` lines
    /// (`valid-free`, `valid-deref`, `valid-memtrack`); any of them maps to
    /// [`Property::ValidMemsafety`]. `valid-memcleanup` is a distinct property and
    /// is tested first (it shares the `valid-mem` prefix). The distinctive tokens
    /// only occur inside the `LTL(...)` bodies of the CHECK lines.
    ///
    /// Returns `None` when no known property expression is recognized; the caller
    /// should then emit `unknown`.
    pub fn from_prp(contents: &str) -> Option<Self> {
        if contents.contains("valid-memcleanup") {
            Some(Self::ValidMemcleanup)
        } else if contents.contains("reach_error") || contents.contains("__VERIFIER_error") {
            Some(Self::UnreachCall)
        } else if contents.contains("valid-free")
            || contents.contains("valid-deref")
            || contents.contains("valid-memtrack")
        {
            Some(Self::ValidMemsafety)
        } else if contents.contains("overflow") {
            Some(Self::NoOverflow)
        } else if contents.contains("data-race") {
            Some(Self::NoDataRace)
        } else if contents.contains("F end") {
            Some(Self::Termination)
        } else {
            None
        }
    }
}

/// Programming language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    C,
    Cpp,
}

/// Data model (affects pointer and integer sizes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DataModel {
    /// 32-bit pointers, 32-bit int, 32-bit long
    ILP32,

    /// 64-bit pointers, 32-bit int, 64-bit long
    #[default]
    LP64,
}

impl DataModel {
    /// Returns the clang flag for this data model.
    pub fn clang_flag(&self) -> &'static str {
        match self {
            Self::ILP32 => "-m32",
            Self::LP64 => "-m64",
        }
    }

    /// Clang flags for a native-replay compile of this data model.
    ///
    /// LP64 is just `-m64`. **ILP32 additionally forces SSE floating point**
    /// (`-msse2 -mfpmath=sse`): the default `-m32` code generator emits x87 FPU
    /// instructions whose **80-bit extended-precision** intermediates make
    /// `double`/`float` arithmetic diverge from the IEEE-754 single/double semantics
    /// SV-COMP (and the LP64 build) assume. That excess precision spuriously fails
    /// floating-point equality assertions during native replay, fabricating a wrong
    /// `false(unreach-call)` on labeled-TRUE FP tasks
    /// (`floats-esbmc-regression/nearbyint2`, `rint2`). Pinning SSE makes the ILP32
    /// replay evaluate `double`/`float` at their true IEEE widths, matching the
    /// reference semantics — a soundness fix, not an abstain, so genuinely-buggy FP
    /// tasks still confirm. SSE2 is universally available on the x86-64 host that
    /// runs `-m32`, so this never fails to compile.
    #[must_use]
    pub fn clang_flags(&self) -> &'static [&'static str] {
        match self {
            Self::ILP32 => &["-m32", "-msse2", "-mfpmath=sse"],
            Self::LP64 => &["-m64"],
        }
    }
}

#[cfg(test)]
mod prp_tests {
    use super::*;

    #[test]
    fn parses_unreach_call_reach_error() {
        let prp = "CHECK( init(main()), LTL(G ! call(reach_error())) )";
        assert_eq!(Property::from_prp(prp), Some(Property::UnreachCall));
    }

    #[test]
    fn parses_unreach_call_verifier_error() {
        let prp = "CHECK( init(main()), LTL(G ! call(__VERIFIER_error())) )";
        assert_eq!(Property::from_prp(prp), Some(Property::UnreachCall));
    }

    #[test]
    fn parses_memsafety_three_checks() {
        // A real valid-memsafety.prp carries three CHECK lines.
        let prp = "CHECK( init(main()), LTL(G valid-free) )\n\
                   CHECK( init(main()), LTL(G valid-deref) )\n\
                   CHECK( init(main()), LTL(G valid-memtrack) )\n";
        assert_eq!(Property::from_prp(prp), Some(Property::ValidMemsafety));
    }

    #[test]
    fn parses_memcleanup_distinct_from_memsafety() {
        let prp = "CHECK( init(main()), LTL(G valid-memcleanup) )";
        assert_eq!(Property::from_prp(prp), Some(Property::ValidMemcleanup));
    }

    #[test]
    fn parses_no_overflow() {
        let prp = "CHECK( init(main()), LTL(G ! overflow) )";
        assert_eq!(Property::from_prp(prp), Some(Property::NoOverflow));
    }

    #[test]
    fn parses_no_data_race() {
        let prp = "CHECK( init(main()), LTL(G ! data-race) )";
        assert_eq!(Property::from_prp(prp), Some(Property::NoDataRace));
    }

    #[test]
    fn parses_termination() {
        let prp = "CHECK( init(main()), LTL(F end) )";
        assert_eq!(Property::from_prp(prp), Some(Property::Termination));
    }

    #[test]
    fn unrecognized_returns_none() {
        assert_eq!(
            Property::from_prp("CHECK( init(main()), LTL(G something-else) )"),
            None
        );
        assert_eq!(Property::from_prp(""), None);
    }

    #[test]
    fn keys_on_content_not_filename() {
        // The filename would say "unreach-call" but the body is memcleanup:
        // from_prp must follow the LTL body, returning ValidMemcleanup.
        let prp = "CHECK( init(main()), LTL(G valid-memcleanup) )";
        assert_eq!(Property::from_prp(prp), Some(Property::ValidMemcleanup));
    }
}
