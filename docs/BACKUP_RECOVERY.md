# Backup & Recovery Guide

Complete guide for backing up and recovering Vectorizer data.

## Backup Strategy

### Backup Types

1. **Full Backup**: Complete snapshot of all data
2. **Incremental Backup**: Only changed data since last backup
3. **Continuous Backup**: Real-time replication

### Backup Frequency

- **Production**: Daily full backups, hourly incremental
- **Development**: Weekly full backups
- **Critical Systems**: Continuous replication + daily backups

### Retention Policy

- **Daily Backups**: 30 days
- **Weekly Backups**: 12 weeks
- **Monthly Backups**: 12 months

## Backup Procedures

### Manual Backup

```bash
# Create backup via API
curl -X POST http://localhost:15002/api/backups/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "backup-$(date +%Y%m%d)",
    "collections": []
  }'

# List backups
curl http://localhost:15002/api/backups

# Download backup
curl http://localhost:15002/api/backups/{backup_id}/download \
  -o backup.tar.gz
```

### Automated Backup Script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/vectorizer"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="backup-$DATE"

# Create backup
curl -X POST http://localhost:15002/api/backups/create \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"$BACKUP_NAME\"}"

# Wait for backup to complete
sleep 30

# Copy backup to storage
cp -r /var/lib/vectorizer/backups/$BACKUP_NAME $BACKUP_DIR/

# Cleanup old backups (keep last 30 days)
find $BACKUP_DIR -name "backup-*" -mtime +30 -delete

# Upload to S3 (optional)
aws s3 sync $BACKUP_DIR s3://vectorizer-backups/
```

### Kubernetes CronJob

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: vectorizer-backup
  namespace: vectorizer
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: backup
              image: curlimages/curl:latest
              command:
                - /bin/sh
                - -c
                - |
                  curl -X POST http://vectorizer:15002/api/backups/create \
                    -H "Content-Type: application/json" \
                    -d '{"name": "backup-$(date +%Y%m%d)"}'
          restartPolicy: OnFailure
```

## Restore Procedures

### Full Restore

```bash
# Upload backup
scp backup.tar.gz user@server:/tmp/

# Extract backup
tar -xzf backup.tar.gz -C /tmp/backup

# Restore via API
curl -X POST http://localhost:15002/api/backups/restore \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "backup-20240115",
    "collections": []
  }'
```

### Selective Restore

```bash
# Restore specific collections
curl -X POST http://localhost:15002/api/backups/restore \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "backup-20240115",
    "collections": ["collection1", "collection2"]
  }'
```

### Point-in-Time Recovery

```bash
# List available backups
curl http://localhost:15002/api/backups

# Restore from specific backup
curl -X POST http://localhost:15002/api/backups/restore \
  -H "Content-Type: application/json" \
  -d '{
    "backup_id": "backup-20240115-120000"
  }'
```

## Disaster Recovery

### Recovery Scenarios

#### Scenario 1: Single Node Failure

1. **Detect Failure**: Monitor detects node down
2. **Failover**: Traffic routed to replica
3. **Restore Node**: Restore from latest backup
4. **Rejoin Cluster**: Node rejoins as replica

#### Scenario 2: Data Corruption

1. **Stop Service**: Prevent further corruption
2. **Identify Backup**: Find last known good backup
3. **Restore Data**: Restore from backup
4. **Verify Integrity**: Check data integrity
5. **Resume Service**: Start service

#### Scenario 3: Complete Data Loss

1. **Assess Damage**: Determine scope of loss
2. **Restore Infrastructure**: Recreate infrastructure
3. **Restore Data**: Restore from off-site backup
4. **Verify System**: Test all functionality
5. **Resume Operations**: Gradually resume traffic

### Recovery Time Objectives (RTO)

- **Critical Systems**: < 1 hour
- **Production Systems**: < 4 hours
- **Development Systems**: < 24 hours

### Recovery Point Objectives (RPO)

- **Critical Systems**: < 15 minutes
- **Production Systems**: < 1 hour
- **Development Systems**: < 24 hours

## Backup Storage

### Local Storage

```bash
# Backup directory structure
/backups/vectorizer/
├── daily/
│   ├── backup-20240115.tar.gz
│   ├── backup-20240116.tar.gz
│   └── ...
├── weekly/
│   ├── backup-20240108.tar.gz
│   └── ...
└── monthly/
    ├── backup-202312.tar.gz
    └── ...
```

### Cloud Storage

#### AWS S3

```bash
# Upload to S3
aws s3 sync /backups/vectorizer s3://vectorizer-backups/

# Download from S3
aws s3 sync s3://vectorizer-backups/ /backups/vectorizer/
```

#### Google Cloud Storage

```bash
# Upload to GCS
gsutil -m cp -r /backups/vectorizer gs://vectorizer-backups/

# Download from GCS
gsutil -m cp -r gs://vectorizer-backups/ /backups/vectorizer/
```

### Backup Verification

```bash
#!/bin/bash
# verify-backup.sh

BACKUP_ID=$1

# Verify backup exists
if [ ! -d "/backups/vectorizer/$BACKUP_ID" ]; then
    echo "Backup not found: $BACKUP_ID"
    exit 1
fi

# Verify backup integrity
tar -tzf "/backups/vectorizer/$BACKUP_ID.tar.gz" > /dev/null
if [ $? -ne 0 ]; then
    echo "Backup corrupted: $BACKUP_ID"
    exit 1
fi

# Verify backup size
SIZE=$(du -sh "/backups/vectorizer/$BACKUP_ID" | cut -f1)
echo "Backup verified: $BACKUP_ID ($SIZE)"
```

## Upgrade Procedures

### Pre-Upgrade Checklist

- [ ] Backup current data
- [ ] Test upgrade in staging
- [ ] Review release notes
- [ ] Plan rollback procedure
- [ ] Notify stakeholders

### Upgrade Steps

1. **Backup Current Version**
   ```bash
   ./backup.sh
   ```

2. **Stop Service**
   ```bash
   systemctl stop vectorizer
   ```

3. **Upgrade Binary**
   ```bash
   cp vectorizer-new /usr/local/bin/vectorizer
   ```

4. **Start Service**
   ```bash
   systemctl start vectorizer
   ```

5. **Verify Upgrade**
   ```bash
   curl http://localhost:15002/api/status
   ```

### Rollback Procedure

1. **Stop Service**
   ```bash
   systemctl stop vectorizer
   ```

2. **Restore Previous Version**
   ```bash
   cp vectorizer-old /usr/local/bin/vectorizer
   ```

3. **Restore Data** (if needed)
   ```bash
   ./restore.sh backup-YYYYMMDD
   ```

4. **Start Service**
   ```bash
   systemctl start vectorizer
   ```

## Best Practices

1. **Automate Backups**: Use cron jobs or Kubernetes CronJobs
2. **Test Restores**: Regularly test restore procedures
3. **Off-Site Storage**: Store backups off-site
4. **Encrypt Backups**: Encrypt sensitive backup data
5. **Monitor Backups**: Alert on backup failures
6. **Document Procedures**: Keep runbooks updated
7. **Version Control**: Track backup versions
8. **Regular Testing**: Test disaster recovery procedures

## Troubleshooting

### Backup Failures

1. Check disk space
2. Verify permissions
3. Check network connectivity
4. Review logs for errors

### Restore Failures

1. Verify backup integrity
2. Check disk space
3. Verify permissions
4. Check service status

### Performance Issues

1. Schedule backups during low-traffic periods
2. Use incremental backups
3. Compress backups
4. Use parallel backup streams

