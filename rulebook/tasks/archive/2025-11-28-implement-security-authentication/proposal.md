# Implement Security & Authentication

## Status: pending

## Why

Currently, Vectorizer has an authentication module (`src/auth/`) that includes JWT, API Keys, Rate Limiting, and RBAC implementation, but **it is NOT integrated into the server routes**. All API endpoints are accessible without any authentication, which is a critical security vulnerability for production deployments.

The Synap project (a sibling project) has already implemented a complete security system that we can use as reference:
- Authentication middleware applied to ALL routes
- API Key and Basic Auth support
- Permission checking per route
- Rate limiting per API key
- Configurable auth (enable/disable via config)

## What Changes

### 1. Create Authentication Middleware Integration

Apply the existing `AuthMiddleware` from `src/auth/middleware.rs` to all routes in `src/server/mod.rs`:

```rust
// In src/server/mod.rs - create_router or equivalent
if auth_enabled {
    router = router.layer(axum::middleware::from_fn(auth_middleware_fn));
}
```

### 2. Add Auth Configuration to config.yml

```yaml
auth:
  enabled: true  # Enable/disable authentication
  require_auth: true  # Require auth for all routes (except /health)
  jwt_secret: "your-secret-key"
  jwt_expiration: 3600
  api_key_length: 32
  rate_limit_per_minute: 100
  rate_limit_per_hour: 1000
```

### 3. Create Auth API Handlers

New file `src/server/auth_handlers.rs` with endpoints:
- `POST /auth/login` - User login (returns JWT)
- `GET /auth/me` - Get current user info
- `POST /auth/keys` - Create API key
- `GET /auth/keys` - List API keys
- `DELETE /auth/keys/{id}` - Revoke API key
- `POST /auth/users` - Create user (admin only)
- `GET /auth/users` - List users (admin only)
- `DELETE /auth/users/{username}` - Delete user (admin only)

### 4. Integrate AuthMiddleware into Router

Reference: `synap-server/src/server/router.rs` lines 439-508

The middleware should:
1. Check for API Key in `Authorization: Bearer <key>` header
2. Check for Basic Auth if no API key
3. Return 401 if auth required but not provided
4. Insert `AuthContext` into request extensions
5. Allow anonymous access if `require_auth: false`

### 5. Add Permission Checking to Sensitive Routes

Use extractors like:
```rust
async fn delete_collection(
    Extension(auth): Extension<AuthContext>,
    // ...
) -> Result<...> {
    if !auth.has_permission("collections", Action::Delete) {
        return Err(StatusCode::FORBIDDEN);
    }
    // ...
}
```

### 6. Routes Protection Matrix

| Route Category | Auth Required | Permission Level |
|----------------|---------------|------------------|
| `/health` | No | - |
| `/metrics` | No | - |
| `/auth/*` | Partial* | - |
| `/collections` GET | Yes | read |
| `/collections` POST | Yes | write |
| `/collections` DELETE | Yes | admin |
| `/search` | Yes | read |
| `/insert`, `/update`, `/delete` | Yes | write |
| `/batch_*` | Yes | write |
| `/admin/*` | Yes | admin |
| `/qdrant/*` | Yes | Same as native |
| `/mcp` | Yes | read/write |
| `/umicp` | Yes | read/write |

*Auth endpoints: `/auth/login` is public, others require auth

### 7. Update Existing Auth Module

The `src/auth/` module exists but needs:
- Integration with the router
- Persistence for users/API keys (currently in-memory)
- Default admin user creation on first start

## Impact

- **Security**: All API endpoints will be protected
- **Backward Compatibility**: Can be disabled via config for development
- **SDKs**: Will need to support API key/JWT auth (already documented)

## Reference Implementation

See Synap's implementation:
- `synap-server/src/auth/mod.rs` - Auth module structure
- `synap-server/src/auth/middleware.rs` - Middleware implementation
- `synap-server/src/server/router.rs` - Route integration
- `synap-server/src/server/auth_handlers.rs` - Auth API handlers

## Testing

1. Start server with `auth.enabled: false` - should work as before
2. Start server with `auth.enabled: true` - all routes return 401
3. Create API key via config or endpoint
4. Access routes with API key - should work
5. Test rate limiting
6. Test permission checking

