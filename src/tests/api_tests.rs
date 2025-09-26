//! API integration tests for Vectorizer

use crate::{
    api::server::VectorizerServer,
    db::VectorStore,
    embedding::EmbeddingManager,
    models::{CollectionConfig, DistanceMetric, HnswConfig, Vector},
};
use std::sync::Arc;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_health_check() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager);
    let app = server.create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_collections_list() {
    let store = Arc::new(VectorStore::new());

    // Create a test collection first
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("test_collection", config).unwrap();

    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager);
    let app = server.create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/collections")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[cfg(feature = "onnx-models")]
#[tokio::test]
async fn test_api_create_collection() {
    let store = Arc::new(VectorStore::new());
    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager);
    let app = server.create_app();

    let collection_config = serde_json::json!({
        "dimension": 384,
        "metric": "cosine",
        "hnsw_config": {
            "m": 16,
            "ef_construction": 200
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/collections/test_collection")
                .header("content-type", "application/json")
                .body(Body::from(collection_config.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[cfg(feature = "onnx-models")]
#[tokio::test]
async fn test_api_insert_vectors() {
    let store = Arc::new(VectorStore::new());

    // Create collection first
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("test_collection", config).unwrap();

    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager);
    let app = server.create_app();

    let vectors = serde_json::json!([
        {
            "id": "vec1",
            "data": [1.0, 2.0, 3.0]
        },
        {
            "id": "vec2",
            "data": [4.0, 5.0, 6.0]
        }
    ]);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/collections/test_collection/vectors")
                .header("content-type", "application/json")
                .body(Body::from(vectors.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[cfg(feature = "onnx-models")]
#[tokio::test]
async fn test_api_search_vectors() {
    let store = Arc::new(VectorStore::new());

    // Create collection and insert vectors first
    let config = CollectionConfig {
        dimension: 3,
        metric: DistanceMetric::Euclidean,
        hnsw_config: HnswConfig::default(),
        quantization: None,
        compression: Default::default(),
    };
    store.create_collection("test_collection", config).unwrap();

    let vectors = vec![
        Vector::new("vec1".to_string(), vec![1.0, 0.0, 0.0]),
        Vector::new("vec2".to_string(), vec![0.0, 1.0, 0.0]),
        Vector::new("vec3".to_string(), vec![0.0, 0.0, 1.0]),
    ];
    store.insert("test_collection", vectors).unwrap();

    let embedding_manager = EmbeddingManager::new();
    let server = VectorizerServer::new("127.0.0.1", 0, store, embedding_manager);
    let app = server.create_app();

    let search_query = serde_json::json!({
        "vector": [1.0, 0.0, 0.0],
        "k": 2
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/collections/test_collection/search")
                .header("content-type", "application/json")
                .body(Body::from(search_query.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
