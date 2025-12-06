# Vectorizer Production Deployment Plan

HiveHub Cluster Mode - Production Deployment

## Deployment Summary

| Item | Value |
|------|-------|
| Version | 1.8.0 |
| Feature | HiveHub Cluster Mode |
| Deployment Type | Rolling Update |
| Estimated Duration | 2-4 hours |
| Rollback Time | < 15 minutes |

## Pre-Deployment (D-7 to D-1)

### Week Before

- [ ] **Code Freeze** - No new features merged
- [ ] **Final Testing**
  - All unit tests passing (865 tests)
  - Integration tests passing
  - Load tests completed
  - Security scan completed

- [ ] **Documentation Review**
  - API documentation updated
  - Runbooks reviewed
  - Rollback plan reviewed

- [ ] **Infrastructure Preparation**
  - Kubernetes cluster capacity verified
  - Monitoring dashboards ready
  - Alert rules configured
  - Log aggregation configured

### Day Before (D-1)

- [ ] **Final Backup**
  ```bash
  # Create full backup
  curl -X POST http://production:15002/qdrant/snapshots

  # Upload to S3
  aws s3 cp /data/snapshots/latest.tar.gz \
    s3://backups/vectorizer/pre-1.8.0-$(date +%Y%m%d).tar.gz

  # Verify backup
  aws s3 ls s3://backups/vectorizer/ | tail -1
  ```

- [ ] **Tag Release**
  ```bash
  git tag -a v1.8.0 -m "HiveHub Cluster Mode Release"
  git push origin v1.8.0
  ```

- [ ] **Build Production Image**
  ```bash
  docker build -t hivellm/vectorizer:1.8.0 .
  docker push hivellm/vectorizer:1.8.0
  ```

- [ ] **Notify Teams**
  - Platform team
  - HiveHub team
  - Customer support
  - On-call engineers

## Deployment Day (D-0)

### Pre-Deployment Checks (T-60 min)

```bash
# 1. Verify cluster health
kubectl get nodes
kubectl top nodes

# 2. Check current deployment
kubectl get deployment vectorizer -o wide
kubectl get pods -l app=vectorizer

# 3. Verify monitoring
curl http://prometheus:9090/-/healthy
curl http://grafana:3000/api/health

# 4. Check HiveHub connectivity
curl -s https://api.hivehub.cloud/health

# 5. Verify backup exists
aws s3 ls s3://backups/vectorizer/ | tail -1
```

### Deployment Procedure

#### Phase 1: Canary Deployment (T+0)

Deploy to 1 pod first:

```bash
# Update image for canary
kubectl set image deployment/vectorizer \
  vectorizer=hivellm/vectorizer:1.8.0 \
  --record

# Pause rollout after first pod
kubectl rollout pause deployment/vectorizer

# Wait for canary pod
kubectl rollout status deployment/vectorizer --timeout=5m
```

**Canary Validation (15 min):**
```bash
# Get canary pod
CANARY=$(kubectl get pods -l app=vectorizer --sort-by=.metadata.creationTimestamp -o name | tail -1)

# Check logs
kubectl logs $CANARY --tail=100

# Check health
kubectl exec $CANARY -- curl -s localhost:15002/health

# Check metrics
kubectl exec $CANARY -- curl -s localhost:15002/metrics | grep -E "^vectorizer_"

# Test HiveHub integration
kubectl exec $CANARY -- curl -s localhost:15002/cluster/status
```

#### Phase 2: Gradual Rollout (T+15)

If canary is healthy, continue rollout:

```bash
# Resume rollout
kubectl rollout resume deployment/vectorizer

# Monitor rollout
kubectl rollout status deployment/vectorizer

# Watch pods
kubectl get pods -l app=vectorizer -w
```

#### Phase 3: Verification (T+30)

```bash
# 1. All pods healthy
kubectl get pods -l app=vectorizer
for pod in $(kubectl get pods -l app=vectorizer -o name); do
  echo "$pod: $(kubectl exec $pod -- curl -s localhost:15002/health | jq -r '.status')"
done

# 2. Check error rate
curl "http://prometheus:9090/api/v1/query?query=rate(vectorizer_errors_total[5m])"

# 3. Check latency
curl "http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,rate(vectorizer_search_latency_seconds_bucket[5m]))"

# 4. Verify HiveHub integration
curl http://vectorizer:15002/cluster/status

# 5. Test tenant isolation
./scripts/test-tenant-isolation.sh
```

