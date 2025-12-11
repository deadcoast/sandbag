//! Configuration parser module

use crate::config::{ASTNode, ConfigAST, ConfigFormat, ConfigMetadata, NodeType, ParseError};
use anyhow::Result;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;

/// Configuration parser trait
pub trait ConfigParser {
    /// Parse configuration from string
    fn parse(&self, content: &str) -> Result<ConfigAST, ParseError>;

    /// Serialize configuration to string
    fn serialize(&self, ast: &ConfigAST) -> Result<String, ParseError>;

    /// Get the format this parser handles
    fn get_format(&self) -> ConfigFormat;
}

/// JSON configuration parser
pub struct JsonParser;

impl super::ConfigParser for JsonParser {
    fn parse(&self, content: &str) -> Result<ConfigAST, super::ParseError> {
        let json: JsonValue = serde_json::from_str(content)?;
        let root = json_to_ast(&json);
        Ok(ConfigAST {
            format: ConfigFormat::Json,
            root,
            metadata: ConfigMetadata {
                format: ConfigFormat::Json,
                file_path: String::new(),
                last_modified: None,
                backup_count: 0,
            },
        })
    }

    fn serialize(&self, ast: &ConfigAST) -> Result<String, super::SerializeError> {
        let json = ast_to_json(&ast.root);
        let s = serde_json::to_string_pretty(&json)?;
        Ok(s)
    }

    fn insert_rule(
        &self,
        ast: &mut ConfigAST,
        rule: &crate::core::ConfigEntry,
    ) -> Result<(), super::ModifyError> {
        // Convert AST to JSON, update, then convert back
        let mut json = ast_to_json(&ast.root);

        // Determine insertion path and value
        let path = rule
            .metadata
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let value = if let Some(v) = rule.metadata.get("value") {
            v.clone()
        } else {
            // Default by action for common linters
            match rule.action {
                crate::core::ConfigAction::Disable => JsonValue::Bool(false),
                crate::core::ConfigAction::Ignore => JsonValue::String("off".to_string()),
                crate::core::ConfigAction::Enable => JsonValue::String("on".to_string()),
                crate::core::ConfigAction::Modify => JsonValue::Null,
            }
        };

        set_json_path(&mut json, path, &rule.rule_id, value);

        ast.root = json_to_ast(&json);
        Ok(())
    }
}

/// YAML configuration parser
pub struct YamlParser;

impl super::ConfigParser for YamlParser {
    fn parse(&self, content: &str) -> Result<ConfigAST, super::ParseError> {
        let yaml: YamlValue = serde_yaml::from_str(content)?;
        let root = yaml_to_ast(&yaml);
        Ok(ConfigAST {
            format: ConfigFormat::Yaml,
            root,
            metadata: ConfigMetadata {
                format: ConfigFormat::Yaml,
                file_path: String::new(),
                last_modified: None,
                backup_count: 0,
            },
        })
    }

    fn serialize(&self, ast: &ConfigAST) -> Result<String, super::SerializeError> {
        let yaml = ast_to_yaml(&ast.root);
        let s = serde_yaml::to_string(&yaml)?;
        Ok(s)
    }

    fn insert_rule(
        &self,
        ast: &mut ConfigAST,
        rule: &crate::core::ConfigEntry,
    ) -> Result<(), super::ModifyError> {
        // Convert AST to YAML value, update, then convert back
        let mut yaml = ast_to_yaml(&ast.root);

        // Determine path
        let path = rule
            .metadata
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let value = if let Some(v) = rule.metadata.get("value") {
            json_to_yaml(v)
        } else {
            match rule.action {
                crate::core::ConfigAction::Disable => YamlValue::Bool(false),
                crate::core::ConfigAction::Ignore => YamlValue::String("off".to_string()),
                crate::core::ConfigAction::Enable => YamlValue::String("on".to_string()),
                crate::core::ConfigAction::Modify => YamlValue::Null,
            }
        };

        set_yaml_path(&mut yaml, path, &rule.rule_id, value);

        ast.root = yaml_to_ast(&yaml);
        Ok(())
    }
}

