# 🪟 DirectX 12 GPU Setup Guide

## Overview

DirectX 12 é a API gráfica de baixo nível da Microsoft, otimizada para GPUs NVIDIA, AMD e Intel em sistemas Windows. O Vectorizer utiliza DirectX 12 via `wgpu` para oferecer aceleração GPU nativa no Windows 10/11 com desempenho superior ao Vulkan em muitos cenários.

---

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| **Windows 10** | ✅ Excellent | Requires Fall Creators Update (1709+) |
| **Windows 11** | ✅ Excellent | Native DX12 Ultimate support |
| **Xbox Series X/S** | ⚠️ Experimental | Requires special SDK |
| **Linux** | ❌ Not Supported | Use Vulkan instead |
| **macOS** | ❌ Not Supported | Use Metal instead |

---

## Prerequisites

### 1. Windows Version Requirements

**Minimum Requirements**:
- Windows 10 Fall Creators Update (Version 1709, Build 16299) or later
- Windows 11 (any version)

**Check Windows Version**:
```powershell
# Open PowerShell
winver

# Or
Get-ComputerInfo | Select-Object WindowsVersion, WindowsBuildLabEx
```

**Update Windows**:
```powershell
# Settings > Update & Security > Windows Update
# Or via PowerShell (as Administrator)
Install-Module PSWindowsUpdate
Get-WindowsUpdate
Install-WindowsUpdate -AcceptAll -AutoReboot
```

---

### 2. GPU Driver Installation

#### NVIDIA GPUs (Recommended for DX12)

**Minimum**: GeForce GTX 900 series (Maxwell) or newer
**Optimal**: RTX 20 series (Turing) or newer

**Installation**:
1. Download GeForce Experience: https://www.nvidia.com/en-us/geforce/geforce-experience/
2. Install latest Game Ready or Studio Driver
3. Or download directly: https://www.nvidia.com/Download/index.aspx

**Verification**:
```powershell
# Check driver version
nvidia-smi

# Expected output:
# Driver Version: 535.xx or later
# CUDA Version: 12.x
```

#### AMD GPUs

**Minimum**: Radeon GCN 3.0 (Radeon R9 Fury) or newer
**Optimal**: RDNA 2 (RX 6000 series) or newer

**Installation**:
1. Download AMD Software Adrenalin Edition: https://www.amd.com/en/support
2. Run installer and select "Full Install"
3. Restart system

**Verification**:
```powershell
# Check driver version via Device Manager
# Or via AMD Software > System tab
```

#### Intel GPUs

**Minimum**: Intel HD Graphics 500 series (Skylake) or newer
**Optimal**: Intel Iris Xe (11th gen or newer)

**Installation**:
1. Download Intel Driver & Support Assistant: https://www.intel.com/content/www/us/en/support/detect.html
2. Run installer and update graphics driver
3. Or manual download: https://www.intel.com/content/www/us/en/download-center/home.html

**Verification**:
```powershell
# Check via Device Manager > Display adapters
# Or via Intel Graphics Command Center
```

---

### 3. DirectX 12 SDK (Optional for Development)

**For End Users**: Not required (included in Windows)
**For Developers**: Optional for debugging/profiling

**Installation**:
```powershell
# Download Windows SDK
# https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

# Or via Visual Studio Installer
# Components > Windows 10 SDK (latest version)
```

**Verify DirectX Version**:
```powershell
# Run dxdiag (DirectX Diagnostic Tool)
dxdiag

# Check "System" tab > DirectX Version: DirectX 12
```

---

## Building Vectorizer with DirectX 12 Support

### Prerequisites
```powershell
# Install Rust (if not already installed)
# Download from: https://rustup.rs/

# Or via PowerShell
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe

# Update Rust
rustup update stable
```

