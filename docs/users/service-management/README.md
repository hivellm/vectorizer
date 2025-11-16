---
title: Service Management
module: service-management
id: service-management-index
order: 0
description: Managing the Vectorizer system service
tags: [service, systemd, windows-service, management]
---

# Service Management

Complete guide to managing the Vectorizer system service.

## Guides

### [Service Management Guide](./SERVICE_MANAGEMENT.md)
Managing the Vectorizer service:
- Starting and stopping the service
- Checking service status
- Enabling/disabling auto-start
- Service configuration

### [Log Management](./LOGS.md)
Managing and viewing logs:
- Log locations and access
- Log filtering and search
- Log rotation
- Log aggregation
- Log analysis

## Quick Reference

### Linux (systemd)

```bash
# Status
sudo systemctl status vectorizer

# Start/Stop/Restart
sudo systemctl start vectorizer
sudo systemctl stop vectorizer
sudo systemctl restart vectorizer

# Enable/Disable auto-start
sudo systemctl enable vectorizer
sudo systemctl disable vectorizer

# View logs
sudo journalctl -u vectorizer -f
```

### Windows

```powershell
# Status
Get-Service Vectorizer

# Start/Stop/Restart
Start-Service Vectorizer
Stop-Service Vectorizer
Restart-Service Vectorizer

# View logs
Get-EventLog -LogName Application -Source Vectorizer -Newest 50
```

## Related Topics

- [Installation Guide](../installation/INSTALLATION.md) - Service installation
- [Configuration Guide](../configuration/CONFIGURATION.md) - Service configuration
- [Monitoring Guide](../monitoring/MONITORING.md) - Service monitoring

