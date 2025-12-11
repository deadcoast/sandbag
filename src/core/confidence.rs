//! Confidence scoring algorithms for rule extraction

use crate::core::{ExtractedContext, RuleMatch};
use std::collections::HashMap;

/// Confidence thresholds for different actions
pub const HIGH_CONFIDENCE_THRESHOLD: f64 = 0.85;
/// Medium confidence threshold; below high and above low
pub const MEDIUM_CONFIDENCE_THRESHOLD: f64 = 0.60;
/// Low confidence threshold; minimal acceptable certainty
pub const LOW_CONFIDENCE_THRESHOLD: f64 = 0.30;

/// Confidence calculator for rule matches
pub struct ConfidenceCalculator {
    rule_frequency: HashMap<String, u32>,
    context_weights: HashMap<String, f64>,
}

impl ConfidenceCalculator {
    /// Create a new confidence calculator
    pub fn new() -> Self {
        let mut context_weights = HashMap::new();
        context_weights.insert("file_path".to_string(), 0.2);
        context_weights.insert("line_number".to_string(), 0.15);
        context_weights.insert("message".to_string(), 0.25);
        context_weights.insert("severity".to_string(), 0.1);
        context_weights.insert("rule_id_format".to_string(), 0.3);

        Self {
            rule_frequency: HashMap::new(),
            context_weights,
        }
    }

    /// Calculate overall confidence for a rule match
    pub fn calculate_confidence(&self, rule_match: &RuleMatch) -> f64 {
        let exact_match_score = self.calculate_exact_match_score(rule_match);
        let context_score = self.calculate_context_score(&rule_match.context);
        let frequency_score = self.calculate_frequency_score(&rule_match.rule_id);
        let format_score = self.calculate_format_score(&rule_match.rule_id);

        // Weighted combination
        let weights = [0.4, 0.25, 0.2, 0.15]; // exact, context, frequency, format
        let scores = [
            exact_match_score,
            context_score,
            frequency_score,
            format_score,
        ];

        weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum()
    }

    /// Calculate exact match confidence
    #[allow(clippy::unused_self)]
    fn calculate_exact_match_score(&self, rule_match: &RuleMatch) -> f64 {
        if rule_match.has_exact_id_match() {
            1.0
        } else {
            // Partial match scoring
            let linter_known = matches!(
                rule_match.linter.as_str(),
                "markdownlint" | "eslint" | "pylint"
            );
            if linter_known {
                0.8
            } else {
                0.5
            }
        }
    }

    /// Calculate context-based confidence
    fn calculate_context_score(&self, context: &ExtractedContext) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // File path presence
        if context.file_path.is_some() {
            score += self.context_weights.get("file_path").unwrap_or(&0.0);
        }

        // Line number presence
        if context.line_number.is_some() {
            score += self.context_weights.get("line_number").unwrap_or(&0.0);
        }

        // Message quality
        if let Some(message) = &context.message {
            if message.len() > 10 && message.contains(':') {
                score += self.context_weights.get("message").unwrap_or(&0.0);
            }
        }

        // Severity information
        if context.severity.is_some() {
            score += self.context_weights.get("severity").unwrap_or(&0.0);
        }

        score.min(1.0)
    }

    /// Calculate frequency-based confidence
    fn calculate_frequency_score(&self, rule_id: &str) -> f64 {
        let frequency = self.rule_frequency.get(rule_id).unwrap_or(&0);

        // Normalize frequency to 0-1 range
        match frequency {
            0 => 0.3,      // Unknown rule
            1..=5 => 0.6,  // Rare rule
            6..=20 => 0.8, // Common rule
            _ => 0.9,      // Very common rule
        }
    }

    /// Calculate format-based confidence
    #[allow(clippy::unused_self)]
    fn calculate_format_score(&self, rule_id: &str) -> f64 {
        // Check if rule ID follows expected patterns
        if rule_id.starts_with("MD") && rule_id.len() == 5 {
            // Markdownlint format: MDxxx
            if rule_id[2..].chars().all(|c| c.is_ascii_digit()) {
                return 1.0;
            }
        } else if rule_id.starts_with("ESLint") {
            // ESLint format
            return 0.9;
        } else if rule_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            // Generic alphanumeric format
            return 0.7;
        }

        0.3 // Unknown format
    }

    /// Update rule frequency for learning
    pub fn update_frequency(&mut self, rule_id: &str) {
        *self.rule_frequency.entry(rule_id.to_string()).or_insert(0) += 1;
    }

    /// Get confidence level description
    pub fn get_confidence_level(&self, confidence: f64) -> &'static str {
        match confidence {
            c if c >= HIGH_CONFIDENCE_THRESHOLD => "High",
            c if c >= MEDIUM_CONFIDENCE_THRESHOLD => "Medium",
            c if c >= LOW_CONFIDENCE_THRESHOLD => "Low",
            _ => "Very Low",
        }
    }

    /// Check if confidence is sufficient for automatic application
    pub fn is_auto_apply(&self, confidence: f64) -> bool {
        confidence >= HIGH_CONFIDENCE_THRESHOLD
    }

    /// Check if confidence requires user confirmation
    pub fn requires_confirmation(&self, confidence: f64) -> bool {
        (MEDIUM_CONFIDENCE_THRESHOLD..HIGH_CONFIDENCE_THRESHOLD).contains(&confidence)
    }
}

impl Default for ConfidenceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};

    fn create_test_rule_match() -> RuleMatch {
        RuleMatch {
            rule_id: "MD033".to_string(),
            linter: "markdownlint".to_string(),
            confidence: 0.0,
            context: ExtractedContext {
                file_path: Some("test.md".to_string()),
                line_number: Some(10),
                column: Some(5),
                message: Some("MD033/no-inline-html: Inline HTML [Element: div]".to_string()),
                severity: Some(Severity::Warning),
            },
            suggested_action: ConfigAction::Disable,
        }
    }

    #[test]
    fn test_confidence_calculation() {
        let calculator = ConfidenceCalculator::new();
        let rule_match = create_test_rule_match();
        let confidence = calculator.calculate_confidence(&rule_match);

        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_confidence_levels() {
        let calculator = ConfidenceCalculator::new();

        assert_eq!(calculator.get_confidence_level(0.9), "High");
        assert_eq!(calculator.get_confidence_level(0.7), "Medium");
        assert_eq!(calculator.get_confidence_level(0.4), "Low");
        assert_eq!(calculator.get_confidence_level(0.1), "Very Low");
    }

    #[test]
    fn test_auto_apply_logic() {
        let calculator = ConfidenceCalculator::new();

        assert!(calculator.is_auto_apply(0.9));
        assert!(!calculator.is_auto_apply(0.7));
        assert!(calculator.requires_confirmation(0.7));
        assert!(!calculator.requires_confirmation(0.9));
    }
}
