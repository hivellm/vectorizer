//! Hybrid search combining sparse retrieval with dense re-ranking
//!
//! This module implements a two-stage retrieval pipeline:
//! 1. First stage: Sparse retrieval using BM25/TF-IDF to get top-k candidates
//! 2. Second stage: Re-ranking using dense embeddings (BERT, MiniLM, etc.)

use crate::embedding::EmbeddingProvider;
use crate::evaluation::QueryResult;
use crate::error::Result;

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
    pub fn search_hybrid(
        &self,
        query: &str,
        documents: &[String],
        final_k: usize,
    ) -> Result<Vec<QueryResult>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        // Stage 1: Sparse retrieval to get candidates
        let sparse_results = self.sparse_retrieval(query, documents, self.first_stage_k)?;

        // Stage 2: Dense re-ranking of candidates
        let reranked_results = self.dense_reranking(query, documents, sparse_results, final_k)?;

        Ok(reranked_results)
    }

    /// First stage: Sparse retrieval using BM25/TF-IDF
    fn sparse_retrieval(
        &self,
        query: &str,
        documents: &[String],
        k: usize,
    ) -> Result<Vec<(usize, f32)>> {
        // Get query embedding using sparse method
        let query_embedding = self.sparse_retriever.embed(query)?;

        // Score all documents using sparse similarity
        let mut doc_scores: Vec<(usize, f32)> = documents
            .iter()
            .enumerate()
            .map(|(idx, doc)| {
                let doc_embedding = self.sparse_retriever.embed(doc).unwrap_or_default();
                let score = cosine_similarity(&query_embedding, &doc_embedding);
                (idx, score)
            })
            .collect();

        // Sort by score descending and take top k
        doc_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        doc_scores.truncate(k);

        Ok(doc_scores)
    }

    /// Second stage: Dense re-ranking of sparse candidates
    fn dense_reranking(
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
        let query_embedding = self.dense_reranker.embed(query)?;

        // Re-score candidates using dense embeddings
        let mut reranked_results: Vec<QueryResult> = candidates
            .into_iter()
            .map(|(doc_idx, _sparse_score)| {
                let doc_text = &documents[doc_idx];
                let doc_embedding = self.dense_reranker.embed(doc_text).unwrap_or_default();
                let dense_score = cosine_similarity(&query_embedding, &doc_embedding);

                QueryResult {
                    doc_id: format!("doc_{}", doc_idx),
                    relevance: dense_score,
                }
            })
            .collect();

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

/// Convenience function to create a BM25 + BERT hybrid retriever
pub fn create_bm25_bert_hybrid(
    bm25_vocab_size: usize,
    bert_dimension: usize,
    first_stage_k: usize,
) -> Result<HybridRetriever<crate::Bm25Embedding, crate::BertEmbedding>> {
    let bm25 = crate::Bm25Embedding::new(bm25_vocab_size);
    let mut bert = crate::BertEmbedding::new(bert_dimension);
    bert.load_model()?;

    Ok(HybridRetriever::new(bm25, bert, first_stage_k))
}

/// Convenience function to create a BM25 + MiniLM hybrid retriever
pub fn create_bm25_minilm_hybrid(
    bm25_vocab_size: usize,
    minilm_dimension: usize,
    first_stage_k: usize,
) -> Result<HybridRetriever<crate::Bm25Embedding, crate::MiniLmEmbedding>> {
    let bm25 = crate::Bm25Embedding::new(bm25_vocab_size);
    let mut minilm = crate::MiniLmEmbedding::new(minilm_dimension);
    minilm.load_model()?;

    Ok(HybridRetriever::new(bm25, minilm, first_stage_k))
}

/// Convenience function to create a TF-IDF + SVD + BERT hybrid retriever
pub fn create_tfidf_svd_bert_hybrid(
    vocab_size: usize,
    svd_dimension: usize,
    bert_dimension: usize,
    first_stage_k: usize,
) -> Result<HybridRetriever<crate::SvdEmbedding, crate::BertEmbedding>> {
    let svd = crate::SvdEmbedding::new(svd_dimension, vocab_size);
    let mut bert = crate::BertEmbedding::new(bert_dimension);
    bert.load_model()?;

    Ok(HybridRetriever::new(svd, bert, first_stage_k))
}

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

    #[test]
    fn test_hybrid_retriever_creation() {
        let result = create_bm25_bert_hybrid(1000, 768, 100);
        assert!(result.is_ok());
    }
}
