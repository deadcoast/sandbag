//! Configuration writer module

use crate::config::{ASTNode, ConfigAST, ConfigFormat, ConfigMetadata, NodeType, SerializeError};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::io::Write as _;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

/// Configuration writer trait
pub trait ConfigWriter {
    /// Write configuration to file
    fn write_config(
        &self,
        ast: &ConfigAST,
        path: &Path,
        format: ConfigFormat,
    ) -> Result<(), SerializeError>;

    /// Read configuration from file
    fn read_config(&self, path: &Path, format: ConfigFormat) -> Result<ConfigAST, SerializeError>;
}

/// File-based configuration writer
pub struct FileConfigWriter;

impl ConfigWriter for FileConfigWriter {
    #[allow(unused_variables)]
    fn write_config(
        &self,
        ast: &ConfigAST,
        path: &Path,
        format: ConfigFormat,
    ) -> Result<(), SerializeError> {
        let content = match format {
            ConfigFormat::Json => {
                let json = super::parser::ast_to_json(&ast.root);
                serde_json::to_string_pretty(&json)?
            }
            ConfigFormat::Yaml => {
                let yaml = super::parser::ast_to_yaml(&ast.root);
                serde_yaml::to_string(&yaml)?
            }
            ConfigFormat::Toml => {
                let toml = ast_to_toml(&ast.root);
                toml::to_string_pretty(&toml).map_err(SerializeError::TomlError)?
            }
            ConfigFormat::Ini | ConfigFormat::Custom(_) => {
                // Basic fallback: JSON text of AST
                let json = super::parser::ast_to_json(&ast.root);
                serde_json::to_string_pretty(&json)?
            }
        };

        let mut file = fs::File::create(path).map_err(SerializeError::IoError)?;
        file.write_all(content.as_bytes()).map_err(SerializeError::IoError)?;
        Ok(())
    }

    fn read_config(
        &self,
        path: &Path,
        format: ConfigFormat,
    ) -> Result<ConfigAST, SerializeError> {
        let content = fs::read_to_string(path).map_err(SerializeError::IoError)?;
        let metadata = ConfigMetadata {
            format: format.clone(),
            file_path: path.to_string_lossy().to_string(),
            last_modified: Some(DateTime::<Utc>::from(fs::metadata(path).map_err(SerializeError::IoError)?.modified().unwrap_or_else(|_| std::time::SystemTime::now()))),
            backup_count: 0,
        };

        let root = match format {
            ConfigFormat::Json => {
                let v: JsonValue = serde_json::from_str(&content)?;
                super::parser::json_to_ast(&v)
            }
            ConfigFormat::Yaml => {
                let v: YamlValue = serde_yaml::from_str(&content)?;
                super::parser::yaml_to_ast(&v)
            }
            ConfigFormat::Toml => {
                let v: TomlValue = toml::from_str(&content).map_err(SerializeError::TomlDeError)?;
                toml_to_ast(&v)
            }
            ConfigFormat::Ini | ConfigFormat::Custom(_) => ASTNode {
                node_type: NodeType::Value,
                value: Some(content),
                children: vec![],
                metadata: HashMap::new(),
            },
        };

        Ok(ConfigAST { format, root, metadata })
    }
}

fn toml_to_ast(value: &TomlValue) -> ASTNode {
    match value {
        TomlValue::Table(map) => {
            let mut children = Vec::new();
            for (k, v) in map {
                let key = ASTNode { node_type: NodeType::Key, value: Some(k.clone()), children: vec![], metadata: HashMap::new() };
                let val = toml_to_ast(v);
                children.push(ASTNode { node_type: NodeType::KeyValue, value: None, children: vec![key, val], metadata: HashMap::new() });
            }
            ASTNode { node_type: NodeType::Object, value: None, children, metadata: HashMap::new() }
        }
        TomlValue::Array(arr) => {
            let children = arr.iter().map(toml_to_ast).collect();
            ASTNode { node_type: NodeType::Array, value: None, children, metadata: HashMap::new() }
        }
        TomlValue::String(s) => ASTNode { node_type: NodeType::Value, value: Some(s.clone()), children: vec![], metadata: HashMap::new() },
        TomlValue::Integer(n) => ASTNode { node_type: NodeType::Value, value: Some(n.to_string()), children: vec![], metadata: HashMap::new() },
        TomlValue::Float(n) => ASTNode { node_type: NodeType::Value, value: Some(n.to_string()), children: vec![], metadata: HashMap::new() },
        TomlValue::Boolean(b) => ASTNode { node_type: NodeType::Value, value: Some(b.to_string()), children: vec![], metadata: HashMap::new() },
        TomlValue::Datetime(dt) => ASTNode { node_type: NodeType::Value, value: Some(dt.to_string()), children: vec![], metadata: HashMap::new() },
    }
}

fn ast_to_toml(node: &ASTNode) -> TomlValue {
    match node.node_type {
        NodeType::Object => {
            let mut table = toml::map::Map::new();
            for kv in &node.children {
                if kv.children.len() == 2 {
                    let key_node = &kv.children[0];
                    let val_node = &kv.children[1];
                    if let Some(ref k) = key_node.value {
                        table.insert(k.clone(), ast_to_toml(val_node));
                    }
                }
            }
            TomlValue::Table(table)
        }
        NodeType::Array => TomlValue::Array(node.children.iter().map(ast_to_toml).collect()),
        NodeType::Value => {
            if let Some(ref v) = node.value {
                if let Ok(b) = v.parse::<bool>() { return TomlValue::Boolean(b); }
                if let Ok(i) = v.parse::<i64>() { return TomlValue::Integer(i); }
                if let Ok(f) = v.parse::<f64>() { return TomlValue::Float(f); }
                TomlValue::String(v.clone())
            } else {
                TomlValue::String(String::new())
            }
        }
        _ => TomlValue::String(String::new()),
    }
}
