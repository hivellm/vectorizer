//! Data plane — insert, update, delete, get, and search operations.
//!
//! These methods form the primary CRUD + read surface of the
//! [`Collection`]. They coordinate the vector storage backend, the
//! HNSW index, the payload index, and the sparse vector index, as
//! well as graph relationship discovery during insertion.

use tracing::{debug, info, warn};

use super::Collection;
use crate::db::hybrid_search::{
    DenseSearchResult, HybridSearchConfig, SparseSearchResult, hybrid_search,
};
use crate::error::{Result, VectorizerError};
use crate::models::{DistanceMetric, SearchResult, SparseVector, Vector, vector_utils};

impl Collection {
    /// Insert a batch of vectors
    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()> {
        // Validate dimensions
        for vector in &vectors {
            if vector.dimension() != self.config.dimension {
                return Err(VectorizerError::InvalidDimension {
                    expected: self.config.dimension,
                    got: vector.dimension(),
                });
            }
        }

        // Validate encryption requirements
        if let Some(encryption_config) = &self.config.encryption {
            if encryption_config.required {
                // All payloads must be encrypted
                for vector in &vectors {
                    if let Some(payload) = &vector.payload {
                        if !payload.is_encrypted() {
                            return Err(VectorizerError::EncryptionRequired(
                                "Collection requires encrypted payloads, but received unencrypted payload".to_string()
                            ));
                        }
                    }
                }
            } else if !encryption_config.allow_mixed {
                // Cannot mix encrypted and unencrypted
                let has_encrypted = vectors
                    .iter()
                    .any(|v| v.payload.as_ref().map_or(false, |p| p.is_encrypted()));
                let has_unencrypted = vectors
                    .iter()
                    .any(|v| v.payload.as_ref().map_or(true, |p| !p.is_encrypted()));
                if has_encrypted && has_unencrypted {
                    return Err(VectorizerError::EncryptionRequired(
                        "Collection does not allow mixed encrypted and unencrypted payloads"
                            .to_string(),
                    ));
                }
            }
        }

        // Insert vectors and update index
        let index = self.index.write();
        let mut vector_order = self.vector_order.write();
        // Track only NEW IDs so `vector_count` stays in sync with the
        // underlying storage (both `self.vectors` and `self.quantized_vectors`
        // replace on duplicate key). Counting every batch element would make
        // the counter diverge from `self.vectors.len()` under replay, e.g.
        // when a replica receives overlapping snapshot + streamed ops.
        let is_quantized = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        );
        let mut new_inserts: usize = 0;
        for mut vector in vectors {
            let id = vector.id.clone();
            let mut data = vector.data.clone();

            let is_new = if is_quantized {
                !self.quantized_vectors.lock().contains_key(&id)
            } else {
                !self.vectors.contains_key(&id)?
            };

            // Normalize vector for cosine similarity
            if matches!(self.config.metric, DistanceMetric::Cosine) {
                data = vector_utils::normalize_vector(&data);
                vector.data = data.clone(); // Update stored vector to normalized version
                // If sparse representation exists, update it to reflect normalized values
                if let Some(ref sparse) = vector.sparse {
                    // Recreate sparse from normalized dense vector
                    let normalized_sparse = SparseVector::from_dense(&data);
                    vector.sparse = Some(normalized_sparse);
                }
            }

            // Extract document ID from payload for tracking unique documents
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }

