# File Watcher System - Technical Specification

## Overview

The File Watcher System is a critical component of Phase 5 that provides real-time monitoring of indexed files and automatic incremental reindexing. This system eliminates the need for manual reindexing and ensures that the vector database stays synchronized with file system changes.

**Status**: Phase 5 Implementation Target  
**Priority**: High - Production Performance Critical  
**Timeline**: Weeks 25-28 of Phase 5  
**Dependencies**: GRPC Vector Operations, Incremental Indexing Engine

## Problem Statement

### Current Limitations
- **Manual Reindexing**: Users must manually trigger full reindexing for file changes
- **No Real-time Updates**: Changes to indexed files are not automatically detected
- **Resource Waste**: Full reindexing for minor file changes
- **Poor User Experience**: No automatic synchronization with file system changes
- **Limited Vector Operations**: GRPC lacks update/delete operations for vectors

### Impact on Production
- **Slow Response**: 30-60 seconds startup time for reindexing
- **Resource Inefficiency**: 90% unnecessary CPU/memory consumption
- **User Frustration**: Manual intervention required for file updates
- **Scalability Issues**: Performance degrades with large file collections

## Solution Architecture

### High-Level Design

```
┌─────────────────┐    File Events    ┌──────────────────┐    GRPC    ┌─────────────────┐
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

### Core Components

#### 1. File Watcher Engine
```rust
pub struct FileWatcherSystem {
    watcher: notify::RecommendedWatcher,
    change_queue: Arc<Mutex<VecDeque<FileChangeEvent>>>,
    debounce_timer: Arc<Mutex<Option<tokio::time::Instant>>>,
    grpc_client: GrpcClient,
    config: FileWatcherConfig,
    metrics: FileWatcherMetrics,
}

pub struct FileWatcherConfig {
    pub debounce_delay: Duration,
    pub max_batch_size: usize,
    pub content_hash_validation: bool,
    pub watch_patterns: Vec<glob::Pattern>,
    pub ignore_patterns: Vec<glob::Pattern>,
    pub max_queue_size: usize,
    pub retry_attempts: u32,
    pub health_check_interval: Duration,
}
```

#### 2. File Change Events
```rust
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub timestamp: DateTime<Utc>,
    pub content_hash: Option<String>,
    pub collection_id: Option<String>,
    pub file_size: u64,
    pub retry_count: u32,
}

pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Moved { old_path: PathBuf },
    Renamed { old_path: PathBuf },
}

impl FileChangeEvent {
    pub fn from_notify_event(event: notify::Event) -> Result<Self> {
        // Convert notify event to our internal representation
    }
    
    pub async fn validate_content_change(&mut self) -> Result<bool> {
        // Validate if content actually changed
    }
    
    pub async fn calculate_content_hash(&self) -> Result<String> {
        // Calculate SHA-256 hash of file content
    }
}
```

#### 3. GRPC Vector Operations
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
    
    // File watcher operations
    rpc StartFileWatching(StartWatchingRequest) returns (StartWatchingResponse);
    rpc StopFileWatching(StopWatchingRequest) returns (StopWatchingResponse);
    rpc GetWatcherStatus(WatcherStatusRequest) returns (WatcherStatusResponse);
}

message VectorUpdateRequest {
    string collection_id = 1;
    string vector_id = 2;
    Vector new_vector = 3;
    bool update_metadata = 4;
    bool force_update = 5;
}

message IncrementalReindexRequest {
    string collection_id = 1;
    repeated string file_paths = 2;
    bool force_reindex = 3;
    EmbeddingProvider embedding_provider = 4;
    bool validate_content = 5;
}
```

## Implementation Details

### 1. Cross-Platform File Monitoring

The system uses the `notify` crate for cross-platform file system monitoring:

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind, RecommendedWatcher};

