//! End-to-End Integration Tests
//!
//! These tests verify the complete system functionality from API to database,
//! including MCP integration and real-world usage scenarios.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use vectorizer::{api::server::VectorizerServer, auth::AuthManager, db::VectorStore};
// use std::time::Duration;
use serde_json::json;
use tower::ServiceExt;
// use tokio::time::timeout;

async fn create_test_app() -> Router {
    let vector_store = VectorStore::new();
    let _auth_manager = AuthManager::new_default().unwrap();
    let server = VectorizerServer::new("127.0.0.1", 8080, vector_store);
    server.create_app()
}

#[tokio::test]
async fn test_complete_workflow() {
    let app = create_test_app().await;

    // 1. Create collection
    let collection_data = json!({
        "name": "workflow_test_collection",
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

    // 2. Insert vectors
    let vectors_data = json!({
        "vectors": [
            {
                "id": "workflow_vec1",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Document 1", "content": "Machine learning algorithms"}
            },
            {
                "id": "workflow_vec2",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Document 2", "content": "Deep learning neural networks"}
            },
            {
                "id": "workflow_vec3",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Document 3", "content": "Natural language processing"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/workflow_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // 3. Search vectors
    let search_data = json!({
        "query": "machine learning",
        "limit": 5
    });

    let search_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/workflow_test_collection/search")
        .header("content-type", "application/json")
        .body(Body::from(search_data.to_string()))
        .unwrap();

    let search_response = app.clone().oneshot(search_request).await.unwrap();
    assert_eq!(search_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(search_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(results["results"].is_array());

    // 4. Get specific vector
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/workflow_test_collection/vectors/workflow_vec1")
        .body(Body::empty())
        .unwrap();

    let get_response = app.clone().oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    // 5. Update vector (delete and re-insert)
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/collections/workflow_test_collection/vectors/workflow_vec1")
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Re-insert with updated data
    let updated_vector_data = json!({
        "vectors": [
            {
                "id": "workflow_vec1",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Updated Document 1", "content": "Updated machine learning content"}
            }
        ]
    });

    let reinsert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/workflow_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(updated_vector_data.to_string()))
        .unwrap();

    let reinsert_response = app.clone().oneshot(reinsert_request).await.unwrap();
    assert_eq!(reinsert_response.status(), StatusCode::CREATED);

    // 6. Verify updated vector
    let verify_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/workflow_test_collection/vectors/workflow_vec1")
        .body(Body::empty())
        .unwrap();

    let verify_response = app.oneshot(verify_request).await.unwrap();
    assert_eq!(verify_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(verify_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let vector: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(vector["payload"]["title"], "Updated Document 1");
}

#[tokio::test]
async fn test_multiple_collections_workflow() {
    let app = create_test_app().await;

    // Create multiple collections
    let collections = [
        ("documents", 384, "cosine"),
        ("images", 512, "euclidean"),
        ("code", 512, "dotproduct"),
    ];

    for (name, dimension, metric) in &collections {
        let collection_data = json!({
            "name": name,
            "dimension": dimension,
            "metric": metric
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/collections")
            .header("content-type", "application/json")
            .body(Body::from(collection_data.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Insert data into each collection
    for (name, dimension, _) in &collections {
        let vectors_data = json!({
            "vectors": [
                {
                    "id": format!("{}_vec1", name),
                    "data": (0..*dimension).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                    "payload": {"collection": name, "type": "test"}
                }
            ]
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri(&format!("/api/v1/collections/{}/vectors", name))
            .header("content-type", "application/json")
            .body(Body::from(vectors_data.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // List all collections
    let list_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections")
        .body(Body::empty())
        .unwrap();

    let list_response = app.clone().oneshot(list_request).await.unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let collections_list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(collections_list["collections"].as_array().unwrap().len(), 3);

    // Search in each collection
    for (name, _, _) in &collections {
        let search_data = json!({
            "query": "test query",
            "limit": 5
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri(&format!("/api/v1/collections/{}/search", name))
            .header("content-type", "application/json")
            .body(Body::from(search_data.to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    let app = create_test_app().await;

    // 1. Try to access non-existent collection
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 2. Create the collection
    let collection_data = json!({
        "name": "recovery_test_collection",
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

    // 3. Try to access non-existent vector
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/recovery_test_collection/vectors/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 4. Insert vector
    let vectors_data = json!({
        "vectors": [
            {
                "id": "recovery_vec1",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Recovery Test"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/recovery_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // 5. Now access should work
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/recovery_test_collection/vectors/recovery_vec1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "concurrent_test_collection",
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

    // Run concurrent insertions
    let mut handles = vec![];

    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let vectors_data = json!({
                "vectors": [
                    {
                        "id": format!("concurrent_vec_{}", i),
                        "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                        "payload": {"index": i, "title": format!("Concurrent Document {}", i)}
                    }
                ]
            });

            let request = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/collections/concurrent_test_collection/vectors")
                .header("content-type", "application/json")
                .body(Body::from(vectors_data.to_string()))
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        handles.push(handle);
    }

    let results = futures_util::future::join_all(handles).await;

    // All insertions should succeed
    for result in results {
        assert_eq!(result.unwrap(), StatusCode::CREATED);
    }

    // Run concurrent searches
    let mut search_handles = vec![];

    for i in 0..5 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let search_data = json!({
                "query": format!("document {}", i),
                "limit": 5
            });

            let request = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/collections/concurrent_test_collection/search")
                .header("content-type", "application/json")
                .body(Body::from(search_data.to_string()))
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        search_handles.push(handle);
    }

    let search_results = futures_util::future::join_all(search_handles).await;

    // All searches should succeed
    for result in search_results {
        assert_eq!(result.unwrap(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_data_persistence_simulation() {
    let app = create_test_app().await;

    // Simulate data persistence by creating, using, and verifying data
    let collection_data = json!({
        "name": "persistence_test_collection",
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

    // Insert multiple vectors
    let mut vectors = vec![];
    for i in 0..20 {
        vectors.push(json!({
            "id": format!("persistence_vec_{}", i),
            "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
            "payload": {
                "index": i,
                "title": format!("Persistent Document {}", i),
                "content": format!("This is the content of document {}", i),
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "tags": ["test", "persistence", format!("doc_{}", i)]
                }
            }
        }));
    }

    let vectors_data = json!({
        "vectors": vectors
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/persistence_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Verify all vectors are accessible
    for i in 0..20 {
        let request = Request::builder()
            .method(Method::GET)
            .uri(&format!(
                "/api/v1/collections/persistence_test_collection/vectors/persistence_vec_{}",
                i
            ))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let vector: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(vector["id"], format!("persistence_vec_{}", i));
        assert_eq!(vector["payload"]["index"], i);
    }

    // Test search across all vectors
    let search_data = json!({
        "query": "persistent document",
        "limit": 20
    });

    let search_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/persistence_test_collection/search")
        .header("content-type", "application/json")
        .body(Body::from(search_data.to_string()))
        .unwrap();

    let search_response = app.oneshot(search_request).await.unwrap();
    assert_eq!(search_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(search_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(results["results"].is_array());
}

#[tokio::test]
async fn test_api_consistency() {
    let app = create_test_app().await;

    // Test that API responses are consistent across multiple calls
    let collection_data = json!({
        "name": "consistency_test_collection",
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

    // Insert test vector
    let vectors_data = json!({
        "vectors": [
            {
                "id": "consistency_vec1",
                "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
                "payload": {"title": "Consistency Test", "value": 42}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/consistency_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Make multiple identical requests and verify consistency
    for _ in 0..5 {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/collections/consistency_test_collection/vectors/consistency_vec1")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let vector: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify consistent response structure
        assert_eq!(vector["id"], "consistency_vec1");
        assert!(vector["vector"].is_array());
        assert_eq!(vector["vector"].as_array().unwrap().len(), 384);
        assert_eq!(vector["payload"]["title"], "Consistency Test");
        assert_eq!(vector["payload"]["value"], 42);
    }
}

#[tokio::test]
async fn test_system_health_under_load() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "load_test_collection",
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

    // Insert data
    let mut vectors = vec![];
    for i in 0..100 {
        vectors.push(json!({
            "id": format!("load_vec_{}", i),
            "data": (0..384).map(|_| rand::random::<f32>()).collect::<Vec<f32>>(),
            "payload": {"index": i, "content": format!("Load test content {}", i)}
        }));
    }

    let vectors_data = json!({
        "vectors": vectors
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/load_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Run mixed operations under load
    let mut handles = vec![];

    // Health checks
    for _ in 0..50 {
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

    // Searches
    for i in 0..25 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let search_data = json!({
                "query": format!("content {}", i),
                "limit": 10
            });

            let request = Request::builder()
                .method(Method::POST)
                .uri("/api/v1/collections/load_test_collection/search")
                .header("content-type", "application/json")
                .body(Body::from(search_data.to_string()))
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        handles.push(handle);
    }

    // Vector retrievals
    for i in 0..25 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let request = Request::builder()
                .method(Method::GET)
                .uri(&format!(
                    "/api/v1/collections/load_test_collection/vectors/load_vec_{}",
                    i
                ))
                .body(Body::empty())
                .unwrap();

            let response = app_clone.oneshot(request).await.unwrap();
            response.status()
        });
        handles.push(handle);
    }

    let results = futures_util::future::join_all(handles).await;

    // Verify system remained healthy
    let mut success_count = 0;
    let mut error_count = 0;

    for result in results {
        match result.unwrap() {
            StatusCode::OK => success_count += 1,
            _ => error_count += 1,
        }
    }

    assert!(
        success_count > 90,
        "System health degraded: {} errors out of 100",
        error_count
    );
}
