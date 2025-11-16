# Windows Build Guide - Prevent BSODs

## Overview

Building Rust projects with GPU features and heavy parallelism can cause **Blue Screen of Death (BSOD)** crashes on Windows due to:
- GPU driver instability
- ONNX Runtime DirectML crashes
- Thread/memory exhaustion
- Disk I/O saturation

This guide shows how to build vectorizer **safely** on Windows.

## ⚠️ Critical Rules for Windows

### 1. **NEVER Use Default Features**

```bash
# ❌ DANGEROUS - Will likely cause BSOD
cargo build --release

# ❌ DANGEROUS - Even worse
cargo build --release --all-features

# ✅ SAFE - No GPU, minimal features
cargo build --release --no-default-features

# ✅ SAFE - Fastembed only (no GPU drivers)
cargo build --release --no-default-features --features "fastembed"
```

**Why:** Default features include `hive-gpu` and `fastembed` which:
- Load GPU drivers in kernel mode
- Can crash on outdated/buggy drivers
- Use DirectML which is unstable on some systems

### 2. **Use Safe Build Script**

```powershell
# Recommended method (handles everything)
.\scripts\build-windows-safe.ps1

# Or for tests
.\scripts\build-windows-safe.ps1 test
```

The script automatically:
- Limits parallelism (`CARGO_BUILD_JOBS=2`)
- Reduces thread count (`RAYON_NUM_THREADS=1`)
- Uses safe build profile
- Disables GPU features

### 3. **Use Windows-Safe Configuration**

```bash
# Copy Windows config
cp config.windows.yml config.yml

# Verify settings
cat config.yml | grep "max_threads"
# Should show: max_threads: 2
```

## Prerequisites

### Required

- Windows 10/11 (latest updates)
- Rust 1.85+ nightly
- 8GB RAM minimum
- 10GB free disk space

```powershell
# Install Rust (if not already installed)
# Visit https://rustup.rs and run the installer

# Install nightly toolchain
rustup toolchain install nightly

# Set nightly as default (optional)
rustup default nightly

# Verify installation
rustc --version
cargo --version
```

### Optional (for GPU features)

**⚠️ WARNING:** Only install if you need GPU acceleration and understand the risks.

- Latest GPU drivers:
  - NVIDIA: GeForce Game Ready or Studio Driver
  - AMD: Adrenalin Software
  - Intel: Latest Graphics Driver

- CUDA Toolkit 12.6+ (NVIDIA only)
- DirectML (included in Windows 10 1903+)

## Safe Build Procedures

### Method 1: Automated Script (Recommended)

```powershell
# Navigate to vectorizer directory
cd F:\Node\hivellm\vectorizer

# Run safe build
.\scripts\build-windows-safe.ps1

# Available commands:
# build       - Safe build (no GPU)
# build-fast  - Build with fastembed (no GPU drivers)
# build-full  - Build with all features (RISKY!)
# test        - Run tests (single-threaded)
# test-fast   - Run tests with fastembed
# clean       - Clean build artifacts
```

### Method 2: Manual Build

```powershell
# Set safe environment variables
$env:RAYON_NUM_THREADS = "1"
$env:TOKIO_WORKER_THREADS = "2"
$env:CARGO_BUILD_JOBS = "2"

# Build with safe profile
cargo +nightly build --profile=safe --no-default-features

# Or with fastembed (still safe)
cargo +nightly build --profile=safe --no-default-features --features "fastembed"
```

### Method 3: WSL2 (Most Stable)

```bash
# Install WSL2 (if not already installed)
wsl --install Ubuntu-24.04

# In WSL terminal
cd /mnt/f/Node/hivellm/vectorizer

# Build normally (Linux is more stable)
cargo +nightly build --release
```

## Feature Selection Guide

### Safe Features (Won't Cause BSODs)

```toml
# No features (safest)
--no-default-features

# Fastembed only (ONNX on CPU)
--no-default-features --features "fastembed"

# Monitoring and caching
--no-default-features --features "monitoring,cache"
```

### Risky Features (May Cause BSODs)

