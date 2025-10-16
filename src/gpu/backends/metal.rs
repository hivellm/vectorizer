//! Metal Backend Implementation
//!
//! Provides Metal GPU acceleration support for Apple Silicon (M1/M2/M3/M4)

use crate::error::Result;
// Metal backend is already implemented in the parent gpu module
// This file exists for organizational consistency with other backends
/// Re-export Metal-specific types
pub use crate::gpu::{GpuConfig, GpuContext, MetalCollection};

/// Check if Metal is available (macOS Apple Silicon only)
pub fn is_metal_available() -> bool {
    cfg!(all(
        target_os = "macos",
        target_arch = "aarch64",
        feature = "wgpu-gpu"
    ))
}

/// Get Metal configuration optimized for Apple Silicon
pub fn metal_config() -> GpuConfig {
    GpuConfig::for_metal_silicon()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metal_availability() {
        // This will be true only on macOS Apple Silicon with wgpu-gpu feature
        let available = is_metal_available();

        #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
        assert!(available);

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu")))]
        assert!(!available);
    }
}
