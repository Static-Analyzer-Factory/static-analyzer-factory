//! Name pattern matching for function specifications.
//!
//! Supports three matching modes:
//! - Exact match (default): `name: malloc`
//! - Glob pattern: `name: "glob:str*"`
//! - Regex pattern: `name: "regex:^mem(cpy|set|move)$"`

use regex::Regex;
use std::fmt;

/// A name pattern for matching function names.
#[derive(Debug, Clone)]
pub enum NamePattern {
    /// Exact string match.
    Exact(String),
    /// Glob pattern (using `glob` crate syntax).
    Glob(glob::Pattern),
    /// Regular expression pattern.
    Regex(Regex),
}

impl NamePattern {
    /// Parse a name string into a pattern.
    ///
    /// - `glob:pattern` → Glob pattern
    /// - `regex:pattern` → Regex pattern
    /// - anything else → Exact match
    ///
    /// # Errors
    /// Returns an error if glob or regex pattern is invalid.
    pub fn parse(name: &str) -> Result<Self, PatternError> {
        if let Some(pattern) = name.strip_prefix("glob:") {
            let glob_pattern = glob::Pattern::new(pattern).map_err(|e| PatternError::Glob {
                pattern: pattern.to_string(),
                message: e.to_string(),
            })?;
            Ok(Self::Glob(glob_pattern))
        } else if let Some(pattern) = name.strip_prefix("regex:") {
            let regex = Regex::new(pattern).map_err(|e| PatternError::Regex {
                pattern: pattern.to_string(),
                message: e.to_string(),
            })?;
            Ok(Self::Regex(regex))
        } else {
            Ok(Self::Exact(name.to_string()))
        }
    }

    /// Check if this pattern matches the given function name.
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        match self {
            Self::Exact(exact) => exact == name,
            Self::Glob(pattern) => pattern.matches(name),
            Self::Regex(regex) => regex.is_match(name),
        }
    }

    /// Check if this is an exact match pattern.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact(_))
    }

    /// Get the original pattern string (for display).
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Exact(s) => s,
            Self::Glob(p) => p.as_str(),
            Self::Regex(r) => r.as_str(),
        }
    }
}

impl PartialEq for NamePattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(a), Self::Exact(b)) => a == b,
            (Self::Glob(a), Self::Glob(b)) => a.as_str() == b.as_str(),
            (Self::Regex(a), Self::Regex(b)) => a.as_str() == b.as_str(),
            _ => false,
        }
    }
}

impl fmt::Display for NamePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(s) => write!(f, "{s}"),
            Self::Glob(p) => write!(f, "glob:{}", p.as_str()),
            Self::Regex(r) => write!(f, "regex:{}", r.as_str()),
        }
    }
}

/// Errors that can occur when parsing patterns.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PatternError {
    /// Invalid glob pattern.
    #[error("invalid glob pattern '{pattern}': {message}")]
    Glob {
        /// The pattern that failed to parse.
        pattern: String,
        /// Error message from glob parser.
        message: String,
    },
    /// Invalid regex pattern.
    #[error("invalid regex pattern '{pattern}': {message}")]
    Regex {
        /// The pattern that failed to parse.
        pattern: String,
        /// Error message from regex parser.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let pattern = NamePattern::parse("malloc").unwrap();
        assert!(pattern.is_exact());
        assert!(pattern.matches("malloc"));
        assert!(!pattern.matches("calloc"));
        assert!(!pattern.matches("malloc_usable_size"));
    }

    #[test]
    fn test_glob_prefix_match() {
        let pattern = NamePattern::parse("glob:str*").unwrap();
        assert!(!pattern.is_exact());
        assert!(pattern.matches("strlen"));
        assert!(pattern.matches("strcpy"));
        assert!(pattern.matches("strcat"));
        assert!(!pattern.matches("malloc"));
    }

    #[test]
    fn test_glob_suffix_match() {
        let pattern = NamePattern::parse("glob:*alloc").unwrap();
        assert!(pattern.matches("malloc"));
        assert!(pattern.matches("calloc"));
        assert!(pattern.matches("realloc"));
        assert!(!pattern.matches("free"));
    }

    #[test]
    fn test_glob_complex() {
        let pattern = NamePattern::parse("glob:mem[cs]*").unwrap();
        assert!(pattern.matches("memcpy"));
        assert!(pattern.matches("memset"));
        assert!(pattern.matches("memcmp"));
        assert!(!pattern.matches("memmove"));
    }

    #[test]
    fn test_regex_alternation() {
        let pattern = NamePattern::parse("regex:^mem(cpy|set|move)$").unwrap();
        assert!(pattern.matches("memcpy"));
        assert!(pattern.matches("memset"));
        assert!(pattern.matches("memmove"));
        assert!(!pattern.matches("memcmp"));
        assert!(!pattern.matches("memory"));
    }

    #[test]
    fn test_regex_prefix() {
        let pattern = NamePattern::parse("regex:^pthread_").unwrap();
        assert!(pattern.matches("pthread_create"));
        assert!(pattern.matches("pthread_join"));
        assert!(pattern.matches("pthread_mutex_lock"));
        assert!(!pattern.matches("create_pthread"));
    }

    #[test]
    fn test_invalid_glob() {
        let result = NamePattern::parse("glob:[invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PatternError::Glob { .. }));
    }

    #[test]
    fn test_invalid_regex() {
        let result = NamePattern::parse("regex:[invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PatternError::Regex { .. }));
    }

    #[test]
    fn test_pattern_display() {
        assert_eq!(NamePattern::parse("malloc").unwrap().to_string(), "malloc");
        assert_eq!(
            NamePattern::parse("glob:str*").unwrap().to_string(),
            "glob:str*"
        );
        assert_eq!(
            NamePattern::parse("regex:^mem").unwrap().to_string(),
            "regex:^mem"
        );
    }

    #[test]
    fn test_pattern_equality() {
        let p1 = NamePattern::parse("malloc").unwrap();
        let p2 = NamePattern::parse("malloc").unwrap();
        let p3 = NamePattern::parse("calloc").unwrap();
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);

        let g1 = NamePattern::parse("glob:str*").unwrap();
        let g2 = NamePattern::parse("glob:str*").unwrap();
        let g3 = NamePattern::parse("glob:mem*").unwrap();
        assert_eq!(g1, g2);
        assert_ne!(g1, g3);
    }
}
