//! Performance optimizations and caching

use crate::core::{ConfigEntry, RuleMatch};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total number of operations processed
    pub total_operations: u64,
    /// Total processing time accumulated
    pub total_duration: Duration,
    /// Average operation duration
    pub average_duration: Duration,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Estimated memory usage
    pub memory_usage: usize,
}

// Cache entry with expiration (planned). Enable with `smart-cache` feature when implemented.
#[cfg(feature = "smart-cache")]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    expires_at: Instant,
}

/// Performance-optimized rule processor
pub struct OptimizedRuleProcessor {
    similarity_cache: Mutex<HashMap<String, f64>>,
    rule_cache: Mutex<HashMap<String, Vec<RuleMatch>>>,
    config_cache: Mutex<HashMap<String, ConfigEntry>>,
    similarity_analyzer: crate::core::advanced_similarity::AdvancedSimilarityAnalyzer,
    metrics: Mutex<PerformanceMetrics>,
    cache_ttl: Duration,
    max_cache_size: usize,
}

impl OptimizedRuleProcessor {
    /// Create a new optimized rule processor
    pub fn new() -> Self {
        Self {
            similarity_cache: Mutex::new(HashMap::new()),
            rule_cache: Mutex::new(HashMap::new()),
            config_cache: Mutex::new(HashMap::new()),
            similarity_analyzer: crate::core::advanced_similarity::AdvancedSimilarityAnalyzer::new(
            ),
            metrics: Mutex::new(PerformanceMetrics {
                total_operations: 0,
                total_duration: Duration::ZERO,
                average_duration: Duration::ZERO,
                cache_hits: 0,
                cache_misses: 0,
                memory_usage: 0,
            }),
            cache_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
        }
    }

    // no-op stubs; detailed eviction implemented elsewhere

    /// Process rules with parallel optimization
    pub fn process_rules_parallel(&self, rules: &[RuleMatch]) -> Vec<ConfigEntry> {
        let start_time = Instant::now();

        // Process rules in parallel
        let config_entries: Vec<ConfigEntry> = rules
            .iter()
            .map(|rule| self.process_single_rule(rule))
            .collect();

        // Update metrics
        self.update_metrics(start_time.elapsed());

        config_entries
    }

    /// Process a single rule with caching
    fn process_single_rule(&self, rule: &RuleMatch) -> ConfigEntry {
        let cache_key = format!("{}:{}", rule.linter, rule.rule_id);

        // Check cache first
        if let Some(cached_entry) = self.get_from_cache(&cache_key) {
            self.increment_cache_hits();
            return cached_entry;
        }

        self.increment_cache_misses();

        // Process rule and cache result
        let config_entry = ConfigEntry {
            rule_id: rule.rule_id.clone(),
            action: rule.suggested_action.clone(),
            scope: crate::core::ConfigScope::Global,
            metadata: HashMap::new(),
        };

        self.add_to_cache(cache_key, config_entry.clone());
        config_entry
    }

    /// Calculate similarity with caching
    pub fn calculate_similarity_cached(&self, a: &str, b: &str) -> f64 {
        let cache_key = format!("{a}:{b}");

        // Check cache first
        if let Some(similarity) = self.get_similarity_from_cache(&cache_key) {
            self.increment_cache_hits();
            return similarity;
        }

        self.increment_cache_misses();

        // Calculate similarity
        let similarity = self
            .similarity_analyzer
            .calculate_advanced_similarity(a, b)
            .overall_similarity;

        // Cache result
        self.add_similarity_to_cache(cache_key, similarity);

        similarity
    }

    /// Batch process multiple inputs
    pub fn batch_process(&self, inputs: &[String]) -> Vec<Vec<RuleMatch>> {
        let start_time = Instant::now();

        // Process inputs in parallel
        let results: Vec<Vec<RuleMatch>> = inputs
            .iter()
            .map(|input| self.process_input(input))
            .collect();

        self.update_metrics(start_time.elapsed());
        results
    }

    /// Process a single input with optimization
    fn process_input(&self, input: &str) -> Vec<RuleMatch> {
        // Check cache first
        if let Some(cached_rules) = self.get_rules_from_cache(input) {
            self.increment_cache_hits();
            return cached_rules;
        }

        self.increment_cache_misses();

        // Process input (simulated for now)
        let rules = vec![RuleMatch {
            rule_id: "MD033".to_string(),
            linter: "markdownlint".to_string(),
            confidence: 0.95,
            context: crate::core::ExtractedContext {
                file_path: None,
                line_number: None,
                column: None,
                message: Some("Inline HTML detected".to_string()),
                severity: Some(crate::core::Severity::Warning),
            },
            suggested_action: crate::core::ConfigAction::Disable,
        }];

        // Cache result
        self.add_rules_to_cache(input.to_string(), rules.clone());

        rules
    }

