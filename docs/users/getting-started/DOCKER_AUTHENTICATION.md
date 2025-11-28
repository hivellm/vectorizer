# Docker Authentication Guide

This guide explains how to configure authentication for Vectorizer when running in Docker.

## Default Credentials

By default, Vectorizer Docker images come with authentication **enabled** and the following default credentials:

- **Username:** `admin`
- **Password:** `admin`
- **JWT Secret:** `change-this-secret-in-production`

⚠️ **SECURITY WARNING:** These default credentials MUST be changed in production environments!

## Quick Start

### Using Docker Run

**Basic deployment with default credentials:**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=admin \
  -e VECTORIZER_JWT_SECRET=change-this-secret-in-production \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

**Production deployment with custom credentials:**
```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -v $(pwd)/vectorizer-data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=true \
  -e VECTORIZER_ADMIN_USERNAME=admin \
  -e VECTORIZER_ADMIN_PASSWORD=your-secure-password-here \
  -e VECTORIZER_JWT_SECRET=your-jwt-secret-key-minimum-32-chars \
  --restart unless-stopped \
  hivehub/vectorizer:latest
```

### Using Docker Compose

1. **Copy the example environment file:**
```bash
cp .env.example .env
```

2. **Edit `.env` with your credentials:**
```bash
# .env file
VECTORIZER_AUTH_ENABLED=true
VECTORIZER_ADMIN_USERNAME=admin
VECTORIZER_ADMIN_PASSWORD=your-secure-password
VECTORIZER_JWT_SECRET=your-jwt-secret-key-minimum-32-chars
```

3. **Start with docker-compose:**
```bash
docker-compose up -d
```

## Environment Variables

### Authentication Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VECTORIZER_AUTH_ENABLED` | `true` | Enable/disable authentication |
| `VECTORIZER_ADMIN_USERNAME` | `admin` | Admin username |
| `VECTORIZER_ADMIN_PASSWORD` | `admin` | Admin password (CHANGE THIS!) |
| `VECTORIZER_JWT_SECRET` | `change-this-secret-in-production` | JWT signing secret (min 32 chars) |

### Other Configuration Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VECTORIZER_HOST` | `0.0.0.0` | Server bind address |
| `VECTORIZER_PORT` | `15002` | Server port |
| `RUN_MODE` | `production` | Run mode (production/development) |
| `TZ` | `Etc/UTC` | Timezone |
| `RUST_LOG` | `info` | Log level |

## Security Best Practices

### 1. Strong Password

Use a strong, randomly generated password:

```bash
# Generate a secure password (Linux/Mac)
openssl rand -base64 32

# Generate a secure password (Windows PowerShell)
[Convert]::ToBase64String((1..32 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
```

### 2. Strong JWT Secret

Generate a secure JWT secret (minimum 32 characters):

```bash
# Generate JWT secret (Linux/Mac)
openssl rand -base64 48

# Generate JWT secret (Windows PowerShell)
[Convert]::ToBase64String((1..48 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
```

### 3. Use Environment Variables

Never hardcode credentials in docker-compose.yml. Use environment variables or Docker secrets:

```yaml
# docker-compose.yml
services:
  vectorizer:
    image: hivehub/vectorizer:latest
    environment:
      - VECTORIZER_AUTH_ENABLED=true
      - VECTORIZER_ADMIN_USERNAME=${ADMIN_USERNAME}
      - VECTORIZER_ADMIN_PASSWORD=${ADMIN_PASSWORD}
      - VECTORIZER_JWT_SECRET=${JWT_SECRET}
```

### 4. Docker Secrets (Swarm/Compose)

For production deployments, use Docker secrets:

```yaml
# docker-compose.yml
version: '3.8'
services:
  vectorizer:
    image: hivehub/vectorizer:latest
    secrets:
      - admin_password
      - jwt_secret
    environment:
      - VECTORIZER_AUTH_ENABLED=true
      - VECTORIZER_ADMIN_USERNAME=admin
      - VECTORIZER_ADMIN_PASSWORD_FILE=/run/secrets/admin_password
      - VECTORIZER_JWT_SECRET_FILE=/run/secrets/jwt_secret

secrets:
  admin_password:
    external: true
  jwt_secret:
    external: true
```

Create secrets:
```bash
echo "your-secure-password" | docker secret create admin_password -
echo "your-jwt-secret-key" | docker secret create jwt_secret -
```

## Authentication Flow

1. **Login Request:**
   ```bash
   curl -X POST http://localhost:15002/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"admin","password":"your-password"}'
   ```

2. **Response:**
   ```json
   {
     "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
     "expires_in": 3600
   }
   ```

3. **Use Token in Requests:**
   ```bash
   curl -X GET http://localhost:15002/collections \
     -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
   ```

## Disabling Authentication

For development or testing, you can disable authentication:

```bash
docker run -d \
  --name vectorizer \
  -p 15002:15002 \
  -e VECTORIZER_AUTH_ENABLED=false \
  hivehub/vectorizer:latest
```

⚠️ **WARNING:** Only disable authentication in secure, isolated development environments!

## Troubleshooting

### Authentication Errors

**Problem:** "Invalid credentials" error
- **Solution:** Verify username and password are correct
- Check environment variables are properly set
- Restart container after changing credentials

**Problem:** "Invalid token" error
- **Solution:** Token may be expired, login again
- Verify JWT secret hasn't changed
- Check system time is synchronized

### Container Startup Issues

**Problem:** Container exits immediately
- **Solution:** Check logs: `docker logs vectorizer`
- Verify all required environment variables are set
- Ensure JWT secret is at least 32 characters

## Production Checklist

Before deploying to production:

- [ ] Changed default admin password
- [ ] Generated strong JWT secret (min 32 chars)
- [ ] Using environment variables or Docker secrets
- [ ] Authentication is enabled (`VECTORIZER_AUTH_ENABLED=true`)
- [ ] Credentials are not committed to version control
- [ ] TLS/SSL is configured (use reverse proxy like nginx)
- [ ] Firewall rules are configured
- [ ] Regular backups are configured
- [ ] Monitoring and logging are enabled

## Examples

### Example 1: Production Deployment with Nginx

```yaml
# docker-compose.yml
version: '3.8'
services:
  vectorizer:
    image: hivehub/vectorizer:latest
    environment:
      - VECTORIZER_AUTH_ENABLED=true
      - VECTORIZER_ADMIN_USERNAME=${ADMIN_USERNAME}
      - VECTORIZER_ADMIN_PASSWORD=${ADMIN_PASSWORD}
      - VECTORIZER_JWT_SECRET=${JWT_SECRET}
    volumes:
      - vectorizer-data:/vectorizer/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - vectorizer
    restart: unless-stopped

volumes:
  vectorizer-data:
```

### Example 2: Development Environment

```bash
# Start without authentication for local development
docker run -d \
  --name vectorizer-dev \
  -p 15002:15002 \
  -v $(pwd)/data:/vectorizer/data \
  -e VECTORIZER_AUTH_ENABLED=false \
  -e RUN_MODE=development \
  -e RUST_LOG=debug \
  hivehub/vectorizer:latest
```

## See Also

- [Docker Installation Guide](DOCKER.md)
- [Configuration Reference](../configuration/CONFIGURATION.md)
- [Security Best Practices](../../SECURITY.md)
- [Production Guide](../../PRODUCTION_GUIDE.md)
