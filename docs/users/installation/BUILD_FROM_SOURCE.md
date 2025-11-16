---
title: Building from Source
module: installation
id: build-from-source
order: 3
description: Complete guide to building Vectorizer from source code
tags: [installation, build, source, compilation, rust]
---

# Building from Source

Complete guide to building Vectorizer from source code.

## Prerequisites

### Required Tools

**Rust Toolchain:**

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be 1.9+ with edition 2024
cargo --version
```

**Build Dependencies:**

**Linux (Ubuntu/Debian):**

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git
```

**Linux (Fedora/RHEL):**

```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    curl \
    git
```

**macOS:**

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install openssl pkg-config
```

**Windows:**

```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++" workload

# Install Rust
# Download from: https://rustup.rs/
```

## Basic Build

### Clone Repository

```bash
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer
```

### Build Release Binary

**Standard build:**

```bash
cargo build --release
```

**Binary location:**

- Linux/macOS: `target/release/vectorizer`
- Windows: `target/release/vectorizer.exe`

**Build time:** ~5-15 minutes (depending on hardware)

### Build Debug Binary

**Debug build (faster compilation, slower runtime):**

```bash
cargo build
```

**Binary location:**

- Linux/macOS: `target/debug/vectorizer`
- Windows: `target/debug/vectorizer.exe`

## Feature Flags

### Available Features

**List available features:**

```bash
cargo build --release --help | grep features
```

**Common features:**

- `hive-gpu`: GPU acceleration (macOS Metal)
- `transmutation`: Document conversion support
- `full`: All features enabled

### Build with Features

**GPU support (macOS):**

```bash
cargo build --release --features hive-gpu
```

**Transmutation support:**

```bash
cargo build --release --features transmutation
```

**All features:**

```bash
cargo build --release --features full
```

**Multiple features:**

```bash
cargo build --release --features "hive-gpu transmutation"
```

## Optimization

### Release Profile Optimization

**Default release profile:**

```toml
[profile.release]
opt-level = 3
lto = false
codegen-units = 1
```

**Custom optimization:**

Edit `Cargo.toml`:

```toml
[profile.release]
opt-level = 3          # Optimization level (0-3, s, z)
lto = true            # Link-time optimization
codegen-units = 1     # Code generation units
panic = 'abort'       # Panic strategy
strip = true          # Strip symbols
```

**Build with custom profile:**

```bash
cargo build --release
```

### Cross-Compilation

**Install target:**

```bash
# List available targets
rustup target list

# Install target (example: Linux ARM64)
rustup target add aarch64-unknown-linux-gnu
```

**Build for target:**

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

**Cross-compilation dependencies:**

For cross-compilation, you may need additional tools:

```bash
# Linux ARM64 example
sudo apt-get install gcc-aarch64-linux-gnu
```

## Development Build

### Development Setup

**Install development dependencies:**

```bash
# Install Rust toolchain components
rustup component add rustfmt clippy

# Install development tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-audit   # Security audit
```

### Development Workflow

**Watch mode (auto-rebuild):**

```bash
cargo watch -x "build --release"
```

**Run tests:**

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

**Format code:**

```bash
cargo fmt
```

**Lint code:**

```bash
cargo clippy -- -D warnings
```

## Installation

### Install Binary

**Linux/macOS:**

```bash
# Copy to system directory
sudo cp target/release/vectorizer /usr/local/bin/

# Or install via cargo
cargo install --path . --locked
```

**Windows:**

```powershell
# Copy to PATH directory
Copy-Item target\release\vectorizer.exe $env:USERPROFILE\.cargo\bin\
```

### Verify Installation

```bash
# Check version
vectorizer --version

# Check help
vectorizer --help

# Test run
vectorizer --host 127.0.0.1 --port 15002
```

## Troubleshooting

### Build Failures

**Common issues:**

1. **Out of memory:**
   ```bash
   # Increase swap or reduce parallelism
   CARGO_BUILD_JOBS=2 cargo build --release
   ```

2. **Missing dependencies:**
   ```bash
   # Linux: Install missing packages
   sudo apt-get install libssl-dev pkg-config
   ```

3. **Linker errors:**
   ```bash
   # Set linker path
   export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
   ```

### Slow Builds

**Optimize build speed:**

```bash
# Use more CPU cores
CARGO_BUILD_JOBS=8 cargo build --release

# Enable incremental compilation (default in debug)
CARGO_INCREMENTAL=1 cargo build --release

# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Version Mismatch

**Error:** `edition 2024 is not stable`

**Solution:**

```bash
# Use nightly Rust for edition 2024
rustup toolchain install nightly
rustup default nightly

# Or use specific version
rustup toolchain install 1.90.0
rustup default 1.90.0
```

## CI/CD Integration

### GitHub Actions

**Build workflow:**

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release
      - name: Test
        run: cargo test --release
```

### GitLab CI

**Build pipeline:**

```yaml
build:
  image: rust:1.90
  script:
    - cargo build --release
    - cargo test --release
  artifacts:
    paths:
      - target/release/vectorizer
```

## Related Topics

- [Installation Guide](./INSTALLATION.md) - General installation guide
- [Docker Installation](./DOCKER.md) - Docker deployment
- [Configuration Guide](../configuration/CONFIGURATION.md) - Configuration after build

