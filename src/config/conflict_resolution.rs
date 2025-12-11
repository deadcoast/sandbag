//! Configuration conflict resolution system

use crate::config::ConfigAST;
use crate::core::{ConfigAction, ConfigEntry, ConfigScope};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Conflict types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConflictType {
    /// Multiple rules with same ID but different actions
    ConflictingActions,
    /// Rules that override each other
    OverlappingRules,
    /// Inconsistent scopes for same rule
    ScopeConflict,
    /// Circular dependencies
    CircularDependency,
    /// Redundant rules
    RedundantRules,
}

/// Conflict information
#[derive(Debug, Clone)]
pub struct Conflict {
    /// Type of conflict encountered
    pub conflict_type: ConflictType,
    /// Involved rule identifiers
    pub rule_ids: Vec<String>,
    /// Conflict severity level
    pub severity: ConflictSeverity,
    /// Human-readable description of the conflict
    pub description: String,
    /// Suggested way to resolve the conflict
    pub suggested_resolution: ConflictResolution,
}

/// Conflict severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConflictSeverity {
    /// Informational
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Conflict resolution strategies
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Keep the most specific rule
    KeepMostSpecific,
    /// Keep the most recent rule
    KeepMostRecent,
    /// Merge conflicting rules
    MergeRules,
    /// Remove conflicting rules
    RemoveConflicting,
    /// Ask user for resolution
    UserDecision,
    /// Custom resolution
    Custom(String),
}

/// Conflict resolver
pub struct ConflictResolver {
    resolution_strategies: HashMap<ConflictType, ConflictResolution>,
    conflict_history: Vec<Conflict>,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        let mut resolution_strategies = HashMap::new();
        resolution_strategies.insert(
            ConflictType::ConflictingActions,
            ConflictResolution::KeepMostSpecific,
        );
        resolution_strategies.insert(
            ConflictType::OverlappingRules,
            ConflictResolution::MergeRules,
        );
        resolution_strategies.insert(
            ConflictType::ScopeConflict,
            ConflictResolution::KeepMostSpecific,
        );
        resolution_strategies.insert(
            ConflictType::CircularDependency,
            ConflictResolution::RemoveConflicting,
        );
        resolution_strategies.insert(
            ConflictType::RedundantRules,
            ConflictResolution::RemoveConflicting,
        );

