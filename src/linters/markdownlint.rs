//! Markdownlint specific implementation

use crate::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};
use crate::linters::LinterHandler;
use regex::Regex;
use std::collections::HashMap;

/// Markdownlint handler implementation
#[derive(Debug, Clone)]
pub struct MarkdownlintHandler {
    config_patterns: Vec<Regex>,
    rule_database: HashMap<String, String>,
}

impl MarkdownlintHandler {
    /// Create a new Markdownlint handler
    pub fn new() -> Self {
        let config_patterns = vec![
            Regex::new(r"(MD\d{3})(?:/[\w-]+)?:.*?markdownlint(?:MD\d{3})?").unwrap(),
            Regex::new(r"markdownlint.*?(MD\d{3})").unwrap(),
            Regex::new(r"(MD\d{3}):.*").unwrap(),
        ];

        let mut rule_database = HashMap::new();

        // Populate with common markdownlint rules
        rule_database.insert(
            "MD001".to_string(),
            "Heading levels should only increment by one level at a time".to_string(),
        );
        rule_database.insert(
            "MD002".to_string(),
            "First heading should be a top level heading".to_string(),
        );
        rule_database.insert("MD003".to_string(), "Heading style".to_string());
        rule_database.insert("MD004".to_string(), "Unordered list style".to_string());
        rule_database.insert(
            "MD005".to_string(),
            "Inconsistent indentation for list items at the same level".to_string(),
        );
        rule_database.insert(
            "MD006".to_string(),
            "Consider starting bulleted lists at the beginning of the line".to_string(),
        );
        rule_database.insert(
            "MD007".to_string(),
            "Unordered list indentation".to_string(),
        );
        rule_database.insert("MD009".to_string(), "No trailing spaces".to_string());
        rule_database.insert("MD010".to_string(), "No hard tabs".to_string());
        rule_database.insert("MD011".to_string(), "No reversed links".to_string());
        rule_database.insert(
            "MD012".to_string(),
            "No multiple consecutive blank lines".to_string(),
        );
        rule_database.insert("MD013".to_string(), "Line length".to_string());
        rule_database.insert(
            "MD014".to_string(),
            "Dollar signs used before commands without showing output".to_string(),
        );
        rule_database.insert(
            "MD018".to_string(),
            "No space after hash on atx style heading".to_string(),
        );
        rule_database.insert(
            "MD019".to_string(),
            "Multiple spaces after hash on atx style heading".to_string(),
        );
        rule_database.insert(
            "MD020".to_string(),
            "No space inside hashes on closed atx style heading".to_string(),
        );
        rule_database.insert(
            "MD021".to_string(),
            "Multiple spaces inside hashes on closed atx style heading".to_string(),
        );
        rule_database.insert(
            "MD022".to_string(),
            "Headings should be surrounded by blank lines".to_string(),
        );
        rule_database.insert(
            "MD023".to_string(),
            "Headings must start at the beginning of the line".to_string(),
        );
        rule_database.insert(
            "MD024".to_string(),
            "Multiple headings with the same content".to_string(),
        );
        rule_database.insert(
            "MD025".to_string(),
            "Multiple top level headings in the same document".to_string(),
        );
        rule_database.insert(
            "MD026".to_string(),
            "Trailing punctuation in heading".to_string(),
        );
        rule_database.insert(
            "MD027".to_string(),
            "Multiple spaces after blockquote symbol".to_string(),
        );
        rule_database.insert(
            "MD028".to_string(),
            "Blank line inside blockquote".to_string(),
        );
        rule_database.insert("MD029".to_string(), "Ordered list item prefix".to_string());
        rule_database.insert("MD030".to_string(), "List item marker".to_string());
        rule_database.insert(
            "MD031".to_string(),
            "Blank lines around fenced code blocks".to_string(),
        );
        rule_database.insert(
            "MD032".to_string(),
            "Lists should be surrounded by blank lines".to_string(),
        );
        rule_database.insert("MD033".to_string(), "No inline HTML".to_string());
        rule_database.insert("MD034".to_string(), "Bare URL used".to_string());
        rule_database.insert("MD035".to_string(), "Horizontal rule style".to_string());
        rule_database.insert(
            "MD036".to_string(),
            "No emphasis used for headings".to_string(),
        );
        rule_database.insert(
            "MD037".to_string(),
            "No space inside emphasis markers".to_string(),
        );
        rule_database.insert(
            "MD038".to_string(),
            "No space inside code span elements".to_string(),
        );
        rule_database.insert("MD039".to_string(), "No space inside link text".to_string());
        rule_database.insert(
            "MD040".to_string(),
            "Fenced code blocks should have a language specified".to_string(),
        );
        rule_database.insert(
            "MD041".to_string(),
            "First line in file should be a top level heading".to_string(),
        );
        rule_database.insert("MD042".to_string(), "No empty links".to_string());
        rule_database.insert(
            "MD043".to_string(),
            "Required heading structure".to_string(),
        );
        rule_database.insert(
            "MD044".to_string(),
            "Proper names should have the correct capitalization".to_string(),
        );
        rule_database.insert(
            "MD045".to_string(),
            "Images should have alternate text (alt text)".to_string(),
        );
        rule_database.insert("MD046".to_string(), "Code block style".to_string());
        rule_database.insert(
            "MD047".to_string(),
            "Files should end with a single newline character".to_string(),
        );

        Self {
            config_patterns,
            rule_database,
        }
    }