impl FileWatcherSystem {
    pub async fn start_watching(&mut self, watch_paths: Vec<PathBuf>) -> Result<()> {
        for path in watch_paths {
            if path.exists() {
                self.watcher.watch(&path, RecursiveMode::Recursive)?;
                self.logger.info(format!("Started watching: {}", path.display()));
            } else {
                self.logger.warn(format!("Path does not exist: {}", path.display()));
            }
        }
        
        // Start event processing loop
        tokio::spawn(self.process_events());
        Ok(())
    }
    
    async fn process_events(&mut self) -> Result<()> {
        while let Some(event) = self.watcher.rx.recv().await {
            if let Err(e) = self.handle_file_event(event).await {
                self.logger.error(format!("Error handling file event: {}", e));
                self.metrics.errors += 1;
            }
        }
        Ok(())
    }
    
    async fn handle_file_event(&mut self, event: Event) -> Result<()> {
        for path in event.paths {
            if self.should_ignore_path(&path) {
                continue;
            }
            
            let file_change = FileChangeEvent::from_notify_event(event.clone(), path)?;
            
            // Add to queue
            let mut queue = self.change_queue.lock().await;
            if queue.len() >= self.config.max_queue_size {
                self.logger.warn("File change queue is full, dropping oldest event");
                queue.pop_front();
            }
            queue.push_back(file_change);
            
            // Reset debounce timer
            let mut timer = self.debounce_timer.lock().await;
            *timer = Some(tokio::time::Instant::now() + self.config.debounce_delay);
        }
        
        Ok(())
    }
}
```

### 2. Debounced Processing

To avoid excessive reindexing, the system implements debounced processing:

```rust
impl FileWatcherSystem {
    async fn process_debounced_changes(&mut self) -> Result<()> {
        let changes = self.collect_batched_changes().await;
        if !changes.is_empty() {
            self.logger.info(format!("Processing {} file changes", changes.len()));
            self.process_batch_changes(changes).await?;
        }
        Ok(())
    }
    
    async fn collect_batched_changes(&mut self) -> Vec<FileChangeEvent> {
        let mut changes = Vec::new();
        let mut queue = self.change_queue.lock().await;
        
        while let Some(change) = queue.pop_front() {
            changes.push(change);
            if changes.len() >= self.config.max_batch_size {
                break;
            }
        }
        
        changes
    }
    
