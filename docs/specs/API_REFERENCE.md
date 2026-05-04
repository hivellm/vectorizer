# API Reference & Integrations

**Version**: 1.8.0
**Base URL**: `http://localhost:15002`  
**MCP Endpoint**: `http://localhost:15002/mcp`  
**Status**: ✅ Production Ready

---

## Overview

Vectorizer provides multiple interfaces optimized for different use cases:

| Interface | Type | Description | Use Cases |
|-----------|------|-------------|-----------|
| **REST API** | Server | HTTP/JSON access | Custom integrations, direct API |
| **MCP API** | Server | Model Context Protocol | AI integration, IDE tools (StreamableHTTP) |
| **Python SDK** | Client | Python client library | Data Science, ML pipelines |
| **TypeScript SDK** | Client | TS/JS client library | Web apps, Node.js |
| **Rust SDK** | Client | Rust client library | High-performance apps |
| **CLI Tools** | Client | Command-line tools | Administration, scripts |

---

## Architecture

**Client-Server** with mandatory authentication:
- **Server**: Centralized Rust application (REST/MCP APIs)
- **SDKs**: Lightweight clients (Python, TypeScript, Rust)
- **Security**: All operations require valid API keys
- **Dashboard**: Local key management (localhost only)

---

## Authentication

All endpoints require API key authentication:

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:15002/health
```

### Getting API Keys

**CLI**:
```bash
vectorizer api-keys create --name "my-app"
vectorizer api-keys list
vectorizer api-keys delete <key-id>
```

**Dashboard**: `http://localhost:15002` (localhost only)

### HiveHub Cluster Mode Authentication

When running in HiveHub cluster mode, additional authentication mechanisms are available:

#### Internal Service Header

The `x-hivehub-service` header allows trusted internal services to bypass API key authentication:

```bash
curl -H "x-hivehub-service: true" http://localhost:15002/api/collections
```

> **Note**: This header should only be used by HiveHub internal services, not external applications.

#### User Context Header

For internal requests requiring tenant scoping, include the `x-hivehub-user-id` header:

```bash
curl -H "x-hivehub-service: true" \
     -H "x-hivehub-user-id: 550e8400-e29b-41d4-a716-446655440000" \
     http://localhost:15002/api/collections
```

When both headers are present:
- API key authentication is bypassed
- A tenant context is created for the specified user
- Collection access is filtered to only those owned by the user

See [HUB_INTEGRATION.md](../HUB_INTEGRATION.md) for complete HiveHub authentication documentation.

---

## REST API Endpoints

### Authentication Requirements Summary

| Category | Auth Required | HiveHub Mode | Permission Required |
|----------|---------------|--------------|---------------------|
| Health/Status | No | No | - |
| Read Operations | Yes | Yes | `ReadOnly` or higher |
| Write Operations | Yes | Yes | `ReadWrite` or higher |
| Admin Operations | Yes | Yes | `Admin` |

### Health & Status

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| GET | `/health` | No | Server health check |
| GET | `/status` | No | Detailed server status |
| GET | `/metrics` | No | Prometheus metrics |
| GET | `/metrics/runtime` | Yes (Admin) | JSON runtime snapshot for the dashboard (CPU, memory, connections, rolling QPS, per-route p50/p99, 5xx rate, WAL state) |

#### `GET /metrics/runtime`

Single JSON snapshot consumed by the dashboard's Overview / Monitoring
pages. Sampled every second by an in-process tick task; each request is
a cheap clone of the latest snapshot.

Response shape (snake_case):

```jsonc
{
  "cpu_percent": 12.4,
  "memory_rss_bytes": 124857600,
  "memory_total_bytes": 17179869184,
  "memory_percent": 0.73,
  "active_connections": 8,
  "uptime_seconds": 3712,
  "qps_window_60s": 142.3,
  "error_rate_5xx_60s": 0.001,
  "throughput_by_route": [
    {"route": "/insert_texts", "qps": 12.0, "p50_ms": 8.2, "p99_ms": 41.0}
  ],
  "wal": {
    "current_seq": 482919,
    "size_bytes": 12582912,
    "last_checkpoint_at": 1714828800,
    "last_checkpoint_seq": 482800
  }
}
```

`wal.*` fields are zero in standalone (non-replicated) mode — the
server is honest about not having a WAL when one isn't configured.
`last_checkpoint_at` is unix seconds and only advances when
`min_confirmed_offset` actually moves forward (retried ACKs do not
update the timestamp).

#### `GET /stats`

```jsonc
{
  "collections": 12,
  "total_vectors": 482919,
  "uptime_seconds": 3712,
  "version": "3.3.0",
  "default_quantization": "sq-8bit",
  "compression_ratio": 4.0
}
```

