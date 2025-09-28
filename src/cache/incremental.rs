//! Incremental indexing system

use super::*;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{Duration, Instant};
use walkdir::WalkDir;

/// Incremental indexing processor
pub struct IncrementalProcessor {
    /// Cache manager
    cache_manager: Arc<CacheManager>,

    /// File change detector
    change_detector: Arc<FileChangeDetector>,

    /// Processing queue
    processing_queue: Arc<AsyncMutex<Vec<ProcessingTask>>>,

    /// Background workers
    workers: Arc<AsyncMutex<Vec<tokio::task::JoinHandle<()>>>>,

    /// Configuration
    config: IncrementalConfig,
}

/// Incremental indexing configuration
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Maximum number of workers
    pub max_workers: usize,

    /// Batch size for processing
    pub batch_size: usize,

    /// Debounce duration for file changes
    pub debounce_duration: Duration,

    /// File watching enabled
    pub file_watching_enabled: bool,

    /// Ignore patterns
    pub ignore_patterns: Vec<String>,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            max_workers: 4,
            batch_size: 10,
            debounce_duration: Duration::from_millis(500),
            file_watching_enabled: true,
            ignore_patterns: vec![
                "**/.git/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/dist/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/*.pyc".to_string(),
            ],
        }
    }
}

/// File change detector
pub struct FileChangeDetector {
    /// Watched directories
    watched_dirs: Arc<AsyncMutex<HashMap<String, PathBuf>>>,

    /// Change buffer
    change_buffer: Arc<AsyncMutex<Vec<FileChangeEvent>>>,

    /// Debounce timer
    debounce_timer: Arc<AsyncMutex<Option<Instant>>>,

    /// Debounce duration
    debounce_duration: Duration,
}

/// File change event
#[derive(Debug, Clone)]
pub enum FileChangeEvent {
    /// File created
    Created(PathBuf),
    /// File modified
    Modified(PathBuf),
    /// File deleted
    Deleted(PathBuf),
    /// File moved
    Moved(PathBuf, PathBuf),
}

/// Processing task
#[derive(Debug, Clone)]
pub struct ProcessingTask {
    /// Task ID
    pub id: String,

    /// Task operation
    pub operation: ProcessingOperation,

    /// Task priority
    pub priority: TaskPriority,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Retry count
    pub retry_count: u32,
}

/// Processing operation
#[derive(Debug, Clone)]
pub enum ProcessingOperation {
    /// Index file
    IndexFile {
        collection_name: String,
        file_path: PathBuf,
        change_type: FileChangeEvent,
    },
    /// Reindex collection
    ReindexCollection {
        collection_name: String,
        reason: ReindexReason,
    },
    /// Update metadata
    UpdateMetadata {
        collection_name: String,
        updates: CollectionCacheInfo,
    },
}

/// Task priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// Low priority (background processing)
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical priority (immediate processing)
    Critical,
}

/// Reindex reason
#[derive(Debug, Clone)]
pub enum ReindexReason {
    /// Embedding model changed
    EmbeddingModelChanged,
    /// Configuration changed
    ConfigurationChanged,
    /// Cache corrupted
    CacheCorrupted,
    /// Manual trigger
    ManualTrigger,
    /// File changes detected
    FileChangesDetected,
}

/// Processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Number of processed files
    pub processed_files: usize,

    /// Number of processed chunks
    pub processed_chunks: usize,

    /// Number of created vectors
    pub created_vectors: usize,

    /// Processing duration
    pub duration: Duration,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl ProcessingResult {
    pub fn new() -> Self {
        Self {
            processed_files: 0,
            processed_chunks: 0,
            created_vectors: 0,
            duration: Duration::from_secs(0),
            errors: Vec::new(),
        }
    }
}

impl IncrementalProcessor {
    /// Create new incremental processor
    pub async fn new(
        cache_manager: Arc<CacheManager>,
        config: IncrementalConfig,
    ) -> CacheResult<Self> {
        let change_detector = Arc::new(FileChangeDetector::new(config.debounce_duration));

        Ok(Self {
            cache_manager,
            change_detector,
            processing_queue: Arc::new(AsyncMutex::new(Vec::new())),
            workers: Arc::new(AsyncMutex::new(Vec::new())),
            config,
        })
    }

