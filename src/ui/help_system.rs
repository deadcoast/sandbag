//! Comprehensive help system with interactive documentation

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Select};
use std::collections::HashMap;

/// Help system for comprehensive documentation
pub struct HelpSystem {
    /// Help topics indexed by name
    topics: HashMap<String, HelpTopic>,
    /// Example snippets for topics
    examples: HashMap<String, Vec<HelpExample>>,
    /// If true, enable interactive prompts in help
    interactive_mode: bool,
}

/// Help topic with detailed information
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// Topic title
    pub title: String,
    /// Topic description
    pub description: String,
    /// Usage string
    pub usage: String,
    /// Command-line options for the topic
    pub options: Vec<HelpOption>,
    /// Example invocations
    pub examples: Vec<String>,
    /// Related topics for further reading
    pub related_topics: Vec<String>,
}

/// Help option with description
#[derive(Debug, Clone)]
pub struct HelpOption {
    /// Flag description as shown to the user
    pub flag: String,
    /// Flag explanation
    pub description: String,
    /// Whether the flag is required
    pub required: bool,
    /// Default value if any
    pub default: Option<String>,
}

/// Help example with description
#[derive(Debug, Clone)]
pub struct HelpExample {
    /// Short example title
    pub title: String,
    /// Description of the example
    pub description: String,
    /// Command to execute
    pub command: String,
    /// Expected output
    pub output: String,
}

impl HelpSystem {
    /// Create a new help system
    pub fn new() -> Self {
        let mut topics = HashMap::new();
        let mut examples = HashMap::new();

        // Initialize help topics
        Self::initialize_topics(&mut topics);
        Self::initialize_examples(&mut examples);

        Self {
            topics,
            examples,
            interactive_mode: true,
        }
    }

    /// Initialize help topics
    fn initialize_topics(topics: &mut HashMap<String, HelpTopic>) {
        // Add command
        topics.insert("add".to_string(), HelpTopic {
            title: "Add Rule Configuration".to_string(),
            description: "Add a rule to the linter configuration file. Automatically detects the linter type and applies the appropriate configuration.".to_string(),
            usage: "sandbag add <rule_input> [options]".to_string(),
            options: vec![
                HelpOption {
                    flag: "--linter, -l".to_string(),
                    description: "Force specific linter (skip auto-detection)".to_string(),
                    required: false,
                    default: None,
                },
                HelpOption {
                    flag: "--config, -c".to_string(),
                    description: "Configuration file path (auto-detected if not specified)".to_string(),
                    required: false,
                    default: None,
                },
                HelpOption {
                    flag: "--yes, -y".to_string(),
                    description: "Skip confirmation prompts".to_string(),
                    required: false,
                    default: Some("false".to_string()),
                },
            ],
            examples: vec![
                "sandbag add MD033".to_string(),
                "sandbag add 'MD033/no-inline-html: Inline HTML detected'".to_string(),
                "sandbag add MD033 --linter markdownlint".to_string(),
            ],
            related_topics: vec!["scan".to_string(), "batch".to_string()],
        });

        // Scan command
        topics.insert("scan".to_string(), HelpTopic {
            title: "Scan Project for Rules".to_string(),
            description: "Scan your project for suggested rule suppressions based on existing code patterns.".to_string(),
            usage: "sandbag scan [options]".to_string(),
            options: vec![
                HelpOption {
                    flag: "--path, -p".to_string(),
                    description: "Directory to scan (defaults to current)".to_string(),
                    required: false,
                    default: Some(".".to_string()),
                },
                HelpOption {
                    flag: "--limit, -l".to_string(),
                    description: "Maximum number of suggestions".to_string(),
                    required: false,
                    default: Some("10".to_string()),
                },
                HelpOption {
                    flag: "--include-low-confidence".to_string(),
                    description: "Include low confidence suggestions".to_string(),
                    required: false,
                    default: Some("false".to_string()),
                },
            ],
            examples: vec![
                "sandbag scan".to_string(),
                "sandbag scan --path src/ --limit 20".to_string(),
                "sandbag scan --include-low-confidence".to_string(),
            ],
            related_topics: vec!["add".to_string(), "batch".to_string()],
        });

        // Config command
        topics.insert(
            "config".to_string(),
            HelpTopic {
                title: "Configuration Management".to_string(),
                description: "Manage configuration files, backups, and settings.".to_string(),
                usage: "sandbag config <subcommand> [options]".to_string(),
                options: vec![
                    HelpOption {
                        flag: "list-backups".to_string(),
                        description: "List available backups for a configuration file".to_string(),
                        required: false,
                        default: None,
                    },
                    HelpOption {
                        flag: "restore".to_string(),
                        description: "Restore configuration from a backup".to_string(),
                        required: false,
                        default: None,
                    },
                    HelpOption {
                        flag: "cleanup".to_string(),
                        description: "Clean up old backup files".to_string(),
                        required: false,
                        default: None,
                    },
                ],
                examples: vec![
                    "sandbag config list-backups .markdownlint.json".to_string(),
                    "sandbag config restore backup_123 .markdownlint.json".to_string(),
                    "sandbag config cleanup --max-age 168".to_string(),
                ],
                related_topics: vec!["add".to_string(), "backup".to_string()],
            },
        );
    }

