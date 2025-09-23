//! MCP tools implementation
//! 
//! Provides specific tool implementations for vector database operations

use crate::db::VectorStore;
use crate::embedding::EmbeddingManager;
use crate::embedding::TfIdfEmbedding;
use crate::error::{Result, VectorizerError};
use serde_json;
use tracing::{debug};

/// Tool implementations for MCP
pub struct McpTools;

impl McpTools {
    /// Search vectors in a collection
    pub async fn search_vectors(
        collection: &str,
        query: &str,
        limit: usize,
        vector_store: &VectorStore,
        embedding_manager: &EmbeddingManager,
    ) -> Result<serde_json::Value> {
        debug!("Searching vectors in collection '{}' with query '{}'", collection, query);

        // Check if collection exists
        if !vector_store.list_collections().contains(&collection.to_string()) {
            return Err(VectorizerError::CollectionNotFound(collection.to_string()));
        }

        // Generate embedding for the query
        let query_embedding = embedding_manager.embed(query)
            .map_err(|e| VectorizerError::Other(format!("Failed to embed query: {}", e)))?;

        // Search in the collection
        let results = vector_store.search(collection, &query_embedding, limit)
            .map_err(|e| VectorizerError::Other(format!("Search failed: {}", e)))?;

        // Convert results to JSON
        let json_results: Vec<serde_json::Value> = results
            .into_iter()
            .map(|result| {
                serde_json::json!({
                    "id": result.id,
                    "score": result.score,
                    "payload": result.payload
                })
            })
            .collect();

        Ok(serde_json::json!({
            "results": json_results,
            "query": query,
            "collection": collection,
            "limit": limit,
            "total_results": json_results.len()
        }))
    }

    /// List all collections
    pub async fn list_collections(vector_store: &VectorStore) -> Result<serde_json::Value> {
        debug!("Listing all collections");

        let collections = vector_store.list_collections();
        let _total_count = collections.len();
        
        let collection_info: Vec<serde_json::Value> = collections
            .into_iter()
            .map(|name| {
                // Get collection metadata
                match vector_store.get_collection_metadata(&name) {
                    Ok(metadata) => {
                        serde_json::json!({
                            "name": name,
                            "vector_count": metadata.vector_count,
                            "dimension": metadata.config.dimension,
                            "metric": metadata.config.metric,
                            "hnsw_config": {
                                "m": metadata.config.hnsw_config.m,
                                "ef_construction": metadata.config.hnsw_config.ef_construction,
                                "ef_search": metadata.config.hnsw_config.ef_search
                            }
                        })
                    }
                    Err(_) => {
                        serde_json::json!({
                            "name": name,
                            "error": "Failed to get metadata"
                        })
                    }
                }
            })
            .collect();

        Ok(serde_json::json!({
            "collections": collection_info,
            "total_count": collection_info.len()
        }))
    }

    /// Get detailed information about a collection
    pub async fn get_collection_info(
        collection: &str,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Getting info for collection '{}'", collection);

        let metadata = vector_store.get_collection_metadata(collection)?;

        Ok(serde_json::json!({
            "name": collection,
            "vector_count": metadata.vector_count,
            "dimension": metadata.config.dimension,
            "metric": metadata.config.metric,
            "hnsw_config": {
                "m": metadata.config.hnsw_config.m,
                "ef_construction": metadata.config.hnsw_config.ef_construction,
                "ef_search": metadata.config.hnsw_config.ef_search,
                "seed": metadata.config.hnsw_config.seed
            },
            "compression": {
                "enabled": metadata.config.compression.enabled,
                "threshold_bytes": metadata.config.compression.threshold_bytes,
                "algorithm": metadata.config.compression.algorithm
            },
            "quantization": metadata.config.quantization
        }))
    }

    /// Generate embeddings for text
    pub async fn embed_text(
        text: &str,
        embedding_manager: &EmbeddingManager,
    ) -> Result<serde_json::Value> {
        debug!("Generating embedding for text: '{}'", text);

        let embedding = embedding_manager.embed(text)
            .map_err(|e| VectorizerError::Other(format!("Failed to embed text: {}", e)))?;

        Ok(serde_json::json!({
            "embedding": embedding,
            "text": text,
            "dimension": embedding.len(),
            "provider": "default"
        }))
    }

    /// Insert vectors into a collection
    pub async fn insert_vectors(
        collection: &str,
        vectors: Vec<(String, Vec<f32>, Option<serde_json::Value>)>,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Inserting {} vectors into collection '{}'", vectors.len(), collection);

        use crate::models::Vector;

        let vector_objects: Vec<Vector> = vectors
            .into_iter()
            .map(|(id, data, payload)| {
                if let Some(payload_data) = payload {
                    Vector::with_payload(id, data, crate::models::Payload::from_value(payload_data).unwrap_or_default())
                } else {
                    Vector::new(id, data)
                }
            })
            .collect();

        let inserted_count = vector_objects.len();
        vector_store.insert(collection, vector_objects)?;

        Ok(serde_json::json!({
            "inserted_count": inserted_count,
            "collection": collection,
            "status": "success"
        }))
    }

