# Examples: Real-World Scenarios

> Navigation: [Documentation Hub](../00_MOC.md) | [Project Root](../../00_MOC.md) | [Cursor Rules](../../.cursorrules)

## Migrating from ESLint to Prettier

```rust
use sandbag::linters::{LinterRegistry, LinterFactory};

let registry = LinterRegistry::new();
let eslint_output = "1:10 error no-unused-vars 'x' is assigned a value but never used";
let eslint_rules = registry.extract_rules(eslint_output);
for rule in eslint_rules {
    let prettier_handler = LinterFactory::create_handler("prettier").unwrap();
    let equivalent_rules = prettier_handler.find_equivalent_rules(&rule.rule_id);
}
```

## Bulk Rule Configuration

```rust
use sandbag::core::rule_extractor::RuleExtractor;
use sandbag::config::manager::ConfigManager;

let extractor = RuleExtractor::new();
let mut manager = ConfigManager::new();
let all_rules = extractor.extract_rules_from_directory("/path/to/project").await?;
// group_rules_by_similarity(&all_rules);
```
