//! Semantic configuration analysis using ontologies and graph theory

use crate::core::{ConfigAction, ConfigEntry, ConfigScope};
use std::collections::HashMap;

/// Semantic ontology for linter rules
pub struct LinterOntology {
    /// Concept registry keyed by concept id
    concepts: HashMap<String, OntologyConcept>,
    /// Relationships between concepts
    #[allow(dead_code)]
    relationships: Vec<OntologyRelationship>,
    /// Mapping from rule id to semantic concepts
    rule_semantics: HashMap<String, Vec<String>>, // rule_id -> semantic concepts
}

/// Ontology concept
#[derive(Debug, Clone)]
pub struct OntologyConcept {
    /// Unique concept identifier
    pub id: String,
    /// Human-readable concept name
    pub name: String,
    /// Concept description
    pub description: String,
    /// Parent concepts (is-a relationships)
    pub parent_concepts: Vec<String>,
    /// Child concepts (refinements)
    pub child_concepts: Vec<String>,
    /// Weight used in semantic scoring
    pub semantic_weight: f64,
}

/// Ontology relationship
#[derive(Debug, Clone)]
pub struct OntologyRelationship {
    /// Source concept id
    pub source: String,
    /// Target concept id
    pub target: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship strength [0.0, 1.0]
    pub strength: f64,
}

/// Relationship types
#[derive(Debug, Clone)]
pub enum RelationshipType {
    /// Source is a subtype of target
    IsA,
    /// Source is a component of target
    PartOf,
    /// Source depends on target
    DependsOn,
    /// Source conflicts with target
    ConflictsWith,
    /// Source is similar to target
    SimilarTo,
    /// Source is related to target (generic)
    RelatedTo,
}

/// Semantic analyzer for configuration understanding
pub struct SemanticAnalyzer {
    ontology: LinterOntology,
    graph_analyzer: GraphAnalyzer,
    clustering_engine: ClusteringEngine,
    decision_tree: DecisionTree,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        let ontology = Self::create_default_ontology();

