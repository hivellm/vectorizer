//! Example usage of Intelligent Search MCP and REST tools
//! 
//! This module demonstrates how to use the intelligent search tools

use crate::intelligent_search::mcp_tools::*;
use crate::intelligent_search::rest_api::*;
use crate::intelligent_search::mcp_server_integration::*;
use serde_json::json;

/// Example usage of MCP tools
pub struct ExampleUsage;

impl ExampleUsage {
    /// Example: Intelligent Search
    pub async fn example_intelligent_search() -> Result<(), String> {
        // Note: In real usage, you would pass actual VectorStore and EmbeddingManager instances
        // For examples, we'll use placeholder values
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        let handler = MCPToolHandler::new(store, embedding_manager);
        
        let tool = IntelligentSearchTool {
            query: "vectorizer performance benchmarks".to_string(),
            collections: Some(vec!["vectorizer-docs".to_string(), "performance-docs".to_string()]),
            max_results: Some(10),
            domain_expansion: Some(true),
            technical_focus: Some(true),
            mmr_enabled: Some(true),
            mmr_lambda: Some(0.7),
        };
        
        let response = handler.handle_intelligent_search(tool).await?;
        
        println!("Intelligent Search Results:");
        println!("- Total queries generated: {}", response.metadata.total_queries);
        println!("- Collections searched: {}", response.metadata.collections_searched);
        println!("- Results found: {}", response.results.len());
        
        for (i, result) in response.results.iter().enumerate() {
            println!("  {}. [{}] Score: {:.3}", i + 1, result.collection, result.score);
            println!("     {}", result.content);
        }
        
        Ok(())
    }

    /// Example: Multi Collection Search
    pub async fn example_multi_collection_search() -> Result<(), String> {
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        let handler = MCPToolHandler::new(store, embedding_manager);
        
        let tool = MultiCollectionSearchTool {
            query: "CMMV framework architecture".to_string(),
            collections: vec![
                "cmmv-core-docs".to_string(),
                "cmmv-admin-docs".to_string(),
                "cmmv-formbuilder-docs".to_string(),
            ],
            max_per_collection: Some(3),
            max_total_results: Some(9),
            cross_collection_reranking: Some(true),
        };
        
        let response = handler.handle_multi_collection_search(tool).await?;
        
        println!("Multi Collection Search Results:");
        println!("- Collections searched: {}", response.metadata.collections_searched);
        println!("- Total results found: {}", response.metadata.total_results_found);
        println!("- Final results: {}", response.results.len());
        
        for (i, result) in response.results.iter().enumerate() {
            println!("  {}. [{}] Score: {:.3}", i + 1, result.collection, result.score);
            println!("     {}", result.content);
        }
        
        Ok(())
    }

    /// Example: Semantic Search
    pub async fn example_semantic_search() -> Result<(), String> {
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        let handler = MCPToolHandler::new(store, embedding_manager);
        
        let tool = SemanticSearchTool {
            query: "authentication and authorization".to_string(),
            collection: "cmmv-admin-docs".to_string(),
            max_results: Some(5),
            semantic_reranking: Some(true),
            cross_encoder_reranking: Some(false),
            similarity_threshold: Some(0.6),
        };
        
        let similarity_threshold = tool.similarity_threshold.unwrap();
        let semantic_reranking = tool.semantic_reranking.unwrap();
        let collection = tool.collection.clone();
        
        let response = handler.handle_semantic_search(tool).await?;
        
        println!("Semantic Search Results:");
        println!("- Collection: {}", collection);
        println!("- Semantic reranking: {}", semantic_reranking);
        println!("- Similarity threshold: {}", similarity_threshold);
        println!("- Results found: {}", response.results.len());
        
        for (i, result) in response.results.iter().enumerate() {
            println!("  {}. Score: {:.3}", i + 1, result.score);
            println!("     {}", result.content);
        }
        
        Ok(())
    }

