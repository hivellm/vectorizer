# Vectorizer Disaster Recovery Guide

This guide covers disaster recovery (DR) procedures for Vectorizer deployments, including backup strategies, recovery procedures, and high availability configurations.

## Overview

### Recovery Objectives

| Tier | RPO (Recovery Point Objective) | RTO (Recovery Time Objective) | Strategy |
|------|-------------------------------|-------------------------------|----------|
| Tier 1 | < 1 minute | < 5 minutes | Synchronous replication |
| Tier 2 | < 15 minutes | < 30 minutes | Async replication + snapshots |
| Tier 3 | < 1 hour | < 2 hours | Regular backups |
| Tier 4 | < 24 hours | < 8 hours | Daily backups |

## Backup Strategies

### 1. Snapshot Backups

Create point-in-time snapshots:

```bash
# Create snapshot via API
curl -X POST http://localhost:15002/qdrant/snapshots

# Create collection-specific snapshot
curl -X POST http://localhost:15002/qdrant/collections/my_collection/snapshots
```

**Automated Snapshots (Kubernetes CronJob):**
```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: vectorizer-backup
spec:
  schedule: "0 */6 * * *"  # Every 6 hours
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: curlimages/curl
            command:
            - /bin/sh
            - -c
            - |
              # Create snapshot
              SNAPSHOT=$(curl -s -X POST http://vectorizer:15002/qdrant/snapshots | jq -r '.result.name')

              # Upload to S3
              aws s3 cp /data/snapshots/$SNAPSHOT s3://backups/vectorizer/$SNAPSHOT

              # Cleanup old local snapshots (keep last 3)
              ls -t /data/snapshots/ | tail -n +4 | xargs -I {} rm /data/snapshots/{}
          restartPolicy: OnFailure
```

### 2. Continuous Replication

For minimal data loss, use master-replica replication:

```yaml
# Master config
replication:
  enabled: true
  role: master
  sync_mode: sync  # sync or async
  min_replicas: 2  # Wait for at least 2 replicas to confirm writes
```

### 3. Multi-Tenant Backups

For HiveHub Cloud deployments:

```bash
# Backup specific tenant
curl -X POST http://localhost:15002/api/hub/backups \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"collections": ["*"]}'

# List tenant backups
curl http://localhost:15002/api/hub/backups \
  -H "Authorization: Bearer $API_KEY"

# Download backup
curl http://localhost:15002/api/hub/backups/{backup_id}/download \
  -H "Authorization: Bearer $API_KEY" \
  -o backup.tar.gz
```

## Disaster Scenarios

### Scenario 1: Single Node Failure

**Symptoms:**
- Node unreachable
- Health checks failing
- Requests timing out

**Recovery Steps:**

1. **If using replication (Tier 1/2):**
   ```bash
   # Traffic automatically routes to replicas
   # Verify cluster health
   curl http://load-balancer:15002/health

   # Check replica status
   curl http://replica1:15002/replication/status
   ```

2. **Replace failed node:**
   ```bash
   # Provision new node
   kubectl scale statefulset vectorizer --replicas=3

   # New node will auto-sync from master
   ```

3. **Verify data integrity:**
   ```bash
   # Check vector counts match
   curl http://master:15002/stats
   curl http://new-replica:15002/stats
   ```

### Scenario 2: Master Node Failure

**Symptoms:**
- Write operations failing
- Replicas report master unreachable

**Recovery Steps:**

1. **Automatic failover (if configured):**
   ```yaml
   # config.yml
   replication:
     failover:
       enabled: true
       timeout_ms: 5000
       min_healthy_replicas: 1
   ```

2. **Manual failover:**
   ```bash
   # Promote replica to master
   curl -X POST http://replica1:15002/replication/promote

   # Update load balancer to point to new master
   kubectl patch service vectorizer-master -p \
     '{"spec":{"selector":{"statefulset.kubernetes.io/pod-name":"vectorizer-1"}}}'

   # Configure old master as replica when recovered
   ```

3. **Verify cluster state:**
   ```bash
   curl http://new-master:15002/cluster/status
   ```

### Scenario 3: Data Center Failure

**Symptoms:**
- Entire region/DC unreachable
- All nodes in DC failing health checks

**Recovery Steps:**

1. **Activate DR site:**
   ```bash
   # Update DNS to point to DR region
   aws route53 change-resource-record-sets \
     --hosted-zone-id Z123456 \
     --change-batch file://dr-failover.json

   # Or update load balancer
   kubectl patch service vectorizer-global \
     --context dr-cluster \
     -p '{"spec":{"externalTrafficPolicy":"Local"}}'
   ```

