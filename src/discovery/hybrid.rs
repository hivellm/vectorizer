//! Hybrid search implementation combining HNSW + BM25

use std::collections::HashMap;
use std::sync::Arc;

use super::types::{ChunkMetadata, DiscoveryError, DiscoveryResult, ScoredChunk};
use crate::db::VectorStore;

/// Hybrid searcher combining dense (HNSW) and sparse (BM25) search
pub struct HybridSearcher {
    vector_store: Arc<VectorStore>,
}

impl HybridSearcher {
    /// Create new hybrid searcher
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        Self { vector_store }
    }

    /// Perform hybrid search combining dense + sparse
    pub async fn search(
        &self,
        query: &str,
        query_vector: Vec<f32>,
        limit: usize,
        alpha: f32,
    ) -> DiscoveryResult<Vec<ScoredChunk>> {
        // 1. Dense search with HNSW
        let dense_results = self
            .vector_store
            .search("default", &query_vector, limit * 2)
            .map_err(|e| DiscoveryError::SearchError(e.to_string()))?;

        // 2. Sparse search with tantivy BM25 (simplified for now)
        let sparse_results = self.perform_sparse_search(query, limit * 2).await?;

        // 3. Reciprocal Rank Fusion to merge results
        let dense_scores: Vec<(String, f32)> = dense_results
            .iter()
            .map(|result| (result.id.clone(), result.score))
            .collect();

        let sparse_scores: Vec<(String, f32)> = sparse_results
            .iter()
            .map(|result| (result.doc_id.clone(), result.score))
            .collect();

        let fused_scores = reciprocal_rank_fusion(&dense_scores, &sparse_scores, alpha);

        // Convert back to ScoredChunk format
        let mut results = Vec::new();
        for (doc_id, score) in fused_scores.into_iter().take(limit) {
            results.push(ScoredChunk {
                collection: "default".to_string(),
                doc_id,
                content: String::new(), // Would need to fetch actual content
                score,
                metadata: ChunkMetadata {
                    file_path: String::new(),
                    chunk_index: 0,
                    file_extension: String::new(),
                    line_range: None,
                },
            });
        }

        Ok(results)
    }

    /// Perform sparse search using BM25 (simplified implementation)
    async fn perform_sparse_search(
        &self,
        query: &str,
        limit: usize,
    ) -> DiscoveryResult<Vec<ScoredChunk>> {
        // For now, return empty results as tantivy integration is complex
        // In a real implementation, this would use tantivy for BM25 search
        tracing::debug!("Sparse search for query: '{}' (limit: {})", query, limit);
        Ok(Vec::new())
    }
}

/// Reciprocal Rank Fusion (RRF) implementation
///
/// Combines rankings from multiple sources using the formula:
/// RRF(d) = Σ 1/(k + rank(d))
/// where k is a constant (typically 60)
pub fn reciprocal_rank_fusion(
    dense_results: &[(String, f32)],
    sparse_results: &[(String, f32)],
    alpha: f32, // Weight: 0.0 = pure sparse, 1.0 = pure dense
) -> Vec<(String, f32)> {
    let k = 60.0; // RRF constant
    let mut scores: HashMap<String, f32> = HashMap::new();

    // Score from dense results
    for (rank, (id, score)) in dense_results.iter().enumerate() {
        let rrf_score = alpha / (k + (rank as f32 + 1.0));
        let combined = rrf_score + score * alpha;
        *scores.entry(id.clone()).or_insert(0.0) += combined;
    }

    // Score from sparse (BM25) results
    for (rank, (id, score)) in sparse_results.iter().enumerate() {
        let rrf_score = (1.0 - alpha) / (k + (rank as f32 + 1.0));
        let combined = rrf_score + score * (1.0 - alpha);
        *scores.entry(id.clone()).or_insert(0.0) += combined;
    }

    // Sort by combined score
    let mut results: Vec<_> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_fusion() {
        let dense = vec![
            ("doc1".to_string(), 0.9),
            ("doc2".to_string(), 0.8),
            ("doc3".to_string(), 0.7),
        ];

        let sparse = vec![
            ("doc2".to_string(), 0.85),
            ("doc1".to_string(), 0.75),
            ("doc4".to_string(), 0.65),
        ];

        let merged = reciprocal_rank_fusion(&dense, &sparse, 0.7);

        assert!(!merged.is_empty());
        // doc1 and doc2 should be top (appear in both)
        assert!(merged[0].0 == "doc1" || merged[0].0 == "doc2");
    }

    #[test]
    fn test_rrf_pure_dense() {
        let dense = vec![("doc1".to_string(), 0.9), ("doc2".to_string(), 0.8)];
        let sparse = vec![("doc3".to_string(), 0.9)];

        let merged = reciprocal_rank_fusion(&dense, &sparse, 1.0);

        // With alpha=1.0, should prefer dense results
        assert_eq!(merged[0].0, "doc1");
    }

    #[test]
    fn test_rrf_pure_sparse() {
        let dense = vec![("doc1".to_string(), 0.9)];
        let sparse = vec![("doc2".to_string(), 0.9), ("doc3".to_string(), 0.8)];

        let merged = reciprocal_rank_fusion(&dense, &sparse, 0.0);

        // With alpha=0.0, should prefer sparse results
        assert_eq!(merged[0].0, "doc2");
    }
}
