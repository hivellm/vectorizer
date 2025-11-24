# Runbook: Replication Lag

## Symptoms

- Replication lag > 30 seconds
- Replicas out of sync
- Data inconsistency
- Replica connection errors

## Diagnosis

### Check Replication Status

```bash
# Check replication status
curl http://localhost:15002/api/replication/status

# Check replication lag
curl http://localhost:15002/prometheus/metrics | grep vectorizer_replication_lag

# Check replica connections
curl http://localhost:15002/api/replication/replicas
```

### Identify Lag Source

```bash
# Check master status
curl http://master:15002/api/replication/status

# Check replica status
curl http://replica:15002/api/replication/status

# Check network connectivity
ping replica-host
telnet replica-host 7001
```

## Immediate Actions

### 1. Check Network Connectivity

```bash
# Test connectivity from master to replica
curl http://replica:15002/api/status

# Test replication port
nc -zv replica-host 7001
```

### 2. Restart Replication

```bash
# Restart replica connection
curl -X POST http://replica:15002/api/replication/reconnect

# Or restart service
systemctl restart vectorizer
```

### 3. Reduce Load (Temporary)

```bash
# Reduce write load on master
# Enable rate limiting
rate_limit:
  enabled: true
  requests_per_minute: 1000  # Reduce from default
```

## Root Cause Analysis

### Common Causes

1. **Network Issues**
   - High latency between master and replica
   - Network congestion
   - Firewall blocking replication port

2. **High Write Load**
   - Too many writes on master
   - Replication can't keep up
   - Need more replicas or better network

3. **Replica Performance**
   - Replica CPU/memory constrained
   - Slow disk I/O on replica
   - Replica overloaded

4. **Replication Log Full**
   - Replication log size exceeded
   - Need to increase log size
   - Or reduce lag

5. **Connection Issues**
   - Replica disconnected
   - Connection timeout
   - Need to reconnect

## Resolution Steps

### Step 1: Fix Network Issues

```bash
# Check network latency
ping -c 10 replica-host

# Check bandwidth
iperf3 -c replica-host

# Fix firewall rules if needed
ufw allow 7001/tcp
```

### Step 2: Optimize Replication Configuration

```yaml
# config.yml - Master
replication:
  heartbeat_interval_secs: 5  # More frequent heartbeats
  replica_timeout_secs: 60     # Longer timeout
  log_size: 2000000            # Larger log

# config.yml - Replica
replication:
  reconnect_interval_secs: 5   # Faster reconnection
```

### Step 3: Scale Replicas

```bash
# Add more replicas to distribute load
kubectl scale statefulset vectorizer-replica --replicas=3 -n vectorizer
```

### Step 4: Optimize Replica Performance

```bash
# Increase replica resources
kubectl edit statefulset vectorizer-replica -n vectorizer
# Update resources.limits
```

### Step 5: Force Sync (If Needed)

```bash
# Force full sync from master
curl -X POST http://replica:15002/api/replication/sync \
  -H "Content-Type: application/json" \
  -d '{"full_sync": true}'
```

## Prevention

1. **Monitor Lag**: Set alerts at 10 seconds
2. **Network Monitoring**: Monitor network latency
3. **Capacity Planning**: Plan for write load
4. **Regular Testing**: Test failover regularly
5. **Health Checks**: Monitor replica health

## Verification

```bash
# Verify lag reduced
curl http://localhost:15002/api/replication/status | jq .lag_seconds

# Check replica sync status
curl http://replica:15002/api/replication/status | jq .synced

# Verify data consistency
# Compare vector counts
curl http://master:15002/api/collections | jq '.[].vector_count'
curl http://replica:15002/api/collections | jq '.[].vector_count'
```

## Escalation

If replication lag persists:
1. Check network infrastructure
2. Review write load patterns
3. Consider dedicated replication network
4. Contact network/SRE team

