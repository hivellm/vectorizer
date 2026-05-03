# Proposal: add-dashboard-authentication

## Why

The Vectorizer Dashboard is currently accessible without authentication, which poses serious security risks in production and cluster deployments:

1. **Security Risk**: Anyone can access the dashboard and view/modify all data
2. **Multi-Tenant Violation**: In cluster mode, users can see other tenants' data
3. **Compliance Issue**: No audit trail of who accessed what
4. **Production Deployment Blocker**: Cannot safely expose dashboard to internet

**Current Behavior**:
- Dashboard runs on port configured in `server.dashboard_port`
- No authentication layer
- Full access to all collections and data
- No user/tenant isolation

**Problem Scenarios**:
- In **development mode**: No auth is acceptable (localhost only)
- In **production mode**: Dashboard exposed without auth = security breach
- In **cluster mode**: Multiple tenants can see each other's data

## What Changes

### 1. Add Authentication Layer to Dashboard

Implement login system that integrates with existing auth:

**For Production Mode**:
- Use local user/password authentication
- Support multiple admin users via config
- Session-based authentication with JWT tokens
- Login page before dashboard access

**For Cluster Mode (HiveHub)**:
- Integrate with HiveHub authentication
- Use HiveHub API keys for login
- Tenant-scoped dashboard (show only user's data)
- Support HiveHub SSO (future)

### 2. User Management System

Implement complete user management:
- **Root user auto-creation** on first startup
  - Check if any admin user exists
  - Create root user if none found
  - Print credentials to console (one-time)
  - Support `--ROOT_USER` and `--ROOT_PASSWORD` CLI args
  
- **User storage** in `secrets/users.json`
  - Store in `secrets/` directory (added to .gitignore)
  - JSON format with user list
  - Encrypted passwords (bcrypt)
  - File permissions: 600 (owner read/write only)
  
- **Admin UI for user management**
  - Create new admin users
  - Delete users (except root)
  - Change passwords
  - List all users

### 3. API Key Management for MCP Access

Implement API key management for MCP/programmatic access:
- **API Key Generation**
  - Generate secure random API keys (format: `vec_sk_...`)
  - Associate with user account
  - Set permissions (read, write, admin)
  - Set expiration (optional, default: never)
  - Name/description for each key
  
- **API Key Storage** in `secrets/api_keys.json`
  - Store hashed keys (SHA-256)
  - Store metadata (created_at, last_used, permissions)
  - Link to user account
  - Track usage statistics
  
- **Dashboard UI for API Keys**
  - List all API keys for logged-in user
  - Create new API key with permissions
  - Revoke/delete API keys
  - Copy key to clipboard (show only on creation)
  - Show last used date
  - Show usage statistics (requests count)
  
- **MCP Integration**
  - Use API keys for MCP authentication (instead of dashboard session)
  - Support `x-api-key` header in MCP requests
  - Validate API key permissions for each operation
  - Track API key usage for billing/monitoring

### 3. Login Page Implementation

Create React login page:
- Email/password form (production mode)
- API key input (cluster mode)
- Remember me option (secure cookie)
- Redirect to dashboard after successful login
- Logout functionality

### 3. Session Management

Implement secure session handling:
- JWT tokens with expiration (default: 24 hours)
- Secure HTTP-only cookies
- CSRF protection
- Session renewal/refresh
- Logout clears session

### 4. Dashboard Backend Integration

Update dashboard server:
- Add `/api/auth/login` endpoint
- Add `/api/auth/logout` endpoint
- Add `/api/auth/verify` endpoint (check session)
- Add `/api/auth/refresh` endpoint (renew token)
- Protect all dashboard API routes with auth middleware

### 5. Mode-Specific Configuration

Add dashboard auth config:

```yaml
server:
  dashboard:
    enabled: true
    port: 3000
    
    # Authentication (required in production/cluster)
    auth:
      enabled: true  # Auto-enabled if mode != development
      mode: "local"  # "local" | "hivehub"
      
      # Local auth (production mode)
      local:
        # Users file path (stored in secrets/)
        users_file: "./secrets/users.json"
        
        # Auto-create root user on first startup
        auto_create_root: true
        
        # Default root credentials (override with CLI args)
        root_username: "root"
        root_password: "${ROOT_PASSWORD}"  # Or random if not set
      
      # HiveHub auth (cluster mode)
      hivehub:
        enabled: true  # Auto-enabled if cluster.enabled = true
        
      # Session settings
      session:
        secret: "${DASHBOARD_SESSION_SECRET}"
        duration_hours: 24
        secure_cookie: true  # HTTPS only
        csrf_enabled: true
```

### 6. CLI Arguments for Docker/K8s

Support environment-based root user creation:

```bash
# Docker example
docker run -e ROOT_USER=admin -e ROOT_PASSWORD=SecurePass123! vectorizer

# CLI flags
./vectorizer --ROOT_USER admin --ROOT_PASSWORD SecurePass123!

# Kubernetes secret
kubectl create secret generic vectorizer-root \
  --from-literal=ROOT_USER=admin \
  --from-literal=ROOT_PASSWORD=SecurePass123!
```

### 6. Tenant Scoping in Cluster Mode

When logged in via HiveHub API key:
- Extract `tenant_id` from API key
- Filter all dashboard data by `owner_id`
- Show only user's collections
- Prevent access to other tenants' data
- Display tenant info in dashboard header

### 7. Development Mode

Preserve developer experience:
- If `mode = development`: No auth required (default behavior)
- Show warning banner: "DEVELOPMENT MODE - NO AUTH"
- Easy toggle for local development

## Impact

### Affected Specs
- `docs/specs/DASHBOARD.md` - TO CREATE: Dashboard auth specification
- `docs/specs/API_REFERENCE.md` - Auth endpoints
- `docs/specs/SECURITY.md` - Security requirements

### Affected Code
- `dashboard/src/` - React login page, auth context, user management UI, API key UI
- `dashboard/src/components/auth/` - TO CREATE: Login components
- `dashboard/src/pages/Users.tsx` - TO CREATE: User management page
- `dashboard/src/pages/ApiKeys.tsx` - TO CREATE: API key management page
- `dashboard/src/api/` - Auth API client, API keys client
- `src/server/dashboard_auth.rs` - TO CREATE: Dashboard auth with user storage
- `src/server/dashboard_users.rs` - TO CREATE: User management endpoints
- `src/server/dashboard_api_keys.rs` - TO CREATE: API key management endpoints
- `src/server/mod.rs` - Integrate dashboard auth, CLI args, API key validation
- `src/bin/vectorizer.rs` - Add --ROOT_USER and --ROOT_PASSWORD args
- `secrets/users.json` - TO CREATE: User database (gitignored)
- `secrets/api_keys.json` - TO CREATE: API keys database (gitignored)
- `.gitignore` - Add secrets/ directory
- `config.example.yml` - Dashboard auth configuration

### Breaking Change
**YES** - For production deployments:

**Before** (unsafe):
```yaml
server:
  dashboard:
    enabled: true
    port: 3000
# No authentication!
```

**After** (secure):
```yaml
server:
  dashboard:
    enabled: true
    port: 3000
    auth:
      enabled: true  # Required in production
      mode: "local"
      local:
        users:
          - username: "admin"
            password_hash: "$2b$10$..."
```

**Migration Path**:
1. Version N: Add auth with default disabled (warnings)
2. Version N+1: Enable by default in production
3. Version N+2: Mandatory in production/cluster

### User Benefit

**For Production Deployments**:
- ✅ Secure dashboard access (no unauthorized access)
- ✅ Audit trail of who accessed what
- ✅ Compliance with security standards
- ✅ Safe internet exposure

**For Cluster/Multi-Tenant**:
- ✅ Tenant isolation (users see only their data)
- ✅ HiveHub integration (single sign-on)
- ✅ Per-tenant dashboard experience
- ✅ No data leakage between tenants

**For Developers**:
- ✅ Development mode unchanged (no auth locally)
- ✅ Easy testing without login hassle
- ✅ Clear mode indicator in UI

**Example**:
- **Development**: `http://localhost:3000` → Dashboard (no login)
- **Production**: `https://vectorizer.company.com` → Login page → Dashboard
- **Cluster**: `https://cluster.hivehub.cloud` → HiveHub login → Tenant-scoped dashboard
