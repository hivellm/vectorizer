//! Model tests for the Rust SDK
//! Tests for Vector, Collection, SearchResult, and Batch models

use std::collections::HashMap;
use vectorizer_sdk::*;

#[test]
fn test_vector_creation() {
    let data = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let metadata = Some({
        let mut meta = HashMap::new();
        meta.insert(
            "category".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        meta.insert(
            "source".to_string(),
            serde_json::Value::String("test_doc".to_string()),
        );
        meta
    });

    let vector = Vector {
        id: "test_vector_1".to_string(),
        data: data.clone(),
        metadata: metadata.clone(),
    };

    assert_eq!(vector.id, "test_vector_1");
    assert_eq!(vector.data, data);
    assert_eq!(vector.metadata, metadata);
}

#[test]
fn test_vector_validation() {
    // Test valid vector data
    let valid_data = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let vector = Vector {
        id: "valid_vector".to_string(),
        data: valid_data,
        metadata: None,
    };
    assert_eq!(vector.data.len(), 5);
    assert!(vector.data.iter().all(|&x| x.is_finite()));

    // Test vector with NaN values (should be handled by validation)
    let invalid_data = vec![0.1, f32::NAN, 0.3];
    let vector = Vector {
        id: "invalid_vector".to_string(),
        data: invalid_data,
        metadata: None,
    };
    // Note: In Rust, we can't prevent NaN at compile time, but we can validate at runtime
    assert!(vector.data.iter().any(|&x| x.is_nan()));

    // Test vector with Infinity values
    let infinity_data = vec![0.1, f32::INFINITY, 0.3];
    let vector = Vector {
        id: "infinity_vector".to_string(),
        data: infinity_data,
        metadata: None,
    };
    assert!(vector.data.iter().any(|&x| x.is_infinite()));
}

#[test]
fn test_collection_creation() {
    let collection = Collection {
        name: "test_collection".to_string(),
        dimension: 384,
        similarity_metric: SimilarityMetric::Cosine,
        description: Some("Test collection".to_string()),
        created_at: None,
        updated_at: None,
    };

    assert_eq!(collection.name, "test_collection");
    assert_eq!(collection.dimension, 384);
    assert_eq!(collection.similarity_metric, SimilarityMetric::Cosine);
    assert_eq!(collection.description, Some("Test collection".to_string()));
}

#[test]
fn test_collection_info_creation() {
    let indexing_status = IndexingStatus {
        status: "ready".to_string(),
        progress: 100.0,
        total_documents: 100,
        processed_documents: 100,
        vector_count: 100,
        estimated_time_remaining: None,
        last_updated: "2024-01-01T00:00:00Z".to_string(),
    };

    let collection_info = CollectionInfo {
        name: "test_collection_info".to_string(),
        dimension: 768,
        metric: "cosine".to_string(),
        vector_count: 100,
        document_count: 50,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        indexing_status,
    };

    assert_eq!(collection_info.name, "test_collection_info");
    assert_eq!(collection_info.dimension, 768);
    assert_eq!(collection_info.metric, "cosine");
    assert_eq!(collection_info.vector_count, 100);
    assert_eq!(collection_info.document_count, 50);
}

#[test]
fn test_search_result_creation() {
    let metadata = Some({
        let mut meta = HashMap::new();
        meta.insert(
            "category".to_string(),
            serde_json::Value::String("ai".to_string()),
        );
        meta.insert(
            "confidence".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.95).unwrap()),
        );
        meta
    });

    let search_result = SearchResult {
        id: "search_result_1".to_string(),
        score: 0.95,
        content: Some("This is a search result".to_string()),
        metadata,
    };

    assert_eq!(search_result.id, "search_result_1");
    assert_eq!(search_result.score, 0.95);
    assert_eq!(
        search_result.content,
        Some("This is a search result".to_string())
    );
}

#[test]
fn test_search_response_creation() {
    let results = vec![
        SearchResult {
            id: "result_1".to_string(),
            score: 0.95,
            content: Some("First result".to_string()),
            metadata: None,
        },
        SearchResult {
            id: "result_2".to_string(),
            score: 0.87,
            content: Some("Second result".to_string()),
            metadata: None,
        },
    ];

    let search_response = SearchResponse {
        results: results.clone(),
        query_time_ms: 15.5,
    };

    assert_eq!(search_response.results.len(), 2);
    assert_eq!(search_response.query_time_ms, 15.5);
    assert_eq!(search_response.results[0].score, 0.95);
    assert_eq!(search_response.results[1].score, 0.87);
}

