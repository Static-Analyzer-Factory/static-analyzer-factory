//! Predefined sanitizer selectors for common sanitization patterns.

use super::Selector;

/// Common sanitizer selector presets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanitizerSelector {
    /// Return values of specific functions (sanitization functions).
    FunctionReturns(Vec<String>),
    /// Custom selectors.
    Custom(Vec<Selector>),
}

impl SanitizerSelector {
    /// Convert to a list of selectors.
    #[must_use]
    pub fn to_selectors(&self) -> Vec<Selector> {
        match self {
            Self::FunctionReturns(funcs) => funcs
                .iter()
                .map(|f| Selector::function_return(f.as_str()))
                .collect(),
            Self::Custom(selectors) => selectors.clone(),
        }
    }

    /// SQL escaping sanitizers.
    #[must_use]
    pub fn sql_escape() -> Self {
        Self::FunctionReturns(vec![
            "mysql_real_escape_string".to_string(),
            "sqlite3_mprintf".to_string(),
            "PQescapeLiteral".to_string(),
            "PQescapeIdentifier".to_string(),
            "PQescapeStringConn".to_string(),
        ])
    }

    /// HTML escaping sanitizers.
    #[must_use]
    pub fn html_escape() -> Self {
        Self::FunctionReturns(vec![
            "htmlspecialchars".to_string(),
            "htmlentities".to_string(),
            "escape_html".to_string(),
            "html_escape".to_string(),
        ])
    }

    /// Shell escaping sanitizers.
    #[must_use]
    pub fn shell_escape() -> Self {
        Self::FunctionReturns(vec![
            "escapeshellarg".to_string(),
            "escapeshellcmd".to_string(),
            "shlex_quote".to_string(),
        ])
    }

    /// Path sanitizers.
    #[must_use]
    pub fn path_sanitize() -> Self {
        Self::FunctionReturns(vec![
            "realpath".to_string(),
            "canonicalize_file_name".to_string(),
            "basename".to_string(),
        ])
    }

    /// Input validation functions.
    #[must_use]
    pub fn input_validation() -> Self {
        Self::FunctionReturns(vec![
            "atoi".to_string(),
            "atol".to_string(),
            "atof".to_string(),
            "strtol".to_string(),
            "strtoul".to_string(),
            "strtoll".to_string(),
            "strtoull".to_string(),
            "strtod".to_string(),
            "strtof".to_string(),
        ])
    }

    /// String length limiting functions.
    #[must_use]
    pub fn length_limit() -> Self {
        Self::FunctionReturns(vec![
            "strncpy".to_string(),
            "strncat".to_string(),
            "snprintf".to_string(),
            "strlcpy".to_string(),
            "strlcat".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_returns_to_selectors() {
        let sel =
            SanitizerSelector::FunctionReturns(vec!["escape".to_string(), "sanitize".to_string()]);
        let selectors = sel.to_selectors();
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn sql_escape_sanitizers() {
        let sel = SanitizerSelector::sql_escape();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
        // Should include mysql_real_escape_string
        assert!(selectors.iter().any(|s| matches!(
            s,
            Selector::FunctionReturn { function } if function == "mysql_real_escape_string"
        )));
    }

    #[test]
    fn html_escape_sanitizers() {
        let sel = SanitizerSelector::html_escape();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
    }

    #[test]
    fn shell_escape_sanitizers() {
        let sel = SanitizerSelector::shell_escape();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
    }

    #[test]
    fn path_sanitizers() {
        let sel = SanitizerSelector::path_sanitize();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
        // Should include realpath
        assert!(selectors.iter().any(|s| matches!(
            s,
            Selector::FunctionReturn { function } if function == "realpath"
        )));
    }

    #[test]
    fn input_validation_sanitizers() {
        let sel = SanitizerSelector::input_validation();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
    }
}