2. **Restore from latest backup (if async replication):**
   ```bash
   # Find latest snapshot
   aws s3 ls s3://backups/vectorizer/ --recursive | sort -k1,2 | tail -1

   # Restore snapshot
   curl -X POST http://dr-vectorizer:15002/qdrant/snapshots/recover \
     -d '{"location": "s3://backups/vectorizer/snapshot-2024-01-15.tar"}'
   ```

3. **Verify data and resume operations:**
   ```bash
   # Check collection counts
   curl http://dr-vectorizer:15002/collections

   # Verify vector counts
   for c in $(curl -s http://dr-vectorizer:15002/collections | jq -r '.[]'); do
     echo "$c: $(curl -s http://dr-vectorizer:15002/collections/$c | jq '.vectors_count')"
   done
   ```

### Scenario 4: Data Corruption

**Symptoms:**
- Search returning incorrect results
- Inconsistent vector counts
- CRC/checksum errors in logs

**Recovery Steps:**

1. **Stop affected node(s):**
   ```bash
   kubectl scale deployment vectorizer --replicas=0
   ```

2. **Identify corruption scope:**
   ```bash
   # Check data directory integrity
   find /data -type f -name "*.vecdb" -exec md5sum {} \; > checksums.txt

   # Compare with known good checksums
   diff checksums.txt /backups/checksums-baseline.txt
   ```

3. **Restore from backup:**
   ```bash
   # Option A: Full restore
   rm -rf /data/*
   tar -xzf /backups/latest-snapshot.tar.gz -C /data/

   # Option B: Collection-specific restore
   curl -X POST http://localhost:15002/qdrant/collections/corrupted_collection/snapshots/recover \
     -d '{"location": "/backups/corrupted_collection-snapshot.tar"}'
   ```

4. **Replay any lost transactions:**
   ```bash
   # If using write-ahead log
   vectorizer-cli replay-wal --from-timestamp 2024-01-15T10:00:00Z
   ```

### Scenario 5: Accidental Data Deletion

**Symptoms:**
- Collection or vectors missing
- User reports data loss

**Recovery Steps:**

1. **Check if soft-deleted:**
   ```bash
   # List including deleted
   curl http://localhost:15002/collections?include_deleted=true

   # Restore soft-deleted collection
   curl -X POST http://localhost:15002/collections/deleted_collection/restore
   ```

2. **Point-in-time recovery:**
   ```bash
   # Find backup from before deletion
   aws s3 ls s3://backups/vectorizer/ | grep "2024-01-15"

   # Restore specific collection to new name
   curl -X POST http://localhost:15002/admin/restore \
     -d '{
       "source": "s3://backups/vectorizer/snapshot-2024-01-15.tar",
       "collections": ["deleted_collection"],
       "target_prefix": "restored_"
     }'

   # Verify and rename
   curl http://localhost:15002/collections/restored_deleted_collection
   curl -X POST http://localhost:15002/collections/restored_deleted_collection/rename \
     -d '{"new_name": "deleted_collection"}'
   ```

## Recovery Procedures

### Full System Recovery

1. **Provision infrastructure:**
   ```bash
   # Using Terraform
   terraform apply -var-file=disaster-recovery.tfvars

   # Or Kubernetes
   kubectl apply -f k8s/disaster-recovery/
   ```

2. **Deploy Vectorizer:**
   ```bash
   helm install vectorizer hivellm/vectorizer \
     --values values-production.yaml \
     --set recovery.enabled=true \
     --set recovery.snapshotUrl=s3://backups/latest-snapshot.tar
   ```

3. **Wait for data restoration:**
   ```bash
   # Monitor restore progress
   kubectl logs -f deployment/vectorizer | grep -i restore

   # Check status
   curl http://vectorizer:15002/admin/restore/status
   ```

4. **Verify integrity:**
   ```bash
   # Run integrity check
   curl -X POST http://vectorizer:15002/admin/integrity-check

   # Compare stats with backup manifest
   curl http://vectorizer:15002/stats
   ```

5. **Resume traffic:**
   ```bash
   # Update DNS/load balancer
   # Run smoke tests
   ./scripts/smoke-test.sh

   # Enable traffic
   kubectl patch ingress vectorizer -p '{"metadata":{"annotations":{"nginx.ingress.kubernetes.io/service-upstream":"true"}}}'
   ```

### Tenant Data Recovery

For multi-tenant environments:

