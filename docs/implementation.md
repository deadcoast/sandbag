# Sandbag Implementation Roadmap

> **Navigation**:
> [Documentation Hub](00_MOC.md) | [Project Root](../00_MOC.md) | [Cursor Rules](../.cursorrules)

## Technology Stack Selection

**Core Language**: Rust

- Memory safety and performance
- Excellent CLI ecosystem (clap, tokio)
- Strong pattern matching capabilities
- Cross-platform compilation

**Key Dependencies**:

- `clap` - Command line parsing with rich help generation
- `serde` - Serialization for configuration file handling
- `regex` - Pattern matching and rule extraction
- `tree-sitter` - AST parsing for configuration files
- `tokio` - Async runtime for file operations
- `colored` - Terminal output formatting
- `dialoguer` - Interactive CLI prompts

## Project Structure

```text
sandbag/
 src/
    main.rs                 # CLI entry point
    lib.rs                  # Library exports
    core/
       mod.rs
       rule_extractor.rs   # Rule parsing logic
       similarity.rs       # Mathematical matching
       confidence.rs       # Scoring algorithms
    linters/
       mod.rs
       registry.rs         # Linter definitions
       markdownlint.rs     # Markdownlint implementation
       base.rs             # Common linter traits
    config/
       mod.rs
       parser.rs           # AST-based parsing
       writer.rs           # Safe configuration writing
       backup.rs           # Backup management
    ui/
       mod.rs
       cli.rs              # Command definitions
       interactive.rs      # User interaction
       display.rs          # Output formatting
    utils/
        mod.rs
        fs.rs               # File system operations
        validation.rs       # Input validation
 tests/
    integration/
    fixtures/
    unit/
 docs/
    design.md
    api.md
    examples/
 assets/
     linter_configs/
```

## Core Implementation Details

### Rule Extraction Engine

```rust
#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_id: String,
    pub linter: String,
    pub confidence: f64,
    pub context: ExtractedContext,
    pub suggested_action: ConfigAction,
}

impl RuleExtractor {
    pub fn extract_rules(&self, input: &str) -> Vec<RuleMatch> {
        let patterns = self.get_patterns_for_input(input);
        let mut matches = Vec::new();

        for pattern in patterns {
            if let Some(rule_match) = self.apply_pattern(pattern, input) {
                let confidence = self.calculate_confidence(&rule_match);
                if confidence >= MINIMUM_CONFIDENCE_THRESHOLD {
                    matches.push(RuleMatch {
                        rule_id: rule_match.rule_id,
                        linter: pattern.linter.clone(),
                        confidence,
                        context: rule_match.context,
                        suggested_action: self.determine_action(&rule_match),
                    });
                }
            }
        }

        self.rank_by_confidence(matches)
    }

    fn calculate_confidence(&self, rule_match: &RuleMatch) -> f64 {
        let exact_weight = 0.6;
        let pattern_weight = 0.3;
        let context_weight = 0.1;

        let exact_score = if rule_match.has_exact_id_match() { 1.0 } else { 0.0 };
        let pattern_score = self.calculate_pattern_similarity(rule_match);
        let context_score = self.calculate_context_relevance(rule_match);

        exact_weight * exact_score +
        pattern_weight * pattern_score +
        context_weight * context_score
    }
}
```

### Mathematical Similarity Analysis

```rust
pub struct SimilarityAnalyzer {
    algorithms: Vec<Box<dyn SimilarityAlgorithm>>,
}

trait SimilarityAlgorithm {
    fn calculate(&self, a: &str, b: &str) -> f64;
    fn weight(&self) -> f64;
}

struct LevenshteinSimilarity;
impl SimilarityAlgorithm for LevenshteinSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        let distance = levenshtein_distance(a, b);
        let max_len = a.len().max(b.len()) as f64;
        if max_len == 0.0 { 1.0 } else { 1.0 - (distance as f64 / max_len) }
    }

    fn weight(&self) -> f64 { 0.4 }
}

struct JaccardSimilarity;
impl SimilarityAlgorithm for JaccardSimilarity {
    fn calculate(&self, a: &str, b: &str) -> f64 {
        let set_a: HashSet<char> = a.chars().collect();
        let set_b: HashSet<char> = b.chars().collect();

        let intersection = set_a.intersection(&set_b).count() as f64;
        let union = set_a.union(&set_b).count() as f64;

        if union == 0.0 { 1.0 } else { intersection / union }
    }

    fn weight(&self) -> f64 { 0.3 }
}

impl SimilarityAnalyzer {
    pub fn weighted_similarity(&self, a: &str, b: &str) -> f64 {
        let total_weight: f64 = self.algorithms.iter().map(|alg| alg.weight()).sum();
        let weighted_sum: f64 = self.algorithms.iter()
            .map(|alg| alg.calculate(a, b) * alg.weight())
            .sum();

        weighted_sum / total_weight
    }
}
```

