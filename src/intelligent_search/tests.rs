//! Comprehensive test suite for intelligent search implementation
//! 
//! This module contains unit tests, integration tests, and performance tests
//! for the intelligent search engine.

use crate::intelligent_search::*;
use crate::intelligent_search::query_generator::QueryGenerator;
use crate::intelligent_search::simple_search_engine::SimpleSearchEngine;
use crate::intelligent_search::mmr_diversifier::MMRDiversifier;
use crate::intelligent_search::context_formatter::ContextFormatter;
use std::collections::HashMap;

/// Test data setup helper
fn create_test_documents() -> Vec<Document> {
    let mut metadata1 = HashMap::new();
    metadata1.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
    metadata1.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
    
    let mut metadata2 = HashMap::new();
    metadata2.insert("author".to_string(), serde_json::Value::String("Jane Smith".to_string()));
    metadata2.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
    
    vec![
        Document {
            id: "doc1".to_string(),
            content: "Vectorizer is a high-performance vector database written in Rust. It provides semantic search capabilities and supports multiple embedding models including BM25, TF-IDF, and BERT.".to_string(),
            collection: "vectorizer-docs".to_string(),
            metadata: metadata1,
        },
        Document {
            id: "doc2".to_string(),
            content: "CMMV is a Contract Model Model View framework for building scalable TypeScript applications. It uses contracts to define interfaces and automatically generates APIs.".to_string(),
            collection: "cmmv-docs".to_string(),
            metadata: metadata2,
        },
        Document {
            id: "doc3".to_string(),
            content: "HNSW (Hierarchical Navigable Small World) is a graph-based algorithm for approximate nearest neighbor search. It's used in vector databases for fast similarity search.".to_string(),
            collection: "algorithms-docs".to_string(),
            metadata: HashMap::new(),
        },
        Document {
            id: "doc4".to_string(),
            content: "Vectorizer performance benchmarks show excellent results with sub-millisecond search times and high accuracy across different embedding models.".to_string(),
            collection: "vectorizer-docs".to_string(),
            metadata: HashMap::new(),
        },
        Document {
            id: "doc5".to_string(),
            content: "CMMV framework provides authentication, caching, HTTP APIs, and repository integration out of the box for rapid application development.".to_string(),
            collection: "cmmv-docs".to_string(),
            metadata: HashMap::new(),
        },
    ]
}

#[cfg(test)]
mod query_generator_tests {
    use super::*;

    #[test]
    fn test_query_generator_creation() {
        let generator = QueryGenerator::new(5);
        assert_eq!(generator.max_queries, 5);
    }

    #[test]
    fn test_generate_queries_basic() {
        let generator = QueryGenerator::new(8);
        let queries = generator.generate_queries("vectorizer performance");
        
        assert!(!queries.is_empty());
        assert!(queries.contains(&"vectorizer performance".to_string()));
        assert!(queries.len() <= 8);
    }

    #[test]
    fn test_generate_queries_with_technical_terms() {
        let generator = QueryGenerator::new(10);
        let queries = generator.generate_queries("vectorizer API documentation");
        
        // Should contain the original query
        assert!(queries.contains(&"vectorizer API documentation".to_string()));
        
        // Should contain technical expansions
        assert!(queries.iter().any(|q| q.contains("vectorizer documentation")));
        assert!(queries.iter().any(|q| q.contains("vectorizer API")));
        assert!(queries.iter().any(|q| q.contains("vectorizer features")));
    }

    #[test]
    fn test_generate_queries_domain_expansion() {
        let generator = QueryGenerator::new(10);
        let queries = generator.generate_queries("cmmv");
        
        // Should contain domain-specific expansions
        assert!(queries.iter().any(|q| q.contains("CMMV framework")));
        assert!(queries.iter().any(|q| q.contains("Contract Model View")));
        assert!(queries.iter().any(|q| q.contains("TypeScript framework")));
    }

    #[test]
    fn test_generate_queries_deduplication() {
        let generator = QueryGenerator::new(10);
        let queries = generator.generate_queries("vectorizer vectorizer");
        
        // Should not contain duplicates
        let unique_queries: std::collections::HashSet<&String> = queries.iter().collect();
        assert_eq!(unique_queries.len(), queries.len());
    }

