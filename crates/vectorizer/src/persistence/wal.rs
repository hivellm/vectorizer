// Internal data-layout file: public fields are self-documenting; the
// blanket allow keeps `cargo doc -W missing-docs` clean without padding
// every field with a tautological `///` comment. See
// phase4_enforce-public-api-docs.
#![allow(missing_docs)]

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use serde_json;
use thiserror::Error;
use tokio::fs;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, warn};

use crate::persistence::types::{Operation, Transaction, WALEntry};

/// WAL (Write-Ahead Log) implementation for atomic operations
pub struct WriteAheadLog {
    /// Path to WAL file
    file_path: PathBuf,
    /// Async file handle
    file: Arc<AsyncMutex<File>>,
    /// Current sequence number
    sequence: AtomicU64,
    /// Checkpoint threshold (number of operations)
    checkpoint_threshold: usize,
    /// Maximum WAL size in MB
    max_wal_size_mb: usize,
    /// Background checkpoint interval
    checkpoint_interval: Duration,
    /// fsync after append/checkpoint flush (see [`WALConfig::fsync`])
    fsync: bool,
}

/// WAL errors
#[derive(Debug, Error)]
pub enum WALError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("WAL corruption detected at sequence {0}")]
    Corruption(u64),

    #[error("Checkpoint failed: {0}")]
    CheckpointFailed(String),

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("WAL file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid sequence number: expected {expected}, got {actual}")]
    InvalidSequence { expected: u64, actual: u64 },
}

/// WAL configuration
#[derive(Debug, Clone)]
pub struct WALConfig {
    /// Checkpoint threshold (number of operations)
    pub checkpoint_threshold: usize,
    /// Maximum WAL size in MB
    pub max_wal_size_mb: usize,
    /// Checkpoint interval
    pub checkpoint_interval: Duration,
    /// Enable compression
    pub compression: bool,
    /// fsync (`sync_data`) after every append/checkpoint flush. On by
    /// default: without it acknowledged writes sit in the OS page
    /// cache and are lost on power failure (spec: phase37 durability).
    pub fsync: bool,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            checkpoint_threshold: 1000,
            max_wal_size_mb: 100,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
            compression: false,
            fsync: true,
        }
    }
}

/// Framed-record prefix: `C1 <crc32-hex8> <payload-len> <json>`.
///
/// The framing stays line-oriented so `BufRead::lines()` keeps working
/// and pre-framing WAL files (bare JSON lines) remain readable. The
/// CRC covers the JSON payload bytes; the length detects a torn final
/// line whose truncated payload might still be valid-looking JSON.
const FRAME_TAG: &str = "C1 ";

/// Outcome of parsing one WAL line.
enum ParsedLine {
    Entry(WALEntry),
    /// Line is damaged. Recoverable only when it is the final line of
    /// the file (torn append); anywhere else it is real corruption.
    Damaged(String),
}

fn encode_frame(json: &str) -> String {
    let crc = crc32fast::hash(json.as_bytes());
    format!("{FRAME_TAG}{crc:08x} {} {json}", json.len())
}

fn parse_line(line: &str) -> ParsedLine {
    if let Some(rest) = line.strip_prefix(FRAME_TAG) {
        // "<crc-hex8> <len> <json>"
        let Some((crc_hex, rest)) = rest.split_once(' ') else {
            return ParsedLine::Damaged("framed line missing crc field".into());
        };
        let Some((len_str, json)) = rest.split_once(' ') else {
            return ParsedLine::Damaged("framed line missing length field".into());
        };
        let Ok(expected_crc) = u32::from_str_radix(crc_hex, 16) else {
            return ParsedLine::Damaged(format!("bad crc field {crc_hex:?}"));
        };
        let Ok(expected_len) = len_str.parse::<usize>() else {
            return ParsedLine::Damaged(format!("bad length field {len_str:?}"));
        };
        if json.len() != expected_len {
            return ParsedLine::Damaged(format!(
                "length mismatch: header says {expected_len}, payload has {}",
                json.len()
            ));
        }
        if crc32fast::hash(json.as_bytes()) != expected_crc {
            return ParsedLine::Damaged("crc mismatch".into());
        }
        match serde_json::from_str(json) {
            Ok(entry) => ParsedLine::Entry(entry),
            Err(e) => ParsedLine::Damaged(format!("crc ok but json invalid: {e}")),
        }
    } else {
        // Legacy bare-JSON line (pre-framing WAL).
        match serde_json::from_str(line) {
            Ok(entry) => ParsedLine::Entry(entry),
            Err(e) => ParsedLine::Damaged(format!("legacy line invalid: {e}")),
        }
    }
}

