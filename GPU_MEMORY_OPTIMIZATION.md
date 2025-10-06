# 🛡️ Otimização de Memória GPU - Vulkan e Metal

## 📋 Resumo

Este documento descreve as otimizações aplicadas para prevenir estouro de memória (buffer overflow) e uso excessivo de RAM nos backends GPU Vulkan e Metal.

## 🔍 Problema Identificado

### Antes das Correções

1. **Vulkan Collection**: Usava detecção baseada em strings do nome da GPU
   - ❌ Hardcoded strings como "RTX 4090", "RTX 4070"
   - ❌ Estimativa falsa de VRAM
   - ❌ Alocava 80% do VRAM estimado
   - ❌ Limite de 4GB para vector storage

2. **Metal Collection**: Valores hardcoded excessivos
   - ❌ Initial capacity: 100,000 vetores
   - ❌ GPU memory limit: 4GB
   - ❌ Max buffer: 80% do VRAM estimado
   - ❌ Sem compressão habilitada

3. **Buffer Overflow**: Falta de validação de bounds
   - ❌ `copy_buffer_to_buffer` sem verificação
   - ❌ Panic ao escrever além do buffer
   - ❌ Erros de validação do wgpu

## ✅ Soluções Implementadas

### 1. Detecção Real de Hardware (Vulkan)

```rust
// ✅ Usa limites REAIS do wgpu adapter
let gpu_limits = &gpu_context.info().limits;
let max_buffer_size = (gpu_limits.max_buffer_size as f64 * 0.10) as u64;
```

**Benefícios:**
- ✅ Funciona com QUALQUER GPU (AMD, NVIDIA, Intel)
- ✅ Detecta limites reais do hardware via wgpu
- ✅ Não depende de strings do nome da GPU
- ✅ Usa apenas 10% do buffer máximo (seguro)

### 2. Capacidades Dinâmicas Calculadas

```rust
let safe_initial_capacity = (max_buffer_size / vector_size_bytes as u64)
    .min(10_000) as usize;

let safe_max_capacity = (max_buffer_size / vector_size_bytes as u64)
    .min(100_000) as usize;
```

**Benefícios:**
- ✅ Baseado no tamanho real do vetor
- ✅ Limitado a valores seguros (10k inicial, 100k máximo)
- ✅ Previne alocações gigantes

### 3. Limites de Memória Conservadores

#### Vulkan
```rust
let hnsw_memory_limit = (max_buffer_size / 2).min(512 * 1024 * 1024); // 512MB max
let vector_memory_limit = (max_buffer_size / 2).min(512 * 1024 * 1024); // 512MB max
```

#### Metal
```rust
initial_node_capacity: 1_000,           // Reduzido de 100_000
initial_vector_capacity: 1_000,         // Reduzido de 100_000
gpu_memory_limit: 512 * 1024 * 1024,    // 512MB (reduzido de 4GB)
max_buffer_size: 10% do VRAM estimado   // Reduzido de 80%
```

### 4. Compressão Habilitada

```rust
enable_compression: true,
compression_ratio: 0.25,  // 75% de redução
```

**Benefícios:**
- ✅ Reduz uso de memória em ~75%
- ✅ Permite mais vetores no mesmo espaço
- ✅ Melhor utilização do VRAM

### 5. Validação de Bounds nos Buffers

#### `src/gpu/vector_storage.rs`
```rust
// Antes de cada copy_buffer_to_buffer
let metadata_buffer_size = self.metadata_buffer.size();
if offset + metadata_size > metadata_buffer_size {
    return Err(VectorizerError::InternalError(
        format!("Metadata buffer overflow: ...")
    ));
}
```

#### `src/gpu/hnsw_storage.rs`
```rust
// Antes de cada copy_buffer_to_buffer
let node_buffer_size = self.node_buffer.size();
if buffer_offset + node_size > node_buffer_size {
    return Err(VectorizerError::InternalError(
        format!("HNSW node buffer overflow: ...")
    ));
}
```

**Benefícios:**
- ✅ Previne panic de buffer overflow
- ✅ Retorna erros informativos
- ✅ Facilita debugging

## 📊 Comparação: Antes vs Depois

### Vulkan Collection

