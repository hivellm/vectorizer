# 🎯 Especificação Técnica: Detecção Multi-GPU Universal

## 📋 Visão Geral

Implementar detecção automática de GPU com suporte para **todos os backends** via wgpu:

```
Prioridade: Metal > Vulkan (AMD) > DirectX12 > CUDA > CPU
```

## 🎯 Backends Suportados

### 1. **Metal** 🍎
- **Plataforma**: macOS (Apple Silicon M1/M2/M3)
- **GPU**: Apple GPU integrada
- **Detecção**: `target_os = "macos"` + `target_arch = "aarch64"`
- **Status**: ✅ **IMPLEMENTADO**

### 2. **Vulkan** 🔥
- **Plataforma**: Linux, Windows, Android
- **GPU**: AMD, NVIDIA, Intel, Mobile
- **Detecção**: Prioridade para AMD, fallback universal
- **Status**: 🚧 **A IMPLEMENTAR**

### 3. **DirectX 12** 🪟
- **Plataforma**: Windows
- **GPU**: NVIDIA, AMD, Intel
- **Detecção**: `target_os = "windows"`
- **Status**: 🚧 **A IMPLEMENTAR**

### 4. **CUDA** ⚡
- **Plataforma**: Linux, Windows
- **GPU**: NVIDIA exclusivo
- **Detecção**: Feature `cuda` + biblioteca CUDA
- **Status**: ✅ **IMPLEMENTADO**

### 5. **CPU** 💻
- **Plataforma**: Universal
- **GPU**: Nenhuma
- **Detecção**: Fallback final
- **Status**: ✅ **IMPLEMENTADO**

---

## 🏗️ Arquitetura da Detecção

### Fluxo de Detecção

```rust
pub fn new_auto() -> Self {
    // 1. Metal (Mac Silicon apenas)
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
    if try_metal() { return metal_store(); }
    
    // 2. Vulkan (Prioridade AMD, mas funciona em qualquer GPU)
    #[cfg(feature = "wgpu-gpu")]
    if try_vulkan() { return vulkan_store(); }
    
    // 3. DirectX 12 (Windows com qualquer GPU)
    #[cfg(all(target_os = "windows", feature = "wgpu-gpu"))]
    if try_dx12() { return dx12_store(); }
    
    // 4. CUDA (NVIDIA específico)
    #[cfg(feature = "cuda")]
    if cuda_available() { return cuda_store(); }
    
    // 5. CPU (Fallback universal)
    return cpu_store();
}
```

### Detecção de Backend wgpu

```rust
use wgpu::{Instance, Backends, PowerPreference};

fn detect_best_backend() -> Result<wgpu::Backend> {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(), // Tenta todos
        ..Default::default()
    });
    
    // Priorizar backends por ordem
    let priority = [
        wgpu::Backend::Metal,    // macOS
        wgpu::Backend::Vulkan,   // AMD/Linux/Universal
        wgpu::Backend::Dx12,     // Windows
    ];
    
    for backend in priority {
        if let Ok(adapter) = try_backend(&instance, backend) {
            return Ok(backend);
        }
    }
    
    Err("Nenhum backend GPU disponível")
}
```

---

## 📦 Estrutura de Código

### Novo Módulo: `src/gpu/backends/`

```
src/gpu/
├── mod.rs                  # API pública
├── config.rs               # GpuConfig
├── context.rs              # GpuContext
├── operations.rs           # GpuOperations trait
├── metal_collection.rs     # MetalCollection
├── backends/               # 🆕 Novo módulo
│   ├── mod.rs              # Detecção de backend
│   ├── metal.rs            # Backend Metal específico
│   ├── vulkan.rs           # 🆕 Backend Vulkan
│   ├── dx12.rs             # 🆕 Backend DirectX 12
│   └── detector.rs         # 🆕 Lógica de detecção
└── shaders/                # WGSL shaders (universal)
```

### Novo Enum: `GpuBackendType`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendType {
    /// Apple Metal (M1/M2/M3)
    Metal,
    /// Vulkan (AMD/NVIDIA/Intel/Universal)
    Vulkan,
    /// DirectX 12 (Windows)
    DirectX12,
    /// Não usa wgpu (CUDA direto)
    CudaNative,
    /// CPU apenas
    Cpu,
}

