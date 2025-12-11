#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict-docs", deny(missing_docs))]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines
)]
//! Sandbag - Intelligent linter configuration management

pub mod config;
pub mod core;
pub mod linters;
pub mod performance;
pub mod ui;
pub mod utils;

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

// Re-export main types for convenience
pub use config::{backup::BackupManager, parser::ConfigParser, writer::ConfigWriter};
pub use core::{
    confidence::ConfidenceCalculator, rule_extractor::RuleExtractor, similarity::SimilarityAnalyzer,
};
pub use linters::LinterRegistry;
pub use ui::cli::SandbagCli;

/// Main application struct
pub struct Sandbag {
    // Core components will be added as needed
}

impl Sandbag {
    /// Create a new Sandbag instance
    pub fn new() -> Self {
        Self {}
    }

    /// Extract rules from input string
    pub async fn extract_rules(&self, input: &str) -> Result<Vec<core::RuleMatch>> {
        tokio::task::yield_now().await;
        // Use the linter registry to extract rules from the provided input
        let registry: &linters::LinterRegistry = self.linter_registry();
        let matches = registry.extract_rules(input);
        Ok(matches)
    }

    /// Apply a rule to the system
    pub async fn apply_rule(&mut self, rule_match: &core::RuleMatch) -> Result<()> {
        tokio::task::yield_now().await;
        match rule_match.linter.as_str() {
            "markdownlint" => self.apply_markdownlint_rule(rule_match).await,
            "eslint" => self.apply_eslint_rule(rule_match).await,
            "prettier" => self.apply_prettier_rule(rule_match).await,
            _ => {
                // For now, only markdownlint config writing is implemented
                Ok(())
            }
        }
    }

    /// Get linter registry
    pub fn linter_registry(&self) -> &linters::LinterRegistry {
        // TODO: Return actual registry
        static REGISTRY: std::sync::LazyLock<linters::LinterRegistry> =
            std::sync::LazyLock::new(linters::LinterRegistry::new);
        &REGISTRY
    }

    async fn apply_markdownlint_rule(&mut self, rule_match: &core::RuleMatch) -> Result<()> {
        // Discover config
        let config_path = Sandbag::find_markdownlint_config()
            .unwrap_or_else(|| PathBuf::from(".markdownlint.json"));

        // Ensure file exists
        if !config_path.exists() {
            fs::write(&config_path, b"{\n  \"default\": true\n}\n")?;
        }

        // Backup
        let mut backup_mgr = config::backup::BackupManager::new();
        let _backup_id = backup_mgr.create_backup(&config_path).await?;

        // Detect format and apply change
        let path_str = config_path.to_string_lossy().to_string();
        let ext = std::path::Path::new(&path_str)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("jsonc") {
            Sandbag::apply_markdownlint_json(&config_path, &rule_match.rule_id)?;
        } else if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
            Sandbag::apply_markdownlint_yaml(&config_path, &rule_match.rule_id)?;
        } else {
            // Default to JSON behavior
            Sandbag::apply_markdownlint_json(&config_path, &rule_match.rule_id)?;
        }

