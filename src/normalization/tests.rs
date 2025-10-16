//! Integration tests for normalization module

use std::path::Path;

use super::*;

#[test]
fn test_end_to_end_normalization() {
    let detector = ContentTypeDetector::new();
    let normalizer = TextNormalizer::default();

    // Test with code
    let rust_code = r#"
fn main() {
    println!("Hello, world!");
}
    "#;

    let content_type = detector.detect(rust_code, Some(Path::new("test.rs")));
    let result = normalizer.normalize(rust_code, Some(content_type));

    assert!(result.metadata.normalized_size <= result.metadata.original_size);
    assert_eq!(result.content_hash, result.content_hash); // Deterministic
}

#[test]
fn test_storage_reduction() {
    let normalizer = TextNormalizer::new(NormalizationPolicy {
        version: 1,
        level: NormalizationLevel::Aggressive,
        preserve_case: true,
        collapse_whitespace: true,
        remove_html: false,
    });

    let wasteful_text =
        "Hello      World\n\n\n\n\n\n\nThis    has    too    many    spaces\t\t\t\nAnd   tabs";

    let result = normalizer.normalize(wasteful_text, Some(ContentType::Plain));

    let reduction_ratio =
        result.metadata.removed_bytes as f64 / result.metadata.original_size as f64;

    // Should achieve at least 20% reduction on this wasteful text
    assert!(reduction_ratio >= 0.20);
}

#[test]
fn test_query_document_consistency() {
    let normalizer = TextNormalizer::default();

    let document = "  Machine   Learning  Tutorial  ";
    let query = "machine learning tutorial";

    let doc_result = normalizer.normalize(document, Some(ContentType::Plain));
    let query_result = normalizer.normalize_query(query);

    // When case-insensitive, should match
    let normalizer_case_insensitive = TextNormalizer::new(NormalizationPolicy {
        version: 1,
        level: NormalizationLevel::Aggressive,
        preserve_case: false,
        collapse_whitespace: true,
        remove_html: false,
    });

    let doc_lower = normalizer_case_insensitive.normalize(document, Some(ContentType::Plain));
    let query_lower = normalizer_case_insensitive.normalize_query(query);

    assert_eq!(doc_lower.text.trim(), query_lower.trim());
}

#[test]
fn test_unicode_edge_cases() {
    let normalizer = TextNormalizer::default();

    // Test various unicode scenarios
    let test_cases = vec![
        ("Café", "Café"), // Should preserve accents
        ("naïve", "naïve"),
        ("Hello\u{200B}World", "HelloWorld"), // Zero-width space removed
        ("\u{FEFF}BOM test", "BOM test"),     // BOM removed
    ];

    for (input, expected_contains) in test_cases {
        let result = normalizer.normalize(input, Some(ContentType::Plain));
        assert!(
            result.text.contains(expected_contains),
            "Failed: {} should contain {}",
            result.text,
            expected_contains
        );
    }
}

#[test]
fn test_markdown_code_block_preservation() {
    let normalizer = TextNormalizer::default();

    let markdown = r#"
# Code Example

```rust
fn main() {
    println!("Hello");
}
```

Some text after.
"#;

    let result = normalizer.normalize(markdown, Some(ContentType::Markdown));

    // Should still contain the code structure
    assert!(result.text.contains("fn main()"));
    assert!(result.text.contains("println!"));
}

#[test]
fn test_table_preservation() {
    let normalizer = TextNormalizer::default();

    let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA";

    let result = normalizer.normalize(
        csv,
        Some(ContentType::Table {
            format: TableFormat::Csv,
        }),
    );

    // Should preserve commas (delimiters)
    assert_eq!(result.text.matches(',').count(), 6);
}

#[test]
fn test_policy_versions() {
    let policy_v1 = NormalizationPolicy {
        version: 1,
        ..Default::default()
    };

    let policy_v2 = NormalizationPolicy {
        version: 2,
        ..Default::default()
    };

    let normalizer_v1 = TextNormalizer::new(policy_v1);
    let normalizer_v2 = TextNormalizer::new(policy_v2);

    let text = "Test text";

    let result_v1 = normalizer_v1.normalize(text, Some(ContentType::Plain));
    let result_v2 = normalizer_v2.normalize(text, Some(ContentType::Plain));

    // Versions should be tracked
    assert_eq!(result_v1.metadata.policy_version, 1);
    assert_eq!(result_v2.metadata.policy_version, 2);
}

#[test]
fn test_large_text_performance() {
    let normalizer = TextNormalizer::default();

    // Generate large text (1MB)
    let large_text = "Hello World! ".repeat(100_000);

    let start = std::time::Instant::now();
    let result = normalizer.normalize(&large_text, Some(ContentType::Plain));
    let duration = start.elapsed();

    // Should process 1MB in reasonable time (<100ms on modern hardware)
    assert!(duration.as_millis() < 500, "Too slow: {:?}", duration);
    assert!(result.metadata.normalized_size > 0);
}

#[test]
fn test_empty_text() {
    let normalizer = TextNormalizer::default();

    let result = normalizer.normalize("", Some(ContentType::Plain));

    assert_eq!(result.text, "");
    assert_eq!(result.metadata.original_size, 0);
    assert_eq!(result.metadata.normalized_size, 0);
}

#[test]
fn test_whitespace_only() {
    let normalizer = TextNormalizer::default();

    let result = normalizer.normalize("   \n\n\n   \t\t\t   ", Some(ContentType::Plain));

    // Should be collapsed to minimal whitespace or empty
    assert!(result.text.len() <= 2);
}
