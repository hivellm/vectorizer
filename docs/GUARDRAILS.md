# Vectorizer Guardrails System

## Overview

The Vectorizer guardrails system provides **multiple layers of protection** to prevent system crashes, BSODs, and resource exhaustion. This is especially critical on Windows where GPU drivers and heavy parallelism can cause kernel-mode crashes.

## Protection Layers

### 1. **Compile-Time Guardrails** (build.rs)

Checks during compilation to prevent dangerous configurations:

```rust
// Automatically detects:
- Windows platform
- GPU features enabled
- Release profile with GPU (high risk)
- Excessive parallelism
- All-features flag (very dangerous)
```

**Warnings generated:**
- 🪟 Windows platform detected
- ❌ GPU features on Windows (BSOD risk)
- ⚠️  High parallelism detected
- 💡 Recommended safe build commands

**Example output:**
```
warning: ================================================
warning: ❌ CRITICAL WARNING ❌
warning: GPU features enabled on Windows!
warning: This can cause Blue Screen of Death (BSOD)!
warning: ================================================
```

### 2. **Runtime Guardrails** (guardrails.rs)

Active monitoring and protection during execution:

#### Resource Monitoring
- **Memory usage**: Limits to 75% (60% on Windows)
- **CPU usage**: Limits to 90% (70% on Windows)
- **Concurrent operations**: Max 4 (2 on Windows)
- **Free memory**: Minimum 512MB (1GB on Windows)

#### Auto-Throttling
```rust
// Automatically slows down when:
- Memory usage > 75%
- CPU usage > 90%
- Too many concurrent operations
- System becomes unstable
```

#### Usage in Code

```rust
use crate::guardrails::{Guardrails, GuardrailsConfig};

// Initialize with default config
let guardrails = Guardrails::new(GuardrailsConfig::default());

// Check if operation is safe
match guardrails.check_safe() {
    Ok(_) => {
        // Safe to proceed
    }
    Err(violation) => {
        // System under stress - wait or abort
        warn!("System violation: {}", violation);
    }
}

// Acquire permit for expensive operation
let permit = guardrails.acquire_permit()?;
// ... do expensive work ...
// Permit automatically released on drop
```

#### Windows-Specific Protection

```rust
#[cfg(target_os = "windows")]
{
    // Stricter limits on Windows:
    // - 60% memory instead of 75%
    // - 70% CPU instead of 90%
    // - 2 concurrent ops instead of 4
    // - 1GB free memory instead of 512MB
    
    // GPU driver checks
    windows::check_gpu_drivers()?;
    
    // Apply Windows limits
    let config = windows::apply_windows_limits();
}
```

### 3. **Pre-Build Safety Checks** (pre-build-check.ps1)

Automated system verification before building:

```powershell
.\scripts\pre-build-check.ps1
```

**Checks performed:**
1. ✅ Windows version (10+)
2. ✅ Available memory (8GB+)
3. ✅ Disk space (10GB+)
4. ✅ Rust toolchain installed
5. ✅ GPU drivers (age, version)
6. ✅ Conflicting processes
7. ✅ Antivirus exclusions
8. ✅ Virtual memory (pagefile)
9. ✅ Recent BSODs
10. ✅ System stability

**Output example:**
```
[1/10] Checking Windows version...
      ✅ Windows 10.0 Build 19045

[2/10] Checking system memory...
      Total: 16.00 GB | Free: 8.25 GB

...

✅ All checks passed! System is ready for safe build.
```

### 4. **Safe Build Profiles** (Cargo.toml)

Custom build profiles optimized for safety:

```toml
[profile.safe]
inherits = "dev"
codegen-units = 1  # Single-threaded codegen
incremental = false
opt-level = 0
debug = "line-tables-only"

[profile.test-safe]
inherits = "test"
codegen-units = 1  # Prevents resource explosion
opt-level = 0
```

**Usage:**
```bash
# Safe build (won't cause BSOD)
cargo build --profile=safe --no-default-features

# Safe tests
cargo test --profile=test-safe --no-default-features -- --test-threads=1
```

