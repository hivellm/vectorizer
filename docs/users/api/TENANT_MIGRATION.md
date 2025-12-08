---
title: Tenant Migration API
module: api
id: tenant-migration
order: 10
description: API for tenant data migration, export, transfer, and management
tags: [api, tenant, migration, multi-tenancy, export, transfer]
---

# Tenant Migration API

REST API endpoints for tenant lifecycle management and data migration operations in HiveHub multi-tenant mode.

## Overview

The Tenant Migration API provides tools for:
- Exporting tenant data to JSON files
- Transferring ownership between tenants
- Cloning tenant data
- Moving tenant data between storage backends
- Cleaning up tenant data
- Retrieving tenant statistics

## Base URL

```
/api/hub/tenant
```

## Authentication

All tenant migration endpoints require admin-level authentication. Include your API key or JWT token in the request headers:

```http
Authorization: Bearer <token>
X-API-Key: <api-key>
```

## Endpoints

### Get Tenant Statistics

Retrieve statistics for a specific tenant.

```http
GET /api/hub/tenant/:tenant_id/stats
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `tenant_id` | UUID | The tenant's unique identifier |

#### Response

```json
{
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "collection_count": 5,
  "collections": [
    "documents",
    "images",
    "embeddings"
  ],
  "total_vectors": 150000
}
```

#### Example

```bash
curl -X GET "http://localhost:15002/api/hub/tenant/550e8400-e29b-41d4-a716-446655440000/stats" \
  -H "Authorization: Bearer <token>"
```

---

### Migrate Tenant Data

Migrate tenant data using various strategies.

```http
POST /api/hub/tenant/:tenant_id/migrate
```

#### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `tenant_id` | UUID | Source tenant's unique identifier |

#### Request Body

```json
{
  "migration_type": "export",
  "target_tenant_id": "optional-target-uuid",
  "export_path": "./exports",
  "delete_source": false
}
```

#### Migration Types

| Type | Description | Required Fields |
|------|-------------|-----------------|
| `export` | Export all tenant data to JSON file | `export_path` (optional) |
| `transfer_ownership` | Transfer all collections to another tenant | `target_tenant_id` |
| `clone` | Clone data to a new tenant | `target_tenant_id` |
| `move_storage` | Move to different storage backend | - |

#### Response

```json
{
  "success": true,
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "migration_type": "export",
  "collections_migrated": 5,
  "vectors_migrated": 150000,
  "message": "Successfully exported tenant data",
  "export_path": "./exports/tenant_550e8400-e29b-41d4-a716-446655440000_export.json"
}
```

---

### Export Tenant Data

Export all tenant collections to a JSON file.

```bash
curl -X POST "http://localhost:15002/api/hub/tenant/550e8400-e29b-41d4-a716-446655440000/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "migration_type": "export",
    "export_path": "./backups/tenant_exports"
  }'
```

#### Export File Format

```json
{
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "exported_at": "2024-01-15T10:30:00Z",
  "collections": [
    {
      "name": "documents",
      "config": {
        "dimension": 384,
        "metric": "cosine"
      },
      "vector_count": 50000,
      "vectors": [
        {
          "id": "doc_001",
          "data": [0.1, 0.2, 0.3, ...],
          "payload": {
            "title": "Document Title",
            "category": "research"
          }
        }
      ]
    }
  ]
}
```

---

### Transfer Ownership

Transfer all collections from one tenant to another.

```bash
curl -X POST "http://localhost:15002/api/hub/tenant/source-tenant-uuid/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "migration_type": "transfer_ownership",
    "target_tenant_id": "target-tenant-uuid"
  }'
```

**Note:** This operation updates the owner field on all collections. The original tenant will no longer have access to these collections.

---

### Clone Tenant Data

Clone all data from one tenant to create copies for a new tenant.

```bash
curl -X POST "http://localhost:15002/api/hub/tenant/source-tenant-uuid/migrate" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "migration_type": "clone",
    "target_tenant_id": "new-tenant-uuid"
  }'
```

**Note:** This creates new collections owned by the target tenant. Original data remains unchanged.

---

### Cleanup Tenant Data

Permanently delete all collections and data for a tenant.

```http
POST /api/hub/tenant/cleanup
```

#### Request Body

```json
{
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "confirm": true
}
```

**Warning:** This is a destructive operation. The `confirm` flag must be set to `true`.

#### Response

```json
{
  "collections_deleted": 5,
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Successfully deleted 5 collections"
}
```

#### Example

```bash
curl -X POST "http://localhost:15002/api/hub/tenant/cleanup" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
    "confirm": true
  }'
```

---

## HiveHub Migration (Standalone to Multi-Tenant)

When transitioning from standalone Vectorizer to HiveHub Cloud multi-tenant mode, use the migration tools to assign existing collections to tenants.

### Scan Collections

Identify collections that need migration (those without an owner):

```http
GET /api/hub/migration/scan
```

#### Response

```json
{
  "collections": [
    {
      "original_name": "my_collection",
      "status": "pending",
      "vector_count": 25000,
      "has_owner": false
    }
  ],
  "total_pending": 3,
  "total_skipped": 2
}
```

### Create Migration Plan

Create a plan for migrating collections to specific tenants:

```http
POST /api/hub/migration/plan
```

```json
{
  "mappings": {
    "my_collection": "tenant-uuid-1",
    "other_collection": "tenant-uuid-2"
  },
  "default_owner": "default-tenant-uuid",
  "dry_run": true
}
```

### Execute Migration

Execute the migration plan:

```http
POST /api/hub/migration/execute
```

```json
{
  "plan_id": "migration-plan-uuid",
  "dry_run": false
}
```

---

## Error Responses

### 400 Bad Request

```json
{
  "error": "Invalid tenant UUID",
  "code": "INVALID_INPUT"
}
```

### 404 Not Found

```json
{
  "error": "No collections found for tenant",
  "code": "NOT_FOUND"
}
```

### 403 Forbidden

```json
{
  "error": "Confirmation flag must be set to true",
  "code": "CONFIRMATION_REQUIRED"
}
```

### 500 Internal Server Error

```json
{
  "error": "Failed to export tenant data",
  "code": "INTERNAL_ERROR"
}
```

---

## Best Practices

1. **Backup First**: Always export data before performing destructive operations
2. **Use Dry Run**: Test migrations with `dry_run: true` before executing
3. **Verify Ownership**: Confirm target tenant exists before transfer operations
4. **Monitor Progress**: For large migrations, monitor logs for progress updates
5. **Plan Downtime**: Consider scheduling migrations during low-traffic periods

## Rate Limits

Migration endpoints have the following rate limits:
- Export: 10 requests per minute
- Transfer: 5 requests per minute
- Cleanup: 2 requests per minute

## See Also

- [Authentication](./AUTHENTICATION.md)
- [Multi-Tenancy Guide](../getting-started/MULTI_TENANCY.md)
- [Backup & Recovery](../../BACKUP_RECOVERY.md)
