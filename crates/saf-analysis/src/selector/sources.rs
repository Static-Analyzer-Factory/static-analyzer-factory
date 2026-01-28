//! Predefined source selectors for common taint sources.

use super::Selector;

/// Common source selector presets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SourceSelector {
    /// All function parameters (entry points for external data).
    #[default]
    AllParams,
    /// Parameters of specific functions.
    FunctionParams(Vec<String>),
    /// Return values of specific functions (e.g., `getenv`, `read`).
    FunctionReturns(Vec<String>),
    /// Specific global variables.
    Globals(Vec<String>),
    /// Custom selectors.
    Custom(Vec<Selector>),
}

impl SourceSelector {
    /// Convert to a list of selectors.
    #[must_use]
    pub fn to_selectors(&self) -> Vec<Selector> {
        match self {
            Self::AllParams => vec![Selector::function_param("*", None)],
            Self::FunctionParams(funcs) => funcs
                .iter()
                .map(|f| Selector::function_param(f.as_str(), None))
                .collect(),
            Self::FunctionReturns(funcs) => funcs
                .iter()
                .map(|f| Selector::function_return(f.as_str()))
                .collect(),
            Self::Globals(names) => names.iter().map(|n| Selector::global(n.as_str())).collect(),
            Self::Custom(selectors) => selectors.clone(),
        }
    }

    /// Common user input sources.
    #[must_use]
    pub fn user_input() -> Self {
        Self::FunctionReturns(vec![
            "getenv".to_string(),
            "read".to_string(),
            "fread".to_string(),
            "fgets".to_string(),
            "gets".to_string(),
            "scanf".to_string(),
            "fscanf".to_string(),
            "getchar".to_string(),
            "getline".to_string(),
            "recv".to_string(),
            "recvfrom".to_string(),
        ])
    }

    /// Network input sources.
    #[must_use]
    pub fn network_input() -> Self {
        Self::FunctionReturns(vec![
            "recv".to_string(),
            "recvfrom".to_string(),
            "recvmsg".to_string(),
            "read".to_string(), // When used on sockets
        ])
    }

    /// File input sources.
    #[must_use]
    pub fn file_input() -> Self {
        Self::FunctionReturns(vec![
            "fread".to_string(),
            "fgets".to_string(),
            "fgetc".to_string(),
            "getc".to_string(),
            "read".to_string(),
            "pread".to_string(),
        ])
    }

    /// Environment variable sources.
    #[must_use]
    pub fn environment() -> Self {
        Self::FunctionReturns(vec!["getenv".to_string(), "secure_getenv".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_params_to_selectors() {
        let sel = SourceSelector::AllParams;
        let selectors = sel.to_selectors();
        assert_eq!(selectors.len(), 1);
        assert!(matches!(
            &selectors[0],
            Selector::FunctionParam { function, index } if function == "*" && index.is_none()
        ));
    }

    #[test]
    fn function_params_to_selectors() {
        let sel = SourceSelector::FunctionParams(vec!["main".to_string(), "init".to_string()]);
        let selectors = sel.to_selectors();
        assert_eq!(selectors.len(), 2);
    }

    #[test]
    fn user_input_sources() {
        let sel = SourceSelector::user_input();
        let selectors = sel.to_selectors();
        assert!(!selectors.is_empty());
        // Should include getenv
        assert!(selectors.iter().any(|s| matches!(
            s,
            Selector::FunctionReturn { function } if function == "getenv"
        )));
    }

    #[test]
    fn default_is_all_params() {
        let sel = SourceSelector::default();
        assert_eq!(sel, SourceSelector::AllParams);
    }
}
