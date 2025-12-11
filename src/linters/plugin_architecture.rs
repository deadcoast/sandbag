//! Plugin architecture for multi-linter support

use crate::core::RuleMatch;
use crate::linters::LinterHandler;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version string
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Short description
    pub description: String,
    /// Associated linter name
    pub linter_name: String,
    /// Supported configuration formats
    pub supported_formats: Vec<String>,
    /// Required dependencies
    pub dependencies: Vec<String>,
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Whether the plugin is enabled
    pub enabled: bool,
    /// Plugin priority for ordering
    pub priority: u32,
    /// Arbitrary settings map
    pub settings: HashMap<String, serde_json::Value>,
}

/// Plugin registry for managing linter plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn LinterPlugin>>,
    metadata: HashMap<String, PluginMetadata>,
    configs: HashMap<String, PluginConfig>,
}

/// Plugin trait for linter implementations
pub trait LinterPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;

    /// Get the linter handler
    fn get_handler(&self) -> Box<dyn LinterHandler>;

    /// Check if plugin can handle the given input
    fn can_handle(&self, input: &str) -> bool;

    /// Get plugin configuration
    fn get_config(&self) -> &PluginConfig;

    /// Set plugin configuration
    fn set_config(&mut self, config: PluginConfig) -> Result<()>;
}

/// Built-in markdownlint plugin
pub struct MarkdownlintPlugin {
    handler: crate::linters::markdownlint::MarkdownlintHandler,
    config: PluginConfig,
    metadata: PluginMetadata,
}

impl MarkdownlintPlugin {
    /// Create a new markdownlint plugin
    pub fn new() -> Self {
        Self {
            handler: crate::linters::markdownlint::MarkdownlintHandler::new(),
            config: PluginConfig {
                enabled: true,
                priority: 100,
                settings: HashMap::new(),
            },
            metadata: PluginMetadata {
                name: "markdownlint".to_string(),
                version: "1.0.0".to_string(),
                author: "Sandbag Team".to_string(),
                description: "Markdownlint support plugin".to_string(),
                linter_name: "markdownlint".to_string(),
                supported_formats: vec!["json".to_string(), "yaml".to_string(), "yml".to_string()],
                dependencies: Vec::new(),
            },
        }
    }
}