### AST-Based Configuration Management

```rust
pub struct ConfigManager {
    parsers: HashMap<String, Box<dyn ConfigParser>>,
    backup_manager: BackupManager,
}

trait ConfigParser {
    fn parse(&self, content: &str) -> Result<ConfigAST, ParseError>;
    fn serialize(&self, ast: &ConfigAST) -> Result<String, SerializeError>;
    fn insert_rule(&self, ast: &mut ConfigAST, rule: &ConfigEntry) -> Result<
    (), ModifyError>;
}

#[derive(Debug)]
pub struct ConfigAST {
    pub format: ConfigFormat,
    pub root: ASTNode,
    pub metadata: ConfigMetadata,
}

impl ConfigManager {
    pub async fn apply_rule_safely(&mut self, rule: &ConfigEntry, file_path: &
    Path) -> Result<(), ApplyError> {
        // Create backup
        let backup_id = self.backup_manager.create_backup(file_path).await?;

        // Read and parse current config
        let content = tokio::fs::read_to_string(file_path).await?;
        let parser = self.get_parser_for_file(file_path)?;
        let mut ast = parser.parse(&content)?;

        // Apply modification
        match parser.insert_rule(&mut ast, rule) {
            Ok(_) => {
                let new_content = parser.serialize(&ast)?;

                // Validate before writing
                if self.validate_config(&new_content, parser.as_ref())? {
                    tokio::fs::write(file_path, new_content).await?;
                    Ok(())
                } else {
                    self.backup_manager.restore_backup(&backup_id).await?;
                    Err(ApplyError::ValidationFailed)
                }
            }
            Err(e) => {
                self.backup_manager.restore_backup(&backup_id).await?;
                Err(ApplyError::ModificationFailed(e))
            }
        }
    }
}
```

### Markdownlint Specific Implementation

```rust
pub struct MarkdownlintHandler {
    config_patterns: Vec<Regex>,
    rule_database: RuleDatabase,
}

impl LinterHandler for MarkdownlintHandler {
    fn name(&self) -> &str { "markdownlint" }

    fn extract_rule_id(&self, input: &str) -> Option<RuleMatch> {
        // Pattern: "MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033"
        let primary_pattern = Regex::new(r"(MD\d{3})(?:/[\w-]+)?:.*?markdownlint(?:MD\d{3})?").unwrap();

        if let Some(captures) = primary_pattern.captures(input) {
            let rule_id = captures.get(1)?.as_str().to_string();

            Some(RuleMatch {
                rule_id: rule_id.clone(),
                linter: "markdownlint".to_string(),
                confidence: self.calculate_extraction_confidence(input, &rule_id),
                context: self.extract_context(input),
                suggested_action: ConfigAction::Disable,
            })
        } else {
            None
        }
    }

    fn get_config_files(&self) -> Vec<&str> {
        vec![".markdownlint.json", ".markdownlint.jsonc", ".markdownlint.yaml", ".markdownlint.yml"]
    }

    fn create_config_entry(&self, rule_id: &str) -> ConfigEntry {
        ConfigEntry {
            rule_id: rule_id.to_string(),
            action: ConfigAction::Disable,
            scope: ConfigScope::Global,
            metadata: {
                let mut map = HashMap::new();
                map.insert("disabled_by_sandbag".to_string(), Value::Bool(true));
                map.insert("timestamp".to_string(), Value::String(Utc::now().to_rfc3339()));
                map
            },
        }
    }
}

struct MarkdownlintConfigParser;

impl ConfigParser for MarkdownlintConfigParser {
    fn parse(&self, content: &str) -> Result<ConfigAST, ParseError> {
        // Handle both JSON and YAML formats
        if content.trim_start().starts_with('{') {
            self.parse_json(content)
        } else {
            self.parse_yaml(content)
        }
    }

    fn insert_rule(&self, ast: &mut ConfigAST, rule: &ConfigEntry) -> Result<
    (), ModifyError> {
        match ast.format {
            ConfigFormat::Json => self.insert_json_rule(ast, rule),
            ConfigFormat::Yaml => self.insert_yaml_rule(ast, rule),
            _ => Err(ModifyError::UnsupportedFormat),
        }
    }
}
```

### CLI Interface Implementation

