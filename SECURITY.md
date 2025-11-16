# Security Policy

**Version**: 1.3.0  
**Last Updated**: November 16, 2025

---

## Reporting Security Vulnerabilities

If you discover a security vulnerability, please report it privately:

- **Email**: security@hivellm.dev
- **Expected Response Time**: 48 hours
- **Public Disclosure**: After fix is released

**Please do NOT**:

- Open public GitHub issues for security vulnerabilities
- Disclose vulnerabilities before a fix is available
- Exploit vulnerabilities in production systems

---

## Security Features

### 1. Authentication

#### JWT Tokens

- **Algorithm**: RS256 (RSA with SHA-256)
- **Expiration**: Configurable (default: 24 hours)
- **Refresh**: Supported via token refresh endpoint

#### API Keys

- **Format**: UUID v4 (128-bit random)
- **Storage**: Hashed with bcrypt
- **Rotation**: Supported via API
- **Expiration**: Optional, configurable per key

### 2. Rate Limiting

Prevents API abuse and DoS attacks.

**Configuration** (`config.yml`):

```yaml
security:
  rate_limiting:
    enabled: true
    requests_per_second: 100
    burst_size: 200
```

**Limits**:

- **Per API Key**: 100 req/s (configurable)
- **Burst Capacity**: 200 requests (configurable)
- **Response**: HTTP 429 (Too Many Requests)

**Headers**:

```
HTTP/1.1 429 Too Many Requests
Retry-After: 1
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1698332400
```

### 3. TLS/mTLS

Encrypted communication for production deployments.

#### TLS Configuration

```yaml
security:
  tls:
    enabled: true
    cert_path: "/path/to/server.crt"
    key_path: "/path/to/server.key"
```

**Requirements**:

- TLS 1.3 minimum
- Strong cipher suites only
- Valid certificate from trusted CA

#### mTLS (Mutual TLS)

Client certificate authentication for high-security environments.

```yaml
security:
  tls:
    enabled: true
    mtls_enabled: true
    cert_path: "/path/to/server.crt"
    key_path: "/path/to/server.key"
    client_ca_path: "/path/to/client-ca.crt"
```

### 4. Audit Logging

Tracks all API calls for compliance and forensics.

```yaml
security:
  audit:
    enabled: true
    max_entries: 10000
    log_auth_attempts: true
    log_failed_requests: true
    log_admin_actions: true
```

**Logged Events**:

- All API requests (method, endpoint, status, duration)
- Authentication attempts (success and failures)
- Administrative actions (config changes, server restart)
- Permission checks (RBAC decisions)

**Audit Log Entry**:

```json
{
  "timestamp": "2025-10-25T10:30:45Z",
  "principal": "api-key-abc123",
  "method": "POST",
  "endpoint": "/collections",
  "status_code": 200,
  "duration_ms": 15,
  "client_ip": "192.168.1.100",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### 5. Role-Based Access Control (RBAC)

Fine-grained permissions for different user types.

```yaml
security:
  rbac:
    enabled: true
    default_role: "Viewer"
```

#### Predefined Roles

**Viewer** (Read-Only):

- ✅ List collections
- ✅ Read collection details
- ✅ Search vectors
- ✅ Get vector by ID
- ✅ View system stats
- ❌ Create/update/delete anything

**Editor** (Read/Write):

- ✅ All Viewer permissions
- ✅ Create collections
- ✅ Update collections
- ✅ Insert vectors
- ✅ Update vectors
- ✅ Delete vectors
- ✅ Batch operations
- ❌ Delete collections
- ❌ Admin actions

**Admin** (Full Access):

- ✅ All Editor permissions
- ✅ Delete collections
- ✅ Manage API keys
- ✅ View audit logs
- ✅ Configure server
- ✅ Manage replication
- ✅ View metrics
- ✅ Restart server
- ✅ Backup/restore data

### 6. Enhanced Security Features

Advanced security capabilities for high-security environments.

#### Multi-Factor Authentication (MFA)

Optional MFA support for additional authentication layers:

```yaml
security:
  enhanced:
    authentication:
      enable_mfa: true
      mfa_methods: ["totp", "sms", "email"]
      account_lockout:
        enabled: true
        max_attempts: 5
        lockout_duration_seconds: 900
