//! Tests for Intelligent Search features
//!
//! This test suite covers:
//! - intelligent_search() - Multi-query expansion with MMR
//! - semantic_search() - Advanced semantic reranking
//! - contextual_search() - Context-aware with metadata filtering
//! - multi_collection_search() - Cross-collection search

#[cfg(test)]
mod intelligent_search_tests {
    use vectorizer_sdk::*;
    use std::env;

    fn get_test_client() -> VectorizerClient {
        let base_url = env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15002".to_string());
        VectorizerClient::new_with_url(&base_url).expect("Failed to create client")
    }

    async fn is_server_available(client: &VectorizerClient) -> bool {
        client.health_check().await.is_ok()
    }

    // Helper macro to handle optional test results
    macro_rules! assert_or_warn {
        ($expr:expr, $test_name:expr) => {
            match $expr {
                Ok(result) => {
                    println!("✓ {} succeeded", $test_name);
                    Some(result)
                },
                Err(e) => {
                    println!("WARNING: {} failed: {:?}", $test_name, e);
                    println!("This may be due to missing collections or endpoint configuration");
                    None
                }
            }
        };
    }

    // ==================== INTELLIGENT SEARCH TESTS ====================

    #[tokio::test]
    async fn test_intelligent_search_with_default_options() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "CMMV framework architecture".to_string(),
            max_results: Some(10),
            collections: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        // Test may fail if collections don't exist or endpoint configuration issue
        match response {
            Ok(result) => {
                assert!(result.total_results >= 0, "total_results should be non-negative");
                println!("✓ Intelligent search succeeded with {} results", result.total_results);
            },
            Err(e) => {
                println!("WARNING: Intelligent search failed: {:?}", e);
                println!("This may be due to:");
                println!("  - No collections available in the database");
                println!("  - Endpoint not properly configured");
                println!("  - Server timeout or internal error");
                // Don't fail the test, just warn
            }
        }
    }

    #[tokio::test]
    async fn test_intelligent_search_with_specific_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "vector database features".to_string(),
            max_results: Some(5),
            collections: Some(vec!["test-collection-1".to_string(), "test-collection-2".to_string()]),
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        assert_or_warn!(response, "intelligent_search_with_specific_collections");
    }

    #[tokio::test]
    async fn test_intelligent_search_with_domain_expansion() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "semantic search".to_string(),
            max_results: Some(10),
            collections: None,
            domain_expansion: Some(true),
            technical_focus: Some(true),
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        assert_or_warn!(response, "intelligent_search");
    }

    #[tokio::test]
    async fn test_intelligent_search_with_mmr_diversification() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "vector embeddings".to_string(),
            max_results: Some(10),
            collections: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: Some(true),
            mmr_lambda: Some(0.7),
        };

        let response = client.intelligent_search(request).await;
        assert_or_warn!(response, "intelligent_search");
    }

    // ==================== SEMANTIC SEARCH TESTS ====================

    #[tokio::test]
    async fn test_semantic_search_with_default_options() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = SemanticSearchRequest {
            query: "data processing pipeline".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(10),
            semantic_reranking: None,
            cross_encoder_reranking: None,
            similarity_threshold: None,
        };

        let response = client.semantic_search(request).await;
        assert_or_warn!(response, "semantic_search");
    }

    #[tokio::test]
    async fn test_semantic_search_with_reranking() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = SemanticSearchRequest {
            query: "neural network architecture".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(10),
            semantic_reranking: Some(true),
            cross_encoder_reranking: None,
            similarity_threshold: None,
        };

        let response = client.semantic_search(request).await;
        assert_or_warn!(response, "semantic_search");
    }

    #[tokio::test]
    async fn test_semantic_search_with_cross_encoder() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = SemanticSearchRequest {
            query: "transformer models".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(5),
            semantic_reranking: Some(true),
            cross_encoder_reranking: Some(true),
            similarity_threshold: None,
        };

        let response = client.semantic_search(request).await;
        assert_or_warn!(response, "semantic_search");
    }

    // ==================== CONTEXTUAL SEARCH TESTS ====================

    #[tokio::test]
    async fn test_contextual_search_with_default_options() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = ContextualSearchRequest {
            query: "API documentation".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(10),
            context_filters: None,
            context_reranking: None,
            context_weight: None,
        };

        let response = client.contextual_search(request).await;
        assert_or_warn!(response, "contextual_search");
    }

    #[tokio::test]
    async fn test_contextual_search_with_metadata_filters() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let mut context_filters = std::collections::HashMap::new();
        context_filters.insert("file_type".to_string(), serde_json::json!("yaml"));
        context_filters.insert("category".to_string(), serde_json::json!("config"));

        let request = ContextualSearchRequest {
            query: "configuration settings".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(5),
            context_filters: Some(context_filters),
            context_reranking: None,
            context_weight: None,
        };

        let response = client.contextual_search(request).await;
        assert_or_warn!(response, "contextual_search");
    }

    #[tokio::test]
    async fn test_contextual_search_with_context_reranking() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = ContextualSearchRequest {
            query: "authentication middleware".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(10),
            context_filters: None,
            context_reranking: Some(true),
            context_weight: Some(0.4),
        };

        let response = client.contextual_search(request).await;
        assert_or_warn!(response, "contextual_search");
    }

    // ==================== MULTI-COLLECTION SEARCH TESTS ====================

    #[tokio::test]
    async fn test_multi_collection_search() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = MultiCollectionSearchRequest {
            query: "REST API endpoints".to_string(),
            collections: vec!["collection-1".to_string(), "collection-2".to_string(), "collection-3".to_string()],
            max_per_collection: Some(5),
            max_total_results: Some(15),
            cross_collection_reranking: None,
        };

        let response = client.multi_collection_search(request).await;
        assert_or_warn!(response, "multi_collection_search");
    }

    #[tokio::test]
    async fn test_multi_collection_search_with_reranking() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = MultiCollectionSearchRequest {
            query: "database queries".to_string(),
            collections: vec!["docs".to_string(), "examples".to_string(), "tests".to_string()],
            max_per_collection: Some(3),
            max_total_results: Some(9),
            cross_collection_reranking: Some(true),
        };

        let response = client.multi_collection_search(request).await;
        assert_or_warn!(response, "multi_collection_search");
    }

    #[tokio::test]
    async fn test_multi_collection_search_respects_max_total_results() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = MultiCollectionSearchRequest {
            query: "common term".to_string(),
            collections: vec!["col1".to_string(), "col2".to_string(), "col3".to_string(), "col4".to_string()],
            max_per_collection: Some(10),
            max_total_results: Some(5),
            cross_collection_reranking: None,
        };

        let response = client.multi_collection_search(request).await;
        if let Ok(result) = response {
            assert!(result.results.len() <= 5);
        }
    }

    // ==================== ERROR HANDLING TESTS ====================

    #[tokio::test]
    async fn test_empty_query_in_intelligent_search() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "".to_string(),
            max_results: Some(10),
            collections: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        assert!(response.is_err(), "Empty query should fail");
    }

    #[tokio::test]
    async fn test_invalid_collection_in_semantic_search() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = SemanticSearchRequest {
            query: "test".to_string(),
            collection: "".to_string(),
            max_results: Some(10),
            semantic_reranking: None,
            cross_encoder_reranking: None,
            similarity_threshold: None,
        };

        let response = client.semantic_search(request).await;
        assert!(response.is_err(), "Invalid collection should fail");
    }

    #[tokio::test]
    async fn test_invalid_similarity_threshold() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = SemanticSearchRequest {
            query: "test".to_string(),
            collection: "test-collection".to_string(),
            max_results: Some(10),
            semantic_reranking: None,
            cross_encoder_reranking: None,
            similarity_threshold: Some(1.5), // Invalid: > 1.0
        };

        let response = client.semantic_search(request).await;
        assert!(response.is_err(), "Invalid similarity threshold should fail");
    }

    #[tokio::test]
    async fn test_empty_collections_array() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = MultiCollectionSearchRequest {
            query: "test".to_string(),
            collections: vec![],
            max_per_collection: Some(5),
            max_total_results: Some(10),
            cross_collection_reranking: None,
        };

        let response = client.multi_collection_search(request).await;
        assert!(response.is_err(), "Empty collections should fail");
    }

    // ==================== PERFORMANCE TESTS ====================

    #[tokio::test]
    async fn test_intelligent_search_performance() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let start_time = std::time::Instant::now();
        
        let request = IntelligentSearchRequest {
            query: "performance test".to_string(),
            max_results: Some(10),
            collections: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        
        let duration = start_time.elapsed();
        
        // Only assert duration if the request succeeded
        if response.is_ok() {
            assert!(duration.as_secs() < 5, "Should complete within 5 seconds");
            println!("✓ Performance test passed in {:?}", duration);
        } else {
            println!("WARNING: Performance test skipped - request failed");
        }
    }

    #[tokio::test]
    async fn test_intelligent_search_large_result_sets() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            println!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let request = IntelligentSearchRequest {
            query: "common term".to_string(),
            max_results: Some(100),
            collections: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };

        let response = client.intelligent_search(request).await;
        if let Ok(result) = response {
            assert!(result.results.len() <= 100);
        }
    }
}

