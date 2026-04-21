# HiveHub Cloud Integration

This document describes the integration between Vectorizer and HiveHub.Cloud for multi-tenant cluster mode operation.

## Overview

HiveHub integration enables Vectorizer to operate as a managed service through HiveHub.Cloud with:

- **User Isolation**: Each user's collections are isolated using owner-based filtering
- **Quota Management**: Collection count, vector count, and storage quotas enforced per tenant
- **Usage Tracking**: Automatic tracking and reporting of resource usage
- **Authentication Bypass**: Internal HiveHub requests bypass authentication via `x-hivehub-service` header

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   HiveHub.Cloud │────▶│   Vectorizer    │────▶│   Collections   │
│   (Auth/Billing)│◀────│   (Cluster)     │◀────│   (User Data)   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
        │                       │                       │
        │  x-hivehub-service    │  owner_id filter      │
        │  header bypass        │  on all queries       │
        ▼                       ▼                       ▼
   External Auth           Quota Check            Multi-Tenant
   (JWT/API Keys)          Usage Report           Isolation
```

## Configuration

### Basic Setup

Add the following to your `config.yml`:

```yaml
hub:
  enabled: true
  api_url: "https://api.hivehub.cloud"
  tenant_isolation: "collection"
  usage_report_interval: 300
```

### Environment Variables

```bash
# Required: Service API key for Vectorizer to HiveHub communication
export HIVEHUB_SERVICE_API_KEY="your-service-api-key"

# Optional: Override API URL
export HIVEHUB_API_URL="https://api.hivehub.cloud"
```

### Full Configuration Reference

```yaml
hub:
  # Enable HiveHub integration
  enabled: true

  # HiveHub API URL
  api_url: "https://api.hivehub.cloud"

  # Request timeout in seconds
  timeout_seconds: 30

  # Number of retries for failed requests
  retries: 3

  # Usage reporting interval (seconds)
  usage_report_interval: 300

  # Tenant isolation mode: none, collection, storage
  tenant_isolation: "collection"

  # Caching configuration
  cache:
    enabled: true
    api_key_ttl_seconds: 300
    quota_ttl_seconds: 60
    max_entries: 10000

  # Connection pool settings
  connection_pool:
    max_idle_per_host: 10
    pool_timeout_seconds: 30
```

## Authentication Flow

1. **External Requests**: Users authenticate with HiveHub using JWT or API keys
2. **HiveHub Validation**: HiveHub validates the token and extracts user information
3. **Internal Request**: HiveHub forwards the request to Vectorizer with `x-hivehub-service` header
4. **Bypass Authentication**: Vectorizer trusts requests with this header and processes them
5. **Quota Check**: Before resource-intensive operations, Vectorizer checks quotas via HiveHub SDK
6. **Operation Execution**: If quota allows, the operation proceeds
7. **Usage Recording**: Usage metrics are recorded and periodically synced to HiveHub

```
User ──▶ HiveHub (Auth) ──▶ Vectorizer ──▶ Collection
                │                │
                ▼                ▼
           Validate JWT    Check Quota
           Extract user_id Record Usage
```

## Multi-Tenancy Model

### Collection-Level Isolation

When `tenant_isolation: collection` is configured:

- Collections are prefixed with user ID: `user_{uuid}:{collection_name}`
- Each collection has an `owner_id` field storing the UUID
- All collection queries are filtered by `owner_id`

### Key Methods

- `list_collections_for_owner(owner_id)`: List only collections owned by user
- `get_collection_for_owner(name, owner_id)`: Get collection if owned by user
- `is_collection_owned_by(name, owner_id)`: Check ownership

### Example Collection Names

```
user_550e8400-e29b-41d4-a716-446655440000:documents
user_550e8400-e29b-41d4-a716-446655440000:embeddings
user_6ba7b810-9dad-11d1-80b4-00c04fd430c8:projects
```

## Quota Management

### Quota Types

| Type | Description | HTTP Status on Exceeded |
|------|-------------|-------------------------|
| `CollectionCount` | Maximum collections per user | 429 Too Many Requests |
| `VectorCount` | Maximum vectors per user | 429 Too Many Requests |
| `StorageBytes` | Maximum storage in bytes | 429 Too Many Requests |

### Quota Check Flow

```rust
// Before creating a collection
hub_manager.check_quota(tenant_id, QuotaType::CollectionCount, 1).await?;

