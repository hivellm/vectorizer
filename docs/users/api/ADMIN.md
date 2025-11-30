---
title: Admin and System API
module: api
id: admin-api
order: 11
description: Administrative and system management endpoints
tags: [api, admin, system, management]
---

# Admin and System API

Administrative endpoints for server management, monitoring, and configuration.

## Overview

Admin and system endpoints provide:

- Server status and health monitoring
- Configuration management
- Log access
- Server restart
- Indexing progress tracking
- Prometheus metrics

## System Endpoints

### Get Server Status

Get detailed server status information.

**Endpoint:** `GET /api/status`

**Response:**

```json
{
  "status": "running",
  "version": "1.3.0",
  "uptime_seconds": 3600,
  "collections": 5,
  "total_vectors": 125000,
  "memory_usage_mb": 512,
  "cpu_usage_percent": 15.5
}
```

**Example:**

```bash
curl http://localhost:15002/api/status
```

**Python SDK:**

```python
from vectorizer_sdk import VectorizerClient

client = VectorizerClient("http://localhost:15002")

status = await client.get_status()
print(f"Status: {status['status']}")
print(f"Uptime: {status['uptime_seconds']} seconds")
```

### Get Server Logs

Retrieve server logs.

**Endpoint:** `GET /api/logs`

**Query Parameters:**

| Parameter | Type   | Required | Description                                           |
| --------- | ------ | -------- | ----------------------------------------------------- |
| `limit`   | number | No       | Maximum log entries (default: 100)                    |
| `level`   | string | No       | Filter by log level (trace, debug, info, warn, error) |
| `since`   | string | No       | ISO timestamp to filter logs from                     |

**Response:**

```json
{
  "logs": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "level": "info",
      "message": "Server started successfully",
      "module": "server"
    },
    {
      "timestamp": "2024-01-15T10:30:05Z",
      "level": "info",
      "message": "Collection 'docs' created",
      "module": "collections"
    }
  ],
  "total": 2
}
```

**Example:**

```bash
curl "http://localhost:15002/api/logs?limit=50&level=info"
```

### Get Server Configuration

Get current server configuration.

**Endpoint:** `GET /api/config`

**Response:**

```json
{
  "host": "0.0.0.0",
  "port": 15002,
  "data_dir": "/var/lib/vectorizer",
  "log_level": "info",
  "workers": 8,
  "replication": {
    "enabled": false
  }
}
```

**Example:**

```bash
curl http://localhost:15002/api/config
```

### Update Server Configuration

Update server configuration (requires restart).

**Endpoint:** `POST /api/config`

**Request Body:**

```json
{
  "log_level": "debug",
  "workers": 16,
  "data_dir": "/new/path/to/data"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Configuration updated. Server restart required."
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "log_level": "debug",
    "workers": 16
  }'
```

## Admin Endpoints

### Restart Server

Restart the Vectorizer server.

**Endpoint:** `POST /admin/restart`

**Response:**

```json
{
  "success": true,
  "message": "Server restart initiated"
}
```

**Warning:** This will restart the server and disconnect all active connections.

**Example:**

```bash
curl -X POST http://localhost:15002/admin/restart
```

## Collection Management

### Force Save Collection

Force immediate save of a collection to disk.

**Endpoint:** `POST /api/collections/{name}/force-save`

**Response:**

```json
{
  "success": true,
  "message": "Collection 'docs' saved successfully"
}
```

**Example:**

```bash
curl -X POST http://localhost:15002/api/collections/docs/force-save
```

**Python SDK:**

```python
result = await client.force_save_collection("docs")
if result["success"]:
    print("Collection saved successfully")
```

## Monitoring Endpoints

### Get Indexing Progress

Get current indexing progress for collections.

**Endpoint:** `GET /indexing/progress`

**Response:**

```json
{
  "collections": [
    {
      "name": "docs",
      "total_vectors": 1000,
      "indexed_vectors": 850,
      "progress_percent": 85.0,
      "status": "indexing"
    },
    {
      "name": "code",
      "total_vectors": 500,
      "indexed_vectors": 500,
      "progress_percent": 100.0,
      "status": "complete"
    }
  ]
}
```

**Example:**

```bash
curl http://localhost:15002/indexing/progress
```

### Get Prometheus Metrics

Get metrics in Prometheus format.

**Endpoint:** `GET /prometheus/metrics`

**Response:**

```
# HELP vectorizer_collections_total Total number of collections
# TYPE vectorizer_collections_total gauge
vectorizer_collections_total 5

# HELP vectorizer_vectors_total Total number of vectors
# TYPE vectorizer_vectors_total gauge
vectorizer_vectors_total 125000

# HELP vectorizer_memory_usage_bytes Memory usage in bytes
# TYPE vectorizer_memory_usage_bytes gauge
vectorizer_memory_usage_bytes 536870912
```

**Example:**

```bash
curl http://localhost:15002/prometheus/metrics
```

## Use Cases

### Server Monitoring

Monitor server health and status:

```python
# Get server status
status = await client.get_status()
print(f"Server uptime: {status['uptime_seconds']} seconds")
print(f"Memory usage: {status['memory_usage_mb']} MB")

# Get indexing progress
progress = await client.get_indexing_progress()
for collection in progress["collections"]:
    print(f"{collection['name']}: {collection['progress_percent']}%")
```

### Log Analysis

Analyze server logs:

```python
# Get recent error logs
logs = await client.get_logs(limit=100, level="error")

for log in logs["logs"]:
    print(f"[{log['timestamp']}] {log['level']}: {log['message']}")
```

### Configuration Management

Manage server configuration:

```python
# Get current configuration
config = await client.get_config()
print(f"Log level: {config['log_level']}")
print(f"Workers: {config['workers']}")

# Update configuration
await client.update_config({
    "log_level": "debug",
    "workers": 16
})

# Restart server to apply changes
await client.restart_server()
```

### Collection Management

Force save collections:

```python
# Force save all collections
collections = await client.list_collections()

for collection in collections["collections"]:
    result = await client.force_save_collection(collection["name"])
    if result["success"]:
        print(f"Saved {collection['name']}")
```

## Best Practices

1. **Monitor regularly**: Check server status and logs regularly
2. **Use Prometheus**: Integrate Prometheus metrics for monitoring
3. **Check indexing progress**: Monitor indexing progress for large collections
4. **Backup before restart**: Always backup before restarting server
5. **Review logs**: Regularly review logs for errors and warnings
6. **Configuration changes**: Test configuration changes in development first

## Related Topics

- [Operations Guide](../operations/MONITORING.md) - Monitoring and health checks
- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
- [Troubleshooting Guide](../operations/TROUBLESHOOTING.md) - Common issues
