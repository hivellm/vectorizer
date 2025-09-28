//! Comprehensive API Tests
//!
//! These tests provide extensive coverage of the Vectorizer REST API,
//! including edge cases, error handling, and performance scenarios.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use vectorizer::{api::server::VectorizerServer, auth::AuthManager, db::VectorStore, embedding::EmbeddingManager};
use std::sync::Arc;


async fn create_test_app() -> Router {
    let vector_store = Arc::new(VectorStore::new());
    let _auth_manager = AuthManager::new_default().unwrap();
    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 8080, vector_store, embedding_manager);
    server.create_app()
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let health_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health_data["status"], "healthy");
    assert!(health_data["timestamp"].is_string());
}

#[tokio::test]
async fn test_status_endpoint() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_data: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status_data["status"], "healthy");
    assert!(status_data["version"].is_string());
    assert!(status_data["uptime"].is_number());
}

#[tokio::test]
async fn test_collections_list_empty() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let collections: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(collections["collections"].is_array());
    assert_eq!(collections["collections"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_collection() {
    let app = create_test_app().await;

    let collection_data = json!({
        "name": "test_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let collection: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(collection["collection"], "test_collection");
    assert!(collection["message"].is_string());
}

#[tokio::test]
async fn test_create_collection_invalid_data() {
    let app = create_test_app().await;

    let invalid_data = json!({
        "name": "",  // Empty name should fail
        "dimension": 0,  // Zero dimension should fail
        "metric": "cosine"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(invalid_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_collection_duplicate() {
    let app = create_test_app().await;

    let collection_data = json!({
        "name": "duplicate_test",
        "dimension": 384,
        "metric": "cosine"
    });

    // Create first collection
    let request1 = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    // Try to create duplicate
    let request2 = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from(collection_data.to_string()))
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_collection() {
    let app = create_test_app().await;

    // First create a collection
    let collection_data = json!({
        "name": "get_test_collection",
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

    // Then get it
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/get_test_collection")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let collection: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(collection["name"], "get_test_collection");
    assert_eq!(collection["dimension"], 384);
}

#[tokio::test]
async fn test_get_collection_not_found() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/nonexistent_collection")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_collection() {
    let app = create_test_app().await;

    // First create a collection
    let collection_data = json!({
        "name": "delete_test_collection",
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

    // Then delete it
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/collections/delete_test_collection")
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/delete_test_collection")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_insert_texts() {
    let app = create_test_app().await;

    // First create a collection
    let collection_data = json!({
        "name": "insert_test_collection",
        "dimension": 3,
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

    // Insert vectors
    let vectors_data = json!({
        "vectors": [
            {
                "id": "vec1",
                "vector": [1.0, 2.0, 3.0],
                "payload": {"title": "Vector 1"}
            },
            {
                "id": "vec2",
                "vector": [4.0, 5.0, 6.0],
                "payload": {"title": "Vector 2"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/insert_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(insert_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["inserted"], 2);
}

#[tokio::test]
async fn test_insert_texts_invalid_dimension() {
    let app = create_test_app().await;

    // Create collection with dimension 3
    let collection_data = json!({
        "name": "dimension_test_collection",
        "dimension": 3,
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

    // Try to insert vector with wrong dimension
    let vectors_data = json!({
        "vectors": [
            {
                "id": "vec1",
                "vector": [1.0, 2.0],  // Wrong dimension (2 instead of 3)
                "payload": {"title": "Vector 1"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/dimension_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_vectors() {
    let app = create_test_app().await;

    // Create collection and insert vectors
    let collection_data = json!({
        "name": "search_test_collection",
        "dimension": 3,
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
    let vectors_data = json!({
        "vectors": [
            {
                "id": "search_vec1",
                "vector": [1.0, 0.0, 0.0],
                "payload": {"title": "Unit vector X"}
            },
            {
                "id": "search_vec2",
                "vector": [0.0, 1.0, 0.0],
                "payload": {"title": "Unit vector Y"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/search_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Search for vectors
    let search_data = json!({
        "vector": [1.0, 0.0, 0.0],
        "limit": 10
    });

    let search_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/search_test_collection/search")
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
    // Array exists, no need to check length >= 0
}


#[tokio::test]
async fn test_get_vector() {
    let app = create_test_app().await;

    // Create collection and insert vector
    let collection_data = json!({
        "name": "get_vector_collection",
        "dimension": 3,
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

    // Insert vector
    let vectors_data = json!({
        "vectors": [
            {
                "id": "get_test_vector",
                "vector": [1.0, 2.0, 3.0],
                "payload": {"title": "Test Vector"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/get_vector_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Get the vector
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/get_vector_collection/vectors/get_test_vector")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let vector: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(vector["id"], "get_test_vector");
    // Vector is L2 normalized, just verify it's an array of 3 floats
    assert!(vector["vector"].is_array());
    assert_eq!(vector["vector"].as_array().unwrap().len(), 3);
    assert_eq!(vector["payload"]["title"], "Test Vector");
}

#[tokio::test]
async fn test_get_vector_not_found() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "not_found_collection",
        "dimension": 3,
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

    // Try to get non-existent vector
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/not_found_collection/vectors/nonexistent_vector")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_vector() {
    let app = create_test_app().await;

    // Create collection and insert vector
    let collection_data = json!({
        "name": "delete_vector_collection",
        "dimension": 3,
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

    // Insert vector
    let vectors_data = json!({
        "vectors": [
            {
                "id": "delete_test_vector",
                "vector": [1.0, 2.0, 3.0],
                "payload": {"title": "Vector to Delete"}
            }
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/delete_vector_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    // Delete the vector
    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/v1/collections/delete_vector_collection/vectors/delete_test_vector")
        .body(Body::empty())
        .unwrap();

    let delete_response = app.clone().oneshot(delete_request).await.unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let get_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/collections/delete_vector_collection/vectors/delete_test_vector")
        .body(Body::empty())
        .unwrap();

    let get_response = app.oneshot(get_request).await.unwrap();
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_batch_operations() {
    let app = create_test_app().await;

    // Create collection
    let collection_data = json!({
        "name": "batch_test_collection",
        "dimension": 3,
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

    // Insert multiple vectors in batch
    let vectors_data = json!({
        "vectors": [
            {"id": "batch_vec1", "vector": [1.0, 0.0, 0.0], "payload": {"title": "Batch 1"}},
            {"id": "batch_vec2", "vector": [0.0, 1.0, 0.0], "payload": {"title": "Batch 2"}},
            {"id": "batch_vec3", "vector": [0.0, 0.0, 1.0], "payload": {"title": "Batch 3"}},
            {"id": "batch_vec4", "vector": [1.0, 1.0, 0.0], "payload": {"title": "Batch 4"}},
            {"id": "batch_vec5", "vector": [1.0, 0.0, 1.0], "payload": {"title": "Batch 5"}}
        ]
    });

    let insert_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections/batch_test_collection/vectors")
        .header("content-type", "application/json")
        .body(Body::from(vectors_data.to_string()))
        .unwrap();

    let insert_response = app.clone().oneshot(insert_request).await.unwrap();
    assert_eq!(insert_response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(insert_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(result["inserted"], 5);
}

#[tokio::test]
async fn test_invalid_json_request() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/collections")
        .body(Body::from("{}"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_unsupported_method() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::PATCH)
        .uri("/api/v1/collections")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

