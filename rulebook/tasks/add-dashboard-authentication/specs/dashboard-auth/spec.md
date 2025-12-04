# Dashboard Authentication Specification

## ADDED Requirements

### Requirement: Dashboard Authentication System
The system SHALL implement authentication for the dashboard in production and cluster modes.

##### Scenario: Development Mode - No Authentication
Given the server is running in development mode
And dashboard.auth.enabled is false or mode is "development"
When a user accesses the dashboard URL
Then the system MUST allow direct access without login
And the system MUST display a warning banner "DEVELOPMENT MODE - NO AUTHENTICATION"

##### Scenario: Production Mode - Local Authentication Required
Given the server is running in production mode
And dashboard.auth.mode is "local"
When an unauthenticated user accesses the dashboard URL
Then the system MUST redirect to /login
And the system MUST display a login form
And the system MUST require username and password

##### Scenario: Cluster Mode - HiveHub Authentication Required
Given the server is running in cluster mode
And dashboard.auth.mode is "hivehub"
When an unauthenticated user accesses the dashboard URL
Then the system MUST redirect to /login
And the system MUST display API key input form
And the system MUST validate API key with HiveHub

### Requirement: Local Authentication (Production Mode)
The system SHALL support local user/password authentication for production deployments.

##### Scenario: Login with Valid Credentials
Given a configured admin user with username "admin" and password "SecurePass123!"
When the user submits login form with correct credentials
Then the system MUST validate the password against bcrypt hash
And the system MUST generate a JWT session token
And the system MUST set secure HTTP-only cookie
And the system MUST redirect to /dashboard
And the session MUST be valid for configured duration (default 24 hours)

##### Scenario: Login with Invalid Credentials
Given a user attempts login with invalid password
When the user submits login form
Then the system MUST return 401 Unauthorized
And the system MUST increment failed login counter
And the system MUST NOT reveal whether username or password is wrong
And the system MUST log the failed attempt with IP address

##### Scenario: Brute Force Protection
Given a user has failed login 5 times in 5 minutes
When the user attempts another login
Then the system MUST return 429 Too Many Requests
And the system MUST block login attempts for 15 minutes
And the system MUST log the brute-force attempt

### Requirement: HiveHub Authentication (Cluster Mode)
The system SHALL integrate with HiveHub authentication for cluster deployments.

##### Scenario: Login with Valid HiveHub API Key
Given cluster mode is enabled
And a user has valid HiveHub API key "hh_user_abc_key123"
When the user submits API key in login form
Then the system MUST validate key with HiveHub API
And the system MUST extract tenant_id from validation response
And the system MUST create tenant-scoped session
And the system MUST redirect to tenant-scoped dashboard

##### Scenario: Login with Invalid HiveHub API Key
Given a user enters invalid API key "hh_invalid_key"
When the user submits the form
Then the system MUST return 401 Unauthorized
And the system MUST display error "Invalid API key"
And the system MUST NOT create any session

##### Scenario: Tenant Scoping in Dashboard
Given a user logged in with tenant_id "tenant_alice"
When the user accesses any dashboard page
Then the system MUST filter all data by owner_id = "tenant_alice"
And the system MUST NOT show collections from other tenants
And the system MUST display tenant info in header

### Requirement: Session Management
The system SHALL implement secure session management with JWT tokens.

##### Scenario: Session Token Generation
Given a successful login
When the system generates a session token
Then the token MUST be a valid JWT
And the token MUST include user_id, tenant_id (if applicable), role
And the token MUST have expiration time (iat + duration)
And the token MUST be signed with secret key
And the token MUST be stored in HTTP-only secure cookie

##### Scenario: Session Verification
Given a user has valid session token
When the user accesses a protected dashboard route
Then the system MUST verify JWT signature
And the system MUST check token expiration
And the system MUST allow access if valid
And the system MUST return 401 if invalid or expired

##### Scenario: Session Refresh
Given a user has session token expiring in 1 hour
When the user makes any API request
Then the system MUST check if token expires soon (< 10% of duration)
And the system MUST automatically refresh the token
And the system MUST return new token in response
And the session MUST extend by full duration

##### Scenario: Logout
Given a user is logged in
When the user clicks logout
Then the system MUST invalidate the session token
And the system MUST clear the session cookie
And the system MUST redirect to /login
And the system MUST log the logout event

### Requirement: Protected Routes
The system SHALL protect all dashboard routes except authentication endpoints.

##### Scenario: Access Protected Route Without Auth
Given a user is not logged in
When the user tries to access /dashboard
Then the system MUST redirect to /login
And the system MUST preserve intended route in redirect URL
And the system MUST redirect back after successful login

##### Scenario: Access Auth Endpoint Without Token
Given a user is not logged in
When the user accesses /login or /api/dashboard/auth/login
Then the system MUST allow access
And the system MUST NOT require authentication

### Requirement: Security Headers
The system SHALL add security headers to all dashboard responses.

