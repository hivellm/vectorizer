## 1. Backend - Authentication API

- [x] 1.1 Create src/server/dashboard_auth.rs (implemented in src/auth/ and auth_handlers.rs)
- [x] 1.2 Add DashboardAuthConfig struct (AuthConfig in src/auth/mod.rs)
- [x] 1.3 Implement POST /api/dashboard/auth/login endpoint (POST /auth/login)
- [ ] 1.4 Implement POST /api/dashboard/auth/logout endpoint
- [x] 1.5 Implement GET /api/dashboard/auth/verify endpoint (GET /auth/me)
- [ ] 1.6 Implement POST /api/dashboard/auth/refresh endpoint
- [x] 1.7 Add JWT token generation and validation (src/auth/jwt.rs)
- [ ] 1.8 Add secure cookie handling

## 2. Backend - Local Authentication

- [ ] 2.1 Create user storage system in secrets/users.json (using auth/persistence.rs but not in secrets/)
- [x] 2.2 Implement bcrypt password hashing (cost factor 10)
- [x] 2.3 Add UserStore for reading/writing users.json (AuthPersistence in src/auth/persistence.rs)
- [x] 2.4 Implement password validation and complexity rules
- [x] 2.5 Add role-based access (admin, viewer) (src/auth/roles.rs)
- [x] 2.6 Support multiple admin users
- [ ] 2.7 Add file permissions check (warn if not 600)
- [ ] 2.8 Add backup mechanism for users.json

## 3. Backend - Root User Auto-Creation

- [ ] 3.1 Check if any admin user exists on server startup
- [ ] 3.2 Create secrets/ directory if not exists
- [ ] 3.3 Create users.json if not exists
- [ ] 3.4 Generate root user if no admins found
- [ ] 3.5 Support --ROOT_USER CLI argument (default: "root")
- [ ] 3.6 Support --ROOT_PASSWORD CLI argument
- [ ] 3.7 Generate random secure password if not provided
- [ ] 3.8 Print root credentials to console (one-time only)
- [ ] 3.9 Add warning banner about changing default password
- [ ] 3.10 Add secrets/ to .gitignore

## 4. Backend - HiveHub Integration

- [ ] 4.1 Add HiveHub auth mode to dashboard
- [ ] 4.2 Validate API key via HubManager
- [ ] 4.3 Extract tenant_id from API key
- [ ] 4.4 Create tenant-scoped session
- [ ] 4.5 Add tenant context to all dashboard API calls

## 5. Backend - User Management API

- [ ] 5.1 Create src/server/dashboard_users.rs
- [ ] 5.2 Add POST /api/dashboard/users endpoint (create user)
- [ ] 5.3 Add GET /api/dashboard/users endpoint (list users)
- [ ] 5.4 Add DELETE /api/dashboard/users/:username endpoint
- [ ] 5.5 Add PUT /api/dashboard/users/:username/password endpoint
- [ ] 5.6 Protect all user management endpoints (admin only)
- [ ] 5.7 Prevent deletion of root user
- [ ] 5.8 Prevent deletion of last admin user
- [ ] 5.9 Audit log all user management operations

## 6. Backend - Session Management

- [x] 6.1 Implement JWT session tokens (src/auth/jwt.rs)
- [x] 6.2 Add session storage (in-memory or Redis) (in-memory via AuthManager)
- [x] 6.3 Implement session expiration (default 24h)
- [ ] 6.4 Add session refresh mechanism
- [ ] 6.5 Implement CSRF token generation
- [ ] 6.6 Add secure cookie settings (HTTP-only, Secure, SameSite)

## 7. Backend - Auth Middleware

