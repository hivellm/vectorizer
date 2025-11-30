---
title: Server Configuration
module: configuration
id: server-configuration
order: 1
description: Complete server configuration guide for Vectorizer
tags: [configuration, server, network, ports, host]
---

# Server Configuration

Complete guide to configuring Vectorizer server settings.

## Network Configuration

### Host Binding

Configure which network interfaces Vectorizer listens on.

**Default:** `0.0.0.0` (all interfaces)

**Options:**

- `0.0.0.0`: Listen on all interfaces (default, recommended for servers)
- `127.0.0.1`: Listen only on localhost (for local development)
- `192.168.1.100`: Listen on specific IP address

**Environment Variable:**

```bash
export VECTORIZER_HOST=0.0.0.0
```

**Command Line:**

```bash
vectorizer --host 0.0.0.0
```

**YAML Configuration:**

```yaml
server:
  host: "0.0.0.0"
```

**Service Configuration (Linux systemd):**

```ini
[Service]
Environment="VECTORIZER_HOST=0.0.0.0"
```

### Port Configuration

Configure the port for REST API and MCP server.

**Default:** `15002`

**REST API Port:**

```bash
# Environment variable
export VECTORIZER_PORT=15002

# Command line
vectorizer --port 15002

# YAML
server:
  port: 15002
```

**MCP Port (separate from REST):**

```bash
# Environment variable
export VECTORIZER_MCP_PORT=15003

# YAML
server:
  mcp_port: 15003
```

**Note:** MCP can share the same port as REST API (default) or use a separate port.

### Port Selection Guidelines

**Development:**

- Use default port `15002` for simplicity
- Use `127.0.0.1` for localhost-only access

**Production:**

- Use `0.0.0.0` to accept connections from any interface
- Consider firewall rules to restrict access
- Use reverse proxy (nginx, Caddy) for SSL/TLS termination

**Multiple Instances:**

- Run multiple instances on different ports: `15002`, `15012`, `15022`
- Use different data directories for each instance

## Command Line Arguments

### Complete Argument Reference

```bash
vectorizer [OPTIONS]

OPTIONS:
    --host <HOST>              Host to bind to [default: 0.0.0.0]
    --port <PORT>              Port to listen on [default: 15002]
    --data-dir <DATA_DIR>      Data directory path
    --log-level <LOG_LEVEL>    Logging level [default: info]
                                [possible values: trace, debug, info, warn, error]
    --workers <WORKERS>        Number of worker threads
    --config <CONFIG>          Path to YAML configuration file
    --help                     Print help information
    --version                  Print version information
```

### Common Command Line Examples

**Basic server:**

```bash
vectorizer --host 0.0.0.0 --port 15002
```

**Custom data directory:**

```bash
vectorizer --host 0.0.0.0 --port 15002 --data-dir /var/lib/vectorizer
```

**Development with debug logging:**

```bash
vectorizer --host 127.0.0.1 --port 15002 --log-level debug
```

**Production with custom config:**

```bash
vectorizer --config /etc/vectorizer/config.yml
```

**High-performance server:**

```bash
vectorizer --host 0.0.0.0 --port 15002 --workers 8
```

## Environment Variables

### Complete Environment Variable Reference

| Variable               | Description             | Default           | Example               |
| ---------------------- | ----------------------- | ----------------- | --------------------- |
| `VECTORIZER_HOST`      | Host to bind to         | `0.0.0.0`         | `0.0.0.0`             |
| `VECTORIZER_PORT`      | REST API port           | `15002`           | `15002`               |
| `VECTORIZER_MCP_PORT`  | MCP server port         | `15003`           | `15003`               |
| `VECTORIZER_LOG_LEVEL` | Logging level           | `info`            | `debug`               |
| `VECTORIZER_DATA_DIR`  | Data directory          | Platform-specific | `/var/lib/vectorizer` |
| `VECTORIZER_WORKERS`   | Worker threads          | Auto-detect       | `8`                   |
| `RUST_LOG`             | Rust logging (advanced) | -                 | `debug`               |
| `RUST_BACKTRACE`       | Backtrace on panic      | `0`               | `1`                   |

### Setting Environment Variables

**Linux/macOS (bash/zsh):**

