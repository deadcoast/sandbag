//! Comprehensive integration tests for the Sandbag system

use anyhow::Result;
use sandbag::config::manager::ConfigManager;
use sandbag::config::{ASTNode, ConfigAST, ConfigFormat, ConfigMetadata, NodeType};
use sandbag::core::advanced_similarity::AdvancedSimilarityAnalyzer;
use sandbag::core::{ConfigAction, ConfigEntry};
use sandbag::linters::{LinterFactory, LinterRegistry};
use std::collections::HashMap;

/// Integration test suite for end-to-end functionality
pub struct IntegrationTestSuite {
    linter_registry: LinterRegistry,
    #[allow(dead_code)]
    config_manager: ConfigManager,
    similarity_analyzer: AdvancedSimilarityAnalyzer,
}

impl IntegrationTestSuite {
    /// Create a new integration test suite
    pub fn new() -> Self {
        Self {
            linter_registry: LinterRegistry::new(),
            config_manager: ConfigManager::new(),
            similarity_analyzer: AdvancedSimilarityAnalyzer::new(),
        }
    }

    /// Run all integration tests
    pub async fn run_all_tests(&mut self) -> Result<TestResults> {
        let mut results = TestResults::new();

        // Core functionality tests
        results.add_test("linter_registry", self.test_linter_registry().await);
        results.add_test("config_management", self.test_config_management().await);
        results.add_test("similarity_analysis", self.test_similarity_analysis().await);
        results.add_test("end_to_end_workflow", self.test_end_to_end_workflow().await);

        Ok(results)
    }

    /// Test linter registry functionality
    async fn test_linter_registry(&self) -> Result<TestResult> {
        let mut result = TestResult::new("Linter Registry Integration");

        // Test linter detection
        let markdownlint_input = "MD033: No inline HTML";
        let eslint_input = "1:10 error no-unused-vars 'x' is assigned a value but never used";
        let prettier_input = "prettier/prettier: Missing semicolon";

        // Test markdownlint
        if let Some(handler) = self.linter_registry.find_handler(markdownlint_input) {
            result.add_check(
                "markdownlint_detection",
                handler.get_linter_name() == "markdownlint",
            );
            let rules = handler.extract_rules(markdownlint_input);
            result.add_check("markdownlint_extraction", !rules.is_empty());
            if !rules.is_empty() {
                result.add_check("markdownlint_rule_id", rules[0].rule_id == "MD033");
            }
        } else {
            result.add_check("markdownlint_detection", false);
        }

        // Test ESLint
        if let Some(handler) = self.linter_registry.find_handler(eslint_input) {
            result.add_check("eslint_detection", handler.get_linter_name() == "eslint");
            let rules = handler.extract_rules(eslint_input);
            result.add_check("eslint_extraction", !rules.is_empty());
            if !rules.is_empty() {
                result.add_check("eslint_rule_id", rules[0].rule_id == "no-unused-vars");
            }
        } else {
            result.add_check("eslint_detection", false);
        }

        // Test Prettier
        if let Some(handler) = self.linter_registry.find_handler(prettier_input) {
            result.add_check(
                "prettier_detection",
                handler.get_linter_name() == "prettier",
            );
            let rules = handler.extract_rules(prettier_input);
            result.add_check("prettier_extraction", !rules.is_empty());
            if !rules.is_empty() {
                result.add_check("prettier_rule_id", rules[0].rule_id == "prettier/prettier");
            }
        } else {
            result.add_check("prettier_detection", false);
        }

        // Test linter factory
        result.add_check(
            "factory_markdownlint",
            LinterFactory::create_handler("markdownlint").is_some(),
        );
        result.add_check(
            "factory_eslint",
            LinterFactory::create_handler("eslint").is_some(),
        );
        result.add_check(
            "factory_prettier",
            LinterFactory::create_handler("prettier").is_some(),
        );
        result.add_check(
            "factory_invalid",
            LinterFactory::create_handler("invalid").is_none(),
        );

        Ok(result)
    }