##### Scenario: Security Headers on All Responses
Given any dashboard request
When the server sends response
Then the response MUST include:
- `Content-Security-Policy: default-src 'self'`
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Strict-Transport-Security: max-age=31536000` (HTTPS only)
- `X-XSS-Protection: 1; mode=block`

### Requirement: API Key Management
The system SHALL provide API key management for MCP and programmatic access.

##### Scenario: Create API Key
Given an authenticated admin user
When the user creates a new API key with name "Production Key" and permissions ["read", "write"]
Then the system MUST generate a secure random key with format "vec_sk_{32_random_chars}"
And the system MUST hash the key with SHA-256 for storage
And the system MUST store key metadata (name, permissions, owner_id, created_at)
And the system MUST return the unhashed key ONLY once
And the system MUST display warning "Save this key - it won't be shown again"

##### Scenario: List API Keys
Given an authenticated user with 3 API keys
When the user requests list of API keys
Then the system MUST return only keys owned by that user
And the system MUST include metadata (name, permissions, created_at, last_used)
And the system MUST NOT return the actual key values (only hashed)
And the system MUST include usage statistics (request_count)

##### Scenario: Revoke API Key
Given an API key exists with id "key_123"
When the user revokes the key
Then the system MUST delete the key from storage
And the system MUST invalidate any cached key data
And future requests with that key MUST return 401 Unauthorized
And the system MUST log the revocation event

##### Scenario: Use API Key for MCP Access
Given a valid API key "vec_sk_abc123..."
When a client sends MCP request with header "x-api-key: vec_sk_abc123..."
Then the system MUST validate the key against hashed storage
And the system MUST extract user_id and permissions from key metadata
And the system MUST allow request if permissions match operation
And the system MUST update last_used timestamp
And the system MUST increment usage_count

##### Scenario: API Key with Expired Date
Given an API key with expiration date in the past
When a client attempts to use the key
Then the system MUST return 401 Unauthorized
And the system MUST return error "API key has expired"
And the system MUST log the expired key usage attempt

##### Scenario: API Key Without Sufficient Permissions
Given an API key with permissions ["read"]
When a client attempts a write operation (insert, update, delete)
Then the system MUST return 403 Forbidden
And the system MUST return error "Insufficient permissions"
And the system MUST log the permission violation

### Requirement: Root User Auto-Creation
The system SHALL automatically create a root user on first startup if no admin users exist.

##### Scenario: First Startup Without Root User
Given the server is starting for the first time
And secrets/users.json does not exist
And no --ROOT_USER or --ROOT_PASSWORD arguments provided
When the server initializes
Then the system MUST create secrets/ directory
And the system MUST generate a secure random password (16 characters)
And the system MUST create root user with username "root"
And the system MUST hash password with bcrypt
And the system MUST save to secrets/users.json
And the system MUST print credentials to console:
```
╔════════════════════════════════════════════════════╗
║  ROOT USER CREATED - SAVE THESE CREDENTIALS       ║
╠════════════════════════════════════════════════════╣
║  Username: root                                    ║
║  Password: Xy8#kL9mP2qN5vB3                       ║
╠════════════════════════════════════════════════════╣
║  ⚠️  IMPORTANT: Change this password immediately   ║
║  Access dashboard at http://localhost:3000         ║
╚════════════════════════════════════════════════════╝
```
And the system MUST add to startup logs

##### Scenario: First Startup With CLI Arguments
Given the server is starting for the first time
And --ROOT_USER admin --ROOT_PASSWORD SecurePass123! provided
When the server initializes
Then the system MUST create root user with username "admin"
And the system MUST use provided password "SecurePass123!"
And the system MUST validate password complexity
And the system MUST print confirmation to console (without showing password)
And the system MUST NOT generate random password

##### Scenario: First Startup With Environment Variables
Given environment variables ROOT_USER="admin" and ROOT_PASSWORD="SecurePass123!"
When the server starts
Then the system MUST create root user from environment variables
And the system MUST NOT print password to console (security)
And the system MUST log "Root user created from environment variables"

##### Scenario: Startup With Existing Users
Given secrets/users.json contains admin users
When the server starts
Then the system MUST NOT create root user
And the system MUST load existing users
And the system MUST NOT print any credentials

### Requirement: CSRF Protection
The system SHALL implement CSRF protection for all state-changing operations.

##### Scenario: CSRF Token Generation
Given a user logs in successfully
When the system creates the session
Then the system MUST generate a CSRF token
And the system MUST include CSRF token in response
And the system MUST require token for all POST/PUT/DELETE requests

##### Scenario: Request with Invalid CSRF Token
Given a POST request to dashboard API
And the request has invalid or missing CSRF token
When the server processes the request
Then the system MUST return 403 Forbidden
And the system MUST log CSRF violation attempt

## Configuration Requirements

### DashboardAuthConfig Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAuthConfig {
    /// Enable authentication (auto-enabled in production/cluster)
    pub enabled: bool,
    
    /// Authentication mode
    pub mode: AuthMode,
    
    /// Local auth configuration
    pub local: LocalAuthConfig,
    
    /// HiveHub auth configuration
    pub hivehub: HiveHubAuthConfig,
    
    /// Session configuration
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// No authentication (development only)
    None,
    /// Local username/password
    Local,
    /// HiveHub API key
    HiveHub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthConfig {
    /// Admin users
    pub users: Vec<DashboardUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUser {
    pub username: String,
    pub password_hash: String,  // bcrypt hash
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// JWT secret key (from env DASHBOARD_SESSION_SECRET)
    pub secret: String,
    
    /// Session duration in hours
    pub duration_hours: u32,
    
    /// Use secure cookies (HTTPS only)
    pub secure_cookie: bool,
    
    /// Enable CSRF protection
    pub csrf_enabled: bool,
}
```

