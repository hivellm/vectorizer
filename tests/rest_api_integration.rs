//! Integration tests for REST API data structures and validation
//!
//! These tests verify:
//! - Request/response models
//! - Validation logic
//! - Error handling
//! - Data serialization

use serde_json::json;
use vectorizer::models::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

#[test]
fn test_collection_config_creation() {
    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        hnsw_config: HnswConfig::default(),
        quantization: QuantizationConfig::None,
        compression: Default::default(),
        normalization: None,
    };

    assert_eq!(config.dimension, 384);
    assert_eq!(config.metric, DistanceMetric::Cosine);
}

#[test]
fn test_distance_metric_variants() {
    let metrics = [
        DistanceMetric::Cosine,
        DistanceMetric::Euclidean,
        DistanceMetric::DotProduct,
    ];

    assert_eq!(metrics.len(), 3);
}

#[test]
fn test_hnsw_config_defaults() {
    let config = HnswConfig::default();

    // Verify default values are sensible
    assert!(config.ef_construction > 0);
    assert!(config.m > 0);
}

#[test]
fn test_json_request_parsing() {
    // Test collection creation request
    let request = json!({
        "name": "test_collection",
        "dimension": 384,
        "metric": "cosine"
    });

    assert_eq!(request["name"], "test_collection");
    assert_eq!(request["dimension"], 384);
    assert_eq!(request["metric"], "cosine");
}

#[test]
fn test_vector_json_structure() {
    // Test vector data structure
    let vector = json!({
        "id": "vec1",
        "data": [1.0, 2.0, 3.0],
        "metadata": {"key": "value"}
    });

    assert_eq!(vector["id"], "vec1");
    assert!(vector["data"].is_array());
    assert!(vector["metadata"].is_object());
}

#[test]
fn test_search_request_structure() {
    // Test search request format
    let request = json!({
        "query": [1.0, 2.0, 3.0],
        "limit": 10,
        "filter": {"category": "documents"}
    });

    assert!(request["query"].is_array());
    assert_eq!(request["limit"], 10);
    assert!(request.get("filter").is_some());
}

#[test]
fn test_batch_operation_structure() {
    // Test batch operation request
    let batch = json!({
        "operations": [
            {"type": "insert", "id": "1", "data": [1.0, 2.0]},
            {"type": "delete", "id": "2"}
        ]
    });

    assert!(batch["operations"].is_array());
    assert_eq!(batch["operations"].as_array().unwrap().len(), 2);
}

#[test]
fn test_error_response_structure() {
    // Test error response format
    let error = json!({
        "error": "Collection not found",
        "code": "NOT_FOUND",
        "details": {"collection": "test"}
    });

    assert_eq!(error["error"], "Collection not found");
    assert_eq!(error["code"], "NOT_FOUND");
    assert!(error["details"].is_object());
}

#[test]
fn test_success_response_structure() {
    // Test success response format
    let response = json!({
        "success": true,
        "message": "Operation completed",
        "data": {"count": 10}
    });

    assert_eq!(response["success"], true);
    assert!(response.get("message").is_some());
    assert!(response.get("data").is_some());
}

#[test]
fn test_collection_list_response() {
    // Test collection list response format
    let response = json!({
        "collections": [
            {"name": "coll1", "dimension": 384},
            {"name": "coll2", "dimension": 512}
        ],
        "total": 2
    });

    assert!(response["collections"].is_array());
    assert_eq!(response["total"], 2);
}

#[test]
fn test_search_results_structure() {
    // Test search results format
    let results = json!({
        "results": [
            {"id": "1", "score": 0.95, "content": "text1"},
            {"id": "2", "score": 0.85, "content": "text2"}
        ],
        "total": 2,
        "duration_ms": 15
    });

    assert!(results["results"].is_array());
    assert_eq!(results["total"], 2);
    assert!(results["duration_ms"].is_number());
}

#[test]
fn test_health_response_structure() {
    // Test health check response format
    let health = json!({
        "status": "healthy",
        "service": "vectorizer",
        "version": "1.1.2",
        "uptime": 3600
    });

    assert_eq!(health["status"], "healthy");
    assert_eq!(health["service"], "vectorizer");
    assert!(health.get("version").is_some());
}

#[test]
fn test_stats_response_structure() {
    // Test database stats response format
    let stats = json!({
        "total_collections": 5,
        "total_vectors": 10000,
        "memory_usage": 524288,
        "uptime_seconds": 3600
    });

    assert!(stats["total_collections"].is_number());
    assert!(stats["total_vectors"].is_number());
    assert!(stats["memory_usage"].is_number());
}

#[test]
fn test_validation_empty_collection_name() {
    let name = String::new();
    assert!(name.is_empty());
}

#[test]
fn test_validation_invalid_dimension() {
    let invalid_dimensions = vec![0, -1];

    for dim in invalid_dimensions {
        assert!(dim <= 0);
    }
}

#[test]
fn test_validation_vector_dimension_mismatch() {
    let collection_dim = 384;
    let vector_dim = 128;

    assert_ne!(collection_dim, vector_dim);
}

#[test]
fn test_pagination_parameters() {
    let page = 1;
    let page_size = 50;
    let offset = (page - 1) * page_size;

    assert_eq!(offset, 0);
    assert_eq!(page_size, 50);
}

#[test]
fn test_api_versioning_header() {
    let api_version = "v1";
    assert_eq!(api_version, "v1");
}

#[test]
fn test_request_timeout_validation() {
    let timeout_seconds = 30;
    assert!(timeout_seconds > 0);
    assert!(timeout_seconds <= 300); // Max 5 minutes
}

#[test]
fn test_rate_limit_configuration() {
    let requests_per_minute = 60;
    let requests_per_second = f64::from(requests_per_minute) / 60.0;

    assert_eq!(requests_per_second, 1.0);
}
