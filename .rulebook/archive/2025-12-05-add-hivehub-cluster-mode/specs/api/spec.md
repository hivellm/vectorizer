# API Specification: HiveHub Cluster Mode REST API

## ADDED Requirements

### Requirement: API Authentication Middleware

The system SHALL implement authentication middleware for all API endpoints.

All API endpoints MUST validate the API key and establish tenant context before processing requests. Unauthenticated requests SHALL be rejected with 401 Unauthorized.

#### Scenario: Successful API Authentication

Given a valid API key "hh_live_xyz789"
When a client makes request to "POST /api/v1/collections" with header "Authorization: Bearer hh_live_xyz789"
Then the middleware SHALL validate the API key with HiveHub
And SHALL extract tenant ID "tenant_xyz"
And SHALL attach tenant context to the request
And SHALL allow the request to proceed

#### Scenario: Missing API Key

Given a client request without "Authorization" header
When the request arrives at any protected endpoint
Then the middleware SHALL reject with 401 Unauthorized
And SHALL return JSON error: `{"error": "Missing API key", "code": "AUTH_MISSING"}`
And SHALL NOT process the request

#### Scenario: Invalid API Key Format

Given a malformed API key "not-a-valid-key"
When a client makes request with "Authorization: Bearer not-a-valid-key"
Then the middleware SHALL reject with 401 Unauthorized
And SHALL return JSON error: `{"error": "Invalid API key format", "code": "AUTH_INVALID_FORMAT"}`
And SHALL NOT call HiveHub API

---

### Requirement: Cluster Health Endpoint

The system SHALL provide a cluster health endpoint for monitoring.

The endpoint MUST be accessible only with admin-level API keys. It SHALL return comprehensive health information about the cluster.

#### Scenario: Cluster Health Check (Admin)

Given an admin API key "hh_admin_master123"
When the client requests "GET /api/v1/cluster/health"
Then the system SHALL return 200 OK with:
```json
{
  "status": "healthy",
  "cluster_mode": true,
  "hivehub_connection": "connected",
  "tenant_count": 42,
  "total_storage_gb": 156.7,
  "uptime_seconds": 86400,
  "version": "1.8.0"
}
```

#### Scenario: Cluster Health Check (Non-Admin)

Given a non-admin API key "hh_test_user456"
When the client requests "GET /api/v1/cluster/health"
Then the system SHALL reject with 403 Forbidden
And SHALL return error: `{"error": "Admin access required", "code": "FORBIDDEN"}`

---

### Requirement: Tenant Usage Statistics Endpoint

The system SHALL provide usage statistics for tenants.

Tenant owners SHALL access their own usage data. Admin keys SHALL access any tenant's data.

#### Scenario: Tenant Requests Own Usage

Given a tenant API key for "tenant_alice"
When the client requests "GET /api/v1/cluster/usage"
Then the system SHALL return 200 OK with:
```json
{
  "tenant_id": "tenant_alice",
  "storage": {
    "used_bytes": 524288000,
    "quota_bytes": 1073741824,
    "usage_percent": 48.8
  },
  "rate_limits": {
    "requests_per_minute": {
      "used": 156,
      "limit": 1000,
      "reset_in_seconds": 42
    },
    "requests_per_hour": {
      "used": 4521,
      "limit": 10000,
      "reset_in_seconds": 1842
    }
  },
  "collections": 12,
  "vectors": 145023,
  "period_start": "2025-12-01T00:00:00Z",
  "period_end": "2025-12-31T23:59:59Z"
}
```

#### Scenario: Admin Requests Any Tenant Usage

Given an admin API key
When the client requests "GET /api/v1/cluster/usage?tenant_id=tenant_bob"
Then the system SHALL return usage data for "tenant_bob"
And SHALL include admin-only fields:
- `last_request_at`: Timestamp of last request
- `created_at`: Tenant creation timestamp
- `api_keys_count`: Number of active API keys

---

### Requirement: API Key Validation Endpoint

The system SHALL provide an endpoint to validate API keys.

This endpoint MUST be publicly accessible (no auth required). It SHALL return validity status and key metadata.

#### Scenario: Valid API Key Check

Given a valid API key "hh_test_check456"
When the client requests "POST /api/v1/cluster/keys/validate" with body:
```json
{
  "api_key": "hh_test_check456"
}
```
Then the system SHALL return 200 OK with:
```json
{
  "valid": true,
  "tenant_id": "tenant_check",
  "permissions": ["READ", "WRITE"],
  "expires_at": null
}
```

#### Scenario: Invalid API Key Check

Given an invalid API key "hh_fake_invalid"
When the client requests validation
Then the system SHALL return 200 OK with:
```json
{
  "valid": false,
  "error": "API key not found or revoked"
}
```

---

## MODIFIED Requirements

