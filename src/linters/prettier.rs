//! Prettier linter support

use crate::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};
use crate::linters::LinterHandler;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Prettier linter handler
#[derive(Debug, Clone)]
pub struct PrettierHandler {
    rule_patterns: Vec<Regex>,
    formatting_rules: HashMap<String, String>,
}

impl PrettierHandler {
    /// Create a new Prettier handler
    pub fn new() -> Self {
        let mut handler = Self {
            rule_patterns: Vec::new(),
            formatting_rules: HashMap::new(),
        };

        handler.initialize_patterns();
        handler.initialize_formatting_rules();

        handler
    }

    /// Initialize regex patterns for Prettier rules
    fn initialize_patterns(&mut self) {
        // Prettier rule patterns
        let patterns = vec![
            // Prettier formatting issues
            r"prettier/prettier:\s*(.+)",
            // Code style violations
            r"Code style issues found:\s*(.+)",
            // Formatting errors
            r"Formatting error:\s*(.+)",
            // Simple prettier rule
            r"prettier:\s*(.+)",
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.rule_patterns.push(regex);
            }
        }
    }

    /// Initialize common Prettier formatting rules
    fn initialize_formatting_rules(&mut self) {
        let m = &mut self.formatting_rules;
        m.insert("semi".to_string(), "Add or remove semicolons".to_string());
        m.insert("singleQuote".to_string(), "Use single quotes".to_string());
        m.insert(
            "trailingComma".to_string(),
            "Add trailing commas".to_string(),
        );
        m.insert("printWidth".to_string(), "Line width exceeded".to_string());
        m.insert(
            "tabWidth".to_string(),
            "Incorrect indentation width".to_string(),
        );
        m.insert("useTabs".to_string(), "Use tabs or spaces".to_string());
        m.insert(
            "bracketSpacing".to_string(),
            "Spaces in brackets".to_string(),
        );
        m.insert(
            "arrowParens".to_string(),
            "Parentheses around arrow function parameters".to_string(),
        );
        m.insert(
            "endOfLine".to_string(),
            "Inconsistent line endings".to_string(),
        );
        m.insert(
            "quoteProps".to_string(),
            "Property name quoting".to_string(),
        );
        m.insert("jsxSingleQuote".to_string(), "JSX quote style".to_string());
        m.insert(
            "proseWrap".to_string(),
            "Markdown prose wrapping".to_string(),
        );
        m.insert(
            "htmlWhitespaceSensitivity".to_string(),
            "HTML whitespace sensitivity".to_string(),
        );
        m.insert(
            "vueIndentScriptAndStyle".to_string(),
            "Indent Vue script/style".to_string(),
        );
        m.insert(
            "embeddedLanguageFormatting".to_string(),
            "Format contents of template strings".to_string(),
        );
    }

    /// Extract rule ID from Prettier output
    fn extract_rule_id(&self, input: &str) -> Option<String> {
        for pattern in &self.rule_patterns {
            if let Some(captures) = pattern.captures(input) {
                if captures.len() >= 2 {
                    let message = captures.get(1).unwrap().as_str();

                    // For general prettier output, return the general rule
                    if input.contains("prettier/prettier:") {
                        return Some("prettier/prettier".to_string());
                    }

                    // Try to identify specific formatting rule with more robust matching
                    let msg_lower = message.to_lowercase();
                    let mut candidates: Vec<&str> = Vec::new();
                    for rule in self.formatting_rules.keys() {
                        if msg_lower.contains(&rule.to_lowercase()) {
                            candidates.push(rule);
                        }
                    }
                    // Heuristic disambiguation
                    match candidates.len() {
                        1 => return Some(format!("prettier/{}", candidates[0])),
                        n if n > 1 => {
                            // Prefer exact token matches like 'printwidth' or 'semi'
                            if let Some(exact) = candidates.iter().copied().find(|r| {
                                msg_lower
                                    .split(|c: char| !c.is_alphanumeric())
                                    .any(|t| t == r.to_lowercase())
                            }) {
                                return Some(format!("prettier/{exact}"));
                            }
                            return Some("prettier/prettier".to_string());
                        }
                        _ => {}
                    }

                    // Default to general prettier rule
                    return Some("prettier/prettier".to_string());
                }
            }
        }

        None
    }

    /// Extract context information from Prettier output
    fn extract_context(&self, input: &str) -> ExtractedContext {
        for pattern in &self.rule_patterns {
            if let Some(captures) = pattern.captures(input) {
                if captures.len() >= 2 {
                    let message = captures.get(1).unwrap().as_str().trim().to_string();

                    return ExtractedContext {
                        file_path: None, // Will be set by caller
                        line_number: None,
                        column: None,
                        message: Some(message),
                        severity: Some(Severity::Warning), // Prettier issues are typically warnings
                    };
                }
            }
        }

        ExtractedContext {
            file_path: None,
            line_number: None,
            column: None,
            message: Some(input.trim().to_string()),
            severity: Some(Severity::Warning),
        }
    }

    /// Calculate confidence score for Prettier rule
    fn calculate_confidence(&self, input: &str) -> f64 {
        let mut confidence: f64 = 0.6; // Base confidence for Prettier

        // Increase confidence for well-formed Prettier output
        if input.contains("prettier") {
            confidence += 0.2;
        }

        // Increase confidence if we can extract a rule ID
        if self.extract_rule_id(input).is_some() {
            confidence += 0.2;
        }

        // Increase confidence for formatting-related keywords
        let formatting_keywords = ["format", "style", "semi", "quote", "comma", "width", "tab"];
        for keyword in &formatting_keywords {
            if input.to_lowercase().contains(keyword) {
                confidence += 0.1;
                break;
            }
        }

        confidence.min(1.0)
    }

    /// Suggest action based on Prettier rule
    #[allow(clippy::unused_self)]
    fn suggest_action(&self, rule_id: &str) -> ConfigAction {
        // Prettier rules are typically about formatting, so we usually want to modify
        // rather than disable, unless it's a specific case
        if rule_id.contains("endOfLine") || rule_id.contains("printWidth") {
            ConfigAction::Modify
        } else {
            ConfigAction::Ignore // Let Prettier handle formatting
        }
    }

    /// Get Prettier configuration options
    pub fn get_configuration_options(&self) -> HashMap<String, String> {
        self.formatting_rules.clone()
    }

    /// Validate Prettier configuration
    pub fn validate_config(&self, config: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Basic JSON validation
        if serde_json::from_str::<serde_json::Value>(config).is_err() {
            errors.push("Invalid JSON format".to_string());
        }

        // Check for common Prettier configuration issues
        if !config.contains("semi") && !config.contains("singleQuote") {
            errors.push("Missing common Prettier options".to_string());
        }

        Ok(errors)
    }
}

