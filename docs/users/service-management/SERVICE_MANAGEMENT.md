---
title: Service Management
module: service-management
id: service-management-guide
order: 1
description: Managing the Vectorizer system service on Linux and Windows
tags: [service, systemd, windows-service, management]
---

# Service Management

Vectorizer runs as a system service that starts automatically on boot and restarts on failure.

## Linux (systemd)

### Check Status

```bash
sudo systemctl status vectorizer
```

### Start Service

```bash
sudo systemctl start vectorizer
```

### Stop Service

```bash
sudo systemctl stop vectorizer
```

### Restart Service

```bash
sudo systemctl restart vectorizer
```

### View Logs

```bash
# Follow logs in real-time
sudo journalctl -u vectorizer -f

# View recent logs
sudo journalctl -u vectorizer -n 100

# View logs since boot
sudo journalctl -u vectorizer --since boot
```

### Enable/Disable Auto-Start

```bash
# Enable auto-start on boot
sudo systemctl enable vectorizer

# Disable auto-start on boot
sudo systemctl disable vectorizer
```

### Reload Configuration

```bash
# After modifying service file
sudo systemctl daemon-reload
sudo systemctl restart vectorizer
```

## Windows

### Check Status

```powershell
Get-Service Vectorizer
```

### Start Service

```powershell
Start-Service Vectorizer
```

### Stop Service

```powershell
Stop-Service Vectorizer
```

### Restart Service

```powershell
Restart-Service Vectorizer
```

### View Logs

```powershell
# View recent logs
Get-EventLog -LogName Application -Source Vectorizer -Newest 50

# Follow logs
Get-EventLog -LogName Application -Source Vectorizer -Newest 10 -Wait
```

### Configure Service

```powershell
# Set startup type
Set-Service Vectorizer -StartupType Automatic

# Set startup type to manual
Set-Service Vectorizer -StartupType Manual
```

## Service Configuration

### Linux Service File

Located at `/etc/systemd/system/vectorizer.service`:

```ini
[Unit]
Description=Vectorizer - High-Performance Vector Database
After=network.target

[Service]
Type=simple
User=vectorizer
Group=vectorizer
WorkingDirectory=/var/lib/vectorizer
ExecStart=/usr/local/bin/vectorizer --host 0.0.0.0 --port 15002
Restart=always
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

### Windows Service

Configured via `sc.exe`:

```powershell
sc.exe config Vectorizer start= auto
sc.exe failure Vectorizer reset= 86400 actions= restart/5000/restart/10000/restart/20000
```

## Troubleshooting

### Service Won't Start

1. Check logs for errors
2. Verify binary exists and is executable
3. Check port availability (15002)
4. Verify user permissions

### Service Keeps Restarting

1. Check logs for crash reasons
2. Verify configuration is valid
3. Check system resources (memory, disk)

## Related Topics

- [Installation Guide](../installation/INSTALLATION.md) - Initial setup
- [Configuration](../configuration/CONFIGURATION.md) - Service configuration
- [Troubleshooting](../troubleshooting/TROUBLESHOOTING.md) - Common issues
