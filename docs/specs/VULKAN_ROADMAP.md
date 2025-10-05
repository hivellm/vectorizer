# 🔥 Roadmap: Suporte Vulkan para AMD GPUs

## 🎯 Objetivo

Adicionar suporte completo para **Vulkan** como backend GPU, priorizando **AMD GPUs** mas funcionando universalmente em qualquer GPU compatível.

## 📊 Prioridade de Backends (Atualizada)

```
1. Metal (macOS Apple Silicon) ✅ IMPLEMENTADO
2. Vulkan (AMD/Linux/Universal) 🚧 EM DESENVOLVIMENTO
3. DirectX12 (Windows) 📋 PLANEJADO
4. CUDA (NVIDIA) ✅ IMPLEMENTADO
5. CPU (Universal) ✅ IMPLEMENTADO
```

## 🏗️ Sprint 1: Estrutura Base (Semana 1)

### Entregáveis
- [ ] Módulo `src/gpu/backends/`
- [ ] Enum `GpuBackendType`
- [ ] Função `detect_available_backends()`
- [ ] Testes unitários de detecção

### Código Base

```rust
// src/gpu/backends/mod.rs
pub mod detector;
pub mod vulkan;
pub mod dx12;

pub use detector::*;
```

```rust
// src/gpu/backends/detector.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    Metal,
    Vulkan,
    DirectX12,
    CudaNative,
    Cpu,
}

pub fn detect_available_backends() -> Vec<GpuBackendType> {
    let mut available = Vec::new();
    
    // Tenta cada backend em ordem de prioridade
    if try_metal().is_ok() {
        available.push(GpuBackendType::Metal);
    }
    
    if try_vulkan().is_ok() {
        available.push(GpuBackendType::Vulkan);
    }
    
    if try_dx12().is_ok() {
        available.push(GpuBackendType::DirectX12);
    }
    
    // CUDA e CPU sempre disponíveis (se features habilitadas)
    #[cfg(feature = "cuda")]
    available.push(GpuBackendType::CudaNative);
    
    available.push(GpuBackendType::Cpu);
    
    available
}
```

## 🔥 Sprint 2: Backend Vulkan (Semanas 2-3)

### Entregáveis
- [ ] `src/gpu/backends/vulkan.rs` completo
- [ ] Detecção de GPU AMD
- [ ] `VulkanCollection` implementado
- [ ] Testes em Linux com AMD

### Implementação

```rust
// src/gpu/backends/vulkan.rs
use wgpu::{Instance, Backends, Adapter};

pub struct VulkanBackend {
    instance: Instance,
    adapter: Adapter,
    is_amd: bool,
}

impl VulkanBackend {
    pub async fn new() -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Vulkan adapter não encontrado")?;
        
        let info = adapter.get_info();
        let is_amd = info.vendor == 0x1002 || // AMD Vendor ID
                     info.name.to_lowercase().contains("amd") ||
                     info.name.to_lowercase().contains("radeon");
        
        Ok(Self { instance, adapter, is_amd })
    }
    
    pub fn is_amd_gpu(&self) -> bool {
        self.is_amd
    }
    
    pub fn gpu_name(&self) -> String {
        self.adapter.get_info().name
    }
}
```

### Testes Necessários

1. **Linux + AMD RX 6000/7000**: Teste completo
2. **Windows + AMD**: Vulkan vs DirectX
3. **Linux + NVIDIA**: Vulkan vs CUDA
4. **Benchmark**: Vulkan vs Metal vs CPU

## 🪟 Sprint 3: Backend DirectX 12 (Semanas 4-5)

### Entregáveis
- [ ] `src/gpu/backends/dx12.rs` completo
- [ ] `DirectX12Collection` implementado
- [ ] Testes em Windows 10/11
- [ ] Benchmark comparativo

### Implementação

```rust
// src/gpu/backends/dx12.rs
pub struct DirectX12Backend {
    instance: Instance,
    adapter: Adapter,
    gpu_vendor: GpuVendor,
}

#[derive(Debug, Clone, Copy)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

impl DirectX12Backend {
    pub async fn new() -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .ok_or("DirectX 12 adapter não encontrado")?;
        
        let vendor = match adapter.get_info().vendor {
            0x10DE => GpuVendor::Nvidia,
            0x1002 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            _ => GpuVendor::Other,
        };
        
        Ok(Self { instance, adapter, gpu_vendor: vendor })
    }
}
```

## 🌍 Sprint 4: Detecção Universal (Semana 6)

### Entregáveis
- [ ] `new_auto_universal()` completo
- [ ] CLI flags (`--gpu-backend`)
- [ ] Logs detalhados de detecção
- [ ] Documentação atualizada

### Implementação Final

```rust
impl VectorStore {
    pub fn new_auto_universal() -> Self {
        eprintln!("🔍 Detectando melhor backend GPU...");
        
        let available = detect_available_backends();
        eprintln!("📊 Backends disponíveis: {:?}", available);
        
        for backend in available {
            match backend {
                GpuBackendType::Metal => {
                    if let Ok(store) = Self::try_new_with_metal() {
                        eprintln!("✅ Usando Metal (Apple Silicon)");
                        return store;
                    }
                }
                GpuBackendType::Vulkan => {
                    if let Ok(store) = Self::try_new_with_vulkan() {
                        eprintln!("✅ Usando Vulkan");
                        return store;
                    }
                }
                GpuBackendType::DirectX12 => {
                    if let Ok(store) = Self::try_new_with_dx12() {
                        eprintln!("✅ Usando DirectX 12");
                        return store;
                    }
                }
                GpuBackendType::CudaNative => {
                    if let Ok(store) = Self::try_new_with_cuda() {
                        eprintln!("✅ Usando CUDA");
                        return store;
                    }
                }
                GpuBackendType::Cpu => {
                    eprintln!("💻 Usando CPU (fallback)");
                    return Self::new();
                }
            }
        }
        
        eprintln!("💻 Fallback final para CPU");
        Self::new()
    }
}
```

## ⚡ Sprint 5: Otimizações (Semana 7)

### Entregáveis
- [ ] Benchmarks completos
- [ ] CI/CD multi-plataforma
- [ ] Documentação final
- [ ] Release notes

### Benchmark Esperado

| Operação | CPU | Metal | Vulkan (AMD) | DX12 | CUDA |
|----------|-----|-------|--------------|------|------|
| Search 1K | 100ms | 12ms | 15ms | 18ms | 10ms |
| Index 10K | 5s | 550ms | 600ms | 700ms | 500ms |
| Batch 100 | 2s | 200ms | 220ms | 250ms | 180ms |

## 📚 Documentação Necessária

1. **VULKAN_SETUP.md**: Como configurar Vulkan em diferentes OSs
2. **DIRECTX12_SETUP.md**: Requisitos Windows
3. **GPU_BENCHMARKS.md**: Resultados comparativos
4. **TROUBLESHOOTING.md**: Problemas comuns

## 🎯 Critérios de Sucesso

- ✅ Detecção automática funciona em 95%+ dos casos
- ✅ Vulkan funciona em AMD RX 6000+
- ✅ DirectX funciona em Windows 10+
- ✅ Performance GPU ≥ 5x CPU
- ✅ Documentação completa