### Build with DirectX 12 (via wgpu-gpu feature)
```powershell
cd C:\path\to\vectorizer

# Build in release mode with DirectX 12 support
cargo build --features wgpu-gpu --release

# This enables:
# - DirectX 12 (Windows)
# - Vulkan (fallback on Windows)
# - CPU fallback (all platforms)
```

### Build with explicit DirectX 12 backend
```powershell
# Force DirectX 12 backend
$env:WGPU_BACKEND="dx12"
cargo build --features wgpu-gpu --release
```

---

## Running Vectorizer with DirectX 12

### 1. Auto-detection (Recommended)
```powershell
# Let Vectorizer auto-select the best backend
# Priority on Windows: DirectX12 > Vulkan > CUDA > CPU
.\scripts\start.bat --workspace vectorize-workspace.yml

# Or
.\target\release\vzr.exe start --workspace vectorize-workspace.yml
```

### 2. Explicit DirectX 12 Selection
```powershell
# Force DirectX 12 backend via environment variable
$env:WGPU_BACKEND="dx12"
.\scripts\start.bat --workspace vectorize-workspace.yml

# Or via CLI flag
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend dx12
```

### 3. Verification
```powershell
# Check logs for GPU detection
Get-Content -Wait vectorizer-workspace.log | Select-String "GPU"

# Expected output:
# 🔍 VectorStore::new_auto_universal() - Universal Multi-GPU Detection
# 🪟 DirectX 12 detected...
# ✅ DirectX 12 GPU initialized!
```

---

## Troubleshooting

### 1. "No GPU backends detected"
**Cause**: DirectX 12 not available or drivers outdated

**Solution**:
```powershell
# Check DirectX version
dxdiag
# Should show "DirectX Version: DirectX 12"

# If not, update Windows
Install-Module PSWindowsUpdate
Install-WindowsUpdate -AcceptAll

# Update GPU drivers (see section 2 above)
```

### 2. "DirectX 12 initialization failed"
**Cause**: GPU doesn't support DX12 feature level 12_0

**Solution**:
```powershell
# Check GPU feature level
dxdiag > Save All Information > check Feature Levels

# Minimum required: 12_0 (DirectX 12)
# If GPU is too old (<2015), use CPU fallback or upgrade GPU
```

### 3. "D3D12CreateDevice failed with E_INVALIDARG"
**Cause**: Driver corruption or incompatibility

**Solution**:
```powershell
# Clean reinstall GPU drivers

# NVIDIA:
# 1. Download DDU (Display Driver Uninstaller)
# 2. Boot to Safe Mode
# 3. Run DDU and select "Clean and Restart"
# 4. Install latest NVIDIA driver

# AMD:
# 1. Run AMD Cleanup Utility
# 2. Restart
# 3. Install latest AMD Adrenalin driver

# Intel:
# 1. Uninstall via Device Manager
# 2. Restart
# 3. Install latest Intel Graphics Driver
```

### 4. "DXGI_ERROR_DEVICE_REMOVED"
**Cause**: GPU crash or instability (overclocking, overheating, bad VRAM)

**Diagnosis**:
```powershell
# Check Event Viewer for TDR (Timeout Detection and Recovery) events
# Event Viewer > Windows Logs > System
# Look for "Display driver stopped responding and has recovered"
```

**Solution**:
1. **Reduce GPU Overclock**: Reset to default clocks
2. **Check Temperatures**: Use MSI Afterburner/HWiNFO
   - GPU should be <85°C under load
3. **Test VRAM**: Run MemTestG80 or similar
4. **Increase TDR Timeout**:
```powershell
# Increase GPU timeout (Windows Registry)
# WARNING: Modify registry at your own risk

# Open Registry Editor (regedit)
# Navigate to: HKEY_LOCAL_MACHINE\System\CurrentControlSet\Control\GraphicsDrivers
# Create DWORD (32-bit) Value: TdrDelay
# Set value: 10 (seconds)
# Restart computer
```

### 5. Performance Issues
**Symptoms**: GPU usage is low (<20%), CPU usage is high

