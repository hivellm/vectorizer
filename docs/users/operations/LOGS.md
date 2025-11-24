---
title: Log Management
module: service-management
id: log-management
order: 2
description: Managing and viewing Vectorizer logs
tags: [logs, logging, debugging, troubleshooting]
---

# Log Management

Complete guide to managing and viewing Vectorizer logs.

## Log Locations

### Linux (systemd)

Logs are stored in systemd journal:

```bash
# View logs
sudo journalctl -u vectorizer

# Follow logs in real-time
sudo journalctl -u vectorizer -f

# View recent logs
sudo journalctl -u vectorizer -n 100

# View logs since boot
sudo journalctl -u vectorizer --since boot

# View logs from specific time
sudo journalctl -u vectorizer --since "2024-11-16 10:00:00"
```

### Windows (Event Log)

Logs are stored in Windows Event Log:

```powershell
# View all logs
Get-EventLog -LogName Application -Source Vectorizer

# View recent logs
Get-EventLog -LogName Application -Source Vectorizer -Newest 50

# Follow logs (requires additional setup)
Get-EventLog -LogName Application -Source Vectorizer -Newest 10 -Wait

# Filter by level
Get-EventLog -LogName Application -Source Vectorizer -EntryType Error
```

## Log Levels

### Setting Log Level

**Environment variable:**
```bash
export VECTORIZER_LOG_LEVEL=debug
```

**Command line:**
```bash
vectorizer --log-level debug
```

**Service configuration (Linux):**
```ini
[Service]
Environment="VECTORIZER_LOG_LEVEL=debug"
```

## Filtering Logs

### Linux

**By log level:**
```bash
# Errors only
sudo journalctl -u vectorizer -p err

# Warnings and errors
sudo journalctl -u vectorizer -p warning

# Info and above
sudo journalctl -u vectorizer -p info
```

**By keyword:**
```bash
# Search for errors
sudo journalctl -u vectorizer | grep -i error

# Search for specific collection
sudo journalctl -u vectorizer | grep "my_collection"

# Search for search operations
sudo journalctl -u vectorizer | grep "search"
```

**By time range:**
```bash
# Last hour
sudo journalctl -u vectorizer --since "1 hour ago"

# Today
sudo journalctl -u vectorizer --since today

# Specific date range
sudo journalctl -u vectorizer --since "2024-11-16 00:00:00" --until "2024-11-16 23:59:59"
```

### Windows

**By entry type:**
```powershell
# Errors only
Get-EventLog -LogName Application -Source Vectorizer -EntryType Error

# Warnings and errors
Get-EventLog -LogName Application -Source Vectorizer | Where-Object {$_.EntryType -in @("Error", "Warning")}
```

**By keyword:**
```powershell
# Search for errors
Get-EventLog -LogName Application -Source Vectorizer | Where-Object {$_.Message -like "*error*"}

# Search for specific collection
Get-EventLog -LogName Application -Source Vectorizer | Where-Object {$_.Message -like "*my_collection*"}
```

## Log Rotation

### Linux (systemd)

Systemd automatically manages log rotation. Configure limits:

```ini
# /etc/systemd/journald.conf
[Journal]
SystemMaxUse=500M
SystemKeepFree=1G
SystemMaxFileSize=100M
MaxRetentionSec=30day
```

### Windows

Configure Event Log size limits:

```powershell
# Set maximum log size (500 MB)
wevtutil sl Application /ms:524288000

# Set retention policy
wevtutil sl Application /ab:true
```

## Exporting Logs

### Linux

**Export to file:**
```bash
# Export all logs
sudo journalctl -u vectorizer > vectorizer.log

# Export errors only
sudo journalctl -u vectorizer -p err > vectorizer-errors.log

# Export with timestamps
sudo journalctl -u vectorizer --no-pager > vectorizer-full.log
```

**Export in JSON format:**
```bash
sudo journalctl -u vectorizer -o json > vectorizer.json
```

### Windows

**Export to file:**
```powershell
# Export all logs
Get-EventLog -LogName Application -Source Vectorizer | Export-Csv vectorizer.csv

# Export errors only
Get-EventLog -LogName Application -Source Vectorizer -EntryType Error | Export-Csv vectorizer-errors.csv
```

## Log Analysis

### Common Log Patterns

**Search operations:**
```bash
# Count search operations
sudo journalctl -u vectorizer | grep -c "search"

# Average search latency
sudo journalctl -u vectorizer | grep "search latency" | awk '{sum+=$NF; count++} END {print sum/count}'
```

**Error analysis:**
```bash
# Count errors by type
sudo journalctl -u vectorizer -p err | grep -o "error:.*" | sort | uniq -c
```

**Performance analysis:**
```bash
# Find slow operations (>100ms)
sudo journalctl -u vectorizer | grep "duration.*[0-9]\{3,\}ms"
```

## Log Aggregation

### ELK Stack

**Logstash configuration:**
```ruby
input {
  file {
    path => "/var/log/vectorizer/*.log"
    codec => json
    type => "vectorizer"
  }
}

filter {
  if [type] == "vectorizer" {
    # Parse structured logs
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "vectorizer-logs-%{+YYYY.MM.dd}"
  }
}
```

### Loki (Grafana)

**Promtail configuration:**
```yaml
scrape_configs:
  - job_name: vectorizer
    static_configs:
      - targets:
          - localhost
        labels:
          job: vectorizer
          __path__: /var/log/vectorizer/*.log
```

## Best Practices

1. **Set appropriate log level**: Use `info` for production, `debug` for troubleshooting
2. **Monitor log size**: Configure rotation to prevent disk space issues
3. **Centralize logs**: Use log aggregation for distributed systems
4. **Regular analysis**: Review logs regularly for issues
5. **Retention policy**: Keep logs for appropriate duration

## Related Topics

- [Monitoring Guide](../operations/MONITORING.md) - Metrics and monitoring
- [Troubleshooting Guide](./TROUBLESHOOTING.md) - Using logs for debugging

