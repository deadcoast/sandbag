# Sandbag Examples

This document provides comprehensive examples of how to use Sandbag for various
linter configuration management scenarios.

## Basic Examples

### 1. Rule Extraction

Extract rules from linter output:

```rust
use sandbag::linters::LinterRegistry;

let registry = LinterRegistry::new();

// Extract markdownlint rules
let markdownlint_output = "MD033: No inline HTML\nMD013: Line too long";
let rules = registry.extract_rules(markdownlint_output);

for rule in rules {
    println!("Rule: {}, Confidence: {:.2}", rule.rule_id, rule.confidence);
}
```

### 2. Similarity Analysis

Compare rules for similarity:

```rust
use sandbag::core::advanced_similarity::AdvancedSimilarityAnalyzer;

let analyzer = AdvancedSimilarityAnalyzer::new();

let similarity = analyzer.calculate_advanced_similarity("MD033", "MD034");
println!("Similarity: {:.2}", similarity.overall_similarity);
println!("Fractal similarity: {:.2}", similarity.fractal_similarity);
println!("Spectral similarity: {:.2}", similarity.spectral_similarity);
```

### 3. Configuration Management

Create and manage configuration files:

```rust
use sandbag::config::{ConfigAST, ConfigFormat, ConfigManager};
use sandbag::core::{ConfigAction, ConfigScope};
use std::collections::HashMap;

let mut manager = ConfigManager::new();

// Create configuration entries
let entries = vec![
    ConfigEntry {
        rule_id: "MD033".to_string(),
        action: ConfigAction::Disable,
        scope: ConfigScope::Global,
        metadata: HashMap::new(),
    },
    ConfigEntry {
        rule_id: "MD013".to_string(),
        action: ConfigAction::Ignore,
        scope: ConfigScope::FileSpecific,
        metadata: HashMap::new(),
    },
];

let ast = ConfigAST::new(entries);

// Serialize to JSON
let json_config = manager.serialize_config(&ast, ConfigFormat::Json)?;
println!("JSON Config: {}", json_config);

// Serialize to YAML
let yaml_config = manager.serialize_config(&ast, ConfigFormat::Yaml)?;
println!("YAML Config: {}", yaml_config);
```

## Advanced Examples

### 1. Performance Optimization

Use the performance-optimized rule processor:

```rust
use sandbag::performance::optimizations::OptimizedRuleProcessor;

let mut processor = OptimizedRuleProcessor::new();

// Process rules in parallel
let rules = vec![/* your rules */];
let config_entries = processor.process_rules_parallel(rules);

// Calculate similarity with caching
let similarity = processor.calculate_similarity_cached("rule1", "rule2");

// Get performance metrics
let metrics = processor.get_metrics();
println!("Total operations: {}", metrics.total_operations);
println!("Cache hits: {}", metrics.cache_hits);
println!("Cache misses: {}", metrics.cache_misses);
```

### 2. CLI Integration

Use the enhanced CLI interface:

```rust
use sandbag::ui::enhanced_cli::EnhancedCliRunner;

let mut cli = EnhancedCliRunner::new();

// Run interactive scan
cli.run_interactive_scan("/path/to/project").await?;

// Show progress for rule processing
cli.show_rule_processing_progress(&rules).await?;

// Display similarity analysis results
cli.display_similarity_results(&similarities).await?;
```

### 3. Error Recovery

Handle errors gracefully:

```rust
use sandbag::ui::error_recovery::ErrorRecoveryManager;

let mut recovery = ErrorRecoveryManager::new();
recovery.set_max_retries(3);
recovery.set_interactive_mode(true);

// Handle operation with retry logic
let result = recovery.execute_with_retry(|| {
    // Your operation here
    perform_risky_operation()
}).await?;
```

## Real-World Scenarios

### 1. Migrating from ESLint to Prettier

```rust
use sandbag::linters::{LinterRegistry, LinterFactory};

let registry = LinterRegistry::new();

// Extract ESLint rules
let eslint_output = "1:10 error no-unused-vars 'x' is assigned a value but 
never used";
let eslint_rules = registry.extract_rules(eslint_output);

// Find equivalent Prettier rules
for rule in eslint_rules {
    let prettier_handler = LinterFactory::create_handler("prettier").unwrap();
    let equivalent_rules = prettier_handler.find_equivalent_rules(&rule.rule_id);
    
    println!("ESLint rule {} -> Prettier rules: {:?}", rule.rule_id, equivalent_rules);
}
```

### 2. Bulk Rule Configuration

```rust
use sandbag::core::rule_extractor::RuleExtractor;
use sandbag::config::manager::ConfigManager;

let extractor = RuleExtractor::new();
let mut manager = ConfigManager::new();

// Extract all rules from project
let all_rules = extractor.extract_rules_from_directory("/path/to/project").await?;

// Group similar rules
let grouped_rules = group_rules_by_similarity(&all_rules);

// Create bulk configuration
for (group, rules) in grouped_rules {
    let config_entry = ConfigEntry {
        rule_id: group.to_string(),
        action: ConfigAction::Disable,
        scope: ConfigScope::Global,
        metadata: HashMap::new(),
    };
    
    manager.add_config_entry(config_entry)?;
}

// Generate configuration file
manager.generate_config_file("bulk_config.json", ConfigFormat::Json)?;
```