impl GpuBackendType {
    /// Prioridade do backend (menor = maior prioridade)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Metal => 0,       // Melhor para Mac
            Self::Vulkan => 1,      // Universal, AMD otimizado
            Self::DirectX12 => 2,   // Windows nativo
            Self::CudaNative => 3,  // NVIDIA específico
            Self::Cpu => 255,       // Último recurso
        }
    }
    
    /// Verifica se backend está disponível no sistema atual
    pub fn is_available(&self) -> bool {
        match self {
            Self::Metal => cfg!(all(target_os = "macos", target_arch = "aarch64")),
            Self::Vulkan => true, // Sempre tenta
            Self::DirectX12 => cfg!(target_os = "windows"),
            Self::CudaNative => cfg!(feature = "cuda"),
            Self::Cpu => true,
        }
    }
}
```

---

## 🔧 Implementação por Etapas

### **FASE 1: Estrutura Base** 📐

#### Task 1.1: Criar módulo `backends/`
- [x] `src/gpu/backends/mod.rs`
- [x] `src/gpu/backends/detector.rs`
- [x] Enum `GpuBackendType`
- [x] Função `detect_available_backends()`

#### Task 1.2: Refatorar `GpuConfig`
- [x] Adicionar campo `preferred_backend: Option<GpuBackendType>`
- [x] Adicionar campo `backend_priority: Vec<GpuBackendType>`
- [x] Método `auto_detect_backend()`

#### Task 1.3: Atualizar `GpuContext`
- [x] Adicionar campo `active_backend: GpuBackendType`
- [x] Método `new_with_backend(config, backend)`
- [x] Atualizar logs para mostrar backend ativo

---

### **FASE 2: Backend Vulkan (AMD/Universal)** 🔥

#### Task 2.1: Implementar `backends/vulkan.rs`
- [ ] Struct `VulkanBackend`
- [ ] Detecção de GPU AMD via `wgpu::AdapterInfo`
- [ ] Inicialização de contexto Vulkan
- [ ] Testes de disponibilidade

#### Task 2.2: Criar `VulkanCollection`
- [ ] Similar a `MetalCollection`
- [ ] Usar `GpuContext` com backend Vulkan
- [ ] Implementar trait `Collection`
- [ ] Otimizações específicas AMD

#### Task 2.3: Integrar no `VectorStore`
- [ ] Adicionar `CollectionType::Vulkan`
- [ ] Atualizar `create_collection` para Vulkan
- [ ] Implementar fallback Metal → Vulkan

#### Task 2.4: Testes
- [ ] Teste em Linux com AMD
- [ ] Teste em Windows com AMD
- [ ] Benchmark vs CPU/Metal

---

### **FASE 3: Backend DirectX 12 (Windows)** 🪟

#### Task 3.1: Implementar `backends/dx12.rs`
- [ ] Struct `DirectX12Backend`
- [ ] Detecção via `wgpu::Backend::Dx12`
- [ ] Inicialização de contexto DirectX
- [ ] Testes de disponibilidade Windows

#### Task 3.2: Criar `DirectX12Collection`
- [ ] Similar a `MetalCollection`
- [ ] Usar `GpuContext` com backend DX12
- [ ] Implementar trait `Collection`
- [ ] Otimizações específicas Windows

#### Task 3.3: Integrar no `VectorStore`
- [ ] Adicionar `CollectionType::DirectX12`
- [ ] Atualizar `create_collection` para DX12
- [ ] Implementar fallback Vulkan → DX12

#### Task 3.4: Testes
- [ ] Teste em Windows 10/11
- [ ] Teste com NVIDIA/AMD/Intel
- [ ] Benchmark vs Vulkan/CPU

---

### **FASE 4: Detecção Universal** 🌍

#### Task 4.1: Implementar `new_auto_universal()`
```rust
impl VectorStore {
    pub fn new_auto_universal() -> Self {
        let available = detect_all_backends();
        let best = select_best_backend(available);
        
        match best {
            GpuBackendType::Metal => Self::new_with_metal(),
            GpuBackendType::Vulkan => Self::new_with_vulkan(),
            GpuBackendType::DirectX12 => Self::new_with_dx12(),
            GpuBackendType::CudaNative => Self::new_with_cuda(),
            GpuBackendType::Cpu => Self::new(),
        }
    }
}
```

#### Task 4.2: Atualizar `vzr.rs`
- [ ] Substituir `new_auto()` por `new_auto_universal()`
- [ ] Adicionar logs detalhados de detecção
- [ ] CLI flag `--gpu-backend` para forçar backend

#### Task 4.3: Documentação
- [ ] Atualizar `GPU_AUTO_DETECTION.md`
- [ ] Criar `VULKAN_SETUP.md`
- [ ] Criar `DIRECTX12_SETUP.md`
- [ ] Tabela comparativa de backends

---

### **FASE 5: Otimizações e Testes** ⚡

#### Task 5.1: Benchmarks Comparativos
- [ ] Benchmark Metal vs Vulkan vs DX12 vs CUDA vs CPU
- [ ] Diferentes tamanhos de datasets
- [ ] Diferentes dimensões de vetores
- [ ] Relatório de performance por backend

#### Task 5.2: Testes de Integração
- [ ] CI/CD para Linux (Vulkan)
- [ ] CI/CD para Windows (DX12)
- [ ] CI/CD para macOS (Metal)
- [ ] Testes multi-plataforma

#### Task 5.3: Fallback Inteligente
- [ ] Se GPU falha, tenta próximo backend
- [ ] Cache de backend funcionando
- [ ] Retry logic para detecção
- [ ] Métricas de sucesso/falha

---

## 📊 Tabela Comparativa de Backends

| Backend | Plataforma | GPU Suportadas | Performance | Maturidade | Prioridade |
|---------|------------|----------------|-------------|------------|------------|
| **Metal** | macOS | Apple Silicon | ⭐⭐⭐⭐⭐ | ✅ Prod | 1 |
| **Vulkan** | Linux/Win/Android | AMD/NVIDIA/Intel | ⭐⭐⭐⭐⭐ | 🚧 Beta | 2 |
| **DirectX12** | Windows | AMD/NVIDIA/Intel | ⭐⭐⭐⭐ | 🚧 Beta | 3 |
| **CUDA** | Linux/Win | NVIDIA | ⭐⭐⭐⭐⭐ | ✅ Prod | 4 |
| **CPU** | Universal | Nenhuma | ⭐⭐ | ✅ Prod | 5 |

---

## 🎯 Critérios de Seleção Automática

### Prioridade 1: Plataforma Nativa
- **macOS** → Metal (único backend Apple)
- **Windows** → DirectX12 (nativo Microsoft)
- **Linux** → Vulkan (padrão open-source)

### Prioridade 2: GPU Vendor
- **AMD GPU** → Vulkan (melhor suporte)
- **NVIDIA GPU** → CUDA > Vulkan > DX12
- **Intel GPU** → Vulkan > DX12

### Prioridade 3: Disponibilidade
- Se backend preferido falhar, tenta próximo
- CPU sempre disponível como fallback

---

## 🔍 Detecção de GPU AMD

```rust
fn is_amd_gpu(adapter: &wgpu::Adapter) -> bool {
    let info = adapter.get_info();
    
    // Verificar vendor ID (AMD = 0x1002)
    if info.vendor == 0x1002 {
        return true;
    }
    
    // Verificar nome da GPU
    let name_lower = info.name.to_lowercase();
    name_lower.contains("amd") || 
    name_lower.contains("radeon") ||
    name_lower.contains("ryzen")
}

