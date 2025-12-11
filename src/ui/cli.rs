//! CLI interface implementation with command structure

use crate::core::bayesian::evidence_from_input;
use crate::core::bayesian::{LabeledObservation, NaiveBayesModel};
use crate::linters::LinterHandler;
use crate::Sandbag;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::path::PathBuf;

use crate::config::manager::ConfigManager;
use crate::core::ConfigEntry;

/// Main CLI application
#[derive(Parser)]
#[command(name = "sandbag")]
#[command(about = "Intelligent linter configuration management")]
#[command(version)]
#[command(propagate_version = true)]
pub struct SandbagCli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Show what would be done without making changes
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Skip confirmation prompts
    #[arg(short, long, global = true)]
    pub yes: bool,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Add a rule to ignore list
    Add {
        /// Rule ID or linter output to process
        rule_input: String,

        /// Force specific linter (skip auto-detection)
        #[arg(short, long)]
        linter: Option<String>,

        /// Configuration file path (auto-detected if not specified)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Scan project for suggested rule suppressions
    Scan {
        /// Directory to scan (defaults to current)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Maximum number of suggestions
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Include low confidence suggestions
        #[arg(long)]
        include_low_confidence: bool,

        /// Include documentation directories (docs/) in scan
        #[arg(long)]
        include_docs: bool,
    },

    /// Batch process rules from file
    Batch {
        /// Input file containing rules (one per line)
        input_file: PathBuf,

        /// Continue on errors
        #[arg(short, long)]
        continue_on_error: bool,

        /// Output file for results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Manage configuration and backups
    Config {
        /// Configuration subcommand to execute
        #[command(subcommand)]
        config_command: ConfigCommands,
    },

    /// List supported linters
    List {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show information about a specific rule
    Info {
        /// Rule ID to get information about
        rule_id: String,

        /// Linter name (auto-detected if not specified)
        #[arg(short, long)]
        linter: Option<String>,
    },

    /// Train Bayesian model from labeled data
    TrainBayes {
        /// Path to labeled input file (JSON lines: {"`rule_id`":..., "text":...})
        input: PathBuf,
        /// Output model path (JSON)
        #[arg(short, long)]
        model_out: PathBuf,
        /// Laplace smoothing parameter alpha
        #[arg(long, default_value = "1.0")]
        alpha: f64,
    },
}

/// Configuration management commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// List available backups
    ListBackups {
        /// Configuration file path
        config_file: PathBuf,
    },

    /// Restore from a backup
    Restore {
        /// Backup ID to restore from
        backup_id: String,

        /// Configuration file path
        config_file: PathBuf,
    },

    /// Clean up old backups
    Cleanup {
        /// Maximum age in hours for backups to keep
        #[arg(short, long, default_value = "168")] // 1 week
        max_age_hours: u64,

        /// Configuration file path (optional, cleans all if not specified)
        config_file: Option<PathBuf>,
    },

    /// Show configuration file information
    Info {
        /// Configuration file path
        config_file: PathBuf,
    },

    /// Show backup metadata (timestamp/size/paths) for a specific backup ID
    ShowBackup {
        /// Backup ID (as shown in list-backups)
        backup_id: String,
    },
}

/// CLI runner
pub struct CliRunner {
    core: Sandbag,
    ui: crate::ui::UIManager,
}

impl CliRunner {
    /// Create a new CLI runner
    pub async fn new() -> Result<Self> {
        tokio::task::yield_now().await;
        let core = Sandbag::new();
        let ui = crate::ui::UIManager::new();

        Ok(Self { core, ui })
    }

    /// Run the CLI application
    pub async fn run(&mut self, cli: SandbagCli) -> Result<()> {
        // Set UI options
        self.ui.set_verbose(cli.verbose);
        self.ui.set_dry_run(cli.dry_run);

        match cli.command {
            Commands::Add {
                rule_input,
                linter,
                config,
            } => {
                self.handle_add(rule_input, linter, config).await?;
            }
            Commands::Scan {
                path,
                limit,
                include_low_confidence,
                include_docs,
            } => {
                self.handle_scan(path, limit, include_low_confidence, include_docs)
                    .await?;
            }
            Commands::Batch {
                input_file,
                continue_on_error,
                output,
            } => {
                self.handle_batch(input_file, continue_on_error, output)
                    .await?;
            }
            Commands::Config { config_command } => {
                self.handle_config(config_command).await?;
            }
            Commands::List { detailed } => {
                self.handle_list(detailed).await?;
            }
            Commands::Info { rule_id, linter } => {
                self.handle_info(rule_id, linter).await?;
            }
            Commands::TrainBayes {
                input,
                model_out,
                alpha,
            } => {
                self.handle_train_bayes(input, model_out, alpha).await?;
            }
        }

        Ok(())
    }

    /// Handle add command
    async fn handle_add(
        &mut self,
        rule_input: String,
        _linter: Option<String>,
        _config: Option<PathBuf>,
    ) -> Result<()> {
        self.ui
            .show_success(&format!("Processing rule: {rule_input}"));

        let matches = self.core.extract_rules(&rule_input).await?;

        if matches.is_empty() {
            self.ui.show_error("No valid rules found in input");
            return Ok(());
        }

        for rule_match in matches {
            self.ui.show_rule_match(&rule_match);

            if rule_match.confidence < crate::core::confidence::HIGH_CONFIDENCE_THRESHOLD {
                // TODO: Implement confirmation prompt
                self.ui
                    .show_warning("Low confidence match - consider reviewing");
            }

            if self.ui.dry_run {
                self.ui
                    .show_dry_run(&format!("Would apply rule {}", rule_match.rule_id));
            } else if let Err(e) = self.apply_via_config_manager(&rule_match).await {
                self.ui.show_error(&format!(
                    "Failed to apply rule {}: {}",
                    rule_match.rule_id, e
                ));
            } else {
                self.ui
                    .show_success(&format!("Successfully applied rule {}", rule_match.rule_id));
            }
        }

        Ok(())
    }

    /// Handle scan command
    async fn handle_scan(
        &mut self,
        path: Option<PathBuf>,
        limit: usize,
        include_low_confidence: bool,
        include_docs: bool,
    ) -> Result<()> {
        tokio::task::yield_now().await;
        let scan_path =
            path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        self.ui
            .show_success(&format!("Scanning directory: {}", scan_path.display()));

        // 1) Report present linter configs
        let registry = self.core.linter_registry();
        let supported = registry.get_supported_linters();
        for (name, info) in supported {
            for pattern in info.config_files {
                let candidate = scan_path.join(&pattern);
                if candidate.exists() {
                    self.ui
                        .show_success(&format!("Found {name} config: {}", candidate.display()));
                }
            }
        }

        // 2) Walk repo and extract rules from candidate text files
        let mut to_visit: Vec<PathBuf> = vec![scan_path.clone()];
        let mut matches: Vec<crate::core::RuleMatch> = Vec::new();

        while let Some(dir) = to_visit.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden and target-like dirs for speed
                    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if name.starts_with('.')
                        || name == "target"
                        || name == "node_modules"
                        || (!include_docs && name == "docs")
                    {
                        continue;
                    }
                    to_visit.push(path);
                    continue;
                }

                // Only scan small text-ish files by extension
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let file_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let is_text =
                    matches!(ext.as_str(), "md" | "txt" | "log" | "json" | "yaml" | "yml");
                let is_loglike = file_name.contains("lint")
                    || file_name.contains("problems")
                    || file_name.contains("errors");
                if !is_text && !is_loglike {
                    continue;
                }

                // Read up to 1 MB to avoid huge files
                let Ok(bytes) = std::fs::read(&path) else {
                    continue;
                };
                let slice = if bytes.len() > 1_000_000 {
                    &bytes[..1_000_000]
                } else {
                    &bytes[..]
                };
                let Ok(content) = String::from_utf8(slice.to_vec()) else {
                    continue;
                };

                // Require linter-like patterns somewhere in file to reduce false positives
                let content_lower = content.to_lowercase();
                let looks_like_linter_output = content_lower.contains("md0")
                    || content_lower.contains("md1")
                    || content_lower.contains("eslint")
                    || content_lower.contains("prettier/")
                    || content_lower.contains(": error ")
                    || content_lower.contains(": warn ")
                    || content_lower.contains("warning ");
                if !looks_like_linter_output {
                    continue;
                }

                let extracted = registry.extract_rules(&content);
                if !extracted.is_empty() {
                    self.ui.show_success(&format!(
                        "Extracted {} rule(s) from {}",
                        extracted.len(),
                        path.display()
                    ));
                    matches.extend(extracted);
                }
            }
        }

        // 3) Filter, limit, and display suggestions
        let threshold = if include_low_confidence {
            crate::core::confidence::LOW_CONFIDENCE_THRESHOLD
        } else {
            crate::core::confidence::MEDIUM_CONFIDENCE_THRESHOLD
        };

        matches.retain(|m| m.confidence >= threshold);

        // Deduplicate by (linter, rule_id)
        let mut seen: std::collections::HashSet<(String, String)> =
            std::collections::HashSet::new();
        matches.retain(|m| {
            let key = (m.linter.clone(), m.rule_id.clone());
            if seen.contains(&key) {
                return false;
            }
            seen.insert(key);
            true
        });

        // Sort by descending confidence
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let shown = matches.iter().take(limit).cloned().collect::<Vec<_>>();
        if shown.is_empty() {
            self.ui
                .show_warning("No rule suggestions found in scanned files");
        } else {
            self.ui
                .show_success(&format!("Top {} suggestion(s):", shown.len()));
            for m in shown {
                self.ui.show_rule_match(&m);
            }
        }

        Ok(())
    }

    /// Handle batch command
    async fn handle_batch(
        &mut self,
        input_file: PathBuf,
        _continue_on_error: bool,
        _output: Option<PathBuf>,
    ) -> Result<()> {
        tokio::task::yield_now().await;
        self.ui
            .show_success(&format!("Processing batch file: {}", input_file.display()));

        let content = std::fs::read_to_string(&input_file)?;
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let matches = self.core.extract_rules(trimmed).await?;
            if matches.is_empty() {
                self.ui
                    .show_warning(&format!("Line {}: no rules detected", line_no + 1));
                continue;
            }

            for rule_match in matches {
                self.ui.show_rule_match(&rule_match);
                if self.ui.dry_run {
                    self.ui
                        .show_dry_run(&format!("Would apply rule {}", rule_match.rule_id));
                } else if let Err(e) = self.apply_via_config_manager(&rule_match).await {
                    self.ui.show_error(&format!(
                        "Failed to apply rule {}: {}",
                        rule_match.rule_id, e
                    ));
                } else {
                    self.ui
                        .show_success(&format!("Applied rule {}", rule_match.rule_id));
                }
            }
        }

        Ok(())
    }

    async fn apply_via_config_manager(
        &mut self,
        rule_match: &crate::core::RuleMatch,
    ) -> anyhow::Result<()> {
        // Build a ConfigEntry based on the match
        let config_entry = ConfigEntry {
            rule_id: rule_match.rule_id.clone(),
            action: rule_match.suggested_action.clone(),
            scope: crate::core::ConfigScope::Global,
            metadata: {
                let mut m = std::collections::HashMap::new();
                // Provide a dotted path for ESLint/Prettier common placements
                match rule_match.linter.as_str() {
                    "eslint" => {
                        m.insert(
                            "path".to_string(),
                            serde_json::Value::String("rules".to_string()),
                        );
                        m
                    }
                    _ => m,
                }
            },
        };

        // Discover config file by linter
        let (path, format) = match rule_match.linter.as_str() {
            "markdownlint" => (
                CliRunner::find_config_path(&[
                    ".markdownlint.json",
                    ".markdownlint.yaml",
                    ".markdownlint.yml",
                ])
                .unwrap_or_else(|| PathBuf::from(".markdownlint.json")),
                crate::config::ConfigFormat::Json,
            ),
            "eslint" => (
                CliRunner::find_config_path(&[".eslintrc.json", ".eslintrc.yaml", ".eslintrc.yml"])
                    .unwrap_or_else(|| PathBuf::from(".eslintrc.json")),
                crate::config::ConfigFormat::Json,
            ),
            "prettier" => (
                CliRunner::find_config_path(&[
                    ".prettierrc.json",
                    ".prettierrc.yaml",
                    ".prettierrc.yml",
                ])
                .unwrap_or_else(|| PathBuf::from(".prettierrc.json")),
                crate::config::ConfigFormat::Json,
            ),
            _ => (
                PathBuf::from("config.json"),
                crate::config::ConfigFormat::Json,
            ),
        };

        // Ensure file exists
        if !path.exists() {
            let mgr = ConfigManager::new();
            mgr.create_config_file(&path, format).await?;
        }

        let mut mgr = ConfigManager::new();
        mgr.apply_rule_safely(&config_entry, &path).await?;
        Ok(())
    }

    fn find_config_path(candidates: &[&str]) -> Option<PathBuf> {
        for c in candidates {
            let p = Path::new(c);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    /// Handle training of Bayesian model from labeled JSONL
    pub async fn handle_train_bayes(
        &mut self,
        input: PathBuf,
        model_out: PathBuf,
        alpha: f64,
    ) -> Result<()> {
        tokio::task::yield_now().await;
        self.ui
            .show_success(&format!("Training Bayesian model from {}", input.display()));

        let data = std::fs::read_to_string(&input)?;
        let mut model = NaiveBayesModel::default();
        for (line_no, line) in data.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parsed: serde_json::Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    self.ui
                        .show_warning(&format!("Skipping line {}: {}", line_no + 1, e));
                    continue;
                }
            };
            let rule_id = if let Some(s) = parsed.get("rule_id").and_then(|v| v.as_str()) {
                s.to_string()
            } else {
                self.ui
                    .show_warning(&format!("Skipping line {}: missing rule_id", line_no + 1));
                continue;
            };
            let Some(text) = parsed.get("text").and_then(|v| v.as_str()) else {
                self.ui
                    .show_warning(&format!("Skipping line {}: missing text", line_no + 1));
                continue;
            };
            let ev = evidence_from_input(text);
            model.add_observation(&LabeledObservation {
                rule_id,
                evidence: ev,
            });
        }

        // Save trained model
        model.save_to_path(&model_out)?;
        self.ui.show_success(&format!(
            "Saved Bayesian model to {} (alpha = {})",
            model_out.display(),
            alpha
        ));
        Ok(())
    }

    /// Handle config command
    async fn handle_config(&mut self, config_command: ConfigCommands) -> Result<()> {
        tokio::task::yield_now().await;
        match config_command {
            ConfigCommands::ListBackups { config_file } => {
                self.ui
                    .show_success(&format!("Listing backups for: {}", config_file.display()));

                let manager = crate::config::backup::BackupManager::new();
                let backup_dir = manager.get_backup_dir().to_path_buf();
                let file_stem = config_file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let mut entries: Vec<(String, std::time::SystemTime)> = Vec::new();
                if backup_dir.exists() {
                    if let Ok(read_dir) = std::fs::read_dir(&backup_dir) {
                        for entry in read_dir.flatten() {
                            let path = entry.path();
                            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if !fname.ends_with(".backup") {
                                continue;
                            }
                            if !fname.starts_with(file_stem) {
                                continue;
                            }
                            let id = fname.trim_end_matches(".backup").to_string();
                            let modified = entry
                                .metadata()
                                .and_then(|m| m.modified())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            entries.push((id, modified));
                        }
                    }
                }

                // Sort by modified desc
                entries.sort_by(|a, b| b.1.cmp(&a.1));
                if entries.is_empty() {
                    println!("No backups found in {}", backup_dir.display());
                } else {
                    println!("Backups in {}:", backup_dir.display());
                    for (id, _ts) in entries {
                        println!("- {id}");
                    }
                }
            }
            ConfigCommands::Restore {
                backup_id,
                config_file,
            } => {
                self.ui.show_success(&format!(
                    "Restoring backup {backup_id} for: {}",
                    config_file.display()
                ));

                let manager = crate::config::backup::BackupManager::new();
                let backup_path = manager.get_backup_dir().join(format!("{backup_id}.backup"));
                if backup_path.exists() {
                    std::fs::copy(&backup_path, &config_file)?;
                    self.ui
                        .show_success(&format!("Restored {}", config_file.display()));
                } else {
                    self.ui
                        .show_error(&format!("Backup not found: {}", backup_path.display()));
                }
            }
            ConfigCommands::Cleanup {
                max_age_hours,
                config_file: _,
            } => {
                self.ui.show_success(&format!(
                    "Cleaning up backups older than {max_age_hours} hours"
                ));
                let cutoff = std::time::SystemTime::now()
                    .checked_sub(std::time::Duration::from_secs(max_age_hours * 3600))
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let manager = crate::config::backup::BackupManager::new();
                let backup_dir = manager.get_backup_dir().to_path_buf();
                let mut removed = 0usize;
                if backup_dir.exists() {
                    if let Ok(read_dir) = std::fs::read_dir(&backup_dir) {
                        for entry in read_dir.flatten() {
                            let path = entry.path();
                            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if !fname.ends_with(".backup") {
                                continue;
                            }
                            let modified = entry
                                .metadata()
                                .and_then(|m| m.modified())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            if modified < cutoff {
                                let _ = std::fs::remove_file(&path);
                                removed += 1;
                            }
                        }
                    }
                }
                self.ui
                    .show_success(&format!("Removed {removed} old backup(s)"));
            }
            ConfigCommands::Info { config_file } => {
                self.ui
                    .show_success(&format!("Showing info for: {}", config_file.display()));

                let exists = config_file.exists();
                let size = std::fs::metadata(&config_file).ok().map_or(0, |m| m.len());

                let manager = crate::config::backup::BackupManager::new();
                let backup_dir = manager.get_backup_dir().to_path_buf();
                let file_stem = config_file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let mut count = 0usize;
                if backup_dir.exists() {
                    if let Ok(read_dir) = std::fs::read_dir(&backup_dir) {
                        for entry in read_dir.flatten() {
                            let path = entry.path();
                            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if fname.ends_with(".backup") && fname.starts_with(file_stem) {
                                count += 1;
                            }
                        }
                    }
                }

                println!("Exists: {exists}");
                println!("Size: {size} bytes");
                println!("Backups: {count} in {}", backup_dir.display());
            }
            ConfigCommands::ShowBackup { backup_id } => {
                self.ui
                    .show_success(&format!("Backup metadata for: {backup_id}"));
                let manager = crate::config::backup::BackupManager::new();
                if let Some(info) = manager.get_backup_info(&backup_id) {
                    println!("ID: {}", info.id);
                    println!("Original: {}", info.original_path.display());
                    println!("Backup: {}", info.backup_path.display());
                    println!("Size: {} bytes", info.size);
                    println!("Timestamp: {}", info.timestamp.to_rfc3339());
                } else {
                    self.ui
                        .show_error(&format!("Backup not found in index: {backup_id}"));
                }
            }
        }

        Ok(())
    }

    /// Handle list command
    async fn handle_list(&mut self, detailed: bool) -> Result<()> {
        tokio::task::yield_now().await;
        self.ui.show_success("Supported linters:");

        let linter_names = self.core.linter_registry().get_linter_names();
        for name in linter_names {
            if detailed {
                println!("  - {name} (detailed info not yet implemented)");
            } else {
                println!("  - {name}");
            }
        }

        Ok(())
    }

    /// Handle info command
    async fn handle_info(&mut self, rule_id: String, _linter: Option<String>) -> Result<()> {
        tokio::task::yield_now().await;
        self.ui
            .show_success(&format!("Getting info for rule: {rule_id}"));

        // Basic rule info for markdownlint
        if rule_id.starts_with("MD") {
            let handler = crate::linters::markdownlint::MarkdownlintHandler::new();
            if let Some(desc) = handler.get_rule_description(&rule_id) {
                println!("Linter: markdownlint");
                println!("Rule: {rule_id}");
                println!("Description: {desc}");
                println!("Format: {}", handler.get_rule_format());
                println!("Config files: {:?}", handler.get_config_files());
                return Ok(());
            }
        }

        // ESLint rule info
        if rule_id.contains('/') || rule_id.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            let handler = crate::linters::eslint::ESLintHandler::new();
            if let Some(desc) = handler.get_rule_description(&rule_id) {
                println!("Linter: eslint");
                println!("Rule: {rule_id}");
                println!("Description: {desc}");
                println!("Format: {}", handler.get_rule_format());
                println!("Config files: {:?}", handler.get_config_file_patterns());
                return Ok(());
            }
        }

        // Prettier rule info
        if rule_id.starts_with("prettier") {
            let handler = crate::linters::prettier::PrettierHandler::new();
            println!("Linter: prettier");
            println!("Rule: {rule_id}");
            let options = handler.get_configuration_options();
            if let Some((opt, desc)) = options
                .iter()
                .find(|(k, _)| rule_id.to_lowercase().contains(&k.to_lowercase()))
            {
                println!("Option: {opt}");
                println!("Description: {desc}");
            }
            println!("Format: {}", handler.get_rule_format());
            println!("Config files: {:?}", handler.get_config_file_patterns());
            return Ok(());
        }

        // Fall back to listing linter formats
        let formats = self
            .core
            .linter_registry()
            .get_supported_linters()
            .into_iter()
            .map(|(name, info)| format!("{} => {}", name, info.rule_format))
            .collect::<Vec<_>>();
        println!("Unknown rule. Known linter formats: {}", formats.join(", "));

        Ok(())
    }
}

/// Run the CLI application
pub async fn run_cli() -> Result<()> {
    let cli = SandbagCli::parse();
    let mut runner = CliRunner::new().await?;
    runner.run(cli).await
}
