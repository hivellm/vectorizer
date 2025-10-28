//! Evaluation metrics for search quality assessment
//!
//! This module provides standard information retrieval metrics to evaluate
//! the quality of search results and embedding models.

use std::collections::HashSet;

/// Represents a single query result with its relevance
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryResult {
    /// The document/chunk ID
    pub doc_id: String,
    /// The relevance score (1.0 = highly relevant, 0.0 = not relevant)
    pub relevance: f32,
}

/// Evaluation metrics for a single query
#[derive(Debug)]
pub struct QueryMetrics {
    /// Precision@K: fraction of retrieved documents that are relevant
    pub precision_at_k: Vec<f32>,
    /// Recall@K: fraction of relevant documents that are retrieved
    pub recall_at_k: Vec<f32>,
    /// Average Precision (AP)
    pub average_precision: f32,
    /// Reciprocal Rank (RR) - reciprocal of rank of first relevant document
    pub reciprocal_rank: f32,
}

/// Overall evaluation metrics across multiple queries
#[derive(Debug)]
pub struct EvaluationMetrics {
    /// Mean Average Precision (MAP)
    pub mean_average_precision: f32,
    /// Mean Reciprocal Rank (MRR)
    pub mean_reciprocal_rank: f32,
    /// Precision@K averaged across all queries
    pub precision_at_k: Vec<f32>,
    /// Recall@K averaged across all queries
    pub recall_at_k: Vec<f32>,
    /// Number of queries evaluated
    pub num_queries: usize,
}

/// Evaluate search results for a single query
pub fn evaluate_query_results(
    results: &[QueryResult],
    ground_truth: &HashSet<String>,
    k: usize,
) -> QueryMetrics {
    let mut precision_at_k = Vec::new();
    let mut recall_at_k = Vec::new();
    let mut reciprocal_rank = 0.0;
    let mut average_precision = 0.0;

    let total_relevant = ground_truth.len() as f32;
    let mut num_relevant_found = 0;

    for (i, result) in results.iter().take(k).enumerate() {
        let rank = i + 1; // 1-indexed rank
        let is_relevant = ground_truth.contains(&result.doc_id);

        // Precision@K and Recall@K
        if is_relevant {
            num_relevant_found += 1;
            let precision = num_relevant_found as f32 / rank as f32;
            let recall = num_relevant_found as f32 / total_relevant;

            precision_at_k.push(precision);
            recall_at_k.push(recall);

            // Average Precision contribution
            average_precision += precision;

            // Reciprocal Rank (only for first relevant document)
            if reciprocal_rank == 0.0 {
                reciprocal_rank = 1.0 / rank as f32;
            }
        } else {
            // For non-relevant documents, precision stays the same, recall doesn't change
            if !precision_at_k.is_empty() {
                precision_at_k.push(*precision_at_k.last().unwrap());
            } else {
                precision_at_k.push(0.0);
            }

            if !recall_at_k.is_empty() {
                recall_at_k.push(*recall_at_k.last().unwrap());
            } else {
                recall_at_k.push(0.0);
            }
        }
    }

    // Normalize average precision by number of relevant documents
    if total_relevant > 0.0 {
        average_precision /= total_relevant;
    }

    QueryMetrics {
        precision_at_k,
        recall_at_k,
        average_precision,
        reciprocal_rank,
    }
}

/// Evaluate search results across multiple queries
pub fn evaluate_search_quality(
    query_results: Vec<(Vec<QueryResult>, HashSet<String>)>,
    k: usize,
) -> EvaluationMetrics {
    let mut total_ap = 0.0;
    let mut total_rr = 0.0;
    let mut precision_sums: Vec<f32> = vec![0.0; k];
    let mut recall_sums: Vec<f32> = vec![0.0; k];

    for (results, ground_truth) in &query_results {
        let metrics = evaluate_query_results(results, ground_truth, k);

        total_ap += metrics.average_precision;
        total_rr += metrics.reciprocal_rank;

        // Sum precision and recall at each k
        for i in 0..k.min(metrics.precision_at_k.len()) {
            precision_sums[i] += metrics.precision_at_k[i];
        }
        for i in 0..k.min(metrics.recall_at_k.len()) {
            recall_sums[i] += metrics.recall_at_k[i];
        }
    }

    let num_queries = query_results.len() as f32;

    EvaluationMetrics {
        mean_average_precision: total_ap / num_queries,
        mean_reciprocal_rank: total_rr / num_queries,
        precision_at_k: precision_sums
            .into_iter()
            .map(|sum| sum / num_queries)
            .collect(),
        recall_at_k: recall_sums
            .into_iter()
            .map(|sum| sum / num_queries)
            .collect(),
        num_queries: query_results.len(),
    }
}

