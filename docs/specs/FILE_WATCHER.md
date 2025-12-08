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
  debounce_delay_ms: 1000
  auto_discovery: true
  hot_reload: true
  batch_size: 100
  collection_name: "workspace-files"
  collection_mapping:
    "*/docs/**/*.md": "documentation"
    "*/src/**/*.rs": "rust-code"
    "*/src/**/*.py": "python-code"
    "*/tests/**/*": "test-files"
```

### Collection Mapping

The `collection_mapping` option allows you to configure custom path-to-collection mappings using glob patterns. This gives you fine-grained control over which collection each file is indexed into based on its path.

**Priority Order**:
1. Collection mapping patterns (from `collection_mapping` config)
2. Known project patterns (from workspace.yml)
3. Default collection (from `default_collection` or `collection_name`)

**Example Configuration**:

```yaml
file_watcher:
  enabled: true
  collection_mapping:
    # Documentation files go to docs collection
    "*/docs/**/*.md": "documentation"
    "*/docs/**/*.rst": "documentation"
    
    # Source code by language
    "*/src/**/*.rs": "rust-code"
    "*/src/**/*.py": "python-code"
    "*/src/**/*.js": "javascript-code"
    "*/src/**/*.ts": "typescript-code"
    
    # Tests go to separate collection
    "*/tests/**/*": "test-files"
    "*/test/**/*": "test-files"
    
    # Configuration files
    "**/*.yml": "configuration"
    "**/*.yaml": "configuration"
    "**/*.toml": "configuration"
    
    # Default fallback (use default_collection if no pattern matches)
```

**Pattern Matching**:
- Patterns use glob syntax (same as include/exclude patterns)
- Patterns are checked in order, first match wins
- Path separators are normalized (`\` → `/`) for cross-platform compatibility
- Patterns support wildcards: `*`, `**`, `?`, `[...]`

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

**Collection Mapping**:
```yaml
file_watcher:
  enabled: true
  collection_mapping:
    "*/docs/**/*.md": "documentation"
    "*/src/**/*.rs": "rust-code"
    "*/tests/**/*": "test-files"
```

---

## Cluster Mode Behavior

**Important**: The file watcher is **automatically disabled** when running in cluster mode.

### Why File Watcher is Disabled in Cluster Mode

When `cluster.enabled: true`, the file watcher is incompatible for several reasons:

1. **Distributed Nature**: Each cluster node would independently watch the same files, causing duplicate processing and race conditions
2. **Memory Predictability**: File watcher can cause unpredictable memory spikes when large batches of files change
3. **State Consistency**: File changes should be propagated through the cluster's replication mechanism, not local file watching

### Configuration

```yaml
cluster:
  enabled: true
  memory:
    # Controls file watcher behavior in cluster mode (default: true)
    disable_file_watcher: true
```

When `cluster.memory.disable_file_watcher: true` (default):
- File watcher is automatically disabled at server startup
- A warning is logged if file watcher was enabled in the config
- The server continues starting without the file watcher

### Workaround for Testing

If you need file watching in a cluster environment for testing:

```yaml
cluster:
  enabled: true
  memory:
    disable_file_watcher: false  # Not recommended for production
```

**Warning**: This is not recommended for production clusters as it can cause inconsistent state across nodes.

### Alternative Approaches

For cluster deployments that need file-based updates:

1. **External Ingestion Service**: Use a dedicated service to watch files and push updates via the REST API
2. **Batch Processing**: Use scheduled batch jobs to scan and update files
3. **Event-Driven**: Integrate with message queues (Kafka, RabbitMQ) for file change events

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

