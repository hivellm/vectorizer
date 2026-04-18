//! Unit tests for AdvancedSearchEngine — extracted from `src/search/advanced_search/mod.rs` via the
//! `#[path]` attribute (phase3 monolith test-extraction).

use super::*;


#[test]
fn test_search_config_default() {
    let config = SearchConfig::default();
    assert!(config.modes.enable_text_search);
    assert!(config.modes.enable_vector_search);
    assert!(config.modes.enable_hybrid_search);
    assert!(!config.modes.enable_semantic_search);
    assert!(!config.modes.enable_fuzzy_search);
    assert!(!config.modes.enable_faceted_search);
}

#[test]
fn test_search_document_creation() {
    let document = SearchDocument {
        id: "doc1".to_string(),
        title: "Test Document".to_string(),
        content: "This is a test document".to_string(),
        description: Some("A test document".to_string()),
        tags: vec!["test".to_string(), "document".to_string()],
        category: Some("test".to_string()),
        metadata: HashMap::new(),
        vector: Some(vec![0.1, 0.2, 0.3]),
        score: 0.0,
        timestamp: 1234567890,
        language: Some("en".to_string()),
    };

    assert_eq!(document.id, "doc1");
    assert_eq!(document.title, "Test Document");
    assert_eq!(document.content, "This is a test document");
    assert_eq!(document.tags.len(), 2);
    assert_eq!(document.vector, Some(vec![0.1, 0.2, 0.3]));
}

#[test]
fn test_search_query_creation() {
    let query = SearchQuery {
        query: "test query".to_string(),
        mode: SearchMode::Text,
        collections: vec!["test".to_string()],
        max_results: 10,
        offset: 0,
        filters: HashMap::new(),
        sort: None,
        facets: vec![],
        highlight: None,
    };

    assert_eq!(query.query, "test query");
    assert_eq!(query.mode, SearchMode::Text);
    assert_eq!(query.collections.len(), 1);
    assert_eq!(query.max_results, 10);
}

#[test]
fn test_score_breakdown() {
    let breakdown = ScoreBreakdown {
        text_relevance: 0.8,
        vector_similarity: 0.7,
        recency: 0.6,
        popularity: 0.5,
        quality: 0.9,
        boost: 1.2,
        final_score: 0.8,
    };

    assert_eq!(breakdown.text_relevance, 0.8);
    assert_eq!(breakdown.vector_similarity, 0.7);
    assert_eq!(breakdown.final_score, 0.8);
}
