# Vectorizer Configuration Guide

## Overview

Vectorizer uses a comprehensive YAML-based configuration system that allows you to customize every aspect of the server after build. This enables production deployments, development environments, and specialized use cases without code changes.

## Quick Start

```bash
# Copy the example configuration
cp config.example.yml config.yml

# Edit for your environment
nano config.yml

# Start server with configuration
vectorizer server --config config.yml
```

## Configuration Structure

The configuration file is organized into logical sections:

```yaml
# Top-level structure
server:      # Server basics (port, host, workers)
security:    # API keys, rate limiting, CORS, audit
dashboard:   # Localhost dashboard settings
network:     # Internal vs cloud deployment
performance: # Memory, caching, threading
compression: # LZ4 compression settings
persistence: # Storage and recovery settings
collections: # Default collection configurations
logging:     # Log levels and outputs
monitoring:  # Metrics and health checks
integrations: # LangChain, Aider, external APIs
development: # Debug and profiling options
experimental: # Cutting-edge features
```

## Detailed Configuration

### Server Configuration

```yaml
server:
  host: "127.0.0.1"          # Bind address
  port: 15001                # Server port
  workers: 4                 # Async workers
  max_connections: 1000      # Connection limit
  timeout_seconds: 30        # Request timeout
  name: "Vectorizer Server"  # Server identifier
  environment: "production"  # Environment type
```

### Security Configuration

```yaml
security:
  # API Key settings
  require_api_keys: true
  api_key_length: 32
  api_key_prefix: "vk_"

  # Rate limiting per API key
  rate_limiting:
    enabled: true
    requests_per_minute: 1000
    burst_limit: 100

  # CORS for cloud deployments
  cors:
    enabled: true
    allowed_origins:
      - "https://your-app.com"
    allowed_methods: ["GET", "POST", "PUT", "DELETE"]

  # Audit logging
  audit:
    enabled: true
    retention_days: 30
    log_operations: true
    log_errors: true
```

### Dashboard Configuration

```yaml
dashboard:
  enabled: true
  bind_address: "127.0.0.1"
  port: 15002  # Default: server_port + 1
  theme: "auto"  # light/dark/auto
  locale: "en"
  session_timeout_minutes: 60

  features:
    api_key_management: true
    collection_management: true
    vector_inspection: true
    search_preview: true
    server_monitoring: true
    audit_logs: true
```

### Network Configuration

```yaml
network:
  mode: "internal"  # internal or cloud

  # Internal mode (development)
  internal:
    bind_address: "127.0.0.1"
    allowed_clients:
      - "192.168.1.0/24"

  # Cloud mode (production)
  cloud:
    bind_address: "0.0.0.0"
    tls:
      enabled: false
      cert_path: "/path/to/cert.pem"
      key_path: "/path/to/key.pem"
```

### Performance Configuration

```yaml
performance:
  memory:
    max_usage_gb: 32.0
    memory_pool_size_mb: 512
    gc_threshold_mb: 1024

  threading:
    max_blocking_threads: 512
    async_runtime_threads: 4

  cache:
    enabled: true
    vector_cache_size_mb: 256
    query_cache_size_mb: 128
    cache_ttl_seconds: 3600
    cache_compression: true

  limits:
    max_collections: 100
    max_vectors_per_collection: 10000000
    max_payload_size_kb: 1024
```

### Compression Configuration

```yaml
compression:
  enabled: true
  algorithm: "lz4"
  default_threshold_bytes: 1024

  operations:
    api_responses: true
    persistence: true
    network: true

  lz4:
    compression_level: 1  # 1-16, 1=fastest
    checksum: false
```

### Collection Defaults

```yaml
collections:
  defaults:
    dimension: 768
    metric: "cosine"

    quantization:
      type: "pq"
      pq:
        n_centroids: 256
        n_subquantizers: 8

    embedding:
      model: "native_bow"
      bow:
        vocab_size: 50000
        max_sequence_length: 512

    compression:
      enabled: true
      threshold_bytes: 1024
      algorithm: "lz4"

    index:
      type: "hnsw"
      hnsw:
        m: 16
        ef_construction: 200
        ef_search: 64
```

### Logging Configuration

```yaml
logging:
  level: "info"
  format: "json"

  outputs:
    console:
      enabled: true
      level: "info"
    file:
      enabled: true
      path: "./logs/vectorizer.log"
      level: "debug"
      max_size_mb: 100
      max_files: 5

  modules:
    vectorizer::api: "debug"
    vectorizer::auth: "info"
    vectorizer::compression: "warn"
```

