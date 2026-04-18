# Dashboard Authentication - Technical Design

## Overview

This document describes the technical implementation of authentication for the Vectorizer Dashboard, supporting development (no auth), production (local auth), and cluster (HiveHub auth) modes.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Dashboard Frontend (React)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                Login Page (New)                            │ │
│  │  ┌──────────────────┬───────────────────────────────────┐ │ │
│  │  │  Local Mode      │  HiveHub Mode                     │ │ │
│  │  │  - Username      │  - API Key Input                  │ │ │
│  │  │  - Password      │  - Validate with HiveHub          │ │ │
│  │  │  - Remember Me   │  - Extract tenant_id              │ │ │
│  │  └──────────────────┴───────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              AuthContext (React Context)                   │ │
│  │  - currentUser state                                       │ │
│  │  - login() / logout() functions                           │ │
│  │  - Auto-refresh session                                    │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              ProtectedRoute Component                      │ │
│  │  - Wraps all dashboard routes                             │ │
│  │  - Checks authentication                                   │ │
│  │  - Redirects to /login if not authenticated               │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                Dashboard Pages                             │ │
│  │  - Collections, Vectors, Search, Settings                 │ │
│  │  - Filtered by tenant in cluster mode                     │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                │ HTTP + JWT Cookie
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Dashboard Backend (Rust/Axum)                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │         Dashboard Auth Middleware (New)                    │ │
│  │  1. Extract JWT from cookie                               │ │
│  │  2. Validate JWT signature                                │ │
│  │  3. Check expiration                                       │ │
│  │  4. Extract user/tenant context                           │ │
│  │  5. Allow or reject request                               │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │            Auth Endpoints (New)                            │ │
│  │  POST   /api/dashboard/auth/login                         │ │
│  │  POST   /api/dashboard/auth/logout                        │ │
│  │  GET    /api/dashboard/auth/verify                        │ │
│  │  POST   /api/dashboard/auth/refresh                       │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │         Dashboard API Routes (Protected)                   │ │
│  │  - All existing dashboard endpoints                        │ │
│  │  - Now require valid session                              │ │
│  │  - Tenant-scoped in cluster mode                          │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                    ┌───────────────────────┐
                    │   VectorStore         │
                    │   (Tenant-filtered)   │
                    └───────────────────────┘
```

## Implementation Details

### 1. Backend - Dashboard Auth Module

**Location**: `src/server/dashboard_auth.rs` (NEW)

```rust
//! Dashboard authentication module
//!
//! Provides authentication for dashboard access in production/cluster modes.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{State, Json},
    http::{StatusCode, header::{SET_COOKIE, HeaderMap}},
    response::IntoResponse,
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::error::{Result, VectorizerError};
use crate::hub::HubManager;

