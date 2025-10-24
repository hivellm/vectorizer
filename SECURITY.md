# Security Policy

## Supported Versions

We provide security updates for the following versions of Vectorizer:

| Version | Supported          |
| ------- | ------------------ |
| 1.1.x   | :white_check_mark: |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

### How to Report

If you discover a security vulnerability, please report it by emailing:

**team@hivellm.org**

Include the following information:

1. **Description**: A clear description of the vulnerability
2. **Impact**: What an attacker could accomplish
3. **Reproduction**: Steps to reproduce the issue
4. **Affected Versions**: Which versions are affected
5. **Suggested Fix**: If you have a fix or mitigation (optional)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Assessment**: Within 5 business days
- **Fix Timeline**: Depends on severity (see below)
- **Disclosure**: Coordinated disclosure after fix is released

### Response Timeline

| Severity | Response Time | Fix Timeline |
|----------|--------------|--------------|
| Critical | 24 hours     | 7 days       |
| High     | 48 hours     | 14 days      |
| Medium   | 5 days       | 30 days      |
| Low      | 10 days      | 90 days      |

## Security Considerations

### Authentication & Authorization

- **API Keys**: Use strong, randomly generated API keys
- **JWT Tokens**: Tokens expire after 24 hours by default
- **TLS**: Always use TLS 1.2+ in production
- **Rate Limiting**: Implement rate limiting on all endpoints

### Data Protection

- **Encryption at Rest**: Use encrypted storage for sensitive data
- **Encryption in Transit**: All network communication uses TLS
- **Data Sanitization**: Input is validated and sanitized
- **Access Control**: Implement proper access controls

### Configuration

#### Secure Configuration Example

```yaml
# config.yml
server:
  host: "127.0.0.1"  # Bind to localhost in development
  port: 8080
  tls:
    enabled: true
    cert_path: "/path/to/cert.pem"
    key_path: "/path/to/key.pem"

auth:
  enabled: true
  jwt_secret: "use-strong-random-secret-here"
  token_expiry: 86400  # 24 hours

security:
  rate_limit:
    enabled: true
    requests_per_minute: 60
  cors:
    enabled: true
    allowed_origins: ["https://yourdomain.com"]
```

#### Security Checklist

- ✅ Enable authentication in production
- ✅ Use TLS for all connections
- ✅ Rotate JWT secrets regularly
- ✅ Implement rate limiting
- ✅ Use strong, unique API keys
- ✅ Keep dependencies up to date
- ✅ Review audit logs regularly
- ✅ Disable debug mode in production
- ✅ Use principle of least privilege
- ✅ Implement proper error handling (no sensitive info leaks)

### Network Security

#### Production Deployment

- **Firewall**: Only expose necessary ports
- **Reverse Proxy**: Use nginx/Apache as reverse proxy
- **DDoS Protection**: Implement DDoS mitigation
- **IP Whitelisting**: Restrict access by IP when possible

#### Example nginx Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name vectorizer.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=vectorizer:10m rate=10r/s;
    limit_req zone=vectorizer burst=20;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Dependency Security

#### Regular Updates

```bash
# Check for security advisories
cargo audit

# Update dependencies
cargo update

# Check outdated dependencies
cargo outdated
```

#### Audit Configuration

Create `audit.toml`:

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"
```

### Docker Security

#### Secure Dockerfile Practices

- Use specific version tags (not `latest`)
- Run as non-root user
- Use multi-stage builds
- Scan images for vulnerabilities
- Minimize attack surface

Example:

```dockerfile
FROM rust:1.85-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN useradd -m -u 1000 vectorizer
USER vectorizer
COPY --from=builder /app/target/release/vectorizer /usr/local/bin/
ENTRYPOINT ["vectorizer"]
```

### Logging & Monitoring

#### Security Logging

Log the following security-relevant events:

- Authentication attempts (success/failure)
- Authorization failures
- API rate limit violations
- Configuration changes
- Unusual access patterns
- Error conditions

#### Log Security

- **Rotation**: Implement log rotation
- **Retention**: Define retention policies
- **Access Control**: Restrict log access
- **Encryption**: Encrypt sensitive logs
- **No Secrets**: Never log passwords, tokens, or keys

### Replication Security

#### Secure Replication Setup

- **TLS**: Use TLS for replication traffic
- **Authentication**: Require authentication between nodes
- **Network Isolation**: Use private networks
- **Encryption**: Encrypt replication data

Example:

```yaml
replication:
  enabled: true
  role: master
  replicas:
    - host: "replica1.internal"
      port: 8080
      tls: true
      auth_token: "strong-random-token"
```

### Input Validation

All user input is validated and sanitized:

- **Query strings**: Length limits, character validation
- **File uploads**: Size limits, type validation, virus scanning
- **API parameters**: Type checking, range validation
- **JSON payloads**: Schema validation

### Known Security Issues

Check the [GitHub Security Advisories](https://github.com/hivellm/vectorizer/security/advisories) for known security issues.

## Security Updates

Security updates are released as patch versions:

- **Critical**: Immediate release
- **High**: Within 7 days
- **Medium**: Next minor release
- **Low**: Next major release

Subscribe to releases on GitHub to receive notifications.

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest stable version
2. **Strong Credentials**: Use strong, unique passwords and keys
3. **TLS**: Always enable TLS in production
4. **Network**: Restrict network access
5. **Monitoring**: Monitor logs for suspicious activity
6. **Backups**: Regular backups with encryption
7. **Principle of Least Privilege**: Grant minimal permissions

### For Contributors

1. **Code Review**: All code changes require review
2. **Dependency Audit**: Run `cargo audit` before committing
3. **No Secrets**: Never commit secrets or credentials
4. **Safe Coding**: Follow Rust security best practices
5. **Testing**: Include security tests for new features
6. **Documentation**: Document security implications

## Vulnerability Disclosure Policy

We follow **Coordinated Vulnerability Disclosure**:

1. **Private Report**: Report to team@hivellm.org
2. **Assessment**: We assess and confirm the vulnerability
3. **Fix Development**: We develop and test a fix
4. **Security Release**: We release a security update
5. **Public Disclosure**: We publish a security advisory
6. **Recognition**: We credit the reporter (if desired)

## Security Hall of Fame

We recognize security researchers who responsibly disclose vulnerabilities:

- *No security disclosures yet*

---

**Last Updated**: October 2024

For questions about security, contact: team@hivellm.org

