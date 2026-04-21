//! Usage tracking and reporting module for HiveHub integration
//!
//! Provides usage metrics collection and periodic reporting to HiveHub
//! for billing and quota enforcement purposes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use super::client::{HubClient, UpdateUsageRequest};
use crate::error::{Result, VectorizerError};

/// Usage metrics for a single operation or aggregated period
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Number of vectors inserted
    pub vectors_inserted: u64,
    /// Number of vectors deleted
    pub vectors_deleted: u64,
    /// Storage added in bytes
    pub storage_added: u64,
    /// Storage freed in bytes
    pub storage_freed: u64,
    /// Number of search operations
    pub search_count: u64,
    /// Number of collections created
    pub collections_created: u64,
    /// Number of collections deleted
    pub collections_deleted: u64,
    /// Total API requests
    pub api_requests: u64,
}

impl UsageMetrics {
    /// Create a new empty metrics instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Record vector insertions
    pub fn record_insert(&mut self, count: u64, storage_bytes: u64) {
        self.vectors_inserted += count;
        self.storage_added += storage_bytes;
        self.api_requests += 1;
    }

    /// Record vector deletions
    pub fn record_delete(&mut self, count: u64, storage_bytes: u64) {
        self.vectors_deleted += count;
        self.storage_freed += storage_bytes;
        self.api_requests += 1;
    }

    /// Record a search operation
    pub fn record_search(&mut self) {
        self.search_count += 1;
        self.api_requests += 1;
    }

    /// Record collection creation
    pub fn record_collection_create(&mut self) {
        self.collections_created += 1;
        self.api_requests += 1;
    }

    /// Record collection deletion
    pub fn record_collection_delete(&mut self) {
        self.collections_deleted += 1;
        self.api_requests += 1;
    }

    /// Record a generic API request
    pub fn record_request(&mut self) {
        self.api_requests += 1;
    }

    /// Merge another metrics instance into this one
    pub fn merge(&mut self, other: &UsageMetrics) {
        self.vectors_inserted += other.vectors_inserted;
        self.vectors_deleted += other.vectors_deleted;
        self.storage_added += other.storage_added;
        self.storage_freed += other.storage_freed;
        self.search_count += other.search_count;
        self.collections_created += other.collections_created;
        self.collections_deleted += other.collections_deleted;
        self.api_requests += other.api_requests;
    }

    /// Calculate net vector change
    pub fn net_vectors(&self) -> i64 {
        self.vectors_inserted as i64 - self.vectors_deleted as i64
    }

    /// Calculate net storage change
    pub fn net_storage(&self) -> i64 {
        self.storage_added as i64 - self.storage_freed as i64
    }

    /// Calculate net collection change
    pub fn net_collections(&self) -> i64 {
        self.collections_created as i64 - self.collections_deleted as i64
    }

    /// Check if there are any changes to report
    pub fn has_changes(&self) -> bool {
        self.vectors_inserted > 0
            || self.vectors_deleted > 0
            || self.collections_created > 0
            || self.collections_deleted > 0
    }
}

/// Collection usage state for tracking
#[derive(Debug, Clone)]
struct CollectionUsageState {
    /// Collection ID (UUID)
    pub collection_id: Uuid,
    /// Current total vectors
    pub total_vectors: u64,
    /// Current total storage
    pub total_storage: u64,
    /// Pending metrics to be reported
    pub pending_metrics: UsageMetrics,
    /// Last reported timestamp
    pub last_reported: chrono::DateTime<chrono::Utc>,
}

impl CollectionUsageState {
    fn new(collection_id: Uuid) -> Self {
        Self {
            collection_id,
            total_vectors: 0,
            total_storage: 0,
            pending_metrics: UsageMetrics::new(),
            last_reported: chrono::Utc::now(),
        }
    }
}

/// Usage reporter for HiveHub
///
/// Collects usage metrics from operations and periodically
/// reports them to HiveHub for billing and quota tracking.
///
/// Note: In cluster mode, the Vectorizer runs locally and the
/// HiveHub communicates directly with it. Usage is tracked
/// per-collection and reported via collection_id (UUID).
#[derive(Debug)]
pub struct UsageReporter {
    /// HiveHub client
    client: Arc<HubClient>,
    /// Per-collection usage state (keyed by collection_id as string for HashMap)
    collection_usage: Arc<RwLock<HashMap<Uuid, CollectionUsageState>>>,
    /// Reporting interval
    report_interval: Duration,
    /// Background task handle
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Shutdown signal
    shutdown: Arc<Notify>,
    /// Running state
    running: Arc<RwLock<bool>>,
}

