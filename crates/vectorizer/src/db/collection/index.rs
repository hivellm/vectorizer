//! HNSW index operations — fast batch load, dump/load to disk,
//! and cache directory integration.
//!
//! The index itself is built incrementally via [`Collection::insert_batch`]
//! (in [`data`]); this module owns the bulk-load and persistence paths
//! that reconstitute the index from the `.vecdb` cache or flush it
//! back out to disk.
//!
//! [`data`]: super::data

use tracing::{debug, info, warn};

use super::Collection;
use crate::error::{Result, VectorizerError};
use crate::models::Vector;

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
