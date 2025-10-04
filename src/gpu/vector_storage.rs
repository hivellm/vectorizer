//! GPU Vector Storage Manager
//!
//! This module provides persistent vector storage in GPU VRAM,
//! eliminating CPU-GPU memory transfers during search operations.
//! Supports batch operations and memory management.

use crate::error::{Result, VectorizerError};
use crate::models::{Vector, DistanceMetric};
use crate::gpu::{GpuContext};
use crate::gpu::buffers::BufferManager;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "wgpu-gpu")]
use wgpu::{Buffer, BufferUsages, Device, Queue};

/// GPU Vector Storage Configuration
#[derive(Debug, Clone)]
pub struct GpuVectorStorageConfig {
    /// Vector dimension
    pub dimension: usize,
    /// Initial capacity for vectors
    pub initial_capacity: usize,
    /// Maximum capacity for vectors
    pub max_capacity: usize,
    /// GPU memory limit (bytes)
    pub gpu_memory_limit: u64,
    /// Enable compression for vectors
    pub enable_compression: bool,
    /// Compression ratio (0.0-1.0)
    pub compression_ratio: f32,
}

impl Default for GpuVectorStorageConfig {
    fn default() -> Self {
        Self {
            dimension: 768,
            initial_capacity: 100_000,
            max_capacity: 1_000_000,
            gpu_memory_limit: 4 * 1024 * 1024 * 1024, // 4GB default
            enable_compression: false,
            compression_ratio: 0.5,
        }
    }
}

/// GPU Vector Storage Manager
pub struct GpuVectorStorage {
    /// GPU context for operations
    gpu_context: Arc<GpuContext>,
    /// Buffer manager for GPU memory operations
    buffer_manager: BufferManager,
    /// Storage configuration
    config: GpuVectorStorageConfig,
    /// Vector storage buffer (persistent in VRAM)
    #[cfg(feature = "wgpu-gpu")]
    pub vector_buffer: Buffer,
    /// Vector metadata buffer (IDs, offsets, etc.)
    #[cfg(feature = "wgpu-gpu")]
    pub metadata_buffer: Buffer,
    /// Current vector count
    vector_count: Arc<RwLock<usize>>,
    /// Vector ID to index mapping
    vector_id_map: Arc<RwLock<std::collections::HashMap<String, u32>>>,
    /// Memory usage tracking
    memory_usage: Arc<RwLock<u64>>,
    /// Free space tracking
    free_space: Arc<RwLock<Vec<(u64, u64)>>>, // (offset, size) pairs
}

/// GPU Vector Metadata
#[derive(Debug, Clone, Copy)]
pub struct GpuVectorMetadata {
    /// Vector ID hash (for quick lookup)
    pub id_hash: u64,
    /// Offset in vector buffer
    pub buffer_offset: u64,
    /// Vector dimension
    pub dimension: u32,
    /// Compression flag
    pub compressed: u32,
    /// Creation timestamp
    pub created_at: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
}

#[cfg(feature = "wgpu-gpu")]
unsafe impl bytemuck::Pod for GpuVectorMetadata {}
#[cfg(feature = "wgpu-gpu")]
unsafe impl bytemuck::Zeroable for GpuVectorMetadata {}