    #[test]
    fn test_generate_queries_max_limit() {
        let generator = QueryGenerator::new(3);
        let queries = generator.generate_queries("vectorizer performance API documentation benchmarks");
        
        assert!(queries.len() <= 3);
    }

    #[test]
    fn test_extract_technical_terms() {
        let generator = QueryGenerator::new(8);
        let terms = generator.extract_technical_terms("vectorizer API documentation performance");
        
        assert!(terms.contains(&"vectorizer".to_string()));
        assert!(terms.contains(&"api".to_string()));
        assert!(terms.contains(&"documentation".to_string()));
        assert!(terms.contains(&"performance".to_string()));
    }

    #[test]
    fn test_is_technical_term() {
        let generator = QueryGenerator::new(8);
        
        assert!(generator.is_technical_term("api"));
        assert!(generator.is_technical_term("vectorizer"));
        assert!(generator.is_technical_term("performance"));
        assert!(generator.is_technical_term("database"));
        assert!(!generator.is_technical_term("the"));
        assert!(!generator.is_technical_term("a"));
        assert!(!generator.is_technical_term("is"));
    }
}

#[cfg(test)]
mod simple_search_engine_tests {
    use super::*;

    #[tokio::test]
    async fn test_search_engine_creation() {
        let engine = SimpleSearchEngine::new();
        assert_eq!(engine.get_document_count(), 0);
    }

    #[tokio::test]
    async fn test_add_documents() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        
        let result = engine.add_documents(documents).await;
        assert!(result.is_ok());
        assert_eq!(engine.get_document_count(), 5);
    }

    #[tokio::test]
    async fn test_search_basic() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let results = engine.search("vectorizer", vec!["vectorizer-docs".to_string()], 10).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].collection, "vectorizer-docs");
    }

    #[tokio::test]
    async fn test_search_multiple_collections() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let results = engine.search("framework", vec!["cmmv-docs".to_string(), "vectorizer-docs".to_string()], 10).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let results = engine.search("nonexistent", vec!["vectorizer-docs".to_string()], 10).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_max_results_limit() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let results = engine.search("vectorizer", vec![], 2).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn test_calculate_score() {
        let engine = SimpleSearchEngine::new();
        
        let score = engine.calculate_score("vectorizer is a vector database", "vectorizer");
        assert!(score > 0.0);
        
        let score2 = engine.calculate_score("completely different content", "vectorizer");
        assert_eq!(score2, 0.0);
    }

    #[test]
    fn test_calculate_term_frequency() {
        let engine = SimpleSearchEngine::new();
        
        let tf_score = engine.calculate_term_frequency("vectorizer is a vector database", "vectorizer database");
        assert!(tf_score > 0.0);
    }

    #[test]
    fn test_calculate_collection_relevance() {
        let engine = SimpleSearchEngine::new();
        
        let relevance_score = engine.calculate_collection_relevance("vectorizer-docs", "vectorizer documentation");
        assert!(relevance_score > 0.0);
    }

    #[tokio::test]
    async fn test_get_documents_by_collection() {
        let mut engine = SimpleSearchEngine::new();
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let vectorizer_docs = engine.get_documents_by_collection("vectorizer-docs");
        assert_eq!(vectorizer_docs.len(), 2);
        
        let cmmv_docs = engine.get_documents_by_collection("cmmv-docs");
        assert_eq!(cmmv_docs.len(), 2);
        
        let algorithms_docs = engine.get_documents_by_collection("algorithms-docs");
        assert_eq!(algorithms_docs.len(), 1);
    }
}

#[cfg(test)]
mod mmr_diversifier_tests {
    use super::*;

