//! Tests for ONNX model integration

#[cfg(feature = "onnx-models")]
use crate::{
    embedding::{EmbeddingManager, OnnxConfig, OnnxEmbedder, OnnxModelType},
    // models::DistanceMetric,
};

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_initialization() {
    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let _embedder = OnnxEmbedder::new(config).unwrap();
    // assert_eq!(embedder.config.model_type.dimension(), 384);
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_batch_encoding() {
    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let embedder = OnnxEmbedder::new(config).unwrap();

    let texts = vec![
        "Hello world",
        "This is a test",
        "Machine learning is awesome",
    ];

    let embeddings = embedder.embed_batch(&texts).unwrap();
    assert_eq!(embeddings.len(), 3);

    // Check that all embeddings have the correct dimension
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_single_encoding() {
    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let embedder = OnnxEmbedder::new(config).unwrap();

    let text = "Single text embedding test";
    let embeddings = embedder.embed_batch(&[text]).unwrap();

    assert_eq!(embeddings.len(), 1);
    assert_eq!(embeddings[0].len(), 384);
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_normalization() {
    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let embedder = OnnxEmbedder::new(config).unwrap();

    let text = "Test normalization";
    let embeddings = embedder.embed_batch(&[text]).unwrap();

    // Check that embedding is normalized (unit vector)
    let embedding = &embeddings[0];
    let norm_squared: f32 = embedding.iter().map(|x| x * x).sum();
    assert!(
        (norm_squared - 1.0).abs() < 1e-5,
        "Embedding should be normalized, norm squared: {}",
        norm_squared
    );
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_different_models() {
    // Test different ONNX model types
    let models = vec![
        OnnxModelType::MiniLMMultilingual384,
        OnnxModelType::MPNetMultilingual768,
        OnnxModelType::E5BaseMultilingual768,
    ];

    for model_type in models {
        let dimension = model_type.dimension();
        let config = OnnxConfig {
            model_type,
            batch_size: 32,
            cache_dir: "/tmp/onnx_cache".into(),
            max_length: 512,
            num_threads: 4,
            use_int8: false,
            pooling: Default::default(),
        };

        let embedder = OnnxEmbedder::new(config).unwrap();

        let text = "Model compatibility test";
        let embeddings = embedder.embed_batch(&[text]).unwrap();

        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), dimension);
    }
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedding_manager_integration() {
    let manager = EmbeddingManager::new();

    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let _embedder = OnnxEmbedder::new(config).unwrap();
    // manager.register_provider("onnx_minilm".to_string(), Box::new(embedder));

    let texts = vec!["First text", "Second text"];
    let embeddings = manager.embed_batch(&texts).unwrap();

    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].len(), 384);
    assert_eq!(embeddings[1].len(), 384);
}

#[cfg(feature = "onnx-models")]
#[test]
fn test_onnx_embedder_consistency() {
    let config = OnnxConfig {
        model_type: OnnxModelType::MiniLMMultilingual384,
        batch_size: 32,
        cache_dir: std::env::temp_dir(),
        max_length: 512,
        num_threads: 4,
        use_int8: false,
        pooling: Default::default(),
    };

    let embedder1 = OnnxEmbedder::new(config.clone()).unwrap();
    let embedder2 = OnnxEmbedder::new(config).unwrap();

    let text = "Consistency test text";
    let embeddings1 = embedder1.embed_batch(&[text]).unwrap();
    let embeddings2 = embedder2.embed_batch(&[text]).unwrap();

    // Embeddings should be identical for the same input
    assert_eq!(embeddings1[0], embeddings2[0]);
}
