//! Query Generator Module - Simplified Implementation
//!
//! This module implements intelligent query generation with domain-specific knowledge
//! using a simplified approach focused on performance.

use std::collections::HashSet;

/// Query generator with domain-specific knowledge
pub struct QueryGenerator {
    max_queries: usize,
}

impl QueryGenerator {
    /// Create a new query generator
    pub fn new(max_queries: usize) -> Self {
        Self { max_queries }
    }

    /// Generate multiple search queries from user input
    pub fn generate_queries(&self, user_query: &str) -> Vec<String> {
        let mut queries = vec![user_query.to_string()];

        // Extract technical terms
        let terms = self.extract_technical_terms(user_query);

        if let Some(main_term) = terms.first() {
            // Generate technical documentation queries
            queries.extend(self.generate_technical_queries(main_term));

            // Add domain-specific expansion
            queries.extend(self.expand_domain_terms(main_term));
        }

        // Remove duplicates and limit
        queries
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .take(self.max_queries)
            .collect()
    }

    /// Extract technical terms from query
    fn extract_technical_terms(&self, query: &str) -> Vec<String> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut terms = Vec::new();

        for word in words {
            let clean_word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if clean_word.len() > 2 && self.is_technical_term(&clean_word) {
                terms.push(clean_word);
            }
        }

        terms
    }

    /// Check if a word is a technical term
    fn is_technical_term(&self, word: &str) -> bool {
        // Common technical terms
        let technical_terms = [
            "api",
            "sdk",
            "framework",
            "library",
            "database",
            "vector",
            "search",
            "embedding",
            "model",
            "algorithm",
            "index",
            "query",
            "result",
            "data",
            "config",
            "configuration",
            "setup",
            "install",
            "deploy",
            "build",
            "test",
            "debug",
            "error",
            "log",
            "performance",
            "optimization",
            "caching",
            "memory",
            "cpu",
            "gpu",
            "network",
            "http",
            "rest",
            "json",
            "xml",
            "yaml",
            "toml",
            "rust",
            "python",
            "javascript",
            "typescript",
            "node",
            "react",
            "vue",
            "angular",
            "docker",
            "kubernetes",
            "aws",
            "azure",
            "gcp",
            "github",
            "git",
            "ci",
            "cd",
            "devops",
            "microservice",
            "architecture",
            "pattern",
            "design",
            "best",
            "practice",
            "tutorial",
            "guide",
            "documentation",
            "example",
            "sample",
            "demo",
            "cmmv",
            "vectorizer",
            "hnsw",
            "quantization",
            "persistence",
            "mcp",
        ];

        technical_terms.contains(&word)
    }

    /// Generate technical documentation queries
    fn generate_technical_queries(&self, main_term: &str) -> Vec<String> {
        vec![
            format!("{} documentation", main_term),
            format!("{} features", main_term),
            format!("{} architecture", main_term),
            format!("{} performance", main_term),
            format!("{} API", main_term),
            format!("{} usage examples", main_term),
            format!("{} configuration", main_term),
            format!("{} benchmarks", main_term),
        ]
    }

    /// Expand domain-specific terms
    fn expand_domain_terms(&self, term: &str) -> Vec<String> {
        match term {
            "vectorizer" => vec![
                "vector database".to_string(),
                "semantic search".to_string(),
                "HNSW indexing".to_string(),
                "embedding models".to_string(),
                "similarity search".to_string(),
                "vector quantization".to_string(),
            ],
            "cmmv" => vec![
                "CMMV framework".to_string(),
                "Contract Model View".to_string(),
                "TypeScript framework".to_string(),
                "component architecture".to_string(),
            ],
            "hnsw" => vec![
                "hierarchical navigable small world".to_string(),
                "graph-based indexing".to_string(),
                "approximate nearest neighbor".to_string(),
                "HNSW performance".to_string(),
            ],
            "vector" => vec![
                "vector database".to_string(),
                "embedding storage".to_string(),
                "similarity search".to_string(),
                "vector indexing".to_string(),
            ],
            "search" => vec![
                "semantic search".to_string(),
                "vector search".to_string(),
                "neural search".to_string(),
                "similarity search".to_string(),
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_generator_creation() {
        let generator = QueryGenerator::new(8);
        assert_eq!(generator.max_queries, 8);
    }

    #[test]
    fn test_generate_queries_basic() {
        let generator = QueryGenerator::new(8);
        let queries = generator.generate_queries("vectorizer performance");

        assert!(!queries.is_empty());
        assert!(queries.len() <= 8);
        // At least one query should contain "vectorizer"
        assert!(queries.iter().any(|q| q.contains("vectorizer")));
    }

    #[test]
    fn test_extract_technical_terms() {
        let generator = QueryGenerator::new(8);
        let terms = generator.extract_technical_terms("vectorizer API documentation");

        assert!(terms.contains(&"vectorizer".to_string()));
        assert!(terms.contains(&"api".to_string()));
        assert!(terms.contains(&"documentation".to_string()));
    }

    #[test]
    fn test_is_technical_term() {
        let generator = QueryGenerator::new(8);

        assert!(generator.is_technical_term("api"));
        assert!(generator.is_technical_term("vectorizer"));
        assert!(generator.is_technical_term("performance"));
        assert!(!generator.is_technical_term("the"));
        assert!(!generator.is_technical_term("a"));
    }

    #[test]
    fn test_generate_technical_queries() {
        let generator = QueryGenerator::new(8);
        let queries = generator.generate_technical_queries("vectorizer");

        assert!(queries.contains(&"vectorizer documentation".to_string()));
        assert!(queries.contains(&"vectorizer features".to_string()));
        assert!(queries.contains(&"vectorizer architecture".to_string()));
        assert!(queries.contains(&"vectorizer performance".to_string()));
        assert!(queries.contains(&"vectorizer API".to_string()));
    }

    #[test]
    fn test_expand_domain_terms() {
        let generator = QueryGenerator::new(8);

        let vectorizer_expansions = generator.expand_domain_terms("vectorizer");
        assert!(vectorizer_expansions.contains(&"vector database".to_string()));
        assert!(vectorizer_expansions.contains(&"semantic search".to_string()));

        let cmmv_expansions = generator.expand_domain_terms("cmmv");
        assert!(cmmv_expansions.contains(&"CMMV framework".to_string()));
        assert!(cmmv_expansions.contains(&"Contract Model View".to_string()));

        let unknown_expansions = generator.expand_domain_terms("unknown");
        assert!(unknown_expansions.is_empty());
    }

    #[test]
    fn test_query_deduplication() {
        let generator = QueryGenerator::new(8);
        let queries = generator.generate_queries("vectorizer vectorizer");

        // Should not contain duplicates
        let unique_queries: HashSet<&String> = queries.iter().collect();
        assert_eq!(unique_queries.len(), queries.len());
    }

    #[test]
    fn test_max_queries_limit() {
        let generator = QueryGenerator::new(3);
        let queries = generator.generate_queries("vectorizer performance API documentation");

        assert!(queries.len() <= 3);
    }
}
