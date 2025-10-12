//! # Metal Native Vector Storage
//!
//! High-performance vector storage using Metal GPU acceleration.
//! All vector data is stored in VRAM for maximum efficiency.

use metal::{Buffer as MetalBuffer, Device as MetalDevice, MTLResourceOptions, MTLStorageMode, MTLCPUCacheMode};
use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{Result, VectorizerError};
use crate::models::Vector;
use super::context::MetalNativeContext;
use tracing::{info, warn, debug};

/// Vector metadata structure
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct VectorMetadata {
    pub original_id: String,
    pub index: usize,
    pub timestamp: u64,
}

/// Metal Native Vector Storage
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MetalNativeVectorStorage {
    context: Arc<MetalNativeContext>,
    vectors_buffer: MetalBuffer,
    metadata_buffer: MetalBuffer,
    vector_count: usize,
    buffer_capacity: usize, // Total capacity in vectors
    dimension: usize,
    vector_id_map: HashMap<String, usize>,
    index_to_id: Vec<String>, // Maps index to original ID
    vector_metadata: HashMap<String, VectorMetadata>, // Maps ID to metadata
}

#[cfg(target_os = "macos")]
impl MetalNativeVectorStorage {
    /// Create new Metal native vector storage
    pub fn new(context: Arc<MetalNativeContext>, dimension: usize) -> Result<Self> {
        let device = context.device();
        
        // Calculate initial capacity (minimum 1024 vectors or 1MB worth)
        let min_vectors = 1024;
        let min_bytes = 1024 * 1024; // 1MB
        let min_vectors_by_bytes = min_bytes / (dimension * std::mem::size_of::<f32>());
        let initial_capacity = min_vectors.max(min_vectors_by_bytes);
        
        let initial_size = initial_capacity
            .checked_mul(dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("Initial buffer size calculation overflow".to_string()))?;
        
        // Create vectors buffer (VRAM only, no CPU access)
        let vectors_buffer = device.new_buffer(
            initial_size as u64,
            MTLResourceOptions::StorageModePrivate, // VRAM only, fastest
        );
        
        // Create metadata buffer (VRAM only)
        let metadata_buffer = device.new_buffer(
            initial_capacity as u64 * 256, // 256 bytes per vector metadata
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        debug!("✅ Metal native vector storage created (VRAM only) with capacity: {}", initial_capacity);
        
        Ok(Self {
            context,
            vectors_buffer,
            metadata_buffer,
            vector_count: 0,
            buffer_capacity: initial_capacity,
            dimension,
            vector_id_map: HashMap::new(),
            index_to_id: Vec::new(),
            vector_metadata: HashMap::new(),
        })
    }
    
    /// Add vector to storage (VRAM only)
    pub fn add_vector(&mut self, vector: &Vector) -> Result<usize> {
        // Validate vector ID is unique
        if self.vector_id_map.contains_key(&vector.id) {
            return Err(VectorizerError::Other(format!("Vector with ID '{}' already exists", vector.id)));
        }
        
        // Validate vector dimension
        if vector.data.len() != self.dimension {
            return Err(VectorizerError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.data.len(),
            });
        }
        
        // Validate all values are finite (no NaN/Infinity)
        for (i, &value) in vector.data.iter().enumerate() {
            if !value.is_finite() {
                return Err(VectorizerError::Other(format!("Vector contains non-finite value at index {}: {}", i, value)));
            }
        }
        
        // Validate ID length
        if vector.id.len() > 256 {
            return Err(VectorizerError::Other("Vector ID too long (max 256 chars)".to_string()));
        }
        
        // Check if we need to expand buffer
        if self.vector_count >= self.buffer_capacity {
            self.expand_buffer()?;
        }
        
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Upload new vector data directly to existing buffer
        let vector_data = &vector.data;
        let offset = self.vector_count
            .checked_mul(self.dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("Offset calculation overflow".to_string()))?;
        
        // Create staging buffer for upload
        let staging_size = self.dimension
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| VectorizerError::Other("Staging size calculation overflow".to_string()))?;
        
        let staging_buffer = device.new_buffer_with_data(
            vector_data.as_ptr() as *const std::ffi::c_void,
            staging_size as u64,
            MTLResourceOptions::StorageModeShared, // CPU accessible for upload
        );
        
        // Copy from staging to VRAM buffer
        let command_buffer = queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        blit_encoder.copy_from_buffer(
            &staging_buffer,
            0,
            &self.vectors_buffer,
            offset as u64,
            staging_size as u64,
        );
        
        blit_encoder.end_encoding();
        
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Update state with proper ID tracking
        let index = self.vector_count;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Store metadata
        let metadata = VectorMetadata {
            original_id: vector.id.clone(),
            index,
            timestamp,
        };
        
        self.vector_id_map.insert(vector.id.clone(), index);
        self.index_to_id.push(vector.id.clone());
        self.vector_metadata.insert(vector.id.clone(), metadata);
        self.vector_count += 1;
        