impl UsageReporter {
    /// Create a new UsageReporter
    pub fn new(client: Arc<HubClient>, report_interval_seconds: u64) -> Self {
        Self {
            client,
            collection_usage: Arc::new(RwLock::new(HashMap::new())),
            report_interval: Duration::from_secs(report_interval_seconds),
            task_handle: Arc::new(RwLock::new(None)),
            shutdown: Arc::new(Notify::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the background reporting task
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write();
        if *running {
            warn!("UsageReporter already running");
            return Ok(());
        }

        info!(
            "Starting usage reporter with interval {:?}",
            self.report_interval
        );

        let client = self.client.clone();
        let collection_usage = self.collection_usage.clone();
        let interval = self.report_interval;
        let shutdown = self.shutdown.clone();
        let running_flag = self.running.clone();

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        if let Err(e) = Self::report_all_collections(&client, &collection_usage).await {
                            error!("Failed to report usage: {}", e);
                        }
                    }
                    _ = shutdown.notified() => {
                        info!("UsageReporter shutdown signal received");
                        // Final report before shutdown
                        if let Err(e) = Self::report_all_collections(&client, &collection_usage).await {
                            error!("Failed to report final usage: {}", e);
                        }
                        break;
                    }
                }
            }

            *running_flag.write() = false;
            info!("UsageReporter stopped");
        });

        *self.task_handle.write() = Some(handle);
        *running = true;

        Ok(())
    }

    /// Stop the background reporting task
    pub async fn stop(&self) -> Result<()> {
        let running = *self.running.read();
        if !running {
            return Ok(());
        }

        info!("Stopping usage reporter");
        self.shutdown.notify_one();

        // Wait for task to complete
        if let Some(handle) = self.task_handle.write().take() {
            if let Err(e) = handle.await {
                error!("Error waiting for reporter task: {}", e);
            }
        }

        Ok(())
    }

    /// Record usage metrics for a collection
    pub async fn record(&self, collection_id: Uuid, metrics: UsageMetrics) -> Result<()> {
        trace!(
            "Recording usage for collection {}: {:?}",
            collection_id, metrics
        );

        let mut usage = self.collection_usage.write();
        let state = usage
            .entry(collection_id)
            .or_insert_with(|| CollectionUsageState::new(collection_id));

        // Update totals
        state.total_vectors = (state.total_vectors as i64 + metrics.net_vectors()).max(0) as u64;
        state.total_storage = (state.total_storage as i64 + metrics.net_storage()).max(0) as u64;

        // Accumulate pending metrics
        state.pending_metrics.merge(&metrics);

        Ok(())
    }

    /// Initialize collection usage state with current values
    pub fn initialize_collection(
        &self,
        collection_id: Uuid,
        total_vectors: u64,
        total_storage: u64,
    ) {
        let mut usage = self.collection_usage.write();
        let state = usage
            .entry(collection_id)
            .or_insert_with(|| CollectionUsageState::new(collection_id));

        state.total_vectors = total_vectors;
        state.total_storage = total_storage;
    }

    /// Get current usage for a collection
    pub fn get_collection_usage(&self, collection_id: &Uuid) -> Option<(u64, u64)> {
        let usage = self.collection_usage.read();
        usage
            .get(collection_id)
            .map(|state| (state.total_vectors, state.total_storage))
    }

    /// Force immediate report for all collections
    pub async fn flush(&self) -> Result<()> {
        info!("Flushing pending usage reports");
        Self::report_all_collections(&self.client, &self.collection_usage).await
    }

    /// Report usage for all collections
    async fn report_all_collections(
        client: &HubClient,
        collection_usage: &Arc<RwLock<HashMap<Uuid, CollectionUsageState>>>,
    ) -> Result<()> {
        let collections_to_report: Vec<CollectionUsageState> = {
            let usage = collection_usage.read();
            usage
                .values()
                .filter(|state| state.pending_metrics.has_changes())
                .cloned()
                .collect()
        };

        if collections_to_report.is_empty() {
            trace!("No usage changes to report");
            return Ok(());
        }

        debug!(
            "Reporting usage for {} collections",
            collections_to_report.len()
        );

        for state in collections_to_report {
            match Self::report_collection_usage(client, &state).await {
                Ok(()) => {
                    // Clear pending metrics on success
                    let mut usage = collection_usage.write();
                    if let Some(collection_state) = usage.get_mut(&state.collection_id) {
                        collection_state.pending_metrics = UsageMetrics::new();
                        collection_state.last_reported = chrono::Utc::now();
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to report usage for collection {}: {}",
                        state.collection_id, e
                    );
                    // Keep pending metrics for retry
                }
            }
        }

        Ok(())
    }

    /// Report usage for a single collection
    async fn report_collection_usage(
        client: &HubClient,
        state: &CollectionUsageState,
    ) -> Result<()> {
        trace!(
            "Reporting usage for collection {}: vectors={}, storage={}",
            state.collection_id, state.total_vectors, state.total_storage
        );

        let request = UpdateUsageRequest {
            vector_count: state.total_vectors,
            storage_bytes: state.total_storage,
        };

        client.update_usage(&state.collection_id, request).await?;

        debug!(
            "Successfully reported usage for collection {}",
            state.collection_id
        );
        Ok(())
    }

    /// Check if reporter is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Get reporting interval
    pub fn report_interval(&self) -> Duration {
        self.report_interval
    }

    /// Get number of tracked collections
    pub fn collection_count(&self) -> usize {
        self.collection_usage.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_metrics_default() {
        let metrics = UsageMetrics::new();
        assert_eq!(metrics.vectors_inserted, 0);
        assert_eq!(metrics.vectors_deleted, 0);
        assert_eq!(metrics.storage_added, 0);
        assert_eq!(metrics.storage_freed, 0);
        assert_eq!(metrics.search_count, 0);
        assert_eq!(metrics.api_requests, 0);
    }

    #[test]
    fn test_usage_metrics_record_insert() {
        let mut metrics = UsageMetrics::new();
        metrics.record_insert(100, 1024);

        assert_eq!(metrics.vectors_inserted, 100);
        assert_eq!(metrics.storage_added, 1024);
        assert_eq!(metrics.api_requests, 1);
    }

    #[test]
    fn test_usage_metrics_record_delete() {
        let mut metrics = UsageMetrics::new();
        metrics.record_delete(50, 512);

        assert_eq!(metrics.vectors_deleted, 50);
        assert_eq!(metrics.storage_freed, 512);
        assert_eq!(metrics.api_requests, 1);
    }

    #[test]
    fn test_usage_metrics_net_calculations() {
        let mut metrics = UsageMetrics::new();
        metrics.record_insert(100, 1024);
        metrics.record_delete(30, 300);

        assert_eq!(metrics.net_vectors(), 70);
        assert_eq!(metrics.net_storage(), 724);
    }

    #[test]
    fn test_usage_metrics_merge() {
        let mut metrics1 = UsageMetrics::new();
        metrics1.record_insert(100, 1024);

        let mut metrics2 = UsageMetrics::new();
        metrics2.record_insert(50, 512);
        metrics2.record_search();

        metrics1.merge(&metrics2);

        assert_eq!(metrics1.vectors_inserted, 150);
        assert_eq!(metrics1.storage_added, 1536);
        assert_eq!(metrics1.search_count, 1);
        assert_eq!(metrics1.api_requests, 3);
    }

    #[test]
    fn test_usage_metrics_has_changes() {
        let empty = UsageMetrics::new();
        assert!(!empty.has_changes());

        let mut with_inserts = UsageMetrics::new();
        with_inserts.record_insert(1, 100);
        assert!(with_inserts.has_changes());

        let mut with_search_only = UsageMetrics::new();
        with_search_only.record_search();
        assert!(!with_search_only.has_changes()); // Search doesn't count as "changes"
    }

    #[test]
    fn test_collection_usage_state_new() {
        let collection_id = Uuid::new_v4();
        let state = CollectionUsageState::new(collection_id);
        assert_eq!(state.collection_id, collection_id);
        assert_eq!(state.total_vectors, 0);
        assert_eq!(state.total_storage, 0);
    }
}