/// Dashboard authentication manager
pub struct DashboardAuth {
    config: DashboardAuthConfig,
    hub_manager: Option<Arc<HubManager>>,
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttempts>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAuthConfig {
    pub enabled: bool,
    pub mode: AuthMode,
    pub local: LocalAuthConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    None,       // Development only
    Local,      // Production mode
    HiveHub,    // Cluster mode
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthConfig {
    pub users: Vec<DashboardUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUser {
    pub username: String,
    pub password_hash: String,  // bcrypt
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
    pub secret: String,           // JWT secret
    pub duration_hours: u32,      // Default: 24
    pub secure_cookie: bool,      // HTTPS only
    pub csrf_enabled: bool,       // CSRF protection
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,              // username or user_id
    pub tenant_id: Option<String>, // For cluster mode
    pub role: UserRole,
    pub exp: u64,                 // Expiration timestamp
    pub iat: u64,                 // Issued at timestamp
}

impl DashboardAuth {
    pub fn new(
        config: DashboardAuthConfig,
        hub_manager: Option<Arc<HubManager>>,
    ) -> Self {
        Self {
            config,
            hub_manager,
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Handle login request
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        match self.config.mode {
            AuthMode::None => {
                Err(VectorizerError::ConfigurationError(
                    "Authentication is disabled".to_string()
                ))
            }
            AuthMode::Local => {
                self.login_local(&request.username, &request.password).await
            }
            AuthMode::HiveHub => {
                self.login_hivehub(&request.api_key).await
            }
        }
    }
    
    async fn login_local(
        &self,
        username: &Option<String>,
        password: &Option<String>,
    ) -> Result<LoginResponse> {
        let username = username.as_ref()
            .ok_or_else(|| VectorizerError::AuthenticationError("Username required".to_string()))?;
        let password = password.as_ref()
            .ok_or_else(|| VectorizerError::AuthenticationError("Password required".to_string()))?;
        
        // Check brute-force protection
        if self.is_locked_out(username) {
            return Err(VectorizerError::RateLimitExceeded {
                limit_type: "login_attempts".to_string(),
                limit: 5,
            });
        }
        
        // Find user
        let user = self.config.local.users.iter()
            .find(|u| u.username == *username)
            .ok_or_else(|| {
                self.record_failed_attempt(username);
                VectorizerError::AuthenticationError("Invalid credentials".to_string())
            })?;
        
        // Verify password
        if !bcrypt::verify(password, &user.password_hash).unwrap_or(false) {
            self.record_failed_attempt(username);
            return Err(VectorizerError::AuthenticationError("Invalid credentials".to_string()));
        }
        
        // Clear failed attempts
        self.clear_failed_attempts(username);
        
        // Generate JWT token
        let token = self.generate_token(username, None, user.role.clone())?;
        
        info!("User {} logged in successfully", username);
        
        Ok(LoginResponse {
            token,
            user: UserInfo {
                username: username.clone(),
                role: user.role.clone(),
                tenant_id: None,
            },
        })
    }
    
    async fn login_hivehub(&self, api_key: &Option<String>) -> Result<LoginResponse> {
        let api_key = api_key.as_ref()
            .ok_or_else(|| VectorizerError::AuthenticationError("API key required".to_string()))?;
        
        let hub_manager = self.hub_manager.as_ref()
            .ok_or_else(|| VectorizerError::ConfigurationError("HiveHub not configured".to_string()))?;
        
        // Validate API key with HiveHub
        let tenant_context = hub_manager.validate_api_key(api_key).await?;
        
        // Generate JWT token with tenant context
        let token = self.generate_token(
            &tenant_context.tenant_id,
            Some(tenant_context.tenant_id.clone()),
            UserRole::Admin,  // All HiveHub users are admins of their tenant
        )?;
        
        info!("HiveHub user {} logged in successfully", tenant_context.tenant_id);
        
        Ok(LoginResponse {
            token,
            user: UserInfo {
                username: tenant_context.tenant_name.clone(),
                role: UserRole::Admin,
                tenant_id: Some(tenant_context.tenant_id.clone()),
            },
        })
    }
    
    fn generate_token(
        &self,
        username: &str,
        tenant_id: Option<String>,
        role: UserRole,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let exp = now + (self.config.session.duration_hours as u64 * 3600);
        
        let claims = JwtClaims {
            sub: username.to_string(),
            tenant_id,
            role,
            exp,
            iat: now,
        };
        
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.session.secret.as_bytes()),
        )
        .map_err(|e| VectorizerError::AuthenticationError(format!("Token generation failed: {}", e)))
    }
    
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.config.session.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| VectorizerError::AuthenticationError(format!("Invalid token: {}", e)))
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    // Local mode
    pub username: Option<String>,
    pub password: Option<String>,
    pub remember_me: Option<bool>,
    
    // HiveHub mode
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub role: UserRole,
    pub tenant_id: Option<String>,
}
```

### 2. Frontend - Auth Context

**Location**: `dashboard/src/contexts/AuthContext.tsx` (NEW)

```typescript
import React, { createContext, useContext, useState, useEffect } from 'react';
import axios from 'axios';

interface User {
  username: string;
  role: 'admin' | 'viewer';
  tenantId?: string;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => Promise<void>;
  verifySession: () => Promise<boolean>;
}

interface LoginCredentials {
  username?: string;
  password?: string;
  apiKey?: string;
  rememberMe?: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // Verify session on mount
  useEffect(() => {
    verifySession();
  }, []);

