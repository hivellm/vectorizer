# 🔥 Vulkan GPU Setup Guide

## Overview

Vulkan é uma API gráfica multiplataforma de baixo nível que oferece suporte robusto para AMD, NVIDIA, Intel e outros fabricantes de GPUs. O Vectorizer utiliza Vulkan via `wgpu` para acelerar operações vetoriais em Windows, Linux e outros sistemas compatíveis.

---

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| **Linux** | ✅ Excellent | Native support via Mesa/proprietary drivers |
| **Windows** | ✅ Excellent | Native support via manufacturer drivers |
| **Android** | ✅ Good | Mobile GPU support |
| **macOS** | ⚠️ Limited | Via MoltenVK (Metal translation layer) |

---

## Prerequisites

### 1. Vulkan SDK Installation

#### Linux (Ubuntu/Debian)
```bash
# Install Vulkan SDK and tools
sudo apt update
sudo apt install -y vulkan-tools vulkan-validationlayers

vulkan-sdk libvulkan-dev vulkan-headers

# For AMD GPUs
sudo apt install -y mesa-vulkan-drivers

# For NVIDIA GPUs
sudo apt install -y nvidia-driver-<version> # e.g., nvidia-driver-530

# Verify installation
vulkaninfo | grep "Vulkan Instance Version"
```

#### Linux (Fedora/RHEL)
```bash
# Install Vulkan SDK
sudo dnf install -y vulkan-tools vulkan-validation-layers vulkan-loader-devel

# For AMD GPUs
sudo dnf install -y mesa-vulkan-drivers

# For NVIDIA GPUs
sudo dnf install -y nvidia-driver

# Verify
vulkaninfo | grep "Vulkan Instance Version"
```

#### Windows
1. Download Vulkan SDK from: https://vulkan.lunarg.com/sdk/home#windows
2. Run installer (e.g., `VulkanSDK-1.3.xxx-Installer.exe`)
3. Add to PATH: `C:\VulkanSDK\<version>\Bin`
4. Install GPU drivers:
   - **AMD**: AMD Software Adrenalin Edition
   - **NVIDIA**: GeForce Experience / Quadro drivers
   - **Intel**: Intel Graphics Driver

5. Verify installation:
```powershell
# Open PowerShell
vulkaninfo
```

#### macOS (via MoltenVK)
```bash
# Install Vulkan SDK via Homebrew
brew install --cask vulkan-sdk

# Or download from: https://vulkan.lunarg.com/sdk/home#mac

# Verify (may have limited output due to Metal translation)
vulkaninfo | grep "Vulkan Instance Version"
```

---

### 2. GPU Driver Updates

#### AMD GPUs
**Linux:**
```bash
# Ubuntu/Debian
sudo add-apt-repository ppa:kisak/kisak-mesa
sudo apt update && sudo apt upgrade

# Fedora
sudo dnf upgrade mesa-*
```

**Windows:**
- Download AMD Software from: https://www.amd.com/en/support
- Install latest drivers (Adrenalin Edition)

#### NVIDIA GPUs
**Linux:**
```bash
# Ubuntu/Debian
sudo add-apt-repository ppa:graphics-drivers/ppa
sudo apt update
sudo apt install nvidia-driver-535 # or latest version

# Verify
nvidia-smi
```

**Windows:**
- Download from: https://www.nvidia.com/Download/index.aspx
- Install GeForce Experience or Studio Drivers

#### Intel GPUs
**Linux:**
```bash
# Ubuntu/Debian
sudo apt install intel-media-va-driver-non-free

# Fedora
sudo dnf install intel-media-driver
```

**Windows:**
- Download Intel Graphics Driver from: https://www.intel.com/content/www/us/en/download-center/home.html

---

## Building Vectorizer with Vulkan Support

### Prerequisites
```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update Rust
rustup update stable
```

### Build with Vulkan (via wgpu-gpu feature)
```bash
cd /path/to/vectorizer

# Build in release mode with Vulkan support
cargo build --features wgpu-gpu --release

# This enables:
# - Metal (macOS)
# - Vulkan (Linux/Windows/Android)
# - DirectX 12 (Windows)
# - CPU fallback (all platforms)
```

### Build with explicit Vulkan backend
```bash
# Force Vulkan backend
WGPU_BACKEND=vulkan cargo build --features wgpu-gpu --release
```

---

## Running Vectorizer with Vulkan

### 1. Auto-detection (Recommended)
```bash
# Let Vectorizer auto-select the best backend
# Priority: Metal (macOS) > Vulkan > DirectX12 > CUDA > CPU
./scripts/start.sh --workspace vectorize-workspace.yml
```

### 2. Explicit Vulkan Selection
```bash
# Force Vulkan backend via environment variable
WGPU_BACKEND=vulkan ./scripts/start.sh --workspace vectorize-workspace.yml

# Or via CLI flag (after Sprint 4.2 completion)
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend vulkan
```

### 3. Verification
```bash
# Check logs for GPU detection
tail -f vectorizer-workspace.log | grep "GPU"

# Expected output:
# 🔍 VectorStore::new_auto_universal() - Universal Multi-GPU Detection
# 🔥 Initializing Vulkan GPU backend...
# ✅ Vulkan GPU initialized!
```

---

## Troubleshooting

### 1. "No GPU backends detected"
**Cause**: Vulkan SDK not installed or drivers outdated

**Solution**:
```bash
# Linux
sudo apt install vulkan-tools vulkan-validation layers mesa-vulkan-drivers

# Verify Vulkan is working
vulkaninfo | grep "GPU id"

# Should list available GPUs
```

