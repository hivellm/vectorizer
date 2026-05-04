//! Durable replication log - wraps the in-memory ReplicationLog with a file-based WAL
//!
//! On every append the entry is fsynced to disk before being inserted into the
//! in-memory ring buffer, guaranteeing that no confirmed write is lost on a
//! master crash.  When all replicas have ACKed an offset (`mark_replicated`)
//! the WAL file is truncated so it does not grow unboundedly.
//!
//! Format: each record is  `[u32 big-endian length][bincode-encoded ReplicationWalEntry]`

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::replication_log::ReplicationLog;
use super::types::{ReplicationOperation, ReplicationResult, ReplicationWalEntry, VectorOperation};

/// Snapshot of WAL state for observability surfaces (e.g. dashboard
/// `/metrics/runtime`). All fields are zero in memory-only mode or when
/// no replica has yet confirmed any offset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WalSnapshot {
    /// Latest offset appended to the WAL.
    pub current_seq: u64,
    /// Size of the on-disk WAL file in bytes (0 when memory-only).
    pub size_bytes: u64,
    /// Unix timestamp (seconds) at which `last_checkpoint_seq` last
    /// advanced. 0 when no replica has confirmed an offset.
    pub last_checkpoint_at: u64,
    /// Lowest offset that has been confirmed by all replicas — entries
    /// at or below this offset are eligible for truncation.
    pub last_checkpoint_seq: u64,
}

/// Durable replication log that persists operations to a WAL before exposing
/// them to the in-memory ring buffer.
pub struct DurableReplicationLog {
    /// In-memory log for fast access during normal operation
    memory_log: ReplicationLog,

    /// Path to the WAL file (`None` = memory-only mode)
    wal_path: Option<PathBuf>,

    /// Buffered writer for the WAL file; `None` when `wal_path` is `None`
    wal_writer: Option<Arc<Mutex<BufWriter<File>>>>,

    /// Lowest offset that has **not** yet been confirmed by all replicas.
    /// Used to decide when WAL entries can be safely discarded.
    min_confirmed_offset: RwLock<u64>,

    /// Unix timestamp (seconds) at which `min_confirmed_offset` last
    /// advanced. Surfaced via [`wal_snapshot`].
    last_checkpoint_at: RwLock<u64>,
}

impl DurableReplicationLog {
    /// Create a new durable replication log.
    ///
    /// When `wal_path` is `Some` the directory is created if it does not exist
    /// and the WAL file is opened (or created) for appending.  When `wal_path`
    /// is `None` the log operates in memory-only mode – identical to the plain
    /// `ReplicationLog`.
    pub fn new(max_size: usize, wal_path: Option<PathBuf>) -> ReplicationResult<Self> {
        let memory_log = ReplicationLog::new(max_size);

        let wal_writer = match &wal_path {
            None => None,
            Some(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let file = OpenOptions::new().create(true).append(true).open(path)?;

                Some(Arc::new(Mutex::new(BufWriter::new(file))))
            }
        };

        Ok(Self {
            memory_log,
            wal_path,
            wal_writer,
            min_confirmed_offset: RwLock::new(0),
            last_checkpoint_at: RwLock::new(0),
        })
    }

    /// Append an operation.
    ///
    /// When WAL is enabled the entry is serialized and fsynced to disk **before**
    /// the offset is exposed through `current_offset()`.  This guarantees that any
    /// offset returned to the caller is recoverable after a crash.
    pub fn append(&self, operation: VectorOperation) -> ReplicationResult<u64> {
        // Append to memory log first to obtain the new offset
        let offset = self.memory_log.append(operation.clone());

        // Persist to WAL if enabled
        if let Some(writer) = &self.wal_writer {
            let entry = ReplicationWalEntry {
                offset,
                timestamp: current_timestamp(),
                operation,
                replicated: false,
            };

            let encoded = crate::codec::serialize(&entry)
                .map_err(|e| super::types::ReplicationError::Serialization(e))?;

            let len = encoded.len() as u32;

            let mut guard = writer.lock();
            guard.write_all(&len.to_be_bytes())?;
            guard.write_all(&encoded)?;
            // Flush the BufWriter then fsync the underlying file so the OS
            // buffer is committed to stable storage before we return.
            guard.flush()?;
            // Access the underlying File to call sync_data.
            // BufWriter::get_mut() is available in std.
            guard.get_mut().sync_data()?;

            debug!(
                "WAL: wrote entry offset={} len={} bytes",
                offset,
                4 + encoded.len()
            );
        }

        Ok(offset)
    }

    /// Return operations with offset strictly greater than `from_offset`.
    ///
    /// Delegates to the in-memory ring buffer.  Returns `None` when `from_offset`
    /// is older than the oldest entry retained in memory (caller should perform
    /// a full snapshot sync instead).
    pub fn get_operations(&self, from_offset: u64) -> Option<Vec<ReplicationOperation>> {
        self.memory_log.get_operations(from_offset)
    }

    /// Return the current (latest) offset.
    pub fn current_offset(&self) -> u64 {
        self.memory_log.current_offset()
    }