### config.yml Format
```yaml
server:
  dashboard:
    enabled: true
    port: 3000
    
    # Authentication configuration
    auth:
      # Auto-enabled in production/cluster modes
      enabled: true
      
      # Authentication mode: "none" | "local" | "hivehub"
      mode: "local"
      
      # Local authentication (production mode)
      local:
        users:
          - username: "admin"
            password_hash: "$2b$10$..." # bcrypt hash
            role: "admin"
          - username: "viewer"
            password_hash: "$2b$10$..."
            role: "viewer"
      
      # HiveHub authentication (cluster mode)
      hivehub:
        # Auto-enabled when cluster.enabled = true
        enabled: true
      
      # Session configuration
      session:
        secret: "${DASHBOARD_SESSION_SECRET}"  # Required env var
        duration_hours: 24
        secure_cookie: true  # HTTPS only in production
        csrf_enabled: true
```

## API Endpoints

### POST /api/dashboard/auth/login
**Purpose**: Authenticate user and create session

**Request (Local Mode)**:
```json
{
  "username": "admin",
  "password": "SecurePass123!",
  "remember_me": true
}
```

**Request (HiveHub Mode)**:
```json
{
  "api_key": "hh_user_abc_key123"
}
```

**Response Success (200)**:
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "username": "admin",
    "role": "admin",
    "tenant_id": "tenant_alice"  // Only in cluster mode
  },
  "csrf_token": "csrf_xyz123",
  "expires_at": "2025-12-05T14:30:00Z"
}
```

**Response Error (401)**:
```json
{
  "error": "Authentication failed",
  "message": "Invalid credentials"
}
```

### POST /api/dashboard/auth/logout
**Purpose**: Invalidate session and logout

**Request**: Empty (token from cookie)

**Response (200)**:
```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

### GET /api/dashboard/auth/verify
**Purpose**: Verify session is still valid

**Response Valid (200)**:
```json
{
  "valid": true,
  "user": {
    "username": "admin",
    "role": "admin"
  },
  "expires_at": "2025-12-05T14:30:00Z"
}
```

**Response Invalid (401)**:
```json
{
  "valid": false,
  "error": "Session expired"
}
```

### POST /api/dashboard/auth/refresh
**Purpose**: Refresh session token

**Response (200)**:
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2025-12-06T14:30:00Z"
}
```

## Performance Requirements

### Login Performance
- Login endpoint MUST respond within 100ms (local mode)
- Login endpoint MUST respond within 500ms (HiveHub mode, includes API call)
- Session verification MUST complete within 10ms
- JWT token generation MUST complete within 5ms

### Scalability
- System MUST support 1,000 concurrent dashboard sessions
- System MUST handle 100 login requests/second
- Session storage MUST not exceed 10MB for 1,000 sessions

## Security Requirements

### Password Security
- Passwords MUST be hashed with bcrypt (cost factor 10+)
- Password MUST never be logged or stored in plaintext
- Password complexity: minimum 8 characters, 1 uppercase, 1 number

### Token Security
- JWT tokens MUST use HS256 or RS256 algorithm
- JWT secret MUST be at least 32 bytes
- JWT secret MUST be stored in environment variable
- Tokens MUST expire after configured duration

### Cookie Security
- Session cookies MUST have HttpOnly flag
- Session cookies MUST have Secure flag (HTTPS)
- Session cookies MUST have SameSite=Strict
- CSRF tokens MUST be validated on state-changing operations

## Testing Requirements

### Unit Tests
- Auth middleware MUST have 100% coverage
- Login endpoint MUST test all error cases
- JWT generation/validation MUST have 100% coverage

### Integration Tests
- End-to-end login flow MUST be tested
- Tenant scoping MUST be verified (no data leakage)
- Session expiration MUST be tested

### Security Tests
- Brute-force protection MUST be tested
- CSRF protection MUST be tested
- XSS prevention MUST be tested
- Session hijacking prevention MUST be tested