        Ok(())
    }

    async fn apply_eslint_rule(&mut self, rule_match: &core::RuleMatch) -> Result<()> {
        // Discover config
        let config_path =
            Sandbag::find_eslint_config().unwrap_or_else(|| PathBuf::from(".eslintrc.json"));

        // Ensure file exists
        if !config_path.exists() {
            fs::write(&config_path, b"{\n  \"rules\": {}\n}\n")?;
        }

        // Backup
        let mut backup_mgr = config::backup::BackupManager::new();
        let _backup_id = backup_mgr.create_backup(&config_path).await?;

        let path_str = config_path.to_string_lossy().to_string();
        let ext = std::path::Path::new(&path_str)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("jsonc") {
            Sandbag::apply_eslint_json(&config_path, &rule_match.rule_id)?;
        } else if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
            Sandbag::apply_eslint_yaml(&config_path, &rule_match.rule_id)?;
        } else {
            Sandbag::apply_eslint_json(&config_path, &rule_match.rule_id)?;
        }

        Ok(())
    }

    fn find_markdownlint_config() -> Option<PathBuf> {
        let candidates = [
            ".markdownlint.json",
            ".markdownlint.jsonc",
            ".markdownlint.yaml",
            ".markdownlint.yml",
        ];
        for c in candidates {
            let p = Path::new(c);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    fn apply_markdownlint_json(path: &Path, rule_id: &str) -> Result<()> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
        if !v.is_object() {
            v = serde_json::json!({});
        }
        let obj = v.as_object_mut().unwrap();
        obj.insert(rule_id.to_string(), serde_json::Value::Bool(false));
        let new_content = serde_json::to_string_pretty(&v)? + "\n";
        fs::write(path, new_content)?;
        Ok(())
    }

    fn apply_markdownlint_yaml(path: &Path, rule_id: &str) -> Result<()> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_yaml::Value = serde_yaml::from_str(&content)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::default()));
        // Prefer top-level mapping of rule -> false
        let mut mapping = match v {
            serde_yaml::Value::Mapping(m) => m,
            _ => serde_yaml::Mapping::default(),
        };
        mapping.insert(
            serde_yaml::Value::String(rule_id.to_string()),
            serde_yaml::Value::Bool(false),
        );
        v = serde_yaml::Value::Mapping(mapping);
        let new_content = serde_yaml::to_string(&v)?;
        fs::write(path, new_content)?;
        Ok(())
    }

    fn find_eslint_config() -> Option<PathBuf> {
        let candidates = [
            ".eslintrc.json",
            ".eslintrc.jsonc",
            ".eslintrc.yaml",
            ".eslintrc.yml",
            "eslint.config.js", // not handled here
        ];
        for c in candidates {
            let p = Path::new(c);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    fn apply_eslint_json(path: &Path, rule_id: &str) -> Result<()> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_json::Value =
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
        if !v.is_object() {
            v = serde_json::json!({});
        }
        let obj = v.as_object_mut().unwrap();
        let rules = obj.entry("rules").or_insert_with(|| serde_json::json!({}));
        if rules.is_object() {
            rules.as_object_mut().unwrap().insert(
                rule_id.to_string(),
                serde_json::Value::String("off".to_string()),
            );
        } else {
            // Coerce to object
            let mut map = serde_json::Map::new();
            map.insert(
                rule_id.to_string(),
                serde_json::Value::String("off".to_string()),
            );
            *rules = serde_json::Value::Object(map);
        }
        let new_content = serde_json::to_string_pretty(&v)? + "\n";
        fs::write(path, new_content)?;
        Ok(())
    }

    fn apply_eslint_yaml(path: &Path, rule_id: &str) -> Result<()> {
        let content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_yaml::Value = serde_yaml::from_str(&content)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::default()));
        let mut root = match v {
            serde_yaml::Value::Mapping(m) => m,
            _ => serde_yaml::Mapping::default(),
        };
        // rules: { <rule_id>: off }
        let rules_key = serde_yaml::Value::String("rules".to_string());
        let rules_val = root
            .remove(&rules_key)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::default()));
        let mut rules_map = match rules_val {
            serde_yaml::Value::Mapping(m) => m,
            _ => serde_yaml::Mapping::default(),
        };
        rules_map.insert(
            serde_yaml::Value::String(rule_id.to_string()),
            serde_yaml::Value::String("off".to_string()),
        );
        root.insert(rules_key, serde_yaml::Value::Mapping(rules_map));
        v = serde_yaml::Value::Mapping(root);

        let new_content = serde_yaml::to_string(&v)?;
        fs::write(path, new_content)?;
        Ok(())
    }

    async fn apply_prettier_rule(&mut self, rule_match: &core::RuleMatch) -> Result<()> {
        let config_path =
            Sandbag::find_prettier_config().unwrap_or_else(|| PathBuf::from(".prettierrc.json"));

        if !config_path.exists() {
            fs::write(&config_path, b"{\n}\n")?;
        }

        let mut backup_mgr = config::backup::BackupManager::new();
        let _backup_id = backup_mgr.create_backup(&config_path).await?;

        let path_str = config_path.to_string_lossy().to_string();
        let ext = std::path::Path::new(&path_str)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("jsonc") {
            Sandbag::apply_prettier_json(&config_path, &rule_match.rule_id, &rule_match.context)?;
        } else if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
            Sandbag::apply_prettier_yaml(&config_path, &rule_match.rule_id, &rule_match.context)?;
        } else {
            Sandbag::apply_prettier_json(&config_path, &rule_match.rule_id, &rule_match.context)?;
        }

        Ok(())
    }

    fn find_prettier_config() -> Option<PathBuf> {
        let candidates = [
            ".prettierrc",
            ".prettierrc.json",
            ".prettierrc.jsonc",
            ".prettierrc.yaml",
            ".prettierrc.yml",
            "prettier.config.json",
            "prettier.config.yaml",
            "prettier.config.yml",
        ];
        for c in candidates {
            let p = Path::new(c);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    fn apply_prettier_json(
        path: &Path,
        rule_id: &str,
        extracted: &core::ExtractedContext,
    ) -> Result<()> {
        let file_content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_json::Value =
            serde_json::from_str(&file_content).unwrap_or(serde_json::json!({}));
        if !v.is_object() {
            v = serde_json::json!({});
        }
        let obj = v.as_object_mut().unwrap();

        // Heuristics: set semi=false if rule mentions semicolon, printWidth=120 if mentions width
        let message = extracted.message.as_deref().unwrap_or("").to_lowercase();
        if rule_id.contains("semi") || message.contains("semicolon") {
            obj.insert("semi".to_string(), serde_json::Value::Bool(false));
        } else if rule_id.to_lowercase().contains("printwidth") || message.contains("width") {
            obj.insert(
                "printWidth".to_string(),
                serde_json::Value::Number(serde_json::Number::from(120)),
            );
        } else {
            // Generic toggle: prefer setting a no-op placeholder to document action
            obj.insert(
                "__note".to_string(),
                serde_json::Value::String(format!("Handled rule {rule_id}")),
            );
        }

        let new_content = serde_json::to_string_pretty(&v)? + "\n";
        fs::write(path, new_content)?;
        Ok(())
    }

    fn apply_prettier_yaml(
        path: &Path,
        rule_id: &str,
        extracted: &core::ExtractedContext,
    ) -> Result<()> {
        let file_content = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
        let mut v: serde_yaml::Value = serde_yaml::from_str(&file_content)
            .unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::default()));
        let mut root = match v {
            serde_yaml::Value::Mapping(m) => m,
            _ => serde_yaml::Mapping::default(),
        };

        let message = extracted.message.as_deref().unwrap_or("").to_lowercase();
        if rule_id.contains("semi") || message.contains("semicolon") {
            root.insert(
                serde_yaml::Value::String("semi".to_string()),
                serde_yaml::Value::Bool(false),
            );
        } else if rule_id.to_lowercase().contains("printwidth") || message.contains("width") {
            root.insert(
                serde_yaml::Value::String("printWidth".to_string()),
                serde_yaml::Value::Number(120.into()),
            );
        } else {
            root.insert(
                serde_yaml::Value::String("__note".to_string()),
                serde_yaml::Value::String(format!("Handled rule {rule_id}")),
            );
        }

        v = serde_yaml::Value::Mapping(root);
        let new_content = serde_yaml::to_string(&v)?;
        fs::write(path, new_content)?;
        Ok(())
    }
}

impl Default for Sandbag {
    fn default() -> Self {
        Self::new()
    }
}
