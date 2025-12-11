//! Linter support modules

pub mod eslint;
pub mod markdownlint;
pub mod plugin_architecture;
pub mod prettier;

use crate::core::RuleMatch;
use std::collections::HashMap;

/// Linter handler trait
pub trait LinterHandler: Send + Sync {
    /// Check if this handler can process the given input
    fn can_handle(&self, input: &str) -> bool;

    /// Extract rules from linter output
    fn extract_rules(&self, input: &str) -> Vec<RuleMatch>;

    /// Get the name of the linter
    fn get_linter_name(&self) -> &str;

    /// Get configuration file patterns for this linter
    fn get_config_file_patterns(&self) -> Vec<&str>;

    /// Get the rule format for this linter
    fn get_rule_format(&self) -> &str;
}

/// Linter registry for managing multiple linters
pub struct LinterRegistry {
    handlers: HashMap<String, Box<dyn LinterHandler>>,
}

impl LinterRegistry {
    /// Create a new linter registry
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
        };

        // Register built-in linters
        registry.register_linter(
            "markdownlint",
            Box::new(markdownlint::MarkdownlintHandler::new()),
        );
        registry.register_linter("eslint", Box::new(eslint::ESLintHandler::new()));
        registry.register_linter("prettier", Box::new(prettier::PrettierHandler::new()));

        registry
    }

    /// Register a linter handler
    pub fn register_linter(&mut self, name: &str, handler: Box<dyn LinterHandler>) {
        self.handlers.insert(name.to_string(), handler);
    }

    /// Get a linter handler by name
    pub fn get_handler(&self, name: &str) -> Option<&dyn LinterHandler> {
        self.handlers.get(name).map(std::convert::AsRef::as_ref)
    }

    /// Get all registered linter names
    pub fn get_linter_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Find the appropriate handler for given input
    pub fn find_handler(&self, input: &str) -> Option<&dyn LinterHandler> {
        // Prioritize handlers based on input content
        let input_lower = input.to_lowercase();

        // Check for prettier-specific patterns first
        if input_lower.contains("prettier") {
            if let Some(handler) = self.handlers.get("prettier") {
                if handler.can_handle(input) {
                    return Some(handler.as_ref());
                }
            }
        }

        // Check for eslint-specific patterns
        if input_lower.contains("eslint") || input.contains("error") || input.contains("warn") {
            if let Some(handler) = self.handlers.get("eslint") {
                if handler.can_handle(input) {
                    return Some(handler.as_ref());
                }
            }
        }

        // Check for markdownlint-specific patterns
        if input_lower.contains("md") || input.contains("markdown") {
            if let Some(handler) = self.handlers.get("markdownlint") {
                if handler.can_handle(input) {
                    return Some(handler.as_ref());
                }
            }
        }

        // Fallback: check all handlers in order
        for handler in self.handlers.values() {
            if handler.can_handle(input) {
                return Some(handler.as_ref());
            }
        }

        None
    }

    /// Extract rules using the appropriate handler
    pub fn extract_rules(&self, input: &str) -> Vec<RuleMatch> {
        let mut all_rules = Vec::new();

        // Split input by lines and process each line
        for line in input.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                continue;
            }

            if let Some(handler) = self.find_handler(trimmed_line) {
                let rules = handler.extract_rules(trimmed_line);
                all_rules.extend(rules);
            }
        }

        all_rules
    }

    /// Get handler for specific linter
    pub fn get_handler_for_linter(&self, linter_name: &str) -> Option<&dyn LinterHandler> {
        self.get_handler(linter_name)
    }

    /// Check if a linter is supported
    pub fn is_linter_supported(&self, linter_name: &str) -> bool {
        self.handlers.contains_key(linter_name)
    }

    /// Get supported linters with their capabilities
    pub fn get_supported_linters(&self) -> HashMap<String, LinterInfo> {
        let mut info = HashMap::new();

        for (name, handler) in &self.handlers {
            info.insert(
                name.clone(),
                LinterInfo {
                    name: name.clone(),
                    rule_format: handler.get_rule_format().to_string(),
                    config_files: handler
                        .get_config_file_patterns()
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                    supported: true,
                },
            );
        }

        info
    }
}

