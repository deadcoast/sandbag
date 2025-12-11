# Examples: Testing

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_calculation() {
        let analyzer = AdvancedSimilarityAnalyzer::new();
        let similarity = analyzer.calculate_advanced_similarity("MD033", "MD034");
        assert!(similarity.overall_similarity >= 0.0);
        assert!(similarity.overall_similarity <= 1.0);
    }
}
```

## Integration Test Example

```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    // ... see tests/integration_tests.rs
}
```

## Performance Test Example

```rust
#[tokio::test]
async fn test_performance_benchmarks() {
    // ... see tests/performance_tests.rs
}
```
