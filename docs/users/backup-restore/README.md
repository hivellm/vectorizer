---
title: Backup and Restore
module: backup-restore
id: backup-restore-index
order: 0
description: Backup and restore guides for Vectorizer
tags: [backup, restore, data-protection]
---

# Backup and Restore

Guides for backing up and restoring Vectorizer data.

## Guides

### [Backup Guide](./BACKUP.md)
Complete backup and restore procedures:
- Manual backup procedures
- Automated backup scripts
- Scheduled backups
- Restore procedures
- Best practices

## Quick Start

```bash
# Linux: Quick backup
sudo systemctl stop vectorizer
sudo tar -czf backup.tar.gz /var/lib/vectorizer
sudo systemctl start vectorizer

# Restore
sudo systemctl stop vectorizer
sudo tar -xzf backup.tar.gz -C /
sudo systemctl start vectorizer
```

## Related Topics

- [Service Management](../service-management/SERVICE_MANAGEMENT.md) - Service operations
- [Configuration](../configuration/CONFIGURATION.md) - Data directory configuration

