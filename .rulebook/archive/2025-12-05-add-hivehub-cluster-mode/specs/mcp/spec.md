# MCP Specification: HiveHub Cluster Mode Multi-Tenant MCP

## ADDED Requirements

### Requirement: MCP StreamableHTTP Authentication

The system SHALL implement authentication for MCP StreamableHTTP endpoints.

MCP requests MUST include API keys in headers before accessing any tools. Unauthenticated requests SHALL be rejected.

#### Scenario: MCP Request with Valid API Key

Given a client making MCP StreamableHTTP request
When the client includes API key in header:
```http
POST /mcp HTTP/1.1
x-api-key: hh_test_mcp123
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 1
}
```
Then the server SHALL validate the API key with HiveHub
And SHALL extract tenant context for "tenant_alice" from API key
And SHALL return success with tenant-scoped tools

#### Scenario: MCP Request with Invalid API Key

Given a client with invalid API key "hh_invalid_key"
When the client makes an MCP request
Then the server SHALL reject the request
And SHALL return HTTP 401 Unauthorized:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Authentication failed",
    "data": {
      "reason": "Invalid API key"
    }
  },
  "id": 1
}
```

#### Scenario: MCP Tool Call Without Authentication

Given an MCP request without x-api-key header
When the client attempts to call any tool
Then the server SHALL reject with HTTP 401 and error:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32002,
    "message": "Not authenticated",
    "data": {
      "required": "API key authentication required before calling tools"
    }
  },
  "id": 2
}
```

---

### Requirement: Tenant-Scoped MCP Tools

The system SHALL automatically scope all MCP tools to the authenticated tenant.

MCP tools MUST operate only on data belonging to the authenticated tenant. Cross-tenant access SHALL be impossible through MCP.

#### Scenario: MCP Search Tool with Tenant Scoping

Given an authenticated MCP connection for "tenant_alice"
When the client calls tool "search":
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "collection": "documents",
      "query": "machine learning",
      "limit": 10
    }
  },
  "id": 3
}
```
Then the system SHALL search in "tenant_alice:documents" only
And SHALL NOT access "tenant_bob:documents" or other tenants
And SHALL return results scoped to "tenant_alice"

#### Scenario: MCP List Collections with Tenant Filtering

Given an authenticated MCP connection for "tenant_bob"
When the client calls tool "list_collections"
Then the system SHALL return only collections owned by "tenant_bob"
And SHALL omit tenant prefix in response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "collections": ["videos", "audio", "images"]
  },
  "id": 4
}
```

---

### Requirement: MCP Key Permission Isolation

The system SHALL isolate MCP keys from administrative operations.

MCP keys SHALL have access only to MCP-specific tools. Administrative tools SHALL be unavailable to MCP keys.

#### Scenario: MCP Key Attempts Admin Operation

Given an MCP API key for "tenant_alice"
When the client attempts to call a hypothetical admin tool
Then the system SHALL reject with permission error:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32003,
    "message": "Insufficient permissions",
    "data": {
      "required_permissions": ["ADMIN"],
      "granted_permissions": ["MCP"],
      "operation": "Admin tools not accessible with MCP keys"
    }
  },
  "id": 5
}
```

#### Scenario: MCP Key Normal Operations

Given an MCP API key for "tenant_charlie"
When the client calls allowed MCP tools:
- `search`
- `get_file_content`
- `list_files`
- `intelligent_search`
Then the system SHALL allow all operations
And SHALL scope to "tenant_charlie" data
And SHALL apply MCP-specific rate limits

---

### Requirement: MCP Rate Limiting

The system SHALL enforce rate limits on MCP operations.

MCP rate limits SHALL be separate from REST API limits. Limits SHALL be configurable per tenant via HiveHub.

#### Scenario: MCP Rate Limit Exceeded

Given an MCP connection for "tenant_alice"
And the tenant has MCP rate limit of 100 requests/minute
And 100 requests have been made in current minute
When the client makes another tool call
Then the system SHALL reject with:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32004,
    "message": "Rate limit exceeded",
    "data": {
      "limit": 100,
      "window": "per_minute",
      "reset_at": "2025-12-03T14:35:00Z",
      "retry_after_seconds": 42
    }
  },
  "id": 6
}
```

#### Scenario: MCP Rate Limit Reset

Given MCP rate limit window has reset
When the client makes a tool call
Then the system SHALL process the request normally
And SHALL reset the request counter

---

### Requirement: MCP Quota Enforcement

The system SHALL enforce storage quotas on MCP operations.

Operations that would exceed storage quota SHALL be rejected. Quota checks SHALL be performed before operations.

#### Scenario: MCP Insert Exceeds Quota