                // Index payload for efficient filtering
                self.payload_index.index_vector(id.clone(), payload);
            }

            // Index sparse vector if available
            if let Some(ref sparse) = vector.sparse {
                let mut sparse_idx = self.sparse_index.write();
                if let Err(e) = sparse_idx.add(id.clone(), sparse.clone()) {
                    warn!("Failed to index sparse vector '{}': {}", id, e);
                }
            }

            // Apply quantization if enabled - store in quantized format to save memory
            if matches!(
                self.config.quantization,
                crate::models::QuantizationConfig::SQ { bits: 8 }
                    | crate::models::QuantizationConfig::Binary
            ) {
                // Store as quantized vector (75% memory reduction for SQ-8bit, 96% for Binary)
                let quantized_vector = crate::models::QuantizedVector::from_vector(
                    vector.clone(),
                    &self.config.quantization,
                );
                debug!(
                    "Storing quantized vector '{}' ({} bytes instead of {})",
                    id,
                    quantized_vector.memory_size(),
                    data.len() * 4
                );
                self.quantized_vectors
                    .lock()
                    .insert(id.clone(), quantized_vector);

                // Don't store full precision vector to save memory
                // It will be reconstructed on-demand from quantized version
            } else {
                // Store full precision vector
                self.vectors.insert(id.clone(), vector.clone())?;
            }

            // Track insertion order for persistence consistency.
            // Only new IDs contribute to order and to `vector_count` — see
            // the top-of-loop note on replay idempotence.
            if is_new {
                vector_order.push(id.clone());
                new_inserts += 1;
            }

            // Add to index (using full precision for search accuracy)
            index.add(id.clone(), data.clone())?;

            // Discover graph relationships if graph is enabled
            // Note: Relationship discovery is done synchronously but with limited search scope
            // to avoid timeout during insertion. For large collections, consider disabling
            // auto_relationship.enabled_types or running relationship discovery in background.
            if let Some(graph) = &self.graph {
                if let Some(graph_config) = &self.config.graph {
                    if graph_config.enabled {
                        // Only create node, skip expensive similarity search during insertion
                        // Similarity relationships can be created later via explicit edge creation
                        let node =
                            crate::db::graph::Node::from_vector(&id, vector.payload.as_ref());
                        if let Err(e) = graph.add_node(node) {
                            warn!("Failed to add graph node for vector '{}': {}", id, e);
                        }

                        // Optionally discover relationships if auto_relationship is enabled
                        // Note: Similarity-based relationships are skipped during insertion
                        // to avoid timeout. Only metadata-based relationships are created.
                        let auto_config = &graph_config.auto_relationship;
                        if !auto_config.enabled_types.is_empty() {
                            // Ensure the source node exists before creating relationships
                            // (it should already exist from line 321-324, but double-check)
                            if graph.get_node(&id).is_none() {
                                let node = crate::db::graph::Node::from_vector(
                                    &id,
                                    vector.payload.as_ref(),
                                );
                                let _ = graph.add_node(node);
                            }

                            // Only do fast metadata-based relationships during insertion
                            // Skip SIMILAR_TO to avoid timeout during insertion
                            if let Some(payload) = &vector.payload {
                                use crate::db::graph_relationship_discovery::{
                                    discover_contains_relationships,
                                    discover_derived_from_relationships,
                                    discover_reference_relationships, is_relationship_type_enabled,
                                };

                                // Create metadata-based relationships (fast)
                                if is_relationship_type_enabled("REFERENCES", auto_config) {
                                    if let Err(e) =
                                        discover_reference_relationships(graph, &id, payload)
                                    {
                                        debug!("Failed to discover REFERENCES for '{}': {}", id, e);
                                    }
                                }
                                if is_relationship_type_enabled("CONTAINS", auto_config) {
                                    if let Err(e) =
                                        discover_contains_relationships(graph, &id, payload)
                                    {
                                        debug!("Failed to discover CONTAINS for '{}': {}", id, e);
                                    }
                                }
                                if is_relationship_type_enabled("DERIVED_FROM", auto_config) {
                                    if let Err(e) =
                                        discover_derived_from_relationships(graph, &id, payload)
                                    {
                                        debug!(
                                            "Failed to discover DERIVED_FROM for '{}': {}",
                                            id, e
                                        );
                                    }
                                }
                                // SIMILAR_TO relationships are skipped during insertion to avoid timeout
                                // They can be created later via explicit edge creation
                            }
                        }
                    }
                }
            }
        }

        // Update vector count — only advance by IDs that were genuinely new.
        *self.vector_count.write() += new_inserts;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        // Train PQ if enabled and enough vectors collected
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::PQ { .. }
        ) {
            let count = *self.vector_count.read();
            // Train when we reach 1000 vectors (good balance between quality and startup time)
            if count >= 1000 && count < 1000 + new_inserts {
                debug!("Auto-training PQ with {} vectors", count);
                let _ = self.train_pq_if_needed();
            }
        }

        Ok(())
    }

    /// Insert a single vector
    pub fn insert(&self, vector: Vector) -> Result<()> {
        self.insert_batch(vec![vector])
    }

    /// Update a vector
    pub fn update(&self, mut vector: Vector) -> Result<()> {
        // Validate dimension
        if vector.dimension() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: vector.dimension(),
            });
        }

        let id = vector.id.clone();
        let mut data = vector.data.clone();

        // Check if vector exists (check both quantized and full precision storage)
        let vector_exists = if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        ) {
            self.quantized_vectors.lock().contains_key(&id)
        } else {
            self.vectors.contains_key(&id)?
        };

        if !vector_exists {
            return Err(VectorizerError::VectorNotFound(id));
        }

        // Normalize vector for cosine similarity
        if matches!(self.config.metric, DistanceMetric::Cosine) {
            data = vector_utils::normalize_vector(&data);
            vector.data = data.clone(); // Update stored vector to normalized version
        }

        // Extract document ID from payload for tracking unique documents
        if let Some(payload) = &vector.payload {
            if let Some(file_path) = payload.data.get("file_path") {
                if let Some(file_path_str) = file_path.as_str() {
                    self.document_ids.insert(file_path_str.to_string(), ());
                }
            }

            // Update payload index
            self.payload_index.remove_vector(&id);
            self.payload_index.index_vector(id.clone(), payload);
        }

        // Update sparse index
        {
            let mut sparse_idx = self.sparse_index.write();
            sparse_idx.remove(&id); // Remove old sparse vector if exists
            if let Some(ref sparse) = vector.sparse {
                if let Err(e) = sparse_idx.add(id.clone(), sparse.clone()) {
                    warn!("Failed to update sparse vector '{}': {}", id, e);
                }
            }
        }

        // Update vector storage (quantized or full precision)
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        ) {
            // Update quantized storage
            let quantized_vector = crate::models::QuantizedVector::from_vector(
                vector.clone(),
                &self.config.quantization,
            );
            self.quantized_vectors
                .lock()
                .insert(id.clone(), quantized_vector);
        } else {
            // Update full precision storage
            // For MMAP storage, we need to use update instead of insert
            if self.vectors.contains_key(&id)? {
                self.vectors.update(&id, vector)?;
            } else {
                self.vectors.insert(id.clone(), vector)?;
            }
        }

        // Update index
        let index = self.index.write();
        index.update(&id, &data)?;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Delete a vector
    pub fn delete(&self, vector_id: &str) -> Result<()> {
        // Remove from payload index
        self.payload_index.remove_vector(vector_id);

        // Remove from sparse index
        {
            let mut sparse_idx = self.sparse_index.write();
            sparse_idx.remove(vector_id);
        }

        // Remove from storage (both quantized and full precision)
        let found = if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        ) {
            self.quantized_vectors.lock().remove(vector_id).is_some()
        } else {
            self.vectors.remove(vector_id)?
        };

        if !found {
            return Err(VectorizerError::VectorNotFound(vector_id.to_string()));
        }

        // Remove from order tracking
        let mut vector_order = self.vector_order.write();
        vector_order.retain(|id| id != vector_id);

        // Remove from index
        let index = self.index.write();
        index.remove(vector_id)?;

        // Update vector count
        *self.vector_count.write() -= 1;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        // If quantization is enabled, get from quantized storage
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        ) {
            let quantized_vector = self
                .quantized_vectors
                .lock()
                .get(vector_id)
                .cloned()
                .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

            // Dequantize on-demand (only when needed for API response)
            let mut vector = quantized_vector.to_vector();

            // Normalize payload content (fix line endings from legacy data)
            if let Some(ref mut payload) = vector.payload {
                payload.normalize();
            }

            return Ok(vector);
        }

        // Otherwise get from full precision storage
        let vector = self
            .vectors
            .get(vector_id)?
            .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?;

        // Normalize payload content (fix line endings from legacy data)
        let mut normalized_vector = vector;
        if let Some(ref mut payload) = normalized_vector.payload {
            payload.normalize();
        }

        Ok(normalized_vector)
    }

    /// Search for similar vectors
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        // Validate dimension
        if query_vector.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_vector.len(),
            });
        }

        // Normalize query vector for cosine similarity
        let search_vector = if matches!(self.config.metric, DistanceMetric::Cosine) {
            vector_utils::normalize_vector(query_vector)
        } else {
            query_vector.to_vec()
        };

        // Search in index
        let index = self.index.read();
        let neighbors = index.search(&search_vector, k)?;

        // Build results - check quantized storage first if quantization is enabled
        let mut results = Vec::with_capacity(neighbors.len());
        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        );

        for (id, score) in neighbors {
            let vector = if use_quantization {
                // Get from quantized storage and dequantize on-demand
                if let Some(quantized) = self.quantized_vectors.lock().get(&id) {
                    quantized.to_vector()
                } else {
                    continue; // Vector not found
                }
            } else {
                // Get from full precision storage
                if let Ok(Some(v)) = self.vectors.get(&id) {
                    v
                } else {
                    continue; // Vector not found
                }
            };

            // Normalize payload content (fix line endings from legacy data)
            let normalized_payload = vector.payload.as_ref().map(|p| p.normalized());

            results.push(SearchResult {
                id: id.clone(),
                score,
                dense_score: Some(score), // Dense-only search
                sparse_score: None,
                vector: Some(vector.data.clone()),
                payload: normalized_payload,
            });
        }

        Ok(results)
    }

    /// Perform hybrid search combining dense (HNSW) and sparse vector search
    pub fn hybrid_search(
        &self,
        query_dense: &[f32],
        query_sparse: Option<&SparseVector>,
        config: HybridSearchConfig,
    ) -> Result<Vec<SearchResult>> {
        // Validate dense query dimension
        if query_dense.len() != self.config.dimension {
            return Err(VectorizerError::InvalidDimension {
                expected: self.config.dimension,
                got: query_dense.len(),
            });
        }

        info!(
            "Hybrid search in collection '{}': dense_k={}, sparse_k={}, final_k={}, alpha={}, algorithm={:?}",
            self.name,
            config.dense_k,
            config.sparse_k,
            config.final_k,
            config.alpha,
            config.algorithm
        );

        debug!(
            "Hybrid search query: dense_dim={}, sparse_query={:?}",
            query_dense.len(),
            query_sparse
                .as_ref()
                .map(|sv| format!("{} non-zero elements", sv.indices.len()))
        );

        // Perform dense search
        let dense_results: Vec<DenseSearchResult> = self
            .search(query_dense, config.dense_k)?
            .into_iter()
            .map(|r| DenseSearchResult {
                id: r.id,
                score: r.score,
            })
            .collect();

        let dense_count = dense_results.len();

        // Perform sparse search if query_sparse is provided
        let sparse_results: Vec<SparseSearchResult> = if let Some(query_sparse) = query_sparse {
            let sparse_idx = self.sparse_index.read();
            sparse_idx
                .search(query_sparse, config.sparse_k)
                .into_iter()
                .map(|(id, score)| SparseSearchResult { id, score })
                .collect()
        } else {
            Vec::new()
        };

        let sparse_count = sparse_results.len();

        debug!(
            "Hybrid search retrieved {} dense results and {} sparse results",
            dense_count, sparse_count
        );

        // Combine results using hybrid search algorithm
        let hybrid_results = hybrid_search(dense_results, sparse_results, &config)?;

        info!(
            "Hybrid search completed: {} combined results returned",
            hybrid_results.len()
        );

        // Convert to SearchResult format
        let mut results = Vec::with_capacity(hybrid_results.len());
        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        );

        for hybrid_result in hybrid_results {
            let vector = if use_quantization {
                if let Some(quantized) = self.quantized_vectors.lock().get(&hybrid_result.id) {
                    quantized.to_vector()
                } else {
                    continue;
                }
            } else {
                if let Ok(Some(v)) = self.vectors.get(&hybrid_result.id) {
                    v
                } else {
                    continue;
                }
            };

            let normalized_payload = vector.payload.as_ref().map(|p| p.normalized());

            results.push(SearchResult {
                id: hybrid_result.id.clone(),
                score: hybrid_result.hybrid_score,
                dense_score: hybrid_result.dense_score,
                sparse_score: hybrid_result.sparse_score,
                vector: Some(vector.data.clone()),
                payload: normalized_payload,
            });
        }

        info!(
            "Hybrid search completed: {} results (dense: {}, sparse: {})",
            results.len(),
            dense_count,
            sparse_count
        );

        Ok(results)
    }
}
