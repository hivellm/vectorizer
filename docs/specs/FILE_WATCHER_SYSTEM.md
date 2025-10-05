# File Watcher System - Phase 5 Implementation

## Overview

The File Watcher System is a critical component of Phase 5 that provides real-time monitoring of indexed files and automatic incremental reindexing. This system eliminates the need for manual reindexing and ensures that the vector database stays synchronized with file system changes.

## Architecture

```
┌─────────────────┐    File Events    ┌──────────────────┐    REST    ┌─────────────────┐
│   File System   │ ◄────────────────► │  File Watcher    │ ◄─────────► │ Vector Database │
│                 │   (inotify/fsevents)│  System          │            │  Engine         │
└─────────────────┘                    └──────────────────┘            └─────────────────┘
                                                │
                                                ▼
                                       ┌─────────────────┐
                                       │ Change Queue    │
                                       │ & Debouncing    │
                                       └─────────────────┘
```

## Core Components

### 1. File Watcher Engine

```rust
pub struct FileWatcherSystem {
    watcher: notify::RecommendedWatcher,
    change_queue: Arc<Mutex<VecDeque<FileChangeEvent>>>,
    debounce_timer: Arc<Mutex<Option<tokio::time::Instant>>>,
    rest_client: RestClient,
    config: FileWatcherConfig,
}

pub struct FileWatcherConfig {
    pub debounce_delay: Duration,
    pub max_batch_size: usize,
    pub content_hash_validation: bool,
    pub watch_patterns: Vec<glob::Pattern>,
    pub ignore_patterns: Vec<glob::Pattern>,
}
```

### 2. File Change Events

```rust
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub timestamp: DateTime<Utc>,
    pub content_hash: Option<String>,
    pub collection_id: Option<String>,
}

pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Moved { old_path: PathBuf },
    Renamed { old_path: PathBuf },
}
```

### 3. GRPC Vector Operations

```rust
// New GRPC service methods
service VectorizerService {
    // Existing methods...
    
    // New vector update operations
    rpc UpdateVector(VectorUpdateRequest) returns (VectorUpdateResponse);
    rpc BatchUpdateVectors(BatchVectorUpdateRequest) returns (BatchVectorUpdateResponse);
    rpc DeleteVector(VectorDeleteRequest) returns (VectorDeleteResponse);
    rpc BatchDeleteVectors(BatchVectorDeleteRequest) returns (BatchVectorDeleteResponse);
    
    // Incremental reindexing operations
    rpc ReindexIncremental(IncrementalReindexRequest) returns (IncrementalReindexResponse);
    rpc ReindexCollection(CollectionReindexRequest) returns (CollectionReindexResponse);
}

message VectorUpdateRequest {
    string collection_id = 1;
    string vector_id = 2;
    Vector new_vector = 3;
    bool update_metadata = 4;
}

message IncrementalReindexRequest {
    string collection_id = 1;
    repeated string file_paths = 2;
    bool force_reindex = 3;
    EmbeddingProvider embedding_provider = 4;
}
```

## Implementation Details

### 1. Cross-Platform File Monitoring

The system uses the `notify` crate for cross-platform file system monitoring:

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};

impl FileWatcherSystem {
    pub async fn start_watching(&mut self, watch_paths: Vec<PathBuf>) -> Result<()> {
        for path in watch_paths {
            self.watcher.watch(&path, RecursiveMode::Recursive)?;
        }
        
        // Start event processing loop
        self.process_events().await
    }
    
    async fn process_events(&mut self) -> Result<()> {
        while let Some(event) = self.watcher.rx.recv().await {
            self.handle_file_event(event).await?;
        }
        Ok(())
    }
}
```

### 2. Debounced Processing

To avoid excessive reindexing, the system implements debounced processing:

```rust
impl FileWatcherSystem {
    async fn handle_file_event(&mut self, event: Event) -> Result<()> {
        let file_change = FileChangeEvent::from_notify_event(event)?;
        
        // Add to queue
        self.change_queue.lock().await.push_back(file_change);
        
        // Reset debounce timer
        let mut timer = self.debounce_timer.lock().await;
        *timer = Some(tokio::time::Instant::now() + self.config.debounce_delay);
        
        Ok(())
    }
    
    async fn process_debounced_changes(&mut self) -> Result<()> {
        let changes = self.collect_batched_changes().await;
        if !changes.is_empty() {
            self.process_batch_changes(changes).await?;
        }
        Ok(())
    }
}
```

### 3. Content Hash Validation

To avoid unnecessary reindexing, the system validates content hashes:

```rust
impl FileChangeEvent {
    pub async fn validate_content_change(&mut self) -> Result<bool> {
        if !self.config.content_hash_validation {
            return Ok(true);
        }
        
        let current_hash = self.calculate_content_hash().await?;
        let previous_hash = self.get_previous_hash().await?;
        
        Ok(current_hash != previous_hash)
    }
    
    async fn calculate_content_hash(&self) -> Result<String> {
        let content = tokio::fs::read(&self.path).await?;
        Ok(sha256::digest(&content))
    }
}
```

### 4. GRPC Integration

The file watcher communicates with the vector database via GRPC:

```rust
impl FileWatcherSystem {
    async fn process_batch_changes(&mut self, changes: Vec<FileChangeEvent>) -> Result<()> {
        for change in changes {
            match change.event_type {
                FileEventType::Created | FileEventType::Modified => {
                    self.handle_file_update(change).await?;
                }
                FileEventType::Deleted => {
                    self.handle_file_deletion(change).await?;
                }
                FileEventType::Moved { old_path } => {
                    self.handle_file_move(change, old_path).await?;
                }
            }
        }
        Ok(())
    }
    