// Before inserting vectors
hub_manager.check_quota(tenant_id, QuotaType::VectorCount, count).await?;
```

### Error Response

When quota is exceeded:

```json
{
  "error_type": "QUOTA_EXCEEDED",
  "message": "Collection quota exceeded. Please upgrade your plan or delete unused collections.",
  "status_code": 429
}
```

## Usage Tracking

### Tracked Metrics

| Metric | Description |
|--------|-------------|
| `vectors_inserted` | Number of vectors inserted |
| `vectors_deleted` | Number of vectors deleted |
| `storage_added` | Bytes of storage added |
| `storage_freed` | Bytes of storage freed |
| `search_count` | Number of search operations |
| `collections_created` | Collections created |
| `collections_deleted` | Collections deleted |
| `api_requests` | Total API requests |

### Reporting Interval

Usage metrics are batched and reported to HiveHub at the configured interval (default: 5 minutes).

```yaml
hub:
  usage_report_interval: 300  # 5 minutes
```

## API Endpoints

All standard Vectorizer endpoints work with HiveHub integration. Key behaviors:

### POST /collections
- Checks collection count quota before creation
- Records collection creation in usage metrics
- Sets `owner_id` on the new collection

### POST /collections/{name}/points
- Checks vector count quota before insertion
- Records vector insertions and storage usage

### GET /collections
- Returns only collections owned by the authenticated user

### DELETE /collections/{name}
- Records collection deletion in usage metrics

## Internal Service Headers

### x-hivehub-service

The `x-hivehub-service` header allows internal HiveHub requests to bypass authentication:

```bash
# Internal request from HiveHub
curl -H "x-hivehub-service: true" \
     http://localhost:15002/api/collections
```

This header should only be used by trusted internal services. The Vectorizer does not expose this externally.

### x-hivehub-user-id

For internal requests that need tenant scoping, use the `x-hivehub-user-id` header to specify the user:

```bash
# Internal request with user context
curl -H "x-hivehub-service: true" \
     -H "x-hivehub-user-id: 550e8400-e29b-41d4-a716-446655440000" \
     http://localhost:15002/api/collections/my-collection/search/text \
     -d '{"query": "search term"}'
```

When both headers are present, the Vectorizer:
1. Bypasses API key authentication (trusts the internal service)
2. Creates a tenant context with the provided user ID
3. Filters collection access to only those owned by that user

## Error Handling

### Hub Connection Failures

If connection to HiveHub fails:
- Warning is logged
- Operation continues (fail-open for availability)
- Retries are attempted based on configuration

### Graceful Degradation

```rust
match hub_manager.check_quota(...).await {
    Ok(allowed) => { /* enforce quota */ },
    Err(e) => {
        warn!("Failed to check quota: {}", e);
        // Continue - Hub handles actual enforcement
    }
}
```

## Troubleshooting

### Common Issues

#### 1. "Hub integration disabled" logs
- Check that `hub.enabled: true` in config
- Verify `HIVEHUB_SERVICE_API_KEY` is set

#### 2. Quota checks failing
- Check network connectivity to HiveHub API
- Verify API URL is correct
- Check service API key validity

#### 3. Usage not being reported
- Check `usage_report_interval` configuration
- Look for errors in logs during sync
- Verify HiveHub API is accepting reports

### Debug Logging

Enable debug logging to see HiveHub integration details:

```yaml
logging:
  level: "debug"
