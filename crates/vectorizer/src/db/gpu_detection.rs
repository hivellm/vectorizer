//! GPU Backend Detection (Metal Only)
//!
//! This module provides automatic detection of Metal GPU backend on macOS.
//! Future support for CUDA, ROCm, and WebGPU will be added when hive-gpu supports them.
//!
//! # Current Support
//! - Metal: Apple Silicon and Intel Macs with Metal support (macOS only)
//!
//! # Future Support (Pending hive-gpu)
//! - CUDA: NVIDIA GPUs (Linux/Windows)
//! - ROCm: AMD GPUs (Linux)

use tracing::{debug, info};

/// Available GPU backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    /// Apple Metal backend (macOS only)
    Metal,
    /// CPU-only mode (no GPU)
    None,
}

impl GpuBackendType {
    /// Get human-readable name of backend
    pub fn name(&self) -> &'static str {
        match self {
            Self::Metal => "Metal",
            Self::None => "CPU",
        }
    }

    /// Get emoji icon for backend
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Metal => "🍎",
            Self::None => "💻",
        }
    }
}

/// GPU backend detector
pub struct GpuDetector;

impl GpuDetector {
    /// Detect the best available GPU backend
    ///
    /// Currently only Metal is supported (macOS only).
    /// Future versions will support CUDA, ROCm, and WebGPU.
    ///
    /// # Returns
    /// - `Metal` if on macOS with Metal support
    /// - `None` (CPU) otherwise
    ///
    /// # Example
    /// ```
    /// use vectorizer::db::gpu_detection::{GpuDetector, GpuBackendType};
    ///
    /// let backend = GpuDetector::detect_best_backend();
    /// match backend {
    ///     GpuBackendType::Metal => tracing::info!("Using Metal GPU!"),
    ///     GpuBackendType::None => tracing::info!("Using CPU"),
    /// }
    /// ```
    pub fn detect_best_backend() -> GpuBackendType {
        info!("🔍 Detecting GPU backend...");

        // Check Metal availability (macOS only)
        if Self::is_metal_available() {
            info!("✅ Metal GPU detected and enabled!");
            return GpuBackendType::Metal;
        }

        // Fallback: CPU-only mode (no logging needed, this is the default)
        GpuBackendType::None
    }

    /// Check if Metal backend is available
    ///
    /// Requires:
    /// - macOS operating system
    /// - Apple Silicon or AMD/NVIDIA GPU with Metal support
    /// - `hive-gpu` feature enabled at compile time
    ///
    /// # Returns
    /// `true` if Metal is available and functional
    pub fn is_metal_available() -> bool {
        #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
        {
            debug!("Checking Metal availability...");
            match Self::try_create_metal_context() {
                Ok(_) => {
                    debug!("✅ Metal context created successfully");
                    true
                }
                Err(e) => {
                    debug!("❌ Metal not available: {:?}", e);
                    false
                }
            }
        }
        #[cfg(not(all(feature = "hive-gpu", target_os = "macos")))]
        {
            debug!("Metal support requires macOS + 'hive-gpu' feature");
            false
        }
    }

    /// Try to create a Metal context for testing
    #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
    fn try_create_metal_context() -> Result<(), hive_gpu::HiveGpuError> {
        use hive_gpu::GpuContext;
        use hive_gpu::metal::MetalNativeContext;

        // Try to create Metal context
        let _ctx = MetalNativeContext::new()?;
        Ok(())
    }

    /// Get detailed GPU information for detected backend
    ///
    /// Returns device name, VRAM capacity, driver version if available
    pub fn get_gpu_info(backend: GpuBackendType) -> Option<GpuInfo> {
        match backend {
            GpuBackendType::None => None,
            GpuBackendType::Metal => Some(Self::query_metal_info()),
        }
    }

    /// Query Metal GPU information via hive-gpu's `GpuContext::device_info()`.
    ///
    /// Opens a [`MetalNativeContext`] purely to introspect the device and drops
    /// it before returning; no GPU work is performed. On any failure we log at
    /// `debug!` and return a [`GpuInfo`] with a clearly-marked unknown name so
    /// callers can still render a reasonable diagnostic.
    ///
    /// [`MetalNativeContext`]: hive_gpu::metal::MetalNativeContext
    #[cfg(all(feature = "hive-gpu", target_os = "macos"))]
    fn query_metal_info() -> GpuInfo {
        use hive_gpu::GpuContext;
        use hive_gpu::metal::MetalNativeContext;

        let result = MetalNativeContext::new().and_then(|ctx| ctx.device_info());

        match result {
            Ok(info) => GpuInfo {
                backend: GpuBackendType::Metal,
                device_name: info.name,
                vram_total: Some(info.total_vram_bytes as usize),
                driver_version: Some(info.driver_version),
            },
            Err(e) => {
                debug!("Failed to query Metal device info: {:?}", e);
                GpuInfo {
                    backend: GpuBackendType::Metal,
                    device_name: "Apple GPU (unknown)".to_string(),
                    vram_total: None,
                    driver_version: None,
                }
            }
        }
    }

