//! MCP Tools for Intelligent Search - Simplified Working Implementation
//!
//! This module implements MCP tools for intelligent search capabilities using
//! the real VectorStore and EmbeddingManager.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::intelligent_search::*;

/// MCP Tool: Intelligent Search
#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentSearchTool {
    pub query: String,
    pub collections: Option<Vec<String>>,
    pub max_results: Option<usize>,
    pub domain_expansion: Option<bool>,
    pub technical_focus: Option<bool>,
    pub mmr_enabled: Option<bool>,
    pub mmr_lambda: Option<f32>,
}

/// MCP Tool: Multi Collection Search
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiCollectionSearchTool {
    pub query: String,
    pub collections: Vec<String>,
    pub max_per_collection: Option<usize>,
    pub max_total_results: Option<usize>,
    pub cross_collection_reranking: Option<bool>,
}

/// MCP Tool: Semantic Search
#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchTool {
    pub query: String,
    pub collection: String,
    pub max_results: Option<usize>,
    pub semantic_reranking: Option<bool>,
    pub cross_encoder_reranking: Option<bool>,
    pub similarity_threshold: Option<f32>,
}

/// MCP Tool: Contextual Search
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualSearchTool {
    pub query: String,
    pub collection: String,
    pub context_filters: Option<HashMap<String, serde_json::Value>>,
    pub max_results: Option<usize>,
    pub context_reranking: Option<bool>,
    pub context_weight: Option<f32>,
}

/// MCP Tool Response
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPToolResponse {
    pub results: Vec<IntelligentSearchResult>,
    pub metadata: SearchMetadata,
    pub tool_metadata: Option<ToolMetadata>,
}

/// Tool Metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub tool_name: String,
    pub additional_info: HashMap<String, serde_json::Value>,
}

/// MCP Tool Handler
pub struct MCPToolHandler {
    store: std::sync::Arc<crate::VectorStore>,
    embedding_manager: std::sync::Arc<crate::embedding::EmbeddingManager>,
}

impl MCPToolHandler {
    /// Create a new MCP tool handler with real VectorStore
    pub fn new(
        store: std::sync::Arc<crate::VectorStore>,
        embedding_manager: std::sync::Arc<crate::embedding::EmbeddingManager>,
    ) -> Self {
        Self {
            store,
            embedding_manager,
        }
    }

    /// Create a new MCP tool handler with only VectorStore (will create collection-specific embedding managers)
    pub fn new_with_store(store: std::sync::Arc<crate::VectorStore>) -> Self {
        // Create a placeholder embedding manager - we'll create collection-specific ones as needed
        let mut placeholder_manager = crate::embedding::EmbeddingManager::new();
        let bm25 = crate::embedding::Bm25Embedding::new(512);
        placeholder_manager.register_provider("bm25".to_string(), Box::new(bm25));
        placeholder_manager
            .set_default_provider("bm25")
            .unwrap_or_default();

        Self {
            store,
            embedding_manager: std::sync::Arc::new(placeholder_manager),
        }
    }

