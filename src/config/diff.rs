//! Minimal AST differential utilities for configuration trees.
//! This is a structural diff that reports key insertions/updates/removals and array edits.

use crate::config::{ASTNode, NodeType};

/// A single change in the AST
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstChange {
    /// Insert key at path with value
    Insert {
        /// Path of keys/indexes to the parent node
        path: Vec<String>,
        /// Key (or index as string) being inserted
        key: String,
        /// Value node to insert
        value: ASTNode,
    },
    /// Update key at path from old to new value
    Update {
        /// Path of keys/indexes to the parent node
        path: Vec<String>,
        /// Key (or index as string) being updated
        key: String,
        /// Old value
        old: ASTNode,
        /// New value
        new: ASTNode,
    },
    /// Remove key at path
    Remove {
        /// Path of keys/indexes to the parent node
        path: Vec<String>,
        /// Key (or index as string) being removed
        key: String,
    },
}

/// Compute a structural diff between two object-like AST nodes (maps and arrays simplified)
pub fn diff_ast(old: &ASTNode, new: &ASTNode) -> Vec<AstChange> {
    let mut changes = Vec::new();
    diff_node_recursive(old, new, &mut Vec::new(), &mut changes);
    changes
}

fn diff_node_recursive(
    old: &ASTNode,
    new: &ASTNode,
    path: &mut Vec<String>,
    out: &mut Vec<AstChange>,
) {
    match (&old.node_type, &new.node_type) {
        (NodeType::Object, NodeType::Object) => diff_object(old, new, path, out),
        (NodeType::Array, NodeType::Array) => diff_array(old, new, path, out),
        _ => {
            if old.value != new.value || old.node_type != new.node_type {
                // Represent replacement as update at current path
                out.push(AstChange::Update {
                    path: path.clone(),
                    key: String::new(),
                    old: old.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}

fn diff_object(old: &ASTNode, new: &ASTNode, path: &mut Vec<String>, out: &mut Vec<AstChange>) {
    let old_map = collect_object(old);
    let new_map = collect_object(new);

    // removals / updates
    for (k, old_v) in &old_map {
        if let Some(new_v) = new_map.get(k) {
            // recurse for nested changes
            path.push(k.clone());
            diff_node_recursive(old_v, new_v, path, out);
            path.pop();
        } else {
            out.push(AstChange::Remove {
                path: path.clone(),
                key: k.clone(),
            });
        }
    }

    // insertions
    for (k, new_v) in &new_map {
        if !old_map.contains_key(k) {
            out.push(AstChange::Insert {
                path: path.clone(),
                key: k.clone(),
                value: new_v.clone(),
            });
        }
    }
}

fn diff_array(old: &ASTNode, new: &ASTNode, path: &mut Vec<String>, out: &mut Vec<AstChange>) {
    // For arrays, treat by index; report updates for changed elements and inserts/removes for tail diffs.
    let len_old = old.children.len();
    let len_new = new.children.len();
    let min_len = len_old.min(len_new);

    for i in 0..min_len {
        path.push(i.to_string());
        diff_node_recursive(&old.children[i], &new.children[i], path, out);
        path.pop();
    }

    match len_old.cmp(&len_new) {
        std::cmp::Ordering::Greater => {
            for i in len_new..len_old {
                out.push(AstChange::Remove {
                    path: path.clone(),
                    key: i.to_string(),
                });
            }
        }
        std::cmp::Ordering::Less => {
            for i in len_old..len_new {
                out.push(AstChange::Insert {
                    path: path.clone(),
                    key: i.to_string(),
                    value: new.children[i].clone(),
                });
            }
        }
        std::cmp::Ordering::Equal => {}
    }
}

fn collect_object(node: &ASTNode) -> std::collections::HashMap<String, ASTNode> {
    let mut map = std::collections::HashMap::new();
    for kv in &node.children {
        if kv.children.len() == 2 {
            let key_node = &kv.children[0];
            let val_node = &kv.children[1];
            if let Some(ref k) = key_node.value {
                map.insert(k.clone(), val_node.clone());
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ASTNode, NodeType};

    fn kv(key: &str, value: &str) -> ASTNode {
        let k = ASTNode {
            node_type: NodeType::Key,
            value: Some(key.to_string()),
            children: vec![],
            metadata: std::collections::HashMap::default(),
        };
        let v = ASTNode {
            node_type: NodeType::Value,
            value: Some(value.to_string()),
            children: vec![],
            metadata: std::collections::HashMap::default(),
        };
        ASTNode {
            node_type: NodeType::KeyValue,
            value: None,
            children: vec![k, v],
            metadata: std::collections::HashMap::default(),
        }
    }

    #[test]
    fn test_diff_object_insert_update_remove() {
        let old = ASTNode {
            node_type: NodeType::Object,
            value: None,
            children: vec![kv("a", "1"), kv("b", "2")],
            metadata: std::collections::HashMap::default(),
        };
        let new = ASTNode {
            node_type: NodeType::Object,
            value: None,
            children: vec![kv("a", "10"), kv("c", "3")],
            metadata: std::collections::HashMap::default(),
        };

        let changes = diff_ast(&old, &new);
        assert!(changes
            .iter()
            .any(|c| matches!(c, AstChange::Update { key, .. } if key.is_empty())));
        assert!(changes
            .iter()
            .any(|c| matches!(c, AstChange::Remove { key, .. } if key == "b")));
        assert!(changes
            .iter()
            .any(|c| matches!(c, AstChange::Insert { key, .. } if key == "c")));
    }
}
