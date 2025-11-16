---
title: Configuration Guide
module: configuration
id: configuration-guide
order: 1
description: Configuring Vectorizer server settings and options
tags: [configuration, settings, server, options]
---

# Configuration Guide

Configure Vectorizer server settings and behavior.

## Server Configuration

### Host and Port

Default configuration:
- **Host**: `0.0.0.0` (all interfaces)
- **Port**: `15002`

### Environment Variables

```bash
export VECTORIZER_HOST=0.0.0.0
export VECTORIZER_PORT=15002
```

### Command Line Arguments

```bash
# Basic usage
vectorizer --host 0.0.0.0 --port 15002

# With custom data directory
vectorizer --host 0.0.0.0 --port 15002 --data-dir /var/lib/vectorizer

# Enable verbose logging
vectorizer --host 0.0.0.0 --port 15002 --log-level debug
```

### Available Options

```bash
vectorizer --help
```

Common options:
- `--host`: Bind address (default: 0.0.0.0)
- `--port`: Server port (default: 15002)
- `--data-dir`: Data directory path
- `--log-level`: Logging level (trace, debug, info, warn, error)
- `--workers`: Number of worker threads

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

## Logging Configuration

### Log Levels

```bash
# Set log level via environment variable
export VECTORIZER_LOG_LEVEL=debug

# Or via command line
vectorizer --log-level debug
```

Available levels:
- **trace**: Very detailed logs (development only)
- **debug**: Debug information
- **info**: General information (default)
- **warn**: Warnings only
- **error**: Errors only

### Log Output

Logs are written to:
- **stdout**: When running directly
- **systemd journal**: When running as systemd service (Linux)
- **Event Log**: When running as Windows Service

### Viewing Logs

**Linux (systemd):**
```bash
# View recent logs
sudo journalctl -u vectorizer -n 100

# Follow logs in real-time
sudo journalctl -u vectorizer -f

# Filter by log level
sudo journalctl -u vectorizer -p err
```

**Windows:**
```powershell
# View Event Log
Get-EventLog -LogName Application -Source Vectorizer -Newest 50

# Filter by level
Get-EventLog -LogName Application -Source Vectorizer -EntryType Error
```

## Performance Tuning

### Memory Settings

```bash
# Set maximum memory (if applicable)
export VECTORIZER_MAX_MEMORY=4G
```

### Thread Configuration

```bash
# Set number of threads
export VECTORIZER_THREADS=4
```

## Logging Configuration

### Log Levels

- **error**: Only errors
- **warn**: Warnings and errors
- **info**: Informational messages (default)
- **debug**: Detailed debugging information

### Set Log Level

```bash
export VECTORIZER_LOG_LEVEL=info
```

## Data Directory

### Default Locations

- **Linux**: `/var/lib/vectorizer`
- **Windows**: `%ProgramData%\Vectorizer`

### Custom Data Directory

Modify the service configuration to use a custom directory.

## Related Topics

- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Managing the service
- [Troubleshooting](../troubleshooting/TROUBLESHOOTING.md) - Configuration issues

