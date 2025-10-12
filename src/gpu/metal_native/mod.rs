//! # Metal Native GPU Implementation
//!
//! This module provides high-performance vector storage and search using Metal GPU acceleration.
//! All operations are performed in VRAM for maximum efficiency.
//!
//! ## Architecture
//!
//! The Metal Native implementation consists of three main components:
//! - **VectorStorage**: Handles vector data storage in VRAM
//! - **HnswGraph**: GPU-accelerated HNSW index construction and search
//! - **Context**: Unified Metal device and command queue management
//!
//! ## Performance Characteristics
//!
//! - **Memory**: All data resides in VRAM (no CPU↔GPU copies during search)
//! - **Scaling**: O(1) vector addition with intelligent buffer growth
//! - **Search**: GPU-accelerated HNSW with configurable parameters
//!
//! ## Thread Safety
//!
//! Metal Native collections are not thread-safe by default. For concurrent access:
//!
//! ```rust
//! use std::sync::{Arc, Mutex};
//! use vectorizer::gpu::metal_native::MetalNativeCollection;
//! use vectorizer::models::DistanceMetric;
//!
//! let collection = MetalNativeCollection::new(512, DistanceMetric::Cosine).unwrap();
//! let collection = Arc::new(Mutex::new(collection));
//! ```
//!
//! ## Error Handling
//!
//! Common errors:
//! - `DimensionMismatch`: Vector dimension doesn't match collection
//! - `VramLimitExceeded`: Not enough VRAM for operation
//! - `MetalGpuError`: GPU operation failed

pub mod context;
pub mod vector_storage;
pub mod hnsw_graph;

// Re-export main types
pub use context::MetalNativeContext;
pub use vector_storage::MetalNativeVectorStorage;
pub use hnsw_graph::{MetalNativeHnswGraph, HnswConfig};

// Re-export from parent modules
pub use super::metal_buffer_pool::{MetalBufferPool, OptimizedMetalNativeCollection};
pub use super::vram_monitor::{VramMonitor, VramValidator};

use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::{DistanceMetric, Vector, CollectionConfig};
use tracing::{info, debug};

// Import HNSW types for search
use hnsw_graph::{HnswNode, SearchResult, GpuSearchQuery, GpuVectorMetadata, GpuSearchResult};

// Public constants following SCREAMING_SNAKE_CASE convention
/// Default VRAM limit for Metal Native collections (1GB)
pub const DEFAULT_VRAM_LIMIT_BYTES: usize = 1024 * 1024 * 1024;

/// Maximum buffer pool size for Metal Native
pub const MAX_BUFFER_POOL_SIZE: usize = 100;

/// Default growth factor for small buffers (< 1K vectors)
pub const DEFAULT_GROWTH_FACTOR_SMALL: f32 = 2.0;

/// Default growth factor for medium buffers (1K-10K vectors)
pub const DEFAULT_GROWTH_FACTOR_MEDIUM: f32 = 1.5;

/// Default growth factor for large buffers (> 10K vectors)
pub const DEFAULT_GROWTH_FACTOR_LARGE: f32 = 1.2;

// Global buffer pool for Metal Native collections
#[cfg(target_os = "macos")]
lazy_static::lazy_static! {
    static ref GLOBAL_BUFFER_POOL: std::sync::Mutex<GpuBufferPool> = {
        // This will be initialized when first accessed
        std::sync::Mutex::new(GpuBufferPool {
            device: Arc::new(MetalNativeContext::new().unwrap()),
            vector_buffers: Vec::new(),
            temp_buffers: Vec::new(),
            max_pooled_buffers: MAX_BUFFER_POOL_SIZE,
        })
    };
}

// GPU Buffer optimization structures
#[cfg(target_os = "macos")]
#[derive(Debug)]
struct GpuBufferPool {
    device: Arc<MetalNativeContext>,
    vector_buffers: Vec<metal::Buffer>,
    temp_buffers: Vec<metal::Buffer>,
    max_pooled_buffers: usize,
}

