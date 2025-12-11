//! Display formatting and progress indicators

use console::Term;

/// Display manager for formatted output
pub struct DisplayManager {
    term: Term,
}

impl DisplayManager {
    /// Create a new display manager
    pub fn new() -> Self {
        Self {
            term: Term::stdout(),
        }
    }

    /// Display a progress bar
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn show_progress(&self, message: &str, progress: f64) {
        let bar_length: usize = 40;
        let clamped = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        let filled = (clamped * bar_length as f64).round() as usize;
        let empty = bar_length.saturating_sub(filled);

        let bar = format!(
            "[{}{}] {:.1}%",
            "=".repeat(filled),
            " ".repeat(empty),
            clamped * 100.0
        );

        println!("{message} {bar}");
    }

    /// Display a spinner with message
    pub fn show_spinner(&self, message: &str) {
        // Simple spinner implementation
        println!("⏳ {message}");
    }

    /// Clear the terminal
    pub fn clear(&self) -> Result<(), std::io::Error> {
        self.term.clear_screen()
    }

    /// Display a table
    pub fn show_table(&self, headers: &[&str], rows: &[Vec<String>]) {
        // Simple table display
        println!("{}", headers.join(" | "));
        println!("{}", "-".repeat(headers.join(" | ").len()));

        for row in rows {
            println!("{}", row.join(" | "));
        }
    }
}

impl Default for DisplayManager {
    fn default() -> Self {
        Self::new()
    }
}
