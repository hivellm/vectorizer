# Authentication & Authorization Specification: HiveHub Cluster Mode

## ADDED Requirements

### Requirement: API Key Format and Structure

The system SHALL support HiveHub-issued API keys with specific format requirements.

API keys MUST follow the format: `hh_{environment}_{random}` where:
- `hh_`: Fixed prefix identifying HiveHub keys
- `{environment}`: "test" or "live" 
- `{random}`: 32-character alphanumeric string

#### Scenario: Valid API Key Format

Given an API key "hh_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
When the system validates the key format
Then the system SHALL accept the format as valid
And SHALL proceed to HiveHub validation

#### Scenario: Invalid API Key Format

Given an API key "invalid_key_format"
When the system validates the key format
Then the system SHALL reject with 401 Unauthorized
And SHALL return error: `{"error": "Invalid API key format", "code": "AUTH_INVALID_FORMAT"}`
And SHALL NOT call HiveHub API

---

### Requirement: Permission-Based Access Control

The system SHALL implement granular permission-based access control.

API keys SHALL have one or more permissions. Operations SHALL be allowed only if the key has required permissions.

**Permission Levels**:
- `ADMIN`: Full system access (collection management, admin operations, tenant management)
- `READ_WRITE`: Data operations (insert, update, delete, search)
- `READ_ONLY`: Read and search operations only
- `MCP`: MCP protocol operations (isolated from admin functions)

#### Scenario: Admin Key Full Access

Given an API key with permissions ["ADMIN"]
When the key attempts any operation
Then the system SHALL allow the operation
And SHALL grant access to admin-only endpoints

#### Scenario: Read-Only Key Write Attempt

Given an API key with permissions ["READ_ONLY"]
When the key attempts "POST /api/v1/collections/docs/vectors"
Then the system SHALL reject with 403 Forbidden
And SHALL return error: `{"error": "Insufficient permissions", "code": "FORBIDDEN", "required": ["READ_WRITE"], "granted": ["READ_ONLY"]}`

#### Scenario: MCP Key Admin Endpoint Access

Given an API key with permissions ["MCP"]
When the key attempts "GET /api/v1/cluster/health"
Then the system SHALL reject with 403 Forbidden
And SHALL return error: `{"error": "Admin access required", "code": "FORBIDDEN"}`

---

### Requirement: API Key Scoping and Isolation

The system SHALL enforce strict scoping for different key types.

MCP keys MUST be isolated from administrative functions. Each key type SHALL have access only to its designated operations.

#### Scenario: MCP Key Isolated from Admin Functions

Given an MCP API key for tenant "tenant_alice"
When the key attempts to list all tenants
Then the system SHALL reject with 403 Forbidden
And SHALL NOT expose tenant enumeration
And SHALL log the unauthorized access attempt

#### Scenario: MCP Key Access to Own Collections

Given an MCP API key for tenant "tenant_alice"
When the key searches in collection "documents"
Then the system SHALL allow the operation
And SHALL scope to "tenant_alice:documents" only
And SHALL apply MCP-specific rate limits

---

### Requirement: API Key Validation Caching

The system SHALL implement efficient caching for API key validation.

Cache MUST reduce HiveHub API calls by >90%. Cache invalidation SHALL be immediate on key revocation.

#### Scenario: Cache Hit for Recent Key

Given API key "hh_test_cached123" validated 30 seconds ago
And cache TTL is 300 seconds
When a request arrives with this key
Then the system SHALL use cached tenant data
And SHALL complete auth in <5ms
And SHALL NOT call HiveHub API

#### Scenario: Cache Miss for New Key

Given API key "hh_test_newkey456" not in cache
When a request arrives with this key
Then the system SHALL call HiveHub API
And SHALL cache the response for 300 seconds
And SHALL use cached data for subsequent requests within TTL

#### Scenario: Cache Invalidation on Revocation

Given API key "hh_test_revoked789" in cache
When HiveHub sends revocation webhook notification
Then the system SHALL immediately evict key from cache
And SHALL reject subsequent requests with 401 Unauthorized
And SHALL force fresh validation on next attempt

---

### Requirement: Brute Force Protection

The system SHALL implement brute force protection for authentication attempts.

Failed authentication attempts SHALL be rate-limited per IP address. Excessive failures SHALL result in temporary blocking.

#### Scenario: Failed Authentication Rate Limiting

Given 5 failed auth attempts from IP "203.0.113.42" in 60 seconds
When another invalid key is attempted from same IP
Then the system SHALL reject with 429 Too Many Requests
And SHALL return:
```json
{
  "error": "Too many authentication failures",
  "code": "AUTH_RATE_LIMIT",
  "retry_after_seconds": 300
}
```

#### Scenario: Successful Auth Resets Counter

Given 3 failed auth attempts from IP "203.0.113.42"
When a valid key is successfully authenticated from same IP
Then the system SHALL reset the failure counter
And SHALL allow normal operation

---

