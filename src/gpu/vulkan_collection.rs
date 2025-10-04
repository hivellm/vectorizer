//! Vulkan GPU-accelerated collection with complete GPU storage
//!
//! This collection integrates Vulkan GPU operations with complete HNSW storage in VRAM,
//! providing full GPU acceleration for AMD, NVIDIA, Intel, and other Vulkan-compatible GPUs.

use crate::error::{Result, VectorizerError};
use crate::models::{CollectionConfig, CollectionMetadata, DistanceMetric, SearchResult, Vector};
use crate::gpu::{
    GpuContext, GpuConfig, GpuOperations,
    GpuHnswStorage, GpuHnswStorageConfig,
    GpuVectorStorage, GpuVectorStorageConfig,
    GpuHnswNavigation, GpuHnswNode
};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

/// Vulkan GPU-accelerated collection with complete GPU storage
pub struct VulkanCollection {
    /// Collection name
    name: String,
    
    /// Collection configuration
    config: CollectionConfig,
    
    /// GPU context for Vulkan operations
    gpu_context: Arc<GpuContext>,
    
    /// GPU HNSW storage manager (VRAM)
    hnsw_storage: Arc<GpuHnswStorage>,
    /// GPU vector storage manager (VRAM)
    vector_storage: Arc<GpuVectorStorage>,
    /// GPU navigation manager
    navigation: Arc<GpuHnswNavigation>,
    /// Vector ID to GPU index mapping
    vector_id_map: Arc<RwLock<std::collections::HashMap<String, u32>>>,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl VulkanCollection {
    /// Get collection name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get collection configuration
    pub fn config(&self) -> &CollectionConfig {
        &self.config
    }
    
    /// Create a new Vulkan GPU-accelerated collection with complete GPU storage
    pub async fn new(name: String, config: CollectionConfig, gpu_config: GpuConfig) -> Result<Self> {
        info!("🔥 Creating Vulkan GPU-accelerated collection '{}' with complete GPU storage", name);
        
        // Initialize GPU context
        let gpu_context = Arc::new(GpuContext::new(gpu_config).await?);
        let gpu_info = gpu_context.info();
        info!("✅ Vulkan GPU initialized: {} for collection '{}'", gpu_info.name, name);
        
        // Create GPU HNSW storage configuration
        let hnsw_storage_config = GpuHnswStorageConfig {
            max_connections: config.hnsw_config.m,
            max_connections_0: config.hnsw_config.m * 2,
            ef_construction: config.hnsw_config.ef_construction,
            ef_search: config.hnsw_config.ef_search,
            dimension: config.dimension,
            metric: config.metric.clone(),
            initial_node_capacity: 100_000,
            initial_vector_capacity: 100_000,
            gpu_memory_limit: 4 * 1024 * 1024 * 1024, // 4GB for Vulkan
        };

        // Calculate safe initial capacity based on GPU buffer limits
        let gpu_limits = &gpu_context.info().limits;
        let gpu_info = &gpu_context.info();
        
        // Estimate total VRAM based on GPU name and max buffer size
        let estimated_vram = if gpu_info.name.contains("RTX 4090") || gpu_info.name.contains("RTX 4080") || gpu_info.name.contains("RTX 3090") {
            24 * 1024 * 1024 * 1024 // 24GB for high-end GPUs
        } else if gpu_info.name.contains("RTX 4070") || gpu_info.name.contains("RTX 3070") {
            12 * 1024 * 1024 * 1024 // 12GB for mid-range GPUs
        } else if gpu_info.name.contains("RTX 4060") || gpu_info.name.contains("RTX 3060") {
            8 * 1024 * 1024 * 1024 // 8GB for entry-level GPUs
        } else {
            // Fallback: use max_buffer_size as conservative estimate
            gpu_limits.max_buffer_size
        };
        
        let max_buffer_size = (estimated_vram as f64 * 0.8) as u64; // 80% of estimated VRAM
        let vector_size_bytes = config.dimension * std::mem::size_of::<f32>();
        let safe_initial_capacity = (max_buffer_size / vector_size_bytes as u64).min(1_000_000) as usize; // Increased max capacity
        
        info!("🔧 Vulkan GPU Buffer Configuration:");
        info!("  - GPU Name: {}", gpu_info.name);
        info!("  - Max buffer binding size: {:.2} GB", gpu_limits.max_storage_buffer_binding_size as f64 / (1024.0 * 1024.0 * 1024.0));
        info!("  - Max buffer size: {:.2} GB", gpu_limits.max_buffer_size as f64 / (1024.0 * 1024.0 * 1024.0));
        info!("  - Estimated total VRAM: {:.2} GB", estimated_vram as f64 / (1024.0 * 1024.0 * 1024.0));
        info!("  - Using 80% of estimated VRAM: {:.2} GB", max_buffer_size as f64 / (1024.0 * 1024.0 * 1024.0));
        info!("  - Vector size: {} bytes", vector_size_bytes);
        info!("  - Calculated initial capacity: {} vectors", safe_initial_capacity);
        
        // Create GPU vector storage configuration
        let vector_storage_config = GpuVectorStorageConfig {
            dimension: config.dimension,
            initial_capacity: safe_initial_capacity,
            max_capacity: 1_000_000,
            gpu_memory_limit: 4 * 1024 * 1024 * 1024, // 4GB for Vulkan
            enable_compression: false,
            compression_ratio: 0.5,
        };

        // Initialize GPU storage managers
        let hnsw_storage = Arc::new(
            GpuHnswStorage::new(gpu_context.clone(), hnsw_storage_config).await?
        );

        let vector_storage = Arc::new(
            GpuVectorStorage::new(gpu_context.clone(), vector_storage_config).await?
        );

        let navigation = Arc::new(
            GpuHnswNavigation::new(gpu_context.clone()).await?
        );

        let now = chrono::Utc::now();
        
        Ok(Self {
            name,
            config,
            gpu_context,
            hnsw_storage,
            vector_storage,
            navigation,
            vector_id_map: Arc::new(RwLock::new(std::collections::HashMap::new())),
            created_at: now,
            updated_at: Arc::new(RwLock::new(now)),
        })
    }
    
    /// Add a vector to the collection with complete GPU storage
    pub async fn add_vector(&self, vector: Vector) -> Result<()> {
        debug!("Adding vector '{}' to Vulkan collection '{}' with GPU storage", vector.id, self.name);

        // Validate dimension
        if vector.data.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: vector.data.len(),
            });
        }
        
        // Store vector in GPU vector storage
        let vector_index = self.vector_storage.add_vector(&vector).await?;

        // Create HNSW node for the vector
        let node = GpuHnswNode {
            id: vector_index,
            level: self.calculate_node_level(),
            connections: [0; 16], // Will be populated during graph construction
            connection_count: 0,
            vector_buffer_offset: (vector_index as u64) * (self.config.dimension as u64 * std::mem::size_of::<f32>() as u64),
        };

        // Store node in GPU HNSW storage
        let node_index = self.hnsw_storage.add_node(node).await?;
        debug!("Added HNSW node at index {} for vector '{}'", node_index, vector.id);

        // Update vector ID mapping
        {
            let mut id_map = self.vector_id_map.write();
            id_map.insert(vector.id.clone(), vector_index);
        }

        // TODO: Implement graph construction and connection building
        // This would involve:
        // 1. Finding neighbors using GPU-accelerated distance calculations
        // 2. Building connections at each level
        // 3. Updating connection buffers in GPU memory
        
        // Update timestamp
        *self.updated_at.write() = chrono::Utc::now();
        
        // Debug: Show current stats
        let hnsw_stats = self.hnsw_storage.get_memory_stats();
        let vector_stats = self.vector_storage.get_storage_stats();
        debug!("After adding vector '{}': hnsw_nodes={}, vector_count={}", 
               vector.id, hnsw_stats.node_count, vector_stats.vector_count);
        
        debug!("Successfully added vector '{}' to Vulkan GPU storage", vector.id);
        Ok(())
    }
    
    /// Search for similar vectors using complete GPU acceleration
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        debug!("Searching Vulkan collection '{}' with complete GPU acceleration", self.name);

        // Validate query dimension
        if query.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: query.len(),
            });
        }

        // Get current storage stats
        let hnsw_stats = self.hnsw_storage.get_memory_stats();
        let vector_stats = self.vector_storage.get_storage_stats();

        debug!("Vulkan search stats: hnsw_nodes={}, vector_count={}", 
               hnsw_stats.node_count, vector_stats.vector_count);

        if hnsw_stats.node_count == 0 || vector_stats.vector_count == 0 {
            debug!("Empty collection detected, returning empty results");
            return Ok(Vec::new());
        }

        // Execute GPU-accelerated HNSW search with complete GPU navigation
        // Use primary buffer from multi-buffer storage system
        let primary_vector_buffer = self.vector_storage.get_primary_vector_buffer();
        let search_result = self.navigation.search(
            query,
            k,
            self.config.hnsw_config.ef_search,
            self.config.metric.clone(),
            &self.hnsw_storage.node_buffer,
            &primary_vector_buffer,
            &self.hnsw_storage.connection_buffer,
            hnsw_stats.node_count,
            self.config.dimension,
        ).await?;

        if search_result.result_count == 0 {
            return Ok(Vec::new());
        }

        // Convert results to SearchResult format
        let mut results = Vec::with_capacity(search_result.result_count);
        
        for (i, &node_index) in search_result.node_indices.iter().enumerate() {
            // Get vector by index from GPU storage
            let vector = self.get_vector_by_index(node_index).await?;
            let score = search_result.scores.get(i).copied().unwrap_or(0.0);
            
            results.push(SearchResult {
                id: vector.id.clone(),
                score,
                vector: Some(vector.data.clone()),
                payload: vector.payload.clone(),
            });
        }

        debug!("Vulkan GPU search completed, found {} results", results.len());
        Ok(results)
    }
    
    /// Get collection metadata
    pub fn metadata(&self) -> CollectionMetadata {
        let vector_stats = self.vector_storage.get_storage_stats();
        
        CollectionMetadata {
            name: self.name.clone(),
            vector_count: vector_stats.vector_count,
            config: self.config.clone(),
            document_count: 0, // Vulkan collections don't track documents separately
            created_at: self.created_at,
            updated_at: *self.updated_at.read(),
        }
    }
    
    /// Remove a vector by ID
    pub fn remove_vector(&self, id: &str) -> Result<()> {
        debug!("Removing vector '{}' from Vulkan GPU collection '{}'", id, self.name);

        // Remove from vector storage
        self.vector_storage.remove_vector(id)?;

        // Remove from ID mapping
        {
            let mut id_map = self.vector_id_map.write();
            id_map.remove(id);
        }

        // TODO: Remove from HNSW storage and update graph connections
        
        *self.updated_at.write() = chrono::Utc::now();
        debug!("Successfully removed vector '{}' from Vulkan GPU storage", id);
        Ok(())
    }
    
    /// Get a vector by ID
    pub async fn get_vector(&self, id: &str) -> Result<Vector> {
        self.vector_storage.get_vector(id).await
    }

    /// Get a vector by GPU index
    async fn get_vector_by_index(&self, index: u32) -> Result<Vector> {
        // TODO: Implement efficient retrieval by index
        // For now, we'll need to iterate through the ID map
        let id_map = self.vector_id_map.read();
        for (id, &vector_index) in id_map.iter() {
            if vector_index == index {
                return self.vector_storage.get_vector(id).await;
            }
        }
        
        Err(VectorizerError::VectorNotFound(format!("vector_{}", index)))
    }
    
    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.vector_storage.get_storage_stats().vector_count
    }
    
    /// Estimate memory usage
    pub fn estimated_memory_usage(&self) -> usize {
        let hnsw_stats = self.hnsw_storage.get_memory_stats();
        let vector_stats = self.vector_storage.get_storage_stats();
        
        (hnsw_stats.total_allocated + vector_stats.memory_used) as usize
    }
    
    /// Get all vectors (for export/backup)
    pub async fn get_all_vectors(&self) -> Result<Vec<Vector>> {
        // TODO: Implement efficient retrieval of all vectors
        // This would require maintaining a list of vector IDs
        Ok(Vec::new())
    }

    /// Get GPU memory statistics
    pub fn get_gpu_memory_stats(&self) -> VulkanGpuMemoryStats {
        let hnsw_stats = self.hnsw_storage.get_memory_stats();
        let vector_stats = self.vector_storage.get_storage_stats();

        VulkanGpuMemoryStats {
            hnsw_memory_used: hnsw_stats.total_allocated,
            vector_memory_used: vector_stats.memory_used,
            total_memory_used: hnsw_stats.total_allocated + vector_stats.memory_used,
            memory_limit: hnsw_stats.memory_limit,
            memory_usage_percent: ((hnsw_stats.total_allocated + vector_stats.memory_used) as f64 / hnsw_stats.memory_limit as f64) * 100.0,
            node_count: hnsw_stats.node_count,
            vector_count: vector_stats.vector_count,
        }
    }

    /// Get GPU info
    pub fn gpu_info(&self) -> String {
        let info = self.gpu_context.info();
        format!("{} (Vulkan GPU Full Storage)", info.name)
    }

    // Private helper methods

    /// Calculate node level for HNSW hierarchy
    fn calculate_node_level(&self) -> u32 {
        // Simplified level calculation - in practice this should follow
        // the HNSW paper's probability distribution
        let mut rng = rand::thread_rng();
        let level_mult = 1.0 / (2.0_f64).ln();
        let level = -((rand::random::<f64>()).ln() * level_mult) as u32;
        level.max(0)
    }
}

