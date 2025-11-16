---
title: Installation Guide
module: installation
id: installation-guide
order: 1
description: Complete guide for installing Vectorizer on Linux, macOS, and Windows
tags: [installation, setup, linux, windows, macos]
---

# Installation Guide

This guide covers installing Vectorizer on different platforms.

## Quick Installation

### Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

This will:
- ✅ Install Vectorizer CLI to `/usr/local/bin`
- ✅ Configure as systemd service (Linux)
- ✅ Start service automatically
- ✅ Enable auto-start on boot

### Windows

```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

**Note:** Service installation requires Administrator privileges.

This will:
- ✅ Install Vectorizer CLI to `%USERPROFILE%\.cargo\bin`
- ✅ Configure as Windows Service
- ✅ Start service automatically
- ✅ Enable auto-start on boot

## Manual Installation

### Building from Source

```bash
git clone https://github.com/hivellm/vectorizer.git
cd vectorizer
cargo build --release
```

The binary will be at `target/release/vectorizer` (or `target/release/vectorizer.exe` on Windows).

### Docker Installation

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  --restart unless-stopped \
  ghcr.io/hivellm/vectorizer:latest
```

## Verification

After installation, verify the installation:

```bash
# Check CLI version
vectorizer --version

# Check service status (Linux)
sudo systemctl status vectorizer

# Check service status (Windows)
Get-Service Vectorizer

# Test API endpoint
curl http://localhost:15002/health
```

## Related Topics

- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Managing the Vectorizer service
- [Getting Started](../getting-started/QUICK_START.md) - Next steps after installation
- [Configuration](../configuration/CONFIGURATION.md) - Configuring Vectorizer