### 5. **Cargo Configuration** (.cargo/config.toml)

Platform-specific defaults:

```toml
# Windows - Conservative settings
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "codegen-units=4"]

# Environment variables
[env]
RAYON_NUM_THREADS = { value = "2" }
TOKIO_WORKER_THREADS = { value = "2" }
```

### 6. **Automated Safe Build Script** (build-windows-safe.ps1)

One-command safe build:

```powershell
# Safe build (no GPU)
.\scripts\build-windows-safe.ps1

# With fastembed (no GPU drivers)
.\scripts\build-windows-safe.ps1 build-fast

# Run tests
.\scripts\build-windows-safe.ps1 test
```

**What it does:**
- Sets safe environment variables
- Checks Rust toolchain
- Warns about risky configurations
- Builds with safe profile
- Reports success/failure clearly

## Usage Guide

### For Normal Development (Safe)

```bash
# 1. Run pre-build check
.\scripts\pre-build-check.ps1

# 2. Build safely
.\scripts\build-windows-safe.ps1

# 3. Run tests
.\scripts\build-windows-safe.ps1 test
```

### For Production Build (Still Safe)

```bash
# Use safe profile instead of release
cargo +nightly build --profile=safe --no-default-features
```

### If You Need GPU Features (Risky)

```bash
# 1. Update GPU drivers first!
# 2. Create system restore point
# 3. Build with explicit features (NOT --all-features)
cargo +nightly build --profile=safe --no-default-features --features "hive-gpu"
```

## Violation Types

### MemoryExhaustion
- **Cause**: Memory usage > 75% (60% on Windows)
- **Action**: Reduce batch sizes, close other applications
- **Prevention**: Increase RAM or virtual memory

### CpuOverload
- **Cause**: CPU usage > 90% (70% on Windows)
- **Action**: Wait for CPU to cool down
- **Prevention**: Reduce parallelism, limit build jobs

### DiskIoSaturation
- **Cause**: Heavy disk I/O during indexing/compilation
- **Action**: Pause other disk operations
- **Prevention**: Use SSD, disable file watcher

### TooManyConcurrentOps
- **Cause**: More than max concurrent operations
- **Action**: Wait for current operations to finish
- **Prevention**: Use operation permits properly

### GpuDriverIssue
- **Cause**: GPU features enabled with old/buggy drivers
- **Action**: Update drivers or disable GPU
- **Prevention**: Use --no-default-features

## Monitoring System Health

### Get Current Status

```rust
let status = guardrails.get_status();
println!("{}", status);
// Memory: 45.2% (8192 MB free) | CPU: 35.6% | Ops: 2 | Throttled: false | Safe: true
```

### Check Violations

```rust
use std::time::Duration;

// Get violations in last 5 minutes
let violations = guardrails.get_violations(Duration::from_secs(300));

for (time, violation_type) in violations {
    warn!("Violation at {:?}: {}", time, violation_type);
}
```

### Wait for Stability

```rust
// Wait up to 30 seconds for system to stabilize
guardrails.wait_for_stability(Duration::from_secs(30)).await?;
```

## Configuration

### Custom Guardrails Config

```rust
use crate::guardrails::GuardrailsConfig;

let config = GuardrailsConfig {
    enabled: true,
    max_memory_percent: 80.0,  // Custom limit
    max_cpu_percent: 85.0,
    min_free_memory_mb: 1024,
    max_concurrent_ops: 3,
    auto_throttle: true,
    windows_protection: true,
};

let guardrails = Guardrails::new(config);
```

### Disable Guardrails (Not Recommended)

```rust
let config = GuardrailsConfig {
    enabled: false,  // ⚠️ Dangerous!
    ..Default::default()
};
```

## Best Practices

### DO ✅

1. **Always run pre-build check on Windows**
   ```powershell
   .\scripts\pre-build-check.ps1
   ```

2. **Use safe build script**
   ```powershell
   .\scripts\build-windows-safe.ps1
   ```

3. **Check guardrails before expensive operations**
   ```rust
   guardrails.check_safe()?;
   ```

