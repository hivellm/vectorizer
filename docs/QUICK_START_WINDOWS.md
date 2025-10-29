# Quick Start for Windows Users

## ⚠️ Important Warning

Building Vectorizer on Windows can cause **Blue Screen of Death (BSOD)** if not done correctly. This guide ensures safe building.

## One-Command Safe Build

```powershell
# Navigate to vectorizer directory
cd F:\Node\hivellm\vectorizer

# Run safe build script
.\scripts\build-windows-safe.ps1
```

That's it! The script handles everything automatically.

## What the Script Does

1. Sets safe environment variables
2. Checks Rust toolchain
3. Builds with safe profile
4. No GPU features enabled
5. Limited parallelism to prevent crashes

## Available Commands

```powershell
# Safe build (no GPU)
.\scripts\build-windows-safe.ps1

# Build with fastembed (ONNX on CPU - still safe)
.\scripts\build-windows-safe.ps1 build-fast

# Run tests
.\scripts\build-windows-safe.ps1 test

# Clean build artifacts
.\scripts\build-windows-safe.ps1 clean
```

## Pre-Build Safety Check (Optional)

```powershell
# Check if your system is ready
.\scripts\pre-build-check.ps1

# Verbose mode
.\scripts\pre-build-check.ps1 -Verbose
```

## Manual Build (Advanced)

If you want to build manually:

```powershell
# Set safe environment variables
$env:RAYON_NUM_THREADS = "1"
$env:TOKIO_WORKER_THREADS = "2"
$env:CARGO_BUILD_JOBS = "2"

# Build with safe profile and no GPU
cargo +nightly build --profile=safe --no-default-features

# Test safely
cargo +nightly test --profile=test-safe --no-default-features -- --test-threads=1
```

## What NOT to Do (Will Cause BSOD!)

```powershell
# ❌ NEVER do this on Windows
cargo build --release  # Uses GPU by default (OLD behavior)
cargo build --all-features  # Enables everything including GPU
cargo build --release --features "hive-gpu"  # GPU drivers = BSOD risk
```

## Troubleshooting

### Still Getting BSODs?

1. **Update GPU drivers** to latest version
2. **Update Windows** to latest version
3. **Use WSL2** instead:
   ```bash
   wsl --install Ubuntu-24.04
   cd /mnt/f/Node/hivellm/vectorizer
   cargo build --release
   ```

### Build Fails?

1. Run `cargo clean`
2. Update Rust: `rustup update nightly`
3. Try again with safe script

### Need GPU Features?

Only if you have:
- Latest GPU drivers (< 3 months old)
- Windows fully updated
- Created system restore point
- Tested on non-critical system first

Then:
```powershell
# Build with GPU (AT YOUR OWN RISK)
cargo build --profile=safe --no-default-features --features "hive-gpu"
```

## More Information

- [BSOD Analysis](./BSOD_ANALYSIS.md) - Why BSODs happen
- [Windows Build Guide](./WINDOWS_BUILD_GUIDE.md) - Complete guide
- [Guardrails System](./GUARDRAILS.md) - Protection details
- [Guardrails Quick Start](../README_GUARDRAILS.md) - Overview

## Success Tips

✅ Always use `build-windows-safe.ps1` script  
✅ Never use `--all-features` on Windows  
✅ Keep GPU drivers updated if using GPU features  
✅ Monitor Event Viewer for warnings  
✅ Consider WSL2 for most stable experience  

## Support

If you still experience issues:

1. Check Event Viewer → System → BugCheck events
2. Create GitHub issue with:
   - Windows version
   - GPU model and driver version
   - BSOD error code
   - Build command used
   - System specs (CPU, RAM, disk)


