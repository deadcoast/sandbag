//! Error recovery and confirmation prompts

use anyhow::{Error, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use std::collections::HashMap;

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Skip the current item and continue
    Skip,
    /// Abort the entire operation
    Abort,
    /// Use alternative approach
    Alternative,
    /// Ask user for manual input
    ManualInput,
}

/// Error recovery manager
pub struct ErrorRecoveryManager {
    #[allow(dead_code)]
    recovery_strategies: HashMap<String, RecoveryStrategy>,
    max_retries: u32,
    interactive_mode: bool,
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        Self {
            recovery_strategies: HashMap::new(),
            max_retries: 3,
            interactive_mode: true,
        }
    }

    /// Handle an error with recovery options
    pub async fn handle_error(&mut self, error: &Error, context: &str) -> Result<RecoveryStrategy> {
        tokio::task::yield_now().await;
        println!();
        println!("{}", "❌ Error Encountered".red().bold());
        println!("Context: {}", context.cyan());
        println!("Error: {}", error.to_string().red());
        println!();

        if !self.interactive_mode {
            return Ok(RecoveryStrategy::Abort);
        }

        let strategies = vec![
            "Retry operation",
            "Skip and continue",
            "Abort operation",
            "Use alternative approach",
            "Manual input",
        ];

        let selection = Select::new()
            .with_prompt("How would you like to proceed?")
            .items(&strategies)
            .default(0)
            .interact()
            .map_err(|e| anyhow::anyhow!("Selection failed: {}", e))?;

        #[allow(clippy::match_same_arms)]
        let strategy = match selection {
            0 => RecoveryStrategy::Retry,
            1 => RecoveryStrategy::Skip,
            2 => RecoveryStrategy::Abort,
            3 => RecoveryStrategy::Alternative,
            4 => RecoveryStrategy::ManualInput,
            _ => RecoveryStrategy::Abort,
        };

        Ok(strategy)
    }

    /// Confirm a potentially dangerous operation
    pub fn confirm_dangerous_operation(&self, operation: &str, details: &str) -> Result<bool> {
        if !self.interactive_mode {
            return Ok(true); // Auto-confirm in non-interactive mode
        }

        println!();
        println!("{}", "⚠️  Dangerous Operation Warning".yellow().bold());
        println!("Operation: {}", operation.cyan());
        println!("Details: {details}");
        println!();

        Confirm::new()
            .with_prompt("Are you sure you want to proceed?")
            .default(false)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Confirm rule application
    pub fn confirm_rule_application(
        &self,
        rule_id: &str,
        action: &str,
        confidence: f64,
    ) -> Result<bool> {
        if !self.interactive_mode {
            return Ok(true);
        }

        println!();
        println!("{}", "🔧 Rule Application Confirmation".blue().bold());
        println!("Rule: {}", rule_id.cyan());
        println!("Action: {}", action.yellow());
        println!("Confidence: {:.1}%", confidence * 100.0);

        if confidence < 0.8 {
            println!(
                "{}",
                "⚠️  Low confidence - please review carefully".yellow()
            );
        }

        println!();

        Confirm::new()
            .with_prompt("Apply this rule configuration?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Get user input for manual correction
    pub fn get_manual_input(&self, prompt: &str, default: Option<&str>) -> Result<String> {
        if !self.interactive_mode {
            return Ok(default.unwrap_or("").to_string());
        }

        let mut input = Input::new();
        input.with_prompt(prompt);

        if let Some(default_value) = default {
            input.default(default_value.to_string());
        }

        input
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Input failed: {}", e))
    }

    /// Handle configuration conflicts
    pub fn handle_configuration_conflicts(&self, conflicts: &[String]) -> Result<RecoveryStrategy> {
        if conflicts.is_empty() {
            return Ok(RecoveryStrategy::Skip);
        }

        println!();
        println!("{}", "⚠️  Configuration Conflicts Detected".yellow().bold());
        println!();

        for (i, conflict) in conflicts.iter().enumerate() {
            println!("{}. {}", i + 1, conflict);
        }

        println!();

        let options = vec![
            "Resolve automatically",
            "Resolve manually",
            "Skip conflicts",
            "Abort operation",
        ];

        let selection = Select::new()
            .with_prompt("How would you like to handle these conflicts?")
            .items(&options)
            .default(0)
            .interact()
            .map_err(|e| anyhow::anyhow!("Selection failed: {}", e))?;

        #[allow(clippy::match_same_arms)]
        let strategy = match selection {
            0 => RecoveryStrategy::Alternative,
            1 => RecoveryStrategy::ManualInput,
            2 => RecoveryStrategy::Skip,
            3 => RecoveryStrategy::Abort,
            _ => RecoveryStrategy::Abort,
        };

        Ok(strategy)
    }

    /// Handle backup restoration
    pub fn confirm_backup_restoration(&self, backup_id: &str, file_path: &str) -> Result<bool> {
        if !self.interactive_mode {
            return Ok(true);
        }

        println!();
        println!("{}", "🔄 Backup Restoration".blue().bold());
        println!("Backup ID: {}", backup_id.cyan());
        println!("File: {}", file_path.cyan());
        println!(
            "{}",
            "⚠️  This will overwrite the current configuration".yellow()
        );
        println!();

        Confirm::new()
            .with_prompt("Proceed with backup restoration?")
            .default(false)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Handle batch operation confirmation
    pub fn confirm_batch_operation(&self, file_path: &str, rule_count: usize) -> Result<bool> {
        if !self.interactive_mode {
            return Ok(true);
        }

        println!();
        println!("{}", "📦 Batch Operation Confirmation".blue().bold());
        println!("File: {}", file_path.cyan());
        println!("Rules to process: {rule_count}");
        println!();

        Confirm::new()
            .with_prompt("Proceed with batch processing?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Handle plugin installation
    pub fn confirm_plugin_installation(&self, plugin_name: &str, source: &str) -> Result<bool> {
        if !self.interactive_mode {
            return Ok(true);
        }

        println!();
        println!("{}", "🔌 Plugin Installation".blue().bold());
        println!("Plugin: {}", plugin_name.cyan());
        println!("Source: {}", source.cyan());
        println!();

        Confirm::new()
            .with_prompt("Install this plugin?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Set interactive mode
    pub fn set_interactive_mode(&mut self, interactive: bool) {
        self.interactive_mode = interactive;
    }

    /// Set maximum retries
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.max_retries = max_retries;
    }

    /// Get maximum retries
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Check if retries are available
    pub fn can_retry(&self, current_retries: u32) -> bool {
        current_retries < self.max_retries
    }

    /// Show retry information
    pub fn show_retry_info(&self, current_retries: u32) {
        if self.can_retry(current_retries) {
            println!(
                "{}",
                format!("🔄 Retry {}/{}", current_retries + 1, self.max_retries).yellow()
            );
        } else {
            println!("{}", "❌ Maximum retries exceeded".red());
        }
    }
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Confirmation prompts for common operations
pub struct ConfirmationPrompts;

impl ConfirmationPrompts {
    /// Confirm file overwrite
    pub fn confirm_file_overwrite(file_path: &str) -> Result<bool> {
        println!();
        println!("{}", "⚠️  File Overwrite Warning".yellow().bold());
        println!("File: {}", file_path.cyan());
        println!(
            "{}",
            "This operation will overwrite the existing file".red()
        );
        println!();

        Confirm::new()
            .with_prompt("Overwrite the file?")
            .default(false)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Confirm directory creation
    pub fn confirm_directory_creation(dir_path: &str) -> Result<bool> {
        println!();
        println!("{}", "📁 Directory Creation".blue().bold());
        println!("Directory: {}", dir_path.cyan());
        println!();

        Confirm::new()
            .with_prompt("Create this directory?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Confirm cleanup operation
    pub fn confirm_cleanup_operation(item_count: usize, item_type: &str) -> Result<bool> {
        println!();
        println!("{}", "🧹 Cleanup Operation".blue().bold());
        println!("Items to remove: {item_count} {item_type}");
        println!();

        Confirm::new()
            .with_prompt("Proceed with cleanup?")
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }

    /// Confirm configuration reset
    pub fn confirm_configuration_reset() -> Result<bool> {
        println!();
        println!("{}", "🔄 Configuration Reset".yellow().bold());
        println!(
            "{}",
            "This will reset all configuration to default values".red()
        );
        println!("{}", "All custom settings will be lost".red());
        println!();

        Confirm::new()
            .with_prompt("Reset configuration to defaults?")
            .default(false)
            .interact()
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::new();
        assert_eq!(manager.max_retries, 3);
        assert!(manager.interactive_mode);
    }

    #[test]
    fn test_retry_logic() {
        let manager = ErrorRecoveryManager::new();
        assert!(manager.can_retry(0));
        assert!(manager.can_retry(1));
        assert!(manager.can_retry(2));
        assert!(!manager.can_retry(3));
    }

    #[test]
    fn test_max_retries_setting() {
        let mut manager = ErrorRecoveryManager::new();
        manager.set_max_retries(5);
        assert_eq!(manager.get_max_retries(), 5);
        assert!(manager.can_retry(4));
        assert!(!manager.can_retry(5));
    }
}
