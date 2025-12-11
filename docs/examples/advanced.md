# Examples: Advanced

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## Performance Optimization

```rust
use sandbag::performance::optimizations::OptimizedRuleProcessor;

let mut processor = OptimizedRuleProcessor::new();
let rules = vec![/* your rules */];
let config_entries = processor.process_rules_parallel(rules);
let similarity = processor.calculate_similarity_cached("rule1", "rule2");
let metrics = processor.get_metrics();
```

## Error Recovery

```rust
use sandbag::ui::error_recovery::ErrorRecoveryManager;

let mut recovery = ErrorRecoveryManager::new();
recovery.set_max_retries(3);
recovery.set_interactive_mode(true);
```
