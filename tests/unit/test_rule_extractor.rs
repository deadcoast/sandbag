//! Unit tests for rule extraction engine

use sandbag::core::rule_extractor::RuleExtractor;
use sandbag::core::{ConfigAction, RuleMatch, Severity};

#[tokio::test]
async fn test_markdownlint_rule_extraction() {
    let extractor = RuleExtractor::new();

    // Test standard markdownlint output format
    let input = "MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033";
    let results = extractor.extract_rules(input).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "MD033");
    assert_eq!(results[0].linter, "markdownlint");
    assert!(results[0].confidence > 0.8);
    assert!(matches!(results[0].suggested_action, ConfigAction::Disable));
}

#[tokio::test]
async fn test_rule_extraction_with_multiple_patterns() {
    let extractor = RuleExtractor::new();

    // Test different input formats
    let inputs = vec![
        "MD033/no-inline-html: Inline HTML [Element: div]",
        "markdownlint MD033: No inline HTML",
        "MD033: Inline HTML detected",
    ];

    for input in inputs {
        let results = extractor.extract_rules(input).await.unwrap();
        assert!(
            !results.is_empty(),
            "Failed to extract rule from: {}",
            input
        );
        assert_eq!(results[0].rule_id, "MD033");
    }
}

#[tokio::test]
async fn test_rule_extraction_confidence_scoring() {
    let extractor = RuleExtractor::new();

    // Test high confidence input
    let high_confidence_input =
        "MD033/no-inline-html: Inline HTML [Element: summary]markdownlintMD033";
    let high_results = extractor
        .extract_rules(high_confidence_input)
        .await
        .unwrap();

    // Test low confidence input
    let low_confidence_input = "Some random text that might contain MD033";
    let low_results = extractor.extract_rules(low_confidence_input).await.unwrap();

    if !low_results.is_empty() {
        assert!(high_results[0].confidence > low_results[0].confidence);
    }
}

#[tokio::test]
async fn test_rule_extraction_empty_input() {
    let extractor = RuleExtractor::new();

    let results = extractor.extract_rules("").await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_rule_extraction_invalid_input() {
    let extractor = RuleExtractor::new();

    let results = extractor
        .extract_rules("This is not a linter output")
        .await
        .unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_rule_extraction_multiple_rules() {
    let extractor = RuleExtractor::new();

    // Test input with multiple rules
    let input = "MD033: Inline HTML\nMD041: First line should be heading";
    let results = extractor.extract_rules(input).await.unwrap();

    // Should extract both rules
    assert_eq!(results.len(), 2);

    let rule_ids: Vec<&str> = results.iter().map(|r| r.rule_id.as_str()).collect();
    assert!(rule_ids.contains(&"MD033"));
    assert!(rule_ids.contains(&"MD041"));
}
