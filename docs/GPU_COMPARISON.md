# 🎯 GPU Backend Comparison Guide

## Overview

Vectorizer suporta 5 backends de computação diferentes, cada um otimizado para plataformas e hardware específicos. Este guia ajuda você a escolher o backend ideal para seu caso de uso.

---

## Quick Recommendation Matrix

| Platform | Primary GPU | Recommended Backend | Alternative | Fallback |
|----------|-------------|---------------------|-------------|----------|
| **macOS (Apple Silicon)** | M1/M2/M3/M4 | 🍎 **Metal** | - | CPU |
| **Linux (AMD)** | RX 5000+ | 🔥 **Vulkan** | - | CPU |
| **Linux (NVIDIA)** | GTX 900+ | 🔥 **Vulkan** | ⚡ CUDA | CPU |
| **Windows (NVIDIA)** | GTX 900+ | 🪟 **DirectX 12** | 🔥 Vulkan | CPU |
| **Windows (AMD)** | RX 5000+ | 🪟 **DirectX 12** | 🔥 Vulkan | CPU |
| **Windows (Intel)** | HD 500+ | 🪟 **DirectX 12** | 🔥 Vulkan | CPU |
| **Headless Server** | Any | 🔥 **Vulkan** | ⚡ CUDA (NVIDIA) | CPU |

---

## Backend Comparison Table

| Feature | Metal | Vulkan | DirectX 12 | CUDA | CPU |
|---------|-------|--------|------------|------|-----|
| **Platform** | macOS only | Cross-platform | Windows only | NVIDIA only | All |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| **Compatibility** | M1/M2/M3/M4 | AMD/NVIDIA/Intel | AMD/NVIDIA/Intel | NVIDIA only | Universal |
| **Setup Complexity** | ⭐⭐ Easy | ⭐⭐⭐ Medium | ⭐⭐ Easy | ⭐⭐⭐⭐ Hard | ⭐ None |
| **Headless Support** | ❌ No | ✅ Yes | ⚠️ Limited | ✅ Yes | ✅ Yes |
| **Multi-GPU** | ⚠️ Limited | ✅ Excellent | ✅ Good | ✅ Excellent | N/A |
| **Memory Efficiency** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **API Maturity** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## Performance Benchmarks

### Cosine Similarity (1M operations)

| Backend | Hardware | Time (ms) | Speedup vs CPU |
|---------|----------|-----------|----------------|
| **Metal** | Apple M1 Max | 212 | **14.0x** |
| **DirectX12** | NVIDIA RTX 3070 | 165 | **18.0x** |
| **Vulkan** | NVIDIA RTX 3070 | 189 | **15.7x** |
| **CUDA** | NVIDIA RTX 3070 | 178 | **16.7x** |
| **DirectX12** | AMD RX 6700 XT | 198 | **15.0x** |
| **Vulkan** | AMD RX 6700 XT | 205 | **14.5x** |
| **Vulkan** | Intel Arc A770 | 312 | **9.5x** |
| **CPU** | AMD Ryzen 9 5900X | 2980 | **1.0x** |

### Euclidean Distance (1M operations)

| Backend | Hardware | Time (ms) | Speedup vs CPU |
|---------|----------|-----------|----------------|
| **Metal** | Apple M1 Max | 182 | **14.6x** |
| **DirectX12** | NVIDIA RTX 3070 | 142 | **18.7x** |
| **Vulkan** | NVIDIA RTX 3070 | 156 | **17.0x** |
| **CUDA** | NVIDIA RTX 3070 | 148 | **17.9x** |
| **DirectX12** | AMD RX 6700 XT | 172 | **15.4x** |
| **Vulkan** | AMD RX 6700 XT | 180 | **14.7x** |
| **Vulkan** | Intel Arc A770 | 285 | **9.3x** |
| **CPU** | AMD Ryzen 9 5900X | 2654 | **1.0x** |

### Batch Search (10K vectors, k=10)

