---
title: Operations
module: operations
id: operations-index
order: 0
description: Service management, monitoring, backup, and troubleshooting
tags: [operations, service, monitoring, backup, troubleshooting]
---

# Operations

Complete guides for operating and maintaining Vectorizer in production.

## Guides

### [Service Management](./SERVICE_MANAGEMENT.md)
Managing the Vectorizer service:
- Linux systemd service management
- Windows Service management
- Starting, stopping, restarting
- Service configuration

### [Log Management](./LOGS.md)
Managing and analyzing logs:
- Log locations and access
- Log filtering and search
- Log rotation
- Log aggregation (ELK, Loki)
- Log analysis

### [Monitoring](./MONITORING.md)
Monitoring Vectorizer performance:
- Health checks
- Prometheus metrics
- Grafana dashboards
- Alerting setup
- Performance monitoring

### [Backup and Restore](./BACKUP.md)
Data protection:
- Backup procedures
- Restore operations
- Automated backups
- Scheduled tasks

### [Troubleshooting](./TROUBLESHOOTING.md)
Common issues and solutions:
- Service issues
- Connection problems
- Performance issues
- Data issues
- API errors
- Installation problems
- Debugging tips

## Quick Reference

### Service Status

**Linux:**
```bash
sudo systemctl status vectorizer
```

**Windows:**
```powershell
Get-Service Vectorizer
```

### View Logs

**Linux:**
```bash
sudo journalctl -u vectorizer -f
```

**Windows:**
```powershell
Get-EventLog -LogName Application -Source Vectorizer -Newest 50
```

### Health Check

```bash
curl http://localhost:15002/health
```

## Related Topics

- [Configuration Guide](../configuration/CONFIGURATION.md) - Server configuration
- [Performance Guide](../configuration/PERFORMANCE_TUNING.md) - Performance optimization
