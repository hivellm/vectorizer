//! MCP Server Integration for Intelligent Search
//! 
//! This module shows how to integrate the intelligent search tools with the MCP server

use crate::intelligent_search::mcp_tools::*;
use crate::intelligent_search::rest_api::*;
use serde_json::{Value, Map};
use std::collections::HashMap;

/// MCP Server Integration Example
pub struct MCPServerIntegration {
    mcp_handler: MCPToolHandler,
    rest_handler: RESTAPIHandler,
}

impl MCPServerIntegration {
    /// Create a new MCP server integration
    pub fn new() -> Self {
        // Note: In real usage, you would pass actual VectorStore and EmbeddingManager instances
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        Self {
            mcp_handler: MCPToolHandler::new(store.clone(), embedding_manager.clone()),
            rest_handler: RESTAPIHandler::new(),
        }
    }

    /// Handle MCP tool call
    pub async fn handle_mcp_tool_call(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, String> {
        match tool_name {
            "intelligent_search" => {
                let tool: IntelligentSearchTool = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments: {}", e))?;
                
                let response = self.mcp_handler.handle_intelligent_search(tool).await?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "multi_collection_search" => {
                let tool: MultiCollectionSearchTool = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments: {}", e))?;
                
                let response = self.mcp_handler.handle_multi_collection_search(tool).await?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "semantic_search" => {
                let tool: SemanticSearchTool = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments: {}", e))?;
                
                let response = self.mcp_handler.handle_semantic_search(tool).await?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "contextual_search" => {
                let tool: ContextualSearchTool = serde_json::from_value(arguments)
                    .map_err(|e| format!("Invalid arguments: {}", e))?;
                
                let response = self.mcp_handler.handle_contextual_search(tool).await?;
                Ok(serde_json::to_value(response).unwrap())
            }
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    /// Get available MCP tools
    pub fn get_available_tools(&self) -> Vec<HashMap<String, Value>> {
        vec![
            Self::create_tool_schema(
                "intelligent_search",
                "Perform intelligent search with query generation, deduplication, and MMR diversification",
                vec![
                    ("query", "string", true, "Search query"),
                    ("collections", "array", false, "Collections to search"),
                    ("max_results", "number", false, "Maximum number of results"),
                    ("domain_expansion", "boolean", false, "Enable domain expansion"),
                    ("technical_focus", "boolean", false, "Enable technical focus"),
                    ("mmr_enabled", "boolean", false, "Enable MMR diversification"),
                    ("mmr_lambda", "number", false, "MMR lambda parameter (0.0-1.0)"),
                ],
            ),
            Self::create_tool_schema(
                "multi_collection_search",
                "Search across multiple collections with intelligent ranking",
                vec![
                    ("query", "string", true, "Search query"),
                    ("collections", "array", true, "Collections to search"),
                    ("max_per_collection", "number", false, "Maximum results per collection"),
                    ("max_total_results", "number", false, "Maximum total results"),
                    ("cross_collection_reranking", "boolean", false, "Enable cross-collection reranking"),
                ],
            ),
            Self::create_tool_schema(
                "semantic_search",
                "Perform semantic search with advanced reranking",
                vec![
                    ("query", "string", true, "Search query"),
                    ("collection", "string", true, "Collection to search"),
                    ("max_results", "number", false, "Maximum number of results"),
                    ("semantic_reranking", "boolean", false, "Enable semantic reranking"),
                    ("cross_encoder_reranking", "boolean", false, "Enable cross-encoder reranking"),
                    ("similarity_threshold", "number", false, "Similarity threshold (0.0-1.0)"),
                ],
            ),
            Self::create_tool_schema(
                "contextual_search",
                "Perform context-aware search with metadata filtering",
                vec![
                    ("query", "string", true, "Search query"),
                    ("collection", "string", true, "Collection to search"),
                    ("context_filters", "object", false, "Context metadata filters"),
                    ("max_results", "number", false, "Maximum number of results"),
                    ("context_reranking", "boolean", false, "Enable context-aware reranking"),
                    ("context_weight", "number", false, "Context weight in scoring (0.0-1.0)"),
                ],
            ),
        ]
    }

    /// Create tool schema for MCP
    fn create_tool_schema(
        name: &str,
        description: &str,
        parameters: Vec<(&str, &str, bool, &str)>,
    ) -> HashMap<String, Value> {
        let mut schema = HashMap::new();
        schema.insert("name".to_string(), Value::String(name.to_string()));
        schema.insert("description".to_string(), Value::String(description.to_string()));
        
        let mut properties = Map::new();
        let mut required = Vec::new();
        
        for (param_name, param_type, is_required, param_description) in parameters {
            let mut param_schema = Map::new();
            param_schema.insert("type".to_string(), Value::String(param_type.to_string()));
            param_schema.insert("description".to_string(), Value::String(param_description.to_string()));
            
            properties.insert(param_name.to_string(), Value::Object(param_schema));
            
            if is_required {
                required.push(Value::String(param_name.to_string()));
            }
        }
        
        let mut input_schema = Map::new();
        input_schema.insert("type".to_string(), Value::String("object".to_string()));
        input_schema.insert("properties".to_string(), Value::Object(properties));
        input_schema.insert("required".to_string(), Value::Array(required));
        
        schema.insert("inputSchema".to_string(), Value::Object(input_schema));
        
        schema
    }

    /// Handle REST API request
    pub async fn handle_rest_request(
        &self,
        endpoint: &str,
        method: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        if method != "POST" {
            return Err("Only POST method is supported".to_string());
        }

        let body = body.ok_or("Request body is required")?;

        match endpoint {
            "/api/intelligent-search" => {
                let request: IntelligentSearchRequest = serde_json::from_value(body)
                    .map_err(|e| format!("Invalid request: {}", e))?;
                
                self.rest_handler.validate_intelligent_search_request(&request)?;
                let response = self.rest_handler.handle_intelligent_search(request).await
                    .map_err(|e| e.error)?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "/api/multi-collection-search" => {
                let request: MultiCollectionSearchRequest = serde_json::from_value(body)
                    .map_err(|e| format!("Invalid request: {}", e))?;
                
                self.rest_handler.validate_multi_collection_search_request(&request)?;
                let response = self.rest_handler.handle_multi_collection_search(request).await
                    .map_err(|e| e.error)?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "/api/semantic-search" => {
                let request: SemanticSearchRequest = serde_json::from_value(body)
                    .map_err(|e| format!("Invalid request: {}", e))?;
                
                self.rest_handler.validate_semantic_search_request(&request)?;
                let response = self.rest_handler.handle_semantic_search(request).await
                    .map_err(|e| e.error)?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "/api/contextual-search" => {
                let request: ContextualSearchRequest = serde_json::from_value(body)
                    .map_err(|e| format!("Invalid request: {}", e))?;
                
                self.rest_handler.validate_contextual_search_request(&request)?;
                let response = self.rest_handler.handle_contextual_search(request).await
                    .map_err(|e| e.error)?;
                Ok(serde_json::to_value(response).unwrap())
            }
            "/api/docs" => {
                Ok(serde_json::to_value(APIDocumentation::get_documentation()).unwrap())
            }
            _ => Err(format!("Unknown endpoint: {}", endpoint)),
        }
    }

    /// Get REST API endpoints
    pub fn get_rest_endpoints(&self) -> Vec<HashMap<String, Value>> {
        vec![
            Self::create_endpoint_schema(
                "/api/intelligent-search",
                "POST",
                "Perform intelligent search with query generation, deduplication, and MMR diversification",
            ),
            Self::create_endpoint_schema(
                "/api/multi-collection-search",
                "POST",
                "Search across multiple collections with intelligent ranking",
            ),
            Self::create_endpoint_schema(
                "/api/semantic-search",
                "POST",
                "Perform semantic search with advanced reranking",
            ),
            Self::create_endpoint_schema(
                "/api/contextual-search",
                "POST",
                "Perform context-aware search with metadata filtering",
            ),
            Self::create_endpoint_schema(
                "/api/docs",
                "GET",
                "Get API documentation",
            ),
        ]
    }

    /// Create endpoint schema
    fn create_endpoint_schema(path: &str, method: &str, description: &str) -> HashMap<String, Value> {
        let mut schema = HashMap::new();
        schema.insert("path".to_string(), Value::String(path.to_string()));
        schema.insert("method".to_string(), Value::String(method.to_string()));
        schema.insert("description".to_string(), Value::String(description.to_string()));
        schema
    }
}

impl Default for MCPServerIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_integration_creation() {
        let integration = MCPServerIntegration::new();
        // Integration should be created successfully
        assert!(true);
    }

    #[test]
    fn test_get_available_tools() {
        let integration = MCPServerIntegration::new();
        let tools = integration.get_available_tools();
        
        assert_eq!(tools.len(), 4);
        
        let tool_names: Vec<&str> = tools.iter()
            .map(|t| t.get("name").unwrap().as_str().unwrap())
            .collect();
        
        assert!(tool_names.contains(&"intelligent_search"));
        assert!(tool_names.contains(&"multi_collection_search"));
        assert!(tool_names.contains(&"semantic_search"));
        assert!(tool_names.contains(&"contextual_search"));
    }

    #[test]
    fn test_get_rest_endpoints() {
        let integration = MCPServerIntegration::new();
        let endpoints = integration.get_rest_endpoints();
        
        assert_eq!(endpoints.len(), 5);
        
        let paths: Vec<&str> = endpoints.iter()
            .map(|e| e.get("path").unwrap().as_str().unwrap())
            .collect();
        
        assert!(paths.contains(&"/api/intelligent-search"));
        assert!(paths.contains(&"/api/multi-collection-search"));
        assert!(paths.contains(&"/api/semantic-search"));
        assert!(paths.contains(&"/api/contextual-search"));
        assert!(paths.contains(&"/api/docs"));
    }

    #[test]
    fn test_create_tool_schema() {
        let schema = MCPServerIntegration::create_tool_schema(
            "test_tool",
            "Test tool description",
            vec![
                ("param1", "string", true, "Required parameter"),
                ("param2", "number", false, "Optional parameter"),
            ],
        );
        
        assert_eq!(schema["name"], Value::String("test_tool".to_string()));
        assert_eq!(schema["description"], Value::String("Test tool description".to_string()));
        
        let input_schema = schema["inputSchema"].as_object().unwrap();
        assert_eq!(input_schema["type"], Value::String("object".to_string()));
        
        let required = input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0], Value::String("param1".to_string()));
    }
}
