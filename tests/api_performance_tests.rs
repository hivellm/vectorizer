//! API Performance Tests
//!
//! These tests measure and validate the performance characteristics
//! of the Vectorizer REST API under various load conditions.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use vectorizer::{api::server::VectorizerServer, auth::AuthManager, db::VectorStore, embedding::EmbeddingManager};
use std::sync::Arc;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    let vector_store = Arc::new(VectorStore::new());
    let _auth_manager = AuthManager::new_default().unwrap();
    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 8080, vector_store, embedding_manager);
    server.create_app()
}

#[tokio::test]
async fn test_health_check_performance() {
    let app = create_test_app().await;

    let start = Instant::now();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        duration < Duration::from_millis(100),
        "Health check took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    let app = create_test_app().await;

    let mut handles = vec![];

    // Spawn 100 concurrent health check requests
    for _ in 0..100 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = Request::builder()
                .method(Method::GET)
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        handles.push(handle);
    }

    let start = Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All requests should succeed
    for result in results {
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    assert!(
        duration < Duration::from_millis(1000),
        "100 concurrent health checks took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_collection_creation_performance() {
    let app = create_test_app().await;

    let collection_data = json!({
        "name": "perf_test_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let start = Instant::now();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(
        duration < Duration::from_millis(500),
        "Collection creation took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_batch_vector_insertion_performance() {
    let app = create_test_app().await;

    // Create collection first
    let collection_data = json!({
        "name": "batch_perf_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Prepare batch of vectors
    let mut vectors = vec![];
    for i in 0..100 {
        vectors.push(json!({
            "id": format!("batch_vec_{}", i),
            "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
            "payload": {"index": i, "title": format!("Vector {}", i)}
        }));
    }

    let vectors_data = json!({
        "vectors": vectors
    });

    let start = Instant::now();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/batch_perf_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(
        duration < Duration::from_secs(5),
        "Batch insertion of 100 vectors took too long: {:?}",
        duration
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["inserted_count"], 100);
}

#[tokio::test]
async fn test_search_performance() {
    let app = create_test_app().await;

    // Create collection and insert test data
    let collection_data = json!({
        "name": "search_perf_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Insert test vectors
    let mut vectors = vec![];
    for i in 0..50 {
        vectors.push(json!({
            "id": format!("search_vec_{}", i),
            "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
            "payload": {"index": i, "content": format!("Document content {}", i)}
        }));
    }

    let vectors_data = json!({
        "vectors": vectors
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/search_perf_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Test search performance
    let search_data = json!({
        "query": "document content",
        "limit": 10
    });

    let start = Instant::now();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/search_perf_collection/search")
        .header("content-type", "application/json")
        .body(Body::from(search_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        duration < Duration::from_millis(1000),
        "Search took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_concurrent_searches() {
    let app = create_test_app().await;

    // Setup test data
    let collection_data = json!({
        "name": "concurrent_search_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Insert test vectors
    let mut vectors = vec![];
    for i in 0..100 {
        vectors.push(json!({
            "id": format!("concurrent_vec_{}", i),
            "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
            "payload": {"index": i, "content": format!("Content {}", i)}
        }));
    }

    let vectors_data = json!({
        "vectors": vectors
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/concurrent_search_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Run concurrent searches
    let mut handles = vec![];

    for i in 0..20 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let search_data = json!({
                "query": format!("content {}", i),
                "limit": 5
            });

            let request = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/collections/concurrent_search_collection/search")
                .header("content-type", "application/json")
                .body(Body::from(search_data.to_string()))
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        handles.push(handle);
    }

    let start = Instant::now();
    let results = futures_util::future::join_all(handles).await;
    let duration = start.elapsed();

    // All searches should succeed
    for result in results {
        assert_eq!(result.unwrap(), StatusCode::OK);
    }

    assert!(
        duration < Duration::from_secs(3),
        "20 concurrent searches took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "memory_test_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Insert vectors in batches to test memory usage
    for batch in 0..10 {
        let mut vectors = vec![];
        for i in 0..50 {
            vectors.push(json!({
                "id": format!("memory_vec_{}_{}", batch, i),
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"batch": batch, "index": i}
            }));
        }

        let vectors_data = json!({
            "vectors": vectors
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/collections/memory_test_collection/vectors")
            .header("content-type", "application/json")
            .body(Body::from(vectors_data.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Verify we can still perform operations
    let search_data = json!({
        "query": "test query",
        "limit": 10
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/memory_test_collection/search")
        .header("content-type", "application/json")
        .body(Body::from(search_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_request_timeout_handling() {
    let app = create_test_app().await;

    // Test that requests don't hang indefinitely
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let result = timeout(Duration::from_secs(5), app.oneshot(request)).await;
    assert!(result.is_ok(), "Request timed out");

    let response = result.unwrap().unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_large_payload_handling() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "large_payload_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let create_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let create_response = app.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Create vector with large payload
    let large_payload = json!({
        "title": "Large Document",
        "content": "x".repeat(10000), // 10KB of content
        "metadata": {
            "tags": (0..100).map(|i| format!("tag_{}", i)).collect::<Vec<String>>(),
            "data": (0..1000).map(|i| i).collect::<Vec<i32>>()
        }
    });

    let vector_data = json!({
        "vectors": [
            {
                "id": "large_payload_vector",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": large_payload
            }
        ]
    });

    let start = Instant::now();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/large_payload_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vector_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(
        duration < Duration::from_secs(2),
        "Large payload handling took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_error_response_performance() {
    let app = create_test_app().await;

    // Test that error responses are fast
    let start = Instant::now();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/nonexistent_collection")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(
        duration < Duration::from_millis(100),
        "Error response took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_api_throughput() {
    let app = create_test_app().await;

    let start = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    // Run 1000 requests as fast as possible
    for _i in 0..1000 {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();

        match app.clone().oneshot(request).await {
            Ok(response) => {
                if response.status() == StatusCode::OK {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
            }
            Err(_) => error_count += 1,
        }
    }

    let duration = start.elapsed();
    let throughput = 1000.0 / duration.as_secs_f64();

    assert!(
        success_count > 950,
        "Too many failed requests: {} errors",
        error_count
    );
    assert!(
        throughput > 100.0,
        "Throughput too low: {:.2} req/sec",
        throughput
    );

    println!("API Throughput: {:.2} req/sec", throughput);
    println!(
        "Success rate: {:.2}%",
        (success_count as f64 / 1000.0) * 100.0
    );
}
