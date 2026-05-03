//! HNSW index operations — fast batch load, dump/load to disk,
//! cache directory integration, and online reindex.
//!
//! The index itself is built incrementally via [`Collection::insert_batch`]
//! (in [`data`]); this module owns the bulk-load, persistence, and reindex
//! paths that reconstitute the index from the `.vecdb` cache or flush it
//! back out to disk.
//!
//! [`data`]: super::data

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{debug, info, warn};

use super::Collection;
use crate::db::optimized_hnsw::{OptimizedHnswConfig, OptimizedHnswIndex};
use crate::error::{Result, VectorizerError};
use crate::models::{HnswConfig, Vector};

impl Collection {
    /// Fast load vectors with HNSW index building
    pub fn fast_load_vectors(&self, vectors: Vec<Vector>) -> Result<()> {
        let vectors_len = vectors.len();
        debug!(
            "Fast loading {} vectors into collection '{}' with HNSW index",
            vectors_len, self.name
        );

        let mut vector_order = self.vector_order.write();
        let index = self.index.write();

        // Check if graph is enabled and should create nodes
        // Graph is enabled if it exists (regardless of config, since it can be enabled manually)
        let should_create_graph_nodes = self.graph.is_some();

        // Prepare vectors for batch insertion
        let mut batch_vectors = Vec::with_capacity(vectors_len);

        for vector in vectors {
            let id = vector.id.clone();

            // Extract document ID from payload for tracking unique documents
            if let Some(payload) = &vector.payload {
                if let Some(file_path) = payload.data.get("file_path") {
                    if let Some(file_path_str) = file_path.as_str() {
                        self.document_ids.insert(file_path_str.to_string(), ());
                    }
                }
            }

            // Vector is already normalized by into_runtime_with_payload if needed

            // CRITICAL FIX: Apply quantization if enabled (same as insert_batch does)
            // This ensures vectors are stored consistently whether loaded from disk or inserted fresh
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
                debug!("Storing quantized vector '{}' during fast load", id);
                self.quantized_vectors
                    .lock()
                    .insert(id.clone(), quantized_vector);

                // Don't store full precision vector to save memory
            } else {
                // Store full precision vector
                self.vectors.insert(id.clone(), vector.clone())?;
            }

            // Create graph node if graph is enabled
            if should_create_graph_nodes {
                if let Some(graph) = &self.graph {
                    let node = crate::db::graph::Node::from_vector(&id, vector.payload.as_ref());
                    if let Err(e) = graph.add_node(node) {
                        debug!(
                            "Failed to add graph node for vector '{}' during fast load: {}",
                            id, e
                        );
                    }
                }
            }

            // Add to batch for HNSW index (using full precision for search accuracy)
            batch_vectors.push((id.clone(), vector.data.clone()));

            // Track insertion order
            vector_order.push(id.clone());
        }

        // Batch insert into HNSW index
        index.batch_add(batch_vectors)?;

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        debug!(
            "Fast loaded {} vectors into collection '{}' with HNSW index",
            vector_order.len(),
            self.name
        );
        Ok(())
    }

    /// Rebuild the HNSW index with new parameters from existing stored vectors.
    ///
    /// # Durability
    ///
    /// Takes the `vector_order` write-lock for the duration of the rebuild so
    /// that concurrent inserts queue behind this operation — identical to the
    /// reencode pattern in phase13. The swap is atomic from the caller's
    /// perspective: either the entire new index is in place or the old one is
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if the collection is empty (nothing to reindex) or if
    /// the underlying HNSW index construction fails.
    pub fn reindex_with_params(&self, new_params: HnswConfig) -> Result<()> {
        // Take the write lock to serialise concurrent writes during the swap.
        let vector_order = self.vector_order.write();

        let vector_count = vector_order.len();
        if vector_count == 0 {
            return Err(VectorizerError::Storage(format!(
                "collection '{}' is empty; nothing to reindex",
                self.name
            )));
        }

        info!(
            "reindex_with_params '{}': rebuilding HNSW (M={}, ef_construction={}) for {} vectors",
            self.name, new_params.m, new_params.ef_construction, vector_count
        );

        let new_hnsw_cfg = OptimizedHnswConfig {
            max_connections: new_params.m,
            max_connections_0: new_params.m * 2,
            ef_construction: new_params.ef_construction,
            seed: new_params.seed,
            distance_metric: self.config.metric,
            parallel: true,
            initial_capacity: vector_count.max(1_024),
            batch_size: 1_000,
        };

        // Build the new index offline.
        let new_index = OptimizedHnswIndex::new(self.config.dimension, new_hnsw_cfg)
            .map_err(|e| VectorizerError::Storage(format!("failed to create new HNSW: {}", e)))?;

        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        );

        // Collect (id, raw_f32_data) pairs from existing storage.
        let mut batch: Vec<(String, Vec<f32>)> = Vec::with_capacity(vector_count);

        for id in vector_order.iter() {
            let data = if use_quantization {
                if let Some(qv) = self.quantized_vectors.lock().get(id) {
                    qv.to_vector().data
                } else {
                    continue;
                }
            } else {
                match self.vectors.get(id) {
                    Ok(Some(v)) => v.data,
                    _ => continue,
                }
            };
            batch.push((id.clone(), data));
        }

        new_index.batch_add(batch)?;

        // Atomic swap: replace the live index with the new one.
        *self.index.write() = new_index;

        info!(
            "reindex_with_params '{}': completed ({} vectors indexed with M={})",
            self.name, vector_count, new_params.m
        );
        Ok(())
    }

    /// Dump the HNSW index to files for faster reloading
    pub fn dump_hnsw_index<P: AsRef<std::path::Path>>(&self, path: P) -> Result<String> {
        let basename = format!("{}_hnsw", self.name);
        (*self.index.write()).file_dump(path, &basename)?;
        Ok(basename)
    }

    /// Load HNSW index from dump files
    pub fn load_hnsw_index_from_dump<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        basename: &str,
    ) -> Result<()> {
        (*self.index.write()).load_from_dump(path, basename)
    }

    /// Dump HNSW index to centralized cache directory for faster future loading
    pub fn dump_hnsw_index_for_cache<P: AsRef<std::path::Path>>(
        &self,
        _project_path: P,
    ) -> Result<()> {
        // Get the vectorizer root directory (where config.yml is located)
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cache_dir = current_dir.join("data");

        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir)?;
        }

        let basename = format!("{}_hnsw", self.name);

        // Check if index has vectors
        let index_len = (*self.index.read()).len();

        if index_len == 0 {
            warn!(
                "⚠️ COLLECTION DUMP WARNING: Index is empty for collection '{}'",
                self.name
            );
            return Err(VectorizerError::IndexError(format!(
                "Index is empty for collection '{}'",
                self.name
            )));
        }

        (*self.index.write()).file_dump(&cache_dir, &basename)?;
        info!(
            "✅ Successfully dumped HNSW index for collection '{}' to centralized cache",
            self.name
        );
        Ok(())
    }
}
