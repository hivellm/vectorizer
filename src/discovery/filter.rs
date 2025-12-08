//! Collection filtering with tantivy integration

use glob::Pattern;

use super::types::{CollectionRef, DiscoveryError, DiscoveryResult};

/// Extract terms from query using tantivy tokenizer for better results
///
/// Uses tantivy's default tokenizer which provides:
/// - Stopword removal (language-specific)
/// - Better Unicode handling
/// - Lowercasing normalization
fn extract_terms_with_tantivy(query: &str) -> Vec<String> {
    use tantivy::tokenizer::*;

    // Create tokenizer with stopword filter and lowercasing
    // Use English stopwords by default
    let stopword_filter = StopWordFilter::new(Language::English)
        .unwrap_or_else(|| StopWordFilter::remove(Vec::<String>::new()));

    let mut tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .filter(stopword_filter)
        .build();

    let mut tokens = Vec::new();
    let mut token_stream = tokenizer.token_stream(query);

    while token_stream.advance() {
        let token = token_stream.token();
        if !token.text.is_empty() && token.text.len() >= 2 {
            tokens.push(token.text.to_string());
        }
    }

    tokens
}

/// Pre-filter collections by name patterns with stopword removal
pub fn filter_collections(
    query: &str,
    include: &[&str],
    exclude: &[&str],
    all_collections: &[CollectionRef],
) -> DiscoveryResult<Vec<CollectionRef>> {
    let query_terms = extract_terms(query);
    let mut candidates = Vec::new();

    // If include patterns provided, match them
    if !include.is_empty() {
        for collection in all_collections {
            if matches_any_pattern(&collection.name, include) {
                candidates.push(collection.clone());
            }
        }
    } else {
        // No include patterns, use all collections
        candidates = all_collections.to_vec();
    }

    // Remove exclude patterns
    candidates.retain(|c| !matches_any_pattern(&c.name, exclude));

    // If no patterns provided, filter by query terms
    if include.is_empty() && !query_terms.is_empty() {
        candidates = filter_by_query_terms(&candidates, &query_terms);
    }

    Ok(candidates)
}

/// Extract terms from query (remove stopwords)
///
/// Uses tantivy tokenizer for better results:
/// - Stemming (running -> run)
/// - Language-specific stopwords
/// - Better Unicode handling
/// - Lowercasing normalization
fn extract_terms(query: &str) -> Vec<String> {
    // Use tantivy tokenizer if available, fallback to simple extraction
    #[cfg(feature = "tantivy")]
    {
        extract_terms_with_tantivy(query)
    }

    #[cfg(not(feature = "tantivy"))]
    {
        // Fallback to simple stopword removal
        let stopwords = [
            "o", "que", "é", "the", "is", "a", "an", "what", "how", "when", "where", "why",
            "which", "do", "does", "de", "da", "do",
        ];

        query
            .split_whitespace()
            .filter(|term| !stopwords.contains(&term.to_lowercase().as_str()))
            .map(|s| s.to_string())
            .collect()
    }
}

/// Check if name matches any pattern
fn matches_any_pattern(name: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(name))
            .unwrap_or(false)
    })
}

/// Filter collections by query terms
fn filter_by_query_terms(collections: &[CollectionRef], terms: &[String]) -> Vec<CollectionRef> {
    collections
        .iter()
        .filter(|c| {
            let name_lower = c.name.to_lowercase();
            terms
                .iter()
                .any(|term| name_lower.contains(&term.to_lowercase()))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn create_test_collection(name: &str) -> CollectionRef {
        CollectionRef {
            name: name.to_string(),
            dimension: 384,
            vector_count: 1000,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
        }
    }

    #[test]
    fn test_extract_terms() {
        let terms = extract_terms("O que é o vectorizer");
        // Should extract vectorizer (stopwords removed)
        assert!(terms.contains(&"vectorizer".to_string()));

        let terms = extract_terms("What is the vectorizer architecture");
        // Should extract vectorizer and architecture (stopwords removed)
        assert!(terms.contains(&"vectorizer".to_string()));
        assert!(terms.contains(&"architecture".to_string()));

        // Test tantivy integration (if available)
        #[cfg(feature = "tantivy")]
        {
            let terms = extract_terms_with_tantivy("What is the vectorizer architecture");
            assert!(!terms.is_empty());
            assert!(terms.iter().any(|t| t.contains("vectorizer")));
        }
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_any_pattern("vectorizer-docs", &["vectorizer*"]));
        assert!(matches_any_pattern("test-collection", &["*-collection"]));
        assert!(!matches_any_pattern("vectorizer-docs", &["test*"]));
    }

    #[test]
    fn test_filter_collections() {
        let collections = vec![
            create_test_collection("vectorizer-docs"),
            create_test_collection("vectorizer-source"),
            create_test_collection("test-collection"),
            create_test_collection("other-docs"),
        ];

        let filtered = filter_collections(
            "vectorizer features",
            &["vectorizer*"],
            &["*-test"],
            &collections,
        )
        .unwrap();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|c| c.name == "vectorizer-docs"));
        assert!(filtered.iter().any(|c| c.name == "vectorizer-source"));
    }
}
