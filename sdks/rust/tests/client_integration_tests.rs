//! Client integration tests for the Rust SDK
//! Tests for client operations, model integration, and data transformation

use std::collections::HashMap;
use vectorizer_sdk::*;

#[test]
fn test_client_initialization() {
    // Test default client initialization
    let client = VectorizerClient::new_default().unwrap();
    assert_eq!(client.base_url(), "http://localhost:15002");

    // Test custom URL initialization
    let client_custom = VectorizerClient::new_with_url("http://custom:8080").unwrap();
    assert_eq!(client_custom.base_url(), "http://custom:8080");

    // Test API key initialization
    let client_with_key =
        VectorizerClient::new_with_api_key("http://localhost:15002", "test-key").unwrap();
    assert_eq!(client_with_key.base_url(), "http://localhost:15002");
}

#[test]
fn test_vector_model_validation() {
    // Test Vector model creation and validation
    let valid_data = vec![0.1, 0.2, 0.3, 0.4, 0.5];
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
        data: valid_data.clone(),
        metadata: metadata.clone(),
    };

    // Validate vector properties
    assert_eq!(vector.id, "test_vector_1");
    assert_eq!(vector.data.len(), 5);
    assert!(vector.data.iter().all(|&x| x.is_finite()));
    assert!(vector.metadata.is_some());

    // Test serialization
    let json = serde_json::to_string(&vector).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: Vector = serde_json::from_str(&json).unwrap();
    assert_eq!(vector.id, deserialized.id);
    assert_eq!(vector.data, deserialized.data);
}

#[test]
fn test_collection_model_validation() {
    // Test Collection model creation and validation
    let collection = Collection {
        name: "test_collection".to_string(),
        dimension: 384,
        similarity_metric: SimilarityMetric::Cosine,
        description: Some("Test collection for validation".to_string()),
        created_at: None,
        updated_at: None,
    };

    // Validate collection properties
    assert_eq!(collection.name, "test_collection");
    assert_eq!(collection.dimension, 384);
    assert_eq!(collection.similarity_metric, SimilarityMetric::Cosine);
    assert!(collection.description.is_some());
    assert_eq!(
        collection.description.clone().unwrap(),
        "Test collection for validation"
    );

    // Test serialization
    let json = serde_json::to_string(&collection).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: Collection = serde_json::from_str(&json).unwrap();
    assert_eq!(collection.name, deserialized.name);
    assert_eq!(collection.dimension, deserialized.dimension);
    assert_eq!(collection.similarity_metric, deserialized.similarity_metric);
}

#[test]
fn test_search_result_model_validation() {
    // Test SearchResult model creation and validation
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

    // Validate search result properties
    assert_eq!(search_result.id, "search_result_1");
    assert_eq!(search_result.score, 0.95);
    assert!(search_result.score >= 0.0 && search_result.score <= 1.0);
    assert!(search_result.content.is_some());
    assert_eq!(
        search_result.content.clone().unwrap(),
        "This is a search result"
    );

    // Test serialization
    let json = serde_json::to_string(&search_result).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(search_result.id, deserialized.id);
    assert_eq!(search_result.score, deserialized.score);
}

#[test]
fn test_batch_models_integration() {
    // Test BatchTextRequest creation
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

    // Test BatchConfig creation
    let config = BatchConfig {
        max_batch_size: Some(100),
        parallel_workers: Some(4),
        atomic: Some(true),
    };

    // Test BatchInsertRequest creation
    let batch_insert = BatchInsertRequest {
        texts: vec![batch_request.clone()],
        config: Some(config.clone()),
    };

    // Validate batch models
    assert_eq!(batch_request.id, "batch_text_1");
    assert_eq!(batch_request.text, "This is a batch text request");
    assert!(batch_request.metadata.is_some());

    assert_eq!(config.max_batch_size, Some(100));
    assert_eq!(config.parallel_workers, Some(4));
    assert_eq!(config.atomic, Some(true));

    assert_eq!(batch_insert.texts.len(), 1);
    assert!(batch_insert.config.is_some());

    // Test serialization
    let json = serde_json::to_string(&batch_insert).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: BatchInsertRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(batch_insert.texts.len(), deserialized.texts.len());
    assert_eq!(batch_insert.texts[0].id, deserialized.texts[0].id);
}

#[test]
fn test_embedding_models_integration() {
    // Test EmbeddingRequest creation
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

    // Test EmbeddingResponse creation
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let embedding_response = EmbeddingResponse {
        embedding: embedding.clone(),
        model: "test-model".to_string(),
        text: "test text".to_string(),
        dimension: 5,
        provider: "test-provider".to_string(),
    };

    // Validate embedding models
    assert_eq!(embedding_request.text, "This is a test text");
    assert_eq!(
        embedding_request.model,
        Some("sentence-transformers/all-MiniLM-L6-v2".to_string())
    );
    assert!(embedding_request.parameters.is_some());

    assert_eq!(embedding_response.embedding, embedding);
    assert_eq!(embedding_response.model, "test-model");
    assert_eq!(embedding_response.dimension, 5);
    assert_eq!(embedding_response.provider, "test-provider");

    // Test serialization
    let request_json = serde_json::to_string(&embedding_request).unwrap();
    let response_json = serde_json::to_string(&embedding_response).unwrap();
    assert!(!request_json.is_empty());
    assert!(!response_json.is_empty());
}

#[test]
fn test_health_status_integration() {
    // Test HealthStatus model
    let health = HealthStatus {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        uptime: Some(3600),
        collections: Some(5),
        total_vectors: Some(1000),
    };

    // Validate health status
    assert_eq!(health.status, "healthy");
    assert_eq!(health.version, "0.1.0");
    assert_eq!(health.uptime, Some(3600));
    assert_eq!(health.collections, Some(5));
    assert_eq!(health.total_vectors, Some(1000));

    // Test serialization
    let json = serde_json::to_string(&health).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(health.status, deserialized.status);
    assert_eq!(health.version, deserialized.version);
}