    fn create_test_results() -> Vec<IntelligentSearchResult> {
        vec![
            IntelligentSearchResult {
                content: "Vectorizer is a high-performance vector database".to_string(),
                score: 0.9,
                collection: "docs".to_string(),
                doc_id: "doc1".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
            IntelligentSearchResult {
                content: "Vectorizer provides excellent performance benchmarks".to_string(),
                score: 0.8,
                collection: "docs".to_string(),
                doc_id: "doc2".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
            IntelligentSearchResult {
                content: "CMMV is a Contract Model View framework".to_string(),
                score: 0.7,
                collection: "docs".to_string(),
                doc_id: "doc3".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
            IntelligentSearchResult {
                content: "HNSW algorithm for approximate nearest neighbor search".to_string(),
                score: 0.6,
                collection: "docs".to_string(),
                doc_id: "doc4".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
        ]
    }

    #[test]
    fn test_mmr_diversifier_creation() {
        let diversifier = MMRDiversifier::new(0.7);
        assert_eq!(diversifier.get_lambda(), 0.7);
    }

    #[test]
    fn test_diversify_empty_results() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![];
        let diversified = diversifier.diversify(&results, 5);
        assert!(diversified.is_empty());
    }

    #[test]
    fn test_diversify_single_result() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![
            IntelligentSearchResult {
                content: "Test content".to_string(),
                score: 0.9,
                collection: "test".to_string(),
                doc_id: "doc1".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
        ];
        let diversified = diversifier.diversify(&results, 5);
        assert_eq!(diversified.len(), 1);
    }

    #[test]
    fn test_diversify_multiple_results() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = create_test_results();
        let diversified = diversifier.diversify(&results, 3);
        
        assert_eq!(diversified.len(), 3);
        // First result should be highest relevance
        assert_eq!(diversified[0].score, 0.9);
    }

    #[test]
    fn test_diversify_max_results_limit() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = create_test_results();
        let diversified = diversifier.diversify(&results, 2);
        
        assert_eq!(diversified.len(), 2);
    }

    #[test]
    fn test_calculate_content_similarity() {
        let diversifier = MMRDiversifier::new(0.7);
        
        // Identical content
        let sim1 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "vectorizer is a vector database"
        );
        assert_eq!(sim1, 1.0);
        
        // Similar content
        let sim2 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "vectorizer database for vectors"
        );
        assert!(sim2 > 0.0 && sim2 < 1.0);
        
        // Different content
        let sim3 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "completely different content"
        );
        assert_eq!(sim3, 0.0);
    }

    #[test]
    fn test_lambda_effect() {
        let relevance_focused = MMRDiversifier::new(0.9);
        let diversity_focused = MMRDiversifier::new(0.1);
        
        let results = create_test_results();
        
        let relevance_results = relevance_focused.diversify(&results, 2);
        let diversity_results = diversity_focused.diversify(&results, 2);
        
        assert_eq!(relevance_results.len(), 2);
        assert_eq!(diversity_results.len(), 2);
    }
}

#[cfg(test)]
mod context_formatter_tests {
    use super::*;

