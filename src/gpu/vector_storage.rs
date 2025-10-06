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
use std::collections::BTreeMap;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "wgpu-gpu")]
use wgpu::{Buffer, BufferUsages, Device, Queue};

/// Buffer chunk information for multi-buffer storage
#[derive(Debug, Clone)]
pub struct BufferChunk {
    pub buffer_id: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub capacity: usize,
    pub used: usize,
}

impl BufferChunk {
    pub fn new(buffer_id: usize, start_index: usize, capacity: usize) -> Self {
        Self {
            buffer_id,
            start_index,
            end_index: start_index + capacity - 1,
            capacity,
            used: 0,
        }
    }
    
    pub fn has_space(&self) -> bool {
        self.used < self.capacity
    }
    
    pub fn remaining_capacity(&self) -> usize {
        self.capacity - self.used
    }
}

/// Multi-buffer storage system with balanced tree
#[derive(Debug)]
pub struct MultiBufferStorage {
    /// Buffer chunks indexed by buffer ID
    pub chunks: BTreeMap<usize, BufferChunk>,
    /// Vector-to-buffer mapping (vector_index -> buffer_id)
    pub vector_to_buffer: BTreeMap<usize, usize>,
    /// Next available vector index
    pub next_vector_index: usize,
    /// Buffer manager for creating new buffers
    pub buffer_manager: BufferManager,
    /// Actual GPU buffers
    pub buffers: Vec<Buffer>,
}

impl MultiBufferStorage {
    pub fn new(buffer_manager: BufferManager) -> Self {
        Self {
            chunks: BTreeMap::new(),
            vector_to_buffer: BTreeMap::new(),
            next_vector_index: 0,
            buffer_manager,
            buffers: Vec::new(),
        }
    }
    
    /// Find the best chunk to insert a vector
    pub fn find_best_chunk(&self) -> Option<usize> {
        // Find chunk with most remaining capacity
        self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.has_space())
            .max_by_key(|(_, chunk)| chunk.remaining_capacity())
            .map(|(buffer_id, _)| *buffer_id)
    }
    
    /// Add a new buffer chunk
    pub fn add_buffer_chunk(&mut self, capacity: usize) -> Result<usize> {
        let buffer_id = self.buffers.len();
        let start_index = self.next_vector_index;
        
        // Create GPU buffer
        let vector_size = capacity * 512 * std::mem::size_of::<f32>(); // Assuming 512 dimensions
        let buffer = self.buffer_manager.create_storage_buffer_rw(
            &format!("gpu_vectors_chunk_{}", buffer_id),
            vector_size as u64,
        )?;
        
        // Create chunk
        let chunk = BufferChunk::new(buffer_id, start_index, capacity);
        
        self.chunks.insert(buffer_id, chunk);
        self.buffers.push(buffer);
        
        info!("🔧 Added buffer chunk {} with capacity {} vectors", buffer_id, capacity);
        
        Ok(buffer_id)
    }
}

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

/// GPU Vector Storage Manager with Multi-Buffer Support
pub struct GpuVectorStorage {
    /// GPU context for operations
    gpu_context: Arc<GpuContext>,
    /// Buffer manager for GPU memory operations
    buffer_manager: BufferManager,
    /// Storage configuration
    config: GpuVectorStorageConfig,
    /// Multi-buffer storage system
    multi_buffer_storage: Arc<RwLock<MultiBufferStorage>>,
    /// Vector storage buffer (legacy - will be replaced by multi-buffer)
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

        // Calculate buffer sizes with chunking to respect GPU buffer limits
        let gpu_limits = &gpu_context.info().limits;
        let max_buffer_size = gpu_limits.max_storage_buffer_binding_size as u64;
        let vector_size_bytes = config.dimension * std::mem::size_of::<f32>();
        let vectors_per_chunk = (max_buffer_size / vector_size_bytes as u64).min(config.initial_capacity as u64) as usize;
        
        // Use smaller chunks to respect buffer limits
        let vector_size = vectors_per_chunk * config.dimension * std::mem::size_of::<f32>();
        let metadata_size = vectors_per_chunk * std::mem::size_of::<GpuVectorMetadata>();
        
        info!("🔧 GPU Vector Storage Chunking Configuration:");
        info!("  - Max buffer binding size: {:.2} MB", max_buffer_size as f64 / (1024.0 * 1024.0));
        info!("  - Vector size: {} bytes", vector_size_bytes);
        info!("  - Vectors per chunk: {}", vectors_per_chunk);
        info!("  - Chunk buffer size: {:.2} MB", vector_size as f64 / (1024.0 * 1024.0));

        // Create multi-buffer storage system
        let mut multi_buffer_storage = MultiBufferStorage::new(buffer_manager.clone());
        
        // Add initial buffer chunk
        multi_buffer_storage.add_buffer_chunk(vectors_per_chunk)?;
        