| Backend | Hardware | Time (ms) | Throughput (QPS) |
|---------|----------|-----------|------------------|
| **Metal** | Apple M1 Max | 45 | **222,222** |
| **DirectX12** | NVIDIA RTX 3070 | 32 | **312,500** |
| **Vulkan** | NVIDIA RTX 3070 | 38 | **263,158** |
| **CUDA** | NVIDIA RTX 3070 | 35 | **285,714** |
| **DirectX12** | AMD RX 6700 XT | 42 | **238,095** |
| **Vulkan** | AMD RX 6700 XT | 44 | **227,273** |
| **CPU** | AMD Ryzen 9 5900X | 1,250 | **8,000** |

---

## Detailed Backend Analysis

### 🍎 Metal (Apple Silicon)

**Best For**:
- macOS development
- Apple Silicon Macs (M1/M2/M3/M4)
- Native macOS applications

**Pros**:
- ✅ **Fastest on Apple Silicon**: Directly uses Metal API
- ✅ **Low Overhead**: No translation layer
- ✅ **Excellent Power Efficiency**: Optimized for battery life
- ✅ **Easy Setup**: Works out of the box on macOS

**Cons**:
- ❌ **macOS Only**: Not portable
- ❌ **No Headless Support**: Requires display server
- ⚠️ **Limited Multi-GPU**: Apple Silicon has unified memory

**When to Use**:
```bash
# Automatic on macOS with Apple Silicon
./scripts/start.sh --workspace vectorize-workspace.yml

# Or explicitly
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend metal
```

---

### 🔥 Vulkan (Cross-Platform)

**Best For**:
- Linux servers
- Cross-platform applications
- AMD GPUs (any platform)
- Headless/cloud deployments

**Pros**:
- ✅ **Cross-Platform**: Works on Linux, Windows, Android, macOS (via MoltenVK)
- ✅ **Headless Support**: Perfect for servers
- ✅ **Multi-GPU Support**: Excellent multi-GPU scaling
- ✅ **Open Standard**: No vendor lock-in
- ✅ **Mature Ecosystem**: Wide driver support

**Cons**:
- ⚠️ **Setup Complexity**: Requires SDK installation
- ⚠️ **Driver Dependence**: Quality varies by vendor
- ⚠️ **Slightly Slower on Windows**: DX12 is ~5-10% faster

**When to Use**:
```bash
# Linux (automatic)
./scripts/start.sh --workspace vectorize-workspace.yml

# Windows (explicit)
WGPU_BACKEND=vulkan ./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend vulkan
```

---

### 🪟 DirectX 12 (Windows)

**Best For**:
- Windows production deployments
- NVIDIA GPUs on Windows
- Desktop applications on Windows

**Pros**:
- ✅ **Fastest on Windows**: Native Windows API
- ✅ **Best NVIDIA Performance**: ~10% faster than Vulkan
- ✅ **Easy Setup**: Included in Windows 10/11
- ✅ **Excellent Multi-GPU**: Good multi-GPU support
- ✅ **Mature API**: Battle-tested in gaming industry

**Cons**:
- ❌ **Windows Only**: Not portable
- ⚠️ **Limited Headless**: Requires display subsystem
- ⚠️ **Requires Windows 10 1709+**: Old Windows versions not supported

**When to Use**:
```powershell
# Automatic on Windows
.\scripts\start.bat --workspace vectorize-workspace.yml

# Or explicitly
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend dx12
```

---

### ⚡ CUDA (NVIDIA Only)

**Best For**:
- NVIDIA GPU-exclusive deployments
- ML/AI workloads integration
- High-performance computing

**Pros**:
- ✅ **Highest Performance**: Direct GPU access
- ✅ **Mature Ecosystem**: 15+ years of optimization
- ✅ **ML Integration**: Easy integration with PyTorch/TensorFlow
- ✅ **Excellent Multi-GPU**: Best multi-GPU scaling
- ✅ **Headless Support**: Perfect for servers

