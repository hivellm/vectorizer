---
title: Data Directory Configuration
module: configuration
id: data-directory-configuration
order: 3
description: Configuring data storage directories and persistence
tags: [configuration, storage, data-directory, persistence]
---

# Data Directory Configuration

Complete guide to configuring data storage directories and persistence.

## Default Data Directories

### Platform-Specific Defaults

| Platform | Default Path | Description |
|----------|-------------|-------------|
| Linux | `/var/lib/vectorizer` | System data directory |
| Windows | `%ProgramData%\Vectorizer` | Program data directory |
| macOS | `/var/lib/vectorizer` | System data directory |

### Directory Structure

```
/var/lib/vectorizer/
├── collections/          # Collection data files (.vecdb)
├── indexes/              # HNSW index files (.vecidx)
├── snapshots/            # Backup snapshots
│   ├── 20241116_103045/
│   └── 20241116_120000/
└── wal/                  # Write-ahead log (if enabled)
```

## Custom Data Directory

### Setting Custom Directory

**Command Line:**
```bash
vectorizer --data-dir /custom/path/to/data
```

**Environment Variable:**
```bash
export VECTORIZER_DATA_DIR=/custom/path/to/data
```

**YAML Configuration:**
```yaml
storage:
  data_dir: "/custom/path/to/data"
```

**Service Configuration (Linux):**
```ini
[Service]
Environment="VECTORIZER_DATA_DIR=/custom/path/to/data"
```

### Directory Permissions

**Linux:**
```bash
# Create directory
sudo mkdir -p /var/lib/vectorizer

# Set ownership
sudo chown -R vectorizer:vectorizer /var/lib/vectorizer

# Set permissions
sudo chmod 755 /var/lib/vectorizer
```

**Windows:**
```powershell
# Create directory
New-Item -ItemType Directory -Path "C:\ProgramData\Vectorizer" -Force

# Set permissions (via GUI or icacls)
icacls "C:\ProgramData\Vectorizer" /grant "NT AUTHORITY\SYSTEM:(OI)(CI)F"
```

## Storage Configuration

### Snapshot Configuration

**YAML:**
```yaml
storage:
  snapshots_dir: "/var/lib/vectorizer/snapshots"
  max_snapshots: 10
  retention_days: 7
```

**Parameters:**
- `snapshots_dir`: Directory for storing snapshots
- `max_snapshots`: Maximum number of snapshots to keep
- `retention_days`: Days to retain snapshots before cleanup

### Write-Ahead Log (WAL)

**Configuration:**
```yaml
storage:
  wal:
    enabled: true
    sync_interval: 1000  # milliseconds
    max_size: 100MB
```

**Benefits:**
- Durability: Ensures data is written to disk
- Recovery: Can recover from crashes
- Performance: Batched writes

**Trade-offs:**
- Slightly slower writes
- Additional disk space usage

## Disk Space Management

### Monitoring Disk Usage

**Check directory size:**
```bash
# Linux
du -sh /var/lib/vectorizer

# Per collection
du -sh /var/lib/vectorizer/collections/*

# Windows
Get-ChildItem "C:\ProgramData\Vectorizer" -Recurse | 
    Measure-Object -Property Length -Sum
```

### Disk Space Requirements

**Estimated space per million vectors:**

| Dimension | Without Quantization | With 8-bit Quantization |
|-----------|---------------------|------------------------|
| 384 | ~1.5 GB | ~375 MB |
| 512 | ~2 GB | ~500 MB |
| 768 | ~3 GB | ~750 MB |
| 1536 | ~6 GB | ~1.5 GB |

**Additional space:**
- Indexes: ~20-30% of vector data
- Snapshots: Full copy of data
- WAL: ~10-20% of data (if enabled)

### Disk Space Optimization

**Enable quantization:**
```yaml
quantization:
  enabled: true
  type: "scalar"
  bits: 8  # 4x reduction
```

**Limit snapshots:**
```yaml
storage:
  max_snapshots: 5  # Keep fewer snapshots
  retention_days: 3  # Shorter retention
```

**Compress payloads:**
```yaml
compression:
  enabled: true
  threshold_bytes: 1024
  algorithm: "lz4"
```

## Backup and Restore

### Backup Location

**Default backup directory:**
```
/var/lib/vectorizer/snapshots/
```

**Custom backup directory:**
```yaml
storage:
  snapshots_dir: "/backups/vectorizer"
```

### Manual Backup

**Linux:**
```bash
# Stop service
sudo systemctl stop vectorizer

# Backup entire directory
sudo tar -czf backup-$(date +%Y%m%d).tar.gz /var/lib/vectorizer

# Start service
sudo systemctl start vectorizer
```