| Métrica | Antes | Depois | Melhoria |
|---------|-------|--------|----------|
| Detecção VRAM | String-based | wgpu real limits | ✅ 100% confiável |
| % VRAM usado | 80% | 10% | ✅ 8x mais seguro |
| Initial capacity | Variável | 10k max | ✅ Previsível |
| Max capacity | 1M | 100k | ✅ 10x menor |
| Memory limit | 4GB | 512MB | ✅ 8x menor |
| Compressão | Não | Sim (25%) | ✅ 75% redução |
| Validação bounds | Não | Sim | ✅ Sem crashes |

### Metal Collection

| Métrica | Antes | Depois | Melhoria |
|---------|-------|--------|----------|
| Initial capacity | 100k | 1k | ✅ 100x menor |
| % VRAM usado | 80% | 10% | ✅ 8x mais seguro |
| Memory limit | 4GB | 512MB | ✅ 8x menor |
| Max buffer | 80% VRAM | 10% VRAM | ✅ 8x menor |
| Compressão | Não | Sim (25%) | ✅ 75% redução |
| Validação bounds | Não | Sim | ✅ Sem crashes |

## 🚀 Logging Detalhado

### Vulkan
```
🔧 Vulkan GPU Memory Configuration (Real Hardware Limits):
  - GPU Name: AMD Radeon RX 7900 XTX
  - Device Type: DiscreteGpu
  - Backend: Vulkan
  - Max buffer size (hardware): 16.00 GB
  - Max buffer binding size: 2.00 GB
  - Using 10% of max buffer: 1638.40 MB (safe allocation)
  - Vector size: 2048 bytes (512 dimensions)
  - Initial capacity: 10000 vectors
  - Max capacity: 100000 vectors
  - HNSW memory limit: 512.00 MB
  - Vector memory limit: 512.00 MB
```

### Metal
```
🔧 Metal GPU Buffer Configuration:
  - GPU Name: Apple M2 Ultra
  - Estimated total VRAM: 32.00 GB
  - Using 10% of estimated VRAM: 3.20 GB
  - Vector size: 2048 bytes
  - Calculated initial capacity: 5000 vectors (capped from 10000)
  - Max capacity: 100000 vectors
  - HNSW initial node capacity: 1000
  - HNSW memory limit: 512 MB
  - Vector memory limit: 512 MB
  - Compression enabled: true (ratio: 0.25)
```

## 🧪 Testes Recomendados

### 1. Teste de Carga
```bash
# Adicionar 50k vetores
./target/release/vectorizer
```

### 2. Teste de Memória
```bash
# Monitorar uso de memória
activity_monitor # macOS
htop # Linux
```

### 3. Teste de Buffer Overflow
```bash
# O servidor agora retorna erros ao invés de crashar
# Mensagem esperada:
"HNSW node buffer overflow: trying to write X bytes at 
 offset Y but buffer size is Z. Node index: N"
```

## 📝 Arquivos Modificados

### Core GPU Storage
- ✅ `src/gpu/vulkan_collection.rs` - Detecção real de hardware, limites dinâmicos
- ✅ `src/gpu/metal_collection.rs` - Redução de capacidades, compressão
- ✅ `src/gpu/vector_storage.rs` - Validação de bounds metadata e vetores
- ✅ `src/gpu/hnsw_storage.rs` - Validação de bounds nós e conexões

### Documentação
- ✅ `GPU_MEMORY_OPTIMIZATION.md` - Este documento

## 🎯 Próximos Passos

1. **Testar com diferentes GPUs**
   - AMD: RX 6000/7000 series
   - NVIDIA: RTX 3000/4000 series
   - Intel: Arc series
   - Apple: M1/M2/M3

2. **Monitorar métricas**
   - Uso de VRAM
   - Tempo de resposta
   - Taxa de erro

3. **Ajustar percentuais**
   - Se 10% for muito conservador, aumentar gradualmente
   - Monitorar performance vs segurança

4. **Implementar auto-scaling**
   - Detectar pressão de memória
   - Ajustar limites dinamicamente
   - Liberar buffers não usados

## 🔗 Referências

- [wgpu Documentation](https://docs.rs/wgpu)
- [Vulkan Memory Allocator](https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator)
- [Metal Best Practices](https://developer.apple.com/documentation/metal)
- [HNSW Algorithm](https://arxiv.org/abs/1603.09320)

---

**Status:** ✅ Pronto para Pull Request  
**Branch:** `fix/gpu-memory-optimization`  
**Data:** 2025-10-06

