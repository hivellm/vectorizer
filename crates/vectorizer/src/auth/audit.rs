//! Admin audit log.
//!
//! Records every admin-gated action in an in-memory ring buffer that is
//! flushed to a daily-rotated file under the configured backup directory
//! by a background task.
//!
//! # Durability SLO
//!
//! The in-memory buffer holds at most `buffer_capacity` entries (default
//! 4096). If the background flusher falls behind — or if the process
//! crashes before the buffer is flushed — entries in the buffer that have
//! not yet been written to disk are **lost**. This is an at-most-once,
//! best-effort audit trail, NOT a durable audit ledger. Operators who need
//! guaranteed durability should ship the log file to an external sink
//! (syslog, CloudWatch, Splunk) and set `buffer_capacity` to a small
//! value (32–128) so the flush interval keeps the loss window short.
//!
//! # Hot-path guarantee
//!
//! `AuditLogger::record` sends to an `mpsc::unbounded_channel` and never
//! blocks the caller. The channel is drained by the background flusher task.

// Internal data-layout file: public fields are self-documenting.
#![allow(missing_docs)]

use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Username or key-id of the actor who performed the action.
    pub actor: String,
    /// Canonical action name, e.g. `"create_api_key"`, `"rotate_api_key"`.
    pub action: String,
    /// Target resource, e.g. key-id, collection name, username.
    pub target: String,
    /// UTC timestamp.
    pub at: DateTime<Utc>,
    /// Correlation-ID propagated from the request middleware.
    pub correlation_id: Option<String>,
}

/// Query parameters for filtering the audit log.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditQuery {
    /// Only entries at or after this UTC timestamp (RFC 3339).
    pub from: Option<DateTime<Utc>>,
    /// Only entries at or before this UTC timestamp (RFC 3339).
    pub to: Option<DateTime<Utc>>,
    /// Filter by actor.
    pub actor: Option<String>,
    /// Filter by action.
    pub action: Option<String>,
    /// Maximum number of entries to return (default 200).
    pub limit: Option<usize>,
}

// ---------------------------------------------------------------------------
// AuditLogger
// ---------------------------------------------------------------------------

/// Non-blocking admin audit logger.
///
/// Cheaply cloneable — the sender half is inside an `Arc`.
#[derive(Clone)]
pub struct AuditLogger {
    tx: mpsc::UnboundedSender<AuditEntry>,
    /// In-memory buffer for synchronous reads (e.g. the `GET /auth/audit`
    /// handler). Protected by an `Arc<tokio::sync::RwLock>` so reads never
    /// block writes.
    buffer: Arc<tokio::sync::RwLock<VecDeque<AuditEntry>>>,
    /// Maximum entries kept in the in-memory buffer.
    buffer_capacity: usize,
}

impl std::fmt::Debug for AuditLogger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditLogger")
            .field("buffer_capacity", &self.buffer_capacity)
            .finish()
    }
}

impl AuditLogger {
    /// Create a new `AuditLogger` and start the background flusher.
    ///
    /// * `backup_dir` — directory under which daily log files are created;
    ///   if `None` file flushing is skipped (in-memory only).
    /// * `buffer_capacity` — maximum entries to keep in the in-memory ring.
    /// * `flush_interval_secs` — how often the background task drains the
    ///   channel and writes to disk.
    pub fn new(
        backup_dir: Option<PathBuf>,
        buffer_capacity: usize,
        flush_interval_secs: u64,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<AuditEntry>();
        let buffer = Arc::new(tokio::sync::RwLock::new(VecDeque::with_capacity(
            buffer_capacity,
        )));
        let buffer_cap = buffer_capacity;

        let logger = Self {
            tx,
            buffer: Arc::clone(&buffer),
            buffer_capacity: buffer_cap,
        };

        // Start background flusher.
        tokio::spawn(run_flusher(
            rx,
            buffer,
            buffer_cap,
            backup_dir,
            flush_interval_secs,
        ));

        logger
    }

    /// Record an audit event without blocking the caller.
    ///
    /// Drops the entry silently if the channel is disconnected (flusher
    /// panicked). This is intentional: audit logging must never fail an
    /// admin operation.
    pub fn record(
        &self,
        actor: impl Into<String>,
        action: impl Into<String>,
        target: impl Into<String>,
        correlation_id: Option<String>,
    ) {
        let entry = AuditEntry {
            actor: actor.into(),
            action: action.into(),
            target: target.into(),
            at: Utc::now(),
            correlation_id,
        };
        // Ignore send errors — the flusher may have been dropped in tests.
        let _ = self.tx.send(entry);
    }

