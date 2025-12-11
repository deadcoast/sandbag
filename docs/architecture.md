# Sandbag CLI - Comprehensive Design Document

> **Navigation**:
[Documentation Hub](00_MOC.md) | [Project Root](../00_MOC.md) | [Cursor Rules](../.cursorrules)

## Executive Summary

Sandbag is an intelligent CLI tool that bridges the gap between linter error
identification and configuration management.
It parses linter output, extracts rule identifiers, and automatically updates
configuration files using mathematical similarity analysis and AST parsing for
maximum accuracy and reliability.

## Problem Definition

The current linter ecosystem presents significant barriers to developer productivity:

- **Configuration Complexity**:
- Each linter has unique configuration syntax and rule identification patterns
- **Context Switching Overhead**:
- Developers must manually research rule formats, locate config files,
  and understand syntax variations
- **Entry Barrier for New Developers**:
- The cognitive load of managing linter configurations compounds the
  already steep learning curve
- **Inconsistent Rule Identification**:
- Rule IDs appear in different formats across different
  contexts (IDE problems panel vs CLI output)

## Core Architecture

### Mathematical Foundation

Sandbag employs mathematical analysis for rule identification and configuration management:

**Similarity Scoring Algorithm**:

```text
similarity_score = (exact_match_weight * exact_matches + 
                   pattern_match_weight * pattern_matches + 
                   context_weight * context_matches) / total_possible_matches
```

**Confidence Threshold Calculation**:

- High confidence: ≥ 0.85 (automatic application)
- Medium confidence: 0.60-0.84 (user confirmation)
- Low confidence: < 0.60 (manual intervention required)

### AST-Based Configuration Management

The system uses Abstract Syntax Tree parsing to understand and modify
configuration files:

1. **Parse Existing Configuration**: Build AST representation of current config
2. **Identify Insertion Points**: Locate appropriate sections for rule modifications
3. **Maintain Formatting**: Preserve original file structure and comments
4. **Validate Changes**: Ensure syntactic correctness post-modification

## System Components

### 1. Rule Extraction Engine

**Input Processing**:

- Accepts multiple input formats: raw linter output, rule IDs, file paths
- Normalizes input through regex pattern matching
- Extracts metadata: severity, rule category, description

**Pattern Recognition**:

- Maintains pattern databases for supported linters
- Uses fuzzy matching algorithms for variant detection
- Learns from user corrections to improve accuracy

### 2. Linter Intelligence System

**Supported Linter Registry**:

```text
LinterConfig {
    name: string
    config_files: string[]
    rule_patterns: RegexPattern[]
    config_syntax: ConfigSyntax
    rule_categories: RuleCategory[]
}
```

**Dynamic Linter Detection**:

- Scans project directory for linter configuration files
- Analyzes package.json dependencies
- Checks IDE configuration files

### 3. Configuration Management Engine

**Multi-Format Support**:

- JSON (ESLint, Prettier)
- YAML (markdownlint, stylelint)
- TOML (Ruff, Black)
- INI/Custom formats

**Atomic Operations**:

- Transaction-based file modifications
- Rollback capability on failure
- Backup creation before changes

### 4. User Interface Layer

**Interactive CLI Design**:

- Rich terminal UI with color coding
- Progress indicators for long operations
- Contextual help and suggestions
- Error recovery mechanisms

**Command Structure**:

```bash
sandbag add <rule_id>           # Add single rule
sandbag batch <input_file>      # Process multiple rules
sandbag scan                    # Auto-detect and suggest rules
sandbag config <linter>         # Manage linter-specific settings
sandbag restore <backup_id>     # Rollback changes
```

## Implementation Strategy

### Phase 1: Core Engine

- Rule extraction and parsing system
- Basic markdownlint support
- File system operations and backup management

### Phase 2: Intelligence Layer

- Mathematical similarity algorithms
- AST-based configuration parsing
- Pattern recognition improvements

### Phase 3: User Experience