fn select_backend_for_gpu(adapter: &wgpu::Adapter) -> GpuBackendType {
    if is_amd_gpu(adapter) {
        // AMD funciona melhor com Vulkan
        GpuBackendType::Vulkan
    } else if adapter.get_info().vendor == 0x10DE {
        // NVIDIA (0x10DE) prefere CUDA
        GpuBackendType::CudaNative
    } else {
        // Intel, ARM, outros
        GpuBackendType::Vulkan
    }
}
```

---

## 📚 Dependências Adicionais

### Cargo.toml
```toml
[dependencies]
# Já existente
wgpu = { version = "27.0", features = ["wgsl"], optional = true }

[features]
# Já existente
wgpu-gpu = ["wgpu", "pollster", "bytemuck", "futures", "ctrlc"]

# Novos backends específicos
vulkan = ["wgpu-gpu"]
dx12 = ["wgpu-gpu"]
all-gpu = ["metal", "vulkan", "dx12", "cuda"]
```

---

## 🚀 Exemplo de Uso

### Detecção Automática
```rust
// Detecta automaticamente: Metal/Vulkan/DX12/CUDA/CPU
let store = VectorStore::new_auto_universal();
```

### Forçar Backend
```rust
// Forçar Vulkan (útil para AMD)
let store = VectorStore::new_with_vulkan(VulkanConfig::default());

// Forçar DirectX (útil para Windows)
let store = VectorStore::new_with_dx12(Dx12Config::default());
```

### CLI
```bash
# Auto-detecção (padrão)
./vzr --workspace config.yml

# Forçar Vulkan
./vzr --workspace config.yml --gpu-backend vulkan

# Forçar DirectX
./vzr --workspace config.yml --gpu-backend dx12

# Forçar CPU
./vzr --workspace config.yml --gpu-backend cpu
```

---

## 📈 Roadmap de Implementação

### Sprint 1 (1 semana)
- ✅ Estrutura base (`backends/`)
- ✅ Enum `GpuBackendType`
- ✅ Detector básico

### Sprint 2 (2 semanas)
- 🚧 Backend Vulkan completo
- 🚧 `VulkanCollection`
- 🚧 Testes Linux/AMD

### Sprint 3 (2 semanas)
- 🚧 Backend DirectX 12 completo
- 🚧 `DirectX12Collection`
- 🚧 Testes Windows

### Sprint 4 (1 semana)
- 🚧 Detecção universal
- 🚧 CLI flags
- 🚧 Documentação completa

### Sprint 5 (1 semana)
- 🚧 Benchmarks comparativos
- 🚧 Otimizações
- 🚧 CI/CD multi-plataforma

---

## 🎯 Métricas de Sucesso

- ✅ Detecção automática funciona em 95%+ dos casos
- ✅ Performance GPU ≥ 5x mais rápida que CPU
- ✅ Fallback gracioso se GPU falhar
- ✅ Suporte para AMD, NVIDIA, Intel, Apple
- ✅ Documentação completa e exemplos

---

## 🔗 Referências

- [wgpu Documentation](https://wgpu.rs/)
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [DirectX 12 Programming Guide](https://docs.microsoft.com/en-us/windows/win32/direct3d12/)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)

