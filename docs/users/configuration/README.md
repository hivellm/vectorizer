---
title: Configuration
module: configuration
id: configuration-index
order: 0
description: Complete configuration guide for Vectorizer
tags: [configuration, settings, server, logging, storage, performance]
---

# Configuration Guide

Complete guide to configuring Vectorizer server, logging, storage, and performance settings.

## Configuration Guides

### [Configuration Overview](./CONFIGURATION.md)

Quick reference and overview:

- Configuration methods and priority
- Common options
- Quick examples

### [Server Configuration](./SERVER.md)

Network and server settings:

- Host binding and port configuration
- Command line arguments
- Environment variables
- YAML configuration files
- Service configuration
- Reverse proxy setup
- Network security

### [Logging Configuration](./LOGGING.md)

Logging setup and management:

- Log levels (trace, debug, info, warn, error)
- Log output destinations
- Request/response logging
- Log filtering and rotation
- Log aggregation (ELK, Loki)
- Log analysis

### [Data Directory Configuration](./DATA_DIRECTORY.md)

Storage and persistence:

- Default and custom data directories
- Directory permissions
- Snapshot configuration
- Disk space management
- Network storage (NFS, SMB)
- Data migration

### [Performance Tuning](./PERFORMANCE_TUNING.md)

Performance optimization:

- Thread configuration
- Memory settings
- GPU acceleration
- Batch processing
- Query caching
- Collection optimization
- Search optimization
- System-level optimization

### [Cluster Configuration](./CLUSTER.md)

Distributed sharding configuration:

- Enabling cluster mode
- Node configuration
- Discovery methods
- Network settings
- Health checks
- Best practices

## Quick Start

### Minimal Configuration

```bash
# Uses all defaults
vectorizer
```

### Custom Host and Port

```bash
vectorizer --host 0.0.0.0 --port 15002
```

### With Configuration File

```bash
vectorizer --config /etc/vectorizer/config.yml
```

## Configuration Priority

1. **Command line arguments** (highest priority)
2. **Environment variables**
3. **YAML configuration file**
4. **Default values** (lowest priority)

## Common Configuration Scenarios

### Development

```bash
vectorizer --host 127.0.0.1 --port 15002 --log-level debug
```

### Production

```bash
vectorizer \
    --host 0.0.0.0 \
    --port 15002 \
    --data-dir /var/lib/vectorizer \
    --log-level info \
    --workers 8
```

### Docker

```yaml
environment:
  - VECTORIZER_HOST=0.0.0.0
  - VECTORIZER_PORT=15002
  - VECTORIZER_LOG_LEVEL=info
```

## Related Topics

- [Service Management](../operations/SERVICE_MANAGEMENT.md) - Service configuration
- [Troubleshooting](../operations/TROUBLESHOOTING.md) - Configuration issues
- [Installation Guide](../getting-started/INSTALLATION.md) - Initial setup
