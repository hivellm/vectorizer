//! Vulkan Backend Implementation
//!
//! Provides Vulkan GPU acceleration support for:
//! - AMD GPUs (optimized)
//! - NVIDIA GPUs
//! - Intel GPUs
//! - Any Vulkan-compatible GPU

use crate::error::{Result, VectorizerError};
use tracing::{info, debug};

#[cfg(feature = "wgpu-gpu")]
use wgpu::{Instance, Adapter, Device, Queue, Backends};

/// Vulkan backend configuration
#[derive(Debug, Clone)]
pub struct VulkanConfig {
    /// Enable Vulkan backend
    pub enabled: bool,
    /// Prefer AMD GPUs
    pub prefer_amd: bool,
    /// Minimum Vulkan version required
    pub min_version: (u32, u32),
}

impl Default for VulkanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefer_amd: true, // AMD works best with Vulkan
            min_version: (1, 2), // Vulkan 1.2+
        }
    }
}

/// Vulkan GPU backend
#[cfg(feature = "wgpu-gpu")]
pub struct VulkanBackend {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    gpu_info: GpuInfo,
}

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor_id: u32,
    pub device_type: String,
    pub is_amd: bool,
    pub is_nvidia: bool,
    pub is_intel: bool,
}

#[cfg(feature = "wgpu-gpu")]
impl VulkanBackend {
    /// Create a new Vulkan backend instance
    pub async fn new(config: VulkanConfig) -> Result<Self> {
        info!("🔥 Initializing Vulkan backend...");
        
        if !config.enabled {
            return Err(VectorizerError::Other("Vulkan backend is disabled".to_string()));
        }
        
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| VectorizerError::Other(format!("No Vulkan adapter found: {:?}", e)))?;
        
        let adapter_info = adapter.get_info();
        let gpu_info = GpuInfo::from_adapter_info(&adapter_info);
        
        info!("✅ Vulkan GPU detected: {}", gpu_info.name);
        debug!("   Vendor ID: 0x{:X}", gpu_info.vendor_id);
        debug!("   Device Type: {}", gpu_info.device_type);
        
        if gpu_info.is_amd {
            info!("   🔥 AMD GPU detected (optimized for Vulkan)");
        } else if gpu_info.is_nvidia {
            info!("   ⚡ NVIDIA GPU detected");
        } else if gpu_info.is_intel {
            info!("   🔷 Intel GPU detected");
        }
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Vulkan Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| VectorizerError::Other(format!("Failed to create Vulkan device: {}", e)))?;
        
        info!("✅ Vulkan backend initialized successfully");
        
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            gpu_info,
        })
    }
    
    /// Get GPU information
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
    }
    
    /// Check if this is an AMD GPU
    pub fn is_amd_gpu(&self) -> bool {
        self.gpu_info.is_amd
    }
    
    /// Check if this is an NVIDIA GPU
    pub fn is_nvidia_gpu(&self) -> bool {
        self.gpu_info.is_nvidia
    }
    
    /// Get device reference
    pub fn device(&self) -> &Device {
        &self.device
    }
    
    /// Get queue reference
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

impl GpuInfo {
    #[cfg(feature = "wgpu-gpu")]
    pub fn from_adapter_info(info: &wgpu::AdapterInfo) -> Self {
        let vendor_id = info.vendor;
        
        // AMD Vendor ID: 0x1002
        let is_amd = vendor_id == 0x1002 || 
                     info.name.to_lowercase().contains("amd") ||
                     info.name.to_lowercase().contains("radeon");
        
        // NVIDIA Vendor ID: 0x10DE
        let is_nvidia = vendor_id == 0x10DE || 
                        info.name.to_lowercase().contains("nvidia") ||
                        info.name.to_lowercase().contains("geforce");
        
        // Intel Vendor ID: 0x8086
        let is_intel = vendor_id == 0x8086 || 
                       info.name.to_lowercase().contains("intel");
        
        Self {
            name: info.name.clone(),
            vendor_id,
            device_type: format!("{:?}", info.device_type),
            is_amd,
            is_nvidia,
            is_intel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vulkan_config_default() {
        let config = VulkanConfig::default();
        assert!(config.enabled);
        assert!(config.prefer_amd);
        assert_eq!(config.min_version, (1, 2));
    }
}

