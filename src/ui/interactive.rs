//! Interactive user interface components

use dialoguer::{Confirm, Input, Select};
use anyhow::Result;

/// Interactive prompt manager
pub struct InteractiveManager;

impl InteractiveManager {
    /// Create a new interactive manager
    pub fn new() -> Self {
        Self
    }

    /// Ask for user confirmation
    pub fn confirm(&self, prompt: &str) -> Result<bool> {
        Confirm::new()
            .with_prompt(prompt)
            .default(true)
            .interact()
            .map_err(|e| anyhow::anyhow!("Interactive prompt failed: {}", e))
    }

    /// Ask for user input
    pub fn input(&self, prompt: &str) -> Result<String> {
        Input::new()
            .with_prompt(prompt)
            .interact_text()
            .map_err(|e| anyhow::anyhow!("Interactive input failed: {}", e))
    }

    /// Present a selection menu
    pub fn select(&self, prompt: &str, items: &[String]) -> Result<usize> {
        Select::new()
            .with_prompt(prompt)
            .items(items)
            .default(0)
            .interact()
            .map_err(|e| anyhow::anyhow!("Interactive selection failed: {}", e))
    }
}

impl Default for InteractiveManager {
    fn default() -> Self {
        Self::new()
    }
}
