# Sandbag - Intelligent Linter Configuration Management

> Navigation: [Documentation Hub](00_MOC.md) | [Project Root](../00_MOC.md) | [Cursor Rules](../.cursorrules)

Sandbag is a powerful Rust-based tool that provides mathematical precision for
managing linter configurations through intelligent rule extraction, similarity
analysis, and AST-based configuration management.

## Features

### **Core Functionality**

- **Rule Extraction**: Automatically extract and parse linter rules from
  various output formats
- **Similarity Analysis**: Advanced mathematical algorithms for rule similarity calculation
- **Configuration Management**: AST-based configuration parsing and manipulation
- **Multi-Linter Support**: Built-in support for Markdownlint, ESLint, and Prettier

### **Performance Optimizations**

- **Caching System**: Multi-level caching with TTL and LRU eviction strategies
- **Parallel Processing**: Rayon-based parallel rule processing
- **Memory Optimization**: Efficient memory usage for large rule sets
- **Performance Monitoring**: Real-time performance metrics and benchmarking

### **User Experience**

- **Rich CLI Interface**: Interactive command-line interface with progress indicators
- **Color-Coded Output**: Visual feedback with confidence bars and status indicators
- **Error Recovery**: Intelligent error handling with retry mechanisms
- **Help System**: Comprehensive interactive help and documentation

### **Developer Tools**

- **CI/CD Pipeline**: Automated testing, building, and deployment
- **Comprehensive Testing**: Unit, integration, and performance test suites
- **Benchmarking**: Performance benchmarking and optimization tools
- **Documentation**: Extensive documentation and examples

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/sandbag.git
cd sandbag

# Build the project
cargo build --release

# Run the application
./target/release/sandbag --help
```

### Basic Usage

```bash
# Scan for linter rules in a directory
sandbag scan /path/to/project

# Add a specific rule to configuration
sandbag add MD033 --disable

# Generate configuration file
sandbag config generate --format json

# Run performance benchmarks
sandbag benchmark --all
```

## Architecture

### Core Components

#### 1. Rule Extraction Engine

- **Location**: `src/core/rule_extractor.rs`
- **Purpose**: Extract and parse linter rules from various output formats
- **Features**: Confidence scoring, context extraction, pattern matching

#### 2. Similarity Analysis

- **Location**: `src/core/similarity.rs`, `src/core/advanced_similarity.rs`
- **Purpose**: Calculate similarity between rules using mathematical algorithms
- **Features**: Fractal analysis, spectral analysis, confidence weighting

#### 3. Configuration Management

- **Location**: `src/config/`
- **Purpose**: Parse, validate, and manipulate configuration files
- **Features**: AST-based parsing, conflict resolution, backup management

#### 4. Linter Support

- **Location**: `src/linters/`
- **Purpose**: Handle different linter output formats
- **Supported**: Markdownlint, ESLint, Prettier (extensible)

#### 5. Performance Optimization

- **Location**: `src/performance/`
- **Purpose**: Optimize performance through caching and parallel processing
- **Features**: Multi-level caching, parallel processing, performance monitoring

#### 6. User Interface

- **Location**: `src/ui/`
- **Purpose**: Provide rich command-line interface
- **Features**: Interactive prompts, progress indicators, help system

## Configuration

### Supported Formats

Sandbag supports multiple configuration formats:

#### JSON Configuration

```json
{
  "rules": {
    "MD033": {
      "action": "disable",
      "scope": "global",
      "metadata": {
        "disabled_by_sandbag": true,
        "timestamp": "2024-01-01T00:00:00Z"
      }
    }
  }
}
```

#### YAML Configuration

```yaml
rules:
  MD033:
    action: disable
    scope: global
    metadata:
      disabled_by_sandbag: true
      timestamp: "2024-01-01T00:00:00Z"
```

### Rule Actions

- **`disable`**: Completely disable the rule
- **`ignore`**: Ignore the rule in specific contexts
- **`modify`**: Modify the rule's behavior
- **`enable`**: Explicitly enable the rule

### Rule Scopes

- **`global`**: Apply to entire project
- **`file_specific`**: Apply to specific files
- **`inline`**: Apply to specific lines
- **`directory`**: Apply to specific directories

## API Reference

### Core Types

#### RuleMatch

```rust
pub struct RuleMatch {
    pub rule_id: String,
    pub linter: String,
    pub confidence: f64,
    pub context: ExtractedContext,
    pub suggested_action: ConfigAction,
}
```

#### ConfigEntry

```rust
pub struct ConfigEntry {
    pub rule_id: String,
    pub action: ConfigAction,
    pub scope: ConfigScope,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### Main Functions

#### Rule Extraction

```rust
use sandbag::core::rule_extractor::RuleExtractor;

let extractor = RuleExtractor::new();
let rules = extractor.extract_rules("MD033: No inline HTML").await?;
```

#### Similarity Analysis

```rust
use sandbag::core::advanced_similarity::AdvancedSimilarityAnalyzer;

let analyzer = AdvancedSimilarityAnalyzer::new();
let similarity = analyzer.calculate_advanced_similarity("MD033", "MD034");
```

#### Configuration Management

```rust
use sandbag::config::manager::ConfigManager;

let manager = ConfigManager::new();
let ast = manager.parse_config_file("config.json")?;
let json = manager.serialize_config(&ast, ConfigFormat::Json)?;
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run performance tests
cargo test --test performance_tests

# Run benchmarks
cargo bench
```

### Test Coverage

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end workflow testing
- **Performance Tests**: Benchmarking and optimization validation
- **Benchmarks**: Performance measurement and comparison

## Performance

### Benchmarks

Sandbag includes comprehensive performance benchmarks:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench rule_extraction_bench
```

### Optimization Features

- **Caching**: Multi-level caching with configurable TTL
- **Parallel Processing**: Rayon-based parallel rule processing
- **Memory Management**: Efficient memory usage and cleanup
- **Performance Monitoring**: Real-time metrics and profiling

## Contributing

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/sandbag.git
cd sandbag

# Install dependencies
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Code Style

- Follow Rust coding conventions
- Use meaningful variable and function names
- Add comprehensive documentation
- Include unit tests for new features

### Testing Guidelines

- Write unit tests for all new functionality
- Ensure integration tests cover end-to-end workflows
- Add performance tests for optimization features
- Maintain test coverage above 80%

## License

This project is licensed under the MIT License - see:
[LICENSE](../LICENSE) file for details.

## Acknowledgments

- Built with Rust for performance and safety
- Uses advanced mathematical algorithms for similarity analysis
- Implements modern CLI design patterns
- Follows software engineering best practices

## Support

For support and questions:

- **Issues**: [GitHub Issues](https://github.com/yourusername/sandbag/issues)
- **Documentation**: [Project Wiki](https://github.com/yourusername/sandbag/wiki)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/sandbag/discussions)

## Roadmap

### Upcoming Features

- **Additional Linters**: Support for more linter types
- **Web Interface**: Web-based configuration management
- **Plugin System**: Extensible plugin architecture
- **Cloud Integration**: Cloud-based configuration sync
- **Advanced Analytics**: Detailed usage analytics and insights

### Performance Goals

- **Rule Processing**: < 100ms for 1000 rules
- **Similarity Calculation**: < 50ms for rule comparison
- **Memory Usage**: < 100MB for large projects
- **Cache Hit Rate**: > 80% for typical usage patterns
