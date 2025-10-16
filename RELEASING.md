# Release Process

This document describes the release process for Vectorizer, including building multi-platform binaries, Docker images, and packages.

## Overview

Vectorizer uses automated GitHub Actions workflows to build and publish releases for multiple platforms:

- **Linux**: x86_64 (GNU/MUSL), ARM64 (MUSL)
- **macOS**: x86_64 (Intel), ARM64 (Apple Silicon)
- **Windows**: x86_64 (MSVC)

### Package Formats

- **Linux**:
  - TAR.GZ archives (all architectures)
  - Debian packages (.deb)
  - AppImage (universal Linux binary)
  
- **macOS**:
  - TAR.GZ archives (Intel and Apple Silicon)

- **Windows**:
  - ZIP archives
  - MSI installer

- **Docker**:
  - Multi-platform images (linux/amd64, linux/arm64)
  - Regular and unprivileged variants
  - SBOM and image signing support

## Creating a Release

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "X.Y.Z"
```

### 2. Update CHANGELOG

Add release notes to `CHANGELOG.md` with:
- New features
- Bug fixes
- Breaking changes
- Performance improvements

### 3. Create and Push Tag

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to vX.Y.Z"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
```

### 4. Create GitHub Release

1. Go to https://github.com/hivellm/vectorizer/releases/new
2. Select the tag you just created
3. Click "Generate release notes" or write custom notes
4. Click "Publish release"

The release workflows will automatically:
- Build binaries for all platforms
- Create Debian packages
- Build AppImage
- Create MSI installer
- Build and push Docker images
- Upload all artifacts to the release

## Release Workflows

### Main Workflows

1. **`release-artifacts.yml`**
   - Triggered on: Release published
   - Builds: All platform binaries, Debian packages, AppImage, MSI
   - Outputs: Artifacts uploaded to GitHub release

2. **`docker-image.yml`**
   - Triggered on: Tags matching `v*.*.*`
   - Builds: Multi-platform Docker images
   - Outputs: Pushed to Docker Hub and GitHub Container Registry

3. **`rust-lint.yml`**
   - Triggered on: Push/PR
   - Validates: Code formatting and linting
   - Outputs: CI status

### Manual Workflow Dispatch

You can manually trigger workflows from the Actions tab:

1. Go to https://github.com/hivellm/vectorizer/actions
2. Select the workflow
3. Click "Run workflow"
4. Specify parameters (e.g., tag name)

## Testing Releases

### Local Testing

#### Linux (Debian Package)
```bash
# Download .deb file
wget https://github.com/hivellm/vectorizer/releases/download/vX.Y.Z/vectorizer_X.Y.Z_amd64.deb

# Install
sudo dpkg -i vectorizer_X.Y.Z_amd64.deb

# Verify
vectorizer --version
systemctl status vectorizer
```

#### Linux (AppImage)
```bash
# Download AppImage
wget https://github.com/hivellm/vectorizer/releases/download/vX.Y.Z/vectorizer-x86_64.AppImage

# Make executable
chmod +x vectorizer-x86_64.AppImage

# Run
./vectorizer-x86_64.AppImage --version
```

#### Windows (MSI)
```powershell
# Download MSI
Invoke-WebRequest -Uri "https://github.com/hivellm/vectorizer/releases/download/vX.Y.Z/vectorizer-x86_64.msi" -OutFile "vectorizer.msi"

# Install
msiexec /i vectorizer.msi

# Verify
vectorizer --version
```

#### Docker
```bash
# Pull image
docker pull vectorizer/vectorizer:vX.Y.Z

# Run
docker run -p 15002:15002 vectorizer/vectorizer:vX.Y.Z

# Test multi-arch
docker run --platform linux/amd64 vectorizer/vectorizer:vX.Y.Z
docker run --platform linux/arm64 vectorizer/vectorizer:vX.Y.Z
```

## Build Locally

### Prerequisites

- Rust 1.88+ (edition 2024)
- Protobuf compiler
- Platform-specific tools (see below)

### Linux Build

```bash
# Install dependencies
sudo apt-get install build-essential libssl-dev pkg-config protobuf-compiler

# Build
cargo build --release --features full

# Build Debian package
cargo install cargo-deb
cargo deb --no-strip
```

### macOS Build

```bash
# Install dependencies
brew install protobuf

# Build for current architecture
cargo build --release --features full

# Build for both architectures
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin --features full
cargo build --release --target x86_64-apple-darwin --features full
```

### Windows Build

```powershell
# Install dependencies
choco install protoc

# Build
cargo build --release --features full

# Build MSI (requires WiX Toolset)
dotnet tool install --global wix --version 5.0.2
$env:CargoTargetBinDir = "target\release"
wix build -arch x64 -ext WixToolset.UI.wixext wix\main.wxs -o vectorizer.msi
```

### Docker Build

```bash
# Build for current platform
docker build -t vectorizer:local .

# Build multi-platform
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t vectorizer:multi .

# Build with cargo-chef caching
docker buildx build --cache-from type=local,src=/tmp/.buildx-cache \
                    --cache-to type=local,dest=/tmp/.buildx-cache \
                    -t vectorizer:cached .
```

## Troubleshooting

### Build Failures

1. **Cross-compilation issues**
   - Ensure `taiki-e/setup-cross-toolchain-action` is properly configured
   - Check target triple matches rustc expectations
   - Verify MUSL toolchain is installed for Linux

2. **Protobuf errors**
   - Install protoc: `sudo apt-get install protobuf-compiler`
   - Or use setup-protoc action in CI

3. **Windows MSI build fails**
   - Ensure WiX Toolset v5+ is installed
   - Verify `CargoTargetBinDir` environment variable is set
   - Check .wxs file syntax

4. **Docker build timeouts**
   - Use cargo-chef for dependency caching
   - Enable BuildKit: `DOCKER_BUILDKIT=1`
   - Increase timeout in workflow

### Release Workflow Issues

1. **Artifacts not uploading**
   - Check `GITHUB_TOKEN` permissions
   - Verify release is published (not draft)
   - Check artifact paths in workflow

2. **Docker push fails**
   - Verify `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN` secrets
   - Check repository permissions
   - Ensure multi-platform build succeeds

3. **MSI installer not working**
   - Test locally first
   - Check Windows resources in build.rs
   - Verify icon file exists

## CI/CD Configuration

### Required Secrets

GitHub repository secrets needed:
- `GITHUB_TOKEN` (automatically provided)
- `DOCKERHUB_USERNAME` (for Docker Hub)
- `DOCKERHUB_TOKEN` (for Docker Hub)

### Workflow Triggers

- **release-artifacts.yml**: Release published
- **docker-image.yml**: Tags matching `v*.*.*`
- **rust-lint.yml**: Push to main/develop, all PRs

## Release Checklist

- [ ] Version updated in Cargo.toml
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Tag created and pushed
- [ ] GitHub release published
- [ ] All artifacts uploaded successfully
- [ ] Docker images pushed
- [ ] Release announcement (Discord, Twitter, etc.)
- [ ] Update package managers (crates.io, etc.)

## Support

For release issues:
- GitHub Issues: https://github.com/hivellm/vectorizer/issues
- Discussions: https://github.com/hivellm/vectorizer/discussions