```

**Supported MFA Methods**:

- TOTP (Time-based One-Time Password)
- SMS verification
- Email verification
- Biometric authentication (platform-dependent)

#### Threat Detection

Automated threat detection and response:

```yaml
security:
  enhanced:
    threat_detection:
      enabled: true
      alert_thresholds:
        failed_login_attempts: 5
        suspicious_activity_score: 80
      response_actions:
        - "block_ip"
        - "require_mfa"
        - "notify_admin"
```

**Detected Threats**:

- Brute force attacks
- Suspicious access patterns
- Unusual API usage
- Resource exhaustion attempts
- Anomalous behavior patterns

#### Security Policy Engine

Configurable security policies for compliance:

```yaml
security:
  enhanced:
    security_policy:
      enabled: true
      rules:
        - name: "password_policy"
          type: "password_complexity"
          min_length: 12
          require_uppercase: true
          require_lowercase: true
          require_numbers: true
          require_special: true
```

### 7. System Guardrails

Runtime protection against system crashes and resource exhaustion.

#### Guardrails Configuration

```yaml
security:
  guardrails:
    enabled: true
    max_memory_percent: 75.0
    max_cpu_percent: 90.0
    min_free_memory_mb: 512
    max_concurrent_ops: 4
    auto_throttle: true
    windows_protection: true
```

**Protection Features**:

- Memory usage monitoring (prevents OOM crashes)
- CPU usage throttling (prevents system overload)
- Concurrent operation limits (prevents resource exhaustion)
- Automatic throttling under load
- Windows-specific protections (prevents BSOD)

**Violation Handling**:

- Automatic resource throttling
- Operation queuing when limits exceeded
- Violation logging and alerting
- Graceful degradation

---

## Security Best Practices

### Production Deployment

#### ✅ Required

- [ ] Enable TLS for all external communication
- [ ] Use strong API keys (minimum 32 characters)
- [ ] Enable rate limiting
- [ ] Enable audit logging
- [ ] Use RBAC with least-privilege principle
- [ ] Rotate API keys regularly (every 90 days)
- [ ] Monitor audit logs for suspicious activity
- [ ] Keep dependencies updated (run `cargo audit`)

#### ✅ Recommended

- [ ] Enable mTLS for replication traffic
- [ ] Use separate API keys per client/service
- [ ] Set up alerts for security events
- [ ] Regular security audits
- [ ] Backup audit logs to external storage
- [ ] Use secrets management (Vault, AWS Secrets Manager)
- [ ] Enable correlation IDs for request tracking
- [ ] Enable enhanced security features (MFA, threat detection)
- [ ] Configure system guardrails for production
- [ ] Use client SDKs with built-in security features

#### ⚠️ Avoid

- ❌ Exposing server directly to internet without TLS
- ❌ Using default API keys in production
- ❌ Disabling authentication
- ❌ Running as root user
- ❌ Storing credentials in code or config files
- ❌ Using weak passwords
- ❌ Disabling audit logging

### Network Security

#### Firewall Rules

```bash
# Allow only necessary ports
ufw allow 15002/tcp  # Vectorizer API
ufw allow 7001/tcp   # Replication (if master)
ufw deny 4317/tcp    # Block OTLP (internal only)
ufw enable
```

#### Reverse Proxy

Use nginx/Apache as reverse proxy:

```nginx
server {
    listen 443 ssl http2;
    server_name vectorizer.example.com;

    ssl_certificate /etc/ssl/certs/vectorizer.crt;
    ssl_certificate_key /etc/ssl/private/vectorizer.key;
    ssl_protocols TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://127.0.0.1:15002;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Rate limiting at proxy level
        limit_req zone=vectorizer burst=20 nodelay;
    }
}
```

### API Key Management

#### Generate Secure API Keys

```bash
# Use vectorizer CLI
vectorizer-cli api-key create --name "production-app" --role Editor

# Or generate manually
openssl rand -hex 32
```

#### Rotate API Keys

```bash
# 1. Create new key
vectorizer-cli api-key create --name "production-app-new"

# 2. Update client applications

# 3. Revoke old key
vectorizer-cli api-key revoke "old-key-id"
```

### Secrets Management

#### Environment Variables

```bash
# Never commit .env files
export VECTORIZER_JWT_SECRET="your-secret-here"
export VECTORIZER_API_KEY="your-api-key-here"

# Start server
./vectorizer
```

#### Docker Secrets

```yaml
version: "3.8"
services:
  vectorizer:
    image: vectorizer:latest
    secrets:
      - jwt_secret
      - api_key
    environment:
      - VECTORIZER_JWT_SECRET_FILE=/run/secrets/jwt_secret

