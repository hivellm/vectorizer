# Authentication & Authorization

Vectorizer provides a complete authentication and authorization system for production deployments.

## Overview

The authentication system includes:
- **JWT Authentication**: Token-based authentication for user sessions
- **API Keys**: Long-lived keys for programmatic access
- **Role-Based Access Control (RBAC)**: Fine-grained permissions
- **Rate Limiting**: Request throttling per API key
- **Persistence**: Users and API keys are saved to disk and persist across restarts
- **Default Admin**: Auto-created on first start when authentication is enabled

## Enabling Authentication

By default, authentication is disabled for development. Enable it in your configuration:

```yaml
# config.yml
auth:
  enabled: true
  jwt_secret: "your-secure-secret-key-at-least-32-chars"
  jwt_expiration: 3600  # 1 hour
  api_key_length: 32
  rate_limit_per_minute: 100
  rate_limit_per_hour: 1000
```

Or via environment variables:

```bash
export VECTORIZER_AUTH_ENABLED=true
export VECTORIZER_JWT_SECRET="your-secure-secret-key"
export VECTORIZER_JWT_EXPIRATION=3600
```

## First Start & Default Admin

When authentication is enabled and no users exist, Vectorizer automatically creates a default admin user:

- **Username**: `admin`
- **Password**: `admin123`

> ⚠️ **Important**: Change the default admin password immediately in production!

Auth data is stored in `data/auth.json` and persists across server restarts.

## JWT Authentication

### Login

```http
POST /auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

Response:

```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": 1699999999,
  "user_id": "user-123",
  "username": "admin",
  "roles": ["Admin"]
}
```

### Using JWT Token

Include the token in the Authorization header:

```http
GET /collections
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Get Current User

```http
GET /auth/me
Authorization: Bearer <jwt-token>
```

Response:

```json
{
  "user_id": "user-123",
  "username": "admin",
  "roles": ["Admin"]
}
```

## API Keys

API keys are ideal for server-to-server communication and automation.

### Create API Key

```http
POST /auth/keys
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "name": "production-backend",
  "permissions": ["Read", "Write", "Search"],
  "expires_in_days": null
}
```

Response:

```json
{
  "success": true,
  "api_key": "vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "key_id": "key-123",
  "name": "production-backend",
  "message": "Store this API key securely. It will not be shown again."
}
```

> ⚠️ **Important**: The API key is only shown once. Store it securely!

### Using API Key

Three methods are supported:

**1. X-API-Key Header (Recommended)**
```http
GET /collections
X-API-Key: vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**2. Authorization Header**
```http
GET /collections
Authorization: vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**3. Query Parameter**
```http
GET /collections?api_key=vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### List API Keys

```http
GET /auth/keys
Authorization: Bearer <jwt-token>
```

Response:

```json
{
  "keys": [
    {
      "id": "key-123",
      "name": "production-backend",
      "created_at": 1699999999,
      "last_used": 1699999999,
      "expires_at": null,
      "active": true
    }
  ]
}
```

### Revoke API Key

```http
DELETE /auth/keys/{key_id}
Authorization: Bearer <jwt-token>
```

## User Management (Admin Only)

Administrators can manage users through the following endpoints.

### Create User

```http
POST /auth/users
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "username": "newuser",
  "password": "secure-password",
  "roles": ["User"]
}
```

Response:

```json
{
  "success": true,
  "user_id": "user-456",
  "username": "newuser",
  "roles": ["User"],
  "message": "User created successfully"
}
```

### List Users

```http
GET /auth/users
Authorization: Bearer <admin-jwt-token>
```

Response:

```json
{
  "users": [
    {
      "user_id": "user-123",
      "username": "admin",
      "roles": ["Admin"],
      "created_at": 1699999999,
      "last_login": 1699999999
    },
    {
      "user_id": "user-456",
      "username": "newuser",
      "roles": ["User"],
      "created_at": 1699999999,
      "last_login": null
    }
  ]
}
```

### Delete User

```http
DELETE /auth/users/{username}
Authorization: Bearer <admin-jwt-token>
```

### Change Password

Users can change their own password, and admins can change any user's password:

```http
PUT /auth/users/{username}/password
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "current_password": "old-password",
  "new_password": "new-secure-password"
}
```

> Note: Admins do not need to provide `current_password` when changing another user's password.

## Role-Based Access Control (RBAC)

### Available Roles

| Role | Description |
|------|-------------|
| `Admin` | Full access to all operations, including user management |
| `User` | Standard access to collections and vectors |
| `ReadOnly` | Read-only access to collections and vectors |
| `ApiUser` | Standard API access (assigned to API keys) |

### Permissions

| Permission | Description |
|------------|-------------|
| `Read` | Read collections and search vectors |
| `Write` | Insert/update vectors |
| `Delete` | Delete vectors |
| `Search` | Perform vector searches |
| `CreateCollection` | Create new collections |
| `DeleteCollection` | Delete collections |
| `ManageUsers` | Create/delete users (admin only) |
| `ManageApiKeys` | Create/revoke API keys |
| `ViewLogs` | View server logs |
| `SystemConfig` | Modify server configuration |

### Role-Permission Mapping

```yaml
Admin:
  - All permissions

