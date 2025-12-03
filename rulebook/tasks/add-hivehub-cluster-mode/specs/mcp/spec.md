# MCP Specification: HiveHub Cluster Mode Multi-Tenant MCP

## ADDED Requirements

### Requirement: MCP WebSocket Authentication

The system SHALL implement authentication for MCP WebSocket connections.

MCP connections MUST authenticate using API keys before accessing any tools. Unauthenticated connections SHALL be rejected.

#### Scenario: MCP Connection with Valid API Key

Given a client initiating MCP WebSocket connection
When the client sends authentication message:
```json
{
  "jsonrpc": "2.0",
  "method": "auth/authenticate",
  "params": {
    "api_key": "hh_test_mcp123"
  },
  "id": 1
}
```
Then the server SHALL validate the API key with HiveHub
And SHALL establish tenant context for "tenant_alice"
And SHALL return success:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "authenticated": true,
    "tenant_id": "tenant_alice",
    "permissions": ["MCP"]
  },
  "id": 1
}
```

#### Scenario: MCP Connection with Invalid API Key

Given a client with invalid API key "hh_invalid_key"
When the client attempts authentication
Then the server SHALL reject the connection
And SHALL return error:
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
And SHALL close the WebSocket connection

#### Scenario: MCP Tool Call Without Authentication

Given an unauthenticated MCP connection
When the client attempts to call any tool
Then the server SHALL reject with error:
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
When the WebSocket connection closes
Then the system SHALL clean up session data
And SHALL release any held resources
And SHALL log session end event

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
1. Client Connects to MCP WebSocket
   ↓
2. Server Accepts Connection
   ↓
3. Client Sends auth/authenticate
   ↓
4. Server Validates API Key with HiveHub
   ↓
5. Server Creates Session with Tenant Context
   ↓
6. Server Returns Authentication Success
   ↓
7. Client Can Now Call Tools
   ↓
8. [On Disconnect] → Cleanup Session
```

## MCP Tool Call Flow

```
1. Receive Tool Call Request
   ↓
2. Validate Session Authenticated
   ↓
3. Check Permission for Tool
   ↓
4. Check Rate Limit
   ↓
5. Check Quota (if applicable)
   ↓
6. Apply Tenant Scoping
   ↓
7. Execute Tool Logic
   ↓
8. Update Usage Metrics
   ↓
9. Return Result
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
  
  # Require authentication for all MCP connections
  require_auth: true
  
  # Session timeout (seconds)
  session_timeout: 3600
  
  # MCP-specific rate limits (override tenant defaults)
  rate_limits:
    requests_per_minute: 100
    requests_per_hour: 1000
  
  # WebSocket configuration
  websocket:
    max_connections_per_tenant: 10
    ping_interval: 30
    pong_timeout: 10
```

## Security Considerations

### Connection Security
- ✅ MCP MUST use WSS (WebSocket Secure) in production
- ✅ API keys MUST NOT be logged in MCP server logs
- ✅ Session tokens MUST be cryptographically random
- ✅ Failed auth attempts MUST be rate-limited

### Data Isolation
- ✅ All tools MUST validate tenant context
- ✅ Collection names MUST be automatically prefixed
- ✅ Cross-tenant access attempts MUST be logged as security events
- ✅ Tenant context MUST be immutable after authentication

### Resource Protection
- ✅ Per-tenant connection limits MUST be enforced
- ✅ Long-running operations MUST be cancellable
- ✅ Memory usage per session MUST be bounded
- ✅ Idle sessions MUST be terminated after timeout