/// Calculate Mean Reciprocal Rank (MRR)
pub fn mean_reciprocal_rank(reciprocal_ranks: &[f32]) -> f32 {
    reciprocal_ranks.iter().sum::<f32>() / reciprocal_ranks.len() as f32
}

/// Calculate Mean Average Precision (MAP)
pub fn mean_average_precision(average_precisions: &[f32]) -> f32 {
    average_precisions.iter().sum::<f32>() / average_precisions.len() as f32
}

/// Calculate Precision@K
pub fn precision_at_k(results: &[QueryResult], ground_truth: &HashSet<String>, k: usize) -> f32 {
    let relevant_in_top_k = results
        .iter()
        .take(k)
        .filter(|result| ground_truth.contains(&result.doc_id))
        .count();

    relevant_in_top_k as f32 / k as f32
}

/// Calculate Recall@K
pub fn recall_at_k(results: &[QueryResult], ground_truth: &HashSet<String>, k: usize) -> f32 {
    let total_relevant = ground_truth.len();
    if total_relevant == 0 {
        return 0.0;
    }

    let relevant_in_top_k = results
        .iter()
        .take(k)
        .filter(|result| ground_truth.contains(&result.doc_id))
        .count();

    relevant_in_top_k as f32 / total_relevant as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_at_k() {
        let results = vec![
            QueryResult {
                doc_id: "doc1".to_string(),
                relevance: 1.0,
            },
            QueryResult {
                doc_id: "doc2".to_string(),
                relevance: 0.0,
            },
            QueryResult {
                doc_id: "doc3".to_string(),
                relevance: 1.0,
            },
        ];

        let ground_truth = HashSet::from(["doc1".to_string(), "doc3".to_string()]);

        assert_eq!(precision_at_k(&results, &ground_truth, 1), 1.0); // doc1 is relevant
        assert_eq!(precision_at_k(&results, &ground_truth, 2), 0.5); // 1 relevant out of 2
        assert_eq!(precision_at_k(&results, &ground_truth, 3), 2.0 / 3.0); // 2 relevant out of 3
    }

    #[test]
    fn test_recall_at_k() {
        let results = vec![
            QueryResult {
                doc_id: "doc1".to_string(),
                relevance: 1.0,
            },
            QueryResult {
                doc_id: "doc2".to_string(),
                relevance: 0.0,
            },
            QueryResult {
                doc_id: "doc3".to_string(),
                relevance: 1.0,
            },
        ];

        let ground_truth = HashSet::from(["doc1".to_string(), "doc3".to_string()]);

        assert_eq!(recall_at_k(&results, &ground_truth, 1), 0.5); // 1 out of 2 relevant docs
        assert_eq!(recall_at_k(&results, &ground_truth, 2), 0.5); // still 1 out of 2 (doc2 is not relevant)
        assert_eq!(recall_at_k(&results, &ground_truth, 3), 1.0); // all relevant docs found
    }

    #[test]
    fn test_mean_reciprocal_rank() {
        let ranks = vec![1.0, 0.5, 0.33]; // reciprocal ranks for positions 1, 2, 3
        assert!((mean_reciprocal_rank(&ranks) - 0.61).abs() < 0.01);
    }

    #[test]
    fn test_query_result_creation() {
        let result = QueryResult {
            doc_id: "test_doc".to_string(),
            relevance: 0.95,
        };

        assert_eq!(result.doc_id, "test_doc");
        assert_eq!(result.relevance, 0.95);
    }

    #[test]
    fn test_evaluate_query_results_empty() {
        let results = vec![];
        let ground_truth = HashSet::new();

        let metrics = evaluate_query_results(&results, &ground_truth, 10);
        assert_eq!(metrics.average_precision, 0.0);
        assert_eq!(metrics.reciprocal_rank, 0.0);
    }

    #[test]
    fn test_evaluate_query_results_all_relevant() {
        let results = vec![
            QueryResult {
                doc_id: "doc1".to_string(),
                relevance: 1.0,
            },
            QueryResult {
                doc_id: "doc2".to_string(),
                relevance: 1.0,
            },
            QueryResult {
                doc_id: "doc3".to_string(),
                relevance: 1.0,
            },
        ];

        let ground_truth =
            HashSet::from(["doc1".to_string(), "doc2".to_string(), "doc3".to_string()]);

        let metrics = evaluate_query_results(&results, &ground_truth, 3);
        assert_eq!(metrics.reciprocal_rank, 1.0); // First doc is relevant
        assert!(!metrics.precision_at_k.is_empty());
    }

    #[test]
    fn test_evaluate_search_quality_empty() {
        let query_results: Vec<(Vec<QueryResult>, HashSet<String>)> = vec![];

        let metrics = evaluate_search_quality(query_results, 10);

        // Empty results may produce NaN
        assert!(metrics.mean_average_precision.is_nan() || metrics.mean_average_precision == 0.0);
        assert!(metrics.mean_reciprocal_rank.is_nan() || metrics.mean_reciprocal_rank == 0.0);
        assert_eq!(metrics.num_queries, 0);
    }

    #[test]
    fn test_precision_at_k_no_relevant() {
        let results = vec![
            QueryResult {
                doc_id: "doc1".to_string(),
                relevance: 0.0,
            },
            QueryResult {
                doc_id: "doc2".to_string(),
                relevance: 0.0,
            },
        ];

        let ground_truth = HashSet::from(["doc99".to_string()]);

        let precision = precision_at_k(&results, &ground_truth, 2);
        assert_eq!(precision, 0.0);
    }

    #[test]
    fn test_recall_at_k_all_found() {
        let results = vec![
            QueryResult {
                doc_id: "doc1".to_string(),
                relevance: 1.0,
            },
            QueryResult {
                doc_id: "doc2".to_string(),
                relevance: 1.0,
            },
        ];

        let ground_truth = HashSet::from(["doc1".to_string(), "doc2".to_string()]);

        let recall = recall_at_k(&results, &ground_truth, 2);
        assert_eq!(recall, 1.0);
    }

    #[test]
    fn test_evaluation_metrics_structure() {
        let metrics = EvaluationMetrics {
            mean_average_precision: 0.75,
            mean_reciprocal_rank: 0.85,
            precision_at_k: vec![1.0, 0.8, 0.6],
            recall_at_k: vec![0.5, 0.7, 0.9],
            num_queries: 10,
        };

        assert_eq!(metrics.mean_average_precision, 0.75);
        assert_eq!(metrics.mean_reciprocal_rank, 0.85);
        assert_eq!(metrics.num_queries, 10);
    }

    #[test]
    fn test_query_metrics_structure() {
        let metrics = QueryMetrics {
            precision_at_k: vec![1.0, 0.8],
            recall_at_k: vec![0.5, 0.7],
            average_precision: 0.9,
            reciprocal_rank: 1.0,
        };

        assert_eq!(metrics.average_precision, 0.9);
        assert_eq!(metrics.reciprocal_rank, 1.0);
        assert_eq!(metrics.precision_at_k.len(), 2);
    }

    #[test]
    fn test_precision_at_k_k_zero() {
        let results = vec![QueryResult {
            doc_id: "doc1".to_string(),
            relevance: 1.0,
        }];

        let ground_truth = HashSet::from(["doc1".to_string()]);

        // K=0 should return NaN (division by zero)
        let precision = precision_at_k(&results, &ground_truth, 0);
        assert!(precision.is_nan());
    }

    #[test]
    fn test_recall_at_k_k_zero() {
        let results = vec![QueryResult {
            doc_id: "doc1".to_string(),
            relevance: 1.0,
        }];

        let ground_truth = HashSet::from(["doc1".to_string()]);

        // K=0 should return 0.0
        let recall = recall_at_k(&results, &ground_truth, 0);
        assert_eq!(recall, 0.0);
    }

    #[test]
    fn test_mean_reciprocal_rank_empty() {
        let ranks: Vec<f32> = vec![];
        let result = mean_reciprocal_rank(&ranks);
        // Empty ranks should return NaN or 0
        assert!(result.is_nan() || result == 0.0);
    }

    #[test]
    fn test_mean_reciprocal_rank_single() {
        let ranks = vec![1.0];
        assert_eq!(mean_reciprocal_rank(&ranks), 1.0);
    }
}
