# Sandbag - The .sbag Extension

An integrated solution to silencing linters

Sandbag eliminates the friction between linter errors
 and configuration management through mathematical intelligence
 and automated rule processing.

## The Problem

You're coding. Your linter throws an error.
You know the code is correct, but the linter disagrees. Now you have to:

1. Research the specific rule format for your linter
2. Find the configuration file (if it exists)
3. Figure out the exact syntax to disable this rule
4. Hope you don't break anything else

For new developers especially, this interrupts flow and creates unnecessary barriers.
For experienced developers, it's repetitive busywork that shouldn't exist.

## The Solution

```bash
# Copy the rule ID from your IDE problems panel
sandbag add MD033

# That's it. Rule added to your linter config automatically.
```

Sandbag understands linter languages, parses rule patterns mathematically,
and updates configuration files safely and intelligently.

## Key Features

- **Intelligent Rule Extraction**
  - Parses linter output using probabilistic pattern matching
- **Mathematical Confidence Scoring**
  - Uses Bayesian inference to determine rule accuracy
- **Safe Configuration Updates**
  - AST-based parsing preserves file structure and comments
- **Automatic Backup Management**
  - Every change is reversible
- **Multi-Linter Support**
  - Extensible architecture for any linter ecosystem
- **Interactive CLI**
  - Beautiful terminal interface with smart defaults

## Quick Start

### Installation

```bash
# Install via cargo
cargo install sandbag

# Or download binary from releases
curl -L https://github.com/deadcoast/sandbag/releases/latest/download/sandbag 
-o sandbag
chmod +x sandbag
```

### Basic Usage

```bash
# Add a rule from linter output
sandbag add "MD033/no-inline-html: Inline HTML [Element: div]"

# Add rule by ID directly  
sandbag add MD033

# Scan project for suggested suppressions
sandbag scan

# Process multiple rules from file
sandbag batch rules.txt

# Manage backups and configuration
sandbag config restore latest
```

## Supported Linters

**Currently Supported:**

- markdownlint (JSON, YAML configurations)

**Planned Support:**

- ESLint
- Prettier
- Pylint
- Ruff
- Clippy
- And many more...

## Example Workflow

1. Your IDE shows: `MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033`

2. Copy the rule identifier: `MD033`

3. Run: `sandbag add MD033`

4. Sandbag automatically:
   - Detects you're using markdownlint
   - Finds your `.markdownlint.json` config
   - Adds `"MD033": false` safely
   - Creates a backup for rollback

## Why Sandbag?

**For New Developers:**

- No need to learn linter configuration syntax
- Focus on coding, not tooling configuration
- Intelligent guidance and error recovery

**For Experienced Developers:**

- Eliminate repetitive configuration tasks
- Mathematical accuracy prevents configuration errors
- Extensible for custom workflows

**For Teams:**

- Consistent linter configuration across projects
- Reduced onboarding friction
- Automated configuration management

## Documentation

- [Architecture](/docs/architecture.md) - Mathematical foundations and system design
- [Implementation](/docs/implementation.md) - Technical implementation details
- [Dev-Notes](docs/dev-notes.md) - Notes on Developmennt Process
- [Mathematics](/docs/mathematics.md) - Theoretical foundations and algorithms
- [Contributing](/CONTRIBUTING.md) - Development guidelines and contribution process

## Philosophy

Sandbag represents a different approach to developer tooling.
Instead of optimizing for minimal implementation, we use mathematical rigor
and innovative algorithms to create genuinely better experiences.

This project anticipates the incoming wave of new developers who
deserve tools that understand context, learn patterns,
and make intelligent decisions without requiring deep domain expertise.

## Contributing

We welcome contributions that push boundaries and explore new mathematical
approaches to configuration management.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](./LICENSE) for details.

---

*Built with mathematical precision for the next generation of developers.*