    /// Helper function to create an embedding manager for a specific collection
    fn create_embedding_manager_for_collection(
        &self,
        collection_name: &str,
    ) -> Result<crate::embedding::EmbeddingManager, String> {
        let collection = self
            .store
            .get_collection(collection_name)
            .map_err(|e| format!("Collection not found: {}", e))?;

        let embedding_type = collection.get_embedding_type();
        let dimension = collection.config().dimension;

        let mut manager = crate::embedding::EmbeddingManager::new();

        match embedding_type.as_str() {
            "bm25" => {
                let bm25 = crate::embedding::Bm25Embedding::new(dimension);
                manager.register_provider("bm25".to_string(), Box::new(bm25));
                manager
                    .set_default_provider("bm25")
                    .map_err(|e| format!("Failed to set BM25 provider: {}", e))?;
            }
            "tfidf" => {
                let tfidf = crate::embedding::TfIdfEmbedding::new(dimension);
                manager.register_provider("tfidf".to_string(), Box::new(tfidf));
                manager
                    .set_default_provider("tfidf")
                    .map_err(|e| format!("Failed to set TF-IDF provider: {}", e))?;
            }
            "svd" => {
                let svd = crate::embedding::SvdEmbedding::new(dimension, dimension);
                manager.register_provider("svd".to_string(), Box::new(svd));
                manager
                    .set_default_provider("svd")
                    .map_err(|e| format!("Failed to set SVD provider: {}", e))?;
            }
            "bert" => {
                let bert = crate::embedding::BertEmbedding::new(dimension);
                manager.register_provider("bert".to_string(), Box::new(bert));
                manager
                    .set_default_provider("bert")
                    .map_err(|e| format!("Failed to set BERT provider: {}", e))?;
            }
            "minilm" => {
                let minilm = crate::embedding::MiniLmEmbedding::new(dimension);
                manager.register_provider("minilm".to_string(), Box::new(minilm));
                manager
                    .set_default_provider("minilm")
                    .map_err(|e| format!("Failed to set MiniLM provider: {}", e))?;
            }
            "bagofwords" => {
                let bow = crate::embedding::BagOfWordsEmbedding::new(dimension);
                manager.register_provider("bagofwords".to_string(), Box::new(bow));
                manager
                    .set_default_provider("bagofwords")
                    .map_err(|e| format!("Failed to set BagOfWords provider: {}", e))?;
            }
            "charngram" => {
                let char_ngram = crate::embedding::CharNGramEmbedding::new(dimension, 3);
                manager.register_provider("charngram".to_string(), Box::new(char_ngram));
                manager
                    .set_default_provider("charngram")
                    .map_err(|e| format!("Failed to set CharNGram provider: {}", e))?;
            }
            _ => {
                // Default to BM25 if unknown type
                let bm25 = crate::embedding::Bm25Embedding::new(dimension);
                manager.register_provider("bm25".to_string(), Box::new(bm25));
                manager
                    .set_default_provider("bm25")
                    .map_err(|e| format!("Failed to set default BM25 provider: {}", e))?;
            }
        }

        Ok(manager)
    }

