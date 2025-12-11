//! Utility functions for validation and error handling

pub mod fs;
pub mod validation;

// no external imports needed here

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall validity flag
    pub is_valid: bool,
    /// Collected validation errors
    pub errors: Vec<String>,
    /// Collected validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation passed
    pub fn is_ok(&self) -> bool {
        self.is_valid
    }

    /// Get all validation messages
    pub fn get_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();
        messages.extend(self.errors.iter().map(|e| format!("Error: {e}")));
        messages.extend(self.warnings.iter().map(|w| format!("Warning: {w}")));
        messages
    }
}

/// Error handling utilities
pub struct ErrorHandler;

impl ErrorHandler {
    /// Format error for display
    pub fn format_error(error: &anyhow::Error) -> String {
        format!("Error: {error}")
    }

    /// Format error with context
    pub fn format_error_with_context(error: &anyhow::Error, context: &str) -> String {
        format!("Error in {context}: {error}")
    }

    /// Check if an error is recoverable
    pub fn is_recoverable(_error: &anyhow::Error) -> bool {
        // TODO: Implement error recovery logic
        true
    }

    /// Suggest recovery strategy for an error
    pub fn suggest_recovery(_error: &anyhow::Error) -> Option<String> {
        // TODO: Implement recovery suggestion logic
        Some("Try again with different input".to_string())
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}