  // Auto-refresh session every hour
  useEffect(() => {
    if (user) {
      const interval = setInterval(() => {
        refreshSession();
      }, 3600000); // 1 hour
      
      return () => clearInterval(interval);
    }
  }, [user]);

  const login = async (credentials: LoginCredentials) => {
    try {
      const response = await axios.post('/api/dashboard/auth/login', credentials, {
        withCredentials: true, // Include cookies
      });
      
      const { user, token } = response.data;
      
      // Store token in localStorage if remember me
      if (credentials.rememberMe) {
        localStorage.setItem('dashboard_token', token);
      }
      
      setUser(user);
    } catch (error) {
      throw new Error('Login failed');
    }
  };

  const logout = async () => {
    try {
      await axios.post('/api/dashboard/auth/logout', {}, {
        withCredentials: true,
      });
    } finally {
      setUser(null);
      localStorage.removeItem('dashboard_token');
    }
  };

  const verifySession = async (): Promise<boolean> => {
    try {
      const response = await axios.get('/api/dashboard/auth/verify', {
        withCredentials: true,
      });
      
      if (response.data.valid) {
        setUser(response.data.user);
        return true;
      }
      
      return false;
    } catch {
      return false;
    } finally {
      setLoading(false);
    }
  };

  const refreshSession = async () => {
    try {
      await axios.post('/api/dashboard/auth/refresh', {}, {
        withCredentials: true,
      });
    } catch {
      // If refresh fails, logout
      logout();
    }
  };

  return (
    <AuthContext.Provider value={{ user, loading, login, logout, verifySession }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
```

### 3. Frontend - Login Page

**Location**: `dashboard/src/pages/Login.tsx` (NEW)

```typescript
import React, { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { useNavigate } from 'react-router-dom';

export function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [mode, setMode] = useState<'local' | 'hivehub'>('local');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // Local mode state
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [rememberMe, setRememberMe] = useState(false);

  // HiveHub mode state
  const [apiKey, setApiKey] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      if (mode === 'local') {
        await login({ username, password, rememberMe });
      } else {
        await login({ apiKey });
      }
      
      navigate('/dashboard');
    } catch (err) {
      setError('Login failed. Please check your credentials.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100">
      <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-8">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Vectorizer</h1>
          <p className="text-gray-600 mt-2">Dashboard Login</p>
        </div>

        {/* Mode Toggle */}
        <div className="flex gap-2 mb-6">
          <button
            onClick={() => setMode('local')}
            className={`flex-1 py-2 px-4 rounded ${
              mode === 'local' ? 'bg-blue-600 text-white' : 'bg-gray-200'
            }`}
          >
            Local
          </button>
          <button
            onClick={() => setMode('hivehub')}
            className={`flex-1 py-2 px-4 rounded ${
              mode === 'hivehub' ? 'bg-blue-600 text-white' : 'bg-gray-200'
            }`}
          >
            HiveHub
          </button>
        </div>

        <form onSubmit={handleSubmit}>
          {mode === 'local' ? (
            <>
              <div className="mb-4">
                <label className="block text-gray-700 mb-2">Username</label>
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="w-full px-4 py-2 border rounded"
                  required
                />
              </div>
              <div className="mb-4">
                <label className="block text-gray-700 mb-2">Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full px-4 py-2 border rounded"
                  required
                />
              </div>
              <div className="mb-6">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={rememberMe}
                    onChange={(e) => setRememberMe(e.target.checked)}
                    className="mr-2"
                  />
                  <span className="text-gray-700">Remember me</span>
                </label>
              </div>
            </>
          ) : (
            <div className="mb-6">
              <label className="block text-gray-700 mb-2">HiveHub API Key</label>
              <input
                type="password"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="hh_user_..."
                className="w-full px-4 py-2 border rounded font-mono text-sm"
                required
              />
            </div>
          )}

          {error && (
            <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={loading}
            className="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Logging in...' : 'Login'}
          </button>
        </form>
      </div>
    </div>
  );
}
```

### 4. Auth Middleware

**Location**: `src/server/dashboard_auth.rs` (continued)

```rust
/// Dashboard auth middleware
pub async fn dashboard_auth_middleware(
    State(auth): State<Arc<DashboardAuth>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Skip auth in development mode
    if matches!(auth.config.mode, AuthMode::None) {
        return Ok(next.run(req).await);
    }
    
    // Skip auth for login/logout endpoints
    let path = req.uri().path();
    if path.starts_with("/api/dashboard/auth/") {
        return Ok(next.run(req).await);
    }
    
    // Extract JWT from cookie
    let token = req
        .headers()
        .get(header::COOKIE)
        .and_then(|cookies| {
            cookies.to_str().ok()
                .and_then(|s| extract_token_from_cookies(s))
        })
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Not authenticated".to_string(),
                }),
            )
        })?;
    
