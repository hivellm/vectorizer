# Infrastructure & DevOps

**Version**: 1.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-10-01

---

## File Watcher System

### Overview

Real-time file system monitoring for automatic collection updates.

### Features

**Change Detection**:
- File creation
- File modification
- File deletion
- Directory operations

**Processing**:
- Batch event processing
- Debouncing (300ms)
- Incremental indexing
- Automatic reindexing

### Configuration

```yaml
file_watcher:
  enabled: true
  watch_paths:
    - "/path/to/project"
  debounce_ms: 300
  auto_discovery: true
  hot_reload: true
```

---

## Backup & Restore System

### Backup Types

**Full Backup**:
- Complete collection data
- HNSW indexes
- Metadata
- Configuration

**Incremental Backup**:
- Only changed vectors
- Delta updates
- Minimal storage

### Backup Commands

```bash
# Create backup
vectorizer backup create --collection my-collection --output backup.tar.gz

# List backups
vectorizer backup list

# Restore backup
vectorizer backup restore backup.tar.gz --collection my-collection
```

### Automated Backups

```yaml
backup:
  enabled: true
  schedule: "0 2 * * *"  # Daily at 2 AM
  retention_days: 30
  compression: true
  incremental: true
```

---

## Docker Deployment

### Production Deployment

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/vectorizer /usr/local/bin/
EXPOSE 15002
CMD ["vectorizer"]
```

**Run**:
```bash
docker build -t vectorizer .
docker run -p 15002:15002 -v ./data:/data vectorizer
```

---

## CI/CD

### GitHub Actions

**Test Pipeline**:
- Lint with clippy
- Format check with rustfmt
- Unit tests
- Integration tests
- Coverage report

**Build Pipeline**:
- Multi-platform builds (Linux, macOS, Windows)
- Docker image creation
- Release artifacts

**Deployment Pipeline**:
- Automated deployment to staging
- Smoke tests
- Production rollout

### MCP Coverage

**MCP Tests in CI**:
- Tool registration validation
- Input schema validation
- Output format compliance
- Error handling tests

---

**Maintained by**: HiveLLM Team