    /// Calculate extraction confidence for a rule
    fn calculate_extraction_confidence(&self, input: &str, rule_id: &str) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence

        // Boost confidence if rule exists in database
        if self.rule_database.contains_key(rule_id) {
            confidence += 0.3;
        }

        // Boost confidence if input contains markdownlint identifier
        if input.contains("markdownlint") {
            confidence += 0.2;
        }

        // Boost confidence for exact pattern match
        if input.contains(rule_id) {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Extract context from markdownlint output
    fn extract_context(&self, input: &str) -> ExtractedContext {
        // Try to extract file path and line number from input
        let file_path = self.extract_file_path(input);
        let line_number = self.extract_line_number(input);
        let column = self.extract_column(input);

        ExtractedContext {
            file_path,
            line_number,
            column,
            message: Some(input.to_string()),
            severity: Some(Severity::Warning), // Markdownlint typically shows warnings
        }
    }

    /// Extract file path from input
    #[allow(clippy::unused_self)]
    fn extract_file_path(&self, input: &str) -> Option<String> {
        // Heuristics: match `<path>.md:line:col` prefix or embedded
        let re = Regex::new(r"(?P<path>[^\s:]+\.(md|markdown)):(?P<line>\d+)(?::\d+)?").ok()?;
        if let Some(c) = re.captures(input) {
            return c.name("path").map(|m| m.as_str().to_string());
        }
        // Alternative: windows path with backslashes, capture until ':' before digits
        let re2 = Regex::new(r"(?P<path>\S+\.(md|markdown)):(?P<line>\d+)").ok()?;
        re2.captures(input)
            .and_then(|c| c.name("path").map(|m| m.as_str().to_string()))
    }

    /// Extract line number from input
    #[allow(clippy::unused_self)]
    fn extract_line_number(&self, input: &str) -> Option<u32> {
        let re = Regex::new(r"[^\s:]+\.(md|markdown):(?P<line>\d+)(?::\d+)?").ok()?;
        re.captures(input)
            .and_then(|c| c.name("line").and_then(|m| m.as_str().parse::<u32>().ok()))
    }

    /// Extract column from input
    #[allow(clippy::unused_self)]
    fn extract_column(&self, input: &str) -> Option<u32> {
        let re = Regex::new(r"[^\s:]+\.(md|markdown):\d+:(?P<col>\d+)").ok()?;
        re.captures(input)
            .and_then(|c| c.name("col").and_then(|m| m.as_str().parse::<u32>().ok()))
    }

    /// Extract rule ID from input
    fn extract_rule_id(&self, input: &str) -> Option<RuleMatch> {
        for pattern in &self.config_patterns {
            if let Some(captures) = pattern.captures(input) {
                let rule_id = captures.get(1)?.as_str().to_string();

                return Some(RuleMatch {
                    rule_id: rule_id.clone(),
                    linter: "markdownlint".to_string(),
                    confidence: self.calculate_extraction_confidence(input, &rule_id),
                    context: self.extract_context(input),
                    suggested_action: ConfigAction::Disable,
                });
            }
        }

        None
    }
}

impl Default for MarkdownlintHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LinterHandler for MarkdownlintHandler {
    fn can_handle(&self, input: &str) -> bool {
        // Check if input looks like markdownlint output
        input.contains("MD") && input.contains(':') || self.extract_rule_id(input).is_some()
    }

