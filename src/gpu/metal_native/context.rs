//! # Metal Native Context
//!
//! Unified Metal context for all Metal Native operations.
//! This module provides a single source of truth for Metal device and command queue management.

use metal::{Device as MetalDevice, CommandQueue, MTLGPUFamily, MTLSize};
use std::sync::Arc;
use crate::error::VectorizerError;
use tracing::{info, debug};

/// Metal Native Context - Single source of truth
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct MetalNativeContext {
    device: MetalDevice,
    command_queue: CommandQueue,
}

#[cfg(target_os = "macos")]
impl MetalNativeContext {
    /// Create new Metal native context
    pub fn new() -> Result<Self, VectorizerError> {
        let device = MetalDevice::system_default()
            .ok_or_else(|| VectorizerError::Other("No Metal device available".to_string()))?;
        
        let command_queue = device.new_command_queue();
        
        debug!("✅ Metal native context created: {}", device.name());
        
        Ok(Self {
            device,
            command_queue,
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
}
