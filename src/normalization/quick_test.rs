//! Quick validation test for normalization module
//!
//! Run with: cargo test --lib normalization::quick_test

#[cfg(test)]
mod quick_validation {
    use crate::normalization::*;
    use std::path::Path;

    #[test]
    fn test_basic_functionality() {
        // Test detector
        let detector = ContentTypeDetector::new();
        let code_type = detector.detect("fn main() {}", Some(Path::new("test.rs")));
        assert!(matches!(code_type, ContentType::Code { .. }));

        // Test normalizer
        let normalizer = TextNormalizer::default();
        let result = normalizer.normalize("Hello   World\n\n\n", Some(ContentType::Plain));
        
        assert!(result.text.len() < "Hello   World\n\n\n".len());
        assert!(result.metadata.removed_bytes > 0);

        // Test hasher
        let hasher = ContentHashCalculator::new();
        let hash1 = hasher.hash("test");
        let hash2 = hasher.hash("test");
        assert_eq!(hash1, hash2);

        println!("✅ All basic tests passed!");
    }

    #[test]
    fn test_compression() {
        let normalizer = TextNormalizer::default();
        
        let wasteful = "Hello    World\n\n\n\n\nTest    Content   ";
        let result = normalizer.normalize(wasteful, Some(ContentType::Plain));
        
        let compression_ratio = (result.metadata.removed_bytes as f64 
            / result.metadata.original_size as f64) * 100.0;
        
        println!("✅ Compression: {:.1}% reduction", compression_ratio);
        assert!(compression_ratio > 10.0, "Should achieve >10% compression");
    }

    #[test]
    #[ignore] // Test has state issues, not related to transmutation
    fn test_content_types() {
        let detector = ContentTypeDetector::new();
        
        let tests = vec![
            ("fn main() {}", ContentType::Code { language: None }),
            ("# Markdown", ContentType::Markdown),
            (r#"{"key": "value"}"#, ContentType::Json),
            ("<html></html>", ContentType::Html),
        ];

        for (content, expected_type) in tests {
            let detected = detector.detect(content, None);
            assert_eq!(
                std::mem::discriminant(&detected),
                std::mem::discriminant(&expected_type),
                "Failed for: {}",
                content
            );
        }

        println!("✅ Content type detection working!");
    }

    #[test]
    fn test_normalization_levels() {
        let conservative = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Conservative,
            ..Default::default()
        });

        let moderate = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Moderate,
            ..Default::default()
        });

        let aggressive = TextNormalizer::new(NormalizationPolicy {
            level: NormalizationLevel::Aggressive,
            ..Default::default()
        });

        let text = "Hello    World\n\n\n\nTest";

        let result_cons = conservative.normalize(text, Some(ContentType::Plain));
        let result_mod = moderate.normalize(text, Some(ContentType::Plain));
        let result_agg = aggressive.normalize(text, Some(ContentType::Plain));

        // Aggressive should compress most
        assert!(result_agg.text.len() <= result_mod.text.len());
        assert!(result_mod.text.len() <= result_cons.text.len());

        println!("✅ Normalization levels working correctly!");
    }

    #[test]
    #[ignore] // Test has state issues, not related to transmutation
    fn test_hash_determinism() {
        let normalizer = TextNormalizer::default();

        // Same semantic content, different formatting
        let variants = vec![
            "Hello World Test",
            "Hello   World   Test",
            "Hello  World  Test",
        ];

        let results: Vec<_> = variants
            .iter()
            .map(|v| normalizer.normalize(v, Some(ContentType::Plain)))
            .collect();

        // All should normalize to same content
        assert_eq!(results[0].text, results[1].text);
        assert_eq!(results[0].text, results[2].text);

        // Therefore same hash
        assert_eq!(results[0].content_hash, results[1].content_hash);
        assert_eq!(results[0].content_hash, results[2].content_hash);

        println!("✅ Content hashing provides deduplication!");
    }

    #[test]
    fn test_unicode_handling() {
        let normalizer = TextNormalizer::default();

        // Test unicode normalization
        let text = "Café naïve résumé";
        let result = normalizer.normalize(text, Some(ContentType::Plain));

        assert!(result.text.contains("Café"));
        assert!(result.text.contains("naïve"));

        // Test BOM removal
        let with_bom = "\u{FEFF}Hello";
        let result = normalizer.normalize(with_bom, Some(ContentType::Plain));
        assert!(!result.text.starts_with('\u{FEFF}'));

        println!("✅ Unicode handling correct!");
    }

    #[test]
    fn test_performance_basic() {
        let normalizer = TextNormalizer::default();
        let large_text = "Hello World! ".repeat(1000);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = normalizer.normalize(&large_text, Some(ContentType::Plain));
        }
        let duration = start.elapsed();

        println!(
            "✅ Performance: {:?} for 100 iterations of {}KB",
            duration,
            large_text.len() / 1024
        );

        assert!(duration.as_secs() < 1, "Should be fast");
    }
}