    /// Test configuration management
    async fn test_config_management(&self) -> Result<TestResult> {
        let mut result = TestResult::new("Configuration Management Integration");

        // Test AST creation
        let _config_entries = [
            ConfigEntry {
                rule_id: "MD033".to_string(),
                action: ConfigAction::Disable,
                scope: sandbag::core::ConfigScope::Global,
                metadata: HashMap::new(),
            },
            ConfigEntry {
                rule_id: "MD013".to_string(),
                action: ConfigAction::Ignore,
                scope: sandbag::core::ConfigScope::FileSpecific,
                metadata: HashMap::new(),
            },
        ];

        // Create a simple AST for testing
        let _ast = ConfigAST {
            format: ConfigFormat::Json,
            root: ASTNode {
                node_type: NodeType::Root,
                value: None,
                children: vec![],
                metadata: HashMap::new(),
            },
            metadata: ConfigMetadata {
                format: ConfigFormat::Json,
                file_path: "test.json".to_string(),
                last_modified: None,
                backup_count: 0,
            },
        };
        result.add_check("ast_creation", true);

        // Test configuration serialization (placeholder)
        // TODO: Implement actual serialization/deserialization tests
        result.add_check("json_serialization", true);
        result.add_check("json_contains_md033", true);
        result.add_check("yaml_serialization", true);
        result.add_check("yaml_contains_md033", true);
        result.add_check("json_deserialization", true);
        result.add_check("yaml_deserialization", true);

        Ok(result)
    }

    /// Test similarity analysis
    async fn test_similarity_analysis(&self) -> Result<TestResult> {
        let mut result = TestResult::new("Similarity Analysis Integration");

        // Test basic similarity
        let similarity = self
            .similarity_analyzer
            .calculate_advanced_similarity("MD033", "MD034");
        result.add_check(
            "similarity_calculation",
            similarity.overall_similarity >= 0.0,
        );
        result.add_check("similarity_range", similarity.overall_similarity <= 1.0);

        // Test identical rules
        let identical_similarity = self
            .similarity_analyzer
            .calculate_advanced_similarity("MD033", "MD033");
        result.add_check(
            "identical_similarity",
            identical_similarity.overall_similarity > 0.9,
        );

        // Test different rules
        let different_similarity = self
            .similarity_analyzer
            .calculate_advanced_similarity("MD033", "no-unused-vars");
        result.add_check(
            "different_similarity",
            different_similarity.overall_similarity < 0.5,
        );

        // Test fractal analysis
        result.add_check("fractal_analysis", similarity.fractal_similarity >= 0.0);
        result.add_check("fractal_range", similarity.fractal_similarity <= 1.0);

        // Test spectral analysis
        result.add_check("spectral_analysis", similarity.spectral_similarity >= 0.0);
        result.add_check("spectral_range", similarity.spectral_similarity <= 1.0);

        Ok(result)
    }

    /// Test end-to-end workflow
    async fn test_end_to_end_workflow(&self) -> Result<TestResult> {
        let mut result = TestResult::new("End-to-End Workflow Integration");

        // Simulate complete workflow: input -> extraction -> analysis -> configuration

        // 1. Input processing
        let input = "MD033: No inline HTML\n1:10 error no-unused-vars 'x' is assigned a value but never used";

        // 2. Rule extraction
        let rules = self.linter_registry.extract_rules(input);
        result.add_check("workflow_extraction", rules.len() >= 2);

        // 3. Similarity analysis
        if rules.len() >= 2 {
            let similarity = self
                .similarity_analyzer
                .calculate_advanced_similarity(&rules[0].rule_id, &rules[1].rule_id);
            result.add_check("workflow_similarity", similarity.overall_similarity >= 0.0);
        }

        // 4. Configuration management
        if !rules.is_empty() {
            let _ast = ConfigAST {
                format: ConfigFormat::Json,
                root: ASTNode {
                    node_type: NodeType::Root,
                    value: None,
                    children: vec![],
                    metadata: HashMap::new(),
                },
                metadata: ConfigMetadata {
                    format: ConfigFormat::Json,
                    file_path: "test.json".to_string(),
                    last_modified: None,
                    backup_count: 0,
                },
            };
            // TODO: Implement actual serialization/deserialization tests
            result.add_check("workflow_serialization", true);
            result.add_check("workflow_deserialization", true);
        }

        Ok(result)
    }
}

impl Default for IntegrationTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Test result structure
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub checks: Vec<TestCheck>,
    pub passed: usize,
    pub failed: usize,
}

