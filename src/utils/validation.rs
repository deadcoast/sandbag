//! Input validation utilities

use crate::core::{ConfigAction, ConfigEntry, ConfigScope, RuleMatch};
use crate::utils::ValidationResult;
use std::path::Path;

/// Input validator
pub struct InputValidator;

impl InputValidator {
    /// Validate rule input
    pub fn validate_rule_input(input: &str) -> ValidationResult {
        let mut result = ValidationResult::new();

        if input.trim().is_empty() {
            result.add_error("Rule input cannot be empty".to_string());
            return result;
        }

        if input.len() > 1000 {
            result.add_warning("Rule input is very long, consider using a file".to_string());
        }

        // Check for common patterns
        if !input.contains("MD") && !input.contains("ESLint") && !input.contains("pylint") {
            result.add_warning("Input doesn't contain common linter identifiers".to_string());
        }

        result
    }

    /// Validate rule ID format
    pub fn validate_rule_id(rule_id: &str) -> ValidationResult {
        let mut result = ValidationResult::new();

        if rule_id.trim().is_empty() {
            result.add_error("Rule ID cannot be empty".to_string());
            return result;
        }

        // Check for valid characters
        if !rule_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            result.add_error("Rule ID contains invalid characters".to_string());
        }

        // Check length
        if rule_id.len() > 50 {
            result.add_error("Rule ID is too long".to_string());
        }

        result
    }

    /// Validate configuration entry
    pub fn validate_config_entry(entry: &ConfigEntry) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate rule ID
        let rule_validation = Self::validate_rule_id(&entry.rule_id);
        if !rule_validation.is_ok() {
            for error in rule_validation.errors {
                result.add_error(error);
            }
        }

        // Validate action
        match entry.action {
            ConfigAction::Disable
            | ConfigAction::Ignore
            | ConfigAction::Modify
            | ConfigAction::Enable => {
                // Valid action
            }
        }

        // Validate scope
        match entry.scope {
            ConfigScope::Global
            | ConfigScope::FileSpecific
            | ConfigScope::Inline
            | ConfigScope::Directory => {
                // Valid scope
            }
        }

        result
    }

    /// Validate file path
    pub fn validate_file_path(path: &Path) -> ValidationResult {
        let mut result = ValidationResult::new();

        if !path.exists() {
            result.add_warning("File does not exist".to_string());
        }

        if path.is_dir() {
            result.add_error("Path is a directory, expected a file".to_string());
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy();
            let valid_extensions = ["json", "yaml", "yml", "toml", "ini"];

            if !valid_extensions.contains(&ext_str.as_ref()) {
                result.add_warning(format!("Unusual file extension: {ext_str}"));
            }
        } else {
            result.add_warning("File has no extension".to_string());
        }

        result
    }

    /// Validate linter name
    pub fn validate_linter_name(linter: &str) -> ValidationResult {
        let mut result = ValidationResult::new();

        if linter.trim().is_empty() {
            result.add_error("Linter name cannot be empty".to_string());
            return result;
        }

        let valid_linters = [
            "markdownlint",
            "eslint",
            "pylint",
            "ruff",
            "clippy",
            "stylelint",
        ];

        if !valid_linters.contains(&linter) {
            result.add_warning(format!("Unknown linter: {linter}"));
        }

        result
    }
}

/// Rule validation utilities
pub struct RuleValidator;

impl RuleValidator {
    /// Validate rule match
    pub fn validate_rule_match(rule_match: &RuleMatch) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate rule ID
        let rule_validation = InputValidator::validate_rule_id(&rule_match.rule_id);
        if !rule_validation.is_ok() {
            for error in rule_validation.errors {
                result.add_error(error);
            }
        }

        // Validate linter
        let linter_validation = InputValidator::validate_linter_name(&rule_match.linter);
        if !linter_validation.is_ok() {
            for error in linter_validation.errors {
                result.add_error(error);
            }
        }

        // Validate confidence
        if rule_match.confidence < 0.0 || rule_match.confidence > 1.0 {
            result.add_error("Confidence must be between 0.0 and 1.0".to_string());
        }

        if rule_match.confidence < 0.3 {
            result.add_warning("Very low confidence match".to_string());
        }

        result
    }

    /// Check if rule match is actionable
    pub fn is_actionable(rule_match: &RuleMatch) -> bool {
        rule_match.confidence >= 0.6
    }

    /// Get suggested action for rule match
    pub fn get_suggested_action(rule_match: &RuleMatch) -> ConfigAction {
        if rule_match.confidence >= 0.85 {
            ConfigAction::Disable
        } else if rule_match.confidence >= 0.6 {
            ConfigAction::Ignore
        } else {
            ConfigAction::Modify
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ExtractedContext, Severity};

    #[test]
    fn test_validate_rule_input() {
        // Valid input
        let result = InputValidator::validate_rule_input("MD033/no-inline-html: Inline HTML");
        assert!(result.is_ok());

        // Empty input
        let result = InputValidator::validate_rule_input("");
        assert!(!result.is_ok());

        // Very long input
        let long_input = "MD033".repeat(250); // 1250 characters, should trigger warning
        let result = InputValidator::validate_rule_input(&long_input);
        assert!(result.is_ok()); // Should be valid but with warning
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_validate_rule_id() {
        // Valid rule ID
        let result = InputValidator::validate_rule_id("MD033");
        assert!(result.is_ok());

        // Invalid rule ID with special characters
        let result = InputValidator::validate_rule_id("MD@33");
        assert!(!result.is_ok());

        // Empty rule ID
        let result = InputValidator::validate_rule_id("");
        assert!(!result.is_ok());
    }

    #[test]
    fn test_validate_config_entry() {
        let entry = ConfigEntry {
            rule_id: "MD033".to_string(),
            action: ConfigAction::Disable,
            scope: ConfigScope::Global,
            metadata: std::collections::HashMap::new(),
        };

        let result = InputValidator::validate_config_entry(&entry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rule_match() {
        let rule_match = RuleMatch {
            rule_id: "MD033".to_string(),
            linter: "markdownlint".to_string(),
            confidence: 0.9,
            context: ExtractedContext {
                file_path: None,
                line_number: None,
                column: None,
                message: None,
                severity: Some(Severity::Warning),
            },
            suggested_action: ConfigAction::Disable,
        };

        let result = RuleValidator::validate_rule_match(&rule_match);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_actionable() {
        let high_confidence = RuleMatch {
            rule_id: "MD033".to_string(),
            linter: "markdownlint".to_string(),
            confidence: 0.9,
            context: ExtractedContext {
                file_path: None,
                line_number: None,
                column: None,
                message: None,
                severity: Some(Severity::Warning),
            },
            suggested_action: ConfigAction::Disable,
        };

        let low_confidence = RuleMatch {
            rule_id: "MD033".to_string(),
            linter: "markdownlint".to_string(),
            confidence: 0.3,
            context: ExtractedContext {
                file_path: None,
                line_number: None,
                column: None,
                message: None,
                severity: Some(Severity::Warning),
            },
            suggested_action: ConfigAction::Disable,
        };

        assert!(RuleValidator::is_actionable(&high_confidence));
        assert!(!RuleValidator::is_actionable(&low_confidence));
    }
}