impl GpuVectorStorage {
    /// Create a new GPU vector storage manager
    pub async fn new(
        gpu_context: Arc<GpuContext>,
        config: GpuVectorStorageConfig,
    ) -> Result<Self> {
        info!("Creating GPU vector storage with {}MB limit", 
              config.gpu_memory_limit / (1024 * 1024));

        let buffer_manager = BufferManager::new(
            Arc::new(gpu_context.device().clone()),
            Arc::new(gpu_context.queue().clone()),
        );

        // Calculate buffer sizes
        let vector_size = config.initial_capacity * config.dimension * std::mem::size_of::<f32>();
        let metadata_size = config.initial_capacity * std::mem::size_of::<GpuVectorMetadata>();

        // Create persistent GPU buffers
        #[cfg(feature = "wgpu-gpu")]
        let (vector_buffer, metadata_buffer) = {
            let vector_buffer = buffer_manager.create_storage_buffer_rw(
                "gpu_vectors",
                vector_size as u64,
            )?;

            let metadata_buffer = buffer_manager.create_storage_buffer_rw(
                "gpu_vector_metadata",
                metadata_size as u64,
            )?;

            (vector_buffer, metadata_buffer)
        };

        #[cfg(not(feature = "wgpu-gpu"))]
        let (vector_buffer, metadata_buffer) = (
            wgpu::Buffer::default(),
            wgpu::Buffer::default(),
        );

        let total_memory = vector_size + metadata_size;
        info!("GPU vector storage allocated: {}MB for vectors, {}MB for metadata",
              vector_size / (1024 * 1024),
              metadata_size / (1024 * 1024));

        Ok(Self {
            gpu_context,
            buffer_manager,
            config,
            vector_buffer,
            metadata_buffer,
            vector_count: Arc::new(RwLock::new(0)),
            vector_id_map: Arc::new(RwLock::new(std::collections::HashMap::new())),
            memory_usage: Arc::new(RwLock::new(total_memory as u64)),
            free_space: Arc::new(RwLock::new(vec![(0, total_memory as u64)])),
        })
    }

