//! File system utilities

use anyhow::Result;
use std::path::Path;

/// File system utilities
pub struct FSUtils;

impl FSUtils {
    /// Check if a file exists and is readable
    pub fn file_exists_and_readable(path: &Path) -> bool {
        path.exists() && path.is_file() && std::fs::metadata(path).is_ok()
    }

    /// Get file size
    pub fn get_file_size(path: &Path) -> Result<u64> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// Create directory if it doesn't exist
    pub fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Get file extension
    pub fn get_file_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_lowercase)
    }

    /// Check if file has a specific extension
    pub fn has_extension(path: &Path, extension: &str) -> bool {
        Self::get_file_extension(path).is_some_and(|ext| ext == extension.to_lowercase())
    }
}
