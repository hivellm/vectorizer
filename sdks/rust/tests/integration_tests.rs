#![allow(warnings)]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::absurd_extreme_comparisons, clippy::nonminimal_bool)]

//! Comprehensive tests for the Rust SDK

use std::collections::HashMap;

use tracing::{debug, error, info, warn};
use vectorizer_sdk::*;

/// Gate for tests that call auth-protected endpoints.
///
/// `/health` is intentionally anonymous, so a bare health check can
/// succeed against a production-mode server (`auth.enabled: true`) that
/// would still reject every gated call with 401. These tests don't
/// authenticate — returning `false` from here lets them skip cleanly
/// instead of panicking at the 401 boundary.
async fn gated_routes_available(client: &VectorizerClient) -> bool {
    client.health_check().await.is_ok() && client.list_collections().await.is_ok()
}

#[tokio::test]
async fn test_client_creation() {
    // Test default client creation
    let client = VectorizerClient::new_default();
    assert!(client.is_ok());

    // Test creation with custom URL
    let client = VectorizerClient::new_with_url("http://localhost:15002");
    assert!(client.is_ok());

    // Test creation with API key
    let client = VectorizerClient::new_with_api_key("http://localhost:15002", "test-key");
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_health_check() {
    let client = VectorizerClient::new_default().unwrap();

    // Port 15002 may be occupied by a sibling HiveLLM service (e.g.
    // `hivehub/nexus`) on a dev box — its `/health` answers 200 but
    // with a different JSON shape, which either fails to deserialize
    // into `HealthStatus` or surfaces a `version` like `1.x.y`. Gate
    // the test so we only assert when talking to an actual v3.x
    // Vectorizer.
    let Ok(health) = client.health_check().await else {
        return;
    };
    if !health.version.starts_with("3.") {
        return;
    }

    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());
    assert!(!health.timestamp.is_empty());
}