- Rich CLI interface implementation
- Interactive confirmation system
- Error handling and recovery

### Phase 4: Extension & Polish

- Additional linter support
- Performance optimization
- Comprehensive testing suite

## Technical Specifications

### Data Structures

**Rule Definition**:

```rust
struct Rule {
    id: String,
    linter: String,
    severity: Severity,
    description: Option<String>,
    category: RuleCategory,
    confidence: f64,
}
```

**Configuration Entry**:

```rust
struct ConfigEntry {
    rule_id: String,
    action: ConfigAction, // disable, ignore, modify
    scope: ConfigScope,   // global, file-specific, inline
    metadata: HashMap<String, Value>,
}
```

### Algorithm Details

**Rule Matching Process**:

1. **Tokenization**: Break input into meaningful components
2. **Pattern Matching**: Apply linter-specific regex patterns
3. **Semantic Analysis**: Understand rule context and relationships
4. **Confidence Scoring**: Calculate match probability
5. **Validation**: Verify rule exists in linter documentation

**Configuration Writing Strategy**:

1. **Schema Validation**: Ensure proposed changes are valid
2. **Conflict Resolution**: Handle existing rule conflicts
3. **Formatting Preservation**: Maintain code style
4. **Atomic Application**: All-or-nothing change application

## Quality Assurance

### Testing Framework

- Unit tests for each component
- Integration tests for end-to-end workflows
- Fuzzing for input validation
- Performance benchmarks

### Error Handling

- Graceful degradation on parsing failures
- Detailed error messages with suggested fixes
- Automatic recovery mechanisms
- Comprehensive logging system

### Validation Systems

- Configuration syntax validation
- Rule existence verification
- Circular dependency detection
- Performance impact assessment

## Extensibility Design

### Plugin Architecture

- Modular linter support system
- Custom parser registration
- Rule transformation pipelines
- Configuration template system

### API Design

- Clean separation of concerns
- Well-defined interfaces
- Backwards compatibility guarantees
- Extension point documentation

## Success Metrics

### Quantitative Goals

- Rule identification accuracy > 95%
- Configuration modification success rate > 99%
- Average operation completion time < 2 seconds
- Memory usage < 50MB for typical operations

### Qualitative Objectives

- Intuitive user experience requiring minimal learning
- Comprehensive error messages and guidance
- Seamless integration with existing workflows
- Reliable operation across different environments

## Risk Mitigation

### Technical Risks

- **Configuration Corruption**: Atomic operations with rollback
- **Rule Conflicts**: Intelligent conflict resolution
- **Performance Issues**: Lazy loading and caching strategies
- **Compatibility Problems**: Extensive testing matrix

### User Experience Risks

- **Learning Curve**: Progressive disclosure of features
- **Error Recovery**: Clear guidance and automatic fixes
- **Trust Issues**: Transparent operations with confirmation steps

## Future Enhancements

### Advanced Features

- Machine learning for pattern recognition improvement
- IDE integration plugins
- Team configuration synchronization
- Rule recommendation system based on project analysis

### Ecosystem Integration

- CI/CD pipeline integration
- Git hooks for automatic rule management
- Package manager integration
- Cloud-based configuration sharing

---

## Related Documentation

### Cross-References

- **[Implementation Guide](implementation.md)**
  - Technical implementation details and code specifications
- **[Mathematical Foundation](mathematics.md)**
  - Advanced algorithms and theoretical foundations
- **[Development Notes](dev-notes.md)**
  - Human context and project motivation
- **[Documentation Hub](00_MOC.md)**
  - Complete documentation index
- **[Project Root](../00_MOC.md)**
  - Project overview and status

### Reading Paths

- **For Implementation**: This document → [Implementation Guide](implementation.md)
- **For Advanced Algorithms**: This document → [Mathematical Foundation](mathematics.md)
- **For Context**: [Development Notes](dev-notes.md) → This document
- **For Overview**: [Project Root](../00_MOC.md) → This document

---
