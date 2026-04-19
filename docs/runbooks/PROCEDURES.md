# Vectorizer Operational Runbooks

Standard operating procedures for common operational tasks and incident response.

## Table of Contents

- [Node Recovery](#node-recovery)
- [Data Restoration](#data-restoration)
- [Failover Procedures](#failover-procedures)
- [Integrity Check](#integrity-check)
- [Performance Troubleshooting](#performance-troubleshooting)
- [Capacity Management](#capacity-management)
- [Security Incidents](#security-incidents)

---

## Node Recovery

### Scenario: Single Node Unresponsive

**Severity:** High
**Expected Resolution Time:** 15-30 minutes

#### Symptoms
- Health check failing
- `VectorizerDown` alert firing
- Requests timing out

#### Investigation Steps

```bash
# 1. Check node status
kubectl get pods -l app=vectorizer
kubectl describe pod vectorizer-0

# 2. Check logs
kubectl logs vectorizer-0 --tail=100

# 3. Check resource usage
kubectl top pod vectorizer-0

# 4. Check disk space
kubectl exec vectorizer-0 -- df -h /data
```

#### Resolution Steps

**Option A: Pod restart (transient issue)**
```bash
# Delete pod to trigger restart
kubectl delete pod vectorizer-0

# Wait for pod to be ready
kubectl wait --for=condition=ready pod/vectorizer-0 --timeout=300s

# Verify health
curl http://vectorizer-0.vectorizer:15002/health
```

**Option B: Node replacement (persistent issue)**
```bash
# 1. Cordon the node
kubectl cordon <node-name>

# 2. Drain workloads
kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data

# 3. If using StatefulSet, delete PVC if corrupted
kubectl delete pvc data-vectorizer-0

# 4. Uncordon or replace node
kubectl uncordon <node-name>

# 5. Pod will be rescheduled and sync data
kubectl wait --for=condition=ready pod/vectorizer-0 --timeout=600s
```

#### Verification
```bash
# Check replication status
curl http://vectorizer-0.vectorizer:15002/replication/status

# Verify vector counts match cluster
for pod in $(kubectl get pods -l app=vectorizer -o name); do
  echo "$pod: $(kubectl exec $pod -- curl -s localhost:15002/stats | jq '.total_vectors')"
done
```

---

## Data Restoration

### Scenario: Restore from Backup

**Severity:** Medium
**Expected Resolution Time:** 30-120 minutes (depending on data size)

#### Prerequisites
- Backup file location (S3, local, etc.)
- Target environment ready
- Sufficient disk space (2x backup size)

#### Procedure

**Step 1: Stop affected services**
```bash
# Scale down to prevent writes
kubectl scale deployment vectorizer --replicas=0

# Verify no pods running
kubectl get pods -l app=vectorizer
```

**Step 2: Prepare restoration**
```bash
# Download backup if from remote storage
aws s3 cp s3://backups/vectorizer/snapshot-2024-01-15.tar.gz /tmp/

# Verify backup integrity
tar -tzf /tmp/snapshot-2024-01-15.tar.gz | head
sha256sum /tmp/snapshot-2024-01-15.tar.gz
```

**Step 3: Restore data**
```bash
# Option A: Using API (if service is running)
curl -X POST http://localhost:15002/qdrant/snapshots/recover \
  -H "Content-Type: application/json" \
  -d '{"location": "/tmp/snapshot-2024-01-15.tar.gz"}'

# Option B: Direct file restore (if service is down)
kubectl exec vectorizer-0 -- rm -rf /data/*
kubectl cp /tmp/snapshot-2024-01-15.tar.gz vectorizer-0:/tmp/
kubectl exec vectorizer-0 -- tar -xzf /tmp/snapshot-2024-01-15.tar.gz -C /data/
```

**Step 4: Start services**
```bash
kubectl scale deployment vectorizer --replicas=3

# Wait for all pods ready
kubectl rollout status deployment/vectorizer
```

**Step 5: Verify restoration**
```bash
# Check collection counts
curl http://localhost:15002/collections

# Compare with backup manifest
cat /tmp/backup-manifest.json

# Run integrity check
curl -X POST http://localhost:15002/admin/integrity-check
```

---

## Failover Procedures

### Scenario: Master Node Failure

**Severity:** Critical
**Expected Resolution Time:** 5-15 minutes

#### Automatic Failover (if configured)

```bash
# Check current master
curl http://load-balancer:15002/cluster/status

# Monitor failover progress
kubectl logs -f -l app=vectorizer | grep -i failover

# Verify new master
curl http://load-balancer:15002/cluster/status | jq '.master'
```

#### Manual Failover

**Step 1: Identify healthy replica**
```bash
# List replicas and their status
for pod in $(kubectl get pods -l app=vectorizer -o name); do
  echo -n "$pod: "
  kubectl exec $pod -- curl -s localhost:15002/replication/status | jq -r '.role, .lag_ms'
done
```

**Step 2: Promote replica**
```bash
# Choose replica with lowest lag
REPLICA="vectorizer-1"

# Promote to master
kubectl exec $REPLICA -- curl -X POST localhost:15002/replication/promote

# Verify promotion
kubectl exec $REPLICA -- curl localhost:15002/replication/status
```

**Step 3: Update routing**
```bash
# Update service to point to new master
kubectl patch service vectorizer-master \
  -p '{"spec":{"selector":{"statefulset.kubernetes.io/pod-name":"vectorizer-1"}}}'

# Update DNS if using external DNS
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123456 \
  --change-batch file://update-master-dns.json
```

**Step 4: Recover old master as replica**
```bash
# When old master comes back
kubectl exec vectorizer-0 -- curl -X POST localhost:15002/replication/demote

# Configure to follow new master
kubectl exec vectorizer-0 -- curl -X POST localhost:15002/replication/follow \
  -d '{"master": "vectorizer-1.vectorizer:15003"}'
```

---

## Integrity Check

### Scenario: Periodic Data Integrity Verification

**Severity:** Low (routine)
**Expected Resolution Time:** 10-60 minutes

#### Procedure

**Step 1: Run integrity check**
```bash
# Full integrity check
curl -X POST http://localhost:15002/admin/integrity-check

# Collection-specific
curl -X POST http://localhost:15002/admin/integrity-check \
  -d '{"collections": ["important_collection"]}'
```

**Step 2: Review results**
```bash
# Get check status
curl http://localhost:15002/admin/integrity-check/status

# Expected output:
# {
#   "status": "completed",
#   "checked_vectors": 1000000,
#   "errors": 0,
#   "warnings": 2,
#   "duration_seconds": 45
# }
```

**Step 3: Handle issues**
```bash
# If errors found, check logs
kubectl logs -l app=vectorizer | grep -i "integrity\|error\|corrupt"

# Rebuild index if needed
curl -X POST http://localhost:15002/collections/affected_collection/index/rebuild
```

---

## Performance Troubleshooting

### Scenario: High Search Latency

**Severity:** Medium
**Expected Resolution Time:** 30-60 minutes

#### Investigation

**Step 1: Identify scope**
```bash
# Check overall latency
curl http://localhost:15002/metrics | grep search_latency

# Check per-collection
curl http://localhost:15002/stats | jq '.collections[] | {name, avg_search_ms}'
```

**Step 2: Check resources**
```bash
# CPU and memory
kubectl top pods -l app=vectorizer

# Disk I/O
kubectl exec vectorizer-0 -- iostat -x 1 5

# Network
kubectl exec vectorizer-0 -- ss -s
```

**Step 3: Analyze queries**
```bash
# Enable slow query logging (if not enabled)
kubectl exec vectorizer-0 -- curl -X POST localhost:15002/admin/config \
  -d '{"logging":{"slow_query_threshold_ms":100}}'

# Check slow queries
kubectl logs -l app=vectorizer | grep "slow_query"
```

#### Resolution Options

**Option A: Scale horizontally**
```bash
kubectl scale deployment vectorizer --replicas=5
```

**Option B: Tune HNSW parameters**
```bash
# Reduce ef_search for faster (less accurate) search
curl -X PATCH http://localhost:15002/collections/slow_collection/config \
  -d '{"hnsw":{"ef_search":50}}'
```

**Option C: Add caching**
```bash
# Enable query cache
curl -X POST http://localhost:15002/admin/config \
  -d '{"cache":{"query_cache":{"enabled":true,"size_mb":512}}}'
```

**Option D: Optimize collection**
```bash
# Trigger index optimization
curl -X POST http://localhost:15002/collections/slow_collection/optimize
```

---

## Capacity Management

### Scenario: Storage Running Low

**Severity:** Medium
**Expected Resolution Time:** 1-4 hours

#### Investigation
```bash
# Check current usage
kubectl exec vectorizer-0 -- df -h /data

# Find large collections
curl http://localhost:15002/stats | jq '.collections | sort_by(-.size_bytes) | .[0:5]'
```

#### Resolution Options

**Option A: Expand PVC**
```bash
# Check if StorageClass allows expansion
kubectl get sc fast-ssd -o yaml | grep allowVolumeExpansion

# Expand PVC
kubectl patch pvc data-vectorizer-0 \
  -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'
```

**Option B: Enable compression**
```bash
curl -X POST http://localhost:15002/admin/config \
  -d '{"storage":{"compression":"zstd","compression_level":3}}'

# Compact existing data
curl -X POST http://localhost:15002/admin/compact
```

**Option C: Clean up old data**
```bash
# List collections by age
curl http://localhost:15002/collections?include_metadata=true | \
  jq 'sort_by(.created_at) | .[0:10]'

# Delete old/unused collections
curl -X DELETE http://localhost:15002/collections/old_unused_collection
```

**Option D: Add shards**
```bash
# Add new shard node
helm upgrade vectorizer hivellm/vectorizer \
  --set cluster.shards=4

# Rebalance data
curl -X POST http://localhost:15002/cluster/rebalance
```

---

## Security Incidents

### Scenario: Suspected Unauthorized Access

**Severity:** Critical
**Expected Resolution Time:** Varies

#### Immediate Actions

**Step 1: Contain**
```bash
# Block suspicious IPs
kubectl exec vectorizer-0 -- curl -X POST localhost:15002/admin/ip-block \
  -d '{"ips":["1.2.3.4"]}'

# Revoke compromised API keys
curl -X POST http://localhost:15002/admin/keys/revoke \
  -H "Authorization: Bearer $ADMIN_KEY" \
  -d '{"key_id":"compromised_key_id"}'
```

**Step 2: Investigate**
```bash
# Check audit logs
kubectl logs -l app=vectorizer | grep -i audit

# Export access logs
kubectl exec vectorizer-0 -- cat /var/log/vectorizer/access.log > access.log

# Look for suspicious patterns
grep -E "401|403|DELETE" access.log | sort | uniq -c | sort -rn | head
```

**Step 3: Assess damage**
```bash
# Check for data exfiltration
grep -E "large_export|bulk_download" access.log

# Check for data modification
grep -E "DELETE|PUT|POST" access.log | grep -v search

# Compare current state with known good baseline
curl http://localhost:15002/stats > current_stats.json
diff baseline_stats.json current_stats.json
```

**Step 4: Recover**
```bash
# If data compromised, restore from backup
# See "Data Restoration" runbook

# Rotate all credentials
./scripts/rotate-credentials.sh

# Update security configuration
kubectl apply -f security-hardening.yaml
```

**Step 5: Post-incident**
- Document timeline and actions taken
- Identify root cause
- Update security policies
- Notify affected users/tenants

---

## Quick Reference

### Common Commands

```bash
# Health check
curl http://localhost:15002/health

# Cluster status
curl http://localhost:15002/cluster/status

# Replication status
curl http://localhost:15002/replication/status

# Stats
curl http://localhost:15002/stats

# List collections
curl http://localhost:15002/collections

# Force garbage collection
curl -X POST http://localhost:15002/admin/gc

# Trigger snapshot
curl -X POST http://localhost:15002/qdrant/snapshots

# View metrics
curl http://localhost:15002/metrics
```

### Useful Kubectl Commands

```bash
# Get all Vectorizer pods
kubectl get pods -l app=vectorizer -o wide

# Get logs from all pods
kubectl logs -l app=vectorizer --all-containers

# Execute command on all pods
for pod in $(kubectl get pods -l app=vectorizer -o name); do
  kubectl exec $pod -- <command>
done

# Port forward for debugging
kubectl port-forward svc/vectorizer 15002:15002
```

### Escalation Path

1. **L1 Support:** Basic troubleshooting, restarts, log collection
2. **L2 Support:** Failover, data restoration, configuration changes
3. **L3 Engineering:** Code-level debugging, security incidents
4. **Vendor Support:** HiveLLM support for critical issues

### Contact Information

- **On-call:** pagerduty.com/vectorizer
- **Slack:** #vectorizer-ops
- **Email:** vectorizer-support@example.com
- **Vendor:** support@hivellm.com