**Diagnosis**:
```powershell
# Open Task Manager (Ctrl+Shift+Esc)
# Performance tab > GPU
# Check "Copy" or "Compute_0" utilization

# Or use GPU-Z
# Download from: https://www.techpowerup.com/gpuz/
```

**Solution**:
```yaml
# Adjust gpu_threshold_operations in config.yml
gpu:
  enabled: true
  backend: dx12
  device_id: 0
  power_preference: high_performance
  gpu_threshold_operations: 500  # Lower = more GPU usage
```

### 6. "Feature Level 12_0 not supported"
**Cause**: GPU is too old (pre-2015)

**Solution**:
- **Upgrade GPU**: DirectX 12 requires modern hardware
- **Use Vulkan**: May work on older GPUs
- **Use CPU**: Fallback option
```powershell
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend cpu
```

---

## Performance Tips

### 1. Enable Hardware-Accelerated GPU Scheduling (Windows 10 20H1+)
```powershell
# Settings > System > Display > Graphics settings
# Enable "Hardware-accelerated GPU scheduling"
# Restart computer

# Or via Registry
Set-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers" -Name "HwSchMode" -Value 2
Restart-Computer
```

### 2. Optimize for Large Batches
```rust
// DirectX 12 excels with large batch operations
let queries: Vec<Vec<f32>> = (0..10000).map(|_| random_vector(512)).collect();
let results = store.batch_search("collection", queries, 10)?;
// GPU will be fully utilized
```

### 3. Monitor GPU Performance
```powershell
# Task Manager (Ctrl+Shift+Esc) > Performance > GPU

# Or use GPU-Z for detailed monitoring
# Download: https://www.techpowerup.com/gpuz/

# Or use NVIDIA/AMD overlay
# NVIDIA: Alt+Z (GeForce Experience overlay)
# AMD: Alt+R (Radeon Software overlay)
```

### 4. Adjust Power Settings
```powershell
# Set Windows power plan to "High Performance"
powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c

# Or via Settings > System > Power & sleep > Additional power settings
# Select "High performance"
```

### 5. Configure GPU Memory Allocation
```yaml
# config.yml
gpu:
  enabled: true
  backend: dx12
  device_id: 0
  memory_limit_mb: 4096  # Adjust based on GPU VRAM
  power_preference: high_performance
```

---

## Platform-Specific Notes

### Windows 10
- **Minimum Version**: 1709 (Fall Creators Update)
- **Recommended**: 20H2 or later for best performance
- **Feature Level**: 12_0 minimum, 12_1+ recommended

### Windows 11
- **Best Performance**: Native DX12 Ultimate support
- **Auto HDR**: May interfere with compute workloads (disable if needed)
- **DirectStorage**: Not used by Vectorizer (future feature)

### Windows on ARM
- **Surface Pro X**: Limited DX12 support via emulation
- **Snapdragon GPUs**: May work but not optimized

---

## Benchmarks

### DirectX 12 vs Other Backends (Windows)

| Backend | GPU | Cosine Similarity (1M ops) | Euclidean Distance (1M ops) |
|---------|-----|----------------------------|------------------------------|
| **DirectX12** | NVIDIA RTX 3070 | 165 ms | 142 ms |
| **Vulkan** | NVIDIA RTX 3070 | 189 ms | 156 ms |
| **CUDA** | NVIDIA RTX 3070 | 178 ms | 148 ms |
| **DirectX12** | AMD RX 6700 XT | 198 ms | 172 ms |
| **Vulkan** | AMD RX 6700 XT | 205 ms | 180 ms |
| **DirectX12** | Intel Iris Xe | 425 ms | 389 ms |
| **CPU** | AMD Ryzen 9 5900X | 2,980 ms | 2,654 ms |

*Results from `benchmark/reports/multi_gpu_comparison.json`*