User:
  - Read
  - Write
  - Delete
  - Search
  - CreateCollection
  - DeleteCollection
  - ManageApiKeys

ReadOnly:
  - Read
  - Search

ApiUser:
  - Based on permissions granted when creating the API key
```

## Rate Limiting

API keys are subject to rate limiting to prevent abuse. Vectorizer supports per-API-key rate limiting with configurable tiers and overrides.

### Rate Limiting Architecture

```
Request → Extract API Key → Check Tier/Override → Apply Limits → Allow/Deny
```

### Default Limits

| Limit Type | Default Value |
|------------|---------------|
| Requests per second | 100 |
| Burst size | 200 |

### Rate Limit Tiers

Configure different rate limits for different API key tiers:

```yaml
# config.yml
rate_limit:
  enabled: true
  default_requests_per_second: 100
  default_burst_size: 200

  tiers:
    default:
      requests_per_second: 100
      burst_size: 200

    premium:
      requests_per_second: 500
      burst_size: 1000

    enterprise:
      requests_per_second: 1000
      burst_size: 2000

  key_tiers:
    "vz_premium_key_123": "premium"
    "vz_enterprise_key_456": "enterprise"

  key_overrides:
    "vz_special_key_789":
      requests_per_second: 2000
      burst_size: 5000
```

### Assigning Keys to Tiers

Assign API keys to tiers in configuration:

```yaml
rate_limit:
  key_tiers:
    "vz_abc123": "premium"
    "vz_xyz789": "enterprise"
```

Or programmatically via API (admin only):

```http
POST /auth/keys/{key_id}/tier
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "tier": "premium"
}
```

### Per-Key Overrides

Set custom limits for specific keys:

```yaml
rate_limit:
  key_overrides:
    "vz_special_key":
      requests_per_second: 5000
      burst_size: 10000
```

### Rate Limit Headers

Responses include rate limit information:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699999999
X-RateLimit-Tier: premium
```

### Rate Limit Exceeded

When limits are exceeded:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 1

{
  "error": "Rate limit exceeded",
  "requests_per_second": 100,
  "burst_size": 200,
  "tier": "default",
  "retry_after": 1
}
```

### Get Rate Limit Info

Check rate limit status for your API key:

```http
GET /auth/rate-limit
X-API-Key: vz_xxxxx
```

Response:

```json
{
  "tier": "premium",
  "requests_per_second": 500,
  "burst_size": 1000,
  "current_usage": 45,
  "remaining": 455
}
```

### Best Practices

1. **Use appropriate tiers**: Assign higher tiers to production workloads
2. **Monitor usage**: Track rate limit headers to avoid hitting limits
3. **Implement backoff**: Use exponential backoff when rate limited
4. **Request tier upgrades**: Contact admin for higher limits if needed

## SDK Authentication

### Python

```python
from vectorizer_sdk import VectorizerClient

# With API key
client = VectorizerClient(
    "http://localhost:15002",
    api_key="vz_xxxxx"
)

# With JWT
client = VectorizerClient(
    "http://localhost:15002",
    jwt_token="eyJhbGciOiJIUzI1NiI..."
)
```

### TypeScript

```typescript
import { VectorizerClient } from "@hivellm/vectorizer-sdk";

// With API key
const client = new VectorizerClient("http://localhost:15002", {
  apiKey: "vz_xxxxx"
});

// With JWT
const client = new VectorizerClient("http://localhost:15002", {
  jwtToken: "eyJhbGciOiJIUzI1NiI..."
});
```

### Rust

```rust
use vectorizer_sdk::VectorizerClient;

// With API key
let client = VectorizerClient::builder("http://localhost:15002")
    .api_key("vz_xxxxx")
    .build()?;

// With JWT
let client = VectorizerClient::builder("http://localhost:15002")
    .jwt_token("eyJhbGciOiJIUzI1NiI...")
    .build()?;
```

## Security Best Practices

### JWT Secret

- Use at least 32 characters
- Use a cryptographically secure random value
- Never commit secrets to version control
- Rotate secrets periodically

```bash
# Generate secure secret
openssl rand -base64 32
```

### API Keys

- Use descriptive names for keys
- Set expiration dates for temporary access
- Revoke unused keys promptly
- Monitor key usage via logs

### Production Checklist

- [ ] Enable authentication (`auth.enabled: true`)
- [ ] Set a strong JWT secret
- [ ] Configure appropriate rate limits
- [ ] Enable HTTPS/TLS
- [ ] Monitor authentication logs
- [ ] Implement key rotation strategy

## Troubleshooting

### Invalid Token

```json
{
  "error": "Invalid token",
  "details": "Token has expired"
}
```

**Solution**: Generate a new token or use token refresh.

### API Key Not Found

```json
{
  "error": "API key not found",
  "details": "The provided API key does not exist"
}
```

**Solution**: Check the key is correct and not revoked.

### Permission Denied

```json
{
  "error": "Permission denied",
  "details": "Insufficient permissions for this operation"
}
```

**Solution**: Check user/key roles and permissions.

## Related Topics

- [Admin API](./ADMIN.md) - Administrative endpoints
- [Configuration](../configuration/CONFIGURATION.md) - Server configuration
- [Monitoring](../operations/MONITORING.md) - Security monitoring

