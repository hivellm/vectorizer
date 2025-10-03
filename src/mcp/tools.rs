//! MCP tools implementation
//!
//! Provides specific tool implementations for vector database operations

use crate::db::VectorStore;
use crate::models::QuantizationConfig;
use crate::embedding::EmbeddingManager;
use crate::error::{Result, VectorizerError};
use serde_json;
use tracing::{debug, info, warn};

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
        debug!(
            "Searching vectors in collection '{}' with query '{}'",
            collection, query
        );

        // Check if collection exists
        if !vector_store
            .list_collections()
            .contains(&collection.to_string())
        {
            return Err(VectorizerError::CollectionNotFound(collection.to_string()));
        }

        // Generate embedding for the query
        let query_embedding = embedding_manager
            .embed(query)
            .map_err(|e| VectorizerError::Other(format!("Failed to embed query: {}", e)))?;

        // Search in the collection
        let results = vector_store
            .search(collection, &query_embedding, limit)
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

        let embedding = embedding_manager
            .embed(text)
            .map_err(|e| VectorizerError::Other(format!("Failed to embed text: {}", e)))?;

        Ok(serde_json::json!({
            "embedding": embedding,
            "text": text,
            "dimension": embedding.len(),
            "provider": "default"
        }))
    }

    /// Insert texts into a collection (embeddings generated automatically)
    pub async fn insert_texts(
        collection: &str,
        vectors: Vec<(String, Vec<f32>, Option<serde_json::Value>)>,
        vector_store: &VectorStore,
    ) -> Result<serde_json::Value> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection
        );

        use crate::models::Vector;

        let vector_objects: Vec<Vector> = vectors
            .into_iter()
            .map(|(id, data, payload)| {
                // Create rich payload following document_loader.rs pattern
                let mut payload_data = serde_json::Map::new();

                // Add custom metadata if provided
                if let Some(mut custom_payload) = payload.and_then(|p| p.as_object().cloned()) {
                    // Extract content if present, otherwise use empty string
                    let content = custom_payload.remove("content")
                        .and_then(|c| c.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "No content provided".to_string());

                    payload_data.insert("content".to_string(), serde_json::Value::String(content));

                    // Add all other custom metadata
                    for (key, value) in custom_payload {
                        payload_data.insert(key, value);
                    }
                } else {
                    // No payload provided - add default content
                    payload_data.insert(
                        "content".to_string(),
                        serde_json::Value::String("No content provided".to_string())
                    );
                }

                // Add MCP operation metadata
                payload_data.insert(
                    "operation_type".to_string(),
                    serde_json::Value::String("mcp_insert".to_string()),
                );
                payload_data.insert(
                    "created_at".to_string(),
                    serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
                );

                let rich_payload = crate::models::Payload {
                    data: serde_json::Value::Object(payload_data),
                };

                Vector::with_payload(id, data, rich_payload)
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
        debug!(
            "Deleting {} vectors from collection '{}'",
            vector_ids.len(),
            collection
        );

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
        debug!(
            "Getting vector '{}' from collection '{}'",
            vector_id, collection
        );

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
        debug!(
            "Creating collection '{}' with dimension {}",
            name, dimension
        );

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
            quantization: QuantizationConfig::SQ { bits: 8 },
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

    /// Batch insert texts into a collection (embeddings generated automatically)
    pub async fn batch_insert_texts(
        collection: &str,
        texts: Vec<(String, String, Option<serde_json::Value>)>,
        vector_store: &VectorStore,
        embedding_manager: &EmbeddingManager,
    ) -> Result<serde_json::Value> {
        debug!(
            "Batch inserting {} texts into collection '{}'",
            texts.len(),
            collection
        );

        use crate::models::Vector;

        let mut vector_objects = Vec::new();
        let mut errors = Vec::new();

        for (id, text, metadata) in texts {
            // Generate embedding for the text
            let embedding_data = embedding_manager
                .embed(&text)
                .map_err(|e| {
                    let error_msg = format!("Failed to generate embedding for text {}: {}", id, e);
                    errors.push(error_msg.clone());
                    VectorizerError::Other(error_msg)
                })?;

            // Create rich payload with content and metadata
            let mut payload_data = serde_json::Map::new();
            payload_data.insert(
                "content".to_string(),
                serde_json::Value::String(text.clone()),
            );

            // Add custom metadata if provided
            if let Some(metadata) = metadata {
                if let serde_json::Value::Object(meta_obj) = metadata {
                    for (key, value) in meta_obj {
                        payload_data.insert(key, value);
                    }
                }
            }

            // Add batch operation metadata
            payload_data.insert(
                "operation_type".to_string(),
                serde_json::Value::String("batch_insert_texts".to_string()),
            );
            payload_data.insert(
                "created_at".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
            );

            let rich_payload = crate::models::Payload {
                data: serde_json::Value::Object(payload_data),
            };

            vector_objects.push(Vector::with_payload(id, embedding_data, rich_payload));
        }

        let inserted_count = vector_objects.len();
        
        if !vector_objects.is_empty() {
            vector_store.insert(collection, vector_objects)
                .map_err(|e| VectorizerError::Other(format!("Failed to insert vectors: {}", e)))?;
        }

        let status = if errors.is_empty() { "success" } else if inserted_count > 0 { "partial_success" } else { "error" };
        let message = if errors.is_empty() {
            format!("Successfully batch inserted {} texts", inserted_count)
        } else {
            format!("Batch inserted {} texts, {} errors: {}", inserted_count, errors.len(), errors.join(", "))
        };

        Ok(serde_json::json!({
            "success": true,
            "collection": collection,
            "operation": "batch_insert_texts",
            "total_operations": inserted_count,
            "successful_operations": inserted_count,
            "failed_operations": errors.len(),
            "errors": errors,
            "status": status,
            "message": message
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

    /// Get detailed memory analysis for all collections
    pub async fn get_memory_analysis(vector_store: &VectorStore) -> Result<serde_json::Value> {
        debug!("Getting detailed memory analysis via MCP");

        // Get collections list
        let collections = vector_store.list_collections();

        let mut analysis = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "collections": [],
            "summary": {}
        });

        let mut total_theoretical_memory = 0;
        let mut total_actual_memory = 0;
        let mut collections_with_quantization = 0;
        let mut collections_without_quantization = 0;

        // Analyze each collection individually
        for collection_name in &collections {
            let metadata = vector_store.get_collection_metadata(collection_name)?;

            let vector_count = metadata.vector_count;
            let dimension = metadata.config.dimension;

            // Calculate theoretical memory usage (f32 vectors)
            let theoretical_memory = vector_count * dimension * 4; // 4 bytes per f32

            // Try to get actual memory usage from collection
            let actual_memory = match vector_store.get_collection(collection_name) {
                Ok(collection_ref) => {
                    let estimated_memory = (*collection_ref).estimated_memory_usage();
                    let quantization_enabled = matches!((*collection_ref).config().quantization, crate::models::QuantizationConfig::SQ { bits: 8 });

                    if quantization_enabled {
                        collections_with_quantization += 1;
                    } else {
                        collections_without_quantization += 1;
                    }

                    (estimated_memory, quantization_enabled)
                },
                Err(_) => {
                    // Fallback to theoretical if can't access collection
                    (theoretical_memory, false)
                }
            };

            let (actual_memory_bytes, quantization_enabled) = actual_memory;
            let compression_ratio = if theoretical_memory > 0 {
                actual_memory_bytes as f64 / theoretical_memory as f64
            } else { 1.0 };

            let memory_savings_percent = if theoretical_memory > 0 {
                (1.0 - compression_ratio) * 100.0
            } else { 0.0 };

            let quantization_status = if quantization_enabled {
                if compression_ratio < 0.3 { "4x compression (SQ-8bit)" }
                else if compression_ratio < 0.6 { "2x compression" }
                else if compression_ratio < 0.8 { "Partial compression" }
                else { "Quantization enabled but not effective" }
            } else {
                "No quantization"
            };

            let collection_analysis = serde_json::json!({
                "name": collection_name,
                "dimension": dimension,
                "vector_count": vector_count,
                "document_count": metadata.document_count,
                "embedding_provider": "bm25", // Default for now
                "metric": "cosine", // Default for now
                "created_at": metadata.created_at.to_rfc3339(),
                "updated_at": metadata.updated_at.to_rfc3339(),
                "indexing_status": {
                    "collection_name": collection_name,
                    "status": "ready",
                    "progress": 100.0,
                    "vector_count": vector_count,
                    "last_updated": metadata.updated_at.to_rfc3339()
                },
                "memory_analysis": {
                    "theoretical_memory_bytes": theoretical_memory,
                    "theoretical_memory_mb": theoretical_memory as f64 / (1024.0 * 1024.0),
                    "actual_memory_bytes": actual_memory_bytes,
                    "actual_memory_mb": actual_memory_bytes as f64 / (1024.0 * 1024.0),
                    "memory_saved_bytes": theoretical_memory.saturating_sub(actual_memory_bytes),
                    "memory_saved_mb": (theoretical_memory.saturating_sub(actual_memory_bytes)) as f64 / (1024.0 * 1024.0),
                    "compression_ratio": compression_ratio,
                    "memory_savings_percent": memory_savings_percent,
                    "memory_per_vector_bytes": if vector_count > 0 { actual_memory_bytes / vector_count } else { 0 },
                    "theoretical_memory_per_vector_bytes": dimension * 4
                },
                "quantization": {
                    "enabled": quantization_enabled,
                    "status": quantization_status,
                    "effective": compression_ratio < 0.8,
                    "compression_factor": if compression_ratio > 0.0 { 1.0 / compression_ratio } else { 1.0 }
                },
                "performance": {
                    "memory_efficiency": if compression_ratio < 0.3 { "Excellent" }
                    else if compression_ratio < 0.6 { "Good" }
                    else if compression_ratio < 0.8 { "Fair" }
                    else { "Poor" },
                    "recommendation": if quantization_enabled && compression_ratio >= 0.8 {
                        "Quantization enabled but not working - check implementation"
                    } else if !quantization_enabled && vector_count > 1000 {
                        "Enable quantization for memory savings"
                    } else if quantization_enabled && compression_ratio < 0.3 {
                        "Excellent quantization performance"
                    } else {
                        "No action needed"
                    }
                }
            });

            analysis["collections"].as_array_mut().unwrap().push(collection_analysis);

            total_theoretical_memory += theoretical_memory;
            total_actual_memory += actual_memory_bytes;
        }

        // Calculate overall summary
        let overall_compression_ratio = if total_theoretical_memory > 0 {
            total_actual_memory as f64 / total_theoretical_memory as f64
        } else { 1.0 };

        let overall_memory_savings = if total_theoretical_memory > 0 {
            (1.0 - overall_compression_ratio) * 100.0
        } else { 0.0 };

        analysis["summary"] = serde_json::json!({
            "total_collections": collections.len(),
            "collections_with_quantization": collections_with_quantization,
            "collections_without_quantization": collections_without_quantization,
            "total_vectors": collections.iter().filter_map(|name| vector_store.get_collection_metadata(name).ok()).map(|m| m.vector_count).sum::<usize>(),
            "total_documents": collections.iter().filter_map(|name| vector_store.get_collection_metadata(name).ok()).map(|m| m.document_count).sum::<usize>(),
            "memory_analysis": {
                "total_theoretical_memory_bytes": total_theoretical_memory,
                "total_theoretical_memory_mb": total_theoretical_memory as f64 / (1024.0 * 1024.0),
                "total_actual_memory_bytes": total_actual_memory,
                "total_actual_memory_mb": total_actual_memory as f64 / (1024.0 * 1024.0),
                "total_memory_saved_bytes": total_theoretical_memory.saturating_sub(total_actual_memory),
                "total_memory_saved_mb": (total_theoretical_memory.saturating_sub(total_actual_memory)) as f64 / (1024.0 * 1024.0),
                "overall_compression_ratio": overall_compression_ratio,
                "overall_memory_savings_percent": overall_memory_savings,
                "average_memory_per_vector_bytes": if collections.iter().filter_map(|name| vector_store.get_collection_metadata(name).ok()).map(|m| m.vector_count).sum::<usize>() > 0 {
                    total_actual_memory / collections.iter().filter_map(|name| vector_store.get_collection_metadata(name).ok()).map(|m| m.vector_count as usize).sum::<usize>() as usize
                } else { 0 }
            },
            "quantization_summary": {
                "quantization_coverage_percent": if collections.len() > 0 {
                    (collections_with_quantization as f64 / collections.len() as f64) * 100.0
                } else { 0.0 },
                "overall_quantization_status": if overall_compression_ratio < 0.3 { "4x compression achieved" }
                else if overall_compression_ratio < 0.6 { "2x compression achieved" }
                else if overall_compression_ratio < 0.8 { "Partial compression" }
                else { "Quantization not effective" },
                "recommendation": if overall_compression_ratio >= 0.8 {
                    "Enable quantization on more collections for better memory efficiency"
                } else if overall_compression_ratio < 0.3 {
                    "Excellent quantization performance across all collections"
                } else {
                    "Good quantization performance"
                }
            }
        });

        info!("Detailed memory analysis complete via MCP: {} collections analyzed, {}MB actual vs {}MB theoretical",
              collections.len(),
              total_actual_memory as f64 / (1024.0 * 1024.0),
              total_theoretical_memory as f64 / (1024.0 * 1024.0));

        Ok(analysis)
    }

    /// Requantize all vectors in a collection for memory optimization
    pub async fn requantize_collection(collection_name: &str, vector_store: &VectorStore) -> Result<serde_json::Value> {
        info!("Requantizing collection '{}' via MCP", collection_name);

        // Get the collection
        let collection = match vector_store.get_collection(collection_name) {
            Ok(coll) => coll,
            Err(_) => {
                return Ok(serde_json::json!({
                    "collection_name": collection_name,
                    "success": false,
                    "message": format!("Collection '{}' not found", collection_name),
                    "status": "not_found"
                }));
            }
        };

        // Check if quantization is enabled for this collection
        let quantization_enabled = matches!(collection.config().quantization, crate::models::QuantizationConfig::SQ { bits: 8 });

        if !quantization_enabled {
            return Ok(serde_json::json!({
                "collection_name": collection_name,
                "success": false,
                "message": format!("Quantization not enabled for collection '{}'", collection_name),
                "status": "quantization_disabled"
            }));
        }

        // Perform requantization
        match collection.requantize_existing_vectors() {
            Ok(_) => {
                info!("✅ Successfully requantized collection '{}' via MCP", collection_name);
                Ok(serde_json::json!({
                    "collection_name": collection_name,
                    "success": true,
                    "message": "Collection requantized successfully",
                    "status": "success"
                }))
            },
            Err(e) => {
                warn!("⚠️ Failed to requantize collection '{}' via MCP: {}", collection_name, e);
                Ok(serde_json::json!({
                    "collection_name": collection_name,
                    "success": false,
                    "message": format!("Failed to requantize collection: {}", e),
                    "status": "error"
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::TfIdfEmbedding;

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

        let result = McpTools::embed_text("test text", &embedding_manager)
            .await
            .unwrap();

        assert!(result.get("embedding").is_some());
        assert_eq!(result.get("text").unwrap(), "test text");
        assert_eq!(result.get("dimension").unwrap(), 64);
    }
}