#[test]
fn test_data_transformation_consistency() {
    // Test that data transformations are consistent
    let original_vector = Vector {
        id: "transform_test".to_string(),
        data: vec![0.1, 0.2, 0.3],
        metadata: None,
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&original_vector).unwrap();
    let transformed_vector: Vector = serde_json::from_str(&json).unwrap();

    // Verify consistency
    assert_eq!(original_vector.id, transformed_vector.id);
    assert_eq!(original_vector.data, transformed_vector.data);
    assert_eq!(original_vector.metadata, transformed_vector.metadata);
}

#[test]
fn test_model_edge_cases() {
    // Test edge cases for models

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

    // Test zero values
    let zero_vector = Vector {
        id: "zero".to_string(),
        data: vec![0.0, 0.0, 0.0],
        metadata: None,
    };
    assert!(zero_vector.data.iter().all(|&x| x == 0.0));

    // Test special floating point values
    let special_vector = Vector {
        id: "special".to_string(),
        data: vec![f32::NAN, f32::INFINITY, f32::NEG_INFINITY],
        metadata: None,
    };
    assert!(special_vector.data[0].is_nan());
    assert!(special_vector.data[1].is_infinite());
    assert!(special_vector.data[2].is_infinite());
}

#[test]
fn test_collection_info_integration() {
    // Test CollectionInfo model
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

    // Validate collection info
    assert_eq!(collection_info.name, "test_collection_info");
    assert_eq!(collection_info.dimension, 768);
    assert_eq!(collection_info.metric, "cosine");
    assert_eq!(collection_info.vector_count, 100);
    assert_eq!(collection_info.document_count, 50);
    assert_eq!(collection_info.indexing_status.status, "ready");
    assert_eq!(collection_info.indexing_status.progress, 100.0);

    // Test serialization
    let json = serde_json::to_string(&collection_info).unwrap();
    assert!(!json.is_empty());

    // Test deserialization
    let deserialized: CollectionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(collection_info.name, deserialized.name);
    assert_eq!(collection_info.dimension, deserialized.dimension);
}

#[test]
fn test_similarity_metric_enum_integration() {
    // Test SimilarityMetric enum
    let metrics = vec![
        SimilarityMetric::Cosine,
        SimilarityMetric::Euclidean,
        SimilarityMetric::DotProduct,
    ];

    for metric in metrics {
        // Test serialization
        let json = serde_json::to_string(&metric).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: SimilarityMetric = serde_json::from_str(&json).unwrap();
        assert_eq!(metric, deserialized);
    }

    // Test default value
    assert_eq!(SimilarityMetric::default(), SimilarityMetric::Cosine);
}

#[test]
fn test_summarization_method_enum_integration() {
    // Test SummarizationMethod enum
    let methods = vec![
        SummarizationMethod::Extractive,
        SummarizationMethod::Keyword,
        SummarizationMethod::Sentence,
        SummarizationMethod::Abstractive,
    ];

    for method in methods {
        // Test serialization
        let json = serde_json::to_string(&method).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: SummarizationMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, deserialized);
    }

    // Test default value
    assert_eq!(
        SummarizationMethod::default(),
        SummarizationMethod::Extractive
    );
}

#[test]
fn test_comprehensive_model_integration() {
    // Test comprehensive integration of all models

    // Create a complete workflow scenario
    let vector = Vector {
        id: "workflow_vector".to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert(
                "workflow".to_string(),
                serde_json::Value::String("test".to_string()),
            );
            meta
        }),
    };

    let collection = Collection {
        name: "workflow_collection".to_string(),
        dimension: 5,
        similarity_metric: SimilarityMetric::Cosine,
        description: Some("Workflow test collection".to_string()),
        created_at: None,
        updated_at: None,
    };

    let search_result = SearchResult {
        id: vector.id.clone(),
        score: 0.95,
        content: Some("Workflow test content".to_string()),
        metadata: vector.metadata.clone(),
    };

    let batch_request = BatchTextRequest {
        id: "workflow_batch".to_string(),
        text: "Workflow batch text".to_string(),
        metadata: Some({
            let mut meta = HashMap::new();
            meta.insert("workflow".to_string(), "batch".to_string());
            meta
        }),
    };

    // Validate all models work together
    assert_eq!(vector.data.len(), collection.dimension);
    assert_eq!(search_result.id, vector.id);
    assert!(search_result.score > 0.0);

    // Test serialization of all models
    let vector_json = serde_json::to_string(&vector).unwrap();
    let collection_json = serde_json::to_string(&collection).unwrap();
    let search_json = serde_json::to_string(&search_result).unwrap();
    let batch_json = serde_json::to_string(&batch_request).unwrap();

    assert!(!vector_json.is_empty());
    assert!(!collection_json.is_empty());
    assert!(!search_json.is_empty());
    assert!(!batch_json.is_empty());

    // Test deserialization
    let deserialized_vector: Vector = serde_json::from_str(&vector_json).unwrap();
    let deserialized_collection: Collection = serde_json::from_str(&collection_json).unwrap();
    let deserialized_search: SearchResult = serde_json::from_str(&search_json).unwrap();
    let deserialized_batch: BatchTextRequest = serde_json::from_str(&batch_json).unwrap();

    // Verify consistency
    assert_eq!(vector.id, deserialized_vector.id);
    assert_eq!(collection.name, deserialized_collection.name);
    assert_eq!(search_result.score, deserialized_search.score);
    assert_eq!(batch_request.text, deserialized_batch.text);
}
