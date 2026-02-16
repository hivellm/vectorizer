---
title: Getting Started
module: getting-started
id: getting-started-index
order: 0
description: Installation and quick start guides
tags: [getting-started, installation, quick-start, tutorial]
---

# Getting Started

Complete guides to install Vectorizer and get started quickly.

## Installation Guides

### [Installation Guide](./INSTALLATION.md)
Quick installation and overview:
- Quick installation scripts (Linux/macOS, Windows)
- Manual installation
- Verification steps

### [Docker Installation](./DOCKER.md)
Complete Docker deployment guide:
- Docker Compose examples
- Volumes and networking
- Health checks and resource limits
- Backup and restore

### [Building from Source](./BUILD_FROM_SOURCE.md)
Build Vectorizer from source code:
- Prerequisites and dependencies
- Build process and optimization
- Feature flags and cross-compilation
- Development workflow

## Quick Start Guides

### ⭐ [Setup Wizard Guide](./SETUP_WIZARD.md) (Recommended)
Guided first-time configuration:
- Automatic project detection
- Template-based configuration
- Collection suggestions
- Workspace generation

### [Quick Start Guide](./QUICK_START.md)
Get up and running in minutes:
- Create your first collection
- Insert documents
- Perform searches
- Using SDKs

### [First Steps](./FIRST_STEPS.md)
Complete guide after installation:
- Verify installation
- Create first collection
- Insert first vectors
- Perform first search
- Next steps

### [Quick Start (Windows)](./QUICK_START_WINDOWS.md)
Windows-specific guide:
- Windows installation
- PowerShell commands
- Windows service management

## Quick Installation

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

**Windows:**
```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

## Next Steps

After installation:
1. **[Setup Wizard](./SETUP_WIZARD.md)** - ⭐ Guided configuration (recommended)
2. **[First Steps](./FIRST_STEPS.md)** - Manual verification and setup
3. **[Creating Collections](../collections/CREATING.md)** - Create collections
4. **[Basic Search](../search/BASIC.md)** - Start searching
5. **[Use Cases](../use-cases/)** - See examples

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Collection management
- [Search Guide](../search/SEARCH.md) - Search operations
- [SDKs Guide](../sdks/README.md) - Client SDKs