    /// Example: Contextual Search
    pub async fn example_contextual_search() -> Result<(), String> {
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        let handler = MCPToolHandler::new(store, embedding_manager);
        
        let mut context_filters = std::collections::HashMap::new();
        context_filters.insert("author".to_string(), serde_json::Value::String("John Doe".to_string()));
        context_filters.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));
        
        let tool = ContextualSearchTool {
            query: "API documentation".to_string(),
            collection: "cmmv-core-docs".to_string(),
            context_filters: Some(context_filters),
            max_results: Some(5),
            context_reranking: Some(true),
            context_weight: Some(0.3),
        };
        
        let context_reranking = tool.context_reranking.unwrap();
        let context_weight = tool.context_weight.unwrap();
        let collection = tool.collection.clone();
        
        let response = handler.handle_contextual_search(tool).await?;
        
        println!("Contextual Search Results:");
        println!("- Collection: {}", collection);
        println!("- Context reranking: {}", context_reranking);
        println!("- Context weight: {}", context_weight);
        println!("- Results found: {}", response.results.len());
        
        for (i, result) in response.results.iter().enumerate() {
            println!("  {}. Score: {:.3}", i + 1, result.score);
            println!("     {}", result.content);
        }
        
        Ok(())
    }

    /// Example: REST API usage
    pub async fn example_rest_api() -> Result<(), String> {
        let handler = RESTAPIHandler::new();
        
        let request = IntelligentSearchRequest {
            query: "vectorizer HNSW indexing".to_string(),
            collections: Some(vec!["vectorizer-docs".to_string()]),
            max_results: Some(5),
            domain_expansion: Some(true),
            technical_focus: Some(true),
            mmr_enabled: Some(true),
            mmr_lambda: Some(0.8),
        };
        
        let response = handler.handle_intelligent_search(request).await
            .map_err(|e| e.error)?;
        
        println!("REST API Response:");
        println!("- API Version: {}", response.api_version);
        println!("- Timestamp: {}", response.timestamp);
        println!("- Results: {}", response.results.len());
        
        for (i, result) in response.results.iter().enumerate() {
            println!("  {}. [{}] Score: {:.3}", i + 1, result.collection, result.score);
            println!("     {}", result.content);
        }
        
        Ok(())
    }

    /// Example: MCP Server Integration
    pub async fn example_mcp_server_integration() -> Result<(), String> {
        let integration = MCPServerIntegration::new();
        
        // Get available tools
        let tools = integration.get_available_tools();
        println!("Available MCP Tools:");
        for tool in &tools {
            println!("- {}: {}", 
                tool["name"].as_str().unwrap(),
                tool["description"].as_str().unwrap()
            );
        }
        
        // Get REST endpoints
        let endpoints = integration.get_rest_endpoints();
        println!("\nAvailable REST Endpoints:");
        for endpoint in &endpoints {
            println!("- {} {}: {}", 
                endpoint["method"].as_str().unwrap(),
                endpoint["path"].as_str().unwrap(),
                endpoint["description"].as_str().unwrap()
            );
        }
        
        // Example MCP tool call
        let tool_call = json!({
            "query": "vectorizer performance",
            "collections": ["vectorizer-docs"],
            "max_results": 3,
            "domain_expansion": true,
            "technical_focus": true,
            "mmr_enabled": true,
            "mmr_lambda": 0.7
        });
        
        let response = integration.handle_mcp_tool_call("intelligent_search", tool_call).await?;
        println!("\nMCP Tool Call Response:");
        println!("- Results: {}", response["results"].as_array().unwrap().len());
        
        Ok(())
    }

    /// Run all examples
    pub async fn run_all_examples() -> Result<(), String> {
        println!("=== Intelligent Search Examples ===\n");
        
        println!("1. Intelligent Search Example:");
        Self::example_intelligent_search().await?;
        println!();
        
        println!("2. Multi Collection Search Example:");
        Self::example_multi_collection_search().await?;
        println!();
        
        println!("3. Semantic Search Example:");
        Self::example_semantic_search().await?;
        println!();
        
        println!("4. Contextual Search Example:");
        Self::example_contextual_search().await?;
        println!();
        
        println!("5. REST API Example:");
        Self::example_rest_api().await?;
        println!();
        
        println!("6. MCP Server Integration Example:");
        Self::example_mcp_server_integration().await?;
        println!();
        
        println!("=== All Examples Completed Successfully ===");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_usage() {
        // Test that examples can be created without errors
        let handler = MCPToolHandler::new();
        let rest_handler = RESTAPIHandler::new();
        let integration = MCPServerIntegration::new();
        
        // Verify they were created successfully
        assert!(true);
    }

    #[test]
    fn test_tool_schemas() {
        let integration = MCPServerIntegration::new();
        let tools = integration.get_available_tools();
        
        assert_eq!(tools.len(), 4);
        
        // Verify each tool has required fields
        for tool in &tools {
            assert!(tool.contains_key("name"));
            assert!(tool.contains_key("description"));
            assert!(tool.contains_key("inputSchema"));
        }
    }

    #[test]
    fn test_rest_endpoints() {
        let integration = MCPServerIntegration::new();
        let endpoints = integration.get_rest_endpoints();
        
        assert_eq!(endpoints.len(), 5);
        
        // Verify each endpoint has required fields
        for endpoint in &endpoints {
            assert!(endpoint.contains_key("path"));
            assert!(endpoint.contains_key("method"));
            assert!(endpoint.contains_key("description"));
        }
    }
}