        // Create persistent GPU buffers (legacy)
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
            multi_buffer_storage: Arc::new(RwLock::new(multi_buffer_storage)),
            vector_buffer,
            metadata_buffer,
            vector_count: Arc::new(RwLock::new(0)),
            vector_id_map: Arc::new(RwLock::new(std::collections::HashMap::new())),
            memory_usage: Arc::new(RwLock::new(total_memory as u64)),
            free_space: Arc::new(RwLock::new(vec![(0, total_memory as u64)])),
        })
    }

    /// Add a vector to GPU storage using multi-buffer system
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

        // Use multi-buffer storage system
        let mut multi_buffer = self.multi_buffer_storage.write();
        
        // Find best chunk to insert vector
        let buffer_id = match multi_buffer.find_best_chunk() {
            Some(id) => id,
            None => {
                // No space available, create new chunk
                let gpu_limits = &self.gpu_context.info().limits;
                let max_buffer_size = gpu_limits.max_storage_buffer_binding_size as u64;
                let vector_size_bytes = self.config.dimension * std::mem::size_of::<f32>();
                let vectors_per_chunk = (max_buffer_size / vector_size_bytes as u64).min(100_000) as usize;
                
                multi_buffer.add_buffer_chunk(vectors_per_chunk)?
            }
        };

        // Get vector index and update chunk usage
        let (vector_index, buffer, offset) = {
            let chunk = multi_buffer.chunks.get_mut(&buffer_id).unwrap();
            let vector_index = chunk.start_index + chunk.used;
            let offset = chunk.used * self.config.dimension * std::mem::size_of::<f32>();
            chunk.used += 1;
            
            (vector_index, multi_buffer.buffers[buffer_id].clone(), offset)
        };
        
        // Update mappings
        multi_buffer.vector_to_buffer.insert(vector_index, buffer_id);
        multi_buffer.next_vector_index = vector_index + 1;
        
        // For now, use the existing method - TODO: implement proper multi-buffer upload
        self.upload_vector_data(&vector.data, offset as u64).await?;
        
        // Update tracking
        {
            let mut count = self.vector_count.write();
            let mut id_map = self.vector_id_map.write();
            id_map.insert(vector.id.clone(), vector_index as u32);
            *count += 1;
            
            debug!("Multi-buffer: Added vector '{}' to buffer {} at index {}, total count: {}", 
                   vector.id, buffer_id, vector_index, *count);
        }

        Ok(vector_index as u32)
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

        debug!("GPU Vector Storage Stats: vector_count={}, memory_used={}MB, free_space={}MB", 
               current_count, memory_usage / (1024*1024), total_free / (1024*1024));

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
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Upload data
        queue.write_buffer(&staging_buffer, 0, bytemuck::cast_slice(data));

        // Copy to persistent buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vector_upload"),
        });

        let data_size = data.len() as u64 * std::mem::size_of::<f32>() as u64;
        
        // 🔧 BUG FIX: Validate that the offset doesn't exceed buffer size
        let vector_buffer_size = self.vector_buffer.size();
        if offset + data_size > vector_buffer_size {
            return Err(VectorizerError::InternalError(format!(
                "Vector buffer overflow: trying to write {} bytes at offset {} but buffer size is {}",
                data_size, offset, vector_buffer_size
            )));
        }
        
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.vector_buffer,
            offset,
            data_size,
        );

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn upload_metadata(&self, metadata: &GpuVectorMetadata, index: u32) -> Result<()> {
        let device = self.gpu_context.device();
        let queue = self.gpu_context.queue();
        
        let offset = (index as u64) * std::mem::size_of::<GpuVectorMetadata>() as u64;
        let metadata_size = std::mem::size_of::<GpuVectorMetadata>() as u64;
        
        // 🔧 BUG FIX: Validate that the offset doesn't exceed buffer size
        let metadata_buffer_size = self.metadata_buffer.size();
        if offset + metadata_size > metadata_buffer_size {
            return Err(VectorizerError::InternalError(format!(
                "Metadata buffer overflow: trying to write at offset {} (size {}) but buffer size is {}. Index: {}",
                offset, metadata_size, metadata_buffer_size, index
            )));
        }
        
        // Create staging buffer
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metadata_staging"),
            size: metadata_size,
            usage: BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
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
            metadata_size,
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

    /// Get all vector buffers from multi-buffer storage for HNSW search
    pub fn get_all_vector_buffers(&self) -> Vec<Buffer> {
        let multi_buffer = self.multi_buffer_storage.read();
        multi_buffer.buffers.clone()
    }

    /// Get the primary vector buffer (for compatibility with existing code)
    pub fn get_primary_vector_buffer(&self) -> Buffer {
        let multi_buffer = self.multi_buffer_storage.read();
        // Return the first buffer or fallback to legacy buffer
        multi_buffer.buffers.first()
            .cloned()
            .unwrap_or_else(|| self.vector_buffer.clone())
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