```bash
# Temporary (current session)
export VECTORIZER_HOST=0.0.0.0
export VECTORIZER_PORT=15002
export VECTORIZER_LOG_LEVEL=info

# Permanent (add to ~/.bashrc or ~/.zshrc)
echo 'export VECTORIZER_HOST=0.0.0.0' >> ~/.bashrc
echo 'export VECTORIZER_PORT=15002' >> ~/.bashrc
```

**Linux systemd service:**

```ini
[Service]
Environment="VECTORIZER_HOST=0.0.0.0"
Environment="VECTORIZER_PORT=15002"
Environment="VECTORIZER_LOG_LEVEL=info"
```

**Windows PowerShell:**

```powershell
# Temporary (current session)
$env:VECTORIZER_HOST = "0.0.0.0"
$env:VECTORIZER_PORT = "15002"

# Permanent (System Properties > Environment Variables)
[System.Environment]::SetEnvironmentVariable("VECTORIZER_HOST", "0.0.0.0", "Machine")
```

**Docker:**

```yaml
environment:
  - VECTORIZER_HOST=0.0.0.0
  - VECTORIZER_PORT=15002
  - VECTORIZER_LOG_LEVEL=info
```

## YAML Configuration File

### Configuration File Location

Vectorizer can load configuration from a YAML file:

**Default locations (checked in order):**

1. `./workspace.yml` (current directory)
2. `~/.vectorizer/config.yml` (user home)
3. `/etc/vectorizer/config.yml` (system-wide)

**Custom location:**

```bash
vectorizer --config /path/to/config.yml
```

### Complete YAML Configuration Example

```yaml
# Server configuration
server:
  host: "0.0.0.0"
  port: 15002
  mcp_port: 15003

# Logging configuration
logging:
  level: "info"
  log_requests: true
  log_responses: false
  log_errors: true

# GPU configuration (macOS Metal)
gpu:
  enabled: true # Auto-enabled on macOS, false on other platforms
  device: "auto" # "auto" or specific device ID

# Storage configuration
storage:
  data_dir: "/var/lib/vectorizer"
  snapshots_dir: "/var/lib/vectorizer/snapshots"
  max_snapshots: 10
  retention_days: 7

# File watcher configuration
file_watcher:
  enabled: false
  watch_directories: []
  exclude_patterns: ["**/node_modules/**", "**/.git/**"]

# Summarization configuration
summarization:
  enabled: true
  model: "default"
  max_length: 512

# Transmutation configuration
transmutation:
  enabled: false
  preserve_images: false
```

### Loading Configuration

**Priority order (highest to lowest):**

1. Command line arguments
2. Environment variables
3. YAML configuration file
4. Default values

**Example:**

```bash
# Command line overrides everything
vectorizer --port 15003 --config config.yml
# Port will be 15003 (from command line), not from config.yml

# Environment variable overrides YAML
export VECTORIZER_PORT=15004
vectorizer --config config.yml
# Port will be 15004 (from environment), not from config.yml
```

## Service Configuration

### Linux systemd Service

**Service file:** `/etc/systemd/system/vectorizer.service`

```ini
[Unit]
Description=Vectorizer - High-Performance Vector Database
Documentation=https://github.com/hivellm/vectorizer
After=network.target

[Service]
Type=simple
User=vectorizer
Group=vectorizer
WorkingDirectory=/var/lib/vectorizer

# Command with arguments
ExecStart=/usr/local/bin/vectorizer \
    --host 0.0.0.0 \
    --port 15002 \
    --data-dir /var/lib/vectorizer \
    --log-level info

# Or use environment variables
Environment="VECTORIZER_HOST=0.0.0.0"
Environment="VECTORIZER_PORT=15002"
Environment="VECTORIZER_LOG_LEVEL=info"

# Or use config file
ExecStart=/usr/local/bin/vectorizer --config /etc/vectorizer/config.yml

# Restart policy
Restart=always
RestartSec=5s

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

**Reload and restart:**

```bash
sudo systemctl daemon-reload
sudo systemctl restart vectorizer
```

### Windows Service

**PowerShell configuration:**

```powershell
# Install service
New-Service -Name "Vectorizer" `
    -BinaryPathName "C:\Program Files\Vectorizer\vectorizer.exe --host 0.0.0.0 --port 15002" `
    -DisplayName "Vectorizer Vector Database" `
    -StartupType Automatic

# Configure environment variables
[System.Environment]::SetEnvironmentVariable("VECTORIZER_HOST", "0.0.0.0", "Machine")
[System.Environment]::SetEnvironmentVariable("VECTORIZER_PORT", "15002", "Machine")

# Restart service
Restart-Service Vectorizer
```

