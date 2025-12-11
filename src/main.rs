//! Sandbag CLI - Main entry point
//!
//! Intelligent linter configuration management with mathematical precision

use anyhow::Result;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up colored output
    colored::control::set_override(true);

    // Run the CLI application
    match sandbag::ui::cli::run_cli().await {
        Ok(_) => {
            // Success - exit cleanly
            std::process::exit(0);
        }
        Err(e) => {
            // Error - display and exit with error code
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