```rust
#[derive(Parser)]
#[command(name = "sandbag")]
#[command(about = "Intelligent linter configuration management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a rule to ignore list
    Add {
        /// Rule ID or linter output to process
        rule_input: String,

        /// Force specific linter (skip auto-detection)
        #[arg(short, long)]
        linter: Option<String>,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// Scan project for suggested rule suppressions
    Scan {
        /// Directory to scan (defaults to current)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Maximum number of suggestions
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Batch process rules from file
    Batch {
        /// Input file containing rules (one per line)
        input_file: PathBuf,

        /// Continue on errors
        #[arg(short, long)]
        continue_on_error: bool,
    },

    /// Manage configuration and backups
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut sandbag = SandbagCore::new().await?;

    match cli.command {
        Commands::Add { rule_input, linter, yes } => {
            let matches = sandbag.extract_rules(&rule_input).await?;

            if matches.is_empty() {
                println!("{}", "No valid rules found in input".red());
                return Ok(());
            }

            for rule_match in matches {
                if rule_match.confidence < HIGH_CONFIDENCE_THRESHOLD && !yes {
                    let confirmed = Confirm::new()
                        .with_prompt(format!(
                            "Apply rule {} with {}% confidence?",
                            rule_match.rule_id.green(),
                            (rule_match.confidence * 100.0) as u8
                        ))
                        .default(rule_match.confidence > MEDIUM_CONFIDENCE_THRESHOLD)
                        .interact()?;

                    if !confirmed { continue; }
                }

                match sandbag.apply_rule(&rule_match).await {
                    Ok(_) => println!(
                        "{} Successfully applied rule {}",
                        "".green(),
                        rule_match.rule_id.bold()
                    ),
                    Err(e) => println!(
                        "{} Failed to apply rule {}: {}",
                        "".red(),
                        rule_match.rule_id.bold(),
                        e
                    ),
                }
            }
        }
        // ... other commands
    }

    Ok(())
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdownlint_rule_extraction() {
        let extractor = MarkdownlintHandler::new();
        let input = "MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033";

        let result = extractor.extract_rule_id(input).unwrap();
        assert_eq!(result.rule_id, "MD033");
        assert_eq!(result.linter, "markdownlint");
        assert!(result.confidence > 0.9);
    }

    #[tokio::test]
    async fn test_config_modification_safety() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".markdownlint.json");

        tokio::fs::write(&config_path, r#"{"MD001": false}"#).await.unwrap();

        let mut manager = ConfigManager::new();
        let rule = ConfigEntry::new("MD033", ConfigAction::Disable);

        manager.apply_rule_safely(&rule, &config_path).await.unwrap();

        let content = tokio::fs::read_to_string(&config_path).await.unwrap();
        let config: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(config["MD033"], false);
        assert_eq!(config["MD001"], false); // Original rule preserved
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    let temp_project = setup_test_project().await;
    let sandbag = SandbagCore::new().await.unwrap();

    // Simulate linter output
    let linter_output = "MD033/no-inline-html: Inline HTML [Element: div]markdownlintMD033";

    // Extract rules
    let matches = sandbag.extract_rules(linter_output).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].rule_id, "MD033");

    // Apply rule
    sandbag.apply_rule(&matches[0]).await.unwrap();

    // Verify configuration was updated
    let config_content = tokio::fs::read_to_string(
        temp_project.path().join(".markdownlint.json")
    ).await.unwrap();

    let config: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert_eq!(config["MD033"], false);
}
```

## Performance Considerations

### Memory Management

- Use `String` ownership judiciously, prefer `&str` where possible
- Implement streaming parsing for large configuration files
- Cache compiled regex patterns
- Use memory pools for frequently allocated objects

### Async Operations

- Use `tokio::fs` for all file operations
- Implement timeout controls for network operations
- Use bounded channels for inter-task communication
- Batch multiple configuration changes when possible

### Optimization Targets

- Rule extraction: < 10ms for typical input
- Configuration modification: < 100ms including backup
- Memory usage: < 20MB baseline, < 50MB under load
- Binary size: < 15MB after optimization

This implementation roadmap provides the detailed technical foundation needed
for AI agents to build Sandbag while maintaining the sophisticated architecture
you envisioned. The mathematical approach to rule matching and AST-based
configuration management ensures both accuracy and reliability.

---

## Related Documentation

### Cross-References

- **[System Architecture](architecture.md)** - Design specifications and
  architectural decisions
- **[Mathematical Foundation](mathematics.md)** - Advanced algorithms and
  theoretical foundations
- **[Development Notes](dev-notes.md)** - Human context and project motivation
- **[Documentation Hub](00_MOC.md)** - Complete documentation index
- **[Project Root](../00_MOC.md)** - Project overview and status

### Reading Paths

- **For Design Context**: [System Architecture](architecture.md) → This document
- **For Advanced Algorithms**: [Mathematical Foundation](mathematics.md) → This document
- **For Implementation Context**: [Development Notes](dev-notes.md) → This document
- **For Overview**: [Project Root](../00_MOC.md) → This document

---
