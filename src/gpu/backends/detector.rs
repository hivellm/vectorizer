//! GPU Backend Detection System
//!
//! Automatically detects available GPU backends and selects the best one
//! based on platform, hardware, and performance characteristics.

use crate::error::Result;
use tracing::{info, debug, warn};

/// Supported native GPU backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuBackendType {
    /// Apple Metal Native (macOS Apple Silicon)
    Metal,
    /// CUDA (NVIDIA GPUs on Linux/Windows)
    Cuda,
    /// CPU-only (no GPU acceleration)
    Cpu,
}

impl GpuBackendType {
    /// Get priority of backend (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Metal => 0,       // Best for Apple Silicon
            Self::Cuda => 1,        // NVIDIA specific, high performance
            Self::Cpu => 255,       // Last resort
        }
    }

    /// Check if backend is theoretically available on current platform
    pub fn is_platform_compatible(&self) -> bool {
        match self {
            Self::Metal => cfg!(target_os = "macos"),
            Self::Cuda => cfg!(feature = "cuda"),
            Self::Cpu => true, // Always available
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Metal => "Metal Native",
            Self::Cuda => "CUDA",
            Self::Cpu => "CPU",
        }
    }

    /// Get emoji icon for backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Metal => "🍎",
            Self::Cuda => "🔴",
            Self::Cpu => "💻",
        }
    }
}

impl std::fmt::Display for GpuBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

/// Detect all available native GPU backends on the current system
pub fn detect_available_backends() -> Vec<GpuBackendType> {
    info!("🔍 Detecting available native GPU backends...");
    let mut available = Vec::new();

    // Check Metal Native (macOS only)
    #[cfg(target_os = "macos")]
    {
        debug!("Checking Metal Native availability...");
        if try_detect_metal_native() {
            info!("✅ Metal Native backend available");
            available.push(GpuBackendType::Metal);
        } else {
            debug!("❌ Metal Native backend not available");
        }
    }

    // Check CUDA (if feature enabled)
    #[cfg(feature = "cuda")]
    {
        debug!("Checking CUDA availability...");
        if try_detect_cuda() {
            info!("✅ CUDA backend available");
            available.push(GpuBackendType::Cuda);
        } else {
            debug!("❌ CUDA backend not available");
        }
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

/// Try to detect Metal Native backend (macOS only)
#[cfg(target_os = "macos")]
fn try_detect_metal_native() -> bool {
    // Try to create Metal Native context directly
    match crate::gpu::metal_native::MetalNativeContext::new() {
        Ok(context) => {
            let device_name = context.device_name();
            debug!("Metal Native device found: {}", device_name);
            true
        }
        Err(e) => {
            debug!("Metal Native not available: {:?}", e);
            false
        }
    }
}

/// Try to detect CUDA backend
#[cfg(feature = "cuda")]
fn try_detect_cuda() -> bool {
    // TODO: Implement proper CUDA detection
    // For now, assume CUDA is available if feature is enabled
    debug!("CUDA feature enabled, assuming availability");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backend_priority() {
        // Metal has highest priority (lowest number)
        assert!(GpuBackendType::Metal.priority() < GpuBackendType::Cuda.priority());
        assert!(GpuBackendType::Cuda.priority() < GpuBackendType::Cpu.priority());
    }
    
    #[test]
    fn test_backend_display() {
        assert_eq!(GpuBackendType::Metal.to_string(), "🍎 Metal Native");
        assert_eq!(GpuBackendType::Cuda.to_string(), "🔴 CUDA");
        assert_eq!(GpuBackendType::Cpu.to_string(), "💻 CPU");
    }
    
    #[test]
    fn test_select_best_backend() {
        let backends = vec![
            GpuBackendType::Cpu,
            GpuBackendType::Cuda,
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

