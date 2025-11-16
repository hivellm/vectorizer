---
title: Logging Configuration
module: configuration
id: logging-configuration
order: 2
description: Complete logging configuration guide for Vectorizer
tags: [configuration, logging, debug, monitoring]
---

# Logging Configuration

Complete guide to configuring logging in Vectorizer.

## Log Levels

### Available Log Levels

| Level   | Description         | Use Case                     | Verbosity |
| ------- | ------------------- | ---------------------------- | --------- |
| `trace` | Very detailed logs  | Development, debugging       | Highest   |
| `debug` | Debug information   | Development, troubleshooting | High      |
| `info`  | General information | Production (default)         | Medium    |
| `warn`  | Warnings only       | Production monitoring        | Low       |
| `error` | Errors only         | Critical issues              | Lowest    |

### Setting Log Level

**Environment Variable:**

```bash
export VECTORIZER_LOG_LEVEL=debug
```

**Command Line:**

```bash
vectorizer --log-level debug
```

**YAML Configuration:**

```yaml
logging:
  level: "debug"
```

**Service Configuration (Linux):**

```ini
[Service]
Environment="VECTORIZER_LOG_LEVEL=debug"
```

## Log Output Configuration

### Log Output Destinations

**Direct execution:**

- Logs go to `stdout` (standard output)
- Can be redirected: `vectorizer > vectorizer.log 2>&1`

**systemd service (Linux):**

- Logs go to systemd journal
- View with: `journalctl -u vectorizer`

**Windows Service:**

- Logs go to Windows Event Log
- View with: `Get-EventLog -LogName Application -Source Vectorizer`

### Log Format

**Structured Logging:**
Vectorizer uses structured logging with JSON-like format:

```
2024-11-16T10:30:45.123Z INFO [vectorizer::server] Server started on 0.0.0.0:15002
2024-11-16T10:30:45.124Z DEBUG [vectorizer::db] Collection 'my_collection' created
2024-11-16T10:30:45.125Z INFO [vectorizer::api] POST /collections/my_collection/insert 200 OK
```

**Fields:**

- Timestamp (ISO 8601)
- Log level
- Module path
- Message

## Detailed Logging Options

### Request Logging

Log all HTTP requests:

**YAML:**

```yaml
logging:
  log_requests: true
```

**What gets logged:**

- HTTP method and path
- Request headers (optional)
- Request body size
- Response status code
- Response time

**Example log:**

```
INFO [vectorizer::api] POST /collections/my_collection/search 200 OK (12ms)
```

### Response Logging

Log HTTP responses (can be verbose):

**YAML:**

```yaml
logging:
  log_responses: true
```

**Warning:** This can generate large log files. Use only for debugging.

### Error Logging

Log all errors with stack traces:

**YAML:**

```yaml
logging:
  log_errors: true
```

**Default:** `true` (always enabled)

## Advanced Logging

### Rust Logging (RUST_LOG)

For advanced control, use Rust's `RUST_LOG` environment variable:

```bash
# Set module-specific log levels
export RUST_LOG=vectorizer::db=debug,vectorizer::api=info

# Enable all debug logs
export RUST_LOG=debug

# Enable trace for specific module
export RUST_LOG=vectorizer::search=trace
```

**Module paths:**

- `vectorizer::server` - Server startup and configuration
- `vectorizer::api` - REST API handlers
- `vectorizer::db` - Database operations
- `vectorizer::search` - Search operations
- `vectorizer::mcp` - MCP server
- `vectorizer::embedding` - Embedding generation

### Backtrace on Panic

Enable full backtrace on panic:

```bash
export RUST_BACKTRACE=1
# or
export RUST_BACKTRACE=full
```

**Use cases:**

- Debugging crashes
- Development only (not recommended for production)

## Log Filtering

### Filter by Level (systemd)

**View errors only:**

```bash
sudo journalctl -u vectorizer -p err
```

**View warnings and errors:**