#[test]
fn test_batch_text_request_creation() {
    let metadata = Some({
        let mut meta = HashMap::new();
        meta.insert("category".to_string(), "ai".to_string());
        meta.insert("source".to_string(), "test".to_string());
        meta
    });

    let batch_request = BatchTextRequest {
        id: "batch_text_1".to_string(),
        text: "This is a batch text request".to_string(),
        metadata,
    };

    assert_eq!(batch_request.id, "batch_text_1");
    assert_eq!(batch_request.text, "This is a batch text request");
}

#[test]
fn test_batch_config_creation() {
    let config = BatchConfig {
        max_batch_size: Some(100),
        parallel_workers: Some(4),
        atomic: Some(true),
    };

    assert_eq!(config.max_batch_size, Some(100));
    assert_eq!(config.parallel_workers, Some(4));
    assert_eq!(config.atomic, Some(true));
}

#[test]
fn test_batch_insert_request_creation() {
    let texts = vec![
        BatchTextRequest {
            id: "batch_1".to_string(),
            text: "First batch text".to_string(),
            metadata: None,
        },
        BatchTextRequest {
            id: "batch_2".to_string(),
            text: "Second batch text".to_string(),
            metadata: None,
        },
    ];

    let config = BatchConfig {
        max_batch_size: Some(50),
        parallel_workers: Some(2),
        atomic: Some(false),
    };

    let batch_insert = BatchInsertRequest {
        texts: texts.clone(),
        config: Some(config),
    };

    assert_eq!(batch_insert.texts.len(), 2);
    assert_eq!(batch_insert.texts[0].id, "batch_1");
    assert_eq!(batch_insert.texts[1].id, "batch_2");
    assert!(batch_insert.config.is_some());
}

#[test]
fn test_batch_response_creation() {
    let batch_response = BatchResponse {
        success: true,
        collection: "test_collection".to_string(),
        operation: "insert".to_string(),
        total_operations: 10,
        successful_operations: 10,
        failed_operations: 0,
        duration_ms: 150,
        errors: vec![],
    };

    assert!(batch_response.success);
    assert_eq!(batch_response.collection, "test_collection");
    assert_eq!(batch_response.operation, "insert");
    assert_eq!(batch_response.total_operations, 10);
    assert_eq!(batch_response.successful_operations, 10);
    assert_eq!(batch_response.failed_operations, 0);
    assert_eq!(batch_response.duration_ms, 150);
    assert!(batch_response.errors.is_empty());
}

#[test]
fn test_batch_search_query_creation() {
    let query = BatchSearchQuery {
        query: "machine learning".to_string(),
        limit: Some(10),
        score_threshold: Some(0.8),
    };

    assert_eq!(query.query, "machine learning");
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.score_threshold, Some(0.8));
}

#[test]
fn test_batch_search_request_creation() {
    let queries = vec![
        BatchSearchQuery {
            query: "first query".to_string(),
            limit: Some(5),
            score_threshold: None,
        },
        BatchSearchQuery {
            query: "second query".to_string(),
            limit: Some(10),
            score_threshold: Some(0.7),
        },
    ];

    let batch_search = BatchSearchRequest {
        queries: queries.clone(),
        config: None,
    };

    assert_eq!(batch_search.queries.len(), 2);
    assert_eq!(batch_search.queries[0].query, "first query");
    assert_eq!(batch_search.queries[1].query, "second query");
}

#[test]
fn test_embedding_request_creation() {
    let parameters = EmbeddingParameters {
        max_length: Some(512),
        normalize: Some(true),
        prefix: Some("query: ".to_string()),
    };

    let embedding_request = EmbeddingRequest {
        text: "This is a test text".to_string(),
        model: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
        parameters: Some(parameters),
    };

    assert_eq!(embedding_request.text, "This is a test text");
    assert_eq!(
        embedding_request.model,
        Some("sentence-transformers/all-MiniLM-L6-v2".to_string())
    );
    assert!(embedding_request.parameters.is_some());
}