- [x] 7.1 Create dashboard auth middleware (src/auth/middleware.rs)
- [x] 7.2 Protect all /api/dashboard/* routes (auth middleware active)
- [x] 7.3 Allow /api/dashboard/auth/* without auth (auth routes public)
- [ ] 7.4 Add tenant scoping in cluster mode
- [ ] 7.5 Skip auth in development mode
- [x] 7.6 Add auth bypass for health check endpoint

## 8. Frontend - Login Page

- [x] 8.1 Create dashboard/src/pages/LoginPage.tsx
- [x] 8.2 Add username/password form (local mode)
- [ ] 8.3 Add API key input (cluster mode)
- [ ] 8.4 Add "Remember me" checkbox
- [x] 8.5 Add error handling and validation
- [x] 8.6 Add loading states
- [x] 8.7 Style login page (modern UI)
- [x] 8.8 Add HiveLLM/Vectorizer branding
- [ ] 8.9 Add password strength indicator

## 9. Frontend - User Management Page

- [ ] 9.1 Create dashboard/src/pages/Users.tsx
- [ ] 9.2 Add user list table with role and created date
- [ ] 9.3 Add "Create User" button and modal
- [ ] 9.4 Add create user form (username, password, role)
- [ ] 9.5 Add delete user confirmation dialog
- [ ] 9.6 Add change password form
- [ ] 9.7 Prevent deletion of root user (disable button)
- [ ] 9.8 Show current logged-in user indicator
- [ ] 9.9 Add user management to dashboard navigation (admin only)

## 9A. Backend - API Key Management

- [ ] 9A.1 Create src/server/dashboard_api_keys.rs
- [ ] 9A.2 Implement ApiKeyStore (reads/writes secrets/api_keys.json)
- [ ] 9A.3 Add POST /api/dashboard/api-keys endpoint (create key)
- [ ] 9A.4 Add GET /api/dashboard/api-keys endpoint (list user's keys)
- [ ] 9A.5 Add DELETE /api/dashboard/api-keys/:id endpoint (revoke key)
- [ ] 9A.6 Add PUT /api/dashboard/api-keys/:id endpoint (update permissions)
- [ ] 9A.7 Generate secure random keys (format: vec_sk_32_random_chars)
- [ ] 9A.8 Hash API keys with SHA-256 for storage
- [ ] 9A.9 Link API keys to user account (owner_id)
- [ ] 9A.10 Track last_used timestamp on each request
- [ ] 9A.11 Track usage_count for billing/monitoring
- [ ] 9A.12 Support key expiration (optional)

## 9B. Frontend - API Key Management Page

- [ ] 9B.1 Create dashboard/src/pages/ApiKeys.tsx
- [ ] 9B.2 Add API keys list table (name, permissions, created, last used)
- [ ] 9B.3 Add "Create API Key" button and modal
- [ ] 9B.4 Add create key form (name, permissions, expiration)
- [ ] 9B.5 Show generated key only once (with copy button)
- [ ] 9B.6 Add warning: "Save this key - it won't be shown again"
- [ ] 9B.7 Add revoke key confirmation dialog
- [ ] 9B.8 Add key usage statistics (total requests)
- [ ] 9B.9 Add permissions badges (read/write/admin)
- [ ] 9B.10 Add "API Keys" to dashboard navigation
- [ ] 9B.11 Add key regeneration option (creates new, revokes old)

## 10. Frontend - Auth Context

- [x] 10.1 Create dashboard/src/contexts/AuthContext.tsx
- [x] 10.2 Implement useAuth() hook
- [x] 10.3 Add login() function
- [x] 10.4 Add logout() function
- [x] 10.5 Add verifySession() function
- [ ] 10.6 Add auto-refresh session logic
- [ ] 10.7 Handle auth errors globally

## 11. Frontend - Protected Routes

- [x] 11.1 Create ProtectedRoute component
- [x] 11.2 Wrap all dashboard routes with ProtectedRoute
- [x] 11.3 Redirect to /login if not authenticated
- [x] 11.4 Redirect to /overview after login
- [ ] 11.5 Handle session expiration gracefully
- [x] 11.6 Show loading state while verifying auth

## 12. Frontend - User Interface

- [x] 12.1 Add logout button to dashboard header
- [x] 12.2 Display logged-in username
- [ ] 12.3 Display tenant info (cluster mode)
- [ ] 12.4 Add mode indicator (dev/prod/cluster)
- [ ] 12.5 Show warning banner in development mode
- [ ] 12.6 Add session timeout warning

## 13. Frontend - API Client Updates

- [ ] 13.1 Update API client to include auth token in headers
- [ ] 13.2 Handle 401 Unauthorized responses
- [ ] 13.3 Auto-redirect to login on auth failure
- [ ] 13.4 Implement token refresh on 401
- [ ] 13.5 Add retry logic for failed auth

## 14. Backend - Tenant Scoping (Cluster Mode)

- [ ] 14.1 Filter collections by owner_id in dashboard API
- [ ] 14.2 Filter vectors by owner_id in search results
- [ ] 14.3 Prevent access to other tenants' data
- [ ] 14.4 Add tenant validation in all endpoints
- [ ] 14.5 Log unauthorized access attempts

## 15. CLI Arguments & Environment

- [ ] 15.1 Add --ROOT_USER argument to CLI
- [ ] 15.2 Add --ROOT_PASSWORD argument to CLI
- [ ] 15.3 Support ROOT_USER environment variable
- [ ] 15.4 Support ROOT_PASSWORD environment variable
- [ ] 15.5 Validate password complexity on CLI input
- [ ] 15.6 Print root user credentials on first startup
- [ ] 15.7 Add warning about changing default password
- [ ] 15.8 Document CLI arguments in --help

## 16. Secrets Management

- [ ] 16.1 Create secrets/ directory on first run
- [ ] 16.2 Set directory permissions to 700 (owner only)
- [ ] 16.3 Create users.json with root user
- [ ] 16.4 Create api_keys.json (empty initially)
- [ ] 16.5 Set file permissions to 600 (owner read/write only)
- [ ] 16.6 Add secrets/ to .gitignore
- [ ] 16.7 Add secrets/.gitkeep for directory tracking
- [ ] 16.8 Add secrets/README.md with warning (not gitignored)
- [ ] 16.9 Validate permissions on startup (warn if insecure)
- [ ] 16.10 Document secrets directory structure
- [ ] 16.11 Add backup mechanism for secrets/ directory

## 17. Configuration & Deployment

- [ ] 17.1 Add dashboard.auth section to config.yml
- [ ] 17.2 Add DASHBOARD_SESSION_SECRET env var
- [ ] 17.3 Add ROOT_USER env var support
- [ ] 17.4 Add ROOT_PASSWORD env var support
- [ ] 17.5 Document auth configuration
- [ ] 17.6 Update Docker image with secrets volume
- [ ] 17.7 Add Kubernetes secrets for root credentials
- [ ] 17.8 Create docker-compose.yml example with secrets

## 18. Security Hardening

- [ ] 18.1 Implement rate limiting on login endpoint (5 attempts/minute)
- [ ] 18.2 Add brute-force protection (account lockout 15min after 5 failures)
- [ ] 18.3 Implement CSRF protection
- [ ] 18.4 Add security headers (CSP, X-Frame-Options, etc.)
- [ ] 18.5 Use HTTPS in production (enforce TLS)
- [ ] 18.6 Audit log all login attempts with IP
- [ ] 18.7 Password complexity requirements (8+ chars, uppercase, number)
- [ ] 18.8 Session invalidation on password change
- [ ] 18.9 Encrypt users.json at rest (optional)
- [ ] 18.10 Add IP whitelisting for admin actions (optional)

## 19. Testing - Backend

- [ ] 14.1 Test login endpoint with valid credentials
- [ ] 14.2 Test login endpoint with invalid credentials
- [ ] 14.3 Test logout functionality
- [ ] 14.4 Test session verification
- [ ] 14.5 Test session refresh
- [ ] 14.6 Test JWT token validation
- [ ] 14.7 Test CSRF protection
- [ ] 14.8 Test rate limiting on login
- [ ] 14.9 Test tenant scoping in cluster mode

## 20. Testing - Frontend

- [ ] 20.1 Test login form validation
- [ ] 20.2 Test successful login flow
- [ ] 20.3 Test failed login handling
- [ ] 20.4 Test logout flow
- [ ] 20.5 Test protected routes
- [ ] 20.6 Test admin routes (user management)
- [ ] 20.7 Test session expiration handling
- [ ] 20.8 Test remember me functionality
- [ ] 20.9 Test user management UI (create, delete, change password)
- [ ] 20.10 Test API key management UI (create, revoke, copy)
- [ ] 20.11 Test API key permissions display
- [ ] 20.12 Test key-only-shown-once behavior
- [ ] 20.13 E2E tests with Playwright

## 21. Testing - Integration

- [ ] 16.1 Test local auth mode end-to-end
- [ ] 16.2 Test HiveHub auth mode end-to-end
- [ ] 16.3 Test development mode (no auth)
- [ ] 16.4 Test mode switching
- [ ] 16.5 Test tenant isolation in cluster mode
- [ ] 16.6 Test concurrent sessions
- [ ] 16.7 Test session hijacking prevention

## 22. Documentation

- [ ] 22.1 Create docs/specs/DASHBOARD_AUTH.md
- [ ] 22.2 Document authentication modes (local, hivehub, none)
- [ ] 22.3 Document configuration options
- [ ] 22.4 Create user guide for dashboard login
- [ ] 22.5 Document password management and user creation
- [ ] 22.6 Add security best practices
- [ ] 22.7 Document CLI arguments (--ROOT_USER, --ROOT_PASSWORD)
- [ ] 22.8 Document secrets/ directory structure
- [ ] 22.9 Document Docker deployment with secrets
- [ ] 22.10 Update README with auth requirements
- [ ] 22.11 Update CHANGELOG

## 23. User Experience

- [ ] 18.1 Add password strength indicator
- [ ] 18.2 Add "Forgot password" flow (optional)
- [ ] 18.3 Add session timeout warning (5 min before expiry)
- [ ] 18.4 Add auto-logout on inactivity (configurable)
- [ ] 18.5 Smooth transitions between login and dashboard
- [ ] 18.6 Mobile-responsive login page

## 24. Monitoring & Logging

- [ ] 24.1 Add metric: dashboard_login_attempts_total
- [ ] 24.2 Add metric: dashboard_login_failures_total
- [ ] 24.3 Add metric: dashboard_active_sessions
- [ ] 24.4 Add metric: dashboard_users_total
- [ ] 24.5 Add metric: dashboard_api_keys_total
- [ ] 24.6 Add metric: dashboard_api_key_requests_total (by key_id)
- [ ] 24.7 Log all login attempts with IP and user
- [ ] 24.8 Log all logout events
- [ ] 24.9 Log all user management operations
- [ ] 24.10 Log all API key operations (create, revoke)
- [ ] 24.11 Log root user creation
- [ ] 24.12 Log API key usage (every request)
- [ ] 24.13 Alert on suspicious login patterns
- [ ] 24.14 Alert on multiple failed logins
- [ ] 24.15 Alert on API key with excessive usage

## 25. Verification & Launch

- [ ] 20.1 Security audit of auth implementation
- [ ] 20.2 Penetration testing
- [ ] 20.3 Code review
- [ ] 20.4 Performance testing (login latency < 100ms)
- [ ] 20.5 Cross-browser testing
- [ ] 20.6 Documentation review
- [ ] 20.7 Beta testing with real users
- [ ] 20.8 Production deployment
