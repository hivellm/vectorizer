# Build Optimization Guide

This document describes optimizations applied to speed up Rust builds in the Vectorizer project.

## 🚀 Quick Start

### 1. Install and Setup sccache (Recommended - **Obligatory for large projects**)

**Linux/WSL:**
```bash
# Install sccache
cargo install sccache

# Setup for current session
export RUSTC_WRAPPER=$(which sccache)

# Or use the setup script
source scripts/setup-sccache.sh

# Make permanent (add to ~/.bashrc or ~/.zshrc)
echo 'export RUSTC_WRAPPER=$(which sccache)' >> ~/.bashrc
```

**Windows (PowerShell):**
```powershell
# Install sccache
cargo install sccache

# Setup for current session
$env:RUSTC_WRAPPER = "C:\Users\$env:USERNAME\.cargo\bin\sccache.exe"

# Or use the setup script
.\scripts\setup-sccache.ps1

# Make permanent (add to PowerShell profile)
# Find profile: $PROFILE
# Add: $env:RUSTC_WRAPPER = "C:\Users\$env:USERNAME\.cargo\bin\sccache.exe"
```

**Verify sccache is working:**
```bash
sccache --show-stats
```

### 2. Install LLD (Fast Linker) - **Highly Recommended**

**Linux/WSL:**
```bash
# Install LLD
sudo apt install lld clang

# Or use the setup script
chmod +x scripts/install-lld.sh
./scripts/install-lld.sh
```

**Windows:**
- No installation needed! Rust includes `rust-lld.exe` by default
- Already configured in `.cargo/config.toml`

**macOS:**
```bash
brew install llvm
```

## 📊 Optimization Features

### 1. sccache - Compilation Cache
- **What it does**: Caches compiled artifacts, dramatically speeding up rebuilds
- **Impact**: 50-90% faster builds on repeated compilations
- **Storage**: Defaults to local disk cache (~10GB), can be configured for Redis/S3
- **Status**: ✅ Configured

### 2. LLD (LLVM Linker) - Fast Linking
- **What it does**: Uses LLVM's fast linker instead of default system linker
- **Impact**: 2-5x faster linking for large binaries
- **Status**: ✅ Configured in `.cargo/config.toml`
- **Installation**: See scripts/install-lld.sh

### 3. Incremental Compilation
- **What it does**: Only recompiles changed code
- **Impact**: 30-70% faster incremental builds
- **Status**: ✅ Enabled in dev profile

### 4. Parallel Compilation
- **What it does**: Uses all CPU cores for compilation
- **Impact**: 4-8x faster on multi-core systems
- **Status**: ✅ Configured (jobs = 0 = all cores)

### 5. Optimized Dependencies
- **tokio**: Uses only needed features instead of `["full"]` (~30% faster compilation)
- **default-features = false**: Applied to most dependencies to reduce compile time
- **Status**: ✅ Optimized in Cargo.toml

### 6. Optimized Release Profile
- **LTO**: Full Link-Time Optimization (fat LTO)
- **Codegen Units**: 1 (maximum optimization)
- **Strip**: Symbols stripped to reduce binary size
- **Status**: ✅ Configured

### 7. Optimized Dev Profile
- **Debug Info**: Line tables only (faster compilation)
- **Incremental**: Enabled for faster rebuilds
- **Status**: ✅ Configured

## 🔧 Configuration Files

### `.cargo/config.toml`
- Parallel compilation (all cores)
- Incremental compilation enabled
- LLD linker configured (Linux & Windows)
- Network optimizations
- sccache configuration options

### `Cargo.toml` Profiles
- **dev**: Fast compilation, incremental enabled
- **release**: Maximum optimization, LTO enabled
- **ci**: Fast CI builds, no LTO
- **perf**: Performance testing profile

### `Cargo.toml` Dependencies
- **Optimized features**: Most dependencies use `default-features = false`
- **tokio**: Only essential features (saves ~30% compile time)
- **serde**: Minimal features
- **regex**: Minimal features (still expensive - consider alternatives)

## ⚠️ Code Organization Best Practices

### Avoid `pub use crate::*`
**Problem**: `pub use *` inflates the dependency graph, causing unnecessary recompilations.

