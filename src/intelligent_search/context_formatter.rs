//! Context Formatter Module - Simplified Implementation
//! 
//! This module implements intelligent context formatting for search results
//! with token budget management.

use crate::intelligent_search::IntelligentSearchResult;
use std::collections::HashMap;

/// Context formatter for search results
pub struct ContextFormatter {
    max_content_length: usize,
    max_lines_per_result: usize,
    include_metadata: bool,
}

impl ContextFormatter {
    /// Create a new context formatter
    pub fn new(
        max_content_length: usize,
        max_lines_per_result: usize,
        include_metadata: bool,
    ) -> Self {
        Self {
            max_content_length,
            max_lines_per_result,
            include_metadata,
        }
    }

    /// Format search results into context
    pub fn format_context(
        &self,
        results: &[IntelligentSearchResult],
        query: &str,
    ) -> String {
        if results.is_empty() {
            return String::new();
        }

        let mut context_parts = Vec::new();
        
        for result in results {
            let formatted_result = self.format_single_result(result, query);
            context_parts.push(formatted_result);
        }
        
        context_parts.join("\n\n")
    }

    /// Format a single search result
    fn format_single_result(
        &self,
        result: &IntelligentSearchResult,
        query: &str,
    ) -> String {
        let mut parts = Vec::new();
        
        // Add header with collection and score
        let header = format!(
            "[{}] (score: {:.3})",
            result.collection,
            result.score
        );
        parts.push(header);
        
        // Add formatted content
        let formatted_content = self.format_content(&result.content, query);
        parts.push(formatted_content);
        
        // Add metadata if enabled
        if self.include_metadata && !result.metadata.is_empty() {
            let metadata_str = self.format_metadata(&result.metadata);
            if !metadata_str.is_empty() {
                parts.push(format!("Metadata: {}", metadata_str));
            }
        }
        
        parts.join("\n")
    }

    /// Format content with relevance extraction
    fn format_content(&self, content: &str, query: &str) -> String {
        // Extract relevant lines first
        let relevant_lines = self.extract_relevant_lines(content, query);
        
        if !relevant_lines.is_empty() {
            // Use relevant lines if found
            relevant_lines.join("\n")
        } else {
            // Fall back to truncated content
            self.truncate_content(content)
        }
    }

    /// Extract relevant lines based on query terms
    fn extract_relevant_lines(&self, content: &str, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        
        let lines: Vec<&str> = content.split('\n').collect();
        let mut relevant_lines = Vec::new();
        
        for line in lines {
            let line_lower = line.to_lowercase();
            let is_relevant = query_terms.iter().any(|term| line_lower.contains(term));
            
            if is_relevant {
                relevant_lines.push(line.trim().to_string());
                if relevant_lines.len() >= self.max_lines_per_result {
                    break;
                }
            }
        }
        
        relevant_lines
    }

    /// Truncate content to maximum length
    fn truncate_content(&self, content: &str) -> String {
        if content.len() <= self.max_content_length {
            content.to_string()
        } else {
            let truncated = &content[..self.max_content_length];
            format!("{}...", truncated)
        }
    }

    /// Format metadata
    fn format_metadata(&self, metadata: &HashMap<String, serde_json::Value>) -> String {
        let mut metadata_parts = Vec::new();
        
        for (key, value) in metadata {
            if let Some(value_str) = value.as_str() {
                metadata_parts.push(format!("{}: {}", key, value_str));
            } else if let Some(value_num) = value.as_f64() {
                metadata_parts.push(format!("{}: {}", key, value_num));
            } else if let Some(value_bool) = value.as_bool() {
                metadata_parts.push(format!("{}: {}", key, value_bool));
            }
        }
        
        metadata_parts.join(", ")
    }