### Requirement: Audit Logging for Authentication

The system SHALL log all authentication events for security audit.

Audit logs MUST include sufficient information for security investigation. Logs SHALL be tamper-proof.

#### Scenario: Successful Authentication Logging

Given a successful authentication with API key
When the auth completes
Then the system SHALL log:
- `timestamp`: ISO 8601 timestamp
- `event`: "AUTH_SUCCESS"
- `tenant_id`: Authenticated tenant
- `api_key_id`: Key identifier (not full key)
- `ip_address`: Client IP
- `user_agent`: Client user agent
- `endpoint`: Requested endpoint

#### Scenario: Failed Authentication Logging

Given a failed authentication attempt
When the auth fails
Then the system SHALL log:
- `timestamp`: ISO 8601 timestamp
- `event`: "AUTH_FAILURE"
- `reason`: Failure reason code
- `api_key_prefix`: First 8 characters only
- `ip_address`: Client IP
- `user_agent`: Client user agent

---

### Requirement: API Key Rotation Support

The system SHALL support graceful API key rotation.

During rotation, both old and new keys SHALL be valid for a grace period. Clients SHALL receive warnings before old key expiration.

#### Scenario: Key Rotation Grace Period

Given an API key "hh_test_old123" marked for rotation
And a new key "hh_test_new456" issued
And grace period is 7 days
When a request uses "hh_test_old123"
Then the system SHALL accept the request
And SHALL include warning header: `X-API-Key-Deprecated: true`
And SHALL include header: `X-API-Key-Expires: 2025-12-10T00:00:00Z`

#### Scenario: Expired Key After Grace Period

Given an API key "hh_test_expired789" with expired grace period
When a request uses this key
Then the system SHALL reject with 401 Unauthorized
And SHALL return error: `{"error": "API key expired", "code": "AUTH_KEY_EXPIRED"}`
And SHALL suggest key rotation in response

---

## Permission Matrix

| Operation | ADMIN | READ_WRITE | READ_ONLY | MCP |
|-----------|-------|------------|-----------|-----|
| Create Collection | ✅ | ✅ | ❌ | ❌ |
| Delete Collection | ✅ | ✅ | ❌ | ❌ |
| List Collections | ✅ | ✅ | ✅ | ✅ |
| Insert Vectors | ✅ | ✅ | ❌ | ✅ |
| Update Vectors | ✅ | ✅ | ❌ | ✅ |
| Delete Vectors | ✅ | ✅ | ❌ | ❌ |
| Search Vectors | ✅ | ✅ | ✅ | ✅ |
| Get Collection Info | ✅ | ✅ | ✅ | ✅ |
| Admin Endpoints | ✅ | ❌ | ❌ | ❌ |
| Cluster Health | ✅ | ❌ | ❌ | ❌ |
| Tenant Management | ✅ | ❌ | ❌ | ❌ |

## API Key Metadata

API keys returned from HiveHub SHALL include:

```json
{
  "api_key_id": "key_abc123",
  "tenant_id": "tenant_alice",
  "name": "Production API Key",
  "permissions": ["READ_WRITE"],
  "created_at": "2025-11-01T10:00:00Z",
  "expires_at": null,
  "last_used_at": "2025-12-03T14:30:00Z",
  "rotation_status": "active",
  "rate_limit_override": null
}
```

## Authentication Flow

```
1. Extract API Key from Header
   ↓
2. Validate Key Format
   ↓
3. Check Cache for Key Data
   ↓
4. [Cache Miss] → Call HiveHub API
   ↓
5. [Cache Hit] → Use Cached Data
   ↓
6. Extract Tenant ID & Permissions
   ↓
7. Check Brute Force Protection
   ↓
8. Validate Key Not Revoked/Expired
   ↓
9. Attach Tenant Context to Request
   ↓
10. Log Authentication Event
   ↓
11. Proceed to Authorization Check
```

## Authorization Flow

```
1. Receive Request with Tenant Context
   ↓
2. Determine Required Permission(s)
   ↓
3. Check Key Permissions
   ↓
4. [Insufficient] → 403 Forbidden
   ↓
5. [Sufficient] → Proceed
   ↓
6. Apply Tenant Scoping
   ↓
7. Execute Operation
```

## Security Considerations

### API Key Storage
- ✅ Keys MUST be transmitted over HTTPS only
- ✅ Keys MUST NOT be logged in full (first 8 chars only)
- ✅ Keys MUST NOT be stored in server logs
- ✅ Keys MUST be validated server-side on every request

### Cache Security
- ✅ Cached data MUST include permissions and quotas
- ✅ Cache MUST be invalidated immediately on revocation
- ✅ Cache MUST have reasonable TTL (≤5 minutes)
- ✅ Cache MUST be protected from unauthorized access

### Audit Requirements
- ✅ All auth events MUST be logged
- ✅ Logs MUST be append-only
- ✅ Logs MUST include request correlation ID
- ✅ Failed auth attempts MUST be monitored and alerted