**Key Insights**:
- ✅ **DirectX 12 is 5-12% faster than Vulkan on Windows**
- ✅ **NVIDIA GPUs benefit most from DX12**
- ✅ **AMD GPUs perform well with both DX12 and Vulkan**
- ✅ **Intel GPUs have basic support but slower than dedicated GPUs**

---

## FAQ

### Q: Is DirectX 12 faster than Vulkan on Windows?
**A**: Yes, typically 5-12% faster on NVIDIA GPUs, especially for compute workloads. AMD GPUs show smaller differences (2-5%).

### Q: Can I use DirectX 12 on Windows 7/8?
**A**: No. DirectX 12 requires Windows 10 version 1709 or later, or Windows 11.

### Q: Does DirectX 12 work with AMD GPUs?
**A**: Yes! AMD has excellent DirectX 12 support starting with GCN 3.0 architecture (2015+).

### Q: What's the difference between DirectX 11 and DirectX 12?
**A**: DX12 is a low-level API offering better multi-threading and GPU efficiency. Vectorizer uses DX12 exclusively (no DX11 fallback).

### Q: Can I force Vulkan over DirectX 12 on Windows?
**A**:
```powershell
# Via environment variable
$env:WGPU_BACKEND="vulkan"
.\target\release\vzr.exe start --workspace vectorize-workspace.yml

# Or via CLI flag
.\target\release\vzr.exe start --workspace vectorize-workspace.yml --gpu-backend vulkan
```

### Q: What's DirectX 12 Ultimate?
**A**: DX12 Ultimate is a superset of DX12 with ray tracing, mesh shaders, and variable rate shading. Not required for Vectorizer, but provides best performance on RTX 30/40 series and RX 6000/7000 series.

### Q: Does DirectX 12 work on laptops with integrated GPUs?
**A**: Yes! Intel Iris Xe and AMD Radeon integrated GPUs support DX12. Performance is lower than dedicated GPUs but much better than CPU.

---

## Additional Resources

- **DirectX Official Site**: https://microsoft.com/directx
- **DirectX Developer Blog**: https://devblogs.microsoft.com/directx/
- **wgpu Documentation**: https://wgpu.rs/
- **NVIDIA DirectX 12 Guide**: https://developer.nvidia.com/directx
- **AMD DirectX 12 Optimization**: https://gpuopen.com/learn/understanding-dx12-new-features/

---

## Windows-Specific Commands

### Check DirectX Capabilities
```powershell
# Run DirectX Diagnostic Tool
dxdiag

# Export full report
dxdiag /t C:\dxdiag_report.txt

# Check Feature Levels via PowerShell
$adapter = Get-WmiObject -Class Win32_VideoController
$adapter.Name
$adapter.DriverVersion
$adapter.VideoModeDescription
```

### Monitor GPU via PowerShell
```powershell
# Real-time GPU monitoring
while ($true) {
    $gpu = Get-Counter '\GPU Engine(*)\Utilization Percentage'
    $gpu.CounterSamples | Format-Table -AutoSize
    Start-Sleep -Seconds 1
    Clear-Host
}
```

### Set High Performance Mode for Vectorizer
```powershell
# Windows Graphics Settings
# Settings > System > Display > Graphics settings
# Add vzr.exe
# Set to "High performance"

# Or via PowerShell (requires admin)
$app = "C:\path\to\vectorizer\target\release\vzr.exe"
Add-AppxPackageGraphicsPerformance -AppId $app -PreferenceType HighPerformance
```

---

## Support

If you encounter issues:
1. Check `vectorizer-workspace.log` for error messages
2. Run `dxdiag` and verify DirectX 12 support
3. Update GPU drivers to latest version
4. Check Windows version (1709+ required)
5. Try Vulkan fallback: `--gpu-backend vulkan`
6. Open an issue on GitHub with logs and `dxdiag` output

**Happy Accelerating! 🪟🚀**