#[cfg(target_os = "macos")]
impl GpuBufferPool {
    fn new(device: Arc<MetalNativeContext>) -> Self {
        Self {
            device,
            vector_buffers: Vec::new(),
            temp_buffers: Vec::new(),
            max_pooled_buffers: MAX_BUFFER_POOL_SIZE,
        }
    }

    fn get_or_create_vector_buffer(&mut self, required_size: u64) -> metal::Buffer {
        // Try to find a suitable buffer in the pool
        for (i, buffer) in self.vector_buffers.iter().enumerate() {
            if buffer.length() >= required_size {
                let buffer = self.vector_buffers.swap_remove(i);
                debug!("🔄 Reusing pooled vector buffer: {} bytes", buffer.length());
                return buffer;
            }
        }

        // Create new buffer if none suitable found
        let buffer = self.device.device().new_buffer(
            required_size,
            metal::MTLResourceOptions::StorageModePrivate, // GPU-only for vectors
        );
        debug!("🆕 Created new vector buffer: {} bytes", required_size);
        buffer
    }

    fn get_or_create_temp_buffer(&mut self, required_size: u64) -> metal::Buffer {
        // Try to find a suitable buffer in the pool
        for (i, buffer) in self.temp_buffers.iter().enumerate() {
            if buffer.length() >= required_size {
                let buffer = self.temp_buffers.swap_remove(i);
                debug!("🔄 Reusing pooled temp buffer: {} bytes", buffer.length());
                return buffer;
            }
        }

        // Create new buffer if none suitable found
        let buffer = self.device.device().new_buffer(
            required_size,
            metal::MTLResourceOptions::StorageModeShared, // CPU↔GPU access for temp data
        );
        debug!("🆕 Created new temp buffer: {} bytes", required_size);
        buffer
    }

    fn return_vector_buffer(&mut self, buffer: metal::Buffer) {
        if self.vector_buffers.len() < MAX_BUFFER_POOL_SIZE {
            self.vector_buffers.push(buffer);
            debug!("♻️ Returned vector buffer to pool (total: {})", self.vector_buffers.len());
        }
    }

    fn return_temp_buffer(&mut self, buffer: metal::Buffer) {
        if self.temp_buffers.len() < MAX_BUFFER_POOL_SIZE {
            self.temp_buffers.push(buffer);
            debug!("♻️ Returned temp buffer to pool (total: {})", self.temp_buffers.len());
        }
    }
}

#[cfg(target_os = "macos")]
use metal::{MTLResourceOptions, MTLSize, CompileOptions};

/// Main Metal Native Collection, composed of modular components
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MetalNativeCollection {
    name: String,
    context: Arc<MetalNativeContext>,
    vector_storage: MetalNativeVectorStorage,
    hnsw_storage: MetalNativeHnswGraph,
    dimension: usize,
    metric: DistanceMetric,
    config: CollectionConfig,
}

#[cfg(target_os = "macos")]
impl MetalNativeCollection {
    pub fn new(dimension: usize, metric: DistanceMetric) -> Result<Self> {
        let config = CollectionConfig {
            dimension,
            metric,
            ..Default::default()
        };
        Self::new_with_name_and_config("MetalNativeCollection", config)
    }

    pub fn new_with_name_and_config(name: &str, config: CollectionConfig) -> Result<Self> {
        let context = Arc::new(MetalNativeContext::new()?);
        let vector_storage = MetalNativeVectorStorage::new(context.clone(), config.dimension)?;
        let hnsw_storage = MetalNativeHnswGraph::new(context.clone(), config.dimension, config.hnsw_config.m)?;

        info!("✅ Metal native collection '{}' created: {}D, {:?}", name, config.dimension, config.metric);

        Ok(Self {
            name: name.to_string(),
            context,
            vector_storage,
            hnsw_storage,
            dimension: config.dimension,
            metric: config.metric,
            config,
        })
    }

    pub fn add_vector(&mut self, vector: Vector) -> Result<usize> {
        self.vector_storage.add_vector(&vector)
    }

    pub fn add_vectors_batch(&mut self, vectors: &[Vector]) -> Result<Vec<usize>> {
        self.vector_storage.add_vectors_batch(vectors)
    }

