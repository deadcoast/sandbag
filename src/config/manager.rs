//! Configuration manager for safe file operations and AST-based parsing

use crate::config::{ApplyError, ConfigFormat, ConfigParser, ParseError};
use crate::core::ConfigEntry;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// Configuration manager with backup and safety features
pub struct ConfigManager {
    parsers: std::collections::HashMap<String, Box<dyn ConfigParser>>,
    backup_manager: super::backup::BackupManager,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let mut parsers: std::collections::HashMap<String, Box<dyn ConfigParser>> =
            std::collections::HashMap::new();

        // Register parsers for different formats
        parsers.insert("json".to_string(), Box::new(super::parser::JsonParser));
        parsers.insert("yaml".to_string(), Box::new(super::parser::YamlParser));

        Self {
            parsers,
            backup_manager: super::backup::BackupManager::new(),
        }
    }

    /// Apply a rule safely with backup and validation
    pub async fn apply_rule_safely<P: AsRef<Path>>(
        &mut self,
        rule: &ConfigEntry,
        file_path: P,
    ) -> Result<(), ApplyError> {
        let file_path = file_path.as_ref();
        // Create backup first
        let backup_id = self.backup_manager.create_backup(file_path).await?;

        // Read and parse current config
        let content = fs::read_to_string(file_path).await?;
        let parser = self.get_parser_for_file(file_path)?;
        let mut ast = parser.parse(&content)?;

        // Apply modification
        match parser.insert_rule(&mut ast, rule) {
            Ok(()) => {
                let new_content = parser.serialize(&ast)?;

                // Validate before writing
                if self.validate_config(&new_content, parser)? {
                    fs::write(file_path, new_content).await?;
                    Ok(())
                } else {
                    // Restore from backup if validation fails
                    self.backup_manager.restore_backup(&backup_id).await?;
                    Err(ApplyError::ValidationFailed)
                }
            }
            Err(e) => {
                // Restore from backup if modification fails
                self.backup_manager.restore_backup(&backup_id).await?;
                Err(ApplyError::ModificationFailed(e))
            }
        }
    }

    /// Get appropriate parser for file type
    fn get_parser_for_file(&self, file_path: &Path) -> Result<&dyn ConfigParser, ParseError> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        self.parsers
            .get(extension)
            .map(std::convert::AsRef::as_ref)
            .ok_or_else(|| ParseError::UnsupportedFormat(extension.to_string()))
    }

    /// Validate configuration content
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn validate_config(
        &self,
        content: &str,
        parser: &dyn ConfigParser,
    ) -> Result<bool, ParseError> {
        // Try to parse the content to ensure it's valid
        match parser.parse(content) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Detect configuration format from file extension
    pub fn detect_format(&self, file_path: &Path) -> ConfigFormat {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        match extension.to_lowercase().as_str() {
            "json" | "jsonc" => ConfigFormat::Json,
            "yaml" | "yml" => ConfigFormat::Yaml,
            "toml" => ConfigFormat::Toml,
            "ini" => ConfigFormat::Ini,
            _ => ConfigFormat::Custom(extension.to_string()),
        }
    }

    /// Check if file exists and is readable
    pub async fn check_file_access<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<bool, std::io::Error> {
        Ok(fs::metadata(file_path.as_ref()).await.is_ok())
    }

    /// Get file modification time
    pub async fn get_file_modification_time<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<chrono::DateTime<chrono::Utc>, std::io::Error> {
        let metadata = fs::metadata(file_path.as_ref()).await?;
        let modified = metadata.modified()?;
        Ok(chrono::DateTime::from(modified))
    }

    /// Create a new configuration file if it doesn't exist
    pub async fn create_config_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        format: ConfigFormat,
    ) -> Result<(), std::io::Error> {
        let default_content = match format {
            ConfigFormat::Json => "{\n  \n}\n",
            // Same default content for non-JSON formats for now
            ConfigFormat::Yaml
            | ConfigFormat::Toml
            | ConfigFormat::Ini
            | ConfigFormat::Custom(_) => "# Configuration file\n\n",
        };

        fs::write(file_path.as_ref(), default_content).await
    }

    /// List available backup files
    pub async fn list_backups<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<String>, super::backup::BackupError> {
        self.backup_manager.list_backups(file_path.as_ref()).await
    }

    /// Restore from a specific backup
    pub async fn restore_from_backup(
        &self,
        backup_id: &str,
        _file_path: &Path,
    ) -> Result<(), super::backup::BackupError> {
        self.backup_manager.restore_backup(backup_id).await
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(!manager.parsers.is_empty());
    }

    #[tokio::test]
    async fn test_format_detection() {
        let manager = ConfigManager::new();

        assert!(matches!(
            manager.detect_format(Path::new("config.json")),
            ConfigFormat::Json
        ));

        assert!(matches!(
            manager.detect_format(Path::new("config.yaml")),
            ConfigFormat::Yaml
        ));
    }

    #[tokio::test]
    async fn test_file_access_check() {
        let manager = ConfigManager::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.json");

        // File doesn't exist
        assert!(!manager.check_file_access(&test_file).await.unwrap());

        // Create file and check again
        fs::write(&test_file, "{}").await.unwrap();
        assert!(manager.check_file_access(&test_file).await.unwrap());
    }
}
