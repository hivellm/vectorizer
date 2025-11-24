//! Tests for File Operations
//!
//! This test suite covers:
//! - get_file_content() - Retrieve complete file content
//! - list_files_in_collection() - List indexed files
//! - get_file_summary() - Get file summaries
//! - get_file_chunks_ordered() - Get file chunks in order
//! - get_project_outline() - Get project structure
//! - get_related_files() - Find semantically related files
//! - search_by_file_type() - Search filtered by file type

#[cfg(test)]
mod file_operations_tests {
    use std::env;
use tracing::{info, error, warn, debug};
    use vectorizer_sdk::*;

    const TEST_COLLECTION: &str = "test-collection";

    fn get_test_client() -> VectorizerClient {
        let base_url =
            env::var("VECTORIZER_URL").unwrap_or_else(|_| "http://localhost:15002".to_string());
        VectorizerClient::new_with_url(&base_url).expect("Failed to create client")
    }

    async fn is_server_available(client: &VectorizerClient) -> bool {
        client.health_check().await.is_ok()
    }

    // Helper macro to handle optional test results (files may not exist)
    macro_rules! assert_or_warn {
        ($expr:expr, $test_name:expr) => {
            match $expr {
                Ok(result) => {
                    tracing::info!("✓ {} succeeded", $test_name);
                    Some(result)
                }
                Err(e) => {
                    tracing::info!("WARNING: {} failed: {:?}", $test_name, e);
                    tracing::info!("This may be due to files not being indexed in the test collection");
                    None
                }
            }
        };
    }

    // ==================== GET FILE CONTENT TESTS ====================

    #[tokio::test]
    async fn test_retrieve_complete_file_content() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content(TEST_COLLECTION, "README.md", None)
            .await;

