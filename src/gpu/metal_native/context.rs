//! # Metal Native Context
//!
//! Unified Metal context for all Metal Native operations.
//! This module provides a single source of truth for Metal device and command queue management.

use metal::{Device as MetalDevice, CommandQueue, MTLGPUFamily, MTLSize, Library};
use std::sync::Arc;
use crate::error::VectorizerError;
use tracing::{info, debug};

/// Metal Native Context - Single source of truth
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct MetalNativeContext {
    device: MetalDevice,
    command_queue: CommandQueue,
    library: Library,
}

#[cfg(target_os = "macos")]
impl MetalNativeContext {
    /// Create new Metal native context
    pub fn new() -> Result<Self, VectorizerError> {
        let device = MetalDevice::system_default()
            .ok_or_else(|| VectorizerError::Other("No Metal device available".to_string()))?;

        let command_queue = device.new_command_queue();

        // Load Metal library with HNSW shaders
        let library = Self::load_metal_library(&device)?;

        debug!("✅ Metal native context created: {}", device.name());

        Ok(Self {
            device,
            command_queue,
            library,
        })
    }
    
    /// Get Metal device
    pub fn device(&self) -> &MetalDevice {
        &self.device
    }
    
    /// Get command queue
    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }
    
    /// Get device name
    pub fn device_name(&self) -> String {
        self.device.name().to_string()
    }
    
    /// Check if device supports Metal Performance Shaders
    pub fn supports_mps(&self) -> bool {
        // Check if device supports MPS (Metal Performance Shaders)
        // This is a simplified check - in practice you'd check specific MPS features
        self.device.supports_family(MTLGPUFamily::Apple7)
    }
    
    /// Get maximum threadgroup size for compute shaders
    pub fn max_threadgroup_size(&self) -> MTLSize {
        self.device.max_threads_per_threadgroup()
    }
    
    /// Get maximum buffer size
    pub fn max_buffer_size(&self) -> u64 {
        // Most Metal devices support very large buffers
        // Return a conservative limit of 1GB
        1024 * 1024 * 1024
    }

    /// Get Metal library
    pub fn library(&self) -> &Library {
        &self.library
    }

    /// Load Metal library with HNSW shaders
    fn load_metal_library(device: &MetalDevice) -> Result<Library, VectorizerError> {
        // Load the Metal shader source
        let shader_source = include_str!("../shaders/metal_hnsw.metal");

        let options = metal::CompileOptions::new();
        let library = device.new_library_with_source(shader_source, &options)
            .map_err(|e| VectorizerError::Other(format!("Failed to compile Metal shaders: {:?}", e)))?;

        debug!("✅ Metal library loaded with {} functions", library.function_names().len());
        Ok(library)
    }
}
