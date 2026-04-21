//! Cache load, in-memory population, and memory-usage accounting.
//!
//! This module bridges the on-disk `.vecdb`/`.bin` cache format and the
//! in-memory [`Collection`] state. It also owns the memory-usage
//! introspection used by `/size-info` endpoints and admin tooling —
//! accounting differs meaningfully between quantized and
//! full-precision storage, so it lives here rather than in [`data`].
//!
//! [`data`]: super::data

// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use tracing::{debug, info};

use super::Collection;
use crate::error::Result;
use crate::models::Vector;

impl Collection {
    /// Estimate memory usage in bytes with quantization support
    pub fn estimated_memory_usage(&self) -> usize {
        let vector_count = self.vectors.len();
        let dimension = self.config.dimension;

        // Check if quantization is enabled in config
        let quantization_enabled = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        );

        if quantization_enabled {
            // Calculate memory usage for quantized vectors (4x compression with SQ-8bit)
            let mut total_memory = 0;
            let mut quantized_vectors = 0;
            let vector_order = self.vector_order.read();

            for id in vector_order.iter() {
                if let Ok(Some(vector)) = self.vectors.get(id) {
                    // Base overhead for Vector struct
                    total_memory += std::mem::size_of::<Vector>();

                    // Check if vector is quantized (data cleared)
                    let is_quantized = vector.data.is_empty();

                    if is_quantized {
                        // Vector is quantized - minimal memory usage
                        total_memory += dimension; // 1 byte per dimension for quantized data
                        quantized_vectors += 1;
                    } else {
                        // Vector not yet quantized - use f32 data size
                        total_memory += std::mem::size_of::<f32>() * dimension;
                    }

                    // Payload overhead
                    if let Some(payload) = &vector.payload {
                        total_memory += std::mem::size_of_val(payload);
                    }
                }
            }

            // Debug logging for memory analysis
            if vector_count > 0 {
                let quantization_ratio = quantized_vectors as f32 / vector_count as f32;
                debug!(
                    "🔍 [MEMORY ANALYSIS] Collection '{}': {}/{} vectors quantized ({:.1}%), total_memory: {} bytes",
                    self.name,
                    quantized_vectors,
                    vector_count,
                    quantization_ratio * 100.0,
                    total_memory
                );
            }

            total_memory
        } else {
            // Standard memory usage without quantization
            let vector_size = std::mem::size_of::<f32>() * dimension;
            let entry_overhead = std::mem::size_of::<String>() + std::mem::size_of::<Vector>();
            let total_per_vector = vector_size + entry_overhead;

            vector_count * total_per_vector
        }
    }

    /// Fast load from cache without building HNSW index (index built lazily on first search)
    pub fn load_from_cache(
        &self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
    ) -> Result<()> {
        debug!(
            "Fast loading {} vectors into collection '{}' (lazy index)",
            persisted_vectors.len(),
            self.name
        );

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        debug!("Loaded {} vectors from cache", runtime_vectors.len());

        // Use fast load for runtime vectors
        self.fast_load_vectors(runtime_vectors)?;

        // Apply quantization automatically after loading if enabled
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        ) {
            debug!(
                "Applying automatic quantization to loaded vectors in collection '{}'",
                self.name
            );
            self.requantize_existing_vectors()?;
        }

        debug!(
            "Fast loaded {} vectors into collection '{}' with HNSW index",
            self.vectors.len(),
            self.name
        );
        Ok(())
    }

    pub fn load_from_cache_with_hnsw_dump(
        &self,
        persisted_vectors: Vec<crate::persistence::PersistedVector>,
        hnsw_dump_path: Option<&std::path::Path>,
        hnsw_basename: Option<&str>,
    ) -> Result<()> {
        info!(
            "🚀 [CACHE LOAD] Loading {} vectors into collection '{}' from cache (HNSW dump path: {:?})",
            persisted_vectors.len(),
            self.name,
            hnsw_dump_path
        );

        // Try to load HNSW index from dump first
        let hnsw_loaded = if let (Some(path), Some(basename)) = (hnsw_dump_path, hnsw_basename) {
            match self.load_hnsw_index_from_dump(path, basename) {
                Ok(_) => {
                    info!(
                        "Successfully loaded HNSW index from dump for collection '{}'",
                        self.name
                    );
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        };

        // Convert persisted vectors to runtime vectors
        let mut runtime_vectors = Vec::with_capacity(persisted_vectors.len());
        for pv in persisted_vectors {
            runtime_vectors.push(pv.into_runtime_with_payload()?);
        }

        // IMPORTANT: Do NOT apply quantization here - it will clear vector data
        // and prevent proper re-persistence. Quantization should only be applied
        // in search operations, not in storage.
        // The original code was clearing vector.data after loading from cache,
        // which caused re-saved .bin files to be empty.

        debug!(
            "Loaded {} vectors without applying quantization (preserving data for persistence)",
            runtime_vectors.len()
        );

        if hnsw_loaded {
            // HNSW index already loaded, just load vectors into memory
            debug!(
                "Loading {} vectors into memory (HNSW index already loaded)",
                runtime_vectors.len()
            );
            self.load_vectors_into_memory(runtime_vectors)?;
        } else {
            // Build HNSW index from scratch
            debug!("Building HNSW index from {} vectors", runtime_vectors.len());
            self.fast_load_vectors(runtime_vectors)?;
        }

        debug!(
            "Loaded {} vectors into collection '{}' {}",
            self.vectors.len(),
            self.name,
            if hnsw_loaded {
                "(from dump)"
            } else {
                "(rebuilt)"
            }
        );
        Ok(())
    }

    /// Load vectors into memory without building HNSW index (assumes index is already loaded)
    pub fn load_vectors_into_memory(&self, vectors: Vec<Vector>) -> Result<()> {
        let vectors_len = vectors.len();
        let mut vector_order = self.vector_order.write();

        // Check if graph is enabled and should create nodes
        // Graph is enabled if it exists (regardless of config, since it can be enabled manually)
        let should_create_graph_nodes = self.graph.is_some();

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

            // Store vector
            self.vectors.insert(id.clone(), vector.clone())?;

            // Create graph node if graph is enabled
            if should_create_graph_nodes {
                if let Some(graph) = &self.graph {
                    let node = crate::db::graph::Node::from_vector(&id, vector.payload.as_ref());
                    if let Err(e) = graph.add_node(node) {
                        debug!(
                            "Failed to add graph node for vector '{}' during load: {}",
                            id, e
                        );
                    }
                }
            }

            // Track insertion order
            vector_order.push(id.clone());
        }

        // Update vector count
        *self.vector_count.write() += vectors_len;

        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();

        if should_create_graph_nodes {
            info!(
                "Loaded {} vectors into memory for collection '{}' and created graph nodes",
                vector_order.len(),
                self.name
            );
        } else {
            debug!(
                "Loaded {} vectors into memory for collection '{}'",
                vector_order.len(),
                self.name
            );
        }
        Ok(())
    }

    /// Get all vectors in the collection (for persistence)
    /// Returns vectors in insertion order to maintain HNSW index consistency
    pub fn get_all_vectors(&self) -> Vec<Vector> {
        let vector_order = self.vector_order.read();

        // If quantization is enabled, get from quantized storage
        if matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
                | crate::models::QuantizationConfig::Binary
        ) {
            let quantized = self.quantized_vectors.lock();
            vector_order
                .iter()
                .filter_map(|id| quantized.get(id).map(|qv| qv.to_vector()))
                .collect()
        } else {
            // Get from full precision storage
            vector_order
                .iter()
                .filter_map(|id| self.vectors.get(id).ok().flatten())
                .collect()
        }
    }

    /// Calculate approximate memory usage of the collection
    pub fn calculate_memory_usage(&self) -> (usize, usize, usize) {
        let mut index_size = 0;
        let mut payload_size = 0;
        let mut total_size = 0;

        // Check if quantization is enabled
        let use_quantization = matches!(
            self.config.quantization,
            crate::models::QuantizationConfig::SQ { bits: 8 }
        );

        if use_quantization {
            // Calculate from quantized storage
            let quantized_vectors = self.quantized_vectors.lock();
            let vector_count = quantized_vectors.len();

            for (id, qvector) in quantized_vectors.iter() {
                // Vector ID size
                let id_size = id.len();

                // Quantized vector data size (u8 = 1 byte per element)
                let vector_data_size = qvector.quantized_data.len();

                // Quantization params (2 f32 values)
                let quant_params_size = std::mem::size_of::<f32>() * 2;

                // Payload size (approximate JSON serialization size)
                let vector_payload_size = if let Some(ref payload) = qvector.payload {
                    match serde_json::to_string(&payload.data) {
                        Ok(json_str) => json_str.len(),
                        Err(_) => 0,
                    }
                } else {
                    0
                };

                // Total size for this quantized vector
                let vector_total_size =
                    id_size + vector_data_size + quant_params_size + vector_payload_size;

                index_size += id_size + vector_data_size + quant_params_size;
                payload_size += vector_payload_size;
                total_size += vector_total_size;
            }

            // Add HNSW index overhead (approximate)
            let index_overhead = vector_count * 100;
            index_size += index_overhead;
            total_size += index_overhead;
        } else {
            // Calculate from full precision storage
            let vector_count = self.vectors.len();
            let vector_order = self.vector_order.read();

            for id in vector_order.iter() {
                if let Ok(Some(vector)) = self.vectors.get(id) {
                    // Vector ID size
                    let id_size = id.len();

                    // Vector data size (f32 = 4 bytes per element)
                    let vector_data_size = vector.data.len() * 4;

                    // Payload size (approximate JSON serialization size)
                    let vector_payload_size = if let Some(ref payload) = vector.payload {
                        match serde_json::to_string(&payload.data) {
                            Ok(json_str) => json_str.len(),
                            Err(_) => 0,
                        }
                    } else {
                        0
                    };

                    // Total size for this vector
                    let vector_total_size = id_size + vector_data_size + vector_payload_size;

                    index_size += id_size + vector_data_size;
                    payload_size += vector_payload_size;
                    total_size += vector_total_size;
                }
            }

            // Add HNSW index overhead (approximate)
            let index_overhead = vector_count * 100;
            index_size += index_overhead;
            total_size += index_overhead;
        }

        (index_size, payload_size, total_size)
    }

    /// Get collection size information in a formatted way
    pub fn get_size_info(&self) -> (String, String, String) {
        let (index_size, payload_size, total_size) = self.calculate_memory_usage();

        let format_bytes = |bytes: usize| -> String {
            if bytes >= 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
            } else if bytes >= 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{} B", bytes)
            }
        };

        (
            format_bytes(index_size),
            format_bytes(payload_size),
            format_bytes(total_size),
        )
    }
}