    pub fn get_vector(&self, index: usize) -> Result<Vector> {
        self.vector_storage.get_vector(index)
    }

    pub fn get_vector_by_id(&self, id: &str) -> Result<Vector> {
        // Use the vector storage ID mapping to find vector by ID
        // The vector storage already maintains a HashMap<String, usize> for ID to index mapping
        self.vector_storage.get_vector_by_id(id)
    }

    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        // Use GPU-accelerated search that keeps data in VRAM
        self.search_gpu_accelerated(query, k)
    }

    /// GPU-accelerated search that keeps all data in VRAM
    fn search_gpu_accelerated(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        if self.vector_count() == 0 {
            return Ok(Vec::new());
        }

        let vector_count = self.vector_count();
        let k = k.min(vector_count);

        // Validate query dimension
        if query.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        debug!("🔍 GPU search: {} vectors, k={}, dim={}", vector_count, k, self.dimension);

        let device = self.context.device();
        let queue = self.context.command_queue();

        // Create query buffer
        let mut query_data = [0.0f32; 512];
        let query_len = query.len().min(512);
        query_data[..query_len].copy_from_slice(&query[..query_len]);

        let gpu_query = GpuSearchQuery {
            data: query_data,
            dimension: query_len as u32,
        };

        let query_size = std::mem::size_of::<GpuSearchQuery>() as u64;
        let query_buffer = device.new_buffer_with_data(
            &gpu_query as *const GpuSearchQuery as *const std::ffi::c_void,
            query_size,
            MTLResourceOptions::StorageModeShared,
        );

        // Create metadata buffer for active vectors
        let mut metadata = Vec::new();
        for i in 0..vector_count {
            let is_active = !self.vector_storage.removed_indices.contains(&i);
            metadata.push(GpuVectorMetadata {
                vector_id: i as u32,
                is_active: if is_active { 1 } else { 0 },
            });
        }

        let metadata_size = (metadata.len() * std::mem::size_of::<GpuVectorMetadata>()) as u64;
        let metadata_buffer = device.new_buffer_with_data(
            metadata.as_ptr() as *const std::ffi::c_void,
            metadata_size,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create results buffer (one result per vector)
        let results_buffer = device.new_buffer(
            (vector_count * std::mem::size_of::<GpuSearchResult>()) as u64,
            MTLResourceOptions::StorageModePrivate,
        );

        // Create final results buffer (top-k)
        let final_results_buffer = device.new_buffer(
            (k * std::mem::size_of::<GpuSearchResult>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        // Get search pipeline
        let library = self.context.library();
        let search_function = library.get_function("gpu_full_vector_search", None)
            .map_err(|e| VectorizerError::Other(format!("Failed to get search function: {:?}", e)))?;

        let search_pipeline = device.new_compute_pipeline_state_with_function(&search_function)
            .map_err(|e| VectorizerError::Other(format!("Failed to create search pipeline: {:?}", e)))?;

        // Get top-k pipeline
        let topk_function = library.get_function("gpu_find_top_k_results", None)
            .map_err(|e| VectorizerError::Other(format!("Failed to get top-k function: {:?}", e)))?;
        let topk_pipeline = device.new_compute_pipeline_state_with_function(&topk_function)
            .map_err(|e| VectorizerError::Other(format!("Failed to create top-k pipeline: {:?}", e)))?;

        // Execute search kernel
        let command_buffer = queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&search_pipeline);
        encoder.set_buffer(0, Some(&self.vector_storage.vectors_buffer), 0); // vectors
        encoder.set_buffer(1, Some(&metadata_buffer), 0);                  // metadata
        encoder.set_buffer(2, Some(&query_buffer), 0);                     // query
        encoder.set_buffer(3, Some(&results_buffer), 0);                   // results
        encoder.set_bytes(4, std::mem::size_of_val(&vector_count) as u64, &vector_count as *const usize as *const std::ffi::c_void); // vector_count
        encoder.set_bytes(5, std::mem::size_of_val(&k) as u64, &k as *const usize as *const std::ffi::c_void); // k
        encoder.set_bytes(6, std::mem::size_of_val(&self.dimension) as u64, &self.dimension as *const usize as *const std::ffi::c_void); // dimension

        // Dispatch threads (one per vector)
        let threadgroups = MTLSize::new(((vector_count + 1023) / 1024) as u64, 1, 1);
        let threads_per_group = MTLSize::new(1024, 1, 1);
        encoder.dispatch_thread_groups(threadgroups, threads_per_group);
        encoder.end_encoding();

        // Execute top-k kernel
        let encoder2 = command_buffer.new_compute_command_encoder();
        encoder2.set_compute_pipeline_state(&topk_pipeline);
        encoder2.set_buffer(0, Some(&results_buffer), 0);        // all results
        encoder2.set_buffer(1, Some(&final_results_buffer), 0);  // final results
        encoder2.set_bytes(2, std::mem::size_of_val(&vector_count) as u64, &vector_count as *const usize as *const std::ffi::c_void); // total_vectors
        encoder2.set_bytes(3, std::mem::size_of_val(&k) as u64, &k as *const usize as *const std::ffi::c_void); // k

        let topk_threadgroups = MTLSize::new(k as u64, 1, 1);
        let topk_threads_per_group = MTLSize::new(1, 1, 1);
        encoder2.dispatch_thread_groups(topk_threadgroups, topk_threads_per_group);
        encoder2.end_encoding();

        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Read final results
        let results_ptr = final_results_buffer.contents() as *const GpuSearchResult;
        let results_slice = unsafe { std::slice::from_raw_parts(results_ptr, k) };

        let mut final_results = Vec::new();
        for result in results_slice {
            if result.vector_id != u32::MAX {
                final_results.push((result.vector_id as usize, result.distance));
            }
        }

        final_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        debug!("✅ GPU search completed: found {} results", final_results.len());
        Ok(final_results)
    }

    pub fn build_hnsw_graph(&mut self, vectors: &[Vector]) -> Result<()> {
        self.hnsw_storage.build_graph(vectors)
    }

    /// Build HNSW index for all vectors in the collection
    pub fn build_index(&mut self) -> Result<()> {
        let vectors = self.get_all_vectors()?;
        self.build_hnsw_graph(&vectors)
    }

    pub fn vector_count(&self) -> usize {
        self.vector_storage.vector_count()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn metric(&self) -> DistanceMetric {
        self.metric
    }

    pub fn get_vector_metadata(&self, id: &str) -> Option<&vector_storage::VectorMetadata> {
        self.vector_storage.get_vector_metadata(id)
    }

    pub fn get_vector_id(&self, index: usize) -> Option<&str> {
        self.vector_storage.get_vector_id(index)
    }

    pub fn get_all_vector_ids(&self) -> Vec<String> {
        self.vector_storage.get_all_vector_ids()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }

    pub fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "MetalNative".to_string());
        metadata.insert("dimension".to_string(), self.dimension.to_string());
        metadata.insert("vector_count".to_string(), self.vector_count().to_string());
        metadata.insert("metric".to_string(), format!("{:?}", self.metric));
        metadata
    }

    pub fn remove_vector(&mut self, id: String) -> Result<()> {
        // Remove from vector storage (marks as removed, doesn't reorganize buffers)
        self.vector_storage.remove_vector(&id)?;

        // Note: HNSW graph is not updated - this would require complex GPU operations
        // For now, searches may return slightly worse results due to removed vectors
        // TODO: Implement HNSW graph repair after vector removal

        debug!("✅ Vector '{}' removed from MetalNativeCollection", id);
        Ok(())
    }

    pub fn estimated_memory_usage(&self) -> usize {
        // Estimate memory usage based on vector count and dimension
        let vector_size = self.vector_count() * self.dimension * std::mem::size_of::<f32>();
        let hnsw_size = self.vector_count() * 64; // Rough estimate for HNSW connections
        vector_size + hnsw_size
    }

    pub fn get_all_vectors(&self) -> Result<Vec<Vector>> {
        let mut vectors = Vec::new();
        for i in 0..self.vector_count() {
            vectors.push(self.get_vector(i)?);
        }
        Ok(vectors)
    }
}