### Post-Deployment (T+60)

- [ ] **Update Status Page**
  ```
  Vectorizer 1.8.0 deployed successfully.
  New feature: HiveHub Cluster Mode enabled.
  ```

- [ ] **Notify Stakeholders**
  - Deployment complete email
  - Update ticket/issue

- [ ] **Monitor (2 hours)**
  - Watch error rates
  - Watch latency
  - Watch resource usage
  - Check logs for anomalies

## Deployment Checklist

### Pre-Deployment
- [ ] All tests passing
- [ ] Security scan clean
- [ ] Documentation updated
- [ ] Backup completed
- [ ] Team notified
- [ ] On-call confirmed

### During Deployment
- [ ] Canary deployed
- [ ] Canary validated (15 min)
- [ ] Full rollout completed
- [ ] All pods healthy
- [ ] No errors in logs

### Post-Deployment
- [ ] Health checks passing
- [ ] Error rate < 1%
- [ ] Latency normal
- [ ] HiveHub integration working
- [ ] Tenant isolation verified
- [ ] Status page updated
- [ ] Stakeholders notified

## Rollback Triggers

Immediately rollback if:
- Error rate > 10% for 5+ minutes
- P99 latency > 500ms for 5+ minutes
- Any pod crash-looping
- Authentication failures
- Data corruption detected

See [ROLLBACK_PLAN.md](ROLLBACK_PLAN.md) for procedures.

## Monitoring During Deployment

### Key Metrics to Watch

| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| Error Rate | < 1% | 1-5% | > 5% |
| P99 Latency | < 50ms | 50-100ms | > 100ms |
| CPU Usage | < 70% | 70-85% | > 85% |
| Memory Usage | < 80% | 80-90% | > 90% |
| Pod Restarts | 0 | 1-2 | > 2 |

### Grafana Dashboards
- Main: `http://grafana:3000/d/vectorizer-main`
- HiveHub: `http://grafana:3000/d/vectorizer-hub`
- Alerts: `http://grafana:3000/alerting/list`

### Log Queries
```bash
# Errors
kubectl logs -l app=vectorizer | grep -i error

# HiveHub issues
kubectl logs -l app=vectorizer | grep -i hub

# Auth failures
kubectl logs -l app=vectorizer | grep -i "auth\|unauthorized"
```

## Communication Plan

### Before Deployment
```
Subject: [Planned] Vectorizer 1.8.0 Deployment - HiveHub Cluster Mode

Deployment scheduled for: YYYY-MM-DD HH:MM UTC
Expected duration: 2-4 hours
Impact: Rolling update, minimal disruption expected

New features:
- HiveHub Cloud integration
- Multi-tenant support
- API key authentication

No action required from users.
```

### During Deployment
```
Subject: [In Progress] Vectorizer 1.8.0 Deployment

Status: Deployment in progress
Started: HH:MM UTC
Current phase: Canary/Rollout/Verification

Updates will be posted every 30 minutes.
```

### After Deployment
```
Subject: [Complete] Vectorizer 1.8.0 Deployed Successfully

Deployment completed: HH:MM UTC
Duration: X hours Y minutes
Status: All systems operational

New features now available:
- HiveHub Cloud integration
- Multi-tenant support
- API key authentication

Documentation: https://docs.example.com/vectorizer/1.8.0
```

## Emergency Contacts

| Role | Name | Phone | Slack |
|------|------|-------|-------|
| Deployment Lead | | | |
| Platform On-call | | | @platform-oncall |
| HiveHub Contact | | | |
| Management Escalation | | | |

## Appendix: Useful Commands

```bash
# Quick deployment status
kubectl rollout status deployment/vectorizer

# Force rollback
kubectl rollout undo deployment/vectorizer

# Check all events
kubectl get events --sort-by='.lastTimestamp' | grep vectorizer

# Emergency scale down
kubectl scale deployment/vectorizer --replicas=0

# Get current image
kubectl get deployment vectorizer -o jsonpath='{.spec.template.spec.containers[0].image}'
```

## Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Development Lead | | | |
| Platform Lead | | | |
| Security Lead | | | |
| Product Owner | | | |

**Deployment Approved:** [ ] Yes [ ] No

**Notes:**
