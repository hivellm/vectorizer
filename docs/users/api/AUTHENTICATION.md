# Authentication & Authorization

Vectorizer provides a complete authentication and authorization system for production deployments.

## Overview

The authentication system includes:
- **JWT Authentication**: Token-based authentication for user sessions
- **API Keys**: Long-lived keys for programmatic access
- **Role-Based Access Control (RBAC)**: Fine-grained permissions
- **Rate Limiting**: Request throttling per API key

## Enabling Authentication

By default, authentication is disabled for development. Enable it in your configuration:

```yaml
# vectorizer.yaml
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

## JWT Authentication

### Generate Token

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "secure-password"
}
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": 1699999999,
  "user": {
    "id": "user-123",
    "username": "admin",
    "roles": ["admin"]
  }
}
```

### Using JWT Token

Include the token in the Authorization header:

```http
GET /api/v1/collections
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token Refresh

```http
POST /api/v1/auth/refresh
Authorization: Bearer <current-token>
```

## API Keys

API keys are ideal for server-to-server communication and automation.

### Create API Key

```http
POST /api/v1/auth/api-keys
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "name": "production-backend",
  "permissions": ["read", "write"],
  "expires_at": null
}
```

Response:

```json
{
  "api_key": "vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "id": "key-123",
  "name": "production-backend",
  "permissions": ["read", "write"],
  "created_at": 1699999999,
  "expires_at": null
}
```

> ⚠️ **Important**: The API key is only shown once. Store it securely!

### Using API Key

```http
GET /api/v1/collections
X-API-Key: vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

Or in the Authorization header:

```http
GET /api/v1/collections
Authorization: ApiKey vz_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### List API Keys

```http
GET /api/v1/auth/api-keys
Authorization: Bearer <jwt-token>
```

### Revoke API Key

```http
DELETE /api/v1/auth/api-keys/{key_id}
Authorization: Bearer <jwt-token>
```

## Role-Based Access Control (RBAC)

### Available Roles

| Role | Description |
|------|-------------|
| `admin` | Full access to all operations |
| `write` | Read and write access to collections and vectors |
| `read` | Read-only access |
| `api_user` | Standard API access (assigned to API keys) |

### Permissions

| Permission | Description |
|------------|-------------|
| `collections:create` | Create new collections |
| `collections:read` | Read collection info |
| `collections:update` | Update collection settings |
| `collections:delete` | Delete collections |
| `vectors:write` | Insert/update vectors |
| `vectors:read` | Search and retrieve vectors |
| `vectors:delete` | Delete vectors |
| `admin:users` | Manage users |
| `admin:keys` | Manage API keys |
| `admin:config` | Modify configuration |

### Role-Permission Mapping

```yaml
admin:
  - "*"  # All permissions

write:
  - "collections:create"
  - "collections:read"
  - "collections:update"
  - "vectors:write"
  - "vectors:read"
  - "vectors:delete"

read:
  - "collections:read"
  - "vectors:read"
```

## Rate Limiting

API keys are subject to rate limiting to prevent abuse.

### Default Limits

| Limit Type | Default Value |
|------------|---------------|
| Per minute | 100 requests |
| Per hour | 1000 requests |

### Rate Limit Headers

Responses include rate limit information:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1699999999
```

### Rate Limit Exceeded

When limits are exceeded:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60

{
  "error": "Rate limit exceeded",
  "limit_type": "per_minute",
  "limit": 100,
  "retry_after": 60
}
```

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