    // Verify token
    let claims = auth.verify_token(&token)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid or expired session".to_string(),
                }),
            )
        })?;
    
    // Add user context to request extensions
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}

fn extract_token_from_cookies(cookies: &str) -> Option<String> {
    cookies.split(';')
        .find(|c| c.trim().starts_with("dashboard_token="))
        .and_then(|c| c.split('=').nth(1))
        .map(|s| s.trim().to_string())
}
```

## API Key Management

### API Key Structure

```
Format: vec_sk_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
         │   │  └─ 32 random alphanumeric characters
         │   └──── Key type: sk (secret key)
         └──────── Prefix: vec (vectorizer)

Example: vec_sk_a7f3c9d2e8b1f4a6c3d8e2f9b4a7c3d1
```

### API Key Storage

**Location**: `secrets/api_keys.json`

```json
{
  "keys": [
    {
      "id": "key_01234567-89ab-cdef-0123-456789abcdef",
      "name": "Production MCP Key",
      "key_hash": "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "owner_id": "user_root",
      "permissions": ["read", "write"],
      "created_at": "2025-12-04T15:00:00Z",
      "last_used": "2025-12-04T16:30:00Z",
      "usage_count": 1523,
      "expires_at": null,
      "revoked": false
    }
  ]
}
```

### API Key Validation Flow

```rust
pub async fn validate_api_key(&self, key: &str) -> Result<ApiKeyContext> {
    // 1. Hash the provided key
    let key_hash = sha256(key);
    
    // 2. Look up in api_keys.json
    let api_key = self.find_key_by_hash(&key_hash)?;
    
    // 3. Check if revoked
    if api_key.revoked {
        return Err(VectorizerError::ApiKeyRevoked);
    }
    
    // 4. Check expiration
    if let Some(expires_at) = api_key.expires_at {
        if expires_at < Utc::now() {
            return Err(VectorizerError::ApiKeyExpired);
        }
    }
    
    // 5. Update last_used and usage_count
    self.update_key_usage(&api_key.id).await?;
    
    // 6. Return context
    Ok(ApiKeyContext {
        key_id: api_key.id,
        owner_id: api_key.owner_id,
        permissions: api_key.permissions,
    })
}
```

### User Management API Endpoints

#### POST /api/dashboard/users
**Purpose**: Create new dashboard user (admin only)

**Request**:
```json
{
  "username": "john.doe",
  "password": "SecurePass123!",
  "role": "admin"
}
```

**Response (201)**:
```json
{
  "success": true,
  "user": {
    "username": "john.doe",
    "role": "admin",
    "created_at": "2025-12-04T15:00:00Z"
  }
}
```

#### GET /api/dashboard/users
**Purpose**: List all users (admin only)

**Response (200)**:
```json
{
  "users": [
    {
      "username": "root",
      "role": "admin",
      "created_at": "2025-12-01T10:00:00Z",
      "is_root": true
    },
    {
      "username": "john.doe",
      "role": "viewer",
      "created_at": "2025-12-04T15:00:00Z",
      "is_root": false
    }
  ]
}
```

#### DELETE /api/dashboard/users/:username
**Purpose**: Delete user (admin only, cannot delete root)

**Response (200)**:
```json
{
  "success": true,
  "message": "User john.doe deleted successfully"
}
```

### API Key Management Endpoints

#### POST /api/dashboard/api-keys
**Purpose**: Create new API key

**Request**:
```json
{
  "name": "Production MCP Key",
  "permissions": ["read", "write"],
  "expires_in_days": 365  // Optional
}
```

**Response (201)**:
```json
{
  "success": true,
  "api_key": {
    "id": "key_01234567-89ab-cdef-0123-456789abcdef",
    "name": "Production MCP Key",
    "key": "vec_sk_a7f3c9d2e8b1f4a6c3d8e2f9b4a7c3d1",  // SHOWN ONLY ONCE
    "permissions": ["read", "write"],
    "created_at": "2025-12-04T15:00:00Z",
    "expires_at": "2026-12-04T15:00:00Z"
  },
  "warning": "This key will only be shown once. Please save it securely."
}
```

#### GET /api/dashboard/api-keys
**Purpose**: List user's API keys

**Response (200)**:
```json
{
  "api_keys": [
    {
      "id": "key_01234567-89ab-cdef-0123-456789abcdef",
      "name": "Production MCP Key",
      "permissions": ["read", "write"],
      "created_at": "2025-12-04T15:00:00Z",
      "last_used": "2025-12-04T16:30:00Z",
      "usage_count": 1523,
      "expires_at": "2026-12-04T15:00:00Z",
      "revoked": false
    }
  ]
}
```

#### DELETE /api/dashboard/api-keys/:id
**Purpose**: Revoke API key

**Response (200)**:
```json
{
  "success": true,
  "message": "API key revoked successfully"
}
```

### secrets/ Directory Structure

```
secrets/
├── README.md           # Warning and documentation (NOT gitignored)
├── .gitkeep            # Keep directory in git
├── users.json          # Dashboard users (gitignored, 600 permissions)
└── api_keys.json       # API keys (gitignored, 600 permissions)
```

**secrets/README.md** (not gitignored):
```markdown
# Secrets Directory

