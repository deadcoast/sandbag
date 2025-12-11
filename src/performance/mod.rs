//! Performance optimization and benchmarking modules

pub mod optimizations;

use crate::core::RuleMatch;
use crate::performance::optimizations::{OptimizedRuleProcessor, PerformanceMetrics};
use anyhow::Result;
use core::fmt::Write as _;
use std::time::{Duration, Instant};

/// Performance benchmark runner
pub struct PerformanceBenchmark {
    processor: OptimizedRuleProcessor,
    benchmarks: Vec<BenchmarkResult>,
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Time taken to execute the benchmark
    pub duration: Duration,
    /// Estimated memory usage in bytes
    pub memory_usage: usize,
    /// Number of operations recorded
    pub operations: u64,
    /// Cache hit rate in [0.0, 1.0]
    pub cache_hit_rate: f64,
}

impl PerformanceBenchmark {
    /// Create a new performance benchmark
    pub fn new() -> Self {
        Self {
            processor: OptimizedRuleProcessor::new(),
            benchmarks: Vec::new(),
        }
    }

    /// Run comprehensive benchmarks
    pub fn run_all_benchmarks(&mut self) -> Result<Vec<BenchmarkResult>> {
        let results = vec![
            self.benchmark_rule_processing()?,
            self.benchmark_similarity_calculation()?,
            self.benchmark_cache_performance()?,
            self.benchmark_memory_usage()?,
        ];

        self.benchmarks.clone_from(&results);
        Ok(results)
    }

    /// Benchmark rule processing performance
    #[allow(
        clippy::unnecessary_wraps,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::uninlined_format_args,
        clippy::cast_lossless
    )]
    fn benchmark_rule_processing(&self) -> Result<BenchmarkResult> {
        let start_time = Instant::now();

        // Create test rules
        let rules: Vec<RuleMatch> = (0..1000)
            .map(|i| RuleMatch {
                rule_id: format!("MD{:03}", i % 50),
                linter: "markdownlint".to_string(),
                confidence: 0.8 + f64::from(i % 20) * 0.01,
                context: crate::core::ExtractedContext {
                    file_path: None,
                    line_number: Some(u32::try_from(i).unwrap_or(u32::MAX)),
                    column: None,
                    message: Some(format!("Test rule {i}")),
                    severity: Some(crate::core::Severity::Warning),
                },
                suggested_action: crate::core::ConfigAction::Disable,
            })
            .collect();

        // Process rules in parallel
        let _config_entries = self.processor.process_rules_parallel(&rules);

        let duration = start_time.elapsed();
        let metrics = self.processor.get_metrics();

        Ok(BenchmarkResult {
            name: "Rule Processing".to_string(),
            duration,
            memory_usage: metrics.memory_usage,
            operations: metrics.total_operations,
            cache_hit_rate: if metrics.cache_hits + metrics.cache_misses > 0 {
                (metrics.cache_hits as f64) / ((metrics.cache_hits + metrics.cache_misses) as f64)
            } else {
                0.0
            },
        })
    }

    /// Benchmark similarity calculation performance
    #[allow(
        clippy::unnecessary_wraps,
        clippy::cast_precision_loss,
        clippy::uninlined_format_args
    )]
    fn benchmark_similarity_calculation(&self) -> Result<BenchmarkResult> {
        let start_time = Instant::now();

        // Create test strings
        let strings: Vec<String> = (0..100)
            .map(|i| format!("test_string_{i}_with_some_content_{}", i * 2))
            .collect();

        // Calculate similarity matrix
        let matrix = self.processor.calculate_similarity_matrix(&strings);

        let duration = start_time.elapsed();
        let metrics = self.processor.get_metrics();

        Ok(BenchmarkResult {
            name: "Similarity Calculation".to_string(),
            duration,
            memory_usage: matrix.len() * matrix[0].len() * std::mem::size_of::<f64>(),
            operations: metrics.total_operations,
            cache_hit_rate: if metrics.cache_hits + metrics.cache_misses > 0 {
                (metrics.cache_hits as f64) / ((metrics.cache_hits + metrics.cache_misses) as f64)
            } else {
                0.0
            },
        })
    }

    /// Benchmark cache performance
    #[allow(
        clippy::unnecessary_wraps,
        clippy::cast_precision_loss,
        clippy::uninlined_format_args
    )]
    fn benchmark_cache_performance(&self) -> Result<BenchmarkResult> {
        let start_time = Instant::now();

        // Test cache hits and misses
        for i in 0..1000 {
            let key = format!("test_key_{}", i % 100); // Some keys will be repeated
            self.processor
                .calculate_similarity_cached(&key, "test_value");
        }

        let duration = start_time.elapsed();
        let metrics = self.processor.get_metrics();

        Ok(BenchmarkResult {
            name: "Cache Performance".to_string(),
            duration,
            memory_usage: metrics.memory_usage,
            operations: metrics.total_operations,
            cache_hit_rate: if metrics.cache_hits + metrics.cache_misses > 0 {
                (metrics.cache_hits as f64) / ((metrics.cache_hits + metrics.cache_misses) as f64)
            } else {
                0.0
            },
        })
    }

    /// Benchmark memory usage
    #[allow(
        clippy::unnecessary_wraps,
        clippy::cast_precision_loss,
        clippy::uninlined_format_args
    )]
    fn benchmark_memory_usage(&self) -> Result<BenchmarkResult> {
        let start_time = Instant::now();

        // Create large dataset
        let large_dataset: Vec<String> = (0..10000)
            .map(|i| format!("large_test_string_{}_with_extensive_content_{}", i, i * 3))
            .collect();

        // Process dataset
        let results = self.processor.batch_process(&large_dataset);

        let duration = start_time.elapsed();
        let metrics = self.processor.get_metrics();

        Ok(BenchmarkResult {
            name: "Memory Usage".to_string(),
            duration,
            memory_usage: results.len() * std::mem::size_of::<Vec<crate::core::RuleMatch>>(),
            operations: metrics.total_operations,
            cache_hit_rate: if metrics.cache_hits + metrics.cache_misses > 0 {
                (metrics.cache_hits as f64) / ((metrics.cache_hits + metrics.cache_misses) as f64)
            } else {
                0.0
            },
        })
    }

    /// Get benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.benchmarks
    }

    /// Generate performance report
    #[allow(clippy::cast_precision_loss)]
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Benchmark Report\n");
        report.push_str("===========================\n\n");

        for benchmark in &self.benchmarks {
            let _ = writeln!(report, "Benchmark: {}", benchmark.name);
            let _ = writeln!(report, "  Duration: {:?}", benchmark.duration);
            let _ = writeln!(report, "  Memory Usage: {} bytes", benchmark.memory_usage);
            let _ = writeln!(report, "  Operations: {}", benchmark.operations);
            let _ = writeln!(
                report,
                "  Cache Hit Rate: {:.2}%",
                benchmark.cache_hit_rate * 100.0
            );
            report.push('\n');
        }

        // Calculate averages
        if !self.benchmarks.is_empty() {
            let count = self.benchmarks.len();
            let avg_duration: Duration =
                self.benchmarks.iter().map(|b| b.duration).sum::<Duration>()
                    / u32::try_from(count).unwrap_or(u32::MAX);

            let avg_memory: usize = self
                .benchmarks
                .iter()
                .map(|b| b.memory_usage)
                .sum::<usize>()
                / self.benchmarks.len();

            let avg_cache_hit_rate: f64 = self
                .benchmarks
                .iter()
                .map(|b| b.cache_hit_rate)
                .sum::<f64>()
                / (count as f64);

            report.push_str("Summary:\n");
            report.push_str("--------\n");
            let _ = writeln!(report, "Average Duration: {avg_duration:?}");
            let _ = writeln!(report, "Average Memory Usage: {avg_memory} bytes");
            let _ = writeln!(
                report,
                "Average Cache Hit Rate: {:.2}%",
                avg_cache_hit_rate * 100.0
            );
        }

        report
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.processor.get_metrics()
    }

    /// Clear processor caches
    pub fn clear_caches(&self) {
        self.processor.clear_caches();
    }
}