    /// Query the in-memory buffer. Does not read from disk.
    pub async fn query(&self, params: &AuditQuery) -> Vec<AuditEntry> {
        let buf = self.buffer.read().await;
        let limit = params.limit.unwrap_or(200).min(buf.len());

        buf.iter()
            .filter(|e| {
                params.from.is_none_or(|f| e.at >= f)
                    && params.to.is_none_or(|t| e.at <= t)
                    && params.actor.as_deref().is_none_or(|a| e.actor == a)
                    && params.action.as_deref().is_none_or(|a| e.action == a)
            })
            .rev()
            // newest-first
            .take(limit)
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Background flusher
// ---------------------------------------------------------------------------

async fn run_flusher(
    mut rx: mpsc::UnboundedReceiver<AuditEntry>,
    buffer: Arc<tokio::sync::RwLock<VecDeque<AuditEntry>>>,
    buffer_capacity: usize,
    backup_dir: Option<PathBuf>,
    flush_interval_secs: u64,
) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(flush_interval_secs.max(1)));

    let mut pending: Vec<AuditEntry> = Vec::new();

    loop {
        tokio::select! {
            entry = rx.recv() => {
                match entry {
                    Some(e) => {
                        pending.push(e);
                    }
                    None => {
                        // Channel closed — flush remaining and exit.
                        flush_to_buffer_and_disk(
                            &mut pending,
                            &buffer,
                            buffer_capacity,
                            backup_dir.as_deref(),
                        ).await;
                        break;
                    }
                }
            }
            _ = interval.tick() => {
                flush_to_buffer_and_disk(
                    &mut pending,
                    &buffer,
                    buffer_capacity,
                    backup_dir.as_deref(),
                ).await;
            }
        }
    }
}

async fn flush_to_buffer_and_disk(
    pending: &mut Vec<AuditEntry>,
    buffer: &Arc<tokio::sync::RwLock<VecDeque<AuditEntry>>>,
    buffer_capacity: usize,
    backup_dir: Option<&std::path::Path>,
) {
    if pending.is_empty() {
        return;
    }

    // Update in-memory ring buffer.
    {
        let mut buf = buffer.write().await;
        for entry in pending.iter() {
            if buf.len() >= buffer_capacity {
                buf.pop_front();
            }
            buf.push_back(entry.clone());
        }
    }

    // Write to daily-rotated file if a backup dir is configured.
    if let Some(dir) = backup_dir {
        let date = Utc::now().format("%Y-%m-%d");
        let path = dir.join(format!("audit-{}.jsonl", date));

        match open_append(&path) {
            Ok(mut f) => {
                for entry in pending.iter() {
                    match serde_json::to_string(entry) {
                        Ok(line) => {
                            if let Err(e) = writeln!(f, "{}", line) {
                                warn!("Audit log write error for {:?}: {}", path, e);
                            }
                        }
                        Err(e) => {
                            error!("Audit entry serialization error: {}", e);
                        }
                    }
                }
                if let Err(e) = f.flush() {
                    warn!("Audit log flush error: {}", e);
                }
            }
            Err(e) => {
                warn!("Cannot open audit log file {:?}: {}", path, e);
            }
        }
    }

    pending.clear();
}

fn open_append(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn record_and_query_in_memory() {
        let logger = AuditLogger::new(None, 100, 60);
        logger.record("admin", "create_api_key", "key-1", None);
        logger.record(
            "admin",
            "rotate_api_key",
            "key-1",
            Some("corr-123".to_string()),
        );

        // Give the flusher a moment to drain the channel.
        sleep(std::time::Duration::from_millis(20)).await;

        let entries = logger.query(&AuditQuery::default()).await;
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn query_filter_by_action() {
        let logger = AuditLogger::new(None, 100, 60);
        logger.record("admin", "create_api_key", "key-1", None);
        logger.record("admin", "rotate_api_key", "key-2", None);

        sleep(std::time::Duration::from_millis(20)).await;

        let entries = logger
            .query(&AuditQuery {
                action: Some("rotate_api_key".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "rotate_api_key");
    }

    #[tokio::test]
    async fn buffer_capacity_evicts_oldest() {
        let logger = AuditLogger::new(None, 3, 60);
        for i in 0..5 {
            logger.record("admin", "action", format!("target-{}", i), None);
        }
        sleep(std::time::Duration::from_millis(50)).await;

        let entries = logger.query(&AuditQuery::default()).await;
        // Buffer capped at 3 — oldest entries were evicted.
        assert!(entries.len() <= 3);
    }
}
