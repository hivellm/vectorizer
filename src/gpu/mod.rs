//! Aceleração GPU usando wgpu (Metal, Vulkan, DX12, OpenGL)
//! 
//! Este módulo fornece aceleração GPU cross-platform para operações vetoriais
//! usando wgpu, que suporta Metal (macOS/iOS), Vulkan, DirectX 12 e OpenGL.
//!
//! ## Operações Suportadas
//! 
//! - Similaridade Coseno (Cosine Similarity)
//! - Distância Euclidiana (Euclidean Distance)
//! - Produto Escalar (Dot Product)
//! - Busca em Lote (Batch Search)
//! - Top-K Selection
//!
//! ## Uso
//!
//! ```rust,no_run
//! use vectorizer::gpu::{GpuContext, GpuConfig, GpuOperations};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Criar contexto GPU
//! let config = GpuConfig::default();
//! let gpu = GpuContext::new(config).await?;
//!
//! // Realizar operações vetoriais
//! let query = vec![1.0, 2.0, 3.0];
//! let vectors = vec![
//!     vec![1.0, 0.0, 0.0],
//!     vec![0.0, 1.0, 0.0],
//!     vec![0.0, 0.0, 1.0],
//! ];
//!
//! let similarities = gpu.cosine_similarity(&query, &vectors).await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod context;
pub mod operations;
pub mod shaders;
pub mod buffers;
pub mod utils;
pub mod metal_collection;

pub use config::{GpuConfig, GpuBackend};
pub use context::GpuContext;
pub use operations::GpuOperations;
pub use metal_collection::MetalCollection;

use crate::error::{Result, VectorizerError};
use crate::models::DistanceMetric;

#[cfg(feature = "wgpu-gpu")]
use wgpu;

/// Verifica se a GPU está disponível e qual backend será usado
pub async fn check_gpu_availability() -> Result<GpuBackend> {
    #[cfg(not(feature = "wgpu-gpu"))]
    {
        return Err(VectorizerError::Other(
            "Feature 'wgpu-gpu' não está habilitada. Compile com --features wgpu-gpu".to_string()
        ));
    }

    #[cfg(feature = "wgpu-gpu")]
    {
        use wgpu::Instance;
        
        let instance = Instance::default();
        
        // Tentar detectar o melhor adapter disponível
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await;

        match adapter {
            Ok(adapter) => {
                let info = adapter.get_info();
                let backend = match info.backend {
                    wgpu::Backend::Metal => GpuBackend::Metal,
                    wgpu::Backend::Vulkan => GpuBackend::Vulkan,
                    wgpu::Backend::Dx12 => GpuBackend::DirectX12,
                    wgpu::Backend::Gl => GpuBackend::OpenGL,
                    _ => GpuBackend::None,
                };
                
                tracing::info!(
                    "GPU detectada: {} ({:?}) - Backend: {:?}",
                    info.name,
                    info.device_type,
                    info.backend
                );
                
                Ok(backend)
            }
            Err(_) => {
                tracing::warn!("Nenhum adapter GPU encontrado");
                Ok(GpuBackend::None)
            }
        }
    }
}

/// Informações sobre a GPU disponível
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub backend: GpuBackend,
    pub device_type: String,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub limits: GpuLimits,
}

#[derive(Debug, Clone)]
pub struct GpuLimits {
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u32,
    pub max_compute_workgroups_per_dimension: u32,
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self {
            name: "CPU Fallback".to_string(),
            backend: GpuBackend::None,
            device_type: "Cpu".to_string(),
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_invocations_per_workgroup: 256,
            limits: GpuLimits {
                max_buffer_size: 1 << 30, // 1GB
                max_storage_buffer_binding_size: 1 << 28, // 256MB
                max_compute_workgroups_per_dimension: 65535,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_availability() {
        let result = check_gpu_availability().await;
        println!("GPU disponível: {:?}", result);
        assert!(result.is_ok());
    }
}
