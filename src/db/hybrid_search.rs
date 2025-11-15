//! Hybrid search combining dense (HNSW) and sparse vector search
//!
//! This module implements hybrid search algorithms that combine:
//! - Dense vector search using HNSW index (semantic similarity)
//! - Sparse vector search using SparseVectorIndex (keyword/exact match)
//!
//! Algorithms supported:
//! - Reciprocal Rank Fusion (RRF)
//! - Weighted score combination
//! - Alpha blending

use std::collections::HashMap;

use tracing::{debug, info};

use crate::error::{Result, VectorizerError};
use crate::models::{SearchResult, SparseVector};

/// Hybrid search configuration
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    /// Weight for dense search (0.0 = pure sparse, 1.0 = pure dense)
    /// Default: 0.7 (favor dense/semantic search)
    pub alpha: f32,
    /// Number of results to retrieve from dense search
    pub dense_k: usize,
    /// Number of results to retrieve from sparse search
    pub sparse_k: usize,
    /// Final number of results to return
    pub final_k: usize,
    /// Scoring algorithm to use
    pub algorithm: HybridScoringAlgorithm,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            alpha: 0.7,
            dense_k: 20,
            sparse_k: 20,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
        }
    }
}

/// Hybrid scoring algorithms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HybridScoringAlgorithm {
    /// Reciprocal Rank Fusion (RRF)
    /// Combines rankings using: RRF(d) = Σ 1/(k + rank(d))
    ReciprocalRankFusion,
    /// Weighted score combination
    /// Combines scores using: score = alpha * dense_score + (1-alpha) * sparse_score
    WeightedCombination,
    /// Alpha blending
    /// Similar to weighted but with normalization
    AlphaBlending,
}

/// Result from dense search
#[derive(Debug, Clone)]
pub struct DenseSearchResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
}

/// Result from sparse search
#[derive(Debug, Clone)]
pub struct SparseSearchResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
}

/// Hybrid search result combining both dense and sparse scores
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// Vector ID
    pub id: String,
    /// Combined hybrid score
    pub hybrid_score: f32,
    /// Dense search score
    pub dense_score: Option<f32>,
    /// Sparse search score
    pub sparse_score: Option<f32>,
}

impl HybridSearchResult {
    /// Create a new hybrid search result
    pub fn new(
        id: String,
        hybrid_score: f32,
        dense_score: Option<f32>,
        sparse_score: Option<f32>,
    ) -> Self {
        Self {
            id,
            hybrid_score,
            dense_score,
            sparse_score,
        }
    }
}

