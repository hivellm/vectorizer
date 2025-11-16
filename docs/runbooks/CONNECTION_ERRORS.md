# Runbook: Connection Errors

## Symptoms

- Connection refused errors
- Timeout errors
- "Too many connections" errors
- Service unavailable errors

## Diagnosis

### Check Service Status

```bash
# Check if service is running
systemctl status vectorizer

# Or Kubernetes
kubectl get pods -n vectorizer

# Check service health
curl http://localhost:15002/api/status
```

### Check Connection Limits

```bash
# Check current connections
curl http://localhost:15002/prometheus/metrics | grep vectorizer_connections

# Check connection errors
curl http://localhost:15002/prometheus/metrics | grep vectorizer_connection_errors

# Check system connections
netstat -an | grep 15002 | wc -l
ss -an | grep 15002 | wc -l
```

### Check Logs

```bash
# Check application logs
journalctl -u vectorizer -n 100

# Or Kubernetes
kubectl logs vectorizer-0 -n vectorizer --tail=100

# Check for errors
grep -i error /var/log/vectorizer/*.log
```

## Immediate Actions

### 1. Restart Service

```bash
# Graceful restart
systemctl restart vectorizer

# Or Kubernetes
kubectl rollout restart statefulset vectorizer -n vectorizer
```

### 2. Check Resource Limits

```bash
# Check if service is resource constrained
top -p $(pgrep vectorizer)
free -h

# Check system limits
ulimit -n
```

### 3. Increase Connection Limits

```yaml
# config.yml
server:
  max_connections: 10000 # Increase from default
  connection_timeout: 60
```

## Root Cause Analysis

### Common Causes

1. **Service Down**

   - Service crashed
   - OOM killed
   - Manual stop

2. **Connection Limit Reached**

   - Too many concurrent connections
   - Connection pool exhausted
   - Need to increase limits

3. **Resource Exhaustion**

   - Out of memory
   - Out of file descriptors
   - CPU throttling

4. **Network Issues**

   - Firewall blocking
   - Network congestion
   - DNS resolution issues

5. **Load Balancer Issues**
   - Load balancer misconfigured
   - Health checks failing
   - Backend pool empty

## Resolution Steps

### Step 1: Restart Service

```bash
# Systemd
systemctl restart vectorizer

# Kubernetes
kubectl rollout restart statefulset vectorizer -n vectorizer

# Docker
docker restart vectorizer
```

### Step 2: Increase Connection Limits

```yaml
# config.yml
server:
  max_connections: 10000
  connection_timeout: 60
  keepalive_timeout: 65
```

### Step 3: Increase System Limits

```bash
# Increase file descriptor limit
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Apply immediately
ulimit -n 65536
```

### Step 4: Scale Horizontally

```bash
# Add more instances
kubectl scale statefulset vectorizer --replicas=3 -n vectorizer

# Or Docker Compose
docker-compose scale vectorizer=3
```

### Step 5: Fix Load Balancer

```yaml
# Kubernetes Service
apiVersion: v1
kind: Service
metadata:
  name: vectorizer
spec:
  type: LoadBalancer
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800
```

## Prevention

1. **Monitor Connections**: Set alerts at 80% of limit
2. **Health Checks**: Configure proper health checks
3. **Connection Pooling**: Use connection pooling in clients
4. **Rate Limiting**: Prevent connection abuse
5. **Regular Testing**: Test connection limits

## Verification

```bash
# Verify service is running
curl http://localhost:15002/api/status

# Check connections
curl http://localhost:15002/prometheus/metrics | grep vectorizer_connections

# Test connectivity
curl -v http://localhost:15002/api/status
```

## Escalation

If connection errors persist:

1. Check infrastructure (network, load balancer)
2. Review application architecture
3. Consider connection pooling
4. Contact infrastructure/SRE team