### 3. Performance Benchmarking

```rust
use sandbag::performance::{PerformanceBenchmark, PerformanceMonitor};

let mut benchmark = PerformanceBenchmark::new();
let mut monitor = PerformanceMonitor::new();

// Run comprehensive benchmarks
let results = benchmark.run_all_benchmarks().await?;

// Monitor real-time performance
monitor.record_operation("rule_processing", duration, memory_usage);
let stats = monitor.get_statistics();

println!("Average processing time: {:?}", stats.average_duration);
println!("Peak memory usage: {} MB", stats.peak_memory_usage / 1024 / 1024);
```

## Integration Examples

### 1. CI/CD Pipeline Integration

```yaml
# .github/workflows/sandbag.yml
name: Sandbag Linter Configuration

on:
  pull_request:
    branches: [main]

jobs:
  sandbag:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install Sandbag
        run: cargo install --path .
        
      - name: Run Sandbag Analysis
        run: |
          sandbag scan . --output-format json > linter-analysis.json
          
      - name: Generate Configuration
        run: |
          sandbag config generate --input linter-analysis.json --format json > .sandbag-config.json
          
      - name: Commit Configuration
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add .sandbag-config.json
          git commit -m "Update linter configuration" || exit 0
```

### 2. IDE Integration

```json
// .vscode/settings.json
{
  "sandbag.enabled": true,
  "sandbag.configFile": ".sandbag-config.json",
  "sandbag.autoApply": true,
  "sandbag.linters": ["markdownlint", "eslint", "prettier"],
  "sandbag.performanceMode": true
}
```

### 3. Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run Sandbag analysis
sandbag scan . --output-format json > /tmp/sandbag-analysis.json

# Check if configuration needs updating
if ! cmp -s .sandbag-config.json /tmp/sandbag-analysis.json; then
    echo "Linter configuration needs updating. Please run:"
    echo "sandbag config update --input /tmp/sandbag-analysis.json"
    exit 1
fi

echo "Linter configuration is up to date"
```

## Testing Examples

### 1. Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_extraction() {
        let extractor = RuleExtractor::new();
        let rules = extractor.extract_rules("MD033: No inline HTML").await.unwrap();
        
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_id, "MD033");
        assert!(rules[0].confidence > 0.8);
    }

    #[test]
    fn test_similarity_calculation() {
        let analyzer = AdvancedSimilarityAnalyzer::new();
        let similarity = analyzer.calculate_advanced_similarity("MD033", "MD034");
        
        assert!(similarity.overall_similarity >= 0.0);
        assert!(similarity.overall_similarity <= 1.0);
    }
}
```

### 2. Integration Test Example

```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    let mut suite = IntegrationTestSuite::new();
    let results = suite.run_all_tests().await.unwrap();
    
    assert!(results.is_success());
    assert!(results.success_rate() > 0.8);
    
    println!("{}", results.generate_report());
}
```

### 3. Performance Test Example

```rust
#[tokio::test]
async fn test_performance_benchmarks() {
    let mut suite = PerformanceTestSuite::new();
    let results = suite.run_all_tests().await.unwrap();
    
    // Verify performance requirements
    for test in &results.tests {
        if test.name.contains("processing") {
            let processing_time = test.get_metric("processing_time_ms").unwrap();
            assert!(processing_time < 100.0, "Processing time too slow: {}ms", processing_time);
        }
    }
}
```

## Best Practices

### 1. Configuration Management

- Always backup existing configuration files before modification
- Use version control for configuration files
- Test configuration changes in a development environment first
- Document configuration changes with clear commit messages

### 2. Performance Optimization

- Use caching for frequently accessed data
- Process rules in parallel when possible
- Monitor memory usage for large rule sets
- Profile performance bottlenecks regularly

### 3. Error Handling

- Implement retry logic for transient failures
- Provide clear error messages to users
- Log errors for debugging purposes
- Gracefully handle missing or invalid configuration files

### 4. Testing

- Write comprehensive unit tests for all components
- Include integration tests for end-to-end workflows
- Add performance tests for optimization features
- Maintain high test coverage (>80%)

## Troubleshooting

### Common Issues

1. **Rule Extraction Fails**
   - Check linter output format compatibility
   - Verify regex patterns in linter handlers
   - Ensure input contains valid rule identifiers

2. **Performance Issues**
   - Enable caching for repeated operations
   - Use parallel processing for large datasets
   - Monitor memory usage and optimize accordingly

3. **Configuration Conflicts**
   - Use conflict resolution strategies
   - Validate configuration before applying
   - Backup existing configuration files

### Debug Mode

Enable debug mode for detailed logging:

```bash
RUST_LOG=debug sandbag scan /path/to/project
```

### Performance Profiling

Use built-in profiling tools:

```bash
# Profile rule processing
sandbag benchmark --profile rule-processing

# Monitor memory usage
sandbag benchmark --profile memory-usage

# Generate performance report
sandbag benchmark --report performance-report.json
```
