# Multi-Tenancy

Vectorizer supports multi-tenant deployments with tenant isolation, resource quotas, and access control.

## Overview

Multi-tenancy features include:
- **Tenant Isolation**: Separate data and namespaces per tenant
- **Resource Quotas**: Memory, storage, and QPS limits
- **Access Control**: Tenant-specific permissions
- **Usage Tracking**: Monitor resource consumption

## Enabling Multi-Tenancy

```yaml
# vectorizer.yaml
multi_tenancy:
  enabled: true
  default_quotas:
    max_memory_bytes: 1073741824     # 1 GB
    max_collections: 10
    max_vectors_per_collection: 1000000
    max_qps: 100
    max_storage_bytes: 10737418240   # 10 GB
```

## Tenant Management

### Create Tenant

```http
POST /api/v1/tenants
Content-Type: application/json
Authorization: Bearer <admin-token>

{
  "tenant_id": "tenant-acme",
  "name": "ACME Corporation",
  "quotas": {
    "max_collections": 20,
    "max_vectors_per_collection": 5000000,
    "max_memory_bytes": 5368709120,
    "max_qps": 500
  },
  "metadata": {
    "plan": "enterprise",
    "contact": "admin@acme.com"
  }
}
```

### List Tenants

```http
GET /api/v1/tenants
Authorization: Bearer <admin-token>
```

### Get Tenant Info

```http
GET /api/v1/tenants/{tenant_id}
Authorization: Bearer <admin-token>
```

Response:

```json
{
  "tenant_id": "tenant-acme",
  "name": "ACME Corporation",
  "quotas": {
    "max_collections": 20,
    "max_vectors_per_collection": 5000000,
    "max_memory_bytes": 5368709120,
    "max_qps": 500
  },
  "usage": {
    "collections": 5,
    "total_vectors": 1250000,
    "memory_bytes": 2147483648,
    "current_qps": 45
  },
  "created_at": "2024-01-15T10:30:00Z"
}
```

### Update Tenant Quotas

```http
PATCH /api/v1/tenants/{tenant_id}
Content-Type: application/json
Authorization: Bearer <admin-token>

{
  "quotas": {
    "max_collections": 50,
    "max_qps": 1000
  }
}
```

### Delete Tenant

```http
DELETE /api/v1/tenants/{tenant_id}
Authorization: Bearer <admin-token>
```

## Using Tenants

### Tenant Header

All API calls should include the tenant header:

```http
GET /api/v1/collections
X-Tenant-ID: tenant-acme
Authorization: Bearer <token>
```

### Collection Namespacing

Collections are automatically namespaced by tenant:

```http
POST /api/v1/collections
X-Tenant-ID: tenant-acme
Content-Type: application/json

{
  "name": "documents",
  "dimension": 384
}
```

The collection is stored as `tenant-acme/documents` internally, but accessed as `documents` by the tenant.

### Cross-Tenant Access (Admin Only)

Admins can access collections across tenants:

```http
GET /api/v1/admin/tenants/{tenant_id}/collections
Authorization: Bearer <admin-token>
```

## Resource Quotas

### Quota Types

| Quota | Description | Default |
|-------|-------------|---------|
| `max_memory_bytes` | Maximum memory usage | 1 GB |
| `max_collections` | Maximum number of collections | 10 |
| `max_vectors_per_collection` | Maximum vectors per collection | 1,000,000 |
| `max_qps` | Maximum queries per second | 100 |
| `max_storage_bytes` | Maximum storage usage | 10 GB |

### Quota Enforcement

When quotas are exceeded:

```json
{
  "error": "Quota exceeded",
  "quota_type": "max_collections",
  "current": 10,
  "limit": 10,
  "tenant_id": "tenant-acme"
}
```

### Check Usage

```http
GET /api/v1/tenants/{tenant_id}/usage
Authorization: Bearer <admin-token>
```

Response:

