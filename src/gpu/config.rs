//! Configuração para aceleração GPU

use serde::{Deserialize, Serialize};

/// Backend GPU disponível
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuBackend {
    /// Metal (macOS/iOS)
    Metal,
    /// Vulkan (Linux/Windows/Android)
    Vulkan,
    /// DirectX 12 (Windows)
    DirectX12,
    /// OpenGL (fallback)
    OpenGL,
    /// WebGPU (navegadores)
    WebGpu,
    /// Nenhum (CPU fallback)
    None,
}

impl Default for GpuBackend {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        return Self::Metal;
        
        #[cfg(target_os = "ios")]
        return Self::Metal;
        
        #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
        return Self::DirectX12;
        
        #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
        return Self::Vulkan;
        
        #[cfg(target_arch = "wasm32")]
        return Self::WebGpu;
        
        #[cfg(not(any(
            target_os = "macos",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_arch = "wasm32"
        )))]
        return Self::None;
    }
}

/// Configuração de aceleração GPU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// Habilitar aceleração GPU
    pub enabled: bool,
    
    /// Backend preferido (None = auto-detect)
    pub preferred_backend: Option<GpuBackend>,
    
    /// Limite de memória GPU em MB (0 = sem limite)
    pub memory_limit_mb: usize,
    
    /// Tamanho do workgroup para compute shaders
    pub workgroup_size: u32,
    
    /// Threshold mínimo de operações para usar GPU
    /// Operações menores que isso usarão CPU
    pub gpu_threshold_operations: usize,
    
    /// Usar memória mapeada quando possível
    pub use_mapped_memory: bool,
    
    /// Timeout para operações GPU em milissegundos
    pub timeout_ms: u64,
    
    /// Power preference
    pub power_preference: GpuPowerPreference,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GpuPowerPreference {
    /// Alta performance (GPU discreta)
    HighPerformance,
    /// Baixo consumo (GPU integrada)
    LowPower,
    /// Sem preferência
    None,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_backend: None, // Auto-detect
            memory_limit_mb: 4096,   // 4GB padrão
            workgroup_size: 256,     // Otimizado para a maioria das GPUs
            gpu_threshold_operations: 5_000_000, // 5M operações
            use_mapped_memory: true,
            timeout_ms: 30_000, // 30 segundos
            power_preference: GpuPowerPreference::HighPerformance,
        }
    }
}

impl GpuConfig {
    /// Criar configuração otimizada para Metal (macOS Silicon)
    pub fn for_metal_silicon() -> Self {
        Self {
            enabled: true,
            preferred_backend: Some(GpuBackend::Metal),
            memory_limit_mb: 8192,  // Apple Silicon tem memória unificada
            workgroup_size: 256,
            gpu_threshold_operations: 1_000_000, // Metal é muito eficiente
            use_mapped_memory: true,
            timeout_ms: 30_000,
            power_preference: GpuPowerPreference::HighPerformance,
        }
    }

    /// Criar configuração otimizada para Vulkan (Linux)
    pub fn for_vulkan() -> Self {
        Self {
            enabled: true,
            preferred_backend: Some(GpuBackend::Vulkan),
            memory_limit_mb: 4096,
            workgroup_size: 256,
            gpu_threshold_operations: 5_000_000,
            use_mapped_memory: true,
            timeout_ms: 30_000,
            power_preference: GpuPowerPreference::HighPerformance,
        }
    }

    /// Criar configuração otimizada para DirectX 12 (Windows)
    pub fn for_directx12() -> Self {
        Self {
            enabled: true,
            preferred_backend: Some(GpuBackend::DirectX12),
            memory_limit_mb: 4096,
            workgroup_size: 256,
            gpu_threshold_operations: 5_000_000,
            use_mapped_memory: false, // DX12 tem diferentes padrões
            timeout_ms: 30_000,
            power_preference: GpuPowerPreference::HighPerformance,
        }
    }

    /// Validar configuração
    pub fn validate(&self) -> Result<(), String> {
        if self.workgroup_size == 0 || self.workgroup_size > 1024 {
            return Err("workgroup_size deve estar entre 1 e 1024".to_string());
        }

        if self.workgroup_size & (self.workgroup_size - 1) != 0 {
            return Err("workgroup_size deve ser uma potência de 2".to_string());
        }

        if self.timeout_ms == 0 {
            return Err("timeout_ms deve ser maior que zero".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = GpuConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metal_config() {
        let config = GpuConfig::for_metal_silicon();
        assert_eq!(config.preferred_backend, Some(GpuBackend::Metal));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_workgroup_size() {
        let mut config = GpuConfig::default();
        config.workgroup_size = 0;
        assert!(config.validate().is_err());

        config.workgroup_size = 100; // Não é potência de 2
        assert!(config.validate().is_err());
    }
}
