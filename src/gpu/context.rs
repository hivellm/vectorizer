//! Contexto GPU usando wgpu

use super::config::{GpuConfig, GpuBackend, GpuPowerPreference};
use super::{GpuInfo, GpuLimits};
use crate::error::{Result, VectorizerError};

#[cfg(feature = "wgpu-gpu")]
use wgpu::{Instance, Adapter, Device, Queue, Features, Limits};

/// Contexto GPU que gerencia a conexão com o hardware
pub struct GpuContext {
    config: GpuConfig,
    info: GpuInfo,
    
    #[cfg(feature = "wgpu-gpu")]
    instance: Instance,
    
    #[cfg(feature = "wgpu-gpu")]
    adapter: Adapter,
    
    #[cfg(feature = "wgpu-gpu")]
    device: Device,
    
    #[cfg(feature = "wgpu-gpu")]
    queue: Queue,
}

impl GpuContext {
    /// Criar novo contexto GPU
    pub async fn new(config: GpuConfig) -> Result<Self> {
        // Validar configuração
        config.validate()
            .map_err(|e| VectorizerError::Other(format!("Configuração inválida: {}", e)))?;

        #[cfg(not(feature = "wgpu-gpu"))]
        {
            return Err(VectorizerError::Other(
                "Feature 'wgpu-gpu' não está habilitada".to_string()
            ));
        }

        #[cfg(feature = "wgpu-gpu")]
        {
            Self::new_wgpu(config).await
        }
    }

    #[cfg(feature = "wgpu-gpu")]
    async fn new_wgpu(config: GpuConfig) -> Result<Self> {
        use tracing::{info, warn};

        // Criar instância wgpu
        let instance = Instance::default();

        // Determinar backend
        let backend = Self::select_backend(&config);
        info!("Tentando inicializar backend: {:?}", backend);

        // Request adapter
        let power_pref = match config.power_preference {
            GpuPowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
            GpuPowerPreference::LowPower => wgpu::PowerPreference::LowPower,
            GpuPowerPreference::None => wgpu::PowerPreference::None,
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                VectorizerError::Other(format!("Falha ao obter adapter GPU: {:?}", e))
            })?;

        let adapter_info = adapter.get_info();
        info!(
            "GPU selecionada: {} ({:?})",
            adapter_info.name, adapter_info.backend
        );

        // Verificar limites
        let adapter_limits = adapter.limits();
        