```json
{
  "tenant_id": "tenant-acme",
  "usage": {
    "memory_bytes": 2147483648,
    "memory_percent": 40.0,
    "collections": 5,
    "collections_percent": 25.0,
    "vectors": 1250000,
    "storage_bytes": 3221225472,
    "storage_percent": 30.0,
    "current_qps": 45,
    "qps_percent": 9.0
  },
  "quotas": {
    "max_memory_bytes": 5368709120,
    "max_collections": 20,
    "max_vectors_per_collection": 5000000,
    "max_qps": 500,
    "max_storage_bytes": 10737418240
  }
}
```

## SDK Usage

### Python

```python
from vectorizer_sdk import VectorizerClient

# Create tenant-scoped client
client = VectorizerClient(
    "http://localhost:15002",
    tenant_id="tenant-acme",
    api_key="vz_xxxxx"
)

# All operations are scoped to the tenant
await client.create_collection("documents", dimension=384)
await client.insert_vectors("documents", vectors)
results = await client.search("documents", query_vector)
```

### TypeScript

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

const client = new VectorizerClient("http://localhost:15002", {
  tenantId: "tenant-acme",
  apiKey: "vz_xxxxx"
});

// All operations are scoped to the tenant
await client.createCollection("documents", { dimension: 384 });
await client.insertVectors("documents", vectors);
const results = await client.search("documents", queryVector);
```

## Tenant Plans

### Configuring Plans

```yaml
multi_tenancy:
  plans:
    free:
      max_collections: 3
      max_vectors_per_collection: 10000
      max_memory_bytes: 104857600  # 100 MB
      max_qps: 10
    
    starter:
      max_collections: 10
      max_vectors_per_collection: 100000
      max_memory_bytes: 1073741824  # 1 GB
      max_qps: 50
    
    professional:
      max_collections: 50
      max_vectors_per_collection: 1000000
      max_memory_bytes: 5368709120  # 5 GB
      max_qps: 200
    
    enterprise:
      max_collections: null  # unlimited
      max_vectors_per_collection: null
      max_memory_bytes: null
      max_qps: null
```

### Assigning Plans

```http
PATCH /api/v1/tenants/{tenant_id}
Content-Type: application/json

{
  "plan": "professional"
}
```

## Isolation Strategies

### Namespace Isolation (Default)

- Collections prefixed with tenant ID
- Single database instance
- Efficient resource sharing

### Database Isolation

For maximum isolation:

```yaml
multi_tenancy:
  isolation: "database"
  data_directory: "/data/tenants/{tenant_id}"
```

Each tenant gets separate storage.

## Monitoring

### Tenant Metrics

```http
GET /api/v1/admin/metrics/tenants
Authorization: Bearer <admin-token>
```

Response:

```json
{
  "tenants": [
    {
      "tenant_id": "tenant-acme",
      "qps_current": 45,
      "qps_avg_1h": 38,
      "memory_bytes": 2147483648,
      "collections": 5,
      "vectors_total": 1250000
    }
  ]
}
```

### Prometheus Metrics

```
vectorizer_tenant_qps{tenant_id="tenant-acme"} 45
vectorizer_tenant_memory_bytes{tenant_id="tenant-acme"} 2147483648
vectorizer_tenant_collections{tenant_id="tenant-acme"} 5
vectorizer_tenant_vectors_total{tenant_id="tenant-acme"} 1250000
```

## Best Practices

1. **Set appropriate quotas** based on expected usage
2. **Monitor usage regularly** to prevent quota exhaustion
3. **Use meaningful tenant IDs** (e.g., company names, account IDs)
4. **Implement billing integration** using usage metrics
5. **Plan for growth** with flexible quota adjustments

## Related Topics

- [Authentication](../api/AUTHENTICATION.md) - Authentication and authorization
- [Monitoring](../operations/MONITORING.md) - System monitoring
- [Configuration](../configuration/CONFIGURATION.md) - Server configuration

