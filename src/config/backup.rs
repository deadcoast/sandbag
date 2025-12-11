//! Backup management for safe configuration operations

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Unique backup identifier
    pub id: String,
    /// Original file path backed up
    pub original_path: PathBuf,
    /// Path to the backup file
    pub backup_path: PathBuf,
    /// Backup creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Size of the backup in bytes
    pub size: u64,
}

/// Backup manager for atomic operations
pub struct BackupManager {
    backup_dir: PathBuf,
    index_path: PathBuf,
    backups: HashMap<String, BackupInfo>,
}

/// Backup error types
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// IO error while performing backup operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Requested backup ID not found
    #[error("Backup not found: {0}")]
    BackupNotFound(String),
    /// Failed to create the backup directory
    #[error("Backup directory creation failed: {0}")]
    DirectoryCreationFailed(String),
    /// Failed to create the backup file
    #[error("Backup file creation failed: {0}")]
    FileCreationFailed(String),
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new() -> Self {
        let backup_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".sandbag_backups");

        let index_path = backup_dir.join("index.json");
        let mut mgr = Self {
            backup_dir,
            index_path,
            backups: HashMap::new(),
        };

        mgr.load_index();
        mgr
    }

    /// Create a backup of a file
    pub async fn create_backup(&mut self, file_path: &Path) -> Result<String, BackupError> {
        // Ensure backup directory exists
        tokio::fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| BackupError::DirectoryCreationFailed(e.to_string()))?;

        // Generate backup ID
        let backup_id = self.generate_backup_id(file_path);
        let backup_path = self.backup_dir.join(format!("{backup_id}.backup"));

        // Copy file to backup location
        tokio::fs::copy(file_path, &backup_path)
            .await
            .map_err(|e| BackupError::FileCreationFailed(e.to_string()))?;

        // Get file metadata
        let metadata = tokio::fs::metadata(file_path).await?;
        let timestamp = Utc::now();

        // Store backup info
        let backup_info = BackupInfo {
            id: backup_id.clone(),
            original_path: file_path.to_path_buf(),
            backup_path,
            timestamp,
            size: metadata.len(),
        };

        self.backups.insert(backup_id.clone(), backup_info);
        let _ = self.save_index();

        Ok(backup_id)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_id: &str) -> Result<(), BackupError> {
        let backup_info = self
            .backups
            .get(backup_id)
            .ok_or_else(|| BackupError::BackupNotFound(backup_id.to_string()))?;

        // Copy backup to original location
        tokio::fs::copy(&backup_info.backup_path, &backup_info.original_path).await?;

        Ok(())
    }

    /// List all backups for a file
    pub async fn list_backups(&self, file_path: &Path) -> Result<Vec<String>, BackupError> {
        tokio::task::yield_now().await;
        let mut backup_ids = Vec::new();

        for (id, info) in &self.backups {
            if info.original_path == file_path {
                backup_ids.push(id.clone());
            }
        }

        // Sort by timestamp (newest first)
        backup_ids.sort_by(|a, b| {
            let time_a = self.backups.get(a).map(|info| info.timestamp);
            let time_b = self.backups.get(b).map(|info| info.timestamp);
            time_b.cmp(&time_a)
        });

        Ok(backup_ids)
    }

    /// Get backup information
    pub fn get_backup_info(&self, backup_id: &str) -> Option<&BackupInfo> {
        self.backups.get(backup_id)
    }

    /// Clean up old backups
    pub async fn cleanup_old_backups(&mut self, max_age_hours: u64) -> Result<usize, BackupError> {
        #[allow(clippy::cast_possible_wrap)]
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        let mut removed_count = 0;

        let mut to_remove = Vec::new();

        for (id, info) in &self.backups {
            if info.timestamp < cutoff_time {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            if let Some(info) = self.backups.remove(&id) {
                // Remove backup file
                if let Err(e) = tokio::fs::remove_file(&info.backup_path).await {
                    eprintln!(
                        "Warning: Failed to remove backup file {}: {}",
                        info.backup_path.display(),
                        e
                    );
                }
                removed_count += 1;
            }
        }

        let _ = self.save_index();
        Ok(removed_count)
    }

    /// Generate a unique backup ID
    #[allow(clippy::unused_self)]
    fn generate_backup_id(&self, file_path: &Path) -> String {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let timestamp = Utc::now().timestamp();
        let random_suffix = rand::random::<u32>();

        format!("{file_name}_{timestamp}_{random_suffix:x}")
    }

    /// Get backup directory path
    pub fn get_backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// Set custom backup directory
    pub fn set_backup_dir(&mut self, path: PathBuf) {
        self.backup_dir = path;
    }

    /// Persist current backup index to disk
    fn save_index(&self) -> Result<(), std::io::Error> {
        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&self.backup_dir) {
            eprintln!(
                "Failed to create backup dir {}: {}",
                self.backup_dir.display(),
                e
            );
        }
        let list: Vec<&BackupInfo> = self.backups.values().collect();
        let json = serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".to_string());
        std::fs::write(&self.index_path, json)
    }

    /// Load backup index from disk into memory
    fn load_index(&mut self) {
        let Ok(bytes) = std::fs::read(&self.index_path) else {
            return;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return;
        };
        let Ok(list) = serde_json::from_str::<Vec<BackupInfo>>(&text) else {
            return;
        };
        for info in list {
            self.backups.insert(info.id.clone(), info);
        }
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_creation() {
        let mut manager = BackupManager::new();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.json");

        // Create test file
        tokio::fs::write(&test_file, r#"{"test": "data"}"#)
            .await
            .unwrap();

        // Create backup
        let backup_id = manager.create_backup(&test_file).await.unwrap();
        assert!(!backup_id.is_empty());

        // Verify backup exists
        let backup_info = manager.get_backup_info(&backup_id).unwrap();
        assert_eq!(backup_info.original_path, test_file);
        assert!(backup_info.backup_path.exists());
    }

    #[tokio::test]
    async fn test_backup_restore() {
        let mut manager = BackupManager::new();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.json");

        // Create test file
        let original_content = r#"{"test": "data"}"#;
        tokio::fs::write(&test_file, original_content)
            .await
            .unwrap();

        // Create backup
        let backup_id = manager.create_backup(&test_file).await.unwrap();

        // Modify file
        tokio::fs::write(&test_file, r#"{"modified": "data"}"#)
            .await
            .unwrap();

        // Restore from backup
        manager.restore_backup(&backup_id).await.unwrap();

        // Verify content is restored
        let restored_content = tokio::fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(restored_content, original_content);
    }

    #[tokio::test]
    async fn test_backup_listing() {
        let mut manager = BackupManager::new();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.json");

        // Create test file
        tokio::fs::write(&test_file, r#"{"test": "data"}"#)
            .await
            .unwrap();

        // Create multiple backups
        let backup_id1 = manager.create_backup(&test_file).await.unwrap();
        let backup_id2 = manager.create_backup(&test_file).await.unwrap();

        // List backups
        let backups = manager.list_backups(&test_file).await.unwrap();
        assert_eq!(backups.len(), 2);
        assert!(backups.contains(&backup_id1));
        assert!(backups.contains(&backup_id2));
    }
}
