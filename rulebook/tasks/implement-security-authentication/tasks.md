# Tasks: Implement Security & Authentication

## Status: pending

## 1. Configuration Phase
- [ ] 1.1 Add auth configuration to `src/config/vectorizer.rs`
- [ ] 1.2 Update config.example.yml with auth section
- [ ] 1.3 Add auth config parsing and defaults

## 2. Auth Middleware Integration
- [ ] 2.1 Create `src/server/auth_handlers.rs` with auth API endpoints
- [ ] 2.2 Update `src/server/mod.rs` to integrate AuthMiddleware
- [ ] 2.3 Apply middleware to all routes (except /health, /metrics)
- [ ] 2.4 Add AuthContext extraction to request extensions

## 3. Auth API Endpoints
- [ ] 3.1 Implement `POST /auth/login` (JWT generation)
- [ ] 3.2 Implement `GET /auth/me` (current user info)
- [ ] 3.3 Implement `POST /auth/keys` (create API key)
- [ ] 3.4 Implement `GET /auth/keys` (list API keys)
- [ ] 3.5 Implement `DELETE /auth/keys/{id}` (revoke key)
- [ ] 3.6 Implement user management endpoints (admin only)

## 4. Permission Checking
- [ ] 4.1 Add permission extractors (require_auth, require_admin, require_permission)
- [ ] 4.2 Apply permission checks to collection management routes
- [ ] 4.3 Apply permission checks to vector operations routes
- [ ] 4.4 Apply permission checks to admin routes
- [ ] 4.5 Apply permission checks to Qdrant compatibility routes

## 5. Persistence
- [ ] 5.1 Add user/API key persistence to data directory
- [ ] 5.2 Implement auto-creation of default admin user on first start
- [ ] 5.3 Load users/keys from disk on server startup

## 6. Rate Limiting
- [ ] 6.1 Integrate rate limiting checks in middleware
- [ ] 6.2 Add rate limit headers to responses
- [ ] 6.3 Return 429 when rate limit exceeded

## 7. Testing
- [ ] 7.1 Add integration tests for auth disabled mode
- [ ] 7.2 Add integration tests for auth enabled mode
- [ ] 7.3 Add tests for API key authentication
- [ ] 7.4 Add tests for JWT authentication
- [ ] 7.5 Add tests for permission checking
- [ ] 7.6 Add tests for rate limiting

## 8. Documentation
- [ ] 8.1 Update docs/users/api/AUTHENTICATION.md with implementation details
- [ ] 8.2 Update SDK documentation with auth examples
- [ ] 8.3 Update README with security section

## Dependencies
- Existing `src/auth/` module (JWT, API Keys, RBAC)
- Reference: `synap-server/src/auth/` and `synap-server/src/server/router.rs`

## Acceptance Criteria
- [ ] Server starts with auth disabled - works as before (backward compatible)
- [ ] Server starts with auth enabled - all routes (except /health) return 401
- [ ] API keys can be created and used for authentication
- [ ] JWT tokens can be generated via login and used
- [ ] Permission checks work for different roles
- [ ] Rate limiting prevents abuse
- [ ] Users and API keys persist across restarts

