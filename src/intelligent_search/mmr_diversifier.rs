//! MMR (Maximal Marginal Relevance) Diversifier Module
//!
//! This module implements MMR diversification to balance relevance and diversity
//! in search results, preventing redundant content.

use std::collections::HashSet;

use crate::intelligent_search::IntelligentSearchResult;

/// MMR diversifier for balancing relevance and diversity
pub struct MMRDiversifier {
    lambda: f32, // 0.0 = pure diversity, 1.0 = pure relevance
}

impl MMRDiversifier {
    /// Create a new MMR diversifier
    pub fn new(lambda: f32) -> Self {
        Self { lambda }
    }

    /// Diversify results using MMR algorithm
    pub fn diversify(
        &self,
        results: &[IntelligentSearchResult],
        max_results: usize,
    ) -> Vec<IntelligentSearchResult> {
        if results.is_empty() || max_results == 0 {
            return Vec::new();
        }

        let mut diversified = Vec::new();
        let mut remaining: Vec<IntelligentSearchResult> = results.to_vec();
        let mut selected_indices = HashSet::new();

        // Select first result (highest relevance)
        if let Some(first_result) = remaining.pop() {
            diversified.push(first_result);
            selected_indices.insert(0);
        }

        // MMR selection for remaining results
        while diversified.len() < max_results && !remaining.is_empty() {
            let best_idx = self.select_best_mmr_result(&remaining, &diversified);

            let result = remaining.remove(best_idx);
            diversified.push(result);
        }

        diversified
    }

    /// Select the best result using MMR scoring
    fn select_best_mmr_result(
        &self,
        candidates: &[IntelligentSearchResult],
        selected: &[IntelligentSearchResult],
    ) -> usize {
        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;

        for (i, candidate) in candidates.iter().enumerate() {
            let relevance_score = candidate.score;
            let max_similarity = self.calculate_max_similarity(candidate, selected);

            // MMR score: λ * relevance - (1-λ) * max_similarity
            let mmr_score = self.lambda * relevance_score - (1.0 - self.lambda) * max_similarity;

            if mmr_score > best_score {
                best_score = mmr_score;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Calculate maximum similarity between candidate and selected results
    fn calculate_max_similarity(
        &self,
        candidate: &IntelligentSearchResult,
        selected: &[IntelligentSearchResult],
    ) -> f32 {
        if selected.is_empty() {
            return 0.0;
        }

        let mut max_sim: f32 = 0.0;

        for selected_result in selected {
            let similarity =
                self.calculate_content_similarity(&candidate.content, &selected_result.content);
            max_sim = max_sim.max(similarity);
        }

        max_sim
    }

    /// Calculate content similarity using Jaccard similarity
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
        let words1: HashSet<&str> = content1_lower.split_whitespace().collect();
        let content2_lower = content2.to_lowercase();
        let words2: HashSet<&str> = content2_lower.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Get current lambda value
    pub fn get_lambda(&self) -> f32 {
        self.lambda
    }

    /// Set lambda value
    pub fn set_lambda(&mut self, lambda: f32) {
        self.lambda = lambda.clamp(0.0, 1.0);
    }
}

impl Default for MMRDiversifier {
    fn default() -> Self {
        Self::new(0.7)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn create_test_result(id: &str, content: &str, score: f32) -> IntelligentSearchResult {
        IntelligentSearchResult {
            content: content.to_string(),
            score,
            collection: "test".to_string(),
            doc_id: id.to_string(),
            metadata: HashMap::new(),
            score_breakdown: None,
        }
    }

    #[test]
    fn test_mmr_diversifier_creation() {
        let diversifier = MMRDiversifier::new(0.7);
        assert_eq!(diversifier.get_lambda(), 0.7);
    }

    #[test]
    fn test_default_mmr_diversifier() {
        let diversifier = MMRDiversifier::default();
        assert_eq!(diversifier.get_lambda(), 0.7);
    }

    #[test]
    fn test_set_lambda() {
        let mut diversifier = MMRDiversifier::new(0.5);
        diversifier.set_lambda(0.8);
        assert_eq!(diversifier.get_lambda(), 0.8);

        // Test clamping
        diversifier.set_lambda(1.5);
        assert_eq!(diversifier.get_lambda(), 1.0);

        diversifier.set_lambda(-0.5);
        assert_eq!(diversifier.get_lambda(), 0.0);
    }

    #[test]
    fn test_diversify_empty_results() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![];
        let diversified = diversifier.diversify(&results, 5);
        assert!(diversified.is_empty());
    }

    #[test]
    fn test_diversify_zero_max_results() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![create_test_result("doc1", "content about vectorizer", 0.9)];
        let diversified = diversifier.diversify(&results, 0);
        assert!(diversified.is_empty());
    }

    #[test]
    fn test_diversify_single_result() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![create_test_result("doc1", "content about vectorizer", 0.9)];
        let diversified = diversifier.diversify(&results, 5);
        assert_eq!(diversified.len(), 1);
        assert_eq!(diversified[0].doc_id, "doc1");
    }

    #[test]
    fn test_diversify_multiple_results() {
        let diversifier = MMRDiversifier::new(0.7);
        let results = vec![
            create_test_result("doc1", "content about vectorizer", 0.9),
            create_test_result("doc2", "different content about search", 0.8),
            create_test_result("doc3", "more content about vectorizer", 0.7),
        ];
        let diversified = diversifier.diversify(&results, 2);
        assert_eq!(diversified.len(), 2);

        // Results should be diversified (not necessarily in score order due to MMR)
        assert_eq!(diversified.len(), 2);
        assert!(diversified.iter().all(|r| r.score > 0.0));
    }

    #[test]
    fn test_calculate_content_similarity() {
        let diversifier = MMRDiversifier::new(0.7);

        // Identical content
        let sim1 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "vectorizer is a vector database",
        );
        assert_eq!(sim1, 1.0);

        // Similar content
        let sim2 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "vectorizer database for vectors",
        );
        assert!(sim2 > 0.0 && sim2 < 1.0);

        // Different content
        let sim3 = diversifier.calculate_content_similarity(
            "vectorizer is a vector database",
            "completely different content",
        );
        assert_eq!(sim3, 0.0);
    }

    #[test]
    fn test_calculate_max_similarity() {
        let diversifier = MMRDiversifier::new(0.7);
        let candidate = create_test_result("candidate", "vectorizer database", 0.8);
        let selected = vec![
            create_test_result("doc1", "vectorizer is great", 0.9),
            create_test_result("doc2", "database performance", 0.7),
        ];

        let max_sim = diversifier.calculate_max_similarity(&candidate, &selected);
        assert!(max_sim > 0.0);
    }

    #[test]
    fn test_lambda_effect_on_diversification() {
        // High lambda (more relevance-focused)
        let relevance_focused = MMRDiversifier::new(0.9);

        // Low lambda (more diversity-focused)
        let diversity_focused = MMRDiversifier::new(0.1);

        let results = vec![
            create_test_result("doc1", "vectorizer is a vector database", 0.9),
            create_test_result("doc2", "vectorizer performance benchmarks", 0.8),
            create_test_result("doc3", "completely different topic", 0.7),
        ];

        let relevance_results = relevance_focused.diversify(&results, 2);
        let diversity_results = diversity_focused.diversify(&results, 2);

        // Both should return 2 results
        assert_eq!(relevance_results.len(), 2);
        assert_eq!(diversity_results.len(), 2);

        // Results might be different due to different lambda values
        // (exact order depends on similarity calculations)
    }
}
