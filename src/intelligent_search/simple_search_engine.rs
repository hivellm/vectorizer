//! Simple Search Engine Module
//! 
//! This module implements a simple search engine using basic text matching
//! and similarity scoring without external dependencies.

use crate::intelligent_search::{IntelligentSearchResult, Document, ScoreBreakdown};
use std::collections::HashMap;

/// Simple search engine using basic text matching
pub struct SimpleSearchEngine {
    documents: HashMap<String, Document>,
}

impl SimpleSearchEngine {
    /// Create a new simple search engine
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Add documents to the search index
    pub async fn add_documents(&mut self, documents: Vec<Document>) -> Result<(), Box<dyn std::error::Error>> {
        for doc in documents {
            self.documents.insert(doc.id.clone(), doc);
        }
        Ok(())
    }

    /// Perform search using simple text matching
    pub async fn search(
        &self,
        query: &str,
        collections: Vec<String>,
        max_results: usize,
    ) -> Result<Vec<IntelligentSearchResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        for doc in self.documents.values() {
            // Check collection filter
            if !collections.is_empty() && !collections.contains(&doc.collection) {
                continue;
            }
            
            // Calculate similarity score
            let score = self.calculate_score(&doc.content, query);
            
            if score > 0.0 {
                let score_breakdown = ScoreBreakdown {
                    text_similarity: score,
                    term_frequency: self.calculate_term_frequency(&doc.content, query),
                    collection_relevance: self.calculate_collection_relevance(&doc.collection, query),
                    final_score: score,
                };
                
                results.push(IntelligentSearchResult {
                    content: doc.content.clone(),
                    score,
                    collection: doc.collection.clone(),
                    doc_id: doc.id.clone(),
                    metadata: doc.metadata.clone(),
                    score_breakdown: Some(score_breakdown),
                });
            }
        }
        
        // Sort by score and limit results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(max_results);
        
        Ok(results)
    }

    /// Calculate similarity score between content and query
    fn calculate_score(&self, content: &str, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        // Simple word overlap scoring
        let query_words: std::collections::HashSet<&str> = query_lower.split_whitespace().collect();
        let content_words: std::collections::HashSet<&str> = content_lower.split_whitespace().collect();
        
        let intersection = query_words.intersection(&content_words).count();
        let union = query_words.union(&content_words).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Calculate term frequency score
    fn calculate_term_frequency(&self, content: &str, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut total_frequency = 0.0;
        let mut term_count = 0;
        
        for word in &query_words {
            let frequency = content_lower.matches(word).count() as f32;
            if frequency > 0.0 {
                total_frequency += frequency;
                term_count += 1;
            }
        }
        
        if term_count == 0 {
            0.0
        } else {
            total_frequency / term_count as f32
        }
    }

    /// Calculate collection relevance score
    fn calculate_collection_relevance(&self, collection: &str, query: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let collection_lower = collection.to_lowercase();
        
        // Check if collection name contains query terms
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut matches = 0;
        
        for word in &query_words {
            if collection_lower.contains(word) {
                matches += 1;
            }
        }
        
        if query_words.is_empty() {
            0.0
        } else {
            matches as f32 / query_words.len() as f32
        }
    }

    /// Get document count
    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }

    /// Get documents by collection
    pub fn get_documents_by_collection(&self, collection: &str) -> Vec<&Document> {
        self.documents.values()
            .filter(|doc| doc.collection == collection)
            .collect()
    }
}

impl Default for SimpleSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_search_engine_creation() {
        let engine = SimpleSearchEngine::new();
        assert_eq!(engine.get_document_count(), 0);
    }

    #[tokio::test]
    async fn test_add_documents() {
        let mut engine = SimpleSearchEngine::new();
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        
        let documents = vec![
            Document {
                id: "doc1".to_string(),
                content: "This is a test document about vectorizer".to_string(),
                collection: "test".to_string(),
                metadata: metadata.clone(),
            },
            Document {
                id: "doc2".to_string(),
                content: "Another document about search algorithms".to_string(),
                collection: "test".to_string(),
                metadata,
            },
        ];
        
        let result = engine.add_documents(documents).await;
        assert!(result.is_ok());
        assert_eq!(engine.get_document_count(), 2);
    }

    #[tokio::test]
    async fn test_search() {
        let mut engine = SimpleSearchEngine::new();
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        
        let documents = vec![
            Document {
                id: "doc1".to_string(),
                content: "This is a test document about vectorizer".to_string(),
                collection: "test".to_string(),
                metadata,
            },
        ];
        
        engine.add_documents(documents).await.unwrap();
        
        let results = engine.search("vectorizer", vec!["test".to_string()], 10).await;
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, "doc1");
    }

    #[test]
    fn test_calculate_score() {
        let engine = SimpleSearchEngine::new();
        
        let score = engine.calculate_score("vectorizer is a vector database", "vectorizer");
        assert!(score > 0.0);
        
        let score2 = engine.calculate_score("completely different content", "vectorizer");
        assert_eq!(score2, 0.0);
    }

    #[test]
    fn test_calculate_term_frequency() {
        let engine = SimpleSearchEngine::new();
        
        let tf_score = engine.calculate_term_frequency("vectorizer is a vector database", "vectorizer database");
        assert!(tf_score > 0.0);
    }

    #[test]
    fn test_calculate_collection_relevance() {
        let engine = SimpleSearchEngine::new();
        
        let relevance_score = engine.calculate_collection_relevance("vectorizer-docs", "vectorizer documentation");
        assert!(relevance_score > 0.0);
    }

    #[test]
    fn test_get_documents_by_collection() {
        let mut engine = SimpleSearchEngine::new();
        
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        
        let documents = vec![
            Document {
                id: "doc1".to_string(),
                content: "Test content".to_string(),
                collection: "test1".to_string(),
                metadata: metadata.clone(),
            },
            Document {
                id: "doc2".to_string(),
                content: "More test content".to_string(),
                collection: "test2".to_string(),
                metadata,
            },
        ];
        
        engine.add_documents(documents).await.unwrap();
        
        let test1_docs = engine.get_documents_by_collection("test1");
        assert_eq!(test1_docs.len(), 1);
        assert_eq!(test1_docs[0].id, "doc1");
        
        let test2_docs = engine.get_documents_by_collection("test2");
        assert_eq!(test2_docs.len(), 1);
        assert_eq!(test2_docs[0].id, "doc2");
    }

    #[test]
    fn test_default_implementation() {
        let engine = SimpleSearchEngine::default();
        assert_eq!(engine.get_document_count(), 0);
    }
}