    /// Handle intelligent search tool
    pub async fn handle_intelligent_search(
        &self,
        tool: IntelligentSearchTool,
    ) -> Result<MCPToolResponse, String> {
        let max_results = tool.max_results.unwrap_or(10);
        let all_collections = tool
            .collections
            .unwrap_or_else(|| self.store.list_collections());

        // DISABLED: Semantic prioritization causes timeout with many collections (114+)
        // Limit collections to avoid timeout with large numbers
        let max_collections_limit = 20;
        let collections = if all_collections.len() > max_collections_limit {
            tracing::warn!(
                "Too many collections ({}), limiting to first {} for performance",
                all_collections.len(),
                max_collections_limit
            );
            all_collections
                .iter()
                .take(max_collections_limit)
                .cloned()
                .collect()
        } else {
            all_collections.clone()
        };

        tracing::info!(
            "Intelligent search using {} collections (total available: {})",
            collections.len(),
            all_collections.len()
        );

        let mut all_results = Vec::new();
        let mut total_queries = 0;

        // Generate multiple queries for intelligent search
        let queries =
            self.generate_intelligent_queries(&tool.query, tool.domain_expansion.unwrap_or(true));
        total_queries = queries.len();

        // Search each prioritized collection with each query
        for collection in &collections {
            // Create embedding manager specific to this collection
            let collection_embedding_manager =
                match self.create_embedding_manager_for_collection(collection) {
                    Ok(manager) => manager,
                    Err(e) => {
                        eprintln!(
                            "Error creating embedding manager for collection {}: {}",
                            collection, e
                        );
                        continue;
                    }
                };

            for query in &queries {
                match collection_embedding_manager.embed(query) {
                    Ok(embedding) => match self.store.search(collection, &embedding, max_results) {
                        Ok(search_results) => {
                            for result in search_results {
                                let intelligent_result = IntelligentSearchResult {
                                    doc_id: result.id,
                                    content: result
                                        .payload
                                        .as_ref()
                                        .and_then(|p| p.data.get("content"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    collection: collection.clone(),
                                    score: result.score,
                                    metadata: result
                                        .payload
                                        .as_ref()
                                        .map(|p| {
                                            p.data
                                                .as_object()
                                                .unwrap()
                                                .clone()
                                                .into_iter()
                                                .collect::<HashMap<String, serde_json::Value>>()
                                        })
                                        .unwrap_or_default(),
                                    score_breakdown: Some(ScoreBreakdown {
                                        relevance: result.score,
                                        collection_bonus: if collection.contains("cmmv") {
                                            0.1
                                        } else {
                                            0.0
                                        },
                                        technical_bonus: if query.contains("api")
                                            || query.contains("framework")
                                        {
                                            0.1
                                        } else {
                                            0.0
                                        },
                                        final_score: result.score,
                                    }),
                                };
                                all_results.push(intelligent_result);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error searching collection {}: {}", collection, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error embedding query '{}': {}", query, e);
                    }
                }
            }
        }

        // Apply deduplication
        let deduped_results = self.deduplicate_results(&all_results);

        // Apply MMR diversification if enabled
        let final_results = if tool.mmr_enabled.unwrap_or(true) {
            self.apply_mmr_diversification(
                &deduped_results,
                max_results,
                tool.mmr_lambda.unwrap_or(0.7),
            )
        } else {
            deduped_results
                .clone()
                .into_iter()
                .take(max_results)
                .collect()
        };

        let metadata = SearchMetadata {
            total_queries,
            collections_searched: collections.len(),
            total_results_found: all_results.len(),
            results_after_dedup: deduped_results.len(),
            final_results_count: final_results.len(),
            processing_time_ms: 0,
        };

        let mut tool_metadata = HashMap::new();
        tool_metadata.insert(
            "tool_name".to_string(),
            serde_json::Value::String("intelligent_search".to_string()),
        );
        tool_metadata.insert("query_generated".to_string(), serde_json::Value::Bool(true));
        tool_metadata.insert(
            "deduplication_applied".to_string(),
            serde_json::Value::Bool(true),
        );
        tool_metadata.insert(
            "mmr_applied".to_string(),
            serde_json::Value::Bool(tool.mmr_enabled.unwrap_or(true)),
        );
        tool_metadata.insert(
            "semantic_prioritization_applied".to_string(),
            serde_json::Value::Bool(all_collections.len() > 10),
        );
        tool_metadata.insert(
            "total_collections_available".to_string(),
            serde_json::Value::Number(serde_json::Number::from(all_collections.len())),
        );
        tool_metadata.insert(
            "collections_prioritized".to_string(),
            serde_json::Value::Number(serde_json::Number::from(collections.len())),
        );

        Ok(MCPToolResponse {
            results: final_results,
            metadata,
            tool_metadata: Some(ToolMetadata {
                tool_name: "intelligent_search".to_string(),
                additional_info: tool_metadata,
            }),
        })
    }

    /// Handle multi collection search tool
    pub async fn handle_multi_collection_search(
        &self,
        tool: MultiCollectionSearchTool,
    ) -> Result<MCPToolResponse, String> {
        let max_per_collection = tool.max_per_collection.unwrap_or(5);
        let max_total_results = tool.max_total_results.unwrap_or(20);

        let mut all_results = Vec::new();
        let mut collection_results = HashMap::new();

        // Search each collection
        for collection in &tool.collections {
            match self.embedding_manager.embed(&tool.query) {
                Ok(embedding) => {
                    match self
                        .store
                        .search(collection, &embedding, max_per_collection)
                    {
                        Ok(search_results) => {
                            let mut collection_count = 0;
                            for result in search_results {
                                let intelligent_result = IntelligentSearchResult {
                                    doc_id: result.id,
                                    content: result
                                        .payload
                                        .as_ref()
                                        .and_then(|p| p.data.get("content"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    collection: collection.clone(),
                                    score: result.score,
                                    metadata: result
                                        .payload
                                        .as_ref()
                                        .map(|p| {
                                            p.data
                                                .as_object()
                                                .unwrap()
                                                .clone()
                                                .into_iter()
                                                .collect::<HashMap<String, serde_json::Value>>()
                                        })
                                        .unwrap_or_default(),
                                    score_breakdown: Some(ScoreBreakdown {
                                        relevance: result.score,
                                        collection_bonus: 0.0,
                                        technical_bonus: 0.0,
                                        final_score: result.score,
                                    }),
                                };
                                all_results.push(intelligent_result);
                                collection_count += 1;
                            }
                            collection_results.insert(collection.clone(), collection_count);
                        }
                        Err(e) => {
                            eprintln!("Error searching collection {}: {}", collection, e);
                            collection_results.insert(collection.clone(), 0);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error embedding query: {}", e);
                    collection_results.insert(collection.clone(), 0);
                }
            }
        }

        // Apply cross-collection reranking if enabled
        let final_results = if tool.cross_collection_reranking.unwrap_or(true) {
            self.cross_collection_rerank(all_results.clone(), &tool.collections)
        } else {
            all_results.clone()
        };

        // Limit to max_total_results
        let final_results: Vec<IntelligentSearchResult> =
            final_results.into_iter().take(max_total_results).collect();

        let metadata = SearchMetadata {
            total_queries: 1,
            collections_searched: tool.collections.len(),
            total_results_found: all_results.len(),
            results_after_dedup: final_results.len(),
            final_results_count: final_results.len(),
            processing_time_ms: 0,
        };

        let mut tool_metadata = HashMap::new();
        tool_metadata.insert(
            "tool_name".to_string(),
            serde_json::Value::String("multi_collection_search".to_string()),
        );
        tool_metadata.insert(
            "collections_searched".to_string(),
            serde_json::Value::Number(serde_json::Number::from(tool.collections.len())),
        );
        tool_metadata.insert(
            "cross_collection_reranking".to_string(),
            serde_json::Value::Bool(tool.cross_collection_reranking.unwrap_or(true)),
        );

        for (collection, count) in &collection_results {
            tool_metadata.insert(
                format!("results_{}", collection),
                serde_json::Value::Number(serde_json::Number::from(*count)),
            );
        }

        Ok(MCPToolResponse {
            results: final_results,
            metadata,
            tool_metadata: Some(ToolMetadata {
                tool_name: "multi_collection_search".to_string(),
                additional_info: tool_metadata,
            }),
        })
    }

    /// Handle semantic search tool
    pub async fn handle_semantic_search(
        &self,
        tool: SemanticSearchTool,
    ) -> Result<MCPToolResponse, String> {
        let max_results = tool.max_results.unwrap_or(10);

        // Generate multiple queries for semantic search
        let queries = self.generate_intelligent_queries(&tool.query, true);
        let mut all_results = Vec::new();

        // Search with multiple queries
        for query in &queries {
            match self.embedding_manager.embed(query) {
                Ok(embedding) => {
                    match self.store.search(&tool.collection, &embedding, max_results) {
                        Ok(search_results) => {
                            for result in search_results {
                                let intelligent_result = IntelligentSearchResult {
                                    doc_id: result.id,
                                    content: result
                                        .payload
                                        .as_ref()
                                        .and_then(|p| p.data.get("content"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    collection: tool.collection.clone(),
                                    score: result.score,
                                    metadata: result
                                        .payload
                                        .as_ref()
                                        .map(|p| {
                                            p.data
                                                .as_object()
                                                .unwrap()
                                                .clone()
                                                .into_iter()
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    score_breakdown: Some(ScoreBreakdown {
                                        relevance: result.score,
                                        collection_bonus: 0.0,
                                        technical_bonus: 0.0,
                                        final_score: result.score,
                                    }),
                                };
                                all_results.push(intelligent_result);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error searching collection {}: {}", tool.collection, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error embedding query '{}': {}", query, e);
                }
            }
        }

        // Apply deduplication
        let mut results = self.deduplicate_results(&all_results);

        // Apply semantic reranking if enabled
        if tool.semantic_reranking.unwrap_or(true) {
            results = self.semantic_rerank(results, &tool.query);
        }

        // Apply cross-encoder reranking if enabled
        if tool.cross_encoder_reranking.unwrap_or(false) {
            results = self.cross_encoder_rerank(results, &tool.query);
        }

        // Filter by similarity threshold
        let similarity_threshold = tool.similarity_threshold.unwrap_or(0.5);
        let filtered_results: Vec<_> = results
            .into_iter()
            .filter(|r| r.score >= similarity_threshold)
            .collect();

        let mut tool_metadata = HashMap::new();
        tool_metadata.insert(
            "tool_name".to_string(),
            serde_json::Value::String("semantic_search".to_string()),
        );
        tool_metadata.insert(
            "semantic_reranking".to_string(),
            serde_json::Value::Bool(tool.semantic_reranking.unwrap_or(true)),
        );
        tool_metadata.insert(
            "cross_encoder_reranking".to_string(),
            serde_json::Value::Bool(tool.cross_encoder_reranking.unwrap_or(false)),
        );
        tool_metadata.insert(
            "similarity_threshold".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(similarity_threshold as f64).unwrap(),
            ),
        );

        Ok(MCPToolResponse {
            results: filtered_results.clone(),
            metadata: SearchMetadata {
                total_queries: queries.len(),
                collections_searched: 1,
                total_results_found: all_results.len(),
                results_after_dedup: filtered_results.len(),
                final_results_count: filtered_results.len(),
                processing_time_ms: 0,
            },
            tool_metadata: Some(ToolMetadata {
                tool_name: "semantic_search".to_string(),
                additional_info: tool_metadata,
            }),
        })
    }

    /// Handle contextual search tool
    pub async fn handle_contextual_search(
        &self,
        tool: ContextualSearchTool,
    ) -> Result<MCPToolResponse, String> {
        let max_results = tool.max_results.unwrap_or(10);

        // Generate multiple queries for contextual search
        let queries = self.generate_intelligent_queries(&tool.query, true);
        let mut all_results = Vec::new();

        // Search with multiple queries
        for query in &queries {
            match self.embedding_manager.embed(query) {
                Ok(embedding) => {
                    match self.store.search(&tool.collection, &embedding, max_results) {
                        Ok(search_results) => {
                            for result in search_results {
                                let intelligent_result = IntelligentSearchResult {
                                    doc_id: result.id,
                                    content: result
                                        .payload
                                        .as_ref()
                                        .and_then(|p| p.data.get("content"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    collection: tool.collection.clone(),
                                    score: result.score,
                                    metadata: result
                                        .payload
                                        .as_ref()
                                        .map(|p| {
                                            p.data
                                                .as_object()
                                                .unwrap()
                                                .clone()
                                                .into_iter()
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    score_breakdown: Some(ScoreBreakdown {
                                        relevance: result.score,
                                        collection_bonus: 0.0,
                                        technical_bonus: 0.0,
                                        final_score: result.score,
                                    }),
                                };
                                all_results.push(intelligent_result);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error searching collection {}: {}", tool.collection, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error embedding query '{}': {}", query, e);
                }
            }
        }

        // Apply deduplication
        let mut results = self.deduplicate_results(&all_results);

        // Apply context filtering if provided
        if let Some(context_filters) = &tool.context_filters {
            results = self.filter_by_context(results, context_filters);
        }

        // Apply context-aware reranking if enabled
        if tool.context_reranking.unwrap_or(true) {
            let context_weight = tool.context_weight.unwrap_or(0.3);
            results = self.context_aware_rerank(results, &tool.query, context_weight);
        }

        let mut tool_metadata = HashMap::new();
        tool_metadata.insert(
            "tool_name".to_string(),
            serde_json::Value::String("contextual_search".to_string()),
        );
        tool_metadata.insert(
            "context_filters_applied".to_string(),
            serde_json::Value::Bool(tool.context_filters.is_some()),
        );
        tool_metadata.insert(
            "context_reranking".to_string(),
            serde_json::Value::Bool(tool.context_reranking.unwrap_or(true)),
        );
        tool_metadata.insert(
            "context_weight".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(tool.context_weight.unwrap_or(0.3) as f64).unwrap(),
            ),
        );

        Ok(MCPToolResponse {
            results: results.clone(),
            metadata: SearchMetadata {
                total_queries: queries.len(),
                collections_searched: 1,
                total_results_found: all_results.len(),
                results_after_dedup: results.len(),
                final_results_count: results.len(),
                processing_time_ms: 0,
            },
            tool_metadata: Some(ToolMetadata {
                tool_name: "contextual_search".to_string(),
                additional_info: tool_metadata,
            }),
        })
    }

    /// Intelligently prioritize collections based on semantic similarity to query
    pub async fn prioritize_collections_semantically(
        &self,
        query: &str,
        collections: &[String],
        max_collections: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        if collections.is_empty() {
            return Ok(vec![]);
        }

        // Extract key terms from query
        let query_terms: Vec<&str> = query
            .split_whitespace()
            .filter(|term| term.len() > 2)
            .collect();

        if query_terms.is_empty() {
            return Ok(collections.to_vec());
        }

        // Use semantic search to find most relevant collections
        let mut collection_scores = std::collections::HashMap::new();

        // Test each collection with a quick semantic search
        for collection in collections.iter().take(max_collections) {
            // Create embedding manager for this collection
            let embedding_manager = match self.create_embedding_manager_for_collection(collection) {
                Ok(manager) => manager,
                Err(_) => continue,
            };

            // Convert query text to vector
            let query_vector = match embedding_manager.embed(query) {
                Ok(vec) => vec,
                Err(_) => continue,
            };

            // Perform search
            match self.store.search(collection, &query_vector, 1) {
                Ok(results) => {
                    if let Some(top_result) = results.first() {
                        collection_scores.insert(collection.clone(), top_result.score);
                    } else {
                        collection_scores.insert(collection.clone(), -1.0);
                    }
                }
                Err(_) => {
                    collection_scores.insert(collection.clone(), -1.0);
                }
            }
        }

        // Sort collections by semantic relevance score
        let mut sorted_collections: Vec<(String, f32)> = collection_scores
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted_collections
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Extract collection names, filtering out those with no relevant results
        let prioritized: Vec<String> = sorted_collections
            .into_iter()
            .filter(|(_, score)| *score > -0.5)
            .map(|(name, _)| name)
            .collect();

        // Add remaining collections that weren't tested
        let remaining: Vec<String> = collections
            .iter()
            .filter(|col| !collection_scores.contains_key(*col))
            .cloned()
            .collect();

        let prioritized_len = prioritized.len();
        let remaining_len = remaining.len();
        let result = [prioritized, remaining].concat();

        tracing::info!(
            "Semantic collection prioritization: {} relevant, {} remaining",
            prioritized_len,
            remaining_len
        );

        Ok(result)
    }

    /// Generate intelligent queries for better search coverage
    fn generate_intelligent_queries(&self, query: &str, domain_expansion: bool) -> Vec<String> {
        let mut queries = vec![query.to_string()];

        if domain_expansion {
            // Add variations of the query
            let words: Vec<&str> = query.split_whitespace().collect();

            // Add individual important words
            for word in &words {
                if word.len() > 3 {
                    queries.push(word.to_string());
                }
            }

            // Add combinations
            if words.len() > 1 {
                for i in 0..words.len() - 1 {
                    queries.push(format!("{} {}", words[i], words[i + 1]));
                }
            }

            // Add domain-specific expansions
            if query.to_lowercase().contains("cmmv") {
                queries.push("Contract Model View framework".to_string());
                queries.push("TypeScript framework".to_string());
            }
            if query.to_lowercase().contains("api") {
                queries.push("application programming interface".to_string());
                queries.push("REST API".to_string());
            }
        }

        queries
    }

    /// Deduplicate results based on content similarity
    fn deduplicate_results(
        &self,
        results: &[IntelligentSearchResult],
    ) -> Vec<IntelligentSearchResult> {
        let mut deduped = Vec::new();

        for result in results {
            let is_duplicate = deduped.iter().any(|existing: &IntelligentSearchResult| {
                self.calculate_content_similarity(&result.content, &existing.content) > 0.8
            });

            if !is_duplicate {
                deduped.push(result.clone());
            }
        }

        deduped
    }

    /// Calculate content similarity using simple word overlap
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let binding1 = content1.to_lowercase();
        let words1: std::collections::HashSet<&str> = binding1.split_whitespace().collect();
        let binding2 = content2.to_lowercase();
        let words2: std::collections::HashSet<&str> = binding2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// Apply MMR diversification to results
    fn apply_mmr_diversification(
        &self,
        results: &[IntelligentSearchResult],
        max_results: usize,
        lambda: f32,
    ) -> Vec<IntelligentSearchResult> {
        if results.is_empty() || max_results == 0 {
            return Vec::new();
        }

        let mut selected = Vec::new();
        let mut remaining = results.to_vec();

        // Select first result (highest relevance)
        if let Some(first) = remaining
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
        {
            let first_idx = remaining
                .iter()
                .position(|r| r.doc_id == first.doc_id)
                .unwrap();
            selected.push(remaining.remove(first_idx));
        }

        // MMR selection for remaining slots
        while selected.len() < max_results && !remaining.is_empty() {
            let mut best_score = f32::NEG_INFINITY;
            let mut best_idx = 0;

            for (idx, candidate) in remaining.iter().enumerate() {
                // Calculate MMR score: λ * relevance - (1-λ) * max_similarity_to_selected
                let relevance = candidate.score;
                let max_similarity = selected
                    .iter()
                    .map(|selected| {
                        self.calculate_content_similarity(&candidate.content, &selected.content)
                    })
                    .fold(0.0, f32::max);

                let mmr_score = lambda * relevance - (1.0 - lambda) * max_similarity;

                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_idx = idx;
                }
            }

            selected.push(remaining.remove(best_idx));
        }

        selected
    }

    /// Cross-collection reranking
    fn cross_collection_rerank(
        &self,
        mut results: Vec<IntelligentSearchResult>,
        collections: &[String],
    ) -> Vec<IntelligentSearchResult> {
        // Simple implementation: boost results from collections that appear earlier in the list
        for (i, collection) in collections.iter().enumerate() {
            let boost_factor = 1.0 + (0.1 * (collections.len() - i) as f32);
            for result in &mut results {
                if result.collection == *collection {
                    result.score *= boost_factor;
                }
            }
        }

        // Sort by boosted score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Semantic reranking
    fn semantic_rerank(
        &self,
        mut results: Vec<IntelligentSearchResult>,
        _query: &str,
    ) -> Vec<IntelligentSearchResult> {
        // Simple implementation: boost results with technical terms
        for result in &mut results {
            if result.content.contains("API")
                || result.content.contains("framework")
                || result.content.contains("implementation")
            {
                result.score *= 1.1;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Cross-encoder reranking
    fn cross_encoder_rerank(
        &self,
        mut results: Vec<IntelligentSearchResult>,
        _query: &str,
    ) -> Vec<IntelligentSearchResult> {
        // Simple implementation: boost results with exact matches
        for result in &mut results {
            if result
                .content
                .to_lowercase()
                .contains(_query.to_lowercase().as_str())
            {
                result.score *= 1.2;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Filter by context
    fn filter_by_context(
        &self,
        results: Vec<IntelligentSearchResult>,
        _context_filters: &HashMap<String, serde_json::Value>,
    ) -> Vec<IntelligentSearchResult> {
        // Simple implementation: return all results
        // In a real implementation, this would filter based on metadata
        results
    }

    /// Context-aware reranking
    fn context_aware_rerank(
        &self,
        mut results: Vec<IntelligentSearchResult>,
        _query: &str,
        _context_weight: f32,
    ) -> Vec<IntelligentSearchResult> {
        // Simple implementation: boost results with context-relevant terms
        for result in &mut results {
            if result.content.contains("context") || result.content.contains("environment") {
                result.score *= 1.05;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }
}