impl Default for PrettierHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LinterHandler for PrettierHandler {
    fn can_handle(&self, input: &str) -> bool {
        // Check if input looks like Prettier output
        input.contains("prettier")
            || input.contains("format")
            || input.contains("style")
            || self.extract_rule_id(input).is_some()
    }

    fn extract_rules(&self, input: &str) -> Vec<RuleMatch> {
        let mut rules = Vec::new();

        // Split input into lines and process each line
        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(rule_id) = self.extract_rule_id(trimmed) {
                let context = self.extract_context(trimmed);
                let confidence = self.calculate_confidence(trimmed);
                let suggested_action = self.suggest_action(&rule_id);

                rules.push(RuleMatch {
                    rule_id,
                    linter: "prettier".to_string(),
                    confidence,
                    context,
                    suggested_action,
                });
            }
        }

        rules
    }

    fn get_linter_name(&self) -> &'static str {
        "prettier"
    }

    fn get_config_file_patterns(&self) -> Vec<&str> {
        vec![
            ".prettierrc",
            ".prettierrc.js",
            ".prettierrc.json",
            ".prettierrc.yaml",
            ".prettierrc.yml",
            "prettier.config.js",
            "package.json", // Prettier config can be in package.json
        ]
    }

    fn get_rule_format(&self) -> &'static str {
        "prettier/rule-name"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prettier_handler_creation() {
        let handler = PrettierHandler::new();
        assert!(!handler.rule_patterns.is_empty());
        assert!(!handler.formatting_rules.is_empty());
    }

    #[test]
    fn test_can_handle_prettier_output() {
        let handler = PrettierHandler::new();

        assert!(handler.can_handle("prettier/prettier: Missing semicolon"));
        assert!(handler.can_handle("Code style issues found: Line too long"));
        assert!(handler.can_handle("Formatting error: Inconsistent quotes"));
        assert!(!handler.can_handle("MD033: No inline HTML"));
    }

    #[test]
    fn test_extract_rule_id() {
        let handler = PrettierHandler::new();

        let input = "prettier/prettier: Missing semicolon";
        assert_eq!(
            handler.extract_rule_id(input),
            Some("prettier/prettier".to_string())
        );

        let input = "Code style issues found: Line width exceeded";
        assert_eq!(
            handler.extract_rule_id(input),
            Some("prettier/prettier".to_string())
        );
    }

    #[test]
    fn test_extract_context() {
        let handler = PrettierHandler::new();

        let input = "prettier/prettier: Missing semicolon";
        let context = handler.extract_context(input);

        assert_eq!(context.message, Some("Missing semicolon".to_string()));
        assert_eq!(context.severity, Some(Severity::Warning));
    }

    #[test]
    fn test_extract_rules() {
        let handler = PrettierHandler::new();

        let input = "prettier/prettier: Missing semicolon\nCode style issues found: Line too long";
        let rules = handler.extract_rules(input);

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].rule_id, "prettier/prettier");
        assert_eq!(rules[0].linter, "prettier");
        assert_eq!(rules[1].rule_id, "prettier/prettier");
        assert_eq!(rules[1].linter, "prettier");
    }

    #[test]
    fn test_calculate_confidence() {
        let handler = PrettierHandler::new();

        let input = "prettier/prettier: Missing semicolon";
        let confidence = handler.calculate_confidence(input);

        assert!(confidence > 0.8);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_suggest_action() {
        let handler = PrettierHandler::new();

        assert_eq!(
            handler.suggest_action("prettier/prettier"),
            ConfigAction::Ignore
        );
        assert_eq!(
            handler.suggest_action("prettier/endOfLine"),
            ConfigAction::Modify
        );
        assert_eq!(
            handler.suggest_action("prettier/printWidth"),
            ConfigAction::Modify
        );
    }

    #[test]
    fn test_validate_config() {
        let handler = PrettierHandler::new();

        let valid_config = r#"{"semi": true, "singleQuote": true}"#;
        let errors = handler.validate_config(valid_config).unwrap();
        assert!(errors.is_empty());

        let invalid_config = r#"{"semi": true, "singleQuote": true"#;
        let errors = handler.validate_config(invalid_config).unwrap();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_get_configuration_options() {
        let handler = PrettierHandler::new();
        let options = handler.get_configuration_options();

        assert!(options.contains_key("semi"));
        assert!(options.contains_key("singleQuote"));
        assert!(options.contains_key("trailingComma"));
    }
}
