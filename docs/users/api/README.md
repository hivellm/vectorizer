---
title: API Documentation
module: api
id: api-index
order: 0
description: REST API documentation and reference
tags: [api, rest, endpoints, reference]
---

# API Documentation

Complete REST API documentation for Vectorizer.

## Guides

### [REST API Reference](./API_REFERENCE.md)
Complete reference for all REST API endpoints:
- System endpoints (health, stats)
- Collection management
- Vector operations
- Search endpoints
- Batch operations
- Qdrant-compatible endpoints
- File operations
- Discovery endpoints

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