impl Default for PerformanceBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring
pub struct PerformanceMonitor {
    start_time: Instant,
    operations: Vec<OperationRecord>,
}

/// Operation record
#[derive(Debug, Clone)]
pub struct OperationRecord {
    /// Operation name
    pub name: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Memory usage during the operation
    pub memory_usage: usize,
    /// Timestamp when the operation was recorded
    pub timestamp: Instant,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operations: Vec::new(),
        }
    }

    /// Record an operation
    pub fn record_operation(&mut self, name: &str, duration: Duration, memory_usage: usize) {
        self.operations.push(OperationRecord {
            name: name.to_string(),
            duration,
            memory_usage,
            timestamp: Instant::now(),
        });
    }

    /// Get operation statistics
    pub fn get_statistics(&self) -> OperationStatistics {
        if self.operations.is_empty() {
            return OperationStatistics {
                total_operations: 0,
                total_duration: Duration::ZERO,
                average_duration: Duration::ZERO,
                total_memory_usage: 0,
                average_memory_usage: 0,
                slowest_operation: None,
                fastest_operation: None,
            };
        }

        let total_operations = self.operations.len();
        let total_duration: Duration = self.operations.iter().map(|op| op.duration).sum();
        let average_duration = total_duration / u32::try_from(total_operations).unwrap_or(u32::MAX);
        let total_memory_usage: usize = self.operations.iter().map(|op| op.memory_usage).sum();
        let average_memory_usage = total_memory_usage / total_operations;

        let slowest_operation = self.operations.iter().max_by_key(|op| op.duration).cloned();

        let fastest_operation = self.operations.iter().min_by_key(|op| op.duration).cloned();

        OperationStatistics {
            total_operations,
            total_duration,
            average_duration,
            total_memory_usage,
            average_memory_usage,
            slowest_operation,
            fastest_operation,
        }
    }

    /// Get uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Clear operation history
    pub fn clear_history(&mut self) {
        self.operations.clear();
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStatistics {
    /// Total number of operations recorded
    pub total_operations: usize,
    /// Aggregate duration of all operations
    pub total_duration: Duration,
    /// Average operation duration
    pub average_duration: Duration,
    /// Aggregate memory usage across operations
    pub total_memory_usage: usize,
    /// Average memory usage per operation
    pub average_memory_usage: usize,
    /// Slowest operation record
    pub slowest_operation: Option<OperationRecord>,
    /// Fastest operation record
    pub fastest_operation: Option<OperationRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_benchmark_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert!(benchmark.get_results().is_empty());
    }

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let stats = monitor.get_statistics();
        assert_eq!(stats.total_operations, 0);
    }

    #[test]
    fn test_operation_recording() {
        let mut monitor = PerformanceMonitor::new();
        monitor.record_operation("test", Duration::from_millis(100), 1024);

        let stats = monitor.get_statistics();
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.total_duration, Duration::from_millis(100));
        assert_eq!(stats.total_memory_usage, 1024);
    }

    #[test]
    fn test_uptime() {
        let monitor = PerformanceMonitor::new();
        let uptime = monitor.get_uptime();
        assert!(uptime > Duration::ZERO);
    }
}
