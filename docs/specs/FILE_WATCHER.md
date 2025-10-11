# File Watcher System

**Version**: 1.0  
**Status**: ✅ Production Ready  
**Last Updated**: 2025-10-01

---

## Overview

Real-time file system monitoring with automatic collection updates, supporting creation, modification, and deletion detection.

---

## Features

**Change Detection**:
- File creation (new files added)
- File modification (content changes)
- File deletion (files removed)
- Directory operations (batch changes)

**Processing**:
- Event debouncing (300ms)
- Batch processing
- Incremental indexing
- Automatic reindexing

---

## Configuration

```yaml
file_watcher:
  enabled: true
  watch_paths:
    - "/path/to/project"
  debounce_ms: 300
  auto_discovery: true
  hot_reload: true
  batch_size: 100
```

---

## API

### Start Watching

```bash
POST /api/v1/watch/start
{
  "path": "/path/to/project",
  "patterns": ["**/*.md", "**/*.rs"]
}
```

### Stop Watching

```bash
POST /api/v1/watch/stop
{
  "path": "/path/to/project"
}
```

### Watch Status

```bash
GET /api/v1/watch/status
```

---

## Architecture

**Event Flow**:
1. File system event detected
2. Debouncer collects events (300ms window)
3. Batch processor validates changes
4. Incremental indexer updates collections
5. HNSW index updated
6. Cache invalidated

**Performance**:
- Event detection: <1ms
- Debouncing: 300ms window
- Indexing: <100ms per file
- Total latency: <500ms

---

## User Guide

### Basic Usage

```bash
# Enable file watcher in config
vectorizer config set file_watcher.enabled true

# Add watch path
vectorizer watch add /path/to/project

# View watched paths
vectorizer watch list

# Remove watch path
vectorizer watch remove /path/to/project
```

### Advanced Usage

**Selective Watching**:
```yaml
file_watcher:
  watch_paths:
    - path: "/docs"
      patterns: ["**/*.md"]
    - path: "/src"  
      patterns: ["**/*.rs", "**/*.toml"]
```

**Exclusion Patterns**:
```yaml
file_watcher:
  exclude_patterns:
    - "**/node_modules/**"
    - "**/target/**"
    - "**/.git/**"
```

---

## Troubleshooting

**High CPU Usage**:
- Reduce batch_size
- Increase debounce_ms
- Add more exclusion patterns

**Missed Events**:
- Check file system notifications
- Verify permissions
- Review logs

---

**Status**: ✅ Production Ready  
**Maintained by**: HiveLLM Team