    /// Initialize help examples
    fn initialize_examples(examples: &mut HashMap<String, Vec<HelpExample>>) {
        // Add examples
        examples.insert(
            "add".to_string(),
            vec![
                HelpExample {
                    title: "Basic Rule Addition".to_string(),
                    description: "Add a simple rule ID to disable it".to_string(),
                    command: "sandbag add MD033".to_string(),
                    output: r"🔧 Adding Rule Configuration
==========================================

⏳ Analyzing rule input...
📊 Rule Analysis Results
==================================================

Rule 1: MD033
Confidence: [████████░░] 95.0%
Linter: markdownlint
Suggested Action: Disable
Message: Inline HTML detected

🧠 Semantic Analysis
------------------------------
Rule MD033: security

💡 Recommendations
• Consider using more specific actions instead of disabling rules (Priority: 0.8)

✅ Rule configuration applied successfully!"
                        .to_string(),
                },
                HelpExample {
                    title: "Complex Rule Input".to_string(),
                    description: "Add a rule from linter output with context".to_string(),
                    command: "sandbag add 'MD033/no-inline-html: Inline HTML [Element: div]'"
                        .to_string(),
                    output: r"🔧 Adding Rule Configuration
==========================================

⏳ Analyzing rule input...
📊 Rule Analysis Results
==================================================

Rule 1: MD033
Confidence: [████████░░] 95.0%
Linter: markdownlint
Suggested Action: Disable
Message: Inline HTML [Element: div]

✅ Rule configuration applied successfully!"
                        .to_string(),
                },
            ],
        );

        // Scan examples
        examples.insert(
            "scan".to_string(),
            vec![HelpExample {
                title: "Project Scan".to_string(),
                description: "Scan the current project for rule suggestions".to_string(),
                command: "sandbag scan".to_string(),
                output: r"🔧 Project Configuration Scan
==========================================

████████████████████████████████████████ 100% Scanning... 100%
Scan completed

🔍 Scan Results
Limit: 10 rules
Include low confidence: No

Rule        Description                Confidence Priority
-------------------------------------------------------
MD033       No inline HTML            95.0%      High
MD013       Line length               87.0%      High
MD041       First line heading        72.0%      Medium
MD009       No trailing spaces        65.0%      Medium"
                    .to_string(),
            }],
        );
    }

    /// Show help for a specific topic
    pub fn show_help(&self, topic: Option<&str>) -> Result<()> {
        match topic {
            Some(topic_name) => self.show_topic_help(topic_name),
            None => self.show_general_help(),
        }
    }

