//! GPU Backend Detection and Management
//!
//! This module provides automatic detection and management of different GPU backends:
//! - Metal (Apple Silicon)
//! - Vulkan (AMD/NVIDIA/Intel/Universal)
//! - DirectX 12 (Windows)
//! - GPU (NVIDIA native)
//! - CPU (fallback)

pub mod detector;

#[cfg(feature = "wgpu-gpu")]
pub mod vulkan;

#[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
pub mod dx12;

#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
pub mod metal;

pub use detector::{GpuBackendType, detect_available_backends, select_best_backend};