/// Convert a `serde_json::Value` tree to `ASTNode` representation.
pub fn json_to_ast(value: &JsonValue) -> ASTNode {
    match value {
        JsonValue::Object(map) => {
            let mut children = Vec::new();
            for (k, v) in map {
                let key = ASTNode {
                    node_type: NodeType::Key,
                    value: Some(k.clone()),
                    children: vec![],
                    metadata: HashMap::new(),
                };
                let val = json_to_ast(v);
                children.push(ASTNode {
                    node_type: NodeType::KeyValue,
                    value: None,
                    children: vec![key, val],
                    metadata: HashMap::new(),
                });
            }
            ASTNode {
                node_type: NodeType::Object,
                value: None,
                children,
                metadata: HashMap::new(),
            }
        }
        JsonValue::Array(arr) => {
            let children = arr.iter().map(json_to_ast).collect();
            ASTNode {
                node_type: NodeType::Array,
                value: None,
                children,
                metadata: HashMap::new(),
            }
        }
        JsonValue::String(s) => ASTNode {
            node_type: NodeType::Value,
            value: Some(s.clone()),
            children: vec![],
            metadata: HashMap::new(),
        },
        JsonValue::Number(n) => ASTNode {
            node_type: NodeType::Value,
            value: Some(n.to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
        JsonValue::Bool(b) => ASTNode {
            node_type: NodeType::Value,
            value: Some(b.to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
        JsonValue::Null => ASTNode {
            node_type: NodeType::Value,
            value: Some("null".to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
    }
}

/// Convert an `ASTNode` tree to `serde_json::Value`.
pub fn ast_to_json(node: &ASTNode) -> JsonValue {
    match node.node_type {
        NodeType::Object => {
            let mut map = serde_json::Map::new();
            for kv in &node.children {
                if kv.node_type == NodeType::KeyValue && kv.children.len() == 2 {
                    let key_node = &kv.children[0];
                    let val_node = &kv.children[1];
                    if let Some(ref k) = key_node.value {
                        map.insert(k.clone(), ast_to_json(val_node));
                    }
                }
            }
            JsonValue::Object(map)
        }
        NodeType::Array => JsonValue::Array(node.children.iter().map(ast_to_json).collect()),
        NodeType::Value => {
            if let Some(ref v) = node.value {
                // Best-effort: try boolean/number/null, else string
                if v == "null" {
                    return JsonValue::Null;
                }
                if let Ok(b) = v.parse::<bool>() {
                    return JsonValue::Bool(b);
                }
                if let Ok(n) = v.parse::<f64>() {
                    return JsonValue::from(n);
                }
                JsonValue::String(v.clone())
            } else {
                JsonValue::Null
            }
        }
        NodeType::KeyValue
        | NodeType::Key
        | NodeType::Comment
        | NodeType::Whitespace
        | NodeType::Root => {
            // Represent structural wrappers as object/array by recursing into children if any
            if node.children.is_empty() {
                JsonValue::Null
            } else {
                ast_to_json(&node.children[0])
            }
        }
    }
}

/// Convert a `serde_yaml::Value` tree to `ASTNode` representation.
pub fn yaml_to_ast(value: &YamlValue) -> ASTNode {
    match value {
        YamlValue::Mapping(map) => {
            let mut children = Vec::new();
            for (k, v) in map {
                let key_str = if let Some(s) = k.as_str() {
                    s.to_string()
                } else {
                    format!("{k:?}")
                };
                let key = ASTNode {
                    node_type: NodeType::Key,
                    value: Some(key_str),
                    children: vec![],
                    metadata: HashMap::new(),
                };
                let val = yaml_to_ast(v);
                children.push(ASTNode {
                    node_type: NodeType::KeyValue,
                    value: None,
                    children: vec![key, val],
                    metadata: HashMap::new(),
                });
            }
            ASTNode {
                node_type: NodeType::Object,
                value: None,
                children,
                metadata: HashMap::new(),
            }
        }
        YamlValue::Sequence(seq) => {
            let children = seq.iter().map(yaml_to_ast).collect();
            ASTNode {
                node_type: NodeType::Array,
                value: None,
                children,
                metadata: HashMap::new(),
            }
        }
        YamlValue::String(s) => ASTNode {
            node_type: NodeType::Value,
            value: Some(s.clone()),
            children: vec![],
            metadata: HashMap::new(),
        },
        YamlValue::Number(n) => ASTNode {
            node_type: NodeType::Value,
            value: Some(n.to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
        YamlValue::Bool(b) => ASTNode {
            node_type: NodeType::Value,
            value: Some(b.to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
        YamlValue::Null => ASTNode {
            node_type: NodeType::Value,
            value: Some("null".to_string()),
            children: vec![],
            metadata: HashMap::new(),
        },
        YamlValue::Tagged(_) => ASTNode {
            node_type: NodeType::Value,
            value: Some(format!("{value:?}")),
            children: vec![],
            metadata: HashMap::new(),
        },
    }
}

/// Convert an `ASTNode` tree to `serde_yaml::Value`.
pub fn ast_to_yaml(node: &ASTNode) -> YamlValue {
    match node.node_type {
        NodeType::Object => {
            let mut map = serde_yaml::Mapping::new();
            for kv in &node.children {
                if kv.node_type == NodeType::KeyValue && kv.children.len() == 2 {
                    let key_node = &kv.children[0];
                    let val_node = &kv.children[1];
                    if let Some(ref k) = key_node.value {
                        map.insert(YamlValue::String(k.clone()), ast_to_yaml(val_node));
                    }
                }
            }
            YamlValue::Mapping(map)
        }
        NodeType::Array => YamlValue::Sequence(node.children.iter().map(ast_to_yaml).collect()),
        NodeType::Value => {
            if let Some(ref v) = node.value {
                if v == "null" {
                    return YamlValue::Null;
                }
                if let Ok(b) = v.parse::<bool>() {
                    return YamlValue::Bool(b);
                }
                if let Ok(n) = v.parse::<i64>() {
                    return YamlValue::Number(n.into());
                }
                YamlValue::String(v.clone())
            } else {
                YamlValue::Null
            }
        }
        _ => YamlValue::Null,
    }
}

/// Insert a value under the dotted path into a JSON object, then at terminal level insert key -> value
fn set_json_path(root: &mut JsonValue, path: &str, key: &str, value: JsonValue) {
    // Ensure root is object
    if !root.is_object() {
        *root = JsonValue::Object(serde_json::Map::new());
    }
    let mut current = root;
    if !path.is_empty() {
        for seg in path.split('.') {
            if seg.is_empty() {
                continue;
            }
            let next = current
                .as_object_mut()
                .unwrap()
                .entry(seg.to_string())
                .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
            if !next.is_object() {
                *next = JsonValue::Object(serde_json::Map::new());
            }
            current = next;
        }
    }
    if let Some(obj) = current.as_object_mut() {
        obj.insert(key.to_string(), value);
    }
}

/// Insert a value under the dotted path into a YAML mapping, then at terminal level insert key -> value
fn set_yaml_path(root: &mut YamlValue, path: &str, key: &str, value: YamlValue) {
    if !matches!(root, YamlValue::Mapping(_)) {
        *root = YamlValue::Mapping(serde_yaml::Mapping::default());
    }
    let mut current = root;
    if !path.is_empty() {
        for seg in path.split('.') {
            if seg.is_empty() {
                continue;
            }
            // descend into mapping
            let next = if let YamlValue::Mapping(map) = current {
                let k = YamlValue::String(seg.to_string());
                map.entry(k)
                    .or_insert(YamlValue::Mapping(serde_yaml::Mapping::default()))
            } else {
                *current = YamlValue::Mapping(serde_yaml::Mapping::default());
                match current {
                    YamlValue::Mapping(map) => {
                        let k = YamlValue::String(seg.to_string());
                        map.entry(k)
                            .or_insert(YamlValue::Mapping(serde_yaml::Mapping::default()))
                    }
                    _ => unreachable!(),
                }
            };
            current = next;
        }
    }
    if let YamlValue::Mapping(map) = current {
        map.insert(YamlValue::String(key.to_string()), value);
    }
}

/// Convert a JSON value to a YAML value (best-effort)
fn json_to_yaml(v: &JsonValue) -> YamlValue {
    match v {
        JsonValue::Null => YamlValue::Null,
        JsonValue::Bool(b) => YamlValue::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                YamlValue::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                if let Ok(i) = i64::try_from(u) {
                    YamlValue::Number(i.into())
                } else {
                    YamlValue::String(u.to_string())
                }
            } else if let Some(f) = n.as_f64() {
                YamlValue::Number(serde_yaml::Number::from(f))
            } else {
                YamlValue::Null
            }
        }
        JsonValue::String(s) => YamlValue::String(s.clone()),
        JsonValue::Array(arr) => YamlValue::Sequence(arr.iter().map(json_to_yaml).collect()),
        JsonValue::Object(map) => {
            let mut m = serde_yaml::Mapping::new();
            for (k, val) in map {
                m.insert(YamlValue::String(k.clone()), json_to_yaml(val));
            }
            YamlValue::Mapping(m)
        }
    }
}
