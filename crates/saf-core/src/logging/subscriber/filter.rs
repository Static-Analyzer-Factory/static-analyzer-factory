//! `SAF_LOG` environment variable parser and filter matcher.
//!
//! Grammar:
//! ```text
//! SAF_LOG = "filter1,filter2,..."
//! filter  = [-]module[::phase][tag1,tag2]
//! module  = identifier | "*"
//! phase   = identifier | "*"
//! tag     = identifier
//! ```
//!
//! Special value `all` matches everything.

/// A parsed `SAF_LOG` filter specification.
#[derive(Debug)]
pub struct SafLogFilter {
    includes: Vec<FilterRule>,
    excludes: Vec<FilterRule>,
    active: bool,
}

#[derive(Debug)]
struct FilterRule {
    module: Option<String>,
    phase: Option<String>,
    tags: Option<Vec<String>>,
}

impl SafLogFilter {
    /// Create a no-op filter that matches nothing.
    #[must_use]
    pub fn none() -> Self {
        Self {
            includes: Vec::new(),
            excludes: Vec::new(),
            active: false,
        }
    }

    /// Returns true if this filter is active (`SAF_LOG` was set).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Parse a `SAF_LOG` spec string.
    #[must_use]
    pub fn parse(spec: &str) -> Self {
        let spec = spec.trim();
        if spec.is_empty() {
            return Self::none();
        }

        let mut includes = Vec::new();
        let mut excludes = Vec::new();

        // Split by commas, but respect brackets (commas inside [...] are tag separators)
        for part in split_respecting_brackets(spec) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (is_exclude, part) = if let Some(stripped) = part.strip_prefix('-') {
                (true, stripped)
            } else {
                (false, part)
            };

            let rule = Self::parse_rule(part);

            if is_exclude {
                excludes.push(rule);
            } else {
                includes.push(rule);
            }
        }

        Self {
            active: !includes.is_empty(),
            includes,
            excludes,
        }
    }

    fn parse_rule(spec: &str) -> FilterRule {
        // Handle "all"
        if spec == "all" {
            return FilterRule {
                module: None,
                phase: None,
                tags: None,
            };
        }

        // Extract tags: everything inside [...]
        let (path, tags) = if let Some(bracket_start) = spec.find('[') {
            let bracket_end = spec.find(']').unwrap_or(spec.len());
            let tag_str = &spec[bracket_start + 1..bracket_end];
            let tags: Vec<String> = tag_str.split(',').map(|t| t.trim().to_string()).collect();
            (&spec[..bracket_start], Some(tags))
        } else {
            (spec, None)
        };

        // Parse module::phase
        let (module, phase) = if let Some((m, p)) = path.split_once("::") {
            (
                if m == "*" { None } else { Some(m.to_string()) },
                if p == "*" { None } else { Some(p.to_string()) },
            )
        } else {
            // Module only (or wildcard)
            let module = if path == "*" {
                None
            } else {
                Some(path.to_string())
            };
            (module, None)
        };

        FilterRule {
            module,
            phase,
            tags,
        }
    }

    /// Check if the given module/phase/tag combination matches this filter.
    #[must_use]
    pub fn matches(&self, module: &str, phase: &str, tag: &str) -> bool {
        if !self.active {
            return false;
        }

        let included = self.includes.iter().any(|r| r.matches(module, phase, tag));
        if !included {
            return false;
        }

        let excluded = self.excludes.iter().any(|r| r.matches(module, phase, tag));
        !excluded
    }
}

/// Split a spec string by commas, but don't split inside `[...]` brackets.
fn split_respecting_brackets(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

impl FilterRule {
    fn matches(&self, module: &str, phase: &str, tag: &str) -> bool {
        // Check module
        if let Some(ref m) = self.module {
            if m != module {
                return false;
            }
        }
        // Check phase
        if let Some(ref p) = self.phase {
            if p != phase {
                return false;
            }
        }
        // Check tags
        if let Some(ref tags) = self.tags {
            if !tags.iter().any(|t| t == tag) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_matches_nothing() {
        let f = SafLogFilter::none();
        assert!(!f.matches("pta", "solve", "worklist"));
    }

    #[test]
    fn test_all_matches_everything() {
        let f = SafLogFilter::parse("all");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(f.matches("checker", "memleak", "reasoning"));
    }

    #[test]
    fn test_module_only() {
        let f = SafLogFilter::parse("pta");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(f.matches("pta", "constraint", "extract"));
        assert!(!f.matches("checker", "memleak", "reasoning"));
    }

    #[test]
    fn test_module_phase() {
        let f = SafLogFilter::parse("pta::solve");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(f.matches("pta", "solve", "convergence"));
        assert!(!f.matches("pta", "constraint", "extract"));
    }

    #[test]
    fn test_module_phase_tags() {
        let f = SafLogFilter::parse("pta::solve[worklist,scc]");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(f.matches("pta", "solve", "scc"));
        assert!(!f.matches("pta", "solve", "convergence"));
    }

    #[test]
    fn test_module_with_tags() {
        let f = SafLogFilter::parse("pta[convergence]");
        assert!(f.matches("pta", "solve", "convergence"));
        assert!(f.matches("pta", "constraint", "convergence"));
        assert!(!f.matches("pta", "solve", "worklist"));
    }

    #[test]
    fn test_wildcard_with_tags() {
        let f = SafLogFilter::parse("*[convergence]");
        assert!(f.matches("pta", "solve", "convergence"));
        assert!(f.matches("checker", "memleak", "convergence"));
        assert!(!f.matches("pta", "solve", "worklist"));
    }

    #[test]
    fn test_wildcard_module_phase() {
        let f = SafLogFilter::parse("*::*[stats]");
        assert!(f.matches("pta", "solve", "stats"));
        assert!(f.matches("callgraph", "refine", "stats"));
        assert!(!f.matches("pta", "solve", "worklist"));
    }

    #[test]
    fn test_multiple_filters() {
        let f = SafLogFilter::parse("pta::solve[worklist],checker[reasoning]");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(f.matches("checker", "memleak", "reasoning"));
        assert!(!f.matches("pta", "solve", "convergence"));
    }

    #[test]
    fn test_negation() {
        let f = SafLogFilter::parse("pta,-pta::solve[worklist]");
        assert!(f.matches("pta", "solve", "convergence"));
        assert!(f.matches("pta", "constraint", "extract"));
        assert!(!f.matches("pta", "solve", "worklist"));
    }

    #[test]
    fn test_all_minus() {
        let f = SafLogFilter::parse("all,-*[stats]");
        assert!(f.matches("pta", "solve", "worklist"));
        assert!(!f.matches("pta", "solve", "stats"));
        assert!(!f.matches("callgraph", "refine", "stats"));
    }

    #[test]
    fn test_empty_spec() {
        let f = SafLogFilter::parse("");
        assert!(!f.matches("pta", "solve", "worklist"));
        assert!(!f.is_active());
    }
}
