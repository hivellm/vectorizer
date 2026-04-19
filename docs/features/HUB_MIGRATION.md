# HiveHub Migration Guide

This guide covers migrating existing Vectorizer deployments to HiveHub cluster mode.

## Overview

HiveHub cluster mode adds multi-tenant capabilities to Vectorizer, including:
- User isolation with owner-based collection filtering
- Quota management and enforcement
- Usage tracking and reporting
- User-scoped backups

## Pre-Migration Checklist

Before migrating, ensure you have:

- [ ] Vectorizer v1.8.0 or later
- [ ] `hivehub-internal-sdk` dependency in Cargo.toml
- [ ] Backup of all existing data
- [ ] Service API key from HiveHub.Cloud
- [ ] Downtime window for migration (recommended)

## Migration Steps

### Step 1: Update Configuration

Add the hub configuration section to your `config.yml`:

```yaml
hub:
  enabled: true
  api_url: "https://api.hivehub.cloud"
  tenant_isolation: "collection"
  usage_report_interval: 300
  cache:
    enabled: true
    api_key_ttl_seconds: 300
    quota_ttl_seconds: 60
```

Set the service API key environment variable:

```bash
export HIVEHUB_SERVICE_API_KEY="your-service-api-key"
```

### Step 2: Backup Existing Data

Create a full backup before migration:

```bash
# Using the REST API
curl -X POST http://localhost:15002/backup \
  -H "Content-Type: application/json" \
  -d '{"name": "pre-migration-backup"}'
```

Or use the file system directly:

```bash
cp -r ./data/collections ./data/collections-backup
```

### Step 3: Assign Owners to Existing Collections

Existing collections don't have an `owner_id`. You need to assign them to users.

#### Option A: Assign via API (Recommended)

```bash
# Get list of collections
curl http://localhost:15002/api/collections

# Assign owner to each collection
curl -X PATCH http://localhost:15002/api/collections/my-collection \
  -H "Content-Type: application/json" \
  -H "x-hivehub-service: true" \
  -d '{"owner_id": "550e8400-e29b-41d4-a716-446655440000"}'
```

#### Option B: Bulk Assignment Script

```python
import requests
import json

# Configuration
API_URL = "http://localhost:15002"
OWNER_MAPPING = {
    "documents": "550e8400-e29b-41d4-a716-446655440000",
    "embeddings": "550e8400-e29b-41d4-a716-446655440000",
    "projects": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
}

headers = {
    "Content-Type": "application/json",
    "x-hivehub-service": "true"
}

for collection_name, owner_id in OWNER_MAPPING.items():
    response = requests.patch(
        f"{API_URL}/api/collections/{collection_name}",
        headers=headers,
        json={"owner_id": owner_id}
    )
    print(f"{collection_name}: {response.status_code}")
```

### Step 4: Verify Collection Ownership

Check that collections have correct owners:

```bash
# List collections for a specific user
curl -H "x-hivehub-service: true" \
     -H "x-hivehub-user-id: 550e8400-e29b-41d4-a716-446655440000" \
     http://localhost:15002/api/collections
```

### Step 5: Enable Hub Integration

Restart Vectorizer with hub configuration:

```bash
# Using systemd
sudo systemctl restart vectorizer

# Or directly
./target/release/vectorizer
```

Check the logs for successful initialization:

```
INFO HiveHub integration initialized
INFO Starting usage reporter with interval 300s
```

### Step 6: Verify Health

Check the health endpoint includes Hub status:

```bash
curl http://localhost:15002/health
```

Expected response:

```json
{
  "status": "healthy",
  "hub": {
    "enabled": true,
    "active": true,
    "tenant_isolation": "Collection"
  }
}
```

## Rollback Procedure

If migration fails, rollback to standalone mode:

### Step 1: Disable Hub Integration

Update `config.yml`:

```yaml
hub:
  enabled: false
```

### Step 2: Restore Backup (if needed)

```bash
rm -rf ./data/collections
cp -r ./data/collections-backup ./data/collections
```

### Step 3: Restart Service

```bash
sudo systemctl restart vectorizer
```

## Post-Migration Tasks

After successful migration:

1. **Configure Quotas**: Set up quota limits for each tenant in HiveHub.Cloud
2. **Create Backups**: Set up automated user-scoped backups
3. **Monitor Usage**: Check the Prometheus metrics endpoint for quota usage
4. **Update Client Applications**: Update API clients to include authentication headers

## Troubleshooting

### Collections Not Visible to Users

Ensure collections have `owner_id` set:

```bash
# Check collection metadata
curl http://localhost:15002/api/collections/my-collection \
  -H "x-hivehub-service: true"
```

### Quota Exceeded Errors

Check current quota usage:

```bash
curl http://localhost:15002/metrics | grep hub_quota
```

### Hub Connection Failures

Check service API key and network connectivity:

```bash
# Verify API key is set
echo $HIVEHUB_SERVICE_API_KEY

# Test connectivity
curl -v https://api.hivehub.cloud/health
```

### Usage Not Being Reported

Check usage reporter logs:

```bash
journalctl -u vectorizer | grep "Usage report"
```

## Data Model Changes

### Collection Metadata

Before migration:
```json
{
  "name": "documents",
  "config": { "dimension": 384 }
}
```

After migration:
```json
{
  "name": "user_550e8400-e29b-41d4-a716-446655440000:documents",
  "owner_id": "550e8400-e29b-41d4-a716-446655440000",
  "config": { "dimension": 384 }
}
```

### API Behavior Changes

| Feature | Standalone Mode | Cluster Mode |
|---------|----------------|--------------|
| Collection listing | All collections | Only owned collections |
| Collection creation | Any name | Prefixed with user ID |
| Search | All collections | Only owned collections |
| Backups | Global | Per-user |
| Quota enforcement | None | Per-tenant |

## Support

For migration assistance:
- Documentation: [HUB_INTEGRATION.md](./HUB_INTEGRATION.md)
- Issues: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- HiveHub Support: support@hivehub.cloud
