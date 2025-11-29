//! Tests for search evaluation metrics
//!
//! This file tests IR evaluation functions like MRR, MAP, P@K, R@K.

use vectorizer::evaluation::{evaluate_search_quality, EvaluationMetrics, QueryResult};
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_search_evaluation() {
        // Test evaluation with perfect search (all relevant at top)
        let query_results = vec![
            // Query 1: 2 relevant, both in first 2
            (
                vec![
                    QueryResult { doc_id: "doc_0".to_string(), relevance: 0.9 }, // relevant
                    QueryResult { doc_id: "doc_1".to_string(), relevance: 0.8 }, // relevant
                    QueryResult { doc_id: "doc_2".to_string(), relevance: 0.3 }, // not relevant
                ],
                HashSet::from(["doc_0".to_string(), "doc_1".to_string()]),
            ),
        ];

        let metrics = evaluate_search_quality(query_results, 3);

        assert_eq!(metrics.num_queries, 1);
        assert!(metrics.mean_average_precision > 0.9, "MAP should be high for perfect ranking");
        assert_eq!(metrics.mean_reciprocal_rank, 1.0, "MRR should be 1.0 for first relevant at position 1");
        assert_eq!(metrics.precision_at_k[0], 1.0, "P@1 should be perfect");
        assert_eq!(metrics.recall_at_k[1], 1.0, "R@2 should be perfect (2/2 relevant docs found)");
    }

    #[test]
    fn test_poor_search_evaluation() {
        // Test evaluation with poor search (relevant at end)
        let query_results = vec![
            // Query 1: 2 relevant, but appear at positions 3 and 4
            (
                vec![
                    QueryResult { doc_id: "doc_2".to_string(), relevance: 0.9 }, // not relevant
                    QueryResult { doc_id: "doc_3".to_string(), relevance: 0.8 }, // not relevant
                    QueryResult { doc_id: "doc_0".to_string(), relevance: 0.3 }, // relevant
                    QueryResult { doc_id: "doc_1".to_string(), relevance: 0.2 }, // relevant
                ],
                HashSet::from(["doc_0".to_string(), "doc_1".to_string()]),
            ),
        ];

        let metrics = evaluate_search_quality(query_results, 4);

        assert_eq!(metrics.num_queries, 1);
        assert!(metrics.mean_average_precision < 0.6, "MAP should be lower for poor ranking");
        assert_eq!(metrics.mean_reciprocal_rank, 0.5, "MRR should be 0.5 (1/2) for first relevant at position 3");
        assert_eq!(metrics.precision_at_k[0], 0.0, "P@1 should be 0 (no relevant in top 1)");
        assert_eq!(metrics.recall_at_k[2], 0.5, "R@3 should be 0.5 (1/2 relevant docs found)");
    }

    #[test]
    fn test_multiple_queries_evaluation() {
        // Test evaluation with multiple queries
        let query_results = vec![
            // Query 1: perfect
            (
                vec![
                    QueryResult { doc_id: "doc_0".to_string(), relevance: 0.9 },
                    QueryResult { doc_id: "doc_1".to_string(), relevance: 0.8 },
                ],
                HashSet::from(["doc_0".to_string(), "doc_1".to_string()]),
            ),
            // Query 2: poor
            (
                vec![
                    QueryResult { doc_id: "doc_2".to_string(), relevance: 0.9 },
                    QueryResult { doc_id: "doc_0".to_string(), relevance: 0.3 },
                ],
                HashSet::from(["doc_0".to_string(), "doc_1".to_string()]),
            ),
        ];

        let metrics = evaluate_search_quality(query_results, 2);

        assert_eq!(metrics.num_queries, 2);
        // Average metrics should be between individual values
        assert!(metrics.mean_average_precision > 0.4 && metrics.mean_average_precision < 0.95);
        assert!(metrics.mean_reciprocal_rank > 0.25 && metrics.mean_reciprocal_rank < 1.0);
    }

    #[test]
    fn test_empty_results() {
        // Test behavior with empty results
        let query_results = vec![
            (
                vec![], // No results
                HashSet::from(["doc_0".to_string()]),
            ),
        ];

        let metrics = evaluate_search_quality(query_results, 5);

        assert_eq!(metrics.num_queries, 1);
        assert_eq!(metrics.mean_average_precision, 0.0, "MAP should be 0 with no results");
        assert_eq!(metrics.mean_reciprocal_rank, 0.0, "MRR should be 0 with no results");
        assert!(metrics.precision_at_k.iter().all(|&p| p == 0.0), "All P@K should be 0");
        assert!(metrics.recall_at_k.iter().all(|&r| r == 0.0), "All R@K should be 0");
    }

    #[test]
    fn test_no_relevant_docs() {
        // Test when there are no relevant documents for a query
        let query_results = vec![
            (
                vec![
                    QueryResult { doc_id: "doc_0".to_string(), relevance: 0.9 },
                    QueryResult { doc_id: "doc_1".to_string(), relevance: 0.8 },
                ],
                HashSet::new(), // No relevant documents
            ),
        ];

        let metrics = evaluate_search_quality(query_results, 2);

        assert_eq!(metrics.num_queries, 1);
        assert_eq!(metrics.mean_average_precision, 0.0, "MAP should be 0 with no relevant docs");
        assert_eq!(metrics.mean_reciprocal_rank, 0.0, "MRR should be 0 with no relevant docs");
        // P@K can be calculated, but R@K should be 0
        assert!(metrics.recall_at_k.iter().all(|&r| r == 0.0), "All R@K should be 0");
    }
}
