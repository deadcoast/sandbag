//! Bayesian evidence aggregation for rule identification

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Evidence vector for a potential rule match
#[derive(Debug, Clone, Default)]
pub struct EvidenceVector {
    /// character n-gram features (e.g., tri-grams)
    pub ngram_counts: HashMap<String, u32>,
    /// token features (lowercased words from message)
    pub token_counts: HashMap<String, u32>,
    /// optional file extension context ("rs", "js", ...)
    pub file_extension: Option<String>,
    /// position features (e.g., presence of line:col)
    pub has_line_col: bool,
    /// historical frequency prior (if known)
    pub historical_prior: Option<f64>,
}

/// Build an evidence vector from a raw input string (linter output or text).
pub fn evidence_from_input(input: &str) -> EvidenceVector {
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
    // line:col pattern heuristic
    ev.has_line_col = input.matches(':').count() >= 2;
    ev
}

/// Labeled observation for training
#[derive(Debug, Clone)]
pub struct LabeledObservation {
    /// Ground-truth rule identifier
    pub rule_id: String,
    /// Extracted evidence vector
    pub evidence: EvidenceVector,
}

/// Trainable Naive Bayes model storing counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NaiveBayesModel {
    /// Count of each rule
    pub rule_counts: HashMap<String, u64>,
    /// For each rule, counts of features
    pub feature_counts: HashMap<String, HashMap<String, u64>>, // rule -> feature -> count
    /// Total count of all rules (for priors)
    pub total_rules: u64,
}

impl NaiveBayesModel {
    /// Add one labeled observation (updates counts)
    pub fn add_observation(&mut self, obs: &LabeledObservation) {
        *self.rule_counts.entry(obs.rule_id.clone()).or_insert(0) += 1;
        self.total_rules += 1;

        let feats = self.feature_counts.entry(obs.rule_id.clone()).or_default();

        if let Some(ext) = obs.evidence.file_extension.as_ref() {
            *feats.entry(format!("ext:{ext}")).or_insert(0) += 1;
        }
        if obs.evidence.has_line_col {
            *feats.entry("has_line_col".to_string()).or_insert(0) += 1;
        }
        for (tok, count) in &obs.evidence.token_counts {
            *feats.entry(format!("tok:{tok}")).or_insert(0) += u64::from(*count);
        }
        for (ng, count) in &obs.evidence.ngram_counts {
            *feats.entry(format!("ng:{ng}")).or_insert(0) += u64::from(*count);
        }
    }

    /// Build a scorer with Laplace-smoothed log-likelihoods and log-priors
    pub fn to_scorer(&self, alpha: f64) -> NaiveBayesScorer {
        let mut scorer = NaiveBayesScorer::new();

        // Priors
        for (rule, &cnt) in &self.rule_counts {
            #[allow(clippy::cast_precision_loss)]
            let p = if self.total_rules > 0 {
                (cnt as f64) / (self.total_rules as f64)
            } else {
                0.0
            };
            scorer.rule_priors.insert(rule.clone(), p);
        }

        // Likelihoods per rule
        for (rule, feats) in &self.feature_counts {
            let mut ll_map: HashMap<String, f64> = HashMap::new();
            let vocab_size = feats.len().max(1);
            let total: u64 = feats.values().copied().sum();
            #[allow(clippy::cast_precision_loss)]
            let denom = (total as f64) + alpha * (vocab_size as f64);
            for (f, &c) in feats {
                #[allow(clippy::cast_precision_loss)]
                let num = (c as f64) + alpha;
                let prob = if denom > 0.0 { num / denom } else { 0.0 };
                ll_map.insert(
                    f.clone(),
                    if prob > 0.0 {
                        prob.ln()
                    } else {
                        f64::NEG_INFINITY
                    },
                );
            }
            scorer.feature_likelihoods.insert(rule.clone(), ll_map);
        }

        scorer
    }

    /// Save model as JSON
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }

    /// Load model from JSON
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = fs::read_to_string(path)?;
        let model: Self = serde_json::from_str(&data)?;
        Ok(model)
    }
}

/// Simple Naive Bayes scorer over discrete features
#[derive(Debug, Clone, Default)]
pub struct NaiveBayesScorer {
    /// P(rule)
    pub rule_priors: HashMap<String, f64>,
    /// P(feature|rule) as log probabilities per rule
    pub feature_likelihoods: HashMap<String, HashMap<String, f64>>, // rule -> feature -> log P(f|rule)
    /// default smoothing log-likelihood for unseen features
    pub default_log_likelihood: f64,
}

impl NaiveBayesScorer {
    /// Create a new naive Bayes scorer with default smoothing.
    pub fn new() -> Self {
        Self {
            rule_priors: HashMap::new(),
            feature_likelihoods: HashMap::new(),
            default_log_likelihood: (-10.0),
        }
    }

    /// Compute unnormalized log posterior score: log P(rule) + sum log P(feature|rule)
    pub fn score_rule(&self, rule_id: &str, evidence: &EvidenceVector) -> f64 {
        let log_prior = self
            .rule_priors
            .get(rule_id)
            .map_or(f64::NEG_INFINITY, |p| {
                if *p > 0.0 {
                    p.ln()
                } else {
                    f64::NEG_INFINITY
                }
            });

        let feats = self.feature_likelihoods.get(rule_id);

        let mut log_sum = log_prior;

        if let Some(ext) = evidence.file_extension.as_ref() {
            log_sum += Self::feat_log(feats, &format!("ext:{ext}"), self.default_log_likelihood);
        }
        if evidence.has_line_col {
            log_sum += Self::feat_log(feats, "has_line_col", self.default_log_likelihood);
        }

        for (tok, count) in &evidence.token_counts {
            let ll = Self::feat_log(feats, &format!("tok:{tok}"), self.default_log_likelihood);
            log_sum += ll * f64::from(*count);
        }
        for (ng, count) in &evidence.ngram_counts {
            let ll = Self::feat_log(feats, &format!("ng:{ng}"), self.default_log_likelihood);
            log_sum += ll * f64::from(*count);
        }

        if let Some(pr) = evidence.historical_prior {
            if pr > 0.0 {
                log_sum += pr.ln();
            }
        }

        log_sum
    }

    fn feat_log(feats: Option<&HashMap<String, f64>>, key: &str, default_ll: f64) -> f64 {
        feats
            .and_then(|m| m.get(key).copied())
            .unwrap_or(default_ll)
    }
}