        Self {
            resolution_strategies,
            conflict_history: Vec::new(),
        }
    }

    /// Analyze configuration for conflicts
    pub fn analyze_conflicts(&mut self, ast: &ConfigAST) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        // Extract all rules from AST
        let rules = self.extract_rules_from_ast(ast);

        // Check for conflicting actions
        conflicts.extend(self.check_conflicting_actions(&rules));

        // Check for overlapping rules
        conflicts.extend(self.check_overlapping_rules(&rules));

        // Check for scope conflicts
        conflicts.extend(self.check_scope_conflicts(&rules));

        // Check for circular dependencies
        conflicts.extend(self.check_circular_dependencies(&rules));

        // Check for redundant rules
        conflicts.extend(self.check_redundant_rules(&rules));

        // Store conflicts in history
        self.conflict_history.extend(conflicts.clone());

        conflicts
    }

    /// Extract rules from AST
    #[allow(clippy::unused_self)]
    fn extract_rules_from_ast(&self, _ast: &ConfigAST) -> Vec<ConfigEntry> {
        // TODO: Implement AST rule extraction
        vec![]
    }

    /// Check for conflicting actions on same rule
    #[allow(clippy::unused_self)]
    fn check_conflicting_actions(&self, rules: &[ConfigEntry]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut rule_actions: HashMap<String, Vec<ConfigAction>> = HashMap::new();

        for rule in rules {
            rule_actions
                .entry(rule.rule_id.clone())
                .or_default()
                .push(rule.action.clone());
        }

        for (rule_id, actions) in rule_actions {
            if actions.len() > 1 {
                let unique_actions: HashSet<ConfigAction> = actions.into_iter().collect();
                if unique_actions.len() > 1 {
                    conflicts.push(Conflict {
                        conflict_type: ConflictType::ConflictingActions,
                        rule_ids: vec![rule_id.clone()],
                        severity: ConflictSeverity::Error,
                        description: format!("Multiple actions for rule: {rule_id}"),
                        suggested_resolution: ConflictResolution::KeepMostSpecific,
                    });
                }
            }
        }

        conflicts
    }

    /// Check for overlapping rules
    fn check_overlapping_rules(&self, rules: &[ConfigEntry]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for i in 0..rules.len() {
            for j in i + 1..rules.len() {
                if self.rules_overlap(&rules[i], &rules[j]) {
                    conflicts.push(Conflict {
                        conflict_type: ConflictType::OverlappingRules,
                        rule_ids: vec![rules[i].rule_id.clone(), rules[j].rule_id.clone()],
                        severity: ConflictSeverity::Warning,
                        description: format!(
                            "Rules {} and {} overlap",
                            rules[i].rule_id, rules[j].rule_id
                        ),
                        suggested_resolution: ConflictResolution::MergeRules,
                    });
                }
            }
        }

        conflicts
    }

    /// Check if two rules overlap
    #[allow(clippy::unused_self, clippy::match_same_arms)]
    fn rules_overlap(&self, rule1: &ConfigEntry, rule2: &ConfigEntry) -> bool {
        // Check if rules have overlapping scopes
        match (&rule1.scope, &rule2.scope) {
            (ConfigScope::Global, ConfigScope::Global) => true,
            (ConfigScope::FileSpecific, ConfigScope::FileSpecific) => {
                // TODO: Check if files overlap
                false
            }
            (ConfigScope::Directory, ConfigScope::Directory) => {
                // TODO: Check if directories overlap
                false
            }
            _ => false,
        }
    }

    /// Check for scope conflicts
    #[allow(clippy::unused_self)]
    fn check_scope_conflicts(&self, rules: &[ConfigEntry]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut rule_scopes: HashMap<String, Vec<ConfigScope>> = HashMap::new();

        for rule in rules {
            rule_scopes
                .entry(rule.rule_id.clone())
                .or_default()
                .push(rule.scope.clone());
        }

        for (rule_id, scopes) in rule_scopes {
            if scopes.len() > 1 {
                let has_global = scopes.iter().any(|s| matches!(s, ConfigScope::Global));
                let has_specific = scopes
                    .iter()
                    .any(|s| matches!(s, ConfigScope::FileSpecific | ConfigScope::Directory));

                if has_global && has_specific {
                    conflicts.push(Conflict {
                        conflict_type: ConflictType::ScopeConflict,
                        rule_ids: vec![rule_id.clone()],
                        severity: ConflictSeverity::Warning,
                        description: format!("Rule {rule_id} has both global and specific scopes"),
                        suggested_resolution: ConflictResolution::KeepMostSpecific,
                    });
                }
            }
        }

        conflicts
    }

    /// Check for circular dependencies
    #[allow(clippy::unused_self)]
    fn check_circular_dependencies(&self, _rules: &[ConfigEntry]) -> Vec<Conflict> {
        // TODO: Implement circular dependency detection
        vec![]
    }

    /// Check for redundant rules
    #[allow(clippy::unused_self)]
    fn check_redundant_rules(&self, rules: &[ConfigEntry]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut rule_groups: HashMap<String, Vec<&ConfigEntry>> = HashMap::new();

        for rule in rules {
            rule_groups
                .entry(rule.rule_id.clone())
                .or_default()
                .push(rule);
        }

        for (rule_id, rule_list) in rule_groups {
            if rule_list.len() > 1 {
                // Check if rules are identical
                let first_rule = &rule_list[0];
                let all_identical = rule_list
                    .iter()
                    .all(|r| r.action == first_rule.action && r.scope == first_rule.scope);

                if all_identical {
                    conflicts.push(Conflict {
                        conflict_type: ConflictType::RedundantRules,
                        rule_ids: vec![rule_id.clone()],
                        severity: ConflictSeverity::Info,
                        description: format!("Multiple identical rules for {rule_id}"),
                        suggested_resolution: ConflictResolution::RemoveConflicting,
                    });
                }
            }
        }

        conflicts
    }

    /// Resolve conflicts automatically
    pub fn resolve_conflicts(
        &self,
        conflicts: &[Conflict],
        ast: &mut ConfigAST,
    ) -> Result<Vec<Conflict>> {
        let mut unresolved_conflicts = Vec::new();

        for conflict in conflicts {
            let strategy = self
                .resolution_strategies
                .get(&conflict.conflict_type)
                .unwrap_or(&ConflictResolution::UserDecision);

            match strategy {
                ConflictResolution::KeepMostSpecific => {
                    self.resolve_keep_most_specific(conflict, ast)?;
                }
                ConflictResolution::MergeRules => {
                    self.resolve_merge_rules(conflict, ast)?;
                }
                ConflictResolution::RemoveConflicting => {
                    self.resolve_remove_conflicting(conflict, ast)?;
                }
                _ => {
                    unresolved_conflicts.push(conflict.clone());
                }
            }
        }

        Ok(unresolved_conflicts)
    }

    /// Resolve conflict by keeping most specific rule
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn resolve_keep_most_specific(&self, _conflict: &Conflict, _ast: &mut ConfigAST) -> Result<()> {
        // TODO: Implement keep most specific resolution
        Ok(())
    }

    /// Resolve conflict by merging rules
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn resolve_merge_rules(&self, _conflict: &Conflict, _ast: &mut ConfigAST) -> Result<()> {
        // TODO: Implement rule merging
        Ok(())
    }

    /// Resolve conflict by removing conflicting rules
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn resolve_remove_conflicting(&self, _conflict: &Conflict, _ast: &mut ConfigAST) -> Result<()> {
        // TODO: Implement conflict removal
        Ok(())
    }

    /// Get conflict history
    pub fn get_conflict_history(&self) -> &[Conflict] {
        &self.conflict_history
    }

    /// Clear conflict history
    pub fn clear_conflict_history(&mut self) {
        self.conflict_history.clear();
    }

    /// Set resolution strategy for conflict type
    pub fn set_resolution_strategy(
        &mut self,
        conflict_type: ConflictType,
        strategy: ConflictResolution,
    ) {
        self.resolution_strategies.insert(conflict_type, strategy);
    }

    /// Validate configuration after conflict resolution
    pub fn validate_resolution(&self, ast: &ConfigAST) -> Result<bool> {
        // Re-analyze for conflicts
        let mut temp_resolver = ConflictResolver::new();
        let conflicts = temp_resolver.analyze_conflicts(ast);

        // Check if any critical conflicts remain
        let has_critical_conflicts = conflicts
            .iter()
            .any(|c| c.severity == ConflictSeverity::Critical);

        Ok(!has_critical_conflicts)
    }
}

