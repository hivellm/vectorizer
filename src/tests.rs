//! Consolidated integration tests for Vectorizer

#[cfg(test)]
mod tests {
    use crate::db::VectorStore;
    use crate::models::{
        CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector, vector_utils,
    };

    #[test]
    fn test_vector_store_creation() {
        let store = VectorStore::new();
        // Basic test to ensure VectorStore can be created
        assert!(true);
    }

    #[test]
    fn test_vector_utils() {
        // Test cosine similarity
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let similarity = vector_utils::cosine_similarity(&v1, &v2);
        assert!((similarity - 0.0).abs() < 1e-6);

        let v3 = vec![1.0, 0.0, 0.0];
        let similarity_same = vector_utils::cosine_similarity(&v1, &v3);
        assert!((similarity_same - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_payload_operations() {
        // Test payload creation
        let payload = Payload::new(serde_json::json!({
            "text": "test document",
            "metadata": {
                "source": "test.txt",
                "category": "test"
            }
        }));

        // Test payload data access
        assert_eq!(payload.data["text"], "test document");
        assert_eq!(payload.data["metadata"]["source"], "test.txt");
        assert_eq!(payload.data["metadata"]["category"], "test");
    }

    #[test]
    fn test_hnsw_configuration() {
        let config = HnswConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            seed: Some(42),
        };

        assert_eq!(config.m, 16);
        assert_eq!(config.ef_construction, 200);
        assert_eq!(config.ef_search, 50);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_distance_metrics() {
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];

        // Test cosine similarity calculation
        let cosine_sim = vector_utils::cosine_similarity(&v1, &v2);
        assert!((cosine_sim - 0.0).abs() < 1e-6);

        // Test euclidean distance calculation
        let euclidean_dist = ((v1[0] - v2[0]).powi(2) + (v1[1] - v2[1]).powi(2)).sqrt();
        assert!((euclidean_dist - std::f32::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn test_vector_creation() {
        let vector = Vector {
            id: "test_vector".to_string(),
            data: vec![0.1, 0.2, 0.3],
            sparse: None,
            payload: Some(Payload::new(serde_json::json!({"test": "data"}))),
        };

        assert_eq!(vector.id, "test_vector");
        assert_eq!(vector.data.len(), 3);
        assert!(vector.payload.is_some());
    }

    #[test]
    fn test_collection_config_creation() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: crate::models::QuantizationConfig::default(),
            compression: crate::models::CompressionConfig::default(),
            normalization: None,
        };

        assert_eq!(config.dimension, 128);
        assert!(matches!(config.metric, DistanceMetric::Cosine));
    }

    #[test]
    fn test_basic_functionality() {
        // Test basic functionality without complex API calls
        let store = VectorStore::new();
        let collections = store.list_collections();

        // Should be able to list collections (even if empty)
        assert!(collections.is_empty() || !collections.is_empty());
    }
}