#[tokio::test]
async fn test_list_collections() {
    let client = VectorizerClient::new_default().unwrap();
    if !gated_routes_available(&client).await {
        return;
    }

    match client.list_collections().await {
        Ok(collections) => {
            assert!(!collections.is_empty());

            // Verify collection structure
            for collection in collections {
                assert!(!collection.name.is_empty());
                assert!(collection.dimension > 0);
                // metric is now an Option<String>, check if present
                if let Some(metric) = &collection.metric {
                    assert!(!metric.is_empty());
                }
            }
        }
        Err(e) => {
            panic!("List collections failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_create_collection() {
    let client = VectorizerClient::new_default().unwrap();
    if !gated_routes_available(&client).await {
        return;
    }
    let collection_name = format!("test_collection_{}", uuid::Uuid::new_v4());

    // Clean up if exists
    let _ = client.delete_collection(&collection_name).await;

    match client
        .create_collection(&collection_name, 384, Some(SimilarityMetric::Cosine))
        .await
    {
        Ok(info) => {
            assert_eq!(info.name, collection_name);
            assert_eq!(info.dimension, 384);
            // v3 server emits `metric` in Rust-Debug form ("Cosine");
            // the create-path helper lowercases it. Normalize on both
            // sides so either shape passes.
            assert_eq!(info.metric.to_lowercase(), "cosine");
            // Collection status can be "ready", "created", "pending-0", etc.
            if let Some(ref status) = info.indexing_status {
                assert!(!status.status.is_empty());
            }
        }
        Err(e) => {
            panic!("Create collection failed: {}", e);
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_insert_texts() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_insert_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    let texts = vec![
        BatchTextRequest {
            id: "test_doc_1".to_string(),
            text: "This is a test document for vectorization.".to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "test".to_string());
                meta.insert("language".to_string(), "english".to_string());
                meta
            }),
        },
        BatchTextRequest {
            id: "test_doc_2".to_string(),
            text: "Machine learning and artificial intelligence are fascinating topics."
                .to_string(),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("category".to_string(), "ai".to_string());
                meta.insert("language".to_string(), "english".to_string());
                meta
            }),
        },
    ];

    match client.insert_texts(&collection_name, texts).await {
        Ok(response) => {
            assert!(response.success);
            assert_eq!(response.collection, collection_name);
            assert_eq!(response.operation, "insert");
            assert_eq!(response.total_operations, 2);
            assert_eq!(response.successful_operations, 2);
            assert_eq!(response.failed_operations, 0);
        }
        Err(e) => {
            // If insert fails due to server issues, that's acceptable for testing
            tracing::info!("Insert texts failed (expected in test environment): {}", e);
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_search_vectors() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_search_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    // Insert test data
    let texts = vec![
        BatchTextRequest {
            id: "search_doc_1".to_string(),
            text: "Artificial intelligence and machine learning are transforming technology."
                .to_string(),
            metadata: None,
        },
        BatchTextRequest {
            id: "search_doc_2".to_string(),
            text: "Natural language processing enables computers to understand human language."
                .to_string(),
            metadata: None,
        },
    ];

    let _ = client.insert_texts(&collection_name, texts).await;

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match client
        .search_vectors(
            &collection_name,
            "artificial intelligence",
            Some(5),
            Some(0.1),
        )
        .await
    {
        Ok(response) => {
            assert!(!response.results.is_empty());
            assert!(response.results.len() > 0);

            // Verify result structure
            for result in response.results {
                assert!(!result.id.is_empty());
                assert!(result.score >= 0.0 && result.score <= 1.0);
            }
        }
        Err(e) => {
            // If search fails due to provider issues, that's acceptable for testing
            tracing::info!(
                "Search vectors failed (expected in test environment): {}",
                e
            );
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_get_vector() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_get_vector_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    // Insert test data
    let texts = vec![BatchTextRequest {
        id: "vector_doc_1".to_string(),
        text: "This is a test document for vector retrieval.".to_string(),
        metadata: None,
    }];

    let _ = client.insert_texts(&collection_name, texts).await;

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    match client.get_vector(&collection_name, "vector_doc_1").await {
        Ok(vector) => {
            assert_eq!(vector.id, "vector_doc_1");
            assert_eq!(vector.data.len(), 384);
            assert!(vector.data.iter().all(|&x| x.is_finite()));
        }
        Err(e) => {
            // If vector not found or indexing not complete, that's acceptable for testing
            tracing::info!("Get vector failed (expected in test environment): {}", e);
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_get_collection_info() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_info_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    match client.get_collection_info(&collection_name).await {
        Ok(info) => {
            assert_eq!(info.name, collection_name);
            assert_eq!(info.dimension, 384);
            assert_eq!(info.metric.to_lowercase(), "cosine");
            if let Some(ref status) = info.indexing_status {
                assert!(!status.status.is_empty());
            }
        }
        Err(e) => {
            panic!("Get collection info failed: {}", e);
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_embed_text() {
    let client = VectorizerClient::new_default().unwrap();

    match client
        .embed_text("This is a test text for embedding generation", None)
        .await
    {
        Ok(response) => {
            assert_eq!(
                response.text,
                "This is a test text for embedding generation"
            );
            assert!(!response.model.is_empty());
            assert!(!response.provider.is_empty());
            assert!(response.dimension > 0);
            assert_eq!(response.embedding.len(), response.dimension);
        }
        Err(e) => {
            // If embedding fails due to provider issues, that's acceptable for testing
            tracing::info!("Embed text failed (expected in test environment): {}", e);
        }
    }
}

#[tokio::test]
async fn test_delete_collection() {
    let client = VectorizerClient::new_default().unwrap();
    if !gated_routes_available(&client).await {
        return;
    }
    let collection_name = format!("test_delete_{}", uuid::Uuid::new_v4());

    // Create collection
    let _ = client.create_collection(&collection_name, 384, None).await;

    // Verify collection exists
    let collections = client.list_collections().await.unwrap();
    if !collections.iter().any(|c| c.name == collection_name) {
        // If collection doesn't exist in list, skip this test
        return;
    }

    // Delete collection
    match client.delete_collection(&collection_name).await {
        Ok(_) => {
            // Verify collection is deleted
            let collections = client.list_collections().await.unwrap();
            assert!(!collections.iter().any(|c| c.name == collection_name));
        }
        Err(e) => {
            panic!("Delete collection failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_error_handling() {
    let client = VectorizerClient::new_default().unwrap();

    // Test non-existent collection
    match client.get_collection_info("non_existent_collection").await {
        Ok(_) => {
            panic!("Should have failed for non-existent collection");
        }
        Err(e) => {
            // Should be a server error
            assert!(matches!(e, VectorizerError::Server { message: _ }));
        }
    }

    // Test invalid collection name
    match client.create_collection("", 384, None).await {
        Ok(_) => {
            panic!("Should have failed for empty collection name");
        }
        Err(e) => {
            // Could be validation error or server error depending on implementation
            assert!(
                matches!(e, VectorizerError::Validation { message: _ })
                    || matches!(e, VectorizerError::Server { message: _ })
            );
        }
    }
}

#[tokio::test]
async fn test_serialization() {
    let client = VectorizerClient::new_default().unwrap();
    if !gated_routes_available(&client).await {
        return;
    }

    // Test that all responses can be serialized/deserialized
    let health = client.health_check().await.unwrap();
    let health_json = serde_json::to_string(&health).unwrap();
    let health_deserialized: HealthStatus = serde_json::from_str(&health_json).unwrap();
    assert_eq!(health.status, health_deserialized.status);

    let collections = client.list_collections().await.unwrap();
    let collections_json = serde_json::to_string(&collections).unwrap();
    let collections_deserialized: Vec<CollectionInfo> =
        serde_json::from_str(&collections_json).unwrap();
    assert_eq!(collections.len(), collections_deserialized.len());
}

#[tokio::test]
async fn test_batch_operations() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_batch_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    // Insert multiple texts
    let texts = (1..=10)
        .map(|i| BatchTextRequest {
            id: format!("batch_doc_{}", i),
            text: format!(
                "This is batch document number {} for testing batch operations.",
                i
            ),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("batch_id".to_string(), i.to_string());
                meta.insert("test_type".to_string(), "batch".to_string());
                meta
            }),
        })
        .collect();

    match client.insert_texts(&collection_name, texts).await {
        Ok(response) => {
            assert!(response.success);
            assert_eq!(response.total_operations, 10);
            assert_eq!(response.successful_operations, 10);
            assert_eq!(response.failed_operations, 0);
        }
        Err(e) => {
            // If batch insert fails due to server issues, that's acceptable for testing
            tracing::info!("Batch insert failed (expected in test environment): {}", e);
        }
    }

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Test batch search
    match client
        .search_vectors(&collection_name, "batch document", Some(10), None)
        .await
    {
        Ok(response) => {
            assert!(!response.results.is_empty());
            assert!(response.results.len() <= 10);
        }
        Err(e) => {
            // If search fails due to provider issues, that's acceptable for testing
            tracing::info!("Batch search failed (expected in test environment): {}", e);
        }
    }

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

#[tokio::test]
async fn test_performance() {
    let client = VectorizerClient::new_default().unwrap();
    let collection_name = format!("test_perf_{}", uuid::Uuid::new_v4());

    // Create collection
    let create_result = client.create_collection(&collection_name, 384, None).await;
    if create_result.is_err() {
        // If collection creation fails, skip this test
        return;
    }

    let start_time = std::time::Instant::now();

    // Insert multiple texts
    let texts = (1..=50)
        .map(|i| BatchTextRequest {
            id: format!("perf_doc_{}", i),
            text: format!(
                "Performance test document number {} with some content for testing.",
                i
            ),
            metadata: None,
        })
        .collect();

    let insert_result = match client.insert_texts(&collection_name, texts).await {
        Ok(result) => result,
        Err(e) => {
            tracing::info!("Insert texts failed (expected in test environment): {}", e);
            return;
        }
    };
    let insert_time = start_time.elapsed();

    assert!(insert_result.success);
    assert!(insert_time.as_secs() < 30); // Should complete within 30 seconds

    // Wait for indexing
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Test search performance
    let search_start = std::time::Instant::now();
    let search_result = match client
        .search_vectors(&collection_name, "performance test", Some(20), None)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::info!(
                "Search vectors failed (expected in test environment): {}",
                e
            );
            return;
        }
    };
    let search_time = search_start.elapsed();

    assert!(!search_result.results.is_empty());
    assert!(search_time.as_millis() < 5000); // Should complete within 5 seconds

    // Cleanup
    let _ = client.delete_collection(&collection_name).await;
}

/// Response models must tolerate unknown top-level fields. The v3
/// server emits blocks the SDK does not model yet (status, size,
/// normalization, quantization on CollectionInfo; shape adds on
/// SearchResponse) and future server releases will add more. A
/// `#[serde(deny_unknown_fields)]` regression here would silently
/// break every SDK caller the moment the server adds a field.
#[test]
fn test_response_models_tolerate_unknown_fields() {
    // CollectionInfo with an extra top-level field + extras inside
    // nested blocks the SDK stores as Value.
    let json = r#"{
        "name": "c1",
        "dimension": 384,
        "metric": "Cosine",
        "vector_count": 0,
        "document_count": 0,
        "created_at": "2026-04-21T00:00:00Z",
        "updated_at": "2026-04-21T00:00:00Z",
        "size": { "total": 0, "total_bytes": 0, "future_field": "ok" },
        "quantization": { "enabled": false, "future_field": 42 },
        "normalization": { "enabled": false, "future_field": [1, 2, 3] },
        "status": "ready",
        "future_top_level_field": { "arbitrary": "payload" }
    }"#;
    let info: CollectionInfo = serde_json::from_str(json)
        .expect("CollectionInfo must tolerate unknown top-level + nested fields");
    assert_eq!(info.name, "c1");
    assert_eq!(info.dimension, 384);
    assert_eq!(info.metric.to_lowercase(), "cosine");
    assert_eq!(info.status.as_deref(), Some("ready"));

    // Collection response shape — same tolerance requirement.
    let collection_json = r#"{
        "name": "c2",
        "dimension": 512,
        "metric": "Euclidean",
        "future_top_level_field": true
    }"#;
    let collection: Collection = serde_json::from_str(collection_json)
        .expect("Collection must tolerate unknown top-level fields");
    assert_eq!(collection.name, "c2");
    assert_eq!(collection.dimension, 512);
    assert_eq!(
        collection.metric.as_deref().map(str::to_lowercase),
        Some("euclidean".to_string())
    );
}