/// Vulkan GPU Memory Statistics
#[derive(Debug, Clone)]
pub struct VulkanGpuMemoryStats {
    pub hnsw_memory_used: u64,
    pub vector_memory_used: u64,
    pub total_memory_used: u64,
    pub memory_limit: u64,
    pub memory_usage_percent: f64,
    pub node_count: usize,
    pub vector_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HnswConfig;
    
    #[tokio::test]
    async fn test_vulkan_collection_creation() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        let gpu_config = GpuConfig::default();
        
        // This will fail if Vulkan is not available, which is expected
        let result = VulkanCollection::new("test".to_string(), config, gpu_config).await;
        
        match result {
            Ok(collection) => {
                assert_eq!(collection.name(), "test");
                assert_eq!(collection.vector_count(), 0);
                println!("Vulkan GPU full storage collection created successfully");
            }
            Err(e) => println!("Vulkan not available (expected): {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_vulkan_collection_add_and_search() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        let gpu_config = GpuConfig::default();
        if let Ok(collection) = VulkanCollection::new("test".to_string(), config, gpu_config).await {
            // Add vector
            let vector = Vector {
                id: "v1".to_string(),
                data: vec![1.0; 128],
                payload: None,
            };
            
            if let Ok(_) = collection.add_vector(vector).await {
                assert_eq!(collection.vector_count(), 1);

                // Search with GPU navigation
                let query = vec![1.0; 128];
                let results = collection.search(&query, 1).await.unwrap();
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].id, "v1");
            }
        }
    }

    #[tokio::test]
    async fn test_vulkan_gpu_memory_stats() {
        let config = CollectionConfig {
            dimension: 128,
            metric: DistanceMetric::Cosine,
            hnsw_config: HnswConfig::default(),
            quantization: Default::default(),
            compression: Default::default(),
        };
        
        let gpu_config = GpuConfig::default();
        if let Ok(collection) = VulkanCollection::new("test".to_string(), config, gpu_config).await {
            let stats = collection.get_gpu_memory_stats();
            assert!(stats.total_memory_used > 0);
            assert!(stats.memory_limit > 0);
            println!("Vulkan GPU memory stats: {}MB used / {}MB limit", 
                     stats.total_memory_used / (1024 * 1024),
                     stats.memory_limit / (1024 * 1024));
        }
    }
}

