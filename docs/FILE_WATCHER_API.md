# 🔌 File Watcher API Documentation

## 📋 Overview

This document provides comprehensive API documentation for the File Watcher system components, including public interfaces, configuration options, and usage examples.

## 🏗️ Core API

### **FileWatcherSystem**

The main entry point for the file watching system.

```rust
pub struct FileWatcherSystem {
    config: FileWatcherConfig,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    vector_operations: Arc<operations::VectorOperations>,
    debouncer: Arc<debouncer::Debouncer>,
    hash_validator: Arc<hash_validator::HashValidator>,
    discovery: Option<Arc<discovery::FileDiscovery>>,
    metrics: Arc<MetricsCollector>,
    watcher: Option<watcher::Watcher>,
}
```

#### **Constructor**
```rust
pub fn new(
    config: FileWatcherConfig,
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
) -> Self
```

#### **Methods**

##### `start() -> Result<()>`
Initializes and starts the file watcher system.

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
let mut file_watcher = FileWatcherSystem::new(config, vector_store, embedding_manager);
file_watcher.start().await?;
```

##### `stop() -> Result<()>`
Gracefully stops the file watcher system.

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
file_watcher.stop()?;
```

## 🎯 Event System API

### **FileChangeEvent**

Represents different types of file system events.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeEvent {
    Created(PathBuf),      // New file created
    Modified(PathBuf),     // Existing file modified
    Deleted(PathBuf),      // File removed
    Renamed(PathBuf, PathBuf), // File renamed/moved
}
```

#### **Event Conversion**
```rust
impl FileChangeEvent {
    pub fn from_notify_event(event: notify::Event) -> Self
}
```

**Parameters**:
- `event`: Raw notify event from the file system

**Returns**: `FileChangeEvent` - Converted event with proper handling

**Example**:
```rust
let file_event = FileChangeEvent::from_notify_event(notify_event);
```

### **FileChangeEventWithMetadata**

Enhanced event with additional metadata.

```rust
#[derive(Debug, Clone)]
pub struct FileChangeEventWithMetadata {
    pub event: FileChangeEvent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub content_hash: Option<String>,
    pub file_size: Option<u64>,
}
```

## ⚙️ Configuration API

### **FileWatcherConfig**

Configuration structure for the file watcher system.

```rust
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    pub watch_paths: Vec<PathBuf>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub debounce_duration: Duration,
    pub auto_discovery: bool,
    pub enable_auto_update: bool,
    pub hot_reload: bool,
}
```

#### **Default Configuration**
```rust
impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![],
            include_patterns: vec!["**/*.md".to_string(), "**/*.rs".to_string()],
            exclude_patterns: vec![
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
            ],
            debounce_duration: Duration::from_secs(1),
            auto_discovery: true,
            enable_auto_update: true,
            hot_reload: true,
        }
    }
}
```

#### **Pattern Matching Methods**

##### `should_process_file(path: &Path) -> bool`
Determines if a file should be processed based on include/exclude patterns.

**Parameters**:
- `path`: File path to check

**Returns**: `bool` - True if file should be processed

**Example**:
```rust
if config.should_process_file(&file_path) {
    // Process the file
}
```

##### `should_process_file_silent(path: &Path) -> bool`
Silent version of pattern matching without logging.

**Parameters**:
- `path`: File path to check

**Returns**: `bool` - True if file should be processed

**Example**:
```rust
if config.should_process_file_silent(&file_path) {
    // Process without logging
}
```

## 🔄 Operations API

### **VectorOperations**

Handles vector database operations for file changes.

```rust
pub struct VectorOperations {
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    config: FileWatcherConfig,
}
```

#### **Constructor**
```rust
pub fn new(
    vector_store: Arc<VectorStore>,
    embedding_manager: Arc<RwLock<EmbeddingManager>>,
    config: FileWatcherConfig,
) -> Self
```

#### **Methods**

##### `process_file_change(event: &FileChangeEventWithMetadata) -> Result<()>`
Processes a file change event and updates the vector database.

**Parameters**:
- `event`: File change event with metadata

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
let result = vector_ops.process_file_change(&event).await?;
```

##### `index_file_from_path(path: &Path) -> Result<()>`
Indexes a single file into the vector database.

**Parameters**:
- `path`: Path to the file to index

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
vector_ops.index_file_from_path(&file_path).await?;
```

##### `remove_file_from_path(path: &Path) -> Result<()>`
Removes a file from the vector database.

**Parameters**:
- `path`: Path to the file to remove

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
vector_ops.remove_file_from_path(&file_path).await?;
```

## 🎛️ Debouncer API

### **Debouncer**

Manages event aggregation and rate limiting.

```rust
pub struct Debouncer {
    events: Arc<RwLock<Vec<FileChangeEventWithMetadata>>>,
    callback: Option<Box<dyn Fn(FileChangeEventWithMetadata) + Send + Sync>>,
    debounce_duration: Duration,
    processing_files: Arc<RwLock<HashMap<PathBuf, Instant>>>,
}
```

#### **Constructor**
```rust
pub fn new(debounce_duration: Duration) -> Self
```

#### **Methods**

##### `set_event_callback<F>(callback: F) -> Result<()>`
Sets the callback function to be called when events are processed.

