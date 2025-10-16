//! GPU Backend Detection System
//!
//! Automatically detects available GPU backends and selects the best one
//! based on platform, hardware, and performance characteristics.

use tracing::{debug, info, warn};

use crate::error::Result;

/// Supported GPU backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackendType {
    /// Apple Metal (M1/M2/M3/M4 Silicon)
    Metal,
    /// Vulkan (AMD/NVIDIA/Intel/Universal)
    Vulkan,
    /// DirectX 12 (Windows)
    DirectX12,
    /// GPU (NVIDIA native, not via wgpu)
    GpuNative,
    /// CPU-only (no GPU acceleration)
    Cpu,
}

impl GpuBackendType {
    /// Get priority of backend (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Metal => 0,      // Best for Apple Silicon
            Self::Vulkan => 1,     // Universal, AMD optimized
            Self::DirectX12 => 2,  // Windows native
            Self::Cpu => 255,      // Last resort
        }
    }

    /// Check if backend is theoretically available on current platform
    pub fn is_platform_compatible(&self) -> bool {
        match self {
            Self::Metal => cfg!(all(target_os = "macos", target_arch = "aarch64")),
            Self::Vulkan => true, // Universal (try on all platforms)
            Self::DirectX12 => cfg!(target_os = "windows"),
            Self::Cpu => true, // Always available
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Metal => "Metal",
            Self::Vulkan => "Vulkan",
            Self::DirectX12 => "DirectX 12",
            Self::Cpu => "CPU",
        }
    }

    /// Get emoji icon for backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Metal => "🍎",
            Self::Vulkan => "🔥",
            Self::DirectX12 => "🪟",
            Self::Cpu => "💻",
        }
    }
}

impl std::fmt::Display for GpuBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

/// Detect all available GPU backends on the current system
pub fn detect_available_backends() -> Vec<GpuBackendType> {
    info!("🔍 Detecting available GPU backends...");
    let mut available = Vec::new();

    // Check Metal (macOS Apple Silicon only)
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
    {
        debug!("Checking Metal availability...");
        if try_detect_metal() {
            info!("✅ Metal backend available");
            available.push(GpuBackendType::Metal);
        } else {
            debug!("❌ Metal backend not available");
        }
    }

    // Check Vulkan (universal)
    #[cfg(feature = "wgpu-gpu")]
    {
        debug!("Checking Vulkan availability...");
        if try_detect_vulkan() {
            info!("✅ Vulkan backend available");
            available.push(GpuBackendType::Vulkan);
        } else {
            debug!("❌ Vulkan backend not available");
        }
    }

    // Check DirectX 12 (Windows only)
    #[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
    {
        debug!("Checking DirectX 12 availability...");
        if try_detect_dx12() {
            info!("✅ DirectX 12 backend available");
            available.push(GpuBackendType::DirectX12);
        } else {
            debug!("❌ DirectX 12 backend not available");
        }
    }

    // Check CUDA (if feature enabled)
    // CUDA support removed - use Vulkan for NVIDIA GPUs
    #[cfg(feature = "cuda")]
    {
        debug!("CUDA feature detected but native CUDA backend is deprecated. Use Vulkan instead.");
    }

    // CPU always available as fallback
    available.push(GpuBackendType::Cpu);

    info!("📊 Available backends: {:?}", available);
    available
}

/// Select the best backend from available options
pub fn select_best_backend(available: &[GpuBackendType]) -> GpuBackendType {
    if available.is_empty() {
        warn!("No backends available, defaulting to CPU");
        return GpuBackendType::Cpu;
    }

    // Sort by priority and return first
    let mut sorted = available.to_vec();
    sorted.sort_by_key(|b| b.priority());

    let best = sorted[0];
    info!("🎯 Selected backend: {}", best);
    best
}

/// Try to detect Metal backend (macOS Apple Silicon)
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
fn try_detect_metal() -> bool {
    use wgpu::{Backends, Instance, RequestAdapterOptions};

    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::METAL,
        ..Default::default()
    });

    match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Ok(adapter) => {
            let info = adapter.get_info();
            debug!(
                "Metal adapter found: {} ({:?})",
                info.name, info.device_type
            );
            true
        }
        Err(_) => false,
    }
}

/// Try to detect Vulkan backend
#[cfg(feature = "wgpu-gpu")]
fn try_detect_vulkan() -> bool {
    use wgpu::{Backends, Instance, RequestAdapterOptions};

    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::VULKAN,
        ..Default::default()
    });

    match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Ok(adapter) => {
            let info = adapter.get_info();
            debug!(
                "Vulkan adapter found: {} ({:?})",
                info.name, info.device_type
            );
            true
        }
        Err(_) => false,
    }
}

/// Try to detect DirectX 12 backend (Windows)
#[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
fn try_detect_dx12() -> bool {
    use wgpu::{Backends, Instance, RequestAdapterOptions};

    let instance = Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::DX12,
        ..Default::default()
    });

    match pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) {
        Ok(adapter) => {
            let info = adapter.get_info();
            debug!(
                "DirectX 12 adapter found: {} ({:?})",
                info.name, info.device_type
            );
            true
        }
        Err(_) => false,
    }
}

/// Try to detect CUDA backend
#[cfg(feature = "cuda")]
fn try_detect_cuda() -> bool {
    // For now, just check if CUDA feature is enabled
    // TODO: Actually check for CUDA runtime availability
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_priority() {
        assert!(GpuBackendType::Metal.priority() < GpuBackendType::Vulkan.priority());
        assert!(GpuBackendType::Vulkan.priority() < GpuBackendType::DirectX12.priority());
        assert!(GpuBackendType::DirectX12.priority() < GpuBackendType::Cpu.priority());
    }

    #[test]
    fn test_backend_display() {
        assert_eq!(GpuBackendType::Metal.to_string(), "🍎 Metal");
        assert_eq!(GpuBackendType::Vulkan.to_string(), "🔥 Vulkan");
        assert_eq!(GpuBackendType::DirectX12.to_string(), "🪟 DirectX 12");
    }

    #[test]
    fn test_select_best_backend() {
        let backends = vec![
            GpuBackendType::Cpu,
            GpuBackendType::Vulkan,
            GpuBackendType::Metal,
        ];

        let best = select_best_backend(&backends);
        assert_eq!(best, GpuBackendType::Metal); // Metal has highest priority
    }

    #[test]
    fn test_detect_available_backends() {
        let available = detect_available_backends();

        // CPU should always be available
        assert!(available.contains(&GpuBackendType::Cpu));

        // At least one backend should be available
        assert!(!available.is_empty());
    }
}