        Self {
            ontology,
            graph_analyzer: GraphAnalyzer::new(),
            clustering_engine: ClusteringEngine::new(),
            decision_tree: DecisionTree::new(),
        }
    }

    /// Create default ontology with common linter concepts
    fn create_default_ontology() -> LinterOntology {
        let mut concepts = HashMap::new();
        let mut relationships = Vec::new();
        let mut rule_semantics = HashMap::new();

        // Add core concepts
        concepts.insert(
            "style".to_string(),
            OntologyConcept {
                id: "style".to_string(),
                name: "Code Style".to_string(),
                description: "Rules related to code formatting and style".to_string(),
                parent_concepts: Vec::new(),
                child_concepts: vec!["formatting".to_string(), "naming".to_string()],
                semantic_weight: 0.8,
            },
        );

        concepts.insert(
            "formatting".to_string(),
            OntologyConcept {
                id: "formatting".to_string(),
                name: "Formatting".to_string(),
                description: "Code formatting rules".to_string(),
                parent_concepts: vec!["style".to_string()],
                child_concepts: Vec::new(),
                semantic_weight: 0.6,
            },
        );

        concepts.insert(
            "naming".to_string(),
            OntologyConcept {
                id: "naming".to_string(),
                name: "Naming Conventions".to_string(),
                description: "Variable and function naming rules".to_string(),
                parent_concepts: vec!["style".to_string()],
                child_concepts: Vec::new(),
                semantic_weight: 0.7,
            },
        );

        concepts.insert(
            "security".to_string(),
            OntologyConcept {
                id: "security".to_string(),
                name: "Security".to_string(),
                description: "Security-related rules".to_string(),
                parent_concepts: Vec::new(),
                child_concepts: Vec::new(),
                semantic_weight: 0.9,
            },
        );

        concepts.insert(
            "performance".to_string(),
            OntologyConcept {
                id: "performance".to_string(),
                name: "Performance".to_string(),
                description: "Performance optimization rules".to_string(),
                parent_concepts: Vec::new(),
                child_concepts: Vec::new(),
                semantic_weight: 0.8,
            },
        );

        // Add relationships
        relationships.push(OntologyRelationship {
            source: "formatting".to_string(),
            target: "style".to_string(),
            relationship_type: RelationshipType::IsA,
            strength: 0.9,
        });

        relationships.push(OntologyRelationship {
            source: "naming".to_string(),
            target: "style".to_string(),
            relationship_type: RelationshipType::IsA,
            strength: 0.9,
        });

        relationships.push(OntologyRelationship {
            source: "security".to_string(),
            target: "performance".to_string(),
            relationship_type: RelationshipType::ConflictsWith,
            strength: 0.3,
        });

        // Add rule semantics
        rule_semantics.insert("MD033".to_string(), vec!["security".to_string()]);
        rule_semantics.insert("MD013".to_string(), vec!["formatting".to_string()]);
        rule_semantics.insert("MD041".to_string(), vec!["formatting".to_string()]);

        LinterOntology {
            concepts,
            relationships,
            rule_semantics,
        }
    }

    /// Analyze semantic meaning of configuration
    pub fn analyze_semantics(&mut self, rules: &[ConfigEntry]) -> SemanticAnalysisResult {
        let mut result = SemanticAnalysisResult::new();

        // Analyze rule semantics
        for rule in rules {
            let rule_semantics = self.analyze_rule_semantics(rule);
            result
                .rule_semantics
                .insert(rule.rule_id.clone(), rule_semantics);
        }

        // Perform graph analysis
        result.graph_analysis = self
            .graph_analyzer
            .analyze_rule_graph(rules, &self.ontology);

        // Perform clustering analysis
        result.clustering_analysis = self.clustering_engine.cluster_rules(rules, &self.ontology);

        // Perform decision tree analysis
        result.decision_analysis = self
            .decision_tree
            .analyze_configuration(rules, &self.ontology);

        result
    }

    /// Analyze semantic meaning of a single rule
    fn analyze_rule_semantics(&self, rule: &ConfigEntry) -> RuleSemantics {
        let concepts = self
            .ontology
            .rule_semantics
            .get(&rule.rule_id)
            .cloned()
            .unwrap_or_default();

        let semantic_score = self.calculate_semantic_score(&concepts);
        let related_rules = self.find_related_rules(&concepts);

        RuleSemantics {
            concepts,
            semantic_score,
            related_rules,
            action_semantics: self.analyze_action_semantics(&rule.action),
        }
    }

    /// Calculate semantic score based on concepts
    #[allow(clippy::cast_precision_loss)]
    fn calculate_semantic_score(&self, concepts: &[String]) -> f64 {
        if concepts.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = concepts
            .iter()
            .filter_map(|concept_id| self.ontology.concepts.get(concept_id))
            .map(|concept| concept.semantic_weight)
            .sum();

        total_weight / concepts.len() as f64
    }

    /// Find related rules based on concepts
    fn find_related_rules(&self, concepts: &[String]) -> Vec<String> {
        let mut related_rules = Vec::new();

        for (rule_id, rule_concepts) in &self.ontology.rule_semantics {
            for concept in concepts {
                if rule_concepts.contains(concept) {
                    related_rules.push(rule_id.clone());
                    break;
                }
            }
        }

        related_rules
    }

    /// Analyze semantic meaning of actions
    #[allow(clippy::unused_self)]
    fn analyze_action_semantics(&self, action: &ConfigAction) -> ActionSemantics {
        match action {
            ConfigAction::Disable => ActionSemantics {
                severity: "high".to_string(),
                impact: "removes rule completely".to_string(),
                risk_level: 0.8,
            },
            ConfigAction::Ignore => ActionSemantics {
                severity: "medium".to_string(),
                impact: "ignores rule violations".to_string(),
                risk_level: 0.5,
            },
            ConfigAction::Modify => ActionSemantics {
                severity: "low".to_string(),
                impact: "modifies rule behavior".to_string(),
                risk_level: 0.3,
            },
            ConfigAction::Enable => ActionSemantics {
                severity: "low".to_string(),
                impact: "enables rule enforcement".to_string(),
                risk_level: 0.1,
            },
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph analyzer for rule relationships
pub struct GraphAnalyzer {
    graph: RuleGraph,
}

impl GraphAnalyzer {
    /// Create a new graph analyzer
    pub fn new() -> Self {
        Self {
            graph: RuleGraph::new(),
        }
    }

    /// Analyze rule graph
    pub fn analyze_rule_graph(
        &mut self,
        rules: &[ConfigEntry],
        ontology: &LinterOntology,
    ) -> GraphAnalysis {
        let mut analysis = GraphAnalysis::new();

        // Build graph from rules
        for rule in rules {
            self.graph.add_rule(rule);
        }

        // Add semantic relationships
        for rule in rules {
            if let Some(concepts) = ontology.rule_semantics.get(&rule.rule_id) {
                for concept in concepts {
                    self.graph.add_semantic_relationship(&rule.rule_id, concept);
                }
            }
        }

        // Analyze graph properties
        analysis.centrality_scores = self.calculate_centrality_scores();
        analysis.community_structure = self.detect_communities();
        analysis.influence_paths = self.find_influence_paths();

        analysis
    }

    /// Calculate centrality scores for rules
    #[allow(clippy::unused_self)]
    fn calculate_centrality_scores(&self) -> HashMap<String, f64> {
        // TODO: Implement centrality calculation
        HashMap::new()
    }

    /// Detect communities in the rule graph
    #[allow(clippy::unused_self)]
    fn detect_communities(&self) -> Vec<Vec<String>> {
        // TODO: Implement community detection
        Vec::new()
    }

    /// Find influence paths between rules
    #[allow(clippy::unused_self)]
    fn find_influence_paths(&self) -> Vec<InfluencePath> {
        // TODO: Implement path finding
        Vec::new()
    }
}

impl Default for GraphAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule graph representation
#[derive(Debug)]
pub struct RuleGraph {
    nodes: HashMap<String, RuleNode>,
    #[allow(dead_code)]
    edges: Vec<RuleEdge>,
}

/// Rule node in graph
#[derive(Debug)]
pub struct RuleNode {
    /// Rule identifier
    pub rule_id: String,
    /// Action applied to the rule
    pub action: ConfigAction,
    /// Scope of the rule
    pub scope: ConfigScope,
    /// Attached semantic concepts
    pub semantic_concepts: Vec<String>,
}

/// Rule edge in graph
#[derive(Debug)]
pub struct RuleEdge {
    /// Source rule id
    pub source: String,
    /// Target rule id
    pub target: String,
    /// Relationship type label
    pub relationship_type: String,
    /// Edge weight
    pub weight: f64,
}

impl RuleGraph {
    /// Create a new rule graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Add a rule to the graph
    pub fn add_rule(&mut self, rule: &ConfigEntry) {
        let node = RuleNode {
            rule_id: rule.rule_id.clone(),
            action: rule.action.clone(),
            scope: rule.scope.clone(),
            semantic_concepts: Vec::new(),
        };
        self.nodes.insert(rule.rule_id.clone(), node);
    }

    /// Add semantic relationship
    pub fn add_semantic_relationship(&mut self, rule_id: &str, concept: &str) {
        if let Some(node) = self.nodes.get_mut(rule_id) {
            node.semantic_concepts.push(concept.to_string());
        }
    }
}

impl Default for RuleGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Clustering engine for rule grouping
pub struct ClusteringEngine {
    #[allow(dead_code)]
    clustering_algorithm: ClusteringAlgorithm,
}

impl ClusteringEngine {
    /// Create a new clustering engine
    pub fn new() -> Self {
        Self {
            clustering_algorithm: ClusteringAlgorithm::KMeans,
        }
    }

    /// Cluster rules based on semantic similarity
    pub fn cluster_rules(
        &self,
        rules: &[ConfigEntry],
        ontology: &LinterOntology,
    ) -> ClusteringAnalysis {
        let mut analysis = ClusteringAnalysis::new();

        // Extract features for clustering
        let features = self.extract_clustering_features(rules, ontology);

        // Perform clustering
        let clusters = self.perform_clustering(&features);

        // Analyze cluster quality
        analysis.clusters = clusters;
        analysis.silhouette_score = self.calculate_silhouette_score(&features, &analysis.clusters);
        analysis.cluster_coherence = self.calculate_cluster_coherence(&analysis.clusters, ontology);

        analysis
    }

    /// Extract features for clustering
    fn extract_clustering_features(
        &self,
        rules: &[ConfigEntry],
        ontology: &LinterOntology,
    ) -> Vec<ClusteringFeature> {
        let mut features = Vec::new();

        for rule in rules {
            let semantic_concepts = ontology
                .rule_semantics
                .get(&rule.rule_id)
                .cloned()
                .unwrap_or_default();

            features.push(ClusteringFeature {
                rule_id: rule.rule_id.clone(),
                action_encoding: self.encode_action(&rule.action),
                scope_encoding: self.encode_scope(&rule.scope),
                semantic_vector: self.create_semantic_vector(&semantic_concepts, ontology),
            });
        }

        features
    }

    /// Encode action as numerical value
    #[allow(clippy::unused_self)]
    fn encode_action(&self, action: &ConfigAction) -> f64 {
        match action {
            ConfigAction::Disable => 0.0,
            ConfigAction::Ignore => 0.33,
            ConfigAction::Modify => 0.66,
            ConfigAction::Enable => 1.0,
        }
    }

    /// Encode scope as numerical value
    #[allow(clippy::unused_self)]
    fn encode_scope(&self, scope: &ConfigScope) -> f64 {
        match scope {
            ConfigScope::Global => 0.0,
            ConfigScope::Directory => 0.33,
            ConfigScope::FileSpecific => 0.66,
            ConfigScope::Inline => 1.0,
        }
    }

    /// Create semantic vector for clustering
    #[allow(clippy::unused_self)]
    fn create_semantic_vector(&self, concepts: &[String], ontology: &LinterOntology) -> Vec<f64> {
        let mut vector = vec![0.0; ontology.concepts.len()];

        for (i, concept_id) in ontology.concepts.keys().enumerate() {
            if concepts.contains(concept_id) {
                if let Some(concept) = ontology.concepts.get(concept_id) {
                    vector[i] = concept.semantic_weight;
                }
            }
        }

        vector
    }

    /// Perform clustering algorithm
    #[allow(clippy::unused_self)]
    fn perform_clustering(&self, features: &[ClusteringFeature]) -> Vec<RuleCluster> {
        // TODO: Implement actual clustering algorithm
        vec![RuleCluster {
            cluster_id: 0,
            rules: features.iter().map(|f| f.rule_id.clone()).collect(),
            centroid: vec![0.5; 10],
            coherence_score: 0.8,
        }]
    }

    /// Calculate silhouette score for clustering quality
    #[allow(clippy::unused_self)]
    fn calculate_silhouette_score(
        &self,
        _features: &[ClusteringFeature],
        _clusters: &[RuleCluster],
    ) -> f64 {
        // TODO: Implement silhouette score calculation
        0.7
    }

    /// Calculate cluster coherence
    #[allow(clippy::unused_self)]
    fn calculate_cluster_coherence(
        &self,
        _clusters: &[RuleCluster],
        _ontology: &LinterOntology,
    ) -> f64 {
        // TODO: Implement coherence calculation
        0.8
    }
}

impl Default for ClusteringEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Clustering algorithm types
#[derive(Debug, Clone)]
pub enum ClusteringAlgorithm {
    /// K-Means clustering
    KMeans,
    /// Hierarchical agglomerative clustering
    Hierarchical,
    /// Density-based spatial clustering
    DBSCAN,
    /// Spectral clustering
    Spectral,
}

/// Clustering feature
#[derive(Debug, Clone)]
pub struct ClusteringFeature {
    /// Rule identifier this feature represents
    pub rule_id: String,
    /// Encoded action value
    pub action_encoding: f64,
    /// Encoded scope value
    pub scope_encoding: f64,
    /// Semantic feature vector
    pub semantic_vector: Vec<f64>,
}

/// Rule cluster
#[derive(Debug, Clone)]
pub struct RuleCluster {
    /// Cluster identifier
    pub cluster_id: usize,
    /// Member rule ids
    pub rules: Vec<String>,
    /// Cluster centroid vector
    pub centroid: Vec<f64>,
    /// Cluster coherence score
    pub coherence_score: f64,
}

/// Decision tree for configuration analysis
pub struct DecisionTree {
    #[allow(dead_code)]
    tree: DecisionNode,
}

impl DecisionTree {
    /// Create a new decision tree
    pub fn new() -> Self {
        Self {
            tree: DecisionNode::new_leaf("default".to_string()),
        }
    }

    /// Analyze configuration using decision tree
    pub fn analyze_configuration(
        &mut self,
        rules: &[ConfigEntry],
        ontology: &LinterOntology,
    ) -> DecisionAnalysis {
        let mut analysis = DecisionAnalysis::new();

        for rule in rules {
            let decision_path = self.traverse_decision_tree(rule, ontology);
            analysis
                .decision_paths
                .insert(rule.rule_id.clone(), decision_path);
        }

        analysis.recommendations = self.generate_recommendations(rules, ontology);
        analysis.confidence_scores = self.calculate_confidence_scores(rules, ontology);

        analysis
    }

    /// Traverse decision tree for a rule
    #[allow(clippy::unused_self)]
    fn traverse_decision_tree(
        &self,
        _rule: &ConfigEntry,
        _ontology: &LinterOntology,
    ) -> DecisionPath {
        // TODO: Implement decision tree traversal
        DecisionPath {
            path: vec!["root".to_string(), "action".to_string()],
            confidence: 0.8,
            reasoning: "Based on rule action and scope".to_string(),
        }
    }

    /// Generate recommendations
    #[allow(clippy::unused_self)]
    fn generate_recommendations(
        &self,
        rules: &[ConfigEntry],
        _ontology: &LinterOntology,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Analyze action distribution
        let action_counts: HashMap<ConfigAction, usize> =
            rules.iter().fold(HashMap::new(), |mut acc, rule| {
                *acc.entry(rule.action.clone()).or_insert(0) += 1;
                acc
            });

        // Generate recommendations based on patterns
        if *action_counts.get(&ConfigAction::Disable).unwrap_or(&0) > 5 {
            recommendations.push(Recommendation {
                category: "action".to_string(),
                description: "Consider using more specific actions instead of disabling rules"
                    .to_string(),
                priority: 0.8,
                impact: "Improves code quality".to_string(),
            });
        }

        recommendations
    }

    /// Count actions in rules
    #[allow(clippy::unused_self)]
    #[allow(dead_code)]
    fn count_actions(&self, rules: &[ConfigEntry]) -> HashMap<ConfigAction, usize> {
        let mut counts = HashMap::new();
        for rule in rules {
            *counts.entry(rule.action.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Analyze scope distribution
    #[allow(clippy::unused_self)]
    #[allow(dead_code)]
    fn analyze_scope_distribution(&self, rules: &[ConfigEntry]) -> HashMap<ConfigScope, usize> {
        let mut distribution = HashMap::new();
        for rule in rules {
            *distribution.entry(rule.scope.clone()).or_insert(0) += 1;
        }
        distribution
    }

    /// Calculate confidence scores
    #[allow(clippy::unused_self)]
    fn calculate_confidence_scores(
        &self,
        _rules: &[ConfigEntry],
        _ontology: &LinterOntology,
    ) -> HashMap<String, f64> {
        // TODO: Implement confidence score calculation
        HashMap::new()
    }
}

impl Default for DecisionTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision tree node
#[derive(Debug)]
pub struct DecisionNode {
    /// Node type
    pub node_type: NodeType,
    /// Condition text for decision nodes
    pub condition: Option<String>,
    /// Child nodes
    pub children: Vec<DecisionNode>,
    /// Outcome label for leaf nodes
    pub outcome: Option<String>,
}

/// Node types
#[derive(Debug)]
pub enum NodeType {
    /// Decision node with a condition and children
    Decision,
    /// Leaf node with an outcome
    Leaf,
}

impl DecisionNode {
    /// Create a new leaf node
    pub fn new_leaf(outcome: String) -> Self {
        Self {
            node_type: NodeType::Leaf,
            condition: None,
            children: Vec::new(),
            outcome: Some(outcome),
        }
    }

    /// Create a new decision node
    pub fn new_decision(condition: String) -> Self {
        Self {
            node_type: NodeType::Decision,
            condition: Some(condition),
            children: Vec::new(),
            outcome: None,
        }
    }
}

/// Semantic analysis result
#[derive(Debug, Clone)]
pub struct SemanticAnalysisResult {
    /// Semantic details per rule id
    pub rule_semantics: HashMap<String, RuleSemantics>,
    /// Graph analysis results
    pub graph_analysis: GraphAnalysis,
    /// Clustering analysis results
    pub clustering_analysis: ClusteringAnalysis,
    /// Decision analysis results
    pub decision_analysis: DecisionAnalysis,
}

impl SemanticAnalysisResult {
    /// Create a new semantic analysis result
    pub fn new() -> Self {
        Self {
            rule_semantics: HashMap::new(),
            graph_analysis: GraphAnalysis::new(),
            clustering_analysis: ClusteringAnalysis::new(),
            decision_analysis: DecisionAnalysis::new(),
        }
    }
}

impl Default for SemanticAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule semantics
#[derive(Debug, Clone)]
pub struct RuleSemantics {
    /// Concepts associated with the rule
    pub concepts: Vec<String>,
    /// Aggregate semantic score
    pub semantic_score: f64,
    /// Other rules related by concept
    pub related_rules: Vec<String>,
    /// Action semantics summary
    pub action_semantics: ActionSemantics,
}

/// Action semantics
#[derive(Debug, Clone)]
pub struct ActionSemantics {
    /// Severity level
    pub severity: String,
    /// Impact description
    pub impact: String,
    /// Risk level [0.0, 1.0]
    pub risk_level: f64,
}

/// Graph analysis result
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    /// Centrality scores per rule id
    pub centrality_scores: HashMap<String, f64>,
    /// Community structure (clusters of rule ids)
    pub community_structure: Vec<Vec<String>>,
    /// Influence paths discovered in the graph
    pub influence_paths: Vec<InfluencePath>,
}

impl GraphAnalysis {
    /// Create a new graph analysis
    pub fn new() -> Self {
        Self {
            centrality_scores: HashMap::new(),
            community_structure: Vec::new(),
            influence_paths: Vec::new(),
        }
    }
}

impl Default for GraphAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Influence path
#[derive(Debug, Clone)]
pub struct InfluencePath {
    /// Source node id
    pub source: String,
    /// Target node id
    pub target: String,
    /// Path between nodes
    pub path: Vec<String>,
    /// Path strength [0.0, 1.0]
    pub strength: f64,
}

/// Clustering analysis result
#[derive(Debug, Clone)]
pub struct ClusteringAnalysis {
    /// Clustering output
    pub clusters: Vec<RuleCluster>,
    /// Silhouette score measuring cluster quality
    pub silhouette_score: f64,
    /// Cluster coherence metric
    pub cluster_coherence: f64,
}

impl ClusteringAnalysis {
    /// Create a new clustering analysis
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            silhouette_score: 0.0,
            cluster_coherence: 0.0,
        }
    }
}

impl Default for ClusteringAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision analysis result
#[derive(Debug, Clone)]
pub struct DecisionAnalysis {
    /// Decision paths per rule id
    pub decision_paths: HashMap<String, DecisionPath>,
    /// Recommendations generated from analysis
    pub recommendations: Vec<Recommendation>,
    /// Confidence scores per rule id
    pub confidence_scores: HashMap<String, f64>,
}

impl DecisionAnalysis {
    /// Create a new decision analysis
    pub fn new() -> Self {
        Self {
            decision_paths: HashMap::new(),
            recommendations: Vec::new(),
            confidence_scores: HashMap::new(),
        }
    }
}

impl Default for DecisionAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision path
#[derive(Debug, Clone)]
pub struct DecisionPath {
    /// Nodes traversed in the decision process
    pub path: Vec<String>,
    /// Confidence score for this decision
    pub confidence: f64,
    /// Textual reasoning for the decision
    pub reasoning: String,
}

/// Recommendation
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// Recommendation category (e.g., action, scope)
    pub category: String,
    /// Description of the recommendation
    pub description: String,
    /// Priority [0.0, 1.0]
    pub priority: f64,
    /// Expected impact
    pub impact: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(!analyzer.ontology.concepts.is_empty());
    }

    #[test]
    fn test_rule_semantics_analysis() {
        let analyzer = SemanticAnalyzer::new();
        let rule = ConfigEntry {
            rule_id: "MD033".to_string(),
            action: ConfigAction::Disable,
            scope: ConfigScope::Global,
            metadata: HashMap::new(),
        };

        let semantics = analyzer.analyze_rule_semantics(&rule);
        assert!(!semantics.concepts.is_empty());
        assert!(semantics.semantic_score > 0.0);
    }

    #[test]
    fn test_clustering_engine() {
        let engine = ClusteringEngine::new();
        let rules = vec![ConfigEntry {
            rule_id: "MD033".to_string(),
            action: ConfigAction::Disable,
            scope: ConfigScope::Global,
            metadata: HashMap::new(),
        }];

        let ontology = SemanticAnalyzer::create_default_ontology();
        let analysis = engine.cluster_rules(&rules, &ontology);

        assert!(!analysis.clusters.is_empty());
        assert!(analysis.silhouette_score >= 0.0);
    }
}