    fn create_test_results() -> Vec<IntelligentSearchResult> {
        vec![
            IntelligentSearchResult {
                content: "Vectorizer is a high-performance vector database written in Rust. It provides semantic search capabilities and supports multiple embedding models.".to_string(),
                score: 0.9,
                collection: "vectorizer-docs".to_string(),
                doc_id: "doc1".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
            IntelligentSearchResult {
                content: "CMMV is a Contract Model Model View framework for building scalable TypeScript applications.".to_string(),
                score: 0.8,
                collection: "cmmv-docs".to_string(),
                doc_id: "doc2".to_string(),
                metadata: HashMap::new(),
                score_breakdown: None,
            },
        ]
    }

    #[test]
    fn test_context_formatter_creation() {
        let formatter = ContextFormatter::new(400, 5, false);
        assert_eq!(formatter.get_max_content_length(), 400);
        assert_eq!(formatter.get_max_lines_per_result(), 5);
        assert!(!formatter.is_metadata_included());
    }

    #[test]
    fn test_format_context_empty() {
        let formatter = ContextFormatter::default();
        let results = vec![];
        let context = formatter.format_context(&results, "test query");
        assert!(context.is_empty());
    }

    #[test]
    fn test_format_context_with_results() {
        let formatter = ContextFormatter::default();
        let results = create_test_results();
        let context = formatter.format_context(&results, "vectorizer");
        
        assert!(context.contains("vectorizer-docs"));
        assert!(context.contains("cmmv-docs"));
        assert!(context.contains("0.900"));
        assert!(context.contains("0.800"));
    }

    #[test]
    fn test_format_single_result() {
        let formatter = ContextFormatter::default();
        let result = &create_test_results()[0];
        
        let formatted = formatter.format_single_result(result, "vectorizer");
        assert!(formatted.contains("vectorizer-docs"));
        assert!(formatted.contains("0.900"));
        assert!(formatted.contains("Vectorizer"));
    }

    #[test]
    fn test_extract_relevant_lines() {
        let formatter = ContextFormatter::new(400, 3, false);
        let content = "This is about vectorizer.\nVectorizer is a vector database.\nThis is unrelated content.\nVectorizer performance is excellent.";
        let query = "vectorizer performance";
        
        let relevant_lines = formatter.extract_relevant_lines(content, query);
        assert!(!relevant_lines.is_empty());
        assert!(relevant_lines.len() <= 3);
    }

    #[test]
    fn test_truncate_content() {
        let formatter = ContextFormatter::new(50, 5, false);
        let content = "This is a very long content that should be truncated because it exceeds the maximum length";
        
        let truncated = formatter.truncate_content(content);
        assert!(truncated.len() <= 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_format_metadata() {
        let formatter = ContextFormatter::default();
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        metadata.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
        metadata.insert("active".to_string(), serde_json::Value::Bool(true));
        
        let formatted = formatter.format_metadata(&metadata);
        assert!(formatted.contains("author: John Doe"));
        assert!(formatted.contains("version: 1"));
        assert!(formatted.contains("active: true"));
    }

    #[test]
    fn test_format_enhanced_context() {
        let formatter = ContextFormatter::default();
        let results = create_test_results();
        
        let enhanced = formatter.format_enhanced_context(&results, "vectorizer", Some("Found 2 results"));
        assert!(enhanced.contains("Search Context: Found 2 results"));
        assert!(enhanced.contains("Query: vectorizer"));
        assert!(enhanced.contains("Results: 2 found"));
        assert!(enhanced.contains("---"));
    }
}

#[cfg(test)]
mod intelligent_search_engine_tests {
    use super::*;

    #[test]
    fn test_intelligent_search_engine_creation() {
        let config = IntelligentSearchConfig::default();
        let engine = IntelligentSearchEngine::new(config);
        // Engine should be created successfully
        assert!(true);
    }

    #[test]
    fn test_default_config() {
        let config = IntelligentSearchConfig::default();
        assert_eq!(config.max_queries, 8);
        assert!(config.domain_expansion);
        assert!(config.technical_focus);
        assert!(config.synonym_expansion);
        assert_eq!(config.similarity_threshold, 0.8);
        assert!(config.reranking_enabled);
        assert!(config.mmr_enabled);
        assert_eq!(config.mmr_lambda, 0.7);
    }

    #[tokio::test]
    async fn test_add_documents() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        
        let result = engine.add_documents(documents).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_basic() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let (results, metadata) = engine.search("vectorizer", None, Some(5)).await.unwrap();
        
        assert!(!results.is_empty());
        assert!(metadata.total_queries > 0);
        assert!(metadata.processing_time_ms > 0);
    }

    #[tokio::test]
    async fn test_search_with_collections() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let (results, metadata) = engine.search(
            "framework", 
            Some(vec!["cmmv-docs".to_string()]), 
            Some(3)
        ).await.unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(metadata.collections_searched, 1);
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let (results, metadata) = engine.search("nonexistent", None, Some(5)).await.unwrap();
        
        assert!(results.is_empty());
        assert!(metadata.total_queries > 0);
    }

    #[test]
    fn test_calculate_similarity() {
        let engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        
        let sim1 = engine.calculate_similarity("vectorizer database", "vectorizer database");
        assert_eq!(sim1, 1.0);
        
        let sim2 = engine.calculate_similarity("vectorizer database", "vectorizer");
        assert!(sim2 > 0.0 && sim2 < 1.0);
        
        let sim3 = engine.calculate_similarity("vectorizer database", "completely different");
        assert_eq!(sim3, 0.0);
    }

    #[test]
    fn test_config_management() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        
        let config = engine.get_config();
        assert_eq!(config.max_queries, 8);
        
        let mut new_config = IntelligentSearchConfig::default();
        new_config.max_queries = 10;
        engine.update_config(new_config);
        
        let updated_config = engine.get_config();
        assert_eq!(updated_config.max_queries, 10);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_search_workflow() {
        // Create engine with custom config
        let mut config = IntelligentSearchConfig::default();
        config.max_queries = 5;
        config.mmr_lambda = 0.8;
        
        let mut engine = IntelligentSearchEngine::new(config);
        
        // Add test documents
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        // Perform search
        let (results, metadata) = engine.search("vectorizer performance", None, Some(3)).await.unwrap();
        
        // Verify results
        assert!(!results.is_empty());
        assert!(results.len() <= 3);
        
        // Verify metadata
        assert!(metadata.total_queries > 0);
        assert!(metadata.collections_searched > 0);
        assert!(metadata.processing_time_ms > 0);
        
        // Verify result quality
        for result in &results {
            assert!(!result.content.is_empty());
            assert!(!result.collection.is_empty());
            assert!(!result.doc_id.is_empty());
            assert!(result.score > 0.0);
        }
    }

    #[tokio::test]
    async fn test_multiple_search_scenarios() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        // Test different query types
        let scenarios = vec![
            ("vectorizer", Some(vec!["vectorizer-docs".to_string()])),
            ("cmmv framework", Some(vec!["cmmv-docs".to_string()])),
            ("hnsw algorithm", Some(vec!["algorithms-docs".to_string()])),
            ("performance", None),
        ];
        
        for (query, collections) in scenarios {
            let (results, metadata) = engine.search(query, collections, Some(2)).await.unwrap();
            
            assert!(metadata.total_queries > 0);
            assert!(metadata.processing_time_ms > 0);
            
            if !results.is_empty() {
                assert!(results[0].score > 0.0);
            }
        }
    }

