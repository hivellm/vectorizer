//! Native Metal API helpers for synchronous buffer operations
//!
//! This module provides direct access to Apple's Metal API for operations
//! that are difficult or impossible with wgpu's async model, specifically
//! synchronous buffer read-back from GPU to CPU.
//!
//! Only compiled on macOS targets.

#![cfg(target_os = "macos")]

use crate::error::Result;
use metal::{
    Buffer as MetalBuffer, CommandQueue, Device as MetalDevice, MTLResourceOptions,
};
use std::sync::Arc;
use tracing::{debug, trace};

/// Native Metal buffer reader for synchronous GPU-to-CPU operations
///
/// Provides direct access to Metal API for buffer read-back without
/// the async complexity of wgpu. Uses Metal's synchronous wait mechanisms.
#[derive(Debug, Clone)]
pub struct MetalBufferReader {
    device: MetalDevice,
    command_queue: CommandQueue,
}

impl MetalBufferReader {
    /// Create a new Metal buffer reader from a Metal device
    pub fn new(device: MetalDevice) -> Result<Self> {
        let command_queue = device.new_command_queue();
        
        debug!("Created MetalBufferReader with native Metal device");
        
        Ok(Self {
            device,
            command_queue,
        })
    }
    
    /// Read buffer synchronously from GPU to CPU
    ///
    /// Uses Metal's native blit encoder and synchronous wait to ensure
    /// data is available immediately after this function returns.
    ///
    /// # Arguments
    /// * `source_buffer` - GPU buffer to read from
    /// * `size` - Number of bytes to read
    ///
    /// # Returns
    /// Raw bytes from the GPU buffer
    pub fn read_buffer_sync(&self, source_buffer: &MetalBuffer, size: usize) -> Result<Vec<u8>> {
        trace!("Reading {} bytes from Metal buffer synchronously", size);
        
        // Create staging buffer in shared memory (CPU-accessible)
        // StorageModeShared allows both CPU and GPU access
        let staging_buffer = self.device.new_buffer(
            size as u64,
            MTLResourceOptions::StorageModeShared | MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        
        // Create command buffer and blit encoder for GPU-to-GPU copy
        let command_buffer = self.command_queue.new_command_buffer();
        let blit_encoder = command_buffer.new_blit_command_encoder();
        
        // Copy from GPU buffer to staging buffer
        blit_encoder.copy_from_buffer(
            source_buffer,
            0,  // source offset
            &staging_buffer,
            0,  // destination offset
            size as u64,
        );
        
        // ✅ KEY: Synchronize to ensure staging buffer is ready for CPU access
        blit_encoder.synchronize_resource(&staging_buffer);
        blit_encoder.end_encoding();
        
        // ✅ Commit and wait synchronously (BLOCKING)
        command_buffer.commit();
        command_buffer.wait_until_completed();
        
        trace!("Metal command buffer completed, reading from staging");
        
        // ✅ Read from staging buffer (now in CPU-accessible memory)
        let contents_ptr = staging_buffer.contents();
        let data_slice = unsafe {
            std::slice::from_raw_parts(contents_ptr as *const u8, size)
        };
        
        // Copy to owned Vec to avoid lifetime issues
        let result = data_slice.to_vec();
        
        debug!("Successfully read {} bytes from Metal buffer", result.len());
        Ok(result)
    }
    
    /// Read buffer and interpret as f32 values
    ///
    /// # Arguments
    /// * `source_buffer` - GPU buffer to read from
    /// * `count` - Number of f32 elements to read
    pub fn read_buffer_sync_f32(
        &self,
        source_buffer: &MetalBuffer,
        count: usize,
    ) -> Result<Vec<f32>> {
        let size = count * std::mem::size_of::<f32>();
        let bytes = self.read_buffer_sync(source_buffer, size)?;
        
        // Cast bytes to f32
        #[cfg(feature = "metal-native")]
        let floats: Vec<f32> = {
            let len = bytes.len() / 4;
            let mut result = Vec::with_capacity(len);
            for chunk in bytes.chunks_exact(4) {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                result.push(value);
            }
            result
        };
        
        #[cfg(not(feature = "metal-native"))]
        let floats: Vec<f32> = bytemuck::cast_slice(&bytes).to_vec();
        
        debug!("Read {} f32 values from Metal buffer", floats.len());
        Ok(floats)
    }
    
    /// Read buffer and interpret as u32 values
    ///
    /// # Arguments
    /// * `source_buffer` - GPU buffer to read from
    /// * `count` - Number of u32 elements to read
    pub fn read_buffer_sync_u32(
        &self,
        source_buffer: &MetalBuffer,
        count: usize,
    ) -> Result<Vec<u32>> {
        let size = count * std::mem::size_of::<u32>();
        let bytes = self.read_buffer_sync(source_buffer, size)?;
        
        // Cast bytes to u32
        #[cfg(feature = "metal-native")]
        let ints: Vec<u32> = {
            let len = bytes.len() / 4;
            let mut result = Vec::with_capacity(len);
            for chunk in bytes.chunks_exact(4) {
                let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                result.push(value);
            }
            result
        };
        
        #[cfg(not(feature = "metal-native"))]
        let ints: Vec<u32> = bytemuck::cast_slice(&bytes).to_vec();
        
        debug!("Read {} u32 values from Metal buffer", ints.len());
        Ok(ints)
    }
    
    /// Get reference to the underlying Metal device
    pub fn device(&self) -> &MetalDevice {
        &self.device
    }
    
    /// Get reference to the command queue
    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }
}

/// Create a Metal device directly from system default
///
/// This is simpler than trying to extract from wgpu and avoids
/// dealing with wgpu-hal's internal API.
pub fn create_system_metal_device() -> Result<MetalDevice> {
    use crate::error::VectorizerError;
    
    let device = MetalDevice::system_default()
        .ok_or_else(|| VectorizerError::Other("No Metal device available on this system".to_string()))?;
    
    debug!("Created Metal device from system default");
    Ok(device)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(target_os = "macos")]
    fn test_metal_buffer_reader_creation() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let reader = MetalBufferReader::new(device);
        assert!(reader.is_ok());
    }
    
    #[test]
    #[cfg(target_os = "macos")]
    fn test_synchronous_buffer_read() {
        let device = MetalDevice::system_default().expect("No Metal device");
        let reader = MetalBufferReader::new(device.clone()).unwrap();
        
        // Create test buffer with some data
        let test_data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let buffer = device.new_buffer_with_data(
            test_data.as_ptr() as *const std::ffi::c_void,
            (test_data.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        
        // Read back synchronously
        let result = reader.read_buffer_sync_f32(&buffer, test_data.len()).unwrap();
        
        assert_eq!(result.len(), test_data.len());
        assert_eq!(result, test_data);
    }
}