    async fn handle_file_update(&mut self, change: FileChangeEvent) -> Result<()> {
        // Generate embeddings for the updated file
        let embeddings = self.generate_embeddings(&change.path).await?;
        
        // Update vectors via GRPC
        let request = IncrementalReindexRequest {
            collection_id: change.collection_id.unwrap_or_default(),
            file_paths: vec![change.path.to_string_lossy().to_string()],
            force_reindex: false,
            embedding_provider: EmbeddingProvider::Auto,
        };
        
        self.grpc_client.reindex_incremental(request).await?;
        Ok(())
    }
}
```

## Configuration

### File Watcher Configuration

```yaml
# vectorize.yml
file_watcher:
  enabled: true
  debounce_delay: "2s"
  max_batch_size: 100
  content_hash_validation: true
  watch_patterns:
    - "**/*.md"
    - "**/*.txt"
    - "**/*.py"
    - "**/*.rs"
    - "**/*.js"
    - "**/*.ts"
  ignore_patterns:
    - "**/node_modules/**"
    - "**/target/**"
    - "**/__pycache__/**"
    - "**/.git/**"
  collections:
    - name: "documentation"
      watch_paths: ["docs/", "*.md"]
      embedding_provider: "bm25"
    - name: "source_code"
      watch_paths: ["src/", "*.rs", "*.py"]
      embedding_provider: "tfidf"
```

## Performance Considerations

### 1. Memory Usage
- **Event Queue**: Limited to 1000 events to prevent memory bloat
- **Content Hashing**: Cached hashes to avoid repeated file reads
- **Batch Processing**: Configurable batch sizes for optimal throughput

### 2. CPU Usage
- **Debouncing**: Reduces CPU usage by batching events
- **Content Validation**: Only processes files with actual content changes
- **Background Processing**: Non-blocking event processing

### 3. Network Usage
- **GRPC Efficiency**: Binary protocol reduces network overhead
- **Batch Operations**: Multiple changes processed in single GRPC call
- **Compression**: Optional compression for large file updates

## Error Handling

### 1. File System Errors
```rust
pub enum FileWatcherError {
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    FileSystemError(String),
    NetworkError(String),
    GrpcError(String),
}

impl FileWatcherSystem {
    async fn handle_error(&mut self, error: FileWatcherError) -> Result<()> {
        match error {
            FileWatcherError::FileNotFound(path) => {
                self.logger.warn(format!("File not found: {}", path.display()));
                // Remove from watch list
                self.remove_watch_path(path).await?;
            }
            FileWatcherError::PermissionDenied(path) => {
                self.logger.warn(format!("Permission denied: {}", path.display()));
                // Skip this file
            }
            _ => {
                self.logger.error(format!("File watcher error: {:?}", error));
                // Implement retry logic
                self.retry_failed_operation().await?;
            }
        }
        Ok(())
    }
}
```

### 2. Network Resilience
- **Retry Logic**: Exponential backoff for GRPC failures
- **Circuit Breaker**: Temporary disable on persistent failures
- **Health Checks**: Periodic validation of GRPC connection

## Monitoring and Metrics

### 1. Performance Metrics
```rust
pub struct FileWatcherMetrics {
    pub events_processed: u64,
    pub files_updated: u64,
    pub files_deleted: u64,
    pub processing_time_ms: u64,
    pub grpc_calls: u64,
    pub errors: u64,
}
```

### 2. Health Monitoring
- **Event Processing Rate**: Events per second
- **GRPC Response Time**: Average response time for vector operations
- **Error Rate**: Percentage of failed operations
- **Queue Depth**: Number of pending events

## Testing Strategy

### 1. Unit Tests
- File event parsing and validation
- Debouncing logic
- Content hash calculation
- Error handling scenarios

### 2. Integration Tests
- End-to-end file monitoring
- GRPC communication
- Cross-platform compatibility
- Performance under load

### 3. Performance Tests
- Large file processing
- High-frequency file changes
- Memory usage under stress
- Network resilience

## Deployment Considerations

### 1. Resource Requirements
- **Memory**: 50-100MB for event queue and caching
- **CPU**: Low impact with debouncing enabled
- **Network**: Minimal with efficient GRPC communication

### 2. Platform Support
- **Linux**: Full support with inotify
- **macOS**: Full support with FSEvents
- **Windows**: Full support with ReadDirectoryChangesW

### 3. Security Considerations
- **File Access**: Only monitors configured paths
- **Network Security**: GRPC over TLS
- **Permission Validation**: Respects file system permissions

## Future Enhancements

### 1. Advanced Features
- **Smart Filtering**: ML-based change detection
- **Compression**: Automatic file compression for large updates
- **Distributed Monitoring**: Multi-node file watching
- **Real-time Notifications**: WebSocket updates for UI

### 2. Performance Optimizations
- **Parallel Processing**: Multi-threaded event processing
- **Memory Mapping**: Efficient large file handling
- **Predictive Caching**: Anticipate file changes
- **Adaptive Debouncing**: Dynamic delay adjustment

---

**Implementation Priority**: High - Phase 5 Week 25-28
**Dependencies**: GRPC Vector Operations, Incremental Indexing Engine
**Estimated Effort**: 4 weeks
**Team Requirements**: 1 Senior Rust Developer, 1 Performance Engineer

