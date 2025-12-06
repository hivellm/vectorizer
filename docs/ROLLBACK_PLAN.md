# Vectorizer Rollback Plan

This document describes the rollback procedures for the HiveHub Cluster Mode deployment.

## Overview

| Scenario | Rollback Time | Data Impact | Procedure |
|----------|---------------|-------------|-----------|
| Minor issues | < 5 min | None | Feature flag toggle |
| Major issues | < 15 min | None | Version rollback |
| Critical issues | < 30 min | Possible | Full rollback + restore |

## Pre-Deployment Checklist

Before deploying, ensure:

- [ ] Current version tagged in git
- [ ] Database backup completed
- [ ] Configuration backup saved
- [ ] Health check endpoints verified
- [ ] Monitoring dashboards ready
- [ ] On-call team notified

```bash
# Tag current version
git tag -a v1.7.x-pre-cluster -m "Pre-cluster mode release"

# Create backup
curl -X POST http://localhost:15002/qdrant/snapshots
aws s3 cp /data/snapshots/latest.tar.gz s3://backups/pre-cluster/

# Save current config
kubectl get configmap vectorizer-config -o yaml > config-backup.yaml
```

## Rollback Triggers

Initiate rollback if any of these occur:

### Critical (Immediate Rollback)
- Data corruption detected
- Authentication completely broken
- All requests failing (>90% error rate)
- Security breach detected

### Major (Rollback within 15 min)
- Error rate > 10% sustained for 5+ minutes
- P99 latency > 500ms sustained
- Replication failing
- HiveHub integration completely broken

### Minor (Consider Rollback)
- Error rate > 5% sustained for 15+ minutes
- Performance degradation > 50%
- Non-critical features broken

## Rollback Procedures

### Level 1: Feature Flag Toggle (< 5 min)

Disable cluster mode without redeployment:

```bash
# Option A: Environment variable
kubectl set env deployment/vectorizer HIVEHUB_ENABLED=false

# Option B: ConfigMap update
kubectl patch configmap vectorizer-config -p '{"data":{"hub.enabled":"false"}}'

# Restart pods
kubectl rollout restart deployment/vectorizer

# Verify
kubectl rollout status deployment/vectorizer
curl http://vectorizer:15002/health
```

### Level 2: Version Rollback (< 15 min)

Roll back to previous version:

```bash
# Check deployment history
kubectl rollout history deployment/vectorizer

# Rollback to previous version
kubectl rollout undo deployment/vectorizer

# Or rollback to specific revision
kubectl rollout undo deployment/vectorizer --to-revision=3

# Verify
kubectl rollout status deployment/vectorizer
curl http://vectorizer:15002/health
```

**Using Helm:**
```bash
# Check history
helm history vectorizer

# Rollback
helm rollback vectorizer 1

# Verify
helm status vectorizer
```

### Level 3: Full Rollback + Data Restore (< 30 min)

For critical issues with potential data corruption:

```bash
# 1. Stop all traffic
kubectl scale deployment vectorizer --replicas=0

# 2. Restore previous configuration
kubectl apply -f config-backup.yaml

# 3. Restore data from backup
kubectl exec vectorizer-0 -- rm -rf /data/*
kubectl cp s3://backups/pre-cluster/snapshot.tar.gz vectorizer-0:/tmp/
kubectl exec vectorizer-0 -- tar -xzf /tmp/snapshot.tar.gz -C /data/

# 4. Deploy previous version
kubectl set image deployment/vectorizer vectorizer=hivellm/vectorizer:1.7.x

# 5. Scale back up
kubectl scale deployment vectorizer --replicas=3

# 6. Verify
kubectl rollout status deployment/vectorizer
curl http://vectorizer:15002/health
curl http://vectorizer:15002/stats
```

## Post-Rollback Actions

### Immediate (within 1 hour)

1. **Verify System Health**
   ```bash
   # Check all pods running
   kubectl get pods -l app=vectorizer

   # Check health endpoints
   for pod in $(kubectl get pods -l app=vectorizer -o name); do
     kubectl exec $pod -- curl -s localhost:15002/health
   done

   # Check error rates
   curl http://prometheus:9090/api/v1/query?query=rate(vectorizer_errors_total[5m])
   ```

2. **Notify Stakeholders**
   - Send incident notification
   - Update status page
   - Notify affected tenants (if applicable)

3. **Preserve Evidence**
   ```bash
   # Save logs
   kubectl logs -l app=vectorizer --since=1h > rollback-logs.txt

   # Save metrics
   curl "http://prometheus:9090/api/v1/query_range?query=vectorizer_requests_total&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=60" > rollback-metrics.json

   # Save events
   kubectl get events --sort-by='.lastTimestamp' > rollback-events.txt
   ```

### Short-term (within 24 hours)

1. **Root Cause Analysis**
   - Review logs for errors
   - Analyze metrics for anomalies
   - Check configuration differences
   - Review recent code changes

2. **Document Incident**
   - Timeline of events
   - Impact assessment
   - Root cause
   - Remediation steps

3. **Plan Fix**
   - Identify required changes
   - Create hotfix branch
   - Plan re-deployment

## Rollback Verification Checklist

After rollback, verify:

- [ ] All pods running and healthy
- [ ] Health endpoint returning 200
- [ ] Collections accessible
- [ ] Search working correctly
- [ ] Error rate < 1%
- [ ] Latency within normal range
- [ ] No data loss detected
- [ ] Monitoring showing normal metrics

## Communication Templates

### Incident Start
```
[INCIDENT] Vectorizer Cluster Mode Rollback Initiated

Time: YYYY-MM-DD HH:MM UTC
Severity: Critical/Major/Minor
Trigger: <description>
Impact: <affected services/users>
Status: Rollback in progress

Updates will follow.
```

### Rollback Complete
```
[RESOLVED] Vectorizer Rollback Complete

Time: YYYY-MM-DD HH:MM UTC
Duration: X minutes
Resolution: Rolled back to version X.X.X
Impact: <summary of impact>
Next Steps: Root cause analysis in progress

Service is now stable.
```

## Emergency Contacts

| Role | Name | Contact |
|------|------|---------|
| On-call Engineer | | |
| Platform Lead | | |
| Security Lead | | |
| HiveHub Contact | | |

## Rollback Testing

Regularly test rollback procedures:

```bash
# Monthly rollback drill
./scripts/rollback-drill.sh

# Verify backup restore
./scripts/test-backup-restore.sh

# Test feature flag toggle
./scripts/test-feature-toggle.sh
```

## Version Compatibility Matrix

| Vectorizer Version | HiveHub SDK | Breaking Changes |
|-------------------|-------------|------------------|
| 1.8.0 (cluster)   | 1.0.0       | New auth headers |
| 1.7.x (standalone)| N/A         | Baseline |

## Appendix: Quick Commands

```bash
# Check current version
kubectl get deployment vectorizer -o jsonpath='{.spec.template.spec.containers[0].image}'

# Check rollout history
kubectl rollout history deployment/vectorizer

# Quick health check
curl -s http://vectorizer:15002/health | jq

# Check error rate
kubectl logs -l app=vectorizer --since=5m | grep -c ERROR

# Emergency stop
kubectl scale deployment vectorizer --replicas=0
```