    /// Format context with enhanced structure
    pub fn format_enhanced_context(
        &self,
        results: &[IntelligentSearchResult],
        query: &str,
        search_metadata: Option<&str>,
    ) -> String {
        let mut context_parts = Vec::new();
        
        // Add search metadata if provided
        if let Some(metadata) = search_metadata {
            context_parts.push(format!("Search Context: {}", metadata));
        }
        
        // Add query information
        context_parts.push(format!("Query: {}", query));
        context_parts.push(format!("Results: {} found", results.len()));
        
        // Add separator
        context_parts.push("---".to_string());
        
        // Add formatted results
        let formatted_results = self.format_context(results, query);
        context_parts.push(formatted_results);
        
        context_parts.join("\n")
    }

    /// Get configuration
    pub fn get_max_content_length(&self) -> usize {
        self.max_content_length
    }

    /// Get maximum lines per result
    pub fn get_max_lines_per_result(&self) -> usize {
        self.max_lines_per_result
    }

    /// Check if metadata is included
    pub fn is_metadata_included(&self) -> bool {
        self.include_metadata
    }
}

impl Default for ContextFormatter {
    fn default() -> Self {
        Self::new(400, 5, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_result(content: &str, score: f32) -> IntelligentSearchResult {
        IntelligentSearchResult {
            content: content.to_string(),
            score,
            collection: "docs".to_string(),
            doc_id: "doc1".to_string(),
            metadata: HashMap::new(),
            score_breakdown: None,
        }
    }

    #[test]
    fn test_context_formatter_creation() {
        let formatter = ContextFormatter::new(400, 5, false);
        assert_eq!(formatter.get_max_content_length(), 400);
        assert_eq!(formatter.get_max_lines_per_result(), 5);
        assert!(!formatter.is_metadata_included());
    }

    #[test]
    fn test_format_context_empty() {
        let formatter = ContextFormatter::default();
        let results = vec![];
        let context = formatter.format_context(&results, "test query");
        assert!(context.is_empty());
    }

    #[test]
    fn test_format_single_result() {
        let formatter = ContextFormatter::default();
        let result = create_test_result("vectorizer is a vector database for semantic search", 0.85);
        
        let formatted = formatter.format_single_result(&result, "vectorizer");
        assert!(formatted.contains("docs"));
        assert!(formatted.contains("0.850"));
        assert!(formatted.contains("vectorizer"));
    }

    #[test]
    fn test_extract_relevant_lines() {
        let formatter = ContextFormatter::new(400, 3, false);
        let content = "This is about vectorizer.\nVectorizer is a vector database.\nThis is unrelated content.\nVectorizer performance is excellent.";
        let query = "vectorizer performance";
        
        let relevant_lines = formatter.extract_relevant_lines(content, query);
        assert!(!relevant_lines.is_empty());
        assert!(relevant_lines.len() <= 3);
    }

    #[test]
    fn test_truncate_content() {
        let formatter = ContextFormatter::new(50, 5, false);
        let content = "This is a very long content that should be truncated because it exceeds the maximum length";
        
        let truncated = formatter.truncate_content(content);
        assert!(truncated.len() <= 53); // 50 + "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_format_metadata() {
        let formatter = ContextFormatter::default();
        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        metadata.insert("version".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
        metadata.insert("active".to_string(), serde_json::Value::Bool(true));
        
        let formatted = formatter.format_metadata(&metadata);
        assert!(formatted.contains("author: John Doe"));
        assert!(formatted.contains("version: 1"));
        assert!(formatted.contains("active: true"));
    }

    #[test]
    fn test_format_enhanced_context() {
        let formatter = ContextFormatter::default();
        let results = vec![
            create_test_result("vectorizer is a vector database", 0.8),
        ];
        
        let enhanced = formatter.format_enhanced_context(&results, "vectorizer", Some("Found 1 result"));
        assert!(enhanced.contains("Search Context: Found 1 result"));
        assert!(enhanced.contains("Query: vectorizer"));
        assert!(enhanced.contains("Results: 1 found"));
        assert!(enhanced.contains("---"));
    }

    #[test]
    fn test_default_implementation() {
        let formatter = ContextFormatter::default();
        assert_eq!(formatter.get_max_content_length(), 400);
        assert_eq!(formatter.get_max_lines_per_result(), 5);
        assert!(!formatter.is_metadata_included());
    }
}