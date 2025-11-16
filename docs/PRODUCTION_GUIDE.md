# Production Deployment Guide

Complete guide for deploying Vectorizer in production environments.

## Table of Contents

1. [Pre-Production Checklist](#pre-production-checklist)
2. [Performance Configuration](#performance-configuration)
3. [Reliability Configuration](#reliability-configuration)
4. [Security Hardening](#security-hardening)
5. [Capacity Planning](#capacity-planning)
6. [Deployment Options](#deployment-options)
7. [Monitoring](#monitoring)
8. [Backup & Recovery](#backup--recovery)
9. [Runbooks](#runbooks)
10. [Best Practices](#best-practices)

## Pre-Production Checklist

### Infrastructure Requirements

- [ ] **Compute Resources**

  - [ ] Minimum 4 CPU cores (8+ recommended)
  - [ ] Minimum 8GB RAM (16GB+ recommended)
  - [ ] SSD storage (NVMe preferred)
  - [ ] Network bandwidth: 1Gbps+

- [ ] **Storage**

  - [ ] Persistent volume configured
  - [ ] Backup storage configured
  - [ ] Disk space: 2x expected data size
  - [ ] I/O performance: 10K+ IOPS

- [ ] **Network**

  - [ ] Firewall rules configured
  - [ ] Load balancer configured
  - [ ] DNS records set up
  - [ ] SSL/TLS certificates ready

- [ ] **Monitoring**
  - [ ] Prometheus configured
  - [ ] Grafana dashboards ready
  - [ ] Alert rules configured
  - [ ] Log aggregation set up

### Application Configuration

- [ ] **Server Settings**

  - [ ] Production config file created
  - [ ] Log level set to `warn` or `error`
  - [ ] Data directory configured
  - [ ] Ports configured correctly

- [ ] **Security**

  - [ ] Authentication enabled (if required)
  - [ ] TLS/SSL configured
  - [ ] Firewall rules applied
  - [ ] Rate limiting configured

- [ ] **Performance**

  - [ ] Thread count optimized
  - [ ] Memory limits set
  - [ ] Batch sizes configured
  - [ ] Caching enabled

- [ ] **Reliability**
  - [ ] Replication configured (if needed)
  - [ ] Auto-save enabled
  - [ ] Health checks configured
  - [ ] Graceful shutdown enabled

### Testing

- [ ] **Load Testing**

  - [ ] Expected load tested
  - [ ] Peak load tested (2x expected)
  - [ ] Stress testing completed
  - [ ] Performance benchmarks recorded

- [ ] **Failover Testing**
  - [ ] Replication failover tested
  - [ ] Backup restore tested
  - [ ] Disaster recovery tested
  - [ ] Rollback procedure tested

## Performance Configuration

### Server Configuration

```yaml
# config.production.yml
server:
  host: "0.0.0.0"
  port: 15002
  data_dir: "/var/lib/vectorizer"

logging:
  level: "warn" # Production: warn or error
  format: "json"
  log_requests: false # Disable in production for performance
  log_responses: false

performance:
  cpu:
    max_threads: 16 # Match CPU cores
    enable_simd: true
    memory_pool_size_mb: 4096 # 4GB pool
  batch:
    default_size: 500
    max_size: 2000
    parallel_processing: true
```

### Collection Configuration

```yaml
# Optimized collection config
collections:
  default:
    dimension: 384
    metric: "cosine"
    hnsw_config:
      m: 32 # Higher for better recall
      ef_construction: 200 # Higher for better quality
      ef_search: 100 # Higher for better results
    quantization:
      enabled: true
      type: "scalar"
      bits: 8 # 8-bit quantization for 4x memory savings
    compression:
      enabled: true
      threshold_bytes: 1024
```

### Memory Optimization

- **Enable Quantization**: Reduces memory by 75%
- **Use Compression**: Reduces storage by 20-30%
- **Set Memory Limits**: Prevent OOM kills
- **Enable Memory Pool**: Reduce allocations

### CPU Optimization

- **Match Threads to Cores**: `max_threads = CPU cores`
- **Enable SIMD**: Vector operations acceleration
- **Batch Processing**: Process multiple operations together
- **Parallel Search**: Use multiple threads for search

## Reliability Configuration

### Replication Setup

**Master Node**:

```yaml
replication:
  enabled: true
  role: "master"
  bind_address: "0.0.0.0:7001"
  heartbeat_interval_secs: 5
  replica_timeout_secs: 30
  log_size: 1000000
```

**Replica Node**:

```yaml
replication:
  enabled: true
  role: "replica"
  master_address: "master.example.com:7001"
  reconnect_interval_secs: 5
```

### Auto-Save Configuration

```yaml
auto_save:
  enabled: true
  interval_seconds: 300 # Save every 5 minutes
  min_operations: 1000 # Or after 1000 operations
```

### Health Checks

**HTTP Health Check**:

```bash
curl http://localhost:15002/api/status
```

**Kubernetes Liveness Probe**:

```yaml
livenessProbe:
  httpGet:
    path: /api/status
    port: 15002
  initialDelaySeconds: 30
  periodSeconds: 10
```

**Kubernetes Readiness Probe**:

```yaml
readinessProbe:
  httpGet:
    path: /api/status
    port: 15002
  initialDelaySeconds: 10
  periodSeconds: 5
```

## Security Hardening

### Network Security

1. **Firewall Rules**

   ```bash
   # Allow only necessary ports
   ufw allow 15002/tcp  # API port
   ufw allow 7001/tcp   # Replication port (if used)
   ufw deny 22/tcp      # SSH (if not needed)
   ```

2. **Reverse Proxy**

   - Use nginx or Traefik
   - Enable rate limiting
   - Configure SSL/TLS
   - Hide internal ports

3. **Network Isolation**
   - Use private networks
   - Restrict access to admin endpoints
   - Use VPN for management access

### Application Security

1. **Authentication** (if enabled)

   ```yaml
   auth:
     enabled: true
     jwt_secret: "your-secret-key"
     token_expiry: 3600
   ```

2. **Rate Limiting**

   ```yaml
   rate_limit:
     enabled: true
     requests_per_minute: 1000
     burst_size: 100
   ```

3. **Input Validation**
   - Validate all API inputs
   - Limit vector dimensions
   - Limit collection sizes
   - Sanitize file paths

### Data Security

1. **Encryption at Rest**

   - Use encrypted volumes
   - Encrypt backup files
   - Use secure key management

2. **Encryption in Transit**

   - Enable TLS/SSL
   - Use strong ciphers
   - Regular certificate rotation

3. **Access Control**
   - Restrict file permissions
   - Use service accounts
   - Implement least privilege

## Capacity Planning

### Resource Requirements by Scale

| Scale          | Vectors | Collections | CPU       | RAM   | Storage | Network |
| -------------- | ------- | ----------- | --------- | ----- | ------- | ------- |
| **Small**      | 100K    | 5           | 4 cores   | 8GB   | 10GB    | 100Mbps |
| **Medium**     | 1M      | 20          | 8 cores   | 16GB  | 100GB   | 1Gbps   |
| **Large**      | 10M     | 50          | 16 cores  | 32GB  | 1TB     | 10Gbps  |
| **Enterprise** | 100M+   | 100+        | 32+ cores | 64GB+ | 10TB+   | 10Gbps+ |

### Memory Calculation

```
Total Memory = Base + (Vectors × Vector Size) + (Collections × Overhead)

Where:
- Base: ~2GB (OS + application)
- Vector Size:
  - Dense: dimension × 4 bytes
  - Quantized (8-bit): dimension × 1 byte
  - Quantized (binary): dimension / 8 bytes
- Collection Overhead: ~100MB per collection
```

### Storage Calculation

```
Storage = (Vectors × Vector Size) × Compression Ratio + Metadata + Snapshots

Where:
- Compression Ratio: 0.7-0.8 (with compression enabled)
- Metadata: ~1% of vector data
- Snapshots: 2-3x data size (with retention)
```

### Network Bandwidth

```
Required Bandwidth = (Queries/sec × Query Size) + (Writes/sec × Write Size) × 2

Where:
- Query Size: ~1KB per query
- Write Size: ~10KB per vector
- Factor of 2 for replication overhead
```

## Deployment Options

### Docker Compose

See [Docker Compose Production Example](./deployment/docker-compose.production.yml)

### Kubernetes

See [Kubernetes Deployment Guide](./deployment/KUBERNETES.md)

### Systemd Service

See [Service Management Guide](../users/operations/SERVICE_MANAGEMENT.md)

## Monitoring

### Key Metrics

- **Performance**: Query latency, throughput, CPU usage
- **Reliability**: Uptime, error rate, replication lag
- **Capacity**: Memory usage, disk usage, vector count
- **Health**: Response time, connection count, queue depth

### Alerting

See [Monitoring Setup Guide](./MONITORING_SETUP.md)

## Backup & Recovery

### Backup Strategy

- **Frequency**: Daily full backups, hourly incremental
- **Retention**: 30 days daily, 7 days hourly
- **Storage**: Off-site backup storage
- **Testing**: Weekly restore tests

### Recovery Procedures

See [Backup & Recovery Guide](./BACKUP_RECOVERY.md)

## Runbooks

See [Runbooks Directory](./runbooks/)

- [High CPU](./runbooks/HIGH_CPU.md)
- [High Memory](./runbooks/HIGH_MEMORY.md)
- [Slow Searches](./runbooks/SLOW_SEARCHES.md)
- [Replication Lag](./runbooks/REPLICATION_LAG.md)
- [Connection Errors](./runbooks/CONNECTION_ERRORS.md)

## Best Practices

### Performance

1. **Use Quantization**: Enable 8-bit quantization for 75% memory savings
2. **Enable Compression**: Reduce storage by 20-30%
3. **Optimize HNSW**: Tune `m`, `ef_construction`, `ef_search`
4. **Batch Operations**: Group multiple operations together
5. **Use Caching**: Cache frequent queries

### Reliability

1. **Enable Replication**: For high availability
2. **Regular Backups**: Automated daily backups
3. **Health Checks**: Monitor and alert on failures
4. **Graceful Shutdown**: Allow in-flight requests to complete
5. **Circuit Breakers**: Prevent cascade failures

### Security

1. **Use TLS**: Encrypt all traffic
2. **Rate Limiting**: Prevent abuse
3. **Input Validation**: Validate all inputs
4. **Least Privilege**: Minimal permissions
5. **Regular Updates**: Keep software updated

### Operations

1. **Monitor Everything**: Comprehensive monitoring
2. **Document Changes**: Keep deployment docs updated
3. **Test Backups**: Regular restore tests
4. **Plan Capacity**: Monitor and plan for growth
5. **Automate Deployments**: Use CI/CD pipelines

## Support

For production support:

- Documentation: [docs.vectorizer.io](https://docs.vectorizer.io)
- Issues: [GitHub Issues](https://github.com/hivellm/vectorizer/issues)
- Community: [Discord](https://discord.gg/vectorizer)
