//! Tests for real transformer models using Candle

#[cfg(feature = "candle-models")]
use crate::{
    embedding::{EmbeddingManager, EmbeddingProvider, RealModelEmbedder, RealModelType},
    models::DistanceMetric,
};

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_initialization() {
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();
    assert_eq!(embedder.dimension, 384);
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_batch_encoding() {
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();

    let texts = vec![
        "Hello world",
        "This is a test",
        "Machine learning is awesome",
    ];

    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let embeddings = EmbeddingProvider::embed_batch(&embedder, &text_refs).unwrap();
    assert_eq!(embeddings.len(), 3);

    // Check that all embeddings have the correct dimension
    for embedding in &embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_normalization() {
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();

    let text = "Test normalization";
    let embeddings = EmbeddingProvider::embed_batch(&embedder, &[text]).unwrap();

    // Check that embedding is normalized (unit vector)
    let embedding = &embeddings[0];
    let norm_squared: f32 = embedding.iter().map(|x| x * x).sum();
    assert!(
        (norm_squared - 1.0).abs() < 1e-5,
        "Embedding should be normalized, norm squared: {}",
        norm_squared
    );
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_different_models() {
    // Test different real model types
    let models = vec![
        RealModelType::MiniLMMultilingual,
        RealModelType::DistilUseMultilingual,
        RealModelType::MPNetMultilingualBase,
    ];

    for model_type in models {
        let embedder = RealModelEmbedder::new(model_type).unwrap();

        let text = "Model compatibility test";
        let embeddings = EmbeddingProvider::embed_batch(&embedder, &[text]).unwrap();

        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), model_type.dimension());
    }
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedding_manager_integration() {
    let mut manager = EmbeddingManager::new();
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();
    manager.register_provider("real_minilm".to_string(), Box::new(embedder));
    manager.set_default_provider("real_minilm").unwrap();

    let texts = vec!["First text", "Second text"];
    let text_refs: Vec<&str> = texts.iter().map(|s| *s).collect();
    let embeddings = manager.embed_batch(&text_refs).unwrap();

    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].len(), 384);
    assert_eq!(embeddings[1].len(), 384);
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_consistency() {
    let embedder1 = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();
    let embedder2 = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();

    let text = "Consistency test text";
    let embeddings1 = EmbeddingProvider::embed_batch(&embedder1, &[text]).unwrap();
    let embeddings2 = EmbeddingProvider::embed_batch(&embedder2, &[text]).unwrap();

    // Embeddings should be identical for the same input
    assert_eq!(embeddings1[0], embeddings2[0]);
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_large_batch() {
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();

    // Test with larger batch
    let texts: Vec<String> = (0..10).map(|i| format!("Test text number {}", i)).collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let embeddings = EmbeddingProvider::embed_batch(&embedder, &text_refs).unwrap();
    assert_eq!(embeddings.len(), 10);

    for embedding in &embeddings {
        assert_eq!(embedding.len(), 384);
    }
}

#[cfg(feature = "candle-models")]
#[test]
fn test_real_model_embedder_empty_input() {
    let embedder = RealModelEmbedder::new(RealModelType::MiniLMMultilingual).unwrap();

    let texts: Vec<&str> = vec![];
    let embeddings = EmbeddingProvider::embed_batch(&embedder, &texts).unwrap();

    assert_eq!(embeddings.len(), 0);
}