```

Look for log messages prefixed with:
- `HiveHub integration initialized`
- `Quota check for tenant`
- `Usage report sent`

## SDK Reference

The integration uses `hivehub-internal-sdk` v1.0.0:

```toml
[dependencies]
hivehub-internal-sdk = "1.0.0"
```

Key SDK types:
- `HiveHubCloudClient`: Main client for HiveHub API
- `QuotaCheckRequest`: Request to check quota
- `QuotaCheckResponse`: Response with `allowed`, `remaining`, `limit`
- `UsageReportRequest`: Request to report usage metrics

## Operation Logging and Tracking

HiveHub integration includes comprehensive operation logging and tracking for analytics, auditing, and monitoring purposes.

### Tracked Operations

The MCP Gateway automatically logs all operations performed through the MCP protocol:

| Operation Type | Description | Requires Write Permission |
|----------------|-------------|---------------------------|
| `ListCollections` | List all collections | No |
| `CreateCollection` | Create a new collection | Yes |
| `DeleteCollection` | Delete a collection | Yes |
| `GetCollectionInfo` | Get collection metadata | No |
| `Insert` | Insert vectors/documents | Yes |
| `Search` | Search operations (all types) | No |
| `GetVector` | Get vector by ID | No |
| `UpdateVector` | Update vector data | Yes |
| `DeleteVector` | Delete vectors | Yes |
| `GraphOperation` | Graph-related operations | No |
| `ClusterOperation` | Cluster management | No |
| `FileOperation` | File discovery operations | No |
| `Discovery` | Discovery features | No |

### Operation Log Structure

Each operation is logged with the following information:

```rust
{
    "operation_id": "uuid-v4",           // Unique operation identifier
    "tenant_id": "user_123",             // Tenant/user ID
    "operation": "search",               // Tool/operation name
    "operation_type": "search",          // Categorized operation type
    "collection": "documents",           // Collection name (if applicable)
    "timestamp": 1234567890,             // Unix timestamp (milliseconds)
    "duration_ms": 50,                   // Operation duration
    "success": true,                     // Whether operation succeeded
    "error": null,                       // Error message (if failed)
    "metadata": {                        // Additional operation metadata
        "query": "search term",
        "limit": 10
    }
}
```

### Automatic Log Flushing

Operation logs are buffered in memory and automatically flushed to HiveHub Cloud when:

- Buffer reaches 1,000 entries (default)
- Periodic flush interval triggers (every 5 minutes by default)
- Server gracefully shuts down

```rust
// Manual flush (if needed)
mcp_gateway.flush_logs().await?;
```

### Cloud Logging Endpoint

Logs are sent to the HiveHub Cloud logging endpoint:

```
POST https://api.hivehub.cloud/api/v1/vectorizer/logs
```

Request format:
```json
{
    "service_id": "vec-abc123",
    "logs": [
        {
            "operation_id": "uuid",
            "tenant_id": "user_123",
            "operation": "search",
            "operation_type": "search",
            "collection": "docs",
            "timestamp": 1234567890,
            "duration_ms": 50,
            "success": true,
            "error": null,
            "metadata": {}
        }
    ]
}
```

Response format:
```json
{
    "accepted": true,
    "processed": 10,
    "error": null
}
```

### Usage Metrics Tracking

In addition to operation logs, the system tracks aggregate usage metrics per tenant:

```rust
UsageMetrics {
    vectors_inserted: 1000,      // Total vectors inserted
    vectors_deleted: 50,         // Total vectors deleted
    search_count: 500,           // Number of search operations
    storage_added: 1048576,      // Bytes added
    storage_freed: 102400,       // Bytes freed
    collections_created: 5,      // Collections created
    collections_deleted: 1,      // Collections deleted
    api_requests: 5000,          // Total API requests
}
```

These metrics are periodically synced to HiveHub Cloud for billing and quota management.

### Authorization and Quota Checks

Before executing operations, the MCP Gateway performs authorization and quota checks:

```rust
// Check if tenant can perform operation
mcp_gateway.authorize_operation(
    &tenant,
    McpOperationType::CreateCollection,
    Some("new_collection")
).await?;

// Check quota for resource-intensive operations
hub_manager.check_quota(
    tenant_id,
    QuotaType::VectorCount,
    vector_count
).await?;
```

### Log Buffer Configuration

Configure the log buffer size via code (default: 1,000 entries):

```rust
let mcp_gateway = McpHubGateway::with_buffer_size(
    hub_manager,
    5000  // Custom buffer size
);
```

### Error Handling

Operation logging is designed to be non-blocking:

- If cloud logging fails, operations continue normally
- Failed log transmissions are logged but don't impact user operations
- Logs are cleared after flush attempts to prevent unbounded memory growth

### Monitoring and Analytics

Operation logs enable:

- **Usage Analytics**: Track which operations are most common
- **Performance Monitoring**: Identify slow operations via `duration_ms`
- **Error Tracking**: Monitor operation failures and error patterns
- **Tenant Activity**: See per-tenant operation patterns
- **Audit Trail**: Complete audit log of all operations

### Example: Querying Operation Logs

While logs are sent to HiveHub Cloud, you can query them via the HiveHub dashboard or API:

```bash
# Get operation logs for a tenant
curl https://api.hivehub.cloud/api/v1/tenants/user_123/logs \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "start_time": "2024-01-01T00:00:00Z",
    "end_time": "2024-01-02T00:00:00Z",
    "operation_type": "search"
  }'