```toml
# ⚠️ GPU features - Requires latest drivers
--features "hive-gpu"
--features "hive-gpu-cuda"  # NVIDIA only
--features "hive-gpu-wgpu"  # DirectX 12

# ⚠️ All features - High BSOD risk
--all-features
```

### Feature Descriptions

| Feature | Risk | Description |
|---------|------|-------------|
| (none) | ✅ SAFE | CPU-only, no GPU drivers |
| `fastembed` | ⚠️ LOW | ONNX on CPU, no GPU |
| `hive-gpu` | 🔴 HIGH | GPU drivers, can crash kernel |
| `hive-gpu-cuda` | 🔴 CRITICAL | CUDA drivers, frequent crashes |
| `hive-gpu-wgpu` | 🔴 HIGH | DirectX 12, unstable on some systems |
| `benchmarks` | ⚠️ MEDIUM | Heavy resource usage |
| `full` | 🔴 CRITICAL | All features, highest risk |

## Testing Safely

### Unit Tests (Safe)

```powershell
# Single-threaded tests
cargo +nightly test --profile=test-safe --no-default-features -- --test-threads=1

# With fastembed
cargo +nightly test --profile=test-safe --no-default-features --features "fastembed" -- --test-threads=1
```

### Integration Tests (Caution)

```powershell
# Run one test at a time
cargo +nightly test --profile=test-safe --no-default-features --test test_name -- --test-threads=1

# Skip expensive tests
cargo +nightly test --profile=test-safe --no-default-features -- --test-threads=1 --skip benchmark
```

### Benchmark Tests (High Risk)

**⚠️ WARNING:** Benchmarks can cause BSODs due to high CPU/memory usage.

```powershell
# Only run on stable system with latest drivers
cargo +nightly bench --no-default-features --features "benchmarks" -- --test-threads=1
```

## Troubleshooting BSODs

### If You Get a BSOD

1. **Check Error Code** in Event Viewer:
   ```powershell
   Get-EventLog -LogName System -Source "BugCheck" -Newest 5
   ```

2. **Common BSOD Codes:**
   - `DRIVER_IRQL_NOT_LESS_OR_EQUAL` → GPU driver issue
   - `SYSTEM_SERVICE_EXCEPTION` → Kernel-mode crash (GPU/I/O)
   - `PAGE_FAULT_IN_NONPAGED_AREA` → Memory/driver issue
   - `KERNEL_SECURITY_CHECK_FAILURE` → Memory corruption

3. **Immediate Actions:**
   - Reboot in Safe Mode
   - Update GPU drivers
   - Update Windows
   - Run Windows Memory Diagnostic
   - Check disk with `chkdsk /f /r`

4. **Build Again Safely:**
   ```powershell
   cd F:\Node\hivellm\vectorizer
   .\scripts\build-windows-safe.ps1 clean
   .\scripts\build-windows-safe.ps1 build
   ```

### Prevention Strategies

#### 1. **Driver Updates**

```powershell
# Check GPU driver version
Get-WmiObject Win32_VideoController | Select-Object Name, DriverVersion

# Update drivers:
# - NVIDIA: https://www.nvidia.com/Download/index.aspx
# - AMD: https://www.amd.com/en/support
# - Intel: https://www.intel.com/content/www/us/en/download-center/home.html
```

#### 2. **Antivirus Exclusions**

Add to Windows Defender exclusions:

```powershell
# Run as Administrator
Add-MpPreference -ExclusionPath "F:\Node\hivellm\vectorizer\target"
Add-MpPreference -ExclusionProcess "cargo.exe"
Add-MpPreference -ExclusionProcess "rustc.exe"
Add-MpPreference -ExclusionExtension ".pdb"
```

#### 3. **Virtual Memory**

Increase page file:

1. System Properties → Advanced → Performance Settings
2. Virtual Memory → Change
3. Custom size:
   - Initial: 16384 MB (16 GB)
   - Maximum: 32768 MB (32 GB)
4. Set → OK → Reboot

#### 4. **Monitoring During Build**

```powershell
# Monitor memory usage
while ($true) {
    $mem = Get-Process | Measure-Object WorkingSet -Sum
    $memGB = [math]::Round($mem.Sum/1GB, 2)
    Write-Host "Memory: $memGB GB" -ForegroundColor $(if ($memGB -gt 8) { "Red" } else { "Green" })
    Start-Sleep -Seconds 5
}
```

