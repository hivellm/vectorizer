//! Hybrid search combining sparse retrieval with dense re-ranking
//!
//! This module implements a two-stage retrieval pipeline:
//! 1. First stage: Sparse retrieval using BM25/TF-IDF to get top-k candidates
//! 2. Second stage: Re-ranking using dense embeddings (BERT, MiniLM, etc.)

use crate::embedding::EmbeddingProvider;
use crate::error::Result;
use crate::evaluation::QueryResult;

/// Hybrid search retriever combining sparse and dense methods
pub struct HybridRetriever<T: EmbeddingProvider, U: EmbeddingProvider> {
    /// Sparse retrieval method (BM25, TF-IDF, etc.)
    sparse_retriever: T,
    /// Dense re-ranking method (BERT, MiniLM, etc.)
    dense_reranker: U,
    /// Number of candidates to retrieve in first stage
    first_stage_k: usize,
}

impl<T: EmbeddingProvider, U: EmbeddingProvider> HybridRetriever<T, U> {
    /// Create a new hybrid retriever
    pub fn new(sparse_retriever: T, dense_reranker: U, first_stage_k: usize) -> Self {
        Self {
            sparse_retriever,
            dense_reranker,
            first_stage_k,
        }
    }

    /// Perform hybrid search on a collection of documents
    pub async fn search_hybrid(
        &self,
        query: &str,
        documents: &[String],
        final_k: usize,
    ) -> Result<Vec<QueryResult>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        // Stage 1: Sparse retrieval to get candidates
        let sparse_results = self
            .sparse_retrieval(query, documents, self.first_stage_k)
            .await?;

        // Stage 2: Dense re-ranking of candidates
        let reranked_results = self
            .dense_reranking(query, documents, sparse_results, final_k)
            .await?;

        Ok(reranked_results)
    }

    /// First stage: Sparse retrieval using BM25/TF-IDF
    async fn sparse_retrieval(
        &self,
        query: &str,
        documents: &[String],
        k: usize,
    ) -> Result<Vec<(usize, f32)>> {
        // Get query embedding using sparse method
        let query_embedding = self
            .sparse_retriever
            .embed(query)
            .await
            .map_err(|e| crate::error::VectorizerError::EmbeddingError(e.to_string()))?;

        // Score all documents using sparse similarity
        let mut doc_scores: Vec<(usize, f32)> = Vec::with_capacity(documents.len());
        for (idx, doc) in documents.iter().enumerate() {
            let doc_embedding = self.sparse_retriever.embed(doc).await.unwrap_or_default();
            let score = cosine_similarity(&query_embedding, &doc_embedding);
            doc_scores.push((idx, score));
        }

        // Sort by score descending and take top k
        doc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        doc_scores.truncate(k);

        Ok(doc_scores)
    }

    /// Second stage: Dense re-ranking of sparse candidates
    async fn dense_reranking(
        &self,
        query: &str,
        documents: &[String],
        candidates: Vec<(usize, f32)>,
        final_k: usize,
    ) -> Result<Vec<QueryResult>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Get dense query embedding
        let query_embedding = self
            .dense_reranker
            .embed(query)
            .await
            .map_err(|e| crate::error::VectorizerError::EmbeddingError(e.to_string()))?;

        // Re-score candidates using dense embeddings
        let mut reranked_results: Vec<QueryResult> = Vec::with_capacity(candidates.len());
        for (doc_idx, _sparse_score) in candidates {
            let doc_text = &documents[doc_idx];
            let doc_embedding = self
                .dense_reranker
                .embed(doc_text)
                .await
                .unwrap_or_default();
            let dense_score = cosine_similarity(&query_embedding, &doc_embedding);

            reranked_results.push(QueryResult {
                doc_id: format!("doc_{}", doc_idx),
                relevance: dense_score,
            });
        }

        // Sort by dense score and take final k
        reranked_results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        reranked_results.truncate(final_k);

        Ok(reranked_results)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

// Note: Convenience factory functions removed as they're not needed with current EmbeddingManager API
// Users should create HybridRetriever directly with their preferred embedding providers

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let similarity = cosine_similarity(&a, &b);
        assert!((similarity - 0.974).abs() < 0.001); // Expected: 32 / (sqrt(14) * sqrt(77))
    }
}
