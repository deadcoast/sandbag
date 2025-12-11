//! ESLint linter support

use crate::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};
use crate::linters::LinterHandler;
use regex::Regex;
use std::collections::HashMap;

/// ESLint linter handler
#[derive(Debug, Clone)]
pub struct ESLintHandler {
    rule_patterns: Vec<Regex>,
    severity_mapping: HashMap<String, Severity>,
    rule_descriptions: HashMap<String, String>,
}

impl ESLintHandler {
    /// Create a new ESLint handler
    pub fn new() -> Self {
        let mut handler = Self {
            rule_patterns: Vec::new(),
            severity_mapping: HashMap::new(),
            rule_descriptions: HashMap::new(),
        };

        handler.initialize_patterns();
        handler.initialize_severity_mapping();
        handler.initialize_rule_descriptions();

        handler
    }

    /// Initialize regex patterns for ESLint rules
    fn initialize_patterns(&mut self) {
        // ESLint rule patterns - simplified and more robust
        let patterns = vec![
            // ESLint error format: line:col error rule-name message
            r"(\d+):(\d+)\s+error\s+(\w+)\s+(.+)",
            // ESLint warning format: line:col warn rule-name message
            r"(\d+):(\d+)\s+warn\s+(\w+)\s+(.+)",
            // Standard ESLint output format: line:col severity rule-name message
            r"(\d+):(\d+)\s+(\w+)\s+(\w+)\s+(.+)",
            // Alternative format with rule name: line:col severity category/rule-name message
            r"(\d+):(\d+)\s+(\w+)\s+(\w+)/(\w+)\s+(.+)",
            // Simple rule format
            r"(\w+)/(\w+):\s*(.+)",
            // Rule with severity
            r"(\w+)/(\w+)\s+\((\w+)\)\s+(.+)",
            // Simple eslint format
            r"eslint:\s*(\w+)",
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.rule_patterns.push(regex);
            }
        }
    }

    /// Initialize severity mapping for ESLint rules
    fn initialize_severity_mapping(&mut self) {
        self.severity_mapping
            .insert("error".to_string(), Severity::Error);
        self.severity_mapping
            .insert("warn".to_string(), Severity::Warning);
        self.severity_mapping
            .insert("info".to_string(), Severity::Info);
        self.severity_mapping
            .insert("off".to_string(), Severity::Off);
    }

    /// Initialize common ESLint rule descriptions
    fn initialize_rule_descriptions(&mut self) {
        let m = &mut self.rule_descriptions;
        m.insert(
            "no-unused-vars".to_string(),
            "Disallow unused variables".to_string(),
        );
        m.insert(
            "no-console".to_string(),
            "Disallow the use of console".to_string(),
        );
        m.insert("eqeqeq".to_string(), "Require === and !==".to_string());
        m.insert(
            "quotes".to_string(),
            "Enforce the consistent use of quotes".to_string(),
        );
        m.insert(
            "semi".to_string(),
            "Require or disallow semicolons".to_string(),
        );
        m.insert(
            "prefer-const".to_string(),
            "Suggest using const".to_string(),
        );
        m.insert(
            "no-undef".to_string(),
            "Disallow the use of undeclared variables".to_string(),
        );
        m.insert(
            "no-extra-semi".to_string(),
            "Disallow unnecessary semicolons".to_string(),
        );
        m.insert(
            "no-redeclare".to_string(),
            "Disallow variable redeclaration".to_string(),
        );
        m.insert(
            "no-eval".to_string(),
            "Disallow the use of eval()".to_string(),
        );
        m.insert(
            "no-debugger".to_string(),
            "Disallow the use of debugger".to_string(),
        );
        m.insert(
            "curly".to_string(),
            "Require consistent brace style for all control statements".to_string(),
        );
        m.insert(
            "no-trailing-spaces".to_string(),
            "Disallow trailing whitespace at the end of lines".to_string(),
        );
        m.insert(
            "comma-dangle".to_string(),
            "Require or disallow trailing commas".to_string(),
        );
        m.insert(
            "no-mixed-spaces-and-tabs".to_string(),
            "Disallow mixed spaces and tabs for indentation".to_string(),
        );
        m.insert(
            "no-empty".to_string(),
            "Disallow empty block statements".to_string(),
        );
        m.insert(
            "no-use-before-define".to_string(),
            "Disallow the use of variables before they are defined".to_string(),
        );
        m.insert(
            "no-shadow".to_string(),
            "Disallow variable declarations from shadowing variables declared in the outer scope"
                .to_string(),
        );
        m.insert(
            "consistent-return".to_string(),
            "Require return statements to either always or never specify values".to_string(),
        );
    }

    /// Get rule description if known
    pub fn get_rule_description(&self, rule_id: &str) -> Option<String> {
        // Accept category/rule or rule format
        let rule = if let Some((_, r)) = rule_id.split_once('/') {
            r
        } else {
            rule_id
        };
        self.rule_descriptions.get(rule).cloned()
    }

    /// Extract rule ID from ESLint output
    fn extract_rule_id(&self, input: &str) -> Option<String> {
        // Simple string parsing approach
        let input = input.trim();

        // Handle simple eslint format
        if input.starts_with("eslint:") {
            if let Some(rule) = input.strip_prefix("eslint:") {
                return Some(rule.trim().to_string());
            }
        }

        // Handle line:col error/warn rule-name format
        if input.contains("error") || input.contains("warn") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() >= 4 {
                // Format: line:col error rule-name message
                if parts[1] == "error" || parts[1] == "warn" {
                    return Some(parts[2].to_string());
                }
            }
        }

        // Do not attempt to parse generic "path:..." lines as rules; rely on explicit patterns

        // Try regex patterns as fallback
        for pattern in &self.rule_patterns {
            if let Some(captures) = pattern.captures(input) {
                // Handle simple eslint format first
                if captures.len() == 2 && input.starts_with("eslint:") {
                    if let Some(rule) = captures.get(1) {
                        return Some(rule.as_str().to_string());
                    }
                }

                // Handle line:col error/warn rule-name format
                if captures.len() >= 5 {
                    if let Some(rule) = captures.get(3) {
                        return Some(rule.as_str().to_string());
                    }
                }

                // Handle line:col severity rule-name format
                if captures.len() >= 5 {
                    if let (Some(_severity), Some(rule)) = (captures.get(3), captures.get(4)) {
                        return Some(rule.as_str().to_string());
                    }
                }

                // Handle line:col severity category/rule-name format
                if captures.len() >= 6 {
                    if let (Some(category), Some(rule)) = (captures.get(4), captures.get(5)) {
                        return Some(format!("{}/{}", category.as_str(), rule.as_str()));
                    }
                }

                // Handle simple category/rule format
                if captures.len() >= 3 {
                    if let (Some(category), Some(rule)) = (captures.get(1), captures.get(2)) {
                        return Some(format!("{}/{}", category.as_str(), rule.as_str()));
                    }
                }
            }
        }

        None
    }

    /// Extract severity from ESLint output
    fn extract_severity(&self, input: &str) -> Option<Severity> {
        let input = input.trim();

        // Simple string parsing approach
        if input.contains("error") {
            return Some(Severity::Error);
        } else if input.contains("warn") {
            return Some(Severity::Warning);
        } else if input.contains("info") {
            return Some(Severity::Info);
        } else if input.contains("off") {
            return Some(Severity::Off);
        }

        // Try regex patterns as fallback
        for pattern in &self.rule_patterns {
            if let Some(captures) = pattern.captures(input) {
                // Handle line:col error/warn format
                if captures.len() >= 4 {
                    if let Some(severity_str) = captures.get(3) {
                        return self.severity_mapping.get(severity_str.as_str()).cloned();
                    }
                }

                // Handle line:col severity rule-name format
                if captures.len() >= 4 {
                    if let Some(severity_str) = captures.get(3) {
                        return self.severity_mapping.get(severity_str.as_str()).cloned();
                    }
                }

                // Handle rule with severity in parentheses
                if captures.len() >= 3 {
                    if let Some(severity_str) = captures.get(2) {
                        return self.severity_mapping.get(severity_str.as_str()).cloned();
                    }
                }
            }
        }

        None
    }

    /// Extract context information from ESLint output
    fn extract_context(&self, input: &str) -> ExtractedContext {
        for pattern in &self.rule_patterns {
            if let Some(captures) = pattern.captures(input) {
                if captures.len() >= 6 {
                    // Extract line and column numbers
                    let line_number = captures.get(1).and_then(|m| m.as_str().parse::<u32>().ok());
                    let column = captures.get(2).and_then(|m| m.as_str().parse::<u32>().ok());

                    // Extract message
                    let message = captures
                        .get(captures.len() - 1)
                        .map(|m| m.as_str().trim().to_string());

                    return ExtractedContext {
                        file_path: None, // Will be set by caller
                        line_number,
                        column,
                        message,
                        severity: self.extract_severity(input),
                    };
                }
            }
        }

        ExtractedContext {
            file_path: None,
            line_number: None,
            column: None,
            message: Some(input.trim().to_string()),
            severity: self.extract_severity(input),
        }
    }

    /// Calculate confidence score for ESLint rule
    fn calculate_confidence(&self, input: &str) -> f64 {
        let mut confidence: f64 = 0.6; // Higher base confidence

        // Increase confidence for well-formed ESLint output
        if input.contains(':') && input.contains(' ') {
            confidence += 0.2;
        }

        // Increase confidence if we can extract a rule ID
        if self.extract_rule_id(input).is_some() {
            confidence += 0.2;
        }

        // Increase confidence if we can extract severity
        if self.extract_severity(input).is_some() {
            confidence += 0.1;
        }

        // Increase confidence for common ESLint rule patterns
        if input.contains('/') && input.contains(':') {
            confidence += 0.1;
        }

        // Increase confidence for line:col format
        if input.matches(':').count() >= 2 {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Suggest action based on ESLint rule and severity
    #[allow(clippy::unused_self)]
    fn suggest_action(&self, rule_id: &str, severity: Option<&Severity>) -> ConfigAction {
        match severity {
            Some(Severity::Error) => ConfigAction::Disable,
            Some(Severity::Warning) => ConfigAction::Ignore,
            Some(Severity::Info | Severity::Hint) => ConfigAction::Modify,
            Some(Severity::Off) => ConfigAction::Enable,
            None => {
                // Default action based on rule type
                if rule_id.contains("no-") {
                    ConfigAction::Disable
                } else {
                    ConfigAction::Ignore
                }
            }
        }
    }
}

impl Default for ESLintHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LinterHandler for ESLintHandler {
    fn can_handle(&self, input: &str) -> bool {
        // Check if input looks like ESLint output
        input.contains("eslint")
            || input.matches(':').count() >= 2  // line:col format
            || input.contains("error") || input.contains("warn")
            || (input.contains('/') && input.contains(':') && !input.contains("prettier"))
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
                let suggested_action = self.suggest_action(&rule_id, context.severity.as_ref());

                rules.push(RuleMatch {
                    rule_id,
                    linter: "eslint".to_string(),
                    confidence,
                    context,
                    suggested_action,
                });
            }
        }

        rules
    }

    fn get_linter_name(&self) -> &'static str {
        "eslint"
    }

    fn get_config_file_patterns(&self) -> Vec<&str> {
        vec![
            ".eslintrc.js",
            ".eslintrc.json",
            ".eslintrc.yaml",
            ".eslintrc.yml",
            "eslint.config.js",
        ]
    }

    fn get_rule_format(&self) -> &'static str {
        "category/rule-name"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eslint_handler_creation() {
        let handler = ESLintHandler::new();
        assert!(!handler.rule_patterns.is_empty());
        assert!(!handler.severity_mapping.is_empty());
    }

    #[test]
    fn test_can_handle_eslint_output() {
        let handler = ESLintHandler::new();

        assert!(
            handler.can_handle("1:10 error no-unused-vars 'x' is assigned a value but never used")
        );
        assert!(handler.can_handle("eslint: no-console"));
        assert!(handler.can_handle("prefer-const/const: Use const instead of let"));
        assert!(!handler.can_handle("MD033: No inline HTML"));
    }

    #[test]
    fn test_extract_rule_id() {
        let handler = ESLintHandler::new();

        let input = "1:10 error no-unused-vars 'x' is assigned a value but never used";
        assert_eq!(
            handler.extract_rule_id(input),
            Some("no-unused-vars".to_string())
        );

        let input = "2:5 warn prefer-const/const Use const instead of let";
        assert_eq!(
            handler.extract_rule_id(input),
            Some("prefer-const/const".to_string())
        );
    }

    #[test]
    fn test_extract_severity() {
        let handler = ESLintHandler::new();

        let input = "1:10 error no-unused-vars 'x' is assigned a value but never used";
        assert_eq!(handler.extract_severity(input), Some(Severity::Error));

        let input = "2:5 warn prefer-const/const Use const instead of let";
        assert_eq!(handler.extract_severity(input), Some(Severity::Warning));
    }

    #[test]
    fn test_extract_rules() {
        let handler = ESLintHandler::new();

        let input = "1:10 error no-unused-vars 'x' is assigned a value but never used\n2:5 warn prefer-const/const Use const instead of let";
        let rules = handler.extract_rules(input);

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].rule_id, "no-unused-vars");
        assert_eq!(rules[0].linter, "eslint");
        assert_eq!(rules[1].rule_id, "prefer-const/const");
        assert_eq!(rules[1].linter, "eslint");
    }

    #[test]
    fn test_calculate_confidence() {
        let handler = ESLintHandler::new();

        let input = "1:10 error no-unused-vars 'x' is assigned a value but never used";
        let confidence = handler.calculate_confidence(input);

        assert!(confidence > 0.8);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_suggest_action() {
        let handler = ESLintHandler::new();

        assert_eq!(
            handler.suggest_action("no-unused-vars", Some(&Severity::Error)),
            ConfigAction::Disable
        );
        assert_eq!(
            handler.suggest_action("prefer-const", Some(&Severity::Warning)),
            ConfigAction::Ignore
        );
        assert_eq!(
            handler.suggest_action("no-console", None),
            ConfigAction::Disable
        );
    }
}
