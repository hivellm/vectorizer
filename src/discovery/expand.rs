//! Query expansion

use super::config::ExpansionConfig;
use super::types::DiscoveryResult;

/// Deterministic query expansion with semantic variations
pub fn expand_queries_baseline(query: &str, config: &ExpansionConfig) -> DiscoveryResult<Vec<String>> {
    let expander = QueryExpander { config };
    Ok(expander.expand(query))
}

struct QueryExpander<'a> {
    config: &'a ExpansionConfig,
}

impl<'a> QueryExpander<'a> {
    fn expand(&self, query: &str) -> Vec<String> {
        let mut expansions = vec![query.to_string()];
        let base_term = self.extract_main_term(query);
        
        if self.config.include_definition {
            expansions.push(format!("{} definition", base_term));
            expansions.push(format!("what is {}", base_term));
        }
        
        if self.config.include_features {
            expansions.push(format!("{} features", base_term));
            expansions.push(format!("{} capabilities", base_term));
            expansions.push(format!("{} main functionality", base_term));
        }
        
        if self.config.include_architecture {
            expansions.push(format!("{} architecture", base_term));
            expansions.push(format!("{} components", base_term));
            expansions.push(format!("{} system design", base_term));
        }
        
        if self.config.include_api {
            expansions.push(format!("{} API", base_term));
            expansions.push(format!("{} usage", base_term));
        }
        
        if self.config.include_performance {
            expansions.push(format!("{} performance", base_term));
            expansions.push(format!("{} benchmarks", base_term));
        }
        
        if self.config.include_use_cases {
            expansions.push(format!("{} use cases", base_term));
            expansions.push(format!("{} examples", base_term));
        }
        
        expansions.truncate(self.config.max_expansions);
        expansions
    }
    
    fn extract_main_term(&self, query: &str) -> String {
        // Remove stopwords and get main term
        let stopwords = ["o", "que", "é", "the", "is", "a", "what", "how", "de", "da", "do"];
        
        query
            .split_whitespace()
            .filter(|w| !stopwords.contains(&w.to_lowercase().as_str()))
            .next()
            .unwrap_or(query)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expand_queries() {
        let config = ExpansionConfig::default();
        let queries = expand_queries_baseline("O que é o vectorizer", &config).unwrap();
        
        assert!(!queries.is_empty());
        assert_eq!(queries[0], "O que é o vectorizer");
        assert!(queries.iter().any(|q| q.contains("definition")));
        assert!(queries.iter().any(|q| q.contains("features")));
        assert!(queries.iter().any(|q| q.contains("architecture")));
    }
    
    #[test]
    fn test_extract_main_term() {
        let config = ExpansionConfig::default();
        let expander = QueryExpander { config: &config };
        
        assert_eq!(expander.extract_main_term("O que é o vectorizer"), "vectorizer");
        assert_eq!(expander.extract_main_term("What is the database"), "database");
    }
    
    #[test]
    fn test_max_expansions() {
        let mut config = ExpansionConfig::default();
        config.max_expansions = 3;
        
        let queries = expand_queries_baseline("test query", &config).unwrap();
        assert!(queries.len() <= 3);
    }
}

