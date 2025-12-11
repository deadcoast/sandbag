//! Configuration management with AST-based parsing and safe operations

pub mod backup;
pub mod conflict_resolution;
pub mod manager;
pub mod parser;
pub mod writer;
pub mod diff;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration format types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    /// JSON configuration
    Json,
    /// YAML configuration
    Yaml,
    /// TOML configuration
    Toml,
    /// INI configuration
    Ini,
    /// Custom configuration, with format name
    Custom(String),
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration format
    pub format: ConfigFormat,
    /// Path to the configuration file
    pub file_path: String,
    /// Last modification timestamp
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of available backups
    pub backup_count: u32,
}

/// AST node for configuration representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASTNode {
    /// Node type
    pub node_type: NodeType,
    /// Optional node value
    pub value: Option<String>,
    /// Child nodes
    pub children: Vec<ASTNode>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of AST nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Root of the AST
    Root,
    /// Object/dictionary node
    Object,
    /// Array/list node
    Array,
    /// Key-value pair node
    KeyValue,
    /// Key node
    Key,
    /// Value node
    Value,
    /// Comment node
    Comment,
    /// Whitespace node
    Whitespace,
}

/// Configuration AST representation
#[derive(Debug)]
pub struct ConfigAST {
    /// Configuration format
    pub format: ConfigFormat,
    /// Root of the configuration AST
    pub root: ASTNode,
    /// Associated metadata
    pub metadata: ConfigMetadata,
}

/// Configuration parser trait
pub trait ConfigParser {
    /// Parse a configuration string into an AST
    fn parse(&self, content: &str) -> Result<ConfigAST, ParseError>;
    /// Serialize an AST into a configuration string
    fn serialize(&self, ast: &ConfigAST) -> Result<String, SerializeError>;
    /// Insert a rule into the configuration AST
    fn insert_rule(
        &self,
        ast: &mut ConfigAST,
        rule: &crate::core::ConfigEntry,
    ) -> Result<(), ModifyError>;
}

/// Parse error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// JSON parsing failed
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// YAML parsing failed
    #[error("Invalid YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
    /// TOML parsing failed
    #[error("Invalid TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),
    /// Unsupported configuration format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    /// IO error during parsing
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Serialization error types
#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    /// IO error during serialization/deserialization
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// JSON serialization failed
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// YAML serialization failed
    #[error("YAML serialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    /// TOML serialization failed
    #[error("TOML serialization error: {0}")]
    TomlError(#[from] toml::ser::Error),
    /// TOML deserialization failed
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),
    /// Unsupported configuration format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Configuration modification error types
#[derive(Debug, thiserror::Error)]
pub enum ModifyError {
    /// Rule already exists
    #[error("Rule already exists: {0}")]
    RuleExists(String),
    /// Invalid rule format
    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),
    /// Configuration structure error
    #[error("Configuration structure error: {0}")]
    StructureError(String),
    /// Wrapper for parse errors
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

/// Apply error types
#[derive(Debug, thiserror::Error)]
pub enum ApplyError {
    /// Validation failed
    #[error("Validation failed")]
    ValidationFailed,
    /// Underlying modification failed
    #[error("Modification failed: {0}")]
    ModificationFailed(#[from] ModifyError),
    /// Backup subsystem error
    #[error("Backup error: {0}")]
    BackupError(#[from] backup::BackupError),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Parse error wrapper
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    /// Serialize error wrapper
    #[error("Serialize error: {0}")]
    SerializeError(#[from] SerializeError),
}
