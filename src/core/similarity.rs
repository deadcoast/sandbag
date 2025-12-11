//! Mathematical similarity analysis with multiple algorithms

use std::collections::HashSet;

/// Trait for similarity algorithms
pub trait SimilarityAlgorithm {
    /// Calculate similarity between two inputs, returning a score in [0.0, 1.0]
    fn calculate(&self, a: &str, b: &str) -> f64;
    /// Relative weight for combining this algorithm with others
    fn weight(&self) -> f64;
}

/// Levenshtein distance similarity
pub struct LevenshteinSimilarity;

impl SimilarityAlgorithm for LevenshteinSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        let distance = levenshtein_distance(a, b);
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            1.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                1.0 - (distance as f64 / max_len as f64)
            }
        }
    }

    fn weight(&self) -> f64 {
        0.4
    }
}

/// Jaccard similarity for character sets
pub struct JaccardSimilarity;

impl SimilarityAlgorithm for JaccardSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        let set_a: HashSet<char> = a.chars().collect();
        let set_b: HashSet<char> = b.chars().collect();

        #[allow(clippy::cast_precision_loss)]
        let intersection = set_a.intersection(&set_b).count() as f64;
        #[allow(clippy::cast_precision_loss)]
        let union = set_a.union(&set_b).count() as f64;

        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }

    fn weight(&self) -> f64 {
        0.3
    }
}

/// Cosine similarity for word-based comparison
pub struct CosineSimilarity;

impl SimilarityAlgorithm for CosineSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        let words_a: Vec<&str> = a.split_whitespace().collect();
        let words_b: Vec<&str> = b.split_whitespace().collect();

        let set_a: HashSet<&str> = words_a.iter().copied().collect();
        let set_b: HashSet<&str> = words_b.iter().copied().collect();

        #[allow(clippy::cast_precision_loss)]
        let intersection = set_a.intersection(&set_b).count() as f64;
        #[allow(clippy::cast_precision_loss)]
        let magnitude_a = (set_a.len() as f64).sqrt();
        #[allow(clippy::cast_precision_loss)]
        let magnitude_b = (set_b.len() as f64).sqrt();

        let product = magnitude_a * magnitude_b;
        if product == 0.0 {
            0.0
        } else {
            intersection / product
        }
    }

    fn weight(&self) -> f64 {
        0.2
    }
}

/// Exact match similarity
pub struct ExactMatchSimilarity;

impl SimilarityAlgorithm for ExactMatchSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        if a == b {
            1.0
        } else {
            0.0
        }
    }

    fn weight(&self) -> f64 {
        0.1
    }
}

/// Multi-dimensional similarity analyzer
pub struct SimilarityAnalyzer {
    algorithms: Vec<Box<dyn SimilarityAlgorithm>>,
}

impl SimilarityAnalyzer {
    /// Create a new similarity analyzer with default algorithms
    pub fn new() -> Self {
        let algorithms: Vec<Box<dyn SimilarityAlgorithm>> = vec![
            Box::new(LevenshteinSimilarity),
            Box::new(JaccardSimilarity),
            Box::new(CosineSimilarity),
            Box::new(ExactMatchSimilarity),
        ];

        Self { algorithms }
    }

    /// Calculate weighted similarity between two strings
    pub fn weighted_similarity(&self, a: &str, b: &str) -> f64 {
        let total_weight: f64 = self.algorithms.iter().map(|alg| alg.weight()).sum();
        let weighted_sum: f64 = self
            .algorithms
            .iter()
            .map(|alg| alg.calculate(a, b) * alg.weight())
            .sum();

        if total_weight == 0.0 {
            0.0
        } else {
            weighted_sum / total_weight
        }
    }

    /// Find the most similar string from a list of candidates
    pub fn find_most_similar(&self, target: &str, candidates: &[String]) -> Option<(String, f64)> {
        candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.clone(),
                    self.weighted_similarity(target, candidate),
                )
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get similarity breakdown for debugging
    pub fn similarity_breakdown(&self, a: &str, b: &str) -> Vec<(String, f64, f64)> {
        self.algorithms
            .iter()
            .map(|alg| {
                let similarity = alg.calculate(a, b);
                let weight = alg.weight();
                let name = std::any::type_name::<dyn SimilarityAlgorithm>();
                (name.to_string(), similarity, weight)
            })
            .collect()
    }
}

impl Default for SimilarityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }

    for (j, cell) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *cell = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_similarity() {
        let similarity = LevenshteinSimilarity;
        assert!((similarity.calculate("hello", "hello") - 1.0).abs() < f64::EPSILON);
        assert!((similarity.calculate("hello", "helo") - 0.8).abs() < 1e-9);
        assert!((similarity.calculate("", "hello") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaccard_similarity() {
        let similarity = JaccardSimilarity;
        assert!((similarity.calculate("hello", "hello") - 1.0).abs() < f64::EPSILON);
        assert!(similarity.calculate("hello", "world") < 1.0);
    }

    #[test]
    fn test_weighted_similarity() {
        let analyzer = SimilarityAnalyzer::new();
        let similarity = analyzer.weighted_similarity("MD033", "MD033");
        assert!((similarity - 1.0).abs() < f64::EPSILON);
    }
}
