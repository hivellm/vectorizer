//! Text Normalization Performance Benchmarks
//!
//! Measures throughput and efficiency of text normalization.

use std::time::Instant;
use vectorizer::normalization::{
    ContentType, ContentTypeDetector, NormalizationLevel, NormalizationPolicy, TextNormalizer,
};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          Text Normalization Performance Benchmark             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Test data
    let test_samples = generate_test_samples();

    // Run benchmarks
    benchmark_content_type_detection(&test_samples);
    println!();
    benchmark_normalization_levels(&test_samples);
    println!();
    benchmark_throughput(&test_samples);
    println!();
    benchmark_compression_ratios(&test_samples);
    println!();
    benchmark_hash_performance(&test_samples);
}

struct TestSample {
    name: &'static str,
    content: String,
    expected_type: ContentType,
}

fn generate_test_samples() -> Vec<TestSample> {
    vec![
        TestSample {
            name: "Small Plain Text",
            content: "This is a simple text document with some words.".to_string(),
            expected_type: ContentType::Plain,
        },
        TestSample {
            name: "Medium Plain Text",
            content: "Lorem ipsum dolor sit amet. ".repeat(100),
            expected_type: ContentType::Plain,
        },
        TestSample {
            name: "Large Plain Text",
            content: "Lorem ipsum dolor sit amet. ".repeat(10_000),
            expected_type: ContentType::Plain,
        },
        TestSample {
            name: "Wasteful Whitespace",
            content: "Hello    World\n\n\n\n\nWith   too   many   spaces\t\t\t".repeat(100),
            expected_type: ContentType::Plain,
        },
        TestSample {
            name: "Rust Code",
            content: r#"
fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
}
"#
            .repeat(50),
            expected_type: ContentType::Code {
                language: Some("rust".to_string()),
            },
        },
        TestSample {
            name: "Markdown Document",
            content: r#"
# Heading 1

This is a paragraph with **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2
- List item 3

```rust
fn example() {
    println!("code block");
}
```
"#
            .repeat(20),
            expected_type: ContentType::Markdown,
        },
        TestSample {
            name: "JSON Data",
            content: r#"{"name": "John", "age": 30, "city": "New York"}"#.repeat(100),
            expected_type: ContentType::Json,
        },
    ]
}

fn benchmark_content_type_detection(samples: &[TestSample]) {
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│               Content Type Detection Speed                  │");
    println!("├─────────────────────────────────────────────────────────────┤");

    let detector = ContentTypeDetector::new();

    for sample in samples {
        let iterations = 10_000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = detector.detect(&sample.content, None);
        }

        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();

        println!(
            "│ {:30} │ {:>10.0} ops/s │",
            sample.name, ops_per_sec
        );
    }

    println!("└─────────────────────────────────────────────────────────────┘");
}

fn benchmark_normalization_levels(samples: &[TestSample]) {
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                      Normalization Level Performance                        │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Sample                     │ Conservative │  Moderate   │ Aggressive │");
    println!("│                            │    (μs)      │    (μs)     │   (μs)     │");
    println!("├─────────────────────────────────────────────────────────────────────────────┤");

    let levels = [
        NormalizationLevel::Conservative,
        NormalizationLevel::Moderate,
        NormalizationLevel::Aggressive,
    ];

    for sample in samples {
        print!("│ {:27}", sample.name);

        for level in levels {
            let policy = NormalizationPolicy {
                level,
                ..Default::default()
            };
            let normalizer = TextNormalizer::new(policy);

            let iterations = 1000;
            let start = Instant::now();

            for _ in 0..iterations {
                let _ = normalizer.normalize(&sample.content, Some(ContentType::Plain));
            }

            let duration = start.elapsed();
            let avg_micros = duration.as_micros() / iterations;

            print!("│ {:>10} ", avg_micros);
        }

        println!("│");
    }

    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}

fn benchmark_throughput(samples: &[TestSample]) {
    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│                    Normalization Throughput                      │");
    println!("├──────────────────────────────────────────────────────────────────┤");
    println!("│ Sample                     │  Size (KB)  │  Throughput (MB/s)  │");
    println!("├──────────────────────────────────────────────────────────────────┤");

    let normalizer = TextNormalizer::default();

    for sample in samples {
        let size_kb = sample.content.len() as f64 / 1024.0;
        let iterations = if size_kb < 10.0 { 10_000 } else { 1_000 };

        let start = Instant::now();

        for _ in 0..iterations {
            let _ = normalizer.normalize(&sample.content, Some(ContentType::Plain));
        }

        let duration = start.elapsed();
        let total_bytes = (sample.content.len() * iterations) as f64;
        let throughput_mbps = (total_bytes / duration.as_secs_f64()) / (1024.0 * 1024.0);

        println!(
            "│ {:27} │  {:>8.2}   │      {:>8.2}        │",
            sample.name, size_kb, throughput_mbps
        );
    }

    println!("└──────────────────────────────────────────────────────────────────┘");
}

fn benchmark_compression_ratios(samples: &[TestSample]) {
    println!("┌───────────────────────────────────────────────────────────────────────┐");
    println!("│                      Storage Reduction Analysis                       │");
    println!("├───────────────────────────────────────────────────────────────────────┤");
    println!("│ Sample                │ Original │ Normalized │ Reduction │ Savings  │");
    println!("│                       │  (bytes) │   (bytes)  │     (%)   │  (bytes) │");
    println!("├───────────────────────────────────────────────────────────────────────┤");

    let normalizer = TextNormalizer::default();

    for sample in samples {
        let result = normalizer.normalize(&sample.content, Some(ContentType::Plain));

        let reduction_pct = (result.metadata.removed_bytes as f64
            / result.metadata.original_size as f64)
            * 100.0;

        println!(
            "│ {:22} │ {:>8} │ {:>10} │ {:>8.1}  │ {:>8} │",
            sample.name,
            result.metadata.original_size,
            result.metadata.normalized_size,
            reduction_pct,
            result.metadata.removed_bytes
        );
    }

    println!("└───────────────────────────────────────────────────────────────────────┘");
}

fn benchmark_hash_performance(samples: &[TestSample]) {
    println!("┌───────────────────────────────────────────────────────┐");
    println!("│           Content Hashing Performance                │");
    println!("├───────────────────────────────────────────────────────┤");
    println!("│ Sample                │ Size (KB) │ Throughput (MB/s) │");
    println!("├───────────────────────────────────────────────────────┤");

    let normalizer = TextNormalizer::default();

    for sample in samples {
        let size_kb = sample.content.len() as f64 / 1024.0;
        let iterations = if size_kb < 10.0 { 10_000 } else { 1_000 };

        // First normalize to get consistent text
        let normalized = normalizer.normalize(&sample.content, Some(ContentType::Plain));

        let start = Instant::now();

        for _ in 0..iterations {
            // Hash is already computed in normalize, but we'll measure it separately
            use vectorizer::normalization::ContentHashCalculator;
            let hasher = ContentHashCalculator::new();
            let _ = hasher.hash(&normalized.text);
        }

        let duration = start.elapsed();
        let total_bytes = (normalized.text.len() * iterations) as f64;
        let throughput_mbps = (total_bytes / duration.as_secs_f64()) / (1024.0 * 1024.0);

        println!(
            "│ {:22} │ {:>8.2}  │      {:>8.2}      │",
            sample.name, size_kb, throughput_mbps
        );
    }

    println!("└───────────────────────────────────────────────────────┘");
}

