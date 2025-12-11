//! Core functionality for rule extraction, similarity analysis, and confidence scoring

pub mod advanced_similarity;
pub mod bayesian;
pub mod confidence;
pub mod information;
pub mod rule_extractor;
pub mod semantic_analysis;
pub mod similarity;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a rule with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique rule identifier (e.g., "MD033")
    pub id: String,
    /// Name of the linter that defines this rule
    pub linter: String,
    /// Severity associated with the rule
    pub severity: Severity,
    /// Optional human-readable description
    pub description: Option<String>,
    /// Category of the rule for organization
    pub category: RuleCategory,
    /// Confidence score for associated detections
    pub confidence: f64,
}

/// Rule severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Informational severity
    Info,
    /// Hint level
    Hint,
    /// Rule disabled
    Off,
}

/// Rule categories for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCategory {
    /// Code style rules
    Style,
    /// Best-practice recommendations
    BestPractice,
    /// Error-prone code patterns
    ErrorProne,
    /// Complexity-related rules
    Complexity,
    /// Performance-related rules
    Performance,
    /// Security-related rules
    Security,
    /// Documentation-related rules
    Documentation,
    /// Other uncategorized rules
    Other,
}

/// Configuration entry for a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// Target rule identifier
    pub rule_id: String,
    /// Action to take for this rule
    pub action: ConfigAction,
    /// Scope where the configuration applies
    pub scope: ConfigScope,
    /// Arbitrary metadata for tool-specific options
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Actions that can be taken on a rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigAction {
    /// Disable the rule
    Disable,
    /// Ignore rule violations
    Ignore,
    /// Modify rule behavior
    Modify,
    /// Enable rule enforcement
    Enable,
}

/// Scope of configuration application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// Project-wide scope
    Global,
    /// Specific file scope
    FileSpecific,
    /// Inline (line-level) scope
    Inline,
    /// Directory-level scope
    Directory,
}

/// Extracted context from rule input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContext {
    /// Source file path if known
    pub file_path: Option<String>,
    /// Line number where the issue occurred
    pub line_number: Option<u32>,
    /// Column number where the issue occurred
    pub column: Option<u32>,
    /// Original linter message
    pub message: Option<String>,
    /// Parsed severity from the linter output
    pub severity: Option<Severity>,
}

/// Result of rule extraction with confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMatch {
    /// Matched rule identifier
    pub rule_id: String,
    /// Linter that produced this match
    pub linter: String,
    /// Confidence score of the match [0.0, 1.0]
    pub confidence: f64,
    /// Extracted context from the input
    pub context: ExtractedContext,
    /// Suggested action based on heuristics
    pub suggested_action: ConfigAction,
}

impl RuleMatch {
    /// Check if this match has an exact ID match
    pub fn has_exact_id_match(&self) -> bool {
        // TODO: Implement exact match logic
        self.confidence > 0.95
    }
}