#[test]
fn test_embedding_response_creation() {
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    let embedding_response = EmbeddingResponse {
        embedding: embedding.clone(),
        model: "test-model".to_string(),
        text: "test text".to_string(),
        dimension: 5,
        provider: "test-provider".to_string(),
    };

    assert_eq!(embedding_response.embedding, embedding);
    assert_eq!(embedding_response.model, "test-model");
    assert_eq!(embedding_response.text, "test text");
    assert_eq!(embedding_response.dimension, 5);
    assert_eq!(embedding_response.provider, "test-provider");
}

#[test]
fn test_health_status_creation() {
    let health = HealthStatus {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        uptime: Some(3600),
        collections: Some(5),
        total_vectors: Some(1000),
    };

    assert_eq!(health.status, "healthy");
    assert_eq!(health.version, "0.1.0");
    assert_eq!(health.uptime, Some(3600));
    assert_eq!(health.collections, Some(5));
    assert_eq!(health.total_vectors, Some(1000));
}

#[test]
fn test_serialization_deserialization() {
    // Test Vector serialization
    let vector = Vector {
        id: "test_serialization".to_string(),
        data: vec![0.1, 0.2, 0.3],
        metadata: None,
    };

    let json = serde_json::to_string(&vector).unwrap();
    let deserialized: Vector = serde_json::from_str(&json).unwrap();
    assert_eq!(vector.id, deserialized.id);
    assert_eq!(vector.data, deserialized.data);

    // Test Collection serialization
    let collection = Collection {
        name: "test_collection".to_string(),
        dimension: 384,
        similarity_metric: SimilarityMetric::Cosine,
        description: None,
        created_at: None,
        updated_at: None,
    };

    let json = serde_json::to_string(&collection).unwrap();
    let deserialized: Collection = serde_json::from_str(&json).unwrap();
    assert_eq!(collection.name, deserialized.name);
    assert_eq!(collection.dimension, deserialized.dimension);
    assert_eq!(collection.similarity_metric, deserialized.similarity_metric);
}

#[test]
fn test_similarity_metric_serialization() {
    let cosine = SimilarityMetric::Cosine;
    let euclidean = SimilarityMetric::Euclidean;
    let dot_product = SimilarityMetric::DotProduct;

    assert_eq!(serde_json::to_string(&cosine).unwrap(), "\"cosine\"");
    assert_eq!(serde_json::to_string(&euclidean).unwrap(), "\"euclidean\"");
    assert_eq!(
        serde_json::to_string(&dot_product).unwrap(),
        "\"dot_product\""
    );
}

#[test]
fn test_summarization_method_serialization() {
    let extractive = SummarizationMethod::Extractive;
    let keyword = SummarizationMethod::Keyword;
    let sentence = SummarizationMethod::Sentence;
    let abstractive = SummarizationMethod::Abstractive;

    assert_eq!(
        serde_json::to_string(&extractive).unwrap(),
        "\"extractive\""
    );
    assert_eq!(serde_json::to_string(&keyword).unwrap(), "\"keyword\"");
    assert_eq!(serde_json::to_string(&sentence).unwrap(), "\"sentence\"");
    assert_eq!(
        serde_json::to_string(&abstractive).unwrap(),
        "\"abstractive\""
    );
}

#[test]
fn test_default_values() {
    assert_eq!(SimilarityMetric::default(), SimilarityMetric::Cosine);
    assert_eq!(
        SummarizationMethod::default(),
        SummarizationMethod::Extractive
    );
}

#[test]
fn test_model_validation_edge_cases() {
    // Test empty vector data
    let empty_vector = Vector {
        id: "empty".to_string(),
        data: vec![],
        metadata: None,
    };
    assert!(empty_vector.data.is_empty());

    // Test large vector data
    let large_data: Vec<f32> = (0..1000).map(|i| i as f32 * 0.001).collect();
    let large_vector = Vector {
        id: "large".to_string(),
        data: large_data.clone(),
        metadata: None,
    };
    assert_eq!(large_vector.data.len(), 1000);
    assert_eq!(large_vector.data[0], 0.0);
    assert!((large_vector.data[999] - 0.999).abs() < 0.001); // Allow for floating point precision

    // Test vector with zero values
    let zero_vector = Vector {
        id: "zero".to_string(),
        data: vec![0.0, 0.0, 0.0],
        metadata: None,
    };
    assert!(zero_vector.data.iter().all(|&x| x == 0.0));
}