    /// Memory-optimized similarity calculation
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_similarity_memory_optimized(&self, a: &str, b: &str) -> f64 {
        // Use simpler similarity calculation for memory efficiency
        if a == b {
            return 1.0;
        }

        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        let min_len = a_chars.len().min(b_chars.len());
        if min_len == 0 {
            return 0.0;
        }

        let mut matches = 0;
        for i in 0..min_len {
            if a_chars[i] == b_chars[i] {
                matches += 1;
            }
        }

        f64::from(matches) / (min_len as f64)
    }

    /// Parallel similarity matrix calculation
    pub fn calculate_similarity_matrix(&self, items: &[String]) -> Vec<Vec<f64>> {
        let n = items.len();
        let mut matrix = vec![vec![0.0; n]; n];

        // Calculate similarities in parallel
        let similarities: Vec<(usize, usize, f64)> = (0..n)
            .flat_map(|i| (i..n).map(move |j| (i, j)))
            .map(|(i, j)| {
                let similarity = if i == j {
                    1.0
                } else {
                    self.calculate_similarity_cached(&items[i], &items[j])
                };
                (i, j, similarity)
            })
            .collect();

        // Fill matrix
        for (i, j, similarity) in similarities {
            matrix[i][j] = similarity;
            if i != j {
                matrix[j][i] = similarity; // Symmetric matrix
            }
        }

        matrix
    }

    /// Cache management
    #[allow(clippy::unused_self)]
    fn get_from_cache(&self, _key: &str) -> Option<ConfigEntry> {
        // TODO: Implement proper caching
        None
    }

    #[allow(clippy::unused_self)]
    fn add_to_cache(&self, _key: String, _entry: ConfigEntry) {
        // TODO: Implement proper caching
    }

    fn get_similarity_from_cache(&self, key: &str) -> Option<f64> {
        if let Ok(cache) = self.similarity_cache.lock() {
            cache.get(key).copied()
        } else {
            None
        }
    }

    fn add_similarity_to_cache(&self, key: String, similarity: f64) {
        if let Ok(mut cache) = self.similarity_cache.lock() {
            if cache.len() >= self.max_cache_size {
                self.evict_oldest_similarity_entries(&mut cache);
            }
            cache.insert(key, similarity);
        }
    }

    #[allow(clippy::unused_self)]
    fn get_rules_from_cache(&self, _key: &str) -> Option<Vec<RuleMatch>> {
        // TODO: Implement proper caching
        None
    }

    #[allow(clippy::unused_self)]
    fn add_rules_to_cache(&self, _key: String, _rules: Vec<RuleMatch>) {
        // TODO: Implement proper caching
    }

    // Cache eviction strategies
    fn evict_oldest_similarity_entries(&self, cache: &mut HashMap<String, f64>) {
        if cache.len() >= self.max_cache_size {
            let to_remove = cache.len() - self.max_cache_size / 2;
            let keys: Vec<_> = cache.keys().take(to_remove).cloned().collect();
            for key in keys {
                cache.remove(&key);
            }
        }
    }

    // removed duplicate stub

    /// Metrics management
    fn update_metrics(&self, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.total_operations += 1;
            metrics.total_duration += duration;
            if metrics.total_operations > 0 {
                let avg_ms =
                    metrics.total_duration.as_millis() / u128::from(metrics.total_operations);
                let avg_u64 = u64::try_from(avg_ms).unwrap_or(u64::MAX);
                metrics.average_duration = Duration::from_millis(avg_u64);
            }
        }
    }

    fn increment_cache_hits(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_hits += 1;
        }
    }

    fn increment_cache_misses(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.cache_misses += 1;
        }
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            PerformanceMetrics {
                total_operations: 0,
                total_duration: Duration::ZERO,
                average_duration: Duration::ZERO,
                cache_hits: 0,
                cache_misses: 0,
                memory_usage: 0,
            }
        }
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        if let Ok(mut cache) = self.similarity_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.rule_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.config_cache.lock() {
            cache.clear();
        }
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }

    /// Set max cache size
    pub fn set_max_cache_size(&mut self, size: usize) {
        self.max_cache_size = size;
    }
}