`default_quantization` is the most-common quantization label across
active collections (`none`, `binary`, `sq-4bit`, `sq-8bit`, `sq-16bit`,
`sq`, or `pq`). `compression_ratio` is the mean static ratio
(uncompressed bits / compressed bits) across collections sharing that
label — dimension-aware for PQ, dimension-independent for the others.
Empty store reports `("none", 1.0)`.

### Collection Management

| Method | Endpoint | Auth | Permission | Description |
|--------|----------|------|------------|-------------|
| GET | `/collections` | Yes | ReadOnly | List all collections (filtered by owner in HiveHub mode) |
| GET | `/collections/{name}` | Yes | ReadOnly | Get collection details |
| GET | `/collections/empty` | Yes | Admin | List all empty collections |
| POST | `/collections` | Yes | ReadWrite | Create collection (quota check in HiveHub mode) |
| DELETE | `/collections/{name}` | Yes | ReadWrite | Delete collection |
| DELETE | `/collections/cleanup` | Yes | Admin | Delete all empty collections (supports ?dry_run=true) |
| POST | `/collections/{name}/reindex` | Yes | Admin | Reindex collection |

### Vector Operations

| Method | Endpoint | Auth | Permission | Description |
|--------|----------|------|------------|-------------|
| POST | `/collections/{name}/search` | Yes | ReadOnly | Search vectors |
| POST | `/collections/{name}/vectors` | Yes | ReadWrite | Insert vectors (quota check in HiveHub mode) |
| PUT | `/collections/{name}/vectors/{id}` | Yes | ReadWrite | Update vector |
| DELETE | `/collections/{name}/vectors/{id}` | Yes | ReadWrite | Delete vector |

### Intelligent Search

| Method | Endpoint | Auth | Permission | Description |
|--------|----------|------|------------|-------------|
| POST | `/intelligent_search` | Yes | ReadOnly | Advanced multi-query search |
| POST | `/semantic_search` | Yes | ReadOnly | Pure semantic search |
| POST | `/contextual_search` | Yes | ReadOnly | Context-aware search |
| POST | `/multi_collection_search` | Yes | ReadOnly | Cross-collection search |

### HiveHub Backup API

These endpoints are only available when HiveHub mode is enabled.

| Method | Endpoint | Auth | Permission | Description |
|--------|----------|------|------------|-------------|
| GET | `/api/hub/backups` | Yes | ReadOnly | List user backups |
| POST | `/api/hub/backups` | Yes | ReadWrite | Create backup |
| GET | `/api/hub/backups/{id}` | Yes | ReadOnly | Get backup metadata |
| GET | `/api/hub/backups/{id}/download` | Yes | ReadOnly | Download backup file |
| POST | `/api/hub/backups/restore` | Yes | ReadWrite | Restore from backup |
| POST | `/api/hub/backups/upload` | Yes | ReadWrite | Upload backup file |
| DELETE | `/api/hub/backups/{id}` | Yes | ReadWrite | Delete backup |

### Permission Levels

When running in HiveHub cluster mode, permissions are enforced based on the user's tenant context:

| Permission | Description | Allowed Operations |
|------------|-------------|-------------------|
| `Admin` | Full access | All operations including reindex, admin endpoints |
| `ReadWrite` | Read and write access | Create, update, delete collections/vectors |
| `ReadOnly` | Read access only | List, search, get operations |
| `Mcp` | MCP-limited access | List, search, insert, update (no delete) |

---

## MCP Tools

See [MCP.md](./MCP.md) for complete reference.

---

## SDK Usage

### Python

```python
from vectorizer import VectorizerClient

client = VectorizerClient(
    host="localhost",
    port=15002,
    api_key="your-api-key"
)

# Search
results = client.search("query", "collection")

# Insert
client.insert_text("collection", "id", "text", metadata={})
```

### TypeScript

```typescript
import { VectorizerClient } from '@hivellm/vectorizer';

const client = new VectorizerClient({
  host: 'localhost',
  port: 15002,
  apiKey: 'your-api-key'
});

// Search
const results = await client.search('query', 'collection');

// Insert
await client.insertText('collection', 'id', 'text', {});
```

---

## Error Handling

### Standard Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | Bad Request | Invalid parameters |
| 401 | Unauthorized | Invalid/missing API key |
| 404 | Not Found | Collection/vector not found |
| 500 | Internal Server Error | Server error |

### HiveHub Cluster Mode Errors

| Status | Error Type | Description |
|--------|------------|-------------|
| 429 | QUOTA_EXCEEDED | User has exceeded their quota (collections, vectors, or storage) |
| 503 | BACKUP_DISABLED | Backup functionality is not enabled |
| 403 | ACCESS_DENIED | User does not have access to the requested collection |

#### Quota Exceeded Response

```json
{
  "error_type": "QUOTA_EXCEEDED",
  "message": "Collection quota exceeded. Please upgrade your plan or delete unused collections.",
  "status_code": 429
}
```

---

**Complete API documentation**: See individual endpoint specs in this directory  
**Maintained by**: HiveLLM Team