secrets:
  jwt_secret:
    external: true
  api_key:
    external: true
```

### Client SDK Security

All Vectorizer client SDKs implement security best practices:

#### SDK Security Features

**TypeScript/JavaScript SDKs**:

- ✅ Secure credential storage (never in code)
- ✅ TLS/HTTPS enforcement
- ✅ Request signing support
- ✅ Automatic retry with exponential backoff
- ✅ Input validation and sanitization

**Python SDK**:

- ✅ Environment variable support for credentials
- ✅ Secure credential management
- ✅ TLS certificate validation
- ✅ Request timeout protection
- ✅ Input sanitization

**Rust SDK**:

- ✅ Type-safe credential handling
- ✅ Zero-copy where possible
- ✅ Memory-safe operations
- ✅ TLS certificate pinning support
- ✅ Secure defaults

**Go SDK**:

- ✅ Secure credential storage
- ✅ TLS configuration support
- ✅ Context-based cancellation
- ✅ Input validation
- ✅ Error handling without information leakage

**C# SDK**:

- ✅ Secure credential management
- ✅ Async/await for non-blocking operations
- ✅ TLS certificate validation
- ✅ Disposable pattern for resource cleanup
- ✅ Strong typing for security

#### SDK Security Best Practices

```typescript
// ✅ GOOD: Use environment variables
const client = new VectorizerClient({
  baseURL: process.env.VECTORIZER_URL,
  apiKey: process.env.VECTORIZER_API_KEY,
});

// ❌ BAD: Hardcoded credentials
const client = new VectorizerClient({
  baseURL: "https://api.example.com",
  apiKey: "hardcoded-key-12345", // NEVER DO THIS
});
```

**Recommendations**:

- Store API keys in secure vaults (AWS Secrets Manager, HashiCorp Vault)
- Use separate API keys per environment (dev/staging/prod)
- Rotate API keys regularly
- Never commit credentials to version control
- Use least-privilege principle for API key permissions

---

## Compliance

### GDPR

- ✅ Audit logs track data access
- ✅ Data deletion support (delete collections/vectors)
- ✅ Data portability (export via API)
- ⚠️ User consent management (application responsibility)

### SOC 2

- ✅ Access control (RBAC)
- ✅ Audit logging
- ✅ Encryption in transit (TLS)
- ✅ Encryption at rest (Zstd compression)
- ⚠️ Incident response plan (documentation required)

### HIPAA

- ✅ Access control (RBAC)
- ✅ Audit logging
- ✅ Encryption in transit (TLS)
- ⚠️ Business Associate Agreement required
- ⚠️ Additional controls may be needed

---

## Security Checklist

### Development

- [ ] Review code for security issues
- [ ] Run `cargo audit` for dependency vulnerabilities
- [ ] Run `cargo clippy` with security lints
- [ ] Write security tests
- [ ] Document security decisions

### Staging

- [ ] Enable TLS
- [ ] Configure strong API keys
- [ ] Enable audit logging
- [ ] Test rate limiting
- [ ] Review RBAC configuration
- [ ] Penetration testing

### Production

- [ ] All staging checks
- [ ] Enable mTLS for replication
- [ ] Set up security monitoring
- [ ] Configure alert rules
- [ ] Document incident response
- [ ] Regular security audits
- [ ] Compliance documentation
- [ ] Enable enhanced security features
- [ ] Configure system guardrails
- [ ] Test threat detection rules
- [ ] Verify MFA configuration (if enabled)
- [ ] Review security policy rules

---

## Vulnerability Disclosure

### Supported Versions

| Version | Supported  |
| ------- | ---------- |
| 1.3.x   | ✅ Yes     |
| 1.2.x   | ✅ Yes     |
| 1.1.x   | ✅ Yes     |
| 1.0.x   | ⚠️ Limited |
| < 1.0   | ❌ No      |

### Security Updates

- **Critical**: Released within 24 hours
- **High**: Released within 7 days
- **Medium**: Released within 30 days
- **Low**: Released in next regular release

---

## Contact

- **Security Team**: security@hivellm.dev
- **General Support**: support@hivellm.dev
- **GitHub**: https://github.com/hivellm/vectorizer/security

---

## Acknowledgments

We thank security researchers who responsibly disclose vulnerabilities.
Hall of Fame for contributors will be maintained in `SECURITY_HALL_OF_FAME.md`.

---

**For monitoring and observability, see**: `docs/MONITORING.md`