#### 5. **Stability Test Before Full Build**

```powershell
# Test minimal build first
cargo +nightly build --profile=safe --no-default-features --lib

# If successful, try full build
cargo +nightly build --profile=safe --no-default-features
```

## Performance Comparison

### Build Times (i7-8700K, 16GB RAM, Windows 11)

| Configuration | Time | BSOD Risk | Notes |
|--------------|------|-----------|-------|
| Safe (no GPU) | 8min | ✅ None | Recommended |
| Fastembed only | 12min | ⚠️ Low | ONNX on CPU |
| GPU + Fastembed | 15min | 🔴 High | May crash |
| All features | 20min | 🔴 Critical | Frequent crashes |

### Resource Usage

| Configuration | CPU | RAM | Disk I/O |
|--------------|-----|-----|----------|
| Safe | 30% | 4GB | 50 MB/s |
| Fastembed | 40% | 6GB | 80 MB/s |
| GPU + Fastembed | 70% | 10GB | 150 MB/s |
| All features | 100% | 14GB | 200 MB/s |

## Production Deployment on Windows

### Windows Server Recommendations

```yaml
# Use Windows Server 2019+ with:
# - Latest updates installed
# - Latest GPU drivers (if using GPU)
# - Antivirus configured with exclusions
# - Sufficient virtual memory (32GB+)
# - SSD for target/ directory
```

### Docker Alternative (Safer)

```dockerfile
# Use Linux container (more stable)
FROM rust:1.85-nightly

WORKDIR /app
COPY . .

RUN cargo build --release --no-default-features
```

### WSL2 Alternative (Best for Development)

```bash
# Install WSL2
wsl --install Ubuntu-24.04

# Build in Linux environment (more stable)
cd /mnt/f/Node/hivellm/vectorizer
cargo build --release
```

## FAQ

### Q: Why do I get BSODs only during build/test?

**A:** Rust compilation is extremely resource-intensive:
- High CPU usage triggers thermal throttling
- Memory allocations stress RAM/page file
- Parallel compilation creates many threads
- GPU features load kernel-mode drivers

Combined, these can expose hardware/driver issues that don't appear during normal use.

### Q: Can I safely use GPU features on Windows?

**A:** Only if:
- You have latest GPU drivers (released within 3 months)
- You've updated Windows to latest version
- You've tested on non-critical system first
- You've created system restore point
- You're prepared for potential BSODs

Even then, use `--features "hive-gpu"` NOT `--all-features`.

### Q: Is WSL2 really more stable?

**A:** Yes, significantly:
- Linux kernel is more stable for heavy compilation
- Better thread management
- More efficient I/O subsystem
- No DirectX/DirectML instability
- Native Rust toolchain support

### Q: What if I still get BSODs with safe settings?

**A:** This indicates hardware or Windows issues:
1. Run Windows Memory Diagnostic
2. Check disk health (CrystalDiskInfo)
3. Monitor temperatures (HWMonitor)
4. Update BIOS/chipset drivers
5. Test RAM with MemTest86
6. Consider WSL2 as alternative

### Q: Can I use release profile instead of safe profile?

**A:** Not recommended for Windows:
- Release profile uses LTO (link-time optimization)
- LTO is extremely memory/CPU intensive
- Can trigger BSODs even without GPU features
- Use `--profile=safe` or `--profile=dev` instead

## Resources

- [BSOD Analysis Report](./BSOD_ANALYSIS.md)
- [Windows Event Viewer](ms-settings:windowsupdate)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [ONNX Runtime DirectML](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html)

## Support

If you continue to experience BSODs after following this guide:

1. Create issue on GitHub with:
   - Windows version (`winver`)
   - GPU model and driver version
   - BSOD error code from Event Viewer
   - Build command used
   - System specs (CPU, RAM, disk type)

2. Try WSL2 as alternative:
   ```bash
   wsl --install Ubuntu-24.04
   # Build in Linux environment
   ```

3. Consider using Linux VM:
   - VirtualBox or VMware
   - Allocate 8GB+ RAM
   - Use Linux for Rust development