```

## User-Scoped Backup System

HiveHub cluster mode includes a user-scoped backup system that allows creating, downloading, and restoring backups isolated per user.

### Backup Features

- **User Isolation**: Each user's backups are stored in separate directories
- **Compression**: Backups are compressed with gzip (configurable)
- **Checksum**: SHA-256 verification for data integrity
- **Retention**: Automatic cleanup of old backups (configurable)
- **Format**: JSON-based format with all collection data and metadata

### Backup API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/hub/backups?user_id=<uuid>` | List all backups for a user |
| POST | `/api/hub/backups` | Create a new backup |
| GET | `/api/hub/backups/:backup_id?user_id=<uuid>` | Get backup metadata |
| GET | `/api/hub/backups/:backup_id/download?user_id=<uuid>` | Download backup file |
| POST | `/api/hub/backups/restore` | Restore from backup |
| POST | `/api/hub/backups/upload?user_id=<uuid>` | Upload a backup file |
| DELETE | `/api/hub/backups/:backup_id?user_id=<uuid>` | Delete a backup |

### Create Backup

```bash
curl -X POST http://localhost:15002/api/hub/backups \
  -H "Content-Type: application/json" \
  -H "x-hivehub-service: true" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "daily_backup",
    "description": "Daily automated backup",
    "collections": ["documents", "embeddings"]
  }'
```

Response:
```json
{
  "success": true,
  "message": "Backup 'daily_backup' created successfully",
  "backup": {
    "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "daily_backup",
    "created_at": "2024-01-15T10:30:00Z",
    "collections": ["documents", "embeddings"],
    "vector_count": 5000,
    "size_bytes": 1048576,
    "compressed": true,
    "checksum": "abc123..."
  }
}
```

### Download Backup

```bash
curl -O http://localhost:15002/api/hub/backups/a1b2c3d4-e5f6-7890-abcd-ef1234567890/download?user_id=550e8400-e29b-41d4-a716-446655440000 \
  -H "x-hivehub-service: true"
```

### Restore Backup

```bash
curl -X POST http://localhost:15002/api/hub/backups/restore \
  -H "Content-Type: application/json" \
  -H "x-hivehub-service: true" \
  -d '{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "backup_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "overwrite": true
  }'
```

Response:
```json
{
  "success": true,
  "message": "Restored 2 collections with 5000 vectors",
  "restore_result": {
    "backup_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "collections_restored": ["documents", "embeddings"],
    "collections_skipped": [],
    "vectors_restored": 5000,
    "errors": []
  }
}
```

### Backup Configuration

The backup system uses default configuration but can be customized:

```rust
BackupConfig {
    backup_dir: PathBuf::from("./data/hub_backups"),
    max_backups_per_user: 10,     // Keep up to 10 backups per user
    max_backup_age_hours: 0,       // 0 = unlimited retention
    compression_enabled: true,     // Enable gzip compression
    compression_level: 6,          // 1-9, higher = better compression
}
```

### Backup File Structure

```
data/hub_backups/
├── 550e8400-e29b-41d4-a716-446655440000/   # User 1
│   ├── a1b2c3d4-e5f6-7890-abcd-ef1234567890.backup.gz
│   ├── a1b2c3d4-e5f6-7890-abcd-ef1234567890.meta.json
│   └── ...
└── 6ba7b810-9dad-11d1-80b4-00c04fd430c8/   # User 2
    ├── b2c3d4e5-f6a7-8901-bcde-f12345678901.backup.gz
    └── b2c3d4e5-f6a7-8901-bcde-f12345678901.meta.json
```
