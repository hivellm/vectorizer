//! DirectX 12 Backend Implementation
//!
//! Provides DirectX 12 GPU acceleration support for Windows:
//! - NVIDIA GPUs
//! - AMD GPUs
//! - Intel GPUs

use crate::error::{Result, VectorizerError};
use tracing::{info, debug};

#[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
use wgpu::{Instance, Adapter, Device, Queue, Backends};

/// DirectX 12 backend configuration
#[derive(Debug, Clone)]
pub struct DirectX12Config {
    /// Enable DirectX 12 backend
    pub enabled: bool,
    /// Use hardware acceleration
    pub hardware_acceleration: bool,
}

impl Default for DirectX12Config {
    fn default() -> Self {
        Self {
            enabled: true,
            hardware_acceleration: true,
        }
    }
}

/// DirectX 12 GPU backend
#[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
pub struct DirectX12Backend {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    gpu_vendor: GpuVendor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

#[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
impl DirectX12Backend {
    /// Create a new DirectX 12 backend instance
    pub async fn new(config: DirectX12Config) -> Result<Self> {
        info!("🪟 Initializing DirectX 12 backend...");
        
        if !config.enabled {
            return Err(VectorizerError::Other("DirectX 12 backend is disabled".to_string()));
        }
        
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::DX12,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: !config.hardware_acceleration,
            })
            .await
            .map_err(|e| VectorizerError::Other(format!("No DirectX 12 adapter found: {:?}", e)))?;
        
        let adapter_info = adapter.get_info();
        let gpu_vendor = GpuVendor::from_vendor_id(adapter_info.vendor);
        
        info!("✅ DirectX 12 GPU detected: {}", adapter_info.name);
        debug!("   Vendor: {:?}", gpu_vendor);
        debug!("   Device Type: {:?}", adapter_info.device_type);
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("DirectX 12 Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| VectorizerError::Other(format!("Failed to create DirectX 12 device: {}", e)))?;
        
        info!("✅ DirectX 12 backend initialized successfully");
        
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            gpu_vendor,
        })
    }
    
    /// Get GPU vendor
    pub fn gpu_vendor(&self) -> GpuVendor {
        self.gpu_vendor
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

impl GpuVendor {
    pub fn from_vendor_id(vendor_id: u32) -> Self {
        match vendor_id {
            0x10DE => Self::Nvidia, // NVIDIA
            0x1002 => Self::Amd,    // AMD
            0x8086 => Self::Intel,  // Intel
            _ => Self::Other,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
            Self::Other => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dx12_config_default() {
        let config = DirectX12Config::default();
        assert!(config.enabled);
        assert!(config.hardware_acceleration);
    }
    
    #[test]
    fn test_gpu_vendor_from_id() {
        assert_eq!(GpuVendor::from_vendor_id(0x10DE), GpuVendor::Nvidia);
        assert_eq!(GpuVendor::from_vendor_id(0x1002), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_vendor_id(0x8086), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_vendor_id(0x9999), GpuVendor::Other);
    }
}