**Windows:**
```powershell
# Stop service
Stop-Service Vectorizer

# Backup directory
Compress-Archive -Path "$env:ProgramData\Vectorizer" `
    -DestinationPath "backup-$(Get-Date -Format 'yyyyMMdd').zip"

# Start service
Start-Service Vectorizer
```

## Multiple Data Directories

### Running Multiple Instances

**Instance 1:**
```bash
vectorizer --data-dir /var/lib/vectorizer1 --port 15002
```

**Instance 2:**
```bash
vectorizer --data-dir /var/lib/vectorizer2 --port 15012
```

**Instance 3:**
```bash
vectorizer --data-dir /var/lib/vectorizer3 --port 15022
```

### Use Cases

- **Testing**: Separate test data from production
- **Multi-tenant**: Different data directories per tenant
- **Backup**: Backup instance with separate directory
- **Migration**: Temporary directory during migration

## Network Storage

### NFS Mount (Linux)

**Mount NFS share:**
```bash
# Mount NFS
sudo mount -t nfs nfs-server:/vectorizer-data /var/lib/vectorizer

# Auto-mount on boot (/etc/fstab)
nfs-server:/vectorizer-data /var/lib/vectorizer nfs defaults 0 0
```

**Considerations:**
- Network latency affects performance
- Ensure NFS is reliable
- Use NFSv4 for better performance

### SMB/CIFS Mount (Windows)

**Mount SMB share:**
```powershell
# Map network drive
New-PSDrive -Name "V" -PSProvider FileSystem `
    -Root "\\server\vectorizer-data" `
    -Persist

# Use mapped drive
$env:VECTORIZER_DATA_DIR = "V:\"
```

## Docker Volumes

### Named Volume

```yaml
services:
  vectorizer:
    volumes:
      - vectorizer-data:/var/lib/vectorizer

volumes:
  vectorizer-data:
```

### Host Directory

```yaml
services:
  vectorizer:
    volumes:
      - ./vectorizer-data:/var/lib/vectorizer
```

### External Volume

```yaml
services:
  vectorizer:
    volumes:
      - external-vectorizer-data:/var/lib/vectorizer

volumes:
  external-vectorizer-data:
    external: true
```

## Data Migration

### Moving Data Directory

**Linux:**
```bash
# Stop service
sudo systemctl stop vectorizer

# Copy data
sudo cp -r /var/lib/vectorizer /new/path/vectorizer

# Update permissions
sudo chown -R vectorizer:vectorizer /new/path/vectorizer

# Update service configuration
sudo systemctl edit vectorizer
# Add: Environment="VECTORIZER_DATA_DIR=/new/path/vectorizer"

# Reload and start
sudo systemctl daemon-reload
sudo systemctl start vectorizer
```

**Windows:**
```powershell
# Stop service
Stop-Service Vectorizer

# Copy data
Copy-Item -Path "$env:ProgramData\Vectorizer" `
    -Destination "D:\Vectorizer" -Recurse

# Update environment variable
[System.Environment]::SetEnvironmentVariable(
    "VECTORIZER_DATA_DIR", 
    "D:\Vectorizer", 
    "Machine"
)

# Start service
Start-Service Vectorizer
```

## Troubleshooting

### Permission Denied

**Error:** `Permission denied: /var/lib/vectorizer`

**Solution:**
```bash
sudo chown -R vectorizer:vectorizer /var/lib/vectorizer
sudo chmod 755 /var/lib/vectorizer
```

### Disk Full

**Error:** `No space left on device`

**Solutions:**
1. Clean old snapshots
2. Enable quantization
3. Delete unused collections
4. Increase disk space

### Cannot Write to Directory

**Error:** `Cannot write to data directory`

**Solutions:**
1. Check directory permissions
2. Verify disk space
3. Check filesystem mount status
4. Verify SELinux/AppArmor policies (Linux)

## Best Practices

1. **Use dedicated disk**: Separate data disk from OS
2. **Regular backups**: Automated backup schedule
3. **Monitor disk space**: Set up alerts
4. **Use SSD**: Better performance for random I/O
5. **Enable snapshots**: For point-in-time recovery
6. **Separate test/prod**: Different directories
7. **Document paths**: Keep configuration documented

## Related Topics

- [Server Configuration](./SERVER.md) - Server settings
- [Backup and Restore](../backup-restore/BACKUP.md) - Backup procedures
- [Performance Guide](../performance/PERFORMANCE.md) - Storage optimization