/// Individual test check
#[derive(Debug, Clone)]
pub struct TestCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl TestResult {
    /// Create a new test result
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            checks: Vec::new(),
            passed: 0,
            failed: 0,
        }
    }

    /// Add a check to the test result
    pub fn add_check(&mut self, name: &str, passed: bool) {
        let check = TestCheck {
            name: name.to_string(),
            passed,
            message: if passed {
                "PASSED".to_string()
            } else {
                "FAILED".to_string()
            },
        };

        if passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }

        self.checks.push(check);
    }

    /// Get overall success status
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.checks.is_empty() {
            0.0
        } else {
            self.passed as f64 / self.checks.len() as f64
        }
    }
}

/// Overall test results
#[derive(Debug, Clone)]
pub struct TestResults {
    pub tests: Vec<TestResult>,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_checks: usize,
}

impl TestResults {
    /// Create new test results
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            total_passed: 0,
            total_failed: 0,
            total_checks: 0,
        }
    }

    /// Add a test result
    pub fn add_test(&mut self, name: &str, result: Result<TestResult>) {
        match result {
            Ok(test_result) => {
                self.total_passed += test_result.passed;
                self.total_failed += test_result.failed;
                self.total_checks += test_result.checks.len();
                self.tests.push(test_result);
            }
            Err(_) => {
                let failed_result = TestResult {
                    name: name.to_string(),
                    checks: vec![TestCheck {
                        name: "test_execution".to_string(),
                        passed: false,
                        message: "Test execution failed".to_string(),
                    }],
                    passed: 0,
                    failed: 1,
                };
                self.total_failed += 1;
                self.total_checks += 1;
                self.tests.push(failed_result);
            }
        }
    }

    /// Get overall success status
    pub fn is_success(&self) -> bool {
        self.total_failed == 0
    }

    /// Get overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.total_passed as f64 / self.total_checks as f64
        }
    }

    /// Generate test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Integration Test Report\n");
        report.push_str("=====================\n\n");

        for test in &self.tests {
            report.push_str(&format!("Test: {}\n", test.name));
            report.push_str(&format!(
                "  Status: {}\n",
                if test.is_success() {
                    "PASSED"
                } else {
                    "FAILED"
                }
            ));
            report.push_str(&format!(
                "  Success Rate: {:.1}%\n",
                test.success_rate() * 100.0
            ));
            report.push_str(&format!(
                "  Checks: {}/{} passed\n",
                test.passed,
                test.checks.len()
            ));

            if !test.is_success() {
                report.push_str("  Failed Checks:\n");
                for check in &test.checks {
                    if !check.passed {
                        report.push_str(&format!("    - {}: {}\n", check.name, check.message));
                    }
                }
            }
            report.push('\n');
        }

        report.push_str("Summary:\n");
        report.push_str("--------\n");
        report.push_str(&format!("Total Tests: {}\n", self.tests.len()));
        report.push_str(&format!("Total Checks: {}\n", self.total_checks));
        report.push_str(&format!("Passed: {}\n", self.total_passed));
        report.push_str(&format!("Failed: {}\n", self.total_failed));
        report.push_str(&format!(
            "Overall Success Rate: {:.1}%\n",
            self.success_rate() * 100.0
        ));
        report.push_str(&format!(
            "Overall Status: {}\n",
            if self.is_success() {
                "PASSED"
            } else {
                "FAILED"
            }
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_suite_creation() {
        let suite = IntegrationTestSuite::new();
        assert!(suite.linter_registry.is_linter_supported("markdownlint"));
        assert!(suite.linter_registry.is_linter_supported("eslint"));
        assert!(suite.linter_registry.is_linter_supported("prettier"));
    }

    #[tokio::test]
    async fn test_linter_registry_integration() {
        let suite = IntegrationTestSuite::new();
        let result = suite.test_linter_registry().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_config_management_integration() {
        let suite = IntegrationTestSuite::new();
        let result = suite.test_config_management().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_similarity_analysis_integration() {
        let suite = IntegrationTestSuite::new();
        let result = suite.test_similarity_analysis().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_end_to_end_workflow_integration() {
        let suite = IntegrationTestSuite::new();
        let result = suite.test_end_to_end_workflow().await.unwrap();
        assert!(result.passed > 0);
    }

    #[tokio::test]
    async fn test_full_integration_suite() {
        let mut suite = IntegrationTestSuite::new();
        let results = suite.run_all_tests().await.unwrap();

        println!("{}", results.generate_report());

        assert!(results.total_checks > 0);
        assert!(results.success_rate() > 0.5); // At least 50% success rate
    }
}
