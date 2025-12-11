# Sandbag CLI Reference

> Navigation: [Documentation Hub](00_MOC.md) | [Project Root](../00_MOC.md) | [Cursor Rules](../.cursorrules)

## Overview

Sandbag provides an opinionated CLI for extracting linter rules, analyzing similarity, and managing configuration files. This document describes each command, its purpose, arguments, options, and the practical effects on your project.

## Global Options

- `-v, --verbose`: Enable verbose output (shows extra progress and details)
- `--dry-run`: Show intended actions without making changes
- `-y, --yes`: Skip confirmation prompts (where applicable)
- `-h, --help`: Show help
- `-V, --version`: Show version

## Commands

### add
- **Purpose**: Add a rule to the ignore/disable list, based on a rule ID or raw linter output.
- **Usage**:
  ```bash
  sandbag add <rule_input> [--linter <name>] [--config <path>] [--dry-run] [-v] [-y]
  ```
- **Arguments**:
  - `rule_input` (string): Rule ID or linter output line (e.g., `MD033: No inline HTML`)
- **Options**:
  - `--linter <name>`: Force a specific linter (`markdownlint`, `eslint`, `prettier`)
  - `--config <path>`: Target a specific configuration file (auto-detected if omitted)
- **Effects**:
  - Extracts rules from the input via `LinterRegistry` and suggests actions.
  - Respects `--dry-run` (prints what would be applied without writing).
  - Current implementation prints results; configuration application is being expanded.

### scan
- **Purpose**: Recursively scan a project to extract rule suggestions from logs and text files.
- **Usage**:
  ```bash
  sandbag scan [--path <dir>] [--limit <n>] [--include-low-confidence] [--include-docs] [-v]
  ```
- **Options**:
  - `--path <dir>`: Directory to scan (default: current directory)
  - `--limit <n>`: Show at most N suggestions (default: 10)
  - `--include-low-confidence`: Include suggestions below the medium confidence threshold
  - `--include-docs`: Include `docs/` in the scan (skipped by default)
- **Heuristics and behavior**:
  - Skips hidden directories, `target/`, and `node_modules/`.
  - Skips `docs/` unless `--include-docs` is provided.
  - Reads text-like files (`.md`, `.txt`, `.log`, `.json`, `.yaml`, `.yml`) and log-like filenames (containing `lint`, `problems`, `errors`).
  - Requires linter-like patterns in content (e.g., `MDxxx`, `eslint`, `prettier/`, `: error `, `: warn `, `warning `) before attempting extraction.
  - Extracts rules via `LinterRegistry::extract_rules`, filters by confidence (≥ medium by default), de-duplicates `(linter, rule_id)`, sorts by confidence, and prints the top N.
- **Effects**:
  - Read-only. No modifications are made.

### batch
- **Purpose**: Batch process a file of rule inputs (one per line).
- **Usage**:
  ```bash
  sandbag batch <input_file> [--continue-on-error] [--output <path>] [--dry-run] [-v]
  ```
- **Arguments**:
  - `input_file` (path): File containing rule inputs (one per line)
- **Options**:
  - `--continue-on-error`: Continue processing even if a line fails
  - `--output <path>`: Write results to a file
- **Effects**:
  - Planned: Will extract, suggest, and optionally apply rules in bulk (respects `--dry-run`).

### config
- **Purpose**: Manage configuration backups and information.
- **Usage**:
  ```bash
  sandbag config <subcommand> [...]
  ```
- **Subcommands**:
  - `list-backups <config_file>`: List backups for a configuration file
  - `restore <backup_id> <config_file>`: Restore a configuration backup
  - `cleanup [--max-age-hours <h>] [--config-file <path>]`: Cleanup old backups (default 168h)
  - `info <config_file>`: Show info for a configuration file
- **Effects**:
  - Planned: Safe backup/restore and metadata operations for configuration files.

### list
- **Purpose**: List supported linters.
- **Usage**:
  ```bash
  sandbag list [--detailed]
  ```
- **Options**:
  - `--detailed`: Show additional information (format and config files)
- **Effects**:
  - Read-only. Prints registered linter names and info.

### info
- **Purpose**: Show information about a specific rule ID.
- **Usage**:
  ```bash
  sandbag info <rule_id> [--linter <name>]
  ```
- **Arguments**:
  - `rule_id` (string): Target rule ID (e.g., `MD033`)
- **Options**:
  - `--linter <name>`: Force linter context
- **Effects**:
  - Planned: Will fetch description, category, and guidance for the rule.

### train-bayes
- **Purpose**: Train a Naive Bayes model from labeled data for probabilistic rule extraction.
- **Usage**:
  ```bash
  sandbag train-bayes <input> --model-out <path> [--alpha <f64>] [-v]
  ```
- **Arguments**:
  - `input` (path): JSON Lines file; each line: `{ "rule_id": "...", "text": "..." }`
- **Options**:
  - `--model-out <path>`: Output path for the trained model (JSON)
  - `--alpha <f64>`: Laplace smoothing parameter (default: 1.0)
- **Effects**:
  - Reads labeled examples, builds a Naive Bayes model, and writes it to `--model-out`.

## Examples

- Add a markdownlint rule from Problems output:
  ```bash
  sandbag add "MD033: No inline HTML" --dry-run -v
  ```

- Scan the repository (skip docs by default):
  ```bash
  sandbag scan --limit 5 -v
  ```

- Scan including docs and low-confidence suggestions:
  ```bash
  sandbag scan --include-docs --include-low-confidence --limit 20 -v
  ```

- Train a Bayes model:
  ```bash
  sandbag train-bayes data/labeled.jsonl --model-out models/nb.json --alpha 0.5 -v
  ```

## Notes and Safety

- Use `--dry-run` to preview changes for mutating operations (e.g., `add`, future `batch`).
- Backups and atomic writes are part of the configuration manager design; restore and cleanup are exposed under `config` subcommands.
- Some subcommands are read-only today and will gain write capabilities in future iterations.
