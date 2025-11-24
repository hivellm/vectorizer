---
title: API Documentation
module: api
id: api-index
order: 0
description: REST API and integration documentation
tags: [api, rest, endpoints, integration]
---

# API Documentation

Complete REST API documentation and integration guides.

## API Reference

### [REST API Reference](./API_REFERENCE.md)

Complete reference for all API endpoints:

**REST API:**

- System endpoints (health, stats)
- Collection management
- Vector operations
- Search endpoints (basic, intelligent, semantic, hybrid)
- Batch operations
- File operations
- Discovery endpoints

**MCP (Model Context Protocol):**

- StreamableHTTP connection (v0.9.0)
- JSON-RPC 2.0 protocol
- 38+ MCP tools (collections, vectors, search, batch, discovery, files)
- Complete tool reference with examples

**UMICP (Universal Multi-Agent Communication Protocol):**

- UMICP v0.2.1 support
- Envelope-based communication
- Tool discovery endpoint (`/umicp/discover`)
- All 38+ MCP tools available via UMICP

**Qdrant Compatibility:**

- Full REST API compatibility
- Collection management (create, get, update, delete, list)
- Point operations (upsert, retrieve, delete, count, scroll)
- Search operations (search, batch search, recommend)
- Collection aliases
- See [Qdrant Compatibility Documentation](../qdrant/) for complete guide
- Migration guide

## Advanced APIs

### [Discovery API](./DISCOVERY.md)

Intelligent content exploration:

- Multi-collection discovery
- Query expansion and refinement
- Collection filtering and scoring
- Evidence compression
- Answer plan generation

### [File Operations API](./FILE_OPERATIONS.md)

File-level operations:

- Retrieve complete file content
- List files with filtering
- Get file summaries
- Access ordered file chunks
- Project structure exploration
- Find related files

### [Replication API](./REPLICATION.md)

Master-replica replication:

- High availability setup
- Read scaling
- Replication monitoring
- Failover procedures

### [Backup and Restore API](./BACKUP_RESTORE.md)

Data protection:

- Create backups
- Restore collections
- Backup management
- Automated backup strategies

### [Workspace Management API](./WORKSPACE.md)

Multi-project workspace management:

- Add and remove workspaces
- Workspace configuration
- Multi-project indexing
- File watcher integration

### [UMICP Protocol](./UMICP.md)

Universal Multi-Agent Communication Protocol:

- Envelope-based communication
- Tool discovery
- All 38+ MCP tools accessible
- High-performance streaming

### [Admin and System API](./ADMIN.md)

Administrative endpoints:

- Server status and monitoring
- Configuration management
- Log access
- Server restart
- Indexing progress
- Prometheus metrics

## Integration

### [Integration Guide](./INTEGRATION.md)

Integrating Vectorizer with other systems:

- Web frameworks (FastAPI, Express, Axum)
- Databases (PostgreSQL, MongoDB)
- LLMs (OpenAI, LangChain)
- ETL pipelines (Airflow, Kafka)
- Monitoring (Prometheus, Grafana, Datadog)
- CI/CD (GitHub Actions, GitLab CI)
- Reverse proxy (Nginx, Caddy, Traefik)
- Authentication and load balancing

## Quick Reference

### Base URL

```
http://localhost:15002
```

### Common Endpoints

- `GET /health` - Health check
- `GET /collections` - List collections
- `POST /collections` - Create collection
- `POST /collections/{name}/search` - Search vectors
- `POST /collections/{name}/insert` - Insert vector

## Authentication

Currently, Vectorizer does not require authentication. All endpoints are publicly accessible.

## Response Format

**Success:**

```json
{
  "status": "success",
  "data": { ... }
}
```

**Error:**

```json
{
  "error": {
    "type": "error_type",
    "message": "Error message",
    "status_code": 400
  }
}
```

## Related Topics

- [Collections Guide](../collections/COLLECTIONS.md) - Using collections via API
- [Search Guide](../search/SEARCH.md) - Search operations
- [SDKs Guide](../sdks/README.md) - Client SDKs that wrap the API