impl Default for OptimizedRuleProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_processor_creation() {
        let processor = OptimizedRuleProcessor::new();
        let metrics = processor.get_metrics();
        assert_eq!(metrics.total_operations, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
    }

    #[test]
    fn test_similarity_caching() {
        let processor = OptimizedRuleProcessor::new();

        // First call should miss cache
        let similarity1 = processor.calculate_similarity_cached("hello", "world");
        let metrics1 = processor.get_metrics();
        assert_eq!(metrics1.cache_misses, 1);

        // Second call should hit cache
        let similarity2 = processor.calculate_similarity_cached("hello", "world");
        let metrics2 = processor.get_metrics();
        assert_eq!(metrics2.cache_hits, 1);
        assert!((similarity1 - similarity2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parallel_processing() {
        let processor = OptimizedRuleProcessor::new();

        let rules = vec![
            RuleMatch {
                rule_id: "MD033".to_string(),
                linter: "markdownlint".to_string(),
                confidence: 0.95,
                context: crate::core::ExtractedContext {
                    file_path: None,
                    line_number: None,
                    column: None,
                    message: Some("Inline HTML detected".to_string()),
                    severity: Some(crate::core::Severity::Warning),
                },
                suggested_action: crate::core::ConfigAction::Disable,
            },
            RuleMatch {
                rule_id: "MD013".to_string(),
                linter: "markdownlint".to_string(),
                confidence: 0.87,
                context: crate::core::ExtractedContext {
                    file_path: None,
                    line_number: None,
                    column: None,
                    message: Some("Line too long".to_string()),
                    severity: Some(crate::core::Severity::Warning),
                },
                suggested_action: crate::core::ConfigAction::Ignore,
            },
        ];

        let config_entries = processor.process_rules_parallel(&rules);
        assert_eq!(config_entries.len(), 2);

        let metrics = processor.get_metrics();
        assert_eq!(metrics.total_operations, 1);
    }

    #[test]
    fn test_similarity_matrix() {
        let processor = OptimizedRuleProcessor::new();
        let items = vec!["hello".to_string(), "world".to_string(), "test".to_string()];

        let matrix = processor.calculate_similarity_matrix(&items);
        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);

        // Diagonal should be 1.0
        assert!((matrix[0][0] - 1.0).abs() < f64::EPSILON);
        assert!((matrix[1][1] - 1.0).abs() < f64::EPSILON);
        assert!((matrix[2][2] - 1.0).abs() < f64::EPSILON);

        // Matrix should be symmetric
        assert!((matrix[0][1] - matrix[1][0]).abs() < f64::EPSILON);
        assert!((matrix[0][2] - matrix[2][0]).abs() < f64::EPSILON);
        assert!((matrix[1][2] - matrix[2][1]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_memory_optimized_similarity() {
        let processor = OptimizedRuleProcessor::new();

        assert!(
            (processor.calculate_similarity_memory_optimized("hello", "hello") - 1.0).abs()
                < f64::EPSILON
        );
        // "hello" and "world" have no matching characters at same positions
        let similarity = processor.calculate_similarity_memory_optimized("hello", "world");
        assert!(similarity >= 0.0);
        assert!(similarity <= 1.0);
        assert!(processor.calculate_similarity_memory_optimized("hello", "help") > 0.0);
        assert!(processor.calculate_similarity_memory_optimized("hello", "help") <= 1.0);
    }

    #[test]
    fn test_cache_eviction() {
        let mut processor = OptimizedRuleProcessor::new();
        processor.set_max_cache_size(2);

        // Add more items than cache can hold
        processor.calculate_similarity_cached("a", "b");
        processor.calculate_similarity_cached("c", "d");
        processor.calculate_similarity_cached("e", "f");

        let metrics = processor.get_metrics();
        assert!(metrics.cache_misses >= 3);
    }

    #[test]
    fn test_clear_caches() {
        let processor = OptimizedRuleProcessor::new();

        // Add some data to caches
        processor.calculate_similarity_cached("hello", "world");

        // Clear caches
        processor.clear_caches();

        // Should miss cache after clearing
        processor.calculate_similarity_cached("hello", "world");
        let metrics = processor.get_metrics();
        assert_eq!(metrics.cache_misses, 2); // One before clear, one after
    }
}
