//! Hybrid search implementation combining HNSW + BM25

use std::collections::HashMap;
use std::sync::Arc;

use super::broad::apply_mmr;
use super::types::{ChunkMetadata, DiscoveryError, DiscoveryResult, ScoredChunk};
use crate::VectorStore;
use crate::embedding::EmbeddingManager;

/// Hybrid searcher combining dense (HNSW) and sparse (BM25) search
pub struct HybridSearcher {
    store: Arc<VectorStore>,
    embedding_manager: Arc<EmbeddingManager>,
}

impl HybridSearcher {
    /// Create new hybrid searcher with VectorStore and EmbeddingManager
    pub fn new(store: Arc<VectorStore>, embedding_manager: Arc<EmbeddingManager>) -> Self {
        Self {
            store,
            embedding_manager,
        }
    }

    /// Perform hybrid search combining dense (vector) + sparse (BM25/keyword) search
    ///
    /// # Arguments
    /// * `query` - Text query for sparse/BM25 search
    /// * `query_vector` - Pre-computed query vector for dense search
    /// * `collection_name` - Collection to search in
    /// * `limit` - Maximum number of results to return
    /// * `alpha` - Blend factor: 0.0 = pure sparse, 1.0 = pure dense
    ///
    /// # Returns
    /// Combined and ranked search results using Reciprocal Rank Fusion
    pub async fn search(
        &self,
        query: &str,
        query_vector: Vec<f32>,
        collection_name: &str,
        limit: usize,
        alpha: f32,
    ) -> DiscoveryResult<Vec<ScoredChunk>> {
        // Fetch more results than needed for RRF merging
        let fetch_limit = limit * 3;

        // 1. Dense search with HNSW
        let dense_results = self
            .store
            .search(collection_name, &query_vector, fetch_limit)
            .map_err(|e| DiscoveryError::SearchError(format!("Dense search error: {}", e)))?;

        // Convert to (id, score) pairs
        let dense_pairs: Vec<(String, f32)> = dense_results
            .iter()
            .map(|r| (r.id.clone(), r.score))
            .collect();

        // 2. Sparse/BM25 search using payload text index
        let sparse_pairs = self.sparse_search(query, collection_name, fetch_limit)?;

        // 3. Reciprocal Rank Fusion to merge results
        let merged = reciprocal_rank_fusion(&dense_pairs, &sparse_pairs, alpha);

        // 4. Build ScoredChunk results
        let mut chunks = Vec::new();
        for (id, rrf_score) in merged.into_iter().take(limit) {
            // Find the original result to get payload
            let content = dense_results
                .iter()
                .find(|r| r.id == id)
                .and_then(|r| {
                    r.payload.as_ref().and_then(|p| {
                        p.data
                            .get("content")
                            .or_else(|| p.data.get("text"))
                            .and_then(|v| v.as_str())
                    })
                })
                .unwrap_or("")
                .to_string();

            if content.is_empty() {
                continue;
            }

            let metadata = extract_metadata(&id, collection_name);

            chunks.push(ScoredChunk {
                collection: collection_name.to_string(),
                doc_id: id,
                content,
                score: rrf_score,
                metadata,
            });
        }

        Ok(chunks)
    }

    /// Perform hybrid search with automatic query embedding
    pub async fn search_with_text(
        &self,
        query: &str,
        collection_name: &str,
        limit: usize,
        alpha: f32,
    ) -> DiscoveryResult<Vec<ScoredChunk>> {
        // Embed the query
        let query_vector = self
            .embedding_manager
            .embed(query)
            .map_err(|e| DiscoveryError::SearchError(format!("Embedding error: {}", e)))?;

        self.search(query, query_vector, collection_name, limit, alpha)
            .await
    }