4. **Use operation permits**
   ```rust
   let _permit = guardrails.acquire_permit()?;
   ```

5. **Monitor violations**
   ```rust
   let violations = guardrails.get_violations(Duration::from_secs(300));
   ```

### DON'T ❌

1. **Don't disable guardrails in production**
   ```rust
   // ❌ BAD
   let config = GuardrailsConfig { enabled: false, .. };
   ```

2. **Don't use --all-features on Windows**
   ```bash
   # ❌ VERY BAD - Will likely cause BSOD
   cargo build --all-features
   ```

3. **Don't ignore violation errors**
   ```rust
   // ❌ BAD
   let _ = guardrails.check_safe();  // Ignoring error
   
   // ✅ GOOD
   if let Err(violation) = guardrails.check_safe() {
       warn!("System violation: {}", violation);
       // Take action or abort
   }
   ```

4. **Don't run many concurrent operations**
   ```rust
   // ❌ BAD - Can cause resource exhaustion
   for i in 0..100 {
       tokio::spawn(expensive_operation());
   }
   
   // ✅ GOOD - Use permits
   for i in 0..100 {
       let permit = guardrails.acquire_permit()?;
       tokio::spawn(async move {
           let _permit = permit;  // Hold permit
           expensive_operation().await
       });
   }
   ```

## Troubleshooting

### Guardrails Blocking Operations

**Symptoms:**
- Operations fail with `MemoryExhaustion` or `CpuOverload`
- System appears throttled

**Solutions:**
1. Wait for system to stabilize
   ```rust
   guardrails.wait_for_stability(Duration::from_secs(60)).await?;
   ```

2. Close other applications

3. Reduce batch sizes in config:
   ```yaml
   performance:
     batch:
       default_size: 50  # Reduced from 100
   ```

4. Check system status:
   ```rust
   let status = guardrails.get_status();
   info!("System status: {}", status);
   ```

### Build Fails with Warnings

**Symptoms:**
- Many warnings about GPU/Windows/parallelism

**Solutions:**
1. Use safe build script:
   ```powershell
   .\scripts\build-windows-safe.ps1
   ```

2. Or build with safe options:
   ```bash
   cargo build --profile=safe --no-default-features
   ```

### Still Getting BSODs

If guardrails are enabled and you still get BSODs:

1. **Disable ALL optional features:**
   ```bash
   cargo build --profile=safe --no-default-features
   ```

2. **Update drivers:**
   - GPU drivers
   - Chipset drivers
   - BIOS/UEFI

3. **Check hardware:**
   - Run Windows Memory Diagnostic
   - Check disk health (CrystalDiskInfo)
   - Monitor temperatures (HWMonitor)

4. **Use WSL2 instead:**
   ```bash
   wsl --install Ubuntu-24.04
   # Build in Linux environment (more stable)
   ```

## Performance Impact

Guardrails have minimal performance impact:

| Operation | Overhead | When |
|-----------|----------|------|
| `check_safe()` | ~1ms | Every ~1 second (rate limited) |
| `acquire_permit()` | ~0.1ms | Per operation |
| Resource monitoring | 0.1% CPU | Background thread |

The safety benefits far outweigh the minimal overhead.

## Future Enhancements

Planned improvements:

1. **GPU memory monitoring** - Track VRAM usage
2. **Predictive throttling** - Slow down before violations
3. **Automatic recovery** - Restart operations after stabilization
4. **Metrics export** - Prometheus metrics for violations
5. **Smart batching** - Automatically adjust batch sizes

## See Also

- [BSOD Analysis](./BSOD_ANALYSIS.md) - Root cause analysis
- [Windows Build Guide](./WINDOWS_BUILD_GUIDE.md) - Safe building on Windows
- [Performance Guide](./PERFORMANCE.md) - Optimization strategies

## Support

If guardrails aren't preventing issues:

1. Check configuration: `config.windows.yml`
2. Review violations: `guardrails.get_violations()`
3. Check Event Viewer (Windows)
4. Create GitHub issue with:
   - System specs
   - Guardrails config
   - Violation history
   - BSOD error code (if applicable)


