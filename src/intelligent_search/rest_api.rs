//! REST API Tools for Intelligent Search
//!
//! This module implements REST API endpoints for intelligent search capabilities:
//! - /api/intelligent-search: Main intelligent search endpoint
//! - /api/multi-collection-search: Multi-collection search endpoint
//! - /api/semantic-search: Semantic search endpoint
//! - /api/contextual-search: Contextual search endpoint

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::intelligent_search::mcp_tools::*;
use crate::intelligent_search::*;

/// REST API Request for Intelligent Search
#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentSearchRequest {
    /// Search query
    pub query: String,
    /// Collections to search (optional)
    pub collections: Option<Vec<String>>,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable domain expansion
    pub domain_expansion: Option<bool>,
    /// Enable technical focus
    pub technical_focus: Option<bool>,
    /// Enable MMR diversification
    pub mmr_enabled: Option<bool>,
    /// MMR lambda parameter
    pub mmr_lambda: Option<f32>,
}

/// REST API Request for Multi Collection Search
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiCollectionSearchRequest {
    /// Search query
    pub query: String,
    /// Collections to search
    pub collections: Vec<String>,
    /// Maximum results per collection
    pub max_per_collection: Option<usize>,
    /// Maximum total results
    pub max_total_results: Option<usize>,
    /// Enable cross-collection reranking
    pub cross_collection_reranking: Option<bool>,
}

/// REST API Request for Semantic Search
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Search query
    pub query: String,
    /// Collection to search
    pub collection: String,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable semantic reranking
    pub semantic_reranking: Option<bool>,
    /// Enable cross-encoder reranking
    pub cross_encoder_reranking: Option<bool>,
    /// Similarity threshold
    pub similarity_threshold: Option<f32>,
}

/// REST API Request for Contextual Search
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualSearchRequest {
    /// Search query
    pub query: String,
    /// Collection to search
    pub collection: String,
    /// Context metadata filters
    pub context_filters: Option<HashMap<String, serde_json::Value>>,
    /// Maximum number of results
    pub max_results: Option<usize>,
    /// Enable context-aware reranking
    pub context_reranking: Option<bool>,
    /// Context weight in scoring
    pub context_weight: Option<f32>,
}

/// REST API Response
#[derive(Debug, Serialize, Deserialize)]
pub struct RESTSearchResponse {
    /// Search results
    pub results: Vec<IntelligentSearchResult>,
    /// Search metadata
    pub metadata: SearchMetadata,
    /// Tool-specific metadata
    pub tool_metadata: Option<ToolMetadata>,
    /// API version
    pub api_version: String,
    /// Request timestamp
    pub timestamp: String,
}

/// REST API Error Response
#[derive(Debug, Serialize, Deserialize)]
pub struct RESTErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
    /// API version
    pub api_version: String,
    /// Request timestamp
    pub timestamp: String,
}

/// REST API Handler
pub struct RESTAPIHandler {
    mcp_handler: MCPToolHandler,
}

impl RESTAPIHandler {
    /// Create a new REST API handler
    pub fn new() -> Self {
        // Note: In real usage, you would pass actual VectorStore and EmbeddingManager instances
        let store = std::sync::Arc::new(crate::VectorStore::new());
        let embedding_manager = std::sync::Arc::new(crate::embedding::EmbeddingManager::new());
        Self {
            mcp_handler: MCPToolHandler::new(store, embedding_manager),
        }
    }

    /// Create a new REST API handler with existing VectorStore
    pub fn new_with_store(store: std::sync::Arc<crate::VectorStore>) -> Self {
        Self {
            mcp_handler: MCPToolHandler::new_with_store(store),
        }
    }