    /// Show general help
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn show_general_help(&self) -> Result<()> {
        println!();
        println!("{}", "📚 SANDBAG HELP".blue().bold());
        println!("{}", "=".repeat(50));
        println!();
        println!(
            "{}",
            "Intelligent linter configuration management with mathematical precision".green()
        );
        println!();
        println!("{}", "Available Commands:".yellow().bold());
        println!();

        let commands = vec![
            ("add", "Add a rule to ignore list"),
            ("scan", "Scan project for suggested suppressions"),
            ("batch", "Batch process rules from file"),
            ("config", "Manage configuration and backups"),
            ("list", "List supported linters"),
            ("info", "Show information about a specific rule"),
        ];

        for (cmd, desc) in commands {
            println!("  {:<10} {}", cmd.cyan(), desc);
        }

        println!();
        println!("{}", "For detailed help on a specific command:".yellow());
        println!("  sandbag help <command>");
        println!();
        println!("{}", "Examples:".yellow());
        println!("  sandbag help add");
        println!("  sandbag help scan");
        println!("  sandbag help config");
        println!();

        Ok(())
    }

    /// Show help for a specific topic
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn show_topic_help(&self, topic_name: &str) -> Result<()> {
        if let Some(topic) = self.topics.get(topic_name) {
            self.display_topic(topic);

            // Show examples if available
            if let Some(examples) = self.examples.get(topic_name) {
                self.display_examples(examples);
            }

            // Show related topics
            if !topic.related_topics.is_empty() {
                self.display_related_topics(&topic.related_topics);
            }
        } else {
            self.show_topic_not_found(topic_name);
        }

        Ok(())
    }

    /// Display a help topic
    #[allow(clippy::unused_self)]
    fn display_topic(&self, topic: &HelpTopic) {
        println!();
        println!("{}", format!("📖 {}", topic.title).blue().bold());
        println!("{}", "=".repeat(topic.title.len() + 4));
        println!();
        println!("{}", topic.description);
        println!();
        println!("{}", "Usage:".yellow().bold());
        println!("  {}", topic.usage.cyan());
        println!();

        if !topic.options.is_empty() {
            println!("{}", "Options:".yellow().bold());
            for option in &topic.options {
                let required_text = if option.required { " (required)" } else { "" };
                let default_text = option
                    .default
                    .as_ref()
                    .map(|d| format!(" (default: {d})"))
                    .unwrap_or_default();
                println!(
                    "  {:<20} {}{}{}",
                    option.flag.cyan(),
                    option.description,
                    required_text.red(),
                    default_text.green()
                );
            }
            println!();
        }

        if !topic.examples.is_empty() {
            println!("{}", "Examples:".yellow().bold());
            for example in &topic.examples {
                println!("  {}", example.cyan());
            }
            println!();
        }
    }

    /// Display examples for a topic
    #[allow(clippy::unused_self)]
    fn display_examples(&self, examples: &[HelpExample]) {
        println!("{}", "💡 Interactive Examples:".yellow().bold());
        println!();

        for (i, example) in examples.iter().enumerate() {
            println!(
                "{}",
                format!("Example {}: {}", i + 1, example.title)
                    .blue()
                    .bold()
            );
            println!("{}", example.description.italic());
            println!();
            println!("{}", "Command:".yellow());
            println!("  {}", example.command.cyan());
            println!();
            println!("{}", "Output:".yellow());
            println!("{}", example.output);
            println!();
        }
    }

    /// Display related topics
    #[allow(clippy::unused_self)]
    fn display_related_topics(&self, related_topics: &[String]) {
        println!("{}", "🔗 Related Topics:".yellow().bold());
        for topic in related_topics {
            println!("  • {}", topic.cyan());
        }
        println!();
    }