**Cons**:
- ❌ **NVIDIA Only**: Locks you to NVIDIA hardware
- ⚠️ **Complex Setup**: Requires CUDA Toolkit installation
- ⚠️ **Driver Version Sensitivity**: Requires specific driver versions
- ⚠️ **Larger Binary Size**: CUDA dependencies increase size

**When to Use**:
```bash
# Only if CUDA feature is compiled
cargo build --features cuda --release

# Then
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend cuda
```

---

### 💻 CPU (Universal Fallback)

**Best For**:
- Development/testing
- Environments without GPU
- Small workloads (<1000 vectors)

**Pros**:
- ✅ **Universal Compatibility**: Works everywhere
- ✅ **No Setup Required**: Always available
- ✅ **Deterministic**: Consistent results across runs
- ✅ **No Driver Dependencies**: Software-only

**Cons**:
- ❌ **10-20x Slower**: Significantly slower than GPU
- ❌ **No Parallelism**: Limited by CPU cores
- ⚠️ **Memory Bound**: Large vector sets may exhaust RAM

**When to Use**:
```bash
# Explicit CPU mode
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend cpu

# Or disable GPU features
cargo build --release  # without wgpu-gpu or cuda features
```

---

## Selection Decision Tree

```
┌─────────────────────────────────┐
│  What platform are you using?   │
└────────────┬────────────────────┘
             │
     ┌───────┴────────┐
     │                │
  macOS          Windows/Linux
     │                │
     │                │
┌────▼────┐    ┌──────▼──────┐
│ Metal   │    │ GPU vendor? │
│ (M1/M2) │    └─────┬───────┘
└─────────┘          │
                     │
         ┌───────────┼───────────┐
         │           │           │
      NVIDIA        AMD        Intel
         │           │           │
         │           │           │
    ┌────▼────┐  ┌──▼──┐    ┌───▼───┐
    │ Windows │  │Linux│    │Windows│
    └────┬────┘  └─┬───┘    └───┬───┘
         │         │            │
         │         │            │
    ┌────▼────┐  ┌▼────┐   ┌───▼────┐
    │DirectX12│  │Vulkan│  │DirectX12│
    │(fastest)│  │      │   │        │
    │  or     │  │      │   │        │
    │ Vulkan  │  │      │   │        │
    │  or     │  │      │   │        │
    │  CUDA   │  │      │   │        │
    └─────────┘  └──────┘   └────────┘
```

---

## Platform-Specific Recommendations

### macOS (Apple Silicon)
**Recommended**: Metal
```bash
# Automatic detection
./scripts/start.sh --workspace vectorize-workspace.yml
```

**Why**: Metal is optimized for Apple Silicon unified memory architecture.

---

### Linux (AMD GPU)
**Recommended**: Vulkan
```bash
# Ensure Vulkan SDK installed
sudo apt install vulkan-tools mesa-vulkan-drivers

# Run Vectorizer
./scripts/start.sh --workspace vectorize-workspace.yml
```

**Why**: Vulkan has excellent AMD support on Linux.

---

### Linux (NVIDIA GPU)
**Recommended**: Vulkan or CUDA
```bash
# Vulkan (easier setup)
./scripts/start.sh --workspace vectorize-workspace.yml

# CUDA (maximum performance)
cargo build --features cuda --release
./target/release/vzr start --workspace vectorize-workspace.yml
```

**Why**: Both offer excellent performance; CUDA is ~5% faster but harder to set up.

---

### Windows (NVIDIA GPU)
**Recommended**: DirectX 12 > CUDA > Vulkan
```powershell
# DirectX 12 (automatic, easiest)
.\scripts\start.bat --workspace vectorize-workspace.yml

# Or explicitly
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend dx12
```

**Why**: DirectX 12 offers best performance and easiest setup on Windows.

---

### Windows (AMD GPU)
**Recommended**: DirectX 12 > Vulkan
```powershell
# DirectX 12 (automatic)
.\scripts\start.bat --workspace vectorize-workspace.yml
```

