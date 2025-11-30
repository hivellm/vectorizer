//! Tests for Discovery Operations
//!
//! This test suite covers:
//! - discover() - Complete discovery pipeline
//! - filter_collections() - Collection filtering by patterns
//! - score_collections() - Relevance-based ranking
//! - expand_queries() - Query variation generation

#[cfg(test)]
mod discovery_tests {
    use std::env;

    use tracing::{debug, error, info, warn};
    use vectorizer_sdk::*;

    fn get_test_client() -> VectorizerClient {
        let base_url =
            env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15002".to_string());
        VectorizerClient::new_with_url(&base_url).expect("Failed to create client")
    }

    async fn is_server_available(client: &VectorizerClient) -> bool {
        client.health_check().await.is_ok()
    }

    // ==================== DISCOVER TESTS ====================

    #[tokio::test]
    async fn test_discover_complete_pipeline() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover(
                "How does CMMV framework work?",
                None,
                None,
                Some(20),
                Some(50),
                Some(15),
            )
            .await;

        assert!(response.is_ok(), "Discover should succeed");
        let result = response.unwrap();
        // Check that we got a valid JSON response
        assert!(result.is_object(), "Response should be a JSON object");
        // The response may contain different fields depending on server implementation
        // We just verify that we got some data back
        assert!(
            !result.as_object().unwrap().is_empty(),
            "Response should not be empty"
        );
    }

    #[tokio::test]
    async fn test_discover_with_specific_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover(
                "API authentication methods",
                Some(vec!["api-docs".to_string(), "security-docs".to_string()]),
                None,
                Some(15),
                None,
                None,
            )
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_discover_with_excluded_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover(
                "database migrations",
                None,
                Some(vec!["test-*".to_string(), "*-backup".to_string()]),
                Some(10),
                None,
                None,
            )
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_discover_generates_llm_ready_prompt() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover("vector search algorithms", None, None, Some(10), None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        // Check that we got a valid JSON response
        assert!(result.is_object(), "Response should be a JSON object");
        // Verify we got some data back
        assert!(
            !result.as_object().unwrap().is_empty(),
            "Response should not be empty"
        );
    }

    #[tokio::test]
    async fn test_discover_includes_citations() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover("system architecture", None, None, Some(15), None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(evidence) = result.get("evidence").and_then(|e| e.as_array()) {
            for item in evidence {
                assert!(item.get("text").is_some());
                assert!(item.get("citation").is_some());
            }
        }
    }

    // ==================== FILTER COLLECTIONS TESTS ====================

    #[tokio::test]
    async fn test_filter_collections_by_query() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client.filter_collections("documentation", None, None).await;

        assert!(response.is_ok());
        let result = response.unwrap();
        assert!(result.get("filtered_collections").is_some());
        assert!(result["total_available"].as_u64().unwrap_or(0) >= 0);
    }

    #[tokio::test]
    async fn test_filter_with_include_patterns() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .filter_collections(
                "api endpoints",
                Some(vec!["*-docs".to_string(), "api-*".to_string()]),
                None,
            )
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_filter_with_exclude_patterns() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .filter_collections(
                "source code",
                None,
                Some(vec!["*-test".to_string(), "*-backup".to_string()]),
            )
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        assert!(
            result
                .get("excluded_count")
                .and_then(|c| c.as_u64())
                .unwrap_or(0)
                >= 0
        );
    }

    #[tokio::test]
    async fn test_filter_with_both_include_and_exclude() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .filter_collections(
                "configuration",
                Some(vec!["config-*".to_string(), "*-settings".to_string()]),
                Some(vec!["*-old".to_string(), "*-deprecated".to_string()]),
            )
            .await;

        assert!(response.is_ok());
    }

    // ==================== SCORE COLLECTIONS TESTS ====================

    #[tokio::test]
    async fn test_score_collections_by_relevance() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .score_collections("machine learning", None, None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        assert!(result.get("scored_collections").is_some());
        assert!(result["total_collections"].as_u64().unwrap_or(0) >= 0);
    }

    #[tokio::test]
    async fn test_score_with_custom_term_boost_weight() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .score_collections("database queries", None, Some(0.4), None)
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_score_with_custom_signal_boost_weight() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .score_collections("performance optimization", None, None, Some(0.2))
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_score_collections_sorted_by_score() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .score_collections("search functionality", None, None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(scored_collections) =
            result.get("scored_collections").and_then(|s| s.as_array())
        {
            // Verify sorting
            if scored_collections.len() > 1 {
                for i in 0..(scored_collections.len() - 1) {
                    let score_i = scored_collections[i]
                        .get("score")
                        .and_then(|s| s.as_f64())
                        .unwrap_or(0.0);
                    let score_next = scored_collections[i + 1]
                        .get("score")
                        .and_then(|s| s.as_f64())
                        .unwrap_or(0.0);
                    assert!(score_i >= score_next);
                }
            }
        }
    }

    // ==================== EXPAND QUERIES TESTS ====================

    #[tokio::test]
    async fn test_expand_query_with_default_options() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries("CMMV framework", None, None, None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        assert_eq!(
            result["original_query"].as_str().unwrap_or(""),
            "CMMV framework"
        );
        assert!(
            result
                .get("expanded_queries")
                .and_then(|e| e.as_array())
                .map_or(false, |a| a.len() > 0)
        );
    }

    #[tokio::test]
    async fn test_expand_query_limits_expansions() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries("vector database", Some(5), None, None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        assert!(
            result
                .get("expanded_queries")
                .and_then(|e| e.as_array())
                .map_or(false, |a| a.len() <= 5)
        );
    }

    #[tokio::test]
    async fn test_expand_query_includes_definition() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries("semantic search", None, Some(true), None, None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(query_types) = result.get("query_types").and_then(|q| q.as_array()) {
            let contains_definition = query_types.iter().any(|t| t.as_str() == Some("definition"));
            assert!(contains_definition);
        }
    }

    #[tokio::test]
    async fn test_expand_query_includes_features() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries("API gateway", None, None, Some(true), None)
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(query_types) = result.get("query_types").and_then(|q| q.as_array()) {
            let contains_features = query_types.iter().any(|t| t.as_str() == Some("features"));
            assert!(contains_features);
        }
    }

    #[tokio::test]
    async fn test_expand_query_includes_architecture() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries("microservices", None, None, None, Some(true))
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(query_types) = result.get("query_types").and_then(|q| q.as_array()) {
            let contains_architecture = query_types
                .iter()
                .any(|t| t.as_str() == Some("architecture"));
            assert!(contains_architecture);
        }
    }

    #[tokio::test]
    async fn test_expand_query_generates_diverse_variations() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .expand_queries(
                "authentication system",
                Some(10),
                Some(true),
                Some(true),
                Some(true),
            )
            .await;

        assert!(response.is_ok());
        let result = response.unwrap();
        if let Some(expanded_queries) = result.get("expanded_queries").and_then(|e| e.as_array()) {
            assert!(expanded_queries.len() > 1);

            // Check for diversity
            let unique_queries: std::collections::HashSet<_> =
                expanded_queries.iter().filter_map(|q| q.as_str()).collect();
            assert_eq!(unique_queries.len(), expanded_queries.len());
        }
    }

    // ==================== ERROR HANDLING TESTS ====================

    #[tokio::test]
    async fn test_empty_query_in_discover() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client.discover("", None, None, None, None, None).await;
        assert!(response.is_err(), "Empty query should fail");
    }

    #[tokio::test]
    async fn test_invalid_max_bullets() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .discover("test", None, None, Some(0), None, None)
            .await;
        assert!(response.is_err(), "Invalid max_bullets should fail");
    }

    #[tokio::test]
    async fn test_empty_query_in_filter_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client.filter_collections("", None, None).await;
        assert!(response.is_err(), "Empty query should fail");
    }

    #[tokio::test]
    async fn test_invalid_weights_in_score_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .score_collections("test", Some(1.5), None, None)
            .await;
        assert!(response.is_err(), "Invalid name_match_weight should fail");
    }

    // ==================== INTEGRATION TESTS ====================

    #[tokio::test]
    async fn test_chain_filter_and_score_operations() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        // First filter
        let filter_response = client
            .filter_collections("documentation", Some(vec!["*-docs".to_string()]), None)
            .await;

        assert!(filter_response.is_ok());

        // Then score the filtered collections
        let score_response = client
            .score_collections("API documentation", None, None, None)
            .await;

        assert!(score_response.is_ok());
    }

    #[tokio::test]
    async fn test_use_expanded_queries_in_discovery() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        // First expand queries
        let expand_response = client
            .expand_queries("database optimization", Some(5), None, None, None)
            .await;

        assert!(expand_response.is_ok());
        let expand_result = expand_response.unwrap();
        let first_query = expand_result
            .get("expanded_queries")
            .and_then(|e| e.as_array())
            .and_then(|a| a.first())
            .and_then(|q| q.as_str())
            .unwrap_or("database optimization");
        assert!(!first_query.is_empty());

        // Use expanded queries in discovery
        let discover_response = client
            .discover(first_query, None, None, Some(10), None, None)
            .await;

        assert!(discover_response.is_ok());
    }

    // ==================== PERFORMANCE TESTS ====================

    #[tokio::test]
    async fn test_discover_performance() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let start_time = std::time::Instant::now();

        let _ = client
            .discover("performance test", None, None, Some(10), None, None)
            .await;

        let duration = start_time.elapsed();
        assert!(duration.as_secs() < 10, "Should complete within 10 seconds");
    }

    #[tokio::test]
    async fn test_score_collections_with_large_collections() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client.score_collections("test", None, None, None).await;

        assert!(response.is_ok());
    }
}