⚠️ **IMPORTANT**: This directory contains sensitive data!

## Files (gitignored):
- `users.json` - Dashboard user credentials
- `api_keys.json` - API keys for MCP/programmatic access

## Security:
- Files MUST have 600 permissions (owner read/write only)
- Directory MUST have 700 permissions (owner only)
- Never commit these files to git
- Backup regularly with encryption

## Backup:
```bash
# Backup secrets
tar -czf secrets-backup-$(date +%Y%m%d).tar.gz secrets/
gpg -c secrets-backup-*.tar.gz  # Encrypt with password
```
```

### Root User Auto-Creation

```rust
pub async fn ensure_root_user(
    config: &DashboardAuthConfig,
    cli_args: &CliArgs,
) -> Result<()> {
    let users_path = Path::new(&config.local.users_file);
    
    // Check if users file exists and has admin users
    if users_path.exists() {
        let users = UserStore::load(&users_path)?;
        if users.iter().any(|u| u.role == UserRole::Admin) {
            info!("Admin users exist, skipping root creation");
            return Ok(());
        }
    }
    
    // Determine root username and password
    let username = cli_args.root_user
        .or(env::var("ROOT_USER").ok())
        .unwrap_or_else(|| "root".to_string());
    
    let password = cli_args.root_password
        .or(env::var("ROOT_PASSWORD").ok())
        .unwrap_or_else(|| generate_secure_password(16));
    
    // Validate password complexity
    validate_password_complexity(&password)?;
    
    // Hash password
    let password_hash = bcrypt::hash(&password, 10)?;
    
    // Create root user
    let root_user = DashboardUser {
        username: username.clone(),
        password_hash,
        role: UserRole::Admin,
        created_at: Utc::now(),
        is_root: true,
    };
    
    // Save to file
    let mut users = UserStore::new();
    users.add(root_user);
    users.save(&users_path)?;
    
    // Set file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&users_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&users_path, perms)?;
    }
    
    // Print credentials (only if generated, not from env)
    if cli_args.root_password.is_none() && env::var("ROOT_PASSWORD").is_err() {
        print_root_credentials(&username, &password);
    } else {
        info!("Root user '{}' created successfully", username);
    }
    
    Ok(())
}

