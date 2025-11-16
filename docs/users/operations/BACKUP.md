---
title: Backup and Restore
module: backup-restore
id: backup-guide
order: 1
description: Backup and restore Vectorizer collections and data
tags: [backup, restore, snapshot, data-protection]
---

# Backup and Restore

Complete guide to backing up and restoring Vectorizer data.

## Backup Strategies

### Manual Backup

**Linux:**

```bash
# Stop service
sudo systemctl stop vectorizer

# Backup data directory
sudo tar -czf vectorizer-backup-$(date +%Y%m%d).tar.gz /var/lib/vectorizer

# Start service
sudo systemctl start vectorizer
```

**Windows:**

```powershell
# Stop service
Stop-Service Vectorizer

# Backup data directory
$backupPath = "C:\backups\vectorizer-backup-$(Get-Date -Format 'yyyyMMdd').zip"
Compress-Archive -Path "$env:ProgramData\Vectorizer" -DestinationPath $backupPath

# Start service
Start-Service Vectorizer
```

### Automated Backup Script

**Linux:**

```bash
#!/bin/bash
# backup-vectorizer.sh

BACKUP_DIR="/backups/vectorizer"
DATA_DIR="/var/lib/vectorizer"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup
sudo tar -czf "$BACKUP_DIR/vectorizer-$DATE.tar.gz" $DATA_DIR

# Keep only last 7 days
find $BACKUP_DIR -name "vectorizer-*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/vectorizer-$DATE.tar.gz"
```

**Windows (PowerShell):**

```powershell
# backup-vectorizer.ps1

$BackupDir = "C:\backups\vectorizer"
$DataDir = "$env:ProgramData\Vectorizer"
$Date = Get-Date -Format "yyyyMMdd_HHmmss"

# Create backup directory
New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null

# Backup
$BackupPath = "$BackupDir\vectorizer-$Date.zip"
Compress-Archive -Path $DataDir -DestinationPath $BackupPath

# Keep only last 7 days
Get-ChildItem $BackupDir -Filter "vectorizer-*.zip" |
    Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-7) } |
    Remove-Item

Write-Host "Backup completed: $BackupPath"
```

### Scheduled Backups

**Linux (cron):**

```bash
# Add to crontab: crontab -e
# Daily backup at 2 AM
0 2 * * * /path/to/backup-vectorizer.sh
```

**Windows (Task Scheduler):**

```powershell
# Create scheduled task
$Action = New-ScheduledTaskAction -Execute "PowerShell.exe" -Argument "-File C:\scripts\backup-vectorizer.ps1"
$Trigger = New-ScheduledTaskTrigger -Daily -At 2am
Register-ScheduledTask -TaskName "Vectorizer Backup" -Action $Action -Trigger $Trigger
```

## Restore from Backup

### Full Restore

**Linux:**

```bash
# Stop service
sudo systemctl stop vectorizer

# Restore from backup
sudo tar -xzf vectorizer-backup-20241116.tar.gz -C /

# Fix permissions
sudo chown -R vectorizer:vectorizer /var/lib/vectorizer

# Start service
sudo systemctl start vectorizer
```

**Windows:**

```powershell
# Stop service
Stop-Service Vectorizer

# Restore from backup
Expand-Archive -Path "C:\backups\vectorizer-backup-20241116.zip" -DestinationPath "$env:ProgramData\" -Force

# Start service
Start-Service Vectorizer
```

### Selective Restore

Restore specific collections:

```bash
# Extract specific collection files
tar -xzf vectorizer-backup.tar.gz var/lib/vectorizer/collections/my_collection.vecdb

# Copy to data directory
sudo cp var/lib/vectorizer/collections/my_collection.vecdb /var/lib/vectorizer/collections/

# Restart service
sudo systemctl restart vectorizer
```

## Backup Best Practices

1. **Regular backups**: Daily backups for production
2. **Off-site storage**: Store backups in different location
3. **Test restores**: Regularly test restore procedures
4. **Version control**: Keep multiple backup versions
5. **Documentation**: Document backup and restore procedures

## Related Topics

- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Service operations
- [Troubleshooting](./TROUBLESHOOTING.md) - Data recovery