    /// Add a vector to GPU storage
    pub async fn add_vector(&self, vector: &Vector) -> Result<u32> {
        // Validate dimension
        if vector.data.len() != self.config.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.config.dimension,
                actual: vector.data.len(),
            });
        }

        // Check if vector already exists
        {
            let id_map = self.vector_id_map.read();
            if id_map.contains_key(&vector.id) {
                return Err(VectorizerError::InternalError(
                    format!("Vector with ID '{}' already exists", vector.id)
                ));
            }
        }

        // Check capacity
        let current_count = *self.vector_count.read();
        if current_count >= self.config.max_capacity {
            return Err(VectorizerError::InternalError(
                format!("Maximum capacity ({}) reached", self.config.max_capacity)
            ));
        }

        // Find free space
        let required_size = self.config.dimension * std::mem::size_of::<f32>();
        let buffer_offset = self.allocate_space(required_size as u64)?;

        // Prepare vector data
        let mut vector_data = vector.data.clone();
        
        // Apply compression if enabled
        if self.config.enable_compression {
            vector_data = self.compress_vector(&vector_data)?;
        }

        // Upload vector data to GPU
        #[cfg(feature = "wgpu-gpu")]
        {
            self.upload_vector_data(&vector_data, buffer_offset).await?;
        }

        // Create metadata
        let metadata = GpuVectorMetadata {
            id_hash: self.hash_vector_id(&vector.id),
            buffer_offset,
            dimension: self.config.dimension as u32,
            compressed: if self.config.enable_compression { 1 } else { 0 },
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_accessed: 0,
        };

        // Upload metadata to GPU
        #[cfg(feature = "wgpu-gpu")]
        {
            self.upload_metadata(&metadata, current_count as u32).await?;
        }

        // Update tracking
        {
            let mut count = self.vector_count.write();
            let mut id_map = self.vector_id_map.write();
            id_map.insert(vector.id.clone(), *count as u32);
            *count += 1;
        }

        debug!("Added vector {} to GPU storage at offset {}", vector.id, buffer_offset);
        Ok(current_count as u32)
    }

    /// Batch add vectors to GPU storage
    pub async fn batch_add_vectors(&self, vectors: &[Vector]) -> Result<Vec<u32>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        info!("Batch adding {} vectors to GPU storage", vectors.len());

        let mut results = Vec::with_capacity(vectors.len());
        
        for vector in vectors {
            let index = self.add_vector(vector).await?;
            results.push(index);
        }

        info!("Successfully batch added {} vectors to GPU storage", results.len());
        Ok(results)
    }

    /// Get vector data from GPU storage
    pub async fn get_vector(&self, vector_id: &str) -> Result<Vector> {
        // Get vector index
        let vector_index = {
            let id_map = self.vector_id_map.read();
            id_map.get(vector_id)
                .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?
                .clone()
        };

        // Get metadata from GPU
        #[cfg(feature = "wgpu-gpu")]
        let metadata = self.get_metadata_from_gpu(vector_index).await?;

        #[cfg(not(feature = "wgpu-gpu"))]
        let metadata = GpuVectorMetadata::default();

        // Download vector data from GPU
        #[cfg(feature = "wgpu-gpu")]
        let vector_data = self.download_vector_data(metadata.buffer_offset, metadata.dimension as usize).await?;

        #[cfg(not(feature = "wgpu-gpu"))]
        let vector_data = vec![0.0f32; self.config.dimension];

        // Decompress if needed
        let final_data = if metadata.compressed == 1 {
            self.decompress_vector(&vector_data)?
        } else {
            vector_data
        };

        // Update last accessed time
        #[cfg(feature = "wgpu-gpu")]
        {
            self.update_last_accessed(vector_index).await?;
        }

        Ok(Vector {
            id: vector_id.to_string(),
            data: final_data,
            payload: None, // TODO: Store payload separately
        })
    }

    /// Get multiple vectors by IDs
    pub async fn get_vectors(&self, vector_ids: &[String]) -> Result<Vec<Vector>> {
        let mut results = Vec::with_capacity(vector_ids.len());
        
        for vector_id in vector_ids {
            let vector = self.get_vector(vector_id).await?;
            results.push(vector);
        }
        
        Ok(results)
    }

    /// Remove a vector from GPU storage
    pub fn remove_vector(&self, vector_id: &str) -> Result<()> {
        // Get vector index
        let vector_index = {
            let mut id_map = self.vector_id_map.write();
            id_map.remove(vector_id)
                .ok_or_else(|| VectorizerError::VectorNotFound(vector_id.to_string()))?
        };

        // Get metadata to find buffer offset
        // TODO: Implement proper removal with space reclamation
        
        // Update tracking
        {
            let mut count = self.vector_count.write();
            *count -= 1;
        }

        debug!("Removed vector {} from GPU storage", vector_id);
        Ok(())
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> GpuVectorStorageStats {
        let current_count = *self.vector_count.read();
        let memory_usage = *self.memory_usage.read();
        let free_space = self.free_space.read();
        
        let total_free = free_space.iter().map(|(_, size)| *size).sum::<u64>();
        let memory_usage_percent = (memory_usage as f64 / self.config.gpu_memory_limit as f64) * 100.0;

        GpuVectorStorageStats {
            vector_count: current_count,
            max_capacity: self.config.max_capacity,
            memory_used: memory_usage,
            memory_limit: self.config.gpu_memory_limit,
            memory_usage_percent,
            total_free_space: total_free,
            free_space_fragments: free_space.len(),
        }
    }

    // Private helper methods

    fn allocate_space(&self, size: u64) -> Result<u64> {
        let mut free_space = self.free_space.write();
        
        // Find first fit
        for (i, (offset, free_size)) in free_space.iter().enumerate() {
            if *free_size >= size {
                let allocated_offset = *offset;
                
                if *free_size > size {
                    // Update the free space entry
                    free_space[i] = (offset + size, free_size - size);
                } else {
                    // Remove the free space entry
                    free_space.remove(i);
                }
                
                return Ok(allocated_offset);
            }
        }
        
        Err(VectorizerError::InternalError(
            "Not enough free space in GPU memory".to_string()
        ))
    }

    fn hash_vector_id(&self, id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish()
    }

    fn compress_vector(&self, data: &[f32]) -> Result<Vec<f32>> {
        // Simple compression: reduce precision
        let compressed: Vec<f32> = data.iter()
            .map(|&x| (x * self.config.compression_ratio).round() / self.config.compression_ratio)
            .collect();
        
        Ok(compressed)
    }

    fn decompress_vector(&self, data: &[f32]) -> Result<Vec<f32>> {
        // Simple decompression: restore precision
        let decompressed: Vec<f32> = data.iter()
            .map(|&x| x / self.config.compression_ratio)
            .collect();
        
        Ok(decompressed)
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn upload_vector_data(&self, data: &[f32], offset: u64) -> Result<()> {
        let device = self.gpu_context.device();
        let queue = self.gpu_context.queue();
        
        // Create staging buffer
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vector_staging"),
            size: data.len() as u64 * std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Upload data
        queue.write_buffer(&staging_buffer, 0, bytemuck::cast_slice(data));

        // Copy to persistent buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vector_upload"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.vector_buffer,
            offset,
            data.len() as u64 * std::mem::size_of::<f32>() as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn upload_metadata(&self, metadata: &GpuVectorMetadata, index: u32) -> Result<()> {
        let device = self.gpu_context.device();
        let queue = self.gpu_context.queue();
        
        let offset = (index as u64) * std::mem::size_of::<GpuVectorMetadata>() as u64;
        
        // Create staging buffer
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metadata_staging"),
            size: std::mem::size_of::<GpuVectorMetadata>() as u64,
            usage: BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Upload metadata
        queue.write_buffer(&staging_buffer, 0, bytemuck::bytes_of(metadata));

        // Copy to persistent buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("metadata_upload"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.metadata_buffer,
            offset,
            std::mem::size_of::<GpuVectorMetadata>() as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn get_metadata_from_gpu(&self, index: u32) -> Result<GpuVectorMetadata> {
        let offset = (index as u64) * std::mem::size_of::<GpuVectorMetadata>() as u64;
        
        // Create staging buffer for reading
        let staging_buffer = self.buffer_manager.create_staging_buffer(
            "metadata_read_staging",
            std::mem::size_of::<GpuVectorMetadata>() as u64,
        )?;

        // Copy from GPU to staging
        let device = self.gpu_context.device();
        let queue = self.gpu_context.queue();
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("metadata_read"),
        });

        encoder.copy_buffer_to_buffer(
            &self.metadata_buffer,
            offset,
            &staging_buffer,
            0,
            std::mem::size_of::<GpuVectorMetadata>() as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read back metadata
        let metadata_bytes = self.buffer_manager.read_buffer_sync(&staging_buffer)?;
        let metadata = bytemuck::cast_slice::<f32, GpuVectorMetadata>(&metadata_bytes)[0];
        
        Ok(metadata)
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn download_vector_data(&self, offset: u64, dimension: usize) -> Result<Vec<f32>> {
        let size = dimension * std::mem::size_of::<f32>();
        
        // Create staging buffer for reading
        let staging_buffer = self.buffer_manager.create_staging_buffer(
            "vector_read_staging",
            size as u64,
        )?;

        // Copy from GPU to staging
        let device = self.gpu_context.device();
        let queue = self.gpu_context.queue();
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vector_read"),
        });

        encoder.copy_buffer_to_buffer(
            &self.vector_buffer,
            offset,
            &staging_buffer,
            0,
            size as u64,
        );

        queue.submit(std::iter::once(encoder.finish()));

        // Read back vector data
        let vector_data = self.buffer_manager.read_buffer_sync(&staging_buffer)?;
        Ok(vector_data)
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn update_last_accessed(&self, index: u32) -> Result<()> {
        // TODO: Implement last accessed time update
        Ok(())
    }
}

/// GPU Vector Storage Statistics
#[derive(Debug, Clone)]
pub struct GpuVectorStorageStats {
    pub vector_count: usize,
    pub max_capacity: usize,
    pub memory_used: u64,
    pub memory_limit: u64,
    pub memory_usage_percent: f64,
    pub total_free_space: u64,
    pub free_space_fragments: usize,
}

/// Safe to use across threads
unsafe impl Send for GpuVectorStorage {}
unsafe impl Sync for GpuVectorStorage {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_vector_storage_creation() {
        let config = GpuVectorStorageConfig {
            dimension: 128,
            ..Default::default()
        };

        let gpu_config = crate::gpu::GpuConfig::default();
        if let Ok(gpu_context) = GpuContext::new(gpu_config).await {
            let result = GpuVectorStorage::new(Arc::new(gpu_context), config).await;
            
            match result {
                Ok(storage) => {
                    let stats = storage.get_storage_stats();
                    assert!(stats.memory_used > 0);
                    println!("GPU vector storage created successfully");
                }
                Err(e) => println!("GPU not available (expected): {}", e),
            }
        }
    }
}