**Parameters**:
- `callback`: Function to call with processed events

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
debouncer.set_event_callback(|event| {
    println!("Processing event: {:?}", event);
}).await?;
```

##### `add_event_with_metadata(event: FileChangeEventWithMetadata)`
Adds an event to the debouncer queue.

**Parameters**:
- `event`: Event with metadata to add

**Example**:
```rust
debouncer.add_event_with_metadata(event_with_metadata).await;
```

##### `clear_pending_events()`
Clears all pending events from the queue.

**Example**:
```rust
debouncer.clear_pending_events().await;
```

## 📊 Metrics API

### **MetricsCollector**

Collects and provides performance metrics.

```rust
pub struct MetricsCollector {
    events_processed: AtomicU64,
    files_indexed: AtomicU64,
    errors_count: AtomicU64,
    last_event_time: AtomicU64,
}
```

#### **Methods**

##### `get_metrics() -> FileWatcherMetrics`
Returns current metrics snapshot.

**Returns**: `FileWatcherMetrics` - Current metrics data

**Example**:
```rust
let metrics = metrics_collector.get_metrics();
println!("Events processed: {}", metrics.events_processed);
```

##### `increment_events_processed()`
Increments the events processed counter.

**Example**:
```rust
metrics_collector.increment_events_processed();
```

##### `increment_files_indexed()`
Increments the files indexed counter.

**Example**:
```rust
metrics_collector.increment_files_indexed();
```

##### `increment_errors()`
Increments the error counter.

**Example**:
```rust
metrics_collector.increment_errors();
```

## 🔧 Watcher API

### **Watcher**

Low-level file system event monitoring.

```rust
pub struct Watcher {
    event_sender: Option<mpsc::UnboundedSender<FileChangeEvent>>,
    notify_watcher: Option<notify::RecommendedWatcher>,
}
```

#### **Constructor**
```rust
pub fn new() -> Result<Self>
```

#### **Methods**

##### `start<F>(config: FileWatcherConfig, callback: F) -> Result<()>`
Starts the file system watcher.

**Parameters**:
- `config`: File watcher configuration
- `callback`: Event processing callback

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
watcher.start(config, |event| {
    println!("File changed: {:?}", event);
})?;
```

##### `add_watch_path(path: &Path) -> Result<()>`
Adds a path to watch.

**Parameters**:
- `path`: Directory or file path to monitor

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
watcher.add_watch_path(&Path::new("/path/to/watch"))?;
```

##### `stop() -> Result<()>`
Stops the file system watcher.

**Returns**: `Result<()>` - Success or error status

**Example**:
```rust
watcher.stop()?;
```

## 🚀 Usage Examples

### **Basic Setup**

```rust
use vectorizer::file_watcher::{FileWatcherSystem, FileWatcherConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = FileWatcherConfig {
        watch_paths: vec![PathBuf::from("/path/to/monitor")],
        include_patterns: vec!["**/*.md".to_string()],
        exclude_patterns: vec!["**/.git/**".to_string()],
        ..Default::default()
    };
    
    // Create file watcher system
    let mut file_watcher = FileWatcherSystem::new(
        config,
        vector_store,
        embedding_manager,
    );
    
    // Start monitoring
    file_watcher.start().await?;
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    // Stop gracefully
    file_watcher.stop()?;
    
    Ok(())
}
```

### **Custom Event Processing**

```rust
use vectorizer::file_watcher::{Debouncer, FileChangeEventWithMetadata};

async fn setup_custom_processing() -> Result<(), Box<dyn std::error::Error>> {
    let mut debouncer = Debouncer::new(Duration::from_secs(2));
    
    debouncer.set_event_callback(|event: FileChangeEventWithMetadata| {
        match event.event {
            FileChangeEvent::Created(path) => {
                println!("New file created: {:?}", path);
            }
            FileChangeEvent::Modified(path) => {
                println!("File modified: {:?}", path);
            }
            FileChangeEvent::Deleted(path) => {
                println!("File deleted: {:?}", path);
            }
            _ => {}
        }
    }).await?;
    
    Ok(())
}
```

### **Configuration from YAML**

```rust
use vectorizer::config::load_file_watcher_config;

async fn load_config_from_yaml() -> Result<FileWatcherConfig, Box<dyn std::error::Error>> {
    let config = load_file_watcher_config().await?;
    Ok(config)
}
```

## 🛡️ Error Handling

### **Common Error Types**

```rust
#[derive(Debug, thiserror::Error)]
pub enum FileWatcherError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Pattern matching error: {0}")]
    PatternMatching(String),
    
    #[error("Vector operation error: {0}")]
    VectorOperation(String),
}
```

### **Error Handling Best Practices**

```rust
async fn robust_file_processing() -> Result<(), FileWatcherError> {
    match vector_ops.process_file_change(&event).await {
        Ok(_) => {
            metrics.increment_events_processed();
        }
        Err(e) => {
            metrics.increment_errors();
            tracing::error!("Failed to process file change: {}", e);
            // Continue processing other events
        }
    }
    Ok(())
}
```

---

**Last Updated**: October 10, 2025  
**Version**: 1.0  
**Status**: ✅ Production Ready