impl Default for LinterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a linter
#[derive(Debug, Clone)]
pub struct LinterInfo {
    /// Linter name
    pub name: String,
    /// Canonical rule format (e.g., `MDxxx`)
    pub rule_format: String,
    /// Supported configuration files
    pub config_files: Vec<String>,
    /// Whether this linter is supported in the current build
    pub supported: bool,
}

/// Linter factory for creating handlers
pub struct LinterFactory;

impl LinterFactory {
    /// Create a handler for the specified linter
    pub fn create_handler(linter_name: &str) -> Option<Box<dyn LinterHandler>> {
        match linter_name {
            "markdownlint" => Some(Box::new(markdownlint::MarkdownlintHandler::new())),
            "eslint" => Some(Box::new(eslint::ESLintHandler::new())),
            "prettier" => Some(Box::new(prettier::PrettierHandler::new())),
            _ => None,
        }
    }

    /// Get all available linter names
    pub fn get_available_linters() -> Vec<String> {
        vec![
            "markdownlint".to_string(),
            "eslint".to_string(),
            "prettier".to_string(),
        ]
    }

    /// Validate linter name
    pub fn is_valid_linter(linter_name: &str) -> bool {
        Self::get_available_linters().contains(&linter_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_registry_creation() {
        let registry = LinterRegistry::new();
        assert!(registry.is_linter_supported("markdownlint"));
        assert!(registry.is_linter_supported("eslint"));
        assert!(registry.is_linter_supported("prettier"));
        assert!(!registry.is_linter_supported("unknown"));
    }

    #[test]
    fn test_find_handler() {
        let registry = LinterRegistry::new();

        // Test markdownlint
        let input = "MD033: No inline HTML";
        let handler = registry.find_handler(input);
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().get_linter_name(), "markdownlint");

        // Test eslint
        let input = "1:10 error no-unused-vars 'x' is assigned a value but never used";
        let handler = registry.find_handler(input);
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().get_linter_name(), "eslint");

        // Test prettier
        let input = "prettier/prettier: Missing semicolon";
        let handler = registry.find_handler(input);
        assert!(handler.is_some());
        assert_eq!(handler.unwrap().get_linter_name(), "prettier");
    }

    #[test]
    fn test_extract_rules() {
        let registry = LinterRegistry::new();

        let input = "MD033: No inline HTML";
        let rules = registry.extract_rules(input);
        assert!(!rules.is_empty());
        assert_eq!(rules[0].linter, "markdownlint");
        assert_eq!(rules[0].rule_id, "MD033");
    }

    #[test]
    fn test_linter_factory() {
        assert!(LinterFactory::is_valid_linter("markdownlint"));
        assert!(LinterFactory::is_valid_linter("eslint"));
        assert!(LinterFactory::is_valid_linter("prettier"));
        assert!(!LinterFactory::is_valid_linter("unknown"));

        let available = LinterFactory::get_available_linters();
        assert!(available.contains(&"markdownlint".to_string()));
        assert!(available.contains(&"eslint".to_string()));
        assert!(available.contains(&"prettier".to_string()));
    }

    #[test]
    fn test_create_handler() {
        assert!(LinterFactory::create_handler("markdownlint").is_some());
        assert!(LinterFactory::create_handler("eslint").is_some());
        assert!(LinterFactory::create_handler("prettier").is_some());
        assert!(LinterFactory::create_handler("unknown").is_none());
    }

    #[test]
    fn test_supported_linters() {
        let registry = LinterRegistry::new();
        let supported = registry.get_supported_linters();

        assert!(supported.contains_key("markdownlint"));
        assert!(supported.contains_key("eslint"));
        assert!(supported.contains_key("prettier"));

        let markdownlint_info = &supported["markdownlint"];
        assert_eq!(markdownlint_info.name, "markdownlint");
        assert!(markdownlint_info.supported);
    }
}