/// Perform hybrid search combining dense and sparse results
pub fn hybrid_search(
    dense_results: Vec<DenseSearchResult>,
    sparse_results: Vec<SparseSearchResult>,
    config: &HybridSearchConfig,
) -> Result<Vec<HybridSearchResult>> {
    if dense_results.is_empty() && sparse_results.is_empty() {
        return Ok(Vec::new());
    }

    info!(
        "Hybrid search: {} dense results, {} sparse results, alpha={}, algorithm={:?}, final_k={}",
        dense_results.len(),
        sparse_results.len(),
        config.alpha,
        config.algorithm,
        config.final_k
    );

    debug!(
        "Hybrid search configuration: dense_k={}, sparse_k={}, final_k={}, alpha={}, algorithm={:?}",
        config.dense_k, config.sparse_k, config.final_k, config.alpha, config.algorithm
    );

    let results = match config.algorithm {
        HybridScoringAlgorithm::ReciprocalRankFusion => {
            reciprocal_rank_fusion(dense_results, sparse_results, config.alpha)
        }
        HybridScoringAlgorithm::WeightedCombination => {
            weighted_combination(dense_results, sparse_results, config.alpha)
        }
        HybridScoringAlgorithm::AlphaBlending => {
            alpha_blending(dense_results, sparse_results, config.alpha)
        }
    };

    // Sort by hybrid score descending and take top k
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| {
        b.hybrid_score
            .partial_cmp(&a.hybrid_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted_results.truncate(config.final_k);

    Ok(sorted_results)
}

/// Reciprocal Rank Fusion (RRF) algorithm
///
/// Combines rankings from multiple sources using:
/// RRF(d) = Σ 1/(k + rank(d))
/// where k is a constant (typically 60)
fn reciprocal_rank_fusion(
    dense_results: Vec<DenseSearchResult>,
    sparse_results: Vec<SparseSearchResult>,
    alpha: f32,
) -> Vec<HybridSearchResult> {
    const RRF_K: f32 = 60.0;
    let mut scores: HashMap<String, (Option<f32>, Option<f32>)> = HashMap::new();

    // Score from dense results
    for (rank, result) in dense_results.iter().enumerate() {
        let rrf_score = alpha / (RRF_K + (rank as f32 + 1.0));
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.0 = Some(result.score * alpha + rrf_score);
    }

    // Score from sparse results
    for (rank, result) in sparse_results.iter().enumerate() {
        let rrf_score = (1.0 - alpha) / (RRF_K + (rank as f32 + 1.0));
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.1 = Some(result.score * (1.0 - alpha) + rrf_score);
    }

    // Combine scores
    scores
        .into_iter()
        .map(|(id, (dense_score, sparse_score))| {
            let hybrid_score = dense_score.unwrap_or(0.0) + sparse_score.unwrap_or(0.0);
            HybridSearchResult::new(id, hybrid_score, dense_score, sparse_score)
        })
        .collect()
}

/// Weighted combination algorithm
///
/// Combines scores using: score = alpha * dense_score + (1-alpha) * sparse_score
fn weighted_combination(
    dense_results: Vec<DenseSearchResult>,
    sparse_results: Vec<SparseSearchResult>,
    alpha: f32,
) -> Vec<HybridSearchResult> {
    let mut scores: HashMap<String, (Option<f32>, Option<f32>)> = HashMap::new();

    // Collect dense scores
    for result in dense_results {
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.0 = Some(result.score);
    }

    // Collect sparse scores
    for result in sparse_results {
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.1 = Some(result.score);
    }

    // Combine scores
    scores
        .into_iter()
        .map(|(id, (dense_score, sparse_score))| {
            let hybrid_score = match (dense_score, sparse_score) {
                (Some(d), Some(s)) => alpha * d + (1.0 - alpha) * s,
                (Some(d), None) => alpha * d,
                (None, Some(s)) => (1.0 - alpha) * s,
                (None, None) => 0.0,
            };
            HybridSearchResult::new(id, hybrid_score, dense_score, sparse_score)
        })
        .collect()
}

/// Alpha blending algorithm
///
/// Similar to weighted combination but with score normalization
fn alpha_blending(
    dense_results: Vec<DenseSearchResult>,
    sparse_results: Vec<SparseSearchResult>,
    alpha: f32,
) -> Vec<HybridSearchResult> {
    // Normalize scores to [0, 1] range
    let dense_max = dense_results.iter().map(|r| r.score).fold(0.0, f32::max);
    let sparse_max = sparse_results.iter().map(|r| r.score).fold(0.0, f32::max);

    let normalize_dense = |score: f32| {
        if dense_max > 0.0 {
            score / dense_max
        } else {
            score
        }
    };

    let normalize_sparse = |score: f32| {
        if sparse_max > 0.0 {
            score / sparse_max
        } else {
            score
        }
    };

    let mut scores: HashMap<String, (Option<f32>, Option<f32>)> = HashMap::new();

    // Collect normalized dense scores
    for result in dense_results {
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.0 = Some(normalize_dense(result.score));
    }

    // Collect normalized sparse scores
    for result in sparse_results {
        let entry = scores.entry(result.id.clone()).or_insert((None, None));
        entry.1 = Some(normalize_sparse(result.score));
    }

    // Combine normalized scores
    scores
        .into_iter()
        .map(|(id, (dense_score, sparse_score))| {
            let hybrid_score = match (dense_score, sparse_score) {
                (Some(d), Some(s)) => alpha * d + (1.0 - alpha) * s,
                (Some(d), None) => alpha * d,
                (None, Some(s)) => (1.0 - alpha) * s,
                (None, None) => 0.0,
            };
            HybridSearchResult::new(id, hybrid_score, dense_score, sparse_score)
        })
        .collect()
}

/// Convert SearchResult to DenseSearchResult
impl From<SearchResult> for DenseSearchResult {
    fn from(result: SearchResult) -> Self {
        Self {
            id: result.id,
            score: result.score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reciprocal_rank_fusion() {
        let dense = vec![
            DenseSearchResult {
                id: "doc1".to_string(),
                score: 0.9,
            },
            DenseSearchResult {
                id: "doc2".to_string(),
                score: 0.8,
            },
            DenseSearchResult {
                id: "doc3".to_string(),
                score: 0.7,
            },
        ];

        let sparse = vec![
            SparseSearchResult {
                id: "doc2".to_string(),
                score: 0.85,
            },
            SparseSearchResult {
                id: "doc1".to_string(),
                score: 0.75,
            },
            SparseSearchResult {
                id: "doc4".to_string(),
                score: 0.65,
            },
        ];

        let config = HybridSearchConfig {
            alpha: 0.7,
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::ReciprocalRankFusion,
        };

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert!(!results.is_empty());
        // doc1 and doc2 should be top (appear in both)
        assert!(results[0].id == "doc1" || results[0].id == "doc2");
    }

    #[test]
    fn test_weighted_combination() {
        let dense = vec![
            DenseSearchResult {
                id: "doc1".to_string(),
                score: 0.9,
            },
            DenseSearchResult {
                id: "doc2".to_string(),
                score: 0.8,
            },
        ];

        let sparse = vec![
            SparseSearchResult {
                id: "doc2".to_string(),
                score: 0.85,
            },
            SparseSearchResult {
                id: "doc3".to_string(),
                score: 0.75,
            },
        ];

        let config = HybridSearchConfig {
            alpha: 0.7,
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::WeightedCombination,
        };

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert_eq!(results.len(), 3); // doc1, doc2, doc3
        // doc2 should have highest score (appears in both with good scores)
        assert_eq!(results[0].id, "doc2");
    }

    #[test]
    fn test_alpha_blending() {
        let dense = vec![DenseSearchResult {
            id: "doc1".to_string(),
            score: 0.9,
        }];

        let sparse = vec![SparseSearchResult {
            id: "doc2".to_string(),
            score: 0.9,
        }];

        let config = HybridSearchConfig {
            alpha: 0.5, // Equal weight
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::AlphaBlending,
        };

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert_eq!(results.len(), 2);
        // Both should have similar scores after normalization
        assert!(results[0].hybrid_score > 0.0);
    }

    #[test]
    fn test_pure_dense() {
        let dense = vec![
            DenseSearchResult {
                id: "doc1".to_string(),
                score: 0.9,
            },
            DenseSearchResult {
                id: "doc2".to_string(),
                score: 0.8,
            },
        ];

        let sparse = vec![];

        let config = HybridSearchConfig {
            alpha: 1.0, // Pure dense
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::WeightedCombination,
        };

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
        assert_eq!(results[1].id, "doc2");
    }

    #[test]
    fn test_pure_sparse() {
        let dense = vec![];

        let sparse = vec![
            SparseSearchResult {
                id: "doc1".to_string(),
                score: 0.9,
            },
            SparseSearchResult {
                id: "doc2".to_string(),
                score: 0.8,
            },
        ];

        let config = HybridSearchConfig {
            alpha: 0.0, // Pure sparse
            dense_k: 10,
            sparse_k: 10,
            final_k: 10,
            algorithm: HybridScoringAlgorithm::WeightedCombination,
        };

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
        assert_eq!(results[1].id, "doc2");
    }

    #[test]
    fn test_empty_results() {
        let dense = vec![];
        let sparse = vec![];

        let config = HybridSearchConfig::default();

        let results = hybrid_search(dense, sparse, &config).unwrap();

        assert!(results.is_empty());
    }
}
