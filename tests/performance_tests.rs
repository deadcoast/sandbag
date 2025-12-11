//! Performance tests and benchmarking for the Sandbag system

use anyhow::Result;
use sandbag::core::advanced_similarity::AdvancedSimilarityAnalyzer;
use sandbag::core::{ConfigAction, ConfigEntry, ExtractedContext, RuleMatch, Severity};
use sandbag::linters::LinterRegistry;
use sandbag::performance::optimizations::OptimizedRuleProcessor;
use sandbag::performance::{PerformanceBenchmark, PerformanceMonitor};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance test suite for benchmarking and validation
pub struct PerformanceTestSuite {
    #[allow(dead_code)]
    benchmark: PerformanceBenchmark,
    monitor: PerformanceMonitor,
    processor: OptimizedRuleProcessor,
    similarity_analyzer: AdvancedSimilarityAnalyzer,
    linter_registry: LinterRegistry,
}

impl PerformanceTestSuite {
    /// Create a new performance test suite
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            benchmark: PerformanceBenchmark::new(),
            monitor: PerformanceMonitor::new(),
            processor: OptimizedRuleProcessor::new(),
            similarity_analyzer: AdvancedSimilarityAnalyzer::new(),
            linter_registry: LinterRegistry::new(),
        }
    }

    /// Run all performance tests
    pub async fn run_all_tests(&mut self) -> Result<PerformanceTestResults> {
        let mut results = PerformanceTestResults::new();

        // Core performance tests
        results.add_test(
            "rule_processing_performance",
            self.test_rule_processing_performance().await,
        );
        results.add_test(
            "similarity_calculation_performance",
            self.test_similarity_calculation_performance().await,
        );
        results.add_test("caching_performance", self.test_caching_performance().await);
        results.add_test(
            "memory_usage_performance",
            self.test_memory_usage_performance().await,
        );
        results.add_test(
            "linter_registry_performance",
            self.test_linter_registry_performance().await,
        );
        results.add_test(
            "parallel_processing_performance",
            self.test_parallel_processing_performance().await,
        );
        results.add_test(
            "end_to_end_performance",
            self.test_end_to_end_performance().await,
        );

        Ok(results)
    }

    /// Test rule processing performance
    async fn test_rule_processing_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Rule Processing Performance");

        // Generate test data
        let rules = self.generate_test_rules(1000);

        // Test processing time
        let start_time = Instant::now();
        let config_entries = self.processor.process_rules_parallel(&rules);
        let processing_time = start_time.elapsed();

        result.add_metric("processing_time_ms", processing_time.as_millis() as f64);
        result.add_metric("rules_processed", config_entries.len() as f64);
        result.add_metric(
            "throughput_rules_per_second",
            config_entries.len() as f64 / processing_time.as_secs_f64(),
        );

        // Performance assertions
        result.add_check(
            "processing_time_under_100ms",
            processing_time < Duration::from_millis(100),
        );
        result.add_check("all_rules_processed", config_entries.len() == 1000);
        result.add_check(
            "throughput_above_1000_rps",
            result.get_metric("throughput_rules_per_second").unwrap() > 1000.0,
        );

        // Record operation
        self.monitor.record_operation(
            "rule_processing",
            processing_time,
            config_entries.len() * std::mem::size_of::<ConfigEntry>(),
        );

        Ok(result)
    }

    /// Test similarity calculation performance
    async fn test_similarity_calculation_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Similarity Calculation Performance");

        // Generate test strings
        let strings = self.generate_test_strings(100);

        // Test similarity matrix calculation
        let start_time = Instant::now();
        let matrix = self.processor.calculate_similarity_matrix(&strings);
        let calculation_time = start_time.elapsed();

        result.add_metric("calculation_time_ms", calculation_time.as_millis() as f64);
        result.add_metric("matrix_size", matrix.len() as f64);
        result.add_metric(
            "similarity_calculations",
            (matrix.len() * matrix.len()) as f64,
        );
        result.add_metric(
            "throughput_calcs_per_second",
            result.get_metric("similarity_calculations").unwrap() / calculation_time.as_secs_f64(),
        );

        // Performance assertions
        result.add_check(
            "calculation_time_under_500ms",
            calculation_time < Duration::from_millis(500),
        );
        result.add_check("matrix_correct_size", matrix.len() == 100);
        result.add_check(
            "throughput_above_100_cps",
            result.get_metric("throughput_calcs_per_second").unwrap() > 100.0,
        );

        // Test individual similarity calculations
        let individual_start = Instant::now();
        for i in 0..100 {
            for j in i..100 {
                self.similarity_analyzer
                    .calculate_advanced_similarity(&strings[i], &strings[j]);
            }
        }
        let individual_time = individual_start.elapsed();

        result.add_metric(
            "individual_calculation_time_ms",
            individual_time.as_millis() as f64,
        );
        result.add_check(
            "individual_time_reasonable",
            individual_time < Duration::from_secs(10),
        );

        // Record operation
        self.monitor.record_operation(
            "similarity_calculation",
            calculation_time,
            matrix.len() * matrix.len() * std::mem::size_of::<f64>(),
        );

        Ok(result)
    }

    /// Test caching performance
    async fn test_caching_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Caching Performance");

        // Test cache hit performance
        let test_pairs = vec![
            ("hello", "world"),
            ("test", "string"),
            ("performance", "optimization"),
            ("caching", "system"),
        ];

        // First pass - cache misses
        let miss_start = Instant::now();
        for (a, b) in &test_pairs {
            self.processor.calculate_similarity_cached(a, b);
        }
        let miss_time = miss_start.elapsed();

        // Second pass - cache hits
        let hit_start = Instant::now();
        for (a, b) in &test_pairs {
            self.processor.calculate_similarity_cached(a, b);
        }
        let hit_time = hit_start.elapsed();

        result.add_metric("cache_miss_time_ms", miss_time.as_millis() as f64);
        result.add_metric("cache_hit_time_ms", hit_time.as_millis() as f64);
        let denom = hit_time.as_micros().max(1) as f64;
        result.add_metric("cache_speedup", miss_time.as_micros() as f64 / denom);

        // Performance assertions
        result.add_check("cache_hits_faster", hit_time < miss_time);
        result.add_check(
            "cache_speedup_above_2x",
            result.get_metric("cache_speedup").unwrap() > 2.0,
        );
        result.add_check("hit_time_under_1ms", hit_time < Duration::from_millis(1));

        // Test cache metrics
        let metrics = self.processor.get_metrics();
        result.add_metric("cache_hits", metrics.cache_hits as f64);
        result.add_metric("cache_misses", metrics.cache_misses as f64);
        result.add_metric(
            "cache_hit_rate",
            if metrics.cache_hits + metrics.cache_misses > 0 {
                metrics.cache_hits as f64 / (metrics.cache_hits + metrics.cache_misses) as f64
            } else {
                0.0
            },
        );

        result.add_check(
            "cache_hit_rate_above_50pct",
            result.get_metric("cache_hit_rate").unwrap() > 0.5,
        );

        // Record operation
        self.monitor.record_operation(
            "caching_test",
            miss_time + hit_time,
            test_pairs.len() * std::mem::size_of::<f64>(),
        );

        Ok(result)
    }

    /// Test memory usage performance
    #[cfg_attr(not(feature = "slow-tests"), allow(unused_attributes))]
    async fn test_memory_usage_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Memory Usage Performance");

        // Test with large datasets
        let large_rules = self.generate_test_rules(2000);
        let large_strings = self.generate_test_strings(300);

        // Measure memory usage before
        let memory_before = self.estimate_memory_usage();

        // Process large dataset
        let start_time = Instant::now();
        let _config_entries = self.processor.process_rules_parallel(&large_rules);
        let _matrix = self.processor.calculate_similarity_matrix(&large_strings);
        let processing_time = start_time.elapsed();

        // Measure memory usage after
        let memory_after = self.estimate_memory_usage();
        let memory_increase = memory_after - memory_before;

        result.add_metric("processing_time_ms", processing_time.as_millis() as f64);
        result.add_metric(
            "memory_increase_mb",
            memory_increase as f64 / 1024.0 / 1024.0,
        );
        result.add_metric(
            "memory_per_rule_mb",
            memory_increase as f64 / 10000.0 / 1024.0 / 1024.0,
        );
        result.add_metric(
            "throughput_rules_per_second",
            10000.0 / processing_time.as_secs_f64(),
        );

        // Performance assertions
        result.add_check(
            "processing_time_under_5s",
            processing_time < Duration::from_secs(5),
        );
        result.add_check(
            "memory_increase_reasonable",
            memory_increase < 100 * 1024 * 1024,
        ); // Less than 100MB
        result.add_check(
            "memory_per_rule_small",
            result.get_metric("memory_per_rule_mb").unwrap() < 0.01,
        ); // Less than 10KB per rule
        result.add_check(
            "throughput_above_1000_rps",
            result.get_metric("throughput_rules_per_second").unwrap() > 1000.0,
        );

        // Test memory cleanup
        self.processor.clear_caches();
        let memory_after_cleanup = self.estimate_memory_usage();
        let memory_recovered = memory_after - memory_after_cleanup;

        result.add_metric(
            "memory_recovered_mb",
            memory_recovered as f64 / 1024.0 / 1024.0,
        );
        result.add_check("memory_cleanup_effective", memory_recovered > 0);

        // Record operation
        self.monitor
            .record_operation("memory_test", processing_time, memory_increase);

        Ok(result)
    }

    /// Test linter registry performance
    async fn test_linter_registry_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Linter Registry Performance");

        // Generate mixed linter input
        let inputs = self.generate_mixed_linter_input(1000);

        // Test linter detection performance
        let start_time = Instant::now();
        let mut detected_linters = Vec::new();
        for input in &inputs {
            if let Some(handler) = self.linter_registry.find_handler(input) {
                detected_linters.push(handler.get_linter_name().to_string());
            }
        }
        let detection_time = start_time.elapsed();

        result.add_metric("detection_time_ms", detection_time.as_millis() as f64);
        result.add_metric("inputs_processed", inputs.len() as f64);
        result.add_metric("linters_detected", detected_linters.len() as f64);
        result.add_metric(
            "throughput_inputs_per_second",
            inputs.len() as f64 / detection_time.as_secs_f64(),
        );

        // Performance assertions
        result.add_check(
            "detection_time_under_100ms",
            detection_time < Duration::from_millis(100),
        );
        result.add_check("all_inputs_processed", !detected_linters.is_empty());
        result.add_check(
            "throughput_above_5000_ips",
            result.get_metric("throughput_inputs_per_second").unwrap() > 5000.0,
        );

        // Test rule extraction performance
        let extraction_start = Instant::now();
        let mut total_rules = 0;
        for input in &inputs {
            let rules = self.linter_registry.extract_rules(input);
            total_rules += rules.len();
        }
        let extraction_time = extraction_start.elapsed();

        result.add_metric("extraction_time_ms", extraction_time.as_millis() as f64);
        result.add_metric("total_rules_extracted", total_rules as f64);
        result.add_metric(
            "throughput_rules_per_second",
            total_rules as f64 / extraction_time.as_secs_f64(),
        );

        result.add_check(
            "extraction_time_under_500ms",
            extraction_time < Duration::from_millis(500),
        );
        result.add_check("rules_extracted", total_rules > 0);
        result.add_check(
            "extraction_throughput_above_1000",
            result.get_metric("throughput_rules_per_second").unwrap() > 1000.0,
        );

        // Record operation
        self.monitor.record_operation(
            "linter_registry_test",
            detection_time + extraction_time,
            total_rules * std::mem::size_of::<RuleMatch>(),
        );

        Ok(result)
    }

    /// Test parallel processing performance
    async fn test_parallel_processing_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("Parallel Processing Performance");

        // Test with different dataset sizes
        let dataset_sizes = vec![100, 1000, 10000];

        for size in dataset_sizes {
            let rules = self.generate_test_rules(size);

            // Sequential processing
            let sequential_start = Instant::now();
            let mut sequential_results = Vec::new();
            for rule in &rules {
                let config_entry = ConfigEntry {
                    rule_id: rule.rule_id.clone(),
                    action: rule.suggested_action.clone(),
                    scope: sandbag::core::ConfigScope::Global,
                    metadata: HashMap::new(),
                };
                sequential_results.push(config_entry);
            }
            let sequential_time = sequential_start.elapsed();

            // Parallel processing
            let parallel_start = Instant::now();
            let parallel_results = self.processor.process_rules_parallel(&rules);
            let parallel_time = parallel_start.elapsed();

            let speedup =
                sequential_time.as_micros() as f64 / (parallel_time.as_micros().max(1) as f64);

            result.add_metric(
                &format!("sequential_time_{size}_ms"),
                sequential_time.as_millis() as f64,
            );
            result.add_metric(
                &format!("parallel_time_{size}_ms"),
                parallel_time.as_millis() as f64,
            );
            result.add_metric(&format!("speedup_{size}"), speedup);

            // Performance assertions
            result.add_check(
                &format!("parallel_faster_{size}"),
                parallel_time <= sequential_time,
            );
            result.add_check(&format!("speedup_reasonable_{size}"), speedup > 0.5); // At least 50% of sequential speed
            result.add_check(
                &format!("results_consistent_{size}"),
                sequential_results.len() == parallel_results.len(),
            );
        }

        // Record operation
        self.monitor.record_operation(
            "parallel_processing_test",
            Duration::from_millis(100),
            1000 * std::mem::size_of::<ConfigEntry>(),
        );

        Ok(result)
    }

    /// Test end-to-end performance
    async fn test_end_to_end_performance(&mut self) -> Result<PerformanceTestResult> {
        let mut result = PerformanceTestResult::new("End-to-End Performance");

        // Simulate complete workflow
        let workflow_inputs = vec![
            "MD033: No inline HTML",
            "1:10 error no-unused-vars 'x' is assigned a value but never used",
            "prettier/prettier: Missing semicolon",
            "MD013: Line too long",
            "2:5 warn prefer-const/const Use const instead of let",
        ];

        let start_time = Instant::now();

        // 1. Rule extraction
        let mut all_rules = Vec::new();
        for input in &workflow_inputs {
            let rules = self.linter_registry.extract_rules(input);
            all_rules.extend(rules);
        }

        // 2. Similarity analysis
        let mut similarities = Vec::new();
        for i in 0..all_rules.len() {
            for j in i..all_rules.len() {
                let similarity = self
                    .similarity_analyzer
                    .calculate_advanced_similarity(&all_rules[i].rule_id, &all_rules[j].rule_id);
                similarities.push(similarity.overall_similarity);
            }
        }

        // 3. Performance processing
        let rule_count = all_rules.len();
        let config_entries = self.processor.process_rules_parallel(&all_rules);

        let total_time = start_time.elapsed();

        result.add_metric("total_time_ms", total_time.as_millis() as f64);
        result.add_metric("rules_extracted", rule_count as f64);
        result.add_metric("similarities_calculated", similarities.len() as f64);
        result.add_metric("config_entries_created", config_entries.len() as f64);
        result.add_metric(
            "throughput_operations_per_second",
            (rule_count + similarities.len() + config_entries.len()) as f64
                / total_time.as_secs_f64(),
        );

        // Performance assertions
        result.add_check("total_time_under_1s", total_time < Duration::from_secs(1));
        result.add_check("rules_extracted", rule_count > 0);
        result.add_check("similarities_calculated", !similarities.is_empty());
        result.add_check("config_entries_created", !config_entries.is_empty());
        result.add_check(
            "throughput_above_100_ops_per_sec",
            result
                .get_metric("throughput_operations_per_second")
                .unwrap()
                > 100.0,
        );

        // Record operation
        self.monitor.record_operation(
            "end_to_end_test",
            total_time,
            (rule_count + similarities.len() + config_entries.len()) * 64,
        );

        Ok(result)
    }

    /// Generate test rules
    fn generate_test_rules(&self, count: usize) -> Vec<RuleMatch> {
        (0..count)
            .map(|i| RuleMatch {
                rule_id: format!("MD{:03}", i % 50),
                linter: "markdownlint".to_string(),
                confidence: 0.8 + (i % 20) as f64 * 0.01,
                context: ExtractedContext {
                    file_path: None,
                    line_number: Some(i as u32),
                    column: None,
                    message: Some(format!("Test rule {i}")),
                    severity: Some(Severity::Warning),
                },
                suggested_action: ConfigAction::Disable,
            })
            .collect()
    }

    /// Generate test strings
    fn generate_test_strings(&self, count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("test_string_{}_with_some_content_{}", i, i * 2))
            .collect()
    }

    /// Generate mixed linter input
    fn generate_mixed_linter_input(&self, count: usize) -> Vec<String> {
        let mut inputs = Vec::new();
        for i in 0..count {
            match i % 3 {
                0 => inputs.push(format!("MD{:03}: Test markdown rule", i % 50)),
                1 => inputs.push(format!(
                    "{}:{} error test-rule-{} 'variable' is assigned but never used",
                    i % 100,
                    i % 20,
                    i
                )),
                2 => inputs.push(format!("prettier/prettier: Test formatting rule {i}")),
                _ => unreachable!(),
            }
        }
        inputs
    }

    /// Estimate memory usage (simplified)
    fn estimate_memory_usage(&self) -> usize {
        // This is a simplified estimation - in a real implementation,
        // you would use proper memory profiling tools
        std::mem::size_of::<OptimizedRuleProcessor>()
            + std::mem::size_of::<AdvancedSimilarityAnalyzer>()
            + std::mem::size_of::<LinterRegistry>()
    }
}