    /// Perform hybrid search across multiple collections
    pub async fn search_multi_collection(
        &self,
        query: &str,
        query_vector: Vec<f32>,
        collection_names: &[String],
        limit: usize,
        alpha: f32,
    ) -> DiscoveryResult<Vec<ScoredChunk>> {
        let mut all_results = Vec::new();
        let per_collection_limit = (limit * 2) / collection_names.len().max(1);

        for collection_name in collection_names {
            match self
                .search(
                    query,
                    query_vector.clone(),
                    collection_name,
                    per_collection_limit,
                    alpha,
                )
                .await
            {
                Ok(results) => all_results.extend(results),
                Err(e) => {
                    tracing::warn!(
                        "Hybrid search error in collection {}: {}",
                        collection_name,
                        e
                    );
                }
            }
        }

        // Sort by score and apply MMR for diversity
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply MMR for diversity (lambda=0.7 balances relevance and diversity)
        let final_results = apply_mmr(all_results, limit, 0.7);

        Ok(final_results)
    }

    /// Sparse/keyword search using BM25-like scoring
    fn sparse_search(
        &self,
        query: &str,
        collection_name: &str,
        limit: usize,
    ) -> DiscoveryResult<Vec<(String, f32)>> {
        // Tokenize query into keywords
        let keywords: Vec<&str> = query
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() >= 2)
            .collect();

        if keywords.is_empty() {
            return Ok(Vec::new());
        }

        // Get collection for payload search
        let collection = self.store.get_collection(collection_name).map_err(|e| {
            DiscoveryError::CollectionNotFound(format!("{}: {}", collection_name, e))
        })?;

        // Search through vectors and score by keyword matches
        let mut results: HashMap<String, f32> = HashMap::new();
        let vectors = collection.get_all_vectors();

        for vector in vectors.iter().take(10000) {
            // Limit scan for performance
            if let Some(payload) = &vector.payload {
                // Get text content from payload
                let text = payload
                    .data
                    .get("content")
                    .or_else(|| payload.data.get("text"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if text.is_empty() {
                    continue;
                }

                // Calculate BM25-like score
                let score = self.bm25_score(text, &keywords);
                if score > 0.0 {
                    results.insert(vector.id.clone(), score);
                }
            }
        }

        // Sort by score and return top results
        let mut sorted: Vec<_> = results.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(sorted.into_iter().take(limit).collect())
    }

    /// BM25-like scoring for keyword matching
    fn bm25_score(&self, text: &str, keywords: &[&str]) -> f32 {
        let text_lower = text.to_lowercase();
        let text_len = text.split_whitespace().count() as f32;

        // BM25 parameters
        let k1 = 1.2;
        let b = 0.75;
        let avg_doc_len = 100.0; // Approximate average document length

        let mut score = 0.0;

        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();

            // Count term frequency
            let tf = text_lower.matches(&keyword_lower).count() as f32;

            if tf > 0.0 {
                // Simplified BM25 scoring
                let length_norm = 1.0 - b + b * (text_len / avg_doc_len);
                let tf_component = (tf * (k1 + 1.0)) / (tf + k1 * length_norm);

                // IDF approximation (assuming keyword is relatively rare)
                let idf = (2.0_f32).ln();

                score += idf * tf_component;
            }
        }

        score
    }
}

impl Default for HybridSearcher {
    fn default() -> Self {
        // Create with empty store and manager for testing
        // Real usage should call new() with proper dependencies
        Self {
            store: Arc::new(VectorStore::new()),
            embedding_manager: Arc::new(EmbeddingManager::new()),
        }
    }
}

/// Extract metadata from document ID
fn extract_metadata(doc_id: &str, _collection_name: &str) -> ChunkMetadata {
    let parts: Vec<&str> = doc_id.split("::").collect();

    let file_path = parts.get(1).unwrap_or(&"unknown").to_string();
    let chunk_index = parts
        .get(2)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let file_extension = std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt")
        .to_string();

    ChunkMetadata {
        file_path,
        chunk_index,
        file_extension,
        line_range: None,
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
