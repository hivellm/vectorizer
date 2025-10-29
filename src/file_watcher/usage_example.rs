//! Example usage of the integrated file watcher synchronization system
//! 
//! This example demonstrates how to use the IntegratedSyncManager
//! with existing Vectorizer systems, following governance principles.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

use crate::embedding::cache::{EmbeddingCache, CacheConfig};
use crate::embedding::EmbeddingManager;
use crate::db::VectorStore;
use crate::batch::{ParallelProcessor, config::BatchConfig};

use super::{
    HashValidator,
    IntegratedSyncManager, IntegratedSyncConfig,
    FileChangeEvent, FileChangeEventWithMetadata,
    MetricsCollector,
};

/// Example of how to set up the integrated file watcher system
pub async fn setup_integrated_file_watcher() -> Result<IntegratedSyncManager> {
    // 1. Create existing systems (these would typically be created elsewhere)
    let hash_validator = Arc::new(HashValidator::new());
    let embedding_cache = Arc::new(EmbeddingCache::new(CacheConfig::default())?);
    let vector_store = Arc::new(VectorStore::new());
    let embedding_manager = Arc::new(RwLock::new(EmbeddingManager::new(crate::embedding::EmbeddingConfig::default())));
    let parallel_processor = Arc::new(ParallelProcessor::new(Arc::new(BatchConfig::default())));

    // 2. Configure the integrated sync manager
    let config = IntegratedSyncConfig {
        enable_proactive_invalidation: true,
        enable_atomic_updates: true,
        max_concurrent_operations: 4,
        max_retry_attempts: 3,
        retry_delay_ms: 1000,
        enable_metrics: true,
    };

    // 3. Create metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());

    // 4. Create the integrated sync manager
    let sync_manager = IntegratedSyncManager::new(
        config,
        hash_validator,
        embedding_cache,
        vector_store,
        embedding_manager,
        parallel_processor,
        metrics_collector,
    );

    Ok(sync_manager)
}

/// Example of how to process file change events
pub async fn process_file_changes_example() -> Result<()> {
    // Set up the system
    let sync_manager = setup_integrated_file_watcher().await?;

    // Example: Process a file creation event
    let file_path = PathBuf::from("/path/to/docs/architecture/design.md");
    let event = FileChangeEventWithMetadata {
        event: FileChangeEvent::Created(file_path.clone()),
        timestamp: chrono::Utc::now(),
        content_hash: Some("abc123".to_string()),
        file_size: Some(1024),
    };

    // Process the event using existing systems
    sync_manager.process_file_change(&event).await?;

    // Example: Process a file modification event
    let event = FileChangeEventWithMetadata {
        event: FileChangeEvent::Modified(file_path.clone()),
        timestamp: chrono::Utc::now(),
        content_hash: Some("def456".to_string()),
        file_size: Some(2048),
    };

    sync_manager.process_file_change(&event).await?;

    // Example: Process a file deletion event
    let event = FileChangeEventWithMetadata {
        event: FileChangeEvent::Deleted(file_path.clone()),
        timestamp: chrono::Utc::now(),
        content_hash: None,
        file_size: None,
    };

    sync_manager.process_file_change(&event).await?;

    // Example: Process a file rename event
    let old_path = PathBuf::from("/path/to/docs/architecture/old_design.md");
    let new_path = PathBuf::from("/path/to/docs/architecture/new_design.md");
    let event = FileChangeEventWithMetadata {
        event: FileChangeEvent::Renamed(old_path, new_path),
        timestamp: chrono::Utc::now(),
        content_hash: Some("ghi789".to_string()),
        file_size: Some(1536),
    };

    sync_manager.process_file_change(&event).await?;

    // Get statistics
    let stats = sync_manager.get_statistics().await;
    println!("Processed {} events, {} successful, {} failed", 
             stats.total_events_processed,
             stats.successful_operations,
             stats.failed_operations);

    Ok(())
}

/// Example of how to integrate with existing file watcher
pub async fn integrate_with_existing_watcher() -> Result<()> {
    use super::watcher::Watcher;
    use super::config::FileWatcherConfig;

    // 1. Set up the integrated sync manager
    let sync_manager = setup_integrated_file_watcher().await?;

    // 2. Create existing file watcher
    let watcher_config = FileWatcherConfig {
        watch_paths: Some(vec![PathBuf::from("/path/to/docs")]),
        debounce_delay_ms: 250,
        ..Default::default()
    };

    let mut watcher = Watcher::new(
        watcher_config,
        Arc::new(super::debouncer::Debouncer::new(250)),
        sync_manager.hash_validator(),
    )?;

    // 3. Note: The watcher processes events internally
    // For external event processing, you would need to implement
    // a different approach or extend the watcher

    // 4. Start watching
    watcher.start().await?;

    // The watcher will now automatically process file changes
    // using the integrated sync manager and existing systems

    Ok(())
}

/// Example of how to use with existing metrics system
pub async fn integrate_with_metrics() -> Result<()> {
    use super::metrics::MetricsCollector;

    // 1. Set up the integrated sync manager
    let sync_manager = setup_integrated_file_watcher().await?;

    // 2. Create existing metrics collector
    let metrics_collector = MetricsCollector::new();

    // 3. Process events and collect metrics
    let file_path = PathBuf::from("/path/to/docs/architecture/design.md");
    let event = FileChangeEventWithMetadata {
        event: FileChangeEvent::Created(file_path),
        timestamp: chrono::Utc::now(),
        content_hash: Some("abc123".to_string()),
        file_size: Some(1024),
    };

    // Process the event
    sync_manager.process_file_change(&event).await?;

    // Get metrics from both systems
    let sync_stats = sync_manager.get_statistics().await;
    let metrics = metrics_collector.get_metrics().await;

    println!("Sync stats: {:?}", sync_stats);
    println!("Metrics: {:?}", metrics);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_integrated_file_watcher() {
        // This test would require proper mocking of dependencies
        // For now, we just test that the function compiles
        let result = setup_integrated_file_watcher().await;
        // In a real test, we would assert the result
        let _ = result;
    }

    #[tokio::test]
    async fn test_process_file_changes_example() {
        // This test would require proper mocking of dependencies
        // For now, we just test that the function compiles
        let result = process_file_changes_example().await;
        // In a real test, we would assert the result
        let _ = result;
    }
}
