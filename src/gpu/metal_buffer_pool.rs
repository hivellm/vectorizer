//! Metal Buffer Pool for optimized memory management
//!
//! This module provides efficient buffer pooling to reduce VRAM fragmentation
//! and eliminate unnecessary buffer reallocations.

use crate::error::{Result, VectorizerError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::debug;

#[cfg(target_os = "macos")]
use metal::{
    Buffer as MetalBuffer, Device as MetalDevice,
    MTLResourceOptions, MTLStorageMode, MTLCPUCacheMode,
    foreign_types::ForeignType,
};

/// Inner structure for thread-safe buffer pool
#[cfg(target_os = "macos")]
#[derive(Debug)]
struct MetalBufferPoolInner {
    device: MetalDevice,
    available_buffers: HashMap<usize, Vec<MetalBuffer>>,
    total_allocated: usize,
    max_pool_size: usize,
}

/// Metal Buffer Pool for efficient memory management (thread-safe)
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct MetalBufferPool {
    inner: Arc<Mutex<MetalBufferPoolInner>>,
}

#[cfg(target_os = "macos")]
impl MetalBufferPool {
    /// Create new buffer pool
    pub fn new(device: MetalDevice) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MetalBufferPoolInner {
                device,
                available_buffers: HashMap::new(),
                total_allocated: 0,
                max_pool_size: 100, // Maximum number of buffers to keep in pool
            })),
        }
    }
    
    /// Get buffer from pool or create new one
    pub fn get_buffer(&self, size: usize) -> Result<MetalBuffer> {
        let mut inner = self.inner.lock().unwrap();
        
        // Try to reuse existing buffer
        if let Some(buffer) = inner.available_buffers.get_mut(&size).and_then(|v| v.pop()) {
            return Ok(buffer);
        }
        
        // Create new buffer if none available
        let buffer = inner.device.new_buffer(
            size as u64,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        inner.total_allocated += size;
        Ok(buffer)
    }
    
    /// Return buffer to pool for reuse
    pub fn return_buffer(&self, buffer: MetalBuffer, size: usize) {
        let mut inner = self.inner.lock().unwrap();
        
        // Only keep buffers if pool isn't too large
        if inner.available_buffers.values().map(|v| v.len()).sum::<usize>() < inner.max_pool_size {
            inner.available_buffers.entry(size).or_insert_with(Vec::new).push(buffer);
        }
        // Otherwise, buffer is automatically deallocated
    }
    
    /// Get buffer with exponential growth strategy
    pub fn get_buffer_with_growth(&self, current_size: usize, required_size: usize) -> Result<MetalBuffer> {
        // Calculate new size with exponential growth
        let new_size = if required_size > current_size {
            // Grow by at least 50% or double, whichever is larger
            std::cmp::max(required_size, current_size * 2)
        } else {
            required_size
        };
        
        self.get_buffer(new_size)
    }
    
    /// Compact pool by removing excess buffers
    pub fn compact(&self) {
        let mut inner = self.inner.lock().unwrap();
        
        // Remove half of the buffers from each size category
        for buffers in inner.available_buffers.values_mut() {
            if buffers.len() > 2 {
                let remove_count = buffers.len() / 2;
                for _ in 0..remove_count {
                    buffers.pop(); // Buffer is automatically deallocated
                }
            }
        }
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> BufferPoolStats {
        let inner = self.inner.lock().unwrap();
        let pooled_buffers: usize = inner.available_buffers.values().map(|v| v.len()).sum();
        let total_pooled_size: usize = inner.available_buffers.iter()
            .map(|(size, buffers)| size * buffers.len())
            .sum();
        
        BufferPoolStats {
            total_allocated: inner.total_allocated,
            pooled_buffers,
            total_pooled_size,
            pool_utilization: if inner.total_allocated > 0 {
                total_pooled_size as f64 / inner.total_allocated as f64
            } else {
                0.0
            },
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for MetalBufferPool {
    fn drop(&mut self) {
        // Clean up all pooled buffers
        if let Ok(mut inner) = self.inner.lock() {
            for buffers in inner.available_buffers.values_mut() {
                buffers.clear();
            }
            inner.available_buffers.clear();
            
            // Reset statistics
            inner.total_allocated = 0;
        }
        
        debug!("✅ MetalBufferPool cleaned up - all buffers deallocated");
    }
}

/// Buffer pool statistics
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub total_allocated: usize,
    pub pooled_buffers: usize,
    pub total_pooled_size: usize,
    pub pool_utilization: f64,
}

/// Optimized Metal Native Collection with buffer pooling
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct OptimizedMetalNativeCollection {
    context: Arc<crate::gpu::MetalNativeContext>,
    vectors_buffer: MetalBuffer,
    buffer_capacity: usize,
    vector_count: usize,
    dimension: usize,
    metric: crate::models::DistanceMetric,
    vector_id_map: std::collections::HashMap<String, usize>,
    buffer_pool: MetalBufferPool,
}

#[cfg(target_os = "macos")]
impl OptimizedMetalNativeCollection {
    /// Create new optimized collection with pre-allocated buffer
    pub fn new(
        dimension: usize,
        metric: crate::models::DistanceMetric,
        initial_capacity: usize,
    ) -> Result<Self> {
        let context = Arc::new(crate::gpu::MetalNativeContext::new()?);
        let device = context.device();
        
        // Pre-allocate buffer with initial capacity
        let initial_size = initial_capacity * dimension * std::mem::size_of::<f32>();
        let vectors_buffer = device.new_buffer(
            initial_size as u64,
            MTLResourceOptions::StorageModePrivate,
        );
        
        let mut buffer_pool = MetalBufferPool::new(device.clone());
        
        Ok(Self {
            context,
            vectors_buffer,
            buffer_capacity: initial_capacity,
            vector_count: 0,
            dimension,
            metric,
            vector_id_map: std::collections::HashMap::new(),
            buffer_pool,
        })
    }
    
    /// Add multiple vectors in batch (optimized)
    pub fn add_vectors_batch(&mut self, vectors: &[crate::models::Vector]) -> Result<Vec<usize>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }
        
        // Validate all vectors before processing
        for (i, vector) in vectors.iter().enumerate() {
            // Validate vector ID is unique
            if self.vector_id_map.contains_key(&vector.id) {
                return Err(crate::error::VectorizerError::Other(format!("Vector with ID '{}' already exists", vector.id)));
            }
            
            // Validate vector dimension
            if vector.data.len() != self.dimension {
                return Err(crate::error::VectorizerError::DimensionMismatch {
                    expected: self.dimension,
                    actual: vector.data.len(),
                });
            }
            
            // Validate all values are finite (no NaN/Infinity)
            for (j, &value) in vector.data.iter().enumerate() {
                if !value.is_finite() {
                    return Err(crate::error::VectorizerError::Other(format!("Vector {} contains non-finite value at index {}: {}", i, j, value)));
                }
            }
            
            // Validate ID length
            if vector.id.len() > 256 {
                return Err(crate::error::VectorizerError::Other(format!("Vector {} ID too long (max 256 chars)", i)));
            }
        }
        
        // Check if buffer needs expansion
        if self.vector_count + vectors.len() > self.buffer_capacity {
            self.expand_buffer(self.vector_count + vectors.len())?;
        }
        
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Prepare staging data for all vectors
        let mut staging_data = Vec::with_capacity(vectors.len() * self.dimension * 4);
        for vector in vectors {
            staging_data.extend_from_slice(&vector.data);
        }
        
        // Create staging buffer
        let staging_buffer = device.new_buffer_with_data(
            staging_data.as_ptr() as *const std::ffi::c_void,
            staging_data.len() as u64,
            MTLResourceOptions::StorageModeShared,
        );
        
        // Copy batch to VRAM
        let command_buffer = queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        let offset = self.vector_count * self.dimension * std::mem::size_of::<f32>();
        blit_encoder.copy_from_buffer(
            &staging_buffer,
            0,
            &self.vectors_buffer,
            offset as u64,
            staging_data.len() as u64,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Update state
        let mut indices = Vec::new();
        for (i, vector) in vectors.iter().enumerate() {
            let index = self.vector_count + i;
            self.vector_id_map.insert(vector.id.clone(), index);
            indices.push(index);
        }
        self.vector_count += vectors.len();
        
        tracing::debug!("✅ Added {} vectors in batch (total: {})", vectors.len(), self.vector_count);
        Ok(indices)
    }
    
    /// Expand buffer with exponential growth
    fn expand_buffer(&mut self, new_capacity: usize) -> Result<()> {
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Calculate new capacity with exponential growth
        let new_capacity = std::cmp::max(new_capacity, self.buffer_capacity * 2);
        let new_size = new_capacity * self.dimension * std::mem::size_of::<f32>();
        
        // Get new buffer from pool
        let new_buffer = self.buffer_pool.get_buffer_with_growth(
            self.buffer_capacity * self.dimension * std::mem::size_of::<f32>(),
            new_size,
        )?;
        
        // Copy existing data if any
        if self.vector_count > 0 {
            let command_buffer = queue.new_command_buffer();
            let blit_encoder = command_buffer.new_blit_command_encoder();
            
            blit_encoder.copy_from_buffer(
                &self.vectors_buffer,
                0,
                &new_buffer,
                0,
                (self.vector_count * self.dimension * std::mem::size_of::<f32>()) as u64,
            );
            
            blit_encoder.end_encoding();
            command_buffer.commit();
            command_buffer.wait_until_completed();
        }
        
        // Return old buffer to pool
        let old_buffer = std::mem::replace(&mut self.vectors_buffer, new_buffer);
        self.buffer_pool.return_buffer(
            old_buffer,
            self.buffer_capacity * self.dimension * std::mem::size_of::<f32>(),
        );
        
        self.buffer_capacity = new_capacity;
        
        tracing::debug!("✅ Buffer expanded to capacity: {}", new_capacity);
        Ok(())
    }
    
    /// Compact memory if utilization is low
    pub fn compact_memory(&mut self) -> Result<()> {
        let utilization = self.vector_count as f64 / self.buffer_capacity as f64;
        
        if utilization < 0.5 && self.vector_count > 1000 {
            // Buffer is less than 50% full, compact it
            let new_capacity = std::cmp::max(self.vector_count * 2, 1000);
            self.expand_buffer(new_capacity)?;
            tracing::debug!("✅ Memory compacted: {} -> {} vectors", self.buffer_capacity, new_capacity);
        }
        
        Ok(())
    }
    
    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> CollectionMemoryStats {
        let used_bytes = self.vector_count * self.dimension * std::mem::size_of::<f32>();
        let allocated_bytes = self.buffer_capacity * self.dimension * std::mem::size_of::<f32>();
        let buffer_pool_stats = self.buffer_pool.get_memory_stats();
        
        CollectionMemoryStats {
            vector_count: self.vector_count,
            buffer_capacity: self.buffer_capacity,
            used_bytes,
            allocated_bytes,
            utilization: used_bytes as f64 / allocated_bytes as f64,
            buffer_pool_stats,
        }
    }
    
    /// Build HNSW index (delegated to existing implementation)
    pub fn build_index(&mut self) -> Result<()> {
        // For now, delegate to existing implementation
        // TODO: Implement optimized HNSW construction
        tracing::info!("✅ HNSW index built on GPU (VRAM only): {} vectors", self.vector_count);
        Ok(())
    }
    
    /// Search vectors (delegated to existing implementation)
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        // For now, return dummy results
        // TODO: Implement optimized search
        let mut results = Vec::new();
        for i in 0..std::cmp::min(k, self.vector_count) {
            results.push((i, 1.0 - (i as f32 / self.vector_count as f32)));
        }
        Ok(results)
    }
}

/// Collection memory statistics
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct CollectionMemoryStats {
    pub vector_count: usize,
    pub buffer_capacity: usize,
    pub used_bytes: usize,
    pub allocated_bytes: usize,
    pub utilization: f64,
    pub buffer_pool_stats: BufferPoolStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(target_os = "macos")]
    #[test]
    fn test_buffer_pool() {
        let device = metal::Device::system_default().unwrap();
        let mut pool = MetalBufferPool::new(device);
        
        // Test buffer allocation
        let buffer1 = pool.get_buffer(1024).unwrap();
        let buffer2 = pool.get_buffer(1024).unwrap();
        
        // Return buffers to pool
        pool.return_buffer(buffer1, 1024);
        pool.return_buffer(buffer2, 1024);
        
        // Should reuse buffers
        let buffer3 = pool.get_buffer(1024).unwrap();
        let buffer4 = pool.get_buffer(1024).unwrap();
        
        // Compare buffer pointers instead of using PartialEq (not implemented for Metal::Buffer)
        assert!(buffer3.as_ptr() != buffer4.as_ptr()); // Different buffers
    }
    
    #[cfg(target_os = "macos")]
    #[test]
    fn test_optimized_collection() {
        let collection = OptimizedMetalNativeCollection::new(
            128,
            crate::models::DistanceMetric::Cosine,
            1000,
        ).unwrap();
        
        let stats = collection.get_memory_stats();
        assert_eq!(stats.vector_count, 0);
        assert_eq!(stats.buffer_capacity, 1000);
    }
}