    /// Query Metal GPU information (fallback for non-macOS)
    #[cfg(not(all(feature = "hive-gpu", target_os = "macos")))]
    fn query_metal_info() -> GpuInfo {
        GpuInfo::default()
    }
}

/// GPU device information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// Backend type
    pub backend: GpuBackendType,
    /// Device name (e.g., "Apple M1 Pro")
    pub device_name: String,
    /// Total VRAM in bytes
    pub vram_total: Option<usize>,
    /// Driver version string
    pub driver_version: Option<String>,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            backend: GpuBackendType::None,
            device_name: "Unknown".to_string(),
            vram_total: None,
            driver_version: None,
        }
    }
}

impl std::fmt::Display for GpuInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} - {}",
            self.backend.icon(),
            self.backend.name(),
            self.device_name
        )?;
        if let Some(vram) = self.vram_total {
            write!(
                f,
                " ({:.1} GB VRAM)",
                vram as f64 / 1024.0 / 1024.0 / 1024.0
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_name() {
        assert_eq!(GpuBackendType::Metal.name(), "Metal");
        assert_eq!(GpuBackendType::None.name(), "CPU");
    }

    #[test]
    fn test_backend_type_icon() {
        assert_eq!(GpuBackendType::Metal.icon(), "🍎");
        assert_eq!(GpuBackendType::None.icon(), "💻");
    }

    #[test]
    fn test_backend_detection() {
        // Should not panic
        let backend = GpuDetector::detect_best_backend();
        tracing::info!("Detected backend: {:?}", backend);

        #[cfg(target_os = "macos")]
        {
            // On macOS, should detect Metal or CPU
            assert!(
                backend == GpuBackendType::Metal || backend == GpuBackendType::None,
                "Expected Metal or CPU on macOS"
            );
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS, should always be CPU
            assert_eq!(
                backend,
                GpuBackendType::None,
                "Expected CPU-only on non-macOS"
            );
        }
    }

    #[test]
    fn test_metal_availability() {
        let available = GpuDetector::is_metal_available();
        tracing::info!("Metal available: {}", available);

        #[cfg(target_os = "macos")]
        {
            // On macOS, Metal may or may not be available
            tracing::info!("Metal availability on macOS: {}", available);
        }

        #[cfg(not(target_os = "macos"))]
        {
            // On non-macOS, Metal should never be available
            assert!(!available, "Metal should not be available on non-macOS");
        }
    }

    #[test]
    fn test_gpu_info_display() {
        let info = GpuInfo {
            backend: GpuBackendType::Metal,
            device_name: "Apple M1 Pro".to_string(),
            vram_total: Some(16 * 1024 * 1024 * 1024), // 16 GB
            driver_version: Some("Metal 3.0".to_string()),
        };
        let display = format!("{}", info);
        assert!(display.contains("Metal"));
        assert!(display.contains("Apple M1 Pro"));
        assert!(display.contains("16"));
    }

    #[test]
    fn test_gpu_info_no_vram() {
        let info = GpuInfo {
            backend: GpuBackendType::Metal,
            device_name: "Apple GPU".to_string(),
            vram_total: None,
            driver_version: None,
        };
        let display = format!("{}", info);
        assert!(display.contains("Metal"));
        assert!(display.contains("Apple GPU"));
        assert!(!display.contains("GB VRAM"));
    }

    /// On macOS with the `hive-gpu` feature enabled, a live Metal context must
    /// produce a non-static device name and a non-zero VRAM total. Regression
    /// guard so the old literal `"Apple GPU"` string never reappears in
    /// `query_metal_info`.
    #[cfg(all(target_os = "macos", feature = "hive-gpu"))]
    #[test]
    fn test_query_metal_info_returns_real_data() {
        if !GpuDetector::is_metal_available() {
            // Headless CI runners without a GPU: skip the data assertions.
            return;
        }

        let info = GpuDetector::query_metal_info();
        assert_eq!(info.backend, GpuBackendType::Metal);
        assert_ne!(
            info.device_name, "Apple GPU",
            "device name is still the pre-0.2 default string"
        );
        assert_ne!(
            info.device_name, "Apple GPU (unknown)",
            "device_info() errored — hive-gpu API regression?"
        );
        assert!(
            matches!(info.vram_total, Some(n) if n > 0),
            "vram_total should be populated on a live Metal device"
        );
        assert!(
            info.driver_version.is_some(),
            "driver_version should be populated on a live Metal device"
        );
    }
}