**Current Status**: Found 32 instances of `pub use *` in the codebase:
- `src/api/mod.rs`: 3 instances
- `src/models/qdrant/mod.rs`: 5 instances
- `src/workspace/mod.rs`: 5 instances
- And more...

**Recommendation**: 
- Replace `pub use module::*` with explicit `pub use module::Item`
- Only export what's actually needed
- Consider splitting large modules into smaller crates (workspace)

**Example:**
```rust
// ❌ Bad - inflates dependency graph
pub use advanced_api::*;

// ✅ Good - explicit exports
pub use advanced_api::{SearchRequest, SearchResponse, CreateCollection};
```

### Build Scripts (`build.rs`)
**Current Status**: `build.rs` is necessary for protobuf compilation.

**Best Practices**:
- ✅ Proto files are committed (not generated)
- ✅ Only recompiles when proto files change
- ⚠️ Any change to `build.rs` triggers full rebuild

**Recommendation**:
- Keep `build.rs` minimal
- Generate code before commit when possible
- Only use `build.rs` when absolutely necessary

### Dependency Management

**Heavy Dependencies to Watch**:
1. **chrono** (166 usages) - Consider migrating to `time` crate in future
2. **regex** - Expensive, use simpler string matching where possible
3. **tokio** - ✅ Optimized (no longer using `["full"]`)

**Optimization Applied**:
- ✅ `default-features = false` on most dependencies
- ✅ Minimal feature sets
- ✅ Optional dependencies properly marked

## 📈 Expected Performance Improvements

| Scenario | Without Optimizations | With Optimizations | Improvement |
|----------|----------------------|-------------------|-------------|
| Clean build | 100% | 100% | Baseline |
| Incremental build (no changes) | 80% | 5-10% | **8-16x faster** |
| Incremental build (small changes) | 60% | 15-25% | **2.4-4x faster** |
| Full rebuild (with sccache) | 100% | 20-40% | **2.5-5x faster** |
| Linking (with LLD) | 100% | 20-50% | **2-5x faster** |

## 🎯 Best Practices

1. **Always use sccache** - It's the single biggest improvement
2. **Use LLD linker** - Significantly faster for large binaries
3. **Use incremental builds** - Don't use `cargo clean` unless necessary
4. **Build in release mode only when needed** - Dev builds are much faster
5. **Use `cargo check` for quick validation** - Faster than full build
6. **Use `cargo clippy` for linting** - Faster than full build
7. **Avoid `pub use *`** - Use explicit exports
8. **Minimize build.rs changes** - They trigger full rebuilds

## 🔍 Monitoring Build Performance

Check sccache statistics:
```bash
sccache --show-stats
```

Check linker being used:
```bash
# Linux
ldd --version  # Default linker
lld --version   # LLD linker

# Windows
link.exe /?     # MSVC linker
rust-lld.exe --version  # Rust LLD
```

## 🐛 Troubleshooting

### sccache not working
- Verify: `echo $RUSTC_WRAPPER` (Linux) or `$env:RUSTC_WRAPPER` (Windows)
- Check: `sccache --show-stats` shows cache hits
- Restart: `sccache --stop-server && sccache --start-server`

### LLD not found
- Linux: Install with `sudo apt install lld clang`
- Windows: Use `rust-lld.exe` (bundled with Rust)
- macOS: Install with `brew install llvm`
- Verify: Check `.cargo/config.toml` has correct linker path

### Slow builds
- Check CPU usage: `htop` or Task Manager
- Verify parallel compilation: Look for multiple rustc processes
- Check disk I/O: sccache cache location might be slow
- Verify LLD is being used: Check build output for linker

### Cache size issues
- Default cache: ~10GB in `~/.cache/sccache`
- Configure size: `sccache --set-config cache.size=20G`
- Use remote cache: Redis or S3 for team sharing

### Dependency compilation issues
- If a dependency fails with `default-features = false`, check if it needs specific features
- Some crates require default features - check their documentation
- Consider making problematic dependencies optional

## 📚 Additional Resources

- [sccache Documentation](https://github.com/mozilla/sccache)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Cargo Book - Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [LLD Documentation](https://lld.llvm.org/)
- [Rust Compilation Performance](https://nnethercote.github.io/perf-book/compile-times.html)