    /// Start background workers
    pub async fn start_workers(&self) -> CacheResult<()> {
        for worker_id in 0..self.config.max_workers {
            let processor = Arc::new(self.clone());
            let worker = tokio::spawn(async move {
                Self::worker_loop(worker_id, processor).await;
            });

            let mut workers = self.workers.lock().await;
            workers.push(worker);
        }

        Ok(())
    }

    /// Worker loop
    async fn worker_loop(worker_id: usize, processor: Arc<IncrementalProcessor>) {
        loop {
            let task = {
                let mut queue = processor.processing_queue.lock().await;
                queue.pop()
            };

            if let Some(task) = task {
                if let Err(e) = processor.process_task(task).await {
                    eprintln!("Worker {} failed to process task: {}", worker_id, e);
                }
            } else {
                // No tasks available, wait a bit
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    /// Process a single task
    async fn process_task(&self, task: ProcessingTask) -> CacheResult<ProcessingResult> {
        let start_time = Instant::now();
        let mut result = ProcessingResult::new();

        match task.operation {
            ProcessingOperation::IndexFile {
                collection_name,
                file_path,
                change_type,
            } => {
                result = self
                    .process_file_indexing(collection_name, file_path, change_type)
                    .await?;
            }
            ProcessingOperation::ReindexCollection {
                collection_name,
                reason,
            } => {
                result = self
                    .process_collection_reindexing(collection_name, reason)
                    .await?;
            }
            ProcessingOperation::UpdateMetadata {
                collection_name,
                updates,
            } => {
                self.cache_manager.update_collection_info(updates).await?;
            }
        }

        result.duration = start_time.elapsed();
        Ok(result)
    }

    /// Process file indexing
    async fn process_file_indexing(
        &self,
        collection_name: String,
        file_path: PathBuf,
        change_type: FileChangeEvent,
    ) -> CacheResult<ProcessingResult> {
        let mut result = ProcessingResult::new();

        match change_type {
            FileChangeEvent::Created(path) | FileChangeEvent::Modified(path) => {
                result = self.index_file(collection_name, path).await?;
            }
            FileChangeEvent::Deleted(path) => {
                result = self.remove_file_index(collection_name, path).await?;
            }
            FileChangeEvent::Moved(from, to) => {
                // Handle file move as delete + create
                let _ = self
                    .remove_file_index(collection_name.clone(), from)
                    .await?;
                result = self.index_file(collection_name, to).await?;
            }
        }

        Ok(result)
    }

    /// Index a single file
    async fn index_file(
        &self,
        collection_name: String,
        file_path: PathBuf,
    ) -> CacheResult<ProcessingResult> {
        let mut result = ProcessingResult::new();

        // Check if file exists
        if !file_path.exists() {
            result
                .errors
                .push(format!("File {} does not exist", file_path.display()));
            return Ok(result);
        }

        // Get file metadata
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len();
        let modified_time = DateTime::from_timestamp(
            metadata
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            0,
        )
        .unwrap_or_else(Utc::now);

        // Calculate file hash
        let content_hash = self.calculate_file_hash(&file_path).await?;

        // Check if file has changed
        if let Some(collection_info) = self
            .cache_manager
            .get_collection_info(&collection_name)
            .await
        {
            if let Some(file_info) = collection_info.get_file_hash(&file_path) {
                if file_info.content_hash == content_hash && !file_info.is_modified(modified_time) {
                    // File hasn't changed, skip indexing
                    return Ok(result);
                }
            }
        }

        // TODO: Implement actual file indexing logic here
        // This would involve:
        // 1. Loading the file content
        // 2. Chunking the content
        // 3. Creating embeddings
        // 4. Storing vectors in the vector store
        // 5. Updating cache metadata

        result.processed_files = 1;
        result.processed_chunks = 1; // Placeholder
        result.created_vectors = 1; // Placeholder

        // Update cache metadata
        let file_info = FileHashInfo::new(
            content_hash,
            file_size,
            modified_time,
            result.processed_chunks,
            vec!["vector_id_placeholder".to_string()],
        );

        if let Some(mut collection_info) = self
            .cache_manager
            .get_collection_info(&collection_name)
            .await
        {
            collection_info.update_file_hash(file_path, file_info);
            collection_info.update_indexed();
            self.cache_manager
                .update_collection_info(collection_info)
                .await?;
        }

        Ok(result)
    }

    /// Remove file index
    async fn remove_file_index(
        &self,
        collection_name: String,
        file_path: PathBuf,
    ) -> CacheResult<ProcessingResult> {
        let mut result = ProcessingResult::new();

        if let Some(mut collection_info) = self
            .cache_manager
            .get_collection_info(&collection_name)
            .await
        {
            if collection_info.remove_file_hash(&file_path).is_some() {
                collection_info.update_indexed();
                self.cache_manager
                    .update_collection_info(collection_info)
                    .await?;
                result.processed_files = 1;
            }
        }

        Ok(result)
    }

    /// Process collection reindexing
    async fn process_collection_reindexing(
        &self,
        collection_name: String,
        reason: ReindexReason,
    ) -> CacheResult<ProcessingResult> {
        let mut result = ProcessingResult::new();

        // TODO: Implement collection reindexing logic
        // This would involve:
        // 1. Finding all files in the collection
        // 2. Processing each file
        // 3. Updating cache metadata

        result.processed_files = 0; // Placeholder
        result.processed_chunks = 0; // Placeholder
        result.created_vectors = 0; // Placeholder

        Ok(result)
    }

    /// Calculate file hash
    async fn calculate_file_hash(&self, file_path: &Path) -> CacheResult<String> {
        let content = fs::read(file_path).await?;
        let mut hasher = Sha256::default();
        hasher.update(&content);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Add processing task to queue
    pub async fn add_task(&self, task: ProcessingTask) -> CacheResult<()> {
        let mut queue = self.processing_queue.lock().await;
        queue.push(task);

        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(())
    }

    /// Get processing queue size
    pub async fn queue_size(&self) -> usize {
        let queue = self.processing_queue.lock().await;
        queue.len()
    }

    /// Stop all workers
    pub async fn stop_workers(&self) {
        let mut workers = self.workers.lock().await;
        for worker in workers.drain(..) {
            worker.abort();
        }
    }
}

impl Clone for IncrementalProcessor {
    fn clone(&self) -> Self {
        Self {
            cache_manager: Arc::clone(&self.cache_manager),
            change_detector: Arc::clone(&self.change_detector),
            processing_queue: Arc::clone(&self.processing_queue),
            workers: Arc::clone(&self.workers),
            config: self.config.clone(),
        }
    }
}

impl FileChangeDetector {
    /// Create new file change detector
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            watched_dirs: Arc::new(AsyncMutex::new(HashMap::new())),
            change_buffer: Arc::new(AsyncMutex::new(Vec::new())),
            debounce_timer: Arc::new(AsyncMutex::new(None)),
            debounce_duration,
        }
    }

    /// Watch directory for changes
    pub async fn watch_directory(&self, name: String, path: PathBuf) -> CacheResult<()> {
        let mut watched_dirs = self.watched_dirs.lock().await;
        watched_dirs.insert(name, path);
        Ok(())
    }

    /// Stop watching directory
    pub async fn unwatch_directory(&self, name: &str) -> CacheResult<()> {
        let mut watched_dirs = self.watched_dirs.lock().await;
        watched_dirs.remove(name);
        Ok(())
    }

    /// Get pending changes
    pub async fn get_pending_changes(&self) -> CacheResult<Vec<FileChangeEvent>> {
        let mut buffer = self.change_buffer.lock().await;
        let changes = buffer.drain(..).collect();
        Ok(changes)
    }

    /// Add change event
    pub async fn add_change(&self, event: FileChangeEvent) -> CacheResult<()> {
        let mut buffer = self.change_buffer.lock().await;
        buffer.push(event);
        Ok(())
    }

    /// Scan directory for changes
    pub async fn scan_directory(&self, path: &Path) -> CacheResult<Vec<FileChangeEvent>> {
        let mut changes = Vec::new();

        for entry in WalkDir::new(path) {
            let entry = entry?;
            let path = entry.path().to_path_buf();

            if entry.file_type().is_file() {
                // Check if file is new or modified
                if let Ok(metadata) = std::fs::metadata(&path) {
                    let modified_time = DateTime::from_timestamp(
                        metadata
                            .modified()?
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs() as i64,
                        0,
                    )
                    .unwrap_or_else(Utc::now);

                    // TODO: Compare with cached modification time
                    changes.push(FileChangeEvent::Modified(path));
                }
            }
        }

        Ok(changes)
    }
}