Given an MCP connection for "tenant_alice"
And "tenant_alice" has 950MB used of 1GB quota
When the client attempts to insert 100MB via MCP tool
Then the system SHALL reject with:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32005,
    "message": "Storage quota exceeded",
    "data": {
      "current_bytes": 995328000,
      "quota_bytes": 1073741824,
      "requested_bytes": 104857600,
      "available_bytes": 78413824
    }
  },
  "id": 7
}
```

---

### Requirement: MCP Session Management

The system SHALL maintain session state for authenticated MCP connections.

Sessions SHALL persist tenant context throughout the connection lifecycle. Session data SHALL be cleaned up on disconnect.

#### Scenario: MCP Session Lifecycle

Given a client establishes MCP connection
When the client authenticates with API key
Then the system SHALL create session with:
- `session_id`: Unique session identifier
- `tenant_id`: Authenticated tenant
- `permissions`: API key permissions
- `created_at`: Session start timestamp
- `last_activity_at`: Last request timestamp
And SHALL maintain session until disconnect

#### Scenario: MCP Session Cleanup on Disconnect

Given an active MCP session for "tenant_bob"
When the request completes
Then the system SHALL clean up request context
And SHALL release any held resources
And SHALL log request completion

---

## MODIFIED Requirements

### Requirement: MCP Tools (Modified for Multi-Tenant)

The system SHALL modify all existing MCP tools to support tenant scoping.

MODIFICATION: All MCP tools automatically filter and scope data to authenticated tenant.

#### Scenario: Get File Content with Tenant Scoping

Given an MCP connection for "tenant_alice"
And "tenant_alice" has indexed file "src/main.rs" in collection "codebase"
When the client calls "get_file_content":
```json
{
  "name": "get_file_content",
  "arguments": {
    "collection": "codebase",
    "file_path": "src/main.rs"
  }
}
```
Then the system SHALL retrieve from "tenant_alice:codebase"
And SHALL return file content
And SHALL NOT access files from other tenants

---

## MCP Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32001 | Authentication failed | Invalid API key or auth failure |
| -32002 | Not authenticated | Tool call without authentication |
| -32003 | Insufficient permissions | MCP key lacks required permissions |
| -32004 | Rate limit exceeded | MCP rate limit hit |
| -32005 | Storage quota exceeded | Operation would exceed quota |
| -32006 | Tenant not found | Invalid tenant context |
| -32007 | Collection not found | Collection doesn't exist in tenant scope |

## MCP Authentication Flow

```
1. Client Sends MCP Request (StreamableHTTP)
   ↓
2. Server Receives Request
   ↓
3. Server Extracts x-api-key Header
   ↓
4. Server Validates API Key with HiveHub
   ↓
5. Server Establishes Tenant Context for Request
   ↓
6. Request Proceeds with Tenant Scoping
   ↓
7. Server Processes Tool Call
   ↓
8. [On Completion] → Record Metrics & Cleanup
```

## MCP Tool Call Flow

```
1. Receive Tool Call Request (StreamableHTTP)
   ↓
2. Extract & Validate API Key from Header
   ↓
3. Establish Tenant Context
   ↓
4. Check Permission for Tool
   ↓
5. Check Rate Limit
   ↓
6. Check Quota (if applicable)
   ↓
7. Apply Tenant Scoping
   ↓
8. Execute Tool Logic
   ↓
9. Update Usage Metrics
   ↓
10. Return Result
```

## Tenant-Scoped MCP Tools

All MCP tools SHALL be automatically scoped to tenant:

| Tool | Tenant Scoping Behavior |
|------|-------------------------|
| `search` | Search only tenant's collections |
| `intelligent_search` | Search only tenant's collections |
| `semantic_search` | Search only tenant's collections |
| `list_collections` | List only tenant's collections |
| `get_collection_info` | Info only for tenant's collections |
| `list_files` | Files only from tenant's collections |
| `get_file_content` | Content only from tenant's files |
| `get_file_chunks` | Chunks only from tenant's files |
| `insert_text` | Insert to tenant's collection only |
| `create_collection` | Create in tenant's namespace |
| `delete_collection` | Delete from tenant's namespace |

## MCP Configuration

```yaml
mcp:
  # Enable multi-tenant mode for MCP
  multi_tenant: true
  
  # Require authentication for all MCP requests
  require_auth: true
  
  # Request timeout (seconds)
  request_timeout: 300
  
  # MCP-specific rate limits (override tenant defaults)
  rate_limits:
    requests_per_minute: 100
    requests_per_hour: 1000
  
  # StreamableHTTP configuration
  http:
    enabled: true
    max_concurrent_requests_per_tenant: 10
    timeout_seconds: 300
```

## Security Considerations

### Request Security
- ✅ MCP MUST use HTTPS (TLS) in production
- ✅ API keys MUST NOT be logged in MCP server logs
- ✅ API keys MUST be validated on every request
- ✅ Failed auth attempts MUST be rate-limited

### Data Isolation
- ✅ All tools MUST validate tenant context
- ✅ Collection names MUST be automatically prefixed
- ✅ Cross-tenant access attempts MUST be logged as security events
- ✅ Tenant context MUST be immutable after authentication

### Resource Protection
- ✅ Per-tenant concurrent request limits MUST be enforced
- ✅ Long-running operations MUST have timeouts
- ✅ Memory usage per request MUST be bounded
- ✅ Idle sessions MUST be terminated after timeout