```bash
# 1. List available backups for tenant
curl http://localhost:15002/api/hub/backups \
  -H "X-HiveHub-Service: $SERVICE_KEY" \
  -H "X-HiveHub-User-ID: $TENANT_ID"

# 2. Restore specific backup
curl -X POST http://localhost:15002/api/hub/backups/restore \
  -H "X-HiveHub-Service: $SERVICE_KEY" \
  -H "X-HiveHub-User-ID: $TENANT_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "backup_20240115_100000",
    "overwrite": false
  }'

# 3. Verify restoration
curl http://localhost:15002/collections \
  -H "Authorization: Bearer $TENANT_API_KEY"
```

## High Availability Configuration

### Multi-Region Setup

```yaml
# Primary Region
regions:
  primary:
    name: us-east-1
    endpoints:
      - vectorizer-1.us-east-1.example.com
      - vectorizer-2.us-east-1.example.com
    replication:
      role: master

  secondary:
    name: us-west-2
    endpoints:
      - vectorizer-1.us-west-2.example.com
      - vectorizer-2.us-west-2.example.com
    replication:
      role: replica
      master: vectorizer-1.us-east-1.example.com

  disaster_recovery:
    name: eu-west-1
    endpoints:
      - vectorizer-1.eu-west-1.example.com
    replication:
      role: replica
      mode: async
      lag_threshold_ms: 60000
```

### Failover Configuration

```yaml
# config.yml
failover:
  enabled: true

  health_check:
    interval_ms: 5000
    timeout_ms: 2000
    unhealthy_threshold: 3
    healthy_threshold: 2

  automatic:
    enabled: true
    promotion_delay_ms: 10000  # Wait before promoting replica

  notification:
    webhook_url: https://alerts.example.com/vectorizer
    email: ops@example.com
```

## Testing DR Procedures

### Regular DR Drills

```bash
#!/bin/bash
# dr-drill.sh - Monthly DR test

echo "Starting DR drill..."

# 1. Create baseline snapshot
curl -X POST http://production:15002/qdrant/snapshots

# 2. Simulate failure
kubectl cordon production-node-1
kubectl drain production-node-1 --force

# 3. Verify failover
sleep 30
curl http://production:15002/health || exit 1

# 4. Test recovery
kubectl uncordon production-node-1

# 5. Verify sync
sleep 60
PROD_COUNT=$(curl -s http://production:15002/stats | jq '.total_vectors')
REPLICA_COUNT=$(curl -s http://replica:15002/stats | jq '.total_vectors')

if [ "$PROD_COUNT" == "$REPLICA_COUNT" ]; then
  echo "DR drill successful"
else
  echo "WARNING: Vector count mismatch"
fi
```

### Backup Verification

```bash
#!/bin/bash
# verify-backup.sh - Verify backup integrity

BACKUP_FILE=$1
TEST_DB=/tmp/vectorizer-test

# Extract and verify
tar -xzf $BACKUP_FILE -C $TEST_DB

# Start temporary instance
vectorizer --data-dir $TEST_DB --port 15099 &
VPID=$!
sleep 10

# Verify data
COLLECTIONS=$(curl -s http://localhost:15099/collections | jq length)
VECTORS=$(curl -s http://localhost:15099/stats | jq '.total_vectors')

echo "Backup contains $COLLECTIONS collections with $VECTORS vectors"

# Run integrity check
curl -X POST http://localhost:15099/admin/integrity-check

# Cleanup
kill $VPID
rm -rf $TEST_DB
```

## Monitoring and Alerts

### DR-Related Metrics

```promql
# Replication lag
vectorizer_replication_lag_seconds > 60

# Backup age
time() - vectorizer_last_backup_timestamp > 86400

# Node health
count(vectorizer_node_healthy == 0) > 0

# Failover events
increase(vectorizer_failover_total[1h]) > 0
```

### Alert Rules

```yaml
groups:
- name: vectorizer-dr
  rules:
  - alert: VectorizerReplicationLag
    expr: vectorizer_replication_lag_seconds > 300
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Replication lag exceeds 5 minutes"

  - alert: VectorizerBackupStale
    expr: time() - vectorizer_last_backup_timestamp > 86400
    for: 1h
    labels:
      severity: warning
    annotations:
      summary: "No backup in last 24 hours"

  - alert: VectorizerFailover
    expr: increase(vectorizer_failover_total[5m]) > 0
    labels:
      severity: critical
    annotations:
      summary: "Failover event detected"
```

## Runbook Reference

- [Node Recovery Runbook](RUNBOOKS.md#node-recovery)
- [Data Restoration Runbook](RUNBOOKS.md#data-restoration)
- [Failover Runbook](RUNBOOKS.md#failover)
- [Integrity Check Runbook](RUNBOOKS.md#integrity-check)