impl LinterPlugin for MarkdownlintPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn initialize(&mut self, config: &PluginConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }

    fn get_handler(&self) -> Box<dyn LinterHandler> {
        Box::new(self.handler.clone())
    }

    fn can_handle(&self, input: &str) -> bool {
        input.contains("MD") || input.contains("markdownlint")
    }

    fn get_config(&self) -> &PluginConfig {
        &self.config
    }

    fn set_config(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    registry: PluginRegistry,
    #[allow(dead_code)]
    plugin_paths: Vec<String>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let mut registry = PluginRegistry {
            plugins: HashMap::new(),
            metadata: HashMap::new(),
            configs: HashMap::new(),
        };

        // Register built-in plugins
        let markdownlint_plugin = Box::new(MarkdownlintPlugin::new());
        let metadata = markdownlint_plugin.metadata();
        let name = metadata.name.clone();

        registry.plugins.insert(name.clone(), markdownlint_plugin);
        registry.metadata.insert(name.clone(), metadata);
        registry.configs.insert(
            name,
            PluginConfig {
                enabled: true,
                priority: 100,
                settings: HashMap::new(),
            },
        );

        Self {
            registry,
            plugin_paths: Vec::new(),
        }
    }

    /// Load plugins from directory
    pub fn load_plugins_from_directory(&mut self, _path: &Path) -> Result<()> {
        // TODO: Implement dynamic plugin loading
        // This would involve scanning for .so/.dll files and loading them
        Ok(())
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, name: String, plugin: Box<dyn LinterPlugin>) -> Result<()> {
        let metadata = plugin.metadata();
        let config = plugin.get_config().clone();

        self.registry.plugins.insert(name.clone(), plugin);
        self.registry.metadata.insert(name.clone(), metadata);
        self.registry.configs.insert(name, config);

        Ok(())
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn LinterPlugin> {
        self.registry
            .plugins
            .get(name)
            .map(std::convert::AsRef::as_ref)
    }

    /// Get all plugin names
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.registry.plugins.keys().cloned().collect()
    }

    /// Get enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<&dyn LinterPlugin> {
        self.registry
            .plugins
            .values()
            .filter(|p| p.get_config().enabled)
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Extract rules using all enabled plugins
    pub fn extract_rules(&self, input: &str) -> Vec<RuleMatch> {
        let mut all_matches = Vec::new();

        for plugin in self.get_enabled_plugins() {
            if plugin.can_handle(input) {
                let handler = plugin.get_handler();
                if let Some(rule_match) = handler.extract_rules(input).first().cloned() {
                    all_matches.push(rule_match);
                }
            }
        }

        // Sort by plugin priority
        all_matches.sort_by(|a, b| {
            let a_priority = self.get_plugin_priority(&a.linter);
            let b_priority = self.get_plugin_priority(&b.linter);
            b_priority.cmp(&a_priority) // Higher priority first
        });

        all_matches
    }

    /// Get plugin priority
    fn get_plugin_priority(&self, linter_name: &str) -> u32 {
        for plugin in self.get_enabled_plugins() {
            if plugin.metadata().linter_name == linter_name {
                return plugin.get_config().priority;
            }
        }
        0
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.registry.configs.get_mut(name) {
            config.enabled = true;
        }
        if let Some(plugin) = self.registry.plugins.get_mut(name) {
            let mut config = plugin.get_config().clone();
            config.enabled = true;
            plugin.set_config(config)?;
        }
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.registry.configs.get_mut(name) {
            config.enabled = false;
        }
        if let Some(plugin) = self.registry.plugins.get_mut(name) {
            let mut config = plugin.get_config().clone();
            config.enabled = false;
            plugin.set_config(config)?;
        }
        Ok(())
    }

    /// Update plugin configuration
    pub fn update_plugin_config(&mut self, name: &str, config: PluginConfig) -> Result<()> {
        if let Some(plugin) = self.registry.plugins.get_mut(name) {
            plugin.set_config(config.clone())?;
            self.registry.configs.insert(name.to_string(), config);
        }
        Ok(())
    }

    /// Get plugin metadata
    pub fn get_plugin_metadata(&self, name: &str) -> Option<&PluginMetadata> {
        self.registry.metadata.get(name)
    }

    /// List all plugins with their status
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();

        for (name, metadata) in &self.registry.metadata {
            let default_config = PluginConfig {
                enabled: false,
                priority: 0,
                settings: HashMap::new(),
            };
            let config = self.registry.configs.get(name).unwrap_or(&default_config);

            plugins.push(PluginInfo {
                name: name.clone(),
                metadata: metadata.clone(),
                config: config.clone(),
            });
        }

        plugins.sort_by(|a, b| b.config.priority.cmp(&a.config.priority));
        plugins
    }
}

impl Default for MarkdownlintPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin information for listing
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Current plugin configuration
    pub config: PluginConfig,
}

/// Plugin factory for creating plugins
pub trait PluginFactory {
    /// Create a new plugin instance
    fn create_plugin(&self) -> Box<dyn LinterPlugin>;

    /// Get factory metadata
    fn metadata(&self) -> PluginMetadata;
}

/// Built-in plugin factory
pub struct BuiltinPluginFactory;

impl PluginFactory for BuiltinPluginFactory {
    fn create_plugin(&self) -> Box<dyn LinterPlugin> {
        Box::new(MarkdownlintPlugin::new())
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "markdownlint".to_string(),
            version: "1.0.0".to_string(),
            author: "Sandbag Team".to_string(),
            description: "Built-in markdownlint plugin".to_string(),
            linter_name: "markdownlint".to_string(),
            supported_formats: vec!["json".to_string(), "yaml".to_string()],
            dependencies: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        let plugin_names = manager.get_plugin_names();
        assert!(plugin_names.contains(&"markdownlint".to_string()));
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(MarkdownlintPlugin::new());

        manager
            .register_plugin("test_plugin".to_string(), plugin)
            .unwrap();
        assert!(manager
            .get_plugin_names()
            .contains(&"test_plugin".to_string()));
    }

    #[test]
    fn test_rule_extraction_with_plugins() {
        let manager = PluginManager::new();
        let input = "MD033: No inline HTML";
        let matches = manager.extract_rules(input);

        assert!(!matches.is_empty());
        assert_eq!(matches[0].rule_id, "MD033");
    }

    #[test]
    fn test_plugin_enable_disable() {
        let mut manager = PluginManager::new();

        manager.disable_plugin("markdownlint").unwrap();
        let enabled_plugins = manager.get_enabled_plugins();
        assert!(enabled_plugins.is_empty());

        manager.enable_plugin("markdownlint").unwrap();
        let enabled_plugins = manager.get_enabled_plugins();
        assert!(!enabled_plugins.is_empty());
    }
}