    /// Show topic not found message
    fn show_topic_not_found(&self, topic_name: &str) {
        println!();
        println!(
            "{}",
            format!("❌ Help topic '{topic_name}' not found")
                .red()
                .bold()
        );
        println!();
        println!("{}", "Available topics:".yellow());
        for topic in self.topics.keys() {
            println!("  • {}", topic.cyan());
        }
        println!();
        println!("Use 'sandbag help' to see all available commands.");
    }

    /// Show interactive help
    pub fn show_interactive_help(&self) -> Result<()> {
        if !self.interactive_mode {
            return self.show_help(None);
        }

        println!();
        println!("{}", "🎯 Interactive Help System".blue().bold());
        println!("{}", "=".repeat(30));
        println!();

        let topics: Vec<&str> = self
            .topics
            .keys()
            .map(std::string::String::as_str)
            .collect();
        let selection = Select::new()
            .with_prompt("Select a topic to learn more about:")
            .items(&topics)
            .default(0)
            .interact()
            .map_err(|e| anyhow::anyhow!("Selection failed: {}", e))?;

        let selected_topic = topics[selection];
        self.show_topic_help(selected_topic)?;

        // Ask if user wants to see examples
        if Confirm::new()
            .with_prompt("Would you like to see interactive examples?")
            .default(true)
            .interact()
            .unwrap_or(false)
        {
            self.show_interactive_examples(selected_topic)?;
        }

        Ok(())
    }

    /// Show interactive examples
    fn show_interactive_examples(&self, topic_name: &str) -> Result<()> {
        if let Some(examples) = self.examples.get(topic_name) {
            println!();
            println!("{}", "🎮 Interactive Examples".blue().bold());
            println!("{}", "=".repeat(25));
            println!();

            for (i, example) in examples.iter().enumerate() {
                println!(
                    "{}",
                    format!("Example {}: {}", i + 1, example.title)
                        .green()
                        .bold()
                );
                println!("{}", example.description);
                println!();

                if Confirm::new()
                    .with_prompt(format!("Run example {}?", i + 1))
                    .default(true)
                    .interact()
                    .unwrap_or(false)
                {
                    self.run_example(example)?;
                }
                println!();
            }
        }

        Ok(())
    }

    /// Run an example (simulated)
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn run_example(&self, example: &HelpExample) -> Result<()> {
        println!("{}", "🚀 Running example...".blue());
        println!("Command: {}", example.command.cyan());
        println!();
        println!("{}", "Output:".yellow());
        println!("{}", example.output);
        println!();
        println!("{}", "✅ Example completed successfully!".green());
        Ok(())
    }

    /// Search help topics
    pub fn search_help(&self, query: &str) -> Result<()> {
        println!();
        println!(
            "{}",
            format!("🔍 Search Results for '{query}'").blue().bold()
        );
        println!("{}", "=".repeat(40));
        println!();

        let mut results = Vec::new();

        for (topic_name, topic) in &self.topics {
            if topic_name.contains(query)
                || topic.title.to_lowercase().contains(&query.to_lowercase())
                || topic
                    .description
                    .to_lowercase()
                    .contains(&query.to_lowercase())
            {
                results.push((topic_name, topic));
            }
        }

        if results.is_empty() {
            println!("{}", "No results found.".red());
            println!("Try searching for: add, scan, config, batch, list, info");
        } else {
            for (topic_name, topic) in results {
                println!("{}", format!("📖 {}", topic.title).green().bold());
                println!("  Command: {}", topic_name.cyan());
                println!("  {}", topic.description);
                println!();
            }
        }

        Ok(())
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_system_creation() {
        let help = HelpSystem::new();
        assert!(!help.topics.is_empty());
        assert!(!help.examples.is_empty());
    }

    #[test]
    fn test_topic_help() {
        let help = HelpSystem::new();
        assert!(help.topics.contains_key("add"));
        assert!(help.topics.contains_key("scan"));
    }

    #[test]
    fn test_examples() {
        let help = HelpSystem::new();
        assert!(help.examples.contains_key("add"));
        assert!(help.examples.contains_key("scan"));
    }
}