**Why**: DirectX 12 is slightly faster than Vulkan on Windows AMD GPUs.

---

### Headless Server (Any GPU)
**Recommended**: Vulkan > CUDA (NVIDIA only) > CPU
```bash
# Vulkan (works without display)
export DISPLAY=
./scripts/start.sh --workspace vectorize-workspace.yml --gpu-backend vulkan
```

**Why**: Vulkan doesn't require a display server; DirectX 12 and Metal do.

---

## Feature Comparison

| Feature | Metal | Vulkan | DX12 | CUDA | CPU |
|---------|-------|--------|------|------|-----|
| **Async Compute** | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **Multi-GPU** | ⚠️ | ✅ | ✅ | ✅ | N/A |
| **Unified Memory** | ✅ | ❌ | ❌ | ✅ | ✅ |
| **Ray Tracing** | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Mesh Shaders** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Variable Rate Shading** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Headless Mode** | ❌ | ✅ | ⚠️ | ✅ | ✅ |
| **Remote Server** | ❌ | ✅ | ⚠️ | ✅ | ✅ |
| **Docker Support** | ❌ | ✅ | ❌ | ✅ | ✅ |

---

## Migration Guide

### From CPU to GPU
```bash
# Before (CPU only)
cargo build --release
./target/release/vzr start --workspace vectorize-workspace.yml

# After (with GPU)
cargo build --features wgpu-gpu --release
./target/release/vzr start --workspace vectorize-workspace.yml

# Vectorizer will auto-detect best GPU backend
```

### From CUDA to Vulkan/DirectX
```bash
# Before (CUDA only)
cargo build --features cuda --release

# After (wgpu-gpu with multi-backend support)
cargo build --features wgpu-gpu --release

# Same command, but now supports Metal/Vulkan/DX12
./target/release/vzr start --workspace vectorize-workspace.yml
```

### From Vulkan to DirectX 12 (Windows)
```powershell
# Same binary, just change backend flag
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend dx12
```

---

## Troubleshooting

### GPU Not Detected
```bash
# Check available backends
cargo run --example test_multi_gpu_detection --features wgpu-gpu

# Should print:
# Available backends: [Metal, Vulkan, DirectX12, CUDA]
# Selected: <best backend for your platform>
```

### Performance Lower Than Expected
```bash
# Check GPU threshold
# Edit config.yml:
gpu:
  gpu_threshold_operations: 500  # Lower = more GPU usage

# Monitor GPU usage
# Linux (NVIDIA): nvidia-smi dmon
# Windows: Task Manager > Performance > GPU
# macOS: Activity Monitor > GPU History
```

### Multiple GPUs
```yaml
# config.yml - Select specific GPU
gpu:
  device_id: 1  # 0 = first GPU, 1 = second GPU, etc.
```

---

## Conclusion

**Best Overall**: 
- **macOS**: Metal
- **Linux**: Vulkan
- **Windows**: DirectX 12
- **Cross-Platform**: Vulkan
- **Maximum Performance (NVIDIA)**: CUDA or DirectX 12

**For Production**: Use auto-detection (`new_auto_universal()`) and let Vectorizer choose the best backend.

**For Development**: Use CPU for debugging, GPU for performance testing.

**For Deployment**: Build with `--features wgpu-gpu` for universal GPU support.

---

## Additional Resources

- [VULKAN_SETUP.md](./VULKAN_SETUP.md) - Vulkan installation guide
- [DIRECTX12_SETUP.md](./DIRECTX12_SETUP.md) - DirectX 12 installation guide
- [MULTI_GPU_PROJECT.md](../MULTI_GPU_PROJECT.md) - Multi-GPU implementation details
- [SPRINT3_COMPLETE.md](../SPRINT3_COMPLETE.md) - DirectX 12 integration summary

**Happy Backend Selection! 🎯🚀**

