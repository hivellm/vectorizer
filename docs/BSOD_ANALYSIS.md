# BSOD Analysis Report - Windows Build/Test Crashes

## Problem Description

Multiple Blue Screen of Death (BSOD) events occurring during `cargo build` and `cargo test` operations on Windows 10.

## Root Causes Identified

### 1. GPU Acceleration Enabled by Default (CRITICAL)

**Issue:**
- `hive-gpu` feature is enabled in default features
- Attempts to initialize CUDA/DirectX/Vulkan drivers
- Can cause kernel-mode driver crashes on Windows

**Evidence:**
```toml
[features]
default = ["hive-gpu", "fastembed"]
```

**Impact:** HIGH
- Direct interaction with GPU drivers in kernel mode
- Known to cause BSODs with outdated/buggy GPU drivers
- Particularly problematic during parallel compilation

### 2. ONNX Runtime Native Dependencies

**Issue:**
- `fastembed` depends on ONNX Runtime with native Windows libraries
- Uses DirectML backend on Windows which interfaces with GPU
- `features = ["half"]` enables FP16 operations that may not be supported

**Evidence:**
```toml
fastembed = { version = "5.2", optional = true }
ort = { version = "2.0.0-rc.10", optional = true, features = ["half"] }
```

**Impact:** MEDIUM-HIGH
- Native DLL loading can fail with driver incompatibilities
- DirectML crashes can propagate to kernel mode

### 3. Excessive Parallelism

**Issue:**
- Multiple thread pools running simultaneously
- Tokio with "full" features spawns many worker threads
- Rayon creates additional thread pool for parallel iterators
- Combined with GPU operations, causes resource exhaustion

**Evidence:**
```toml
tokio = { version = "1.47", features = ["full"] }
rayon = "1.10"
crossbeam = "0.8"
```

```yaml
cpu:
  max_threads: 8
batch:
  parallel_processing: true
```

**Impact:** MEDIUM
- Memory pressure during compilation
- CPU oversubscription
- Can trigger Windows memory management BSODs

### 4. Memory-Mapped File Operations

**Issue:**
- `memmap2` performs kernel-mode I/O operations
- If file access fails (antivirus, permissions), can cause kernel panic
- Large memory mappings can cause page fault storms

**Evidence:**
```toml
memmap2 = "0.9"
```

**Impact:** MEDIUM
- Kernel-mode driver interaction
- Can trigger SYSTEM_SERVICE_EXCEPTION BSODs

### 5. Intensive Disk I/O (Tantivy Indexing)

**Issue:**
- Tantivy creates disk-based indices during tests
- Combined with other I/O operations, can saturate disk subsystem
- On slower HDDs or under heavy load, can cause I/O timeouts → BSOD

**Evidence:**
```toml
tantivy = "0.25"
```

**Impact:** LOW-MEDIUM
- Heavy disk I/O during tests
- Can trigger DRIVER_IRQL_NOT_LESS_OR_EQUAL with buggy storage drivers

## Recommended Solutions

### Immediate Actions (CRITICAL)

#### 1. Disable GPU Features by Default

**Reason:** Most users don't need GPU acceleration, and it's the highest BSOD risk

**Change:**
```toml
# OLD (dangerous)
[features]
default = ["hive-gpu", "fastembed"]

# NEW (safe)
[features]
default = []  # Minimal safe defaults
```

#### 2. Create Safe Build Profile

**Reason:** Reduce parallelism and resource usage during builds

**Add to Cargo.toml:**
```toml
[profile.safe]
inherits = "dev"
codegen-units = 1  # Single threaded codegen
incremental = false
debug = "line-tables-only"
```

**Build command:**
```bash
cargo +nightly build --profile=safe --no-default-features
```

#### 3. Reduce Tokio Thread Count

**Reason:** Prevent thread explosion on Windows

**Add to config:**
```yaml
performance:
  cpu:
    max_threads: 2  # Safe for Windows
    enable_simd: false  # Disable SIMD on problematic hardware
```

### Medium-Term Actions

#### 4. Optional GPU Feature

**Make GPU completely optional:**
```toml
[features]
default = []
fastembed = ["dep:fastembed"]
gpu = ["dep:hive-gpu"]  # Require explicit opt-in
gpu-cuda = ["gpu", "hive-gpu/cuda"]
gpu-safe = ["fastembed"]  # Fastembed only, no GPU
full = ["gpu", "fastembed", "benchmarks"]
```

#### 5. Add Windows-Specific Configuration

