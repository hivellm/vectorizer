# Core Specification: HiveHub Cluster Mode Multi-Tenant System

## ADDED Requirements

### Requirement: HiveHub Cloud SDK Integration

The system SHALL integrate the official `hivehub-cloud-internal-sdk` for HiveHub Cloud API communication.

The SDK integration MUST provide:
- API key validation and tenant information retrieval
- Quota management (fetch, validate, update)
- Automatic caching with configurable TTL
- Retry logic with exponential backoff
- Connection pooling for performance
- Type-safe request/response models

#### Scenario: API Key Validation Success

Given a valid API key "hh_test_abc123"
When the system calls `client.auth().validate_api_key("hh_test_abc123")`
Then the SDK SHALL return tenant information including:
- `tenant_id`: Unique identifier for the tenant
- `tenant_name`: Display name
- `quotas`: Storage and rate limit quotas
- `permissions`: List of allowed operations
And the SDK SHALL automatically cache this information for 5 minutes

#### Scenario: API Key Validation Failure

Given an invalid API key "hh_invalid_key"
When the system calls `client.auth().validate_api_key("hh_invalid_key")`
Then the SDK SHALL return `HiveHubError::InvalidApiKey`
And the system SHALL reject the request with 401 Unauthorized
And the SDK SHALL NOT cache the failed result

#### Scenario: HiveHub API Unavailable

Given the HiveHub API is temporarily unavailable
When the system calls `client.auth().validate_api_key()`
Then the SDK SHALL automatically retry up to 3 times with exponential backoff
And if all retries fail, the SDK SHALL use cached data if available
And if no cache exists, the SDK SHALL return `HiveHubError::ServiceUnavailable`
And the system SHALL return 503 Service Unavailable to client

---

### Requirement: Multi-Tenant Data Isolation

The system SHALL implement complete data isolation between tenants at the storage layer.

All tenant data MUST be isolated using tenant-prefixed namespacing. No data SHALL be accessible across tenant boundaries without explicit authorization.

#### Scenario: Collection Creation with Tenant Isolation

Given a tenant with ID "tenant_alice"
When the tenant creates a collection named "documents"
Then the system SHALL store it as "tenant_alice:documents"
And the collection SHALL only be accessible by "tenant_alice"
And other tenants SHALL NOT see this collection in list operations

#### Scenario: Cross-Tenant Access Attempt

Given tenant "tenant_alice" owns collection "tenant_alice:documents"
When tenant "tenant_bob" attempts to access "tenant_alice:documents"
Then the system SHALL reject the request with 403 Forbidden
And SHALL NOT reveal whether the collection exists
And SHALL log the access attempt for security audit

#### Scenario: Tenant Data Deletion

Given a tenant "tenant_charlie" with multiple collections
When the tenant is deleted from HiveHub
Then the system SHALL delete all collections prefixed with "tenant_charlie:"
And SHALL delete all associated storage files
And SHALL remove all cached data
And SHALL free up quota allocation

---

### Requirement: Quota Management System

The system SHALL implement comprehensive quota tracking and enforcement.

Quotas MUST be enforced in real-time with minimal performance overhead (<10ms). The system SHALL prevent operations that would exceed quota limits.

#### Scenario: Storage Quota Enforcement

Given a tenant with storage quota of 1GB
And the tenant currently uses 950MB
When the tenant attempts to insert vectors totaling 100MB
Then the system SHALL reject the operation with 429 Too Many Requests
And SHALL return error message: "Storage quota exceeded"
And SHALL include current usage and limit in response headers

#### Scenario: Rate Limit Enforcement

Given a tenant with rate limit of 1000 requests per minute
And the tenant has made 1000 requests in the current minute
When the tenant makes another request
Then the system SHALL reject with 429 Too Many Requests
And SHALL include "Retry-After" header with seconds until reset
And SHALL include current rate limit usage in response headers

#### Scenario: Quota Check Optimization

Given quota data cached from HiveHub
When a request requires quota validation
Then the system SHALL use cached quota if cache is fresh (<60 seconds)
And SHALL NOT make unnecessary HiveHub API calls
And SHALL refresh cache asynchronously when near expiration

---

### Requirement: Tenant Context Management

The system SHALL maintain tenant context throughout the request lifecycle.

Every authenticated request MUST carry tenant context. All operations MUST be scoped to the authenticated tenant.

#### Scenario: Tenant Context Extraction

Given an API request with header "Authorization: Bearer hh_test_abc123"
When the authentication middleware processes the request
Then the system SHALL extract the tenant ID from the validated key
And SHALL attach tenant context to the request
And SHALL make tenant context available to all downstream operations

#### Scenario: Tenant Context Validation

Given a request with tenant context for "tenant_alice"
When executing a collection operation
Then the system SHALL automatically scope the operation to "tenant_alice"
And SHALL NOT require explicit tenant filtering in business logic
And SHALL prevent accidental cross-tenant data access

---

### Requirement: Storage Usage Tracking

The system SHALL accurately track storage usage per tenant.

Storage tracking MUST be real-time and accurate within 1% margin. Usage data SHALL be reported to HiveHub periodically.

#### Scenario: Vector Insertion Storage Tracking

Given a tenant with current storage usage of 500MB
When the tenant inserts vectors totaling 50MB
Then the system SHALL update usage to 550MB
And SHALL persist the updated usage
And SHALL verify against quota before committing

#### Scenario: Vector Deletion Storage Tracking

Given a tenant with current storage usage of 600MB
When the tenant deletes vectors totaling 100MB
Then the system SHALL update usage to 500MB
And SHALL reclaim storage space
And SHALL update quota availability

#### Scenario: Usage Reporting to HiveHub