### Requirement: Collection Endpoints (Modified for Multi-Tenant)

The system SHALL modify all collection endpoints to enforce tenant scoping.

MODIFICATION: All collection operations automatically scoped to authenticated tenant. Collection names in requests are relative to tenant namespace.

#### Scenario: Create Collection in Multi-Tenant Mode

Given an authenticated tenant "tenant_alice"
When the client requests "POST /api/v1/collections" with:
```json
{
  "name": "documents",
  "dimension": 768,
  "metric": "cosine"
}
```
Then the system SHALL create collection "tenant_alice:documents"
And SHALL return 201 Created with:
```json
{
  "name": "documents",
  "full_name": "tenant_alice:documents",
  "dimension": 768,
  "metric": "cosine"
}
```
And SHALL enforce storage quota before creation

#### Scenario: List Collections in Multi-Tenant Mode

Given tenant "tenant_alice" with collections ["docs", "images", "videos"]
When the client requests "GET /api/v1/collections"
Then the system SHALL return only collections owned by "tenant_alice"
And SHALL omit the tenant prefix in response:
```json
{
  "collections": ["docs", "images", "videos"]
}
```

---

### Requirement: Vector Endpoints (Modified for Multi-Tenant)

The system SHALL modify vector endpoints to include tenant scoping and quota enforcement.

MODIFICATION: All vector operations validate quotas and enforce tenant isolation.

#### Scenario: Insert Vectors with Quota Check

Given tenant "tenant_alice" with 900MB used of 1GB quota
When the client requests "POST /api/v1/collections/documents/vectors" with 150MB of vectors
Then the system SHALL reject with 429 Too Many Requests
And SHALL return:
```json
{
  "error": "Storage quota exceeded",
  "code": "QUOTA_EXCEEDED",
  "usage": {
    "current_bytes": 943718400,
    "quota_bytes": 1073741824,
    "requested_bytes": 157286400,
    "available_bytes": 130023424
  }
}
```

#### Scenario: Search Vectors with Rate Limit

Given tenant "tenant_bob" has exceeded rate limit
When the client requests "POST /api/v1/collections/documents/search"
Then the system SHALL reject with 429 Too Many Requests
And SHALL include headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1701360000
Retry-After: 42
```

---

### Requirement: Admin Endpoints (Modified for Cluster Mode)

The system SHALL add cluster-specific admin endpoints.

MODIFICATION: New endpoints for cluster management, accessible only with admin keys.

#### Scenario: List All Tenants (Admin Only)

Given an admin API key
When the client requests "GET /api/v1/cluster/tenants"
Then the system SHALL return 200 OK with:
```json
{
  "tenants": [
    {
      "tenant_id": "tenant_alice",
      "name": "Alice Corp",
      "created_at": "2025-11-01T10:00:00Z",
      "storage_used_bytes": 524288000,
      "storage_quota_bytes": 1073741824,
      "collections": 12,
      "vectors": 145023,
      "active": true
    },
    {
      "tenant_id": "tenant_bob",
      "name": "Bob Industries",
      "created_at": "2025-11-15T14:30:00Z",
      "storage_used_bytes": 262144000,
      "storage_quota_bytes": 536870912,
      "collections": 5,
      "vectors": 67890,
      "active": true
    }
  ],
  "total": 2
}
```

---

## API Response Headers

All authenticated responses SHALL include:

```
X-Tenant-ID: <tenant_id>
X-RateLimit-Limit: <requests_per_minute>
X-RateLimit-Remaining: <remaining_requests>
X-RateLimit-Reset: <unix_timestamp>
X-Storage-Used: <bytes>
X-Storage-Quota: <bytes>
```

## Error Response Format

All errors SHALL follow this format:

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": {
    // Optional additional context
  },
  "request_id": "req_abc123xyz"
}
```

## HTTP Status Codes

| Status | Usage |
|--------|-------|
| 200 OK | Successful operation |
| 201 Created | Resource created successfully |
| 400 Bad Request | Invalid request parameters |
| 401 Unauthorized | Missing or invalid API key |
| 403 Forbidden | Insufficient permissions |
| 404 Not Found | Resource not found (within tenant scope) |
| 429 Too Many Requests | Rate limit or quota exceeded |
| 500 Internal Server Error | Server error |
| 503 Service Unavailable | HiveHub API unavailable |

## Authentication Flow

```
Client Request
    ↓
[Auth Middleware]
    ↓
Validate API Key
    ↓
[HiveHub API / Cache]
    ↓
Extract Tenant ID
    ↓
Check Quotas & Rate Limits
    ↓
Attach Tenant Context
    ↓
Route to Handler
    ↓
Apply Tenant Scoping
    ↓
Execute Operation
    ↓
Update Usage Metrics
    ↓
Return Response
```

