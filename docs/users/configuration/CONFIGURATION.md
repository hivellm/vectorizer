---
title: Configuration Guide
module: configuration
id: configuration-guide
order: 1
description: Configuring Vectorizer server settings and options
tags: [configuration, settings, server, options]
---

# Configuration Guide

Complete guide to configuring Vectorizer server settings and behavior.

## Configuration Methods

Vectorizer supports multiple configuration methods with the following priority (highest to lowest):

1. **Command line arguments** - Highest priority
2. **Environment variables** - Second priority
3. **YAML configuration file** - Third priority
4. **Default values** - Lowest priority

## Quick Reference

### Essential Configuration

**Minimal configuration (uses defaults):**

```bash
vectorizer
```

**Custom host and port:**

```bash
vectorizer --host 0.0.0.0 --port 15002
```

**With configuration file:**

```bash
vectorizer --config /etc/vectorizer/config.yml
```

## Configuration Guides

- **[Server Configuration](./SERVER.md)** - Network, ports, host binding, reverse proxy
- **[TLS/SSL Configuration](./TLS.md)** - HTTPS, certificates, mTLS, cipher suites
- **[Logging Configuration](./LOGGING.md)** - Log levels, output, filtering, aggregation
- **[Data Directory](./DATA_DIRECTORY.md)** - Storage paths, snapshots, backups
- **[Performance Tuning](./PERFORMANCE_TUNING.md)** - Threads, memory, optimization

## Common Options

### Command Line Arguments

```bash
vectorizer [OPTIONS]

OPTIONS:
    --host <HOST>              Host to bind to [default: 0.0.0.0]
    --port <PORT>              Port to listen on [default: 15002]
    --data-dir <DATA_DIR>      Data directory path
    --log-level <LOG_LEVEL>    Logging level [default: info]
                                [possible values: trace, debug, info, warn, error]
    --workers <WORKERS>        Number of worker threads
    --config <CONFIG>          Path to YAML configuration file
    --help                     Print help information
    --version                  Print version information
```

### Environment Variables

| Variable               | Description     | Default           |
| ---------------------- | --------------- | ----------------- |
| `VECTORIZER_HOST`      | Host to bind to | `0.0.0.0`         |
| `VECTORIZER_PORT`      | REST API port   | `15002`           |
| `VECTORIZER_MCP_PORT`  | MCP server port | `15003`           |
| `VECTORIZER_LOG_LEVEL` | Logging level   | `info`            |
| `VECTORIZER_DATA_DIR`  | Data directory  | Platform-specific |
| `VECTORIZER_WORKERS`   | Worker threads  | Auto-detect       |

## Collection Configuration

### Default Settings

```json
{
  "dimension": 384,
  "metric": "cosine",
  "quantization": {
    "enabled": true,
    "type": "scalar",
    "bits": 8
  },
  "compression": {
    "enabled": false
  }
}
```

### Custom Configuration

```bash
curl -X POST http://localhost:15002/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my_collection",
    "dimension": 512,
    "metric": "euclidean",
    "quantization": {
      "enabled": true,
      "type": "scalar",
      "bits": 8
    }
  }'
```

## Embedding Configuration

### Default Provider

Vectorizer uses BM25 as the default embedding provider.

### Available Providers

- **BM25**: Fast, keyword-based (default)
- **BERT**: Deep learning embeddings
- **MiniLM**: Lightweight transformer
- **Custom**: Your own embedding model

## YAML Configuration File

### Configuration File Format

Vectorizer supports YAML configuration files for comprehensive settings:

```yaml
# Server configuration
server:
  host: "0.0.0.0"
  port: 15002
  mcp_port: 15003

# Logging configuration
logging:
  level: "info"
  log_requests: true
  log_responses: false
  log_errors: true

# Storage configuration
storage:
  data_dir: "/var/lib/vectorizer"
  snapshots_dir: "/var/lib/vectorizer/snapshots"
  max_snapshots: 10
  retention_days: 7

# GPU configuration (macOS Metal)
gpu:
  enabled: true
  device: "auto"

# File Watcher configuration
file_watcher:
  enabled: true
  watch_paths:
    - "/path/to/project"
  debounce_delay_ms: 1000
  collection_name: "workspace-files"
  collection_mapping:
    "*/docs/**/*.md": "documentation"
    "*/src/**/*.rs": "rust-code"
```

### Configuration File Locations

Vectorizer checks for configuration files in this order:

1. Path specified by `--config` argument
2. `./workspace.yml` (current directory)
3. `~/.vectorizer/config.yml` (user home)
4. `/etc/vectorizer/config.yml` (system-wide)

## Quick Configuration Examples

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

- [Service Management](../operations/SERVICE_MANAGEMENT.md) - Managing the service
- [Troubleshooting](../operations/TROUBLESHOOTING.md) - Configuration issues