/// Read every entry in the WAL file, tolerating a torn final record.
///
/// A damaged FINAL line is a torn append (crash mid-write): it is
/// discarded with a warning and everything before it is returned. A
/// damaged line anywhere else is real corruption and aborts with
/// [`WALError::Corruption`].
fn read_entries(file: &mut File) -> Result<Vec<WALEntry>, WALError> {
    file.seek(SeekFrom::Start(0)).map_err(WALError::IoError)?;
    let reader = BufReader::new(&*file);

    let mut entries = Vec::new();
    let mut pending_damage: Option<(u64, String)> = None;

    for line in reader.lines() {
        let line = line.map_err(WALError::IoError)?;
        if line.trim().is_empty() {
            continue;
        }

        if let Some((seq, reason)) = pending_damage.take() {
            // The damaged line was NOT the final line — corruption.
            error!("WAL corruption mid-file after sequence {seq}: {reason}");
            return Err(WALError::Corruption(seq));
        }

        match parse_line(&line) {
            ParsedLine::Entry(entry) => entries.push(entry),
            ParsedLine::Damaged(reason) => {
                let last_seq = entries.last().map(|e: &WALEntry| e.sequence).unwrap_or(0);
                pending_damage = Some((last_seq, reason));
            }
        }
    }

    if let Some((seq, reason)) = pending_damage {
        warn!(
            "WAL: discarding torn final record after sequence {seq} ({reason}); \
             recovery continues with {} complete entries",
            entries.len()
        );
    }

    Ok(entries)
}

impl WriteAheadLog {
    /// Create new WAL instance
    pub async fn new<P: AsRef<Path>>(file_path: P, config: WALConfig) -> Result<Self, WALError> {
        let file_path = file_path.as_ref().to_path_buf();

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(WALError::IoError)?;
        }

        // Open or create WAL file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(true)
            .open(&file_path)
            .map_err(WALError::IoError)?;

        let mut wal = Self {
            file_path,
            file: Arc::new(AsyncMutex::new(file)),
            sequence: AtomicU64::new(0),
            checkpoint_threshold: config.checkpoint_threshold,
            max_wal_size_mb: config.max_wal_size_mb,
            checkpoint_interval: config.checkpoint_interval,
            fsync: config.fsync,
        };

        // Initialize sequence number from existing WAL
        wal.initialize_sequence().await?;

        info!(
            "WAL initialized at {} with sequence {}",
            wal.file_path.display(),
            wal.sequence.load(Ordering::Relaxed)
        );