    /// Handle intelligent search request
    pub async fn handle_intelligent_search(
        &self,
        request: IntelligentSearchRequest,
    ) -> Result<RESTSearchResponse, RESTErrorResponse> {
        let tool = IntelligentSearchTool {
            query: request.query,
            collections: request.collections,
            max_results: request.max_results,
            domain_expansion: request.domain_expansion,
            technical_focus: request.technical_focus,
            mmr_enabled: request.mmr_enabled,
            mmr_lambda: request.mmr_lambda,
        };

        match self.mcp_handler.handle_intelligent_search(tool).await {
            Ok(response) => Ok(RESTSearchResponse {
                results: response.results,
                metadata: response.metadata,
                tool_metadata: response.tool_metadata,
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            Err(error) => Err(RESTErrorResponse {
                error,
                code: "INTELLIGENT_SEARCH_ERROR".to_string(),
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Handle multi collection search request
    pub async fn handle_multi_collection_search(
        &self,
        request: MultiCollectionSearchRequest,
    ) -> Result<RESTSearchResponse, RESTErrorResponse> {
        let tool = MultiCollectionSearchTool {
            query: request.query,
            collections: request.collections,
            max_per_collection: request.max_per_collection,
            max_total_results: request.max_total_results,
            cross_collection_reranking: request.cross_collection_reranking,
        };

        match self.mcp_handler.handle_multi_collection_search(tool).await {
            Ok(response) => Ok(RESTSearchResponse {
                results: response.results,
                metadata: response.metadata,
                tool_metadata: response.tool_metadata,
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            Err(error) => Err(RESTErrorResponse {
                error,
                code: "MULTI_COLLECTION_SEARCH_ERROR".to_string(),
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Handle semantic search request
    pub async fn handle_semantic_search(
        &self,
        request: SemanticSearchRequest,
    ) -> Result<RESTSearchResponse, RESTErrorResponse> {
        let tool = SemanticSearchTool {
            query: request.query,
            collection: request.collection,
            max_results: request.max_results,
            semantic_reranking: request.semantic_reranking,
            cross_encoder_reranking: request.cross_encoder_reranking,
            similarity_threshold: request.similarity_threshold,
        };

        match self.mcp_handler.handle_semantic_search(tool).await {
            Ok(response) => Ok(RESTSearchResponse {
                results: response.results,
                metadata: response.metadata,
                tool_metadata: response.tool_metadata,
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            Err(error) => Err(RESTErrorResponse {
                error,
                code: "SEMANTIC_SEARCH_ERROR".to_string(),
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Handle contextual search request
    pub async fn handle_contextual_search(
        &self,
        request: ContextualSearchRequest,
    ) -> Result<RESTSearchResponse, RESTErrorResponse> {
        let tool = ContextualSearchTool {
            query: request.query,
            collection: request.collection,
            context_filters: request.context_filters,
            max_results: request.max_results,
            context_reranking: request.context_reranking,
            context_weight: request.context_weight,
        };

        match self.mcp_handler.handle_contextual_search(tool).await {
            Ok(response) => Ok(RESTSearchResponse {
                results: response.results,
                metadata: response.metadata,
                tool_metadata: response.tool_metadata,
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
            Err(error) => Err(RESTErrorResponse {
                error,
                code: "CONTEXTUAL_SEARCH_ERROR".to_string(),
                api_version: "1.0.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        }
    }

    /// Validate request parameters
    pub fn validate_intelligent_search_request(
        &self,
        request: &IntelligentSearchRequest,
    ) -> Result<(), String> {
        if request.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        if let Some(max_results) = request.max_results {
            if max_results == 0 || max_results > 1000 {
                return Err("Max results must be between 1 and 1000".to_string());
            }
        }

        if let Some(mmr_lambda) = request.mmr_lambda {
            if mmr_lambda < 0.0 || mmr_lambda > 1.0 {
                return Err("MMR lambda must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Validate multi collection search request
    pub fn validate_multi_collection_search_request(
        &self,
        request: &MultiCollectionSearchRequest,
    ) -> Result<(), String> {
        if request.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        if request.collections.is_empty() {
            return Err("At least one collection must be specified".to_string());
        }

        if let Some(max_per_collection) = request.max_per_collection {
            if max_per_collection == 0 || max_per_collection > 100 {
                return Err("Max per collection must be between 1 and 100".to_string());
            }
        }

        if let Some(max_total_results) = request.max_total_results {
            if max_total_results == 0 || max_total_results > 1000 {
                return Err("Max total results must be between 1 and 1000".to_string());
            }
        }

        Ok(())
    }

    /// Validate semantic search request
    pub fn validate_semantic_search_request(
        &self,
        request: &SemanticSearchRequest,
    ) -> Result<(), String> {
        if request.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        if request.collection.trim().is_empty() {
            return Err("Collection cannot be empty".to_string());
        }

        if let Some(max_results) = request.max_results {
            if max_results == 0 || max_results > 1000 {
                return Err("Max results must be between 1 and 1000".to_string());
            }
        }

        if let Some(similarity_threshold) = request.similarity_threshold {
            if similarity_threshold < 0.0 || similarity_threshold > 1.0 {
                return Err("Similarity threshold must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Validate contextual search request
    pub fn validate_contextual_search_request(
        &self,
        request: &ContextualSearchRequest,
    ) -> Result<(), String> {
        if request.query.trim().is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        if request.collection.trim().is_empty() {
            return Err("Collection cannot be empty".to_string());
        }

        if let Some(max_results) = request.max_results {
            if max_results == 0 || max_results > 1000 {
                return Err("Max results must be between 1 and 1000".to_string());
            }
        }

        if let Some(context_weight) = request.context_weight {
            if context_weight < 0.0 || context_weight > 1.0 {
                return Err("Context weight must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }
}

impl Default for RESTAPIHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// API Documentation
pub struct APIDocumentation;

impl APIDocumentation {
    /// Get API documentation
    pub fn get_documentation() -> HashMap<String, serde_json::Value> {
        let mut docs = HashMap::new();

        docs.insert(
            "version".to_string(),
            serde_json::Value::String("1.0.0".to_string()),
        );
        docs.insert(
            "title".to_string(),
            serde_json::Value::String("Vectorizer Intelligent Search API".to_string()),
        );
        docs.insert(
            "description".to_string(),
            serde_json::Value::String("REST API for intelligent search capabilities".to_string()),
        );

        let endpoints = serde_json::json!({
            "intelligent_search": {
                "path": "/api/intelligent-search",
                "method": "POST",
                "description": "Perform intelligent search with query generation, deduplication, and MMR diversification",
                "parameters": {
                    "query": "string (required) - Search query",
                    "collections": "array[string] (optional) - Collections to search",
                    "max_results": "number (optional) - Maximum number of results (1-1000)",
                    "domain_expansion": "boolean (optional) - Enable domain expansion",
                    "technical_focus": "boolean (optional) - Enable technical focus",
                    "mmr_enabled": "boolean (optional) - Enable MMR diversification",
                    "mmr_lambda": "number (optional) - MMR lambda parameter (0.0-1.0)"
                }
            },
            "multi_collection_search": {
                "path": "/api/multi-collection-search",
                "method": "POST",
                "description": "Search across multiple collections with intelligent ranking",
                "parameters": {
                    "query": "string (required) - Search query",
                    "collections": "array[string] (required) - Collections to search",
                    "max_per_collection": "number (optional) - Maximum results per collection (1-100)",
                    "max_total_results": "number (optional) - Maximum total results (1-1000)",
                    "cross_collection_reranking": "boolean (optional) - Enable cross-collection reranking"
                }
            },
            "semantic_search": {
                "path": "/api/semantic-search",
                "method": "POST",
                "description": "Perform semantic search with advanced reranking",
                "parameters": {
                    "query": "string (required) - Search query",
                    "collection": "string (required) - Collection to search",
                    "max_results": "number (optional) - Maximum number of results (1-1000)",
                    "semantic_reranking": "boolean (optional) - Enable semantic reranking",
                    "cross_encoder_reranking": "boolean (optional) - Enable cross-encoder reranking",
                    "similarity_threshold": "number (optional) - Similarity threshold (0.0-1.0)"
                }
            },
            "contextual_search": {
                "path": "/api/contextual-search",
                "method": "POST",
                "description": "Perform context-aware search with metadata filtering",
                "parameters": {
                    "query": "string (required) - Search query",
                    "collection": "string (required) - Collection to search",
                    "context_filters": "object (optional) - Context metadata filters",
                    "max_results": "number (optional) - Maximum number of results (1-1000)",
                    "context_reranking": "boolean (optional) - Enable context-aware reranking",
                    "context_weight": "number (optional) - Context weight in scoring (0.0-1.0)"
                }
            }
        });

        docs.insert("endpoints".to_string(), endpoints);

        docs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_api_handler_creation() {
        let handler = RESTAPIHandler::new();
        // Handler should be created successfully
        assert!(true);
    }

    #[test]
    fn test_intelligent_search_request_serialization() {
        let request = IntelligentSearchRequest {
            query: "test query".to_string(),
            collections: Some(vec!["test".to_string()]),
            max_results: Some(10),
            domain_expansion: Some(true),
            technical_focus: Some(true),
            mmr_enabled: Some(true),
            mmr_lambda: Some(0.7),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: IntelligentSearchRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.query, deserialized.query);
        assert_eq!(request.collections, deserialized.collections);
        assert_eq!(request.max_results, deserialized.max_results);
    }

    #[test]
    fn test_validate_intelligent_search_request() {
        let handler = RESTAPIHandler::new();

        // Valid request
        let valid_request = IntelligentSearchRequest {
            query: "test query".to_string(),
            collections: None,
            max_results: Some(10),
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };
        assert!(
            handler
                .validate_intelligent_search_request(&valid_request)
                .is_ok()
        );

        // Invalid request - empty query
        let invalid_request = IntelligentSearchRequest {
            query: "".to_string(),
            collections: None,
            max_results: None,
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };
        assert!(
            handler
                .validate_intelligent_search_request(&invalid_request)
                .is_err()
        );

        // Invalid request - max_results too high
        let invalid_request2 = IntelligentSearchRequest {
            query: "test query".to_string(),
            collections: None,
            max_results: Some(2000),
            domain_expansion: None,
            technical_focus: None,
            mmr_enabled: None,
            mmr_lambda: None,
        };
        assert!(
            handler
                .validate_intelligent_search_request(&invalid_request2)
                .is_err()
        );
    }

    #[test]
    fn test_validate_multi_collection_search_request() {
        let handler = RESTAPIHandler::new();

        // Valid request
        let valid_request = MultiCollectionSearchRequest {
            query: "test query".to_string(),
            collections: vec!["collection1".to_string(), "collection2".to_string()],
            max_per_collection: Some(5),
            max_total_results: Some(10),
            cross_collection_reranking: Some(true),
        };
        assert!(
            handler
                .validate_multi_collection_search_request(&valid_request)
                .is_ok()
        );

        // Invalid request - empty collections
        let invalid_request = MultiCollectionSearchRequest {
            query: "test query".to_string(),
            collections: vec![],
            max_per_collection: None,
            max_total_results: None,
            cross_collection_reranking: None,
        };
        assert!(
            handler
                .validate_multi_collection_search_request(&invalid_request)
                .is_err()
        );
    }

    #[test]
    fn test_api_documentation() {
        let docs = APIDocumentation::get_documentation();

        assert!(docs.contains_key("version"));
        assert!(docs.contains_key("title"));
        assert!(docs.contains_key("description"));
        assert!(docs.contains_key("endpoints"));

        assert_eq!(
            docs["version"],
            serde_json::Value::String("1.0.0".to_string())
        );
        assert_eq!(
            docs["title"],
            serde_json::Value::String("Vectorizer Intelligent Search API".to_string())
        );
    }
}
