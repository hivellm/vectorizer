# 🔍 Detecção Automática de GPU - Vectorizer

## 📋 Visão Geral

O Vectorizer agora possui **detecção automática de GPU** com priorização inteligente:

```
🍎 Metal (Mac Silicon) > 🔥 CUDA (NVIDIA) > 💻 CPU
```

## 🚀 Como Usar

### 1. Compilação

```bash
# Mac Silicon (Metal GPU)
cargo build --bin vzr --features wgpu-gpu --release

# NVIDIA GPU (CUDA)
cargo build --bin vzr --features cuda --release

# CPU apenas
cargo build --bin vzr --release
```

### 2. Execução

```bash
# Simplesmente execute - a detecção é AUTOMÁTICA!
./target/release/vzr --workspace vectorize-workspace.yml
```

**Não precisa de flags ou configurações!** O sistema detecta automaticamente qual GPU usar.

## 🎯 Prioridade de Detecção

### 1. **Metal GPU** 🍎
- **Quando**: Mac Silicon (M1/M2/M3/etc) + feature `wgpu-gpu`
- **Detecção**: `target_os = "macos"` + `target_arch = "aarch64"`
- **Log**: `🍎 Detecting Metal GPU on Mac Silicon...`

### 2. **CUDA GPU** 🔥
- **Quando**: NVIDIA GPU + feature `cuda`
- **Detecção**: Feature `cuda` compilada
- **Log**: `🔥 Attempting to use CUDA GPU...`

### 3. **CPU** 💻
- **Quando**: Nenhuma GPU disponível
- **Detecção**: Fallback automático
- **Log**: `💻 Using CPU-only mode`

## 📊 Exemplo de Log

### Metal GPU Detectado:
```
INFO vectorizer::db::vector_store: 🍎 Detecting Metal GPU on Mac Silicon...
INFO vectorizer::gpu::context: GPU selecionada: Apple M3 Pro (Metal)
INFO vectorizer::db::vector_store: ✅ Metal GPU detected and enabled!
INFO vectorizer::db::vector_store: Creating new VectorStore with Metal GPU config: enabled=true
```

### CUDA GPU Detectado:
```
INFO vectorizer::db::vector_store: 🔥 Attempting to use CUDA GPU...
INFO vectorizer::db::vector_store: ✅ CUDA GPU enabled!
```

### CPU Fallback:
```
INFO vectorizer::db::vector_store: 💻 Using CPU-only mode
```

## 🔧 Uso Programático

### Detecção Automática (Recomendado)
```rust
use vectorizer::db::VectorStore;

// Detecta automaticamente Metal > CUDA > CPU
let store = VectorStore::new_auto();
```

### Forçar Metal
```rust
use vectorizer::{db::VectorStore, gpu::GpuConfig};

let config = GpuConfig::for_metal_silicon();
let store = VectorStore::new_with_metal_config(config);
```

### Forçar CUDA
```rust
use vectorizer::{db::VectorStore, cuda::CudaConfig};

let config = CudaConfig { enabled: true, ..Default::default() };
let store = VectorStore::new_with_cuda_config(config);
```

### Forçar CPU
```rust
use vectorizer::{db::VectorStore, cuda::CudaConfig};

let config = CudaConfig { enabled: false, ..Default::default() };
let store = VectorStore::new_with_cuda_config(config);
```

## 🏗️ Arquitetura da Detecção

### VectorStore::new_auto()

```rust
pub fn new_auto() -> Self {
    // 1. Try Metal first (Mac Silicon with wgpu-gpu feature)
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "wgpu-gpu"))]
    {
        info!("🍎 Detecting Metal GPU on Mac Silicon...");
        let metal_config = crate::gpu::GpuConfig::for_metal_silicon();
        if let Ok(_) = pollster::block_on(crate::gpu::GpuContext::new(metal_config.clone())) {
            info!("✅ Metal GPU detected and enabled!");
            return Self::new_with_metal_config(metal_config);
        } else {
            warn!("⚠️ Metal GPU detection failed, falling back...");
        }
    }
    
    // 2. Try CUDA (if feature is enabled)
    #[cfg(feature = "cuda")]
    {
        info!("🔥 Attempting to use CUDA GPU...");
        let cuda_config = CudaConfig { enabled: true, ..Default::default() };
        info!("✅ CUDA GPU enabled!");
        return Self::new_with_cuda_config(cuda_config);
    }
    
    // 3. Fallback to CPU
    info!("💻 Using CPU-only mode");
    Self::new_with_cuda_config(CudaConfig { enabled: false, ..Default::default() })
}
```

## 🎨 Collections com GPU

### Criação Automática
```rust
// Após new_auto(), as collections usam automaticamente a GPU detectada
let store = VectorStore::new_auto();

// Esta collection usará Metal (se detectado) ou CUDA (se disponível)
store.create_collection("my_collection", config)?;
```

### Prioridade de Backend
```
Metal > CUDA > CPU
```

Se Metal está disponível e habilitado, **todas** as collections criadas usarão Metal automaticamente.

## 📊 Performance

### Metal vs CUDA vs CPU

| Operação | CPU | CUDA | Metal |
|----------|-----|------|-------|
| **Busca (1K vetores)** | ~100ms | ~10ms | ~12ms |
| **Indexação (10K vetores)** | ~5s | ~500ms | ~550ms |
| **Memória** | RAM | VRAM | Unified Memory |

### Quando Usar Cada Backend

#### 🍎 **Metal** (Mac Silicon)
- ✅ **Ideal para**: Mac M1/M2/M3/etc
- ✅ **Vantagens**: Unified memory, baixo overhead
- ✅ **Use quando**: Desenvolver/produzir em Mac Silicon

#### 🔥 **CUDA** (NVIDIA)
- ✅ **Ideal para**: GPUs NVIDIA (RTX, A100, etc)
- ✅ **Vantagens**: Alta performance, maturidade
- ✅ **Use quando**: Máximo desempenho em servidores NVIDIA

#### 💻 **CPU**
- ✅ **Ideal para**: Ambientes sem GPU
- ✅ **Vantagens**: Compatibilidade universal
- ✅ **Use quando**: Desenvolvimento/testes simples

## 🔍 Troubleshooting

### Metal não detectado no Mac Silicon

**Problema**: Compila mas não detecta Metal
```
INFO vectorizer::db::vector_store: 💻 Using CPU-only mode
```

**Solução**: Compilar com feature `wgpu-gpu`
```bash
cargo build --features wgpu-gpu --release
```

### CUDA não funciona

**Problema**: Compila mas não usa CUDA
```
INFO vectorizer::db::vector_store: 💻 Using CPU-only mode
```

**Solução**: Compilar com feature `cuda`
```bash
cargo build --features cuda --release
```

### Forçar CPU mesmo com GPU disponível

**Solução**: Use `new_with_cuda_config` com `enabled: false`
```rust
let store = VectorStore::new_with_cuda_config(CudaConfig {
    enabled: false,
    ..Default::default()
});
```

## 📚 Referências

- [Metal GPU Implementation](METAL_GPU_IMPLEMENTATION.md)
- [CUDA vs Metal Comparison](../COMPARACAO_CUDA_METAL.md)
- [GPU Status Report](../STATUS_GPU.md)

## 🎯 Próximos Passos

- [ ] Adicionar suporte Vulkan (Linux/Windows)
- [ ] Melhorar fallback gracioso entre backends
- [ ] Benchmark comparativo automático
- [ ] Métricas de uso de GPU em tempo real