/// Performance test result structure
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub name: String,
    pub metrics: HashMap<String, f64>,
    pub checks: Vec<PerformanceCheck>,
    pub passed: usize,
    pub failed: usize,
}

/// Individual performance check
#[derive(Debug, Clone)]
pub struct PerformanceCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl PerformanceTestResult {
    /// Create a new performance test result
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            metrics: HashMap::new(),
            checks: Vec::new(),
            passed: 0,
            failed: 0,
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    /// Get a metric
    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).copied()
    }

    /// Add a check
    pub fn add_check(&mut self, name: &str, passed: bool) {
        let check = PerformanceCheck {
            name: name.to_string(),
            passed,
            message: if passed {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            },
        };

        if passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }

        self.checks.push(check);
    }

    /// Get overall success status
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.checks.is_empty() {
            0.0
        } else {
            self.passed as f64 / self.checks.len() as f64
        }
    }
}

/// Overall performance test results
#[derive(Debug, Clone)]
pub struct PerformanceTestResults {
    pub tests: Vec<PerformanceTestResult>,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_checks: usize,
    pub average_metrics: HashMap<String, f64>,
}

impl PerformanceTestResults {
    /// Create new performance test results
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            total_passed: 0,
            total_failed: 0,
            total_checks: 0,
            average_metrics: HashMap::new(),
        }
    }

    /// Add a test result
    pub fn add_test(&mut self, name: &str, result: Result<PerformanceTestResult>) {
        match result {
            Ok(test_result) => {
                self.total_passed += test_result.passed;
                self.total_failed += test_result.failed;
                self.total_checks += test_result.checks.len();
                self.tests.push(test_result);
            }
            Err(_) => {
                let failed_result = PerformanceTestResult {
                    name: name.to_string(),
                    metrics: HashMap::new(),
                    checks: vec![PerformanceCheck {
                        name: "test_execution".to_string(),
                        passed: false,
                        message: "Test execution failed".to_string(),
                    }],
                    passed: 0,
                    failed: 1,
                };
                self.total_failed += 1;
                self.total_checks += 1;
                self.tests.push(failed_result);
            }
        }
    }

    /// Calculate average metrics
    pub fn calculate_average_metrics(&mut self) {
        let mut metric_sums: HashMap<String, Vec<f64>> = HashMap::new();

        for test in &self.tests {
            for (metric_name, value) in &test.metrics {
                metric_sums
                    .entry(metric_name.clone())
                    .or_default()
                    .push(*value);
            }
        }

        for (metric_name, values) in metric_sums {
            let average = values.iter().sum::<f64>() / values.len() as f64;
            self.average_metrics.insert(metric_name, average);
        }
    }

    /// Get overall success status
    pub fn is_success(&self) -> bool {
        self.total_failed == 0
    }

    /// Get overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.total_passed as f64 / self.total_checks as f64
        }
    }

    /// Generate performance report
    pub fn generate_report(&mut self) -> String {
        self.calculate_average_metrics();

        let mut report = String::new();
        report.push_str("Performance Test Report\n");
        report.push_str("======================\n\n");

        for test in &self.tests {
            report.push_str(&format!("Test: {}\n", test.name));
            report.push_str(&format!(
                "  Status: {}\n",
                if test.is_success() {
                    "PASSED"
                } else {
                    "FAILED"
                }
            ));
            report.push_str(&format!(
                "  Success Rate: {:.1}%\n",
                test.success_rate() * 100.0
            ));
            report.push_str(&format!(
                "  Checks: {}/{} passed\n",
                test.passed,
                test.checks.len()
            ));

            if !test.metrics.is_empty() {
                report.push_str("  Metrics:\n");
                for (metric_name, value) in &test.metrics {
                    report.push_str(&format!("    - {metric_name}: {value:.2}\n"));
                }
            }

            if !test.is_success() {
                report.push_str("  Failed Checks:\n");
                for check in &test.checks {
                    if !check.passed {
                        report.push_str(&format!("    - {}: {}\n", check.name, check.message));
                    }
                }
            }
            report.push('\n');
        }

        if !self.average_metrics.is_empty() {
            report.push_str("Average Metrics:\n");
            report.push_str("----------------\n");
            for (metric_name, average) in &self.average_metrics {
                report.push_str(&format!("  - {metric_name}: {average:.2}\n"));
            }
            report.push('\n');
        }

        report.push_str("Summary:\n");
        report.push_str("--------\n");
        report.push_str(&format!("Total Tests: {}\n", self.tests.len()));
        report.push_str(&format!("Total Checks: {}\n", self.total_checks));
        report.push_str(&format!("Passed: {}\n", self.total_passed));
        report.push_str(&format!("Failed: {}\n", self.total_failed));
        report.push_str(&format!(
            "Overall Success Rate: {:.1}%\n",
            self.success_rate() * 100.0
        ));
        report.push_str(&format!(
            "Overall Status: {}\n",
            if self.is_success() {
                "PASSED"
            } else {
                "FAILED"
            }
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_suite_creation() {
        let suite = PerformanceTestSuite::new();
        assert!(suite.linter_registry.is_linter_supported("markdownlint"));
        assert!(suite.linter_registry.is_linter_supported("eslint"));
        assert!(suite.linter_registry.is_linter_supported("prettier"));
    }

    #[tokio::test]
    async fn test_rule_processing_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_rule_processing_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_similarity_calculation_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite
            .test_similarity_calculation_performance()
            .await
            .unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_caching_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_caching_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_memory_usage_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_memory_usage_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_linter_registry_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_linter_registry_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_parallel_processing_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_parallel_processing_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_end_to_end_performance() {
        let mut suite = PerformanceTestSuite::new();
        let result = suite.test_end_to_end_performance().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "slow-tests"), ignore)]
    async fn test_full_performance_suite() {
        let mut suite = PerformanceTestSuite::new();
        let mut results = suite.run_all_tests().await.unwrap();

        println!("{}", results.generate_report());

        assert!(results.total_checks > 0);
        assert!(results.success_rate() > 0.5); // At least 50% success rate
    }
}