        Ok(wal)
    }

    /// Initialize sequence number from existing WAL file
    async fn initialize_sequence(&self) -> Result<(), WALError> {
        let mut file = self.file.lock().await;
        let entries = read_entries(&mut file)?;

        // Store one PAST the max: `fetch_add` in `append` RETURNS the
        // pre-increment value, so storing the max itself would hand the
        // last entry's sequence out again after every restart.
        let next_sequence = entries
            .iter()
            .map(|e| e.sequence)
            .max()
            .map_or(0, |max| max + 1);

        self.sequence.store(next_sequence, Ordering::Relaxed);
        debug!("WAL sequence initialized to {}", next_sequence);

        Ok(())
    }

    /// Append entry to WAL
    pub async fn append(&self, collection_id: &str, operation: Operation) -> Result<u64, WALError> {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);

        let entry = WALEntry {
            sequence,
            timestamp: chrono::Utc::now(),
            operation,
            collection_id: collection_id.to_string(),
            transaction_id: None,
        };

        self.write_entry(&entry).await?;

        debug!(
            "WAL entry {} appended for collection {}",
            sequence, collection_id
        );
        Ok(sequence)
    }

    /// Append transaction to WAL
    pub async fn append_transaction(&self, transaction: &Transaction) -> Result<u64, WALError> {
        // Reserve the whole sequence range atomically UP FRONT. Reading
        // the counter and bumping it after the write (the previous
        // scheme) let two concurrent transactions compute the same base
        // and write overlapping sequence numbers.
        let n = transaction.operations.len() as u64;
        let base_sequence = self.sequence.fetch_add(n, Ordering::SeqCst);

        let mut entries = Vec::new();
        for (i, operation) in transaction.operations.iter().enumerate() {
            let sequence = base_sequence + i as u64;
            let entry = WALEntry {
                sequence,
                timestamp: transaction.started_at,
                operation: operation.clone(),
                collection_id: transaction.collection_id.clone(),
                transaction_id: Some(transaction.id),
            };
            entries.push(entry);
        }

        // Write all entries atomically
        let mut file = self.file.lock().await;
        for entry in entries {
            self.write_entry_to_file(&mut *file, &entry)?;
        }
        self.flush_file(&mut file)?;

        info!(
            "Transaction {} appended with {} operations",
            transaction.id,
            transaction.operations.len()
        );

        Ok(base_sequence)
    }

    /// Write single entry to WAL
    async fn write_entry(&self, entry: &WALEntry) -> Result<(), WALError> {
        let mut file = self.file.lock().await;
        self.write_entry_to_file(&mut *file, entry)?;
        self.flush_file(&mut file)?;
        Ok(())
    }

    /// Flush and (when configured) fsync the WAL file. `sync_data` is
    /// enough: sequence/length live in file content, and we don't rely
    /// on the inode metadata a full `sync_all` would additionally sync.
    fn flush_file(&self, file: &mut File) -> Result<(), WALError> {
        file.flush().map_err(WALError::IoError)?;
        if self.fsync {
            file.sync_data().map_err(WALError::IoError)?;
        }
        Ok(())
    }

    /// Write entry to file handle (CRC32 + length framed, see [`FRAME_TAG`])
    fn write_entry_to_file(&self, file: &mut File, entry: &WALEntry) -> Result<(), WALError> {
        let json = serde_json::to_string(entry).map_err(WALError::SerializationError)?;
        writeln!(file, "{}", encode_frame(&json)).map_err(WALError::IoError)?;
        Ok(())
    }

    /// Read entries from WAL starting from given sequence
    pub async fn read_from(&self, sequence: u64) -> Result<Vec<WALEntry>, WALError> {
        let mut file = self.file.lock().await;
        let mut entries = read_entries(&mut file)?;
        entries.retain(|e| e.sequence >= sequence);

        debug!(
            "Read {} WAL entries from sequence {}",
            entries.len(),
            sequence
        );
        Ok(entries)
    }

    /// Read all entries for a specific collection
    pub async fn read_collection_entries(
        &self,
        collection_id: &str,
    ) -> Result<Vec<WALEntry>, WALError> {
        let mut file = self.file.lock().await;
        let mut entries = read_entries(&mut file)?;
        entries.retain(|e| e.collection_id == collection_id);

        debug!(
            "Read {} WAL entries for collection {}",
            entries.len(),
            collection_id
        );
        Ok(entries)
    }

    /// Create checkpoint (truncate WAL)
    pub async fn checkpoint(&self) -> Result<u64, WALError> {
        let current_sequence = self.sequence.load(Ordering::Relaxed);

        // Truncate WAL file (no checkpoint marker needed)
        self.truncate().await?;

        info!("WAL checkpoint created at sequence {}", current_sequence);
        Ok(current_sequence)
    }

    /// Truncate WAL file
    async fn truncate(&self) -> Result<(), WALError> {
        // Create new WAL file
        let temp_path = self.file_path.with_extension("wal.tmp");
        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(WALError::IoError)?;

        // Replace old file with new one
        drop(new_file);
        fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(WALError::IoError)?;

        // Reopen file for appending
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .write(true)
            .open(&self.file_path)
            .map_err(WALError::IoError)?;

        // Make the truncation itself durable so a crash right after
        // checkpoint can't resurrect already-checkpointed entries.
        if self.fsync {
            file.sync_data().map_err(WALError::IoError)?;
        }

        *self.file.lock().await = file;

        debug!("WAL truncated successfully");
        Ok(())
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u64 {
        self.sequence.load(Ordering::Relaxed)
    }

    /// Get WAL file size in bytes
    pub async fn file_size(&self) -> Result<u64, WALError> {
        match fs::metadata(&self.file_path).await {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0), // File doesn't exist yet
            Err(e) => Err(WALError::IoError(e)),
        }
    }

    /// Check if checkpoint is needed
    pub async fn should_checkpoint(&self) -> Result<bool, WALError> {
        let file_size = self.file_size().await?;
        let file_size_mb = file_size / (1024 * 1024);

        Ok(file_size_mb >= self.max_wal_size_mb as u64)
    }

    /// Validate WAL integrity
    pub async fn validate_integrity(&self) -> Result<(), WALError> {
        let entries = {
            let mut file = self.file.lock().await;
            read_entries(&mut file)?
        };

        let mut expected_sequence = 0u64;
        for entry in &entries {
            if entry.sequence != expected_sequence {
                return Err(WALError::InvalidSequence {
                    expected: expected_sequence,
                    actual: entry.sequence,
                });
            }
            expected_sequence += 1;
        }

        info!("WAL integrity validation passed");
        Ok(())
    }

    /// Recover from WAL (replay operations)
    ///
    /// Sequences in the WAL are assigned from a single global [`AtomicU64`]
    /// shared across every collection, so per-collection sequences are a
    /// monotonically increasing (but not necessarily dense) subset of the
    /// global sequence space. The validation below checks strict
    /// monotonicity rather than dense `0..N` indexing to keep corruption
    /// detection while supporting multi-collection WALs.
    pub async fn recover(&self, collection_id: &str) -> Result<Vec<WALEntry>, WALError> {
        let entries = self.read_collection_entries(collection_id).await?;

        info!(
            "Recovering {} WAL entries for collection {}",
            entries.len(),
            collection_id
        );

        let mut previous: Option<u64> = None;
        for entry in &entries {
            if let Some(prev) = previous {
                if entry.sequence <= prev {
                    return Err(WALError::InvalidSequence {
                        expected: prev + 1,
                        actual: entry.sequence,
                    });
                }
            }
            previous = Some(entry.sequence);
        }

        Ok(entries)
    }

    /// Get WAL statistics
    pub async fn get_stats(&self) -> Result<WALStats, WALError> {
        let file_size = self.file_size().await?;
        let current_sequence = self.current_sequence();

        // Count entries
        let entry_count = {
            let mut file = self.file.lock().await;
            read_entries(&mut file)?.len()
        };

        Ok(WALStats {
            file_size_bytes: file_size,
            entry_count,
            current_sequence,
            checkpoint_threshold: self.checkpoint_threshold,
            max_size_mb: self.max_wal_size_mb,
        })
    }
}

