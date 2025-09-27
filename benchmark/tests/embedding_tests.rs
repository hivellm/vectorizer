//! Specific unit tests for embedding functionalities
//!
//! This file contains focused tests on embedding implementations,
//! validating their isolated behavior.

use vectorizer::embedding::{EmbeddingManager, TfIdfEmbedding, Bm25Embedding, SvdEmbedding};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tfidf_deterministic_vocabulary() {
        // Test that TF-IDF generates deterministic vocabulary
        let documents = vec![
            "machine learning is awesome".to_string(),
            "artificial intelligence rocks".to_string(),
            "machine learning and ai".to_string(),
        ];

        let mut manager1 = EmbeddingManager::new();
        let mut manager2 = EmbeddingManager::new();

        let tfidf1 = TfIdfEmbedding::new(50);
        let tfidf2 = TfIdfEmbedding::new(50);

        manager1.register_provider("tfidf".to_string(), Box::new(tfidf1)).unwrap();
        manager2.register_provider("tfidf".to_string(), Box::new(tfidf2)).unwrap();

        // Build vocabulary from same documents
        if let Some(provider) = manager1.get_provider_mut("tfidf") {
            if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
        }

        if let Some(provider) = manager2.get_provider_mut("tfidf") {
            if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                tfidf.build_vocabulary(&documents.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
        }

        // Embed same query
        let embedding1 = manager1.embed("machine learning").unwrap();
        let embedding2 = manager2.embed("machine learning").unwrap();

        // Should be identical (deterministic)
        assert_eq!(embedding1, embedding2, "TF-IDF embeddings should be deterministic");
    }

    #[test]
    fn test_bm25_parameters() {
        // Test default BM25 parameters
        let bm25 = Bm25Embedding::new(50);

        // Verify parameters are at expected values
        // (we can't access directly, but can test behavior)
        let query = "test query";
        let embedding = bm25.embed(query).unwrap();

        assert!(!embedding.is_empty(), "BM25 should produce non-empty embedding");
        assert_eq!(embedding.len(), 50, "BM25 embedding should have correct dimension");
    }

    #[test]
    fn test_svd_transformation() {
        // Test SVD transformation
        let mut svd = SvdEmbedding::new(10, 100); // 10D reduced from 100 vocab

        let documents = vec![
            "machine learning algorithms".to_string(),
            "artificial intelligence systems".to_string(),
            "neural network models".to_string(),
        ];

        let doc_refs: Vec<&str> = documents.iter().map(|s| s.as_str()).collect();
        svd.fit_svd(&doc_refs).unwrap();

        // Test embedding
        let embedding = <SvdEmbedding as vectorizer::embedding::EmbeddingProvider>::embed(&svd, "machine learning").unwrap();
        assert_eq!(embedding.len(), 10, "SVD should reduce to target dimension");
    }

    #[test]
    fn test_embedding_consistency() {
        // Test embedding consistency for same input
        let mut manager = EmbeddingManager::new();
        let tfidf = TfIdfEmbedding::new(50);
        manager.register_provider("tfidf".to_string(), Box::new(tfidf)).unwrap();

        if let Some(provider) = manager.get_provider_mut("tfidf") {
            if let Some(tfidf) = provider.as_any_mut().downcast_mut::<TfIdfEmbedding>() {
                tfidf.build_vocabulary(&["test document".as_ref()]);
            }
        }

        let emb1 = manager.embed("test query").unwrap();
        let emb2 = manager.embed("test query").unwrap();

        assert_eq!(emb1, emb2, "Same input should produce identical embeddings");
    }
}