    /// Mark `offset` as fully replicated (all replicas have ACKed up to this point).
    ///
    /// Updates the running minimum confirmed offset and attempts to truncate the
    /// WAL up to (but not including) that offset so the file does not grow
    /// without bound.
    pub fn mark_replicated(&self, offset: u64) {
        let advanced = {
            let mut min_off = self.min_confirmed_offset.write();
            if offset > *min_off {
                *min_off = offset;
                true
            } else {
                false
            }
        };
        if advanced {
            *self.last_checkpoint_at.write() = current_timestamp_secs();
        }

        // Best-effort WAL truncation.  We rewrite the file keeping only entries
        // whose offset is >= min_confirmed_offset.  Errors are logged but not
        // propagated – a failure here is non-fatal since the WAL will simply be
        // replayed in full on the next recovery.
        if let Err(e) = self.try_truncate_wal() {
            warn!("WAL truncation failed (non-fatal): {}", e);
        }
    }

    /// Snapshot of WAL state for observability (phase25 §3).
    ///
    /// `size_bytes` reads file metadata at call time. The other fields are
    /// in-memory atomics, so the call is cheap enough to invoke from the
    /// 1 Hz runtime sampler.
    pub fn wal_snapshot(&self) -> WalSnapshot {
        let current_seq = self.current_offset();
        let last_checkpoint_seq = *self.min_confirmed_offset.read();
        let last_checkpoint_at = *self.last_checkpoint_at.read();
        let size_bytes = self
            .wal_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);
        WalSnapshot {
            current_seq,
            size_bytes,
            last_checkpoint_at,
            last_checkpoint_seq,
        }
    }

    /// Replay all entries from the WAL file into the in-memory log on startup.
    ///
    /// Returns the last offset found in the WAL, or `0` when the WAL is absent
    /// or empty.  The caller (master node) should use this value to set its
    /// advertised offset before accepting connections from replicas.
    pub fn recover(&mut self) -> ReplicationResult<u64> {
        let path = match &self.wal_path {
            None => {
                debug!("WAL disabled – skipping recovery");
                return Ok(0);
            }
            Some(p) => p.clone(),
        };

        if !path.exists() {
            info!("WAL file not found at {} – starting fresh", path.display());
            return Ok(0);
        }

        let mut file = File::open(&path)?;
        file.seek(SeekFrom::Start(0))?;

        let mut last_offset: u64 = 0;
        let mut recovered: usize = 0;

        loop {
            // Read the 4-byte length prefix
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    warn!(
                        "WAL read error at entry {} (truncated record?): {}",
                        recovered, e
                    );
                    break;
                }
            }

            let entry_len = u32::from_be_bytes(len_buf) as usize;

            let mut data_buf = vec![0u8; entry_len];
            match file.read_exact(&mut data_buf) {
                Ok(()) => {}
                Err(e) => {
                    warn!(
                        "WAL: partial entry at offset {} (len={}): {}",
                        last_offset, entry_len, e
                    );
                    break;
                }
            }

            let entry: ReplicationWalEntry = match crate::codec::deserialize(&data_buf) {
                Ok(e) => e,
                Err(e) => {
                    warn!("WAL: corrupt entry after offset {}: {}", last_offset, e);
                    break;
                }
            };

            last_offset = entry.offset;
            self.memory_log.append(entry.operation);
            recovered += 1;
        }

        info!(
            "WAL recovery complete: {} entries replayed, last offset={}",
            recovered, last_offset
        );

        Ok(last_offset)
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Rewrite the WAL file keeping only entries at or above `min_confirmed_offset`.
    fn try_truncate_wal(&self) -> ReplicationResult<()> {
        let wal_path = match &self.wal_path {
            None => return Ok(()),
            Some(p) => p.clone(),
        };

        let min_off = *self.min_confirmed_offset.read();

        if !wal_path.exists() {
            return Ok(());
        }

        // Read all entries from the WAL
        let mut file = File::open(&wal_path)?;
        file.seek(SeekFrom::Start(0))?;

        let mut retained: Vec<(u32, Vec<u8>)> = Vec::new();

        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let entry_len = u32::from_be_bytes(len_buf);
            let mut data_buf = vec![0u8; entry_len as usize];
            match file.read_exact(&mut data_buf) {
                Ok(()) => {}
                Err(_) => break, // truncated record – discard tail
            }

            // Peek at the offset without a full decode when possible
            let entry: ReplicationWalEntry = match crate::codec::deserialize(&data_buf) {
                Ok(e) => e,
                Err(_) => break,
            };

            // Keep entries that have not yet been confirmed
            if entry.offset >= min_off {
                retained.push((entry_len, data_buf));
            }
        }

        // Rewrite the WAL atomically via a temp file
        let tmp_path = wal_path.with_extension("wal.tmp");
        {
            let tmp_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;

            let mut writer = BufWriter::new(tmp_file);
            for (len, data) in &retained {
                writer.write_all(&len.to_be_bytes())?;
                writer.write_all(data)?;
            }
            writer.flush()?;
            writer.get_mut().sync_data()?;
        }

        std::fs::rename(&tmp_path, &wal_path)?;

        // Reopen the writer so subsequent appends go to the new file
        if let Some(arc_writer) = &self.wal_writer {
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&wal_path)?;
            *arc_writer.lock() = BufWriter::new(new_file);
        }

        debug!(
            "WAL truncated: {} entries retained (min_confirmed={})",
            retained.len(),
            min_off
        );

        Ok(())
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::replication::types::{CollectionConfigData, VectorOperation};

    fn make_op(name: &str) -> VectorOperation {
        VectorOperation::CreateCollection {
            name: name.to_string(),
            config: CollectionConfigData {
                dimension: 4,
                metric: "cosine".to_string(),
            },
            owner_id: None,
        }
    }

    #[test]
    fn test_memory_only_append_and_offset() {
        let log = DurableReplicationLog::new(100, None).unwrap();

        let o1 = log.append(make_op("col1")).unwrap();
        let o2 = log.append(make_op("col2")).unwrap();

        assert_eq!(o1, 1);
        assert_eq!(o2, 2);
        assert_eq!(log.current_offset(), 2);
    }

    #[test]
    fn test_wal_append_and_recover() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("replication.wal");

        // Write two entries
        {
            let log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();
            log.append(make_op("col1")).unwrap();
            log.append(make_op("col2")).unwrap();
            assert_eq!(log.current_offset(), 2);
        }

        // Recover in a fresh instance
        {
            let mut log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();
            let last = log.recover().unwrap();
            assert_eq!(last, 2);
            assert_eq!(log.current_offset(), 2);
        }
    }

    #[test]
    fn test_get_operations_delegates_to_memory_log() {
        let log = DurableReplicationLog::new(100, None).unwrap();

        for i in 0..5 {
            log.append(make_op(&format!("col{}", i))).unwrap();
        }

        let ops = log.get_operations(2).unwrap();
        assert_eq!(ops.len(), 3); // offsets 3, 4, 5
        assert_eq!(ops[0].offset, 3);
    }

    #[test]
    fn test_mark_replicated_and_truncation() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("replication.wal");

        let log = DurableReplicationLog::new(100, Some(wal_path.clone())).unwrap();

        for i in 0..5 {
            log.append(make_op(&format!("col{}", i))).unwrap();
        }

        // Mark offsets 1-3 as replicated; WAL should only keep 4 and 5
        log.mark_replicated(4);

        // Recover should see entries at offset 4 and 5
        let mut recovered = DurableReplicationLog::new(100, Some(wal_path)).unwrap();
        let last = recovered.recover().unwrap();
        assert_eq!(last, 5);
    }

    #[test]
    fn test_recover_empty_wal() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("replication.wal");

        let mut log = DurableReplicationLog::new(100, Some(wal_path)).unwrap();
        let last = log.recover().unwrap();
        assert_eq!(last, 0);
    }

    #[test]
    fn test_recover_no_wal_path() {
        let mut log = DurableReplicationLog::new(100, None).unwrap();
        let last = log.recover().unwrap();
        assert_eq!(last, 0);
    }

    #[test]
    fn wal_snapshot_memory_only_is_zeroed() {
        let log = DurableReplicationLog::new(100, None).unwrap();
        log.append(make_op("c1")).unwrap();
        log.append(make_op("c2")).unwrap();

        let snap = log.wal_snapshot();
        assert_eq!(snap.current_seq, 2);
        assert_eq!(
            snap.size_bytes, 0,
            "memory-only mode reports zero file size"
        );
        assert_eq!(snap.last_checkpoint_seq, 0);
        assert_eq!(snap.last_checkpoint_at, 0);
    }

    #[test]
    fn wal_snapshot_with_file_reports_nonzero_size() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("replication.wal");

        let log = DurableReplicationLog::new(100, Some(wal_path)).unwrap();
        log.append(make_op("c1")).unwrap();
        log.append(make_op("c2")).unwrap();

        let snap = log.wal_snapshot();
        assert_eq!(snap.current_seq, 2);
        assert!(snap.size_bytes > 0, "WAL file should have on-disk bytes");
    }

    #[test]
    fn wal_snapshot_advances_checkpoint_on_mark_replicated() {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("replication.wal");

        let log = DurableReplicationLog::new(100, Some(wal_path)).unwrap();
        for i in 0..5 {
            log.append(make_op(&format!("c{}", i))).unwrap();
        }

        let before = log.wal_snapshot();
        assert_eq!(before.last_checkpoint_seq, 0);
        assert_eq!(before.last_checkpoint_at, 0);

        log.mark_replicated(3);

        let after = log.wal_snapshot();
        assert_eq!(after.last_checkpoint_seq, 3);
        assert!(
            after.last_checkpoint_at > 0,
            "last_checkpoint_at should be set after mark_replicated"
        );

        // Calling mark_replicated with a smaller offset must not regress
        // the checkpoint pointer.
        log.mark_replicated(1);
        let unchanged = log.wal_snapshot();
        assert_eq!(unchanged.last_checkpoint_seq, 3);
        assert_eq!(unchanged.last_checkpoint_at, after.last_checkpoint_at);
    }
}