    #[tokio::test]
    async fn test_deduplication_effectiveness() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        
        // Add documents with similar content
        let mut documents = create_test_documents();
        documents.push(Document {
            id: "doc6".to_string(),
            content: "Vectorizer is a high-performance vector database written in Rust.".to_string(),
            collection: "vectorizer-docs".to_string(),
            metadata: HashMap::new(),
        });
        
        engine.add_documents(documents).await.unwrap();
        
        let (results, metadata) = engine.search("vectorizer", None, Some(10)).await.unwrap();
        
        // Should have deduplicated similar results
        assert!(metadata.results_after_dedup <= metadata.total_results_found);
        
        // Verify no exact duplicates
        let mut seen_contents = std::collections::HashSet::new();
        for result in &results {
            assert!(!seen_contents.contains(&result.content));
            seen_contents.insert(&result.content);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_search_performance() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        let documents = create_test_documents();
        engine.add_documents(documents).await.unwrap();
        
        let start = Instant::now();
        let (results, metadata) = engine.search("vectorizer performance", None, Some(5)).await.unwrap();
        let duration = start.elapsed();
        
        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 1000); // 1 second
        assert!(!results.is_empty());
        assert!(metadata.processing_time_ms > 0);
    }

    #[tokio::test]
    async fn test_large_document_set() {
        let mut engine = IntelligentSearchEngine::new(IntelligentSearchConfig::default());
        
        // Create a larger set of documents
        let mut documents = Vec::new();
        for i in 0..100 {
            documents.push(Document {
                id: format!("doc_{}", i),
                content: format!("Document {} about vectorizer and search algorithms", i),
                collection: "test-docs".to_string(),
                metadata: HashMap::new(),
            });
        }
        
        engine.add_documents(documents).await.unwrap();
        
        let start = Instant::now();
        let (results, metadata) = engine.search("vectorizer", None, Some(10)).await.unwrap();
        let duration = start.elapsed();
        
        // Should handle larger document sets efficiently
        assert!(duration.as_millis() < 2000); // 2 seconds
        assert!(!results.is_empty());
        assert_eq!(metadata.collections_searched, 1);
    }
}
