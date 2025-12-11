//! Unit tests for similarity analysis algorithms

use sandbag::core::similarity::SimilarityAnalyzer;

#[test]
fn test_exact_match_similarity() {
    let analyzer = SimilarityAnalyzer::new();

    // Test exact matches
    assert_eq!(analyzer.weighted_similarity("MD033", "MD033"), 1.0);
    assert_eq!(analyzer.weighted_similarity("", ""), 1.0);

    // Test different strings
    assert!(analyzer.weighted_similarity("MD033", "MD034") < 1.0);
    assert!(analyzer.weighted_similarity("MD033", "ESLint") < 1.0);
}

#[test]
fn test_levenshtein_similarity() {
    let analyzer = SimilarityAnalyzer::new();

    // Test similar strings
    let similarity = analyzer.weighted_similarity("MD033", "MD034");
    assert!(similarity > 0.8); // Should be very similar

    // Test more different strings
    let similarity = analyzer.weighted_similarity("MD033", "MD999");
    assert!(similarity > 0.5); // Should be somewhat similar

    // Test very different strings
    let similarity = analyzer.weighted_similarity("MD033", "ESLint");
    assert!(similarity < 0.5); // Should be less similar
}

#[test]
fn test_jaccard_similarity() {
    let analyzer = SimilarityAnalyzer::new();

    // Test strings with similar character sets
    let similarity = analyzer.weighted_similarity("MD033", "MD034");
    assert!(similarity > 0.7); // Should be similar

    // Test strings with different character sets
    let similarity = analyzer.weighted_similarity("MD033", "ESLint");
    assert!(similarity < 0.5); // Should be less similar
}

#[test]
fn test_cosine_similarity() {
    let analyzer = SimilarityAnalyzer::new();

    // Test similar word patterns
    let similarity = analyzer.weighted_similarity("MD033 inline HTML", "MD034 inline HTML");
    assert!(similarity > 0.6); // Should be similar

    // Test different word patterns
    let similarity = analyzer.weighted_similarity("MD033 inline HTML", "ESLint no-console");
    assert!(similarity < 0.4); // Should be less similar
}

#[test]
fn test_find_most_similar() {
    let analyzer = SimilarityAnalyzer::new();

    let candidates = vec![
        "MD033".to_string(),
        "MD034".to_string(),
        "MD035".to_string(),
        "ESLint".to_string(),
    ];

    // Find most similar to MD033
    let result = analyzer.find_most_similar("MD033", &candidates);
    assert!(result.is_some());
    let (most_similar, confidence) = result.unwrap();
    assert_eq!(most_similar, "MD033");
    assert_eq!(confidence, 1.0);

    // Find most similar to MD033 (slight variation)
    let result = analyzer.find_most_similar("MD033", &candidates);
    assert!(result.is_some());
    let (most_similar, confidence) = result.unwrap();
    assert_eq!(most_similar, "MD033");
    assert_eq!(confidence, 1.0);
}

#[test]
fn test_similarity_breakdown() {
    let analyzer = SimilarityAnalyzer::new();

    let breakdown = analyzer.similarity_breakdown("MD033", "MD034");

    // Should have breakdown for each algorithm
    assert!(!breakdown.is_empty());

    // Each breakdown should have algorithm name, similarity score, and weight
    for (name, similarity, weight) in breakdown {
        assert!(!name.is_empty());
        assert!(similarity >= 0.0 && similarity <= 1.0);
        assert!(weight > 0.0);
    }
}

#[test]
fn test_edge_cases() {
    let analyzer = SimilarityAnalyzer::new();

    // Test empty strings
    assert_eq!(analyzer.weighted_similarity("", ""), 1.0);
    assert!(analyzer.weighted_similarity("", "MD033") < 1.0);

    // Test single character strings
    assert!(analyzer.weighted_similarity("M", "N") < 1.0);
    assert!(analyzer.weighted_similarity("M", "M") == 1.0);

    // Test very long strings
    let long_string1 = "MD033".repeat(100);
    let long_string2 = "MD034".repeat(100);
    let similarity = analyzer.weighted_similarity(&long_string1, &long_string2);
    assert!(similarity > 0.0 && similarity < 1.0);
}