/// Conflict resolution report
#[derive(Debug, Clone)]
pub struct ConflictReport {
    /// Total number of conflicts found
    pub total_conflicts: usize,
    /// Number of conflicts resolved
    pub resolved_conflicts: usize,
    /// Number of conflicts remaining
    pub unresolved_conflicts: usize,
    /// Conflicts organized by severity
    pub conflicts_by_severity: HashMap<ConflictSeverity, usize>,
    /// Total time spent resolving
    pub resolution_time: std::time::Duration,
}

impl ConflictReport {
    /// Create a new conflict report
    pub fn new() -> Self {
        Self {
            total_conflicts: 0,
            resolved_conflicts: 0,
            unresolved_conflicts: 0,
            conflicts_by_severity: HashMap::new(),
            resolution_time: std::time::Duration::ZERO,
        }
    }

    /// Add conflict to report
    pub fn add_conflict(&mut self, conflict: &Conflict) {
        self.total_conflicts += 1;
        *self
            .conflicts_by_severity
            .entry(conflict.severity.clone())
            .or_insert(0) += 1;
    }

    /// Mark conflict as resolved
    pub fn mark_resolved(&mut self) {
        self.resolved_conflicts += 1;
    }

    /// Mark conflict as unresolved
    pub fn mark_unresolved(&mut self) {
        self.unresolved_conflicts += 1;
    }

    /// Set resolution time
    pub fn set_resolution_time(&mut self, duration: std::time::Duration) {
        self.resolution_time = duration;
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConflictReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigFormat;

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver::new();
        assert!(!resolver.resolution_strategies.is_empty());
    }

    #[test]
    fn test_conflict_analysis() {
        let mut resolver = ConflictResolver::new();
        let ast = ConfigAST {
            format: ConfigFormat::Json,
            root: crate::config::ASTNode {
                node_type: crate::config::NodeType::Root,
                value: None,
                children: Vec::new(),
                metadata: HashMap::new(),
            },
            metadata: crate::config::ConfigMetadata {
                format: ConfigFormat::Json,
                file_path: "test.json".to_string(),
                last_modified: None,
                backup_count: 0,
            },
        };

        let conflicts = resolver.analyze_conflicts(&ast);
        assert!(conflicts.is_empty()); // Empty AST should have no conflicts
    }

    #[test]
    fn test_conflict_report() {
        let mut report = ConflictReport::new();
        let conflict = Conflict {
            conflict_type: ConflictType::ConflictingActions,
            rule_ids: vec!["MD033".to_string()],
            severity: ConflictSeverity::Error,
            description: "Test conflict".to_string(),
            suggested_resolution: ConflictResolution::KeepMostSpecific,
        };

        report.add_conflict(&conflict);
        assert_eq!(report.total_conflicts, 1);
        assert_eq!(
            report.conflicts_by_severity.get(&ConflictSeverity::Error),
            Some(&1)
        );
    }
}