### 2. "Vulkan initialization failed"
**Cause**: Driver incompatibility or missing validation layers

**Solution**:
```bash
# Disable validation layers
export VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT=1

# Or update drivers
# AMD:
sudo apt upgrade mesa-* # Linux
# NVIDIA:
sudo apt install nvidia-driver-535 # Linux
```

### 3. "vkCreateInstance failed with VK_ERROR_INCOMPATIBLE_DRIVER"
**Cause**: Vulkan SDK version mismatch

**Solution**:
```bash
# Linux - Reinstall Vulkan SDK
sudo apt remove --purge vulkan-*
sudo apt autoremove
sudo apt install vulkan-tools vulkan-validationlayers mesa-vulkan-drivers

# Windows - Reinstall Vulkan SDK
# Download from https://vulkan.lunarg.com/sdk/home#windows
```

### 4. Performance Issues
**Symptoms**: GPU usage is low, CPU usage is high

**Diagnosis**:
```bash
# Check GPU usage (Linux)
watch -n 1 nvidia-smi # NVIDIA
watch -n 1 radeontop # AMD

# Check GPU usage (Windows)
# Task Manager > Performance > GPU

# If GPU usage is <20%, check:
# 1. gpu_threshold_operations in GpuConfig (default: 1000)
# 2. Vector batch size (should be >1000 for GPU benefit)
```

**Solution**:
```rust
// Adjust in src/gpu/config.rs or via config.yml
gpu_config:
  enabled: true
  gpu_threshold_operations: 500  # Lower threshold for more GPU usage
  preferred_backend: vulkan
```

### 5. Vulkan Validation Layer Errors
**Cause**: Debug mode validation errors

**Solution**:
```bash
# Disable validation layers
export VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT=1
export VK_LOADER_DEBUG=none

# Or build in release mode (validation disabled by default)
cargo build --features wgpu-gpu --release
```

---

## Performance Tips

### 1. Optimize for Large Batches
```rust
// Vectorizer automatically uses GPU for batches > gpu_threshold_operations
// Ensure your searches use batch operations
let queries = vec![query1, query2, ..., queryN]; // N > 1000
let results = store.batch_search("collection", queries, 10)?;
```

### 2. Monitor GPU Utilization
```bash
# Linux (NVIDIA)
nvidia-smi dmon -s u

# Linux (AMD)
radeontop

# Windows
# Task Manager > Performance > GPU > Copy engine
```

### 3. Adjust GPU Configuration
```yaml
# config.yml
gpu:
  enabled: true
  backend: vulkan
  device_id: 0 # First GPU
  power_preference: high_performance
  gpu_threshold_operations: 500
```

---

## Platform-Specific Notes

### Linux
- **Best Support**: Vulkan is first-class on Linux
- **Multi-GPU**: Automatic selection of most powerful GPU
- **Headless Servers**: Vulkan works without X11/Wayland

### Windows
- **DirectX 12 Alternative**: Windows users may prefer DX12 (see DIRECTX12_SETUP.md)
- **Vulkan Layers**: May cause slight overhead vs DX12

### macOS
- **MoltenVK Translation**: Adds ~10-15% overhead
- **Prefer Metal**: Use Metal directly for better performance on macOS

---

## Benchmarks

### Vulkan vs Other Backends

| Backend | Platform | Cosine Similarity (1M ops) | Euclidean Distance (1M ops) |
|---------|----------|----------------------------|------------------------------|
| **Vulkan** | Linux AMD | 245 ms | 198 ms |
| **Vulkan** | Windows NVIDIA | 189 ms | 156 ms |
| **DirectX12** | Windows NVIDIA | 175 ms | 148 ms |
| **Metal** | macOS M1 | 212 ms | 182 ms |
| **CPU** | All | 3,450 ms | 2,980 ms |

*Results from `benchmark/reports/multi_gpu_comparison.json`*

---

## FAQ

### Q: Can I use Vulkan with NVIDIA GPUs?
**A**: Yes! NVIDIA has excellent Vulkan support. DirectX 12 may be slightly faster on Windows, but Vulkan offers cross-platform consistency.

### Q: Does Vulkan work on headless servers?
**A**: Yes! Vulkan doesn't require a display server (X11/Wayland). Perfect for cloud/headless deployments.

### Q: How do I force Vulkan over DirectX 12 on Windows?
**A**:
```bash
WGPU_BACKEND=vulkan ./target/release/vzr start --workspace vectorize-workspace.yml
# Or
./target/release/vzr start --workspace vectorize-workspace.yml --gpu-backend vulkan
```

### Q: What's the minimum Vulkan version required?
**A**: Vulkan 1.1 or later (most GPUs from 2016+ support this)

### Q: Can I run Vulkan and CUDA simultaneously?
**A**: No. Vectorizer selects one backend at a time. Priority: Metal > Vulkan > DirectX12 > CUDA > CPU

---

## Additional Resources

- **Vulkan Official Site**: https://vulkan.org/
- **Vulkan SDK Downloads**: https://vulkan.lunarg.com/sdk/home
- **wgpu Documentation**: https://wgpu.rs/
- **AMD Vulkan Drivers**: https://www.amd.com/en/technologies/vulkan
- **NVIDIA Vulkan Drivers**: https://developer.nvidia.com/vulkan-driver

---

## Support

If you encounter issues:
1. Check `vectorizer-workspace.log` for error messages
2. Run `vulkaninfo` to verify Vulkan installation
3. Update GPU drivers to latest version
4. Open an issue on GitHub with logs

**Happy Accelerating! 🔥🚀**