## Network Security

### Firewall Configuration

**Linux (ufw):**

```bash
# Allow port 15002
sudo ufw allow 15002/tcp

# Allow from specific IP
sudo ufw allow from 192.168.1.0/24 to any port 15002

# Check status
sudo ufw status
```

**Linux (firewalld):**

```bash
# Add port
sudo firewall-cmd --permanent --add-port=15002/tcp
sudo firewall-cmd --reload

# Allow from zone
sudo firewall-cmd --permanent --zone=public --add-port=15002/tcp
```

**Windows Firewall:**

```powershell
# Allow port
New-NetFirewallRule -DisplayName "Vectorizer" `
    -Direction Inbound `
    -LocalPort 15002 `
    -Protocol TCP `
    -Action Allow
```

### Reverse Proxy (nginx)

**Basic nginx configuration:**

```nginx
server {
    listen 80;
    server_name vectorizer.example.com;

    location / {
        proxy_pass http://127.0.0.1:15002;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

**With SSL/TLS (Let's Encrypt):**

```nginx
server {
    listen 443 ssl http2;
    server_name vectorizer.example.com;

    ssl_certificate /etc/letsencrypt/live/vectorizer.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/vectorizer.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:15002;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Configuration Scenarios

### Scenario 1: Local Development

```bash
# Simple local development
vectorizer --host 127.0.0.1 --port 15002 --log-level debug
```

**YAML:**

```yaml
server:
  host: "127.0.0.1"
  port: 15002
logging:
  level: "debug"
```

### Scenario 2: Production Server

```bash
# Production with custom data directory
vectorizer \
    --host 0.0.0.0 \
    --port 15002 \
    --data-dir /var/lib/vectorizer \
    --log-level info \
    --workers 8
```

**YAML:**

```yaml
server:
  host: "0.0.0.0"
  port: 15002
logging:
  level: "info"
storage:
  data_dir: "/var/lib/vectorizer"
```

### Scenario 3: Docker Container

```yaml
# docker-compose.yml
services:
  vectorizer:
    image: ghcr.io/hivellm/vectorizer:latest
    ports:
      - "15002:15002"
    environment:
      - VECTORIZER_HOST=0.0.0.0
      - VECTORIZER_PORT=15002
      - VECTORIZER_LOG_LEVEL=info
    volumes:
      - ./data:/var/lib/vectorizer
```

### Scenario 4: Multiple Instances

**Instance 1:**

```bash
vectorizer --port 15002 --data-dir /var/lib/vectorizer1
```

**Instance 2:**

```bash
vectorizer --port 15012 --data-dir /var/lib/vectorizer2
```

**Instance 3:**

```bash
vectorizer --port 15022 --data-dir /var/lib/vectorizer3
```

## Troubleshooting

### Port Already in Use

**Error:** `Address already in use`

**Solutions:**

```bash
# Check what's using the port
# Linux
sudo lsof -i :15002
sudo netstat -tulpn | grep 15002

# Windows
netstat -ano | findstr 15002

# Kill process (Linux)
sudo kill -9 $(lsof -t -i:15002)

# Use different port
vectorizer --port 15012
```

### Cannot Bind to Host

**Error:** `Cannot assign requested address`

**Solutions:**

1. Verify host IP exists: `ip addr show` (Linux) or `ipconfig` (Windows)
2. Use `0.0.0.0` to bind to all interfaces
3. Check firewall rules

### Connection Refused

**Symptoms:** Cannot connect to server

**Solutions:**

1. Verify server is running: `curl http://localhost:15002/health`
2. Check firewall rules
3. Verify host binding (not `127.0.0.1` if connecting remotely)
4. Check network connectivity

## Related Topics

- [Logging Configuration](./LOGGING.md) - Detailed logging setup
- [Data Directory Configuration](./DATA_DIRECTORY.md) - Storage configuration
- [Performance Tuning](./PERFORMANCE.md) - Performance optimization
- [Service Management](../operations/SERVICE_MANAGEMENT.md) - Service configuration