/// WAL statistics
#[derive(Debug, Clone)]
pub struct WALStats {
    pub file_size_bytes: u64,
    pub entry_count: usize,
    pub current_sequence: u64,
    pub checkpoint_threshold: usize,
    pub max_size_mb: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_wal_creation_and_append() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();

        let operation = Operation::InsertVector {
            collection_name: "test_collection".to_string(),
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let sequence = wal.append("collection1", operation).await.unwrap();
        assert_eq!(sequence, 0);

        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.current_sequence, 1);
    }

    #[tokio::test]
    async fn test_wal_transaction() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();

        let mut transaction = Transaction::new(1, "collection1".to_string());
        transaction.add_operation(Operation::InsertVector {
            collection_name: "test_collection".to_string(),
            vector_id: "vec1".to_string(),
            data: vec![1.0, 2.0],
            metadata: HashMap::new(),
        });
        transaction.add_operation(Operation::InsertVector {
            collection_name: "test_collection".to_string(),
            vector_id: "vec2".to_string(),
            data: vec![3.0, 4.0],
            metadata: HashMap::new(),
        });

        let sequence = wal.append_transaction(&transaction).await.unwrap();
        assert_eq!(sequence, 0);

        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 2);
        assert_eq!(stats.current_sequence, 2);
    }

    #[tokio::test]
    async fn test_wal_read_entries() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();

        // Add some entries
        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec1".to_string(),
                data: vec![1.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec2".to_string(),
                data: vec![2.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        // Read entries
        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 0);
        assert_eq!(entries[1].sequence, 1);

        // Read from specific sequence
        let entries = wal.read_from(1).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].sequence, 1);
    }

    #[tokio::test]
    async fn test_wal_checkpoint() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();

        // Add some entries
        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec1".to_string(),
                data: vec![1.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec2".to_string(),
                data: vec![2.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        // Create checkpoint
        let checkpoint_sequence = wal.checkpoint().await.unwrap();
        assert_eq!(checkpoint_sequence, 2);

        // Verify WAL is truncated (empty after checkpoint)
        let stats = wal.get_stats().await.unwrap();
        assert_eq!(stats.entry_count, 0); // Empty after checkpoint
    }

    #[tokio::test]
    async fn test_wal_integrity_validation() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");

        let config = WALConfig::default();
        let wal = WriteAheadLog::new(&wal_path, config).await.unwrap();

        // Add some entries
        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec1".to_string(),
                data: vec![1.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "vec2".to_string(),
                data: vec![2.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        // Validate integrity
        wal.validate_integrity().await.unwrap();
    }

    #[tokio::test]
    async fn test_wal_recover_multi_collection_sparse_sequences() {
        // Regression guard for phase4_fix-wal-multi-collection-replay:
        // sequences are global, so per-collection recover() must accept
        // a monotonically increasing (but sparse) subset, not a dense 0..N.
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("multi.wal");
        let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
            .await
            .unwrap();

        // Interleaved writes: collection1 gets seq 0 and 2, collection2 gets seq 1.
        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "v1".to_string(),
                data: vec![1.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();
        wal.append(
            "collection2",
            Operation::InsertVector {
                collection_name: "collection2".to_string(),
                vector_id: "v2".to_string(),
                data: vec![2.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();
        wal.append(
            "collection1",
            Operation::InsertVector {
                collection_name: "collection1".to_string(),
                vector_id: "v3".to_string(),
                data: vec![3.0],
                metadata: HashMap::new(),
            },
        )
        .await
        .unwrap();

        // collection2 entries are [seq=1] — must succeed even though
        // the sequence doesn't start at 0.
        let c2 = wal.recover("collection2").await.unwrap();
        assert_eq!(c2.len(), 1);
        assert_eq!(c2[0].sequence, 1);

        // collection1 entries are [seq=0, seq=2] — must succeed because
        // sequences are strictly monotonic even if sparse.
        let c1 = wal.recover("collection1").await.unwrap();
        assert_eq!(c1.len(), 2);
        assert_eq!(c1[0].sequence, 0);
        assert_eq!(c1[1].sequence, 2);
    }

    fn insert_op(id: &str) -> Operation {
        Operation::InsertVector {
            collection_name: "c".to_string(),
            vector_id: id.to_string(),
            data: vec![1.0],
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_wal_torn_final_record_is_discarded_not_fatal() {
        // Spec (phase37 durability): a truncated final record — the
        // signature of a crash mid-append — must not abort recovery of
        // the complete records before it.
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("torn.wal");
        {
            let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
                .await
                .unwrap();
            wal.append("c", insert_op("v1")).await.unwrap();
            wal.append("c", insert_op("v2")).await.unwrap();
        }

        // Simulate the torn write: chop the last 20 bytes off the file.
        let content = std::fs::read(&wal_path).unwrap();
        std::fs::write(&wal_path, &content[..content.len() - 20]).unwrap();

        let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
            .await
            .unwrap();
        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 1, "complete record must survive");
        assert_eq!(entries[0].sequence, 0);

        // The torn record's sequence must NOT be reused for new appends
        // ... but since it was discarded, the next append continues
        // after the last COMPLETE entry.
        let seq = wal.append("c", insert_op("v3")).await.unwrap();
        assert_eq!(seq, 1);
    }

    #[tokio::test]
    async fn test_wal_mid_file_corruption_is_fatal() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("corrupt.wal");
        {
            let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
                .await
                .unwrap();
            wal.append("c", insert_op("v1")).await.unwrap();
            wal.append("c", insert_op("v2")).await.unwrap();
            wal.append("c", insert_op("v3")).await.unwrap();
        }

        // Corrupt the MIDDLE line — not a torn append, real damage.
        let content = std::fs::read_to_string(&wal_path).unwrap();
        let mut lines: Vec<String> = content.lines().map(String::from).collect();
        assert_eq!(lines.len(), 3);
        lines[1] = lines[1].replace(char::is_alphanumeric, "x");
        std::fs::write(&wal_path, lines.join("\n") + "\n").unwrap();

        let result = WriteAheadLog::new(&wal_path, WALConfig::default()).await;
        assert!(
            result.is_err(),
            "mid-file corruption must fail recovery, not be silently dropped"
        );
    }

    #[tokio::test]
    async fn test_wal_legacy_json_lines_still_readable() {
        // Pre-framing WALs are bare JSON lines — the reader must accept
        // them so an upgrade doesn't orphan existing logs.
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("legacy.wal");

        let entry = WALEntry {
            sequence: 0,
            timestamp: chrono::Utc::now(),
            operation: insert_op("v1"),
            collection_id: "c".to_string(),
            transaction_id: None,
        };
        let legacy_line = serde_json::to_string(&entry).unwrap();
        std::fs::write(&wal_path, legacy_line + "\n").unwrap();

        let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
            .await
            .unwrap();
        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].sequence, 0);

        // New appends on the legacy file are framed and mix fine.
        let seq = wal.append("c", insert_op("v2")).await.unwrap();
        assert_eq!(seq, 1);
        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_wal_sequence_not_duplicated_after_reopen() {
        // Regression (phase37): initialize_sequence used to store the
        // max sequence itself, so the first append after every restart
        // reused the last entry's sequence number.
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("reopen.wal");
        {
            let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
                .await
                .unwrap();
            let s0 = wal.append("c", insert_op("v1")).await.unwrap();
            assert_eq!(s0, 0);
        }

        let wal = WriteAheadLog::new(&wal_path, WALConfig::default())
            .await
            .unwrap();
        let s1 = wal.append("c", insert_op("v2")).await.unwrap();
        assert_eq!(s1, 1, "sequence must continue past the recovered max");

        // Density restored: validate_integrity expects 0..N.
        wal.validate_integrity().await.unwrap();
    }

    #[tokio::test]
    async fn test_wal_concurrent_transactions_get_disjoint_sequences() {
        // Regression (phase37): append_transaction read the counter
        // before taking the file lock, so two concurrent transactions
        // could compute the same base sequence.
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("concurrent.wal");
        let wal = Arc::new(
            WriteAheadLog::new(&wal_path, WALConfig::default())
                .await
                .unwrap(),
        );

        let mut handles = Vec::new();
        for t in 0..8u64 {
            let wal = Arc::clone(&wal);
            handles.push(tokio::spawn(async move {
                let mut tx = Transaction::new(t, "c".to_string());
                for i in 0..5 {
                    tx.add_operation(insert_op(&format!("t{t}-v{i}")));
                }
                wal.append_transaction(&tx).await.unwrap()
            }));
        }

        let mut bases = Vec::new();
        for h in handles {
            bases.push(h.await.unwrap());
        }
        bases.sort_unstable();
        // 8 transactions × 5 ops: bases must be disjoint multiples of 5
        // covering 0..40 with no overlap.
        assert_eq!(bases, vec![0, 5, 10, 15, 20, 25, 30, 35]);

        let entries = wal.read_from(0).await.unwrap();
        assert_eq!(entries.len(), 40);
        let mut seqs: Vec<u64> = entries.iter().map(|e| e.sequence).collect();
        seqs.sort_unstable();
        seqs.dedup();
        assert_eq!(seqs.len(), 40, "no duplicate sequences");
    }
}