    fn extract_rules(&self, input: &str) -> Vec<RuleMatch> {
        let mut rules = Vec::new();

        // Split input into lines and process each line
        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(rule_match) = self.extract_rule_id(trimmed) {
                rules.push(rule_match);
            }
        }

        rules
    }

    fn get_linter_name(&self) -> &'static str {
        "markdownlint"
    }

    fn get_config_file_patterns(&self) -> Vec<&str> {
        vec![
            ".markdownlint.json",
            ".markdownlint.jsonc",
            ".markdownlint.yaml",
            ".markdownlint.yml",
            "markdownlint.json",
            "markdownlint.yaml",
            "markdownlint.yml",
        ]
    }

    fn get_rule_format(&self) -> &'static str {
        "MDxxx"
    }
}

impl MarkdownlintHandler {
    /// Validate if a rule ID follows markdownlint format
    pub fn validate_rule_id(&self, rule_id: &str) -> bool {
        // Check if rule follows markdownlint format (MDxxx)
        if rule_id.starts_with("MD") && rule_id.len() == 5 {
            if let Some(number_part) = rule_id.get(2..) {
                return number_part.chars().all(|c| c.is_ascii_digit());
            }
        }
        false
    }

    /// Get rule description from database
    pub fn get_rule_description(&self, rule_id: &str) -> Option<String> {
        self.rule_database.get(rule_id).cloned()
    }

    /// Get configuration files (legacy method for tests)
    pub fn get_config_files(&self) -> Vec<&str> {
        self.get_config_file_patterns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdownlint_handler_creation() {
        let handler = MarkdownlintHandler::new();
        assert_eq!(handler.get_linter_name(), "markdownlint");
        assert!(!handler.config_patterns.is_empty());
        assert!(!handler.rule_database.is_empty());
    }

    #[test]
    fn test_rule_extraction() {
        let handler = MarkdownlintHandler::new();
        let input = "MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033";

        let result = handler.extract_rule_id(input).unwrap();
        assert_eq!(result.rule_id, "MD033");
        assert_eq!(result.linter, "markdownlint");
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn test_rule_validation() {
        let handler = MarkdownlintHandler::new();

        assert!(handler.validate_rule_id("MD033"));
        assert!(handler.validate_rule_id("MD001"));
        assert!(!handler.validate_rule_id("MD1"));
        assert!(!handler.validate_rule_id("ESLint"));
    }

    #[test]
    fn test_config_files() {
        let handler = MarkdownlintHandler::new();
        let config_files = handler.get_config_files();

        assert!(config_files.contains(&".markdownlint.json"));
        assert!(config_files.contains(&".markdownlint.yaml"));
        assert!(config_files.contains(&".markdownlint.yml"));
    }

    #[test]
    fn test_rule_description() {
        let handler = MarkdownlintHandler::new();

        let description = handler.get_rule_description("MD033").unwrap();
        assert_eq!(description, "No inline HTML");

        assert!(handler.get_rule_description("INVALID").is_none());
    }
}
