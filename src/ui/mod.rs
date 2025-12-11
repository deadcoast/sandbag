//! User interface components for CLI interaction

pub mod cli;
pub mod display;
pub mod enhanced_cli;
pub mod error_recovery;
pub mod help_system;
pub mod interactive;

use crate::core::RuleMatch;

/// User interface manager
pub struct UIManager {
    /// If true, enable interactive prompts
    pub interactive: bool,
    /// If true, show verbose output
    pub verbose: bool,
    /// If true, perform a dry run without side effects
    pub dry_run: bool,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new() -> Self {
        Self {
            interactive: true,
            verbose: false,
            dry_run: false,
        }
    }

    /// Set interactive mode
    pub fn set_interactive(&mut self, interactive: bool) {
        self.interactive = interactive;
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Set dry run mode
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Display a success message
    pub fn show_success(&self, message: &str) {
        if self.verbose {
            println!("✅ {message}");
        } else {
            println!("{message}");
        }
    }

    /// Display an error message
    pub fn show_error(&self, message: &str) {
        eprintln!("❌ {message}");
    }

    /// Display a warning message
    pub fn show_warning(&self, message: &str) {
        if self.verbose {
            eprintln!("⚠️  {message}");
        }
    }

    /// Display rule match information
    pub fn show_rule_match(&self, rule_match: &RuleMatch) {
        if self.verbose {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let percent: u8 = (rule_match.confidence * 100.0) as u8;
            println!("📋 Rule: {} ({}% confidence)", rule_match.rule_id, percent);
            println!("   Linter: {}", rule_match.linter);
            println!("   Action: {:?}", rule_match.suggested_action);
        } else {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let percent: u8 = (rule_match.confidence * 100.0) as u8;
            println!("Rule: {} ({}%)", rule_match.rule_id, percent);
        }
    }

    /// Display dry run information
    pub fn show_dry_run(&self, message: &str) {
        if self.dry_run {
            println!("🔍 [DRY RUN] {message}");
        }
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}
