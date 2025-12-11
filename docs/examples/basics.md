# Examples: Basics

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## Rule Extraction

```rust
use sandbag::linters::LinterRegistry;

let registry = LinterRegistry::new();
let markdownlint_output = "MD033: No inline HTML\nMD013: Line too long";
let rules = registry.extract_rules(markdownlint_output);
for rule in rules {
    println!("Rule: {}, Confidence: {:.2}", rule.rule_id, rule.confidence);
}
```

## Similarity Analysis

```rust
use sandbag::core::advanced_similarity::AdvancedSimilarityAnalyzer;

let analyzer = AdvancedSimilarityAnalyzer::new();
let similarity = analyzer.calculate_advanced_similarity("MD033", "MD034");
println!("Similarity: {:.2}", similarity.overall_similarity);
```

## Configuration Management

```rust
use sandbag::config::{ConfigAST, ConfigFormat, ConfigManager};
use sandbag::core::{ConfigAction, ConfigScope};
use std::collections::HashMap;

let mut manager = ConfigManager::new();
let entries = vec![
    ConfigEntry { rule_id: "MD033".to_string(), action: ConfigAction::Disable, scope: ConfigScope::Global, metadata: HashMap::new() },
    ConfigEntry { rule_id: "MD013".to_string(), action: ConfigAction::Ignore, scope: ConfigScope::FileSpecific, metadata: HashMap::new() },
];
let ast = ConfigAST::new(entries);
let json_config = manager.serialize_config(&ast, ConfigFormat::Json)?;
let yaml_config = manager.serialize_config(&ast, ConfigFormat::Yaml)?;
```