        // Request device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Vectorizer GPU Device"),
                required_features: Features::empty(),
                required_limits: Limits {
                    max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
                    max_buffer_size: adapter_limits.max_buffer_size,
                    max_compute_workgroup_size_x: adapter_limits.max_compute_workgroup_size_x,
                    max_compute_workgroup_size_y: adapter_limits.max_compute_workgroup_size_y,
                    max_compute_workgroup_size_z: adapter_limits.max_compute_workgroup_size_z,
                    max_compute_workgroups_per_dimension: adapter_limits.max_compute_workgroups_per_dimension,
                    max_compute_invocations_per_workgroup: adapter_limits.max_compute_invocations_per_workgroup,
                    ..Default::default()
                },
                memory_hints: Default::default(),
                experimental_features: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| VectorizerError::Other(format!("Falha ao criar device: {}", e)))?;

        let detected_backend = match adapter_info.backend {
            wgpu::Backend::Metal => GpuBackend::Metal,
            wgpu::Backend::Vulkan => GpuBackend::Vulkan,
            wgpu::Backend::Dx12 => GpuBackend::DirectX12,
            wgpu::Backend::Gl => GpuBackend::OpenGL,
            wgpu::Backend::BrowserWebGpu => GpuBackend::WebGpu,
            _ => GpuBackend::None,
        };

        let info = GpuInfo {
            name: adapter_info.name.clone(),
            backend: detected_backend,
            device_type: format!("{:?}", adapter_info.device_type),
            max_compute_workgroup_size_x: adapter_limits.max_compute_workgroup_size_x,
            max_compute_workgroup_size_y: adapter_limits.max_compute_workgroup_size_y,
            max_compute_workgroup_size_z: adapter_limits.max_compute_workgroup_size_z,
            max_compute_invocations_per_workgroup: adapter_limits.max_compute_invocations_per_workgroup,
            limits: GpuLimits {
                max_buffer_size: adapter_limits.max_buffer_size,
                max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
                max_compute_workgroups_per_dimension: adapter_limits.max_compute_workgroups_per_dimension,
            },
        };

        info!("Contexto GPU inicializado com sucesso");
        info!("  - Backend: {:?}", info.backend);
        info!("  - Device: {}", info.name);
        info!("  - Tipo: {}", info.device_type);
        info!("  - Max workgroup size: {}x{}x{}", 
            info.max_compute_workgroup_size_x,
            info.max_compute_workgroup_size_y,
            info.max_compute_workgroup_size_z
        );

        Ok(Self {
            config,
            info,
            instance,
            adapter,
            device,
            queue,
        })
    }

    #[cfg(feature = "wgpu-gpu")]
    fn select_backend(config: &GpuConfig) -> GpuBackend {
        if let Some(backend) = config.preferred_backend {
            return backend;
        }
        
        GpuBackend::default()
    }

    /// Obter informações sobre a GPU
    pub fn info(&self) -> &GpuInfo {
        &self.info
    }

    /// Obter configuração
    pub fn config(&self) -> &GpuConfig {
        &self.config
    }

    #[cfg(feature = "wgpu-gpu")]
    pub fn device(&self) -> &Device {
        &self.device
    }

    #[cfg(feature = "wgpu-gpu")]
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    #[cfg(feature = "wgpu-gpu")]
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    /// Verificar se deve usar GPU com base no número de operações
    pub fn should_use_gpu(&self, operations: usize) -> bool {
        if !self.config.enabled {
            return false;
        }

        operations >= self.config.gpu_threshold_operations
    }

    /// Calcular tamanho ótimo do workgroup
    pub fn optimal_workgroup_size(&self, elements: usize) -> u32 {
        let max_size = self.config.workgroup_size.min(
            self.info.max_compute_workgroup_size_x
        );

        // Encontrar a maior potência de 2 que não excede max_size
        let mut size = 256;
        while size > max_size {
            size /= 2;
        }

        // Ajustar baseado no número de elementos
        if elements < size as usize {
            let mut adjusted = size;
            while adjusted > 32 && elements < adjusted as usize / 2 {
                adjusted /= 2;
            }
            adjusted
        } else {
            size
        }
    }

    /// Calcular número de workgroups necessários
    pub fn workgroups_needed(&self, elements: usize, workgroup_size: u32) -> u32 {
        ((elements as u32 + workgroup_size - 1) / workgroup_size)
            .min(self.info.limits.max_compute_workgroups_per_dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_context() {
        let config = GpuConfig::default();
        let result = GpuContext::new(config).await;
        
        match result {
            Ok(ctx) => {
                println!("GPU Context criado com sucesso");
                println!("Backend: {:?}", ctx.info().backend);
                println!("Device: {}", ctx.info().name);
            }
            Err(e) => {
                println!("Não foi possível criar contexto GPU: {}", e);
            }
        }
    }

    #[test]
    fn test_workgroup_calculations() {
        let mut info = GpuInfo::default();
        info.max_compute_workgroup_size_x = 256;
        
        let config = GpuConfig::default();
        let ctx = GpuContext {
            config: config.clone(),
            info,
            #[cfg(feature = "wgpu-gpu")]
            instance: Instance::default(),
            #[cfg(feature = "wgpu-gpu")]
            adapter: panic!("Test only"),
            #[cfg(feature = "wgpu-gpu")]
            device: panic!("Test only"),
            #[cfg(feature = "wgpu-gpu")]
            queue: panic!("Test only"),
        };

        let wg_size = ctx.optimal_workgroup_size(10000);
        assert!(wg_size <= 256);
        assert!(wg_size.is_power_of_two());

        let wg_count = ctx.workgroups_needed(10000, wg_size);
        assert!(wg_count * wg_size >= 10000);
    }
}