```bash
sudo journalctl -u vectorizer -p warning
```

**View info and above:**

```bash
sudo journalctl -u vectorizer -p info
```

### Filter by Keyword

**Search for errors:**

```bash
sudo journalctl -u vectorizer | grep -i error
```

**Search for specific collection:**

```bash
sudo journalctl -u vectorizer | grep "my_collection"
```

**Search for search operations:**

```bash
sudo journalctl -u vectorizer | grep "search"
```

### Filter by Time

**Last hour:**

```bash
sudo journalctl -u vectorizer --since "1 hour ago"
```

**Today:**

```bash
sudo journalctl -u vectorizer --since today
```

**Specific date range:**

```bash
sudo journalctl -u vectorizer --since "2024-11-16 00:00:00" --until "2024-11-16 23:59:59"
```

## Log Rotation

### systemd Journal Rotation

**Configure journal limits:**

Edit `/etc/systemd/journald.conf`:

```ini
[Journal]
# Maximum disk space for journal
SystemMaxUse=500M

# Keep free space
SystemKeepFree=1G

# Maximum size per file
SystemMaxFileSize=100M

# Retention period
MaxRetentionSec=30day
```

**Reload configuration:**

```bash
sudo systemctl restart systemd-journald
```

### Windows Event Log Rotation

**Configure log size:**

```powershell
# Set maximum log size (500 MB)
wevtutil sl Application /ms:524288000

# Set retention policy (overwrite as needed)
wevtutil sl Application /ab:true
```

## Log Aggregation

### ELK Stack Integration

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
    grok {
      match => { "message" => "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} \[%{DATA:module}\] %{GREEDYDATA:message}" }
    }
  }
}

output {
  elasticsearch {
    hosts => ["localhost:9200"]
    index => "vectorizer-logs-%{+YYYY.MM.dd}"
  }
}
```

### Loki Integration (Grafana)

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

## Log Analysis

### Common Log Patterns

**Search operations:**

```bash
# Count search operations
sudo journalctl -u vectorizer | grep -c "search"

# Average search latency
sudo journalctl -u vectorizer | grep "search latency" | \
    awk '{sum+=$NF; count++} END {print sum/count "ms"}'
```

**Error analysis:**

```bash
# Count errors by type
sudo journalctl -u vectorizer -p err | \
    grep -o "error:.*" | sort | uniq -c
```

**Performance analysis:**

```bash
# Find slow operations (>100ms)
sudo journalctl -u vectorizer | grep "duration.*[0-9]\{3,\}ms"
```

## Production Logging Best Practices

1. **Use `info` level** for production (default)
2. **Enable request logging** for monitoring
3. **Disable response logging** (too verbose)
4. **Configure log rotation** to prevent disk space issues
5. **Set up log aggregation** for distributed systems
6. **Monitor error logs** regularly
7. **Use structured logging** for easier parsing

## Configuration Examples

### Development

```yaml
logging:
  level: "debug"
  log_requests: true
  log_responses: true
  log_errors: true
```

**Environment:**

```bash
export VECTORIZER_LOG_LEVEL=debug
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

### Production

```yaml
logging:
  level: "info"
  log_requests: true
  log_responses: false
  log_errors: true
```

**Environment:**

```bash
export VECTORIZER_LOG_LEVEL=info
```

### Troubleshooting

```yaml
logging:
  level: "debug"
  log_requests: true
  log_responses: false
  log_errors: true
```

**Environment:**

```bash
export VECTORIZER_LOG_LEVEL=debug
export RUST_LOG=vectorizer::db=debug,vectorizer::search=debug
```

## Related Topics

- [Server Configuration](./SERVER.md) - Server settings
- [Monitoring Guide](../monitoring/MONITORING.md) - Metrics and monitoring
- [Troubleshooting Guide](../troubleshooting/TROUBLESHOOTING.md) - Using logs for debugging
- [Service Management](../service-management/LOGS.md) - Log management