Given multiple tenants with storage usage
When the usage reporting interval elapses (300 seconds)
Then the system SHALL send usage report to HiveHub containing:
- `tenant_id`: Tenant identifier
- `storage_bytes`: Current storage usage
- `vector_count`: Number of vectors stored
- `collection_count`: Number of collections
- `timestamp`: Report timestamp
And SHALL handle reporting failures gracefully

---

### Requirement: Cache Layer for HiveHub Data

The system SHALL implement an efficient caching layer for HiveHub API responses.

Caching MUST reduce HiveHub API load by >90%. Cache invalidation SHALL be automatic and configurable.

#### Scenario: API Key Cache Hit

Given an API key "hh_test_abc123" cached with tenant data
When a request arrives with this API key
Then the system SHALL retrieve tenant data from cache
And SHALL NOT call HiveHub API
And SHALL complete auth in <5ms

#### Scenario: API Key Cache Miss

Given an API key "hh_test_new456" not in cache
When a request arrives with this API key
Then the system SHALL call HiveHub API for validation
And SHALL cache the response for 5 minutes
And SHALL use cached data for subsequent requests

#### Scenario: Cache Invalidation on API Key Revocation

Given an API key "hh_test_revoked" in cache
When HiveHub sends revocation notification
Then the system SHALL immediately remove the key from cache
And SHALL reject subsequent requests with this key
And SHALL not use stale cached data

---

## MODIFIED Requirements

### Requirement: Collection Management (Modified for Multi-Tenant)

The system SHALL modify collection management to support tenant isolation.

MODIFICATION: All collection operations now require tenant context and use tenant-prefixed naming.

#### Scenario: List Collections with Tenant Filtering

Given tenant "tenant_alice" with collections ["docs", "images"]
And tenant "tenant_bob" with collections ["videos", "audio"]
When "tenant_alice" lists collections
Then the system SHALL return only ["docs", "images"]
And SHALL NOT include "tenant_bob" collections
And SHALL automatically filter by tenant prefix "tenant_alice:"

---

### Requirement: Vector Operations (Modified for Multi-Tenant)

The system SHALL modify all vector operations to include tenant scoping.

MODIFICATION: Insert, update, delete, and search operations are automatically scoped to tenant context.

#### Scenario: Vector Search with Tenant Isolation

Given "tenant_alice" has vectors in collection "documents"
And "tenant_bob" has vectors in collection "documents"
When "tenant_alice" searches in collection "documents"
Then the system SHALL search only in "tenant_alice:documents"
And SHALL NOT return results from "tenant_bob:documents"
And SHALL enforce rate limits specific to "tenant_alice"

---

## Configuration Schema

```yaml
cluster:
  # Enable cluster mode (default: false)
  enabled: boolean
  
  # HiveHub Cloud SDK configuration
  hivehub:
    api_url: string  # HiveHub Cloud API URL
    service_api_key: string  # Service authentication key
    timeout_seconds: integer  # Request timeout
    retries: integer  # Max retry attempts
    
    # SDK Cache configuration
    cache:
      enabled: boolean
      ttl_seconds: integer  # API key cache TTL
      quota_ttl_seconds: integer  # Quota cache TTL
      max_entries: integer  # Max cache size
    
    # Connection pool
    connection_pool:
      max_idle_per_host: integer
      pool_timeout_seconds: integer
  
  # Usage reporting interval (seconds)
  usage_report_interval: integer

auth:
  # Require authentication on all endpoints
  require_authentication: boolean
  
  # API key header name
  api_key_header: string  # Default: "Authorization"
  
  # API key prefix
  api_key_prefix: string  # Default: "Bearer "

rate_limiting:
  # Default rate limits (if not specified by HiveHub)
  default_requests_per_minute: integer
  default_requests_per_hour: integer
  default_requests_per_day: integer
```

## SDK Usage Examples

### Initialize HiveHub Client

```rust
use hivehub_cloud_internal_sdk::HiveHubCloudClient;

// Initialize from environment
let client = HiveHubCloudClient::from_env()?;

// Or with explicit configuration
let client = HiveHubCloudClient::new(
    "svc_vectorizer_abc123...",
    "https://api.hivehub.cloud".to_string(),
)?;
```

### Validate API Key

```rust
// In authentication middleware
let tenant_info = client.auth()
    .validate_api_key(&api_key)
    .await?;

// Extract tenant context
let tenant_context = TenantContext {
    tenant_id: tenant_info.tenant_id,
    permissions: tenant_info.permissions,
};
```

### Check and Update Quota

```rust
// Check quota before operation
let quota = client.vectorizer()
    .get_user_quota(&tenant_id)
    .await?;

if !quota.storage.can_insert {
    return Err(VectorizerError::QuotaExceeded);
}

// Update usage after operation
client.vectorizer()
    .update_usage(&tenant_id, &UpdateUsageRequest {
        vector_count: new_count,
        storage_bytes: new_storage,
    })
    .await?;
```

### Validate Collection Ownership

```rust
// Validate collection belongs to tenant
let validation = client.vectorizer()
    .validate_collection(&collection_id, &tenant_id)
    .await?;

if !validation.valid {
    return Err(VectorizerError::Forbidden);
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 401 | Unauthorized | API key missing or invalid |
| 403 | Forbidden | Insufficient permissions for operation |
| 429 | Too Many Requests | Rate limit or quota exceeded |
| 503 | Service Unavailable | HiveHub API unavailable and no cache available |

## Performance Targets

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| API Key Validation (cached) | <5ms | 99th percentile |
| API Key Validation (uncached) | <100ms | Includes HiveHub API call |
| Quota Check (cached) | <1ms | In-memory lookup |
| Quota Check (uncached) | <50ms | Includes HiveHub API call |
| Rate Limit Check | <2ms | Token bucket algorithm |
| Storage Usage Update | <10ms | Async persistence |