**Create `config.windows.yml`:**
```yaml
performance:
  cpu:
    max_threads: 2
    enable_simd: false
  batch:
    parallel_processing: false
    max_size: 50

# Disable intensive operations on Windows
file_watcher:
  enabled: false  # Can cause I/O storms

monitoring:
  system_metrics:
    enabled: false  # Reduce background threads
```

#### 6. Safe Test Profile

**Add to Cargo.toml:**
```toml
[profile.test-safe]
inherits = "test"
opt-level = 0
codegen-units = 1
```

**Test command:**
```bash
cargo +nightly test --profile=test-safe --no-default-features -- --test-threads=1
```

### Long-Term Actions

#### 7. Windows-Specific Build Script

**Create `scripts/build-windows-safe.ps1`:**
```powershell
# Safe build for Windows (no GPU, minimal parallelism)
$env:RAYON_NUM_THREADS = "1"
$env:TOKIO_WORKER_THREADS = "2"
$env:CARGO_BUILD_JOBS = "2"

Write-Host "Building with safe Windows configuration..."
cargo +nightly build --release --no-default-features --features "cache,monitoring"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build successful!"
} else {
    Write-Host "❌ Build failed"
    exit 1
}
```

#### 8. Diagnostic Mode

**Add diagnostic feature:**
```toml
[features]
diagnostic = []  # Enable extra logging, disable risky features
```

**Use in code:**
```rust
#[cfg(feature = "diagnostic")]
{
    // Safe initialization
    info!("Running in diagnostic mode - GPU disabled");
}
```

#### 9. Driver Compatibility Check

**Add startup check:**
```rust
#[cfg(target_os = "windows")]
fn check_gpu_drivers() -> Result<()> {
    // Check if GPU drivers are up to date
    // Warn if problematic versions detected
    // Disable GPU features if unsafe
}
```

## Testing Plan

### Phase 1: Verify Minimal Build
```bash
# No features - should NEVER crash
cargo +nightly build --release --no-default-features
cargo +nightly test --no-default-features -- --test-threads=1
```

### Phase 2: Add Features Incrementally
```bash
# Add fastembed only
cargo +nightly build --release --no-default-features --features "fastembed"

# Add GPU only
cargo +nightly build --release --no-default-features --features "hive-gpu"

# Full features (risky)
cargo +nightly build --release --all-features
```

### Phase 3: Monitor System
- Use Event Viewer to check BSOD error codes
- Monitor Task Manager during builds
- Check Windows Reliability Monitor

## Validation Criteria

### Success Metrics
- ✅ No BSODs during 10 consecutive builds
- ✅ No BSODs during full test suite (3 runs)
- ✅ Build completes with `--no-default-features`
- ✅ Memory usage stays under 4GB during build
- ✅ CPU usage doesn't spike to 100% for extended periods

### Warning Signs
- ⚠️ Memory usage > 8GB
- ⚠️ Disk I/O > 100MB/s sustained
- ⚠️ CPU 100% for > 2 minutes
- ⚠️ GPU driver errors in Event Log

## Windows-Specific Workarounds

### Antivirus Exclusions
Add these to Windows Defender exclusions:
- `target/` directory
- `*.pdb` files
- `cargo.exe`
- `rustc.exe`

### Virtual Memory
Increase page file size:
1. System Properties → Advanced → Performance Settings
2. Virtual Memory → Change
3. Set custom size: Initial 16GB, Maximum 32GB

### Driver Updates
Required before using GPU features:
- NVIDIA: Latest Game Ready or Studio Driver
- AMD: Latest Adrenalin Driver
- Intel: Latest Graphics Driver

## Monitoring Commands

```powershell
# Check for BSOD history
Get-EventLog -LogName System -Source "BugCheck" -Newest 10

# Monitor memory during build
while ($true) {
    $mem = Get-Process | Measure-Object WorkingSet -Sum
    Write-Host "Total Memory: $(($mem.Sum/1GB).ToString('F2')) GB"
    Start-Sleep -Seconds 5
}

# Check GPU driver status
Get-WmiObject Win32_VideoController | Select-Object Name, DriverVersion, Status
```

## References

- Windows BSOD Codes: https://docs.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-code-reference
- ONNX Runtime Windows: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
- Cargo Build Configuration: https://doc.rust-lang.org/cargo/reference/profiles.html

## Next Steps

1. Implement minimal default features (no GPU)
2. Create safe build profile
3. Add Windows-specific configuration
4. Test with `--no-default-features`
5. Document safe build procedures
6. Add driver compatibility checks


