# Runbook: High Memory Usage

## Symptoms

- Memory usage above 90%
- OOM (Out of Memory) kills
- Slow performance
- System swapping

## Diagnosis

### Check Current Memory Usage

```bash
# Check memory usage
free -h

# Check process memory
ps aux | grep vectorizer

# Or using metrics
curl http://localhost:15002/prometheus/metrics | grep process_resident_memory
```

### Identify Memory Consumers

```bash
# Check vector count
curl http://localhost:15002/prometheus/metrics | grep vectorizer_vectors_total

# Check collections
curl http://localhost:15002/prometheus/metrics | grep vectorizer_collections_total

# Check memory per collection
curl http://localhost:15002/api/collections
```

## Immediate Actions

### 1. Enable Quantization

```yaml
# config.yml
collections:
  default:
    quantization:
      enabled: true
      type: "scalar"
      bits: 8  # Reduces memory by 75%
```

### 2. Enable Compression

```yaml
collections:
  default:
    compression:
      enabled: true
      threshold_bytes: 1024
```

### 3. Reduce Memory Pool

```yaml
performance:
  cpu:
    memory_pool_size_mb: 1024  # Reduce from 2048
```

### 4. Restart Service

```bash
# Graceful restart
systemctl restart vectorizer

# Or Kubernetes
kubectl rollout restart statefulset vectorizer -n vectorizer
```

## Root Cause Analysis

### Common Causes

1. **Large Vector Count**
   - Too many vectors in memory
   - Need quantization or offloading

2. **Large Vector Dimensions**
   - High-dimensional vectors consume more memory
   - Consider dimension reduction

3. **No Quantization**
   - Full precision vectors use 4x more memory
   - Enable quantization immediately

4. **Memory Leaks**
   - Check for memory leaks in application
   - Review recent changes

5. **Insufficient Resources**
   - Not enough RAM allocated
   - Need more memory

## Resolution Steps

### Step 1: Enable Quantization (Immediate)

```yaml
# config.yml - Apply to all collections
collections:
  default:
    quantization:
      enabled: true
      type: "scalar"
      bits: 8
```

### Step 2: Optimize Collections

```bash
# Recreate collections with quantization
# 1. Export data
curl http://localhost:15002/api/collections/{name}/export

# 2. Delete collection
curl -X DELETE http://localhost:15002/api/collections/{name}

# 3. Create with quantization
curl -X POST http://localhost:15002/api/collections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "{name}",
    "config": {
      "quantization": {"enabled": true, "type": "scalar", "bits": 8}
    }
  }'

# 4. Re-import data
curl -X POST http://localhost:15002/api/collections/{name}/import
```

### Step 3: Scale Vertically

```bash
# Increase memory limits (Kubernetes)
kubectl edit statefulset vectorizer -n vectorizer
# Update resources.limits.memory to "16Gi" or higher
```

### Step 4: Scale Horizontally

```bash
# Add more replicas to distribute load
kubectl scale statefulset vectorizer --replicas=3 -n vectorizer
```

## Prevention

1. **Enable Quantization**: Always use quantization in production
2. **Monitor Memory**: Set alerts at 80% usage
3. **Capacity Planning**: Plan for data growth
4. **Regular Cleanup**: Remove unused collections
5. **Memory Limits**: Set appropriate memory limits

## Verification

```bash
# Verify memory usage reduced
free -h

# Check process memory
ps aux | grep vectorizer

# Verify quantization enabled
curl http://localhost:15002/api/collections/{name} | jq .config.quantization
```

## Escalation

If memory usage remains high:
1. Enable quantization immediately
2. Consider offloading old data
3. Scale resources or horizontally
4. Contact SRE team