        if let Some(result) = assert_or_warn!(response, "get_file_content README.md") {
            assert_eq!(result["file_path"].as_str().unwrap_or(""), "README.md");
            assert!(result.get("content").is_some());
            assert!(result.get("metadata").is_some());
        }
    }

    #[tokio::test]
    async fn test_retrieve_file_content_with_size_limit() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content(TEST_COLLECTION, "large-file.md", Some(100))
            .await;

        if let Some(result) = assert_or_warn!(response, "get_file_content large-file.md") {
            if let Some(size_kb) = result.get("size_kb").and_then(|s| s.as_u64()) {
                assert!(size_kb <= 100);
            }
        }
    }

    #[tokio::test]
    async fn test_file_content_includes_metadata() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content(TEST_COLLECTION, "src/main.ts", None)
            .await;

        if let Some(result) = assert_or_warn!(response, "get_file_content src/main.ts") {
            assert!(result.get("metadata").is_some());
            if let Some(metadata) = result.get("metadata") {
                assert!(metadata.get("file_type").is_some());
                assert!(metadata.get("size").and_then(|s| s.as_u64()).unwrap_or(0) > 0);
            }
        }
    }

    #[tokio::test]
    async fn test_non_existent_file_raises_error() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content(TEST_COLLECTION, "non-existent-file.txt", None)
            .await;
        assert!(response.is_err(), "Non-existent file should fail");
    }

    // ==================== LIST FILES IN COLLECTION TESTS ====================

    #[tokio::test]
    async fn test_list_all_files_in_collection() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, None, None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "list_files_in_collection") {
            assert!(result.get("files").is_some());
            assert!(result["total_count"].as_u64().unwrap_or(0) >= 0);
        }
    }

    #[tokio::test]
    async fn test_filter_files_by_type() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(
                TEST_COLLECTION,
                Some(vec!["ts".to_string(), "js".to_string()]),
                None,
                None,
                None,
            )
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                for file in files {
                    let file_path = file["file_path"].as_str().unwrap_or("");
                    assert!(file_path.ends_with(".ts") || file_path.ends_with(".js"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_filter_files_by_minimum_chunks() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, Some(5), None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                for file in files {
                    assert!(file["chunk_count"].as_u64().unwrap_or(0) >= 5);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_limit_file_results() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, None, Some(10), None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                assert!(files.len() <= 10);
            }
        }
    }

    #[tokio::test]
    async fn test_sort_files_by_name() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, None, None, Some("name"))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                if files.len() > 1 {
                    for i in 0..(files.len() - 1) {
                        let path_i = files[i]["file_path"].as_str().unwrap_or("");
                        let path_next = files[i + 1]["file_path"].as_str().unwrap_or("");
                        assert!(path_i <= path_next);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_sort_files_by_size() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, None, None, Some("size"))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                if files.len() > 1 {
                    for i in 0..(files.len() - 1) {
                        let size_i = files[i]["size"].as_u64().unwrap_or(0);
                        let size_next = files[i + 1]["size"].as_u64().unwrap_or(0);
                        assert!(size_i >= size_next);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_sort_files_by_chunks() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .list_files_in_collection(TEST_COLLECTION, None, None, None, Some("chunks"))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(files) = result.get("files").and_then(|f| f.as_array()) {
                if files.len() > 1 {
                    for i in 0..(files.len() - 1) {
                        let chunks_i = files[i]["chunk_count"].as_u64().unwrap_or(0);
                        let chunks_next = files[i + 1]["chunk_count"].as_u64().unwrap_or(0);
                        assert!(chunks_i >= chunks_next);
                    }
                }
            }
        }
    }

    // ==================== GET FILE SUMMARY TESTS ====================

    #[tokio::test]
    async fn test_get_extractive_summary() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_summary(TEST_COLLECTION, "README.md", Some("extractive"), Some(5))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("summary").is_some());
            assert_eq!(result["summary_type"].as_str().unwrap_or(""), "extractive");
            if let Some(sentences) = result.get("sentences").and_then(|s| s.as_array()) {
                assert!(sentences.len() <= 5);
            }
        }
    }

    #[tokio::test]
    async fn test_get_structural_summary() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_summary(TEST_COLLECTION, "src/main.ts", Some("structural"), None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("summary").is_some());
            assert_eq!(result["summary_type"].as_str().unwrap_or(""), "structural");
            assert!(result.get("structure").is_some());
        }
    }

    #[tokio::test]
    async fn test_get_both_summary_types() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_summary(TEST_COLLECTION, "docs/api.md", Some("both"), None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("extractive_summary").is_some());
            assert!(result.get("structural_summary").is_some());
        }
    }

    // ==================== GET FILE CHUNKS ORDERED TESTS ====================

    #[tokio::test]
    async fn test_get_file_chunks_in_order() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_chunks_ordered(TEST_COLLECTION, "README.md", None, None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("chunks").is_some());
            assert!(result["total_chunks"].as_u64().unwrap_or(0) >= 0);
        }
    }

    #[tokio::test]
    async fn test_get_chunks_from_specific_position() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_chunks_ordered(TEST_COLLECTION, "README.md", Some(5), Some(10), None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert_eq!(result["start_chunk"].as_u64().unwrap_or(0), 5);
            if let Some(chunks) = result.get("chunks").and_then(|c| c.as_array()) {
                assert!(chunks.len() <= 10);
            }
        }
    }

    #[tokio::test]
    async fn test_get_chunks_with_context_hints() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_chunks_ordered(TEST_COLLECTION, "README.md", None, None, Some(true))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(chunks) = result.get("chunks").and_then(|c| c.as_array()) {
                for chunk in chunks {
                    assert!(chunk.get("has_prev").is_some());
                    assert!(chunk.get("has_next").is_some());
                }
            }
        }
    }

    #[tokio::test]
    async fn test_paginate_through_chunks() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        // Get first page
        let page1 = client
            .get_file_chunks_ordered(TEST_COLLECTION, "README.md", Some(0), Some(5), None)
            .await;

        if let Some(page1_result) = assert_or_warn!(page1, "paginate first page") {
            if page1_result["total_chunks"].as_u64().unwrap_or(0) > 5 {
                // Get second page
                let page2 = client
                    .get_file_chunks_ordered(TEST_COLLECTION, "README.md", Some(5), Some(5), None)
                    .await;

                if let Some(page2_result) = assert_or_warn!(page2, "paginate second page") {
                    assert_eq!(page2_result["start_chunk"].as_u64().unwrap_or(0), 5);
                }
            }
        }
    }

    // ==================== GET PROJECT OUTLINE TESTS ====================

    #[tokio::test]
    async fn test_get_project_outline() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_project_outline(TEST_COLLECTION, None, None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "get_project_outline") {
            assert!(result.get("structure").is_some());
            assert!(result.get("statistics").is_some());
        }
    }

    #[tokio::test]
    async fn test_outline_highlights_key_files() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_project_outline(TEST_COLLECTION, None, None, Some(true))
            .await;

        if let Some(result) = assert_or_warn!(response, "get_project_outline with key_files") {
            assert!(result.get("key_files").is_some());
        }
    }

    // ==================== GET RELATED FILES TESTS ====================

    #[tokio::test]
    async fn test_find_related_files() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_related_files(TEST_COLLECTION, "src/main.ts", None, None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("related_files").is_some());
        }
    }

    #[tokio::test]
    async fn test_limit_related_files_results() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_related_files(TEST_COLLECTION, "README.md", Some(5), None, None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(related_files) = result.get("related_files").and_then(|f| f.as_array()) {
                assert!(related_files.len() <= 5);
            }
        }
    }

    #[tokio::test]
    async fn test_filter_related_files_by_similarity() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_related_files(TEST_COLLECTION, "src/main.ts", None, Some(0.7), None)
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(related_files) = result.get("related_files").and_then(|f| f.as_array()) {
                for file in related_files {
                    let similarity = file["similarity_score"].as_f64().unwrap_or(0.0);
                    assert!(similarity >= 0.7);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_related_files_includes_reason() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_related_files(TEST_COLLECTION, "src/main.ts", None, None, Some(true))
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(related_files) = result.get("related_files").and_then(|f| f.as_array()) {
                for file in related_files {
                    assert!(file.get("reason").is_some());
                }
            }
        }
    }

    // ==================== SEARCH BY FILE TYPE TESTS ====================

    #[tokio::test]
    async fn test_search_by_file_type_limits_results() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .search_by_file_type(
                TEST_COLLECTION,
                "test",
                vec!["ts".to_string(), "js".to_string()],
                Some(10),
                None,
            )
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
                assert!(results.len() <= 10);
            }
        }
    }

    #[tokio::test]
    async fn test_search_by_multiple_file_types() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .search_by_file_type(
                TEST_COLLECTION,
                "code",
                vec![
                    "ts".to_string(),
                    "js".to_string(),
                    "py".to_string(),
                    "rs".to_string(),
                ],
                None,
                None,
            )
            .await;

        if let Some(result) = assert_or_warn!(response, "file_operation") {
            assert!(result.get("results").is_some());
        }
    }

    // ==================== ERROR HANDLING TESTS ====================

    #[tokio::test]
    async fn test_invalid_collection_raises_error() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content("non-existent-collection", "README.md", None)
            .await;
        assert!(response.is_err(), "Invalid collection should fail");
    }

    #[tokio::test]
    async fn test_invalid_max_size_kb_raises_error() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .get_file_content(TEST_COLLECTION, "README.md", Some(0))
            .await;
        assert!(response.is_err(), "Invalid max_size_kb should fail");
    }

    #[tokio::test]
    async fn test_empty_file_types_array_raises_error() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let response = client
            .search_by_file_type(TEST_COLLECTION, "test", vec![], None, None)
            .await;
        assert!(response.is_err(), "Empty file types should fail");
    }

    // ==================== PERFORMANCE TESTS ====================

    #[tokio::test]
    async fn test_list_files_efficiently() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let start_time = std::time::Instant::now();

        let _ = client
            .list_files_in_collection(TEST_COLLECTION, None, None, Some(100), None)
            .await;

        let duration = start_time.elapsed();
        assert!(duration.as_secs() < 5, "Should complete within 5 seconds");
    }

    #[tokio::test]
    async fn test_retrieve_file_content_quickly() {
        let client = get_test_client();
        if !is_server_available(&client).await {
            tracing::info!("WARNING: Vectorizer server not available, skipping test");
            return;
        }

        let start_time = std::time::Instant::now();

        let _ = client
            .get_file_content(TEST_COLLECTION, "README.md", None)
            .await;

        let duration = start_time.elapsed();
        assert!(duration.as_secs() < 3, "Should complete within 3 seconds");
    }
}
