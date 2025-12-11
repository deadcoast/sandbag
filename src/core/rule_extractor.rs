//! Rule extraction engine with confidence scoring

use crate::core::bayesian::NaiveBayesModel;
use crate::core::bayesian::{EvidenceVector, NaiveBayesScorer};
use crate::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Minimum confidence threshold for rule extraction
const MINIMUM_CONFIDENCE_THRESHOLD: f64 = 0.6;

/// Rule extraction patterns for different linters
#[derive(Debug, Clone)]
pub struct ExtractionPattern {
    /// Name of the linter the pattern targets
    pub linter: String,
    /// Regular expression used to extract rule identifiers
    pub pattern: Regex,
    /// Weight of this pattern in confidence calculation
    pub confidence_weight: f64,
}

/// Rule extraction engine
pub struct RuleExtractor {
    patterns: Vec<ExtractionPattern>,
    rule_database: HashMap<String, String>, // rule_id -> description
    bayes: NaiveBayesScorer,
}

impl RuleExtractor {
    /// Create a new rule extractor
    pub fn new() -> Self {
        let patterns = vec![
            // Markdownlint pattern
            ExtractionPattern {
                linter: "markdownlint".to_string(),
                pattern: Regex::new(r"(MD\d{3})(?:/[\w-]+)?:.*?markdownlint(?:MD\d{3})?").unwrap(),
                confidence_weight: 0.9,
            },
        ];

        // TODO: Add more linter patterns

        let mut rule_database = HashMap::new();
        // Populate with common markdownlint rules
        rule_database.insert("MD033".to_string(), "No inline HTML".to_string());
        rule_database.insert(
            "MD041".to_string(),
            "First line in file should be a top level heading".to_string(),
        );
        rule_database.insert("MD013".to_string(), "Line length".to_string());

        Self {
            patterns,
            rule_database,
            bayes: NaiveBayesScorer::new(),
        }
    }

    /// Load a Naive Bayes model from JSON file and set the scorer.
    pub fn with_bayes_model_from_path<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let model = NaiveBayesModel::load_from_path(path)?;
        self.bayes = model.to_scorer(1.0);
        Ok(())
    }

    /// Extract rules from input string
    pub async fn extract_rules(&self, input: &str) -> Result<Vec<RuleMatch>> {
        // Allow cooperative scheduling in async contexts
        tokio::task::yield_now().await;
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            if let Some(rule_match) = self.apply_pattern(pattern, input) {
                let confidence = self.calculate_confidence_bayesian(input, &rule_match, pattern);
                if confidence >= MINIMUM_CONFIDENCE_THRESHOLD {
                    matches.push(rule_match);
                }
            }
        }

        self.rank_by_confidence(matches)
    }

    /// Apply a specific pattern to extract rules
    fn apply_pattern(&self, pattern: &ExtractionPattern, input: &str) -> Option<RuleMatch> {
        if let Some(captures) = pattern.pattern.captures(input) {
            let rule_id = captures.get(1)?.as_str().to_string();

            Some(RuleMatch {
                rule_id: rule_id.clone(),
                linter: pattern.linter.clone(),
                confidence: 0.0, // Will be calculated later
                context: self.extract_context(input),
                suggested_action: ConfigAction::Disable,
            })
        } else {
            None
        }
    }

    /// Calculate confidence using Naive Bayes over evidence, with heuristic fallback
    fn calculate_confidence_bayesian(
        &self,
        input: &str,
        rule_match: &RuleMatch,
        pattern: &ExtractionPattern,
    ) -> f64 {
        let evidence = self.build_evidence_vector(input);
        let bayes_score = self.bayes.score_rule(&rule_match.rule_id, &evidence);
        if bayes_score.is_finite() {
            // convert unnormalized log-posterior to [0,1] via logistic squash
            let p = 1.0 / (1.0 + (-bayes_score).exp());
            if p.is_finite() {
                return p;
            }
        }
        // fallback to heuristic confidence
        self.calculate_confidence_heuristic(rule_match, pattern)
    }

    #[allow(clippy::unused_self)]
    fn build_evidence_vector(&self, input: &str) -> EvidenceVector {
        let mut ev = EvidenceVector::default();
        // tokens
        for tok in input.split(|c: char| !c.is_ascii_alphanumeric()) {
            let t = tok.to_lowercase();
            if t.is_empty() {
                continue;
            }
            *ev.token_counts.entry(t).or_insert(0) += 1;
        }
        // crude n-grams over ascii
        let s = input.as_bytes();
        for w in s.windows(3) {
            let ng = format!("{}{}{}", w[0] as char, w[1] as char, w[2] as char);
            *ev.ngram_counts.entry(ng).or_insert(0) += 1;
        }
        // line:col pattern
        ev.has_line_col = input.matches(':').count() >= 2;
        ev
    }

    /// Heuristic confidence (legacy)
    fn calculate_confidence_heuristic(
        &self,
        rule_match: &RuleMatch,
        pattern: &ExtractionPattern,
    ) -> f64 {
        let exact_weight = 0.6;
        let pattern_weight = 0.3;
        let context_weight = 0.1;

        let exact_score = if rule_match.has_exact_id_match() {
            1.0
        } else {
            0.0
        };
        let pattern_score = self.calculate_pattern_similarity(rule_match, pattern);
        let context_score = self.calculate_context_relevance(rule_match);

        exact_weight * exact_score + pattern_weight * pattern_score + context_weight * context_score
    }

    /// Calculate pattern similarity score
    #[allow(clippy::unused_self)]
    fn calculate_pattern_similarity(
        &self,
        _rule_match: &RuleMatch,
        pattern: &ExtractionPattern,
    ) -> f64 {
        // TODO: Implement more sophisticated pattern similarity
        pattern.confidence_weight
    }

    /// Calculate context relevance score
    fn calculate_context_relevance(&self, rule_match: &RuleMatch) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Boost score if we have a description in our database
        if self.rule_database.contains_key(&rule_match.rule_id) {
            score += 0.3;
        }

        // Boost score if we have context information
        if rule_match.context.message.is_some() {
            score += 0.2;
        }

        score.min(1.0)
    }

    /// Extract context from input string
    #[allow(clippy::unused_self)]
    fn extract_context(&self, input: &str) -> ExtractedContext {
        // TODO: Implement more sophisticated context extraction
        ExtractedContext {
            file_path: None,
            line_number: None,
            column: None,
            message: Some(input.to_string()),
            severity: Some(Severity::Warning),
        }
    }

    /// Rank matches by confidence
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn rank_by_confidence(&self, mut matches: Vec<RuleMatch>) -> Result<Vec<RuleMatch>> {
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(matches)
    }

    /// Get patterns for input (future: intelligent pattern selection)
    #[allow(dead_code)]
    fn get_patterns_for_input(&self, _input: &str) -> &[ExtractionPattern] {
        &self.patterns
    }

    /// Determine suggested action for a rule
    #[allow(clippy::unused_self)]
    #[allow(dead_code)]
    fn determine_action(&self, _rule_match: &RuleMatch) -> ConfigAction {
        ConfigAction::Disable
    }
}

impl Default for RuleExtractor {
    fn default() -> Self {
        Self::new()
    }
}