    /// Delete vectors from a collection
    pub async fn delete_vectors(
        collection: &str,
        vector_ids: Vec<String>,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Deleting {} vectors from collection '{}'", vector_ids.len(), collection);

        let mut deleted_count = 0;
        let mut errors = Vec::new();

        for vector_id in vector_ids {
            match vector_store.delete(collection, &vector_id) {
                Ok(_) => deleted_count += 1,
                Err(e) => {
                    errors.push(format!("Failed to delete {}: {}", vector_id, e));
                }
            }
        }

        Ok(serde_json::json!({
            "deleted_count": deleted_count,
            "collection": collection,
            "errors": errors,
            "status": if errors.is_empty() { "success" } else { "partial_success" }
        }))
    }

    /// Get vector by ID
    pub async fn get_vector(
        collection: &str,
        vector_id: &str,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Getting vector '{}' from collection '{}'", vector_id, collection);

        let vector = vector_store.get_vector(collection, vector_id)?;

        Ok(serde_json::json!({
            "id": vector.id,
            "data": vector.data,
            "payload": vector.payload,
            "collection": collection
        }))
    }

    /// Create a new collection
    pub async fn create_collection(
        name: &str,
        dimension: usize,
        metric: &str,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Creating collection '{}' with dimension {}", name, dimension);

        use crate::models::{CollectionConfig, DistanceMetric};

        let distance_metric = match metric.to_lowercase().as_str() {
            "euclidean" => DistanceMetric::Euclidean,
            "cosine" => DistanceMetric::Cosine,
            "dot_product" => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine, // Default
        };

        let config = CollectionConfig {
            dimension,
            metric: distance_metric,
            hnsw_config: crate::models::HnswConfig::default(),
            quantization: None,
            compression: crate::models::CompressionConfig::default(),
        };

        vector_store.create_collection(name, config)?;

        Ok(serde_json::json!({
            "name": name,
            "dimension": dimension,
            "metric": metric,
            "status": "created"
        }))
    }

    /// Delete a collection
    pub async fn delete_collection(
        name: &str,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!("Deleting collection '{}'", name);

        vector_store.delete_collection(name)?;

        Ok(serde_json::json!({
            "name": name,
            "status": "deleted"
        }))
    }

    /// Get database statistics
    pub async fn get_database_stats(vector_store: &VectorStore) -> Result<serde_json::Value> {
        debug!("Getting database statistics");

        let collections = vector_store.list_collections();
        let mut total_vectors = 0;
        let mut total_memory_estimate = 0;

        let collection_stats: Vec<serde_json::Value> = collections
            .into_iter()
            .map(|name| {
                match vector_store.get_collection_metadata(&name) {
                    Ok(metadata) => {
                        total_vectors += metadata.vector_count;
                        total_memory_estimate += metadata.vector_count * metadata.config.dimension * 4; // 4 bytes per f32
                        
                        serde_json::json!({
                            "name": name,
                            "vector_count": metadata.vector_count,
                            "dimension": metadata.config.dimension,
                            "memory_estimate_bytes": metadata.vector_count * metadata.config.dimension * 4
                        })
                    }
                    Err(_) => {
                        serde_json::json!({
                            "name": name,
                            "error": "Failed to get metadata"
                        })
                    }
                }
            })
            .collect();

        Ok(serde_json::json!({
            "total_collections": collection_stats.len(),
            "total_vectors": total_vectors,
            "total_memory_estimate_bytes": total_memory_estimate,
            "collections": collection_stats
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_collections() {
        let vector_store = VectorStore::new();
        let result = McpTools::list_collections(&vector_store).await.unwrap();
        
        assert!(result.get("collections").is_some());
        assert!(result.get("total_count").is_some());
    }

    #[tokio::test]
    async fn test_get_database_stats() {
        let vector_store = VectorStore::new();
        let result = McpTools::get_database_stats(&vector_store).await.unwrap();
        
        assert!(result.get("total_collections").is_some());
        assert!(result.get("total_vectors").is_some());
        assert!(result.get("collections").is_some());
    }

    #[tokio::test]
    async fn test_embed_text() {
        let mut embedding_manager = EmbeddingManager::new();
        let mut tfidf = TfIdfEmbedding::new(64);
        tfidf.build_vocabulary(&["test document"]);
        embedding_manager.register_provider("tfidf".to_string(), Box::new(tfidf));
        embedding_manager.set_default_provider("tfidf").unwrap();

        let result = McpTools::embed_text("test text", &embedding_manager).await.unwrap();
        
        assert!(result.get("embedding").is_some());
        assert_eq!(result.get("text").unwrap(), "test text");
        assert_eq!(result.get("dimension").unwrap(), 64);
    }
}