        debug!("✅ Vector added to VRAM: {} (total: {})", vector.id, self.vector_count);
        Ok(index)
    }
    
    /// Expand buffer with adaptive growth strategy
    fn expand_buffer(&mut self) -> Result<()> {
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Calculate new capacity with adaptive growth
        let growth_factor = if self.buffer_capacity < 1000 {
            2.0 // Double for small buffers
        } else if self.buffer_capacity < 10000 {
            1.5 // 50% growth for medium buffers
        } else {
            1.2 // 20% growth for large buffers
        };
        
        let new_capacity = (self.buffer_capacity as f32 * growth_factor).ceil() as usize;
        let new_capacity = new_capacity.max(self.vector_count + 1); // Ensure we can fit at least one more
        
        // Check VRAM limits (conservative 1GB limit)
        let new_size = new_capacity
            .checked_mul(self.dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("New buffer size calculation overflow".to_string()))?;
        
        const VRAM_LIMIT_BYTES: usize = 1024 * 1024 * 1024; // 1GB
        if new_size > VRAM_LIMIT_BYTES {
            return Err(VectorizerError::Other("VRAM limit exceeded".to_string()));
        }
        
        // Create new larger buffer
        let new_vectors_buffer = device.new_buffer(
            new_size as u64,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        // Copy existing data to new buffer
        if self.vector_count > 0 {
            let command_buffer = queue.new_command_buffer();
            let blit_encoder = command_buffer.new_blit_command_encoder();
            
            let copy_size = self.vector_count
                .checked_mul(self.dimension)
                .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
                .ok_or_else(|| VectorizerError::Other("Copy size calculation overflow".to_string()))?;
            
            blit_encoder.copy_from_buffer(
                &self.vectors_buffer,
                0,
                &new_vectors_buffer,
                0,
                copy_size as u64,
            );
            
            blit_encoder.end_encoding();
            command_buffer.commit();
        command_buffer.wait_until_completed();
        }
        
        // Update metadata buffer size
        let new_metadata_buffer = device.new_buffer(
            new_capacity as u64 * 256, // 256 bytes per vector metadata
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        // Update state
        self.vectors_buffer = new_vectors_buffer;
        self.metadata_buffer = new_metadata_buffer;
        self.buffer_capacity = new_capacity;
        
        debug!("✅ Buffer expanded: {} -> {} vectors (growth: {:.1}x)", 
               self.vector_count, new_capacity, growth_factor);
        
        Ok(())
    }
    
    /// Get vector by index (from VRAM)
    pub fn get_vector(&self, index: usize) -> Result<Vector> {
        if index >= self.vector_count {
            return Err(VectorizerError::Other(format!("Vector index {} out of range", index)));
        }
        
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Create staging buffer for readback
        let staging_buffer = device.new_buffer(
            (self.dimension * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared, // CPU accessible for readback
        );
        
        // Copy from VRAM to staging
        let command_buffer = queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        let offset = index
            .checked_mul(self.dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("Get vector offset calculation overflow".to_string()))?;
        blit_encoder.copy_from_buffer(
            &self.vectors_buffer,
            offset as u64,
            &staging_buffer,
            0,
            (self.dimension * std::mem::size_of::<f32>()) as u64,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Read from staging buffer with safe pointer access
        let contents_ptr = staging_buffer.contents();
        if contents_ptr.is_null() {
            return Err(VectorizerError::Other("Failed to get buffer contents".to_string()));
        }
        
        let contents_ptr = contents_ptr as *const f32;
        let slice = unsafe {
            std::slice::from_raw_parts(contents_ptr, self.dimension)
        };
        
        // Copy data to safe Vec to avoid dangling pointer issues
        let mut safe_data = Vec::with_capacity(self.dimension);
        safe_data.extend_from_slice(slice);
        
        // Get real vector ID
        let real_id = if index < self.index_to_id.len() {
            self.index_to_id[index].clone()
        } else {
            format!("vector_{}", index) // Fallback for invalid index
        };
        
        Ok(Vector {
            id: real_id,
            data: safe_data,
            payload: None,
        })
    }
    
    /// Get vector count
    pub fn vector_count(&self) -> usize {
        self.vector_count
    }
    
    /// Add multiple vectors in batch (optimized)
    pub fn add_vectors_batch(&mut self, vectors: &[Vector]) -> Result<Vec<usize>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }
        
        // Validate all vectors before processing
        for (i, vector) in vectors.iter().enumerate() {
            // Validate vector ID is unique
            if self.vector_id_map.contains_key(&vector.id) {
                return Err(VectorizerError::Other(format!("Vector with ID '{}' already exists", vector.id)));
            }
            
            // Validate vector dimension
            if vector.data.len() != self.dimension {
                return Err(VectorizerError::DimensionMismatch {
                    expected: self.dimension,
                    actual: vector.data.len(),
                });
            }
            
            // Validate all values are finite (no NaN/Infinity)
            for (j, &value) in vector.data.iter().enumerate() {
                if !value.is_finite() {
                    return Err(VectorizerError::Other(format!("Vector {} contains non-finite value at index {}: {}", i, j, value)));
                }
            }
            
            // Validate ID length
            if vector.id.len() > 256 {
                return Err(VectorizerError::Other(format!("Vector {} ID too long (max 256 chars)", i)));
            }
        }
        
        // Check if we need to expand buffer
        if self.vector_count + vectors.len() > self.buffer_capacity {
            let required_capacity = self.vector_count + vectors.len();
            self.expand_buffer_to_capacity(required_capacity)?;
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
            MTLResourceOptions::StorageModeShared, // CPU accessible for upload
        );
        
        // Calculate offset for batch upload
        let batch_offset = self.vector_count
            .checked_mul(self.dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("Batch offset calculation overflow".to_string()))?;
        
        // Copy all vectors in single operation
        let command_buffer = queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        blit_encoder.copy_from_buffer(
            &staging_buffer,
            0,
            &self.vectors_buffer,
            batch_offset as u64,
            staging_data.len() as u64,
        );
        
        blit_encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        // Update state with proper ID tracking
        let mut indices = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        for (i, vector) in vectors.iter().enumerate() {
            let index = self.vector_count + i;
            
            // Store metadata
            let metadata = VectorMetadata {
                original_id: vector.id.clone(),
                index,
                timestamp,
            };
            
            self.vector_id_map.insert(vector.id.clone(), index);
            self.index_to_id.push(vector.id.clone());
            self.vector_metadata.insert(vector.id.clone(), metadata);
            indices.push(index);
        }
        self.vector_count += vectors.len();
        
        debug!("✅ Added {} vectors in batch (total: {})", vectors.len(), self.vector_count);
        Ok(indices)
    }
    
    /// Expand buffer to specific capacity
    fn expand_buffer_to_capacity(&mut self, required_capacity: usize) -> Result<()> {
        let device = self.context.device();
        let queue = self.context.command_queue();
        
        // Calculate new capacity with adaptive growth
        let growth_factor = if self.buffer_capacity < 1000 {
            2.0 // Double for small buffers
        } else if self.buffer_capacity < 10000 {
            1.5 // 50% growth for medium buffers
        } else {
            1.2 // 20% growth for large buffers
        };
        
        let new_capacity = (self.buffer_capacity as f32 * growth_factor).ceil() as usize;
        let new_capacity = new_capacity.max(required_capacity);
        
        // Check VRAM limits (conservative 1GB limit)
        let new_size = new_capacity
            .checked_mul(self.dimension)
            .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
            .ok_or_else(|| VectorizerError::Other("New buffer size calculation overflow".to_string()))?;
        
        const VRAM_LIMIT_BYTES: usize = 1024 * 1024 * 1024; // 1GB
        if new_size > VRAM_LIMIT_BYTES {
            return Err(VectorizerError::Other("VRAM limit exceeded".to_string()));
        }
        
        // Create new larger buffer
        let new_vectors_buffer = device.new_buffer(
            new_size as u64,
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        // Copy existing data to new buffer
        if self.vector_count > 0 {
            let command_buffer = queue.new_command_buffer();
            let blit_encoder = command_buffer.new_blit_command_encoder();
            
            let copy_size = self.vector_count
                .checked_mul(self.dimension)
                .and_then(|x| x.checked_mul(std::mem::size_of::<f32>()))
                .ok_or_else(|| VectorizerError::Other("Copy size calculation overflow".to_string()))?;
            
            blit_encoder.copy_from_buffer(
                &self.vectors_buffer,
                0,
                &new_vectors_buffer,
                0,
                copy_size as u64,
            );
            
            blit_encoder.end_encoding();
            command_buffer.commit();
        command_buffer.wait_until_completed();
        }
        
        // Update metadata buffer size
        let new_metadata_buffer = device.new_buffer(
            new_capacity as u64 * 256, // 256 bytes per vector metadata
            MTLResourceOptions::StorageModePrivate, // VRAM only
        );
        
        // Update state
        self.vectors_buffer = new_vectors_buffer;
        self.metadata_buffer = new_metadata_buffer;
        self.buffer_capacity = new_capacity;
        
        debug!("✅ Buffer expanded to capacity: {} vectors (growth: {:.1}x)", 
               new_capacity, growth_factor);
        
        Ok(())
    }
    
    /// Get dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    
    /// Get vector metadata by ID
    pub fn get_vector_metadata(&self, id: &str) -> Option<&VectorMetadata> {
        self.vector_metadata.get(id)
    }
    
    /// Get vector ID by index
    pub fn get_vector_id(&self, index: usize) -> Option<&str> {
        self.index_to_id.get(index).map(|s| s.as_str())
    }
    
    /// Get all vector IDs
    pub fn get_all_vector_ids(&self) -> Vec<String> {
        self.index_to_id.clone()
    }
}