## Environment-Specific Configurations

### Development Configuration

```yaml
# config.dev.yml
server:
  host: "127.0.0.1"
  port: 15001
  environment: "development"

security:
  require_api_keys: false  # Disabled for easier development

dashboard:
  enabled: true
  theme: "dark"

logging:
  level: "debug"

development:
  debug_mode: true
  trace_requests: true
```

### Production Configuration

```yaml
# config.prod.yml
server:
  host: "0.0.0.0"
  port: 15001
  workers: 8
  environment: "production"

network:
  mode: "cloud"

security:
  require_api_keys: true
  rate_limiting:
    enabled: true
    requests_per_minute: 5000

dashboard:
  enabled: false  # Disabled for security

monitoring:
  metrics:
    enabled: true
  health_check:
    enabled: true

logging:
  level: "warn"
  outputs:
    file:
      enabled: true
      path: "/var/log/vectorizer.log"
```

### Cloud Deployment Configuration

```yaml
# config.cloud.yml
server:
  host: "0.0.0.0"
  port: 15001
  workers: 12

network:
  mode: "cloud"
  cloud:
    tls:
      enabled: true
      cert_path: "/etc/ssl/certs/vectorizer.crt"
      key_path: "/etc/ssl/private/vectorizer.key"

security:
  cors:
    enabled: true
    allowed_origins:
      - "https://your-app.com"

performance:
  memory:
    max_usage_gb: 64.0

monitoring:
  external:
    prometheus:
      enabled: true
      port: 9090
```

## Configuration Validation

### Validate Configuration

```bash
# Validate configuration without starting server
vectorizer config validate --config config.yml

# Check for deprecated settings
vectorizer config validate --config config.yml --strict

# Show configuration differences
vectorizer config diff config.old.yml config.new.yml
```

### Hot Reloading

```yaml
# Enable hot reloading for development
development:
  hot_reload: true
  config_watch_interval_seconds: 30
```

## Advanced Configuration

### Environment Variables

```yaml
# Use environment variables in configuration
server:
  port: ${PORT:-15001}
  host: ${HOST:-127.0.0.1}

security:
  api_key_prefix: ${API_KEY_PREFIX:-vk_}

integrations:
  external_apis:
    openai:
      api_key: ${OPENAI_API_KEY}
```

### Conditional Configuration

```yaml
# Conditional settings based on environment
{{#if (eq server.environment "production")}}
security:
  rate_limiting:
    requests_per_minute: 10000
{{/if}}

{{#if (eq server.environment "development")}}
security:
  require_api_keys: false
{{/if}}
```

### Custom Plugins

```yaml
extensions:
  plugins:
    - "/path/to/custom/plugin.so"
  custom_modules:
    - "my_custom_embedding"
  webhooks:
    enabled: true
    endpoints:
      - "https://webhook.site/xyz"
    events:
      - "collection_created"
      - "api_key_created"
```

## Best Practices

### Security
- Always enable API keys in production
- Use strong rate limiting
- Keep dashboard disabled in cloud deployments
- Enable audit logging
- Use TLS in cloud environments

### Performance
- Adjust worker count based on CPU cores
- Configure memory limits appropriately
- Enable caching for read-heavy workloads
- Use compression for large payloads
- Monitor and tune HNSW parameters

### Monitoring
- Enable metrics collection
- Set up health checks
- Configure log rotation
- Monitor memory usage
- Track API key usage patterns

### Maintenance
- Enable automatic backups
- Set up log rotation
- Configure maintenance windows
- Monitor disk space
- Plan for scaling

## Troubleshooting

### Common Issues

1. **Configuration not loading**
   ```bash
   vectorizer config validate --config config.yml
   ```

2. **Performance issues**
   - Check memory limits
   - Monitor cache hit rates
   - Adjust worker count

3. **Security warnings**
   - Enable API keys
   - Configure CORS properly
   - Check audit logs

4. **Network issues**
   - Verify bind addresses
   - Check firewall rules
   - Validate TLS certificates

### Debug Configuration

```yaml
development:
  debug_mode: true
  trace_requests: true
  profile: true

logging:
  level: "debug"
  outputs:
    console:
      enabled: true
      level: "trace"
```

---

This configuration system provides complete control over Vectorizer's behavior, enabling deployments from development to enterprise production environments.
