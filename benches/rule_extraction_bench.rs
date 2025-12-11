//! Benchmark for rule extraction performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sandbag::core::{ConfigAction, ExtractedContext, RuleMatch, Severity};
use sandbag::linters::LinterRegistry;
use sandbag::performance::optimizations::OptimizedRuleProcessor;

fn benchmark_rule_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_extraction");

    let linter_registry = LinterRegistry::new();
    let _processor = OptimizedRuleProcessor::new();

    // Test data
    let markdownlint_input = "MD033: No inline HTML\nMD013: Line too long\nMD041: First line in file should be a top level heading";
    let eslint_input = "1:10 error no-unused-vars 'x' is assigned a value but never used\n2:5 warn prefer-const/const Use const instead of let";
    let prettier_input =
        "prettier/prettier: Missing semicolon\nCode style issues found: Line width exceeded";

    group.bench_function("markdownlint_extraction", |b| {
        b.iter(|| {
            black_box(linter_registry.extract_rules(markdownlint_input));
        });
    });

    group.bench_function("eslint_extraction", |b| {
        b.iter(|| {
            black_box(linter_registry.extract_rules(eslint_input));
        });
    });

    group.bench_function("prettier_extraction", |b| {
        b.iter(|| {
            black_box(linter_registry.extract_rules(prettier_input));
        });
    });

    group.bench_function("mixed_linter_extraction", |b| {
        let mixed_input = format!("{markdownlint_input}\n{eslint_input}\n{prettier_input}");
        b.iter(|| {
            black_box(linter_registry.extract_rules(&mixed_input));
        });
    });

    group.finish();
}

fn benchmark_similarity_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("similarity_calculation");

    let processor = OptimizedRuleProcessor::new();

    // Test strings
    let strings = vec![
        "MD033".to_string(),
        "MD034".to_string(),
        "no-unused-vars".to_string(),
        "prefer-const".to_string(),
        "prettier/prettier".to_string(),
    ];

    group.bench_function("similarity_matrix", |b| {
        b.iter(|| {
            black_box(processor.calculate_similarity_matrix(&strings));
        });
    });

    group.bench_function("cached_similarity", |b| {
        b.iter(|| {
            black_box(processor.calculate_similarity_cached("MD033", "MD034"));
        });
    });

    group.finish();
}

fn benchmark_rule_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_processing");

    let processor = OptimizedRuleProcessor::new();

    // Generate test rules
    let rules: Vec<RuleMatch> = (0..100)
        .map(|i| RuleMatch {
            rule_id: format!("MD{:03}", i % 50),
            linter: "markdownlint".to_string(),
            confidence: 0.8 + (i % 20) as f64 * 0.01,
            context: ExtractedContext {
                file_path: None,
                line_number: Some(i as u32),
                column: None,
                message: Some(format!("Test rule {i}")),
                severity: Some(Severity::Warning),
            },
            suggested_action: ConfigAction::Disable,
        })
        .collect();

    group.bench_function("parallel_processing", |b| {
        b.iter(|| {
            black_box(processor.process_rules_parallel(&rules));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_rule_extraction,
    benchmark_similarity_calculation,
    benchmark_rule_processing
);
criterion_main!(benches);