    async fn process_batch_changes(&mut self, changes: Vec<FileChangeEvent>) -> Result<()> {
        // Group changes by collection
        let mut collection_changes: HashMap<String, Vec<FileChangeEvent>> = HashMap::new();
        
        for change in changes {
            let collection_id = change.collection_id.clone().unwrap_or_default();
            collection_changes.entry(collection_id).or_insert_with(Vec::new).push(change);
        }
        
        // Process each collection's changes
        for (collection_id, changes) in collection_changes {
            self.process_collection_changes(collection_id, changes).await?;
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
        if !self.path.exists() {
            return Ok(String::new());
        }
        
        let content = tokio::fs::read(&self.path).await?;
        Ok(sha256::digest(&content))
    }
    
    async fn get_previous_hash(&self) -> Result<Option<String>> {
        // Retrieve from cache or database
        // Implementation depends on storage backend
        Ok(None)
    }
}
```

### 4. GRPC Integration

The file watcher communicates with the vector database via GRPC:

```rust
impl FileWatcherSystem {
    async fn process_collection_changes(&mut self, collection_id: String, changes: Vec<FileChangeEvent>) -> Result<()> {
        let mut files_to_update = Vec::new();
        let mut files_to_delete = Vec::new();
        
        for change in changes {
            match change.event_type {
                FileEventType::Created | FileEventType::Modified => {
                    if change.validate_content_change().await? {
                        files_to_update.push(change.path.to_string_lossy().to_string());
                    }
                }
                FileEventType::Deleted => {
                    files_to_delete.push(change.path.to_string_lossy().to_string());
                }
                FileEventType::Moved { old_path } => {
                    // Handle file move as delete + create
                    files_to_delete.push(old_path.to_string_lossy().to_string());
                    if change.validate_content_change().await? {
                        files_to_update.push(change.path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        // Process deletions first
        if !files_to_delete.is_empty() {
            self.delete_vectors_for_files(&collection_id, &files_to_delete).await?;
        }
        
        // Process updates
        if !files_to_update.is_empty() {
            self.update_vectors_for_files(&collection_id, &files_to_update).await?;
        }
        
        Ok(())
    }
    
    async fn update_vectors_for_files(&mut self, collection_id: &str, file_paths: &[String]) -> Result<()> {
        let request = IncrementalReindexRequest {
            collection_id: collection_id.to_string(),
            file_paths: file_paths.to_vec(),
            force_reindex: false,
            embedding_provider: EmbeddingProvider::Auto,
            validate_content: true,
        };
        
        let response = self.grpc_client.reindex_incremental(request).await?;
        self.logger.info(format!("Updated {} files in collection {}", response.updated_count, collection_id));
        
        Ok(())
    }
    
    async fn delete_vectors_for_files(&mut self, collection_id: &str, file_paths: &[String]) -> Result<()> {
        for file_path in file_paths {
            let request = VectorDeleteRequest {
                collection_id: collection_id.to_string(),
                vector_id: self.path_to_vector_id(file_path)?,
            };
            
            self.grpc_client.delete_vector(request).await?;
        }
        
        self.logger.info(format!("Deleted {} files from collection {}", file_paths.len(), collection_id));
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
  max_queue_size: 1000
  content_hash_validation: true
  retry_attempts: 3
  health_check_interval: "30s"
  
  watch_patterns:
    - "**/*.md"
    - "**/*.txt"
    - "**/*.py"
    - "**/*.rs"
    - "**/*.js"
    - "**/*.ts"
    - "**/*.json"
    - "**/*.yaml"
    - "**/*.yml"
  
  ignore_patterns:
    - "**/node_modules/**"
    - "**/target/**"
    - "**/__pycache__/**"
    - "**/.git/**"
    - "**/.*"
    - "**/*.tmp"
    - "**/*.log"
    - "**/*.cache"
  
  collections:
    - name: "documentation"
      watch_paths: ["docs/", "*.md", "README.md"]
      embedding_provider: "bm25"
      auto_index: true
      
    - name: "source_code"
      watch_paths: ["src/", "*.rs", "*.py", "*.js", "*.ts"]
      embedding_provider: "tfidf"
      auto_index: true
      
    - name: "config_files"
      watch_paths: ["*.yml", "*.yaml", "*.json"]
      embedding_provider: "bow"
      auto_index: false
```

### Environment Variables

```bash
# File Watcher Configuration
VECTORIZER_WATCHER_ENABLED=true
VECTORIZER_WATCHER_DEBOUNCE_DELAY=2s
VECTORIZER_WATCHER_MAX_BATCH_SIZE=100
VECTORIZER_WATCHER_CONTENT_HASH_VALIDATION=true
VECTORIZER_WATCHER_HEALTH_CHECK_INTERVAL=30s

# GRPC Configuration
VECTORIZER_GRPC_HOST=localhost
VECTORIZER_GRPC_PORT=15003
VECTORIZER_GRPC_TIMEOUT=30s
VECTORIZER_GRPC_RETRY_ATTEMPTS=3
```

## Performance Considerations

### 1. Memory Usage
- **Event Queue**: Limited to 1000 events to prevent memory bloat
- **Content Hashing**: Cached hashes to avoid repeated file reads
- **Batch Processing**: Configurable batch sizes for optimal throughput
- **Memory Monitoring**: Automatic cleanup of old events and hashes

### 2. CPU Usage
- **Debouncing**: Reduces CPU usage by batching events
- **Content Validation**: Only processes files with actual content changes
- **Background Processing**: Non-blocking event processing
- **Smart Filtering**: Pattern-based file filtering to reduce processing

### 3. Network Usage
- **GRPC Efficiency**: Binary protocol reduces network overhead
- **Batch Operations**: Multiple changes processed in single GRPC call
- **Compression**: Optional compression for large file updates
- **Connection Pooling**: Reuse GRPC connections for better performance

### 4. File System Impact
- **Efficient Monitoring**: Use native OS file system events
- **Minimal I/O**: Only read files when content validation is needed
- **Smart Caching**: Cache file metadata to reduce system calls
- **Error Recovery**: Handle file system errors gracefully

## Error Handling

### 1. File System Errors
```rust
pub enum FileWatcherError {
    FileNotFound(PathBuf),
    PermissionDenied(PathBuf),
    FileSystemError(String),
    NetworkError(String),
    GrpcError(String),
    ConfigurationError(String),
    QueueFull,
    ProcessingTimeout,
}

impl FileWatcherSystem {
    async fn handle_error(&mut self, error: FileWatcherError) -> Result<()> {
        match error {
            FileWatcherError::FileNotFound(path) => {
                self.logger.warn(format!("File not found: {}", path.display()));
                self.remove_watch_path(path).await?;
            }
            FileWatcherError::PermissionDenied(path) => {
                self.logger.warn(format!("Permission denied: {}", path.display()));
                // Skip this file and continue
            }
            FileWatcherError::QueueFull => {
                self.logger.warn("File change queue is full, dropping oldest events");
                self.clear_oldest_events().await?;
            }
            FileWatcherError::ProcessingTimeout => {
                self.logger.error("File processing timeout, retrying...");
                self.retry_failed_operations().await?;
            }
            _ => {
                self.logger.error(format!("File watcher error: {:?}", error));
                self.metrics.errors += 1;
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
- **Connection Recovery**: Automatic reconnection on network issues

### 3. Recovery Mechanisms
- **State Persistence**: Save watcher state for recovery
- **Checkpoint System**: Resume from last successful operation
- **Error Reporting**: Detailed error logs for debugging
- **Graceful Degradation**: Continue operation with reduced functionality

## Monitoring and Metrics

### 1. Performance Metrics
```rust
pub struct FileWatcherMetrics {
    pub events_processed: u64,
    pub files_updated: u64,
    pub files_deleted: u64,
    pub files_created: u64,
    pub processing_time_ms: u64,
    pub grpc_calls: u64,
    pub grpc_errors: u64,
    pub queue_depth: u64,
    pub errors: u64,
    pub last_processed: DateTime<Utc>,
    pub uptime: Duration,
}

impl FileWatcherMetrics {
    pub fn new() -> Self {
        Self {
            events_processed: 0,
            files_updated: 0,
            files_deleted: 0,
            files_created: 0,
            processing_time_ms: 0,
            grpc_calls: 0,
            grpc_errors: 0,
            queue_depth: 0,
            errors: 0,
            last_processed: Utc::now(),
            uptime: Duration::from_secs(0),
        }
    }
    
    pub fn record_event_processed(&mut self) {
        self.events_processed += 1;
        self.last_processed = Utc::now();
    }
    
    pub fn record_file_updated(&mut self) {
        self.files_updated += 1;
    }
    
    pub fn record_grpc_call(&mut self, success: bool) {
        self.grpc_calls += 1;
        if !success {
            self.grpc_errors += 1;
        }
    }
}
```

### 2. Health Monitoring
- **Event Processing Rate**: Events per second
- **GRPC Response Time**: Average response time for vector operations
- **Error Rate**: Percentage of failed operations
- **Queue Depth**: Number of pending events
- **Memory Usage**: Current memory consumption
- **File System Health**: Monitor file system performance

### 3. Alerting
- **High Error Rate**: Alert when error rate exceeds threshold
- **Queue Overflow**: Alert when queue is consistently full
- **GRPC Failures**: Alert on persistent GRPC connection issues
- **Performance Degradation**: Alert when processing time increases

## Testing Strategy

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_file_change_event_parsing() {
        // Test parsing of notify events
    }
    
    #[tokio::test]
    async fn test_debouncing_logic() {
        // Test debouncing mechanism
    }
    
    #[tokio::test]
    async fn test_content_hash_validation() {
        // Test content hash calculation and validation
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // Test various error scenarios
    }
}
```

### 2. Integration Tests
- **End-to-end file monitoring**: Complete workflow testing
- **GRPC communication**: Test all GRPC operations
- **Cross-platform compatibility**: Test on different OS
- **Performance under load**: Stress testing with many files

### 3. Performance Tests
- **Large file processing**: Test with large files
- **High-frequency changes**: Test with rapid file changes
- **Memory usage**: Monitor memory consumption
- **Network resilience**: Test network failure scenarios

### 4. End-to-End Tests
```rust
#[tokio::test]
async fn test_complete_file_watching_workflow() {
    // 1. Start file watcher
    // 2. Create/modify files
    // 3. Verify vectors are updated
    // 4. Verify search results reflect changes
}
```

## Deployment Considerations

### 1. Resource Requirements
- **Memory**: 50-100MB for event queue and caching
- **CPU**: Low impact with debouncing enabled
- **Network**: Minimal with efficient GRPC communication
- **Disk**: Minimal for configuration and logs

### 2. Platform Support
- **Linux**: Full support with inotify
- **macOS**: Full support with FSEvents
- **Windows**: Full support with ReadDirectoryChangesW

### 3. Security Considerations
- **File Access**: Only monitors configured paths
- **Network Security**: GRPC over TLS
- **Permission Validation**: Respects file system permissions
- **Input Validation**: Validate all file paths and operations

### 4. Scalability
- **Horizontal Scaling**: Multiple watcher instances
- **Load Balancing**: Distribute file monitoring across instances
- **Resource Limits**: Configurable limits for memory and CPU
- **Performance Tuning**: Optimize for different workloads

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

### 3. Integration Enhancements
- **Cloud Storage**: Support for cloud file systems
- **Version Control**: Integration with Git/SVN
- **Database Integration**: Direct database change monitoring
- **API Integration**: Monitor remote file changes via APIs

## Implementation Timeline

### Week 25: Core Infrastructure
- [ ] File watcher engine implementation
- [ ] Basic event processing
- [ ] Configuration system
- [ ] Unit tests

### Week 26: GRPC Integration
- [ ] GRPC vector operations
- [ ] Batch processing
- [ ] Error handling
- [ ] Integration tests

### Week 27: Advanced Features
- [ ] Content hash validation
- [ ] Debouncing mechanism
- [ ] Performance optimization
- [ ] Cross-platform testing

### Week 28: Production Readiness
- [ ] Monitoring and metrics
- [ ] Documentation
- [ ] Performance testing
- [ ] Deployment preparation

## Success Criteria

### Functional Requirements
- [ ] Real-time file change detection (< 1 second latency)
- [ ] Automatic vector updates without manual intervention
- [ ] Cross-platform compatibility (Windows, macOS, Linux)
- [ ] Configurable file patterns and ignore rules
- [ ] Robust error handling and recovery

### Performance Requirements
- [ ] 90% reduction in processing for unchanged files
- [ ] < 50ms response time for GRPC operations
- [ ] < 100MB memory usage for event processing
- [ ] Support for 10,000+ files per collection
- [ ] 99.9% uptime with automatic recovery

### Quality Requirements
- [ ] 95% test coverage for core functionality
- [ ] Zero critical security vulnerabilities
- [ ] Comprehensive documentation
- [ ] Performance benchmarks and monitoring
- [ ] Production deployment validation

---

**Document Version**: 1.0  
**Last Updated**: September 26, 2025  
**Next Review**: After Phase 5 Week 25 implementation  
**Status**: Ready for Implementation