fn print_root_credentials(username: &str, password: &str) {
    println!("\n{}", "═".repeat(56));
    println!("║  ROOT USER CREATED - SAVE THESE CREDENTIALS         ║");
    println!("{}", "═".repeat(56));
    println!("║  Username: {:<43} ║", username);
    println!("║  Password: {:<43} ║", password);
    println!("{}", "═".repeat(56));
    println!("║  ⚠️  IMPORTANT: Change this password immediately     ║");
    println!("║  Access dashboard at http://localhost:3000          ║");
    println!("{}\n", "═".repeat(56));
}

fn generate_secure_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

### Security Considerations

### Password Hashing
- Use **bcrypt** with cost factor 10
- Never store passwords in plaintext
- Password hash format: `$2b$10$...` (60 characters)

### JWT Security
- Secret MUST be at least 32 bytes random
- Store in environment variable `DASHBOARD_SESSION_SECRET`
- Use HS256 algorithm
- Include expiration (exp) claim
- Validate signature on every request

### Cookie Security
```rust
// Set secure cookie
let cookie = format!(
    "dashboard_token={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
    token,
    duration_hours * 3600
);
```

### CSRF Protection
- Generate CSRF token on login
- Require `X-CSRF-Token` header on all POST/PUT/DELETE
- Validate CSRF token matches session

## Mode Detection

```rust
fn determine_auth_mode(config: &Config) -> AuthMode {
    if config.mode == "development" {
        return AuthMode::None;
    }
    
    if config.cluster.enabled {
        return AuthMode::HiveHub;
    }
    
    if config.mode == "production" {
        return AuthMode::Local;
    }
    
    // Default to no auth (development)
    AuthMode::None
}
```

## Migration Strategy

### Phase 1: Add Auth (Optional)
- Add auth system with `enabled: false` by default
- Users can opt-in
- Show warnings in production without auth

### Phase 2: Enable by Default in Production
- Auto-enable if `mode = production`
- Provide migration guide
- Generate default admin user

### Phase 3: Make Mandatory
- Fail to start if production without auth
- Mandatory in cluster mode always

## Testing Strategy

### Backend Tests
```rust
#[tokio::test]
async fn test_login_with_valid_credentials() {
    // Test successful login
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    // Test login failure
}

#[tokio::test]
async fn test_brute_force_protection() {
    // Test account lockout after 5 failed attempts
}

#[tokio::test]
async fn test_jwt_token_validation() {
    // Test token verify
}

#[tokio::test]
async fn test_session_expiration() {
    // Test expired token rejection
}
```

### Frontend Tests
```typescript
describe('Login Page', () => {
  it('should render login form', () => {});
  it('should submit credentials', () => {});
  it('should show error on failure', () => {});
  it('should redirect after success', () => {});
});

describe('Protected Routes', () => {
  it('should redirect to login if not authenticated', () => {});
  it('should allow access if authenticated', () => {});
});
```

## Performance Targets

- Login latency: < 100ms (local), < 500ms (HiveHub)
- Session verification: < 10ms
- Token generation: < 5ms
- Support 1,000 concurrent sessions
- Handle 100 login requests/second

