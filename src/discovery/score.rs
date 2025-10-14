//! Collection scoring

use super::config::ScoringConfig;
use super::types::{CollectionRef, DiscoveryResult};
use chrono::Utc;

/// Score collections by name match, term boost, and signal boost
pub fn score_collections(
    query_terms: &[&str],
    collections: &[CollectionRef],
    config: &ScoringConfig,
) -> DiscoveryResult<Vec<(CollectionRef, f32)>> {
    let scorer = CollectionScorer { config };
    
    let mut scored: Vec<(CollectionRef, f32)> = collections
        .iter()
        .map(|c| {
            let score = scorer.score(c, query_terms);
            (c.clone(), score)
        })
        .collect();
    
    // Sort by score (highest first)
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(scored)
}

struct CollectionScorer<'a> {
    config: &'a ScoringConfig,
}

impl<'a> CollectionScorer<'a> {
    fn score(&self, collection: &CollectionRef, query_terms: &[&str]) -> f32 {
        let name_score = self.name_match_score(&collection.name, query_terms);
        let term_score = self.term_boost_score(&collection.name);
        let signal_score = self.signal_boost_score(collection);
        
        name_score * self.config.name_match_weight
            + term_score * self.config.term_boost_weight
            + signal_score * self.config.signal_boost_weight
    }
    
    fn name_match_score(&self, name: &str, terms: &[&str]) -> f32 {
        if terms.is_empty() {
            return 0.0;
        }
        
        let name_lower = name.to_lowercase();
        
        // Count exact matches
        let exact_matches = terms
            .iter()
            .filter(|term| name_lower.contains(&term.to_lowercase()))
            .count();
        
        let mut score = (exact_matches as f32) / (terms.len() as f32);
        
        // Boost if name starts with query term
        if terms.iter().any(|t| name_lower.starts_with(&t.to_lowercase())) {
            score *= 1.5;
        }
        
        score.min(1.0)
    }
    
    fn term_boost_score(&self, name: &str) -> f32 {
        let boost_terms = ["docs", "source", "api", "sdk", "core"];
        let name_lower = name.to_lowercase();
        
        let matches = boost_terms
            .iter()
            .filter(|term| name_lower.contains(*term))
            .count();
        
        (matches as f32) / (boost_terms.len() as f32)
    }
    
    fn signal_boost_score(&self, collection: &CollectionRef) -> f32 {
        // Size signal (normalize by 1M vectors)
        let size_score = (collection.vector_count as f32 / 1_000_000.0).min(1.0);
        
        // Recency signal (exponential decay)
        let days_old = (Utc::now() - collection.updated_at).num_days() as f32;
        let recency_score = (-days_old / self.config.recency_decay_days).exp();
        
        // Tag signal
        let important_tags = ["documentation", "code", "api"];
        let tag_matches = collection
            .tags
            .iter()
            .filter(|t| important_tags.contains(&t.as_str()))
            .count();
        let tag_score = if !important_tags.is_empty() {
            (tag_matches as f32) / (important_tags.len() as f32)
        } else {
            0.0
        };
        
        (size_score + recency_score + tag_score) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    fn create_test_collection(name: &str, vector_count: usize) -> CollectionRef {
        CollectionRef {
            name: name.to_string(),
            dimension: 384,
            vector_count,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
        }
    }
    
    #[test]
    fn test_score_collections() {
        let collections = vec![
            create_test_collection("vectorizer-docs", 10000),
            create_test_collection("vectorizer-source", 50000),
            create_test_collection("other-collection", 1000),
        ];
        
        let config = ScoringConfig::default();
        let scored = score_collections(&["vectorizer"], &collections, &config).unwrap();
        
        assert_eq!(scored.len(), 3);
        assert!(scored[0].1 > 0.0);
        
        // vectorizer collections should score higher
        assert!(scored.iter().take(2).all(|(c, _)| c.name.contains("vectorizer")));
    }
    
    #[test]
    fn test_name_match_score() {
        let scorer = CollectionScorer {
            config: &ScoringConfig::default(),
        };
        
        let score1 = scorer.name_match_score("vectorizer-docs", &["vectorizer"]);
        let score2 = scorer.name_match_score("other-docs", &["vectorizer"]);
        
        assert!(score1 > score2);
        assert!(score1 > 0.0);
        assert_eq!(score2, 0.0);
    }
}


